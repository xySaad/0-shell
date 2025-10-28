use crate::cli::read_input;
mod cli;
mod cmd;
mod compiler;
mod utils;


fn main() {
    
    // welcome func
    if let Err(err) = cli::print("$ ") {
        panic!("{}", err);
    }

    read_input()
}
