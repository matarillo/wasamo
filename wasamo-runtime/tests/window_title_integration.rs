//! Mock-free Windows-only runtime evidence for M3-Phase 6 T6.
//!
//! Drives a component-level static `title:` through `.ui` lowering,
//! `wasamo_load_ui`, and the native HWND title text.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::ffi::c_void;
use std::ptr;

use wasamo_runtime::ffi;

use windows::Win32::UI::WindowsAndMessaging::{GetWindowTextLengthW, GetWindowTextW};

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<window-title-integration>";
    let tokens = lexer::tokenize(src, path).expect("lex failed");
    let ast = parser::parse(&tokens, path).expect("parse failed");
    let checked = check::check(&ast, path);
    assert!(
        !checked.has_errors(),
        "check errors: {:?}",
        checked.diagnostics
    );
    emit::emit(&lower::lower(&ast, &checked.namespace))
}

unsafe fn hwnd_title(window: *mut ffi::WasamoWindow) -> String {
    let hwnd = (*window).hwnd;
    let len = GetWindowTextLengthW(hwnd);
    assert!(len >= 0, "GetWindowTextLengthW returned {len}");
    let mut buf = vec![0u16; len as usize + 1];
    let written = GetWindowTextW(hwnd, &mut buf);
    assert!(written >= 0, "GetWindowTextW returned {written}");
    String::from_utf16_lossy(&buf[..written as usize])
}

#[test]
fn static_component_title_reaches_native_window() {
    run_on_owning_runtime_thread_or_skip("static Window title integration test", move || {
        let ir = lower_ui_to_ir(
            r#"
component Gallery inherits Window {
    title: "Gallery"

    VStack {
        Text { content: "Title proof" }
    }
}
"#,
        );

        let mut window: *mut ffi::WasamoWindow = ptr::null_mut();
        let status = unsafe {
            ffi::wasamo_load_ui(
                ffi::WASAMO_LOAD_MEMORY,
                ir.as_ptr() as *const c_void,
                ir.len(),
                &mut window as *mut *mut ffi::WasamoWindow,
            )
        };
        assert_eq!(status, ffi::WASAMO_OK);
        assert!(!window.is_null(), "wasamo_load_ui must return a window");

        let title = unsafe { hwnd_title(window) };
        unsafe {
            ffi::wasamo_window_destroy(window);
        }

        assert_eq!(title, "Gallery");
    });
}
