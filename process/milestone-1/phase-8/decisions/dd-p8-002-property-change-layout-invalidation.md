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
[`architecture.md` §6](../../../../docs/architecture.md#6-layout-engine-phase-3)):**

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
  [DD-V-001](../../../cross-milestone/decisions/m1-acceptance-criteria.md), property-change
  animations are M5 scope. The relayout here is instant —
  consistent with SwiftUI / Compose / Flutter / CSS defaults.

---
