use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::handler::{
    evaluate_binding, evaluate_binding_optional, evaluate_bool_binding,
    evaluate_bool_binding_optional, EvalContext, EvalError, HandlerExpr,
};
use wasamo_ir::IrType;

const MUTATION_CAP: usize = 16;

/// Health state of the reactive engine. Once `Diverged`, all ABI calls
/// (except destroy) must return `WASAMO_ERR_REACTIVE_DIVERGED`.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum RuntimeHealth {
    Healthy,
    Diverged,
}

/// Diagnostics captured when the drain loop exceeds `MUTATION_CAP`.
#[derive(Clone, Debug)]
pub(crate) struct DivergenceDiagnostics {
    pub(crate) offending_effect_id: u64,
    pub(crate) iteration_count: usize,
    pub(crate) last_dirty_signal_ids: Vec<u64>,
}

thread_local! {
    static HEALTH: Cell<RuntimeHealth> = const { Cell::new(RuntimeHealth::Healthy) };
    static DIVERGENCE_DIAG: RefCell<Option<DivergenceDiagnostics>> = const { RefCell::new(None) };
}

pub(crate) fn runtime_health() -> RuntimeHealth {
    HEALTH.with(|h| h.get())
}

#[cfg(test)]
pub(crate) fn set_runtime_health_for_test(health: RuntimeHealth) {
    HEALTH.with(|h| h.set(health));
    if health == RuntimeHealth::Healthy {
        DIVERGENCE_DIAG.with(|d| *d.borrow_mut() = None);
    }
}

pub(crate) fn divergence_diagnostics() -> Option<DivergenceDiagnostics> {
    DIVERGENCE_DIAG.with(|d| d.borrow().clone())
}

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
    writes: HashMap<EffectId, HashSet<SignalId>>,
    tracking_stack: Vec<EffectId>,
    closures: HashMap<EffectId, Weak<RefCell<Box<dyn FnMut()>>>>,
}

impl ReactiveGraph {
    fn new() -> Self {
        Self {
            forward: HashMap::new(),
            back: HashMap::new(),
            writes: HashMap::new(),
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

    fn track_write(&mut self, signal_id: SignalId) {
        if let Some(&effect_id) = self.tracking_stack.last() {
            self.writes.entry(effect_id).or_default().insert(signal_id);
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
        g.writes.remove(&effect_id);
        for sig_id in &old_sigs {
            if let Some(deps) = g.forward.get_mut(sig_id) {
                deps.remove(&effect_id);
            }
        }
    });

    let closure_rc = GRAPH.with(|g| {
        g.borrow()
            .closures
            .get(&effect_id)
            .and_then(|w| w.upgrade())
    });
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
///
/// If the cap is exceeded the runtime transitions irreversibly to `Diverged`
/// and records diagnostics in `DIVERGENCE_DIAG`.
fn drain_dirty_effects() {
    for iteration in 0..MUTATION_CAP {
        let dirty: Vec<EffectId> = DIRTY_EFFECTS.with(|d| {
            let mut set = d.borrow_mut();
            let dirty: HashSet<EffectId> = set.drain().collect();
            GRAPH.with(|g| {
                let g = g.borrow();
                order_dirty_effects_topologically(&g.forward, &g.back, &g.writes, &dirty)
            })
        });
        if dirty.is_empty() {
            return;
        }
        if iteration + 1 == MUTATION_CAP {
            // Capture diagnostics before running the final batch so we know
            // which effect caused the last-iteration dirty signals.
            let offending_effect_id = dirty.first().map(|e| e.0).unwrap_or(0);
            let last_dirty_signals: Vec<u64> = GRAPH.with(|g| {
                let g = g.borrow();
                dirty
                    .iter()
                    .flat_map(|eid| g.back.get(eid).into_iter().flatten().map(|s| s.0))
                    .collect()
            });
            // Run the effects so they can dirty signals again.
            for effect_id in &dirty {
                run_effect(*effect_id);
            }
            // Check whether effects are still dirty after the final iteration.
            let still_dirty = DIRTY_EFFECTS.with(|d| !d.borrow().is_empty());
            if still_dirty {
                DIRTY_EFFECTS.with(|d| d.borrow_mut().clear());
                HEALTH.with(|h| h.set(RuntimeHealth::Diverged));
                let diag = DivergenceDiagnostics {
                    offending_effect_id,
                    iteration_count: MUTATION_CAP,
                    last_dirty_signal_ids: last_dirty_signals,
                };
                DIVERGENCE_DIAG.with(|d| *d.borrow_mut() = Some(diag.clone()));
                let msg = format!(
                    "wasamo reactive: reactive divergence after {} iterations; \
                     offending Effect id={}; last dirty Signal ids={:?}",
                    diag.iteration_count, diag.offending_effect_id, diag.last_dirty_signal_ids,
                );
                crate::abi::set_last_error(msg);
            }
            return;
        }
        for effect_id in dirty {
            run_effect(effect_id);
        }
    }
}

pub(crate) struct Signal<T> {
    id: SignalId,
    value: Rc<RefCell<T>>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal {
            id: self.id,
            value: Rc::clone(&self.value),
        }
    }
}

impl<T: Clone + 'static> Signal<T> {
    pub(crate) fn new(value: T) -> Self {
        Signal {
            id: next_signal_id(),
            value: Rc::new(RefCell::new(value)),
        }
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
            let mut g = g.borrow_mut();
            g.track_write(self.id);
            g.forward
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

impl<T: Clone + PartialEq + 'static> Signal<T> {
    pub(crate) fn set_if_changed(&self, value: T) -> bool {
        if *self.value.borrow() == value {
            return false;
        }
        self.set(value);
        true
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
        EffectHandle {
            id,
            _closure: closure,
        }
    }
}

impl Drop for EffectHandle {
    fn drop(&mut self) {
        GRAPH.with(|g| {
            let mut g = g.borrow_mut();
            g.closures.remove(&self.id);
            g.writes.remove(&self.id);
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

fn order_dirty_effects_topologically(
    forward: &HashMap<SignalId, HashSet<EffectId>>,
    back: &HashMap<EffectId, HashSet<SignalId>>,
    writes: &HashMap<EffectId, HashSet<SignalId>>,
    dirty: &HashSet<EffectId>,
) -> Vec<EffectId> {
    let mut outgoing: HashMap<EffectId, HashSet<EffectId>> = HashMap::new();
    let mut indegree: HashMap<EffectId, usize> = dirty.iter().map(|&id| (id, 0)).collect();

    for &writer in dirty {
        let Some(written_signals) = writes.get(&writer) else {
            continue;
        };
        for signal_id in written_signals {
            let Some(readers) = forward.get(signal_id) else {
                continue;
            };
            for &reader in readers {
                if reader == writer || !dirty.contains(&reader) {
                    continue;
                }
                if !back
                    .get(&reader)
                    .is_some_and(|signals| signals.contains(signal_id))
                {
                    continue;
                }
                if outgoing.entry(writer).or_default().insert(reader) {
                    *indegree.entry(reader).or_insert(0) += 1;
                }
            }
        }
    }

    let mut ready: Vec<EffectId> = indegree
        .iter()
        .filter_map(|(&id, &degree)| (degree == 0).then_some(id))
        .collect();
    ready.sort_unstable();

    let mut ordered = Vec::with_capacity(dirty.len());
    while let Some(effect_id) = ready.first().copied() {
        ready.remove(0);
        ordered.push(effect_id);

        let mut children: Vec<EffectId> = outgoing
            .get(&effect_id)
            .into_iter()
            .flatten()
            .copied()
            .collect();
        children.sort_unstable();
        for child in children {
            let Some(degree) = indegree.get_mut(&child) else {
                continue;
            };
            *degree -= 1;
            if *degree == 0 {
                ready.push(child);
            }
        }
        ready.sort_unstable();
    }

    if ordered.len() != dirty.len() {
        let mut remaining: Vec<EffectId> = dirty
            .iter()
            .copied()
            .filter(|id| !ordered.contains(id))
            .collect();
        remaining.sort_unstable();
        ordered.extend(remaining);
    }

    ordered
}

/// Flush all dirty Effects (called from emit::drain_if_outermost, DD-M2-P5-004 = B).
/// Runs inside a batched-writes context so that writes made by Effect bodies
/// are deferred and coalesced before the next drain iteration.
pub(crate) fn drain_reactive() {
    with_batched_writes(|| {});
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

/// Per-type Signal storage keyed by `wasamoc`-resolved state names (DD-M2-P6-007).
///
/// M2 supports `i32` and `String` typed Signals; M3 type expansion adds fields
/// without changing the registration call site.
///
pub(crate) struct SignalRegistry {
    pub(crate) i32s: HashMap<String, Signal<i32>>,
    pub(crate) strings: HashMap<String, Signal<String>>,
    pub(crate) bools: HashMap<String, Signal<bool>>,
    pub(crate) i32_lists: HashMap<String, Signal<Vec<i32>>>,
    pub(crate) string_lists: HashMap<String, Signal<Vec<String>>>,
    pub(crate) bool_lists: HashMap<String, Signal<Vec<bool>>>,
}

impl SignalRegistry {
    pub(crate) fn new() -> Self {
        Self {
            i32s: HashMap::new(),
            strings: HashMap::new(),
            bools: HashMap::new(),
            i32_lists: HashMap::new(),
            string_lists: HashMap::new(),
            bool_lists: HashMap::new(),
        }
    }
}

/// Read-only `EvalContext` adapter for binding expressions (DD-M2-P5-002 = B).
///
/// Wraps a `SignalRegistry` reference. `i32` property reads go through
/// `Signal::get()`, which registers the read with the thread-local reactive
/// tracking stack, causing the enclosing Effect (if any) to subscribe
/// automatically.
///
/// Write attempts (`set_i32`) always return `EvalError::WriteInBindingContext`;
/// binding expressions are read-only by contract (DD-M2-P5-006 = A).
pub(crate) struct BindingEvalContext<'a> {
    registry: &'a SignalRegistry,
}

impl<'a> BindingEvalContext<'a> {
    pub(crate) fn new(registry: &'a SignalRegistry) -> Self {
        Self { registry }
    }
}

impl<'a> EvalContext for BindingEvalContext<'a> {
    fn get_i32(&self, path: &str) -> Result<i32, EvalError> {
        self.registry
            .i32s
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
            .map(|s| s.get_untracked())
    }

    fn get_string(&self, path: &str) -> Result<String, EvalError> {
        self.registry
            .strings
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
            .map(|s| s.get_untracked())
    }

    fn get_bool(&self, path: &str) -> Result<bool, EvalError> {
        self.registry
            .bools
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
            .map(|s| s.get_untracked())
    }

    fn set_i32(&mut self, path: &str, _value: i32) -> Result<(), EvalError> {
        Err(EvalError::WriteInBindingContext {
            path: path.to_string(),
        })
    }

    fn set_bool(&mut self, path: &str, _value: bool) -> Result<(), EvalError> {
        Err(EvalError::WriteInBindingContext {
            path: path.to_string(),
        })
    }

    fn read_i32_tracked(&self, path: &str) -> Result<i32, EvalError> {
        self.registry
            .i32s
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
            .map(|s| s.get())
    }

    fn read_string_tracked(&self, path: &str) -> Result<String, EvalError> {
        self.registry
            .strings
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
            .map(|s| s.get())
    }

    fn read_bool_tracked(&self, path: &str) -> Result<bool, EvalError> {
        self.registry
            .bools
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
            .map(|s| s.get())
    }
}

/// Read/write `EvalContext` adapter used by inline handler evaluation
/// (DD-M2-P6-006). Reads are *untracked* — handlers run outside the reactive
/// scope, so dependency collection is not desired here. Writes mutate the
/// underlying `Signal`, triggering reactive cascade through the existing
/// `Signal::set` path.
pub(crate) struct HandlerEvalContext<'a> {
    registry: &'a SignalRegistry,
}

impl<'a> HandlerEvalContext<'a> {
    pub(crate) fn new(registry: &'a SignalRegistry) -> Self {
        Self { registry }
    }
}

impl<'a> EvalContext for HandlerEvalContext<'a> {
    fn get_i32(&self, path: &str) -> Result<i32, EvalError> {
        self.registry
            .i32s
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
            .map(|s| s.get_untracked())
    }

    fn get_string(&self, path: &str) -> Result<String, EvalError> {
        self.registry
            .strings
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
            .map(|s| s.get_untracked())
    }

    fn get_bool(&self, path: &str) -> Result<bool, EvalError> {
        self.registry
            .bools
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
            .map(|s| s.get_untracked())
    }

    fn set_i32(&mut self, path: &str, value: i32) -> Result<(), EvalError> {
        let sig = self
            .registry
            .i32s
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))?;
        sig.set(value);
        Ok(())
    }

    fn set_bool(&mut self, path: &str, value: bool) -> Result<(), EvalError> {
        let sig = self
            .registry
            .bools
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))?;
        sig.set(value);
        Ok(())
    }

    fn collection_element_type(&self, path: &str) -> Result<IrType, EvalError> {
        if self.registry.i32_lists.contains_key(path) {
            Ok(IrType::I32)
        } else if self.registry.string_lists.contains_key(path) {
            Ok(IrType::Str)
        } else if self.registry.bool_lists.contains_key(path) {
            Ok(IrType::Bool)
        } else {
            Err(EvalError::UnknownProperty(path.to_string()))
        }
    }

    fn get_i32_list(&self, path: &str) -> Result<Vec<i32>, EvalError> {
        self.registry
            .i32_lists
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
            .map(|s| s.get_untracked())
    }

    fn set_i32_list(&mut self, path: &str, value: Vec<i32>) -> Result<bool, EvalError> {
        let sig = self
            .registry
            .i32_lists
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))?;
        Ok(sig.set_if_changed(value))
    }

    fn get_string_list(&self, path: &str) -> Result<Vec<String>, EvalError> {
        self.registry
            .string_lists
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
            .map(|s| s.get_untracked())
    }

    fn set_string_list(&mut self, path: &str, value: Vec<String>) -> Result<bool, EvalError> {
        let sig = self
            .registry
            .string_lists
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))?;
        Ok(sig.set_if_changed(value))
    }

    fn get_bool_list(&self, path: &str) -> Result<Vec<bool>, EvalError> {
        self.registry
            .bool_lists
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
            .map(|s| s.get_untracked())
    }

    fn set_bool_list(&mut self, path: &str, value: Vec<bool>) -> Result<bool, EvalError> {
        let sig = self
            .registry
            .bool_lists
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))?;
        Ok(sig.set_if_changed(value))
    }
}

#[derive(Clone)]
pub(crate) struct ForItemContext {
    pub(crate) collection: String,
    pub(crate) elem: IrType,
    pub(crate) binder: String,
    pub(crate) index_binder: Option<String>,
    pub(crate) position: usize,
}

pub(crate) struct ForItemEvalContext<'a> {
    registry: &'a SignalRegistry,
    item: &'a ForItemContext,
}

impl<'a> ForItemEvalContext<'a> {
    pub(crate) fn new(registry: &'a SignalRegistry, item: &'a ForItemContext) -> Self {
        Self { registry, item }
    }

    fn ensure_item_binder(&self, binder: &str, expected: IrType) -> Result<(), EvalError> {
        if binder != self.item.binder {
            return Err(EvalError::UnknownProperty(binder.to_string()));
        }
        if self.item.elem != expected {
            return Err(EvalError::TypeMismatch {
                path: binder.to_string(),
            });
        }
        Ok(())
    }
}

impl<'a> EvalContext for ForItemEvalContext<'a> {
    fn get_i32(&self, path: &str) -> Result<i32, EvalError> {
        BindingEvalContext::new(self.registry).get_i32(path)
    }

    fn get_string(&self, path: &str) -> Result<String, EvalError> {
        BindingEvalContext::new(self.registry).get_string(path)
    }

    fn get_bool(&self, path: &str) -> Result<bool, EvalError> {
        BindingEvalContext::new(self.registry).get_bool(path)
    }

    fn set_i32(&mut self, path: &str, _value: i32) -> Result<(), EvalError> {
        Err(EvalError::WriteInBindingContext {
            path: path.to_string(),
        })
    }

    fn set_bool(&mut self, path: &str, _value: bool) -> Result<(), EvalError> {
        Err(EvalError::WriteInBindingContext {
            path: path.to_string(),
        })
    }

    fn read_i32_tracked(&self, path: &str) -> Result<i32, EvalError> {
        BindingEvalContext::new(self.registry).read_i32_tracked(path)
    }

    fn read_string_tracked(&self, path: &str) -> Result<String, EvalError> {
        BindingEvalContext::new(self.registry).read_string_tracked(path)
    }

    fn read_bool_tracked(&self, path: &str) -> Result<bool, EvalError> {
        BindingEvalContext::new(self.registry).read_bool_tracked(path)
    }

    fn read_item_i32_tracked(&self, binder: &str) -> Result<Option<i32>, EvalError> {
        self.ensure_item_binder(binder, IrType::I32)?;
        let signal = self
            .registry
            .i32_lists
            .get(&self.item.collection)
            .ok_or_else(|| EvalError::UnknownProperty(self.item.collection.clone()))?;
        Ok(signal.get().get(self.item.position).copied())
    }

    fn read_item_string_tracked(&self, binder: &str) -> Result<Option<String>, EvalError> {
        self.ensure_item_binder(binder, IrType::Str)?;
        let signal = self
            .registry
            .string_lists
            .get(&self.item.collection)
            .ok_or_else(|| EvalError::UnknownProperty(self.item.collection.clone()))?;
        Ok(signal.get().get(self.item.position).cloned())
    }

    fn read_item_bool_tracked(&self, binder: &str) -> Result<Option<bool>, EvalError> {
        self.ensure_item_binder(binder, IrType::Bool)?;
        let signal = self
            .registry
            .bool_lists
            .get(&self.item.collection)
            .ok_or_else(|| EvalError::UnknownProperty(self.item.collection.clone()))?;
        Ok(signal.get().get(self.item.position).copied())
    }

    fn read_item_binding_tracked(&self, binder: &str) -> Result<Option<String>, EvalError> {
        match self.item.elem {
            IrType::I32 => self
                .read_item_i32_tracked(binder)
                .map(|v| v.map(|n| n.to_string())),
            IrType::Str => self.read_item_string_tracked(binder),
            IrType::Bool => Err(EvalError::TypeMismatch {
                path: binder.to_string(),
            }),
        }
    }

    fn read_index_tracked(&self, binder: &str) -> Result<Option<i32>, EvalError> {
        match self.item.index_binder.as_deref() {
            Some(index) if index == binder => Ok(Some(self.item.position as i32)),
            _ => Err(EvalError::UnknownProperty(binder.to_string())),
        }
    }
}

// ── Active SignalRegistry handoff (DD-M2-P6-006) ────────────────────────────
//
// The IR loader (`ir_loader::build_widget_tree`) installs the per-component
// SignalRegistry here so click-handler dispatch (`widget::hit_test_click`)
// can reach it without threading the registry through every WidgetNode method.
// Single-threaded GUI; one component per runtime in M2 — M3 may swap this for
// per-window scoping when multi-window support lands.

thread_local! {
    static ACTIVE_REGISTRY: RefCell<Option<Rc<SignalRegistry>>> = const { RefCell::new(None) };
}

pub(crate) fn set_active_registry(registry: Rc<SignalRegistry>) {
    ACTIVE_REGISTRY.with(|r| *r.borrow_mut() = Some(registry));
}

pub(crate) fn active_registry() -> Option<Rc<SignalRegistry>> {
    ACTIVE_REGISTRY.with(|r| r.borrow().clone())
}

// ── Binding registration (DD-M2-P5-005) ──────────────────────────────────────

/// Opaque handle to a widget node. Avoids a circular import between reactive.rs
/// and widget.rs by erasing the concrete `WidgetNode` type. The caller (widget.rs
/// or a future loader) is responsible for casting back to the real pointer.
#[derive(Copy, Clone)]
pub(crate) struct WidgetId(pub(crate) *mut ());

// SAFETY: wasamo-runtime is single-threaded GUI; WidgetId is only used on the
// UI thread. The `*mut ()` makes the struct !Send/!Sync, matching that contract.
unsafe impl Send for WidgetId {}

/// Property key — corresponds to the PROP_* constants in widget.rs.
pub(crate) type PropertyKey = u32;

/// The binding target — what gets written when a reactive binding re-evaluates.
pub(crate) enum BindingTarget {
    /// Write to a widget property identified by its node pointer and property id.
    WidgetProperty { node: WidgetId, prop: PropertyKey },
    /// Structurally insert/remove one conditional subtree under a parent.
    ConditionalSubtree {
        parent: WidgetId,
        declared_member_index: usize,
    },
    /// Structurally reconcile the generated range under one `for` slot.
    ForLoopSubtree {
        parent: WidgetId,
        declared_member_index: usize,
    },
}

/// Register a reactive binding that evaluates `expr` against `registry` and calls
/// `write_fn(node, prop, value)` whenever a tracked Signal changes.
///
/// `write_fn` is a plain function pointer (not a closure) so that `reactive.rs`
/// does not need to import `widget::WidgetNode`. The concrete implementation
/// lives in `widget.rs`.
pub(crate) fn register_binding(
    target: BindingTarget,
    expr: HandlerExpr,
    registry: Rc<SignalRegistry>,
    write_fn: fn(WidgetId, PropertyKey, &str),
) -> EffectHandle {
    let BindingTarget::WidgetProperty { node, prop } = target else {
        panic!("register_binding called with non-property target");
    };
    register_binding_with_writer(
        Box::new(move |value: String| write_fn(node, prop, &value)),
        expr,
        registry,
    )
}

/// Core: build an `EffectHandle` whose closure evaluates `expr` and pipes the
/// `String` result to `writer`. Shared between production (`register_binding`)
/// and unit tests (which supply a mock writer).
fn register_binding_with_writer(
    mut writer: Box<dyn FnMut(String)>,
    expr: HandlerExpr,
    registry: Rc<SignalRegistry>,
) -> EffectHandle {
    EffectHandle::new(move || {
        let mut ctx = BindingEvalContext::new(&registry);
        match evaluate_binding(&expr, &mut ctx) {
            Ok(value) => writer(value),
            Err(e) => eprintln!("wasamo: binding eval error: {e}"),
        }
    })
}

/// Bool-typed counterpart of `register_binding` (DD-M3-P1-007 Option A).
///
/// The loader selects this entry point when the target property's declared
/// `IrType` is `Bool` (per DD-M3-P1-009's `resolve_prop_key` widening). The
/// reactive engine itself stays type-agnostic — the per-type seam lives at
/// the call site here, not inside the engine. `write_fn` is a plain function
/// pointer paired with `widget::widget_write_property_bool`.
pub(crate) fn register_bool_binding(
    target: BindingTarget,
    expr: HandlerExpr,
    registry: Rc<SignalRegistry>,
    write_fn: fn(WidgetId, PropertyKey, bool),
) -> EffectHandle {
    let BindingTarget::WidgetProperty { node, prop } = target else {
        panic!("register_bool_binding called with non-property target");
    };
    register_bool_binding_with_writer(
        Box::new(move |value: bool| write_fn(node, prop, value)),
        expr,
        registry,
    )
}

/// Core: build an `EffectHandle` whose closure evaluates `expr` through
/// `evaluate_bool_binding` and pipes the `bool` result to `writer`. Shared
/// between production (`register_bool_binding`) and unit tests.
fn register_bool_binding_with_writer(
    mut writer: Box<dyn FnMut(bool)>,
    expr: HandlerExpr,
    registry: Rc<SignalRegistry>,
) -> EffectHandle {
    EffectHandle::new(move || {
        let mut ctx = BindingEvalContext::new(&registry);
        match evaluate_bool_binding(&expr, &mut ctx) {
            Ok(value) => writer(value),
            Err(e) => eprintln!("wasamo: binding eval error: {e}"),
        }
    })
}

pub(crate) fn register_for_item_binding(
    target: BindingTarget,
    expr: HandlerExpr,
    registry: Rc<SignalRegistry>,
    item: ForItemContext,
    write_fn: fn(WidgetId, PropertyKey, &str),
) -> EffectHandle {
    let BindingTarget::WidgetProperty { node, prop } = target else {
        panic!("register_for_item_binding called with non-property target");
    };
    let writer = move |value: String| write_fn(node, prop, &value);
    EffectHandle::new(move || {
        let mut ctx = ForItemEvalContext::new(&registry, &item);
        match evaluate_binding_optional(&expr, &mut ctx) {
            Ok(Some(value)) => writer(value),
            Ok(None) => {}
            Err(e) => eprintln!("wasamo: for-item binding eval error: {e}"),
        }
    })
}

pub(crate) fn register_for_item_bool_binding(
    target: BindingTarget,
    expr: HandlerExpr,
    registry: Rc<SignalRegistry>,
    item: ForItemContext,
    write_fn: fn(WidgetId, PropertyKey, bool),
) -> EffectHandle {
    let BindingTarget::WidgetProperty { node, prop } = target else {
        panic!("register_for_item_bool_binding called with non-property target");
    };
    let writer = move |value: bool| write_fn(node, prop, value);
    EffectHandle::new(move || {
        let mut ctx = ForItemEvalContext::new(&registry, &item);
        match evaluate_bool_binding_optional(&expr, &mut ctx) {
            Ok(Some(value)) => writer(value),
            Ok(None) => {}
            Err(e) => eprintln!("wasamo: for-item binding eval error: {e}"),
        }
    })
}

pub(crate) fn register_conditional_binding(
    target: BindingTarget,
    expr: HandlerExpr,
    registry: Rc<SignalRegistry>,
    mut mutate_fn: impl FnMut(WidgetId, usize, bool) + 'static,
) -> EffectHandle {
    let BindingTarget::ConditionalSubtree {
        parent,
        declared_member_index,
    } = target
    else {
        panic!("register_conditional_binding called with non-conditional target");
    };
    EffectHandle::new(move || {
        let mut ctx = BindingEvalContext::new(&registry);
        match evaluate_bool_binding(&expr, &mut ctx) {
            Ok(value) => mutate_fn(parent, declared_member_index, value),
            Err(e) => eprintln!("wasamo: conditional binding eval error: {e}"),
        }
    })
}

pub(crate) fn register_for_loop_binding(
    target: BindingTarget,
    collection: HandlerExpr,
    registry: Rc<SignalRegistry>,
    mut mutate_fn: impl FnMut(WidgetId, usize, usize) + 'static,
) -> EffectHandle {
    let BindingTarget::ForLoopSubtree {
        parent,
        declared_member_index,
    } = target
    else {
        panic!("register_for_loop_binding called with non-for-loop target");
    };
    EffectHandle::new(
        move || match collection_len_tracked(&collection, &registry) {
            Ok(len) => mutate_fn(parent, declared_member_index, len),
            Err(e) => eprintln!("wasamo: for-loop binding eval error: {e}"),
        },
    )
}

fn collection_len_tracked(
    collection: &HandlerExpr,
    registry: &SignalRegistry,
) -> Result<usize, EvalError> {
    let HandlerExpr::ListPropRead { path, elem } = collection else {
        return Err(EvalError::TypeMismatch {
            path: "<non-list expression in for-loop binding>".into(),
        });
    };
    match elem {
        IrType::I32 => registry
            .i32_lists
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.clone()))
            .map(|signal| signal.get().len()),
        IrType::Str => registry
            .string_lists
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.clone()))
            .map(|signal| signal.get().len()),
        IrType::Bool => registry
            .bool_lists
            .get(path)
            .ok_or_else(|| EvalError::UnknownProperty(path.clone()))
            .map(|signal| signal.get().len()),
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

    fn add_synthetic_edge(
        forward: &mut HashMap<SignalId, HashSet<EffectId>>,
        back: &mut HashMap<EffectId, HashSet<SignalId>>,
        writes: &mut HashMap<EffectId, HashSet<SignalId>>,
        writer: EffectId,
        signal: SignalId,
        reader: EffectId,
    ) {
        writes.entry(writer).or_default().insert(signal);
        forward.entry(signal).or_default().insert(reader);
        back.entry(reader).or_default().insert(signal);
    }

    fn synthetic_order(
        forward: &HashMap<SignalId, HashSet<EffectId>>,
        back: &HashMap<EffectId, HashSet<SignalId>>,
        writes: &HashMap<EffectId, HashSet<SignalId>>,
        dirty: &[EffectId],
    ) -> Vec<EffectId> {
        let dirty = dirty.iter().copied().collect();
        order_dirty_effects_topologically(forward, back, writes, &dirty)
    }

    #[test]
    fn topo_walk_orders_chain() {
        let a = EffectId(1);
        let b = EffectId(2);
        let c = EffectId(3);
        let ab = SignalId(10);
        let bc = SignalId(11);
        let mut forward = HashMap::new();
        let mut back = HashMap::new();
        let mut writes = HashMap::new();

        add_synthetic_edge(&mut forward, &mut back, &mut writes, a, ab, b);
        add_synthetic_edge(&mut forward, &mut back, &mut writes, b, bc, c);

        assert_eq!(
            synthetic_order(&forward, &back, &writes, &[c, b, a]),
            vec![a, b, c]
        );
    }

    #[test]
    fn topo_walk_orders_diamond_with_deterministic_ties() {
        let a = EffectId(1);
        let b = EffectId(2);
        let c = EffectId(3);
        let d = EffectId(4);
        let ab = SignalId(10);
        let ac = SignalId(11);
        let bd = SignalId(12);
        let cd = SignalId(13);
        let mut forward = HashMap::new();
        let mut back = HashMap::new();
        let mut writes = HashMap::new();

        add_synthetic_edge(&mut forward, &mut back, &mut writes, a, ab, b);
        add_synthetic_edge(&mut forward, &mut back, &mut writes, a, ac, c);
        add_synthetic_edge(&mut forward, &mut back, &mut writes, b, bd, d);
        add_synthetic_edge(&mut forward, &mut back, &mut writes, c, cd, d);

        assert_eq!(
            synthetic_order(&forward, &back, &writes, &[d, c, b, a]),
            vec![a, b, c, d]
        );
    }

    #[test]
    fn topo_walk_orders_fan_out_wider_than_mutation_cap() {
        let root = EffectId(1);
        let mut forward = HashMap::new();
        let mut back = HashMap::new();
        let mut writes = HashMap::new();
        let mut dirty = vec![root];

        for offset in 0..=MUTATION_CAP {
            let child = EffectId(10 + offset as u64);
            let signal = SignalId(100 + offset as u64);
            add_synthetic_edge(&mut forward, &mut back, &mut writes, root, signal, child);
            dirty.push(child);
        }

        let ordered = synthetic_order(&forward, &back, &writes, &dirty);
        assert_eq!(ordered.first(), Some(&root));
        assert_eq!(ordered.len(), MUTATION_CAP + 2);
        assert_eq!(&ordered[1..], &dirty[1..]);
    }

    #[test]
    fn topo_walk_handles_out_of_id_order_dependency() {
        let downstream_smaller_id = EffectId(1);
        let upstream_larger_id = EffectId(20);
        let signal = SignalId(10);
        let mut forward = HashMap::new();
        let mut back = HashMap::new();
        let mut writes = HashMap::new();

        add_synthetic_edge(
            &mut forward,
            &mut back,
            &mut writes,
            upstream_larger_id,
            signal,
            downstream_smaller_id,
        );

        assert_eq!(
            synthetic_order(
                &forward,
                &back,
                &writes,
                &[downstream_smaller_id, upstream_larger_id],
            ),
            vec![upstream_larger_id, downstream_smaller_id],
        );
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
        assert!(
            !in_forward,
            "EffectId leaked in forward-edge map after Drop"
        );
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
        // Wrap a Signal<i32> in BindingEvalContext via SignalRegistry, evaluate a
        // PropRead binding inside an Effect, then update the Signal and assert the
        // Effect re-ran.
        use crate::handler::{evaluate_binding, HandlerExpr, InterpolationPart};

        let sig = Signal::new(0i32);
        let mut registry = SignalRegistry::new();
        registry.i32s.insert("root.count".to_string(), sig.clone());
        let registry = Rc::new(registry);

        let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let log_c = Rc::clone(&log);
        let registry_c = Rc::clone(&registry);

        let _h = EffectHandle::new(move || {
            let mut ctx = BindingEvalContext::new(&registry_c);
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
    fn binding_ctx_string_reads_are_tracked() {
        use crate::handler::{evaluate_binding, HandlerExpr};

        let sig = Signal::new("hello".to_string());
        let mut registry = SignalRegistry::new();
        registry.strings.insert("label".to_string(), sig.clone());
        let registry = Rc::new(registry);

        let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let log_c = Rc::clone(&log);
        let registry_c = Rc::clone(&registry);

        let _h = EffectHandle::new(move || {
            let mut ctx = BindingEvalContext::new(&registry_c);
            let expr = HandlerExpr::StrPropRead {
                path: "label".into(),
            };
            let result = evaluate_binding(&expr, &mut ctx).unwrap();
            log_c.borrow_mut().push(result);
        });

        assert_eq!(*log.borrow(), vec!["hello"]);

        sig.set("world".to_string());
        assert_eq!(*log.borrow(), vec!["hello", "world"]);
    }

    #[test]
    fn binding_ctx_set_returns_write_error() {
        use crate::handler::EvalError;

        let registry = SignalRegistry::new();
        let mut ctx = BindingEvalContext::new(&registry);
        let result = ctx.set_i32("x", 1);
        assert_eq!(
            result,
            Err(EvalError::WriteInBindingContext { path: "x".into() })
        );
    }

    #[test]
    fn binding_ctx_get_untracked_vs_tracked() {
        // get_i32 must NOT register a dependency; read_i32_tracked must.
        use crate::handler::EvalContext;

        let sig = Signal::new(42i32);
        let mut registry = SignalRegistry::new();
        registry.i32s.insert("p".to_string(), sig.clone());
        let registry = Rc::new(registry);

        let run_count = Rc::new(RefCell::new(0i32));
        let run_count_c = Rc::clone(&run_count);
        let registry_untracked = Rc::clone(&registry);

        // Effect using get_i32 (untracked): Signal update must NOT re-run it.
        let _h_untracked = EffectHandle::new(move || {
            *run_count_c.borrow_mut() += 1;
            let ctx = BindingEvalContext::new(&registry_untracked);
            let _ = ctx.get_i32("p").unwrap();
        });
        assert_eq!(*run_count.borrow(), 1);
        sig.set(99);
        assert_eq!(
            *run_count.borrow(),
            1,
            "get_i32 should not register a dependency"
        );

        // Effect using read_i32_tracked: Signal update MUST re-run it.
        let run_count2 = Rc::new(RefCell::new(0i32));
        let run_count2_c = Rc::clone(&run_count2);
        let registry_tracked = Rc::clone(&registry);

        let _h_tracked = EffectHandle::new(move || {
            *run_count2_c.borrow_mut() += 1;
            let ctx = BindingEvalContext::new(&registry_tracked);
            let _ = ctx.read_i32_tracked("p").unwrap();
        });
        assert_eq!(*run_count2.borrow(), 1);
        sig.set(100);
        assert_eq!(
            *run_count2.borrow(),
            2,
            "read_i32_tracked should register a dependency"
        );
    }

    #[test]
    fn binding_ctx_get_bool_untracked_vs_tracked() {
        // M3-Phase 1 T7: parallels the i32/String tracked-read tests for
        // the new bool surface. `get_bool` must NOT register a dependency;
        // `read_bool_tracked` must.
        use crate::handler::EvalContext;

        let sig = Signal::new(false);
        let mut registry = SignalRegistry::new();
        registry.bools.insert("ready".to_string(), sig.clone());
        let registry = Rc::new(registry);

        let run_count = Rc::new(RefCell::new(0i32));
        let run_count_c = Rc::clone(&run_count);
        let registry_untracked = Rc::clone(&registry);

        let _h_untracked = EffectHandle::new(move || {
            *run_count_c.borrow_mut() += 1;
            let ctx = BindingEvalContext::new(&registry_untracked);
            let _ = ctx.get_bool("ready").unwrap();
        });
        assert_eq!(*run_count.borrow(), 1);
        sig.set(true);
        assert_eq!(
            *run_count.borrow(),
            1,
            "get_bool should not register a dependency"
        );

        let run_count2 = Rc::new(RefCell::new(0i32));
        let run_count2_c = Rc::clone(&run_count2);
        let registry_tracked = Rc::clone(&registry);

        let _h_tracked = EffectHandle::new(move || {
            *run_count2_c.borrow_mut() += 1;
            let ctx = BindingEvalContext::new(&registry_tracked);
            let _ = ctx.read_bool_tracked("ready").unwrap();
        });
        assert_eq!(*run_count2.borrow(), 1);
        sig.set(false);
        assert_eq!(
            *run_count2.borrow(),
            2,
            "read_bool_tracked should register a dependency"
        );
    }

    #[test]
    fn binding_ctx_set_bool_returns_write_error() {
        use crate::handler::EvalError;

        let registry = SignalRegistry::new();
        let mut ctx = BindingEvalContext::new(&registry);
        let result = ctx.set_bool("ready", true);
        assert_eq!(
            result,
            Err(EvalError::WriteInBindingContext {
                path: "ready".into()
            })
        );
    }

    /// `HandlerEvalContext::set_bool` drives `Signal<bool>::set`, the path
    /// the live `Assign { rhs: BoolLit | BoolPropRead }` evaluator arm
    /// from T7 takes when an inline `on click { ready = false }` handler
    /// fires (DD-M3-P1-008 Option A).
    #[test]
    fn handler_ctx_set_bool_drives_signal_set() {
        use crate::handler::{evaluate, EvalContext, HandlerExpr};

        let sig = Signal::new(true);
        let mut registry = SignalRegistry::new();
        registry.bools.insert("ready".to_string(), sig.clone());

        // Confirm an effect tracking `ready` re-runs after the handler
        // write — proves the reactive cascade fires on bool writes too.
        let run_count = Rc::new(RefCell::new(0i32));
        let run_count_c = Rc::clone(&run_count);
        let sig_c = sig.clone();
        let _h = EffectHandle::new(move || {
            *run_count_c.borrow_mut() += 1;
            let _ = sig_c.get();
        });
        assert_eq!(*run_count.borrow(), 1);

        // Drive the assign through the public `evaluate()` entry point,
        // not just `set_bool` directly — proves the T7 arm reaches the
        // typed registry write path through `HandlerEvalContext`.
        let mut ctx = HandlerEvalContext::new(&registry);
        let expr = HandlerExpr::Assign {
            lhs: "ready".into(),
            rhs: Box::new(HandlerExpr::BoolLit(false)),
        };
        assert_eq!(evaluate(&expr, &mut ctx), Ok(0));
        assert_eq!(ctx.get_bool("ready"), Ok(false));
        // Reactive cascade fired.
        assert_eq!(*run_count.borrow(), 2);
    }

    #[test]
    fn handler_ctx_set_bool_unknown_path_errors() {
        use crate::handler::{EvalContext, EvalError};

        let registry = SignalRegistry::new();
        let mut ctx = HandlerEvalContext::new(&registry);
        assert_eq!(
            ctx.set_bool("nope", true),
            Err(EvalError::UnknownProperty("nope".into()))
        );
    }

    #[test]
    fn binding_ctx_get_string_untracked_vs_tracked() {
        use crate::handler::EvalContext;

        let sig = Signal::new("a".to_string());
        let mut registry = SignalRegistry::new();
        registry.strings.insert("p".to_string(), sig.clone());
        let registry = Rc::new(registry);

        let run_count = Rc::new(RefCell::new(0i32));
        let run_count_c = Rc::clone(&run_count);
        let registry_untracked = Rc::clone(&registry);

        let _h_untracked = EffectHandle::new(move || {
            *run_count_c.borrow_mut() += 1;
            let ctx = BindingEvalContext::new(&registry_untracked);
            let _ = ctx.get_string("p").unwrap();
        });
        assert_eq!(*run_count.borrow(), 1);
        sig.set("b".to_string());
        assert_eq!(*run_count.borrow(), 1);

        let run_count2 = Rc::new(RefCell::new(0i32));
        let run_count2_c = Rc::clone(&run_count2);
        let registry_tracked = Rc::clone(&registry);

        let _h_tracked = EffectHandle::new(move || {
            *run_count2_c.borrow_mut() += 1;
            let ctx = BindingEvalContext::new(&registry_tracked);
            let _ = ctx.read_string_tracked("p").unwrap();
        });
        assert_eq!(*run_count2.borrow(), 1);
        sig.set("c".to_string());
        assert_eq!(*run_count2.borrow(), 2);
    }

    // ── register_binding tests (DD-M2-P5-005) ────────────────────────────────

    thread_local! {
        static FOR_ITEM_STRING_WRITES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
        static FOR_ITEM_BOOL_WRITES: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
    }

    fn record_for_item_string(_node: WidgetId, _prop: PropertyKey, value: &str) {
        FOR_ITEM_STRING_WRITES.with(|writes| writes.borrow_mut().push(value.to_string()));
    }

    fn record_for_item_bool(_node: WidgetId, _prop: PropertyKey, value: bool) {
        FOR_ITEM_BOOL_WRITES.with(|writes| writes.borrow_mut().push(value));
    }

    #[test]
    fn register_for_item_binding_writes_item_index_and_skips_out_of_range() {
        use crate::handler::{HandlerExpr, InterpolationPart};

        FOR_ITEM_STRING_WRITES.with(|writes| writes.borrow_mut().clear());

        let labels = Signal::new(vec!["a".to_string(), "b".to_string()]);
        let mut registry = SignalRegistry::new();
        registry
            .string_lists
            .insert("labels".to_string(), labels.clone());
        let registry = Rc::new(registry);

        let item = ForItemContext {
            collection: "labels".into(),
            elem: IrType::Str,
            binder: "label".into(),
            index_binder: Some("i".into()),
            position: 1,
        };
        let expr = HandlerExpr::Interpolation(vec![
            InterpolationPart::Expr(HandlerExpr::ItemRead {
                binder: "label".into(),
            }),
            InterpolationPart::Literal("#".into()),
            InterpolationPart::Expr(HandlerExpr::IndexRead { binder: "i".into() }),
        ]);

        let _h = register_for_item_binding(
            BindingTarget::WidgetProperty {
                node: WidgetId(std::ptr::null_mut()),
                prop: 0,
            },
            expr,
            Rc::clone(&registry),
            item,
            record_for_item_string,
        );
        FOR_ITEM_STRING_WRITES.with(|writes| {
            assert_eq!(&*writes.borrow(), &["b#1".to_string()]);
        });

        labels.set(vec!["a".to_string()]);
        FOR_ITEM_STRING_WRITES.with(|writes| {
            assert_eq!(
                &*writes.borrow(),
                &["b#1".to_string()],
                "out-of-range item read must skip the write"
            );
        });
    }

    #[test]
    fn register_for_item_bool_binding_tracks_bool_item_value() {
        use crate::handler::HandlerExpr;

        FOR_ITEM_BOOL_WRITES.with(|writes| writes.borrow_mut().clear());

        let flags = Signal::new(vec![true]);
        let mut registry = SignalRegistry::new();
        registry
            .bool_lists
            .insert("flags".to_string(), flags.clone());
        let registry = Rc::new(registry);

        let item = ForItemContext {
            collection: "flags".into(),
            elem: IrType::Bool,
            binder: "flag".into(),
            index_binder: None,
            position: 0,
        };

        let _h = register_for_item_bool_binding(
            BindingTarget::WidgetProperty {
                node: WidgetId(std::ptr::null_mut()),
                prop: 0,
            },
            HandlerExpr::ItemRead {
                binder: "flag".into(),
            },
            Rc::clone(&registry),
            item,
            record_for_item_bool,
        );
        FOR_ITEM_BOOL_WRITES.with(|writes| assert_eq!(&*writes.borrow(), &[true]));

        flags.set(vec![false]);
        FOR_ITEM_BOOL_WRITES.with(|writes| assert_eq!(&*writes.borrow(), &[true, false]));
    }

    #[test]
    fn register_for_item_binding_stringifies_i32_item_value() {
        use crate::handler::{HandlerExpr, InterpolationPart};

        FOR_ITEM_STRING_WRITES.with(|writes| writes.borrow_mut().clear());

        let nums = Signal::new(vec![10, 20]);
        let mut registry = SignalRegistry::new();
        registry.i32_lists.insert("nums".to_string(), nums.clone());
        let registry = Rc::new(registry);

        let item = ForItemContext {
            collection: "nums".into(),
            elem: IrType::I32,
            binder: "n".into(),
            index_binder: Some("i".into()),
            position: 1,
        };
        let expr = HandlerExpr::Interpolation(vec![
            InterpolationPart::Expr(HandlerExpr::ItemRead { binder: "n".into() }),
            InterpolationPart::Literal("@".into()),
            InterpolationPart::Expr(HandlerExpr::IndexRead { binder: "i".into() }),
        ]);

        let _h = register_for_item_binding(
            BindingTarget::WidgetProperty {
                node: WidgetId(std::ptr::null_mut()),
                prop: 0,
            },
            expr,
            Rc::clone(&registry),
            item,
            record_for_item_string,
        );

        FOR_ITEM_STRING_WRITES.with(|writes| assert_eq!(&*writes.borrow(), &["20@1"]));
    }

    #[test]
    fn register_binding_writes_initial_and_updates() {
        use crate::handler::{HandlerExpr, InterpolationPart};

        let sig = Signal::new(0i32);
        let mut registry = SignalRegistry::new();
        registry.i32s.insert("root.count".to_string(), sig.clone());
        let registry = Rc::new(registry);

        let written: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let written_c = Rc::clone(&written);
        let writer: Box<dyn FnMut(String)> = Box::new(move |v| written_c.borrow_mut().push(v));

        let expr = HandlerExpr::Interpolation(vec![
            InterpolationPart::Literal("Count: ".into()),
            InterpolationPart::Expr(HandlerExpr::PropRead {
                path: "root.count".into(),
            }),
        ]);

        let _h = register_binding_with_writer(writer, expr, registry);
        assert_eq!(*written.borrow(), vec!["Count: 0"]);

        sig.set(5);
        assert_eq!(*written.borrow(), vec!["Count: 0", "Count: 5"]);
    }

    #[test]
    fn register_binding_writer_called_for_size_affecting_prop() {
        // The writer is called on initial run and on every Signal change.
        // In production, the writer calls set_property which triggers
        // DD-P8-002 layout-dirty for size-affecting properties.
        use crate::handler::HandlerExpr;

        let sig = Signal::new(42i32);
        let mut registry = SignalRegistry::new();
        registry.i32s.insert("p".to_string(), sig.clone());
        let registry = Rc::new(registry);

        let dirty = Rc::new(Cell::new(false));
        let dirty_c = Rc::clone(&dirty);
        let writer: Box<dyn FnMut(String)> = Box::new(move |_v| dirty_c.set(true));

        let expr = HandlerExpr::PropRead { path: "p".into() };
        let _h = register_binding_with_writer(writer, expr, registry);
        assert!(dirty.get(), "writer not called on initial binding run");

        dirty.set(false);
        sig.set(99);
        assert!(dirty.get(), "writer not called after Signal update");
    }

    #[test]
    fn register_binding_writes_string_signal_initial_and_updates() {
        use crate::handler::HandlerExpr;

        let sig = Signal::new("Ready".to_string());
        let mut registry = SignalRegistry::new();
        registry.strings.insert("label".to_string(), sig.clone());
        let registry = Rc::new(registry);

        let written: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let written_c = Rc::clone(&written);
        let writer: Box<dyn FnMut(String)> = Box::new(move |v| written_c.borrow_mut().push(v));

        let expr = HandlerExpr::StrPropRead {
            path: "label".into(),
        };
        let _h = register_binding_with_writer(writer, expr, registry);
        assert_eq!(*written.borrow(), vec!["Ready"]);

        sig.set("Done".to_string());
        assert_eq!(*written.borrow(), vec!["Ready", "Done"]);
    }

    // ── register_bool_binding tests (M3-Phase 1 T8 / DD-M3-P1-007) ──────────

    #[test]
    fn register_bool_binding_writes_initial_and_updates_for_bool_prop_read() {
        // BoolPropRead reaches through read_bool_tracked, so the binding
        // subscribes to the source Signal<bool> and the writer fires both on
        // initial run and on every set.
        use crate::handler::HandlerExpr;

        let sig = Signal::new(true);
        let mut registry = SignalRegistry::new();
        registry.bools.insert("ready".to_string(), sig.clone());
        let registry = Rc::new(registry);

        let written: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
        let written_c = Rc::clone(&written);
        let writer: Box<dyn FnMut(bool)> = Box::new(move |v| written_c.borrow_mut().push(v));

        let expr = HandlerExpr::BoolPropRead {
            path: "ready".into(),
        };
        let _h = register_bool_binding_with_writer(writer, expr, registry);
        assert_eq!(*written.borrow(), vec![true]);

        sig.set(false);
        assert_eq!(*written.borrow(), vec![true, false]);

        sig.set(true);
        assert_eq!(*written.borrow(), vec![true, false, true]);
    }

    #[test]
    fn register_bool_binding_writes_initial_for_bool_lit() {
        // A bool literal binding (e.g. `bind enabled: true`) is a constant —
        // it fires exactly once on initial run and never subscribes to any
        // Signal (no tracked read happens during evaluation).
        use crate::handler::HandlerExpr;

        let registry = Rc::new(SignalRegistry::new());

        let written: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
        let written_c = Rc::clone(&written);
        let writer: Box<dyn FnMut(bool)> = Box::new(move |v| written_c.borrow_mut().push(v));

        let expr = HandlerExpr::BoolLit(false);
        let _h = register_bool_binding_with_writer(writer, expr, registry);
        assert_eq!(*written.borrow(), vec![false]);
    }

    // ── SignalRegistry string-typed Signal tests (DD-M2-P6-007) ─────────────

    #[test]
    fn signal_registry_strings_register_and_read() {
        // Signal<String> can be inserted into SignalRegistry.strings and read
        // back with the correct value — pure-logic verification that the strings
        // field is wired correctly.  Binding-evaluator integration is deferred
        // to DD-M2-P6-011.
        let mut registry = SignalRegistry::new();
        let sig = Signal::new("hello".to_string());
        registry.strings.insert("label".to_string(), sig.clone());

        assert_eq!(
            registry.strings.get("label").unwrap().get_untracked(),
            "hello"
        );

        sig.set("world".to_string());
        assert_eq!(
            registry.strings.get("label").unwrap().get_untracked(),
            "world"
        );
    }

    #[test]
    fn signal_string_set_invalidates_dependents() {
        // Signal<String>::set notifies dependent Effects — same tracking
        // contract as Signal<i32>.
        let mut registry = SignalRegistry::new();
        let sig = Signal::new("a".to_string());
        registry.strings.insert("s".to_string(), sig.clone());

        let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let log_c = Rc::clone(&log);
        let sig_c = sig.clone();

        let _h = EffectHandle::new(move || {
            log_c.borrow_mut().push(sig_c.get());
        });
        assert_eq!(*log.borrow(), vec!["a"]);

        sig.set("b".to_string());
        assert_eq!(*log.borrow(), vec!["a", "b"]);
    }

    #[test]
    fn signal_set_if_changed_skips_equal_value_dirtying() {
        let sig = Signal::new(vec![1, 2]);
        let log: Rc<RefCell<Vec<Vec<i32>>>> = Rc::new(RefCell::new(Vec::new()));
        let log_c = Rc::clone(&log);
        let sig_c = sig.clone();

        let _h = EffectHandle::new(move || {
            log_c.borrow_mut().push(sig_c.get());
        });
        assert_eq!(*log.borrow(), vec![vec![1, 2]]);

        assert!(!sig.set_if_changed(vec![1, 2]));
        assert_eq!(
            *log.borrow(),
            vec![vec![1, 2]],
            "equal collection write must not dirty dependents"
        );

        assert!(sig.set_if_changed(vec![1, 2, 3]));
        assert_eq!(*log.borrow(), vec![vec![1, 2], vec![1, 2, 3]]);
    }

    // ── drain ordering tests (DD-M2-P5-004) ──────────────────────────────────

    // drain_if_outermost ordering contract: observer drain → reactive drain → layout drain.
    //
    // The full path (including the observer queue and flush_layout) requires a live
    // WindowState and Win32 calls, so those phases are covered by the Phase 5-close
    // GUI checkpoint.  These tests verify the reactive-phase behaviour that is
    // exercisable with pure logic: dirty Effects run when drain_reactive() is called,
    // and writes made by Effects during the reactive phase take effect before the
    // hypothetical layout drain that follows.

    #[test]
    fn drain_reactive_flushes_dirty_effects() {
        // A Signal dirtied while BATCH_DEPTH > 0 is not drained until
        // drain_reactive() is called.  This exercises the entry point that
        // emit::drain_if_outermost calls as the reactive phase (phase 2).
        let sig = Signal::new(0i32);
        let log: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));

        let sig_c = sig.clone();
        let log_c = Rc::clone(&log);
        let _h = EffectHandle::new(move || {
            log_c.borrow_mut().push(sig_c.get());
        });
        assert_eq!(*log.borrow(), vec![0]); // initial run

        // Dirty the Signal inside a batch so the drain is deferred.
        BATCH_DEPTH.with(|d| d.set(d.get() + 1));
        sig.set(1);
        assert_eq!(*log.borrow(), vec![0], "effect must not fire while batched");

        BATCH_DEPTH.with(|d| d.set(d.get() - 1));
        // Now simulate the reactive phase of drain_if_outermost.
        drain_reactive();
        assert_eq!(
            *log.borrow(),
            vec![0, 1],
            "drain_reactive must flush the dirty effect"
        );
    }

    #[test]
    fn reactive_effect_write_visible_before_layout_phase() {
        // Simulate the ordering: observer phase (no-op here) → reactive phase →
        // layout phase.  A Signal written by a reactive Effect must be updated
        // by the time the layout phase reads it.
        //
        // In production the "layout phase" calls flush_layout(), which requires
        // Win32; here we use a read of the Signal as a proxy for what the layout
        // pass would observe.
        let source = Signal::new(0i32);
        let derived = Signal::new(-1i32);

        let source_c = source.clone();
        let derived_c = derived.clone();
        let _h = EffectHandle::new(move || {
            derived_c.set(source_c.get() * 2);
        });
        // After initial run: derived == 0.
        assert_eq!(derived.get_untracked(), 0);

        // Change the source inside a batch (simulates a property change arriving
        // during the observer phase, before reactive drain runs).
        BATCH_DEPTH.with(|d| d.set(d.get() + 1));
        source.set(5);
        assert_eq!(
            derived.get_untracked(),
            0,
            "derived not yet updated mid-batch"
        );
        BATCH_DEPTH.with(|d| d.set(d.get() - 1));

        // Reactive drain (phase 2 of drain_if_outermost).
        drain_reactive();

        // A layout pass reading `derived` now sees the updated value — this
        // is the DD-P8-002 contract: reactive writes precede the layout drain.
        assert_eq!(
            derived.get_untracked(),
            10,
            "layout phase must observe the reactive-updated value"
        );
    }

    // ── Phase 1 ordering tests (DD-M2-P6-001) ────────────────────────────────

    #[test]
    fn phase1_fifo_emission_order() {
        // Two independent Signals each with one Effect. Writes in FIFO order
        // must produce Effect runs in the same order as the writes.
        let sig_a = Signal::new(0i32);
        let sig_b = Signal::new(0i32);
        let order: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));

        let sig_a_c = sig_a.clone();
        let order_a = Rc::clone(&order);
        let _ha = EffectHandle::new(move || {
            let _ = sig_a_c.get();
            order_a.borrow_mut().push("a");
        });
        let sig_b_c = sig_b.clone();
        let order_b = Rc::clone(&order);
        let _hb = EffectHandle::new(move || {
            let _ = sig_b_c.get();
            order_b.borrow_mut().push("b");
        });
        // Initial runs record "a", "b".
        assert_eq!(*order.borrow(), vec!["a", "b"]);

        // Batch both writes so effects run in one drain cycle.
        with_batched_writes(|| {
            sig_a.set(1);
            sig_b.set(1);
        });
        // After the batch: initial "a","b" + re-run "a","b" in FIFO order.
        assert_eq!(*order.borrow(), vec!["a", "b", "a", "b"]);
    }

    #[test]
    fn phase1_topological_resolution_two_dependent_effects() {
        // effect_b depends on a Signal written by effect_a.
        // Topological order must run effect_a before effect_b within one drain
        // cycle, so effect_b always sees effect_a's output value.
        let source = Signal::new(0i32);
        let mid = Signal::new(0i32);

        let source_c = source.clone();
        let mid_c_write = mid.clone();
        // effect_a: reads source, writes mid.
        let _ha = EffectHandle::new(move || {
            mid_c_write.set(source_c.get() * 10);
        });
        // Initial: source=0 → mid=0.
        assert_eq!(mid.get_untracked(), 0);

        let mid_c_read = mid.clone();
        let seen: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));
        let seen_c = Rc::clone(&seen);
        // effect_b: reads mid (depends on effect_a's output Signal).
        let _hb = EffectHandle::new(move || {
            seen_c.borrow_mut().push(mid_c_read.get());
        });
        // Initial run of effect_b sees mid=0.
        assert_eq!(*seen.borrow(), vec![0]);

        source.set(3);
        // effect_a fires first (source dependency), sets mid=30.
        // effect_b fires after (mid dependency), sees mid=30.
        assert_eq!(mid.get_untracked(), 30);
        assert_eq!(*seen.borrow(), vec![0, 30]);
    }

    #[test]
    fn phase1_topological_resolution_does_not_follow_effect_id_order() {
        let source = Signal::new(0i32);
        let mid = Signal::new(0i32);
        let seen: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));

        let mid_downstream = mid.clone();
        let seen_downstream = Rc::clone(&seen);
        let _downstream_smaller_id = EffectHandle::new(move || {
            seen_downstream.borrow_mut().push(mid_downstream.get());
        });

        let source_upstream = source.clone();
        let mid_upstream = mid.clone();
        let _upstream_larger_id = EffectHandle::new(move || {
            mid_upstream.set(source_upstream.get() * 10);
        });

        with_batched_writes(|| {
            mid.set(-1);
            source.set(3);
        });

        assert!(
            !seen.borrow().contains(&-1),
            "downstream Effect must not run before the larger-id upstream writer",
        );
        assert_eq!(seen.borrow().last(), Some(&30));
    }

    #[test]
    fn phase1_last_wins_reduces_observer_entries_to_one() {
        // Multiple writes to the same Signal within one batch must collapse to
        // a single Effect re-run (last-wins: only the final value matters).
        let sig = Signal::new(0i32);
        let run_count = Rc::new(RefCell::new(0i32));
        let sig_c = sig.clone();
        let count_c = Rc::clone(&run_count);
        let _h = EffectHandle::new(move || {
            let _ = sig_c.get();
            *count_c.borrow_mut() += 1;
        });
        assert_eq!(*run_count.borrow(), 1); // initial run

        with_batched_writes(|| {
            sig.set(1);
            sig.set(2);
            sig.set(3);
        });
        // Three writes → exactly one additional Effect run (count = 2).
        assert_eq!(*run_count.borrow(), 2);
        // And the observed value is the last-written value.
        assert_eq!(sig.get_untracked(), 3);
    }

    // ── Phase 3 mutation guard tests (DD-M2-P6-001) ───────────────────────────

    #[test]
    fn phase3_state_mutating_call_in_observer_returns_error() {
        // Simulate the IN_OBSERVER_CALLBACK flag being set (Phase 3) and verify
        // that check_not_in_observer returns WASAMO_ERR_OBSERVER_MUTATION.
        use crate::abi::WASAMO_ERR_OBSERVER_MUTATION;
        use crate::emit::IN_OBSERVER_CALLBACK;

        IN_OBSERVER_CALLBACK.with(|f| f.set(true));
        let result = crate::abi::check_not_in_observer_pub("test_fn");
        IN_OBSERVER_CALLBACK.with(|f| f.set(false));

        assert_eq!(result, Some(WASAMO_ERR_OBSERVER_MUTATION));
    }

    #[test]
    fn phase3_flag_clear_allows_calls() {
        // When IN_OBSERVER_CALLBACK is false, check_not_in_observer returns None.
        use crate::emit::IN_OBSERVER_CALLBACK;

        IN_OBSERVER_CALLBACK.with(|f| f.set(false));
        let result = crate::abi::check_not_in_observer_pub("test_fn");
        assert_eq!(result, None);
    }

    // ── Divergence state machine tests (DD-M2-P6-001) ────────────────────────

    #[test]
    fn divergence_cap_break_transitions_to_diverged() {
        // Reset health state so this test is independent of others.
        HEALTH.with(|h| h.set(RuntimeHealth::Healthy));
        DIVERGENCE_DIAG.with(|d| *d.borrow_mut() = None);

        // A divergent Effect that always re-enqueues itself will hit MUTATION_CAP.
        let sig = Signal::new(0i32);
        let sig_c = sig.clone();
        let _h = EffectHandle::new(move || {
            let v = sig_c.get();
            sig_c.set(v.wrapping_add(1));
        });

        assert_eq!(
            runtime_health(),
            RuntimeHealth::Diverged,
            "cap exhaustion must transition runtime to Diverged"
        );
    }

    #[test]
    fn diverged_subsequent_calls_return_diverged_error() {
        use crate::abi::WASAMO_ERR_REACTIVE_DIVERGED;

        // Put runtime in Diverged state.
        HEALTH.with(|h| h.set(RuntimeHealth::Diverged));

        let result = crate::abi::check_not_diverged_pub("test_fn");
        assert_eq!(result, Some(WASAMO_ERR_REACTIVE_DIVERGED));

        // Restore health for other tests.
        HEALTH.with(|h| h.set(RuntimeHealth::Healthy));
    }

    #[test]
    fn divergence_diagnostics_payload_populated() {
        // Reset state.
        HEALTH.with(|h| h.set(RuntimeHealth::Healthy));
        DIVERGENCE_DIAG.with(|d| *d.borrow_mut() = None);

        let sig = Signal::new(0i32);
        let sig_c = sig.clone();
        let _h = EffectHandle::new(move || {
            let v = sig_c.get();
            sig_c.set(v.wrapping_add(1));
        });

        let diag = divergence_diagnostics();
        assert!(diag.is_some(), "diagnostics must be set after divergence");
        let diag = diag.unwrap();
        assert_eq!(
            diag.iteration_count, 16,
            "iteration_count must equal MUTATION_CAP"
        );
        assert_ne!(
            diag.offending_effect_id, 0,
            "offending_effect_id must be non-zero"
        );
        assert!(
            !diag.last_dirty_signal_ids.is_empty(),
            "last_dirty_signal_ids must name the diverging signal"
        );

        // Restore health.
        HEALTH.with(|h| h.set(RuntimeHealth::Healthy));
        DIVERGENCE_DIAG.with(|d| *d.borrow_mut() = None);
    }
}
