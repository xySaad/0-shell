use std::path::{ Path, PathBuf };
use std::io;
use std::io::{ ErrorKind };
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
    status_code: RefCell<i32>,
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
            status_code: RefCell::new(0),
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
                        eprintln!("ls: invalid option -- '{c}'");
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

    // if any error is encountered the function must return the status code
    fn extract_valid_entries(&mut self) {
        self.target_paths.iter().for_each(|target_path| {
            let path = Path::new(&target_path);
            match fs::metadata(&path) {
                Ok(metadata) => {}
                Err(e) => {
                    if e.kind() == ErrorKind::NotFound {
                        self.status_code = RefCell::new(2);
                        eprintln!("ls: cannot access '{}': No such file or directory", target_path);
                    } else {
                        if *self.status_code.borrow() != 2 {
                            *self.status_code.borrow_mut() = 1;
                        }
                        eprintln!("{}", e);
                    }
                }
            }
        });
        self.target_paths.retain(|target_path| Path::new(&target_path).exists() == true);
        self.target_paths.sort_by(|a, b| a.cmp(&b));
    }

    fn print_ls(&mut self) {
        self.parse();
        self.extract_valid_entries();

        for (target_path, resulted_entry) in read_target_path(self) {
            match resulted_entry {
                Ok(entries_vec) => {
                    if self.target_paths.len() != 1 || *self.status_code.borrow() != 0 {
                        println!("{}:", target_path);
                    }
                    for ent in &entries_vec {
                        let new_entry = Entry::new(ent, self);
                        match new_entry {
                            Ok(entry) => {
                                if self.l_falg_set {
                                    println!("{}", entry.long_format());
                                } else {
                                    println!("{}", entry.regular_format());
                                }
                            }
                            // could be happening if the constructor returns an error while try to access the metadata
                            Err(e) => {
                                if *self.status_code.borrow() != 2 {
                                    *self.status_code.borrow_mut() = 1;
                                }
                                eprintln!("{}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    match e.kind() {
                        ErrorKind::PermissionDenied => {
                            *self.status_code.borrow_mut() = 2;
                            eprintln!("ls: cannot open directory '{}': Permission denied", target_path);
                        }
                        _ => {
                            if *self.status_code.borrow() != 2 {
                                *self.status_code.borrow_mut() = 1;
                            }
                            eprintln!("{}", e.kind());
                        }
                    }
                }
            }
        }
    }
}

pub fn ls(args: &Vec<String>) -> i32 {
    let mut ls_config = LsConfig::new(args);
    ls_config.print_ls();
    let status_code = *ls_config.status_code.borrow();
    println!("status code: {}", status_code);
    return *ls_config.status_code.borrow();
}

// we have to sort the entries alphabetically and we need to filter the valid targets from the ls and print the errors and then
// order them in alphabtical order and then process them

pub fn read_target_path(
    ls_config: &LsConfig
) -> impl Iterator<Item = (String, Result<Vec<PathBuf>, io::Error>)> {
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
                                eprintln!("target : {} {}", target_path, err);
                            }
                        }
                    }

                    let mut paths = if ls_config.a_flag_set {
                        paths.push(Path::new(".").to_path_buf());
                        paths.push(Path::new("..").to_path_buf());
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
                    paths.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

                    return (target_path.clone(), Ok(paths.clone()));
                }
                // here we need to return the error and the kind of it
                Err(e) => {
                    return (target_path.clone(), Err(e));
                }
            };
        } else {
            return (target_path.clone(), Ok(vec![]));
        }
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

// the file mode needs to be changed for the sake of coloration ( we need the persmissions to see if we can colorise things)
#[derive(Debug)]
pub struct Entry {
    permissions: String,
    number_of_links: u64,
    onwer_name: String,
    group_name: String,
    size: u64,
    last_modified: i64,
    entry_name: String,
    file_type: FileTypeEnum,
    is_executable: RefCell<bool>,
    colored_entry_name: RefCell<String>,
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

        Ok(Self {
            permissions: Self::format_file_mode(path, &metadata),
            file_type: Self::get_file_type(&metadata),
            size: metadata.size(),
            number_of_links: metadata.nlink(),
            last_modified: metadata.mtime(),
            entry_name: Self::get_entry_name(path),
            onwer_name: Self::get_user_name(metadata.uid()),
            group_name: Self::get_group_name(metadata.gid()),
            is_executable: RefCell::new(false),
            colored_entry_name: RefCell::new(format!("{}", Self::get_entry_name(path).white())),
            path: path.clone(),
            ls_config: ls_config.clone(),
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
        if self.permissions.contains('x') && self.file_type == FileTypeEnum::Regular {
            *self.is_executable.borrow_mut() = true;
        }
        match true {
            _ if *self.is_executable.borrow() == true => {
                *self.colored_entry_name.borrow_mut() = format!(
                    "{}",
                    self.entry_name.clone().bold().green()
                );
            }
            _ if *entry_type == FileTypeEnum::Directory => {
                *self.colored_entry_name.borrow_mut() = format!(
                    "{}",
                    self.entry_name.clone().blue().bold()
                );
            }
            _ if
                *entry_type == FileTypeEnum::BlockDevice ||
                *entry_type == FileTypeEnum::CharDevice ||
                *entry_type == FileTypeEnum::NamedPipe
            => {
                *self.colored_entry_name.borrow_mut() = format!(
                    "{}",
                    self.entry_name.clone().bold().yellow()
                );
            }
            _ if *entry_type == FileTypeEnum::Symlink => {
                *self.colored_entry_name.borrow_mut() = format!(
                    "{}",
                    self.entry_name.cyan().bold()
                );
            }

            _ if *entry_type == FileTypeEnum::Socket => {
                *self.colored_entry_name.borrow_mut() = format!(
                    "{}",
                    self.entry_name.clone().bold().white()
                );
            }
            _ => {
                *self.colored_entry_name.borrow_mut() = format!(
                    "{}",
                    self.entry_name.clone().bright_white()
                );
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

    // here just for one entry
    // "%s %u %s %s %u %s %s\n", <file mode>, <number of links>,
    //  <owner name>, <group name>, <number of bytes in the file>,
    //  <date and time>, <pathname>
    pub fn long_format(&self) -> String {
        self.color_entry_name();
        if self.ls_config.f_flag_set {
            self.append_file_type_indicator();
        }
        let formatted_string = format!(
            "{} {:>5} {} {} {:>5} {} {}",
            self.permissions,
            self.number_of_links,
            self.onwer_name,
            self.group_name,
            self.size,
            self.format_date(),
            *self.colored_entry_name.borrow()
        );
        if self.file_type == FileTypeEnum::Symlink {
            let pointed_to = if let Ok(pointed_to) = self.path.read_link() {
                println!("point to :  {:?}", pointed_to);
                Self::get_entry_name(&pointed_to)
            } else {
                "".to_string()
            };
            return formatted_string + " -> " + &pointed_to;
        }
        formatted_string
    }
    // they need to be aligned ;)
    pub fn regular_format(&self) -> String {
        self.color_entry_name();
        if self.ls_config.f_flag_set {
            self.append_file_type_indicator();
        }
        format!("{}", *self.colored_entry_name.borrow())
    }

    /*Do not follow symbolic links named as operands unless the -H or -L options are specified.
    Write a slash ( '/' ) immediately after each pathname that is a directory,
    an asterisk ( '*' ) after each that is executable,
    a vertical bar ( '|' ) after each that is a FIFO,
    and an at sign ( '@' ) after each that is a symbolic link.
    For other file types, other symbols may be written.*/

    pub fn append_file_type_indicator(&self) {
        if self.file_type == FileTypeEnum::Directory {
            self.colored_entry_name.borrow_mut().push_str("/");
        } else if *self.is_executable.borrow() {
            self.colored_entry_name.borrow_mut().push_str("*");
        } else if self.file_type == FileTypeEnum::Symlink && !self.ls_config.l_falg_set {
            self.colored_entry_name.borrow_mut().push_str("@");
        } else if self.file_type == FileTypeEnum::NamedPipe {
            self.colored_entry_name.borrow_mut().push_str("|");
        }
    }
}

// we need to colorize the output if and only if istty()  ;)
// we need to edit the impl iterator again and see if we can return 2 iterators (one of the valid targets and the ther for errors)
// the print also needs more way of handling (we need to find the max of each field and then format )
// need to know more about hiw

// to test things out here's the path  :   ../../../../dev
