use std::env;
// cp utility according to https://pubs.opengroup.org/onlinepubs/9699919799/utilities/cp.html
pub fn pwd(args: &[String]) -> Result<String, String> {
    println!("{:?}", args);
    println!("{:?}", env::current_dir());
    let current_path = match env::current_dir() {
        Ok(p) =>  p.display().to_string(),
        Err(e) => format!("Error cannot get current dir: {}", e) 
    };
    let old_path = match env::var("OLDPWD") {
        Ok(p) => p.to_string(),
        Err(e) => format!("No previous directory found (OLDPWD not set): {}", e)
    };
    Ok(format!("Current: {}, Old_pwd: {}", current_path, old_path))
}

// Basic Usage
// bash# Display current directory
// pwd
// # Output: /home/username/projects

// # That's it! pwd is simple but essential
// Command Options
// bash# Display logical path (default, follows symlinks)
// pwd -L
// # Output: /home/username/mylink

// # Display physical path (resolve all symlinks)
// pwd -P
// # Output: /home/username/actual/real/path
// Practical Scenarios
// bash# Confirm your location before dangerous operations
// pwd
// rm -rf *  # Now you know where you're deleting from!

// # Check where you are after multiple cd commands
// cd ../../..
// pwd
// # Output: /home

// # Use in scripts to save current location
// current_dir=$(pwd)
// echo "Working in: $current_dir"

// # Store current directory and return to it later
// original=$(pwd)
// cd /tmp
// # ... do some work ...
// cd "$original"
// Combining with Other Commands
// bash# Copy current path to clipboard (macOS)
// pwd | pbcopy

// # Copy current path to clipboard (Linux with xclip)
// pwd | xclip -selection clipboard

// # Use current directory in commands
// cp file.txt $(pwd)/backup/

// # Show current directory in a variable
// echo "You are currently in: $(pwd)"

// # Create a file with the current path
// pwd > current_location.txt
// In Scripts
// bash#!/bin/bash
// # Save and restore working directory
// SCRIPT_DIR=$(pwd)
// echo "Script started in: $SCRIPT_DIR"

// cd /tmp
// echo "Now in: $(pwd)"

// cd "$SCRIPT_DIR"
// echo "Back to: $(pwd)"
// Real-World Examples
// bash# Before compiling/building
// pwd
// # Ensure you're in the project root
// make

// # Documenting your location for support
// echo "Issue occurred in: $(pwd)"

// # Creating absolute paths from relative ones
// file="data.txt"
// fullpath="$(pwd)/$file"
// echo $fullpath
// # Output: /home/username/projects/data.txt

// # Checking if you're in the right directory
// if [[ $(pwd) == "/home/username/projects" ]]; then
//     echo "Correct directory!"
// else
//     echo "Wrong directory!"
// fi
// Understanding Symlinks
// bash# Create a symbolic link
// ln -s /var/www/html ~/webroot

// # Go to the symlink
// cd ~/webroot

// # Logical path (shows symlink)
// pwd -L
// # Output: /home/username/webroot

// # Physical path (shows real location)
// pwd -P
// # Output: /var/www/html
// Quick Tips
// bash# pwd has no arguments for files/directories
// # It only shows where YOU are currently

// # Common workflow
// cd ~/projects/website
// pwd
// # Output: /home/username/projects/website

// # Use with ls to understand context
// pwd && ls
// # Shows where you are AND what's there

// # In deep directory structures
// pwd
// # Output: /home/user/work/clients/acme/projects/2024/website/src/components/forms
// Integration with Shell Prompt
// Many shells show pwd in the prompt:
// bash# Your prompt might look like:
// username@hostname:/home/username/projects$

// # Or just:
// ~/projects$

// # pwd confirms this
// pwd
// # Output: /home/username/projects
// Common Use Cases Summary

// Orientation - Know where you are in the filesystem
// Safety - Verify location before destructive commands
// Scripting - Capture current directory for later use
// Path generation - Create absolute paths from relative ones
// Debugging - Troubleshoot path-related issues
// Documentation - Record working directories in logs

// Pro tip: Most shells display the current directory in the prompt, but pwd is useful for scripts, copying paths,
//  or when you need the absolute path!



///////////////////////////////

// Here are common cases when pwd doesn't work and how to fix them:
// Current Directory Deleted
// bash# Directory was deleted while you're in it
// cd /tmp/mydir
// # In another terminal: rm -rf /tmp/mydir
// pwd
// # Error: pwd: error retrieving current directory: getcwd: cannot access parent directories: No such file or directory

// # Fix: Change to a valid directory
// cd ~
// pwd
// # Now works normally

// # Or use physical path
// pwd -P
// # May still show the old path until you cd elsewhere
// Broken Symbolic Links in Path
// bash# Current path contains broken symlinks
// cd /path/with/broken/symlink
// pwd -P
// # Error: pwd: error retrieving current directory: getcwd: cannot access parent directories: No such file or directory

// # Logical path may still work
// pwd -L
// # Output: /path/with/broken/symlink

// # Fix: Navigate to a valid path
// cd $(pwd -L)/..
// cd "$(pwd -L | sed 's|/[^/]*$||')"
// Permission Issues on Parent Directories
// bash# No execute permission on parent directory
// chmod -x /tmp/testdir
// cd /tmp/testdir/subdir
// pwd
// # Error: pwd: error retrieving current directory: getcwd: cannot access parent directories: Permission denied

// # Check permissions
// ls -ld /tmp/testdir
// # Output: drw-r--r-- 2 user user 4096 Jan 1 12:00 /tmp/testdir

// # Fix: Restore execute permission
// chmod +x /tmp/testdir
// pwd
// # Now works
// Network/NFS Mount Issues
// bash# Stale NFS handle
// cd /mnt/nfs/somedir
// # Network issue occurs
// pwd
// # Error: pwd: error retrieving current directory: getcwd: cannot access parent directories: Stale file handle

// # Fix: Remount or navigate away
// cd ~
// sudo umount -l /mnt/nfs
// sudo mount -a
// cd /mnt/nfs/somedir
// Shell Not Loaded Properly
// bash# pwd command not in PATH
// pwd
// # Error: bash: pwd: command not found

// # Check PATH
// echo $PATH

// # Fix: Use builtin
// builtin pwd

// # Or use full path
// /bin/pwd

// # Or fix PATH
// export PATH=/bin:/usr/bin:$PATH
// pwd Aliased or Overridden
// bash# pwd replaced with broken alias/function
// alias pwd='echo "pwd disabled"'
// pwd
// # Output: pwd disabled

// # Fix: Use builtin
// builtin pwd
// # Or
// \pwd

// # Remove alias
// unalias pwd

// # Check for functions
// type pwd
// # Fix: Unset function if exists
// unset -f pwd
// Extremely Deep Directory Nesting
// bash# Path exceeds system limits (very rare)
// cd /very/deep/path/level1/level2/.../level1000/
// pwd
// # Error: pwd: error retrieving current directory: getcwd: File name too long

// # Fix: Navigate to shallower directory
// cd /
// pwd
// Corrupted Filesystem
// bash# Filesystem corruption
// pwd
// # Error: pwd: error retrieving current directory: getcwd: Input/output error

// # Check system logs
// dmesg | tail

// # Fix: Filesystem check (requires unmounting)
// cd ~
// sudo fsck /dev/sda1

// # Or remount
// sudo mount -o remount /
// PWD Environment Variable Corruption
// bash# PWD variable manually corrupted
// PWD="/fake/path"
// pwd
// # May show incorrect path in some shells

// # Check variable
// echo $PWD
// # Output: /fake/path

// # Fix: Unset and let shell reset it
// unset PWD
// cd .
// pwd
// # Now shows correct path

// # Or use physical path
// /bin/pwd
// Restricted Shell Environment
// bash# In restricted shell (rbash)
// pwd
// # May work but with limitations

// # Check shell type
// echo $SHELL
// echo $0

// # Fix: Exit restricted shell if allowed
// bash --norc
// pwd
// Process Working Directory Issues
// bash# In a subshell or script where cwd is unclear
// (cd /tmp && pwd)  # Works in subshell
// pwd  # Different in parent shell

// # Fix: Don't rely on subshell's pwd
// # Or capture it explicitly
// SUBDIR=$(cd /tmp && pwd)
// echo $SUBDIR
// SELinux Context Issues (Rare)
// bash# SELinux preventing directory access
// pwd
// # Error: pwd: error retrieving current directory: getcwd: Permission denied

// # Check SELinux status
// getenforce

// # Check context
// ls -Zd .

// # Fix: Adjust context or navigate away
// cd ~
// Terminal/TTY Issues
// bash# Working in weird terminal state
// pwd
// # No output or garbled output

// # Check if stdout is working
// echo "test"

// # Fix: Reset terminal
// reset
// pwd

// # Or redirect to see errors
// pwd 2>&1
// Directory Moved While Inside It
// bash# Directory renamed/moved while you're in it
// cd /tmp/oldname
// # In another terminal: mv /tmp/oldname /tmp/newname
// pwd
// # Output: /tmp/oldname (but path no longer exists)

// # Fix: Change to valid directory
// cd /tmp/newname
// pwd
// # Or
// cd ~
// cd /tmp/newname
// Disk Full Issues
// bash# Disk full preventing directory operations
// pwd
// # Error: pwd: error retrieving current directory: getcwd: No space left on device

// # Check disk space
// df -h

// # Fix: Free up space
// rm /tmp/largefile
// pwd
// Binary/Encoding Issues
// bash# Directory name with non-UTF8 characters
// cd $'\xff\xfe'directory
// pwd
// # May show garbled output

// # Fix: Use LC_ALL
// LC_ALL=C pwd

// # Or escape properly
// pwd | cat -v
// Debugging pwd Issues
// bash# Check if pwd is builtin or external
// type pwd
// # Output: pwd is a shell builtin

// # Use external pwd command
// /bin/pwd
// command pwd
// $(which pwd)

// # Check PWD environment variable
// echo $PWD
// echo $OLDPWD

// # Verify current directory exists
// ls -ld .
// # Or
// stat .

// # Check parent accessibility
// ls -ld ..

// # Test with different options
// pwd -L  # Logical path
// pwd -P  # Physical path

// # Check inode of current directory
// ls -lid .

// # See all directory stack
// dirs -v

// # Enable debugging
// set -x
// pwd
// set +x
// Workarounds When pwd Fails
// bash# Use /proc filesystem (Linux)
// readlink /proc/$$/cwd
// cat /proc/$$/environ | tr '\0' '\n' | grep PWD

// # Use PWD variable (if not corrupted)
// echo $PWD

// # Use external command
// /bin/pwd

// # Get directory from running ls
// ls -id . && find / -inum $(ls -id . | awk '{print $1}') 2>/dev/null

// # Navigate to known location
// cd ~
// cd -  # Return to previous (if it exists)
// Prevention Tips
// bash# Always check if directory exists before operations
// [ -d "$(pwd)" ] || cd ~

// # In scripts, verify pwd success
// current_dir=$(pwd) || exit 1

// # Don't delete directories you're working in
// cd ..  # Before deleting subdirectory

// # Keep backups of directory locations
// SAVED_DIR=$(pwd)
// # ... do work ...
// cd "$SAVED_DIR"

// # Use pushd/popd for directory stack
// pushd /tmp
// # ... work ...
// popd  # Returns to original directory

// # In scripts, use explicit paths
// cd /absolute/path || exit 1
// Script Example with Error Handling
// bash#!/bin/bash
// # Robust pwd usage

// # Check if we can get current directory
// if ! current_dir=$(pwd); then
//     echo "Error: Cannot determine current directory" >&2
//     # Try to recover
//     cd ~ 2>/dev/null || cd / || exit 1
//     current_dir=$(pwd)
// fi

// echo "Working directory: $current_dir"

// # Verify directory exists
// if [ ! -d "$current_dir" ]; then
//     echo "Warning: Current directory no longer exists" >&2
//     cd ~ || exit 1
// fi

// # Use physical path to resolve symlinks
// physical_dir=$(pwd -P)
// echo "Physical path: $physical_dir"
// Common Error Messages
// bash# "cannot access parent directories: No such file or directory"
// # → Directory was deleted while you're in it
// cd ~

// # "cannot access parent directories: Permission denied"  
// # → Parent directory lacks execute permission
// chmod +x /parent/dir

// # "Stale file handle"
// # → NFS/network issue
// sudo umount -l /mount/point

// # "command not found"
// # → pwd not in PATH
// /bin/pwd

// # "Input/output error"
// # → Filesystem corruption
// sudo fsck /dev/sda1
// Pro tip: pwd rarely fails, but when it does, it's usually because: 1) Your current directory was deleted, 2)
//  Parent directories have permission issues, or 3) Network/NFS problems. Just cd ~ to recover and start fresh!