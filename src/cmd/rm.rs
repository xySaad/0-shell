// use std::fs;
// cp utility according to https://pubs.opengroup.org/onlinepubs/9699919799/utilities/cp.html
pub fn rm(args: &[String]) -> i32 {
    println!("{:?}", args);





    0
}


// Basic File Deletion
// bash# Delete a single file
// rm file.txt

// # Delete multiple files
// rm file1.txt file2.txt file3.txt

// # Delete files with wildcard patterns
// rm *.txt          # All .txt files in current directory
// rm file*          # All files starting with "file"
// rm *.log *.tmp    # All .log and .tmp files
// Directory Deletion
// bash# Delete an empty directory
// rmdir empty_folder

// # Delete a directory and all its contents (recursive)
// rm -r folder_name
// rm -rf folder_name   # Force deletion without prompts

// # Delete multiple directories
// rm -r dir1 dir2 dir3
// Safe Deletion (Interactive Mode)
// bash# Prompt before each deletion
// rm -i file.txt

// # Prompt before deleting more than 3 files or recursive
// rm -I *.txt

// # Prompt before removing write-protected files
// rm -i *.log
// Force Deletion
// bash# Force delete without prompts
// rm -f file.txt

// # Force delete directory and contents
// rm -rf old_project/

// # Delete write-protected files without confirmation
// rm -f protected_file.txt
// Verbose Mode
// bash# Show what's being deleted
// rm -v file.txt

// # Verbose recursive deletion
// rm -rv old_folder/

// # Combine verbose and interactive
// rm -iv *.txt
// Practical Scenarios
// bash# Clean up temporary files
// rm -f *.tmp *.cache

// # Remove all files except specific ones
// rm !(important.txt)  # Requires extglob option

// # Delete files older than a certain date (with find)
// find . -name "*.log" -mtime +30 -exec rm {} \;

// # Remove all files in current directory (dangerous!)
// rm *

// # Delete hidden files
// rm .*
// rm .hidden_file

// # Remove files with special characters
// rm "file with spaces.txt"
// rm 'file-name.txt'
// Common Combinations
// bash# Recursive, force, and verbose
// rm -rfv backup_folder/

// # Interactive recursive deletion
// rm -ri test_folder/

// # Force delete multiple file types
// rm -f *.bak *.swp *~
// Safety Tips and Warnings
// bash# ⚠️ EXTREMELY DANGEROUS - Never run these!
// # rm -rf /     # Deletes everything (most systems protect against this)
// # rm -rf /*    # Deletes all system files
// # rm -rf ~/*   # Deletes all your home directory files

// # Safer practices:
// # 1. Use -i flag when deleting important files
// rm -i important_file.txt

// # 2. Double-check with ls before rm
// ls *.txt
// rm *.txt

// # 3. Use trash/move to trash instead
// mv file.txt ~/.Trash/  # macOS
// Advanced Examples
// bash# Delete files but not directories
// find . -type f -name "*.txt" -delete

// # Remove empty files only
// find . -type f -empty -delete

// # Delete files with specific permissions
// find . -type f -perm 644 -name "*.txt" -delete

// # Combine with other commands
// ls | grep "old" | xargs rm -v
// Common Options Summary

// -r or -R: Recursive (for directories)
// -f: Force (no prompts, ignore nonexistent files)
// -i: Interactive (prompt before each deletion)
// -I: Prompt once before deleting multiple files
// -v: Verbose (show what's being deleted)
// -d: Remove empty directories

// Pro tip: Consider using trash-cli or similar tools instead of rm for a safer deletion with the ability to recover files!


/////////////////////////////// 


// Here are common cases when rm doesn't work and how to fix them:
// Permission Denied
// bash# No permission to delete the file
// rm file.txt
// # Error: rm: cannot remove 'file.txt': Permission denied

// # Check permissions
// ls -l file.txt
// # Output: -rw-r--r-- 1 root root 100 Jan 1 12:00 file.txt

// # Fix: Use sudo (if you have rights)
// sudo rm file.txt

// # Or change ownership first
// sudo chown yourusername file.txt
// rm file.txt
// Write-Protected Files
// bash# File is write-protected
// rm protected.txt
// # Prompt: rm: remove write-protected regular file 'protected.txt'?

// # Fix option 1: Answer 'y' to the prompt
// y

// # Fix option 2: Force deletion
// rm -f protected.txt

// # Fix option 3: Change permissions first
// chmod u+w protected.txt
// rm protected.txt
// Directory Not Empty
// bash# Trying to delete non-empty directory without -r
// rm folder
// # Error: rm: cannot remove 'folder': Is a directory

// # Or with rmdir
// rmdir folder
// # Error: rmdir: failed to remove 'folder': Directory not empty

// # Fix: Use recursive flag
// rm -r folder
// rm -rf folder  # Force version

// # Or empty it first
// rm folder/*
// rmdir folder
// File is Being Used
// bash# File is open by another process
// rm logfile.txt
// # Error: rm: cannot remove 'logfile.txt': Text file busy

// # Check what's using it
// lsof logfile.txt
// fuser logfile.txt

// # Fix: Close the program or kill the process
// kill <PID>
// # Then try again
// rm logfile.txt

// # Or force it (may cause issues)
// rm -f logfile.txt
// File Doesn't Exist
// bash# Typo or already deleted
// rm fiel.txt
// # Error: rm: cannot remove 'fiel.txt': No such file or directory

// # Fix: Check spelling or use wildcards carefully
// ls fie*
// rm file.txt

// # Ignore errors if file might not exist
// rm -f file.txt  # No error if file doesn't exist
// Special Characters in Filename
// bash# Files with spaces
// rm my file.txt
// # Error: rm: cannot remove 'my': No such file or directory

// # Fix: Quote the filename
// rm "my file.txt"
// rm 'my file.txt'
// rm my\ file.txt

// # Files starting with dash
// rm -file.txt
// # Error: rm: invalid option -- 'f'

// # Fix: Use -- or ./
// rm -- -file.txt
// rm ./-file.txt

// # Files with special characters
// rm file*.txt  # Might match unintended files
// rm 'file*.txt'  # Treats * literally
// Read-Only File System
// bash# Trying to delete on read-only filesystem
// rm /mnt/readonly/file.txt
// # Error: rm: cannot remove '/mnt/readonly/file.txt': Read-only file system

// # Check mount status
// mount | grep readonly

// # Fix: Remount as read-write
// sudo mount -o remount,rw /mnt/readonly
// rm /mnt/readonly/file.txt

// # Or check if it's a CD/DVD/USB that shouldn't be written to
// Immutable Files
// bash# File has immutable attribute
// rm important.txt
// # Error: rm: cannot remove 'important.txt': Operation not permitted

// # Check attributes
// lsattr important.txt
// # Output: ----i--------e-- important.txt

// # Fix: Remove immutable flag
// sudo chattr -i important.txt
// rm important.txt
// Too Many Arguments
// bash# Argument list too long with wildcards
// rm *.txt
// # Error: bash: /bin/rm: Argument list too long

// # Fix: Use find with -delete
// find . -name "*.txt" -type f -delete

// # Or use xargs
// find . -name "*.txt" -type f | xargs rm

// # Or loop through
// for file in *.txt; do rm "$file"; done
// Symbolic Link Issues
// bash# Removing symlink removes link, not target
// rm mylink
// # This works and removes only the link

// # Common mistake: Adding trailing slash
// rm mylink/
// # Error: rm: cannot remove 'mylink/': Not a directory

// # Fix: Don't add slash for symlinks
// rm mylink

// # To remove target and link
// rm -r mylink  # If mylink points to directory
// Filename with Newlines or Special Characters
// bash# Files with newlines in name (rare but possible)
// rm "file
// name.txt"
// # Error: Various issues

// # Fix: Use find with -print0 and xargs -0
// find . -name "*file*" -print0 | xargs -0 rm

// # Or use tab completion to auto-escape
// rm [TAB]

// # For really problematic names, use inode
// ls -li
// find . -inum <inode-number> -delete
// SELinux/AppArmor Restrictions
// bash# Security context preventing deletion
// rm /var/www/file.txt
// # Error: rm: cannot remove '/var/www/file.txt': Permission denied

// # Check SELinux context
// ls -Z file.txt

// # Fix: Adjust SELinux context or use sudo
// sudo rm file.txt

// # Or temporarily disable (not recommended)
// sudo setenforce 0
// NFS/Network Mount Issues
// bash# Stale NFS handle
// rm /mnt/nfs/file.txt
// # Error: rm: cannot remove '/mnt/nfs/file.txt': Stale file handle

// # Fix: Remount the NFS share
// sudo umount -l /mnt/nfs
// sudo mount -a

// # Or force unmount
// sudo umount -f /mnt/nfs
// Disk Full or I/O Errors
// bash# Disk errors preventing deletion
// rm file.txt
// # Error: rm: cannot remove 'file.txt': Input/output error

// # Check disk health
// dmesg | tail
// sudo smartctl -a /dev/sda

// # Check filesystem
// df -h
// sudo fsck /dev/sda1  # After unmounting
// Parent Directory Issues
// bash# No write permission on parent directory
// rm /protected/dir/file.txt
// # Error: rm: cannot remove '/protected/dir/file.txt': Permission denied

// # Check parent directory permissions
// ls -ld /protected/dir
// # Output: dr-xr-xr-x 2 root root 4096 Jan 1 12:00 /protected/dir

// # Fix: Need write permission on parent
// sudo chmod u+w /protected/dir
// rm /protected/dir/file.txt
// Recursive Deletion Failures
// bash# Mixed permissions in directory tree
// rm -r project/
// # Error: rm: cannot remove 'project/subfolder/file': Permission denied
// # (Partial deletion occurs)

// # Fix: Force with sudo
// sudo rm -rf project/

// # Or fix permissions first
// chmod -R u+w project/
// rm -r project/
// Wildcard Expansion Issues
// bash# No files match pattern
// rm *.xyz
// # Error: rm: cannot remove '*.xyz': No such file or directory

// # Fix: Check if files exist first
// ls *.xyz 2>/dev/null && rm *.xyz

// # Or use -f to suppress errors
// rm -f *.xyz

// # Or use find
// find . -name "*.xyz" -type f -delete
// Interactive Mode Cancelled
// bash# Using -i and cancelling
// rm -i *.txt
// # Prompt: rm: remove regular file 'file1.txt'?
// n  # You say no

// # File not deleted (expected behavior)

// # Fix: Answer 'y' or use -f
// rm -f *.txt
// Root-Owned Files in User Directory
// bash# Files created by sudo in your home
// rm ~/file.txt
// # Error: rm: cannot remove '/home/user/file.txt': Permission denied

// ls -l ~/file.txt
// # Output: -rw-r--r-- 1 root root 100 Jan 1 12:00 /home/user/file.txt

// # Fix: Use sudo
// sudo rm ~/file.txt

// # Better: Change ownership
// sudo chown $USER:$USER ~/file.txt
// rm ~/file.txt
// Deletion of Current Directory
// bash# Trying to delete the directory you're in
// cd /tmp/mydir
// rm -r /tmp/mydir
// # Error: rm: cannot remove '/tmp/mydir': Device or resource busy

// # Fix: Go to parent directory first
// cd ..
// rm -r /tmp/mydir

// # Or use absolute path from elsewhere
// cd ~
// rm -r /tmp/mydir
// Hidden Files Not Deleted
// bash# Wildcard doesn't match hidden files
// rm *
// # Hidden files remain

// # Fix: Include hidden files explicitly
// rm * .*
// # Or
// rm -r * .[^.]* ..?*

// # Better: Use find
// find . -delete  # Deletes everything including hidden
// Debugging Commands
// bash# Check what you're about to delete
// ls -la file.txt

// # Check permissions on file and parent
// ls -ld file.txt
// ls -ld $(dirname file.txt)

// # Check what's preventing deletion
// lsof file.txt  # What process is using it
// fuser file.txt # Alternative check
// lsattr file.txt  # Check file attributes

// # Test with verbose mode
// rm -v file.txt

// # Check filesystem status
// df -h
// mount | grep $(dirname file.txt)

// # Check if it's really gone
// stat file.txt
// # Error: stat: cannot stat 'file.txt': No such file or directory
// Safe Practices
// bash# Preview before deleting
// ls *.txt
// # Then delete
// rm *.txt

// # Use -i for important files
// rm -i important*

// # Move to trash instead
// mkdir -p ~/.trash
// mv file.txt ~/.trash/

// # Or use trash-cli
// trash file.txt  # Can be restored

// # Check available space before mass deletion
// df -h .

// # In scripts, always check success
// if rm file.txt; then
//     echo "Deleted successfully"
// else
//     echo "Failed to delete"
// fi
// Pro tip: When rm fails, check: 1) Permissions (ls -l), 2) File attributes (lsattr), 3) What's using it (lsof), 4) 
// Parent directory permissions (ls -ld parent/). Use rm -f only when you're absolutely sure!