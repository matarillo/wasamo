//! `wasamo-sys` — raw FFI declarations for the Wasamo C ABI.
//!
//! Mirrors [`bindings/c/wasamo.h`](../../../bindings/c/wasamo.h). The
//! canonical specification is `docs/abi_spec.md`; this crate is a
//! transliteration, not a re-design.
//!
//! # Scope (DD-P7-004 — Hello-Counter-minimal)
//!
//! Property-change observers (`wasamo_observe_property` /
//! `wasamo_unobserve_property`) and the generic signal connect/disconnect
//! pair (`wasamo_signal_connect` / `wasamo_signal_disconnect`) are
//! intentionally **not** declared in this crate. Hello Counter does not
//! exercise them. They will be added when Phase 8 (or a later phase)
//! demonstrates a concrete consumer.
//!
//! `WasamoPropertyObserverFn` is omitted for the same reason.
//!
//! # ABI notes
//!
//! All public functions and host-supplied callback typedefs in `wasamo.h`
//! are declared `__cdecl` (`WASAMO_API`). On x64 Windows that is the only
//! calling convention and Rust's `extern "C"` matches it; on x86 and
//! ARM64EC the explicit `__cdecl` will be honored once the runtime
//! supports those targets.
//!
//! UTF-8 is the only string encoding accepted or returned. The runtime
//! is strictly UI-thread-affine: every function in this crate must be
//! called from the thread that called `wasamo_init`, except where the
//! C header documents otherwise.

#![allow(non_camel_case_types)]

use std::ffi::{c_char, c_void};

// ─── 3.1 WasamoStatus ──────────────────────────────────────────────────

pub type WasamoStatus = i32;

pub const WASAMO_OK: WasamoStatus = 0;
pub const WASAMO_ERR_INVALID_ARG: WasamoStatus = -1;
pub const WASAMO_ERR_RUNTIME: WasamoStatus = -2;
pub const WASAMO_ERR_NOT_INITIALIZED: WasamoStatus = -3;
pub const WASAMO_ERR_WRONG_THREAD: WasamoStatus = -4;

// ─── 3.2 Opaque handles ────────────────────────────────────────────────

#[repr(C)]
pub struct WasamoWindow {
    _private: [u8; 0],
}

#[repr(C)]
pub struct WasamoWidget {
    _private: [u8; 0],
}

// ─── 3.3 WasamoValue (tagged union) ────────────────────────────────────

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
pub union WasamoValueAs {
    pub v_i32: i32,
    pub v_f64: f64,
    pub v_bool: i32,
    pub v_string: WasamoStringView,
    pub v_widget: *mut WasamoWidget,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct WasamoValue {
    pub tag: WasamoValueTag,
    pub as_: WasamoValueAs,
}

// ─── 3.4 Callback typedefs ─────────────────────────────────────────────

pub type WasamoDestroyFn = unsafe extern "C" fn(user_data: *mut c_void);

pub type WasamoSignalHandlerFn = unsafe extern "C" fn(
    sender: *mut WasamoWidget,
    args: *const WasamoValue,
    arg_count: usize,
    user_data: *mut c_void,
);

// WasamoPropertyObserverFn intentionally omitted — see crate-level docs.

// ─── M1 experimental property-ID constants ─────────────────────────────

pub const WASAMO_BUTTON_LABEL: u32 = 1;
pub const WASAMO_BUTTON_STYLE: u32 = 2;
pub const WASAMO_TEXT_CONTENT: u32 = 3;
pub const WASAMO_TEXT_STYLE: u32 = 4;

// ─── Function declarations ─────────────────────────────────────────────
//
// `destroy_fn` parameters are `Option<WasamoDestroyFn>` because the
// header allows them to be NULL (DD-P6-003: "Hosts that don't need
// cleanup pass NULL for destroy_fn"). All other function pointers are
// non-nullable per the spec.

extern "C" {
    // ── 4.1 Runtime lifecycle ──────────────────────────────────────────
    pub fn wasamo_init() -> WasamoStatus;
    pub fn wasamo_shutdown();
    pub fn wasamo_last_error_message() -> *const c_char;

    // ── 4.2 Window and event loop ──────────────────────────────────────
    pub fn wasamo_window_create(
        title_utf8: *const c_char,
        title_len: usize,
        width: i32,
        height: i32,
        out: *mut *mut WasamoWindow,
    ) -> WasamoStatus;
    pub fn wasamo_window_show(window: *mut WasamoWindow) -> WasamoStatus;
    pub fn wasamo_window_destroy(window: *mut WasamoWindow) -> WasamoStatus;
    pub fn wasamo_run();
    pub fn wasamo_quit();

    // ── 4.3 Property get/set ───────────────────────────────────────────
    pub fn wasamo_get_property(
        widget: *mut WasamoWidget,
        property_id: u32,
        out_value: *mut WasamoValue,
    ) -> WasamoStatus;
    pub fn wasamo_set_property(
        widget: *mut WasamoWidget,
        property_id: u32,
        value: *const WasamoValue,
    ) -> WasamoStatus;

    // 4.4 / 4.5 — observers and generic signal connect/disconnect
    //            intentionally omitted; see crate-level docs.

    // ── 5. M1 experimental layer ───────────────────────────────────────
    pub fn wasamo_text_create(
        content_utf8: *const c_char,
        content_len: usize,
        out: *mut *mut WasamoWidget,
    ) -> WasamoStatus;
    pub fn wasamo_button_create(
        label_utf8: *const c_char,
        label_len: usize,
        out: *mut *mut WasamoWidget,
    ) -> WasamoStatus;
    pub fn wasamo_vstack_create(
        children: *mut *mut WasamoWidget,
        count: usize,
        out: *mut *mut WasamoWidget,
    ) -> WasamoStatus;
    pub fn wasamo_hstack_create(
        children: *mut *mut WasamoWidget,
        count: usize,
        out: *mut *mut WasamoWidget,
    ) -> WasamoStatus;
    pub fn wasamo_window_set_root(
        window: *mut WasamoWindow,
        root: *mut WasamoWidget,
    ) -> WasamoStatus;
    pub fn wasamo_button_set_clicked(
        button: *mut WasamoWidget,
        callback: WasamoSignalHandlerFn,
        user_data: *mut c_void,
        destroy_fn: Option<WasamoDestroyFn>,
        out_token: *mut u64,
    ) -> WasamoStatus;
}

// ─── Link smoke test ───────────────────────────────────────────────────
//
// `cargo test` produces a real executable, so the test binary is what
// actually exercises the build script's link directives. Taking the
// address of an `extern` function is enough to force the linker to
// resolve the symbol against `wasamo.dll.lib`.
//
// We deliberately do not *call* the runtime here: doing so requires
// `wasamo_init` on a UI thread, and the test harness runs from a worker
// thread with no message loop.

#[cfg(test)]
mod link_smoke {
    use super::*;

    #[test]
    fn symbols_resolve() {
        let _: unsafe extern "C" fn() -> WasamoStatus = wasamo_init;
        let _: unsafe extern "C" fn() = wasamo_shutdown;
        let _: unsafe extern "C" fn() -> *const c_char = wasamo_last_error_message;
        let _: unsafe extern "C" fn() = wasamo_run;
        let _: unsafe extern "C" fn() = wasamo_quit;
    }
}
