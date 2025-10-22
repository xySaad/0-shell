use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Utc};
use chrono_tz::Africa::Casablanca;
use colored::Colorize;
use libc::{major, minor};
use std::fs::{self, Metadata};
use std::io::{self, ErrorKind};
use std::os::linux::fs::MetadataExt as LinuxMetadataExt;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use users::{get_group_by_gid, get_user_by_uid};

use super::{ls_config::LsConfig, utils::is_broken_link};

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
    
    // pub symlink_path : Option<PathBuf>,
}

impl Entry {
    pub fn new(path: &PathBuf, ls_config: &LsConfig) -> Option<Self> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(some_metadata) => some_metadata,
            Err(e) => {
                return None;
            }
        };

        Some(Self {
            metadata: metadata.clone(),
            ls_config: ls_config.clone(),
            path: path.clone(),
        })
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
            file_name,
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
        let colored_entry = match true {
            _ if entry_type == FileType::Executable => format!("{}", result.bold().green()),

            _ if entry_type == FileType::Directory => format!("{}", result.blue().bold()),

            _ if entry_type == FileType::BlockDevice
                || entry_type == FileType::CharDevice
                || entry_type == FileType::NamedPipe =>
            {
                format!("{}", result.bold().yellow())
            }

            _ if entry_type == FileType::Symlink => format!("{}", result.cyan().bold()),
            _ if entry_type == FileType::BrokenSymlink => format!("{}", result.red().bold()),

            _ if entry_type == FileType::Socket => format!("{}", result.bold().magenta()),
            _ => format!("{}", result.bright_white()),
        };

        colored_entry
    }
    pub fn get_entry_type(&self) -> (FileType, char, char) {
        let is_executable = (self.metadata.permissions().mode() & 0o111) != 0;
        match true {
            _ if self.metadata.file_type().is_dir() => (FileType::Directory, 'd', '/'),
            _ if self.metadata.file_type().is_symlink() && is_broken_link(&self.path) => {
                (FileType::BrokenSymlink, 'l', '@')
            }
            _ if self.metadata.file_type().is_symlink() => (FileType::Symlink, 'l', '@'),
            _ if self.metadata.file_type().is_block_device() => (FileType::BlockDevice, 'b', ' '),
            _ if self.metadata.file_type().is_char_device() => (FileType::CharDevice, 'c', ' '),
            _ if self.metadata.file_type().is_fifo() => (FileType::NamedPipe, 'p', '|'),
            _ if self.metadata.file_type().is_socket() => (FileType::Socket, 's', '='),
            _ if self.metadata.file_type().is_file() && is_executable => {
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

        // Calculate difference in time
        let six_months_ago = current_date_time - Duration::days(183);
        let six_months_future = current_date_time + Duration::days(183);

        // Show time if modification date is within +/- 6 months of current date
        if datetime > six_months_ago && datetime < six_months_future {
            return datetime.format("%b %e %H:%M").to_string();
        }

        return datetime.format("%b %e  %Y").to_string();
    }

    fn get_permissions(&self) -> String {
        let (_, symbol, _) = self.get_entry_type();
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
            other_exec,
        ];

        symbol.to_string() + &permissions.iter().collect::<String>()
    }

    pub fn append_file_type_indicator(&self) -> String {
        let (file_type, _, suffix) = self.get_entry_type();
        let mut colored_name = self.color_name(false);

        if self.ls_config.l_flag_set
            && (file_type == FileType::Symlink || file_type == FileType::BrokenSymlink)
        {
            let pointed_to = if let Ok(pointed_to) = self.path.read_link() {
                let path_result = if !pointed_to.is_absolute() {
                    Path::new(self.path.parent().unwrap())
                        .join(&pointed_to.to_string_lossy().to_string())
                } else {
                    pointed_to.clone()
                };
                // eprintln!("huunaaa :! {:?}", pointed_to.is_absolute());
                // eprintln!("huunaaa : {:?}", path);
                let path_pointed_to = if file_type == FileType::Symlink {
                    let target = Entry::new(&path_result, &self.ls_config).unwrap();
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
