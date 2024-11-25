use std::ffi::{CStr, CString};
use std::path::Path;
use libc::{c_char, c_int, c_void};

#[link(name = "magic")]
extern "C" {
    fn magic_open(flags: c_int) -> *mut c_void;
    fn magic_close(cookie: *mut c_void);
    fn magic_load(cookie: *mut c_void, filename: *const c_char) -> c_int;
    fn magic_file(cookie: *mut c_void, filename: *const c_char) -> *const c_char;
}

const MAGIC_MIME_TYPE: c_int = 0x000010;

pub struct Magic {
    cookie: *mut c_void,
}

impl Magic {
    pub fn new() -> Option<Self> {
        unsafe {
            let cookie = magic_open(MAGIC_MIME_TYPE);
            if cookie.is_null() {
                return None;
            }

            if magic_load(cookie, std::ptr::null()) != 0 {
                magic_close(cookie);
                return None;
            }

            Some(Magic { cookie })
        }
    }

    pub fn get_mime_type(&self, path: &Path) -> Option<String> {
        let path_str = CString::new(path.to_string_lossy().as_bytes()).ok()?;
        unsafe {
            let mime_type = magic_file(self.cookie, path_str.as_ptr());
            if mime_type.is_null() {
                return None;
            }

            CStr::from_ptr(mime_type)
                .to_string_lossy()
                .into_owned()
                .split(';')
                .next()
                .map(String::from)
        }
    }
}

impl Drop for Magic {
    fn drop(&mut self) {
        unsafe {
            magic_close(self.cookie);
        }
    }
}
