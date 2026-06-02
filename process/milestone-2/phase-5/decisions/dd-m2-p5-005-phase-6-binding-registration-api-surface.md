### DD-M2-P5-005 — Phase 6 binding registration API surface

**Status:** Superseded in part by [DD-M2-P6-007](../../phase-6/decisions/preamble.md#dd-m2-p6-007--signalregistry-per-type-struct) (the `properties: Rc<HashMap<String, Signal<i32>>>` parameter shape is replaced by per-type `SignalRegistry { i32s, strings }`). The registration API itself — one generic `register_binding(target: BindingTarget, expr: HandlerExpr)` with `BindingTarget::WidgetProperty { node, prop }` as the sole M2 variant — is preserved.

**Context:**
Phase 6 (`.ui → runtime` lowering) consumes the textual IR and
constructs the widget tree, attaching bindings as it goes. The
binding-registration call shape is the API surface between Phase 5
(reactive engine internals) and Phase 6 (IR loader). Getting it
right matters because:

- Phase 5 wants the registration shape minimal enough that the
  engine can change internals freely.
- Phase 6 wants the shape ergonomic enough that the IR loader's
  binding emission is a few lines per binding shape, not a section.
- M3 will add binding shapes (Computed, conditional, for-loop);
  the M2 registration API should not be the limiting factor.

**Options:**

Option A — One generic `register_binding(target, expression)` (recommended)
- Single internal Rust API:
  ```rust
  pub(crate) fn register_binding(
      target: BindingTarget,
      expr: HandlerExpr,  // shared with Phase 3 evaluator
  ) -> EffectHandle;
  ```
  where `BindingTarget` enumerates the property-write sink (in M2:
  `WidgetProperty { widget: WidgetNodeRef, property_id: u32 }`;
  M3 may add `ConditionalSubtree`, `ForLoopSubtree`, etc.).
- The engine wraps `expr` in an Effect closure that evaluates `expr`
  with a `BindingEvalContext` (read-only, dependency-tracking) and
  writes the result into `target`. Dependency collection is automatic
  per DD-M2-P5-002 = B.
- Phase 6 emits one `register_binding` call per `Text { content:
  "..." }` (or similar bound property); the IR loader does not need
  to know about Effects, Signals, or the dependency graph.

- What you gain: Phase 6's binding emission collapses to a
  one-liner per binding. The `HandlerExpr` reuse from Phase 3 means
  no new IR-side expression language for bindings; binding
  expressions and handler bodies are the same AST (with binding
  context disabling assignment statements). M3 binding shapes
  (conditional / for-loop) become new `BindingTarget` variants; the
  registration API itself does not change.
- What you give up: `BindingTarget` is an internal enum; future
  variants are not pre-specified. M3 has to add them, but additive
  changes to a `pub(crate)` enum are mechanically free.
- **Technical risk: Low.** The wrapping closure is mechanical; the
  evaluator is the existing handler evaluator with a different
  context.

Option B — Per-target-shape registration functions
- Multiple internal APIs:
  ```rust
  pub(crate) fn bind_text_content(widget, expr) -> EffectHandle;
  pub(crate) fn bind_button_label(widget, expr) -> EffectHandle;
  // ... one per bindable property kind
  ```

- What you gain: Each function can validate that the expression's
  result type matches the target property type at registration
  rather than at first run.
- What you give up: API count grows linearly with bindable property
  count. Type validation is also achievable in Option A by having
  `BindingTarget` carry the expected `PropertyValueKind` and
  checking once at registration.
- **Technical risk: Low** mechanically; design risk is API
  proliferation for no acceptance benefit.

Option C — Phase 6 builds the Effect closure itself
- Phase 5 exposes `Signal::get` / `Signal::set` and an
  `Effect::create(body: Box<dyn FnMut()>)` constructor; Phase 6
  builds the binding closure manually:
  ```rust
  let widget_handle = ...;
  let count_signal = ...;
  Effect::create(Box::new(move || {
      let v = count_signal.get();
      widget_handle.set_property(TEXT_CONTENT, format!("Count: {v}"));
  }));
  ```

- What you gain: Maximum flexibility — Phase 6 can compose any
  binding shape it wants from primitives.
- What you give up: Phase 6 carries the IR-walking *and* the
  closure-building responsibility. The textual IR's `Bind` form
  has to be lowered to a Rust closure at runtime, which means the
  IR loader has to interpret the binding expression itself rather
  than handing it off to the evaluator. Forces Phase 6 to build a
  parallel evaluation path; defeats the handler/binding evaluator-
  core sharing.
- **Technical risk: Medium.** The duplicate evaluation path is the
  risk; not in implementing it, but in keeping it consistent with
  the handler evaluator over future evolutions.

**Recommendation:** **Option A.**

The single `register_binding(target, expr)` call collapses the
Phase 5 / Phase 6 boundary to its smallest meaningful shape. The
engine internals (dependency tracker, dirty-set, drain loop) stay
fully `pub(crate)`, free to evolve in M3 without an API break.
Phase 6's IR loader emits one call per binding, with the same
`HandlerExpr` AST it already produces for inline handlers.

The `BindingTarget::WidgetProperty` variant carries enough
information for the engine to perform the write through the
existing `set_property` path; size-affecting properties trigger
layout invalidation as DD-P8-002 already arranges. M3 binding
shapes add `BindingTarget` variants without disturbing this
function's signature.

**Forward-compat exposure:** Options differ. The relevant out-of-
scope items are M3 conditional / for-loop bindings, M3 Computed,
and M3 DSL spec finalisation (which decides binding-expression
grammar).

- Option A's `BindingTarget` enum is the natural extension point
  for M3 structural bindings — variants are added; existing
  callers are unaffected. M3 grammar additions land in
  `HandlerExpr` (the existing AST), not in the registration API.
- Option B's per-shape functions multiply for every M3 binding
  kind; renaming or generalising them later means churning Phase 6
  call sites.
- Option C externalises the binding-evaluation path, so M3 grammar
  changes (e.g. function-call expressions, ternary) require Phase
  6 to update its closure-builder in lockstep with the evaluator.

This axis reinforces Option A: minimal API, additive on the
M3-foreseeable axes.

**Technical-risk re-evaluation:** Option A is the lowest-impl-risk
of the three. Option B's risk is design-shape API proliferation.
Option C's risk is parallel-evaluator divergence. Risk reinforces
Option A.

---
