# M3-Phase 4 — ScrollView primitive (minimal): Architecture Decisions

**Phase:** M3-Phase 4 (ScrollView primitive — minimal)
**Date:** 2026-05-25
**Status:** Proposed

## Context

M3 acceptance criterion **A5** (see
[ROADMAP.md M3](../../ROADMAP.md#m3-dsl-surface),
[m3-plan.md §Acceptance criteria](../plans/m3-plan.md#acceptance-criteria)):

> ScrollView primitive (minimal: inner unbounded measure + viewport
> clip + content offset binding; scrollbar widget, wheel handler,
> and drag are deferred to M4).

The pre-doc framing for this phase was aligned with the owner on
2026-05-25 and is recorded in
[docs/notes/m3-phase-4/pre-doc-framing.md](../notes/m3-phase-4/pre-doc-framing.md)
(commit `8f19c5f` for the initial framing draft + `234a0fa` for the
owner-requested scoping-intent clarification). That framing fixed
the 6-DD slate carried below, the visible-proof composition
(framing decision E — sibling `ScrollView { WrapPanel { Box × 30–40
} }` slice grown additively from Phase 3's standalone WrapPanel
slice in `examples/gallery/` + `examples/gallery-rust/`), the
verification-strategy menu picks (framing decision C), the two
upstream-document-revision moments inherited verbatim from Phase 2
/ Phase 3 (framing decision D), the Phase 3 carry-over R2 closure
inside Phase 4 (framing decision F), and the explicit scoping
intent that Phase 4's `offset-y` binding is **one** future scroll
model, not the only one (framing decision A + DD-003 scoping
paragraph).

Per the M2-Phase 2 framing decision D postmortem
([m3-phase-2 framing notes](../notes/m3-phase-2/m3-phase-2-pre-doc-framing.md))
and Phase 3's same-shape inheritance, the
"Moment is not a commit unit" rule applies: each upstream-document
edit in a Moment lands as its own commit on the pre-doc branch,
scoped by review concern per
[CLAUDE.md §Commit rules](../../CLAUDE.md#commit-rules) and the
doc set in
[retrospectives.md §phase-sync (Moment 2) で触る doc セット](../notes/retrospectives.md#phase-sync-moment-2-で触る-doc-セット).

The M2 / M3-Phase-1 / M3-Phase-2 / M3-Phase-3 end-state shape that
this phase extends without breaking:

- `wasamo-ir` ([wasamo-ir/src/lib.rs](../../wasamo-ir/src/lib.rs)):
  `IrType` is `I32 | Str | Bool`; `IrLiteral` is `Int | Str | Ident
  | Bool | Ratio | Color`. Phase 3 reused existing `i32` plumbing
  for all WrapPanel attributes without widening either enum.
  Phase 4 likewise introduces **no new `IrType`, no new
  `IrLiteral` variant, no new `PropertyValue` variant** —
  `offset-y` is `i32` per DD-003, and binding reuses the M2
  string-baked path that `IrType::I32` properties currently
  dispatch through (per
  [architecture.md §6.8 *Per-type seam* paragraph](../architecture.md#68-reactive-engine-m2-phase-5)).
- `wasamo-runtime` widget catalog
  ([wasamo-runtime/src/widget.rs](../../wasamo-runtime/src/widget.rs)):
  `Rectangle | VStack | HStack | Text | Button | Box | WrapPanel`
  (Phase 3 added `WrapPanel`). Phase 4 adds `ScrollView` as a
  per-kind tag (DD-001).
- Layout engine
  ([wasamo-runtime/src/layout.rs](../../wasamo-runtime/src/layout.rs)):
  pure-data `LayoutNode` / `measure` / `arrange` boundary,
  Win32/WinRT-free. Phase 2 introduced
  `LayoutError::{BoxAspectUnboundedBoth, BoxNoExtent}`; Phase 3
  did not extend the error class (one-line-flow disposition for
  unbounded main-axis); Phase 4 extends it with
  `LayoutError::ScrollViewUnboundedAxis` per DD-002 / DD-005.
- Binding pipeline: per-type writer seam pattern (DD-M3-P1-007).
  Phase 4's `offset-y` is bindable read-only (DD-003), using the
  existing M2 i32 string-baked dispatch path; **no typed-`i32`
  writer pair is built** (the anticipated "third pair" from
  [architecture.md §6.8 *Per-type seam* paragraph](../architecture.md#68-reactive-engine-m2-phase-5)
  is deferred to M4 input-handling work — see §M4 hand-off
  below). F5 (`TypedValue` deferral) is held in force by
  construction.
- `wasamoc` ([wasamoc/src/check.rs](../../wasamoc/src/check.rs)):
  state-name → declared-type table; identifier resolution lowers
  to typed `*PropRead` variants. Phase 4 adds no new value type;
  ScrollView's `offset-y` is an `i32` literal (or `i32` binding)
  and `wasamoc check` rejects non-integer literals through the
  existing diagnostic surface. ScrollView gains a child-count
  diagnostic (exactly 1 child) per DD-006.
- Composition / Visual Layer:
  ([architecture.md §6.5](../architecture.md#65-widgetnode-and-visual-layer-sync))
  `LayoutNode` offsets are absolute (root-relative); `sync_visuals()`
  converts each child offset to parent-relative `Visual.Offset`
  before writing the Composition visual tree. Phase 4 ScrollView
  installs a `Visual.Clip = InsetClip{0,0,0,0}` on its outer Visual
  and translates its content child Visual by `Visual.Offset = (0,
  -offset_y, 0)` per DD-004.

This ADR is framed against A5 and the m3-plan's "minimal: inner
unbounded measure + viewport clip + content offset binding"
phrasing
([m3-plan.md §Acceptance criteria](../plans/m3-plan.md#acceptance-criteria)).
Phase 5 Grid remains the milestone's "second novel-normative-spec
phase" proper (star sizing is the heavier algorithmic content);
Phase 4 introduces **smaller novel normative content** of its own
— the viewport / content / offset triangle plus the offset-clamp
semantics, which have no analogue in M2 / Phase 1–3 surfaces but
are lighter in scope than either WrapPanel's two-stage measure-
arrange or Grid's star sizing.

It does **not** re-open F5 (`TypedValue` deferral) — `offset-y`
ships as `i32` constant-or-bindable-read-only; no `f64` / ratio
shape, no new value type. Image-widget deferral remains in force,
carried by Phase 2's DD-M3-P2-006 placeholder pattern; Phase 4
sub-screen content is Box + Text placeholders.

The acceptance lens for this phase: A5 is satisfied when (i) `.ui`
declares `ScrollView { offset-y: <i32-or-\{state.scroll_y}>; <single
content child> }` and the shared crates lower → load → render it
with correct inner-unbounded measure + viewport clip + offset
application, (ii) the ScrollView chapter lands in `docs/dsl_spec.md`
§4.11 as a normative spec at the milestone-end-criteria bar
([m3-plan.md §Milestone-end criteria item 5](../plans/m3-plan.md#milestone-end-criteria))
applied at phase close, and (iii) `examples/gallery/` +
`examples/gallery-rust/` are grown additively with a sibling
`ScrollView { WrapPanel { Box × 30–40 } }` slice (the Phase 3
standalone WrapPanel slice stays untouched). Per A11, all sides
advance together by phase close.

### Governance note (M3 phase-ADR vs RFC transition)

The governance question Phase 3 ADR raised (M3-onward RFC wording
in VISION §9.2 / decisions-README vs realised phase-ADR practice)
was resolved upstream by
[vision-governance-rfc-deferral.md (DD-V-018)](./vision-governance-rfc-deferral.md)
on 2026-05-25 (commit `632a30b docs: apply DD-V-018 — defer RFC
adoption to post-1.0`). VISION §9.2 / §11 and decisions-README
now describe a two-stage governance policy: pre-1.0 BDFL + ADRs,
post-1.0 open + RFC. M3 is pre-1.0, so phase ADRs remain the
authoritative format. This Phase 4 ADR continues the M3 phase-ADR
pattern Phase 1 / Phase 2 / Phase 3 established, without the
"transition disconnect" caveat the Phase 3 ADR's governance note
carried.

## Decisions

### DD-M3-P4-001 — ScrollView IR node form, 1-child contract, and scroll-axis exposure

**Status:** Proposed

**Context:** ScrollView is a new widget in `wasamo-ir` and
`wasamo-runtime`. Phase 4 must commit to (i) the IR node shape
(per-kind tag vs structural variant), (ii) the child-count contract
(exactly 1 vs 0/1 vs 0+), and (iii) whether the scroll axis is an
attribute (`scroll-axis: vertical|horizontal|both`) or hardcoded.
The Phase 4 gallery sub-screen and the wireframe overflow strip
both use vertical scroll only.

**Options (IR node shape):**

- **Option A — Per-kind tag (recommended).** ScrollView appears
  as a new `widget_type: "ScrollView"` value on the generic
  `IrNode`, parallel to `HStack` / `VStack` / `Rectangle` / `Box`
  / `WrapPanel`. Phase 2 / Phase 3 settled this pattern.
  - What you gain: consistency with all prior layout primitives;
    no IR-level structural change; the parser already accepts the
    generic shape unchanged.
  - What you give up: nothing relative to the established pattern.
- Option B — Structural variant in `IrLayout`. ScrollView
  participates as a layout-flavour discriminator rather than a
  widget kind.
  - What you gain: arguably cleaner separation of "container that
    arranges children" from "leaf widget".
  - What you give up: contradicts Phase 2 / Phase 3 precedent;
    rewires the IR's existing categorisation; cost without
    visible benefit at Phase 4 scope.

**Options (child-count contract):**

- **Option A — Exactly 1 child (recommended).** `validate()`
  rejects 0-child and >1-child ScrollView with
  `WASAMO_ERR_IR_MALFORMED`.
  - What you gain: smallest spec surface; ScrollView's single-
    content semantics are unambiguous; matches WPF
    `ScrollViewer.Content` (single-child) precedent.
  - What you give up: authors must wrap N children in an explicit
    container (`ScrollView { VStack { … } }`) rather than relying
    on synthesis.
- Option B — Accept 0 or 1 child. 0 = empty viewport.
  - What you gain: parallel to Phase 3 WrapPanel's 0-child
    admission.
  - What you give up: 0-child ScrollView has no meaning beyond
    "Box-shaped viewport with nothing in it", which Box already
    provides; adds a spec edge case for no acceptance gain.
- Option C — Accept N children with implicit synthetic wrapper
  (e.g. lower to `ScrollView { VStack { N children } }` at IR-load
  time).
  - What you gain: ergonomic shorthand.
  - What you give up: requires additional normative spec
    (which wrapper, how its attributes default, how it composes
    with the author's explicit children) for no acceptance
    criterion in A5; postponable without surface compatibility
    cost.

**Options (scroll-axis exposure):**

- **Option A — Hardcode vertical-only (recommended).** No
  `scroll-axis` attribute in Phase 4; the scroll axis is a fixed
  property of ScrollView.
  - What you gain: smallest spec surface; matches the wireframe
    overflow strip exactly; mirrors Phase 3 DD-002's
    hardcode-horizontal-main-axis decision and the additive-
    extension principle (a later phase that needs horizontal or
    bidirectional scroll opens its own DD and adds the attribute
    additively).
  - What you give up: a future horizontal-only ScrollView use
    case must wait for the attribute to ship.
- Option B — Expose `scroll-axis: vertical|horizontal`. Constant-
  only, no binding.
  - What you gain: future horizontal use case is one attribute
    flip away.
  - What you give up: doubles the measure-arrange spec content
    (DD-005 must spec both axis orientations); no A5 requirement;
    bindable / constant-only sub-issue surfaces for no gallery
    use.
- Option C — Expose `scroll-axis: vertical|horizontal|both`.
  - What you gain: complete two-axis surface.
  - What you give up: significant additional measure-arrange spec
    (diagonal scrolling, two-dimensional clamping); contradicts
    A5's "minimal" phrasing outright.

**Decision:** Option A (per-kind tag) + Option A (exactly 1 child)
+ Option A (hardcode vertical-only). Smallest IR / spec surface
that satisfies A5 verbatim. Each Phase-4-non-shipped surface
(N-child synthesis, horizontal axis, both-axis) is additively
expressible in a later phase without breaking the Phase 4
contract.

**Layering with DD-002 / DD-005.** The 1-child rule names *what*
a ScrollView contains; DD-002 settles the viewport bound; DD-005
names *how* the content is measured against that bound. An Option
that admitted N>1 children with implicit synthesis would not
contradict DD-005 structurally (the synthetic wrapper would
become the single content), but would push wrapper-defaulting
semantics into the spec for no A5 requirement. An Option that
inferred scroll axis from parent shape would contradict DD-005's
normative requirement that the scroll axis is a static IR
property.

### DD-M3-P4-002 — Viewport size source

**Status:** Proposed

**Context:** ScrollView's outer extent (the "window" through
which content is viewed) must come from somewhere. The candidate
sources are parent constraint passthrough (WPF / Compose / CSS
default for block-level overflow elements), an explicit attribute
pair (`viewport-width: <i32>` / `viewport-height: <i32>`), or a
hybrid (parent on cross axis, attribute on scroll axis). DD-005's
measure-arrange consumes the viewport extent as the cross-axis
bound passed to content and as the scroll-axis bound used in the
offset clamp.

**Options (default source):**

- **Option A — Parent constraint passthrough on both axes
  (recommended).** ScrollView fills its parent slot on both axes;
  the parent's layout role (VStack member, HStack member, etc.)
  sizes the ScrollView slot.
  - What you gain: matches WPF / Compose / CSS reader
    expectations; smallest spec surface (no new attribute, no new
    bindable surface); composes cleanly with all existing layout
    parents.
  - What you give up: direct fixed-viewport sizing is deferred.
    In Phase 4, ScrollView obtains its viewport size from the
    slot its parent layout allocates to it (root window bounds,
    VStack / HStack member allocation, etc.); ScrollView itself
    exposes no `viewport-*`, `width`, or `height` attribute. A
    future phase that needs author-controlled viewport sizing
    opens its own DD and adds the attribute additively.
- Option B — Explicit `viewport-width` / `viewport-height`
  attribute pair, no passthrough. Author declares viewport
  dimensions; ScrollView ignores parent constraint.
  - What you gain: direct sizing ergonomic; symmetry with
    Phase 3 `item-cross-size` precedent for "container declares
    the bound".
  - What you give up: contradicts the WPF / Compose / CSS
    convention readers will arrive with; constant-only vs
    bindable sub-issue surfaces for no gallery use; adds two
    new attributes for no A5 requirement.
- Option C — Hybrid (parent passthrough on one axis, attribute on
  the other).
  - What you gain: captures the "fixed-height scroll region in
    fluid-width parent" pattern.
  - What you give up: significantly more complex spec (per-axis
    source resolution rules); the gallery's vertical-only scroll
    use case does not pressure it.

**Options (unbounded scroll-axis parent behaviour):**

- **Option A — Layout-time runtime error
  (`LayoutError::ScrollViewUnboundedAxis`) (recommended).**
  ScrollView's scroll axis being unbounded is structurally
  meaningless (no bound to scroll *to*).
  - What you gain: no silent dropout; the no-silent-dropout virtue
    Phase 2 chose for `BoxNoExtent` transfers cleanly; the
    layout-time error names the structural problem.
  - What you give up: the future Phase 5 Grid star-sizing pre-
    resolution intrinsic measure pass cannot embed an unbounded-
    scroll-axis ScrollView without the author explicitly fixing
    the scroll-axis bound; this is the desired behaviour, not a
    cost.
- Option B — Degenerate to viewport-equals-content (no
  scrolling). ScrollView fills whatever extent the content
  reports along the scroll axis.
  - What you gain: silent success.
  - What you give up: the "ScrollView with no scroll bound" is
    behaviour-indistinguishable from a Box containing the
    content; ScrollView becomes a no-op widget in this state,
    masking the structural problem.
- Option C — Reserved (defer to whichever phase introduces the
  unbounded-parent context).
  - What you gain: postponable.
  - What you give up: the unbounded-parent case is reachable
    today via any host that constructs an unbounded measure
    context; deferring leaves the runtime to crash or behave
    unpredictably until a future phase covers it.

**Options (bindable surface for viewport attribute, conditional on Option B/C above):**

Sub-issue collapses under the recommended Option A (parent
passthrough). Recorded for completeness: if a future phase adopts
Option B / C, the constant-only stance mirroring Phase 3 DD-003
/ DD-004 is the default; the per-type writer seam pressure (if
any) would ride DD-003's offset writer pair anyway.

**Decision:** Option A (parent passthrough on both axes) + Option
A (unbounded scroll-axis parent → `LayoutError::ScrollViewUnboundedAxis`).
The new `LayoutError` variant is **internal only**; no
`WASAMO_LAYOUT_ERROR_*` ABI tag is added per
[m3-phase-4 pre-doc-inputs §3](../notes/m3-phase-4/pre-doc-inputs.md)
(the host receives layout failure as opaque in Phase 4 — no host
code can meaningfully observe the new variant).

**Layering with DD-005.** DD-005's content measure pass uses the
viewport size as the cross-axis bound passed to content. With
Option A, the viewport size equals the parent constraint
ScrollView received. The unbounded-scroll-axis Option-A error
fires in DD-005's algorithm before the content measure happens,
not after.

### DD-M3-P4-003 — Content offset surface and binding direction (load-bearing)

**Status:** Proposed

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

- Option A — Constant-only. `offset-y: 42` only; no `\{state.scroll_y}`
  binding.
  - What you gain: maximum conservatism; defers all binding work.
  - What you give up: the gallery sub-screen's scroll position
    cannot change at runtime; programmatic scrolling is
    impossible without re-loading the IR; A5's "content offset
    binding" wording becomes hard to satisfy.
- **Option B — Bindable read-only (recommended).** `offset-y:
  \{state.scroll_y}` admitted; runtime reads the bound state on
  each update and applies the offset; **no writer direction**
  (the runtime does not write back to the bound state when the
  layout-time clamp changes the applied offset).
  - What you gain: matches A5's "content offset binding" wording
    exactly (binding is present; direction left unspecified);
    reuses the M2 i32 string-baked dispatch path; **no new
    `PropertyValue` / `IrType` / `IrLiteral` variants**; no
    typed-`i32` writer pair built (seam-building discipline
    preserved); gallery sub-screen demonstrates programmatic
    scrolling via buttons that mutate `state.scroll_y`.
  - What you give up: when the layout-time clamp differs from
    the bound state's value, the source state and the applied
    offset diverge silently (the author observes the displayed
    scroll position as ground truth, not the bound value);
    user-input-driven scrolling (wheel / drag) requires the
    writer seam, which is deferred to M4.
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
`Visual.Offset` on the content child Visual. DD-005's per-pass
arithmetic re-applies the clamp on every layout pass (window
resize via `WM_SIZE`, content size change, programmatic state
mutation via the binding).

### DD-M3-P4-004 — Clip surface installation and Composition primitive choice

**Status:** Proposed

**Context:** A5 names "viewport clip" as a load-bearing
component. Phase 3 T8 established that **WrapPanel installs no
clip surface** (see
[Phase 3 ADR DD-005 oversized-line section](./m3-phase-3-wrap-panel.md));
ScrollView is the **dual** — it must install a clip surface
because the gallery's overflow state (`ScrollView { … }`) is
exactly where the "parent clips" contract Phase 3 deferred to
becomes active. The DD settles (i) which Composition primitive
implements the clip, (ii) which Composition primitive applies the
offset, and (iii) where in the Visual tree the clip sits.

**Options (clip primitive):**

- **Option A — `Visual.Clip = InsetClip{0,0,0,0}` (recommended).**
  An InsetClip whose insets are all zero, applied to the
  ScrollView's outer Visual whose extent matches the viewport.
  - What you gain: canonical Windows.UI.Composition pattern for
    "clip to my own bounds"; the clip extent automatically
    follows the Visual's `Size` property on resize; no manual
    rectangle bookkeeping.
  - What you give up: nothing relative to the alternatives;
    InsetClip is the most idiomatic primitive for this use.
- Option B — `Visual.Clip = RectangleClip` (or
  `CompositionGeometricClip` with `CompositionRectangleGeometry`)
  with explicit `(0, 0, viewport_w, viewport_h)` extent.
  - What you gain: explicit rectangle is easier to reason about
    when reading code.
  - What you give up: manual rectangle bookkeeping — the
    rectangle must be re-set on every viewport size change
    (window resize), adding a code path Option A does not need.
- Option C — `Visual.Clip = InsetClip` with non-zero insets
  derived from a future `padding` attribute.
  - What you gain: forward-compatible with future padding.
  - What you give up: over-engineered for Phase 4 (padding is out
    of A5 minimal scope); the zero-inset Option A composes
    additively with a future padding attribute without rework.

**Options (offset application primitive):**

- **Option A — `Visual.Offset` on the content child Visual
  (recommended).** Mutation = `SetOffset(0, -offset_y, 0)`
  (negative because moving content up exposes lower content
  through the viewport).
  - What you gain: matches the existing M2 visual-layer
    convention (LayoutNode offsets → parent-relative
    `Visual.Offset` per
    [architecture.md §6.5](../architecture.md#65-widgetnode-and-visual-layer-sync));
    no new Composition primitive introduced; `i32` pixel offset
    + no animation makes the simpler primitive sufficient.
  - What you give up: fractional offsets and Composition-driven
    animation become awkward (would require switching to
    `TransformMatrix`) — neither is Phase 4 territory.
- Option B — `Visual.TransformMatrix` on the content child
  Visual. Mutation = `SetTransformMatrix(Matrix4x4.CreateTranslation(0,
  -offset_y, 0))`.
  - What you gain: forward-compatible with fractional offsets and
    Composition animation; M4's smooth-scroll work would land
    here naturally.
  - What you give up: heavier primitive for the Phase 4 use
    case; introduces a divergence from the
    `LayoutNode.offset → Visual.Offset` convention §6.5
    establishes (TransformMatrix is an alternative, not a
    parallel); M4 can switch when momentum / smooth-scroll
    actually pressures it without breaking the Phase 4 surface.

**Options (Visual tree shape):**

- **Option A — Outer (clipped) + inner (offset) (recommended).**
  ScrollView's own Visual carries the clip; the content Visual
  is a child of the outer Visual whose `Visual.Offset` carries
  the scroll position. Visual tree:
  ```
  ScrollView Visual (Size = viewport, Clip = InsetClip{0,0,0,0})
    └── content Visual (Offset = (0, -offset_y, 0))
          └── … widget tree (Box thumbnails / WrapPanel / etc.)
  ```
  - What you gain: clean separation of "viewport" (outer) from
    "scrollable canvas" (inner); the clip naturally clips the
    translated content; verified compatible with
    [architecture.md §6.5](../architecture.md#65-widgetnode-and-visual-layer-sync).
  - What you give up: nothing relative to the natural Composition
    tree shape.

**Decision:** Option A (`Visual.Clip = InsetClip{0,0,0,0}`) +
Option A (`Visual.Offset` on content) + Option A (outer-clipped /
inner-offset tree shape). All three match the existing M2
visual-layer offset convention, use the simplest clip primitive
for the required viewport clip, and avoid introducing a
TransformMatrix-based offset path.

**R2 (Phase 3 carry-over) — close inside Phase 4.** Phase 3 T9
surfaced a `sync_visuals` bug whose root cause was the implicit
absolute-vs-parent-relative offset convention. The architecture
fix landed in
[architecture.md §6.5](../architecture.md#65-widgetnode-and-visual-layer-sync);
the test-coverage half was filed open as R2 (per
[Phase 3 ADR Out-of-phase residuals](./m3-phase-3-wrap-panel.md)).
Per Phase 4 framing decision F, R2 closes inside Phase 4 via the
Windows integration test's three-level offset assertion
(ScrollView Visual at parent offset X, content Visual at offset
(0, -offset_y) relative to ScrollView, Box thumbnails inside
content at their own offsets — the three-level nesting Phase 3
lacked). See §Phase 4 verification closure item 4 below.

**Layering with DD-003 / DD-005.** DD-003 supplies the `offset_y`
value (read-only-bound `i32` per recommendation, clamped per
DD-005); DD-004 applies it via `Visual.Offset` on the content
Visual; DD-005's arrange pass re-computes the clamp on every
layout pass.

### DD-M3-P4-005 — Measure-arrange algorithm (novel normative viewport / content / offset semantics)

**Status:** Proposed

**Context:** Introduces novel normative spec content into
`docs/dsl_spec.md` of a different *kind* than Phase 3 (no line-
formation algorithm) and lighter in *scope* than the upcoming
Phase 5 Grid star sizing which is the milestone's "second novel-
normative-spec phase" proper. The DD settles the content-measure
pass, viewport-vs-content size relationship, and offset clamping;
the ADR section is also the **seed** of the dsl_spec §4.11 chapter
(Moment 1 lands the spec chapter in design-spec-draft form;
Moment 2 re-syncs to implementation findings).

The algorithm is structurally simpler than Phase 3 WrapPanel's
two-stage measure-arrange — ScrollView has one child, no line
formation, no multi-pass measurement. The novel content is the
**unbounded-axis + bounded-axis asymmetric input** to the
content's measure pass, plus the **offset clamp** semantics that
have no analogue in Phase 1–3 surfaces.

**Sub-issues:**

- **Content measure pass.** Content is measured with a constraint
  of `(viewport_width, +∞)` — bounded cross axis (= viewport
  width per DD-001's vertical-only recommendation), unbounded
  scroll axis. Inverse of WrapPanel's measure input (WrapPanel:
  unbounded main + cross bound from Phase 3 DD-M3-P3-004
  `item-cross-size` / parent-cross passthrough). DD-001's vertical-only
  implies "scroll axis = vertical, unbounded direction =
  vertical, viewport-equals-cross-axis-bound = width".
- **Viewport vs content size relationship.**
  - `content_size_scroll_axis <= viewport_size_scroll_axis`:
    content fits within viewport. Offset is clamped to 0 (no
    scrolling possible).
  - `content_size_scroll_axis > viewport_size_scroll_axis`:
    content exceeds viewport. Offset is clamped to
    `[0, content_size - viewport_size]`. Visible content along
    the scroll axis is `[offset, offset + viewport_size)`.
- **Offset application.** After the content measure, the content's
  resolved rect is translated by `(0, -offset)` (in absolute
  layout-engine coordinates, before `sync_visuals()` converts to
  parent-relative). The content's outer rect within ScrollView's
  local space is then `(0, -offset, content_w, content_h)`;
  visible clipping is the rendering-side operation owned by
  DD-004's Composition clip.
- **ScrollView outer size.** Equals viewport size, regardless of
  content size. Cascading parent-bound violations are excluded —
  even if content size exceeds parent's slot, ScrollView's outer
  size stays at viewport. Phase 4 analogue of Phase 3 DD-005's
  "WrapPanel outer main-axis size does not grow to accommodate
  oversized children" rule.
- **Content-smaller-than-viewport behaviour.** Content paints at
  its measured size, anchored at the viewport's top-leading corner
  (`(0, 0)` in viewport-local coordinates). Remaining viewport
  area shows whatever visual content is behind the ScrollView
  (for example a Box fill supplied by surrounding composition);
  Phase 4 adds no ScrollView-level background attribute. Offset
  is forced to 0 by the clamp.
- **Unbounded scroll-axis parent.** Per DD-002 decision, fires
  `LayoutError::ScrollViewUnboundedAxis` at layout time. No
  degenerate Phase 4 ScrollView shape: unbounded scroll axis is
  structurally meaningless.
- **Rounding contract.** Inherits Phase 2 DD-005 / Phase 3 DD-005:
  `f32` for layout-engine internals, `i32` for attribute literals,
  promoted to `f32` at comparison. No pixel-snapping in Phase 4.
  `i32` offset is promoted to `f32` for the clamp arithmetic.
- **LayoutError surface.** New
  `LayoutError::ScrollViewUnboundedAxis` variant per DD-002.
  Internal-only; no `WASAMO_LAYOUT_ERROR_*` ABI tag is added in
  Phase 4 (no host can observe the new variant meaningfully).

**Options (overall algorithm):**

- **Option A — Asymmetric measure + clamp (as detailed above)
  (recommended).** Content measured with `(viewport_w, +∞)`;
  offset clamped to `[0, max(0, content_size - viewport_size)]`;
  outer size = viewport size; unbounded scroll-axis parent →
  `LayoutError::ScrollViewUnboundedAxis`.
  - What you gain: matches A5 verbatim ("inner unbounded measure
    + viewport clip + content offset binding"); composes cleanly
    with Phase 3 WrapPanel's pairing contract (per Phase 3 ADR
    DD-005 ScrollView pairing); narrowest spec surface.
  - What you give up: nothing relative to A5.
- Option B — Symmetric measure (content also measured with
  bounded scroll-axis = viewport size). Content cannot exceed
  viewport; scroll is impossible.
  - What you gain: simplest algorithm.
  - What you give up: contradicts A5's "inner unbounded
    measure"; ScrollView becomes a clipped Box, not a scrollable
    viewport.
- Option C — Lazy measure (only measure visible content based
  on current offset). Content extent unknown until scrolled to.
  - What you gain: theoretical performance gain for very large
    content.
  - What you give up: pre-1.0 over-engineering; Phase 4 sub-
    screen (30–40 thumbnails) does not pressure performance;
    introduces stateful measure cache; the offset clamp upper
    bound (`content_size - viewport_size`) becomes ill-defined
    if content_size is itself lazy.

**Decision:** Option A. The arrange pass re-applies the clamp on
every layout pass (window resize, content size change,
programmatic state mutation via the DD-003 binding). The
unbounded-scroll-axis check fires *before* the content measure;
the content measure pass is what runs in the recommended Option A
happy path.

**Spec content seed (Moment 1 §4.11 draft).** The DD-005
sub-issues map 1:1 to the §4.11 chapter outline:

1. ScrollView is a 1-child container with vertical-only scroll
   axis (anchors to DD-001).
2. Viewport size source = parent constraint passthrough; explicit
   `viewport-*` attribute deferred (anchors to DD-002).
3. `offset-y` attribute: `i32` pixels, bindable read-only, default
   0 at widget-catalog constructor (anchors to DD-003).
4. Content measure pass: bounded cross axis (= viewport width) +
   unbounded scroll axis (= vertical). Inverse of WrapPanel's
   measure input.
5. Offset clamp: `[0, max(0, content_size - viewport_size)]`.
   Silent clamp; over/under-scroll not admitted in Phase 4.
6. ScrollView outer size = viewport size; does not grow to
   accommodate content overflow.
7. Visible clip: ScrollView Visual installs `Visual.Clip =
   InsetClip{0,0,0,0}` (anchors to DD-004).
8. Content Visual carries the offset: `Visual.Offset = (0,
   -offset_y, 0)`.
9. Unbounded scroll-axis parent fires
   `LayoutError::ScrollViewUnboundedAxis` (anchors to DD-002 /
   DD-005).
10. ScrollView mental model subsection (5 facts) — see Phase 4
    pre-doc framing decision I.

**Layering with DD-001 / DD-002 / DD-003 / DD-004.** The
algorithm assumes:
- A 1-child ScrollView (per DD-001).
- A viewport size from DD-002 (parent passthrough by default).
- An offset value from DD-003 (read-only `i32` binding by
  default, clamped per the rule above).
- A clip + offset application via Composition primitives in
  DD-004 (Visual.Clip + Visual.Offset).

Any Option in DD-005 that re-derives any of these contradicts
the chain. In particular, an Option that re-measures the content
with a *bounded* scroll-axis constraint (Option B above)
contradicts A5's "inner unbounded measure" load-bearing
phrasing.

### DD-M3-P4-006 — IR-loader defense-in-depth invariants

**Status:** Proposed

**Context:** Phase 2 T7 surfaced the principle: IR-load → runtime-
materialise invariants belong in pure-logic `validate()`, not in
WinRT-bound `build_node`, so the same invariant is enforced
regardless of which entry point materialises the IR. Phase 3 T6
extended this with WrapPanel's value-range invariants (negative-
literal rejection). Phase 4 extends it with ScrollView's
invariants, which are a **different shape** than either Phase 2
(structural placement) or Phase 3 (value range): Phase 4 needs a
**compound** shape combining structural child-count rejection
(Phase-2-flavour) with runtime-clamp for the offset value (which
is *not* a validate-time reject) per
[m3-phase-4 pre-doc-inputs §5](../notes/m3-phase-4/pre-doc-inputs.md).

**Sub-issues:**

- **Child count.** Per DD-001 (exactly 1 child), `validate()`
  rejects 0-child and >1-child ScrollView with
  `WASAMO_ERR_IR_MALFORMED`. Symmetric with Phase 2 T7's `Box`
  child-count rejection in shape.
- **Offset value range.** Per DD-003 (`offset-y: <i32>`),
  `wasamoc check` rejects non-`IntLit` RHS shapes at compile
  time (existing infrastructure). The Phase 3 DD-006 "negative
  literal rejection" pattern **does not apply**: negative
  offsets are layout-time-clamped to 0 per DD-005 (not IR-
  rejected) because an author may bind a `state.scroll_y` that
  legitimately transitions through negative values during state
  changes. The two-gate defense-in-depth pattern still applies,
  but the runtime gate is the **clamp in DD-005's arrange pass**,
  not a `validate()`-time reject. This is the value-range half
  of the compound shape, distinct from Phase 3's pattern.
- **Bound-direction validation (conditional on DD-003).** Per
  DD-003 Option B (bindable read-only) decision, this sub-issue
  collapses — `validate()` has no mutability check to perform.
  Recorded for completeness: if DD-003 were Option C (bindable
  in-out), `validate()` would need to check the bound state is
  mutable, which is currently outside the IR's vocabulary and
  would have to defer to `wasamoc check`.
- **Error class.** All ScrollView IR-loader invariant violations
  surface as `WASAMO_ERR_IR_MALFORMED`, consistent with Phase 2 /
  Phase 3 precedent.

**Options:**

- **Option A — Compound: structural child-count gate at
  `validate()` + runtime clamp for offset at the arrange pass
  (recommended).** Matches the invariant shape per
  pre-doc-inputs §5.
  - What you gain: each invariant is enforced at the layer
    appropriate to its shape; structural invariants are rejected
    early (Phase-2-flavour), value-range invariants are
    accommodated dynamically (binding-friendly).
  - What you give up: nothing relative to the alternatives.
- Option B — Reject negative offset at `validate()`. Same
  pattern as Phase 3 DD-006.
  - What you gain: consistency with Phase 3 pattern.
  - What you give up: makes bindable offset fragile (any binding
    transition through a negative intermediate value becomes a
    runtime error); contradicts the layout-time clamp semantics
    DD-005 specifies.
- Option C — No defense-in-depth at all (rely on `wasamoc
  check`). Phase 1 / Phase 2 T7 / Phase 3 T6 explicitly
  established the two-gate principle; abandoning it for Phase 4
  is regressive.

**Decision:** Option A (compound: structural child-count gate +
runtime clamp). The `validate()` extension rejects 0 and >1
children; the runtime arrange pass clamps the offset. No
`validate()`-time offset value-range check.

**Layering with DD-001 / DD-003 / DD-005.** Inherits child-count
contract from DD-001; relies on DD-003's read-only binding
direction to make the no-mutability-check sub-issue collapse;
delegates value-range enforcement to DD-005's arrange pass.

## Phase 4 verification closure (what counts as A5 evidence)

This section is not a DD — it records the agreed shape of the
proof that closes Phase 4 per framing decision C, so the
implementation plan inherits a concrete target.

A5 (ScrollView minimal — inner unbounded measure + viewport clip
+ content offset binding) is considered satisfied when **all
four** of the following are observed:

1. **`wasamoc check` compile-time evidence (host-independent).**
   Pure-logic tests in `wasamoc`'s check / lower path cover:
   - **Child-count rejection** (DD-006 structural gate, compile-
     time half) — `ScrollView { }` (0 children) and `ScrollView
     { Box {} Box {} }` (>1 children) each surface a `wasamoc
     check` diagnostic naming the offending shape. The runtime
     `validate()` half is covered by item 3 below.
   - **`offset-y` literal shape acceptance / rejection** —
     `offset-y: 42` accepted; `offset-y: "hello"` /
     `offset-y: 1.5` / `offset-y: #336699` / `offset-y: 16:9` /
     `offset-y: true` rejected (each surfaces a `wasamoc check`
     diagnostic naming the rejected attribute). `offset-y: -5`
     **accepted** (negative literals are layout-time-clamped per
     DD-005 / DD-006, not compile-time-rejected — the Phase 3
     pattern explicitly does not apply here).
   - **`offset-y` binding admission** — `offset-y:
     \{state.scroll_y}` accepted when `scroll_y` is declared as
     `i32` in `state`; rejected when `scroll_y` is undeclared,
     `bool`, or `String`. Reuses the existing i32 binding
     dispatch surface; no new `wasamoc` infrastructure.
   - **Unknown ScrollView attribute rejection** — any attribute
     other than `offset-y` on ScrollView is rejected (no
     `viewport-*`, no `scroll-axis`, no `padding` — all out of
     Phase 4 scope per DD-002 / DD-001 / Out-of-scope).
   - **Sub-screen positive control** — the gallery sub-screen's
     `.ui` (item 5 below) compiles cleanly as the positive
     control.

   These run on any CI runner; the diagnostics are pure-logic
   in `wasamoc`.

2. **Measure-arrange unit-test evidence (host-independent).**
   Pure-logic tests against the layout engine's ScrollView
   measure-arrange (`wasamo-runtime/src/layout.rs` extension)
   cover:
   - **Bounded scroll-axis parent (happy path)** — content
     measured with `(viewport_w, +∞)`; content fits within
     viewport → offset clamped to 0; content exceeds viewport
     → offset clamped to `[0, content_size - viewport_size]`.
   - **Unbounded scroll-axis parent** — fires
     `LayoutError::ScrollViewUnboundedAxis` (reject test;
     pins the DD-002 / DD-005 branch per
     [m3-phase-4 pre-doc-inputs §6](../notes/m3-phase-4/pre-doc-inputs.md)).
   - **Content smaller than viewport** — content paints at its
     measured size at top-leading corner; offset clamped to 0.
   - **Content equal to viewport** — boundary case; offset
     clamped to 0.
   - **Offset clamp arithmetic** — `offset-y` values across
     `< 0`, `= 0`, `mid-range`, `= max`, `> max` each produce
     correctly clamped applied offsets.
   - **ScrollView outer size = viewport size** invariant
     regardless of content size.
   - **Rounding contract** — `i32` offsets promoted to `f32`
     for arithmetic; no pixel-snapping.

   These run on any CI runner; the measure-arrange algorithm
   is a pure function (input → output) per framing decision C.

3. **IR-loader / `validate()` invariant evidence (host-
   independent).** Pure-logic tests in `wasamo-runtime`'s
   `ir_loader::validate()` path cover (DD-006 structural gate,
   runtime half):
   - **Child-count rejection** for 0-child ScrollView and >1-
     child ScrollView (each surfaces
     `WASAMO_ERR_IR_MALFORMED`). Symmetric with Phase 2 T7's
     Box-child-count `validate()` discipline.
   - **No offset value-range rejection** — `offset-y: -5` and
     `offset-y: <very large>` in memory-IR pass `validate()`
     (the clamp is the runtime gate per DD-006 compound shape).

4. **Windows-runtime layout evidence (CI-gated, including R2
   closure).** A mock-free integration test (per
   [CLAUDE.md §Testing rules](../../CLAUDE.md#testing-rules))
   on the Windows CI runner exercises:

   - **Scroll-path fixture (primary).** A `.ui` declares a
     ScrollView with `offset-y: \{state.scroll_y}` containing a
     `VStack { Box × N }` whose total height exceeds a known
     viewport size, inside a parent of fixed dimensions.
     `state.scroll_y` is declared `i32` initialised to 0. The
     test loads the IR, runs the layout pass, and asserts:
     - (a) the ScrollView's resolved rectangle matches the
       expected viewport dimensions (per DD-005 outer size =
       viewport);
     - (b) the content Visual's `Visual.Offset` is `(0, 0, 0)`
       when `scroll_y = 0`;
     - (c) after mutating `state.scroll_y = 100`, the content
       Visual's `Visual.Offset` becomes `(0, -100, 0)`;
     - (d) after mutating `state.scroll_y = -50`, the content
       Visual's `Visual.Offset` becomes `(0, 0, 0)` (clamped to
       0 per DD-005);
     - (e) after mutating `state.scroll_y` to a value larger
       than `content_h - viewport_h`, the content Visual's
       `Visual.Offset` becomes `(0, -(content_h - viewport_h),
       0)` (clamped to max per DD-005);
     - (f) ScrollView's outer Visual has a `Visual.Clip`
       property set to a non-null clip (the InsetClip from
       DD-004) — clip **presence** assertion;
     - (g) the content child Visual has `Visual.Clip = null`
       — clip **absence** regression guard (the symmetric
       inverse of Phase 3 T8's WrapPanel clip-absence
       assertion).
   - **R2 closure — three-level nested offset assertion (R2
     test-coverage half from Phase 3).** The scroll-path fixture
     above traverses three levels of `Visual.Offset` nesting:
     parent → ScrollView Visual (at some `(parent_offset_x, …,
     0)`) → content Visual (at `(0, -offset_y, 0)`) → Box
     thumbnails inside content (at their own layout-derived
     offsets). The test asserts that each thumbnail's
     *root-relative* position (computed by summing parent-
     relative offsets up the chain) equals the expected position
     given the scroll state — i.e. the
     absolute-vs-parent-relative convention from
     [architecture.md §6.5](../architecture.md#65-widgetnode-and-visual-layer-sync)
     is observed end-to-end with non-trivial nesting. This is
     the test-coverage half of R2 per Phase 4 framing decision
     F.
   - **Unbounded scroll-axis runtime fixture** — a `.ui`
     declares a ScrollView inside a parent whose scroll-axis is
     unbounded (synthesisable by embedding in an intrinsic-
     measure context). The test asserts the layout pass returns
     `Err(LayoutError::ScrollViewUnboundedAxis)`. If no
     ergonomic way to synthesise this fixture exists at the IR
     level, the unbounded-parent case may be exercised purely
     in unit tests (item 2) and this fixture downgraded to
     pure-logic; the integration-test version is preferred when
     feasible.

   All fixtures fail (not skip) on a runner that cannot create
   the Compositor — the test gates A5 evidence in CI, not local
   convenience. Skip-guard inherits the Phase 2 T11 / Phase 3
   pattern verbatim (fires on `0x80070005` from `wasamo_init`).

5. **End-to-end host evidence (visible smoke).**
   `examples/gallery/gallery.ui` is **grown additively** from
   Phase 3's standalone WrapPanel slice (which remains
   unchanged) by adding a sibling slice containing the canonical
   `ScrollView { WrapPanel { Box × 30–40 } }` composition with
   programmatic scroll controls (a pair of Button widgets
   mutating `state.scroll_y` by ±100 px per click).
   `examples/gallery-rust/` (already a workspace member from
   Phase 2 / Phase 3) builds and runs the grown sub-screen.
   `Start-Process` launch is recorded as successful by the
   assistant; **visual correctness** (viewport clips at its
   bottom edge; programmatic scroll button moves the content;
   clipped content is invisible; thumbnails outside the viewport
   come into view as scroll progresses; clipping edge is sharp)
   is **owner-manual GUI smoke** per framing decision G — the
   assistant does not assert on pixel- or eyeball-level
   correctness.

Items (1)–(4) are required for A5 acceptance; item (5) ties the
evidence back to the m3-plan target-app trajectory and grows the
gallery sub-screen Phase 2 / Phase 3 seeded with the canonical
`ScrollView { WrapPanel { … } }` composition Phase 7's iteration
grammar will later swap for collection-driven generation as a
strict superset. C and Zig hosts for the ScrollView sub-screen
are explicitly **not** required in Phase 4 (per framing decision
E and the Out of scope list); Phase 8 broadens the full gallery
to all three.

Per [m3-phase-4 pre-doc-inputs §10](../notes/m3-phase-4/pre-doc-inputs.md),
evidence items (1)–(4) do not collapse into one even though they
share helper infrastructure — the `wasamoc check` diagnostics,
the measure-arrange tests, the IR-load `validate()` gate tests,
and the Windows integration test (with R2 closure) each have
distinct evidence meanings.

The acceptance / non-acceptance of test items (1)–(5) is the
operational form of "Phase 4 done"; the corresponding
implementation checklist (which crate / which test file / which
fixture) belongs in the Phase 4 progress file, not here.

## M4 hand-off

Per the DD-003 scoping intent paragraph, Phase 4's `offset-y` is
one bindable control surface; the following surfaces are
explicitly **anticipated for M4 or beyond** and are documented
here so the M4 input / scrollbar / animation work has a named
landing point:

1. **Input-driven internal scrolling.** Wheel handler, drag
   handler, keyboard PgUp / PgDn / Home / End / arrow key
   gestures. These mutate the ScrollView's offset *directly*
   inside the runtime, without traversing any author-bound state.
   Lands as part of M4 input handling.

2. **Optional state write-back / in-out binding.** The deferred
   DD-003 Option C shape. When the author has bound `offset-y` to
   a state and the runtime mutates the offset via item 1 above,
   the runtime writes the new value back through the binding.
   Requires building the typed-`i32` evaluator / writer pair —
   the "third pair" anticipated in
   [architecture.md §6.8 *Per-type seam* paragraph](../architecture.md#68-reactive-engine-m2-phase-5).
   The Phase 4 surface (read-only binding) is forward-compatible:
   `offset-y: \{state.scroll_y}` remains valid syntax when M4
   adds in-out direction; no IR change is required.

3. **Scrollbar widget synchronization.** A separate widget
   (likely `ScrollBar` as a sibling primitive, not built into
   ScrollView) whose position both reflects and drives the
   ScrollView offset. Two-way coupling between scrollbar drag
   and content offset rides item 2's writer surface.

4. **Imperative `scroll_to(x, y)` / `scroll_by(dx, dy)` command
   surface.** Host-facing API for programmatic-without-state-
   binding scrolling. Analogue of WPF's
   `ScrollViewer.ScrollToVerticalOffset` or SwiftUI's
   `ScrollViewReader`. Useful for "jump to anchor" / "scroll
   to highlighted item" host code paths that should not be
   forced through DSL state binding.

None of items 1–4 require modifying Phase 4's `offset-y`
attribute, IR shape, default behaviour, or measure-arrange
algorithm. All four are additive on top of the Phase 4 surface.

## Out of scope

The following are not included in Phase 4 and are not deferred
by oversight — each is explicitly out of A5 minimal scope or
deferred to a later phase / milestone:

- **Scrollbar widget**, wheel handler, drag — A5 explicit
  deferral to M4. Phase 4 sub-screen demonstrates programmatic
  scroll (via the Button widgets) not user-input scroll.
- **Horizontal and bidirectional scroll axes** — DD-001 hardcodes
  vertical-only; later phase adds attribute additively.
- **`viewport-width` / `viewport-height` attributes** — DD-002
  defers; parent passthrough is the Phase 4 path.
- **`f64` / ratio offset surface** — DD-003 ships `i32` pixels;
  ratio is a sibling future addition.
- **In-out offset binding (writer seam)** — DD-003 ships
  read-only; writer seam is an M4 hand-off (see §M4 hand-off
  item 2).
- **Future scroll-model surfaces (M4+)** — input-driven internal
  scrolling, in-out binding, scrollbar widget synchronization,
  imperative scroll commands (see §M4 hand-off items 1–4).
- **Over/under-scroll**, bounce, momentum — touch-flick / smooth-
  scroll territory, M4 input + animation.
- **Background `fill` on ScrollView** — Phase 4 does not
  introduce a ScrollView-level `fill` attribute; the visible
  background is whatever parent / sibling provides.
- **Nested ScrollViews** — structurally permitted (nothing in
  the IR or layout forbids it), but Phase 4 ships no test
  fixture or sub-screen exercising the case. The unbounded-
  parent runtime error from DD-002 covers the pathological
  inner ScrollView whose parent is itself an unbounded
  ScrollView.
- **Image widget as scroll content** — Image deferred to M4 per
  Phase 2 DD-006 / M3 plan; Phase 4 sub-screen content is
  Box + Text placeholders.
- **`TypedValue` generic value union** (F5 maintained — Phase 4
  introduces no new scalar type per DD-003).
- **Padding on ScrollView** — out of A5 minimal; defer to later
  phase if needed.
- **R1 (`.gitignore` `*.uic`)** — Phase 3 carry-over residual;
  cross-cutting hygiene unrelated to ScrollView; defer per Phase
  4 framing decision F (R2 closes inside Phase 4; R1 stays
  open).

## Upstream document revisions (Moment 1 / Moment 2)

Phase 4 inherits the two-moment structure from
[m3-phase-2 framing decision D](../notes/m3-phase-2/m3-phase-2-pre-doc-framing.md#d-upstream-document-revision-timing-two-sync-moments)
and Phase 3's same-shape inheritance, per Phase 4 pre-doc framing
decision D. Doc set and commit shape follow the living rule in
[retrospectives.md](../notes/retrospectives.md) (framings inherit
the *structure*, not the historical doc list verbatim — see the
operational note at retrospectives.md §phase-sync). The Phase 4
`dsl_spec.md` section marker mirrors the Phase 2 / Phase 3 form:

```
**Phase status:** M3-Phase 4 ADR-accepted design draft; pending
implementation re-sync
```

flipping at phase close to:

```
**Phase status:** M3-Phase 4 closed; implementation-synced
```

placed as the first line under the ScrollView chapter heading
(new §4.11 alongside Phase 2's §4.9 Box and Phase 3's §4.10
WrapPanel chapters).

**Moment 1 — ADR Accepted commit set (design-spec draft).**
Constituent commits, each landing as its own commit on the
pre-doc branch per the per-review-concern rule in
[CLAUDE.md §Commit rules](../../CLAUDE.md#commit-rules) and
[retrospectives.md](../notes/retrospectives.md). The draft-side
doc set Phase 4 commits to at Moment 1 is enumerated below;
retrospectives.md §phase-sync で触る doc セット規定は phase-end
Moment 2 を対象とした規範であり、Moment 1 の draft set は
その mirror として **直接同一視されるものではない**:

- `docs/decisions/m3-phase-4-scroll-view.md` — ADR
  `Status: Accepted` flip (this file).
- `docs/dsl_spec.md` — new §4.11 ScrollView chapter as design-
  spec draft (DD-005 sub-issues 1–10 as the chapter outline; the
  ScrollView mental model + ecosystem-contrast subsection per
  Phase 4 framing decision I).
- `docs/architecture.md` — ScrollView entry under the M2-revised
  IR section; layout-engine section updated for the new pure-data
  ScrollView types; §6.5 may receive a one-paragraph addition
  naming Visual.Offset as the scroll-position primitive (cross-
  reference target for DD-004).
- `docs/abi_spec.md` — **no touch expected** per DD-005 / DD-006
  (LayoutError stays internal; no new ABI tag).
- `docs/plans/m3-plan.md` — Progress section's Phase 4 row
  populated (Status: in progress; Progress file link; ADR link).
- `docs/plans/progress/m3-phase-4-progress.md` — opens with
  `status: active`, task list mapped to the verification closure
  items above.

Implementation begins only after these commits land.

**Moment 2 — Phase close commit set (impl re-sync).**

- `docs/dsl_spec.md` §4.11 — section marker flips to "closed;
  implementation-synced", plus any corrections required if the
  design draft and implementation diverged (marker flip is
  required regardless of divergence; corrections are conditional
  on what re-sync surfaces). Per
  [m3-phase-4 pre-doc-inputs §10 / retroactive spec-gap fold](../notes/m3-phase-4/pre-doc-inputs.md)
  inherited from Phase 2 / Phase 3, earlier-phase spec gaps
  surfaced during the re-sync may fold into the same commit with
  explicit owner confirmation.
- `docs/architecture.md` — top Status flips to `M3-Phase 4
  complete`; impl-divergent paragraphs re-synced.
- `docs/plans/progress/m3-phase-4-progress.md` — phase-close
  retrospective link, CI evidence pointer, impl summary; the
  progress file then enters the standard `active → closing →
  retired → archived` lifecycle.
- `docs/plans/m3-plan.md` Progress row — Status flips to
  complete.
- `docs/decisions/m3-phase-4-scroll-view.md` (this file) —
  touch only if one of the three retrospectives.md §phase-sync
  ADR-touch cases applies (AC discharged-vs-impl divergence;
  out-of-phase residual cross-ref; thesis-level finding).
- Step retro `phase-sync` items (per
  [retrospectives.md item 10](../notes/retrospectives.md#step-end-固有-merge--phase-ブランチ))
  must all close into `doc-folded` / `carry-forward` /
  `local-only` at Moment 2 — no open `phase-sync` items survive
  past phase close. Phase 4 is the first phase to use the item
  10 disposition vocabulary from T1.

No ROADMAP revision is anticipated — A5 is already explicit; this
ADR operationalises it.

## Inputs absorbed

Mapping from [pre-doc-framing.md](../notes/m3-phase-4/pre-doc-framing.md)
framing decisions to DDs and ADR sections:

| Framing decision | Disposition | Consumed at |
|---|---|---|
| A — Typed-`i32` writer pair deferred to M4 | Cross-DD intent | DD-003 (Recommendation; scoping intent paragraph); §M4 hand-off item 2 |
| B — DD slate completeness check | Discipline | DD slate (6 DDs); §Out of scope (everything not A5) |
| C — Verification strategy (4 evidence categories + owner GUI smoke) | Constraint | §Phase 4 verification closure items 1–5 |
| D — Two-moment sync structure | Constraint | §Upstream document revisions (Moment 1 / Moment 2) |
| E — Sibling `ScrollView { WrapPanel { Box × 30–40 } }` slice | Constraint | §Phase 4 verification closure item 5 (gallery sub-screen growth) |
| F — R2 closes inside Phase 4 | Direct input | DD-004 (R2 closure paragraph); §Phase 4 verification closure item 4 (R2 closure assertion) |
| G — GUI smoke responsibility separation | Discipline | §Phase 4 verification closure item 5 (owner-manual GUI smoke clause) |
| H — Live-note re-evaluation triggers | Disposition table | (No direct ADR section — the framing's per-note disposition feeds DD layering and §Out of scope; the live notes themselves are not modified by Phase 4 unless framing decision F's R2-related architecture.md update warrants it) |
| I — ScrollView mental model + ecosystem contrast subsection | Spec content | DD-005 §Spec content seed item 10; the subsection lands in dsl_spec.md §4.11 at Moment 1 |

Mapping from [pre-doc-framing.md](../notes/m3-phase-4/pre-doc-framing.md)
DD slate to this ADR's DD numbering: 1:1 (DD-001 → DD-M3-P4-001
etc.; the framing's recommendation directions are consumed as the
recommended Options of each DD here).

## Revision history

| Date | Change |
|---|---|
| 2026-05-25 | Initial draft (Status: Proposed). All 6 DDs at Proposed pending owner review pass. Framing-level owner alignment confirmed in chat 2026-05-25 (commits `8f19c5f`, `234a0fa` for pre-doc-framing.md). |
