//! Queued callback emission with re-entrancy guard.
//!
//! Spec contract (abi_spec §6): while the host is inside a `wasamo_*`
//! call, the runtime does not invoke any callback on that thread.
//! Emissions triggered by a call are queued and drained at a later
//! safe point.
//!
//! Implementation sketch:
//! - `enqueue_property_change` / `enqueue_signal` resolve matching
//!   tokens through `registry` and push `Pending` entries onto a
//!   thread-local FIFO. They never invoke callbacks themselves.
//! - `drain_if_outermost` is called from every public ABI entry's
//!   tail and from `wasamo_run`'s message loop between dispatches.
//!   Re-entry into it is a no-op (the outer loop keeps popping, so
//!   emissions queued by callbacks fire in the same drain cycle).
//! - At dispatch time tokens are re-resolved through `registry`;
//!   if a handler disconnected itself or another handler before its
//!   turn, the lookup returns `None` and we skip — this is the
//!   "disconnect-during-emission" semantics required by §4.4.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::os::raw::c_char;

use crate::abi::{
    WasamoStringView, WasamoValue, WasamoValuePayload, WasamoWidget,
    WASAMO_VALUE_BOOL, WASAMO_VALUE_F64, WASAMO_VALUE_I32, WASAMO_VALUE_NONE,
    WASAMO_VALUE_STRING, WASAMO_VALUE_WIDGET,
};
use crate::registry;

// Variants F64/Bool/Widget/None are part of the closed M4 tag set
// (abi_spec §3.3); they aren't produced by M1 widgets but the
// dispatcher must round-trip them once future widgets emit them.
#[allow(dead_code)]
#[derive(Clone)]
pub enum OwnedArg {
    None,
    I32(i32),
    F64(f64),
    Bool(bool),
    String(String),
    Widget(*mut WasamoWidget),
}

enum Pending {
    Observer { token: u64, value: OwnedArg },
    Signal { token: u64, args: Vec<OwnedArg> },
}

thread_local! {
    static QUEUE: RefCell<VecDeque<Pending>> = RefCell::new(VecDeque::new());
    static DISPATCHING: Cell<bool> = const { Cell::new(false) };
}

pub fn enqueue_property_change(
    widget: *mut WasamoWidget,
    property_id: u32,
    value: OwnedArg,
) {
    let tokens = registry::observer_tokens_for(widget, property_id);
    if tokens.is_empty() {
        return;
    }
    QUEUE.with(|q| {
        let mut q = q.borrow_mut();
        for t in tokens {
            q.push_back(Pending::Observer { token: t, value: value.clone() });
        }
    });
}

pub fn enqueue_signal(widget: *mut WasamoWidget, name: &str, args: Vec<OwnedArg>) {
    let tokens = registry::signal_tokens_for(widget, name);
    if tokens.is_empty() {
        return;
    }
    QUEUE.with(|q| {
        let mut q = q.borrow_mut();
        for t in tokens {
            q.push_back(Pending::Signal { token: t, args: args.clone() });
        }
    });
}

pub fn drain_if_outermost() {
    if DISPATCHING.with(|d| d.get()) {
        return;
    }
    DISPATCHING.with(|d| d.set(true));
    loop {
        let next = QUEUE.with(|q| q.borrow_mut().pop_front());
        match next {
            Some(p) => dispatch(p),
            None => break,
        }
    }
    DISPATCHING.with(|d| d.set(false));
}

fn dispatch(p: Pending) {
    match p {
        Pending::Observer { token, value } => {
            let Some((cb, widget, prop_id, user_data)) =
                registry::lookup_observer(token)
            else {
                return;
            };
            let Some(cb) = cb else { return };
            let v = owned_to_value(&value);
            // Safety: callback is __cdecl per DD-P6-007; pointer in
            // v_string (if any) backs onto `value` which lives until
            // this function returns.
            unsafe { cb(widget, prop_id, &v, user_data) };
        }
        Pending::Signal { token, args } => {
            let Some((cb, widget, user_data)) = registry::lookup_signal(token)
            else {
                return;
            };
            let Some(cb) = cb else { return };
            let vals: Vec<WasamoValue> = args.iter().map(owned_to_value).collect();
            let (ptr, len): (*const WasamoValue, usize) = if vals.is_empty() {
                (std::ptr::null(), 0)
            } else {
                (vals.as_ptr(), vals.len())
            };
            unsafe { cb(widget, ptr, len, user_data) };
            // `args` and `vals` drop here, after the callback returns,
            // keeping any string buffers alive for the call's duration.
            drop(args);
            drop(vals);
            let _ = user_data;
        }
    }
}

fn owned_to_value(a: &OwnedArg) -> WasamoValue {
    match a {
        OwnedArg::None => WasamoValue {
            tag: WASAMO_VALUE_NONE,
            payload: WasamoValuePayload { v_i32: 0 },
        },
        OwnedArg::I32(v) => WasamoValue {
            tag: WASAMO_VALUE_I32,
            payload: WasamoValuePayload { v_i32: *v },
        },
        OwnedArg::F64(v) => WasamoValue {
            tag: WASAMO_VALUE_F64,
            payload: WasamoValuePayload { v_f64: *v },
        },
        OwnedArg::Bool(b) => WasamoValue {
            tag: WASAMO_VALUE_BOOL,
            payload: WasamoValuePayload { v_bool: if *b { 1 } else { 0 } },
        },
        OwnedArg::String(s) => WasamoValue {
            tag: WASAMO_VALUE_STRING,
            payload: WasamoValuePayload {
                v_string: WasamoStringView {
                    ptr: s.as_ptr() as *const c_char,
                    len: s.len(),
                },
            },
        },
        OwnedArg::Widget(w) => WasamoValue {
            tag: WASAMO_VALUE_WIDGET,
            payload: WasamoValuePayload { v_widget: *w },
        },
    }
}

