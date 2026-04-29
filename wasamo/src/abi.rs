//! Wasamo C ABI surface. The canonical specification is `docs/abi_spec.md`;
//! ADR `docs/decisions/phase-6-c-abi.md` records the decisions behind it.
//!
//! Layout invariants in this module must match `bindings/c/wasamo.h`. The
//! CI smoke test (compile + link a TU including `wasamo.h` against
//! `wasamo.dll.lib`) catches drift.

// Constants and types are exported across the C ABI by name; rustc's
// dead_code lint cannot see those uses.
#![allow(dead_code)]

use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

use crate::widget::WidgetNode;
use crate::window::WindowState;

// ── Type aliases for opaque handles ──────────────────────────────────────────
//
// The C header declares `WasamoWindow` and `WasamoWidget` as opaque
// forward-declared structs. Internally we use `WindowState` and `WidgetNode`;
// only pointer-sized opaque pointers cross the ABI, so the type-alias bridge
// is binary-equivalent to the header's forward declaration.

pub type WasamoWindow = WindowState;
pub type WasamoWidget = WidgetNode;

// ── 3.1 WasamoStatus ─────────────────────────────────────────────────────────

pub type WasamoStatus = i32;

pub const WASAMO_OK: WasamoStatus = 0;
pub const WASAMO_ERR_INVALID_ARG: WasamoStatus = -1;
pub const WASAMO_ERR_RUNTIME: WasamoStatus = -2;
pub const WASAMO_ERR_NOT_INITIALIZED: WasamoStatus = -3;
pub const WASAMO_ERR_WRONG_THREAD: WasamoStatus = -4;

// ── 3.3 WasamoValue ──────────────────────────────────────────────────────────

pub type WasamoValueTag = i32;

pub const WASAMO_VALUE_NONE: WasamoValueTag = 0;
pub const WASAMO_VALUE_I32: WasamoValueTag = 1;
pub const WASAMO_VALUE_F64: WasamoValueTag = 2;
pub const WASAMO_VALUE_BOOL: WasamoValueTag = 3;
pub const WASAMO_VALUE_STRING: WasamoValueTag = 4;
pub const WASAMO_VALUE_WIDGET: WasamoValueTag = 5;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct WasamoStringView {
    pub ptr: *const c_char,
    pub len: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union WasamoValuePayload {
    pub v_i32: i32,
    pub v_f64: f64,
    pub v_bool: i32,
    pub v_string: WasamoStringView,
    pub v_widget: *mut WasamoWidget,
    _none: (),
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct WasamoValue {
    pub tag: WasamoValueTag,
    /// Mirrors the C field named `as` (a Rust keyword).
    pub payload: WasamoValuePayload,
}

// ── 3.4 Callback typedefs ────────────────────────────────────────────────────

pub type WasamoDestroyFn = Option<unsafe extern "C" fn(user_data: *mut std::ffi::c_void)>;

pub type WasamoSignalHandlerFn = Option<
    unsafe extern "C" fn(
        sender: *mut WasamoWidget,
        args: *const WasamoValue,
        arg_count: usize,
        user_data: *mut std::ffi::c_void,
    ),
>;

pub type WasamoPropertyObserverFn = Option<
    unsafe extern "C" fn(
        widget: *mut WasamoWidget,
        property_id: u32,
        new_value: *const WasamoValue,
        user_data: *mut std::ffi::c_void,
    ),
>;

// ── Thread-local last-error storage ──────────────────────────────────────────

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

pub(crate) fn set_last_error(msg: impl Into<Vec<u8>>) {
    let cs = CString::new(msg).unwrap_or_else(|_| CString::new("(error message contained NUL)").unwrap());
    LAST_ERROR.with(|cell| *cell.borrow_mut() = Some(cs));
}

pub(crate) fn clear_last_error() {
    LAST_ERROR.with(|cell| *cell.borrow_mut() = None);
}

// ── 4.1 Runtime lifecycle ────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn wasamo_init() -> WasamoStatus {
    match crate::runtime::init() {
        Ok(()) => {
            clear_last_error();
            WASAMO_OK
        }
        Err(e) => {
            set_last_error(format!("wasamo_init: {e}"));
            WASAMO_ERR_RUNTIME
        }
    }
}

#[no_mangle]
pub extern "C" fn wasamo_shutdown() {
    // M1: runtime state is process-global and lives until process exit.
    // A real shutdown (releasing Compositor / DispatcherQueue, severing all
    // signal/observer registrations) lands together with the registry
    // implementation. For now this is a documented no-op.
    clear_last_error();
}

#[no_mangle]
pub extern "C" fn wasamo_last_error_message() -> *const c_char {
    LAST_ERROR.with(|cell| {
        cell.borrow()
            .as_ref()
            .map_or(ptr::null(), |s| s.as_ptr())
    })
}

// ── 4.2 Window and event loop ────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn wasamo_window_create(
    title_utf8: *const c_char,
    title_len: usize,
    width: i32,
    height: i32,
    out: *mut *mut WasamoWindow,
) -> WasamoStatus {
    if out.is_null() {
        set_last_error("wasamo_window_create: out is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    *out = ptr::null_mut();

    let title = if title_utf8.is_null() || title_len == 0 {
        "Wasamo"
    } else {
        let bytes = std::slice::from_raw_parts(title_utf8 as *const u8, title_len);
        match std::str::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => {
                set_last_error("wasamo_window_create: title_utf8 is not valid UTF-8");
                return WASAMO_ERR_INVALID_ARG;
            }
        }
    };

    match crate::window::create(title, width, height) {
        Ok(state) => {
            *out = Box::into_raw(state);
            clear_last_error();
            WASAMO_OK
        }
        Err(e) => {
            set_last_error(format!("wasamo_window_create: {e}"));
            WASAMO_ERR_RUNTIME
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_window_show(window: *mut WasamoWindow) -> WasamoStatus {
    if window.is_null() {
        set_last_error("wasamo_window_show: window is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    crate::window::show(&*window);
    clear_last_error();
    WASAMO_OK
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_window_destroy(window: *mut WasamoWindow) -> WasamoStatus {
    if window.is_null() {
        // Idempotent on null per spec §4.2.
        return WASAMO_OK;
    }
    let boxed = Box::from_raw(window);
    let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(boxed.hwnd);
    clear_last_error();
    WASAMO_OK
}

#[no_mangle]
pub extern "C" fn wasamo_run() {
    crate::run();
}

#[no_mangle]
pub extern "C" fn wasamo_quit() {
    unsafe {
        windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
    }
}

// ── 4.3 / 4.4 / 4.5 — property R/W, observers, signals ───────────────────────
//
// The dispatch table on widgets, the token-based signal/observer registry,
// and the queued-emission machinery land in subsequent commits within
// Phase 6. The function symbols are declared here so the header and the
// Rust extern "C" surface stay in alignment from the start.

#[no_mangle]
pub unsafe extern "C" fn wasamo_get_property(
    _widget: *mut WasamoWidget,
    _property_id: u32,
    _out_value: *mut WasamoValue,
) -> WasamoStatus {
    set_last_error("wasamo_get_property: not yet implemented (phase 6 in progress)");
    WASAMO_ERR_RUNTIME
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_set_property(
    _widget: *mut WasamoWidget,
    _property_id: u32,
    _value: *const WasamoValue,
) -> WasamoStatus {
    set_last_error("wasamo_set_property: not yet implemented (phase 6 in progress)");
    WASAMO_ERR_RUNTIME
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_observe_property(
    _widget: *mut WasamoWidget,
    _property_id: u32,
    _callback: WasamoPropertyObserverFn,
    _user_data: *mut std::ffi::c_void,
    _destroy_fn: WasamoDestroyFn,
    _out_token: *mut u64,
) -> WasamoStatus {
    set_last_error("wasamo_observe_property: not yet implemented (phase 6 in progress)");
    WASAMO_ERR_RUNTIME
}

#[no_mangle]
pub extern "C" fn wasamo_unobserve_property(_token: u64) -> WasamoStatus {
    set_last_error("wasamo_unobserve_property: not yet implemented (phase 6 in progress)");
    WASAMO_ERR_RUNTIME
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_signal_connect(
    _widget: *mut WasamoWidget,
    _signal_name_utf8: *const c_char,
    _name_len: usize,
    _callback: WasamoSignalHandlerFn,
    _user_data: *mut std::ffi::c_void,
    _destroy_fn: WasamoDestroyFn,
    _out_token: *mut u64,
) -> WasamoStatus {
    set_last_error("wasamo_signal_connect: not yet implemented (phase 6 in progress)");
    WASAMO_ERR_RUNTIME
}

#[no_mangle]
pub extern "C" fn wasamo_signal_disconnect(_token: u64) -> WasamoStatus {
    set_last_error("wasamo_signal_disconnect: not yet implemented (phase 6 in progress)");
    WASAMO_ERR_RUNTIME
}
