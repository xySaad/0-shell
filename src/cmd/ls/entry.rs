use chrono::{ DateTime, Datelike, Duration, Local, TimeZone, Utc };
use chrono_tz::Africa::Casablanca;
use colored::Colorize;
use libc::{ major, minor };
use std::fs::{ self, Metadata };
use std::io::{ self, ErrorKind };
use std::os::linux::fs::MetadataExt as LinuxMetadataExt;
use std::os::unix::fs::{ FileTypeExt, MetadataExt, PermissionsExt };
use std::path::{ Path, PathBuf };
use users::{ get_group_by_gid, get_user_by_uid };

use super::{ ls_config::LsConfig, utils::is_broken_link };

#[derive(Debug, PartialEq, Clone)]
pub enum FileType {
    Regular,
    Directory,
    Symlink,
    BrokenSymlink,
    Executable,
    CharDevice,
    BlockDevice,
    Socket,
    NamedPipe,
}
#[derive(Debug, Clone)]
pub struct Entry {
    pub metadata: Metadata,
    pub ls_config: LsConfig,
    pub path: PathBuf,
    pub symlink_target_path: Option<PathBuf>,
    pub target_metadata: Option<Metadata>,

    // pub symlink_path : Option<PathBuf>,
}

impl Entry {
    pub fn new(path: &PathBuf, ls_config: &LsConfig) -> Option<Self> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(some_metadata) => some_metadata,
            Err(_) => {
                return None;
            }
        };
        let mut symlink_target_path: Option<PathBuf> = None;
        let mut target_metadata: Option<Metadata> = None;

        if metadata.file_type().is_symlink() {
            match fs::read_link(path) {
                Ok(target_path) => {
                    let resolved_target_path = if target_path.is_absolute() {
                        target_path.clone()
                    } else {
                        path.parent()
                            .unwrap_or_else(|| Path::new(""))
                            .join(&target_path)
                    };
                    symlink_target_path = Some(resolved_target_path);
                    // we need to use metadata to follow the link to the inner target ;)
                    match fs::metadata(path) {
                        Ok(some_metadata) => {
                            target_metadata = Some(some_metadata);
                        }
                        Err(_) => {}
                    };
                }
                Err(_) => {}
            };
        }
        Some(Self {
            metadata: metadata.clone(),
            ls_config: ls_config.clone(),
            path: path.clone(),
            symlink_target_path: symlink_target_path,
            target_metadata: target_metadata,
        })
    }

    pub fn as_array(&mut self) -> Vec<String> {
        let mut file_name = self.color_name();
        if self.ls_config.l_flag_set {
            if self.symlink_target_path.is_some() {
                file_name.push_str(" -> ");
                file_name.push_str(
                    &self.symlink_target_path.clone().unwrap().to_string_lossy().to_string()
                );
            }
        }
        if self.ls_config.f_flag_set {
            file_name.push(self.append_file_type_indicator());
        }

        if !self.ls_config.l_flag_set {
            return vec![file_name];
        }

        let (major, minor) = self.get_size();

        vec![
            self.get_permissions(),
            self.metadata.nlink().to_string(),
            self.get_user_name(),
            self.get_group_name(),
            major.clone(),
            minor.clone(),
            self.get_date(),
            file_name
        ]
    }

    fn get_size(&self) -> (String, String) {
        let (file_type, _, _) = Self::get_entry_type(&self.metadata.clone());
        if file_type == FileType::CharDevice || file_type == FileType::BlockDevice {
            let rdev = self.metadata.rdev();

            let mut major_val = major(rdev).to_string();
            major_val.push(',');
            let minor_val = minor(rdev).to_string();

            return (major_val, minor_val);
        }

        ("".to_string(), self.metadata.size().to_string())
    }

    fn get_entry_name(&self) -> String {
        if self.path.to_string_lossy().to_string().ends_with("..") {
            "..".to_string()
        } else if self.path.to_string_lossy().to_string().ends_with(".") {
            ".".to_string()
        } else {
            self.path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| self.path.to_string_lossy().to_string())
        }
    }

    fn color_name(&self) -> String {
        let (entry_type, _, _) = Self::get_entry_type(&self.metadata.clone());
        let result = self.get_entry_name();
        let colored_entry = match true {
            _ if entry_type == FileType::Executable => format!("{}", result.bold().green()),

            _ if entry_type == FileType::Directory => format!("{}", result.blue().bold()),

            _ if
                entry_type == FileType::BlockDevice ||
                entry_type == FileType::CharDevice ||
                entry_type == FileType::NamedPipe
            => {
                format!("{}", result.bold().yellow())
            }

            _ if entry_type == FileType::Symlink => format!("{}", result.cyan().bold()),
            _ if entry_type == FileType::BrokenSymlink => format!("{}", result.red().bold()),

            _ if entry_type == FileType::Socket => format!("{}", result.bold().magenta()),
            _ => format!("{}", result.bright_white()),
        };

        colored_entry
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
        match get_user_by_uid(self.metadata.uid()) {
            Some(user) => user.name().to_string_lossy().to_string(),
            None => self.metadata.uid().to_string(),
        }
    }

    fn get_group_name(&self) -> String {
        match get_group_by_gid(self.metadata.gid()) {
            Some(group) => group.name().to_string_lossy().to_string(),
            None => self.metadata.gid().to_string(),
        }
    }

    fn get_date(&self) -> String {
        let dt = DateTime::<Utc>::from_timestamp(self.metadata.mtime(), 0);
        let datetime = dt.unwrap().with_timezone(&Casablanca);
        let current_date_time = Utc::now().with_timezone(&Casablanca);

        let six_months_ago = current_date_time - Duration::days(183);
        let six_months_future = current_date_time + Duration::days(183);

        if datetime > six_months_ago && datetime < six_months_future {
            return datetime.format("%b %e %H:%M").to_string();
        }

        datetime.format("%b %e  %Y").to_string()
    }

    fn get_permissions(&self) -> String {
        let (_, symbol, _) = Self::get_entry_type(&self.metadata.clone());
        let mode = self.metadata.permissions().mode();

        let is_mode = |bit| (mode & bit) != 0;

        let owner_read = if is_mode(0o400) { 'r' } else { '-' };
        let owner_write = if is_mode(0o200) { 'w' } else { '-' };
        let owner_exec = if is_mode(0o100) {
            if is_mode(0o4000) { 's' } else { 'x' }
        } else {
            if is_mode(0o4000) { 'S' } else { '-' }
        };

        // Group permissions
        let group_read = if is_mode(0o040) { 'r' } else { '-' };
        let group_write = if is_mode(0o020) { 'w' } else { '-' };
        let group_exec = if is_mode(0o010) {
            if is_mode(0o2000) { 's' } else { 'x' }
        } else {
            if is_mode(0o2000) { 'S' } else { '-' }
        };

        let other_read = if is_mode(0o004) { 'r' } else { '-' };
        let other_write = if is_mode(0o002) { 'w' } else { '-' };
        let other_exec = if is_mode(0o001) {
            if is_mode(0o1000) { 't' } else { 'x' }
        } else {
            if is_mode(0o1000) { 'T' } else { '-' }
        };

        let permissions = vec![
            owner_read,
            owner_write,
            owner_exec,
            group_read,
            group_write,
            group_exec,
            other_read,
            other_write,
            other_exec
        ];

        symbol.to_string() + &permissions.iter().collect::<String>()
    }

    pub fn append_file_type_indicator(&self) -> char {
        if self.ls_config.l_flag_set {
            if Self::get_entry_type(&self.metadata).0 == FileType::Symlink {
                if self.target_metadata.is_some() {
                    let (_, _, suffix) = Self::get_entry_type(
                        &self.target_metadata.clone().unwrap()
                    );
                    return suffix;
                } else {
                    return ' ' 
                }
            }
        }
        let (_, _, suffix) = Self::get_entry_type(&self.metadata);
        suffix
    }
}
