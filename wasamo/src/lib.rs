mod runtime;
mod window;

pub use window::WindowState;

use windows::Win32::UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, TranslateMessage, MSG};

// ── Rust-native API (used by examples and future bindings) ───────────────────

pub fn init() -> windows::core::Result<()> {
    runtime::init()
}

pub fn window_create(
    title: &str,
    width: i32,
    height: i32,
) -> windows::core::Result<Box<WindowState>> {
    window::create(title, width, height)
}

pub fn window_show(state: &WindowState) {
    window::show(state);
}

pub fn get_compositor() -> &'static windows::UI::Composition::Compositor {
    &runtime::get().compositor
}

pub fn run() {
    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

// ── C ABI (wasamo.h surface) ─────────────────────────────────────────────────

/// Initialise the Wasamo runtime on the calling thread.
///
/// Must be called once from the main thread before any other wasamo function.
#[no_mangle]
pub extern "C" fn wasamo_init() -> i32 {
    match runtime::init() {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Create a new top-level window and attach the Visual Layer to it.
///
/// Returns an opaque `*mut WindowState` handle, or null on failure.
/// The caller must eventually pass the handle to `wasamo_window_destroy`.
#[no_mangle]
pub extern "C" fn wasamo_window_create(
    title: *const u8,
    title_len: usize,
    width: i32,
    height: i32,
) -> *mut WindowState {
    let title = unsafe { std::str::from_utf8(std::slice::from_raw_parts(title, title_len)) }
        .unwrap_or("Wasamo");
    match window::create(title, width, height) {
        Ok(state) => Box::into_raw(state),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Show a window created by `wasamo_window_create`.
#[no_mangle]
pub extern "C" fn wasamo_window_show(state: *mut WindowState) {
    if state.is_null() {
        return;
    }
    window::show(unsafe { &*state });
}

/// Destroy a window and free its resources.
#[no_mangle]
pub extern "C" fn wasamo_window_destroy(state: *mut WindowState) {
    if state.is_null() {
        return;
    }
    let boxed = unsafe { Box::from_raw(state) };
    unsafe { windows::Win32::UI::WindowsAndMessaging::DestroyWindow(boxed.hwnd).ok() };
}

/// Run the Win32 message loop until all windows are closed.
///
/// Blocks until `WM_QUIT` is received. Must be called from the main thread.
#[no_mangle]
pub extern "C" fn wasamo_run() {
    run();
}
