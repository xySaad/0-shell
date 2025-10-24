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

use super::{ ls_config::LsConfig, utils::{  apply_color } };

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

#[derive(Debug, PartialEq, Clone)]
pub enum ColorStyle {
    BoldGreen,
    BlueBold,
    BoldYellow,
    CyanBold,
    RedBold,
    BoldMagenta,
    BrightWhite,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub metadata: Option<Metadata>,
    pub ls_config: LsConfig,
    pub path: PathBuf,
    pub symlink_target_path: Option<PathBuf>,
    pub target_metadata: Option<Metadata>,
}

impl Entry {
    pub fn new(path: &PathBuf, ls_config: &LsConfig) -> Result<Self, (Option<Self>, io::Error) {
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
                    symlink_target_path = Some(target_path);
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
        let color_style = self.color_name_style();
        let mut file_name = apply_color(&self.get_entry_name(), color_style);
        if self.ls_config.l_flag_set {
            if self.symlink_target_path.is_some() {
                file_name.push_str(" -> ");
                let target_color = match &self.target_metadata {
                    Some(metadata) => Self::color_style_for_metadata(&metadata),
                    None => ColorStyle::RedBold,
                };
                let target_name = self.symlink_target_path
                    .clone()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                file_name.push_str(&apply_color(&target_name, target_color));
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

    fn color_name_style(&self) -> ColorStyle {
        let (entry_type, _, _) = Self::get_entry_type(&self.metadata);

        let is_broken_symlink =
            entry_type == FileType::Symlink &&
            self.symlink_target_path.is_some() &&
            self.target_metadata.is_none();

        if is_broken_symlink {
            return ColorStyle::RedBold;
        }
        match entry_type {
            FileType::Executable => ColorStyle::BoldGreen,
            FileType::Directory => ColorStyle::BlueBold,
            FileType::BlockDevice | FileType::CharDevice | FileType::NamedPipe =>
                ColorStyle::BoldYellow,
            FileType::Symlink => ColorStyle::CyanBold,
            FileType::BrokenSymlink => ColorStyle::RedBold,
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

        let mut permissions = symbol.to_string() + &permissions.iter().collect::<String>();
        let attr_len = unsafe {
            libc::listxattr(
                self.path.to_str().unwrap_or("").as_ptr() as *const _,
                std::ptr::null_mut(),
                0
            )
        };
        if attr_len > 0 {
            permissions.push('+');
        }

        permissions
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
                    return ' ';
                }
            }
        }
        let (_, _, suffix) = Self::get_entry_type(&self.metadata);
        suffix
    }
}
