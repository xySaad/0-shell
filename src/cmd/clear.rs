use std::io::{Write, stdout};

pub fn clear() -> i32 {
    print!("\x1b[H\x1b[2J\x1b[3J");
    stdout().flush().unwrap();
    return 0;
}
