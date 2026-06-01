### DD-M3-P1-007 — Binding evaluation result shape for `bool`

**Status:** Accepted

**Context:**
M2 ended with `evaluate_binding()` returning `Result<String, EvalError>`
([wasamo-runtime/src/handler.rs L220-L223](../../wasamo-runtime/src/handler.rs#L220-L223)),
and `widget_write_property(id, prop: u32, value: &str)`
([wasamo-runtime/src/widget.rs L937](../../wasamo-runtime/src/widget.rs#L937))
building `PropertyValue::String(value.to_string())` unconditionally
before dispatching to the per-widget setter. The reactive seam
declares its writer as `write_fn: fn(WidgetId, PropertyKey, &str)`
([architecture.md §6.7.7](../../../../docs/architecture.md#677-binding-registration-api-after-m2)). The entire binding
write pipeline is string-baked.

DD-M3-P1-003 / DD-M3-P1-005 add `BoolPropRead` and `Button.enabled:
bool`, but do not by themselves answer how the bool value reaches the
widget's property setter without going through that string seam. This
DD resolves it.

**Options:**

Option A — Per-type binding evaluator + per-type writer (recommended)
- Keep `evaluate_binding() -> Result<String, EvalError>` unchanged
  for string-typed bindings.
- Add `evaluate_bool_binding(expr, ctx) -> Result<bool, EvalError>`
  in `handler.rs`. Accepts `BoolLit`, `BoolPropRead`, and rejects all
  other variants with `EvalError::TypeMismatch` (mirroring the way
  `evaluate()` rejects string-typed forms).
- Add `widget_write_property_bool(id, prop, value: bool)` in
  `widget.rs`, constructing `PropertyValue::Bool(bool)` and
  dispatching to a per-widget setter (Phase 1 ships exactly one:
  `Button.enabled`).
- Extend the `register_binding` write-seam:
  `write_fn` becomes per-type at the call site
  ([architecture.md §6.7.7](../../../../docs/architecture.md#677-binding-registration-api-after-m2)) — the loader
  picks the bool writer when the target property is bool-typed, the
  string writer otherwise. The reactive engine itself stays
  type-agnostic.

  - What you gain: F5 (`TypedValue` deferral) is preserved by
    construction — there is no single union type that all binding
    types funnel through; each scalar has its own evaluator + writer
    pair. The choice between string / bool happens at the loader
    against the target property's type, not at runtime against a
    value tag. Mirrors the M2 read trait's per-type method shape
    (`get_string` / `get_i32`).
  - What you give up: A second evaluator function and a second
    writer function; the loader's dispatch table grows by one row.
    All mechanical, all bounded.
  - **Technical risk:** Low. The pattern is already established for
    reads; this extends it to writes. No new abstraction.

Option B — Unify `evaluate_binding()` to return `PropertyValue`
- Change the return type to `Result<PropertyValue, EvalError>`. The
  writer becomes `widget_write_property(id, prop, value:
  PropertyValue)`, dispatching on the value tag.

  - What you gain: One evaluator, one writer signature.
  - What you give up: `PropertyValue` becomes the binding result
    type, which is one short step from being the runtime value
    union that F5 defers. Phase 6 and Phase 8 would naturally widen
    it (collection bindings, then handler-side bool writes), and at
    that point `PropertyValue` is `TypedValue` in all but name.
    Option A's per-type seam is the structural fence that keeps F5
    deferred; Option B removes it.
  - **Technical risk:** Low to implement; **high forward-compat
    exposure** to the `TypedValue` deferral.

Option C — Stringify bool as `"true"` / `"false"` through the existing
string pipeline; parse at the widget setter
- Reuse the M2 string-baked seam verbatim. `evaluate_bool_binding`
  produces `"true"` or `"false"`; the `Button.enabled` setter parses
  it.

  - What you give up: Hidden string ↔ bool coercion at the
    per-widget setter. Exactly the kind of context-sensitive type
    interpretation DD-M3-P1-001 Option B was rejected for. Fragile
    against future bool properties on other widgets — each setter
    re-implements the parse. Misclassifies as parse failure any
    legitimate string-typed property whose value happens to be
    "true". Easy footgun.

**Forward-compat exposure:**
Option A's exposure under foreseeable future events (Out of scope):
when `TypedValue` is reconsidered after M3, the per-type seams
collapse into the union naturally — but they also remain perfectly
serviceable if F5 stays in force. The shape is dual-survivable.
Option B's exposure: it has already partially built the union; once
M4+ adds more types, the union *is* `TypedValue` with no remaining
fence. The exposure is asymmetric — Option A survives F5 staying or
reversing; Option B implicitly commits to reversing F5.

**Recommendation:** Option A. The owner's pre-doc-review note
([m3-target-app-predoc — Tabs / Button selected-state surface closure (Reservation 3)](../notes/m3/m3-target-app-predoc.md#保留-3-closure-tabs--button-選択状態-surface--採用-bool-を-3-つ目の-scalar-として導入))
explicitly maintained F5 deferral as the condition for admitting
`bool`; the per-type seam is what makes that condition mechanically
enforceable.

---
