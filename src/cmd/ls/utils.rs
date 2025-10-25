use colored::Colorize;
use std::path::Path;
use std::fs;
use super::{ entry::ColorStyle };

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
    for i in 0..matrix[0].len() {
        let mut max = matrix[0][i].len();

        for row in matrix {
            if max < row[i].len() {
                max = row[i].len();
            }
        }
        vec_max.push(max);
    }
    vec_max
}

pub fn is_broken_link(target_path: String) -> bool {
    let path = Path::new(&target_path);
    match fs::metadata(path) {
        Ok(_) => false,

        Err(_) => true,
    }
}
