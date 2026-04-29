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

use crate::widget::{PropertyError, PropertyValue, WidgetNode};
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
    // Holds the most recent string returned through `wasamo_get_property`.
    // `WasamoValue.v_string.ptr` points into this buffer; valid until the
    // next ABI call on the same thread (abi_spec §3.3, §2.3 rule 2).
    static PROP_STRING_BUF: RefCell<Option<CString>> = const { RefCell::new(None) };
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
    // M1: Compositor / DispatcherQueue are kept alive for the process; we
    // only sever signal/observer registrations and clear thread-local
    // diagnostic buffers. Each surviving destroy_fn is invoked exactly
    // once (abi_spec §4.4 / §4.5).
    crate::registry::drain_all();
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

fn property_error_to_status(e: &PropertyError) -> WasamoStatus {
    match e {
        PropertyError::UnknownId | PropertyError::TypeMismatch => WASAMO_ERR_INVALID_ARG,
        PropertyError::Runtime(_) => WASAMO_ERR_RUNTIME,
    }
}

fn property_error_msg(prefix: &str, e: &PropertyError) -> String {
    match e {
        PropertyError::UnknownId => format!("{prefix}: unknown property id for this widget"),
        PropertyError::TypeMismatch => format!("{prefix}: value type does not match property"),
        PropertyError::Runtime(s) => format!("{prefix}: {s}"),
    }
}

unsafe fn read_property_value(
    value: *const WasamoValue,
) -> Result<PropertyValue, &'static str> {
    if value.is_null() {
        return Err("value is null");
    }
    let v = &*value;
    match v.tag {
        WASAMO_VALUE_I32 => Ok(PropertyValue::I32(v.payload.v_i32)),
        WASAMO_VALUE_STRING => {
            let view = v.payload.v_string;
            let s = if view.ptr.is_null() || view.len == 0 {
                String::new()
            } else {
                let bytes = std::slice::from_raw_parts(view.ptr as *const u8, view.len);
                std::str::from_utf8(bytes)
                    .map_err(|_| "string payload is not valid UTF-8")?
                    .to_owned()
            };
            Ok(PropertyValue::String(s))
        }
        _ => Err("unsupported value tag"),
    }
}

fn write_property_value(out: &mut WasamoValue, value: PropertyValue) {
    match value {
        PropertyValue::I32(v) => {
            out.tag = WASAMO_VALUE_I32;
            out.payload = WasamoValuePayload { v_i32: v };
        }
        PropertyValue::String(s) => {
            // Store the CString in TLS; the pointer we hand back stays valid
            // until the next ABI call on this thread overwrites the slot.
            let cs = CString::new(s).unwrap_or_else(|_| {
                CString::new("(string contained NUL)").unwrap()
            });
            let len = cs.as_bytes().len();
            // Borrow the buffer slot, replace its contents, and re-borrow to
            // grab a stable pointer into the now-owned CString.
            let ptr = PROP_STRING_BUF.with(|cell| {
                let mut slot = cell.borrow_mut();
                *slot = Some(cs);
                slot.as_ref().unwrap().as_ptr()
            });
            out.tag = WASAMO_VALUE_STRING;
            out.payload = WasamoValuePayload {
                v_string: WasamoStringView { ptr, len },
            };
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_get_property(
    widget: *mut WasamoWidget,
    property_id: u32,
    out_value: *mut WasamoValue,
) -> WasamoStatus {
    if widget.is_null() {
        set_last_error("wasamo_get_property: widget is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    if out_value.is_null() {
        set_last_error("wasamo_get_property: out_value is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    match (*widget).get_property(property_id) {
        Ok(value) => {
            write_property_value(&mut *out_value, value);
            clear_last_error();
            WASAMO_OK
        }
        Err(e) => {
            set_last_error(property_error_msg("wasamo_get_property", &e));
            property_error_to_status(&e)
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_set_property(
    widget: *mut WasamoWidget,
    property_id: u32,
    value: *const WasamoValue,
) -> WasamoStatus {
    if widget.is_null() {
        set_last_error("wasamo_set_property: widget is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    let pv = match read_property_value(value) {
        Ok(v) => v,
        Err(msg) => {
            set_last_error(format!("wasamo_set_property: {msg}"));
            return WASAMO_ERR_INVALID_ARG;
        }
    };
    match (*widget).set_property(property_id, &pv) {
        Ok(()) => {
            clear_last_error();
            WASAMO_OK
        }
        Err(e) => {
            set_last_error(property_error_msg("wasamo_set_property", &e));
            property_error_to_status(&e)
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_observe_property(
    widget: *mut WasamoWidget,
    property_id: u32,
    callback: WasamoPropertyObserverFn,
    user_data: *mut std::ffi::c_void,
    destroy_fn: WasamoDestroyFn,
    out_token: *mut u64,
) -> WasamoStatus {
    if widget.is_null() {
        set_last_error("wasamo_observe_property: widget is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    if callback.is_none() {
        set_last_error("wasamo_observe_property: callback is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    if out_token.is_null() {
        set_last_error("wasamo_observe_property: out_token is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    let token = crate::registry::add_observer(
        widget, property_id, callback, user_data, destroy_fn,
    );
    *out_token = token;
    clear_last_error();
    WASAMO_OK
}

#[no_mangle]
pub extern "C" fn wasamo_unobserve_property(token: u64) -> WasamoStatus {
    if crate::registry::remove(token) {
        clear_last_error();
        WASAMO_OK
    } else {
        set_last_error("wasamo_unobserve_property: unknown token");
        WASAMO_ERR_INVALID_ARG
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_signal_connect(
    widget: *mut WasamoWidget,
    signal_name_utf8: *const c_char,
    name_len: usize,
    callback: WasamoSignalHandlerFn,
    user_data: *mut std::ffi::c_void,
    destroy_fn: WasamoDestroyFn,
    out_token: *mut u64,
) -> WasamoStatus {
    if widget.is_null() {
        set_last_error("wasamo_signal_connect: widget is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    if callback.is_none() {
        set_last_error("wasamo_signal_connect: callback is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    if out_token.is_null() {
        set_last_error("wasamo_signal_connect: out_token is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    if signal_name_utf8.is_null() || name_len == 0 {
        set_last_error("wasamo_signal_connect: signal_name is empty");
        return WASAMO_ERR_INVALID_ARG;
    }
    let bytes = std::slice::from_raw_parts(signal_name_utf8 as *const u8, name_len);
    let name = match std::str::from_utf8(bytes) {
        Ok(s) => s.to_owned(),
        Err(_) => {
            set_last_error("wasamo_signal_connect: signal_name is not valid UTF-8");
            return WASAMO_ERR_INVALID_ARG;
        }
    };
    let token = crate::registry::add_signal(
        widget, name, callback, user_data, destroy_fn,
    );
    *out_token = token;
    clear_last_error();
    WASAMO_OK
}

#[no_mangle]
pub extern "C" fn wasamo_signal_disconnect(token: u64) -> WasamoStatus {
    if crate::registry::remove(token) {
        clear_last_error();
        WASAMO_OK
    } else {
        set_last_error("wasamo_signal_disconnect: unknown token");
        WASAMO_ERR_INVALID_ARG
    }
}
