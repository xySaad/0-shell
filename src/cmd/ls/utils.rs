use std::os::unix::fs::{ FileTypeExt, MetadataExt, PermissionsExt };
use std::path::PathBuf;
use std::fs;

pub fn is_broken_link(path: &PathBuf) -> bool {
    match fs::metadata(path) {
        Ok(_) => false,
        Err(_) => true,
    }
}





