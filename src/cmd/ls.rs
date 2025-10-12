use std::path::{ Path, PathBuf };
use std::io;
use std::fs::{ self, DirEntry, Metadata, Permissions };
use std::os::unix::fs::{ FileTypeExt, MetadataExt, PermissionsExt };
use users::{ get_user_by_uid, get_group_by_gid };
use chrono::{ NaiveDateTime, Local, TimeZone };
use colored::{ Colorize, ColoredString, Color };
use std::cell::RefCell;

#[derive(Debug, Eq, Clone, PartialEq)]
pub struct LsConfig {
    a_flag_set: bool,
    l_falg_set: bool,
    f_flag_set: bool,
    default_target_path: String,
    target_paths: Vec<String>,
    flags: Vec<String>,
}

impl LsConfig {
    fn new(args: &Vec<String>) -> Self {
        Self {
            a_flag_set: false,
            l_falg_set: false,
            f_flag_set: false,
            default_target_path: ".".to_string(),
            target_paths: args
                .iter()
                .filter(|a| !a.starts_with('-'))
                .cloned() // hadiiii 7itash bla biha &String instead of owned Strings // and i don't want to consume l args
                .collect(),
            flags: args
                .iter()
                .filter(|a| a.starts_with('-'))
                .cloned()
                .collect(),
        }
    }
    // didn't like too much i'll see if there is another way to do the same thing!
    fn parse(&mut self) {
        for flag in &self.flags {
            for c in flag.chars().skip(1) {
                match c {
                    'a' => {
                        self.a_flag_set = true;
                    }
                    'l' => {
                        self.l_falg_set = true;
                    }
                    'F' => {
                        self.f_flag_set = true;
                    }
                    _ => {
                        eprintln!("ls: invalid option -- {c}");
                        std::process::exit(2);
                    }
                }
            }
        }

        // if the target_paths is 0 push the default to targets_paths

        if self.target_paths.len() == 0 {
            self.target_paths.push(self.default_target_path.clone());
        }
    }
}

pub fn ls(args: &Vec<String>) -> i32 {
    let mut ls_config = LsConfig::new(args);
    ls_config.parse();

    for target_path in read_target_path(&ls_config) {
        //println!(" target path :{:?}", target_path);
        match target_path {
            Ok(entries_vec) => {
                for ent in &entries_vec {
                    //println!("entry {:?}", ent);
                    let new_entry = Entry::new(ent);
                    match new_entry {
                        Ok(mut valid_entry) => {
                            if ls_config.l_falg_set {
                                eprintln!("{}", valid_entry.long_format());
                            } else {
                                eprintln!("{}", valid_entry.regular_format());
                            }
                        }
                        Err(e) => {
                            eprintln!("{:?}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{:}", e);
            }
        }
    }

    return 0;
}

// LET'S create a fn that handles the process of reading one target(file/ directory and returns the target with its DirEntries as a Vect)
// -a hnaa is handled as well
// the error lackes the target_name
// to be fixed later
pub fn read_target_path(
    ls_config: &LsConfig
) -> impl Iterator<Item = Result<Vec<PathBuf>, io::Error>> {
    ls_config.target_paths.iter().map(|target_path| {
        let path = Path::new(target_path);
        if path.is_dir() {
            match fs::read_dir(target_path) {
                Ok(entries) => {
                    let mut paths = Vec::new();
                    for entry in entries {
                        match entry {
                            Ok(valid_entry) => {
                                paths.push(valid_entry.path());
                            }
                            Err(err) => {
                                eprintln!("{}", err);
                            }
                        }
                    }

                    let paths = if ls_config.a_flag_set {
                        paths.push(Path::new(".").to_path_buf());
                        paths.push(Path::new("..").to_path_buf());
                        paths.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
                        paths
                    } else {
                        paths
                            .clone()
                            .into_iter()
                            .filter(
                                |entry|
                                    !entry.file_name().unwrap().to_string_lossy().starts_with(".")
                            )
                            .collect::<Vec<_>>()
                    };

                    return Ok(paths.clone());
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(vec![Path::new(target_path).to_path_buf()])
    })
}

#[derive(Debug, PartialEq)]
enum FileTypeEnum {
    Regular,
    Directory,
    Symlink,
    CharDevice,
    BlockDevice,
    Socket,
    NamedPipe,
    // Unknown,
}

#[derive(Debug)]
pub struct Entry {
    permissions: Permissions,
    number_of_links: u64,
    onwer_name: String,
    group_name: String,
    size: u64,
    last_modified: i64,
    entry_name: String,
    file_type: FileTypeEnum,
    is_executable: RefCell<bool>,
    colored_string: RefCell<ColoredString>,
}

impl Entry {
    pub fn new(dir: &PathBuf) -> Result<Self, io::Error> {
        let metadata = match dir.metadata() {
            Ok(some_metadata) => some_metadata,
            Err(e) => {
                return Err(e);
            }
        };

        Ok(Self {
            permissions: metadata.permissions(),
            file_type: Self::get_file_type(&metadata),
            size: metadata.size(),
            number_of_links: metadata.nlink(),
            last_modified: metadata.mtime(),
            entry_name: Self::get_entry_name(dir),
            onwer_name: Self::get_user_name(metadata.uid()),
            group_name: Self::get_group_name(metadata.gid()),
            is_executable: RefCell::new(false),
            colored_string: RefCell::new(Self::get_entry_name(dir).white()),
        })
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

    fn color_entry_name(&self) {
        let entry_type = &self.file_type;
        let is_executable = self.is_executable.borrow();
        println!("entry_type: {:?}", entry_type);
        match true {
            _ if *entry_type == FileTypeEnum::Directory => {
                *self.colored_string.borrow_mut() = self.entry_name.clone().blue().bold();
            },
            _ if
                *entry_type == FileTypeEnum::BlockDevice ||
                *entry_type == FileTypeEnum::CharDevice ||
                *entry_type == FileTypeEnum::NamedPipe
            => {
                *self.colored_string.borrow_mut() = self.entry_name.clone().bold().yellow();
            },
            _ if *entry_type == FileTypeEnum::Symlink => {
                println!("cyan!!");
                *self.colored_string.borrow_mut() = self.entry_name.cyan();
            }

            _ if *entry_type == FileTypeEnum::Socket => {
                *self.colored_string.borrow_mut() = self.entry_name.clone().bold().white();
            }
            _ if *entry_type == FileTypeEnum::Directory && *is_executable == true => {
                *self.colored_string.borrow_mut() = self.entry_name.clone().bold().green();
            }
            _ => {
                *self.colored_string.borrow_mut() = self.entry_name.clone().bold().white();
            }
        };
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

    fn format_date(&self) -> String {
        let naive_datetime = NaiveDateTime::from_timestamp_opt(self.last_modified, 0).expect(
            "Invalid timestamp"
        );
        let datetime = Local.from_local_datetime(&naive_datetime).unwrap();
        datetime.format("%b %e %H:%M").to_string()
    }

    fn format_file_mode(&self) -> String {
        let entry_type = match self.file_type {
            FileTypeEnum::Directory => "d",
            FileTypeEnum::BlockDevice => "b",
            FileTypeEnum::CharDevice => "c",
            FileTypeEnum::Symlink => "l",
            FileTypeEnum::NamedPipe => "p",
            FileTypeEnum::Socket => "s",
            FileTypeEnum::Regular => "-",
            // FileTypeEnum::Unknown => "?",
        };

        let mode = self.permissions.mode();
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
        if permissions.contains("x") {
            let mut executable = self.is_executable.borrow_mut();
            *executable = true;
        }

        entry_type.to_string() + &permissions
    }

    // here just for one entry
    // "%s %u %s %s %u %s %s\n", <file mode>, <number of links>,
    //  <owner name>, <group name>, <number of bytes in the file>,
    //  <date and time>, <pathname>
    pub fn long_format(&mut self) -> String {
        self.color_entry_name();
        format!(
            "{} {:>5} {} {} {:>5} {} {}",
            self.format_file_mode(),
            self.number_of_links,
            self.onwer_name,
            self.group_name,
            self.size,
            self.format_date(),
            *self.colored_string.borrow()
        )
    }
    // they need to be aligned ;)
    pub fn regular_format(&self) -> String {
        self.color_entry_name();
        format!("{}", *self.colored_string.borrow())
    }
}

// pub fn append_file_type_indicator() {}
