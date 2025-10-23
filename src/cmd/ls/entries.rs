use super::{ entry::Entry, ls_config::LsConfig };
use std::io::ErrorKind;
use std::os::linux::fs::MetadataExt;
use std::path::PathBuf;
use std::{ fmt, fs };
// seems a good idea
#[derive(Debug, Clone)]
pub struct Entries {
    pub entries: Vec<Vec<String>>,
    pub total: u64,
}

impl Entries {
    pub fn new(paths: &Vec<PathBuf>, ls_config: &LsConfig) -> Self {
        let mut entries = Vec::new();
        let mut total = 0;
        for path in paths {
            // as we have access to the ls_config we can mutate the value of the status code
            match Entry::new(path, ls_config) {
                Some(mut valid_entry) => {
                    entries.push(valid_entry.as_array());
                    total += valid_entry.metadata.st_blocks();
                }
                None => {
                    match fs::symlink_metadata(path) {
                        Ok(_) => {}
                        Err(e) => {
                            if e.kind() == ErrorKind::NotFound {
                                eprintln!(
                                    "ls: cannot access '{}': No such file or directory",
                                    path.to_string_lossy()
                                );
                            } else if e.kind() == ErrorKind::PermissionDenied {
                                eprintln!(
                                    "ls: cannot access '{}': Permission denied",
                                    path.to_string_lossy()
                                );
                                let status_code = ls_config.status_code.borrow();
                                if *status_code != 2 {
                                    *ls_config.status_code.borrow_mut() = 1;
                                }
                            } else {
                                 let status_code = ls_config.status_code.borrow();
                                if *status_code != 2 {
                                    *ls_config.status_code.borrow_mut() = 1;
                                }
                                eprintln!("{}", e);
                            }
                        }
                    };
                }
            }
        }
        Self {
            entries: entries,
            total: total / 2,
        }
    }
}

// don't know if it will work
// i will need the ls_config
impl fmt::Display for Entries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //  iterate through the columns to find the max
        let mut vec_max = Vec::new();
        // println!("{:?}", self.entries);
        for i in 0..self.entries[0].len() {
            let mut max = self.entries[0][i].len();

            for row in &self.entries {
                if max < row[i].len() {
                    max = row[i].len();
                }
            }
            vec_max.push(max);
        }
        // eprintln!(" hnaa: {:?}", vec_max);

        // we need to find the max for each field
        for j in 0..self.entries.len() {
            //eprintln!("entries : {:?}", self.entries);
            for k in 0..self.entries[j].len() {
                let value = vec_max[k];
                // case of numbers to (from the right)
                if k == 1 || k == 4 || k == 5 {
                    // eprintln!("hunaaa k : {}", k );
                    let formatted = format!("{0:>1$}", self.entries[j][k], value);
                    write!(f, "{formatted} ")?;
                    // write!(f, " ")?;
                    // from the left
                } else if k == self.entries[j].len() - 1 {
                    write!(f, "{}", self.entries[j][k].trim())?;
                } else {
                    let formatted = format!("{0:<1$}", self.entries[j][k], value);
                    write!(f, "{}", formatted)?;
                    if self.entries[j][4] == "" && k == 3 {
                        continue;
                    }
                    write!(f, " ")?;
                }
            }
            if j != self.entries.len() - 1 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}
