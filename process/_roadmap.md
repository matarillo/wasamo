# Wasamo Roadmap

Milestones are defined by **acceptance criteria**, not dates. This
file is the SSOT for those criteria
([DD-V-010](./cross-milestone/decisions/doc-system.md#dd-v-010--acceptance-criteria-ssot)).
For thesis-level framing see [VISION.md §7](../VISION.md#7-roadmap).
For shipped milestones see [CHANGELOG.md](../CHANGELOG.md). For the
current state of work see the **Status** section of
[README.md](./README.md).

Phase structure for the active milestone lives in its plan
(`process/milestone-N/plan.md`), not here
([DD-V-016](./cross-milestone/decisions/doc-system.md#dd-v-016--plan--roadmap-commit-flow-redefinition)).
Design decisions are recorded as ADRs under each phase's
`decisions/` directory; the document structure and pre-implementation
discipline are described in [process/README.md](./README.md).

---

<a id="m1-proof-of-concept"></a>

## M1: Proof of Concept ✅ shipped 2026-05-01 (v0.1.0)

See [CHANGELOG entry](../CHANGELOG.md#v010--2026-05-01--m1-proof-of-concept)
and the [ADRs](./milestone-1/).

<a id="m2-foundation"></a>

## M2: Foundation ✅ shipped 2026-05-11

**Goal:** close the loop on the DSL side — make `.ui` files actually
drive the runtime, with reactive state propagation, so Hello Counter
in each language is written against the DSL rather than reproduced
by hand through the experimental C ABI.

**Acceptance criteria**

- `examples/counter/counter.ui` drives the running Hello Counter
  in C, Rust, and Zig — the M1 host-imperative trees are replaced
  by hosts that load the DSL through the agreed wasamoc pipeline
- Reactive state propagation works without host-side property-set
  plumbing: `count++` in the host updates the visible label
  through the M2 reactive path, not a manual `wasamo_set_property`
- The cdylib / rlib filename collision flagged in
  [DD-P7-002](./milestone-1/phase-7/decisions/preamble.md) is
  discharged
- The C ABI gains the tree-mutation primitives required by the
  reactive engine; the experimental layer's all-at-once
  constructors remain available but are no longer the only way
  to construct UI
- Reactive Foundation Hardening: the reactive engine's
  execution-order guarantee (topological drain of dirty Effects)
  and the runtime's re-entrancy/guard placement principle are
  settled at design level (Accepted ADRs) and reflected in
  implementation; the guard placement principle is recorded in
  `docs/architecture.md` as a global runtime invariant
- Type-Agnostic Reactive Binding: the reactive binding path is
  demonstrated end-to-end with a non-`i32` property type
  (`String`), proving the `EvalContext` / `HandlerExpr` / IR
  design is not silently `i32`-specialized

See [CHANGELOG entry](../CHANGELOG.md#m2-foundation--shipped-2026-05-11)
and [process/milestone-2/plan.md](./milestone-2/plan.md) for the completed
phase breakdown.

<a id="m3-dsl-surface"></a>

## M3: DSL surface ✅ shipped 2026-07-06

**Thesis:** the DSL is expressive enough to write real layouts, and
is published as a stable public draft.

**Acceptance criteria**

- **A1.** `examples/gallery/gallery.ui` (Photo Gallery target app)
  drives the M3 surface end-to-end on the M2 reactive foundation,
  exercising every M3 layout primitive, grammar construct, scalar
  extension, and widget surface enumerated below through the
  `.ui -> IR -> runtime` path. The Gallery composes Grid (overall
  frame), WrapPanel + ScrollView (thumbnail grid), ZStack
  (lightbox overlay), and Box (aspect-constrained placeholders);
  `bool` bindings drive conditional rendering (lightbox open /
  close) and the `ToggleButton` `checked` selected state (tab-like
  sections); an iteration grammar generates thumbnails from a
  collection binding
- **A2.** Grid layout primitive (1 cell 1 child, star sizing +
  spanning; same-cell overlap is not provided — overlay is
  ZStack's responsibility), demonstrating that DSL can express 2D
  measure-arrange including star sizing's cross-axis dependency
  resolution, as an axis of layout proof independent of
  WrapPanel's main / cross-axis reflow
- **A3.** WrapPanel layout primitive, demonstrating that DSL can
  express a two-stage measure-arrange — linear main-axis placement
  plus cross-axis wrap on main-axis overflow — that goes beyond
  the linear arrangement (HStack / VStack) established in M2
- **A4.** ZStack layout primitive (sibling z-order by document
  order), demonstrating that DSL can express the layout semantics
  of overlapping siblings. Through M2 the widget tree was linear;
  this primitive is a genuine extension of the M2 surface
- **A5.** ScrollView primitive (minimal: inner unbounded measure +
  viewport clip + content offset binding; scrollbar widget, wheel
  handler, and drag are deferred to M4), demonstrating that the
  viewport concept and content offset binding traverse DSL / IR /
  runtime — a structural prerequisite for the thumbnail-scale
  scenarios in the target app
- **A6.** Box layout primitive (0+ child container; `aspect:
  <ratio>` attribute subsumes AspectRatio; minimal `fill: <color>`
  attribute for scrim use), proving that aspect can be folded as
  an attribute rather than a standalone primitive, and that the
  M3 deferral of an Image widget surface can be carried by a
  placeholder form (Box + Text child) without opening the
  asset / decoder surface
- **A7.** Conditional rendering grammar (binding-driven widget
  present / absent), demonstrating that binding can drive the
  structure of the widget tree — the present / absent status of
  a subtree. Through M2, binding drove property values only;
  this construct extends what bindings reach into tree shape
- **A8.** Iteration grammar (collection-driven widget tree
  generation), demonstrating that binding can drive the
  cardinality of the widget tree. The foundation for the
  dynamic-UI surfaces (filter, sort, virtualization) that come
  after M3
- **A9.** `bool` added as the third scalar binding type alongside
  `i32` and `String`, demonstrating that subtree presence
  (conditional rendering) and widget attribute state
  (`ToggleButton` `checked`) can both be driven by a scalar extension
  without introducing the `TypedValue` generic value union, which
  remains deferred
- **A10.** Selected / toggle state surface admitted, settled under
  M3 as a dedicated `ToggleButton` widget carrying a controlled
  one-way `checked: <bool>` attribute (ordinary `Button` keeps its
  momentary action-only meaning), demonstrating that a `bool` scalar
  binding can drive a widget attribute
- **A11.** DSL spec, implementation, and E2E proof are
  synchronized per phase: each M3 phase updates `docs/dsl_spec.md`
  for the surface it ships and exercises that surface in
  `examples/gallery/` within the same phase. Spec drafting is a
  per-phase deliverable, not an end-of-milestone byproduct
- **A12.** DSL specification first public draft (covers M2 surface
  plus the above M3 primitives, grammar surface, scalar type, the
  `ToggleButton` selected / toggle state surface, and the
  parent-interpreted placement authoring surface of A13). The novel
  normative content is the
  measure-arrange spec for WrapPanel and Grid, the grammar
  surface (conditional rendering, iteration), and the
  parent-interpreted placement authoring surface
- **A13.** Parent-interpreted placement authoring surface: Grid cell
  placement (`row` / `column` / `row-span` / `column-span` / `h-align` /
  `v-align`) and ZStack alignment (`h-align` / `v-align`) are authored as
  parent-interpreted `slot.*` metadata — not intrinsic widget properties
  — unified across containers on one `slot.` namespace, with Grid
  additionally retaining the `Cell` grouping form (Grid accepts both
  `Cell` and direct `slot.*`, one form per child; ZStack accepts
  `slot.*`). Demonstrates that the DSL distinguishes "data the parent
  interprets about a child" from a widget's own properties

See the [CHANGELOG M3 entry](../CHANGELOG.md) and
[process/milestone-3/plan.md](./milestone-3/plan.md) for the completed
phase breakdown and the A1–A13 discharge mapping (§Milestone close);
the M3 → M4 carry-forward is
[process/milestone-3/handoff.md](./milestone-3/handoff.md).

## M4: Interaction stack

**Thesis:** input, multi-window, text input, and accessibility share
a focus model; they ship together so the focus model is settled
once. Wasamo's identity feature (Mica/Acrylic) becomes demonstrable
from this milestone, and the first contributor-facing showcase ships
here.

**Acceptance criteria**

- Input handling: keyboard, mouse, touch; focus model and event
  routing
- Multi-window support (per-window state, cross-window focus).
  Included pre-1.0 because its ABI implications are cross-cutting
  and an append-only post-freeze surface cannot accommodate them
- TextField widget (minimum editable text widget; required by IME
  verification)
- IME via TSF (Japanese / CJK input)
- AccessKit / UIA integration
- Mica / Acrylic root-window backdrop; system accent color
  follow-through (initial — full theming surface is M5)
- Per-monitor DPI awareness: declare process / window DPI awareness,
  render crisply on high-DPI displays without DWM bitmap scaling, and
  handle DPI changes across monitors (per
  [DD-V-022](cross-milestone/decisions/dpi-awareness-m4-deferral.md);
  the runtime is DPI-unaware as of M3 — a precondition for the
  Mica/Acrylic identity showcase above)
- First showcase application — sufficient to demonstrate Wasamo
  identity for contributor outreach, even if rough around
  polish-level details
- Author-controllable sizing (Problem B) design spike — preferred
  in this milestone. M4 planning records the spike disposition
  (default M4; deferral to M5 positively justified) per the
  [author-controllable sizing VDR](./cross-milestone/decisions/author-controllable-sizing-surface.md)

## M5: Identity & tooling

**Thesis:** Wasamo looks like Wasamo by default, and authoring `.ui`
is a first-class editor experience.

**Acceptance criteria**

- Full theming surface (light / dark, accent propagation through
  widgets, type ramp coverage)
- Official widget set (CheckBox, ComboBox, Menu, and the rest
  beyond TextField)
- VS Code extension (LSP, syntax highlighting, diagnostics). The
  VS Code work may begin in parallel any time after M3's DSL spec
  public draft is agreed; M5 is its acceptance gate, not its
  earliest start
- Author-controllable sizing (Problem B) implementation if the
  M4/M5 spike concludes it is warranted; otherwise it falls back to
  the M6 disposition below, per the
  [author-controllable sizing VDR](./cross-milestone/decisions/author-controllable-sizing-surface.md)

<a id="pre-10-candidate-pool"></a>

## Pre-1.0 candidate pool

Triaged items the owner wants before 1.0 that are not (yet) assigned
to a milestone. Entries carry **no acceptance criteria** and are not
commitments; they exist so milestone planning and the M6 freeze
decision see them. Governing rules (entry criterion, tags, lifecycle,
per-planning disposition duty) are in
[DD-V-028](./cross-milestone/decisions/pre-1.0-candidate-pool.md).
Undifferentiated wishes are **not** listed here — they stay in
[docs/notes/owner-intake.md](../docs/notes/owner-intake.md) until
triaged.

Items — preliminary seed from a full `docs/notes/` sweep (2026-07-07,
agent-triaged; to be confirmed at M4 planning §1.1). One table row per
item; entries are capability desires or freeze-relevant dispositions,
not open design questions (those stay in their notes):

| Item | ABI-bearing | Leans | Origin |
|---|---|---|---|
| Literal `fill` on layout containers beyond `Box` | no | small additive | [owner-intake](../docs/notes/owner-intake.md) |
| Background color on themed widgets (`Button`, `ToggleButton`, `Text`) | unknown | M5 theming | [owner-intake](../docs/notes/owner-intake.md) |
| Reactive `fill` (color as a bound value) | unknown | `TypedValue` design space (row below) | [owner-intake](../docs/notes/owner-intake.md) |
| Host state boundary: host-supplied initial state, host write, in-out write-back | **yes** | M4 (TextField write-back); M6 backstop | [host-state-boundary](../docs/notes/host-state-boundary.md) |
| Expression predicates + scalar calc (M-expr1/2a: `if count > 0`, `labels[i]`) | no | early M4 | [expression-language-roadmap](../docs/notes/expression-language-roadmap.md) |
| `TypedValue` + structured data (M-expr2b/3: computed color / dimension, `item.filename`) | unknown — `abi_spec` check pending | dedicated pre-1.0 slot (M4–M5) | [expression-language-roadmap](../docs/notes/expression-language-roadmap.md), [typed-value-evaluator](../docs/notes/typed-value-evaluator.md) |
| Top-layer overlays (popover / dropdown / tooltip / menu) | unknown | M4 input-focus adjacency | [top-layer-overlays](../docs/notes/top-layer-overlays.md) |
| Window config props: dynamic title, initial window size, `WindowConfig` | unknown | M4 multi-window | [dsl-grammar Q2](../docs/notes/dsl-grammar.md) |
| `Image` widget (asset / decoder surface) | unknown | M5 widget set | [layout-engine §3.3](../docs/notes/layout-engine.md) |
| Developer debug support (first step: `wasamoc` lint for silent 0-collapse) | no (lint); unknown (runtime diag channel) | M5 tooling | [developer-debugging](../docs/notes/developer-debugging.md) |
| Release distribution (artifacts, channels, discovery, versioning) | no | M4 showcase outreach; M6 1.0 | [release-distribution](../docs/notes/release-distribution.md) |
| Component extension model — freeze check only (registration ABI append-only-safe?) | unknown | post-1.0 + M6 disposition | [component-extension-model](../docs/notes/component-extension-model.md) |

Disposition log (one dated line per milestone planning, per
DD-V-028):

- _none yet — first pass due at M4 planning (§1.1)._

## M6: 1.0 — C ABI stabilization

**Thesis:** the ABI is settled, performance targets are met, a
polished showcase ships, and SemVer applies.

**Acceptance criteria**

- C ABI freeze; SemVer applies from this point
- Public backward-compatibility commitment
- Author-controllable sizing (Problem B) disposition before ABI
  freeze — implement if ABI-bearing, or record why a post-freeze
  append-only addition is safe. Backstop if the M4/M5 schedule
  slips, per the
  [author-controllable sizing VDR](./cross-milestone/decisions/author-controllable-sizing-surface.md)
- Performance targets: <100 ms cold start, <30 MB memory,
  single-digit-MB binaries
- Polished showcase application (production-grade, distinct from
  M4's contributor-outreach showcase)
- C / Rust / Zig bindings mature. Swift and Go bindings are out of
  scope for 1.0; they are welcomed as community-prototyped
  bindings post-1.0 (see [VISION §11](../VISION.md#11-how-to-contribute))

## Post-1.0

- Hot reload (interpreter mode during development) — feasibility
  depends on the wasamoc output format chosen in M2-Phase 2
- Higher-level animation DSL (the public property-change animation
  API deferred from Phase 5; see
  [DD-V-001](./cross-milestone/decisions/m1-acceptance-criteria.md))
- Advanced layout (LazyList, CollectionView)
- System tray and notification integration
- MSIX packaging integration
- Swift / Go bindings (community-maintained)
