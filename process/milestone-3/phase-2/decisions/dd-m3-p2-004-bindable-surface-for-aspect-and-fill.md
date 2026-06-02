### DD-M3-P2-004 — Bindable surface for `aspect` and `fill`

**Status:** Accepted

**Context:**
Whether `aspect` and `fill` can each be driven by a reactive Signal
at runtime, or are constant at load time. The two attributes decide
independently. Either bindable path requires the per-type writer
seam — a new `evaluate_<T>_binding` + `widget_write_property_<T>` +
`register_<T>_binding` triple selected at `ir_loader::build_node`
(DD-M3-P1-007 pattern;
[predoc-inputs.md §2](../requirements/constraints.md#2-新しい-bindable-property-は-per-type-writer-seam-を-ir_loader-call-site-で選ぶ)).
F5 (`TypedValue` deferral) is preserved by construction either way —
the seam structurally enforces it.

The decisive criterion is: does any wireframe surface in
[docs/references/m3-gallery-wireframe.html](../references/m3-gallery-wireframe.html)
require Box's aspect or fill to vary reactively *within Phase 2's
sub-screen*?

**Options:**

Option A — Both `aspect` and `fill` constant-only in Phase 2
(recommended)
- Neither attribute admits a state binding; both are load-time
  literals only. `wasamoc check` rejects `bind aspect: ...` and
  `bind fill: ...` with a diagnostic naming the rejected attribute.
  No new per-type writer seam triple is built in Phase 2; no new
  `IrType` (Ratio / Color) is added; no `PropertyValue::Ratio` /
  `PropertyValue::Color` is added; no `WASAMO_VALUE_RATIO` /
  `WASAMO_VALUE_COLOR` ABI tag is added. Aspect and fill live as
  Box-internal domain types (`Ratio`, `Color`) on `WidgetData::Box`,
  populated by `ir_loader` directly from `IrLiteral::Ratio` /
  `IrLiteral::Color`. See DD-002's IR / runtime plumbing block and
  DD-003's variant-strategy Option A for the symmetric details.

  - What you gain: Smallest Phase 2 surface — IR literal plumbing
    plus Box-internal domain types only; no `PropertyValue` widening,
    no ABI surface change. F5 deferral is doubly protected (no new
    seam to pressure it). Future phase that needs bindable aspect or
    fill adds the seam triple as an additive extension; the IR /
    runtime plumbing built in Phase 2 is forward-compatible (the
    Phase 2 `Ratio` / `Color` domain types are what gets wrapped by
    later `PropertyValue::Ratio(Ratio)` / `PropertyValue::Color(Color)`).
  - What you give up: A theme-driven scrim color or an
    animated-aspect lightbox cannot be expressed in M3 until the
    phase that opens the bindable surface (M4+ theming, plausibly).
    The gallery wireframe's Phase 2 sub-screen — Box + Text
    placeholder against a trivial frame — needs neither.
  - **Technical risk:** Low.

Option B — `fill` bindable, `aspect` constant-only
- Admits `bind fill: <state-of-color>`; rejects `bind aspect: ...`.
  Adds `evaluate_color_binding` + `widget_write_property_color` +
  `register_color_binding` triple; adds `IrType::Color`,
  `HandlerExpr::ColorLit` / `ColorPropRead`; adds
  `WASAMO_VALUE_COLOR` ABI tag.

  - What you gain: Theme-driven scrim color expressible in M3. M4+
    theming work has one less seam to build.
  - What you give up: A full per-type seam built for an attribute
    no Phase 2 sub-screen exercises reactively. The wireframe's
    Phase 6 lightbox uses scrim; that's the phase that should pay
    the seam cost, since it's the first phase that *uses* a bindable
    fill. Speculative seam-building violates the Phase 1 discipline
    of "build the seam in the phase that needs it" (DD-M3-P1-007 was
    built for `Button.enabled`, not pre-built for unknown later
    callers).

Option C — Both `aspect` and `fill` bindable
- Both seam triples built in Phase 2.

  - What you give up: All of Option B's objections, doubled.

**Recommendation:** Option A — both constant-only in Phase 2. The
gallery wireframe's Phase 2 sub-screen exercises neither attribute
reactively; the Phase 1 discipline of "build the seam in the phase
that needs it" applies symmetrically. F5 deferral is structurally
preserved: no new bindable surface means no new pressure on
`TypedValue`. The seam pattern from Phase 1 remains available for
the phase that first uses bindable fill / aspect.

**Forward-compat exposure:** Option A keeps the Phase 2 IR / runtime
plumbing (`IrLiteral::Ratio`, `IrLiteral::Color`, Box-internal
`Ratio` / `Color` domain types) additive and forward-compatible with
a future bindable surface. When that phase lands, the additions are:

- New `PropertyValue::Ratio(Ratio)` / `PropertyValue::Color(Color)`
  variants wrapping the existing Phase 2 domain types.
- New `IrType::Ratio` / `IrType::Color` for state declarations.
- New `HandlerExpr::RatioLit` / `RatioPropRead` /
  `HandlerExpr::ColorLit` / `ColorPropRead`.
- New `evaluate_ratio_binding` / `evaluate_color_binding` writer
  triples.
- New `WASAMO_VALUE_RATIO` / `WASAMO_VALUE_COLOR` ABI tags **and**
  the corresponding `read_property_value` / `write_property_value` /
  `property_value_to_owned` arms in `abi.rs`, folded into the same
  step per
  [predoc-inputs.md §1](../requirements/constraints.md#1-box-が新規-propertyvalue-variant-を入れるなら-abi-value-conversion-arm-は同じ-step-に-fold-する).
- `abi_spec.md` updated to record the new tags.
- Widget-catalog `IrType` for the relevant `PROP_*` ids becomes
  `IrType::Ratio` / `IrType::Color`.

Each addition follows the Phase 1 type-suffix pattern verbatim;
the Phase 2 literal plumbing is not revised, only extended.

---
