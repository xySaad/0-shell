use crate::cli::Shell;

pub fn pwd(args: &[String], shell: &Shell) -> i32 {
    if args.len() != 0 {
        println!("pwd: too many arguments");
        return  1;
    }
    // should handle path when removed /home/amellagu/.local/share/Trash/files/test !!
    match shell.env.get("PWD") {
        Some(value) => println!("{}", value),
        None => println!("No previous directory found (PWD not set yet)")
    };

    // println!("{:?}", shell.env.get("PWD").cloned().unwrap_or_default()); // unwrap_or_else(|_| String::from("/")

    0
}