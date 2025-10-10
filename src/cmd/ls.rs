use std::{ fs::{ self, DirEntry, Metadata }, os::unix::fs::MetadataExt };
use std::env;

// here we will add everything till finding the need to add another file

// basic ls :) to test things out

//  we need to parse the args too
// we need to check if the args passed are either valid flags or either a valid file or folder

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

#[derive(Debug, Eq, Clone, PartialEq)]
pub struct LsConfig {
    a_flag_set: bool,
    l_falg_set: bool,
    f_flag_set: bool,
    target_paths: Vec<String>,
    flags: Vec<String>,
}

impl LsConfig {
    fn new(args: &Vec<String>) -> Self {
        Self {
            a_flag_set: false,
            l_falg_set: false,
            f_flag_set: false,
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
    fn parse(&mut self) -> i32{
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
                        return 1
                    }
                }
            }
        }
        return 0
    }
}

pub fn ls(args: &Vec<String>) -> i32 {
    println!("args {:?}: ", args);

    let directory_path = match env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Failed to get current directory: {}", e);
            std::process::exit(1);
        }
    };

    let mut ls_config = LsConfig::new(args);
    println!("before {:?}", ls_config);
    ls_config.parse();
    println!("after {:?}", ls_config);

    // gives the current directory where the program is run (which is not what i want )
    // no idea sara7aa

    return 0;
}

pub fn print_long_format() {}
pub fn include_hidden_files() {}

pub fn append_file_type_indicator() {}
