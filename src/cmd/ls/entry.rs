use chrono::{ DateTime, Duration, Utc };
use chrono_tz::Africa::Casablanca;
use libc::{ major, minor, llistxattr };
use std::fs::{ self, Metadata };
use std::io::{ ErrorKind };
use std::os::unix::fs::{ FileTypeExt, MetadataExt, PermissionsExt };
use std::path::{ PathBuf };
use users::{ get_group_by_gid, get_user_by_uid };
use std::path::Path;

use super::{ ls_config::LsConfig, utils::{ apply_color, to_str, has_acl } };

#[derive(PartialEq)]
pub enum FileType {
    Regular,
    Directory,
    Symlink,
    Executable,
    CharDevice,
    BlockDevice,
    Socket,
    NamedPipe,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ColorStyle {
    BoldGreen,
    BlueBold,
    BoldYellow,
    CyanBold,
    BoldRed,
    BoldMagenta,
    BrightWhite,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub metadata: Option<Metadata>,
    pub ls_config: LsConfig,
    pub path: PathBuf,
    pub sym_path: Option<PathBuf>,
    pub sym_metadata: Option<Metadata>,
    pub target_entry: String,
}

impl Entry {
    pub fn new(path: &PathBuf, ls_config: &LsConfig, target_entry: &String) -> Option<Self> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(some_metadata) => some_metadata,
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    eprintln!(
                        "ls: cannot access '{}': No such file or directory",
                        path.to_string_lossy()
                    );
                    return None;
                } else if e.kind() == ErrorKind::PermissionDenied {
                    println!("ls: cannot access '{}': Permission denied", path.to_string_lossy());

                    if *ls_config.status_code.borrow() != 2 {
                        *ls_config.status_code.borrow_mut() = 1;
                    }
                    return Some(Self {
                        metadata: None,
                        ls_config: ls_config.clone(),
                        path: path.clone(),
                        sym_metadata: None,
                        sym_path: None,
                        target_entry: target_entry.to_string(),
                    });
                } else {
                    if *ls_config.status_code.borrow() != 2 {
                        *ls_config.status_code.borrow_mut() = 1;
                    }
                    eprintln!("Error while Creating the entry: {}", e);
                    return None;
                }
            }
        };
        let mut sym_path: Option<PathBuf> = None;
        let mut sym_metadata: Option<Metadata> = None;

        if metadata.file_type().is_symlink() {
            match fs::read_link(path) {
                Ok(target_path) => {
                    sym_path = Some(target_path.clone());
                    // we need to use metadata to follow the link to the inner target ;)
                    match fs::metadata(&path) {
                        Ok(some_metadata) => {
                            sym_metadata = Some(some_metadata.clone());
                        }
                        Err(_) => {}
                    };
                }
                Err(_) => {}
            };
        }
        Some(Self {
            metadata: Some(metadata.clone()),
            ls_config: ls_config.clone(),
            path: path.clone(),
            sym_path: sym_path,
            sym_metadata: sym_metadata,
            target_entry: target_entry.clone(),
        })
    }

    pub fn handle_entry(&mut self) -> Vec<String> {
        if self.metadata.is_some() {
            return self.as_array();
        }
        self.as_pseudo_array()
    }
    pub fn as_pseudo_array(&mut self) -> Vec<String> {
        let color_style = self.color_name_style();
        let mut file_name = apply_color(&self.get_entry_name(), color_style);
        if self.ls_config.f_flag_set {
            file_name.push(self.append_file_type_indicator());
        }

        if !self.ls_config.l_flag_set {
            return vec![file_name];
        }

        let (minor, major) = self.get_size();

        vec![
            self.get_permissions(),
            "?".to_string(),
            self.get_user_name(),
            self.get_group_name(),
            major.clone(),
            minor.clone(),
            "        ?".to_string(),
            file_name
        ]
    }

    pub fn as_array(&mut self) -> Vec<String> {
        let color_style = self.color_name_style();
        let mut file_name = apply_color(&self.get_entry_name(), color_style);
        if self.ls_config.l_flag_set {
            if self.sym_path.is_some() {
                file_name.push_str(" -> ");
                let target_color = match &self.sym_metadata {
                    Some(metadata) => Self::color_style_for_metadata(&metadata),
                    None => ColorStyle::BoldRed,
                };
                // unwrap here is safe :)
                let target_name = to_str(self.sym_path.clone().unwrap());
                file_name.push_str(&apply_color(&target_name, target_color));
            }
        }
        if self.ls_config.f_flag_set {
            file_name.push(self.append_file_type_indicator());
        }

        if !self.ls_config.l_flag_set {
            return vec![file_name];
        }
        // here also is safe !!!!
        let (major, minor) = self.get_size();
        vec![
            self.get_permissions(),
            self.metadata.clone().unwrap().nlink().to_string(),
            self.get_user_name(),
            self.get_group_name(),
            major.clone(),
            minor.clone(),
            self.get_date(),
            file_name
        ]
    }

    // using unwrap here is also safe !!
    fn get_size(&self) -> (String, String) {
        if self.metadata.is_none() {
            return ("".to_string(), "?".to_string());
        }
        let (file_type, _, _) = Self::get_entry_type(&self.metadata.clone().unwrap());
        if file_type == FileType::CharDevice || file_type == FileType::BlockDevice {
            let rdev = self.metadata.clone().unwrap().rdev();

            let mut major_val = major(rdev).to_string();
            major_val.push(',');
            let minor_val = minor(rdev).to_string();

            return (major_val, minor_val);
        }

        ("".to_string(), self.metadata.clone().unwrap().size().to_string())
    }

    // unwraping metadata is safe here as well !!
    fn get_entry_name(&self) -> String {
        if to_str(&self.path) == self.target_entry && self.path.is_absolute() {
            return to_str(&self.path);
        }
        // in this case we are obliged to convert to string as for Cargo.toml == Cargo.toml/. in PATH
        if to_str(&self.path) == to_str(&Path::new(&self.target_entry).join(".")) {
            return ".".to_string();
        }
        if self.path == Path::new(&self.target_entry).join("..") {
            return "..".to_string();
        }

        self.path
            .file_name()
            .map(|name| to_str(&name))
            .unwrap_or_else(|| to_str(&self.path))
    }

    fn color_name_style(&self) -> ColorStyle {
        let entry_type = if self.metadata.is_some() {
            Self::get_entry_type(&self.metadata.clone().unwrap()).0
        } else {
            self.get_pseudo_entry_type().0
        };

        let is_broken_symlink = entry_type == FileType::Symlink && self.sym_metadata.is_none();

        if is_broken_symlink {
            return ColorStyle::BoldRed;
        }

        match entry_type {
            FileType::Executable => ColorStyle::BoldGreen,
            FileType::Directory => ColorStyle::BlueBold,
            FileType::BlockDevice | FileType::CharDevice | FileType::NamedPipe =>
                ColorStyle::BoldYellow,
            FileType::Symlink => ColorStyle::CyanBold,
            FileType::Socket => ColorStyle::BoldMagenta,
            _ => ColorStyle::BrightWhite,
        }
    }

    pub fn color_style_for_metadata(metadata: &Metadata) -> ColorStyle {
        let (entry_type, _, _) = Self::get_entry_type(metadata);
        match entry_type {
            FileType::Executable => ColorStyle::BoldGreen,
            FileType::Directory => ColorStyle::BlueBold,
            FileType::BlockDevice | FileType::CharDevice | FileType::NamedPipe =>
                ColorStyle::BoldYellow,
            FileType::Symlink => ColorStyle::CyanBold,
            FileType::Socket => ColorStyle::BoldMagenta,
            _ => ColorStyle::BrightWhite,
        }
    }

    // does not work for now !!!
    pub fn get_pseudo_entry_type(&self) -> (FileType, char, char) {
        // we need the absolute path otherwise it worn't work :)
        if
            to_str(&self.path) == to_str(&Path::new(&self.target_entry).join(".")) ||
            to_str(&self.path) == to_str(&Path::new(&self.target_entry).join(".."))
        {
            return (FileType::Directory, 'd', '/');
        }

        let parent = match self.path.parent() {
            Some(parent) => parent,
            None => {
                std::process::exit(2);
            }
        };
        match fs::read_dir(parent) {
            Ok(entries) => {
                // Get the first entry or handle empty directory as needed

                for entry in entries {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(_) => {
                            std::process::exit(1);
                        }
                    };
                    if entry.path() == self.path {
                        match entry.file_type() {
                            Ok(file_type) => if file_type.is_symlink() {
                                return (FileType::Symlink, 'l', '@');
                            } else if file_type.is_dir() {
                                return (FileType::Directory, 'd', '/');
                            } else if file_type.is_fifo() {
                                return (FileType::NamedPipe, 'p', '|');
                            } else if file_type.is_socket() {
                                return (FileType::Socket, 's', '=');
                            } else if file_type.is_char_device() {
                                return (FileType::Socket, 'c', ' ');
                            } else if file_type.is_block_device() {
                                return (FileType::Socket, 'b', ' ');
                            } else if file_type.is_file() {
                                return (FileType::Regular, '-', ' ');
                            } else {
                                return (FileType::Regular, '?', '?');
                            }
                            Err(_) => {
                                // inside here
                                return (FileType::Regular, '?', '?');
                            }
                        }
                    }
                }
                // we couldn't detect
                return (FileType::Regular, '?', '?');
            }
            // for later
            Err(_) => (FileType::Regular, '?', '?'),
        }
    }

    pub fn get_entry_type(metadata: &Metadata) -> (FileType, char, char) {
        let is_executable = (metadata.permissions().mode() & 0o111) != 0;
        match true {
            _ if metadata.file_type().is_dir() => (FileType::Directory, 'd', '/'),
            _ if metadata.file_type().is_symlink() => (FileType::Symlink, 'l', '@'),
            _ if metadata.file_type().is_block_device() => (FileType::BlockDevice, 'b', ' '),
            _ if metadata.file_type().is_char_device() => (FileType::CharDevice, 'c', ' '),
            _ if metadata.file_type().is_fifo() => (FileType::NamedPipe, 'p', '|'),
            _ if metadata.file_type().is_socket() => (FileType::Socket, 's', '='),
            _ if metadata.file_type().is_file() && is_executable => {
                (FileType::Executable, '-', '*')
            }
            _ => (FileType::Regular, '-', ' '),
        }
    }
    fn get_user_name(&self) -> String {
        if self.metadata.is_none() {
            return "?".to_string();
        }
        match get_user_by_uid(self.metadata.clone().unwrap().uid()) {
            Some(user) => to_str(&user.name()),
            None => self.metadata.clone().unwrap().uid().to_string(),
        }
    }

    fn get_group_name(&self) -> String {
        if self.metadata.is_none() {
            return "?".to_string();
        }
        match get_group_by_gid(self.metadata.clone().unwrap().gid()) {
            Some(group) => to_str(&group.name()),
            None => self.metadata.clone().unwrap().gid().to_string(),
        }
    }

    fn get_date(&self) -> String {
        let dt = DateTime::<Utc>::from_timestamp(self.metadata.clone().unwrap().mtime(), 0);
        // unwrap() ???

        let datetime = match dt {
            Some(date) => date,
            None => {
                eprintln!("Error: Invalid timestamp");
                std::process::exit(1);
            }
        };
        let datetime = datetime.with_timezone(&Casablanca);
        let current_date_time = Utc::now().with_timezone(&Casablanca);
        let six_months_ago = current_date_time - Duration::days(183);

        // file can be created with a modification date (custom)
        if datetime > six_months_ago && datetime < current_date_time {
            return datetime.format("%b %e %H:%M").to_string();
        }

        datetime.format("%b %e  %Y").to_string()
    }

    fn get_permissions(&self) -> String {
        if self.metadata.is_none() {
            let (_, symbol, _) = self.get_pseudo_entry_type();
            symbol.to_string().push_str("?????????");
            return symbol.to_string();
        }
        let (_, symbol, _) = Self::get_entry_type(&self.metadata.clone().unwrap().clone());
        let mode = self.metadata.clone().unwrap().permissions().mode();

        let mut perms = String::with_capacity(10);
        perms.push(symbol);

        let perm_bits = [
            (0o400, 0o200, 0o100, 0o4000, 's', 'S'), //  r, w, x, setuid
            (0o040, 0o020, 0o010, 0o2000, 's', 'S'),
            (0o004, 0o002, 0o001, 0o1000, 't', 'T'),
        ];

        for &(read, write, exec, special, exec_char, no_exec_char) in &perm_bits {
            perms.push(if (mode & read) != 0 { 'r' } else { '-' });
            perms.push(if (mode & write) != 0 { 'w' } else { '-' });
            perms.push(
                if (mode & exec) != 0 {
                    if (mode & special) != 0 { exec_char } else { 'x' }
                } else {
                    if (mode & special) != 0 { no_exec_char } else { '-' }
                }
            );
        }

        // println!("hhh {:?} ", has_acl(&self.path));

        perms
    }

    pub fn append_file_type_indicator(&self) -> char {
        if self.metadata.is_none() {
            let (_, _, suffix) = self.get_pseudo_entry_type();
            return suffix;
        }
        if self.ls_config.l_flag_set {
            if Self::get_entry_type(&self.metadata.clone().unwrap()).0 == FileType::Symlink {
                if self.sym_metadata.is_some() {
                    let (_, _, suffix) = Self::get_entry_type(&self.sym_metadata.clone().unwrap());
                    return suffix;
                } else {
                    return ' ';
                }
            }
        }
        let (_, _, suffix) = Self::get_entry_type(&self.metadata.clone().unwrap());
        suffix
    }
}
