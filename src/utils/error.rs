use libc::c_int;
use std::{ffi::CStr, io::Error};
pub trait StrError {
    fn str(&self) -> String;
}

impl StrError for Error {
    fn str(&self) -> String {
        return if let Some(errno) = self.raw_os_error() {
            strerror(errno)
        } else {
            self.to_string()
        };
    }
}

pub fn strerror(errno: c_int) -> String {
    unsafe {
        let c_rawchar = libc::strerror(errno);
        let c_string = CStr::from_ptr(c_rawchar);
        return String::from_utf8_lossy(c_string.to_bytes()).to_string();
    }
}

pub fn clear_error(err: Error) -> String {
    let mut msg = err.to_string();
    if let Some(idx) = msg.find(" (os error") {
        msg.truncate(idx);
    }
    msg.trim().to_string()
}