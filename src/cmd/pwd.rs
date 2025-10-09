use std::env;

pub fn pwd(args: &[String]) -> Result<String, String> {
    if args.len() != 0 {
        return  Ok(String::from("pwd: too many arguments"));
    }
    // should handle path when removed /home/amellagu/.local/share/Trash/files/test !!
    let current_path = match env::var("PWD") {
        Ok(p) => p.to_string(),
        Err(e) => format!("No previous directory found (PWD not set): {}", e)
    };
    // let old_path = match env::var("OLDPWD") {
    //     Ok(p) => p.to_string(),
    //     Err(e) => format!("No previous directory found (OLDPWD not set): {}", e)
    // };
    // Ok(format!("Current: {}, Old_pwd: {}", current_path, old_path))
    Ok(String::from(current_path))
}