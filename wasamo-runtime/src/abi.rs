//! Wasamo C ABI surface. The canonical specification is `docs/abi_spec.md`;
//! ADR `docs/decisions/phase-6-c-abi.md` records the decisions behind it.
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
            // abi_spec §4.3: schedule observers AFTER the call returns.
            // We push to the emission queue here; the actual callbacks
            // fire when `drain_if_outermost` runs at the tail.
            crate::emit::enqueue_property_change(
                widget,
                property_id,
                property_value_to_owned(&pv),
            );
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
    let node = match WidgetNode::vstack(
        &rt.compositor, 8.0, 8.0, crate::layout::Alignment::Center,
    ) {
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
    let node = match WidgetNode::hstack(
        &rt.compositor, 8.0, 8.0, crate::layout::Alignment::Center,
    ) {
        Ok(n) => n,
        Err(e) => {
            set_last_error(format!("wasamo_hstack_create: {e}"));
            return WASAMO_ERR_RUNTIME;
        }
    };
    finish_stack(node, kids, out, "wasamo_hstack_create")
}

#[no_mangle]
pub unsafe extern "C" fn wasamo_window_set_root(
    window: *mut WasamoWindow,
    root: *mut WasamoWidget,
) -> WasamoStatus {
    if window.is_null() {
        set_last_error("wasamo_window_set_root: window is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    if root.is_null() {
        set_last_error("wasamo_window_set_root: root is null");
        return WASAMO_ERR_INVALID_ARG;
    }
    let root_box: Box<WidgetNode> = Box::from_raw(root);
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
