### DD-M3-P1-009 — Property type metadata and writer dispatch

**Status:** Accepted

**Context:**
DD-M3-P1-007 chooses per-type binding writers
(`widget_write_property_bool` alongside the existing string-typed
writer). For the loader to pick the right writer when a binding
target is `Button.enabled: bool`, the `(widget_type, prop_name) →
PropertyKey` lookup needs to also carry the property's type.

Today,
[`resolve_prop_key` (ir_loader.rs L797)](../../../../wasamo-runtime/src/ir_loader.rs#L797)
returns `Option<PropertyKey>` (= `Option<u32>`); the property's type
is implicit in the per-widget setter's `match` on the `PROP_*` id
([widget.rs L375 onwards](../../../../wasamo-runtime/src/widget.rs#L375)).
That works for M2 (i32 and String dispatched by the setter), but the
*binding loader* doesn't see the type — it just hands the string-baked
writer to `register_binding`. To select a typed writer at binding
registration time, the type needs to be exposed at the lookup
boundary.

**Options:**

Option A — Widen `resolve_prop_key` to return
`Option<(PropertyKey, IrType)>` (recommended)
- The widget catalog grows from `(widget, prop) -> u32` to
  `(widget, prop) -> (u32, IrType)`. The binding loader matches on
  the returned `IrType` to pick `widget_write_property` (string),
  `widget_write_property_i32` (if/when added), or
  `widget_write_property_bool`.

  - What you gain: Single source of truth for property type lives
    in the widget catalog. The mapping is co-located with the
    setter-side `match` it has to agree with — they can be reviewed
    together. Adding a new bool property to a future widget is one
    new row, type included.
  - What you give up: One enum field in the catalog table. Touches
    every existing row (M2: 4 rows — `Text.text` String, `Text.font`
    String, `Button.text` String, `Button.style` I32) with their
    explicit `IrType`.
  - **Technical risk:** Low. Pure refactor of an internal lookup
    function; no public API change.

Option B — Add a parallel `prop_type_for(prop_key) -> IrType` lookup
- Keep `resolve_prop_key` as-is; introduce a second lookup keyed by
  `PropertyKey`.

  - What you gain: `resolve_prop_key`'s signature stays compatible.
  - What you give up: Two lookup tables to keep in sync (M2 already
    has one source of truth for the `PROP_*` id; the type would now
    live in a second). Drift risk. Two callers (the binding
    registration site and the setter) must agree about which one is
    authoritative.

Option C — Encode type in `PROP_*` u32 bit-layout (e.g. high byte =
type tag)
- Magic encoding: `PROP_BUTTON_ENABLED = (BOOL_TAG << 24) | 0x03`.

  - What you give up: Opaque encoding for a problem better solved
    by a struct field. The ABI exposes `property_id: u32`
    ([abi_spec §3.3 + abi.rs L711](../../../../wasamo-runtime/src/abi.rs#L711));
    leaking type bits into ABI identifiers is a long-term
    maintenance liability.

**Recommendation:** Option A. The widget catalog is already the
right place for property metadata; extending the row by one field
is the lowest-overhead route and is invisible across the ABI
boundary (the `PROP_*` u32 values stay unchanged).

This DD is what makes DD-M3-P1-007's per-type seam operational. The
binding loader queries `(widget, prop) → (key, ty)`, dispatches to
the bool writer when `ty == IrType::Bool`, and the reactive engine's
`write_fn` parameter becomes typed at the call site rather than
globally string-baked.

**Out-of-Phase-1 question (recorded, not decided here):** the
inverse direction — `wasamoc` validating that a binding's *expression
type* matches the *target property's type* — is DD-M3-P1-010's
territory.

---
