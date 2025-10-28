use std::env;

pub fn pwd(args: &[String]) -> i32 {
    if args.len() != 0 {
        println!("pwd: too many arguments");
        return  1;
    }

    match env::var("PWD") {
        Ok(value) => println!("{}", value),
        Err(error) => {
            let res = match env::current_dir() {
                Ok(val) => val.display().to_string(),
                Err(err) => format!("0-shell: cd: {error}, {err}")
            };
            println!("{}", res)
        }
    };

    0
}