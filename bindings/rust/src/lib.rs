//! `wasamo` — safe Rust bindings for the Wasamo UI framework.
//!
//! # Structure
//!
//! - **Stable-core surface** (this module root): `Runtime`, `Window`,
//!   `Widget`, `Value`, `OwnedValue`, `Connection`, `Error`.
//! - **[`experimental`]** submodule: `text`, `button`, `vstack`,
//!   `hstack` widget constructors and property-ID constants. These
//!   mirror the `WASAMO_EXPERIMENTAL` layer in `wasamo.h` and must be
//!   expected to break in any M2+ release.
//!
//! # ABI contract carried through from the C layer
//!
//! - All types are `!Send + !Sync`: the runtime has strict UI-thread
//!   affinity (`wasamo_init`'s thread owns everything).
//! - Widget handles are lightweight, `Copy`, and remain valid for
//!   property R/W for the lifetime of the window that owns the root
//!   widget tree. After `window.set_root(widget)` or after children
//!   are passed to `vstack`/`hstack`, the allocation is runtime-owned;
//!   the Rust handles remain usable for property updates.
//! - `Runtime::drop` calls `wasamo_shutdown`.
//! - `Window::drop` calls `wasamo_window_destroy`.

use std::ffi::{c_void, CStr};
use std::marker::PhantomData;
use wasamo_sys as sys;

// ── Error ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Error {
    pub status: i32,
    pub message: String,
}

impl Error {
    fn from_status(status: sys::WasamoStatus) -> Self {
        let message = unsafe {
            let ptr = sys::wasamo_last_error_message();
            if ptr.is_null() || *ptr == 0 {
                format!("WasamoStatus({})", status)
            } else {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            }
        };
        Error { status, message }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wasamo error {}: {}", self.status, self.message)
    }
}

impl std::error::Error for Error {}

fn check(status: sys::WasamoStatus) -> Result<(), Error> {
    if status == sys::WASAMO_OK {
        Ok(())
    } else {
        Err(Error::from_status(status))
    }
}

// ── Widget handle ──────────────────────────────────────────────────────

/// Lightweight handle to a runtime-owned widget.
///
/// `Widget` is `Copy`: the raw pointer is a stable identity for the
/// widget as long as its containing window is alive. Multiple copies of
/// the same handle all refer to the same runtime object.
#[derive(Copy, Clone, Debug)]
pub struct Widget {
    raw: *mut sys::WasamoWidget,
    _not_send: PhantomData<*const ()>,
}

impl Widget {
    /// Get a property value from this widget.
    pub fn get_property(&self, property_id: u32) -> Result<OwnedValue, Error> {
        let mut raw_val = sys::WasamoValue {
            tag: sys::WASAMO_VALUE_NONE,
            as_: sys::WasamoValueAs { v_i32: 0 },
        };
        unsafe { check(sys::wasamo_get_property(self.raw, property_id, &mut raw_val))?; }
        Ok(raw_value_to_owned(&raw_val))
    }

    /// Set a property value on this widget.
    pub fn set_property(&self, property_id: u32, value: &Value<'_>) -> Result<(), Error> {
        let raw_val = value_to_raw(value);
        unsafe { check(sys::wasamo_set_property(self.raw, property_id, &raw_val)) }
    }

    /// Register a click handler on a Button widget. **EXPERIMENTAL.**
    ///
    /// The closure runs on the UI thread, never re-entrantly during a
    /// `wasamo_*` call (DD-P6-003 queued-emission guarantee).
    pub fn on_clicked<F>(&self, f: F) -> Connection
    where
        F: FnMut() + 'static,
    {
        extern "C" fn trampoline(
            _sender: *mut sys::WasamoWidget,
            _args: *const sys::WasamoValue,
            _arg_count: usize,
            user_data: *mut c_void,
        ) {
            unsafe {
                let f = &mut *(user_data as *mut Box<dyn FnMut()>);
                f();
            }
        }
        extern "C" fn drop_box(user_data: *mut c_void) {
            unsafe { drop(Box::from_raw(user_data as *mut Box<dyn FnMut()>)); }
        }

        let erased: Box<dyn FnMut()> = Box::new(f);
        let raw_ud = Box::into_raw(Box::new(erased)) as *mut c_void;
        let mut token: u64 = 0;
        let status = unsafe {
            sys::wasamo_button_set_clicked(self.raw, trampoline, raw_ud, Some(drop_box), &mut token)
        };
        if status != sys::WASAMO_OK {
            // Runtime rejected the connect; free the boxed closure now.
            drop_box(raw_ud);
        }
        Connection { token }
    }
}

// ── Value (borrowed) ───────────────────────────────────────────────────

/// A value borrowed for the duration of a property set or callback.
#[derive(Copy, Clone, Debug)]
pub enum Value<'a> {
    None,
    I32(i32),
    F64(f64),
    Bool(bool),
    /// Borrowed UTF-8 string. The runtime copies internally if it retains it.
    String(&'a str),
    Widget(Widget),
}

// ── OwnedValue (returned from get_property) ───────────────────────────

#[derive(Clone, Debug)]
pub enum OwnedValue {
    None,
    I32(i32),
    F64(f64),
    Bool(bool),
    String(String),
    Widget(Widget),
}

// ── Connection ─────────────────────────────────────────────────────────

/// An opaque token identifying a signal or observer connection.
///
/// In M1 Hello Counter the clicked handler lives for the app's lifetime,
/// so no auto-disconnect on drop is implemented. Explicit disconnection
/// will be added in a later phase.
pub struct Connection {
    pub token: u64,
}

// ── Runtime ────────────────────────────────────────────────────────────

/// RAII guard for the Wasamo runtime.
///
/// Calls `wasamo_init` on construction and `wasamo_shutdown` on drop.
/// Must live on the UI thread for the entire duration of the app.
pub struct Runtime {
    _not_send: PhantomData<*const ()>,
}

impl Runtime {
    pub fn init() -> Result<Self, Error> {
        unsafe { check(sys::wasamo_init())? };
        Ok(Runtime { _not_send: PhantomData })
    }

    /// Enter the event loop. Blocks until `quit()` is called.
    pub fn run(&self) {
        unsafe { sys::wasamo_run(); }
    }

    /// Signal the event loop to exit.
    pub fn quit(&self) {
        unsafe { sys::wasamo_quit(); }
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe { sys::wasamo_shutdown(); }
    }
}

// ── Window ─────────────────────────────────────────────────────────────

/// RAII handle to a Wasamo window.
///
/// Calls `wasamo_window_destroy` on drop.
pub struct Window {
    raw: *mut sys::WasamoWindow,
    _not_send: PhantomData<*const ()>,
}

impl Window {
    pub fn create(title: &str, width: i32, height: i32) -> Result<Self, Error> {
        let mut raw = std::ptr::null_mut();
        unsafe {
            check(sys::wasamo_window_create(
                title.as_ptr() as *const _,
                title.len(),
                width,
                height,
                &mut raw,
            ))?;
        }
        Ok(Window { raw, _not_send: PhantomData })
    }

    pub fn show(&self) -> Result<(), Error> {
        unsafe { check(sys::wasamo_window_show(self.raw)) }
    }

    /// Attach a widget tree as the window's root. **EXPERIMENTAL.**
    ///
    /// The runtime takes ownership of the widget tree. Widget handles
    /// remain valid for property R/W.
    pub fn set_root(&self, widget: Widget) -> Result<(), Error> {
        unsafe { check(sys::wasamo_window_set_root(self.raw, widget.raw)) }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { sys::wasamo_window_destroy(self.raw); }
        }
    }
}

// ── experimental module ────────────────────────────────────────────────

/// M1 experimental widget constructors and property-ID constants.
///
/// All symbols here map to `WASAMO_EXPERIMENTAL`-marked entries in
/// `wasamo.h`. Expect breakage in any M2+ release.
///
/// Build widgets bottom-up, pass children to containers by value
/// (the underlying C call moves them into the container). The Rust
/// `Widget` handles remain valid after the call for property R/W.
pub mod experimental {
    use super::*;
    use std::marker::PhantomData;

    pub use sys::{
        WASAMO_BUTTON_LABEL, WASAMO_BUTTON_STYLE, WASAMO_TEXT_CONTENT, WASAMO_TEXT_STYLE,
    };

    pub fn text(content: &str) -> Result<Widget, Error> {
        let mut raw = std::ptr::null_mut();
        unsafe {
            check(sys::wasamo_text_create(
                content.as_ptr() as *const _,
                content.len(),
                &mut raw,
            ))?;
        }
        Ok(Widget { raw, _not_send: PhantomData })
    }

    pub fn button(label: &str) -> Result<Widget, Error> {
        let mut raw = std::ptr::null_mut();
        unsafe {
            check(sys::wasamo_button_create(
                label.as_ptr() as *const _,
                label.len(),
                &mut raw,
            ))?;
        }
        Ok(Widget { raw, _not_send: PhantomData })
    }

    /// Create a vertical stack. Children are consumed: the runtime takes
    /// ownership of the underlying allocations. The Rust `Widget` handles
    /// are still valid for property R/W after this call.
    pub fn vstack(children: &[Widget]) -> Result<Widget, Error> {
        let mut raw_children: Vec<*mut sys::WasamoWidget> =
            children.iter().map(|w| w.raw).collect();
        let mut raw = std::ptr::null_mut();
        unsafe {
            check(sys::wasamo_vstack_create(
                raw_children.as_mut_ptr(),
                raw_children.len(),
                &mut raw,
            ))?;
        }
        Ok(Widget { raw, _not_send: PhantomData })
    }

    /// Create a horizontal stack. Same ownership semantics as [`vstack`].
    pub fn hstack(children: &[Widget]) -> Result<Widget, Error> {
        let mut raw_children: Vec<*mut sys::WasamoWidget> =
            children.iter().map(|w| w.raw).collect();
        let mut raw = std::ptr::null_mut();
        unsafe {
            check(sys::wasamo_hstack_create(
                raw_children.as_mut_ptr(),
                raw_children.len(),
                &mut raw,
            ))?;
        }
        Ok(Widget { raw, _not_send: PhantomData })
    }
}

// ── internal helpers ───────────────────────────────────────────────────

fn value_to_raw(v: &Value<'_>) -> sys::WasamoValue {
    match v {
        Value::None => sys::WasamoValue {
            tag: sys::WASAMO_VALUE_NONE,
            as_: sys::WasamoValueAs { v_i32: 0 },
        },
        Value::I32(x) => sys::WasamoValue {
            tag: sys::WASAMO_VALUE_I32,
            as_: sys::WasamoValueAs { v_i32: *x },
        },
        Value::F64(x) => sys::WasamoValue {
            tag: sys::WASAMO_VALUE_F64,
            as_: sys::WasamoValueAs { v_f64: *x },
        },
        Value::Bool(b) => sys::WasamoValue {
            tag: sys::WASAMO_VALUE_BOOL,
            as_: sys::WasamoValueAs { v_bool: if *b { 1 } else { 0 } },
        },
        Value::String(s) => sys::WasamoValue {
            tag: sys::WASAMO_VALUE_STRING,
            as_: sys::WasamoValueAs {
                v_string: sys::WasamoStringView {
                    ptr: s.as_ptr() as *const _,
                    len: s.len(),
                },
            },
        },
        Value::Widget(w) => sys::WasamoValue {
            tag: sys::WASAMO_VALUE_WIDGET,
            as_: sys::WasamoValueAs { v_widget: w.raw },
        },
    }
}

fn raw_value_to_owned(v: &sys::WasamoValue) -> OwnedValue {
    match v.tag {
        sys::WASAMO_VALUE_I32 => OwnedValue::I32(unsafe { v.as_.v_i32 }),
        sys::WASAMO_VALUE_F64 => OwnedValue::F64(unsafe { v.as_.v_f64 }),
        sys::WASAMO_VALUE_BOOL => OwnedValue::Bool(unsafe { v.as_.v_bool != 0 }),
        sys::WASAMO_VALUE_STRING => {
            let sv = unsafe { v.as_.v_string };
            let s = if sv.ptr.is_null() {
                String::new()
            } else {
                unsafe {
                    let slice = std::slice::from_raw_parts(sv.ptr as *const u8, sv.len);
                    String::from_utf8_lossy(slice).into_owned()
                }
            };
            OwnedValue::String(s)
        }
        sys::WASAMO_VALUE_WIDGET => OwnedValue::Widget(Widget {
            raw: unsafe { v.as_.v_widget },
            _not_send: PhantomData,
        }),
        _ => OwnedValue::None,
    }
}
