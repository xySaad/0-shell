use std::os::unix::fs::{ FileTypeExt, MetadataExt, PermissionsExt };
use std::path::PathBuf;
use std::fs;
use colored::Colorize;

use super::{ entry::ColorStyle };


pub fn is_broken_link(path: &PathBuf) -> bool {
    match fs::metadata(path) {
        Ok(_) => false,
        Err(_) => true,
    }
}

pub fn apply_color(result: &str, style: ColorStyle) -> String {
    match style {
        ColorStyle::BoldGreen => result.green().bold().to_string(),
        ColorStyle::BlueBold => result.blue().bold().to_string(),
        ColorStyle::BoldYellow => result.yellow().bold().to_string(),
        ColorStyle::CyanBold => result.cyan().bold().to_string(),
        ColorStyle::RedBold => result.red().bold().to_string(),
        ColorStyle::BoldMagenta => result.magenta().bold().to_string(),
        ColorStyle::BrightWhite => result.bright_white().to_string(),
    }
}


