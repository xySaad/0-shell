use std::path::{ Path, PathBuf };
use std::io;
use std::fs::{ self, Metadata };
use std::os::unix::fs::{ FileTypeExt, MetadataExt, PermissionsExt };
use std::os::linux::fs::MetadataExt as LinuxMetadataExt;
use libc::{ major, minor };
use users::{ get_user_by_uid, get_group_by_gid };
use chrono::{ DateTime, Utc, Local, TimeZone };
use chrono_tz::Africa::Casablanca;
use colored::Colorize;

use super::{ ls_config::{ LsConfig }, utils::{ is_broken_link } };

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
    // permissions: String,
    // number_of_links: String,
    // onwer_name: String,
    // group_name: String,
    // minor: String,
    // major: String,
    // last_modified: String,
    // colored_entry_name: String,
    // pub file_type: FileType,
    // ls_config: LsConfig,
    // pub num_blocks: u64,
    pub metadata: Metadata,
    pub ls_config: LsConfig,
    pub path: PathBuf,
}

impl Entry {
    pub fn new(path: &PathBuf, ls_config: &LsConfig) -> Result<Self, io::Error> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(some_metadata) => some_metadata,
            Err(e) => {
                return Err(e);
            }
        };

        Ok(Self {
            metadata: metadata.clone(),
            ls_config: ls_config.clone(),
            path: path.clone(),
        })

        // let (major, minor) = Self::get_size(&metadata, path);
        // let (entry_type, _, _) = Self::get_entry_type(&metadata, path);
        // Ok(Self {
        //     permissions: Self::get_permissions(&metadata, path),
        //     file_type: entry_type,
        //     major: major,
        //     minor: minor,
        //     number_of_links: metadata.nlink().to_string(),
        //     last_modified: Self::get_date(&metadata),
        //     onwer_name: Self::get_user_name(metadata.uid()),
        //     group_name: Self::get_group_name(metadata.gid()),
        //     colored_entry_name: Self::color_name(path, &metadata, &ls_config, false),
        //     ls_config: ls_config.clone(),
        //     num_blocks: metadata.st_blocks(),
        // })
    }

    pub fn as_array(&mut self) -> Vec<String> {
        let mut file_name = self.color_name(false);
        if self.ls_config.f_flag_set {
            file_name = self.append_file_type_indicator();
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
        let (file_type, _, _) = self.get_entry_type();
        if file_type == FileType::CharDevice || file_type == FileType::BlockDevice {
            let rdev = self.metadata.rdev();

            let mut major_val = major(rdev).to_string();
            major_val.push(',');
            let minor_val = minor(rdev).to_string();

            return (major_val, minor_val);
        }

        ("".to_string(), self.metadata.size().to_string())
    }

    fn get_entry_name(&self, is_whole_path: bool) -> String {
        if is_whole_path {
            return self.path.to_string_lossy().to_string();
        }
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

    fn color_name(&self, is_whole_path: bool) -> String {
        let (entry_type, _, _) = self.get_entry_type();
        let result = self.get_entry_name(is_whole_path);
        let mut colored_entry = match true {
            _ if entry_type == FileType::Executable => format!("{}", result.bold().green()),

            _ if entry_type == FileType::Directory => format!("{}", result.blue().bold()),

            _ if
                entry_type == FileType::BlockDevice ||
                entry_type == FileType::CharDevice ||
                entry_type == FileType::NamedPipe
            => format!("{}", result.bold().yellow()),

            _ if entry_type == FileType::Symlink => format!("{}", result.cyan().bold()),
            _ if entry_type == FileType::BrokenSymlink => format!("{}", result.red().bold()),

            _ if entry_type == FileType::Socket => format!("{}", result.bold().magenta()),
            _ => format!("{}", result.bright_white()),
        };

        // handle the special case of the symlink
        // if
        //     (entry_type == FileType::Symlink || entry_type == FileType::BrokenSymlink) &&
        //     self.ls_config.l_flag_set
        // {
        //     let pointed_to = if let Ok(pointed_to) = self.path.read_link() {
        //         if self.ls_config.f_flag_set && entry_type == FileType::Symlink {
        //             let  path_result = if !pointed_to.is_absolute() {
        //                 Path::new(self.path.parent().unwrap()).join(
        //                     &pointed_to.to_string_lossy().to_string()
        //                 )
        //             } else {
        //                 pointed_to.clone()
        //             };
        //             // eprintln!("huunaaa :! {:?}", pointed_to.is_absolute());
        //             // eprintln!("huunaaa : {:?}", path);
        //             let mut path_pointed_to = Entry::new(&path_result, &self.ls_config).unwrap();
        //             let (_, _, suffix) = path_pointed_to.get_entry_type();
        //             &pointed_to.to_string_lossy().to_string().push(suffix);
        //         }
        //         pointed_to.to_string_lossy().to_string()
        //     } else {
        //         "".to_string()
        //     };
        //     colored_entry.push_str(" -> ");
        //     colored_entry.push_str(&pointed_to);
        //     return colored_entry;
        // }

        colored_entry
    }
    pub fn get_entry_type(&self) -> (FileType, char, char) {
        let is_executable = (self.metadata.permissions().mode() & 0o111) != 0;
        match true {
            _ if self.metadata.file_type().is_dir() => (FileType::Directory, 'd', '/'),
            _ if self.metadata.file_type().is_symlink() && is_broken_link(&self.path) =>
                (FileType::BrokenSymlink, 'l', '@'),
            _ if self.metadata.file_type().is_symlink() => (FileType::Symlink, 'l', '@'),
            _ if self.metadata.file_type().is_block_device() => (FileType::BlockDevice, 'b', ' '),
            _ if self.metadata.file_type().is_char_device() => (FileType::CharDevice, 'c', ' '),
            _ if self.metadata.file_type().is_fifo() => (FileType::NamedPipe, 'p', '|'),
            _ if self.metadata.file_type().is_socket() => (FileType::Socket, 's', '='),
            _ if self.metadata.file_type().is_file() && is_executable =>
                (FileType::Executable, '-', '*'),
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
        let formatted = datetime.format("%b %e %H:%M").to_string();
        formatted
    }

    fn get_permissions(&self) -> String {
        let (_, symbol, _) = self.get_entry_type();
        let mode = self.metadata.permissions().mode();
        let permissions = [
            // owner permissions
            if (mode & 0o400) != 0 {
                'r'
            } else {
                '-'
            },
            if (mode & 0o200) != 0 { 'w' } else { '-' },
            if (mode & 0o100) != 0 { 'x' } else { '-' },
            // group ones
            if (mode & 0o040) != 0 {
                'r'
            } else {
                '-'
            },
            if (mode & 0o020) != 0 { 'w' } else { '-' },
            if (mode & 0o010) != 0 { 'x' } else { '-' },
            // others ones
            if (mode & 0o004) != 0 {
                'r'
            } else {
                '-'
            },
            if (mode & 0o002) != 0 { 'w' } else { '-' },
            if (mode & 0o001) != 0 { 'x' } else { '-' },
        ];
        let permissions: String = permissions.iter().collect();
        symbol.to_string() + &permissions
    }

    pub fn append_file_type_indicator(&self) -> String {
        let (file_type, _, suffix) = self.get_entry_type();
        let mut colored_name = self.color_name(false);

        if
            self.ls_config.l_flag_set &&
            (file_type == FileType::Symlink || file_type == FileType::BrokenSymlink)
        {
            let pointed_to = if let Ok(pointed_to) = self.path.read_link() {
                let path_result = if !pointed_to.is_absolute() {
                    Path::new(self.path.parent().unwrap()).join(
                        &pointed_to.to_string_lossy().to_string()
                    )
                } else {
                    pointed_to.clone()
                };
                // eprintln!("huunaaa :! {:?}", pointed_to.is_absolute());
                // eprintln!("huunaaa : {:?}", path);
                let mut path_pointed_to = if file_type == FileType::Symlink {
                    let target =Entry::new(&path_result, &self.ls_config).unwrap();
                    let mut colored_target = target.color_name(true);
                    let (_, _, suffix_target) = target.get_entry_type();
                    colored_target.push(suffix_target);
                    colored_target
                } else {
                    pointed_to.to_string_lossy().to_string()
                };

                path_pointed_to
            } else {
                "".to_string()
            };
            colored_name.push_str(" -> ");
            colored_name.push_str(&pointed_to);
            return colored_name;
        }

        colored_name.push(suffix);
        colored_name
    }
}
