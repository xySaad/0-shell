use std::{env, fs};
use std::path::{Path, PathBuf};

// to do //

// should handle error output by message error !!

pub fn cd(args: &[String]) -> Result<String, String> {
    println!("arguments: {:?}", args);
    println!("befor: {:?}", env::current_dir());

    // let current_path = env::current_dir()
    //     .map(|p| p.display().to_string())
    //     .unwrap_or_else(|e| format!("Error cannot get current dir: {}", e));

    let old_path = env::var("OLDPWD").unwrap_or_else(|_| String::new());
    let current_path = env::var("PWD").unwrap_or_else(|_| String::new());

    let res: String = match args.len() {
        0 => match env::var("HOME") {
            Ok(p) => {
                match env::set_current_dir(Path::new(&p)) {
                    Ok(_) => {
                        unsafe { env::set_var("OLDPWD", current_path) };
                        // Get canonical path after changing dir
                        let new_pwd = env::current_dir()
                            .unwrap_or_else(|_| Path::new(&p).to_path_buf())
                            .display()
                            .to_string();
                        unsafe { env::set_var("PWD", new_pwd) };
                        "go to the main rout HOME".to_string()
                    }
                    Err(e) => format!("error: {}", e),
                }
            }
            Err(e) => format!("error: {}", e),
        },
        1 => {
            // let mut new_path: &str = "";
            match args[0].clone() {
                // case cd - should print the path where go !!
                value if value == "-" => match env::set_current_dir(Path::new(&old_path)) {
                    Ok(_) => {
                        unsafe { env::set_var("OLDPWD", current_path) };
                        let new_pwd = env::current_dir()
                            .unwrap_or_else(|_| Path::new(&old_path).to_path_buf())
                            .display()
                            .to_string();
                        unsafe { env::set_var("PWD", new_pwd) };
                        "switch to the previeus path".to_string()
                    }
                    Err(e) => format!("error: {}", e),
                },
                // value if value == "~" => match env::var("HOME") {
                //     Ok(p) => {
                //         match env::set_current_dir(Path::new(&p)) {
                //             Ok(_) => {
                //                 unsafe { env::set_var("OLDPWD", current_path) };
                //                 unsafe { env::set_var("PWD", p) };
                //                 "go to the main rout HOME".to_string()
                //             },
                //             Err(e) => format!("error: {}", e)
                //         }
                //     },
                //     Err(e) => format!("error: {}", e)
                // },

                // ./../ or ../.././ case !!
                _ => {
                    let base = env::var("PWD").unwrap_or_else(|_| current_path.clone());
                    let new_path = Path::new(&base).join(Path::new(&args[0]));
                    match env::set_current_dir(
                        Path::new(&new_path),
                    ) {
                        Ok(_) => {
                            unsafe { env::set_var("OLDPWD", current_path) };
                            let new_pwd = if Path::new(&args[0]).is_absolute() && !args[0].contains("..") {
                                args[0].clone()
                            } else {
                                // normalize relative paths, but don't resolve symlinks
                                Path::new(&base)
                                    .join(&args[0])
                                    .components()
                                    .collect::<PathBuf>()
                                    .display()
                                    .to_string()
                            };
                            // if new_pwd.contains("..") || new_pwd.contains(".") {
                            //     new_pwd = std::fs::canonicalize(current_path)
                            //         .unwrap_or_else(|_| new_path.clone())
                            //         .display()
                            //         .to_string();
                            // }
                            unsafe { env::set_var("PWD", new_pwd) };
                            "go to new path".to_string()
                        }
                        Err(e) => format!("error: {}", e),
                    }
                }
            }
        }
        _ => "0-shell: cd: too many arguments".to_string(),
    };
    println!("after: {:?}", env::current_dir());
    Ok(res)

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
}

// Basic Navigation
// bash# Go to your home directory
// cd ~
// cd

// # Go to a specific directory
// cd /usr/local/bin

// # Go to Documents folder in your home directory
// cd ~/Documents

// # Move up one directory level
// cd ..

// # Move up two directory levels
// cd ../..

// # Go to the previous directory you were in
// cd -
// Relative Paths
// bash# Navigate to a subdirectory of current location
// cd projects/website

// # Go up one level, then into a different directory
// cd ../other-folder

// # Navigate using current directory notation
// cd ./subfolder
// Absolute Paths
// bash# Go to root directory
// cd /

// # Navigate to a system directory
// cd /etc

// # Go to another user's home (if you have permissions)
// cd /home/username
// Working with Spaces and Special Characters
// bash# Use quotes for directories with spaces
// cd "My Documents"
// cd 'Project Files'

// # Use backslash to escape spaces
// cd My\ Documents

// # Use tab completion to auto-escape
// cd My[TAB]  # Will auto-complete and escape spaces
// Practical Scenarios
// bash# Quickly switch between two directories
// cd /var/log    # First directory
// cd ~/projects  # Second directory
// cd -           # Back to /var/log
// cd -           # Back to ~/projects

// # Navigate to deeply nested directories
// cd ~/work/projects/2024/website/src/components

// # Return to home from anywhere
// cd

// Tips

// Use pwd after cd to confirm your current location
// Use ls after cd to see what's in the new directory
// Tab completion is your friend - type partial names and hit Tab
// cd with no arguments always takes you home

//////////////////////::

// Here are common cases when cd doesn't work and how to fix them:
// Permission Denied
// bash# Trying to access a directory without permission
// cd /root
// # Error: bash: cd: /root: Permission denied

// # Fix: Use sudo (if you have rights)
// sudo -i
// cd /root

// # Or check permissions
// ls -ld /root
// # Output: drwx------ 5 root root 4096 Jan 1 12:00 /root
// Directory Doesn't Exist
// bash# Typo or wrong path
// cd /home/user/Docments
// # Error: bash: cd: /home/user/Docments: No such file or directory

// # Fix: Check spelling and use tab completion
// cd /home/user/Doc[TAB]
// # Or verify the path exists
// ls /home/user/

// # Common mistake: forgetting the directory was deleted
// cd old_project
// # Error: No such file or directory
// Not a Directory
// bash# Trying to cd into a file
// cd file.txt
// # Error: bash: cd: file.txt: Not a directory

// # Fix: Check what it is
// file file.txt
// ls -l file.txt

// # Or use the parent directory
// cd $(dirname file.txt)
// Spaces in Directory Names
// bash# Unquoted spaces
// cd My Documents
// # Error: bash: cd: My: No such file or directory

// # Fix: Use quotes or escaping
// cd "My Documents"
// cd 'My Documents'
// cd My\ Documents
// Special Characters Issues
// bash# Directory with special characters
// cd folder$name
// # Shell tries to expand $name as a variable

// # Fix: Quote or escape
// cd 'folder$name'
// cd "folder\$name"
// cd folder\$name

// # Other problematic characters
// cd folder(1)    # Parentheses
// cd "folder(1)"  # Fixed

// cd folder&stuff    # Ampersand
// cd 'folder&stuff'  # Fixed
// Too Many Arguments
// bash# Multiple arguments without quotes
// cd My Important Files
// # Error: bash: cd: too many arguments

// # Fix: Quote the entire path
// cd "My Important Files"
// Broken Symbolic Links
// bash# Symlink points to non-existent location
// cd mylink
// # Error: bash: cd: mylink: No such file or directory

// # Check if it's a broken symlink
// ls -l mylink
// # Output: lrwxrwxrwx 1 user user 20 Jan 1 12:00 mylink -> /deleted/path

// # Fix: Remove and recreate the symlink
// rm mylink
// ln -s /correct/path mylink
// Path Too Long
// bash# Extremely long path exceeds system limits
// cd /very/long/path/that/goes/on/and/on/...
// # Error: bash: cd: /very/long/...: File name too long

// # Fix: Use shorter paths or intermediate cd commands
// cd /very/long/path
// cd that/goes/on
// Variable Expansion Issues
// bash# Variable not set or empty
// cd $MYDIR
// # If $MYDIR is empty, cd goes to home directory (unexpected)

// # Fix: Quote and check variable
// cd "$MYDIR"
// echo "MYDIR is: $MYDIR"

// # Better: Use default value
// cd "${MYDIR:-/default/path}"
// Directory is Actually a File
// bash# Hidden file that looks like a directory
// cd .config
// # Error: bash: cd: .config: Not a directory

// # Fix: Check what it actually is
// file .config
// ls -la .config
// Insufficient Disk Space
// bash# Rare: System issues preventing directory access
// cd /mnt/full_disk
// # Error: bash: cd: /mnt/full_disk: Input/output error

// # Fix: Check disk space and system health
// df -h
// dmesg | tail
// SELinux or Security Context Issues
// bash# SELinux blocking access
// cd /protected/dir
// # Error: bash: cd: /protected/dir: Permission denied

// # Check SELinux status
// ls -Z /protected/dir
// getenforce

// # Fix: Adjust SELinux context (advanced)
// # Or temporarily disable (not recommended for production)
// Network/NFS Mount Issues
// bash# Network drive not mounted or stale
// cd /mnt/network_share
// # Error: bash: cd: /mnt/network_share: Stale file handle

// # Fix: Remount the network share
// sudo umount -l /mnt/network_share
// sudo mount -a

// # Or check mount status
// mount | grep network_share
// Command Chaining Issues
// bash# cd in subshell doesn't affect parent
// (cd /tmp && pwd)
// pwd  # Still in original directory!

// # Fix: Don't use subshell
// cd /tmp && pwd

// # Or use separate commands
// cd /tmp
// pwd
// Alias or Function Interference
// bash# cd aliased to something else
// alias cd='echo "cd is disabled"'
// cd /tmp
// # Output: cd is disabled

// # Fix: Use builtin command
// builtin cd /tmp

// # Or unalias
// unalias cd
// Relative Path Confusion
// bash# Wrong starting point for relative path
// cd ../../../folder
// # Error: bash: cd: ../../../folder: No such file or directory

// # Fix: Use absolute path or verify current location
// pwd
// cd /absolute/path/to/folder

// # Or navigate step by step
// cd ..
// pwd
// cd ..
// pwd
// Case Sensitivity (Linux/Mac)
// bash# Wrong case on case-sensitive systems
// cd Documents
// # vs
// cd documents
// # Error: No such file or directory

// # Fix: Match exact case
// ls  # Check actual directory name
// cd Documents  # Use correct case
// Debugging Tips
// bash# Check if path exists
// ls -ld /path/to/directory

// # Check permissions
// ls -ld /path/to/directory
// namei -l /path/to/directory

// # Verify you're not in a restricted shell
// echo $SHELL
// echo $0

// # Check for hidden characters
// cat -A path.txt  # If reading from file

// # Test with absolute path
// cd /full/absolute/path

// # Enable debugging
// set -x
// cd problematic/path
// set +x

// # Check directory stack
// dirs -v
// Prevention Tips
// bash# Always use tab completion
// cd /home/use[TAB]

// # Quote paths with spaces
// cd "$MY_PATH"

// # Check before changing
// [ -d "/path" ] && cd "/path" || echo "Directory doesn't exist"

// # Use absolute paths in scripts
// cd /absolute/path/to/dir

// # Verify success in scripts
// cd /some/dir || exit 1
// Pro tip: When cd fails, check these in order: 1) Does it exist? (ls), 2) Can you access it? (ls -ld), 3) Is the spelling correct?
//  (tab completion), 4) Any special characters? (quote it!)RetryClaude does not have the ability to run the code it generates yet.
