### DD-M3-P5-004 — Track-resolution algorithm (novel normative star sizing)

**Status:** Proposed

**Context:** This DD introduces the **central novel-normative spec
content** of Phase 5. Grid is M3's *second* novel-normative-spec
phase (Phase 3 WrapPanel was the first, and lighter — line-formation
is a layout-time algorithm but does not carry a track-resolution
abstraction). Grid's deterministic track-resolution algorithm —
fixed-first then weighted-star distribution over `f32` prefix
boundaries, with a Grid-specific unbounded-axis error — is the
spec content the dsl_spec.md §4.12 chapter (Moment 1) must commit to
in normative form.

The 2026-05-28 owner alignment settled the **structurally branching
sub-decision**: when star tracks meet an unbounded parent axis,
Grid raises a **Grid-specific unbounded-star error** (Flutter-style;
consistent with the Phase 4 ScrollView unbounded-axis precedent),
not a fallback to zero-minimum intrinsic behaviour. The remaining
sub-decisions (algorithm shape, rounding contract, spanning
reconciliation, `auto` slot reservation) are written below as
Recommendations and approved at ADR review.

The algorithm is structurally simpler than CSS Grid's full
track-sizing algorithm because Phase 5 admits only fixed and
weighted-star forms (no `auto`, no `minmax`, no content-based
sizing). The novel content is the **per-axis fixed-first / star-
distribute pass over `f32` prefix boundaries** plus the
**unbounded-star error** semantics, with explicit reservation for a
future `auto` demand-distribution pass.

**Sub-issues:**

- **Axis independence.** Rows and columns use the same track
  resolution algorithm over their respective dimensions.
- **Fixed-first / star-distribute pass.** Fixed tracks consume
  definite parent space first; remaining bounded space is divided
  among star tracks by positive integer weight.
- **Rounding contract.** Track resolution works in `f32` layout
  space (matching Phase 2 / 3 / 4 convention); no integer pixel
  snap; prefix boundaries are deterministic.
- **Spanning reconciliation.** Spanning `Cell` rectangles are
  measured against the combined resolved span; oversized spanning
  children overflow per DD-M3-P5-005.
- **Unbounded-parent branch.** Star tracks meeting an unbounded
  parent axis fire `LayoutError::GridUnboundedStarAxis`.
- **Negative remaining space.** When fixed tracks alone exceed
  parent bound, star tracks resolve to zero; overflow per
  DD-M3-P5-005.
- **`auto` algorithm slot.** Reserved before star distribution
  (does not run in Phase 5).

**Algorithm (per axis; rows and columns symmetric):**

Inputs:

- `tracks: &[TrackSize]` — the resolved `Vec<TrackSize>` from
  DD-M3-P5-001 (non-empty per DD-M3-P5-001 minimum-shape
  recommendation; positive fixed and star-weight `>= 1` per
  DD-M3-P5-002 validation).
- `axis_bound: AxisBound` — either `Bounded(f32)` (the parent's
  available space on this axis) or `Unbounded`.

Algorithm:

```
function resolve_axis(tracks, axis_bound) -> Result<Vec<f32>, LayoutError>:
    let fixed_sum: f32 = sum of TrackSize::Fixed(px) as f32 over fixed tracks
    let star_weight_sum: u64 = sum of (w as u64)
                               for TrackSize::Star(w) over star tracks
                               // u64 sum closes overflow at the spec level:
                               // per-weight cap is 1024 (DD-M3-P5-002), and
                               // u64::MAX / 1024 ≈ 1.8e16, which exceeds any
                               // structurally feasible track count (each
                               // TrackSize occupies memory).
    let has_star: bool = star_weight_sum > 0

    if has_star and axis_bound is Unbounded:
        return Err(LayoutError::GridUnboundedStarAxis)

    // Phase 5 auto pass reservation: no-op.
    // A future phase admits TrackSize::Auto and inserts a demand
    // pass here that grows auto tracks to fit content. The pass
    // must execute BEFORE star distribution so star tracks divide
    // the space that remains after fixed + auto consumption.

    let bound: f32 = match axis_bound:
        Bounded(b) => b
        Unbounded => fixed_sum     // no star track exists; the axis
                                   // resolves to the fixed sum

    let remaining_after_fixed: f32 = max(0.0, bound - fixed_sum)

    let resolved: Vec<f32> = tracks.map(|t| match t:
        Fixed(px) => px as f32
        Star(weight) => remaining_after_fixed
                        * (weight as f32 / star_weight_sum as f32)
                        // u64 -> f32 cast: f32 represents integers
                        // exactly up to 2^24 (~1.67e7). Practical
                        // star-weight sums sit in the 10^2 – 10^5
                        // range, well within that. Larger sums
                        // (>= 2^24) lose precision in the cast but
                        // the proportional division still produces a
                        // deterministic distribution; see Technical
                        // risk re-evaluation below.
    )

    Ok(resolved)
```

Prefix boundaries (consumed by DD-M3-P5-005 arrange and by
DD-M3-P5-003 spanning reconciliation):

```
boundary[0] = 0.0
boundary[n] = boundary[n - 1] + resolved[n - 1]
boundary[tracks.len()] = sum of resolved // total resolved track extent
                                         // (NOT Grid's outer Visual rect; see
                                         // "Grid outer rect" below)
```

Both rows and columns invoke `resolve_axis` independently.

**Grid outer rect (consequence):** The track-resolution algorithm's
`boundary[tracks.len()]` is the **total resolved track extent**, not
the Grid's outer Visual rect. Per the established M3 layout
precedent (Phase 3 WrapPanel: "outer main-axis size does not grow
to accommodate oversized children", per DD-M3-P3-005; Phase 4
ScrollView: "ScrollView outer size = viewport size, regardless of
content size", per DD-M3-P4-005), Grid's outer rect on a
**bounded** axis equals the parent's allocation on that axis. On
an **unbounded** axis (only reachable with no star tracks per the
unbounded-star branch above), Grid's outer rect equals
`fixed_sum` (the natural track-resolved extent). Cell rectangles
use the prefix boundaries computed above and may extend past
Grid's outer rect when `fixed_sum > bound` (the negative-remaining-
space case below); those rectangles overflow Grid's outer rect and
are clipped by DD-M3-P5-005's outer-bounds clip.

| Axis bound | Grid outer extent | Cell rectangles relative to outer rect |
|---|---|---|
| `Bounded(b)` with star tracks | `b` (parent allocation) | Sum of resolved = `b`; Cells fit exactly |
| `Bounded(b)`, fixed only, `fixed_sum <= b` | `b` (parent allocation) | Sum of resolved = `fixed_sum <= b`; trailing space inside Grid |
| `Bounded(b)`, fixed only, `fixed_sum > b` | `b` (parent allocation) | Sum of resolved = `fixed_sum > b`; rightmost Cells overflow, clipped per DD-M3-P5-005 |
| `Bounded(b)`, mixed fixed + star, `fixed_sum > b` | `b` (parent allocation) | Star tracks resolve to 0; sum of resolved = `fixed_sum > b`; rightmost Cells overflow, clipped per DD-M3-P5-005 |
| `Unbounded` (no star tracks; star + unbounded errors above) | `fixed_sum` | Sum of resolved = `fixed_sum`; Cells fit exactly |

**Spanning reconciliation (consumes per-axis resolution):**

A `Cell` with `(row, column, row-span, column-span)` resolves to
the rectangle:

```
left   = column_boundary[column]
right  = column_boundary[column + column-span]
top    = row_boundary[row]
bottom = row_boundary[row + row-span]
```

The cell rectangle is `(left, top, right - left, bottom - top)`.
Spanning `Cell`s are measured against the combined resolved span
extent; the spanned tracks are **not** grown to accommodate a
larger child (no `auto`-like demand back-propagation in Phase 5).
Oversized spanning children overflow per DD-M3-P5-005 paint-overflow
rule.

**Options (overall algorithm):**

- **Option A — Fixed-first / star-distribute over `f32` prefix
  boundaries with `LayoutError::GridUnboundedStarAxis` for
  unbounded-star (recommended; owner-settled at framing).**
  - What you gain: deterministic single-pass per axis; matches
    the DD-M3-P5-002 admitted forms exactly; Flutter-style
    unbounded-axis error is consistent with Phase 4 ScrollView
    precedent; the spec content is small enough to ship as the
    §4.12 chapter at Moment 1 without implementation feedback;
    spanning reconciliation is one rectangle calculation per
    `Cell`; `auto` slot is reserved cleanly for a future phase.
  - What you give up: oversized spanning children overflow rather
    than growing their spanned tracks (a future `auto` admission
    is the channel for grow-to-content semantics, not Phase 5).
- Option B — Star fallback to zero-minimum intrinsic on unbounded
  parent axis. `LayoutError::GridUnboundedStarAxis` not raised;
  star tracks resolve to `0` if `axis_bound` is unbounded.
  - What you gain: never errors at layout time.
  - What you give up: silently produces a degenerate Grid (all
    star tracks collapsed to zero) that an author cannot
    distinguish from "Grid received zero space"; contradicts the
    Phase 4 ScrollView precedent for unbounded-axis structural
    rejection; owner-rejected at framing.
- Option C — Two-pass algorithm with content-demand back-prop into
  star tracks (CSS Grid-like spanning content distribution).
  - What you gain: oversized spanning children would not
    overflow.
  - What you give up: requires `auto`-like demand-distribution
    rules even in the all-star case; substantially larger
    novel-normative spec content; framing recorded this as out of
    Phase 5 scope (Phase 5 defers `auto` and its demand
    machinery).

**Options (rounding contract):**

- **Option A — `f32` prefix boundaries; no integer pixel snap
  (recommended).** Track resolution operates in `f32` layout
  space; prefix boundaries are deterministic `f32` cumulative
  sums; any final device-pixel snapping is a renderer / platform
  concern, not a Grid algorithm step.
  - What you gain: matches Phase 2 / 3 / 4 layout-engine
    convention (`f32` layout-engine internals; `i32` attribute
    literals promoted to `f32` at comparison); deterministic
    arithmetic; no integer-pixel-snap rounding bias.
  - What you give up: the resolved final cell boundary may not
    land on an integer device pixel; this is the same situation as
    every other layout primitive in M3 and is handled by the
    renderer's existing snapping convention.
- Option B — Integer pixel snap at each boundary. Each prefix
  boundary rounds to the nearest integer pixel.
  - What you gain: pixel-aligned boundaries.
  - What you give up: introduces rounding semantics Phase 5 has
    no acceptance requirement for; cumulative rounding error
    becomes visible at the final boundary (the sum of rounded
    track widths may not equal the rounded Grid outer extent);
    rejected on uniformity grounds with Phase 2 / 3 / 4.

**Options (unbounded-parent branch):**

- **Option A — Raise `LayoutError::GridUnboundedStarAxis` if any
  star track exists and `axis_bound` is `Unbounded` (recommended;
  owner-settled).** Flutter-style; consistent with Phase 4's
  `LayoutError::ScrollViewUnboundedAxis`.
  - What you gain: errors are local to layout pass; structural
    "this Grid is in an environment that cannot provide finite
    space for star tracks" message is clear; matches Phase 4
    precedent for structurally-meaningless layout shape;
    composition with ScrollView (Grid inside ScrollView's
    unbounded scroll-axis) is well-defined as a deterministic
    error (rather than a silent zero-collapse).
  - What you give up: nothing; the alternative would be silent
    degeneracy.
- Option B — Zero-minimum fallback (Option B above). Owner-
  rejected at framing.

**Options (negative remaining space):**

- **Option A — Star tracks resolve to zero when fixed sum exceeds
  bound (recommended).** When `bound - fixed_sum <= 0`, the
  `remaining_after_fixed` clamps to `0.0` and every star track
  resolves to width `0`. The fixed tracks retain their declared
  size in the prefix boundaries (so Cell rectangles can be
  computed deterministically). **Grid's outer rect on this axis
  remains the parent allocation `bound`** — Grid does not grow to
  accommodate oversized fixed tracks (Phase 3 WrapPanel / Phase 4
  ScrollView precedent; see "Grid outer rect" above). Cell
  rectangles whose prefix boundaries extend past `bound` overflow
  Grid's outer rect and are clipped at the outer-bounds clip per
  DD-M3-P5-005.
  - What you gain: deterministic behaviour for an over-tight
    parent allocation; fixed tracks are honoured in the prefix
    boundary (the author explicitly requested fixed pixels);
    star tracks degenerate rather than negative-sizing; Grid's
    outer rect stays inside the parent's slot so sibling layout
    is never disturbed by an oversized Grid.
  - What you give up: Cell rectangles past `bound` are clipped by
    DD-M3-P5-005's outer-bounds clip — the author sees truncated
    paint, not an error. Authors who need fixed-track-driven Grid
    sizing must size the parent's allocation accordingly; Grid
    does not grow its parent.
- Option B — Reject at layout (raise
  `LayoutError::GridFixedExceedsBound`). Rejected because the
  parent allocation may legitimately shrink due to window resize
  or sibling growth; a layout-time error would make Grid fragile
  under runtime resize.
- Option C — Shrink fixed tracks proportionally. Rejected because
  fixed pixels lose their meaning under proportional shrinkage.

**Options (`auto` slot reservation):**

- **Option A — Reserve the demand-distribution pass position
  before star distribution; do not run it in Phase 5 (recommended;
  consistent with DD-M3-P5-002 owner-settled deferral).** The
  algorithm contains an explicit comment / structural placeholder
  documenting where the future demand pass would execute (between
  fixed-sum computation and star distribution). Phase 5 ships no
  `auto`-handling code; the placeholder is purely documentation.
  - What you gain: future `auto` admission is additive at this
    documented position; no implicit assumption about where the
    `auto` pass lands; the algorithm shape in the dsl_spec.md
    §4.12 chapter mirrors the future shape (Phase 5 spec text
    reads "fixed → reserved auto slot (no-op in Phase 5) → star").
  - What you give up: nothing.

**Decision (Recommendation):**

- Overall algorithm: **Option A** (fixed-first / star-distribute;
  `f32` prefix boundaries).
- Rounding: **Option A** (`f32`; no pixel snap).
- Unbounded-parent branch: **Option A**
  (`LayoutError::GridUnboundedStarAxis`) — owner-settled at
  framing.
- Negative remaining: **Option A** (star tracks resolve to zero;
  overflow per DD-M3-P5-005).
- `auto` slot: **Option A** (reserved before star; no-op in Phase
  5).

**LayoutError surface (consequence):**

A new variant is added to `wasamo-runtime`'s `LayoutError` enum:

```rust
pub enum LayoutError {
    // ...
    BoxAspectUnboundedBoth,             // Phase 2
    BoxNoExtent,                        // Phase 2
    ScrollViewUnboundedAxis,            // Phase 4
    GridUnboundedStarAxis,              // Phase 5 (this DD)
}
```

`LayoutError::GridUnboundedStarAxis` is internal-only; no
`WASAMO_LAYOUT_ERROR_*` ABI tag is added in Phase 5 (no host
currently observes layout-error variants meaningfully). The
variant exists to gate the runtime layout pass and to surface in
pure-logic tests.

**Forward-compat exposure:**

- **`auto` admission (Post-Phase-5 hand-off item 1).** The
  reserved slot in the algorithm is the documented extension
  point. Admitting `auto` requires:
  - extending `TrackSize` with an `Auto` variant (DD-M3-P5-002
    forward-compat);
  - inserting a measure-side demand pass at the reserved slot
    that grows auto tracks to fit content;
  - specifying auto-vs-span demand reconciliation (a
    `Cell` with `column-span: 3` spanning two fixed tracks and one
    auto track distributes content demand only to the auto
    portion).
  Phase 5 does not commit to a specific auto-spanning rule; the
  future phase that admits `auto` owns the rule.
- **`minmax(min, max)` (Post-Phase-5 hand-off item 2).**
  Additive at the `TrackSize` level. The track-resolution
  algorithm would gain a clamp step after the per-track
  resolution.
- **Floating-point weighted star.** Additive at the weight type
  level (`Star(u32)` → `Star(Rational)` or `Star(f32)`); the
  prefix-boundary arithmetic is already `f32`-based.
- **Per-axis bidirectional spacing (gap).** A future
  `column-gap: <i32>` / `row-gap: <i32>` would subtract gap
  contribution from `remaining_after_fixed` before star
  distribution.
- **Bindable track values.** When `TrackSize` participates in
  bindings, `resolve_axis` runs once per layout pass with the
  current resolved `TrackSize` values; the algorithm shape is
  unchanged.

**Technical risk re-evaluation:**

- **Star arithmetic precision and overflow.** Star distribution
  divides `f32` remaining space by the star-weight sum. The
  per-weight cap `[1, 1024]` (DD-M3-P5-002 / DD-M3-P5-006) bounds
  the per-axis weight sum at `1024 * track_count`. The sum is
  accumulated in `u64`, which tolerates `track_count` up to
  ~`1.8 × 10^16` before overflow — well beyond any structurally
  feasible IR (each `TrackSize` allocates memory; a track count
  approaching `2^32` would already require gigabytes of IR
  storage, and the `u64` headroom is many orders of magnitude
  beyond that). The Phase 5 spec therefore closes overflow at
  the type level rather than at a "realistic input" assumption.
  `f32` precision: the cast `star_weight_sum: u64 -> f32` is
  exact up to `2^24` (~16.7M); practical sums sit in the `10^2`
  – `10^5` range, far below the precision boundary, so
  `weight as f32 / star_weight_sum as f32` is essentially exact
  for any realistic Grid.
- **Prefix-boundary determinism.** Cumulative `f32` sums are
  deterministic for fixed inputs (IEEE 754 addition is
  deterministic per-thread). Cross-thread or cross-platform
  determinism is not Phase 5's concern; layout runs single-
  threaded.
- **Unbounded-star error composition with ScrollView.** A Grid
  with star tracks inside a `ScrollView` whose scroll axis matches
  the star direction would fire
  `LayoutError::GridUnboundedStarAxis`. This composition is
  structurally meaningless (a star track in an unbounded measure
  context has no defined size); the error is the intended
  outcome. Phase 5 documents the composition but does not test it
  as an integration fixture (FD-C verification closure scopes the
  Grid-rooted and VStack(Grid)-rooted shapes; nested ScrollView /
  Grid composition is out of Phase 5 scope).
- **Negative remaining space at the boundary.** A Grid whose fixed
  tracks exactly equal the parent bound has zero remaining for
  star tracks; star tracks resolve to width `0`. This is the
  boundary case, not a fault.
- **Auto-slot reservation as documentation, not code.** The
  reserved slot is a normative-spec position in the §4.12 chapter
  ("After fixed tracks consume definite space, the demand pass for
  intrinsic / `auto` tracks would execute here in a future phase
  that admits `auto`."). Phase 5 ships no auto-handling code; the
  reservation is purely future-shape documentation.

**Layering with DD-M3-P5-001 / DD-M3-P5-002 / DD-M3-P5-003 /
DD-M3-P5-005 / DD-M3-P5-006:**

- DD-M3-P5-004 consumes:
  - `Vec<TrackSize>` per axis from DD-M3-P5-002 (validated to
    contain only Fixed and Star variants by DD-M3-P5-006).
  - `Cell` rectangles `(row, column, row-span, column-span)` from
    DD-M3-P5-003 (validated to fit within declared track count by
    DD-M3-P5-006).
  - Parent `AxisBound` from the existing layout-engine
    measure-arrange interface.
- DD-M3-P5-004 produces:
  - Per-axis `Vec<f32>` track widths / heights.
  - Per-`Cell` resolved rectangles in Grid-local coordinates.
  - `Result<.., LayoutError::GridUnboundedStarAxis>` on the
    unbounded-star branch.
- DD-M3-P5-005 consumes the per-`Cell` resolved rectangle to
  arrange the content widget within the cell (with alignment) and
  to apply the Grid outer-bounds clip.

Invalid combinations explicitly rejected by this DD:

- DD-M3-P5-002 = no weighted-star surface + DD-M3-P5-004 =
  weighted-star algorithm. Does not arise: DD-M3-P5-002 admits
  weighted star.
- DD-M3-P5-002 = `auto` deferred + DD-M3-P5-004 = auto demand pass
  as normative Phase 5 behaviour. Does not arise: this DD reserves
  the slot but does not run the pass in Phase 5.

**Spec content seed (Moment 1 §4.12 draft):**

The DD-M3-P5-004 sub-issues map 1:1 to the §4.12 chapter
algorithm section:

1. Grid is a 2D layout primitive with one track list per axis
   (anchors to DD-M3-P5-001).
2. Track sizing forms: fixed integer pixels + weighted star;
   `auto` is reserved for a future phase (anchors to DD-M3-P5-002).
3. Per-axis track resolution: fixed tracks consume definite space
   first; remaining bounded space divides among star tracks by
   positive integer weight.
4. Reserved `auto` demand-distribution pass position (no-op in
   Phase 5).
5. `f32` prefix boundaries; no integer pixel snap (anchors to
   Phase 2 / 3 / 4 layout-engine convention).
6. Cell rectangle resolution: each `Cell` occupies the rectangle
   bounded by `column_boundary[column]` /
   `column_boundary[column + column-span]` (and analogously for
   row).
7. Spanning children are measured against the combined resolved
   span; oversized children overflow per §4.12 overflow section
   (anchors to DD-M3-P5-005).
8. Negative remaining space: star tracks resolve to zero;
   overflow per overflow section.
9. Unbounded star-axis parent: layout returns
   `LayoutError::GridUnboundedStarAxis` (anchors to this DD).
10. Mental-model anchor + ecosystem-contrast subsection (anchors
    to FD-K).
