use libc::{fopen, FILE};
use std::ffi::CString;
use std::os::raw::c_char;
use std::path::Path;

pub fn string_from_path_ref<T: AsRef<Path>>(path_ref: &T) -> String {
    path_ref
        .as_ref()
        .to_str()
        .expect("Unable to serialize string")
        .to_owned()
}

pub fn join_to_path_then_string<T: AsRef<Path>>(path_ref: &T, join: &str) -> String {
    let new_path = path_ref.as_ref().join(join);
    string_from_path_ref(&new_path)
}

pub unsafe fn fopen_with_string(string: &str, mode: &str) -> *mut FILE {
    let target_file_path = CString::new(string).unwrap();
    let t = fopen(
        target_file_path.as_ptr() as *const c_char,
        mode.as_ptr() as *const c_char,
    );

    if t.is_null() {
        // break with the correct error here
        panic!("Target file does not exist")
    };
    t
}
