use crate::api::SovereignDriveEngine;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[no_mangle]
pub extern "C" fn sd_version() -> *mut c_char {
    CString::new(VERSION).expect("package version contains no NUL").into_raw()
}

#[no_mangle]
pub extern "C" fn sd_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        // SAFETY: pointers returned by this crate are allocated by CString::into_raw.
        unsafe { drop(CString::from_raw(ptr)); }
    }
}

#[no_mangle]
pub extern "C" fn sd_ingest_json(input: *const c_char) -> *mut c_char {
    let output = if input.is_null() {
        r#"{"error":"null input"}"#.to_owned()
    } else {
        // SAFETY: caller must provide a valid NUL-terminated C string.
        let input = unsafe { CStr::from_ptr(input) }.to_string_lossy();
        match SovereignDriveEngine::default().ingest_bytes(input.as_bytes()) {
            Ok(result) => serde_json::to_string(&result).unwrap_or_else(|_| r#"{"error":"serialization failed"}"#.to_owned()),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    };
    CString::new(output).expect("JSON output contains no NUL").into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn null_input_returns_json_error() {
        let ptr = sd_ingest_json(std::ptr::null());
        let out = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned();
        sd_free_string(ptr);
        assert!(out.contains("null input"));
    }

    #[test]
    fn ingest_json_returns_hash() {
        let input = CString::new("hello").unwrap();
        let ptr = sd_ingest_json(input.as_ptr());
        let out = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned();
        sd_free_string(ptr);
        assert!(out.contains("hash"));
    }
}
