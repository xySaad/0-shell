use std::path::{ Path, PathBuf };
use std::io;
use std::io::{ ErrorKind };
use std::fs::{ self };
use std::cell::RefCell;

use super::{ entries::{ Entries }, entry::{ FileTypeEnum, Entry } };

#[derive(Debug, Eq, Clone, PartialEq)]
pub struct LsConfig {
    pub a_flag_set: bool,
    pub l_flag_set: bool,
    pub f_flag_set: bool,
    default_target_path: String,
    pub target_paths: Vec<String>,
    pub status_code: RefCell<i32>,
    flags: Vec<String>,
}

impl LsConfig {
    pub fn new(args: &Vec<String>) -> Self {
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
                Ok(_) => {}
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

    pub fn print_ls(&mut self) {
        self.parse();
        self.extract_valid_entries();

        for (target_path, resulted_entry) in read_target_path(self) {
            let is_directory = match Entry::new(&Path::new(&target_path).to_path_buf(), self) {
                Ok(valid_entry) => valid_entry.file_type == FileTypeEnum::Directory,
                Err(_) => false,
            };
            match resulted_entry {
                Ok(entries_vec) => {
                    let entries = Entries::new(&entries_vec, self);
                    if
                        (self.target_paths.len() != 1 || *self.status_code.borrow() != 0) &&
                        is_directory
                    {
                        println!("{}:", target_path);
                    }
                    if is_directory {
                        println!("total {}", entries.total);
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
                    paths.sort_by(|a, b| {
                        let binding_a = a
                            .clone()
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string();
                        let entry_a = binding_a.strip_prefix(".").unwrap_or(&binding_a);
                        let binding_b = b
                            .clone()
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string();
                        let entry_b = binding_b.strip_prefix(".").unwrap_or(&binding_b);
                        entry_a.cmp(&entry_b)
                    });

                    let mut paths = if ls_config.a_flag_set {
                        paths.insert(0, Path::new(".").to_path_buf());
                        paths.insert(1, Path::new("..").to_path_buf());
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
