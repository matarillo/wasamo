use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct SignalId(u64);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct EffectId(u64);

static NEXT_SIGNAL_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_EFFECT_ID: AtomicU64 = AtomicU64::new(1);

fn next_signal_id() -> SignalId {
    SignalId(NEXT_SIGNAL_ID.fetch_add(1, Ordering::Relaxed))
}

fn next_effect_id() -> EffectId {
    EffectId(NEXT_EFFECT_ID.fetch_add(1, Ordering::Relaxed))
}

struct ReactiveGraph {
    forward: HashMap<SignalId, HashSet<EffectId>>,
    back: HashMap<EffectId, HashSet<SignalId>>,
    tracking_stack: Vec<EffectId>,
    closures: HashMap<EffectId, Weak<RefCell<Box<dyn FnMut()>>>>,
}

impl ReactiveGraph {
    fn new() -> Self {
        Self {
            forward: HashMap::new(),
            back: HashMap::new(),
            tracking_stack: Vec::new(),
            closures: HashMap::new(),
        }
    }

    fn track_read(&mut self, signal_id: SignalId) {
        if let Some(&effect_id) = self.tracking_stack.last() {
            self.forward.entry(signal_id).or_default().insert(effect_id);
            self.back.entry(effect_id).or_default().insert(signal_id);
        }
    }
}

thread_local! {
    static GRAPH: RefCell<ReactiveGraph> = RefCell::new(ReactiveGraph::new());
}

fn run_effect(effect_id: EffectId) {
    // Clear stale deps so this run can repopulate from scratch.
    GRAPH.with(|g| {
        let mut g = g.borrow_mut();
        let old_sigs = g.back.remove(&effect_id).unwrap_or_default();
        for sig_id in &old_sigs {
            if let Some(deps) = g.forward.get_mut(sig_id) {
                deps.remove(&effect_id);
            }
        }
    });

    let closure_rc =
        GRAPH.with(|g| g.borrow().closures.get(&effect_id).and_then(|w| w.upgrade()));
    let Some(closure_rc) = closure_rc else { return };

    GRAPH.with(|g| g.borrow_mut().tracking_stack.push(effect_id));
    (closure_rc.borrow_mut())();
    GRAPH.with(|g| g.borrow_mut().tracking_stack.pop());
}

pub(crate) struct Signal<T> {
    id: SignalId,
    value: Rc<RefCell<T>>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal { id: self.id, value: Rc::clone(&self.value) }
    }
}

impl<T: Clone + 'static> Signal<T> {
    pub(crate) fn new(value: T) -> Self {
        Signal { id: next_signal_id(), value: Rc::new(RefCell::new(value)) }
    }

    pub(crate) fn get(&self) -> T {
        GRAPH.with(|g| g.borrow_mut().track_read(self.id));
        self.value.borrow().clone()
    }

    pub(crate) fn get_untracked(&self) -> T {
        self.value.borrow().clone()
    }

    pub(crate) fn set(&self, value: T) {
        *self.value.borrow_mut() = value;
        // with_batched_writes (next task) will replace this with dirty-set enqueue.
        let dependents: Vec<EffectId> = GRAPH.with(|g| {
            g.borrow()
                .forward
                .get(&self.id)
                .map(|s| s.iter().copied().collect())
                .unwrap_or_default()
        });
        for effect_id in dependents {
            run_effect(effect_id);
        }
    }
}

pub(crate) struct EffectHandle {
    id: EffectId,
    _closure: Rc<RefCell<Box<dyn FnMut()>>>,
}

impl EffectHandle {
    pub(crate) fn new<F: FnMut() + 'static>(f: F) -> Self {
        let id = next_effect_id();
        let closure: Rc<RefCell<Box<dyn FnMut()>>> = Rc::new(RefCell::new(Box::new(f)));
        GRAPH.with(|g| g.borrow_mut().closures.insert(id, Rc::downgrade(&closure)));
        run_effect(id);
        EffectHandle { id, _closure: closure }
    }
}

impl Drop for EffectHandle {
    fn drop(&mut self) {
        GRAPH.with(|g| {
            let mut g = g.borrow_mut();
            g.closures.remove(&self.id);
            if let Some(old_sigs) = g.back.remove(&self.id) {
                for sig_id in &old_sigs {
                    if let Some(deps) = g.forward.get_mut(sig_id) {
                        deps.remove(&self.id);
                    }
                }
            }
        });
    }
}

/// Execute `f` with writes batched: invalidation cascades triggered inside
/// `f` are deferred until `f` returns, then flushed once.
///
/// Phase 5 next task: fill in depth counter + dirty-Effect drain.
pub(crate) fn with_batched_writes<R, F: FnOnce() -> R>(f: F) -> R {
    f()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_set_invalidates_dependents() {
        let sig = Signal::new(0i32);
        let count = Rc::new(RefCell::new(0i32));
        let sig_c = sig.clone();
        let count_c = Rc::clone(&count);
        let _h = EffectHandle::new(move || {
            let _ = sig_c.get();
            *count_c.borrow_mut() += 1;
        });
        assert_eq!(*count.borrow(), 1);
        sig.set(42);
        assert_eq!(*count.borrow(), 2);
    }

    #[test]
    fn effect_repopulates_deps_each_run() {
        let sig_a = Signal::new(0i32);
        let sig_b = Signal::new(0i32);
        let use_a = Signal::new(true);
        let count = Rc::new(RefCell::new(0i32));

        let sig_a_c = sig_a.clone();
        let sig_b_c = sig_b.clone();
        let use_a_c = use_a.clone();
        let count_c = Rc::clone(&count);

        let _h = EffectHandle::new(move || {
            *count_c.borrow_mut() += 1;
            if use_a_c.get() {
                let _ = sig_a_c.get();
            } else {
                let _ = sig_b_c.get();
            }
        });

        assert_eq!(*count.borrow(), 1);
        sig_a.set(1);
        assert_eq!(*count.borrow(), 2);

        use_a.set(false);
        assert_eq!(*count.borrow(), 3);

        sig_a.set(2);
        assert_eq!(*count.borrow(), 3); // sig_a no longer tracked

        sig_b.set(1);
        assert_eq!(*count.borrow(), 4);
    }

    #[test]
    fn nested_effect_stack_isolation() {
        // Inner effect created during outer effect's run tracks only its own
        // signals; sig_inner reads must not be attributed to the outer effect.
        let sig_outer = Signal::new(0i32);
        let sig_inner = Signal::new(0i32);
        let count_outer = Rc::new(RefCell::new(0i32));
        let count_inner = Rc::new(RefCell::new(0i32));
        let inner_handle: Rc<RefCell<Option<EffectHandle>>> = Rc::new(RefCell::new(None));

        let sig_outer_c = sig_outer.clone();
        let sig_inner_c = sig_inner.clone();
        let count_outer_c = Rc::clone(&count_outer);
        let count_inner_c = Rc::clone(&count_inner);
        let inner_c = Rc::clone(&inner_handle);
        let created = Rc::new(RefCell::new(false));

        let _outer = EffectHandle::new(move || {
            let _ = sig_outer_c.get();
            *count_outer_c.borrow_mut() += 1;
            if !*created.borrow() {
                *created.borrow_mut() = true;
                let sig_inner_cc = sig_inner_c.clone();
                let count_inner_cc = Rc::clone(&count_inner_c);
                let h = EffectHandle::new(move || {
                    let _ = sig_inner_cc.get();
                    *count_inner_cc.borrow_mut() += 1;
                });
                *inner_c.borrow_mut() = Some(h);
            }
        });

        assert_eq!(*count_outer.borrow(), 1);
        assert_eq!(*count_inner.borrow(), 1);

        sig_inner.set(1);
        assert_eq!(*count_outer.borrow(), 1);
        assert_eq!(*count_inner.borrow(), 2);

        sig_outer.set(1);
        assert_eq!(*count_outer.borrow(), 2);
        assert_eq!(*count_inner.borrow(), 2);
    }

    #[test]
    fn get_untracked_does_not_record_dependency() {
        let sig = Signal::new(0i32);
        let count = Rc::new(RefCell::new(0i32));
        let sig_c = sig.clone();
        let count_c = Rc::clone(&count);
        let _h = EffectHandle::new(move || {
            let _ = sig_c.get_untracked();
            *count_c.borrow_mut() += 1;
        });
        assert_eq!(*count.borrow(), 1);
        sig.set(99);
        assert_eq!(*count.borrow(), 1);
    }

    #[test]
    fn dropped_handle_stops_effect() {
        let sig = Signal::new(0i32);
        let count = Rc::new(RefCell::new(0i32));
        let sig_c = sig.clone();
        let count_c = Rc::clone(&count);
        let h = EffectHandle::new(move || {
            let _ = sig_c.get();
            *count_c.borrow_mut() += 1;
        });
        assert_eq!(*count.borrow(), 1);
        drop(h);
        sig.set(1);
        assert_eq!(*count.borrow(), 1);
    }
}
