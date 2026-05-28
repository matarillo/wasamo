### DD-M3-P5-002 — Track sizing forms (fixed + weighted star)

**Status:** Proposed

**Context:** Grid declares one track list per axis (`columns:` and
`rows:` on `Grid`, per DD-M3-P5-001 Surface A2). Phase 5 must choose
which track sizing forms are normative in
[`docs/dsl_spec.md`](../../../../docs/dsl_spec.md) §4.12. The
candidate forms are **fixed integer pixels** (`180`), **unit star**
(`*`), **weighted star** (`2*`, `3*`), and **intrinsic / `auto`**
(content-demand sized).

Star sizing is the **central novel-normative content** of Phase 5
([../requirements/framing.md §Phase 5 acceptance criteria](../requirements/framing.md#phase-5-acceptance-criteria-restated):
"second novel-normative-spec phase"). The DD therefore must commit
to a complete, deterministic star surface; partial star (unit-only)
or partial `auto` (admitted but not specified for spans) would
constrain v1.0 compatibility.

The 2026-05-28 owner alignment settled the structurally branching
sub-decision: **`auto` deferred from Phase 5 with the algorithm
slot reserved**. Phase 5 admits fixed and weighted-star tracks
only. The remaining sub-decisions (parser path, value range,
positive-integer star weights) are written below as Recommendations
and approved at ADR review.

**Sub-issues:**

- **Fixed tracks.** Integer pixel sizes carried as existing `IntLit`
  tokens; positive values only.
- **Star tracks.** Unit star (`*`) and weighted star (`2*`, `3*`)
  with positive integer weights, capped at `1024` (per-weight
  upper bound; see Recommendation below). The cap is recorded as a
  DD-M3-P5-006 invariant so the validate surface is self-contained
  rather than relying on "no practical author would write
  this".
- **Auto / intrinsic tracks.** Owner-settled deferral; the
  `TrackSize` domain type reserves the slot.
- **Track-list parser surface.** A narrow parser path for Grid
  `columns:` / `rows:` attributes that does **not** open a general
  list / collection grammar.
- **`TrackSize` domain type.** Where the parsed sizing values live
  in the IR.

**`TrackSize` domain type (consequence of DD-M3-P5-001 Option A IR
shape):**

```rust
// wasamo-ir
pub enum TrackSize {
    Fixed(i32),          // px; must satisfy 1 <= value
    Star(u32),           // weight; must satisfy 1 <= weight <= 1024
                         // (per-weight cap combined with DD-M3-P5-004's
                         // u64 star-weight-sum accumulator closes
                         // overflow at the type level — see DD-M3-P5-004
                         // for the resolve_axis algorithm)
    // Auto reserved for a future phase (Post-Phase-5 hand-off
    // item 1). Adding this variant later is additive: existing
    // .ui that does not use `auto` lowers unchanged.
}
```

`Grid` IR node carries `columns: Vec<TrackSize>` and `rows:
Vec<TrackSize>` in the Grid-specific kind payload (`KindPayload::Grid`
on `IrNode`, per DD-M3-P5-001 carrier decision **c1**) — not as
`IrProp` entries, because `IrProp.value` stays strictly
`IrLiteral`. Both `Vec`s must be non-empty (DD-M3-P5-001
minimum-shape recommendation).

**Options (fixed tracks):**

- **Option A — Existing `IntLit` token, positive-only at validate
  (recommended).** `columns: 180 1*` parses `180` as `IntLit(180)`
  and admits it as `TrackSize::Fixed(180)` after the narrow track-
  list parser path. Zero and negative fixed values are rejected
  at `wasamoc check` (DD-M3-P5-006).
  - What you gain: reuses Phase 3 / 4 IntLit plumbing unchanged;
    zero-pixel fixed track is rejected (a zero-pixel track is
    indistinguishable from an unweighted absent track and produces
    no useful layout); negative pixels are obviously malformed.
  - What you give up: nothing relative to existing precedent.

**Options (star tracks):**

- **Option A — Unit star + positive-integer weighted star (1..=1024)
  (recommended).** `*` is sugar for `1*`; weighted star tokens are
  `n*` where `n` is a positive integer in `[1, 1024]`. `0*`,
  negative weights, and weights `> 1024` are rejected at
  `wasamoc check` and `validate()` (DD-M3-P5-006). All-zero star
  sum (every star weight is `0`) cannot arise because each
  individual weight is `>= 1`; the layout-time arithmetic in
  DD-M3-P5-004 therefore has a non-zero divisor. The per-weight
  upper bound `1024`, combined with the `u64` star-weight-sum
  accumulator in DD-M3-P5-004's algorithm, closes the overflow
  invariant at the type level: the sum is bounded by
  `1024 * track_count`, and `u64` tolerates `track_count` up to
  ~`1.8 × 10^16` — well beyond any structurally feasible IR
  (allocating that many `TrackSize` values would already exceed
  any conceivable memory budget). No "no practical author would
  write this" gap remains.
  - What you gain: complete star surface (the central
    novel-normative-spec content of Phase 5); no half-baked unit-
    only star to deprecate in v1.0; deterministic measure-arrange
    in DD-M3-P5-004; the per-weight cap plus `u64` sum close the
    star-weight-sum invariant at the type level (the spec is not
    dependent on "realistic track count" assumptions).
  - What you give up: positive **integer** weights only; floating-
    point weights like `1.5*` are deferred. Authors can express
    `1.5 : 1` proportions as `3* 2*`. Ratios > 1024:1 between two
    star tracks are not expressible in Phase 5 (a future phase
    may raise or remove the cap if author demand surfaces; the
    cap exists to bound the invariant, not as a UX statement).
- Option B — Unit star only (`*`); no weighted star. Weighted
  star deferred to a later phase.
  - What you gain: smallest star surface.
  - What you give up: contradicts the framing's "complete star
    surface" obligation; weighted star is the load-bearing
    novel-normative spec content of Phase 5; deferring it would
    leave A2 underspecified for the central acceptance criterion.
- Option C — Floating-point weighted star (`1.5*`). Requires a
  new `f32` token shape inside the track-list parser.
  - What you gain: arbitrary proportions without integer
    expansion.
  - What you give up: opens a new numeric token shape inside
    Grid's narrow parser path for a use case (non-integer
    proportions) the visible-proof slice does not pressure; star
    sum becomes `f32` (precision risk under non-trivial
    proportions); v1.0 compatibility surface widens for marginal
    benefit. The integer-weight surface in Option A is forward-
    compatible (a future `1.5*` would lower to a generalized
    weight that subsumes the integer form).

**Options (`auto` / intrinsic tracks):**

- **Option A — `auto` deferred; algorithm slot reserved (owner-
  settled).** Phase 5 admits fixed + weighted star only. The
  `auto` token is rejected at `wasamoc check` with a diagnostic
  naming it as a reserved future token (not "unknown token"), so
  authors who try it get a hint that this is a deferral, not a
  typo. DD-M3-P5-004 explicitly reserves the demand-distribution
  slot before star distribution; admitting `auto` later is
  additive (extend `TrackSize` with an `Auto` variant + add the
  measure-side demand pass) and does not change Phase 5 semantics
  for fixed / star track lists that contain no `auto`.
  - What you gain: complete fixed + star semantics shipped in
    Phase 5 with full novel-normative spec content; spanning
    interaction with auto tracks (the principal complexity) is
    deferred until owner can decide one rule and ship a complete
    spec; v1.0 compatibility window stays clean.
  - What you give up: authors who want a content-sized "metadata
    column" must size it fixed or wait for the future `auto`
    phase. The framing notes this is a conservative
    *compatibility* choice rather than an *effort* choice — a
    half-specified `auto` track without spanning demand
    distribution would be worse than a clearly deferred surface.
- Option B — Admit `auto`; DD-M3-P5-004 fully specifies auto-
  track demand including spanning children.
  - What you gain: complete track-sizing surface in Phase 5.
  - What you give up: DD-M3-P5-004 grows substantially (auto-
  demand pass + auto-vs-span reconciliation rule); ship risk
  rises for the central Grid algorithm; v1.0 commits to a specific
  auto-spanning rule before owner has experienced authoring
  pressure. Framing explicitly recommended Option A.
- Option C — Admit `auto` but defer the auto-vs-span rule (allow
  `auto` only on rows with no spanning children).
  - What you gain: partial auto surface.
  - What you give up: a usage restriction without a normative
    rule is fragile; authors would discover the restriction at
    `wasamoc check` time rather than from the spec; rejected on
    surface-quality grounds.

**Options (track-list parser surface):**

- **Option A — Narrow Grid-specific parser path for `columns:` /
  `rows:` (recommended).** The DSL parser recognises a track-list
  shape only inside Grid's `columns:` / `rows:` attribute
  positions. The shape is a whitespace-separated sequence of
  `IntLit` or `n*` tokens, terminated by `;` or `\n` per the
  existing attribute-termination rule. The general DSL grammar is
  not modified; no generic list / collection grammar is opened.
  - What you gain: token-level diagnostics (e.g. "expected
    integer or star token") with accurate source location; no
    surface contamination of other attribute positions; future
    extensions (`auto`, `minmax(...)`, named lines) localise to
    Grid's parser path.
  - What you give up: nothing relative to A2's chosen surface.
- Option B — String-encoded track list (`columns: "180 1*"`).
  Track-list syntax is parsed at `wasamoc check` / runtime from
  a string literal.
  - What you gain: zero parser-grammar change.
  - What you give up: token-level diagnostics degrade (the
    parser sees one `Str` token); editor highlighting / completion
    / source location degrade; future grammar extensions are
    trapped inside a string; rejected per the framing's
    invalid-combinations check.
- Option C — General list grammar at the DSL level (`columns: [180
  1*]`). Opens a new grammar form.
  - What you gain: composable list shape reusable for other
    attributes.
  - What you give up: opens a grammar shape Phase 5 does not need
    (and that DD-M3-P5-001 / A2 explicitly avoids); large surface
    change for the smallest visible benefit.

**Comparison summary (star vs `auto`):**

| Decision | Phase 5 ships | Reason |
|---|---|---|
| Fixed tracks | Yes | Smallest necessary form; reused IntLit plumbing |
| Unit star (`*`) | Yes | Central novel-normative spec content |
| Weighted star (`n*`, `n >= 1` integer) | **Yes** | Completes star surface; required for `1:2:1` gallery and form proportions |
| Floating-point star (`1.5*`) | No | Forward-compat; integer weights can express ratios |
| `auto` / intrinsic | **No (reserved slot)** | Owner-settled deferral; spanning interaction complexity warrants its own phase |
| `minmax(min, max)` | No | Out of scope; deferred to a future phase if author demand warrants |
| Named lines | No | Out of scope (Post-Phase-5 hand-off item 2) |

**Decision (Recommendation):**

- Fixed tracks: **Option A** (existing `IntLit`, positive-only at
  validate).
- Star tracks: **Option A** (unit + positive-integer weighted
  star).
- `auto` / intrinsic: **Option A** (deferred; algorithm slot
  reserved; reject token with reserved-future diagnostic) —
  owner-settled at framing.
- Track-list parser surface: **Option A** (narrow Grid-specific
  parser path).

Phase 5 admits the following `TrackSize` token shapes at the `.ui`
boundary:

| Token | Lowers to | Validation |
|---|---|---|
| `180` (or any positive `IntLit`) | `TrackSize::Fixed(180)` | `value >= 1` at `wasamoc check` and `validate()` |
| `*` | `TrackSize::Star(1)` | `1 in [1, 1024]` (passes by construction) |
| `n*` (where `n` is an integer in `[1, 1024]`) | `TrackSize::Star(n)` | `1 <= n <= 1024` at `wasamoc check` and `validate()` |
| `0`, `-5`, `0*`, `-2*` | (rejected) | `wasamoc check` diagnostic; `validate()` `WASAMO_ERR_IR_MALFORMED` |
| `n*` with `n > 1024` (e.g. `2048*`) | (rejected) | `wasamoc check` diagnostic naming the upper bound; `validate()` `WASAMO_ERR_IR_MALFORMED` |
| `auto` | (rejected; reserved future) | `wasamoc check` diagnostic naming it reserved-future |
| `1.5`, `1.5*` | (rejected; not valid in Phase 5) | `wasamoc check` diagnostic |

**Forward-compat exposure:**

- **`auto` admission (Post-Phase-5 hand-off item 1).** Adding
  `TrackSize::Auto` is additive at the IR level. DD-M3-P5-004
  reserves the demand-distribution slot before star distribution;
  the future phase that admits `auto` must specify auto-vs-span
  demand reconciliation as the principal novel content.
- **`minmax(min, max)` and named lines (Post-Phase-5 hand-off item
  2).** Both are additive at the `TrackSize` level (`Minmax(min,
  max)`) or the track-list level (named line declarations between
  tokens). Phase 5's narrow parser path is the localised extension
  point.
- **Floating-point weights.** A future `1.5*` would generalise the
  weight type (`Star(u32)` → `Star(Rational)` or similar). The
  Phase 5 integer-weight surface is a strict subset.
- **Bindable track values (Post-Phase-5 hand-off item 3).** Phase
  5 ships constant-only `TrackSize` values. A future bindable
  `columns: {sidebar_width} 1*` requires `TrackSize` to participate
  in the binding pipeline (`TypedValue` machinery, M4+); the
  domain type is the extension point but Phase 5 does not block
  it.

**Technical risk re-evaluation:**

- **Integer-only star weight precision.** Star distribution divides
  remaining bounded `f32` space by the integer weight sum. For
  practical Grid sizes (gallery: 3-4 tracks; settings panes: under
  10), the precision risk is negligible. DD-M3-P5-004's `f32`
  prefix boundaries are deterministic.
- **Reserved-future diagnostic for `auto`.** The `wasamoc check`
  diagnostic for `auto` must distinguish it from "unknown token"
  so authors who try `auto` get a hint that this is a deferral.
  This is a diagnostic-text obligation, not a structural risk.
- **Track-list parser path scope creep.** Future grammar extensions
  (`minmax`, named lines, `auto`) localise to Grid's narrow parser
  path. The risk is that successive extensions accumulate into a
  CSS-Grid-sized grammar within `columns:` / `rows:`. Phase 5
  records this risk and defers re-evaluation to the phase that
  admits the second extension form.

**Layering with DD-M3-P5-001 / DD-M3-P5-003 / DD-M3-P5-004 /
DD-M3-P5-006:**

- DD-M3-P5-001 defines the `TrackSize` domain type's IR location
  (Grid's `columns:` / `rows:` attributes).
- DD-M3-P5-002 (this DD) defines the `TrackSize` value forms
  admitted in Phase 5.
- DD-M3-P5-003 consumes the resolved track count (`columns.len()`
  and `rows.len()`) to bound `Cell` placement and spans.
- DD-M3-P5-004 consumes `TrackSize` values to resolve track widths
  / heights against bounded parent space.
- DD-M3-P5-006 dual-gates `TrackSize` value-range invariants
  (positive fixed, positive-integer star) at `wasamoc check` and
  `validate()`.

Invalid combinations explicitly rejected by this DD in combination
with downstream DDs:

- DD-M3-P5-002 = no weighted-star surface + DD-M3-P5-004 =
  weighted-star distribution algorithm. The Recommendation Option
  A admits weighted star, so this combination does not arise.
- DD-M3-P5-002 = `auto` deferred + DD-M3-P5-004 = auto-track
  demand distribution as normative Phase 5 behaviour. The
  Recommendation Option A defers `auto` and DD-M3-P5-004 reserves
  the slot but does not implement the demand pass; this
  combination does not arise.
