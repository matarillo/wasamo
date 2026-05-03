# Phase 8 — Hello Counter Sample × 3 Languages: Architecture Decisions

**Phase:** 8 (Hello Counter sample × C / Rust / Zig — final M1 deliverable)
**Date:** 2026-05-01
**Status:** Accepted (2026-05-01)

## Context

Phase 8's acceptance criterion comes directly from
[VISION §7 M1](../../VISION.md#7-roadmap--milestones) and
[ROADMAP M1](../../ROADMAP.md#m1-proof-of-concept):
**"Hello Counter example runs in three languages: C, Rust, and Zig."**

The runtime ([`wasamo-runtime`](../../wasamo/)), the C ABI
([`bindings/c/wasamo.h`](../../bindings/c/wasamo.h),
[`docs/abi_spec.md`](../abi_spec.md)), and the three bindings
([`bindings/c/`](../../bindings/c/),
[`bindings/rust/`](../../bindings/rust/),
[`bindings/zig/`](../../bindings/zig/)) all landed in Phases 6–7.
Phase 8 consumes them: each binding gets one host-language
"counter" program that reproduces [`examples/counter/counter.ui`](../../examples/counter/counter.ui).

The ROADMAP Phase 8 task list ([../../ROADMAP.md L224-L233](../../ROADMAP.md#L224-L233))
has eight items. Per
[Pre-doc discipline](./README.md#pre-doc-discipline) those are
working hypotheses; this ADR revisits them against the acceptance
criterion. Of the questions surfaced, two warrant ADR-level
record:

1. **DD-P8-001** — How `examples/counter/counter.ui` relates to the
   three host programs. The other M1 framing documents
   ([abi_spec §5.1](../abi_spec.md#51-what-m1-experimental-verifies-and-what-it-does-not),
   [VISION §7 M1](../../VISION.md#7-roadmap--milestones)) already
   carve out the M1/M2 split; Phase 8 is where that split first
   becomes visible to end users, so the application warrants one
   explicit decision and a few small upstream wording adjustments.
2. **DD-P8-002** — A runtime change Phase 8 forces. Property
   updates that change a widget's intrinsic size do not currently
   trigger re-layout; Hello Counter's `Count: N` text becomes
   visually stale after `N` grows past one digit. This is a
   permanent runtime addition, not a Phase 8 workaround.

The remaining items from the Phase 8 exploration — `window_create`
signature kept as-is (status quo); string-lifetime clarification
(documentation fix to abi_spec §4.3, not a decision); Quick Start
language (C, on grounds of the project's "C ABI first" framing);
CI builds all three counter examples to release-build success
(operational); release tagged `v0.1.0` (sole semver-clean option
that keeps the M2/M3/M4 = 0.2/0.3/1.0 mapping legible) — are not
ADR-shaped. They are recorded in the Phase 8 ROADMAP entry, in
PR descriptions, and in the affected docs (READMEs, `abi_spec.md`,
CI workflow). See
[Option enumeration discipline](./README.md) for why we keep
ADRs scoped to substantive choices rather than every Phase 8
sub-task.

---

### DD-P8-001 — `counter.ui` positioning in Hello Counter

**Status:** Accepted

**Context:**
[`examples/counter/counter.ui`](../../examples/counter/counter.ui)
already exists from Phase 1 as the canonical reference example for
the `.ui` DSL. Phase 8's three host programs need to satisfy
"Hello Counter runs in three languages." How they should relate to
`counter.ui` is not free of choices.

[abi_spec §5.1](../abi_spec.md#51-what-m1-experimental-verifies-and-what-it-does-not)
already states the principle: "M1 wasamoc is parser-only by design;
host code constructs the equivalent tree directly through the
experimental layer. The lowering itself is M2 scope." This ADR
applies that principle to Phase 8 and decides how visible the
distinction is in the deliverables.

**Options:**

Option A — `.ui` is reference-only; host code is hand-written (recommended)
`counter.ui` is left untouched. Each `examples/counter-{c,rust,zig}/`
program constructs the same widget tree imperatively through the
experimental ABI / its safe wrapper. The example READMEs cross-link
to `counter.ui` and explicitly state that the lowering is M2.
`wasamoc check examples/counter/counter.ui` continues to pass and
is wired into CI.

- What you gain: Aligned with abi_spec §5.1 and VISION §7 M1
  (which scopes M1 to runtime-side ABI mechanics). Phase 8 ships
  the runtime/ABI/Visual-Layer hypothesis check that M1 is
  actually about. No new dependency on M2 design questions
  (codegen vs IR) that would have to be settled prematurely. The
  smallest implementation surface.
- What you give up: A reader who arrives at Phase 8 expecting to
  see `.ui → runtime` will find host-imperative code instead. This
  is honest about the M1/M2 split, but it puts a documentation
  burden on the example READMEs and the README Quick Start to
  explain it without sounding apologetic.

Option B — Hand-translation contract
Same as A, plus each `main.*` carries a header comment annotating
which `counter.ui` lines each block of imperative code corresponds
to. Reviews check the two against each other.

- What you gain: Makes the future `.ui → runtime` lowering visible
  as a structural mapping; readers see the M2 codegen target in
  rough shape without the codegen.
- What you give up: Ongoing review/maintenance overhead. If
  `counter.ui` evolves (e.g., signal body changes), three host
  files need synchronised edits or the comments rot. The benefit
  is documentation-only; the actual M2 lowering work is not made
  easier by these comments because M2 will operate on the AST,
  not the textual form.

Option C — Drop `counter.ui` from Phase 8 framing entirely
Treat `counter.ui` purely as a Phase 1 wasamoc artifact, and
drop the cross-linking from Phase 8 examples. M1 acceptance reads
"three host programs that produce the counter behavior."

- What you gain: No reader-expectation gap; what you see is what
  Phase 8 ships.
- What you give up: Severs the visible connection between the
  three M1 pillars (DSL, C ABI, Visual Layer). VISION §1 frames
  Wasamo as "DSL × C ABI × Visual Layer"; deleting `.ui` from the
  Hello Counter narrative makes one pillar invisible at the
  showcase moment.

**Recommendation: Option A.**

Option A is what abi_spec §5.1 already implies; Phase 8 just
applies it to a concrete deliverable. Option B's
hand-translation comments are documentation-shaped work whose
ongoing cost outweighs the documentation-shaped benefit, and they
risk reading as a partial codegen design that M2 might not
follow. Option C achieves cleanliness by removing one of the M1
pillars from view, which trades long-term framing for short-term
clarity — the wrong direction.

**Upstream document alignment.**

The choice does not require any *substantive* change to upstream
documents — the M1 / M2 split is already in abi_spec §5.1 and the
VISION §7 M1 paragraph. Two small wording tweaks reduce reader
friction:

1. **ROADMAP Phase 8 task list, item 1** currently reads
   `examples/counter/counter.ui`, which is misleading since the
   file already exists from Phase 1. Revise to make clear that
   Phase 8 only verifies it still parses, and add an item for the
   READMEs that carry the M1/M2 framing message:
   - `[ ] verify examples/counter/counter.ui still parses with wasamoc check (already exists from Phase 1)`
   - `[ ] each example README explains: this is the M1 host-imperative shape; .ui → runtime lowering is M2`
2. **VISION §7 M1 paragraph** is correct but stops short of
   naming the visible consequence. Add one sentence at the end of
   the M1 paragraph (after the existing "wasamoc output format…"
   sentence): "Concretely, the M1 Hello Counter examples
   construct the widget tree imperatively through the C ABI; the
   `.ui → runtime` lowering arrives in M2."

VISION §1 ("UI is written in an external DSL … and consumed from
any language through a stable C ABI") is the project's long-term
framing and is correct without M1 caveats. M1-specific qualifications
belong in §7 and in `abi_spec.md`, not in §1.

abi_spec §5.1 needs no change.

**Explicitly deferred:**
- Whether the `counter.ui` source file moves under
  `examples/counter/` or gets a sibling `examples/counter-ui/`
  with its own README. Current placement (file alone in
  `examples/counter/`) is fine; reorganization is M2's call when
  `wasamoc` produces an actual artifact from it.

---

### DD-P8-002 — Property-change layout invalidation

**Status:** Accepted

**Context:**
The current runtime updates `widget.width` / `widget.height`
inside `wasamo_set_property` for size-affecting properties
(`BUTTON_LABEL`, `TEXT_CONTENT`, `TEXT_STYLE`) but does not
trigger a re-layout pass. The only path that drives
`run_layout()` is `WM_SIZE` handling
([`wasamo/src/window.rs`](../../wasamo/src/window.rs)). Hello
Counter's `Text { text: "Count: \{root.count}" }` becomes visually
stale as `count` grows past one digit: the underlying drawing
surface re-renders, but the parent VStack doesn't re-arrange to
the new intrinsic size.

This is a runtime gap, not a Phase 8 example-side problem; any
host (C / Rust / Zig) that calls `wasamo_set_property` on a
size-affecting property hits it. Phase 8 is the first phase that
exercises post-construction property mutation as a user-visible
flow, so it surfaces here.

**Options:**

Option A — Auto re-layout inside `set_property` (recommended)
`wasamo_set_property` classifies the property: if the property
affects intrinsic size, it walks up to the owning window's root
and schedules a `run_layout()` pass before returning. The
classification is per-widget-type per-property-id — small finite
table for M1's four properties.

- What you gain: Transparent to hosts. Matches the SwiftUI /
  Compose / Flutter mental model: state change → invalidate →
  layout pass. The right model for the M2 reactive engine to
  consume; M2 reactivity will trigger property updates and
  expects layout to follow without further plumbing.
- What you give up: Slightly more runtime code; one classification
  table to maintain. Not free, but small.

Option B — Explicit invalidate API
Expose `wasamo_widget_invalidate_layout(WasamoWidget*)` (or
`_invalidate_layout` on the owning window). Hosts call it after
size-affecting `set_property` calls.

- What you gain: Trivial implementation. Cost is fully visible to
  the host.
- What you give up: Every host gets it wrong at least once.
  Adds ABI surface that exists only to compensate for a runtime
  shortcut. M2 reactive engine would have to call it on the
  host's behalf — at which point you've recreated Option A but
  through the ABI boundary. Net negative.

Option C — Workaround via fixed-width Text
Change `examples/counter/counter.ui` to reserve enough width for
many digits (padding or a hypothetical `min-width`). No runtime
change.

- What you gain: Zero runtime work.
- What you give up: Distorts the canonical example to compensate
  for a runtime bug. `min-width` is not in the M1 DSL grammar, so
  this requires DSL surface additions for a non-DSL reason. Worst
  trade-off in the set.

Option D — Restrict counter range to single-digit
Cap `count` at 9, or use a Reset that keeps count single-digit.
No runtime change.

- What you gain: Trivial.
- What you give up: Toy demo. Hides what Hello Counter is
  supposed to validate.

**Recommendation: Option A.**

Option A is the architecturally correct shape and is what M2's
reactive engine will need anyway. Implementing it in M1 means M2
inherits a working "state change → relayout" path instead of
having to retrofit one. The implementation is small (classification
table for four property IDs; root-walk to schedule layout). Option
B replicates this work at the ABI boundary and adds an API that
will be deprecated as soon as M2 internalises the call. Options C
and D contort the example to hide the gap.

**Architecture details (to be reflected in
[`architecture.md` §6](../architecture.md#6-layout-engine-phase-3)):**

- `set_property` for `BUTTON_LABEL` / `TEXT_CONTENT` /
  `TEXT_STYLE` re-computes the widget's intrinsic size, then
  marks the owning window for re-layout.
- A "marked" window runs `run_layout()` once at the next
  message-loop tick (queued, not synchronous, to coalesce
  multiple property changes in the same emission drain). The
  existing queued-emission machinery (`emit.rs`,
  [Phase 6 commit `4de8e7f`](../../wasamo/src/emit.rs)) is the
  right place to drain layout invalidations: after the signal
  queue empties, any marked window runs one layout pass.
- Widgets without an owning window (unattached, pre-`set_root`)
  defer; layout runs when they enter a window via `set_root`.
- `BUTTON_STYLE` does not affect intrinsic size in M1 (Default vs
  Accent share the same metrics); it stays as a simple visual
  refresh.

**Explicitly deferred:**
- Partial-tree relayout (re-measuring only the affected subtree).
  M1 re-runs layout from the root for simplicity. Hello Counter
  trees are small; the cost is invisible. Optimization belongs to
  M3 performance work.
- Animating property-change transitions. Per
  [DD-V-001](./vision-m1-acceptance-criteria.md), property-change
  animations are M5 scope. The relayout here is instant —
  consistent with SwiftUI / Compose / Flutter / CSS defaults.

---

## Out of scope for this ADR

- M2 wasamoc codegen format
- M2 reactive engine design
- Tree-mutation primitives at the ABI surface
- Any new ABI surface beyond what Phase 6/7 already shipped
  (DD-P8-002 is a runtime-internal change; no header changes)
- Multi-window scenarios (M5)

---

## Revision history

| Version | Date       | Notes                             |
|---------|------------|-----------------------------------|
| 0.1     | 2026-05-01 | Initial draft, Accepted same session |
