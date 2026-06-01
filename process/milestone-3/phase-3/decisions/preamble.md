# M3-Phase 3 — WrapPanel layout primitive: Architecture Decisions

**Phase:** M3-Phase 3 (WrapPanel layout primitive)
**Date:** 2026-05-21
**Status:** Accepted

## Context

M3 acceptance criterion **A3** (see
[process/_roadmap.md M3](../../../_roadmap.md#m3-dsl-surface),
[m3-plan.md §Acceptance criteria](../plans/m3-plan.md#acceptance-criteria)):

> WrapPanel layout primitive, demonstrating that DSL can express a
> two-stage measure-arrange — linear main-axis placement plus
> cross-axis wrap on main-axis overflow — that goes beyond the linear
> arrangement (HStack / VStack) established in M2.

The pre-doc framing for this phase was aligned with the owner on
2026-05-21 and is recorded in
[docs/notes/m3-phase-3/pre-doc-framing.md](../notes/m3-phase-3/pre-doc-framing.md).
That framing fixed the 6-DD slate carried below, the visible-proof
location (framing decision E — grow Phase 2's `examples/gallery/` +
`examples/gallery-rust/` sub-screen into a WrapPanel of Boxes), the
verification-strategy menu picks (framing decision C), and the
two upstream-document-revision moments inherited verbatim from
Phase 2 (framing decision D).

Per the M2-Phase 2 framing decision D postmortem
([m3-phase-2 framing notes](../notes/m3-phase-2/m3-phase-2-pre-doc-framing.md)),
the "Moment is not a commit unit" rule applies from the start of
Phase 3: each upstream-document edit in a Moment lands as its own
commit on the pre-doc branch, scoped by review concern, not bundled.

The M2/M3-Phase-1/M3-Phase-2 end-state shape that this phase extends
without breaking:

- `wasamo-ir` ([wasamo-ir/src/lib.rs](../../wasamo-ir/src/lib.rs)):
  `IrType` is `I32 | Str | Bool`; `IrLiteral` is `Int | Str | Ident |
  Bool | Ratio | Color`. The `Ratio` / `Color` literals were added in
  Phase 2 (DD-M3-P2-002 / DD-M3-P2-003) as Box-internal domain types
  with no `PropertyValue` widening. Phase 3 reuses existing `i32`
  plumbing for all WrapPanel attributes; no new literal form is
  introduced.
- `wasamo-runtime` widget catalog
  ([wasamo-runtime/src/widget.rs](../../wasamo-runtime/src/widget.rs)):
  `Rectangle | VStack | HStack | Text | Button | Box`. `PropertyValue`
  enum is `I32(i32) | String(String) | Bool(bool)`; no new variant
  added in Phase 2. Phase 3 adds `WrapPanel` as a per-kind tag (DD-001).
- Layout engine
  ([wasamo-runtime/src/layout.rs](../../wasamo-runtime/src/layout.rs)):
  pure-data `LayoutNode` / `measure` / `arrange` boundary, Win32/WinRT-
  free. Phase 2 introduced `LayoutError::{BoxAspectUnboundedBoth,
  BoxNoExtent}`; Phase 3 inherits the error class and may extend it
  conditionally (DD-005).
- Binding pipeline: per-type writer seam pattern (DD-M3-P1-007). All
  Phase 3 WrapPanel attributes are constant-only (per DD-002 / DD-003
  / DD-004 below), so no new seam triple is built. F5 (`TypedValue`
  deferral) is held in force by construction.
- `wasamoc` ([wasamoc/src/check.rs](../../wasamoc/src/check.rs)):
  state-name → declared-type table; identifier resolution lowers to
  typed `*PropRead` variants. Phase 3 adds no new value type; the
  WrapPanel attributes are integer literals and `wasamoc check` rejects
  negative literals via the existing diagnostic surface.

This ADR is framed against A3 and the m3-plan's "first M3 phase to
introduce novel normative measure-arrange spec in `docs/dsl_spec.md`"
designation
([m3-plan.md §Phase breakdown](../plans/m3-plan.md#phase-breakdown)).
It does **not** re-open F5 (`TypedValue` deferral) — every attribute
the WrapPanel exposes ships constant-only; bindable surface is
deferred per attribute to the phase that first needs it. Image-widget
deferral remains in force, carried by Phase 2's DD-M3-P2-006 placeholder
pattern; Phase 3 simply lays out instances of the pattern in a wrap.

The acceptance lens for this phase: A3 is satisfied when (i) `.ui`
declares `WrapPanel { item-cross-size: <i32>; item-spacing: <i32>;
line-spacing: <i32>; <Box-children> }` and the shared crates lower →
load → render it with correct two-stage measure-arrange, (ii) the
WrapPanel chapter lands in `docs/dsl_spec.md` §4.10 as a normative
spec at the milestone-end criteria bar
([m3-plan.md §Milestone-end criteria item 5](../plans/m3-plan.md#milestone-end-criteria))
applied at phase close, and (iii) `examples/gallery/` +
`examples/gallery-rust/` are grown additively from Phase 2's single-Box
sub-screen into a WrapPanel of Box thumbnails. Per A11, all sides
advance together by phase close.

### Governance note (M3 phase-ADR vs RFC transition)

[VISION.md §9.2](../../VISION.md#92-decision-making) describes a
"gradual transition to RFC-based consensus" for M3 onward, with
major changes discussed in `docs/rfcs/`.
[docs/decisions/README.md §Scope and relation to RFCs](./README.md#scope-and-relation-to-rfcs)
echoes the same wording. Both texts treat M1/M2 phase-ADR practice
as the pre-transition baseline.

In observed practice, M3 Phase 1
([m3-phase-1-bool-scalar.md](./m3-phase-1-bool-scalar.md)) and
M3 Phase 2 ([m3-phase-2-box-layout.md](./m3-phase-2-box-layout.md))
both ran as ADRs without invoking the RFC process;
`docs/rfcs/` does not yet exist in the repository, and the
RFC-process content (template, lifecycle, acceptance rule) has not
been written. This Phase 3 ADR continues the M3 phase-ADR pattern
Phase 1 / Phase 2 established, for consistency with the realised
M3 flow and because the RFC machinery is not in place.

The disconnect between VISION / decisions-README text ("transition
to RFC") and the observed M3 phase-ADR practice is noted here so
it is not silently propagated through Phase 3. **Resolving the
disconnect is out of scope for this Phase 3 ADR** and warrants a
separate vision decision record — either revising VISION §9.2 / decisions-README
§Scope to match the realised phase-ADR flow (deferring RFC adoption
to a later milestone), or standing up `docs/rfcs/` and re-routing
M3 phases through it (which would retroactively reframe M3 Phase 1 /
Phase 2 / Phase 3 as transitional).

**Owner-confirmation request:** this ADR is *Proposed* on the
assumption that the phase-ADR path is acceptable for Phase 3, on
parity with M3 Phase 1 / Phase 2. If the owner instead wants
Phase 3 gated on RFC-process setup, this ADR's `Status: Accepted`
flip blocks until the governance question is resolved upstream.

**Resolution (post-hoc, 2026-05-25).** The upstream governance
question was resolved by
[vision-governance-rfc-deferral.md DD-V-018](./vision-governance-rfc-deferral.md#dd-v-018--defer-rfc-adoption-to-post-10),
which collapsed the three-stage governance trajectory into two
stages (pre-1.0 BDFL + ADRs, post-1.0 open governance + RFC
machinery introduced together). The phase-ADR path was therefore
the correct choice for Phase 3 and remains the path for the
remainder of the pre-1.0 period. This historical note is left
intact per the supersede rule.

### Layering note (DD-001 ⇄ DD-004 ⇄ DD-005)

Phase 3's structural DD (DD-001 — IR shape + child layout contract),
its item-sizing-source DD (DD-004 — cross-axis bound), and its
algorithmic DD (DD-005 — measure-arrange algorithm) are **layered**,
with the dependency direction inverted relative to Phase 2:

- **Phase 2 (Box).** Outer bounds resolve *without* considering child
  intrinsic size when `aspect` is set. Aspect-derived bounds win;
  children do not grow the Box. Outer first, inner second.
- **Phase 3 (WrapPanel).** WrapPanel's outer bounds *do* depend on
  children. Each child's main-axis intrinsic size determines which
  line it joins; each line's cross-axis extent depends on the children
  in that line (and on DD-004's `item-cross-size`, when set);
  WrapPanel's cross-axis outer size is the sum of line cross-axis
  extents (plus any line-spacing). The main-axis outer size is bounded
  by the parent's main-axis constraint. **Inner first, outer second**
  on the cross axis; outer-equals-constraint on the main axis.

DD-004 sits *between* DD-001 and DD-005 in the chain: DD-001 says
"children measured against unbounded main-axis + cross-axis
passthrough"; DD-004 settles *which* cross-axis is passed through
(parent's, or `item-cross-size`-bounded); DD-005 then runs the line
breaker on the resulting child measures.

This inverted layering is the structural content of "two-stage
measure-arrange". Each DD's Recommendation prose cites the layering
so reviewers can verify Option respect for the dependency direction.

Concrete consequence: the following combinations are **invalid** and
do not appear as recommended options —

- DD-005 = "WrapPanel cross-axis ignores children" with any DD-001
  multi-child Option (would make WrapPanel a Grid row, not a
  WrapPanel).
- DD-001 = "children share full WrapPanel main-axis bounds" with any
  DD-005 wrapping algorithm (would make wrapping structurally
  unreachable).
- DD-004 = "no cross-axis constraint policy at all" with any DD-005
  line cross-axis sizing Option (DD-005's line breaker needs a defined
  cross-axis-bound source, even when that source happens to be
  unbounded).

---

## Out of scope (for M3-Phase 3; recorded explicitly)

- **ScrollView pairing.** Phase 4. The wireframe shows
  `ScrollView { WrapPanel { … } }` for the overflow state; Phase 3
  ships the WrapPanel only, no viewport / clip / content offset
  binding. The gallery sub-screen's main-axis remains bounded by
  the window directly until Phase 4.
- **Padding attribute on WrapPanel.** Deferred; left-edge behaviour
  in Phase 3 sub-screen accepts the bare-WrapPanel default. Per
  DD-003 Option A's deferral.
- **Per-child main-axis size override** (e.g. "this thumbnail spans
  2 columns"). Grid territory, Phase 5.
- **Per-child cross-axis alignment override attribute** (DD-001
  Option C). The centred default applies uniformly in Phase 3.
- **Iteration grammar.** Phase 7. The wireframe shows generated
  thumbnails, but Phase 3 ships with a hand-written fixed set in the
  gallery sub-screen.
- **Image widget surface.** M4+; placeholder pattern from Phase 2
  DD-M3-P2-006 carries through unchanged.
- **`TypedValue` generic value union.** F5 maintained
  ([m2-to-m3-handover.md §4](../notes/m2-to-m3-handover.md)).
  DD-002 / DD-003 / DD-004's constant-only stances preserve the
  deferral structurally; no Phase 3 attribute pressures `TypedValue`.
- **Bindable surface for any WrapPanel attribute** exposed by
  DD-002 / DD-003 / DD-004. Each DD's bindable sub-issue settles
  constant-only in Phase 3; the per-type writer seam pattern from
  Phase 1 / Phase 2 remains available for the phase that first opens
  a bindable WrapPanel surface.
- **Vertical-main-axis WrapPanel.** DD-002 settles horizontal-only;
  vertical opens additively when a future phase needs it.
- **Per-item explicit dimensions on the Box child** (`width` /
  `height` are out of M3-Phase 2 DSL surface). DD-004 settles the
  Phase 3 sizing source as `item-cross-size` at WrapPanel level,
  not as item-level dimensions on Box.
- **Vertical-scroll viewport in the gallery sub-screen.** Per
  framing decision E: the Phase 3 sub-screen sizes the thumbnail
  set to wrap within the default window (800×600); overflow handling
  arrives with Phase 4 ScrollView.
- **C / Zig host parity for the WrapPanel sub-screen.**
  [m3-plan.md §Phase-end criteria item 5](../plans/m3-plan.md#phase-end-criteria)
  calls for at least one host per phase; Phase 8 broadens the full
  gallery to all three. Phase 3 grows `examples/gallery-rust/` only.
- **WrapPanel-specific layered diagnostics**
  (`LayoutError::WrapPanelUnboundedCrossWithAspectChild` or similar
  layered backtraces). DD-005 Option A under the unbounded-cross
  sub-issue defers diagnostic-quality improvements to a future
  surface phase; Phase 3 lets Phase 2's existing
  `LayoutError::BoxAspectUnboundedBoth` fire with the Box's IR
  location.

### Phase 3 implementation residuals (out-of-phase, filed at close)

Implementation findings that surfaced during Phase 3 T1–T9 but fall
outside this ADR's scope are recorded under
[m3-phase-3-progress.md §Out-of-phase residuals](../plans/progress/m3-phase-3-progress.md#out-of-phase-residuals):

- **R1** — `.gitignore` `*.uic` pattern (cross-cutting build
  hygiene).
- **R2** — `sync_visuals` ↔ pure-layout boundary test gap
  (post-Phase-3 test coverage; the architecture clarification half
was folded into [docs/architecture.md §6.5](../../../../docs/architecture.md)
  in the T10 close).

## Owner-agreement checkpoints

Two of the DDs above carry value judgements with multiple defensible
answers; they warrant explicit yes/no from the owner before this ADR
moves to Accepted. All other DDs follow mechanically from these two
and from the Phase 2 / Phase 1 inheritance.

### Checkpoint 1 — DD-M3-P3-003 spacing surface scope

**Question:** Does Phase 3 ship `item-spacing: <i32>` and
`line-spacing: <i32>` as constant-only attributes (Option A,
recommended), or does it ship no spacing attributes and accept the
visible deviation from the wireframe in the gallery sub-screen
(Option B)?

**Default answer:** Option A — ship both attributes; default 0;
reuse existing `i32` plumbing.

**Framing for owner:** Option A pays a small attribute-plumbing cost
(two new optional integer attributes on the WrapPanel IR node,
both reusing existing `i32` literal plumbing) for full
wireframe fidelity in the gallery sub-screen. The cost is genuinely
small — no new `IrType`, no new `IrLiteral` variant, no new
`PropertyValue` variant, no ABI surface change. Both attributes are
constant-only; no per-type writer seam built.

Option B saves the plumbing at the cost of a visibly-degraded
sub-screen (touching thumbnails rather than the wireframe's 12px
gap). The deviation is recoverable in a later phase, but the gallery
proof becomes "WrapPanel wraps" rather than "WrapPanel produces the
wireframe's layout". For readers coming from CSS / WPF / XAML
(where gap is table stakes), the absence creates an
expectations-mismatch about what a WrapPanel looks like.

The trade-off is design-quality dominated: Option A is the wireframe-
faithful path with minimal surface cost; Option B is the
minimal-surface path with a visible deviation.

### Checkpoint 2 — DD-M3-P3-004 default behaviour and warning

**Question:** When `item-cross-size` is unset on a WrapPanel, what
does the child receive as its cross-axis measure constraint?
**(a)** parent's full cross-axis constraint passed through
(recommended); **(b)** unbounded cross-axis constraint; **(c)**
`wasamoc check` rejects (compile-time-requires `item-cross-size`).
And: does Phase 3 ship the `wasamoc check` warning for
aspect-only-Box children without `item-cross-size`?

**Default answer:** Option (a) — parent passthrough — plus ship the
warning in Phase 3.

**Framing for owner:** This is the most load-bearing value judgement
in the ADR. The default-behaviour pick determines what happens when an
author writes a WrapPanel of aspect-only Box thumbnails *without*
setting `item-cross-size`:

- Option (a): The thumbnails inherit the parent's cross-axis bound.
  In a tall WrapPanel inside an 800×600 window, a 1:1 Box becomes
  ~600×600 — a single huge thumbnail, with subsequent thumbnails
  pushed onto new lines. The behaviour is *visibly wrong* but
  *deterministically derivable* from the spec; the `dsl_spec.md`
  pitfall note + the `wasamoc check` warning (shipped per the
  companion judgement) point the author at the fix.
- Option (b): The thumbnails immediately hit Phase 2's
  `LayoutError::BoxAspectUnboundedBoth` runtime error. The author
  gets an error rather than a wrong visual, but WrapPanel-in-a-sized-
  container with natural-cross-axis children (Text-only chips
  measuring cross-axis from the font) loses access to the
  container's cross-axis bound — those children measure as if the
  container were unbounded. Counter-intuitive for the common case.
- Option (c): `wasamoc check` rejects the WrapPanel-of-aspect-only-
  Box-without-`item-cross-size` pattern at compile time. The error
  is loud and early, but `wasamoc check` must statically classify
  children by "size source" — a classifier that scales poorly as the
  widget catalogue grows.

The framing's working direction (Option (a) + ship warning) is the
WPF / Slint precedent and the principle "WrapPanel does not redefine
child measure when the author has not asked it to". The warning is
the soft Option (c) — narrow, structurally cheap, catches the known
footgun without committing the spec to size-source classification.

The companion judgement (ship the warning vs defer it):

- Ship: Author-time guidance for the common failure mode. Cost is
  one diagnostic emission; the warning text references the
  `dsl_spec.md` pitfall note.
- Defer: Authors learn the footgun at runtime (huge-thumbnail visual)
  rather than at compile time. Recoverable but slower; reduces the
  Phase 3 diagnostic surface to the minimum.

The Phase 3 gallery sub-screen sets `item-cross-size: 88` explicitly
under both pick directions, so the answer here does not affect the
visible proof — it affects what authors who *omit* the attribute
experience.

---

## Summary of decisions

The **Forward-compat exposure** column rates the recommended option
of each DD per [decisions README §Risk evaluation](./README.md#risk-evaluation).
All six rate `Low` because every recommendation is structurally
additive against the foreseeable future events catalogued in
**Out of scope** above (Phase 4 ScrollView, Phase 5 Grid, Phase 6
ZStack, Phase 7 iteration, M4+ bindable / theming / Image-widget) —
none of those events forces a revision to the Phase 3 IR shape,
default behaviour, or spec text; each extends rather than rewrites.

Per-DD exposure paragraphs in each section above remain the
authoritative narrative; the column is the at-a-glance rating.
(The decisions-README requires the column for the recommended
option of every DD; this Phase 3 ADR adheres to the format. M3
Phase 1 / Phase 2 summary tables omit the column — surfacing the
question of whether to backfill them belongs in a separate
docs-cleanup pass, not this ADR.)

| ID | Topic | Recommendation | Forward-compat exposure |
|---|---|---|---|
| DD-M3-P3-001 | WrapPanel IR node form + N-child main-axis flow contract | Option A across sub-issues — per-kind tag `WidgetKind::WrapPanel`; 0-child and 1-child both valid; document-order placement with spacing-aware `<=` inequality and **unconditional placement of the first child of any line** (normatively spec'd in DD-005); child measure input is unbounded main-axis + DD-004-defined cross-axis; cross-axis item alignment is centred within line | Low |
| DD-M3-P3-002 | Orientation attribute | Option A — hardcode horizontal main-axis; no `orientation` attribute exposed in Phase 3; vertical opens additively in a later phase | Low |
| DD-M3-P3-003 | Spacing attributes (item-spacing / line-spacing / padding) | Option A — ship `item-spacing: <i32>` and `line-spacing: <i32>` as constant-only attributes (default 0; reuse existing `i32` plumbing; no new `PropertyValue` variant); defer padding to a later phase (load-bearing — see Checkpoint 1) | Low |
| DD-M3-P3-004 | Item sizing source | Option A across sub-issues — expose `item-cross-size: <i32>` as optional constant-only attribute; default behaviour when unset is Option (a) parent-cross-axis passthrough; per-line cross-axis sizing rule when attribute set is uniform `item-cross-size`; ship `wasamoc check` warning for aspect-only-Box children without `item-cross-size` (load-bearing — see Checkpoint 2) | Low |
| DD-M3-P3-005 | Measure-arrange algorithm (novel normative spec) | Option A across sub-issues — bounded main-axis greedy line breaker with spacing-aware `<=` inequality for subsequent children and **unconditional placement for the first child of any line** (oversized first-child line extent may exceed `parent_main_bound`; WrapPanel outer main-axis still equals `parent_main_bound` and visible overflow paints past the WrapPanel rectangle); cross-axis line sizing per DD-004 (uniform when `item-cross-size` set; max-of-children when unset); WrapPanel outer cross-axis is sum of line extents + `line_spacing × (line_count − 1)`; unbounded main-axis degenerates to **one-line flow** (no new `LayoutError` variant); unbounded cross-axis with aspect-only child fires Phase 2's existing `LayoutError::BoxAspectUnboundedBoth`; rounding contract inherits Phase 2 DD-005 (no pixel-snapping in Phase 3) | Low |
| DD-M3-P3-006 | IR-loader defense-in-depth invariants | Option A across sub-issues — two-gate defense (`wasamoc check` + `validate()`) for non-negative integer ranges on `item-spacing` / `line-spacing` / `item-cross-size`; zero is valid (author-requested degenerate layout); no child-count restriction (0+); orientation rule conditional on DD-002 (collapses under DD-002 recommendation); error class `WASAMO_ERR_IR_MALFORMED` | Low |

Implementation task list: belongs in the Phase 3 progress file
`docs/plans/progress/m3-phase-3-progress.md` (created when this ADR
is Accepted and Phase 3 starts execution); not in this ADR and not
in `m3-plan.md` itself. See
[plans/README.md §Scope rule (plan vs ADR)](../plans/README.md#scope-rule-plan-vs-adr)
and [plans/README.md §Phase progress file lifecycle](../plans/README.md#phase-progress-file-lifecycle)
for the authoritative location and the `active → closing → retired
→ archived` lifecycle the file follows. The Progress table in
[m3-plan.md](../plans/m3-plan.md) carries only a one-row index entry
pointing at this progress file.

## Spec impact preview (for owner agreement)

When this ADR is Accepted, the following docs change in the
**Moment 1** commit set (framing decision D — ADR-Accepted /
design-spec draft). Each constituent lands as its own commit on the
pre-doc branch, scoped by review concern per
[CLAUDE.md §Commit rules](../../CLAUDE.md#commit-rules):

- **ADR `Status: Accepted` flip** — this file.
- [docs/dsl_spec.md](../../../../docs/dsl_spec.md) — new **§4.10 WrapPanel chapter**
  alongside Phase 2's §4.9 Box chapter. The chapter contains:
  - The WrapPanel sizing mental-model subsection (framing decision H)
    — four-fact anchor + ecosystem-contrast block (WPF / Compose /
    CSS), positioned before the formal measure-arrange algorithm so
    the reader builds the model before the rules. Cross-referenced
    from DD-005's Recommendation prose in this ADR.
  - The two-stage measure-arrange algorithm (DD-005 Recommendation
    prose lifted into normative spec form, with the inequality
    derived from DD-001 and the **first-child unconditional
    placement** carve-out spelled out).
  - The **oversized first-child / oversized line behaviour** —
    line breaker places oversized first-child unconditionally; line
    extent may exceed `parent_main_bound`; WrapPanel outer main-axis
    is still `parent_main_bound`; visible overflow paints past the
    WrapPanel rectangle unless an enclosing parent clips
    (per DD-005 oversized-line Option A).
  - Attribute reference: `item-cross-size` (DD-004), `item-spacing`
    / `line-spacing` (DD-003).
  - The "common pitfalls" note: aspect-only Box children without
    `item-cross-size` (DD-004 Recommendation companion) **and**
    oversized children painting past the WrapPanel boundary
    (DD-005 oversized-line Option A note).
  - Section marker
    `**Phase status:** M3-Phase 3 ADR-accepted design draft; pending
    implementation re-sync` at the chapter top.
- [docs/architecture.md](../../../../docs/architecture.md) §6 — WrapPanel entry
  under the M2-revised IR section if structural placement warrants;
  short paragraph noting (a) WrapPanel's two-stage measure-arrange
  is the first M3 layout primitive with cross-axis line aggregation,
  (b) the per-type binding seam is *not* extended by Phase 3 (all
  WrapPanel attributes constant-only) so the F5 deferral is
  unpressured, and (c) layout engine boundary remains Win32/WinRT-
  free — the line breaker operates on pure data
  ([predoc-inputs.md §8](../notes/m3-phase-3/predoc-inputs.md)).
- [docs/abi_spec.md](../../../../docs/abi_spec.md) — **no changes in Phase 3**.
  No new ABI public function, no new `WASAMO_VALUE_*` tag, no new
  arms in `abi.rs`. All WrapPanel attributes are constant-only
  `i32` and stay on the WrapPanel-internal IR node; they do not
  traverse `PropertyValue`-mediated paths. No new
  `WASAMO_LAYOUT_ERROR_*` extension — DD-005 Option A adds no
  variant, and the unbounded-cross case fires Phase 2's existing
  Box-side error.
- [docs/plans/m3-plan.md](../plans/m3-plan.md) — Progress section's
  Phase 3 row populated (Status: `in progress`; ADR link; progress
  file link).
- [docs/plans/progress/m3-phase-3-progress.md](../plans/progress/) —
  new file opened with task list mapped to this ADR's verification
  closure items below.
- [docs/notes/retrospectives.md](../notes/retrospectives.md) —
  **no Phase-3-specific amendment expected at framing time** per
  framing decision F's process-rules-ssot disposition. Phase 2's
  `cargo fmt` discipline tightening was a one-off; no analogous
  pattern is anticipated for Phase 3 at the framing stage. The
  retrospective discipline (forward distillation per
  [[feedback_retro_forward_distillation]]) carries forward without
  text changes.

The **Moment 2** commit set (framing decision D — Phase close /
implementation re-sync) lands at phase end; the WrapPanel-chapter
spec marker flips to `**Phase status:** M3-Phase 3 closed;
implementation-synced`, any divergence between the design-spec
draft and the implementation is corrected in the same commit, and
earlier-phase spec gaps surfaced during re-sync may fold per
[[feedback_retroactive_spec_gap_fold]] with explicit owner
confirmation. The Phase 3 progress file is retired in the same
commit set per the standard `active → closing → retired →
archived` lifecycle.

No ROADMAP revision is anticipated — A3 is already explicit, this
ADR operationalises it.

## Phase 3 verification closure (what counts as A3 evidence)

This section is not a DD — it records the agreed shape of the proof
that closes Phase 3 per framing decision C, so the implementation
plan inherits a concrete target rather than re-litigating "what does
WrapPanel's verification mean here?".

A3 (WrapPanel layout primitive — two-stage measure-arrange) is
considered satisfied when **all five** of the following are observed:

1. **`wasamoc check` compile-time evidence (host-independent).**
   Pure-logic tests in `wasamoc`'s check / lower path cover:
   - **Negative literal rejection** (DD-006 two-gate, compile-time
     half) — `item-spacing: -1`, `line-spacing: -1`,
     `item-cross-size: -1` each surface a `wasamoc check`
     diagnostic naming the rejected attribute. The runtime
     `validate()` half is covered by item 3 below.
   - **Aspect-only-Box warning** (DD-004 Recommendation companion)
     — a WrapPanel directly containing one or more `Box { aspect:
     <ratio>; … }` children with no `item-cross-size` set on the
     WrapPanel surfaces a `wasamoc check` *warning* (not error)
     suggesting the attribute. The warning fires on the framing's
     working pick; if Checkpoint 2's companion judgement flips to
     "defer the warning", this bullet is dropped together with
     the implementation.
   - **Sub-screen positive control** — the gallery sub-screen's
     `.ui` (item 5 below) is *not* expected to emit either the
     rejection or the warning (`item-cross-size: 88` is set
     explicitly); compiling it cleanly is the positive control.

   These run on any CI runner; the diagnostics are pure-logic in
   `wasamoc`.

2. **Line-breaker and arrange unit-test evidence (host-independent).**
   Pure-logic tests against the layout engine's WrapPanel
   measure/arrange (`wasamo-runtime/src/layout.rs` extension) cover:
   bounded main-axis happy path with multi-line wrap; bounded main-
   axis happy path with single-line fit (no wrap needed);
   **oversized-first-child unconditional placement** (a single
   child whose intrinsic main extent exceeds `parent_main_bound` is
   placed on a one-child line whose recorded extent exceeds the
   bound; subsequent children start new lines); **WrapPanel outer
   main-axis equals `parent_main_bound` even when an oversized
   first-child is present** (per DD-005 oversized-line Option A);
   **arrange-pass evidence of visible overflow** — for the
   oversized-first-child fixture, the arranged child rect's main-
   axis end (`child.x + child.w` in horizontal-main-axis terms)
   exceeds the WrapPanel rect's main-axis end. This is the pure-
   data observable form of "child paints past the WrapPanel
   rectangle"; the absence of a clip surface installed by WrapPanel
   itself is item 4's runtime-side concern; unbounded-main-axis
   branch (one-line flow per DD-005); cross-axis line sizing when
   `item-cross-size` is set (uniform per-line extent); cross-axis
   line sizing when `item-cross-size` is unset (max of children's
   reported sizes); spacing-aware overflow inequality for
   `line_empty == false` (no trailing margin); `item-spacing: 0`
   and `line-spacing: 0` degenerate layouts; `item-cross-size: 0`
   author-requested degenerate layout; unbounded-cross-axis-with-
   aspect-child propagating to Phase 2's
   `LayoutError::BoxAspectUnboundedBoth`. These run on any CI
   runner; the line breaker and arrange pass are pure functions
   (input → output) per framing decision C.

3. **IR-loader / `validate()` invariant evidence (host-independent).**
   Pure-logic tests in `wasamo-runtime`'s `ir_loader::validate()`
   path cover (DD-006 two-gate, runtime half): negative literal
   rejection for `item-spacing`, `line-spacing`, `item-cross-size`
   in memory-IR that bypasses `wasamoc` (each surfaced as
   `WASAMO_ERR_IR_MALFORMED`); 0-child WrapPanel valid; 1-child
   WrapPanel valid; multi-child WrapPanel valid (no upper bound).
   Symmetric with Phase 2 T7's `validate()` discipline.

4. **Windows-runtime layout evidence (CI-gated).** A mock-free
   integration test (per
   [CLAUDE.md §Testing rules](../../CLAUDE.md#testing-rules)) on
   the Windows CI runner exercises two fixtures:

   - **Wrap-path fixture (primary).** A `.ui` declares a WrapPanel
     with `item-cross-size: 88; item-spacing: 12; line-spacing: 12`
     and 5–10 `Box { aspect: 1:1; fill: #336699cc; Text { text: …
     } }` children inside a parent of known main-axis size. The
     test loads the IR, runs the layout pass, and asserts (a) the
     WrapPanel's resolved rectangle matches the expected outer
     dimensions (main-axis = `parent_main_bound`; cross-axis =
     `line_count × 88 + (line_count − 1) × 12`), (b) each child's
     resolved rectangle is 88×88, (c) child positions match the
     expected line / column assignment given the main-axis bound.
   - **Oversized-child fixture (visible-overflow regulation).** A
     small `.ui` declares a WrapPanel inside a narrow parent
     (e.g. `parent_main_bound = 100`) with a single
     `item-cross-size: 50` Box child whose intrinsic main-axis
     extent exceeds the bound (`Box { aspect: 4:1 }` →
     intrinsic main = `50 × 4 = 200`). The test asserts (a) the
     WrapPanel's resolved rectangle still has `main-axis = 100`
     (per DD-005 oversized-line Option A — WrapPanel does not grow),
     (b) the child's resolved rectangle has `width = 200` and
     `x + width > 100` (visible overflow at the runtime layer is
     preserved through to the Visual Layer tree), and (c) WrapPanel
     installs **no clip surface** on its Composition visual — this
     is observable through the Visual tree the runtime produces
     (the WrapPanel's `ContainerVisual` has no `Clip` property set
     by Phase 3 code). The absence-of-clip assertion is the
     runtime-side complement to item 2's arrange-pass evidence:
     together they establish that the spec'd "visible overflow,
     parent clips" convention is implemented end-to-end.

   Both fixtures fail (not skip) on a runner that cannot create the
   Compositor — the test gates A3 evidence in CI, not local
   convenience. Skip-guard inherits the Phase 2 T11 pattern verbatim
   (fires on `0x80070005` from `wasamo_init`).

5. **End-to-end host evidence (visible smoke).**
   `examples/gallery/gallery.ui` is **grown additively** from
   Phase 2's single-Box sub-screen into a WrapPanel of uniform
   1:1 Box thumbnails (5–10 items, hand-written; no iteration,
   no ScrollView). `examples/gallery-rust/` (already a workspace
   member from Phase 2) builds and runs the grown sub-screen.
   `Start-Process` launch is recorded as successful by the assistant;
   **visual correctness** of the WrapPanel rendering (lines wrap
   correctly at the expected main-axis budget; line-spacing
   produces the expected cross-axis gaps; item-spacing produces
   the expected main-axis gaps; the sub-screen visually matches
   the wide-state wireframe within reason) is **owner-manual GUI
   smoke** per framing decision G — the assistant does not assert
   on pixel- or eyeball-level correctness.

Items (1)–(4) are required for A3 acceptance; item (5) ties the
evidence back to the m3-plan target-app trajectory and grows the
gallery sub-screen Phase 2 seeded. C and Zig hosts for the
WrapPanel sub-screen are explicitly **not** required in Phase 3
(per framing decision E and the Out of scope list); Phase 8
broadens the full gallery to all three.

Per [predoc-inputs.md §10](../notes/m3-phase-3/predoc-inputs.md),
evidence items (1)–(4) do not collapse into one even though they
share helper infrastructure — the `wasamoc check` diagnostics, the
line-breaker tests, the IR-load `validate()` gate tests, and the
Windows integration test each have distinct evidence meanings
(compile-time guard enforcement; algorithm correctness; runtime-
side invariant enforcement; live-runtime composition).

The acceptance / non-acceptance of test items (1)–(5) is the
operational form of "Phase 3 done"; the corresponding
implementation checklist (which crate / which test file / which
fixture) belongs in the Phase 3 progress file, not here.
