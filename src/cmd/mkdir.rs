use std::fs;
use std::path::Path;
/// cp utility according to https://pubs.opengroup.org/onlinepubs/9699919799/utilities/cp.html
pub fn mkdir(args: &[String]) -> Result<String, String> {
    println!("{:?}", args);

    for name in args.into_iter() {
        if !Path::new(name).exists() {
            match fs::create_dir(name) {
                Ok(_) => println!("folder created, {}", name),
                Err(e) => println!("cannot create the folder, Error: {}", e)
            };
        } else {
            println!("folder already exist {}", name);
        }
    }

    Ok("OK".to_string())
}
