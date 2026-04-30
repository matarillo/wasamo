//! Token-based registry for property observers and signal handlers.
//!
//! Access is UI-thread-only (abi_spec §6); we therefore use `thread_local`
//! storage rather than a `Mutex`/`OnceLock`. Every registration owns three
//! host-supplied pointers — `(callback, user_data, destroy_fn)` per
//! DD-P6-003 — and produces an opaque `u64` token. `destroy_fn` is invoked
//! exactly once when the registration is severed: explicit disconnect,
//! widget destroy, or `wasamo_shutdown`. The runtime never frees
//! `user_data` itself.
//!
//! Callback *invocation* (signal emission, observer firing) is the next
//! Phase 6 item (queued emission). This module only stores entries and
//! tears them down.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;

use crate::abi::{
    WasamoDestroyFn, WasamoPropertyObserverFn, WasamoSignalHandlerFn, WasamoWidget,
};

pub enum EntryKind {
    Observer {
        property_id: u32,
        callback: WasamoPropertyObserverFn,
    },
    Signal {
        name: String,
        callback: WasamoSignalHandlerFn,
    },
}

pub struct Entry {
    pub widget: *mut WasamoWidget,
    pub kind: EntryKind,
    pub user_data: *mut c_void,
    pub destroy_fn: WasamoDestroyFn,
}

struct Registry {
    next_token: u64,
    entries: HashMap<u64, Entry>,
}

impl Registry {
    fn new() -> Self {
        Self { next_token: 1, entries: HashMap::new() }
    }

    fn alloc_token(&mut self) -> u64 {
        let t = self.next_token;
        // Wrap to 1 (reserve 0 as "invalid"). u64 wrap is theoretical.
        self.next_token = self.next_token.checked_add(1).unwrap_or(1);
        t
    }
}

thread_local! {
    static REG: RefCell<Registry> = RefCell::new(Registry::new());
}

fn insert(entry: Entry) -> u64 {
    REG.with(|r| {
        let mut r = r.borrow_mut();
        let t = r.alloc_token();
        r.entries.insert(t, entry);
        t
    })
}

pub fn add_observer(
    widget: *mut WasamoWidget,
    property_id: u32,
    callback: WasamoPropertyObserverFn,
    user_data: *mut c_void,
    destroy_fn: WasamoDestroyFn,
) -> u64 {
    insert(Entry {
        widget,
        kind: EntryKind::Observer { property_id, callback },
        user_data,
        destroy_fn,
    })
}

pub fn add_signal(
    widget: *mut WasamoWidget,
    name: String,
    callback: WasamoSignalHandlerFn,
    user_data: *mut c_void,
    destroy_fn: WasamoDestroyFn,
) -> u64 {
    insert(Entry {
        widget,
        kind: EntryKind::Signal { name, callback },
        user_data,
        destroy_fn,
    })
}

/// Remove a single registration by token. Returns `true` if a matching
/// entry was found and severed (destroy_fn invoked once on the way out).
pub fn remove(token: u64) -> bool {
    let entry = REG.with(|r| r.borrow_mut().entries.remove(&token));
    match entry {
        Some(e) => {
            invoke_destroy(&e);
            true
        }
        None => false,
    }
}

/// Sever every registration owned by `widget`. Used by widget-destroy
/// hooks (`wasamo_window_destroy`, `window::set_root` when replacing
/// the previous root).
pub fn remove_for_widget(widget: *mut WasamoWidget) {
    let drained = REG.with(|r| {
        let mut r = r.borrow_mut();
        let tokens: Vec<u64> = r
            .entries
            .iter()
            .filter(|(_, e)| std::ptr::eq(e.widget, widget))
            .map(|(t, _)| *t)
            .collect();
        tokens
            .into_iter()
            .filter_map(|t| r.entries.remove(&t))
            .collect::<Vec<_>>()
    });
    for e in drained {
        invoke_destroy(&e);
    }
}

/// Drain the entire registry, invoking every destroy_fn exactly once.
/// Called from `wasamo_shutdown`.
pub fn drain_all() {
    let drained: Vec<Entry> = REG.with(|r| {
        let mut r = r.borrow_mut();
        let owned: HashMap<u64, Entry> = std::mem::take(&mut r.entries);
        r.next_token = 1;
        owned.into_values().collect()
    });
    for e in drained {
        invoke_destroy(&e);
    }
}

/// Tokens of every observer matching `(widget, property_id)`, in
/// insertion (token) order so emission preserves connection order.
pub fn observer_tokens_for(widget: *mut WasamoWidget, property_id: u32) -> Vec<u64> {
    REG.with(|r| {
        let r = r.borrow();
        let mut tokens: Vec<u64> = r
            .entries
            .iter()
            .filter(|(_, e)| {
                std::ptr::eq(e.widget, widget)
                    && matches!(
                        e.kind,
                        EntryKind::Observer { property_id: pid, .. } if pid == property_id
                    )
            })
            .map(|(t, _)| *t)
            .collect();
        tokens.sort_unstable();
        tokens
    })
}

/// Tokens of every signal handler matching `(widget, name)`, in
/// insertion order.
pub fn signal_tokens_for(widget: *mut WasamoWidget, name: &str) -> Vec<u64> {
    REG.with(|r| {
        let r = r.borrow();
        let mut tokens: Vec<u64> = r
            .entries
            .iter()
            .filter(|(_, e)| {
                std::ptr::eq(e.widget, widget)
                    && match &e.kind {
                        EntryKind::Signal { name: n, .. } => n == name,
                        _ => false,
                    }
            })
            .map(|(t, _)| *t)
            .collect();
        tokens.sort_unstable();
        tokens
    })
}

/// Resolve an observer token to its callback pointer and identity. Returns
/// `None` if the token has been disconnected since the emission was queued.
pub fn lookup_observer(
    token: u64,
) -> Option<(
    crate::abi::WasamoPropertyObserverFn,
    *mut WasamoWidget,
    u32,
    *mut c_void,
)> {
    REG.with(|r| {
        let r = r.borrow();
        let e = r.entries.get(&token)?;
        match &e.kind {
            EntryKind::Observer { property_id, callback } => {
                Some((*callback, e.widget, *property_id, e.user_data))
            }
            _ => None,
        }
    })
}

/// Resolve a signal token to its callback pointer and identity.
pub fn lookup_signal(
    token: u64,
) -> Option<(crate::abi::WasamoSignalHandlerFn, *mut WasamoWidget, *mut c_void)> {
    REG.with(|r| {
        let r = r.borrow();
        let e = r.entries.get(&token)?;
        match &e.kind {
            EntryKind::Signal { callback, .. } => {
                Some((*callback, e.widget, e.user_data))
            }
            _ => None,
        }
    })
}

fn invoke_destroy(e: &Entry) {
    if let Some(f) = e.destroy_fn {
        // Safety: the host-supplied destroy_fn is __cdecl per DD-P6-007 and
        // takes ownership semantics described in abi_spec §2.3 rule 3.
        unsafe { f(e.user_data) };
    }
}
