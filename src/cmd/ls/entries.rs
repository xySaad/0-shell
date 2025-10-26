use super::{ entry::Entry, ls_config::LsConfig };
use std::os::linux::fs::MetadataExt;
use std::path::PathBuf;
use std::{ fmt };
// seems a good idea

use super::utils::{ get_column_len };
#[derive(Debug, Clone)]
pub struct Entries {
    pub entries: Vec<Vec<String>>,
    pub total: u64,
    pub ls_config: LsConfig,
    pub target_entry: String,
}

impl Entries {
    pub fn new(paths: &Vec<PathBuf>, ls_config: &LsConfig, target_entry: &String) -> Self {
        let mut entries = Vec::new();
        let mut total = 0;
        for path in paths {
            // as we have access to the ls_config we can mutate the value of the status code
            // here we will need also to handle if it is a directory or just a file "" for or ...
            let group_name: String = if target_entry.is_empty() {
                path.to_string_lossy().into_owned()
            } else {
                target_entry.clone()
            };
            match Entry::new(path, ls_config, &group_name) {
                Some(mut valid_entry) => {
                    entries.push(valid_entry.handle_entry());
                    if valid_entry.metadata.is_some() {
                        total += valid_entry.metadata.unwrap().st_blocks();
                    }
                }
                None => {}
            }
        }
        Self {
            entries: entries,
            total: total / 2,
            ls_config: ls_config.clone(),
            target_entry: target_entry.clone(),
        }
    }
}

impl fmt::Display for Entries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.entries.is_empty() && !self.ls_config.l_flag_set && self.ls_config.num_args == 1 {
            return Ok(());
        }

        // eprintln!(" hnaa: {:?}", vec_max);
        if self.target_entry != "" && self.ls_config.num_args > 1 {
            writeln!(f, "{}: ", self.target_entry)?;
        }

        let vec_max = get_column_len(&self.entries);
        if self.ls_config.l_flag_set && self.target_entry != "" {
            writeln!(f, "total {}", self.total)?;
        }

        if self.entries.is_empty() {
            return Ok(());
        }

        for j in 0..self.entries.len() {
            //eprintln!("entries : {:?}", self.entries);
            let mut line = String::new();
            for k in 0..self.entries[j].len() {
                let value = vec_max[k];
                // case of numbers to (from the right)
                if value == 0 {
                    continue;
                }
                if k == 1 || k == 4 || k == 5 {
                    // eprintln!("hunaaa k : {}", k );
                    let formatted = format!("{0:>1$}", self.entries[j][k], value);
                    line.push_str(&formatted);
                    // minor and major must not have space between each other .
                    if k == 1 || k == 5 {
                        line.push(' ');
                    }
                    // from the left
                } else {
                    let formatted = format!("{0:<1$} ", self.entries[j][k], value);
                    line.push_str(&formatted);
                }
            }
            writeln!(f, "{}", line.trim())?;
        }

        Ok(())
    }
}
