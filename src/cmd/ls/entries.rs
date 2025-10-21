use std::path::{ PathBuf };
use std::fmt;
use super::{ entry::{ Entry }, ls_config::{ LsConfig } };

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
            let to_entry = Entry::new(path, ls_config);
            match to_entry {
                Ok(mut valid_entry) => {
                    entries.push(valid_entry.as_array());
                    total += valid_entry.num_blocks;
                }
                Err(invalid_entry) => {
                    eprintln!("Error : {}", invalid_entry);
                }
            }
        }
        Self { entries: entries, total: total / 2 }
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
        // eprintln!(" hnaa: {:?}", vec_max);

        // we need to find the max for each field
        for j in 0..self.entries.len() {
            for k in 0..self.entries[j].len() {
                let value = vec_max[k];
                // case of numbers to (from the right)
                if k == 1 || k == 4 || k == 5 {
                    let formatted = format!("{0:>1$}", self.entries[j][k], value);
                    write!(f, "{}", formatted)?;
                    // avoid the space between minor and major
                    if k == 1 || k == 5 {
                        write!(f, " ")?;
                    }
                    // from the left
                } else {
                    let formatted = format!("{0:<1$}", self.entries[j][k], value);
                    write!(f, "{} ", formatted)?;
                }
            }
            if j != self.entries.len() - 1 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}
