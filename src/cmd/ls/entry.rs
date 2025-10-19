use std::path::{ Path, PathBuf };
use std::io;
use std::fs::{ self, Metadata };
use std::os::unix::fs::{ FileTypeExt, MetadataExt, PermissionsExt };
use std::os::linux::fs::MetadataExt as LinuxMetadataExt;
use users::{ get_user_by_uid, get_group_by_gid };
use chrono::{ NaiveDateTime, Local, TimeZone };
use colored::Colorize;

use super::ls_config::{ LsConfig };

#[derive(Debug, PartialEq, Clone)]
pub enum FileTypeEnum {
    Regular,
    Directory,
    Symlink,
    // BrokenSymlink,
    CharDevice,
    BlockDevice,
    Socket,
    NamedPipe,
}

#[derive(Debug, Clone)]
pub struct Entry {
    permissions: String,
    number_of_links: String,
    onwer_name: String,
    group_name: String,
    minor: String,
    major: String,
    last_modified: String,
    colored_entry_name: String,
    entry_name: String,
    file_type: FileTypeEnum,
    path: PathBuf,
    ls_config: LsConfig,
}

impl Entry {
    pub fn new(path: &PathBuf, ls_config: &LsConfig) -> Result<Self, io::Error> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(some_metadata) => some_metadata,
            Err(e) => {
                return Err(e);
            }
        };
        let size = Self::get_size(&metadata);
        Ok(Self {
            permissions: Self::format_file_mode(path, &metadata),
            file_type: Self::get_file_type(&metadata),
            major: size[0].clone(),
            minor: size[1].clone(),
            number_of_links: metadata.nlink().to_string(),
            last_modified: Self::format_date(&metadata),
            entry_name: Self::get_entry_name(path),
            onwer_name: Self::get_user_name(metadata.uid()),
            group_name: Self::get_group_name(metadata.gid()),
            colored_entry_name: Self::color_entry_name(path, &metadata),
            path: path.clone(),
            ls_config: ls_config.clone(),
        })
    }

    pub fn as_array(&mut self) -> Vec<String> {
        if self.ls_config.f_flag_set {
            self.append_file_type_indicator();
        }

        if !self.ls_config.l_flag_set {
            return vec![self.colored_entry_name.clone()];
        }

        vec![
            self.permissions.clone(),
            self.number_of_links.clone(),
            self.onwer_name.clone(),
            self.group_name.clone(),
            self.major.clone(),
            self.minor.clone(),
            self.last_modified.clone(),
            self.colored_entry_name.clone()
        ]
    }

    fn get_size(metadata: &Metadata) -> Vec<String> {
        let file_type = Self::get_file_type(metadata);
        if file_type == FileTypeEnum::CharDevice || file_type == FileTypeEnum::BlockDevice {
            let rdev = LinuxMetadataExt::st_rdev(metadata);

            let mut major = (((rdev >> 8) & 0xfff) as u32).to_string();
            major.push(',');
            let minor = (((rdev & 0xff) | ((rdev >> 12) & 0xfff00)) as u32).to_string();

            return vec![major, minor];
        }

        vec!["".to_string(), metadata.size().to_string()]
    }

    fn get_entry_name(dir: &PathBuf) -> String {
        if dir == Path::new(".") {
            ".".to_string()
        } else if dir == Path::new("..") {
            "..".to_string()
        } else {
            dir.file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| dir.to_string_lossy().to_string())
        }
    }

    fn color_entry_name(path: &PathBuf, metadata: &Metadata) -> String {
        let entry_type = Self::get_file_type(metadata);
        let permissions = Self::format_file_mode(path, metadata);
        let mut is_executable = false;
        if permissions.contains('x') && entry_type == FileTypeEnum::Regular {
            is_executable = true;
        }
        match true {
            _ if is_executable == true => format!("{}", Self::get_entry_name(path).bold().green()),

            _ if entry_type == FileTypeEnum::Directory =>
                format!("{}", Self::get_entry_name(path).blue().bold()),

            _ if
                entry_type == FileTypeEnum::BlockDevice ||
                entry_type == FileTypeEnum::CharDevice ||
                entry_type == FileTypeEnum::NamedPipe
            => format!("{}", Self::get_entry_name(path).bold().yellow()),

            _ if entry_type == FileTypeEnum::Symlink =>
                format!("{}", Self::get_entry_name(path).cyan().bold()),

            _ if entry_type == FileTypeEnum::Socket =>
                format!("{}", Self::get_entry_name(path).bold().white()),
            _ => format!("{}", Self::get_entry_name(path).bright_white()),
        }
    }
    fn get_file_type(metadata: &Metadata) -> FileTypeEnum {
        match true {
            _ if metadata.file_type().is_dir() => FileTypeEnum::Directory,
            _ if metadata.file_type().is_symlink() => FileTypeEnum::Symlink,
            _ if metadata.file_type().is_block_device() => FileTypeEnum::BlockDevice,
            _ if metadata.file_type().is_char_device() => FileTypeEnum::CharDevice,
            _ if metadata.file_type().is_fifo() => FileTypeEnum::NamedPipe,
            _ if metadata.file_type().is_socket() => FileTypeEnum::Socket,
            _ => FileTypeEnum::Regular,
        }
    }
    fn get_user_name(u_id: u32) -> String {
        match get_user_by_uid(u_id) {
            Some(user) => user.name().to_string_lossy().to_string(),
            None => u_id.to_string(),
        }
    }

    fn get_group_name(g_id: u32) -> String {
        match get_group_by_gid(g_id) {
            Some(group) => group.name().to_string_lossy().to_string(),
            None => g_id.to_string(),
        }
    }

    fn format_date(metadata: &Metadata) -> String {
        let naive_datetime = NaiveDateTime::from_timestamp_opt(metadata.mtime(), 0).expect(
            "Invalid timestamp"
        );
        let datetime = Local.from_local_datetime(&naive_datetime).unwrap();
        datetime.format("%b %e %H:%M").to_string()
    }

    fn format_file_mode(path: &PathBuf, metadata: &Metadata) -> String {
        let entry_type = match Self::get_file_type(metadata) {
            FileTypeEnum::Directory => "d",
            FileTypeEnum::BlockDevice => "b",
            FileTypeEnum::CharDevice => "c",
            FileTypeEnum::Symlink => "l",
            FileTypeEnum::NamedPipe => "p",
            FileTypeEnum::Socket => "s",
            FileTypeEnum::Regular => "-",
            // FileTypeEnum::Unknown => "?",
        };

        let mode = metadata.permissions().mode();
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
        entry_type.to_string() + &permissions
    }

    pub fn long_format(&mut self) {
        if self.ls_config.f_flag_set {
            self.append_file_type_indicator();
        }
        if self.file_type == FileTypeEnum::Symlink {
            let pointed_to = if let Ok(pointed_to) = self.path.read_link() {
                Self::get_entry_name(&pointed_to)
            } else {
                "".to_string()
            };
            self.colored_entry_name.push_str(" -> ");
            self.colored_entry_name.push_str(&pointed_to);
        }
    }

    pub fn append_file_type_indicator(&mut self) {
        let mut is_executable = false;
        if self.permissions.contains('x') && self.file_type == FileTypeEnum::Regular {
            is_executable = true;
        }
        if self.file_type == FileTypeEnum::Directory {
            self.colored_entry_name.push_str("/");
        } else if is_executable {
            self.colored_entry_name.push_str("*");
        } else if self.file_type == FileTypeEnum::Symlink && !self.ls_config.l_flag_set {
            self.colored_entry_name.push_str("@");
        } else if self.file_type == FileTypeEnum::NamedPipe {
            self.colored_entry_name.push_str("|");
        }
    }
}
