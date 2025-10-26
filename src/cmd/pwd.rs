use std::env;

pub fn pwd(args: &[String]) -> i32 {
    if args.len() != 0 {
        println!("pwd: too many arguments");
        return  1;
    }

    match env::var("PWD") {
        Ok(value) => println!("{}", value),
        Err(e) => eprintln!("0-shell: cd: PWD not set, {}", e)
    };

    0
}