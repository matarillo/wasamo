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
//!
//! Layout invalidation (DD-P8-002):
//! - `mark_layout_dirty` / `unmark_layout_dirty` register windows
//!   that need a layout pass after the signal queue empties.
//! - After each full drain cycle, all marked windows run one layout
//!   pass. Multiple property changes in one drain cycle coalesce
//!   into a single pass per window.
//! - Window registration (`register_window` / `unregister_window`)
//!   is called by `window::create` / `wasamo_window_destroy` so that
//!   `drain_if_outermost` can reach live windows without an explicit
//!   window pointer in `wasamo_set_property`.

use std::cell::{Cell, RefCell};
use std::collections::{HashSet, VecDeque};
use std::os::raw::c_char;

use crate::abi::{
    WasamoStringView, WasamoValue, WasamoValuePayload, WasamoWidget,
    WASAMO_VALUE_I32, WASAMO_VALUE_STRING,
};
use crate::registry;
use crate::window::WindowState;

// Only the value tags M1 widgets actually emit. The full closed tag set
// (abi_spec §3.3) gets variants added here when future widgets need them;
// adding a tag is non-breaking.
#[derive(Clone)]
pub enum OwnedArg {
    I32(i32),
    String(String),
}

enum Pending {
    Observer { token: u64, value: OwnedArg },
    Signal { token: u64, args: Vec<OwnedArg> },
}

thread_local! {
    static QUEUE: RefCell<VecDeque<Pending>> = const { RefCell::new(VecDeque::new()) };
    static DISPATCHING: Cell<bool> = const { Cell::new(false) };
    // Raw pointers to all live WindowState allocations on this thread.
    // Populated by window::create; removed by wasamo_window_destroy.
    static WINDOWS: RefCell<Vec<*mut WindowState>> = const { RefCell::new(Vec::new()) };
    // Windows marked dirty by a size-affecting set_property call.
    // Holds raw pointers that are always a subset of WINDOWS.
    static DIRTY: RefCell<HashSet<*mut WindowState>> = RefCell::new(HashSet::new());
}

// ── Window registration for layout invalidation ───────────────────────────────

pub fn register_window(window: *mut WindowState) {
    WINDOWS.with(|w| w.borrow_mut().push(window));
}

pub fn unregister_window(window: *mut WindowState) {
    WINDOWS.with(|w| w.borrow_mut().retain(|&p| p != window));
    DIRTY.with(|d| { d.borrow_mut().remove(&window); });
}

/// Called from set_property when a size-affecting property changes.
/// Marks the window that owns `widget` as needing a layout pass.
/// If `widget` is not yet attached to any window, this is a no-op;
/// layout will run when the widget enters a window via `set_root`.
pub fn mark_layout_dirty_for(widget: *mut WasamoWidget) {
    WINDOWS.with(|windows| {
        let windows = windows.borrow();
        for &wptr in windows.iter() {
            // Safety: wptr is a live Box<WindowState> allocated on this thread.
            let state = unsafe { &*wptr };
            if let Some(ref root) = state.root_widget {
                let mut found = false;
                root.for_each_ptr(&mut |p| {
                    if p == widget {
                        found = true;
                    }
                });
                if found {
                    DIRTY.with(|d| { d.borrow_mut().insert(wptr); });
                    return;
                }
            }
        }
    });
}

fn flush_layout() {
    let dirty: Vec<*mut WindowState> = DIRTY.with(|d| d.borrow_mut().drain().collect());
    for wptr in dirty {
        // Safety: wptr lives in WINDOWS and is still a valid Box<WindowState>.
        let state = unsafe { &mut *wptr };
        if let Some(ref mut root) = state.root_widget {
            use windows::Win32::UI::WindowsAndMessaging::GetClientRect;
            use windows::Win32::Foundation::RECT;
            let mut rect = RECT::default();
            let (cw, ch) = unsafe {
                if GetClientRect(state.hwnd, &mut rect).is_ok() {
                    ((rect.right - rect.left) as f32, (rect.bottom - rect.top) as f32)
                } else {
                    (0.0, 0.0)
                }
            };
            let _ = root.run_layout(cw, ch);
        }
    }
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
    // After all callbacks have fired, run one layout pass per dirty window.
    // This coalesces multiple property changes from the same drain cycle.
    flush_layout();
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
            // `args` is held in scope so any v_string pointer in `vals`
            // stays valid through the callback.
            let vals: Vec<WasamoValue> = args.iter().map(owned_to_value).collect();
            let (ptr, len): (*const WasamoValue, usize) = if vals.is_empty() {
                (std::ptr::null(), 0)
            } else {
                (vals.as_ptr(), vals.len())
            };
            unsafe { cb(widget, ptr, len, user_data) };
            drop(args);
        }
    }
}

fn owned_to_value(a: &OwnedArg) -> WasamoValue {
    match a {
        OwnedArg::I32(v) => WasamoValue {
            tag: WASAMO_VALUE_I32,
            payload: WasamoValuePayload { v_i32: *v },
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
    }
}

