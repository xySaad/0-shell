use std::env;
use std::path::{Path, PathBuf};

// should handle "////"
// // path = "~/home".to_string(); // handle in parser +++

pub fn cd(args: &[String]) -> i32 {
    println!("befor: {:?}", env::current_dir());

    if args.len() > 1 {
        println!("0-shell: cd: too many arguments");
        return 1;
    }

    let current_pwd = env::var("PWD").unwrap_or("/".to_string());

    let target = if args.is_empty() || args[0] == "--" {
        "~".to_string()
    } else {
        args[0].clone()
    };

    match target.as_str() {
        "~" => {
            let home  = env::var("HOME").unwrap_or(String::from("/"));

            match change_dir(&home, &current_pwd, "~") {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{}", e);
                    return 1;
                }
            };
        }

        "-" => {
            let oldpwd = env::var("OLDPWD").unwrap_or(String::new());

            if oldpwd.is_empty() {
                println!("0-shell: cd: OLDPWD not set");
            } else {
                match change_dir(&oldpwd, &current_pwd, "-") {
                    Ok(_) => println!("{}", oldpwd),
                    Err(e) => {
                        eprintln!("{}", e);
                        return 1;
                    }
                };
            }
        }

        ".." => {
            let mut logical = PathBuf::from(&current_pwd);
            logical.pop();
            let new_path = logical.display().to_string();

            match change_dir(&new_path, &current_pwd, "..") {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{}", e);
                    return 1;
                }
            };
        }

        other => {
            let pwd = Path::new(&current_pwd);
            let abs_path = if Path::new(other).is_absolute() {
                PathBuf::from(other)
            } else {
                pwd.join(other)
            };

            match change_dir(abs_path.to_str().unwrap(), &current_pwd, other) { // unwrap !!
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{}", e);
                    return 1;
                }
            };
        }
    };

    println!("after: {:?}", env::current_dir());
    0
}

fn change_dir(target: &str, oldpwd: &str, input: &str) -> Result<(), String> {
    let path = Path::new(target);
    if !path.exists() {
        return Err(format!("0-shell: cd: {}: No such file or directory", target));
    }

    match env::set_current_dir(path) {
        Ok(_) => (),
        Err(e) => {
            let mut msg = e.to_string();
            if let Some(idx) = msg.find(" (os error") {
                msg.truncate(idx);
            }
            return Err(format!("0-shell: cd: {}: {}", input, msg));
        }
    };

    unsafe {
        env::set_var("OLDPWD", oldpwd);
        env::set_var("PWD", &normalize_path(&PathBuf::from(target)));
    }

    Ok(())
}

fn normalize_path(path: &Path) -> String {
    match path.canonicalize() {
        Ok(p) => p.display().to_string(),
        Err(_) => path.display().to_string(),
    }
}

// fn normalize_path(path: &Path) -> String {
//     let mut parts: Vec<&str> = vec![];

//     for comp in path.components() {
//         match comp.as_os_str().to_str() {
//             Some(".") => continue,
//             Some("..") => { parts.pop(); }
//             Some(s) => parts.push(s),
//             None => continue,
//         }
//     }

//     let mut res = if path.is_absolute() {
//         format!("/{}", parts.join("/"))
//     } else {
//         parts.join("/")
//     };

//     let mut start = 0;
//     for (i, c) in res.clone().chars().enumerate() {
//         if c == '/' {
//             start = i;
//             continue;
//         }
//         break
//     }
//     if start > 1 {
//         res = res[start..].to_string();
//     }
//     res
// }


// // path = "".to_string(); // handle
// // path = "~".to_string(); // handle in parser +++
// // path = "/".to_string(); // simple path +++
// // path = "..".to_string(); // handle ////
// // path = ".".to_string(); // do nothing
// // path = "-".to_string(); // enverments variable ///////
// // path = "my home".to_string(); // simple path //
// // path = "home/src".to_string(); // simple path //
// // path = "file.txt".to_string();
// // path = ".home".to_string(); // hiden file like folder
// // path = "/root".to_string();
// // path = "/home".to_string();
// // path = "~/home".to_string(); // handle in parser +++
// // path = "./home".to_string();
// // path = "../home".to_string();
// // path = "../../home".to_string();