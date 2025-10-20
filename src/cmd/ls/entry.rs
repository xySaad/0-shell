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

pub struct Entry {
    permissions: String,
    number_of_links: String,
    onwer_name: String,
    group_name: String,
    minor: String,
    major: String,
    last_modified: String,
    colored_entry_name: String,
    pub entry_name: String,
    pub file_type: FileTypeEnum,
    path: PathBuf,
    ls_config: LsConfig,
    pub blocks_count: u64,
}

impl Entry {
    pub fn new(path: &PathBuf, ls_config: &LsConfig) -> Result<Self, io::Error> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(some_metadata) => some_metadata,
            Err(e) => {
                return Err(e);
            }
        };
        let (major, minor) = Self::get_size(&metadata);
        Ok(Self {
            permissions: Self::format_file_mode(path, &metadata),
            file_type: Self::get_file_type(&metadata).0,
            major: major,
            minor: minor,
            number_of_links: metadata.nlink().to_string(),
            last_modified: Self::format_date(&metadata),
            entry_name: Self::get_entry_name(path),
            onwer_name: Self::get_user_name(metadata.uid()),
            group_name: Self::get_group_name(metadata.gid()),
            colored_entry_name: Self::color_entry_name(path, &metadata, &ls_config),
            path: path.clone(),
            ls_config: ls_config.clone(),
            blocks_count: metadata.st_blocks(),
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

    fn get_size(metadata: &Metadata) -> (String, String) {
        let (file_type, _) = Self::get_file_type(metadata);
        if file_type == FileTypeEnum::CharDevice || file_type == FileTypeEnum::BlockDevice {
            let rdev = LinuxMetadataExt::st_rdev(metadata);

            let mut major = (((rdev >> 8) & 0xfff) as u32).to_string();
            major.push(',');
            let minor = (((rdev & 0xff) | ((rdev >> 12) & 0xfff00)) as u32).to_string();

            return (major, minor);
        }

        ("".to_string(), metadata.size().to_string())
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

    fn color_entry_name(path: &PathBuf, metadata: &Metadata, ls_config: &LsConfig) -> String {
        let (entry_type, _) = Self::get_file_type(metadata);
        let permissions = Self::format_file_mode(path, metadata);
        let mut is_executable = false;
        if permissions.contains('x') && entry_type == FileTypeEnum::Regular {
            is_executable = true;
        }
        let mut colored_entry = match true {
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
        };

        // handle the special case of the symlink
        if entry_type == FileTypeEnum::Symlink && ls_config.l_flag_set {
            let pointed_to = if let Ok(pointed_to) = path.read_link() {
                pointed_to.to_string_lossy().to_string()
            } else {
                "".to_string()
            };
            colored_entry.push_str(" -> ");
            colored_entry.push_str(&pointed_to);
            return colored_entry;
        }

        colored_entry
    }
    fn get_file_type(metadata: &Metadata) -> (FileTypeEnum, char) {
        match true {
            _ if metadata.file_type().is_dir() => (FileTypeEnum::Directory, 'd'),
            _ if metadata.file_type().is_symlink() => (FileTypeEnum::Symlink, 'l'),
            _ if metadata.file_type().is_block_device() => (FileTypeEnum::BlockDevice, 'b'),
            _ if metadata.file_type().is_char_device() => (FileTypeEnum::CharDevice, 'c'),
            _ if metadata.file_type().is_fifo() => (FileTypeEnum::NamedPipe, 'p'),
            _ if metadata.file_type().is_socket() => (FileTypeEnum::Socket, 's'),
            _ => (FileTypeEnum::Regular, '-'),
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
        let (_, symbol) = Self::get_file_type(metadata);
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
        symbol.to_string() + &permissions
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
