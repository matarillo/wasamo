//! gallery-rust/src/main.rs — M3-Phase 2 gallery seed host.
//!
//! The visible Box + Text Phase 2 sub-screen lives in
//! `examples/gallery/gallery.ui`. This host only loads the build-time IR and
//! runs the window.

use std::ffi::{c_void, CStr};
use std::ptr;

use wasamo_sys::{
    wasamo_init, wasamo_last_error_message, wasamo_load_ui, wasamo_run, wasamo_shutdown,
    wasamo_window_show, WasamoWindow, WASAMO_LOAD_PATH, WASAMO_OK,
};

fn main() {
    unsafe {
        if wasamo_init() != WASAMO_OK {
            panic!("wasamo_init failed: {}", last_error());
        }

        let path: &str = env!("WASAMO_GALLERY_IR");
        let path_bytes = path.as_bytes();

        let mut window: *mut WasamoWindow = ptr::null_mut();
        let status = wasamo_load_ui(
            WASAMO_LOAD_PATH,
            path_bytes.as_ptr() as *const c_void,
            path_bytes.len(),
            &mut window,
        );
        if status != WASAMO_OK {
            panic!("wasamo_load_ui failed: {}", last_error());
        }

        if wasamo_window_show(window) != WASAMO_OK {
            panic!("wasamo_window_show failed: {}", last_error());
        }

        wasamo_run();
        wasamo_shutdown();
    }
}

unsafe fn last_error() -> String {
    let p = wasamo_last_error_message();
    if p.is_null() {
        return "(no error message)".into();
    }
    CStr::from_ptr(p).to_string_lossy().into_owned()
}
