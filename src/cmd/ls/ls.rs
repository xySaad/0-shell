use super::ls_config::{ LsConfig };

// wttf mut m3a Refcell ;) (to change later)
pub fn run_ls(args: &Vec<String>) -> i32 {
    let mut ls_config = LsConfig::new(args);
    ls_config.print_ls();
    // println!("files: {:?}", ls_config.process_files());
    *ls_config.status_code.borrow()
}
