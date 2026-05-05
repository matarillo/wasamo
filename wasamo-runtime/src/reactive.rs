use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::handler::{EvalContext, EvalError};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct SignalId(u64);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
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
    /// Nesting depth of `with_batched_writes` (and of the drain pass itself).
    /// While > 0, `Signal::set` defers Effect execution to the outermost exit.
    static BATCH_DEPTH: Cell<u32> = Cell::new(0);
    /// Effects whose dependency changed and have not yet been re-run.
    static DIRTY_EFFECTS: RefCell<HashSet<EffectId>> = RefCell::new(HashSet::new());
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

    // Raise BATCH_DEPTH while the closure runs so that any Signal::set call
    // inside the body enqueues into DIRTY_EFFECTS rather than triggering an
    // immediate (re-entrant) drain.
    BATCH_DEPTH.with(|d| d.set(d.get() + 1));
    GRAPH.with(|g| g.borrow_mut().tracking_stack.push(effect_id));
    (closure_rc.borrow_mut())();
    GRAPH.with(|g| g.borrow_mut().tracking_stack.pop());
    BATCH_DEPTH.with(|d| d.set(d.get() - 1));
}

/// Drain the dirty-Effect set until quiescent, or until the iteration cap is
/// reached.  Each `run_effect` call manages `BATCH_DEPTH` internally, so
/// writes made inside Effect bodies are deferred to the next drain iteration.
fn drain_dirty_effects() {
    const ITERATION_CAP: usize = 16;
    for _ in 0..ITERATION_CAP {
        let dirty: Vec<EffectId> = DIRTY_EFFECTS.with(|d| {
            let mut set = d.borrow_mut();
            let mut v: Vec<EffectId> = set.drain().collect();
            v.sort_unstable();
            v
        });
        if dirty.is_empty() {
            return;
        }
        for effect_id in dirty {
            run_effect(effect_id);
        }
    }
    if DIRTY_EFFECTS.with(|d| !d.borrow().is_empty()) {
        eprintln!(
            "wasamo reactive: drain loop exceeded iteration cap (16); \
             divergent binding detected"
        );
        DIRTY_EFFECTS.with(|d| d.borrow_mut().clear());
    }
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
        let dependents: Vec<EffectId> = GRAPH.with(|g| {
            g.borrow()
                .forward
                .get(&self.id)
                .map(|s| s.iter().copied().collect())
                .unwrap_or_default()
        });
        DIRTY_EFFECTS.with(|d| d.borrow_mut().extend(dependents));
        if BATCH_DEPTH.with(|d| d.get()) == 0 {
            drain_dirty_effects();
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
        // Drain effects that became dirty during the initial run (e.g. re-entrant writes).
        if BATCH_DEPTH.with(|d| d.get()) == 0 {
            drain_dirty_effects();
        }
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
pub(crate) fn with_batched_writes<R, F: FnOnce() -> R>(f: F) -> R {
    BATCH_DEPTH.with(|d| d.set(d.get() + 1));
    let result = f();
    let depth = BATCH_DEPTH.with(|d| {
        let new = d.get() - 1;
        d.set(new);
        new
    });
    if depth == 0 {
        drain_dirty_effects();
    }
    result
}

/// Read-only `EvalContext` adapter for binding expressions (DD-M2-P5-002 = B).
///
/// Wraps a map of `Signal<i32>` instances keyed by property path. Property
/// reads go through `Signal::get()`, which registers the read with the
/// thread-local reactive tracking stack, causing the enclosing Effect (if any)
/// to subscribe to the Signal automatically.
///
/// Write attempts (`set_i32`) always return `EvalError::WriteInBindingContext`;
/// binding expressions are read-only by contract (DD-M2-P5-006 = A).
///
/// The property map is `&HashMap<String, Signal<i32>>` for M2 (integer
/// properties only; the Text `content` binding stringifies the integer result).
/// The next task (`register_binding`) wires this against the real property
/// storage; for now the shape is validated via unit tests with test-local maps.
pub(crate) struct BindingEvalContext<'a> {
    properties: &'a HashMap<String, Signal<i32>>,
}

impl<'a> BindingEvalContext<'a> {
    pub(crate) fn new(properties: &'a HashMap<String, Signal<i32>>) -> Self {
        Self { properties }
    }
}

impl<'a> EvalContext for BindingEvalContext<'a> {
    fn get_i32(&self, path: &str) -> Result<i32, EvalError> {
        self.properties
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
            .map(|s| s.get_untracked())
    }

    fn set_i32(&mut self, path: &str, _value: i32) -> Result<(), EvalError> {
        Err(EvalError::WriteInBindingContext { path: path.to_string() })
    }

    fn read_i32_tracked(&self, path: &str) -> Result<i32, EvalError> {
        self.properties
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
            .map(|s| s.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Returns (in_back, in_forward) — whether the EffectId appears anywhere in either edge map.
    fn graph_traces_of(id: EffectId) -> (bool, bool) {
        GRAPH.with(|g| {
            let g = g.borrow();
            let in_back = g.back.contains_key(&id);
            let in_forward = g.forward.values().any(|deps| deps.contains(&id));
            (in_back, in_forward)
        })
    }

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
        let effect_id = h.id;
        drop(h);
        let (in_back, in_forward) = graph_traces_of(effect_id);
        assert!(!in_back, "EffectId leaked in back-edge map after Drop");
        assert!(!in_forward, "EffectId leaked in forward-edge map after Drop");
        sig.set(1);
        assert_eq!(*count.borrow(), 1);
    }

    #[test]
    fn batched_writes_coalesce_reruns() {
        let sig = Signal::new(0i32);
        let count = Rc::new(RefCell::new(0i32));
        let sig_c = sig.clone();
        let count_c = Rc::clone(&count);
        let _h = EffectHandle::new(move || {
            let _ = sig_c.get();
            *count_c.borrow_mut() += 1;
        });
        assert_eq!(*count.borrow(), 1); // initial run
        with_batched_writes(|| {
            sig.set(1);
            sig.set(2);
            sig.set(3);
        });
        // Three writes inside the batch produce exactly one re-run.
        assert_eq!(*count.borrow(), 2);
        assert_eq!(sig.get_untracked(), 3);
    }

    #[test]
    fn reentrant_write_in_effect_converges() {
        // An Effect that conditionally writes back to its own dependency.
        // The drain loop must re-run it until the condition is false (convergent).
        let sig = Signal::new(0i32);
        let count = Rc::new(RefCell::new(0i32));
        let sig_c = sig.clone();
        let count_c = Rc::clone(&count);
        let _h = EffectHandle::new(move || {
            let v = sig_c.get();
            *count_c.borrow_mut() += 1;
            if v < 3 {
                sig_c.set(v + 1);
            }
        });
        // Runs: v=0 (initial), v=1, v=2, v=3 — 4 runs, then quiesces.
        assert_eq!(*count.borrow(), 4);
        assert_eq!(sig.get_untracked(), 3);
    }

    #[test]
    fn iteration_cap_exhaustion_does_not_hang() {
        // A divergent Effect that always re-enqueues itself is bounded by the
        // drain iteration cap (16). After cap exhaustion the dirty set is
        // cleared and execution continues normally.
        let sig = Signal::new(0i32);
        let count = Rc::new(RefCell::new(0i32));
        let sig_c = sig.clone();
        let count_c = Rc::clone(&count);
        let _h = EffectHandle::new(move || {
            let v = sig_c.get();
            *count_c.borrow_mut() += 1;
            sig_c.set(v.saturating_add(1)); // always dirty — never converges
        });
        // 1 (initial run_effect) + 16 (drain iterations before cap) = 17 bounded runs.
        assert_eq!(*count.borrow(), 17);
    }

    // ── BindingEvalContext tests (DD-M2-P5-006) ───────────────────────────────

    #[test]
    fn binding_ctx_reads_are_tracked() {
        // Wrap a Signal<i32> in BindingEvalContext, evaluate a PropRead binding
        // inside an Effect, then update the Signal and assert the Effect re-ran.
        use crate::handler::{evaluate_binding, HandlerExpr, InterpolationPart};

        let sig = Signal::new(0i32);
        let mut props = std::collections::HashMap::new();
        props.insert("root.count".to_string(), sig.clone());

        let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let log_c = Rc::clone(&log);
        let props_c = props.clone();

        let _h = EffectHandle::new(move || {
            let mut ctx = BindingEvalContext::new(&props_c);
            let expr = HandlerExpr::Interpolation(vec![
                InterpolationPart::Literal("Count: ".into()),
                InterpolationPart::Expr(HandlerExpr::PropRead {
                    path: "root.count".into(),
                }),
            ]);
            let result = evaluate_binding(&expr, &mut ctx).unwrap();
            log_c.borrow_mut().push(result);
        });

        // Initial run.
        assert_eq!(*log.borrow(), vec!["Count: 0"]);

        // Update the Signal → Effect re-runs because read was tracked.
        sig.set(7);
        assert_eq!(*log.borrow(), vec!["Count: 0", "Count: 7"]);
    }

    #[test]
    fn binding_ctx_set_returns_write_error() {
        use crate::handler::EvalError;

        let props = std::collections::HashMap::<String, Signal<i32>>::new();
        let mut ctx = BindingEvalContext::new(&props);
        let result = ctx.set_i32("x", 1);
        assert_eq!(result, Err(EvalError::WriteInBindingContext { path: "x".into() }));
    }

    #[test]
    fn binding_ctx_get_untracked_vs_tracked() {
        // get_i32 must NOT register a dependency; read_i32_tracked must.
        use crate::handler::EvalContext;

        let sig = Signal::new(42i32);
        let mut props = std::collections::HashMap::new();
        props.insert("p".to_string(), sig.clone());

        let run_count = Rc::new(RefCell::new(0i32));
        let run_count_c = Rc::clone(&run_count);
        let props_for_untracked = props.clone();

        // Effect using get_i32 (untracked): Signal update must NOT re-run it.
        let _h_untracked = EffectHandle::new(move || {
            *run_count_c.borrow_mut() += 1;
            let ctx = BindingEvalContext::new(&props_for_untracked);
            let _ = ctx.get_i32("p").unwrap();
        });
        assert_eq!(*run_count.borrow(), 1);
        sig.set(99);
        assert_eq!(*run_count.borrow(), 1, "get_i32 should not register a dependency");

        // Effect using read_i32_tracked: Signal update MUST re-run it.
        let run_count2 = Rc::new(RefCell::new(0i32));
        let run_count2_c = Rc::clone(&run_count2);
        let props2 = props.clone();

        let _h_tracked = EffectHandle::new(move || {
            *run_count2_c.borrow_mut() += 1;
            let ctx = BindingEvalContext::new(&props2);
            let _ = ctx.read_i32_tracked("p").unwrap();
        });
        assert_eq!(*run_count2.borrow(), 1);
        sig.set(100);
        assert_eq!(*run_count2.borrow(), 2, "read_i32_tracked should register a dependency");
    }
}
