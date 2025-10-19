
use super::ls_config::{LsConfig};

pub fn run_ls(args: &Vec<String>) -> i32 {
    let mut ls_config = LsConfig::new(args);
    ls_config.print_ls();
    let status_code = *ls_config.status_code.borrow();
    return status_code;
}
