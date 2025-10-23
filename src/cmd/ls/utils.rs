use std::os::unix::fs::{ FileTypeExt, MetadataExt, PermissionsExt };
use std::path::PathBuf;
use std::fs;
use colored::Colorize;

use super::{ entry::ColorStyle };
// this takes the symlink and returns the format that follows it in the case of -l flag
// pub fn handle_symlink(path: &PathBuf, ls_config: &LsConfig) -> String {
//     // we use the metadata because it follows the original source of the link
//     let pointed_to = if let Ok(pointed_to) = path.read_link() {
//         //pointed_to.to_string_lossy().to_string()
//         if is_broken_link(path) {
//             return format!("{}", pointed_to.to_string_lossy().to_string().red().bold());
//         }
//         let metatada = fs::metadata(path).unwrap();
//         let source_path = Entry::new(path, ls_config).unwrap().color_entry_name(
//             path,
//             metadata,
//             ls_config,
//             true
//         );

//         source_path
//     } else {
//         "".to_string()
//     };
//    ( " -> " + pointed_to).to_string()
// }

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


