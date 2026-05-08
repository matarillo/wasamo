//! counter-rust/src/main.rs — Hello Counter, M2 declarative shape.
//!
//! All UI structure (widget tree, state, binding, click handler) lives in
//! examples/counter/counter.ui. build.rs compiles that to IR text via
//! wasamoc; this binary just hands the IR's absolute path to wasamo_load_ui
//! and runs the message loop. The host contains no wasamo_set_property
//! calls — A2 is structurally enforced.

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

        let path: &str = env!("WASAMO_COUNTER_IR");
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
