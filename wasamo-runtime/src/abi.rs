//! Wasamo C ABI surface. The canonical specification is `docs/abi_spec.md`;
//! ADR `process/milestone-1/phase-6/decisions/preamble.md` records the decisions behind it.
//!
//! Layout invariants in this module must match `bindings/c/wasamo.h`. The
//! CI smoke test (compile + link a TU including `wasamo.h` against
//! `wasamo.dll.lib`) catches drift.

// The constants below mirror the closed enum tag sets defined in
// `wasamo.h` (§3.1 `WasamoStatus`, §3.3 `WasamoValueTag`). Several
// values aren't emitted by any M1 widget yet but are part of the ABI
// surface and visible to Rust callers of the rlib, so we keep the
// full set declared.
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
/// Returned when a structure-changing ABI call (e.g. `wasamo_load_ui`) is
/// issued while the reactive drain's Phase 1 convergence loop is executing.
pub const WASAMO_ERR_REENTRANT_LOAD: WasamoStatus = -5;
/// Returned by every ABI call (except `wasamo_runtime_destroy`) after the
/// runtime has entered the irreversible `Diverged` health state.
pub const WASAMO_ERR_REACTIVE_DIVERGED: WasamoStatus = -6;
/// Returned when a state-mutating ABI call is issued from within a Phase 3
/// post-commit observer callback.
pub const WASAMO_ERR_OBSERVER_MUTATION: WasamoStatus = -7;
/// Returned by `wasamo_load_ui` when the supplied IR fails header,
/// parse, unknown-widget, or defense-in-depth validation
/// (DD-M2-P6-009 / DD-M2-P6-005).
pub const WASAMO_ERR_IR_MALFORMED: WasamoStatus = -8;

// ── 3.5 wasamo_load_ui resource type (DD-M2-P6-005) ──────────────────────────

pub type WasamoLoadType = i32;

/// `data` is a UTF-8 filesystem path of length `data_len` (no NUL required).
pub const WASAMO_LOAD_PATH: WasamoLoadType = 0;
/// `data` is a `data_len`-byte in-memory blob carrying the IR. M2 accepts
/// only the IR text grammar (DD-M2-P6-002); the byte layout is defined to
/// admit a future binary IR without ABI breakage.
pub const WASAMO_LOAD_MEMORY: WasamoLoadType = 1;

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
    let cs = CString::new(msg)
        .unwrap_or_else(|_| CString::new("(error message contained NUL)").unwrap());
    LAST_ERROR.with(|cell| *cell.borrow_mut() = Some(cs));
}

pub(crate) fn clear_last_error() {
    LAST_ERROR.with(|cell| *cell.borrow_mut() = None);
}

/// Guard: reject if the runtime is uninitialized or the calling thread is
/// not the runtime's owning UI thread (DD-M2-P6-005, abi_spec §6).
///
/// Pre-init calls return `WASAMO_ERR_NOT_INITIALIZED`; cross-thread calls
/// return `WASAMO_ERR_WRONG_THREAD` without performing the requested
/// action. Both leave runtime state untouched.
#[inline]
fn check_owning_thread(fn_name: &str) -> Option<WasamoStatus> {
    if !crate::runtime::is_initialized() {
        set_last_error(format!("{fn_name}: wasamo_init has not been called"));
        return Some(WASAMO_ERR_NOT_INITIALIZED);
    }
    if !crate::runtime::is_owning_thread() {
        set_last_error(format!(
            "{fn_name}: called from non-owning thread (must be the thread that called wasamo_init)"
        ));
        return Some(WASAMO_ERR_WRONG_THREAD);
    }
    None
}

/// Guard: reject if the runtime has diverged (called by most ABI functions).
/// Returns `Some(WASAMO_ERR_REACTIVE_DIVERGED)` if the caller should return
/// immediately; `None` if the runtime is healthy.
///
/// On entry to the Diverged path the structured diagnostics captured by the
/// drain loop (DD-M2-P6-006) are folded into the thread-local last-error
/// payload so binding callers see them through `wasamo_last_error_message`
/// (DD-M2-P6-005 carry-over).
#[inline]
fn check_not_diverged(fn_name: &str) -> Option<WasamoStatus> {
    if crate::reactive::runtime_health() == crate::reactive::RuntimeHealth::Diverged {
        let detail = crate::reactive::divergence_diagnostics()
            .map(|d| {
                format!(
                    " (offending effect id={}, iterations={}, last dirty signals={:?})",
                    d.offending_effect_id, d.iteration_count, d.last_dirty_signal_ids
                )
            })
            .unwrap_or_default();
        set_last_error(format!(
            "{fn_name}: runtime is in Diverged state; call wasamo_runtime_destroy{detail}"
        ));
        Some(WASAMO_ERR_REACTIVE_DIVERGED)
    } else {
        None
    }
}

/// Guard: reject structure-changing calls issued during Phase 1 drain.
#[inline]
fn check_not_in_drain(fn_name: &str) -> Option<WasamoStatus> {
    if crate::emit::in_drain() {
        set_last_error(format!(
            "{fn_name}: structure-changing ABI called during reactive drain (Phase 1)"
        ));
        Some(WASAMO_ERR_REENTRANT_LOAD)
    } else {
        None
    }
}

/// Guard: reject state-mutating calls issued from a Phase 3 observer callback.
#[inline]
fn check_not_in_observer(fn_name: &str) -> Option<WasamoStatus> {
    if crate::emit::in_observer_callback() {
        set_last_error(format!(
            "{fn_name}: state-mutating ABI called from within observer callback (Phase 3)"
        ));
        Some(WASAMO_ERR_OBSERVER_MUTATION)
    } else {
        None
    }
}

/// Test-only pub wrappers so reactive.rs tests can exercise the guard logic
/// without going through the full ABI stack.
#[cfg(test)]
pub(crate) fn check_not_in_observer_pub(fn_name: &str) -> Option<WasamoStatus> {
    check_not_in_observer(fn_name)
}

#[cfg(test)]
pub(crate) fn check_not_diverged_pub(fn_name: &str) -> Option<WasamoStatus> {
    check_not_diverged(fn_name)
}

/// Shorthand: check thread + diverged + in_drain (structure-changing ABI).
macro_rules! guard_structural {
    ($fn_name:expr) => {
        if let Some(s) = check_owning_thread($fn_name) {
            return s;
        }
        if let Some(s) = check_not_diverged($fn_name) {
            return s;
        }
        if let Some(s) = check_not_in_drain($fn_name) {
            return s;
        }
        if let Some(s) = check_not_in_observer($fn_name) {
            return s;
        }
    };
}

/// Shorthand: check thread + diverged + in_observer (state-mutating ABI).
macro_rules! guard_mutating {
    ($fn_name:expr) => {
        if let Some(s) = check_owning_thread($fn_name) {
            return s;
        }
        if let Some(s) = check_not_diverged($fn_name) {
            return s;
        }
        if let Some(s) = check_not_in_observer($fn_name) {
            return s;
        }
    };
}

// ── 4.1 Runtime lifecycle ────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn wasamo_init() -> WasamoStatus {
    // Cleared on *entry*, not on the success arm the way the status-returning
    // entry points in this file do. `runtime::init` records the DPI-awareness
    // declaration's outcome into this same thread-local as a diagnostic
    // (DD-M4-P1-001 §Failure handling, abi_spec §4.1), so a success-path clear
    // would run after that write and discard the disclosure the spec promises.
    // Clearing here still drops a stale error from an earlier call, which is
    // all the convention was buying; what it no longer drops is this
    // function's own output.
    clear_last_error();
    match crate::runtime::init() {
        Ok(()) => WASAMO_OK,
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
    //
    // Thread affinity: shutdown must run on the owning UI thread. From a
    // wrong thread we silently no-op (the void return type leaves no
    // channel to report the error); the last-error TLS is still updated
    // so a subsequent same-thread call can observe it.
    if check_owning_thread("wasamo_shutdown").is_some() {
        return;
    }
    crate::registry::drain_all();
    clear_last_error();
}

#[no_mangle]
pub extern "C" fn wasamo_last_error_message() -> *const c_char {
    LAST_ERROR.with(|cell| cell.borrow().as_ref().map_or(ptr::null(), |s| s.as_ptr()))
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
    guard_structural!("wasamo_window_create");
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
    guard_mutating!("wasamo_window_show");
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
    // Note: wasamo_window_destroy is NOT guarded by check_not_diverged — destroy
    // must succeed even in Diverged state (spec: only wasamo_runtime_destroy is
    // exempt, but window/widget destroy are also safe to allow for cleanup).
    // Thread affinity is still enforced.
    if let Some(s) = check_owning_thread("wasamo_window_destroy") {
        return s;
    }
    if window.is_null() {
        // Idempotent on null per spec §4.2.
        return WASAMO_OK;
    }
    crate::emit::unregister_window(window);
    let boxed = Box::from_raw(window);
    // Sever registry entries for the entire owned widget subtree before any
    // widget memory is freed. Any host-supplied destroy_fn is invoked here.
    if let Some(root) = boxed.root_widget.as_ref() {
        root.for_each_ptr(&mut |p| crate::registry::remove_for_widget(p));
    }
    let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(boxed.hwnd);
    clear_last_error();
    WASAMO_OK
}

// ── 4.6 Tree mutation (DD-M2-P4-001/002/003 = Option A) ──────────────────────

#[no_mangle]
pub unsafe extern "C" fn wasamo_widget_append_child(
    parent: *mut WasamoWidget,
    child: *mut WasamoWidget,
) -> WasamoStatus {
    guard_structural!("wasamo_widget_append_child");
    if parent.is_null() {
        set_last_error("wasamo_widget_append_child: parent is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    if child.is_null() {
        set_last_error("wasamo_widget_append_child: child is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    let child_box = Box::from_raw(child);
    if child_box.attached {
        // Don't consume the box if we're going to return an error; leak-free
        // by converting back to raw and setting the error.
        let _ = Box::into_raw(child_box);
        set_last_error("wasamo_widget_append_child: child is already attached");
        return WASAMO_ERR_INVALID_ARG;
    }
    match (*parent).append_child(child_box) {
        Ok(()) => {
            clear_last_error();
            WASAMO_OK
        }
        Err(e) => {
            set_last_error(format!("wasamo_widget_append_child: {e}"));
            WASAMO_ERR_RUNTIME
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_widget_insert_child(
    parent: *mut WasamoWidget,
    index: usize,
    child: *mut WasamoWidget,
) -> WasamoStatus {
    guard_structural!("wasamo_widget_insert_child");
    if parent.is_null() {
        set_last_error("wasamo_widget_insert_child: parent is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    if child.is_null() {
        set_last_error("wasamo_widget_insert_child: child is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    let child_box = Box::from_raw(child);
    match (*parent).insert_child(index, child_box) {
        Ok(()) => {
            clear_last_error();
            WASAMO_OK
        }
        Err(crate::widget::MutationError::IndexOutOfBounds) => {
            set_last_error(format!(
                "wasamo_widget_insert_child: index {index} out of bounds"
            ));
            WASAMO_ERR_INVALID_ARG
        }
        Err(crate::widget::MutationError::AlreadyAttached) => {
            set_last_error("wasamo_widget_insert_child: child is already attached");
            WASAMO_ERR_INVALID_ARG
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_widget_remove_child(
    parent: *mut WasamoWidget,
    index: usize,
    out_removed: *mut *mut WasamoWidget,
) -> WasamoStatus {
    guard_structural!("wasamo_widget_remove_child");
    if parent.is_null() {
        set_last_error("wasamo_widget_remove_child: parent is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    if out_removed.is_null() {
        set_last_error("wasamo_widget_remove_child: out_removed is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    *out_removed = ptr::null_mut();
    match (*parent).remove_child(index) {
        Ok(removed) => {
            *out_removed = Box::into_raw(removed);
            clear_last_error();
            WASAMO_OK
        }
        Err(crate::widget::MutationError::IndexOutOfBounds) => {
            set_last_error(format!(
                "wasamo_widget_remove_child: index {index} out of bounds"
            ));
            WASAMO_ERR_INVALID_ARG
        }
        Err(crate::widget::MutationError::AlreadyAttached) => {
            set_last_error("wasamo_widget_remove_child: unexpected AlreadyAttached error");
            WASAMO_ERR_RUNTIME
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_widget_replace_child(
    parent: *mut WasamoWidget,
    index: usize,
    new_child: *mut WasamoWidget,
    out_old: *mut *mut WasamoWidget,
) -> WasamoStatus {
    guard_structural!("wasamo_widget_replace_child");
    if parent.is_null() {
        set_last_error("wasamo_widget_replace_child: parent is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    if new_child.is_null() {
        set_last_error("wasamo_widget_replace_child: new_child is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    if out_old.is_null() {
        set_last_error("wasamo_widget_replace_child: out_old is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    *out_old = ptr::null_mut();
    let new_box = Box::from_raw(new_child);
    match (*parent).replace_child(index, new_box) {
        Ok(old) => {
            *out_old = Box::into_raw(old);
            clear_last_error();
            WASAMO_OK
        }
        Err(crate::widget::MutationError::IndexOutOfBounds) => {
            set_last_error(format!(
                "wasamo_widget_replace_child: index {index} out of bounds"
            ));
            WASAMO_ERR_INVALID_ARG
        }
        Err(crate::widget::MutationError::AlreadyAttached) => {
            set_last_error("wasamo_widget_replace_child: new_child is already attached");
            WASAMO_ERR_INVALID_ARG
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_widget_child_count(
    parent: *mut WasamoWidget,
    out_count: *mut usize,
) -> WasamoStatus {
    if let Some(s) = check_owning_thread("wasamo_widget_child_count") {
        return s;
    }
    if let Some(s) = check_not_diverged("wasamo_widget_child_count") {
        return s;
    }
    if parent.is_null() {
        set_last_error("wasamo_widget_child_count: parent is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    if out_count.is_null() {
        set_last_error("wasamo_widget_child_count: out_count is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    *out_count = (*parent).child_count();
    clear_last_error();
    WASAMO_OK
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_widget_destroy(widget: *mut WasamoWidget) -> WasamoStatus {
    // Like wasamo_window_destroy, destroy is allowed in Diverged state for cleanup.
    // Thread affinity is still enforced.
    if let Some(s) = check_owning_thread("wasamo_widget_destroy") {
        return s;
    }
    if widget.is_null() {
        // Idempotent on null, matching wasamo_window_destroy (DD-M2-P4-003).
        return WASAMO_OK;
    }
    if (*widget).attached {
        set_last_error(
            "wasamo_widget_destroy: widget is currently attached; \
             remove it from its parent first or destroy the owning window",
        );
        return WASAMO_ERR_INVALID_ARG;
    }
    let node = Box::from_raw(widget);
    crate::widget::widget_destroy(node);
    clear_last_error();
    WASAMO_OK
}

#[no_mangle]
pub extern "C" fn wasamo_run() {
    // No-op if called from a disallowed runtime state; the void return has no
    // status channel, but the last-error TLS records the violation.
    if check_owning_thread("wasamo_run").is_some() {
        return;
    }
    if check_not_diverged("wasamo_run").is_some() {
        return;
    }
    crate::run();
}

#[no_mangle]
pub extern "C" fn wasamo_quit() {
    if check_owning_thread("wasamo_quit").is_some() {
        return;
    }
    if check_not_diverged("wasamo_quit").is_some() {
        return;
    }
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

unsafe fn read_property_value(value: *const WasamoValue) -> Result<PropertyValue, &'static str> {
    if value.is_null() {
        return Err("value is null");
    }
    let v = &*value;
    match v.tag {
        WASAMO_VALUE_I32 => Ok(PropertyValue::I32(v.payload.v_i32)),
        WASAMO_VALUE_BOOL => Ok(PropertyValue::Bool(v.payload.v_bool != 0)),
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
        PropertyValue::Bool(b) => {
            out.tag = WASAMO_VALUE_BOOL;
            out.payload = WasamoValuePayload {
                v_bool: if b { 1 } else { 0 },
            };
        }
        PropertyValue::String(s) => {
            // Store the CString in TLS; the pointer we hand back stays valid
            // until the next ABI call on this thread overwrites the slot.
            let cs =
                CString::new(s).unwrap_or_else(|_| CString::new("(string contained NUL)").unwrap());
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
    if let Some(s) = check_owning_thread("wasamo_get_property") {
        return s;
    }
    if let Some(s) = check_not_diverged("wasamo_get_property") {
        return s;
    }
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
    guard_mutating!("wasamo_set_property");
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
            // abi_spec §4.3: schedule observers AFTER the call returns.
            // We push to the emission queue here; the actual callbacks
            // fire when `drain_if_outermost` runs at the tail.
            crate::emit::enqueue_property_change(widget, property_id, property_value_to_owned(&pv));
            clear_last_error();
            crate::emit::drain_if_outermost();
            WASAMO_OK
        }
        Err(e) => {
            set_last_error(property_error_msg("wasamo_set_property", &e));
            property_error_to_status(&e)
        }
    }
}

fn property_value_to_owned(pv: &PropertyValue) -> crate::emit::OwnedArg {
    match pv {
        PropertyValue::I32(v) => crate::emit::OwnedArg::I32(*v),
        PropertyValue::Bool(b) => crate::emit::OwnedArg::Bool(*b),
        PropertyValue::String(s) => crate::emit::OwnedArg::String(s.clone()),
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
    guard_structural!("wasamo_observe_property");
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
    let token = crate::registry::add_observer(widget, property_id, callback, user_data, destroy_fn);
    *out_token = token;
    clear_last_error();
    WASAMO_OK
}

#[no_mangle]
pub extern "C" fn wasamo_unobserve_property(token: u64) -> WasamoStatus {
    if let Some(s) = check_owning_thread("wasamo_unobserve_property") {
        return s;
    }
    if let Some(s) = check_not_diverged("wasamo_unobserve_property") {
        return s;
    }
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
    guard_structural!("wasamo_signal_connect");
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
    let token = crate::registry::add_signal(widget, name, callback, user_data, destroy_fn);
    *out_token = token;
    clear_last_error();
    WASAMO_OK
}

#[no_mangle]
pub extern "C" fn wasamo_signal_disconnect(token: u64) -> WasamoStatus {
    if let Some(s) = check_owning_thread("wasamo_signal_disconnect") {
        return s;
    }
    if let Some(s) = check_not_diverged("wasamo_signal_disconnect") {
        return s;
    }
    if crate::registry::remove(token) {
        clear_last_error();
        WASAMO_OK
    } else {
        set_last_error("wasamo_signal_disconnect: unknown token");
        WASAMO_ERR_INVALID_ARG
    }
}

// ── 5. M1 experimental layer (abi_spec §5) ───────────────────────────────────
//
// Constructors return a runtime-owned `*mut WasamoWidget` (boxed `WidgetNode`
// internally). Children handed to a container constructor are MOVED into it;
// the host's child pointers become stale on success and must not be reused.
// Final ownership is transferred to a `WasamoWindow` via `wasamo_window_set_root`,
// which is also responsible for the eventual drop.

unsafe fn read_utf8(ptr: *const c_char, len: usize) -> Result<String, &'static str> {
    if ptr.is_null() || len == 0 {
        return Ok(String::new());
    }
    let bytes = std::slice::from_raw_parts(ptr as *const u8, len);
    std::str::from_utf8(bytes)
        .map(|s| s.to_owned())
        .map_err(|_| "invalid UTF-8")
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_text_create(
    content_utf8: *const c_char,
    content_len: usize,
    out: *mut *mut WasamoWidget,
) -> WasamoStatus {
    guard_structural!("wasamo_text_create");
    if out.is_null() {
        set_last_error("wasamo_text_create: out is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    *out = ptr::null_mut();
    let content = match read_utf8(content_utf8, content_len) {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("wasamo_text_create: {e}"));
            return WASAMO_ERR_INVALID_ARG;
        }
    };
    let rt = crate::runtime::get();
    match WidgetNode::text(
        &rt.compositor,
        &rt.text_renderer,
        &content,
        crate::text::TypographyStyle::Body,
    ) {
        Ok(node) => {
            *out = Box::into_raw(node);
            clear_last_error();
            WASAMO_OK
        }
        Err(e) => {
            set_last_error(format!("wasamo_text_create: {e}"));
            WASAMO_ERR_RUNTIME
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_button_create(
    label_utf8: *const c_char,
    label_len: usize,
    out: *mut *mut WasamoWidget,
) -> WasamoStatus {
    guard_structural!("wasamo_button_create");
    if out.is_null() {
        set_last_error("wasamo_button_create: out is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    *out = ptr::null_mut();
    let label = match read_utf8(label_utf8, label_len) {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("wasamo_button_create: {e}"));
            return WASAMO_ERR_INVALID_ARG;
        }
    };
    let rt = crate::runtime::get();
    match WidgetNode::button(
        &rt.compositor,
        &rt.text_renderer,
        &label,
        crate::widget::ButtonStyle::Default,
    ) {
        Ok(node) => {
            *out = Box::into_raw(node);
            clear_last_error();
            WASAMO_OK
        }
        Err(e) => {
            set_last_error(format!("wasamo_button_create: {e}"));
            WASAMO_ERR_RUNTIME
        }
    }
}

// Each child entered the ABI as a `Box::into_raw` pointer; we recover
// them via `Box::from_raw` and pass them along to `append_child`, which
// also takes `Box<WidgetNode>`. Flattening to `Vec<WidgetNode>` here
// would force an unbox-rebox round trip per child for no benefit.
#[allow(clippy::vec_box)]
unsafe fn collect_children(
    children: *mut *mut WasamoWidget,
    count: usize,
    fn_name: &str,
) -> Result<Vec<Box<WidgetNode>>, WasamoStatus> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if children.is_null() {
        set_last_error(format!("{fn_name}: children is null but count > 0"));
        return Err(WASAMO_ERR_INVALID_ARG);
    }
    let slice = std::slice::from_raw_parts(children, count);
    // Validate everything before taking ownership of any element so we don't
    // leak halfway through a malformed call.
    for &p in slice {
        if p.is_null() {
            set_last_error(format!("{fn_name}: children[i] is null"));
            return Err(WASAMO_ERR_INVALID_ARG);
        }
    }
    let mut out = Vec::with_capacity(count);
    for &p in slice {
        out.push(Box::from_raw(p));
    }
    Ok(out)
}

// See note on `collect_children` for the `Vec<Box<...>>` shape.
#[allow(clippy::vec_box)]
unsafe fn finish_stack(
    mut node: Box<WidgetNode>,
    children: Vec<Box<WidgetNode>>,
    out: *mut *mut WasamoWidget,
    fn_name: &str,
) -> WasamoStatus {
    for c in children {
        if let Err(e) = node.append_child(c) {
            set_last_error(format!("{fn_name}: append_child failed: {e}"));
            return WASAMO_ERR_RUNTIME;
        }
    }
    *out = Box::into_raw(node);
    clear_last_error();
    WASAMO_OK
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_vstack_create(
    children: *mut *mut WasamoWidget,
    count: usize,
    out: *mut *mut WasamoWidget,
) -> WasamoStatus {
    guard_structural!("wasamo_vstack_create");
    if out.is_null() {
        set_last_error("wasamo_vstack_create: out is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    *out = ptr::null_mut();
    let kids = match collect_children(children, count, "wasamo_vstack_create") {
        Ok(v) => v,
        Err(s) => return s,
    };
    let rt = crate::runtime::get();
    let node = match WidgetNode::vstack(&rt.compositor, 8.0, 8.0, crate::layout::Alignment::Center)
    {
        Ok(n) => n,
        Err(e) => {
            set_last_error(format!("wasamo_vstack_create: {e}"));
            return WASAMO_ERR_RUNTIME;
        }
    };
    finish_stack(node, kids, out, "wasamo_vstack_create")
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_hstack_create(
    children: *mut *mut WasamoWidget,
    count: usize,
    out: *mut *mut WasamoWidget,
) -> WasamoStatus {
    guard_structural!("wasamo_hstack_create");
    if out.is_null() {
        set_last_error("wasamo_hstack_create: out is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    *out = ptr::null_mut();
    let kids = match collect_children(children, count, "wasamo_hstack_create") {
        Ok(v) => v,
        Err(s) => return s,
    };
    let rt = crate::runtime::get();
    let node = match WidgetNode::hstack(&rt.compositor, 8.0, 8.0, crate::layout::Alignment::Center)
    {
        Ok(n) => n,
        Err(e) => {
            set_last_error(format!("wasamo_hstack_create: {e}"));
            return WASAMO_ERR_RUNTIME;
        }
    };
    finish_stack(node, kids, out, "wasamo_hstack_create")
}

/// Recover a boxed ABI handle for a fallible preflight without transferring
/// ownership on rejection.
///
/// # Safety
///
/// `raw` must have been produced by `Box::into_raw`, must be valid and
/// uniquely accessible for the duration of this call, and must not be used by
/// the caller after `Ok` transfers ownership. On `Err`, the returned pointer
/// is the same live allocation and remains owned by the caller.
unsafe fn preflight_boxed_handle<T, E>(
    raw: *mut T,
    preflight: impl FnOnce(&mut T) -> Result<(), E>,
) -> Result<Box<T>, (E, *mut T)> {
    let mut owned = unsafe { Box::from_raw(raw) };
    match preflight(&mut owned) {
        Ok(()) => Ok(owned),
        Err(error) => {
            let restored = Box::into_raw(owned);
            Err((error, restored))
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_window_set_root(
    window: *mut WasamoWindow,
    root: *mut WasamoWidget,
) -> WasamoStatus {
    guard_structural!("wasamo_window_set_root");
    if window.is_null() {
        set_last_error("wasamo_window_set_root: window is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    if root.is_null() {
        set_last_error("wasamo_window_set_root: root is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    // T6 made scale-aware text preparation the first fallible set-root step.
    // Run that step while the ABI can still restore the caller's raw handle:
    // ownership transfers only after successful preparation. `window::set_root`
    // repeats the call, but its raster markers make the second pass a no-op.
    let runtime = crate::runtime::get();
    let root_box = match unsafe {
        preflight_boxed_handle(root, |candidate| {
            candidate.refresh_text_surfaces_recursive(
                &runtime.compositor,
                &runtime.text_renderer,
                (*window).scale,
            )
        })
    } {
        Ok(owned) => owned,
        Err((e, restored)) => {
            debug_assert_eq!(restored, root);
            set_last_error(format!("wasamo_window_set_root: {e}"));
            return WASAMO_ERR_RUNTIME;
        }
    };
    match crate::window::set_root(&mut *window, root_box) {
        Ok(()) => {
            clear_last_error();
            WASAMO_OK
        }
        Err(e) => {
            set_last_error(format!("wasamo_window_set_root: {e}"));
            WASAMO_ERR_RUNTIME
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_button_set_clicked(
    button: *mut WasamoWidget,
    callback: WasamoSignalHandlerFn,
    user_data: *mut std::ffi::c_void,
    destroy_fn: WasamoDestroyFn,
    out_token: *mut u64,
) -> WasamoStatus {
    let name = b"clicked";
    wasamo_signal_connect(
        button,
        name.as_ptr() as *const c_char,
        name.len(),
        callback,
        user_data,
        destroy_fn,
        out_token,
    )
}

// ── DD-M2-P6-005 — wasamo_load_ui ────────────────────────────────────────────
//
// Single-function loader (Option α). The `type` discriminant chooses
// between resource-resolution forms (sub-decision A: filesystem path,
// sub-decision C: in-memory blob); both share the `(data, data_len)`
// shape so a future binary IR can be admitted without ABI breakage.

const DEFAULT_WINDOW_TITLE: &str = "Wasamo";
const DEFAULT_WINDOW_WIDTH: i32 = 800;
const DEFAULT_WINDOW_HEIGHT: i32 = 600;

unsafe fn read_load_payload(
    type_: WasamoLoadType,
    data: *const std::ffi::c_void,
    data_len: usize,
) -> Result<String, (WasamoStatus, String)> {
    if data.is_null() {
        return Err((WASAMO_ERR_INVALID_ARG, "data is null".into()));
    }
    if data_len == 0 {
        return Err((WASAMO_ERR_INVALID_ARG, "data_len must be > 0".into()));
    }
    let bytes = std::slice::from_raw_parts(data as *const u8, data_len);
    match type_ {
        WASAMO_LOAD_PATH => {
            let path = std::str::from_utf8(bytes).map_err(|_| {
                (
                    WASAMO_ERR_INVALID_ARG,
                    "path is not valid UTF-8".to_string(),
                )
            })?;
            std::fs::read_to_string(path).map_err(|e| {
                (
                    WASAMO_ERR_RUNTIME,
                    format!("failed to read IR file `{path}`: {e}"),
                )
            })
        }
        WASAMO_LOAD_MEMORY => {
            // M2 accepts only the IR text grammar (DD-M2-P6-002). The
            // (data, data_len) shape is fixed so a future binary IR can be
            // recognized via header magic without ABI changes; if a binary
            // form is added later, this branch dispatches by sniffing the
            // first bytes.
            std::str::from_utf8(bytes)
                .map(|s| s.to_owned())
                .map_err(|_| {
                    (
                        WASAMO_ERR_IR_MALFORMED,
                        "in-memory IR is not valid UTF-8".to_string(),
                    )
                })
        }
        _ => Err((
            WASAMO_ERR_INVALID_ARG,
            format!("unknown WasamoLoadType: {type_}"),
        )),
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_load_ui(
    type_: WasamoLoadType,
    data: *const std::ffi::c_void,
    data_len: usize,
    out_root: *mut *mut WasamoWindow,
) -> WasamoStatus {
    guard_structural!("wasamo_load_ui");
    if out_root.is_null() {
        set_last_error("wasamo_load_ui: out_root is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    *out_root = ptr::null_mut();

    let ir_text = match read_load_payload(type_, data, data_len) {
        Ok(s) => s,
        Err((status, msg)) => {
            set_last_error(format!("wasamo_load_ui: {msg}"));
            return status;
        }
    };

    let component = match crate::ir_loader::parse_ir(&ir_text) {
        Ok(c) => c,
        Err(e) => {
            set_last_error(format!("wasamo_load_ui: {e}"));
            return if e.is_malformed() {
                WASAMO_ERR_IR_MALFORMED
            } else {
                WASAMO_ERR_RUNTIME
            };
        }
    };

    let rt = crate::runtime::get();
    let built =
        match crate::ir_loader::build_widget_tree(&component, &rt.compositor, &rt.text_renderer) {
            Ok(b) => b,
            Err(e) => {
                set_last_error(format!("wasamo_load_ui: {e}"));
                return if e.is_malformed() {
                    WASAMO_ERR_IR_MALFORMED
                } else {
                    WASAMO_ERR_RUNTIME
                };
            }
        };

    let window_title =
        crate::ir_loader::resolve_static_window_title(&component, DEFAULT_WINDOW_TITLE);

    let mut window =
        match crate::window::create(window_title, DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT) {
            Ok(w) => w,
            Err(e) => {
                set_last_error(format!("wasamo_load_ui: window_create: {e}"));
                return WASAMO_ERR_RUNTIME;
            }
        };

    if let Err(e) = crate::window::set_root(&mut window, built.root) {
        set_last_error(format!("wasamo_load_ui: window_set_root: {e}"));
        return WASAMO_ERR_RUNTIME;
    }

    *out_root = Box::into_raw(window);
    clear_last_error();
    WASAMO_OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactive::RuntimeHealth;
    use std::cell::Cell;
    use std::ffi::CStr;
    use std::ptr;

    fn install_owning_thread() {
        crate::runtime::__install_owning_thread_for_test();
        crate::reactive::set_runtime_health_for_test(RuntimeHealth::Healthy);
        clear_last_error();
    }

    fn last_error_string() -> String {
        let ptr = wasamo_last_error_message();
        assert!(!ptr.is_null(), "last-error should be populated");
        unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .expect("last-error must be UTF-8")
            .to_owned()
    }

    #[test]
    fn guard_placement_after_divergence_matches_abi_roles() {
        install_owning_thread();
        crate::reactive::set_runtime_health_for_test(RuntimeHealth::Diverged);

        wasamo_run();
        let msg = last_error_string();
        assert!(msg.contains("wasamo_run"), "{msg}");
        assert!(msg.contains("Diverged"), "{msg}");

        wasamo_quit();
        let msg = last_error_string();
        assert!(msg.contains("wasamo_quit"), "{msg}");
        assert!(msg.contains("Diverged"), "{msg}");

        let window_status = unsafe { wasamo_window_destroy(ptr::null_mut()) };
        assert_eq!(window_status, WASAMO_OK);

        let widget_status = unsafe { wasamo_widget_destroy(ptr::null_mut()) };
        assert_eq!(widget_status, WASAMO_OK);

        crate::reactive::set_runtime_health_for_test(RuntimeHealth::Healthy);
        clear_last_error();
    }

    #[test]
    fn preflight_boxed_handle_restores_same_live_allocation_on_error() {
        struct DropProbe<'a> {
            drops: &'a Cell<u32>,
            value: u32,
        }

        impl Drop for DropProbe<'_> {
            fn drop(&mut self) {
                self.drops.set(self.drops.get() + 1);
            }
        }

        let drops = Cell::new(0);
        let original = Box::into_raw(Box::new(DropProbe {
            drops: &drops,
            value: 7,
        }));

        let rejected = unsafe {
            preflight_boxed_handle(original, |candidate| {
                candidate.value = 42;
                Err::<(), _>("injected rejection")
            })
        };
        let (error, restored) = match rejected {
            Ok(_) => panic!("preflight must reject"),
            Err(rejected) => rejected,
        };

        assert_eq!(error, "injected rejection");
        assert_eq!(restored, original, "the caller must retain its handle");
        assert_eq!(drops.get(), 0, "rejection must not drop the allocation");
        assert_eq!(unsafe { (*restored).value }, 42, "handle must remain live");

        unsafe { drop(Box::from_raw(restored)) };
        assert_eq!(drops.get(), 1, "caller must be able to destroy it once");
    }
}
