### DD-M3-P3-001 — WrapPanel IR node form and N-child main-axis flow contract

**Status:** Accepted

**Context:**
WrapPanel is a new layout primitive in `wasamo-ir` and
`wasamo-runtime`. Phase 3 must commit to (i) the IR node shape,
(ii) the 0-child / 1-child / N-child shapes, (iii) child measure
input, (iv) line membership rule (the overflow comparison), and
(v) cross-axis alignment of items within a line.

**Options (IR node shape):**

Option A — Per-kind tag parallel to `HStack` / `VStack` /
`Rectangle` / `Box` (recommended)
- `WidgetKind::WrapPanel` joins the existing per-kind enumeration;
  the layout function in `wasamo-runtime` dispatches on the tag.
  Phase 2 DD-M3-P2-001 settled the per-kind-tag answer for Box;
  Phase 3 inherits unless evidence forces re-opening.

  - What you gain: Symmetric with every existing M2/M3 widget.
    Pattern matching on `WidgetKind` remains exhaustive at compile
    time.
  - What you give up: One more tag at every `WidgetKind` match site.
    Set is small and discoverable.
  - **Technical risk:** Low.

Option B — Structural variant in an `IrLayout` umbrella
- Re-opens the M2-Phase-6 / M3-Phase-2 layout-family question
  ("would HStack / VStack / Box / WrapPanel benefit from being
  grouped under a structural family enum?") mid-milestone for one new
  widget.

  - What you give up: No payoff inside Phase 3, and the regrouping
    would touch every M2/M3-Phase-2 layout dispatch site. The
    discipline of "build the seam in the phase that needs it" applies:
    no phase yet needs the umbrella.
  - **Technical risk:** Medium.

**Options (0-child / 1-child shape):**

Option A — Both valid; 0-child produces a zero-extent line set,
1-child produces a one-line layout (recommended)
- The IR loader admits empty `children` lists and single-child lists;
  the layout pass produces a zero-line / one-line outcome
  respectively. Consistent with Phase 2 Box's 0-child shape rule.

  - What you gain: Author can write `WrapPanel { … }` with a list
    being populated in iteration order (relevant once Phase 7
    iteration lands) without the IR loader rejecting the in-progress
    shape.
  - **Technical risk:** Low.

Option B — Reject 0-child WrapPanel at IR load; warn / reject 1-child
WrapPanel as "use Box / HStack instead"
- Diagnoses empty / degenerate WrapPanel as a `wasamoc check`
  warning or error.

  - What you give up: 0-child WrapPanel has a real use under future
    iteration (an empty collection yields zero items); rejecting it
    at IR-load contradicts the iteration grammar's future shape.
    1-child WrapPanel is structurally meaningful (the wrap is
    conditional on multi-item content; one item is the boundary case
    of "no wrap yet").

**Options (N-child main-axis flow / overflow comparison):**

Option A — Document-order placement; spacing-aware overflow inequality
with a `line_empty` carve-out (recommended)
- Children are placed in document order along the main axis. When
  the current line already has at least one child
  (`line_empty == false`), the next child is accepted onto the line
  iff

  ```
  current_line_main + item_spacing
    + next_child_main_intrinsic
    <= parent_main_bound
  ```

  When the inequality fails, the next child starts a new line. The
  **first child of any line** (`line_empty == true`) is placed
  unconditionally — even if its intrinsic main-axis extent alone
  exceeds `parent_main_bound`. The line's recorded extent may then
  exceed the bound (the "oversized first-child" case spec'd in DD-005;
  visible overflow handling and the WrapPanel outer-main-axis rule
  are DD-005's "oversized line — arrangement / paint clip" sub-issue).
  No trailing `item_spacing` accrues after the last child of a line.
  DD-001 records the **contract** (document-order, spacing-aware,
  unconditional first-child placement); DD-005 records the
  algorithmic statement and the WrapPanel-outer-bounds consequence.

  - What you gain: Eliminates "trailing margin" ambiguity. Deterministic
    against floating-point intrinsic sizes (the only float in the
    inequality is `next_child_main_intrinsic`; integer literals are
    promoted symmetrically).
  - **Technical risk:** Low.

Option B — Strict greater-than (next-child-must-strictly-fit)
- Drops the `<=` in favour of `<`. A line that ends exactly at
  `parent_main_bound` rejects any further child even if that child
  has zero intrinsic main-axis.

  - What you give up: Zero-width children (degenerate case; not
    expected in Phase 3's sub-screen but possible once iteration
    lands) would force a new line redundantly. The behavioural
    difference is invisible in any realistic test fixture but the
    spec would have to explain it.

**Options (child measure input):**

Option A — Unbounded main-axis + DD-004-defined cross-axis (recommended)
- Each child is measured against an **unbounded main-axis constraint**
  (so the child reports its intrinsic main-axis size, which feeds
  line membership) and a **cross-axis constraint defined by DD-004**
  — either WrapPanel's `item-cross-size` when set, or the parent's
  cross-axis constraint passed through when unset.

  - What you gain: The line breaker has stable input regardless of
    whether the WrapPanel sits inside a bounded or unbounded
    cross-axis parent (the answer is determined by DD-004, not by an
    implicit re-derivation here). Honours the layering note: DD-001
    consumes DD-004's settlement; it does not duplicate it.
  - **Technical risk:** Low.

Option B — Each child receives a slot-sized main-axis constraint
- Each child gets `parent_main_bound / N` (or similar) as its
  main-axis measure input.

  - What you give up: This is the Grid cell semantic. A WrapPanel
    whose children all measure to the same main-axis extent reduces
    the line breaker to a counting exercise — line membership becomes
    "how many fit per row, all the same width". Reject as a back-door
    Grid for the same reason Phase 2 rejected back-door ZStack.
    Contradicts the layering note's DD-001 → DD-005 input shape.

**Options (cross-axis item alignment within a line):**

Option A — Center (recommended, mirrors Phase 2 Box DD-001 default)
- When a line's members have heterogeneous cross-axis sizes
  (smaller items shorter than the line max), smaller items are
  centred on the cross axis within the line. No per-child override
  in Phase 3.

  - What you gain: Matches Phase 2 Box's `align: center` default.
    Reader builds one mental model for "where does the smaller thing
    sit?" across both primitives. The Phase 3 sub-screen uses uniform
    1:1 thumbnails (cross-axis sizes match by construction), so the
    default has no *observable* effect in the gallery proof — Phase 3
    settles the default for the eventual heterogeneous-line case
    rather than reserving no surface.
  - **Technical risk:** Low.

Option B — Start (top in horizontal main-axis)
- Smaller items anchor at the cross-axis-start edge of the line.

  - What you give up: Contradicts Phase 2 Box's alignment default
    without justification. A heterogeneous line of mixed-aspect
    thumbnails would top-align rather than centre, which is the
    less-common photo-gallery convention.

Option C — Configurable per-child via a new attribute
- Add `align: <center|start|...>` to children of WrapPanel.

  - What you give up: New attribute surface unmotivated by any Phase
    3 acceptance criterion. Out of phase scope; defer to the phase
    that needs it.

**Recommendation:** Option A for every sub-issue —

- IR shape: per-kind tag (`WidgetKind::WrapPanel`).
- 0-child / 1-child: both valid.
- N-child main-axis flow: document-order placement with spacing-aware
  `<=` overflow inequality (the inequality is normatively defined in
  DD-005's "Spacing interaction with overflow comparison" sub-issue;
  this DD records the contract).
- Child measure input: unbounded main-axis + DD-004-defined
  cross-axis.
- Cross-axis item alignment: centred within line.

Design quality dominates on the child-measure-input sub-issue: Option B
(slot-sized main-axis) would degenerate WrapPanel into Grid territory
and contradict the layering note. The other sub-issues follow Phase 2's
established discipline (per-kind tag, both empty-and-1-child valid,
centred alignment as the no-override default).

**Forward-compat exposure:** Option A's exposure under foreseeable
future events (see Out of scope):

- Phase 5 Grid lands with its own per-cell measure semantics. The
  per-kind tag stays additive; Grid does not reuse WrapPanel's
  `WidgetKind` slot.
- Phase 7 iteration grammar lands. Iteration generates children into
  WrapPanel; the 0-child shape (an empty iteration result) and the
  document-order placement contract both survive.
- A future per-child `align` attribute is additive — Phase 3's
  centred default does not foreclose Option C if a later phase
  surfaces a heterogeneous-alignment use case.

---
