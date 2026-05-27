### DD-M3-P2-005 — Aspect constraint measure-arrange algorithm

**Status:** Accepted

**Context:**
When Box carries `aspect`, the measure-arrange pass computes a
resolved width × height from parent bounds. The framing nuances
m3-plan's "no novel measure-arrange algorithm" line: "no novel"
refers to the absence of a new measure-arrange *paradigm* (vs Phase
3 WrapPanel's two-stage reflow), not the absence of any algorithmic
spec content. The edge-case enumeration below is a non-trivial spec
contribution.

This DD settles Box's outer bounds (cf. the layering note above);
DD-001 settles what happens *inside* those bounds.

**Options (bounded parent — happy path):**

Option A — Inscribed: fit the aspect-constrained Box inside parent
bounds so it touches the constraining axis, leaving slack on the
other (recommended)
- Given parent `W × H` and `aspect: num:den`, resolve to
  `W' × H'` such that `W' / H' = num / den` and either `W' = W,
  H' = W * den / num` (if `W * den / num <= H`) or
  `H' = H, W' = H * num / den` (otherwise). The Box's resolved
  rectangle is the largest aspect-correct rectangle that fits inside
  the parent.

  - What you gain: Matches conventional "AspectRatio" semantics
    across XAML / SwiftUI / CSS `aspect-ratio` (when paired with
    object-fit: contain). Predictable: the Box never exceeds parent
    bounds.
  - **Technical risk:** Low. The rational pair form (DD-002 Option A)
    keeps the *ratio* exact; the projection onto the f32 parent
    bounds is computed in f32 (see numeric / rounding contract below).

**Numeric / rounding contract:**
- Parent bounds enter layout as `f32` (per `WM_SIZE`'s
  `resize_fn: Option<Box<dyn FnMut(f32, f32)>>` and
  `SizeConstraint::Fixed(f32)` in `architecture.md` §6).
- The aspect projection is evaluated as
  `H' = W * (den as f32) / (num as f32)` (and symmetrically for
  the other branch), with the comparison `W * den <= H * num`
  performed in **integer arithmetic on the rational pair extended
  by the parent's `f32` bounds rounded to the nearest `i64` ×
  `den`/`num`** — i.e. the branch selection never depends on f32
  round-off. Concretely: select inscribed branch by comparing
  `(W as f64) * (den as f64)` vs `(H as f64) * (num as f64)`; once
  the branch is chosen, compute the derived axis in `f32`.
- No pixel-snapping in Phase 2. The resolved rectangle stays in
  the `f32` layout coordinate space; rasterisation / DPI hinting
  remains an open question carried in
  [architecture.md §Open questions](../architecture.md#open-questions-1).
  DPI scaling localisation does not alter the DD-005 algorithm —
  it would re-define what `f32` parent bounds *are*, not how the
  projection is computed from them.

Option B — Circumscribed: cover parent bounds so the Box overflows
the non-constraining axis
- The opposite of Option A: the smaller dimension of the parent is
  matched, the larger dimension is overflowed.

  - What you give up: Box paints outside its parent's rectangle.
    Phase 3 WrapPanel + Phase 4 ScrollView all assume children sit
    within parent bounds; circumscribed Box would break their
    layout assumptions.

Option C — Major-axis driven (parent's longer axis is matched)
- The longer dimension of the parent is matched; the shorter is
  derived.

  - What you give up: Mixes Option A and Option B behaviours
    depending on parent aspect; un-predictable from the spec.

**Options (unbounded parent on one axis):**

Option A — Bounded-axis-wins: derive the unbounded axis from the
bounded × aspect (recommended)
- E.g. parent provides finite width `W` but no height constraint
  (the layout context is intrinsic-sizing on the vertical axis):
  Box resolves to `W × (W * den / num)`. Symmetric for the inverse.

  - What you gain: The Box has a defined intrinsic size in
    intrinsic-sizing contexts (Phase 3 WrapPanel of Boxes, Phase 4
    ScrollView inner). Predictable.
  - **Technical risk:** Low.

Option B — Both axes resolve to spec-defined intrinsic 0
- The Box collapses to zero on the unbounded axis.

  - What you give up: A WrapPanel of zero-tall Boxes paints nothing;
    no realistic use case.

**Options (unbounded parent on both axes):**

Option A — Layout-time runtime error (recommended)
- `wasamoc check` cannot detect this (it depends on runtime
  layout context); `wasamo-runtime`'s layout pass emits an error
  when it encounters a Box with `aspect` set whose parent provides
  no bounded axis. Error surfaces as a runtime diagnostic with
  the Box's IR location. This is **not** an IR-load-time error —
  the condition is only knowable once the layout pass starts with a
  concrete parent constraint, which is downstream of `ir_loader`.

  - What you gain: Honest behaviour. NaN / silent-zero outcomes
    are excluded structurally. Practical: a top-level Box with
    `aspect` set inside a window with no provided size is a real
    DSL author mistake; an error tells them clearly.
  - What you give up: Phase 2 must extend the layout-time error
    surface. The mechanism already exists for other layout errors
    in `wasamo-runtime`; extending it is small.
  - **Technical risk:** Low.

Option B — Spec-defined intrinsic size (e.g. 0×0)
- The Box collapses; rendering proceeds without complaint.

  - What you give up: Silent layout dropouts are bug-magnets. A
    user's "my Box isn't showing" question becomes a layout-context
    detective story.

Option C — Take the child's intrinsic size as the bounded axis if
children exist
- Under the layering note, this would mean child intrinsic
  size *seeds* Box's outer bounds in this specific edge case.

  - What you give up: Introduces a child → parent intrinsic-size
    flow purely for an exotic edge case. Contradicts the layering
    note's "aspect set → child intrinsic does not grow Box" rule
    in the only direction that would still leave the rule mostly
    true. Net spec complexity gain > payoff.

**Options (conflict with explicit width/height):**

> **Phase 2 scope note.** `width` / `height` are **not** in the
> M3-Phase 2 DSL surface — no widget (Box included) currently
> accepts an explicit `width:` or `height:` attribute, and the
> grammar / `dsl_spec.md` introduces neither in Phase 2. This
> sub-issue is therefore **forward-looking**: it settles what the
> *first phase to introduce `width` / `height` on Box* must do,
> so that future phase inherits the rule rather than re-deriving
> it. Phase 2 ships no `wasamoc check` warning code for this
> conflict (there is no surface to detect a conflict on); the
> rule lands as spec text in the Box chapter with a "see also:
> applies when width/height become surfaced" cross-reference.

Option A — Explicit dimensions win; aspect becomes informational
(recommended)
- If Box carries `width: ...` and / or `height: ...` alongside
  `aspect: ...`, the explicit dimensions are used and the aspect
  is ignored (with a `wasamoc check` warning, not error,
  recommending the redundant attribute be removed).

  - What you gain: Author intent is unambiguous when explicit
    dimensions are present. Aspect remains as documentation /
    forward-compatibility for the case where one of width / height
    is removed.
  - What you give up: A warning surface in `wasamoc check` —
    landed by the phase that surfaces `width` / `height`, not by
    Phase 2 (per the scope note above).
  - **Technical risk:** Low.

Option B — Aspect overrides explicit
- The aspect-derived rectangle wins; explicit dimensions ignored.

  - What you give up: Counter-intuitive — author wrote `width: 100`
    expecting it to apply.

Option C — Error if both present
- `wasamoc check` rejects.

  - What you give up: Forces the author to choose at every site,
    even when the aspect is the redundant one (e.g. `Box {
    width: 16; height: 9; aspect: 16:9 }`).

**Options (child intrinsic size participation when `aspect` is
absent):**

Option A — Box shrink-to-fit child's intrinsic size; fall back to
parent bounds when no children (recommended)
- Empty Box: matches parent bounds. Single-child Box (per DD-001
  Option A): matches the child's intrinsic measure.
- **Unbounded-parent consistency.** "Matches parent bounds" assumes
  a bounded parent. When `aspect` is absent **and** the parent
  provides no bounded axis on *both* axes, the empty-Box (and the
  single-child-Box whose child also yields no intrinsic size, e.g.
  a `Text` whose own measure is unbounded — not a Phase 2 concern,
  but cross-referenced for completeness) has nothing to derive size
  from. This case follows the **same layout-time runtime error**
  as the aspect-set unbounded-both-axes branch (Option A under
  "Options (unbounded parent on both axes)"). The diagnostic
  wording differs only in that it names the missing input as
  *"neither aspect nor parent bounds"* rather than *"aspect with
  no bounded parent axis"*; both share the "Box has no extent to
  resolve" structural error class. Spec-defined intrinsic 0×0 is
  rejected here for the same reason it is rejected for the
  aspect-set case: silent zero-size dropouts are bug-magnets, and
  a `Box { fill: #ccc }` scrim placed in a fully-unbounded context
  is an author error worth surfacing.
- When the parent is unbounded on **one** axis only (the
  intrinsic-sizing case), an empty Box collapses to zero on the
  unbounded axis and matches the bounded axis on the other — this
  is the natural reading of "matches parent bounds" with a
  zero-extent unbounded axis, not a runtime error. A scrim-only
  Box in this context paints a zero-thickness strip; the author
  must add `aspect` or wrap the Box in a sized parent to get
  visible extent.

  - **Technical risk:** Low.

Option B — Box always expands to parent bounds
- Empty and child-bearing both fill the parent rectangle.

  - What you give up: A `Box { fill: #ccc; Text { text: "label" } }`
    with no aspect would expand to its parent, painting `#ccc`
    everywhere, which is a footgun for the "label-with-background"
    pattern.

**Options (aspect value validity):**

Option A — `wasamoc check` rejects zero on either side
(recommended)
- The literal form (DD-002 Option A: unsigned integer pair) makes
  zero the only reachable invalid side via author error (e.g.
  `Box { aspect: 0:9 }`); negative sides do not parse as `Ratio`
  literals at all (a leading `-` parses as unary minus, which does
  not form the `Ratio` token). Compile-time rejection of zero per
  Phase 1's T14 discipline ("bad surface forms fail at the
  source-level diagnostic gate"); runtime cannot see invalid
  ratios.

  - **Technical risk:** Low.

Option B — Runtime fallback to spec-defined default
- Bad ratio → Box renders as 1:1 (or similar).

  - What you give up: Silent fallback obscures the author error
    until visual inspection.

**Recommendation:** Option A for every sub-issue —

- Bounded parent: inscribed (fit smaller); projection computed
  per the numeric / rounding contract under Option A above (integer
  branch selection, `f32` derived axis, no pixel-snapping in
  Phase 2).
- Unbounded on one axis: bounded-axis-wins.
- Unbounded on both axes: layout-time runtime error — applies
  symmetrically to the no-aspect empty-Box case (a Box with neither
  `aspect` nor bounded parent extent is the same structural error).
- Width/height conflict: explicit wins; aspect informational + warning.
  Phase-2-scope-deferred: `width` / `height` are not in the M3-Phase 2
  DSL surface, so the rule lands as spec text in Phase 2 and the
  `wasamoc check` warning is implemented by the phase that surfaces
  these attributes.
- Child intrinsic (no aspect): shrink-to-fit; fall back to parent
  bounds when no children, with the unbounded-both-axes runtime
  error noted above.
- Aspect value validity: `wasamoc check` rejects zero on either
  side; negative sides are not ratio literals.

The algorithm has no novel paradigm but the spec content is real:
the edge cases above are written into `docs/dsl_spec.md` so Phase 3
WrapPanel and Phase 4 ScrollView do not re-derive them. Symmetric
with Phase 1's T14 discipline: bad surface forms (zero / negative
ratio) fail at the source-level diagnostic gate, not at runtime.

**Forward-compat exposure:** Bound by the layering note (DD-001's
no-grow-Box rule), the recommendation is dual-compatible with both
of the foreseeable future events:

- Phase 3 WrapPanel inherits a Box that has a defined intrinsic
  size in intrinsic-sizing contexts (unbounded-axis-wins clause).
  WrapPanel's two-stage reflow gets a stable input.
- Phase 4 ScrollView inherits a Box that clips its child (DD-001)
  and stays inside parent bounds (Option A inscribed) — ScrollView's
  separate clip surface is purely the *scrollable viewport*, no
  conflict.

A future "image widget" landing as a Box child (per DD-006) does
not pressure this algorithm: the child measure pass (DD-001) hands
the resolved Box bounds to whatever widget the child is.

---
