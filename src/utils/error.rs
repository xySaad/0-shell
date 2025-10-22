use std::ffi::CStr;

use libc::c_int;
pub fn strerror(errno: c_int) -> String {
    unsafe {
        let c_rawchar = libc::strerror(errno);
        let c_string = CStr::from_ptr(c_rawchar);
        return String::from_utf8_lossy(c_string.to_bytes()).to_string();
    }
}
