# Changelog

All notable shipped milestones for Wasamo. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) at
milestone granularity (see
[DD-V-013](./process/cross-milestone/decisions/doc-system.md#dd-v-013--changelog-granularity-and-length-control)).
Per-phase decision records live under
[process/](./process/); per-release notes live in
[GitHub Releases](https://github.com/matarillo/wasamo/releases).

This file records what has shipped. For what is planned, see
[process/_roadmap.md](./process/_roadmap.md). For the current state of work, see
the **Status** section of [README.md](./README.md).

## [v0.3.0] — 2026-07-06 — M3: DSL surface

M3 grows the DSL surface from "one counter" to real layouts and
publishes the result. Five layout primitives (Box, WrapPanel,
ScrollView, Grid, ZStack), two grammar surfaces (conditional rendering,
iteration), the `bool` scalar, the `ToggleButton` / `checked`
selected-state surface, and the parent-interpreted `slot.*` placement
surface are proven end-to-end by the integrated Photo Gallery on the
C / Rust / Zig hosts, and `docs/dsl_spec.md` is promoted to its first
**public draft** (v1.15). The milestone discharges acceptance criteria
A1–A13; the discharge mapping is recorded in
[process/milestone-3/plan.md](./process/milestone-3/plan.md)
§Milestone close, and the M3 → M4 carry-forward in
[process/milestone-3/handoff.md](./process/milestone-3/handoff.md).

### M3-Phase 8 — ToggleButton selected state, Gallery integration, DSL spec public draft (2026-07-06)

Closes M3 implementation with three deliverables and no new layout
primitive. This is the M3 milestone entry: it links each M3 phase
decision record and the public-draft anchor.

**A10 — `ToggleButton` / `checked`.** A dedicated `ToggleButton`
widget carries Button's `text` / `style` / `enabled` / `clicked` plus
exactly one new attribute: a controlled one-way `checked` boolean
(literal or bool-state binding; runtime default `false`;
background-colour-only selected visual composing with `style` and the
disabled contract). `checked` is admitted on `ToggleButton` only —
compiler reject plus runtime loader re-reject, each with firing tests.
Exactly-one-selected tab exclusion is author-composed in `clicked`
handlers (an M3-era pattern, not a reserved idiom). No new IR type,
binding-target class, or C ABI surface (`abi_spec.md` untouched).

**A1 — integrated Photo Gallery on all three hosts.** The per-phase
verification screens folded into a single Photo Gallery app
(`examples/gallery/gallery.ui`): stretch Grid frame with spans and
`Cell` / direct `slot.*` placement, live-exclusion `ToggleButton` tab
band, ScrollView + WrapPanel + `for`-generated thumbnail grid, aspect
Box placeholders, and a conditional lightbox overlay. New
`examples/gallery-c/` and `examples/gallery-zig/` hosts (ported from
the counter templates, declarative memory-load boundary preserved)
render the same surface as `gallery-rust`; CI builds all three.
Staged owner checkpoints G(1)–G(5) all passed, ending with the final
human-visible smoke.

**A12 — DSL spec public draft.** `docs/dsl_spec.md` is promoted to
`status: public-draft` after a whole-document editorial pass and an
external-reader smoke over every M3 surface; the promotion record is
the [public-draft change history](./docs/dsl_spec.md#public-draft-change-history)
anchor. The draft is not a backward-compatibility guarantee (an M6
concern).

M3 decision records (per phase):
[Phase 1 — `bool` scalar binding](./process/milestone-3/phase-1/decisions/preamble.md);
[Phase 2 — Box](./process/milestone-3/phase-2/decisions/preamble.md);
[Phase 3 — WrapPanel](./process/milestone-3/phase-3/decisions/preamble.md);
[Phase 4 — ScrollView](./process/milestone-3/phase-4/decisions/preamble.md);
[Phase 5 — Grid](./process/milestone-3/phase-5/decisions/preamble.md);
[Phase 6 — ZStack + conditional rendering](./process/milestone-3/phase-6/decisions/preamble.md);
[Phase 7 — iteration](./process/milestone-3/phase-7/decisions/preamble.md);
[Phase 7b — parent-interpreted placement](./process/milestone-3/phase-7b/decisions/preamble.md);
[Phase 8 — selected state + Gallery + public draft](./process/milestone-3/phase-8/decisions/preamble.md).

Carry-forwards live in the M3 handoff (roadmap / trigger-driven
residuals: PM-2 wrapper rule, author-controllable sizing / Problem B,
default alignment, placement spelling and bindability, the five
selected-state deferred axes, the M4 residual cluster).

### M3-Phase 7b — Parent-interpreted placement (2026-06-24)

Aligns the parent-interpreted placement surface shipped piecemeal in
Phases 5–7 onto one author surface and one internal model, discharging M3
acceptance **A13** — before the Phase 8 public draft freezes it. A
corrective phase: no new layout primitive, no new app feature.

Author surface: Grid cell placement (`row` / `column` / `row-span` /
`column-span` / `h-align` / `v-align`) and ZStack alignment (`h-align` /
`v-align`) are authored as parent-interpreted **`slot.*`** metadata, not
intrinsic widget properties, unified on one `slot.` namespace. Grid admits
**both** a `Cell` grouped form and direct `slot.*` (one form per child,
strict mixing reject); ZStack moves bare `h-align` / `v-align` to
`slot.h-align` / `slot.v-align` (no long-lived alias). Placement is
constant per instance (a binding RHS is rejected).

Internal model: `wasamo-ir` carries placement on an explicit child slot
(`IrMember::Widget(IrChildSlot)` + `IrSlotData`); `wasamo-runtime` and the
layout tree store an explicit `ChildSlot` / `LayoutChildSlot` with a
broadly-named `SlotData` carrier, removing Grid's parallel
`cell_placements` vector (and its layout mirror) so a child and its
placement are one record. Textual IR normalises all three authored forms
to one `child { placement <kind> { … } node … }` record; stale `Cell` /
bare-placement IR is reject-and-regenerate. `wasamoc` lowers all forms to
the one record. No new C ABI surface (`abi_spec.md` untouched).

Visible proof: a placement-demo sub-screen in
`examples/gallery/gallery.ui` shows ZStack `slot.h-align` start / center /
end at three distinct positions and Grid stretch-default vs centered cells;
assistant screenshots + owner-manual smoke carry the positive controls, and
the migration is same-position-preserving against the pre-migration
baseline. Carry-forwards: the PM-2 pre-1.0 wrapper-rule decision, the
VS-2 / VS-3 carrier triggers, the Grid structural-mutation trigger,
bindable placement, and the author-controllable `width` / `height` sizing
gap (Phase 8 framing Vision DR).

### M3-Phase 7 — Iteration grammar (2026-06-18)

Adds the iteration grammar, discharging M3 acceptance **A8**. The author
surface is `for <binder> in <collection> { <one widget child> }` with an
optional index binder, over scalar collection state (`i32[]`, `string[]`,
`bool[]`). Collection mutations are expressed by the shipped whole-value
assignment forms: append, drop-last, clear, and static-literal reset.

`wasamo-ir` gains collection state types, list literals, loop-local item /
index reads, and `ControlFlowNode::For`. `wasamoc` reserves and implements
the `for` control-flow member, rejects unsupported placements and scopes,
emits the landed textual IR spellings, and keeps structured item values /
`TypedValue` out of Phase 7. `wasamo-runtime` materialises static iteration,
adds reactive `ForLoopSubtree` mutation with positional un-keyed identity,
stage-then-commit insertion, tail-first disposal, same-return drain behavior,
depth-based mutation-cap accounting, and rollback / cleanup coverage for
commit failures. No new C ABI surface is added.

Visible proof: `examples/gallery/gallery.ui` grows a collection-driven
thumbnail slice in the Rust gallery host. Assistant evidence and owner-manual
smoke cover Add / Remove / Clear / Reset cardinality changes with positive
controls, including a visible count trajectory proving that the generated set
tracks collection mutation rather than a hardcoded tree.

Per-phase spec sync (A11 / A12): `docs/dsl_spec.md` and
`docs/architecture.md` are implementation-synced for iteration grammar,
collection literals / assignment forms, textual IR spellings, positional
identity, runtime mutation timing, validation, child-carried ZStack placement,
and explicit deferrals. Phase 8 carries the positional baseline into the full
gallery assembly; per-item handlers / conditionals, keyed identity, nested
`for`, member-range bodies, general loop-external collection reads, and host
collection APIs remain deferred. The narrower per-item richness cluster
surfaced by the gallery — structured item fields / `TypedValue`,
loop-external indexed reads for paired item data, and bindable `Box.fill` /
dynamic styling — remains deferred unless the owner opens Phase 7b.

Decisions: [DD-M3-P7-001..007](./process/milestone-3/phase-7/decisions/preamble.md).

### M3-Phase 6 — ZStack + conditional rendering (2026-06-09)

Adds the `ZStack` overlay primitive and the first structural grammar surface,
`if <bool-expr> { <widget-child> }`, discharging M3 acceptance **A4** and
**A7**. `ZStack` uses direct children, parent-owned `h-align` / `v-align`,
union sizing, document-order z-order, and an outer-bounds clip without adding
a new ABI surface. Conditional rendering lands as an `IrMember` /
`ControlFlowNode` surface rather than a widget, with load-time presence,
reactive present / absent mutation, subtree disposal, fresh-on-return
semantics, declared Visual order, and same-drain effect observation.

The phase also closes the Phase 4 residual **R1**: static component
`title: "..."` now reaches the native window through the existing
`wasamo_load_ui` -> `window::create` path. DD-M3-P6-008 then moves host-owned
component attributes (`title`, `backdrop`, `theme`) to `IrComponent.host_props`
/ `host_bindings`, preserving the content-root separation and keeping dynamic
host bindings and ABI-facing window descriptors deferred.

Visible proof: `examples/gallery/gallery.ui` grows a root `ZStack` lightbox
slice driven by `is_lightbox_open`, with a scrim, centered 4:3 photo
placeholder, caption, nav, and the native `"Gallery"` title. Assistant
screenshot evidence proves closed / open / closed toggle behavior, z-order,
dimming, and title corroboration; owner-manual smoke confirmed toggle,
resize-fill positive control, title, photo geometry, and caption/nav fit after
the additive caption-row correction.

Per-phase spec sync (A11 / A12): `docs/dsl_spec.md` and
`docs/architecture.md` are implementation-synced for ZStack, structural
conditional semantics, textual IR control-flow members, component host
surfaces, and layout-dirty structural mutation. Direct conditional members
under `ScrollView` remain rejected in Phase 6; future conditionally-empty
ScrollView behavior must reopen that policy. Phase 7 carries the control-flow
family extension, placement storage-model decision, declared/entity identity,
and semantic-migration audit rule-ification; M4 carries modal input, dynamic
host bindings, DPI / text-metric sensitivity, and real image behavior.

Decisions: [DD-M3-P6-001..008](./process/milestone-3/phase-6/decisions/preamble.md).

### M3-Phase 5 — Grid layout primitive (2026-05-30)

Adds the `Grid` 2D layout primitive (one child per `Cell`; fixed and
weighted-star track sizing; row / column spanning), discharging M3
acceptance **A2** and the Phase 5 owner-acceptance slice of **A11**. A
`Grid` declares `columns:` / `rows:` track lists and contains `Cell`
children, each placing a single content child at a `(row, column)` with
optional `row-span` / `column-span` and per-cell `h-align` / `v-align`.
Same-cell overlap is rejected — overlay stays ZStack's responsibility.

`wasamo-ir` stays on the generic node path: Grid's track lists ride a new
`IrNode.kind_payload` carrier (`KindPayload::Grid { columns, rows }`,
DD-M3-P5-001 carrier c1), so `IrProp.value` remains strictly `IrLiteral`
for every kind, and `Cell` is an IR-only node flattened into Grid's
effective children at load. No new `IrType`, `IrLiteral`, `PropertyValue`,
`WASAMO_VALUE_*`, or `WASAMO_LAYOUT_ERROR_*` surface is added;
`LayoutError::GridUnboundedStarAxis` is runtime-internal (Grid adds no
host-facing ABI surface — `docs/abi_spec.md` re-confirmed untouched).

`wasamoc` registers `Grid`, adds a narrow Grid-scoped track-list parser
(a bare `*` lexer token; `columns: 120 1* 2*`) without opening a general
list grammar, and rejects malformed track lists, out-of-range placement /
span, multi-child or empty `Cell`, same-cell / overlapping-rectangle
conflicts, unknown Grid / Cell attributes, and the deferred `auto` token.
`wasamo-runtime` adds pure-data Grid measure / arrange (per-axis
fixed-first + weighted-star resolution with `f32` prefix boundaries,
spanning reconciliation, per-cell alignment with stretch default,
document-order z-order), IR-loader materialisation with `validate()`
defense-in-depth, and a Grid outer-bounds `InsetClip` Visual.

Visible proof: `examples/gallery/gallery.ui` grows additively with a 3×3
Grid slice (mixed fixed + weighted-star tracks, a column-spanning header
and footer, three middle-row cells, and an overflowing footer that
exercises the outer-bounds clip). Owner-manual GUI smoke on the rebuilt
gallery host confirmed the spanning header / footer, the three separate
star-sized columns (`C2` ≈ 2× `C1`, held across a resize positive
control), and the outer-bounds clip (the footer's 4:1 box leaves only a
thin clipped strip and never bleeds into the Photos below).

Per-phase spec sync (A11): `docs/dsl_spec.md` 1.3 -> 1.4 flips §4.12 and
the document status to `M3-Phase 5 closed; implementation-synced`, folds
the Grid carrier-c1 textual IR grammar into §8.5 (`track_decl`), and
re-syncs §5 / §2.2 / §3 to the landed author-surface AST / lexer / parser.
`docs/architecture.md` top-level Status flips to include M3-Phase 5
complete and §6.8.7 re-syncs to the landed `WidgetData::Grid`.

Phase 5 surfaced a pre-existing runtime gap — the runtime is per-monitor
DPI-unaware — deferred to M4 as a new acceptance criterion (DD-V-022 /
DD-V-023); Grid itself computes correctly in logical pixels. Out-of-phase
residual R1 (native Window-title wiring) carries forward to M3-Phase 6.

Decisions: [DD-M3-P5-001..006](./process/milestone-3/phase-5/decisions/preamble.md).

### M3-Phase 4 — ScrollView primitive (minimal) (2026-05-25)

Adds the vertical-only `ScrollView` layout primitive, discharging M3
acceptance **A5** and the Phase 4 owner-acceptance slice of **A11**.
`ScrollView` admits exactly one content child, fills the parent-supplied
viewport, measures content with bounded width and unbounded vertical
space, clips content at the viewport, and translates the content by a
clamped `offset-y` value.

`wasamo-ir` remains on the generic node / integer-property path:
`ScrollView` is another `IrNode.widget_type`, `offset-y` uses existing
`IrLiteral::Int` / `IrType::I32` plumbing, and no new `IrType`,
`IrLiteral`, `PropertyValue`, `WASAMO_VALUE_*`, or
`WASAMO_LAYOUT_ERROR_*` surface is added. `wasamoc` registers
`ScrollView`, enforces exactly one child, accepts `offset-y` as an
`i32` literal or bare `i32` state identifier, rejects unknown
ScrollView attributes (`viewport-*`, `scroll-axis`, `padding`, and
others), and rejects writable `in-out offset-y` forms until a future
write-back surface exists. `wasamo-runtime` adds pure-data layout
support, IR-loader defense-in-depth for child count, a narrow
ScrollView string-to-`i32` binding write bridge, an outer clipped
Visual, and a ScrollView-owned intermediate content Visual that carries
the scroll translation.

Visible proof: `examples/gallery/gallery.ui` grows additively from the
Phase 3 WrapPanel sub-screen into a `VStack` with Button-driven
`scroll_y` controls and a `ScrollView { WrapPanel { Box × 32 } }`
thumbnail slice. Owner-manual GUI smoke on the rebuilt gallery host
confirmed sharp clipping, visible +100 / -100 movement, hidden
off-viewport content, and thumbnails entering view as `scroll_y`
progressed. The T6 visible-smoke failure uncovered the window-root
`VStack` / Fill-child collapse; the runtime now uses
`WidgetNode::run_layout_as_window_root` for real window roots so the
client rect, not the root container's declared shrink height, owns the
top-level viewport.

Per-phase spec sync ([A11](./process/_roadmap.md#m3-dsl-surface)):
`docs/dsl_spec.md` 1.1 -> 1.2 flips §4.11 to
`M3-Phase 4 closed; implementation-synced`, records parser-accepted
multi-line Box examples while leaving `;` as a post-Phase-4 member
separator open question, and updates the ScrollView widget-registry
row out of design-draft status. `docs/architecture.md` top-level
Status flips to include M3-Phase 4 complete, §6.3 records the
window-root sizing runtime-boundary invariant, and §6.5 records the
ScrollView intermediate-Visual `parent_abs_offset` shift rule. Phase 4
also registers out-of-phase residual R1: component-level `.ui title:`
must eventually drive the native Window title; owning M3 phase is to
be assigned during M3-Phase 5 pre-doc framing, with implementation due
no later than M3-Phase 8 Gallery E2E close.

Decisions: [DD-M3-P4-001..006](./process/milestone-3/phase-4/decisions/preamble.md).

### M3-Phase 3 — WrapPanel layout primitive (2026-05-22)

Adds the `WrapPanel` layout primitive, the WrapPanel constituent of
M3 acceptance **A3** (gallery overflow / wrapping evidence).
`WrapPanel` admits zero or more children, supports constant-only
`item-cross-size: <i32>` / `item-spacing: <i32>` / `line-spacing:
<i32>` attributes, runs the first M3 two-stage measure-arrange whose
outer cross-axis size depends on its children, lets oversized
first-children flow into visible overflow (the WrapPanel installs no
clip surface; parents clip), and pairs structurally with the
forthcoming Phase 4 ScrollView (ScrollView bounds the main axis,
WrapPanel resolves the cross axis).

`wasamo-ir` is unchanged at the type / literal level — `WrapPanel`
reuses `IrLiteral::Int`, the generic `IrNode` widget shape, and
`IrProp` keyed by attribute name. No new `IrType`, `IrLiteral`,
`PropertyValue`, `LayoutError`, `WASAMO_VALUE_*`, or
`WASAMO_LAYOUT_ERROR_*` variant lands. `wasamoc` lexes (kebab-case
`Token::Ident` + `IntLit` admitting optional leading `-`; both lexer
generalisations needed for `item-cross-size: -1` to reach the
checker), parses (unchanged grammar), checks (accepts 0+ children;
rejects negative literals, non-`IntLit` shapes, `bind` on the three
attributes, and the three attributes outside `WrapPanel`; warns when
a direct-child aspect-only `Box` is missing `item-cross-size`), and
lowers / emits the three attributes through the existing generic
`IrProp` shapes. `wasamo-runtime` adds a `WidgetData::WrapPanel`
widget kind with widget-catalog defaults (item-cross-size = None /
passthrough, item-spacing = 0, line-spacing = 0), IR-loader
materialisation, defense-in-depth `validate()` rejecting negative
attribute values (`WASAMO_ERR_IR_MALFORMED`), and the pure-data
line-breaker + arrange in `wasamo-runtime/src/layout.rs` (Win32 /
WinRT-free per established layout-engine discipline).

The `sync_visuals` boundary gained a parent-relative offset
conversion for nested non-zero-offset visual trees: WrapPanel is the
first M3 primitive whose children sit at non-zero offsets inside
their parent, exposing an implicit absolute (`LayoutNode`) vs.
parent-relative (`Visual.Offset`) gap that Phase 2 never triggered.

Visible proof: `examples/gallery/gallery.ui` grows additively from
the Phase 2 single-Box sub-screen to a `WrapPanel` of ten uniform
88×88 placeholder thumbnails with the ADR-canonical 88 / 12 / 12
attribute values (7+3 wrap on the default 800×600 window).
`examples/gallery-rust/` builds and launches it through the same
`.ui -> wasamoc -> IR text -> wasamo_load_ui` path; owner-manual
GUI smoke confirmed the wrap and the post-fix non-zero-offset
rendering.

Per-phase spec sync ([A11](./process/_roadmap.md#m3-dsl-surface)):
`docs/dsl_spec.md` 0.9 -> 1.0 flips §4.10 to
`M3-Phase 3 closed; implementation-synced` and folds the T1
lexer-surface change into §2.2 (`Ident` admits kebab-case
continuations; `IntLit` admits an optional leading `-` with a
note that the negative-sign surface is `IntLit`-only).
`docs/architecture.md` top-level Status flips to
`M3-Phase 1, M3-Phase 2, and M3-Phase 3 complete`, and §6.5
(WidgetNode and Visual Layer sync) gains a one-line clarification
of the absolute vs. parent-relative offset convention discovered
via T9 visible-smoke.

Decisions: [DD-M3-P3-001..006](./process/milestone-3/phase-3/decisions/preamble.md).

### M3-Phase 2 — Box layout primitive (2026-05-20)

Adds the `Box` layout primitive, discharging M3 acceptance **A6**.
`Box` admits zero or one child, supports constant-only
`aspect: <ratio>` and `fill: <color>` attributes, centres and clips
its single child, and provides the Box + Text placeholder pattern that
carries the M3 Image-widget deferral.

`wasamo-ir` gains `IrLiteral::Ratio { num, den }` and
`IrLiteral::Color(u32)` without adding `IrType::Ratio`,
`IrType::Color`, `PropertyValue` variants, public ABI tags, or new
handler expression variants. `wasamoc` lexes / parses / checks /
lowers / emits ratio and color literals, rejects non-positive ratios,
rejects `bind aspect:` / `bind fill:`, and rejects 2+ Box children.
`wasamo-runtime` adds a Box widget kind, Box-internal `Ratio` /
`Color` domain types, IR-loader materialisation for Box `aspect` /
`fill`, defense-in-depth IR validation, and pure-layout
measure-arrange support for bounded inscribed fit, one-axis-unbounded
bounded-axis-wins, no-aspect shrink-to-fit, centred child placement,
and layout-time errors for no-extent cases.

Visible proof: `examples/gallery/gallery.ui` now contains the Phase 2
Box sub-screen, and `examples/gallery-rust/` builds and launches it
through the same `.ui -> wasamoc -> IR text -> wasamo_load_ui` path as
the M2/M3 Rust hosts. Owner-manual GUI smoke confirmed the blue
16:9 Box fill and centred placeholder text.

Per-phase spec sync ([A11](./process/_roadmap.md#m3-dsl-surface)):
`docs/dsl_spec.md` 0.7 -> 0.8 flips §4.9 to
`M3-Phase 2 closed; implementation-synced`; no implementation/spec
divergence was found during the close re-sync.

Decisions: [DD-M3-P2-001..006](./process/milestone-3/phase-2/decisions/preamble.md).

### M3-Phase 1 — `bool` scalar binding (2026-05-19)

Adds `bool` as the third scalar binding type alongside `i32` and
`String`, discharging M3 acceptance **A9**. The reactive path stays
type-agnostic — `bool` threads through the same `wasamo-ir` ↔
`wasamoc` ↔ `wasamo-runtime` pipeline that `i32` and `String`
already travel, with `Button.enabled` as the live `WidgetNode`
attribute that proves end-to-end propagation. The `TypedValue`
generic value union remains deferred (F5).

`wasamo-ir` gains `IrType::Bool`, `IrLiteral::Bool`, and
`HandlerExpr::{BoolLit, BoolPropRead}` variants. `wasamoc` reserves
`true` / `false` as keywords, type-checks `state` defaults and
property-bind RHS against a soft widget-property catalog
(`Text.text` / `Button.text` / `Button.enabled`), and lowers
state-name idents to typed `*PropRead` variants based on the
declared state type. `wasamo-runtime` adds `PropertyValue::Bool`,
widens `resolve_prop_key` to return `(PropertyKey, IrType)`,
introduces the per-type binding writer seam
(`evaluate_bool_binding` + `widget_write_property_bool` +
`register_bool_binding`), and extends `EvalContext` with
`get_bool` / `read_bool_tracked` / `set_bool`. The C ABI gains
`PropertyValue::Bool` ↔ `WasamoValue::v_bool` conversion arms
across `read_property_value` / `write_property_value` /
`property_value_to_owned` / `owned_to_value` (no new public ABI
functions; the existing `WASAMO_VALUE_BOOL = 3` tag was M2-reserved).

The Phase 1 `bool` surface itself is also intentionally narrow at the
language level: `bool` is admitted for bool-typed property bindings
(e.g. `Button.enabled`) and inline handler assignments to bool state,
but rejected in string interpolation (e.g. `Text.text: "ready={ready}"`
is a compile error). Display conversion is a deliberate Phase 1
non-goal and remains a later expression / formatting concern.

The Phase 1 `Button.enabled` runtime contract is narrow: when
`false`, `hit_test_click_inner` suppresses host callback / inline
`clicked` handler / `enqueue_signal("clicked", …)` while preserving
child hit-test traversal; `update_hover_inner` freezes hover/press
transitions; `effective_button_color` paints a flat grey
`(A=0x40, R=G=B=0x80)` directly (no `ColorKeyFrameAnimation`); the
layout slot is preserved. Focus / a11y / keyboard activation are
deferred to M4–M5.

Visible proof: a new `examples/bool-demo/bool-demo.ui` fixture
(`state ready: bool = true`, `Button.enabled: ready`,
`clicked => { root.ready = false; }`) drives a new
`examples/bool-demo-rust/` host through the build-time `wasamoc`
pipeline (same shape as `counter-rust`), so the M2 Hello Counter
reference stays unmodified. CI coverage: pure-logic unit tests in
`wasamo-ir` / `wasamoc` / `wasamo-runtime`, an IR round-trip
integration test (`wasamoc::emit` → `wasamo-runtime::parse_ir`),
and two Windows-only mock-free integration tests covering both the
end-to-end `.ui → IR → load → click → state → bound widget
property` chain through the binding pipeline and the direct
widget-setter path via `wasamo_set_property(PROP_BUTTON_ENABLED, …)`
on a live `WidgetNode` (asserting both the `CompositionColorBrush`
colour flip and click-callback suppression).

Per-phase spec sync ([A11](./process/_roadmap.md#m3-dsl-surface)):
`docs/dsl_spec.md` 0.4 → 0.5 (§§2.1 / 2.2 / 3 / 4.3 / 4.6 / 4.7 /
4.8 / 5 / 8.2 / 8.4 / 8.6 / 8.9 / 8.12; also folds in a minimal
retroactive `state` surface entry for the M2-Phase 6 documentation
gap, owner-agreed during the Phase 1 spec sync); `docs/architecture.md` §6.8.7
documents the bool path through `register_bool_binding`,
`SignalRegistry::bools`, `widget_write_property_bool`, and the
DD-M3-P1-007 per-type seam.

Decisions: [DD-M3-P1-001..010](./process/milestone-3/phase-1/decisions/preamble.md).

## [v0.2.0] — 2026-05-11 — M2: Foundation

M2 closes the loop on the DSL side: `.ui` files now drive the runtime
through the agreed `wasamoc` -> IR -> `wasamo_load_ui` pipeline, with
reactive state propagation rather than host-side property-set plumbing.
The milestone discharges acceptance criteria A1-A6: C/Rust/Zig counter
hosts load `counter.ui`, tree-mutation ABI primitives and the cdylib shim
cleanup are in place, and the reactive foundation now has release-mode
ordering, guard-placement, and non-`i32` binding evidence.

### M2-Phase 7 — Reactive Foundation Hardening & Contract Finalization (2026-05-11)

Completes the three deferred Phase 6 DDs that distinguish "the DSL
pipeline runs" from "the DSL pipeline is a Foundation other layers can
rely on." M2 acceptance **A5** and **A6** are discharged.

`dirty_effects` now drains through a dependency-graph topological walk
instead of `EffectId` numeric ordering. The implementation adds explicit
write-edge tracking to `ReactiveGraph`, removes the `sort_unstable()`
production path, and covers chain, diamond, fan-out, and out-of-ID-order
dependency shapes with pure-logic tests. M3 residuals for cycle policy,
ordering ties, and `MUTATION_CAP` interaction are recorded in
`docs/notes/m2-to-m3-handover.md`.

Runtime guard placement is now an accepted architectural invariant:
the ABI boundary owns caller-facing diagnostics, while internal runtime
boundaries protect invariants for entry paths that do not cross the ABI.
`wasamo_run` and `wasamo_quit` now record divergence diagnostics and
return as no-ops after divergence, and focused tests cover
`drain_if_outermost` suppression, re-entrant drain behavior, and cleanup
exceptions.

String-typed property binding is implemented with
`HandlerExpr::StrPropRead`, `EvalContext` String reads, wasamoc lowering
based on declared state type, runtime IR parsing, tracked
`Signal<String>` reads, and regression coverage for the existing integer
binding path. A Windows-only headless integration test proves `.ui`
String binding reaches live `WidgetNode` property state; GitHub Actions
green is evidence that the live path ran rather than skipped.

Decisions: [DD-M2-P6-010..012](./process/milestone-2/phase-7/decisions/preamble.md).

### M2-Phase 6 — `.ui` → runtime lowering (2026-05-08)

Closes the loop between the DSL and the runtime: `examples/counter/counter.ui`
now drives the running Hello Counter in C, Rust, and Zig through the agreed
`wasamoc` → IR → `wasamo_load_ui` pipeline, with reactive state propagation
landing through the M2 reactive path (no host-side `wasamo_set_property`).
M2 acceptance **A1** and **A2** discharged.

`wasamoc` extends from M1's parse+check shape with `state` declarations,
restricted type inference (`i32` + string), property-binding lowering,
handler-body lowering, compile-time name resolution, and IR file emission
following the new `;wasamo-ir v0` normative grammar (`docs/dsl_spec.md` §8).
A new `wasamo-ir` crate hosts the shared `IrComponent` / `HandlerExpr`
types so the compiler emits and the runtime consumes the same in-memory
shape. `wasamo-runtime` gains `ir_loader.rs` (parse + defense-in-depth
validation per DD-M2-P6-009 — header/version, reference resolution,
top-level structure; trusts emitter type integrity), the
`wasamo_load_ui(WasamoLoadType, const void*, size_t, WasamoWindow**)`
C ABI entry point with `WASAMO_LOAD_PATH` / `WASAMO_LOAD_MEMORY` modes,
and `wasamo_last_error_message` as the thread-local last-error channel.
Five new error codes ship: `WASAMO_ERR_OBSERVER_MUTATION`,
`WASAMO_ERR_REACTIVE_DIVERGED`, `WASAMO_ERR_REENTRANT_LOAD`,
`WASAMO_ERR_WRONG_THREAD`, `WASAMO_ERR_IR_MALFORMED`.

The reactive drain transaction is rewritten to the three-phase + terminal
form (DD-M2-P6-001 = D, supersedes DD-M2-P5-004's three-stage framing):
Phase 1 mutation convergence loop (FIFO `signal_queue` + topological
`dirty_effects` + last-wins; structure-changing ABI returns
`WASAMO_ERR_REENTRANT_LOAD`), Phase 2 terminal layout pass, Phase 3
post-commit observer drain (state-mutating ABI returns
`WASAMO_ERR_OBSERVER_MUTATION`). `MUTATION_CAP` exhaustion drives an
irreversible `Healthy → Diverged` transition; subsequent ABI calls
return `WASAMO_ERR_REACTIVE_DIVERGED` except `wasamo_runtime_destroy`.
`SignalRegistry { i32s, strings }` (DD-M2-P6-007) replaces Phase 5's
provisional single-type `properties` map; the registration API
(`register_binding(target, expr)`) is preserved.

Documentation: `VISION.md §4 Principle 2` supplement (observer = post-commit
pure effect; mutation = events-up + bindings-down); `docs/architecture.md`
§6.8.3 documents the three-phase + terminal drain; `docs/abi_spec.md`
§3.1/§5.2/§6 cover the new status codes, `wasamo_load_ui`, and thread
affinity rules; `CLAUDE.md` records the build-ordering requirement
(`cargo build -p wasamoc` precedes C/Zig host builds).

The three deferred DDs — DD-M2-P6-010 (dirty_effects topo sort fidelity),
DD-M2-P6-011 (String-typed property binding), DD-M2-P6-012 (re-entrancy /
safety-guard placement principle) — remain `Proposed` and are scoped to
M2-Phase 7 alongside acceptance criteria A5 / A6 (added by the
2026-05-08 plan revision).

Decisions: [DD-M2-P6-001..009](./process/milestone-2/phase-6/decisions/preamble.md);
DD-M2-P6-010..012 deferred to M2-Phase 7.

### M2-Phase 5 — Reactive engine (2026-05-06)

Implements the M2 thesis-validation surface for acceptance A2:
host `count.set(...)` updates a bound `Text` label without any
host-side `wasamo_set_property` call. Pure-internal Rust; no C ABI
symbol added (per DD-M2-P4-004 = A). `wasamo-runtime/src/reactive.rs`
gains `Signal<T>` + `EffectHandle` + thread-local effect stack +
forward/back dependency edges + dirty-set drain + iteration-cap
divergence trap; `with_batched_writes` body fills the Phase 4
skeleton. The handler evaluator (`HandlerExpr` + `EvalContext`)
is reused via a read-only `BindingEvalContext` that records
Signal reads as dependency edges. `WidgetNode` gains a
`bindings: Vec<EffectHandle>` field disposed at the head of the
Phase 4 `widget_destroy` sweep. `register_binding(target, expr,
write_fn, properties)` is the Phase 6-facing internal API.
`drain_if_outermost` now runs observer drain → reactive drain →
layout drain in one outermost-frame cycle, composing with
DD-P6-003 queued emission and DD-P8-002 layout invalidation. GUI
checkpoint (`exp/m2-p5-reactive-checkpoint`, commit `fdc1545`)
confirms click → label update through the reactive path on real
hardware. A2 fully discharged at Phase 6 close (counter.ui-driven).

Decisions: [DD-M2-P5-001..006](./process/milestone-2/phase-5/decisions/preamble.md).

### M2-Phase 4 — Tree-mutation ABI primitives (2026-05-05)

Grows the stable C ABI with a sixth area (DD-P6-001 defined the
initial five): index-based widget-tree mutation. New stable-core
symbols: `wasamo_widget_append_child` (promoted from internal),
`wasamo_widget_insert_child`, `wasamo_widget_remove_child`,
`wasamo_widget_replace_child`, `wasamo_widget_child_count`,
`wasamo_widget_destroy`. `WidgetNode` gains an `attached: bool`
invariant maintained by all mutators; `wasamo_widget_destroy` rejects
attached widgets. No host-visible batching API added (DD-M2-P4-004 =
Option A; existing queue-and-drain is the M2 batching contract, now
documented in `abi_spec.md §6`). `reactive.rs` skeleton provides
`with_batched_writes` (internal-only; Phase 5 fills the body).
Acceptance criterion A4 of M2 discharged.

Decisions: [DD-M2-P4-001..004](./process/milestone-2/phase-4/decisions/preamble.md).

### M2-Phase 1 — cdylib-shim cleanup (2026-05-03)

Resolved the rlib filename collision (cargo#6313) that was worked
around in M1 by dropping `wasamo-runtime`'s rlib. `wasamo-runtime`
is now rlib-only (`[lib].name = "wasamo_runtime"`); a new
`wasamo-dll` cdylib shim depends on it and re-exports all C ABI
symbols via MSVC `/WHOLEARCHIVE`. `wasamo.dll` filename and all 20
`wasamo_*` ABI symbols are preserved. Acceptance criterion A3 of M2
discharged.

Decisions: [DD-M2-P1-001..006](./process/milestone-2/phase-1/decisions/preamble.md).

Release: [v0.2.0](https://github.com/matarillo/wasamo/releases/tag/v0.2.0).

---

## [v0.1.0] — 2026-05-01 — M1: Proof of Concept

Validated the core hypothesis: external DSL × C ABI × Visual
Layer. VStack / HStack / Text / Button / Rectangle render through
the Visual Layer with DWM compositor independence verified, the
minimal C ABI (`wasamo.h`) is shaped as a stable core plus an M1
experimental layer, and Hello Counter runs end-to-end in C, Rust,
and Zig (host-imperative; the `.ui → runtime` lowering is M2).

Decisions: Phase 0–8 ADRs under
[process/](./process/) (`DD-P2-*` … `DD-P8-*`,
`DD-V-001` … `DD-V-004`).
Release: [v0.1.0](https://github.com/matarillo/wasamo/releases/tag/v0.1.0).

## Document system

This project's document conventions changed on 2026-05-02 alongside
M1 shipping. Acceptance criteria live in
[process/_roadmap.md](./process/_roadmap.md), thesis-level framing in
[VISION.md §7](./VISION.md#7-roadmap), shipped milestones here, and
in-flight work in the active process tree under
[process/](./process/). Rationale:
[DD-V-010..016](./process/cross-milestone/decisions/doc-system.md).
