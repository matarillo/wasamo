//! Queued callback emission with re-entrancy guard.
//!
//! Spec contract (abi_spec §6): while the host is inside a `wasamo_*`
//! call, the runtime does not invoke any callback on that thread.
//! Emissions triggered by a call are queued and drained at a later
//! safe point.
//!
//! Three-phase drain (DD-M2-P6-001, Option D):
//!
//!   Phase 1 — Mutation convergence loop.
//!     `signal_queue` (FIFO) and `dirty_effects` (topological) are processed
//!     until quiescent or MUTATION_CAP.  State mutation (Signal::set) is
//!     permitted; structure-changing ABI (e.g. `wasamo_load_ui`) returns
//!     `WASAMO_ERR_REENTRANT_LOAD`.  The `IN_DRAIN` TLS flag guards this phase.
//!
//!   Phase 2 — Layout pass (terminal, read-only).
//!     One layout pass per dirty window; coalesces all layout invalidations
//!     from Phase 1.
//!
//!   Phase 3 — Post-commit observer drain.
//!     Queued observer/signal callbacks fire under the `IN_OBSERVER_CALLBACK`
//!     TLS flag.  State-mutating ABI returns `WASAMO_ERR_OBSERVER_MUTATION`.
//!     New emissions enqueued during Phase 3 are deferred to the next drain
//!     cycle.
//!
//! Layout invalidation (DD-P8-002):
//! - `mark_layout_dirty` / `unmark_layout_dirty` register windows
//!   that need a layout pass after Phase 1 empties.
//! - Window registration (`register_window` / `unregister_window`)
//!   is called by `window::create` / `wasamo_window_destroy` so that
//!   `drain_if_outermost` can reach live windows without an explicit
//!   window pointer in `wasamo_set_property`.

use std::cell::{Cell, RefCell};
use std::collections::{HashSet, VecDeque};
use std::os::raw::c_char;

use crate::abi::{
    WasamoStringView, WasamoValue, WasamoValuePayload, WasamoWidget, WASAMO_VALUE_BOOL,
    WASAMO_VALUE_I32, WASAMO_VALUE_STRING,
};
use crate::reactive;
use crate::registry;
use crate::window::WindowState;

// Only the value tags M1 widgets actually emit. The full closed tag set
// (abi_spec §3.3) gets variants added here when future widgets need them;
// adding a tag is non-breaking.
#[derive(Clone)]
pub enum OwnedArg {
    I32(i32),
    Bool(bool),
    String(String),
}

enum Pending {
    Observer { token: u64, value: OwnedArg },
    Signal { token: u64, args: Vec<OwnedArg> },
}

thread_local! {
    static QUEUE: RefCell<VecDeque<Pending>> = const { RefCell::new(VecDeque::new()) };
    /// Set while Phase 1 (mutation convergence) is executing.
    /// Structure-changing ABI calls check this and return WASAMO_ERR_REENTRANT_LOAD.
    static IN_DRAIN: Cell<bool> = const { Cell::new(false) };
    /// Set while Phase 3 (observer drain) is executing.
    /// State-mutating ABI calls check this and return WASAMO_ERR_OBSERVER_MUTATION.
    pub(crate) static IN_OBSERVER_CALLBACK: Cell<bool> = const { Cell::new(false) };
    // Raw pointers to all live WindowState allocations on this thread.
    // Populated by window::create; removed by wasamo_window_destroy.
    static WINDOWS: RefCell<Vec<*mut WindowState>> = const { RefCell::new(Vec::new()) };
    // Windows marked dirty by size-affecting property writes or structural
    // subtree mutations.
    // Holds raw pointers that are always a subset of WINDOWS.
    static DIRTY: RefCell<HashSet<*mut WindowState>> = RefCell::new(HashSet::new());
}

pub(crate) fn in_drain() -> bool {
    IN_DRAIN.with(|f| f.get())
}

pub(crate) fn in_observer_callback() -> bool {
    IN_OBSERVER_CALLBACK.with(|f| f.get())
}

// ── Window registration for layout invalidation ───────────────────────────────

pub fn register_window(window: *mut WindowState) {
    WINDOWS.with(|w| w.borrow_mut().push(window));
}

pub fn unregister_window(window: *mut WindowState) {
    WINDOWS.with(|w| w.borrow_mut().retain(|&p| p != window));
    DIRTY.with(|d| {
        d.borrow_mut().remove(&window);
    });
}

/// Called when a size-affecting property changes or a live subtree mutates.
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
                    DIRTY.with(|d| {
                        d.borrow_mut().insert(wptr);
                    });
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
            use windows::Win32::Foundation::RECT;
            use windows::Win32::UI::WindowsAndMessaging::GetClientRect;
            // DD-M4-P1-002 audit row 2's **second** production site (M4-Phase 1
            // T1 finding F-1): the ADR names only `window::set_root`, but the
            // reactive drain's layout phase performs the same
            // `GetClientRect` -> layout conversion after every size-affecting
            // property write, and an unconverted extent here would hand layout
            // physical pixels on the busiest path in the runtime.
            //
            // `state.scale` and `state.hwnd` are read beside the `root_widget`
            // borrow above: disjoint fields of the same struct, so this needs no
            // restructuring. (T1's note that `update_button_label` must read the
            // scale *before* `button_data_mut()` is about a method that borrows
            // all of `self`, and does not apply to a field access.)
            let mut rect = RECT::default();
            let (cw, ch) = unsafe {
                if GetClientRect(state.hwnd, &mut rect).is_ok() {
                    state.scale.pair_to_dip((
                        (rect.right - rect.left) as f32,
                        (rect.bottom - rect.top) as f32,
                    ))
                } else {
                    (0.0, 0.0)
                }
            };
            let _ = root.run_layout(cw, ch);
        }
    }
}

pub fn enqueue_property_change(widget: *mut WasamoWidget, property_id: u32, value: OwnedArg) {
    let tokens = registry::observer_tokens_for(widget, property_id);
    if tokens.is_empty() {
        return;
    }
    QUEUE.with(|q| {
        let mut q = q.borrow_mut();
        for t in tokens {
            q.push_back(Pending::Observer {
                token: t,
                value: value.clone(),
            });
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
            q.push_back(Pending::Signal {
                token: t,
                args: args.clone(),
            });
        }
    });
}

/// Three-phase drain (Option D, DD-M2-P6-001).
///
/// Re-entry while IN_DRAIN is set is a no-op: the outer loop keeps executing
/// and will process any entries enqueued by the nested call.
/// In Diverged state all three phases are suppressed (ADR §divergence
/// commit-forbidden conditions).
pub fn drain_if_outermost() {
    if IN_DRAIN.with(|f| f.get()) {
        return;
    }
    if reactive::runtime_health() == reactive::RuntimeHealth::Diverged {
        return;
    }

    // Phase 1 — mutation convergence loop.
    // reactive::drain_reactive runs dirty Effects (topological order) until
    // quiescent or MUTATION_CAP. Writes inside Effects enqueue further observer
    // notifications (processed in Phase 3) and mark layout dirty (Phase 2).
    IN_DRAIN.with(|f| f.set(true));
    reactive::drain_reactive();
    IN_DRAIN.with(|f| f.set(false));

    // Phase 2 — layout pass (terminal, read-only).
    // Coalesces all layout invalidations from Phase 1 into a single pass per window.
    flush_layout();

    // Phase 3 — post-commit observer drain.
    // Callbacks fire under IN_OBSERVER_CALLBACK; state-mutating ABI calls are
    // rejected. New enqueues during Phase 3 land in QUEUE for the next cycle.
    IN_OBSERVER_CALLBACK.with(|f| f.set(true));
    loop {
        let next = QUEUE.with(|q| q.borrow_mut().pop_front());
        match next {
            Some(p) => dispatch(p),
            None => break,
        }
    }
    IN_OBSERVER_CALLBACK.with(|f| f.set(false));
}

fn dispatch(p: Pending) {
    match p {
        Pending::Observer { token, value } => {
            let Some((cb, widget, prop_id, user_data)) = registry::lookup_observer(token) else {
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
            let Some((cb, widget, user_data)) = registry::lookup_signal(token) else {
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
        OwnedArg::Bool(b) => WasamoValue {
            tag: WASAMO_VALUE_BOOL,
            payload: WasamoValuePayload {
                v_bool: if *b { 1 } else { 0 },
            },
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactive::RuntimeHealth;

    fn reset_emit_state() {
        IN_DRAIN.with(|f| f.set(false));
        IN_OBSERVER_CALLBACK.with(|f| f.set(false));
        QUEUE.with(|q| q.borrow_mut().clear());
        DIRTY.with(|d| d.borrow_mut().clear());
        crate::reactive::set_runtime_health_for_test(RuntimeHealth::Healthy);
    }

    fn queued_count() -> usize {
        QUEUE.with(|q| q.borrow().len())
    }

    #[test]
    fn drain_if_outermost_suppresses_all_phases_after_divergence() {
        reset_emit_state();
        QUEUE.with(|q| {
            q.borrow_mut().push_back(Pending::Signal {
                token: 0,
                args: Vec::new(),
            });
        });

        crate::reactive::set_runtime_health_for_test(RuntimeHealth::Diverged);
        drain_if_outermost();

        assert_eq!(
            queued_count(),
            1,
            "Diverged internal boundary must not drain queued callbacks"
        );
        assert!(!in_drain(), "Diverged suppression must not enter Phase 1");
        assert!(
            !in_observer_callback(),
            "Diverged suppression must not enter Phase 3"
        );

        reset_emit_state();
    }

    #[test]
    fn drain_if_outermost_reentrant_call_is_noop() {
        reset_emit_state();
        QUEUE.with(|q| {
            q.borrow_mut().push_back(Pending::Signal {
                token: 0,
                args: Vec::new(),
            });
        });

        IN_DRAIN.with(|f| f.set(true));
        drain_if_outermost();

        assert_eq!(
            queued_count(),
            1,
            "nested drain must leave work for the outer drain"
        );
        assert!(
            in_drain(),
            "nested drain must not clear the outer Phase 1 flag"
        );

        reset_emit_state();
    }
}
