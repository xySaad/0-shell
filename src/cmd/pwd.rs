use std::env;

pub fn pwd(args: &[String]) -> i32 {
    if args.len() != 0 {
        println!("pwd: too many arguments");
        return  1;
    }
    // should handle path when removed /home/amellagu/.local/share/Trash/files/test !!
    match env::var("PWD") {
        Ok(p) => println!("{}", p.to_string()),
        Err(e) => println!("No previous directory found (PWD not set): {}", e)
    };

    0
}