mod runtime;
mod window;

use windows::Win32::UI::WindowsAndMessaging::{
    GetMessageW, MSG, TranslateMessage, DispatchMessageW,
};

pub use window::WindowState;

/// Initialise the Wasamo runtime on the calling thread.
///
/// Must be called once from the main thread before any other wasamo function.
/// Sets up the DispatcherQueue (standard STA) and creates the WinRT Compositor.
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
    if state.is_null() { return; }
    window::show(unsafe { &*state });
}

/// Destroy a window and free its resources.
#[no_mangle]
pub extern "C" fn wasamo_window_destroy(state: *mut WindowState) {
    if state.is_null() { return; }
    let boxed = unsafe { Box::from_raw(state) };
    unsafe { windows::Win32::UI::WindowsAndMessaging::DestroyWindow(boxed.hwnd).ok() };
}

/// Run the Win32 message loop until all windows are closed.
///
/// Blocks until `WM_QUIT` is received. Must be called from the main thread.
#[no_mangle]
pub extern "C" fn wasamo_run() {
    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
