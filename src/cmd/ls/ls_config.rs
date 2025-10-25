use std::path::{ Path, PathBuf };
use std::io;
use std::io::{ ErrorKind };
use std::fs::{ self };
use std::cell::RefCell;

use super::{ entries::{ Entries }, entry::{ FileType, Entry }, utils::{ is_broken_link } };

#[derive(Debug, Eq, Clone, PartialEq)]
pub struct LsConfig {
    pub a_flag_set: bool,
    pub l_flag_set: bool,
    pub f_flag_set: bool,
    pub target_paths: Vec<String>,
    pub target_files: Vec<String>,
    pub target_dirs: Vec<String>,
    pub status_code: RefCell<i32>,
    flags: Vec<String>,
    pub num_args: usize,
}

impl LsConfig {
    pub fn new(args: &Vec<String>) -> Self {
        let flags = args
            .iter()
            .filter(|a| a.starts_with('-'))
            .cloned() // hadiiii 7itash bla biha &String instead of owned Strings // and i don't want to consume l args
            .collect();
        let targets: Vec<String> = args
            .iter()
            .filter(|a| !a.starts_with('-'))
            .cloned()
            .collect();
        Self {
            a_flag_set: false,
            l_flag_set: false,
            f_flag_set: false,
            target_paths: targets.clone(),
            target_dirs: vec![],
            target_files: vec![],
            flags: flags,
            status_code: RefCell::new(0),
            num_args: targets.len(),
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
            self.target_paths.push(".".to_string());
        }
    }

    // if any error is encountered the function must return the status code
    fn extract_valid_entries(&mut self) {
        self.target_paths.iter().for_each(|target_path| {
            let path = Path::new(&target_path);
            match fs::symlink_metadata(&path) {
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
        // exists won't work here because it's part of metadata
        self.target_paths.retain(|target_path| { fs::symlink_metadata(target_path).is_ok() });

        // filter out the dirs and the files and then sort them alphabeticallly
        self.target_dirs = self.target_paths
            .iter()
            .filter(|target_path| {
                (fs::symlink_metadata(target_path).unwrap().is_dir() &&
                    !fs::symlink_metadata(target_path).unwrap().is_symlink()) ||
                    (fs::symlink_metadata(target_path).unwrap().is_symlink() &&
                        !is_broken_link(target_path.to_string()) &&
                        !self.l_flag_set &&
                        !self.f_flag_set)
            })
            .cloned()
            .collect();
        self.target_files = self.target_paths
            .iter()
            .filter(|target_path| {
                fs::symlink_metadata(target_path).unwrap().is_file() ||
                    (fs::symlink_metadata(target_path).unwrap().is_symlink() &&
                        (self.l_flag_set || self.f_flag_set)) || is_broken_link(target_path.to_string())
            })
            .cloned()
            .collect();
        self.target_dirs.sort_by(|a, b| a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()));
        self.target_files.sort_by(|a, b| a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()));
    }

    pub fn process_files(&self) {
        let files = self.target_files
            .iter()
            .map(|target_path| Path::new(target_path).to_path_buf())
            .collect();
        let target_path = "".to_string();

        let entries = Entries::new(&files, self, &target_path);
        println!("{}", entries);
        if self.target_dirs.len() != 0 {
            println!();
        }
    }

    pub fn print_ls(&mut self) {
        self.parse();
        self.extract_valid_entries();
        if self.target_files.len() != 0 {
            self.process_files();
        }
        let mut iter = process_dirs(self).into_iter().peekable();
        while let Some((target_path, resulted_entry)) = iter.peek() {
            let is_directory = match
                Entry::new(&Path::new(&target_path).to_path_buf(), self, &target_path)
            {
                Some(valid_entry) =>
                    match valid_entry.metadata {
                        Some(metadata) => Entry::get_entry_type(&metadata).0 == FileType::Directory,
                        None => valid_entry.get_pseudo_entry_type().0 == FileType::Directory,
                    }
                None => false,
            };
            //println!("{:?}", resulted_entry);
            match resulted_entry {
                Ok(entries_vec) => {
                    let entries = Entries::new(&entries_vec, self, &target_path);
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

            iter.next();
            if iter.peek().is_some() {
                println!();
            }
        }
    }
}

pub fn process_dirs(
    ls_config: &LsConfig
) -> impl Iterator<Item = (String, Result<Vec<PathBuf>, io::Error>)> {
    ls_config.target_dirs.iter().map(|target_path| {
        let path = Path::new(target_path);
        // if
        //     (path.is_symlink() && !ls_config.l_flag_set && !ls_config.f_flag_set) ||
        //     (path.is_dir() && !path.is_symlink())
        // {
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
                    let binding_a = a.clone().file_name().unwrap().to_string_lossy().to_string();
                    let entry_a = binding_a.strip_prefix(".").unwrap_or(&binding_a);
                    let binding_b = b.clone().file_name().unwrap().to_string_lossy().to_string();

                    let entry_b = binding_b.strip_prefix(".").unwrap_or(&binding_b);
                    entry_a.to_lowercase().cmp(&entry_b.to_lowercase())
                });

                let paths = if ls_config.a_flag_set {
                    paths.insert(0, Path::new(target_path).join(".").to_path_buf());
                    paths.insert(1, Path::new(target_path).join("..").to_path_buf());
                    paths
                } else {
                    paths
                        .clone()
                        .into_iter()
                        .filter(
                            |entry| !entry.file_name().unwrap().to_string_lossy().starts_with(".")
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
        // } else {
        //     return (target_path.clone(), Ok(vec![Path::new(target_path).to_path_buf()]));
        // }
    })
}

// files are those who are symlink too but without -l and -F

// dirs are also symlinks with defalt ls or ls -a
// pub fn process_dirs() {}
