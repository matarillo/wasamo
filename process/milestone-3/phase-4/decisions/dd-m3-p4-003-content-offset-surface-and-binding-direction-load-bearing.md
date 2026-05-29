### DD-M3-P4-003 — Content offset surface and binding direction (load-bearing)

**Status:** Accepted

**Context:** ScrollView's content offset (`offset-y` for the
vertical-only DD-001 recommendation) is A5's "content offset
binding" component. The DD settles (i) the literal shape (`i32`
pixels vs `f64` ratio), (ii) the binding direction (constant-only
vs bindable read-only vs bindable in-out), (iii) clamping
semantics, and (iv) where the absent-attribute default is
materialised. This DD is also the **load-bearing question for
the typed-`i32` writer pair decision** ([architecture.md §6.8
*Per-type seam* paragraph](../architecture.md#68-reactive-engine-m2-phase-5)'s
"third pair" that has been anticipated but not yet built).

**Options (literal shape):**

- **Option A — `i32` pixels (recommended).** `offset-y: 42`
  expresses 42 pixels of scroll.
  - What you gain: reuses Phase 1 / Phase 2 / Phase 3 plumbing
    (`IrLiteral::Int`, `IrType::I32`, `PropertyValue::I32`); no
    new value type; F5 (`TypedValue` deferral) doubly protected;
    the typed-`i32` writer seam, if built, is for `i32` (a type
    already in the catalog).
  - What you give up: very large content surfaces (>2 GiB
    notional pixels) overflow `i32`, which the gallery sub-screen
    does not approach.
- Option B — `f64` ratio in `[0.0, 1.0]`. `offset-y: 0.25`
  expresses "25% from top".
  - What you gain: scrollbar implementations sometimes want a
    fractional position; viewport-size-independent.
  - What you give up: introduces `f64` as a fourth scalar type
    (`IrType::F64`, `IrLiteral::F64`, evaluator surface, at
    minimum a reader seam, and a writer seam if bindable in-out);
    pressures F5 (`TypedValue` deferral) significantly more than
    `i32`; ratio shape is conceptually cleaner for some uses but
    not for the gallery's "scroll by 100 px per button click"
    proof.
- Option C — Both shapes (`offset-y: <i32>` for pixels,
  `offset-ratio: <f64>` for ratio).
  - What you gain: covers both use cases.
  - What you give up: double the spec surface, double the
    attribute count, no Phase 4 use case for the ratio half.

**Options (binding direction):**

- Option A — Constant-only. `offset-y: 42` only; no `offset-y:
  scroll_y` state-identifier binding.
  - What you gain: maximum conservatism; defers all binding work.
  - What you give up: the gallery sub-screen's scroll position
    cannot change at runtime; programmatic scrolling is
    impossible without re-loading the IR; A5's "content offset
    binding" wording becomes hard to satisfy.
- **Option B — Bindable read-only (recommended).** `offset-y:
  scroll_y` admitted (bare state identifier RHS per
  [dsl_spec.md §4.3](../dsl_spec.md#43-property-binding) property-
  binding semantics, with `state scroll_y: i32 = 0` declared at
  component scope per
  [dsl_spec.md §4.7](../dsl_spec.md#47-state-declarations-m2-surface-bool-added-in-m3-phase-1));
  runtime reads the bound state on
  each update and applies the offset; **no writer direction**
  (the runtime does not write back to the bound state when the
  layout-time clamp changes the applied offset).
  - What you gain: matches A5's "content offset binding" wording
    exactly (binding is present; direction left unspecified);
    reuses the existing i32 reader / binding-effect machinery,
    adding only the narrow string-to-`i32` parse step at
    ScrollView's per-widget `set_property` arm needed to land
    the binding result in ScrollView's `i32` `offset-y` field
    (no general typed-`i32` evaluator / writer pair is built —
    seam-building discipline preserved); **no new
    `PropertyValue` / `IrType` / `IrLiteral` variants**; gallery
    sub-screen demonstrates programmatic scrolling via buttons
    that mutate `state.scroll_y`.
  - What you give up: when the layout-time clamp differs from
    the bound state's value, the source state and the applied
    offset diverge silently (the author observes the displayed
    scroll position as ground truth, not the bound value);
    user-input-driven scrolling (wheel / drag) requires the
    writer seam, which is deferred to M4 or later.
- Option C — Bindable in-out. Runtime writes back to the bound
  state when the applied offset differs from the bound value
  (which, in Phase 4 without input handlers, only happens via
  the layout-time clamp). The typed-`i32` writer pair is built.
  - What you gain: most architecturally complete answer;
    natural shape M4 wheel / drag handlers will eventually
    need.
  - What you give up: Phase 4 has no input handler to *exercise*
    the writer direction in a visually meaningful way (the
    layout-time clamp is the only trigger and is rare); building
    the seam ahead of need violates the Phase 1 / Phase 2 seam-
    building discipline; Phase 4 close cannot produce visible
    evidence of the writer working.

**Options (clamping semantics):**

- **Option A — Silent clamp to `[0, max(0, content_size -
  viewport_size)]` (recommended).** Out-of-range bound values
  silently clamp on every layout pass. Over-scroll / under-
  scroll not admitted (touch-flick / bounce is M4 input
  territory).
  - What you gain: well-defined clamp boundary; matches default
    behaviour of all comparable widgets in WPF / Compose /
    SwiftUI before gesture momentum is added.
  - What you give up: silently-clamped state diverges from
    applied offset (Option B binding direction note above
    covers the consequence); the source state is not the
    ground truth for displayed scroll position.
- Option B — Reject out-of-range values at the IR loader. An
  `offset-y: -5` IR literal or a binding that ever produces a
  negative value fires `WASAMO_ERR_IR_MALFORMED` or a runtime
  error.
  - What you gain: source state always equals applied offset.
  - What you give up: a bindable state that legitimately
    transitions through out-of-range values (e.g. `state.scroll_y
    -= 100` on a button click that crosses the top boundary)
    becomes a runtime error rather than a clamp — fragile.
- Option C — Admit over/under-scroll. Out-of-range offsets paint
  content past the viewport.
  - What you gain: future bounce / touch-flick UI gestures fit
    naturally.
  - What you give up: requires defining what "past the viewport"
    means for the clip (clip the over-scrolled content vs let it
    paint past the viewport edge); M4 input territory.

**Options (default for absent attribute):**

- **Option A — `offset-y: 0` (recommended), applied at the
  widget-catalog constructor layer (not the IR loader's
  `unwrap_or`).** Inherits Phase 3 T5's discipline that defaults
  are applied at the runtime widget catalog, not the IR layer.
  - What you gain: consistency with Phase 3; the IR layer carries
    the absent-attribute state through, and the runtime layer
    materialises the default.
  - What you give up: nothing relative to Phase 3 precedent.

**Decision:** Option A (`i32` pixels) + Option B (bindable read-
only) + Option A (silent clamp) + Option A (default at widget
catalog). The typed-`i32` writer pair from
[architecture.md §6.8 *Per-type seam* paragraph](../architecture.md#68-reactive-engine-m2-phase-5)
is **deferred to M4 or later**; see §M4 hand-off below for the
explicit enumeration of additive M4+ scroll-model surfaces.

**Scoping intent — Phase 4 `offset-y` is not the future scroll
model.** Phase 4's `offset-y` binding is a **bindable control
surface** for proving that viewport offset traverses the DSL /
IR / runtime path. It does **not** make state-bound offset the
only — or even the primary — future ScrollView model. M4 and
beyond may additively add **input-driven internal scrolling**
(wheel / drag / keyboard gestures mutating the offset without
traversing author-bound state), **optional state write-back /
in-out binding** (the deferred Option C shape from this DD),
**scrollbar widget synchronization**, and **imperative
`scroll_to(x, y)` / `scroll_by(dx, dy)` command surface** on the
host-facing API. All four are additive on top of the Phase 4
surface — they do not require Phase 4's `offset-y` attribute to
be removed, renamed, or re-semanticised. The §4.11 spec text
presents `offset-y` as *one* control surface (the bindable one),
not as the canonical or definitive one.

**Layering with DD-002 / DD-004 / DD-005.** Clamping bound comes
from DD-002 (viewport size) and DD-005 (content measured size).
The Composition primitive that applies the offset is DD-004's
`Visual.Offset` on the ScrollView-owned intermediate content
Visual. DD-005's per-pass arithmetic re-applies the clamp on every
layout pass (window resize via `WM_SIZE`, content size change,
programmatic state mutation via the binding).
