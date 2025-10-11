use std::path::Path;
use std::io;
use std::fs::{ self, DirEntry };

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
        match target_path {
            Ok((target_name, entries_vec)) => {
                for ent in &entries_vec {
                    print_long_format(ent);
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
pub fn read_target_path(
    ls_config: &LsConfig
) -> impl Iterator<Item = Result<(String, Vec<DirEntry>), io::Error>> {
    ls_config.target_paths.iter().map(|target_path| {
        match fs::read_dir(target_path) {
            Ok(entries) => {
                let dir_entries: Result<Vec<DirEntry>, io::Error> = entries.collect();

                let vect_entries = match dir_entries {
                    Ok(vec) => if !ls_config.a_flag_set {
                        vec
                    } else {
                        vec.into_iter()
                            .filter(|entry| !entry.file_name().to_string_lossy().starts_with("."))
                            .collect()
                    }
                    Err(e) => {
                        return Err(e);
                    }
                };

                return Ok((target_path.clone(), vect_entries));
            }
            Err(e) => {
                return Err(e);
            }
        };
    })
}

// here just for one entry
// "%s %u %s %s %u %s %s\n", <file mode>, <number of links>,
//  <owner name>, <group name>, <number of bytes in the file>,
//  <date and time>, <pathname>

#[derive(Debug)]
enum FileTypeEnum {
    Regular,
    Directory,
    Symlink,
    CharDevice,
    BlockDevice,
    Socket,
    NamedPipe,
    Unknown,
}

#[derive(Debug)]
pub struct Entry {
    permissions: String,
    file_mode: u32,
    number_of_links: u32,
    onwer_name: String,
    group_name: String,
    size: u32,
    date: u64,
    entry_name: String,
    file_type: FileTypeEnum,
}

impl Entry {
    pub fn new(dir: &DirEntry) -> Self {}

    fn get_file_type(metadata: &Metadata) -> FileTypeEnum {
        match true {
            _ if metadata.file_type().is_file => FileTypeEnum::Regular,
            _ if metadata.file_type().is_dir => FileTypeEnum::Directory,
            _ if metadata.file_type().is_symlink => FileTypeEnum::Symlink,
            _ if metadata.file_type().is_block_device => FileTypeEnum::BlockDevice,
            _ if metadata.file_type().is_char_device => FileTypeEnum::CharDevice,
            _ if metadata.file_type().is_fifo => FileTypeEnum::NamedPipe,
            _ if metadata.file_type().is_socket => FileTypeEnum::Socket,
            _ => FileTypeEnum::Unknown,
        }
    }
}

pub fn print_long_format(dir_entry: &DirEntry) {
    if let Ok(metadata) = dir_entry.metadata() {
        println!("metadata permissions => {:?}", metadata.file_type())
    }
}

// pub fn get_and_format_permissions(dir_entry: &DirEntry)->  {

// }

pub fn append_file_type_indicator() {}
