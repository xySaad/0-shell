use super::{ entry::Entry, ls_config::LsConfig };
use std::os::linux::fs::MetadataExt;
use std::path::PathBuf;
use std::{ fmt };
// seems a good idea
#[derive(Debug, Clone)]
pub struct Entries {
    pub entries: Vec<Vec<String>>,
    pub total: u64,
}

impl Entries {
    pub fn new(paths: &Vec<PathBuf>, ls_config: &LsConfig, target_entry: &String) -> Self {
        let mut entries = Vec::new();
        let mut total = 0;
        for path in paths {
            // as we have access to the ls_config we can mutate the value of the status code
            match Entry::new(path, ls_config, target_entry) {
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
                    // write!(f, " ")?;
                    // from the left
                } else {
                    let formatted = format!("{0:<1$} ", self.entries[j][k], value);
                    line.push_str(&formatted);
                }
            }
            write!(f, "{}", line.trim())?;
            if j != self.entries.len() - 1 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}
