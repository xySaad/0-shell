
use colored::Colorize;

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

