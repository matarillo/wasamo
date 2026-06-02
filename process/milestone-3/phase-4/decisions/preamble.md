# M3-Phase 4 — ScrollView primitive (minimal): Architecture Decisions

**Phase:** M3-Phase 4 (ScrollView primitive — minimal)
**Date:** 2026-05-25
**Status:** Accepted

## Context

M3 acceptance criterion **A5** (see
[process/_roadmap.md M3](../../../_roadmap.md#m3-dsl-surface),
[m3-plan.md §Acceptance criteria](../../plan.md#acceptance-criteria)):

> ScrollView primitive (minimal: inner unbounded measure + viewport
> clip + content offset binding; scrollbar widget, wheel handler,
> and drag are deferred to M4).

The pre-doc framing for this phase was aligned with the owner on
2026-05-25 and is recorded in
[docs/notes/m3-phase-4/pre-doc-framing.md](../requirements/framing.md)
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
([m3-phase-2 framing notes](../../phase-2/requirements/framing.md))
and Phase 3's same-shape inheritance, the
"Moment is not a commit unit" rule applies: each upstream-document
edit in a Moment lands as its own commit on the pre-doc branch,
scoped by review concern per
[CLAUDE.md §Commit rules](../../../../CLAUDE.md#commit-rules) and the
doc set in
[retrospectives.md §phase-sync (Moment 2) で触る doc セット](../../../procedures/retrospectives.md#phase-sync-moment-2-で触る-doc-セット).

The M2 / M3-Phase-1 / M3-Phase-2 / M3-Phase-3 end-state shape that
this phase extends without breaking:

- `wasamo-ir` ([wasamo-ir/src/lib.rs](../../../../wasamo-ir/src/lib.rs)):
  `IrType` is `I32 | Str | Bool`; `IrLiteral` is `Int | Str | Ident
  | Bool | Ratio | Color`. Phase 3 reused existing `i32` plumbing
  for all WrapPanel attributes without widening either enum.
  Phase 4 likewise introduces **no new `IrType`, no new
  `IrLiteral` variant, no new `PropertyValue` variant** —
  `offset-y` is `i32` per DD-003. Binding reuses the existing
  i32 reader / binding-effect machinery (`HandlerExpr::PropRead`
  reads the `Signal<i32>`; `register_binding` +
  `widget_write_property` build a `PropertyValue::String` from
  the stringified result); the narrow string-to-`i32` parse step
  needed to land the value in ScrollView's `i32` `offset-y`
  field happens at ScrollView's per-widget `set_property` arm.
  No general typed-`i32` evaluator / writer pair is built (per
[architecture.md §6.7 *Per-type seam* paragraph](../../../../docs/architecture.md#67-reactive-engine-m2-phase-5);
  the "third pair" stays deferred — see §M4 hand-off item 2).
- `wasamo-runtime` widget catalog
  ([wasamo-runtime/src/widget.rs](../../../../wasamo-runtime/src/widget.rs)):
  `Rectangle | VStack | HStack | Text | Button | Box | WrapPanel`
  (Phase 3 added `WrapPanel`). Phase 4 adds `ScrollView` as a
  per-kind tag (DD-001).
- Layout engine
  ([wasamo-runtime/src/layout.rs](../../../../wasamo-runtime/src/layout.rs)):
  pure-data `LayoutNode` / `measure` / `arrange` boundary,
  Win32/WinRT-free. Phase 2 introduced
  `LayoutError::{BoxAspectUnboundedBoth, BoxNoExtent}`; Phase 3
  did not extend the error class (one-line-flow disposition for
  unbounded main-axis); Phase 4 extends it with
  `LayoutError::ScrollViewUnboundedAxis` per DD-002 / DD-005.
- Binding pipeline: per-type writer seam pattern (DD-M3-P1-007).
  Phase 4's `offset-y` is bindable read-only (DD-003). The
  existing i32 reader path (`HandlerExpr::PropRead` →
  `Signal<i32>`) and binding-effect re-run path are reused
  unchanged; the stringified value `widget_write_property`
  passes (as `PropertyValue::String`) is parsed into ScrollView's
  `i32` `offset-y` field at ScrollView's per-widget
  `set_property` arm. **No general typed-`i32` evaluator /
  writer pair is built** — the anticipated "third pair" from
[architecture.md §6.7 *Per-type seam* paragraph](../../../../docs/architecture.md#67-reactive-engine-m2-phase-5)
  is deferred to M4 or later input-handling work (see §M4 hand-off
  below). F5 (`TypedValue` deferral) is held in force by
  construction.
- `wasamoc` ([wasamoc/src/check.rs](../../../../wasamoc/src/check.rs)):
  state-name → declared-type table; identifier resolution lowers
  to typed `*PropRead` variants. Phase 4 adds no new value type;
  ScrollView's `offset-y` is an `i32` literal (or `i32` binding)
  and `wasamoc check` rejects non-integer literals through the
  existing diagnostic surface. ScrollView gains a child-count
  diagnostic (exactly 1 child) per DD-006.
- Composition / Visual Layer:
([architecture.md §6.5](../../../../docs/architecture.md#65-widgetnode-and-visual-layer-sync))
  `LayoutNode` offsets are absolute (root-relative); `sync_visuals()`
  converts each child offset to parent-relative `Visual.Offset`
  before writing the Composition visual tree. The current
  convention is **1 WidgetNode = 1 Visual**: `append_child`
  inserts the child's `WidgetNode.visual` directly beneath the
  parent's `WidgetNode.visual`, and `sync_visuals` writes
  offset/size to each WidgetNode's own Visual. Phase 4 ScrollView
  **locally extends this convention** by owning a ScrollView-
  internal intermediate Visual between its outer Visual and its
  single content child's widget Visual (per DD-004): the outer
  Visual carries `Visual.Clip = InsetClip{0,0,0,0}`; the
  intermediate Visual carries the scroll translation
  `Visual.Offset = (0, -offset_y, 0)`; the child widget's own
  Visual continues to carry its layout-derived offset via
  `sync_visuals` as usual.

This ADR is framed against A5 and the m3-plan's "minimal: inner
unbounded measure + viewport clip + content offset binding"
phrasing
([m3-plan.md §Acceptance criteria](../../plan.md#acceptance-criteria)).
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
declares `ScrollView { offset-y: <i32-literal-or-state-ident>; <single
content child> }` (where the state-ident form is a bare identifier RHS
per [dsl_spec.md §4.3](../../../../docs/dsl_spec.md#43-property-binding) property-
binding semantics, resolving to a component-scope
`state scroll_y: i32 = 0` declaration per
[dsl_spec.md §4.7](../../../../docs/dsl_spec.md#47-state-declarations-m2-surface-bool-added-in-m3-phase-1)
state-declaration grammar, **not** a `\{…}` interpolation — see
Revision history 2026-05-25 erratum) and the shared crates lower →
load → render it
with correct inner-unbounded measure + viewport clip + offset
application, (ii) the ScrollView chapter lands in `docs/dsl_spec.md`
§4.11 as a normative spec at the milestone-end-criteria bar
([m3-plan.md §Milestone-end criteria item 5](../../plan.md#milestone-end-criteria))
applied at phase close, and (iii) `examples/gallery/` +
`examples/gallery-rust/` are grown additively with a sibling
`ScrollView { WrapPanel { Box × 30–40 } }` slice (the Phase 3
standalone WrapPanel slice stays untouched). Per A11, all sides
advance together by phase close.

### Governance note (M3 phase-ADR vs RFC transition)

The governance question Phase 3 ADR raised (M3-onward RFC wording
in VISION §9.2 / decisions-README vs realised phase-ADR practice)
was resolved upstream by
[vision-governance-rfc-deferral.md (DD-V-018)](../../../cross-milestone/decisions/governance-rfc-deferral.md)
on 2026-05-25 (commit `632a30b docs: apply DD-V-018 — defer RFC
adoption to post-1.0`). VISION §9.2 / §11 and decisions-README
now describe a two-stage governance policy: pre-1.0 BDFL + ADRs,
post-1.0 open + RFC. M3 is pre-1.0, so ADRs remain the
authoritative format. This Phase 4 ADR continues the M3 phase-ADR
pattern Phase 1 / Phase 2 / Phase 3 established, without the
"transition disconnect" caveat the Phase 3 ADR's governance note
carried.

## Decisions

## Phase 4 verification closure (what counts as A5 evidence)

This section is not a DD — it records the agreed shape of the
proof that closes Phase 4 per framing decision C, so the
implementation plan inherits a concrete target.

A5 (ScrollView minimal — inner unbounded measure + viewport clip
+ content offset binding) has two evidence layers:

- **Automated / CI-gated A5 evidence:** items (1)–(4).
- **Phase-close / A11 gallery proof:** item (5), including the
  owner-manual visible smoke for the grown gallery sub-screen.

Phase 4 closes only when **all five** of the following are
observed:

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
   - **`offset-y` binding admission** — `offset-y: scroll_y`
     (bare state identifier RHS per
     [dsl_spec.md §4.3](../../../../docs/dsl_spec.md#43-property-binding))
     accepted when `scroll_y` is declared as
     `i32` in `state`; rejected when `scroll_y` is undeclared,
     `bool`, or `String`. Reuses the existing i32 reader /
     binding-effect machinery; the runtime-side narrow
     ScrollView parse / write bridge is covered by item 4.
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
     [m3-phase-4 pre-doc-inputs §6](../requirements/constraints.md)).
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
   [CLAUDE.md §Testing rules](../../../../CLAUDE.md#testing-rules))
   on the Windows CI runner exercises:

   - **Scroll-path fixture (primary).** A `.ui` declares a
     ScrollView with `offset-y: scroll_y` containing a
     `VStack { Box × N }` whose total height exceeds a known
     viewport size, inside a fixture whose root / parent
     allocation supplies known bounded viewport dimensions.
     `state.scroll_y` is declared `i32` initialised to 0. The
     test loads the IR, runs the layout pass, and asserts:
     - (a) the ScrollView's resolved rectangle matches the
       expected viewport dimensions (per DD-005 outer size =
       viewport);
     - (b) the ScrollView-owned intermediate content Visual's
       `Visual.Offset` is `(0, 0, 0)` when `scroll_y = 0`;
     - (c) after mutating `state.scroll_y = 100`, the
       ScrollView-owned intermediate content Visual's
       `Visual.Offset` becomes `(0, -100, 0)`;
     - (d) after mutating `state.scroll_y = -50`, the
       ScrollView-owned intermediate content Visual's
       `Visual.Offset` becomes `(0, 0, 0)` (clamped to 0 per
       DD-005);
     - (e) after mutating `state.scroll_y` to a value larger
       than `content_h - viewport_h`, the ScrollView-owned
       intermediate content Visual's `Visual.Offset` becomes
       `(0, -(content_h - viewport_h), 0)` (clamped to max per
       DD-005);
     - (f) ScrollView's outer Visual has a `Visual.Clip`
       property set to a non-null clip (the InsetClip from
       DD-004) — clip **presence** assertion;
     - (g) the ScrollView-owned intermediate content Visual and
       the single content child widget Visual both have
       `Visual.Clip = null` — clip **absence** regression guard
       (the symmetric inverse of Phase 3 T8's WrapPanel clip-
       absence assertion).
   - **R2 closure — three-level nested offset assertion (R2
     test-coverage half from Phase 3).** The scroll-path fixture
     above traverses three levels of `Visual.Offset` nesting:
     parent → ScrollView Visual (at some `(parent_offset_x, …,
     0)`) → ScrollView-owned intermediate content Visual (at
     `(0, -offset_y, 0)`) → Box thumbnails inside content (at
     their own layout-derived offsets). The test asserts that
     each thumbnail's *root-relative* position (computed by
     summing parent-relative offsets up the chain) equals the
     expected position given the scroll state — i.e. the
     absolute-vs-parent-relative convention from
[architecture.md §6.5](../../../../docs/architecture.md#65-widgetnode-and-visual-layer-sync)
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

Items (1)–(4) are the automated A5 evidence set. Item (5) is
required for Phase 4 close under A11: it ties the evidence back
to the m3-plan target-app trajectory and grows the gallery sub-
screen Phase 2 / Phase 3 seeded with the canonical `ScrollView {
WrapPanel { … } }` composition Phase 7's iteration grammar will
later swap for collection-driven generation as a strict
superset. C and Zig hosts for the ScrollView sub-screen are
explicitly **not** required in Phase 4 (per framing decision E
and the Out of scope list); Phase 8 broadens the full gallery to
all three.

Per [m3-phase-4 pre-doc-inputs §10](../requirements/constraints.md),
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
here so the M4+ input / scrollbar / animation work has a named
landing point:

1. **Input-driven internal scrolling.** Wheel handler, drag
   handler, keyboard PgUp / PgDn / Home / End / arrow key
   gestures. These mutate the ScrollView's offset *directly*
   inside the runtime, without traversing any author-bound state.
   This is the natural landing point for M4 input handling or a
   later input-focused phase.

2. **Optional state write-back / in-out binding.** The deferred
   DD-003 Option C shape. When the author has bound `offset-y` to
   a state and the runtime mutates the offset via item 1 above,
   the runtime writes the new value back through the binding.
   Requires building the general typed-`i32` writer pair — the
   "third pair" anticipated in
   [architecture.md §6.7 *Per-type seam* paragraph](../../../../docs/architecture.md#67-reactive-engine-m2-phase-5).
   The Phase 4 surface (read-only binding) is forward-compatible:
   `offset-y: scroll_y` remains valid syntax when M4 or
   later work adds in-out direction; no IR change is required.

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
  deferral beyond Phase 4 / to M4 or later. Phase 4 sub-screen
  demonstrates programmatic scroll (via the Button widgets) not
  user-input scroll.
- **Horizontal and bidirectional scroll axes** — DD-001 hardcodes
  vertical-only; later phase adds attribute additively.
- **`viewport-width` / `viewport-height` attributes** — DD-002
  defers; parent passthrough is the Phase 4 path.
- **`f64` / ratio offset surface** — DD-003 ships `i32` pixels;
  ratio is a sibling future addition.
- **In-out offset binding (writer seam)** — DD-003 ships
  read-only; writer seam is an M4+ hand-off (see §M4 hand-off
  item 2).
- **Future scroll-model surfaces (M4+)** — input-driven internal
  scrolling, in-out binding, scrollbar widget synchronization,
  imperative scroll commands (see §M4 hand-off items 1–4).
- **Over/under-scroll**, bounce, momentum — touch-flick / smooth-
  scroll territory, M4+ input + animation.
- **Background `fill` on ScrollView** — Phase 4 does not
  introduce a ScrollView-level `fill` attribute; the visible
  background is whatever parent / sibling provides.
- **Nested ScrollViews** — structurally permitted (nothing in
  the IR or layout forbids it), but Phase 4 ships no test
  fixture or sub-screen exercising the case. The unbounded-
  parent runtime error from DD-002 covers the pathological
  inner ScrollView whose parent is itself an unbounded
  ScrollView.
- **Image widget as scroll content** — Image deferred to M4 or
  later per Phase 2 DD-006 / M3 plan; Phase 4 sub-screen content
  is Box + Text placeholders.
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
[m3-phase-2 framing decision D](../../phase-2/requirements/framing.md#d-upstream-document-revision-timing-two-sync-moments)
and Phase 3's same-shape inheritance, per Phase 4 pre-doc framing
decision D. Doc set and commit shape follow the living rule in
[retrospectives.md](../../../procedures/retrospectives.md) (framings inherit
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
[CLAUDE.md §Commit rules](../../../../CLAUDE.md#commit-rules) and
[retrospectives.md](../../../procedures/retrospectives.md). The draft-side
doc set Phase 4 commits to at Moment 1 is enumerated below;
retrospectives.md §phase-sync で触る doc セット規定は phase-end
Moment 2 を対象とした規範であり、Moment 1 の draft set は
その mirror として **直接同一視されるものではない**:

- `process/milestone-3/phase-4/decisions/preamble.md` — ADR
  `Status: Accepted` flip (this file).
- `docs/dsl_spec.md` — new §4.11 ScrollView chapter as design-
  spec draft (DD-005 sub-issues 1–10 as the chapter outline; the
  ScrollView mental model + ecosystem-contrast subsection per
  Phase 4 framing decision I).
- `docs/architecture.md` — ScrollView entry under the M2-revised
  IR section; binding section clarifies the existing i32 reader /
  binding-effect path versus Phase 4's narrow `offset-y` parse /
  write bridge; layout-engine section updated for the new pure-
  data ScrollView types; §6.5 updated to record the ScrollView-
  owned intermediate content Visual and its `Visual.Offset`
  scroll translation.
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
  [m3-phase-4 pre-doc-inputs §10 / retroactive spec-gap fold](../requirements/constraints.md)
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
- `process/milestone-3/phase-4/decisions/preamble.md` (this file) —
  touch only if one of the three retrospectives.md §phase-sync
  ADR-touch cases applies (AC discharged-vs-impl divergence;
  out-of-phase residual cross-ref; thesis-level finding).
- Step retro `phase-sync` items (per
  [retrospectives.md item 10](../../../procedures/retrospectives.md#step-end-固有-merge--phase-ブランチ))
  must all close into `doc-folded` / `carry-forward` /
  `local-only` at Moment 2 — no open `phase-sync` items survive
  past phase close. Phase 4 is the first phase to use the item
  10 disposition vocabulary from T1.

No ROADMAP revision is anticipated — A5 is already explicit; this
ADR operationalises it.

## Inputs absorbed

Mapping from [pre-doc-framing.md](../requirements/framing.md)
framing decisions to DDs and ADR sections:

| Framing decision | Disposition | Consumed at |
|---|---|---|
| A — Typed-`i32` writer pair deferred to M4+ | Cross-DD intent | DD-003 (Recommendation; scoping intent paragraph); §M4 hand-off item 2 |
| B — DD slate completeness check | Discipline | DD slate (6 DDs); §Out of scope (everything not A5) |
| C — Verification strategy (4 evidence categories + owner GUI smoke) | Constraint | §Phase 4 verification closure items 1–5 |
| D — Two-moment sync structure | Constraint | §Upstream document revisions (Moment 1 / Moment 2) |
| E — Sibling `ScrollView { WrapPanel { Box × 30–40 } }` slice | Constraint | §Phase 4 verification closure item 5 (gallery sub-screen growth) |
| F — R2 closes inside Phase 4 | Direct input | DD-004 (R2 closure paragraph); §Phase 4 verification closure item 4 (R2 closure assertion) |
| G — GUI smoke responsibility separation | Discipline | §Phase 4 verification closure item 5 (owner-manual GUI smoke clause) |
| H — Live-note re-evaluation triggers | Disposition table | (No direct ADR section — the framing's per-note disposition feeds DD layering and §Out of scope; the live notes themselves are not modified by Phase 4 unless framing decision F's R2-related architecture.md update warrants it) |
| I — ScrollView mental model + ecosystem contrast subsection | Spec content | DD-005 §Spec content seed item 10; the subsection lands in dsl_spec.md §4.11 at Moment 1 |

Mapping from [pre-doc-framing.md](../requirements/framing.md)
DD slate to this ADR's DD numbering: 1:1 (DD-001 → DD-M3-P4-001
etc.; the framing's recommendation directions are consumed as the
recommended Options of each DD here).

## Revision history

| Date | Change |
|---|---|
| 2026-05-25 | Erratum: corrected `offset-y` binding surface notation. Earlier draft examples wrote the state-bound surface as `offset-y: \{state.scroll_y}`; the actual DSL property-bind surface is `offset-y: scroll_y` (bare state identifier) per existing [dsl_spec.md §4.3](../../../../docs/dsl_spec.md#43-property-binding) property-binding surface, with `state scroll_y: i32 = 0` declared at component scope per [dsl_spec.md §4.7](../../../../docs/dsl_spec.md#47-state-declarations-m2-surface-bool-added-in-m3-phase-1). The `\{…}` syntax is reserved for string interpolation inside string literals per [dsl_spec.md §2.4](../../../../docs/dsl_spec.md#24-string-literals). No design decision changes; the bindable read-only direction of DD-003 is unaffected. |
| 2026-05-25 | Status flipped to Accepted. DD-001 through DD-006 owner-accepted after per-DD review, implementation-shape recheck against existing runtime code, and final Verification / Out of scope / Upstream revisions alignment. |
| 2026-05-25 | Initial draft (Status: Proposed). All 6 DDs at Proposed pending owner review pass. Framing-level owner alignment confirmed in chat 2026-05-25 (commits `8f19c5f`, `234a0fa` for pre-doc-framing.md). |
