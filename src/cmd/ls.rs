use std::path::{ Path, PathBuf };
use std::io;
use std::io::{ ErrorKind };
use std::fs::{ self, DirEntry, Metadata, Permissions };
use std::os::unix::fs::{ FileTypeExt, MetadataExt, PermissionsExt };
use std::os::linux::fs::MetadataExt as LinuxMetadataExt;
use users::{ get_user_by_uid, get_group_by_gid };
use chrono::{ NaiveDateTime, Local, TimeZone };
use colored::{ Colorize, ColoredString, Color };
use std::cell::RefCell;
use std::fmt;

#[derive(Debug, Eq, Clone, PartialEq)]
pub struct LsConfig {
    a_flag_set: bool,
    l_flag_set: bool,
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
            l_flag_set: false,
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
                        self.l_flag_set = true;
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
                    let entries = Entries::new(&entries_vec, self);
                    if self.target_paths.len() != 1 || *self.status_code.borrow() != 0 {
                        println!("{}:", target_path);
                    }
                    println!("{}", entries);
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
            return (target_path.clone(), Ok(vec![Path::new(target_path).to_path_buf()]));
        }
    })
}

#[derive(Debug, PartialEq, Clone)]
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
        let size = Self::get_number_of_bytes(&metadata);
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

    fn as_array(&self) -> Vec<String> {
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

    fn get_number_of_bytes(metadata: &Metadata) -> Vec<String> {
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

    // here just for one entry
    // "%s %u %s %s %u %s %s\n", <file mode>, <number of links>,
    //  <owner name>, <group name>, <number of bytes in the file>,
    //  <date and time>, <pathname>
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
    // they need to be aligned ;)
    pub fn regular_format(&mut self) -> String {
        if self.ls_config.f_flag_set {
            self.append_file_type_indicator();
        }
        format!("{}", self.colored_entry_name)
    }

    /*Do not follow symbolic links named as operands unless the -H or -L options are specified.
    Write a slash ( '/' ) immediately after each pathname that is a directory,
    an asterisk ( '*' ) after each that is executable,
    a vertical bar ( '|' ) after each that is a FIFO,
    and an at sign ( '@' ) after each that is a symbolic link.
    For other file types, other symbols may be written.*/

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

// we need to colorize the output if and only if istty()  ;)
// we need to edit the impl iterator again and see if we can return 2 iterators (one of the valid targets and the ther for errors)
// the print also needs more way of handling (we need to find the max of each field and then format )
// need to know more about hiw

// to test things out here's the path  :   ../../../../dev

// there is a minor and major for the char and the block files
// there is a problem in the time
// there is a t and s (that need to be handled for the the execute)
//  implements the display trait for the vec<PathBuf>
// seems a good idea
#[derive(Debug, Clone)]
struct Entries {
    entries: Vec<Vec<String>>,
    ls_config: LsConfig,
}

impl Entries {
    fn new(paths: &Vec<PathBuf>, ls_config: &LsConfig) -> Self {
        let mut entries = Vec::new();
        for path in paths {
            let to_entry = Entry::new(path, ls_config);
            match to_entry {
                Ok(valid_entry) => {
                    entries.push(valid_entry.as_array());
                }
                Err(invalid_entry) => {
                    eprintln!("Error : {}", invalid_entry);
                }
            }
        }
        Self { entries: entries, ls_config: ls_config.clone() }
    }
}

// don't know if it will work
// i will need the ls_config
impl fmt::Display for Entries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //  iterate through the columns to find the max
        let mut vec_max = Vec::new();
        for i in 0..self.entries[0].len() {
            let mut max = self.entries[0][i].len();

            for row in &self.entries {
                if max < row[i].len() {
                    max = row[i].len();
                }
            }
            vec_max.push(max);
        }
        // eprintln!("{:?}", vec_max);

        if self.ls_config.l_flag_set {
            // we need to find the max for each field
            for j in 0..self.entries.len() {
                for k in 0..self.entries[j].len() {
                    let value = vec_max[k];
                    if k == 1 || k == 4 || k == 5 {
                        let formatted = format!("{0:>1$}", self.entries[j][k], value);
                        write!(f, "{} ", formatted)?;
                    } else {
                        let formatted = format!("{0:<1$}", self.entries[j][k], value);
                        write!(f, "{} ", formatted)?;
                    }
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}
