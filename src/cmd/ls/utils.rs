use colored::Colorize;
use std::path::{ Path };
use std::ffi::{ OsStr, c_char };
use std::{ fs, ptr };
use libc::{ llistxattr };

use super::{ entry::{ColorStyle, FileType}, ls_config::LsConfig };

pub fn apply_color(result: &str, style: ColorStyle) -> String {
    match style {
        ColorStyle::BoldGreen => result.green().bold().to_string(),
        ColorStyle::BlueBold => result.blue().bold().to_string(),
        ColorStyle::BoldYellow => result.yellow().bold().to_string(),
        ColorStyle::CyanBold => result.cyan().bold().to_string(),
        ColorStyle::BoldRed => result.red().bold().to_string(),
        ColorStyle::BoldMagenta => result.magenta().bold().to_string(),
        ColorStyle::BrightWhite => result.bright_white().to_string(),
    }
}

pub fn get_column_len(matrix: &Vec<Vec<String>>) -> Vec<usize> {
    let mut vec_max = Vec::new();
    // println!("{:?}", self.entries);
    if matrix.len() != 0 {
        for i in 0..matrix[0].len() {
            let mut max = matrix[0][i].len();

            for row in matrix {
                if max < row[i].len() {
                    max = row[i].len();
                }
            }
            vec_max.push(max);
        }
    }

    vec_max
}

pub fn is_broken_link(target_path: &Path) -> bool {
    match fs::metadata(target_path) {
        Ok(_) => false,
        Err(_) => true,
    }
}

// follow the symlink to see if the target is a directory
pub fn is_dir(target_path: &Path) -> bool {
    match fs::metadata(target_path) {
        Ok(metadata) => metadata.is_dir(),
        Err(_) => false,
    }
}

// (directory and not a symlink ) or (symlink and -l and -F are false )
pub fn is_directory(target_path: String, ls_config: &LsConfig) -> bool {
    // unwrap is used safely here because the check of valid paths are done before
    let path = Path::new(&target_path);
    let sym_metatada = fs::symlink_metadata(path).unwrap();
    if sym_metatada.is_dir() && !sym_metatada.is_symlink() {
        return true;
    }
    sym_metatada.is_symlink() &&
        is_dir(path) &&
        !is_broken_link(path) &&
        !ls_config.l_flag_set &&
        !ls_config.f_flag_set
}

pub fn is_file(target_path: String, ls_config: &LsConfig) -> bool {
    let path = Path::new(&target_path);
    let sym_metatada = fs::symlink_metadata(path).unwrap();
    if sym_metatada.is_symlink() {
        if ls_config.l_flag_set || ls_config.f_flag_set || is_broken_link(path) || !is_dir(path) {
            return true;
        }
    }

    sym_metatada.is_file() || !is_dir(path)
}

// fn to sort the files and the folders
pub fn sort_entries(files: &mut Vec<String>) {
    files.sort_by(|a, b| {
        let cleaned_a: String = a
            .chars()
            .filter(|c| !c.is_ascii_punctuation())
            .collect();

        let cleaned_b: String = b
            .chars()
            .filter(|c| !c.is_ascii_punctuation())
            .collect();

        cleaned_a.to_ascii_lowercase().cmp(&cleaned_b.to_ascii_lowercase())
    });
}

// here to convert anything that implements the AsRef trait to String
pub fn to_str<T: AsRef<OsStr>>(path: T) -> String {
    path.as_ref().to_string_lossy().into_owned()
}

// method to check the acl from the extra attributes
// we need to work with llistxattr
pub fn has_acl(path: &Path , entry_type: FileType) -> bool {
    let acl = match xattr::get(path, "system.posix_acl_access") {
        Ok(option) => {
            match option {
                Some(_)=> {return true; }
                None=> {return false; }
            }
        },
        Err(_) => false,
    };
    let default_acl = match xattr::get(path, "system.posix_acl_default") {
        Ok(option) => {
            match option {
                Some(_)=> {return true; }
                None=> {return false; }
            }
        },
        Err(_) => false,
    };
 
    if entry_type== FileType::Directory {
        return acl || default_acl; 
    }
    acl 
}
