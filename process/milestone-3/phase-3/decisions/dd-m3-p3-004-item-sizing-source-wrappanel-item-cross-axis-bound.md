### DD-M3-P3-004 — Item sizing source (WrapPanel item cross-axis bound)

**Status:** Accepted

**Context:**
WrapPanel measures children to determine line breaks; the child
measure constraint must come from somewhere. The Phase 3 wireframe's
88×88 thumbnail (`Box { aspect: 1:1; fill: …; Text { … } }`) has
**no intrinsic size of its own** — Phase 2 DD-M3-P2-005's aspect
projection derives the unbounded axis from the bounded axis, so the
Box needs *one* bounded axis as input. If WrapPanel passes the
parent's full cross-axis constraint through to each child, a 1:1 Box
thumbnail in an 800×600 window inherits ~600 cross-axis bound and
grows to ~600×600 — not 88×88. The DD settles where the per-item
cross-axis bound comes from. This is the load-bearing question for
the gallery sub-screen's *visible* correctness.

**Options (attribute exposure):**

Option A — Expose `item-cross-size: <i32>` (working name) as an
optional constant-only attribute (recommended)
- A new optional `i32` attribute on the WrapPanel IR node carrying
  the cross-axis bound passed to each child during measure. Name is
  orientation-neutral so a later phase that admits vertical
  orientation does not need to rename. Alternative names rejected:
  `item-cross-extent` (verbose), `item-height` (orientation-coupled
  WPF-style — rejected for the same reason DD-002 defers orientation),
  `cell-size` (Grid-flavoured — confusable), `thumbnail-size`
  (use-case-specific).

  - What you gain: An explicit, orientation-neutral knob for the
    per-item cross-axis bound. The Phase 3 gallery sub-screen sets
    `item-cross-size: 88` and gets 88×88 thumbnails from 1:1 Boxes
    deterministically. No new `IrType`, no new `IrLiteral` variant
    — `i32` plumbing already exists.
  - **Technical risk:** Low.

Option B — No item-sizing attribute on WrapPanel; require authors
to size each Box child explicitly
- WrapPanel never supplies a per-item cross-axis bound; each child
  must carry intrinsic size of its own (i.e. each Box thumbnail
  carries explicit `width: 88; height: 88`).

  - What you give up: `width` / `height` on Box are out of Phase 2
    DSL surface (Phase 2 DD-M3-P2-005 forward-looking-only). Adopting
    Option B would force opening those attributes in Phase 3 with
    no DD pre-doc'd for them — scope creep. Also: the gallery
    sub-screen would have to re-specify 88 at every thumbnail
    instead of once at the WrapPanel level, which is exactly the
    repetition pattern that motivates container-level item sizing
    in WPF / Slint / Compose.

**Options (default behaviour when `item-cross-size` is unset):**

Option (a) — Pass parent's full cross-axis constraint through to
each child (WPF / Slint precedent, recommended)
- When `item-cross-size` is unset, each child receives the parent's
  cross-axis constraint as its cross-axis measure input. A `Box {
  aspect: 1:1 }` thumbnail in a parent with `H=600` measures to
  `600 × 600` (not the wireframe's 88×88, but a defensible reading
  of "no override → no transformation").

  - What you gain: Composes naturally with sized parents. A
    WrapPanel inside a fixed-height container without
    `item-cross-size` produces children sized to the container —
    matches WPF / Slint "no override → full passthrough" intuition.
  - What you give up: An aspect-only Box thumbnail in a tall
    WrapPanel becomes huge — author-facing footgun. The
    `dsl_spec.md` chapter contains a "common pitfalls" note pointing
    aspect-only-Box authors at the attribute.
  - **Technical risk:** Low.

Option (b) — Measure with unbounded cross-axis
- When `item-cross-size` is unset, each child receives an unbounded
  cross-axis constraint. An aspect-only Box then has both axes
  unbounded and immediately hits Phase 2's
  `LayoutError::BoxAspectUnboundedBoth`.

  - What you gain: No silent "huge thumbnail" outcome — authors
    using aspect-only children without `item-cross-size` get an
    error directly.
  - What you give up: WrapPanel-in-a-sized-container with
    natural-cross-axis children (e.g. Text-only children measuring
    cross-axis from font) loses access to the container's
    cross-axis bound by default — those children measure as if the
    container were unbounded. Counter-intuitive for the common
    case.

Option (c) — `wasamoc check` requires `item-cross-size` when any
direct child uses `aspect`-only sizing
- Compile-time guard: if WrapPanel has no `item-cross-size` and at
  least one direct child is a Box with `aspect` and no other size
  source, `wasamoc check` rejects.

  - What you gain: Footgun caught at author time.
  - What you give up: `wasamoc check` must statically classify
    children by "size source" (aspect-only Box vs natural-intrinsic
    Text vs future non-Box children). The classifier scales poorly
    as the widget catalogue grows. Too strong for a general WrapPanel
    surface.

**Options (per-line cross-axis sizing rule when `item-cross-size` is set):**

Option A — Line cross-axis extent equals `item-cross-size` uniformly
(WPF-`ItemHeight`-style, recommended)
- Every line's cross-axis extent is exactly `item-cross-size`
  regardless of how much each child consumes. Smaller children
  align (per DD-001's centred default) within the line; larger
  children clip (consistent with Phase 2 Box's overflow rule).

  - What you gain: Deterministic line extent. WrapPanel's outer
    cross-axis size is `(line_count × item-cross-size) +
    (line_count − 1) × line_spacing` — directly computable. Matches
    WPF semantics on the *visible* axis: when authors set
    `ItemHeight`, lines are that tall.
  - **Technical risk:** Low.

Option B — Line cross-axis extent is the max of children's
*reported* cross-axis sizes (intrinsic-driven, Slint-style)
- The line's extent collapses to whatever the children actually
  consumed, ignoring `item-cross-size` as a line-extent setter.

  - What you give up: `item-cross-size` becomes purely a measure-
    input bound, with no effect on visible line extent when children
    measure smaller. Authors who expect "all my thumbnails are 88
    tall" by setting `item-cross-size: 88` get a smaller line when
    a thumbnail happens to report a smaller intrinsic. Surprising;
    contradicts the "uniform thumbnail grid" use case.

**Options (`wasamoc check` warning for aspect-only-Box without
`item-cross-size`):**

Option A — Ship the warning in Phase 3
- When a WrapPanel declares no `item-cross-size` and at least one
  direct child is a Box with `aspect` and no other size source,
  `wasamoc check` emits a **warning** (not error) suggesting the
  attribute. The warning is structurally cheap (it doesn't have to
  be sound across all child shapes — only the known aspect-only-Box
  footgun) and preserves Option (a)'s default.

  - What you gain: Author-time guidance for the most common
    failure mode without committing the spec to "size source
    classification".
  - **Technical risk:** Low.

Option B — Defer the warning to a later phase
- Phase 3 ships Option (a) default with no `wasamoc check`
  warning. The `dsl_spec.md` pitfall note is the only guidance.

  - What you give up: Authors learn the footgun at runtime
    (huge-thumbnail visual outcome) rather than at compile time.
    Recoverable but slower.

**Options (bindable surface):**

Option A — Constant-only in Phase 3 (recommended)
- Mirrors DD-002 / DD-003 / Phase 1 / Phase 2 discipline. No new
  per-type writer seam; `i32` literal plumbing already exists.

  - **Technical risk:** Low.

Option B — Admit bindable `item-cross-size`
- A theme-driven thumbnail size could vary at runtime.

  - What you give up: Speculative seam-registration for an
    attribute no Phase 3 sub-screen exercises reactively. Discipline
    of "build the seam in the phase that needs it" applies.

**Recommendation:** Option A across the attribute-exposure /
default-behaviour / per-line / bindable sub-issues, with the
`wasamoc check` warning shipped in Phase 3 —

- Expose `item-cross-size: <i32>` as an optional constant-only
  attribute.
- Default when unset: Option (a) — pass parent's full cross-axis
  constraint through. This matches WPF / Slint precedent and the
  principle "WrapPanel does not redefine child measure when the
  author has not asked it to". The `dsl_spec.md` chapter contains a
  "common pitfalls" note pointing aspect-only-Box authors at the
  attribute.
- Per-line cross-axis sizing rule when attribute set: uniform —
  line cross-axis extent equals `item-cross-size`.
- `wasamoc check` warning: ship in Phase 3 (Option A under the
  warning sub-issue) — narrow guard for the known aspect-only-Box
  footgun.
- Bindable surface: constant-only.

The default-behaviour Option pick (Option (a) vs (b) vs (c)) is a
load-bearing value judgement; the warning ship/defer pick is a
companion judgement. **See Owner-agreement checkpoint 2**.

The Phase 3 gallery sub-screen sets `item-cross-size: 88` explicitly,
so the WrapPanel of 1:1 Boxes produces 88×88 thumbnails matching the
wireframe (no warning triggered).

**Forward-compat exposure:** Option A's exposure under foreseeable
future events:

- A future bindable-`item-cross-size` phase admits binding for the
  attribute at that point. It can reuse the M2 string-baked
  `register_binding` path that `IrType::I32` properties currently
  dispatch to, or open a typed-`i32` evaluator/writer pair if that
  phase warrants it; no revision of Phase 3 IR shape or default
  behaviour is required.
- A future vertical-orientation phase (DD-002 deferred) does not
  need to rename — `item-cross-size` is orientation-neutral by
  design.
- A future Image-widget phase that gives children natural cross-axis
  size: the "when set, override; when unset, passthrough" semantics
  survive — Image children with natural size measure normally under
  the parent passthrough, and the `item-cross-size` override
  continues to mean "use this as the measure bound".

Option (b) (unbounded default) would have committed Phase 3 to a
counter-intuitive composition with sized parents that the future
Image-widget phase would have had to re-spec around. Option (c)
(compile-time-requires) would have committed `wasamoc check` to a
classifier that scales poorly.

---
