### DD-M2-P6-007 — Final signature of `register_binding`

**Status:** Accepted
**Supersedes:** DD-M2-P5-005 (provisional `properties: Rc<HashMap<String, Signal<i32>>>` parameter shape only; the registration-API surface itself is preserved)

**Context:**
DD-M2-P5-005 = A specified
`pub(crate) fn register_binding(target: BindingTarget, expr: HandlerExpr) -> EffectHandle`,
but marked the surrounding context — specifically the
`properties: Rc<HashMap<String, Signal<i32>>>` argument used by
the binding evaluator to resolve named state references — as
provisional. The shape was sized for the spike's single-`i32`
counter and explicitly flagged for revisit at IR-loader time.
Phase 6 settles it.

**Options:**

Option A — Type-erased `Signal<dyn Any>` map; loader downcasts
- `properties: Rc<HashMap<String, Box<dyn AnySignal>>>` where
  `AnySignal` is a small trait with `get_as_value(&self) ->
  PropertyValue` and dependency-tracking hooks.
- What you gain: one map type for all signal value types;
  scales to the M2 type set (`i32`, string) and to M3 expansion
  with no further signature changes.
- What you give up: dynamic dispatch on every read; trait
  object boilerplate; downcasts at the binding-evaluation
  call site (or trait-method indirection) for every read.

Option B — Per-type maps in a struct (recommended)
- Replace the single map with a struct:
  ```rust
  pub(crate) struct SignalRegistry {
      i32s: HashMap<String, Signal<i32>>,
      strings: HashMap<String, Signal<String>>,
  }
  ```
  `register_binding(target, expr, registry: &SignalRegistry)`.
- What you gain: monomorphic Signal access; no dynamic
  dispatch; type errors caught at name-resolution time
  (M2's restricted type set per DD-M2-P6-004 = B makes this
  a 2-field struct). M3 type expansion adds fields; binding
  callers do not change.
- What you give up: each new type adds a field; minor
  boilerplate; conceptually rigid compared to A.
- **Technical risk: Low.**

Option C — Generic `register_binding<T>` with target-bound type
- `register_binding<T>(target: BindingTarget<T>, expr,
  signal: Signal<T>)`. Each binding registration is
  generic; the Effect closure is monomorphic per binding.
- What you gain: no map at all — the binding holds a direct
  Signal handle; reads are straight Signal `get()` calls.
- What you give up: the binding evaluator (which interprets
  arbitrary `HandlerExpr` over named references) needs the
  map to resolve names anyway; eliminating the map shifts
  resolution to the loader, but the loader must build per-
  expression closure factories for each reference shape, which
  duplicates the evaluator. Defeats the
  evaluator-core sharing of DD-M2-P5-002.

**Recommendation:** **Option B.**

DD-M2-P6-004 = B's type restriction (`i32`, string) makes the
explicit per-type registry trivial (two fields); M3 type
expansion adds fields without changing the registration call
site. Type erasure (A) buys flexibility no M2 binding shape
exercises; per-binding generics (C) defeat the evaluator
sharing. B is the smallest shape that fits both M2 acceptance
and M3 type-set growth.

**Registry key semantics.** `SignalRegistry` keys are the
`wasamoc`-resolved state names defined in DD-M2-P6-004's
name-resolution rules: post-resolution, single flat namespace
per `.ui` document, no shadowing. The runtime does not
interpret the key string; access is `HashMap::get` only. M3
component scoping (when introduced) translates to either a
key-namespacing convention or a nested registry shape; the M2
single-document flat case is compatible with either future
choice and does not foreclose them.

**Forward-compat exposure:**

- Out-of-scope items engaged: M3 expanded type set; M3
  Computed (which is signal-shaped); M5 binding-conformance
  tests.
- B is additive on type expansion; Computed slots in as
  another field (`HashMap<String, Computed<...>>`), since
  Computed exposes Signal-shape access. A loses its
  monomorphism advantage as the trait grows; C requires
  binding-side rewrites per type addition.

**Technical-risk re-evaluation:** B's risk is the smallest;
the rewrite from the spike's single-type map is mechanical.
Risk reinforces B.

---
