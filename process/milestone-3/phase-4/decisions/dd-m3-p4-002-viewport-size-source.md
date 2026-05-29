### DD-M3-P4-002 — Viewport size source

**Status:** Accepted

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
