### DD-M2-P6-011 — String-typed property binding

**Status: Accepted (2026-05-10)**

#### Context

DD-M2-P6-007 added `strings: HashMap<String, Signal<String>>` to
`SignalRegistry`, but the binding evaluator path (`BindingEvalContext` /
`HandlerExpr::PropRead` / `evaluate_tracked`) reads only `i32s`.
To support a `.ui` property whose source Signal is `String`-typed,
three gaps must be closed:

1. `EvalContext` trait needs `get_string(&self, path) -> Result<String, EvalError>`
   and `read_string_tracked` (dependency-tracking variant).
2. `BindingEvalContext` must implement both, routing through
   `registry.strings`.
3. `HandlerExpr` / `evaluate_tracked` must dispatch to
   `read_string_tracked` when the expression is a string-typed PropRead.

Gap 3 requires a disambiguation strategy: the evaluator currently treats
every `PropRead` as i32. This gap was surfaced during the DD-M2-P6-007
implementation step (2026-05-07) and deferred because resolving it requires
an IR design decision that is independent of the `SignalRegistry` shape.

#### Options

**Option A — Type-tag `PropRead` at the IR level.**
Add a `ty` field: `PropRead { path: String, ty: PropType }` where
`PropType` is `I32 | Str`. The loader sets `ty` at name-resolution time;
`evaluate_tracked` dispatches on `ty`.

- Pro: single variant; evaluator dispatch is one match arm per type;
  IR stays compact.
- Con: all existing `PropRead` construction sites gain a required field;
  a test-only `PropType::I32` default must be added or all tests updated.

**Option B — Introduce a `StrPropRead` variant (accepted).**
Add `HandlerExpr::StrPropRead { path: String }` alongside the existing
`PropRead`. `evaluate_tracked` dispatches `StrPropRead` to
`ctx.read_string_tracked`; existing `PropRead` path is unchanged.

- Pro: no change to existing `PropRead` construction sites or tests;
  the two read paths are structurally separated in the IR.
- Con: minor IR variant proliferation; conceptually redundant with `PropRead`.

**Option C — Unified `read_typed(path) -> TypedValue` on `EvalContext`.**
Replace `get_i32` / `get_string` with a single polymorphic method returning
a `TypedValue` enum. The evaluator extracts the arm it needs.

- Pro: one method handles all future types.
- Con: replaces the existing `get_i32` / `set_i32` API surface, requiring
  changes to all `EvalContext` implementors (including test stubs);
  `TypedValue` enum adds a dependency between `handler.rs` and a new type.

#### Recommendation

**Accepted: Option B.** Phase 7 pre-doc framing resolved A6 as
demonstrative rather than fully generic: M2 must prove the binding path is
not silently `i32`-specialized by carrying a `.ui` String property bound to
`Signal<String>` through to runtime widget property state, but M2 does not
require full `TypedValue` unification.

Adding `StrPropRead` is the smallest change that closes the three DD-011
gaps while preserving the existing integer `PropRead` path. Option A is
viable but forces every `PropRead` construction site to supply a type tag
today. Option C remains the future-friendly abstraction, but its blast
radius across `EvalContext`, handler evaluation, binding evaluation, test
stubs, and IR tooling is broader than the M2 acceptance pressure.

#### Implementation requirements

Acceptance of Option B carries the following implementation requirements:

1. Add a String read path to `EvalContext` (`get_string` plus tracked read)
   and route `BindingEvalContext` String reads through `SignalRegistry.strings`
   with dependency tracking.
2. Add the accepted String property-read representation
   (`HandlerExpr::StrPropRead { path }` under Option B) and dispatch it to the
   tracked String read path.
3. Provide a real `.ui` / emitted-IR path into the String read form based on
   the declared state type. A hand-written `StrPropRead` unit test alone does
   not discharge A6.
4. Add an automated test that proves a `.ui` or emitted-IR String binding
   reaches runtime widget property state without requiring a visible window,
   pixel inspection, or a mock Visual Layer. Actual on-screen confirmation
   remains part of the existing phase-close GUI/manual regression.
5. Preserve existing integer behavior: `PropRead { path }` remains the i32
   read form; bare integer binding, integer interpolation, and counter-style
   handler mutation are regression-protected.
6. Cross-type reads must fail rather than silently coerce. The exact
   diagnostic (`UnknownProperty` vs `TypeMismatch`) may follow the existing
   registry/error shape unless the implementation can report `TypeMismatch`
   without broad churn.

If implementation evidence changes any of these assumptions, the DD-011
implementation retrospective must record the deviation and update the
appropriate higher-level document (this ADR, the phase progress file, or a
live note such as `docs/notes/typed-value-evaluator.md`) rather than leaving
the design record stale.

#### Forward-compat exposure

Low for M2. `StrPropRead` is additive and leaves the existing integer
binding path intact. The main forward-compat exposure is not a hidden M2
requirement: if later DSL or tooling work introduces a third scalar property
type, typed item/context binding, non-string binding result values, or a
normative expression type system, the project must revisit whether parallel
typed reads are still appropriate. That open question is tracked in
[docs/notes/typed-value-evaluator.md](../../../../docs/notes/typed-value-evaluator.md).

---
