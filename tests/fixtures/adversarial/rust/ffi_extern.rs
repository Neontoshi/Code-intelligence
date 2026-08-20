// tests/fixtures/adversarial/rust/ffi_extern.rs

//! Functions that look dead but are called from external code via FFI

use std::ffi::CString;

// ⚠️ Looks dead - no internal callers, but exported to C
#[no_mangle]
pub extern "C" fn process_data(data: *const u8, len: usize) -> i32 {
    if data.is_null() {
        return -1;
    }
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    slice.len() as i32
}

// ⚠️ Looks dead - called from C via extern
#[no_mangle]
pub extern "C" fn initialize_engine(config: *const u8) -> i32 {
    if config.is_null() {
        return -1;
    }
    // Simulate initialization
    0
}

// ⚠️ Looks dead - used by a C library
#[no_mangle]
pub extern "C" fn get_version() -> *const std::os::raw::c_char {
    let version = CString::new("1.0.0").unwrap();
    version.into_raw()
}

// Internal helper that IS called
fn internal_helper() -> i32 {
    42
}

// Entry point that calls the internal helper
#[no_mangle]
pub extern "C" fn get_answer() -> i32 {
    internal_helper()
}
