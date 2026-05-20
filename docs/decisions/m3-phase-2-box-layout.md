# M3-Phase 2 — Box layout primitive: Architecture Decisions

**Phase:** M3-Phase 2 (Box layout primitive)
**Date:** 2026-05-20
**Status:** Proposed

## Context

M3 acceptance criterion **A6** (see
[ROADMAP.md M3](../../ROADMAP.md#m3-dsl-surface),
[m3-plan.md §Acceptance criteria](../plans/m3-plan.md#acceptance-criteria)):

> Box layout primitive (0+ child container; `aspect: <ratio>` attribute
> subsumes a standalone AspectRatio; minimal `fill: <color>` attribute
> for scrim use). Image-widget deferral is carried by Box + Text-child
> placeholders.

The pre-doc framing for this phase was aligned with the owner on
2026-05-20 and is recorded in
[docs/notes/m3-phase-2/m3-phase-2-pre-doc-framing.md](../notes/m3-phase-2/m3-phase-2-pre-doc-framing.md).
That framing fixed the 6-DD slate carried below, the visible-proof
location (framing decision F — seed `examples/gallery/` +
`examples/gallery-rust/`), the verification-strategy menu picks
(framing decision C), the `cargo fmt` discipline amendment (framing
decision E), and the two upstream-document-revision moments (framing
decision D — Moment 1 design-spec draft at ADR-Accepted commit;
Moment 2 implementation re-sync at phase close).

The M2/M3-Phase 1 end-state shape that this phase extends without
breaking:

- `wasamo-ir` ([wasamo-ir/src/lib.rs](../../wasamo-ir/src/lib.rs)):
  `IrType` is `I32 | Str | Bool`; `IrLiteral` is `Int | Str | Ident |
  Bool`. `HandlerExpr` uses the unified-but-type-suffixed pattern
  (`IntLit` / `StrLit` / `BoolLit` / `PropRead` / `StrPropRead` /
  `BoolPropRead`). Adding new primitive types follows the same
  type-suffix discipline (DD-M2-P6-003 / DD-M3-P1-003).
- `wasamo-runtime` widget catalog
  ([wasamo-runtime/src/widget.rs](../../wasamo-runtime/src/widget.rs)):
  `Rectangle | VStack | HStack | Text | Button`; `PropertyValue` enum
  is `I32(i32) | String(String) | Bool(bool)`. Per-widget per-attribute
  `PROP_*` u32 IDs; `resolve_prop_key` returns `(PropertyKey, IrType)`
  (DD-M3-P1-009).
- Binding pipeline
  ([wasamo-runtime/src/handler.rs](../../wasamo-runtime/src/handler.rs),
  [wasamo-runtime/src/ir_loader.rs](../../wasamo-runtime/src/ir_loader.rs)):
  per-type binding evaluator + per-type widget writer, dispatched at
  `ir_loader::build_node` by the property's `IrType` (DD-M3-P1-007).
  The reactive engine itself remains type-agnostic. F5
  (`TypedValue` deferral) is held in force by this seam pattern.
- `wasamoc` ([wasamoc/src/check.rs](../../wasamoc/src/check.rs)):
  state-name → declared-type table; identifier resolution lowers to
  typed `*PropRead` variants; `bind` LHS / RHS type pairings are
  diagnosed at compile time (DD-M3-P1-010). `TypeName::Float` already
  exists in the AST but has no IR / runtime mirror.

This ADR is framed against A6 and the M2/M3-Phase 1 type-suffix
pattern. It does **not** re-open F5 (`TypedValue` deferral) — adding
new constant-only literal forms and (where admitted) new per-type
bindable seams is the additive path proven in Phase 1.

The acceptance lens for this phase: A6 is satisfied when (i) `.ui`
declares `Box { aspect: <ratio>; fill: <color>; <child> }` and the
shared crates lower → load → render it with the right rectangle, (ii)
the placeholder pattern is canonicalized in `docs/dsl_spec.md` so
Phase 3 (WrapPanel) and Phase 6 (ZStack) cite rather than redefine
it, and (iii) `examples/gallery/` + `examples/gallery-rust/` are
seeded with the Box sub-screen as the visible proof. Per A11, all
sides advance together by phase close.

### Layering note (DD-001 ⇄ DD-005)

The two DDs that govern Box's size and child layout are **layered**,
not co-equal. DD-005 settles Box's **outer / resolved bounds** (the
rectangle Box occupies in its parent's coordinate space). DD-001
then settles **what happens inside those bounds** (child measure,
alignment, overflow). The dependency direction is fixed:

- DD-005 resolves Box's outer bounds **without** considering child
  intrinsic size **when `aspect` is set**. Aspect-derived bounds win;
  children do not get to grow the aspect-fixed Box. Child intrinsic
  size participates in DD-005 only when `aspect` is absent, or as
  the explicitly chosen fallback for the both-axis unbounded edge
  case.
- DD-001 receives Box's resolved outer bounds as input and decides
  child measure / alignment / overflow inside them. The phase
  contract is **child clipped or aligned inside the aspect-fixed
  bounds, never extending them**.

Concrete consequence: the following DD-001 × DD-005 combinations are
**invalid** and do not appear as recommended options —

- DD-005 = "aspect set; child intrinsic size grows the Box" with any
  DD-001 alignment / clip option (would contradict the layering;
  Phase 2 does not admit a stretch-Box-to-fit-child variant).
- DD-001 = "child measure overrides Box outer bounds" with any
  DD-005 option (same contradiction from the inside).

The Option tables below cite this layering in each DD's
Recommendation prose so reviewers can verify Option respect for the
dependency direction.

---

### DD-M3-P2-001 — Box IR node form and 0+ child layout semantics

**Status:** Proposed

**Context:**
Box is a new layout primitive in `wasamo-ir` and `wasamo-runtime`.
Phase 2 must commit to (i) the IR node shape, (ii) the 0-child
shape, (iii) the child measure pass, (iv) child alignment within
Box bounds, (v) overflow / clip behaviour, and (vi) the multi-child
semantics. The last is the load-bearing sub-issue: Box's N-child
layout must **not** be a back-door ZStack — overlay is A4 /
Phase 6's responsibility, and Phase 2's contract here directly
shapes what Phase 6 ZStack's primitive contribution can be.

**Options (IR node shape):**

Option A — Per-kind tag parallel to `HStack` / `VStack` / `Rectangle`
(recommended)
- `WidgetKind::Box` joins the existing per-kind enumeration; the
  layout function in `wasamo-runtime` dispatches on the tag.

  - What you gain: Symmetric with every existing M2 widget. Pattern
    matching on `WidgetKind` is exhaustive at compile time.
  - What you give up: A new tag everywhere `WidgetKind` is matched.
    The set is small and discoverable.
  - **Technical risk:** Low. Pure additive extension of an existing
    enum.

Option B — Structural variant in an `IrLayout` umbrella
- Introduce an `IrLayout` family enum carrying Box, HStack, VStack as
  variants of a "layout container" category, distinct from leaf
  widgets (Text / Button / Rectangle).

  - What you give up: A new structural axis without payoff — M2 already
    treats HStack / VStack as per-kind tags, not as a separate family.
    Adopting an umbrella here would re-open DD-M2-P6 territory for one
    new widget, mid-milestone.
  - **Technical risk:** Medium. Touches every M2 layout dispatch site.

**Options (0-child shape):**

A Box with no children but with `aspect` and/or `fill` must still
produce a visual rectangle. This is the placeholder-shape minimum and
the structural support for DD-006.

Option A — `Box { }` (0 children) is valid and renders the
aspect-derived rectangle filled with `fill` (recommended)
- The IR loader admits empty `children` lists; the layout pass
  produces a sized rectangle; the visual pass paints `fill` (or
  transparent if `fill` is absent).

  - **Technical risk:** Low.

Option B — Reject 0-child Box at IR load
- Diagnoses empty containers as a `wasamoc check` error.

  - What you give up: DD-006's placeholder pattern degenerates: a
    Box-with-just-fill rectangle cannot exist as a structural scrim
    or as the "no label yet" thumbnail in Phase 3.
  - **Technical risk:** Low to implement; pays back negatively in
    DSL ergonomics.

**Options (multi-child semantics — load-bearing):**

Option A — Single-child-only; multi-child rejected at `wasamoc check`
(recommended)
- A Box admits 0 or 1 child. 2+ children is a compile-time
  diagnostic. The "0+ child container" wording of A6 is honoured by
  admitting 0 and 1; "+" is read as "at-most-one in Phase 2" with
  the surface widened (if at all) by Phase 6 when ZStack lands.

  - What you gain: Maximum structural defence against a back-door
    ZStack. Phase 6's ZStack gets full latitude to define z-order
    and multi-child overlap semantics without inheriting an implicit
    Phase 2 contract. The two M3 gallery uses of Box (0-child scrim,
    1-child placeholder) both fit. The diagnostic message points
    users at ZStack / VStack / HStack for multi-child needs.
  - What you give up: A6's "0+" surface wording is narrowed at the
    spec level — readers see "Box admits 0–1 child in M3 Phase 2;
    multi-child overlap belongs in ZStack (Phase 6)." This is a real
    public-surface narrowing, recorded in `docs/dsl_spec.md` and in
    the Phase 2 spec marker.
  - **Technical risk:** Low. Diagnostic + IR-loader rejection are
    small.

Option B — All children share full Box bounds; no z-order declared
- The IR admits N children. Each child measures against Box bounds;
  their visual stacking order is document order, but no z-order
  *semantics* are spec'd — overlapping behaviour is "implementation
  defined" until Phase 6 ZStack lands.

  - What you gain: Honours A6's "0+" literally. No Phase 2 → Phase 6
    spec drift if Phase 6 chooses to re-spec on top.
  - What you give up: "Implementation defined" overlap is a footgun
    Phase 6 will inherit. Either Phase 6 ZStack confirms the implicit
    behaviour as the explicit one (so Phase 2 silently set the
    contract) or it contradicts it (so Phase 6 has to break Phase 2's
    proof). The framing flags this as the back-door-ZStack risk.

Option C — Document-order top-left stacking, each child consuming
the next available space
- Stack-of-rows semantics inside Box. Effectively a degenerate VStack.

  - What you give up: Adds a third stacking primitive to M3 mid-milestone
    with no acceptance criterion calling for it. Conflicts with the
    "pure primitive" framing of Phase 2.

**Options (child measure pass; conditional on at least 1 child):**

Option A — Box's resolved outer bounds (from DD-005) are passed
through to the child as the child's measure constraint (recommended)
- The child measures against the full inner bounds. Smaller children
  align (per the alignment sub-decision below); larger children clip
  (per the overflow sub-decision below).

  - **Technical risk:** Low.

Option B — Child intrinsic size capped at Box bounds (`min(intrinsic,
box)`)
- The child gets its intrinsic size if it fits, the Box bound otherwise.

  - What you give up: Two layout behaviours for "child smaller than
    Box" (intrinsic) vs "child larger than Box" (capped). A child's
    visual position depends on its intrinsic dimensions in non-obvious
    ways. Phase 3 WrapPanel-of-thumbnails would inherit this
    variability.

**Options (child alignment within Box bounds; conditional on at
least 1 child):**

Option A — Center (recommended)
- A child smaller than Box is centred horizontally and vertically
  inside the Box. No per-child override in Phase 2.

  - What you gain: Matches the placeholder use case (a Text label
    centred over a coloured rectangle is the visual the M3 gallery
    references). No new attribute surface in Phase 2.
  - What you give up: No top-left / per-child alignment in Phase 2.
    If a later phase needs per-child alignment (e.g. a "caption
    bottom-aligned" pattern), it opens its own DD; Phase 2 reserves
    no surface for it.

Option B — Top-left
- The child anchors at Box's top-left corner.

  - What you give up: Visual mismatch with the placeholder use
    case — labels typically read centred.

Option C — Configurable per-child via a new attribute
- Add `align: <center|top-left|...>` to children of Box.

  - What you give up: New attribute surface unmotivated by any
    Phase 2 acceptance criterion. Out of phase scope; defer.

**Options (overflow / clip; conditional on at least 1 child):**

Option A — Clip the child to Box bounds (recommended)
- A child measuring larger than Box bounds is visually clipped to
  the rectangle. Layout slot does not grow.

  - What you gain: Consistent with M4 ScrollView's separate clip
    surface — Phase 4 inherits a Box that already clips, so
    ScrollView's contribution is the *scrollable viewport*, not the
    clipping primitive. Honours the layering note: aspect-derived
    bounds are inviolable.
  - **Technical risk:** Low. A clip rectangle is a Direct2D / Visual
    Layer primitive.

Option B — Visible overflow (child paints outside Box bounds)
- The child renders at its intrinsic / measured size, painting
  outside the Box's visual rectangle.

  - What you give up: Breaks the "Box visually equals its
    aspect-derived rectangle" contract that placeholders and scrims
    rely on. Adjacent siblings (Phase 3 WrapPanel-of-Boxes) would
    paint over each other if any one overflows.

**Recommendation:** Option A for every sub-issue —
- IR shape: per-kind tag (`WidgetKind::Box`).
- 0-child: valid; renders aspect-derived rectangle filled with
  `fill`.
- Multi-child: single-child-only; 2+ rejected at `wasamoc check`.
- Child measure: Box bounds passed through unchanged.
- Child alignment: centred.
- Overflow: clip.

Design quality dominates here, particularly on multi-child. The
single-child-only stance is the load-bearing defence against
inheriting an implicit ZStack contract. The placeholder use case
(`Box { aspect: 1:1; fill: #ccc; Text { ... } }`) and the scrim use
case (`Box { fill: #00000080 }`) are both 0 or 1 child; A6's "0+"
surface is narrowed accordingly in the spec text. Phase 6's ZStack
ADR is then free to widen the multi-child surface in whichever
shape ZStack needs, without contradicting Phase 2.

**Forward-compat exposure:** Option A's exposure under foreseeable
future events (see Out of scope below):

- Phase 6 ZStack opens multi-child overlap. The narrowed "single-child"
  Box surface is structurally compatible: ZStack widens the
  *multi-child* surface separately; Box's single-child contract does
  not need revision. Option B would have ZStack contradicting or
  ratifying an implicit Phase 2 multi-child contract, so its
  exposure is asymmetrically higher.
- A future "image widget" landing in M4+ does not pressure Box's
  child-layout contract — DD-006's placeholder pattern is the bridge,
  and image widgets become leaf children of Box like Text does today.
- The `align: ...` per-child attribute (Option C of the alignment
  sub-issue) is additive if a later phase needs it; Phase 2's "centred,
  no override" default does not foreclose it.

---

### DD-M3-P2-002 — `aspect: <ratio>` value-type representation

**Status:** Proposed

**Context:**
Box's `aspect` attribute requires a value type representing an
aspect ratio (width:height). The type threads through `wasamoc` (lex,
parse, check, lower), `wasamo-ir` (literal form, IR text grammar),
and `wasamo-runtime` (loader, measure-arrange pass per DD-005).

Per DD-M3-P2-004's recommendation below, `aspect` is **constant-only**
in Phase 2 — no state binding. This loosens the value-type
requirement: a new `IrType::Ratio` for state declarations is not
needed; only a new `IrLiteral::Ratio` for the literal attribute value.

**Options (numeric form / surface syntax):**

Option A — Rational pair `aspect: <num>:<den>`, with both sides
positive integer literals (recommended)
- Surface syntax: `aspect: 16:9` or `aspect: 1:1`. `wasamoc` lexer
  emits a new `Ratio` token; parser produces
  `IrLiteral::Ratio { num: i32, den: i32 }`.
  `wasamoc check` rejects `num <= 0` or `den <= 0` at compile time.

  - What you gain: Exact arithmetic for measure-arrange (Box bounds
    derive from `parent_width * num / den` or `parent_height * den /
    num` with no float rounding). NaN / infinity are structurally
    unreachable — `wasamoc check` already excludes zero / negative
    sides, so the runtime cannot see them. Surface syntax matches
    conventional aspect-ratio notation (16:9, 4:3, 1:1).
  - What you give up: Two integer fields per literal vs one float
    field. Lexer gains one new token form.
  - **Technical risk:** Low.

Option B — Float literal `aspect: 1.7778`
- Adds `IrType::F32` / `IrLiteral::F32` / `HandlerExpr::F32Lit` to
  the type-suffix chain (per Phase 1 pattern;
  [predoc-inputs.md §7](../notes/m3-phase-2/predoc-inputs.md#7-f32--f64-を-irtype-に入れるかの再評価)).

  - What you gain: One numeric form for all aspect surfaces.
    `TypeName::Float` already exists in `wasamoc`'s AST, so the lex /
    parse cost is largely paid.
  - What you give up: `wasamoc check` must guard against NaN /
    infinity / zero / negative / sub-epsilon values. Runtime
    measure-arrange must handle float rounding (e.g. a 1920×1080
    parent + `aspect: 1.7778` produces 1919.616×1080 — rounding
    direction must be normative). Introduces F32 to IR / runtime
    without a Phase-2-internal need beyond aspect — the cost is
    real, the payoff narrow. F5 deferral note: adding `IrType::F32`
    creates a third scalar type with no widget property and no
    binding admitting it (DD-004 says constant-only), so the type
    enters the IR for one attribute's literal form only. Defensible
    but asymmetric.
  - **Technical risk:** Medium. Float-rounding spec is the part Phase
    2 would have to write.

Option C — Compile-time-parsed string `aspect: "16:9"`
- `wasamoc` parses the string at check time, producing the same
  `IrLiteral::Ratio` as Option A.

  - What you give up: Cosmetic inconsistency with non-string ratio
    literals across DSL ancestry (CSS, XAML use bare 16:9). Per
    [predoc-inputs.md §1](../notes/m3-phase-2/predoc-inputs.md#1-box-が新規-propertyvalue-variant-を入れるなら-abi-value-conversion-arm-は同じ-step-に-fold-する),
    parse failure becomes a `wasamoc check` diagnostic, not a runtime
    fallback — same as Option A — so the strictness is identical;
    only the surface form differs. The lexer save (no new token form)
    is real but small.
  - **Technical risk:** Low.

Option D — Twin i32 attributes `aspect-width: 16` and `aspect-height: 9`
- No new value type at all — reuse existing i32 attributes.

  - What you give up: Surface fragmentation (one logical attribute
    written as two). A reader can write one without the other; spec
    must define the default and the both-absent / one-absent cases.
    Measure-arrange's "aspect set" branch becomes "both aspect-*
    set", which is a weaker structural signal than a single Ratio
    literal.

**ABI / IR plumbing (consequent on Option A):**

- New `IrLiteral::Ratio { num: i32, den: i32 }` variant in
  `wasamo-ir`.
- New `PropertyValue::Ratio { num: i32, den: i32 }` variant in
  `wasamo-runtime/src/widget.rs`. ABI value-conversion arms in
  `wasamo-runtime/src/abi.rs` (`read_property_value` /
  `write_property_value` / `property_value_to_owned`) gain `Ratio`
  arms in the same commit per
  [predoc-inputs.md §1](../notes/m3-phase-2/predoc-inputs.md#1-box-が新規-propertyvalue-variant-を入れるなら-abi-value-conversion-arm-は同じ-step-に-fold-する).
- IR text grammar (`docs/dsl_spec.md` §8) gains the `Ratio` literal
  production.
- No new `WasamoValue` tag for the C ABI is added in Phase 2
  because aspect is constant-only (DD-004) — the C ABI's value union
  is the surface that *bindable* properties reach through, and aspect
  does not. Were DD-004 to flip to bindable for aspect, a new
  `WASAMO_VALUE_RATIO` tag would land then.

**Recommendation:** Option A — rational pair `aspect: <num>:<den>`
with positive-integer sides, `IrLiteral::Ratio` literal form, no
new `IrType` extension. The decision turns on three factors:
exact arithmetic, structurally unreachable NaN / infinity, and
surface-syntax familiarity. Option B's float path forces a
rounding spec on Phase 2 (where the framing already nuances "no
novel measure-arrange algorithm" — DD-005); Option C's string path
saves a lexer token at the cost of cosmetic friction; Option D
fragments the surface.

**Forward-compat exposure:** Option A's exposure under foreseeable
future events (see Out of scope):

- Theming / styling system (M4+) may want to express aspects from a
  named palette. `IrLiteral::Ratio { num, den }` survives — a
  theme-resolved aspect becomes a Ratio literal at theme-resolve time.
  Option B's float form survives the same event symmetrically.
- A future bindable-aspect surface (rejected for Phase 2 by DD-004)
  would add `IrType::Ratio`, `HandlerExpr::RatioLit` /
  `RatioPropRead`, and `WASAMO_VALUE_RATIO` tag at that point —
  additive in the established type-suffix pattern. Option A's
  exposure here is symmetric with Option B's: both add one new type
  to the chain.

---

### DD-M3-P2-003 — `fill: <color>` value-type representation

**Status:** Proposed

**Context:**
The `fill` attribute requires a color representation. The central
question is **alpha**, surfaced ahead of the variant-strategy
question because alpha drives the variant choice rather than being
downstream of it.

A6 names "scrim use" as the motivating use case for `fill`. A scrim
is semantically a semi-transparent overlay; without alpha, the
`fill` value cannot express what A6 calls for, and the
m3-target-app-predoc wording "scrim の alpha 値 styling は M3 では
扱わない" but "不透明 fill で代替する" becomes internally
inconsistent (an opaque "scrim" is not a scrim). Phase 2 must decide
whether the value-type layer carries alpha (and the styling layer
separately decides whether to expose alpha control), or whether
`fill` is intentionally alpha-less and the M3 scrim is opaque-by-spec.

**Options (alpha — central question):**

Option A — Value type carries alpha (recommended)
- `PropertyValue::Color` carries four 8-bit channels (r, g, b, a).
  The Phase 2 surface syntax (below) admits `#RRGGBBAA`. M3 styling
  remains out of scope per the target-app pre-doc, but the value
  layer is built once and reused as alpha-styling controls land in
  M4+.

  - What you gain: A6's scrim use case is expressible. The
    target-app pre-doc's internal tension resolves: "M3 fills can
    carry alpha; M3 styling does not expose alpha control beyond the
    literal hex form" — a one-line spec statement.
  - What you give up: 32 bits per color value vs 24. Negligible.
  - **Technical risk:** Low.

Option B — Value type excludes alpha; M3 scrim is opaque-by-spec
- `PropertyValue::Color` carries (r, g, b). Surface admits `#RRGGBB`
  only.

  - What you give up: A6's scrim wording is internally inconsistent
    as recorded above. M4+ alpha-styling adoption forces a value-type
    revision (`PropertyValue::Color` widened to 4 channels at that
    point, with every existing `#RRGGBB` literal becoming an implied
    `alpha = 0xFF`). The revision is mechanically additive, but it's
    a revision Option A avoids.

**Options (variant strategy — consequent on Option A above):**

Option A — New `PropertyValue::Color(u32)` (recommended)
- `PropertyValue::Color` carries a packed `u32` (0xAARRGGBB or
  0xRRGGBBAA, fixed at one byte order — DSL spec records the
  choice). `IrLiteral::Color(u32)` parallel. `wasamoc` lexer accepts
  `#RRGGBB` (alpha implied 0xFF) and `#RRGGBBAA`. ABI value-
  conversion arms folded into the same step per
  [predoc-inputs.md §1](../notes/m3-phase-2/predoc-inputs.md#1-box-が新規-propertyvalue-variant-を入れるなら-abi-value-conversion-arm-は同じ-step-に-fold-する).

  - What you gain: Type-tagged at the value layer. No string-parsing
    at the widget setter. Forward-compatible with theming bindings
    that produce color values.
  - **Technical risk:** Low.

Option B — Reuse `PropertyValue::Str` with `#RRGGBB[AA]` parsing at
the loader
- No new variant; the loader parses the hex string when binding the
  fill attribute.

  - What you give up: Hidden string ↔ color coercion at the widget
    setter. Same objection as DD-M3-P1-007 Option C (bool stringify):
    every future color property re-implements the parse;
    misclassifies as parse failure any legitimate string-typed
    property whose value happens to be `#`-prefixed. Soft path,
    permanent footgun.

**Options (surface forms):**

Option A — `#RRGGBB` and `#RRGGBBAA` only (recommended)
- Two hex forms; `#RRGGBB` implies alpha 0xFF.

  - What you gain: Minimal lexer extension. Forward-compatible with
    named colors / theme palette (M4+) as additive surface.
  - **Technical risk:** Low.

Option B — Add named colors (`red`, `blue`, `transparent`, ...)
- A small palette in addition to hex.

  - What you give up: Locks in palette content at Phase 2; M4+
    theming arguably wants control over what `red` means. Defer to
    the phase that ships theming.

**Recommendation:** Alpha **included**; new `PropertyValue::Color(u32)`
+ `IrLiteral::Color(u32)`; surface forms `#RRGGBB` and
`#RRGGBBAA` only. The alpha-yes decision is the load-bearing one —
without it, A6's scrim wording is internally inconsistent.

The Phase 2 styling commitment remains: *the value type carries
alpha; the M3 styling layer does not gain alpha-styling controls
beyond the literal hex form*. Theming, palette, and dynamic alpha
adjustment all remain M4+ work (per
[m3-plan.md §Out of scope](../plans/m3-plan.md#out-of-scope-deferred-to-later-milestones)
and target-app pre-doc Visual / styling Out-of-scope).

**Forward-compat exposure:** Option A (alpha-yes) is dual-compatible
with both possible future events:

- If M4+ theming admits alpha control, the value type is already
  there — theming binds in semi-transparent values without value-
  layer revision.
- If M4+ theming explicitly forbids alpha at the styling layer, the
  value type still carries alpha for the structural scrim case
  Phase 2 needs; styling restricts at its layer without value-layer
  revision.

Option B (alpha-no) requires a value-layer revision under the
former event and is "scrim cannot be expressed" under either.

---

### DD-M3-P2-004 — Bindable surface for `aspect` and `fill`

**Status:** Proposed

**Context:**
Whether `aspect` and `fill` can each be driven by a reactive Signal
at runtime, or are constant at load time. The two attributes decide
independently. Either bindable path requires the per-type writer
seam — a new `evaluate_<T>_binding` + `widget_write_property_<T>` +
`register_<T>_binding` triple selected at `ir_loader::build_node`
(DD-M3-P1-007 pattern;
[predoc-inputs.md §2](../notes/m3-phase-2/predoc-inputs.md#2-新しい-bindable-property-は-per-type-writer-seam-を-ir_loader-call-site-で選ぶ)).
F5 (`TypedValue` deferral) is preserved by construction either way —
the seam structurally enforces it.

The decisive criterion is: does any wireframe surface in
[docs/references/m3-gallery-wireframe.html](../references/m3-gallery-wireframe.html)
require Box's aspect or fill to vary reactively *within Phase 2's
sub-screen*?

**Options:**

Option A — Both `aspect` and `fill` constant-only in Phase 2
(recommended)
- Neither attribute admits a state binding; both are load-time
  literals only. `wasamoc check` rejects `bind aspect: ...` and
  `bind fill: ...` with a diagnostic naming the rejected attribute.
  No new per-type writer seam triple is built in Phase 2; no new
  `IrType` (Ratio / Color) is added.

  - What you gain: Smallest Phase 2 surface — value-type literal
    plumbing only. F5 deferral is doubly protected (no new seam to
    pressure it). Future phase that needs bindable aspect or fill
    adds the seam triple as an additive extension; the IR / ABI
    plumbing built in Phase 2 is forward-compatible.
  - What you give up: A theme-driven scrim color or an
    animated-aspect lightbox cannot be expressed in M3 until the
    phase that opens the bindable surface (M4+ theming, plausibly).
    The gallery wireframe's Phase 2 sub-screen — Box + Text
    placeholder against a trivial frame — needs neither.
  - **Technical risk:** Low.

Option B — `fill` bindable, `aspect` constant-only
- Admits `bind fill: <state-of-color>`; rejects `bind aspect: ...`.
  Adds `evaluate_color_binding` + `widget_write_property_color` +
  `register_color_binding` triple; adds `IrType::Color`,
  `HandlerExpr::ColorLit` / `ColorPropRead`; adds
  `WASAMO_VALUE_COLOR` ABI tag.

  - What you gain: Theme-driven scrim color expressible in M3. M4+
    theming work has one less seam to build.
  - What you give up: A full per-type seam built for an attribute
    no Phase 2 sub-screen exercises reactively. The wireframe's
    Phase 6 lightbox uses scrim; that's the phase that should pay
    the seam cost, since it's the first phase that *uses* a bindable
    fill. Speculative seam-building violates the Phase 1 discipline
    of "build the seam in the phase that needs it" (DD-M3-P1-007 was
    built for `Button.enabled`, not pre-built for unknown later
    callers).

Option C — Both `aspect` and `fill` bindable
- Both seam triples built in Phase 2.

  - What you give up: All of Option B's objections, doubled.

**Recommendation:** Option A — both constant-only in Phase 2. The
gallery wireframe's Phase 2 sub-screen exercises neither attribute
reactively; the Phase 1 discipline of "build the seam in the phase
that needs it" applies symmetrically. F5 deferral is structurally
preserved: no new bindable surface means no new pressure on
`TypedValue`. The seam pattern from Phase 1 remains available for
the phase that first uses bindable fill / aspect.

**Forward-compat exposure:** Option A keeps the value-type plumbing
(`IrLiteral::Ratio`, `IrLiteral::Color`, `PropertyValue::Ratio`,
`PropertyValue::Color`) additive and forward-compatible with a
future bindable surface. When that phase lands, the additions are:

- New `IrType::Ratio` / `IrType::Color` for state declarations.
- New `HandlerExpr::RatioLit` / `RatioPropRead` /
  `HandlerExpr::ColorLit` / `ColorPropRead`.
- New `evaluate_ratio_binding` / `evaluate_color_binding` writer
  triples.
- New `WASAMO_VALUE_RATIO` / `WASAMO_VALUE_COLOR` ABI tags.
- Widget-catalog `IrType` for the relevant `PROP_*` ids becomes
  `IrType::Ratio` / `IrType::Color`.

Each addition follows the Phase 1 type-suffix pattern verbatim;
the Phase 2 literal plumbing is not revised, only extended.

---

### DD-M3-P2-005 — Aspect constraint measure-arrange algorithm

**Status:** Proposed

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
  - **Technical risk:** Low. Pure integer arithmetic; the rational
    pair form (DD-002 Option A) makes this exact.

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

Option A — Load-time error (recommended)
- `wasamoc check` cannot detect this (it depends on runtime
  layout context); `wasamo-runtime`'s layout pass emits an error
  when it encounters a Box with `aspect` set whose parent provides
  no bounded axis. Error surfaces as a runtime diagnostic with
  the Box's IR location.

  - What you gain: Honest behaviour. NaN / silent-zero outcomes
    are excluded structurally. Practical: a top-level Box with
    `aspect` set inside a window with no provided size is a real
    DSL author mistake; an error tells them clearly.
  - What you give up: Phase 2 must build the layout-time error
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
  - What you give up: A warning surface in `wasamoc check`.
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

  - **Technical risk:** Low.

Option B — Box always expands to parent bounds
- Empty and child-bearing both fill the parent rectangle.

  - What you give up: A `Box { fill: #ccc; Text { text: "label" } }`
    with no aspect would expand to its parent, painting `#ccc`
    everywhere, which is a footgun for the "label-with-background"
    pattern.

**Options (aspect value validity):**

Option A — `wasamoc check` rejects `num <= 0` or `den <= 0`
(recommended)
- The literal form (DD-002 Option A: positive integer pair) makes
  zero / negative reachable only through author error; compile-time
  rejection per Phase 1's T14 discipline ("bad surface forms fail
  at the source-level diagnostic gate"). Runtime cannot see invalid
  ratios.

  - **Technical risk:** Low.

Option B — Runtime fallback to spec-defined default
- Bad ratio → Box renders as 1:1 (or similar).

  - What you give up: Silent fallback obscures the author error
    until visual inspection.

**Recommendation:** Option A for every sub-issue —
- Bounded parent: inscribed (fit smaller).
- Unbounded on one axis: bounded-axis-wins.
- Unbounded on both axes: load-time error.
- Width/height conflict: explicit wins; aspect informational + warning.
- Child intrinsic (no aspect): shrink-to-fit; fall back to parent
  bounds when no children.
- Aspect value validity: `wasamoc check` rejects non-positive sides.

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

### DD-M3-P2-006 — Placeholder pattern (Box + Text child) canonicalization

**Status:** Proposed

**Context:**
[m3-target-app-predoc.md — 保留 2 closure](../notes/m3/m3-target-app-predoc.md#保留-2-closure-image-widget-surface-の-m3-開封可否--不開封-m4-へ-defer)
establishes that the M3 Image-widget deferral is carried by a
**Box + Text-child** placeholder. Phase 2 settles how this pattern is
canonicalized: where it lives in the spec, how it appears in the
example, and what later phases cite when they consume it.

**Options:**

Option A — Normative spec convention in `docs/dsl_spec.md`
(recommended)
- Add a dedicated subsection of the Box chapter titled "Image
  placeholder pattern (M3)". The pattern is spelled normatively:

  ```
  Box { aspect: <ratio>; fill: <color>; Text { text: <label> } }
  ```

  with the example forms typically used in the gallery (1:1 square,
  16:9 photo, neutral `#cccccc` fill, label text giving photo index
  or filename). The subsection records that the pattern is the
  agreed M3 substitute for the deferred Image widget surface, and
  that Phase 3 (WrapPanel of thumbnails) and Phase 6 (ZStack
  lightbox) consume the same pattern verbatim.

  - What you gain: Single citable spec location. Phase 3 and Phase 6
    spec writing cites it rather than redefining. M5 LSP / tooling
    sees a documented pattern. M4 Image-widget ADR has a clear
    "supersedes" target.
  - What you give up: Spec real estate — one subsection at Phase 2.
    Trivial.
  - **Technical risk:** Low. Spec writing only.

Option B — Informal pattern noted but not normative
- A passing mention in the Box chapter ("placeholders typically
  use Box + Text") without normative spec status.

  - What you give up: Phase 3 / Phase 6 either restate the pattern
    (drift risk) or cite an informal mention (weak citation).

Option C — Helper widget alias (e.g. `Placeholder { ... }`)
- A new widget kind that expands to the Box + Text pattern.

  - What you give up: A new widget for a deferred-Image bridge —
    the alias would have its own scope to defend. The M4 Image
    widget would supersede *both* Box + Text and the alias, doubling
    the supersession surface. Cleaner to keep the bridge structural,
    not nominal.

**Recommendation:** Option A — normative spec convention in
`docs/dsl_spec.md`. Phase 2 ships the subsection with the example
forms and the explicit cross-reference to Phase 3 and Phase 6's
expected usage. The Phase 2 `examples/gallery/gallery.ui` Box
sub-screen (framing decision F) demonstrates the pattern; the
Phase 2 spec marker
(`**Phase status:** M3-Phase 2 ADR-accepted design draft; pending
implementation re-sync`) sits at the top of the Box chapter and
applies to this subsection.

**Forward-compat exposure:** Option A's exposure under foreseeable
future events:

- M4 (or later) Image widget landing supersedes the placeholder
  pattern. The supersession is clean: the normative subsection
  gains a "Superseded by `<Image>` widget (M4 ADR)" header, and
  the spec retains the pattern as a back-compat shape for
  pre-Image authors. Phase 3 and Phase 6 spec citations remain
  valid because the pattern they cite is still spec-recorded;
  they migrate to `<Image>` syntactically when Image lands.

The pattern itself is structural (Box + Text), so it survives any
visual / styling refinement Phase 2's siblings make to Text or to
fill rendering.

---

## Out of scope (for M3-Phase 2; recorded explicitly)

- **Image widget surface, asset pipeline, icon font, image decoder.**
  M4 or later
  ([m3-plan.md §Out of scope](../plans/m3-plan.md#out-of-scope-deferred-to-later-milestones),
  [m3-target-app-predoc.md — 保留 2 closure](../notes/m3/m3-target-app-predoc.md#保留-2-closure-image-widget-surface-の-m3-開封可否--不開封-m4-へ-defer)).
  Phase 2 ships the structural bridge (DD-006); the Image widget
  itself ships when M4+ commits to it.
- **Button content other than text** (e.g. Image inside Button).
  M4 or later (tied to the Image-widget deferral).
- **ZStack overlay primitive and multi-child overlap semantics.**
  Phase 6. DD-001 Option A's single-child-only Box is the
  structural defence against pre-empting ZStack's contract.
- **`TypedValue` generic value union.** F5 deferral maintained
  ([m3-start-framing.md §F5](../notes/m3/m3-start-framing.md);
  [m2-to-m3-handover.md §4](../notes/m2-to-m3-handover.md)).
  DD-004's both-attributes-constant-only stance preserves the
  deferral structurally; the per-type writer seam pattern remains
  available for the phase that first opens a new bindable type.
- **`bool` string-interpolation surface** and any generic
  display-conversion surface. Phase 6+ formatting work
  ([predoc-inputs.md §8](../notes/m3-phase-2/predoc-inputs.md#8-bool-の-display-conversion-は明示-surface-ができるまで禁止)).
  Phase 2 introduces no formatting surface; the rule from
  Phase 1's T14 (no implicit `bool` → string) continues without
  Phase 2 action.
- **Synchronous non-batched drain proof contract.** Cross-phase
  reactive premise carried in
  [m2-to-m3-handover.md §3 item 4](../notes/m2-to-m3-handover.md).
  Box introduces no event / input batching, no layout scheduling,
  and no headless proof boundary changes; Phase 2 does not alter
  this contract
  ([predoc-inputs.md §9](../notes/m3-phase-2/predoc-inputs.md#9-bool-live-proof-は現行の同期-non-batched-drain-に依存している)
  is a back-pointer).
- **Cycle detection / ordering ties / `MUTATION_CAP` × fan-out
  residuals.**
  [m2-to-m3-handover.md §3 items 1–3](../notes/m2-to-m3-handover.md).
  Phase 6/7 work — Phase 2 does not exercise the reactive engine
  beyond the constant-load path.
- **Scrim alpha styling, theme system, multi-color named palette
  resolution.** M4+ (per
  [m3-target-app-predoc.md Out-of-scope §Visual / styling](../notes/m3/m3-target-app-predoc.md)).
  DD-003's alpha-yes decision is at the *value-type* layer; the
  *styling* layer (theme palette, dynamic alpha control) remains
  M4+ work.
- **Bindable surface for `aspect` and `fill`.** Constant-only in
  Phase 2 per DD-004 Option A. The first phase that exercises
  bindable aspect or fill opens the per-type writer seam triple
  for that attribute.
- **Per-child `align: ...` attribute under Box** and any other
  child-positioning attribute beyond "centred". DD-001 Option A
  (alignment) commits to centred-by-default with no override;
  later phases that need other alignments open their own DD.
- **`f32` / `f64` numeric scalar in `IrType`.** Deferred per
  DD-002 Option A (rational-pair aspect) closing the float surface
  for Phase 2.
  [predoc-inputs.md §7](../notes/m3-phase-2/predoc-inputs.md#7-f32--f64-を-irtype-に入れるかの再評価)'s
  default of "do not add" stands.
- **C / Zig host parity for the Box sub-screen.**
  [m3-plan.md §Phase-end criteria item 5](../plans/m3-plan.md#phase-end-criteria)
  calls for at least one host per phase; Phase 8 broadens the full
  gallery to all three. Phase 2 seeds `examples/gallery-rust/`
  only.

## Owner-agreement checkpoints

Two of the DDs above are load-bearing value judgements that warrant
explicit yes/no from the owner before this ADR moves to Accepted.
All other DDs follow mechanically from these two.

### Checkpoint 1 — DD-M3-P2-001 multi-child semantics

**Question:** Is Box single-child-only in Phase 2 (Option A,
recommended), or does Phase 2 admit N children with shared bounds
and no z-order declared (Option B)?

**Default answer:** Option A — single-child-only; 2+ children
rejected at `wasamoc check`.

**Framing for owner:** The recommendation narrows A6's "0+ child
container" surface wording at the spec level — readers see "Box
admits 0 or 1 child in M3 Phase 2; multi-child overlap belongs in
ZStack (Phase 6)." This is a public surface narrowing recorded in
`docs/dsl_spec.md` and visible on the Phase 2 spec marker.

The trade-off:

- Option A keeps Phase 2's contract narrow and gives Phase 6 ZStack
  full latitude to define z-order and multi-child overlap without
  inheriting an implicit Box contract. The two Phase 2 use cases
  (0-child scrim and 1-child placeholder) both fit. The diagnostic
  message points users at ZStack / VStack / HStack for multi-child
  needs.
- Option B honours A6's "0+" literally, at the cost of "implementation
  defined" overlap semantics that Phase 6 either ratifies (silently
  set by Phase 2) or contradicts (breaks Phase 2's proof). The
  framing's load-bearing sub-issue is this risk.

If the owner prefers A6's literal wording preserved, Option B is
acceptable but requires Phase 6's ADR to commit affirmatively on
ZStack-vs-Box layering; the Phase 2 marker would record the
"implementation defined" overlap as a known fold-forward to Phase 6.

### Checkpoint 2 — DD-M3-P2-003 alpha decision

**Question:** Does the `fill` value type carry alpha in Phase 2
(Option A, recommended), or is the M3 scrim opaque-by-spec
(Option B)?

**Default answer:** Option A — alpha-yes; new `PropertyValue::Color`
+ `IrLiteral::Color` carry four 8-bit channels; surface admits
`#RRGGBB` (alpha 0xFF implied) and `#RRGGBBAA`.

**Framing for owner:** A6 explicitly names "scrim use" as the
motivating use case for `fill`. A scrim is semantically a
semi-transparent overlay; an opaque "scrim" is not a scrim. The
m3-target-app-predoc wording "scrim の alpha 値 styling は M3
では扱わない" but "不透明 fill で代替する" is internally
inconsistent without Option A.

Option A's positioning: *the value type carries alpha; the M3
styling layer does not gain alpha-styling controls beyond the
literal hex form*. Theming, palette, and dynamic alpha adjustment
all remain M4+ work. M3 authors can write `fill: #00000080` for
a half-opaque black scrim today; what they *cannot* do is bind
that alpha to a state variable (DD-004 says `fill` is
constant-only in Phase 2) or pull the color from a theme palette
(M4+ work).

Option B's positioning: the value-type layer matches the
styling-layer constraint — both exclude alpha. M3 scrims are
opaque, and the target-app pre-doc wording is internally consistent
only if M3's "scrim" is read as "an opaque background panel". M4+
alpha-styling adoption then forces a value-type revision
(`PropertyValue::Color` widened from 3 to 4 channels), mechanically
additive but a revision Option A avoids.

The decision is design-quality dominated: Option A makes the M3
spec self-consistent at the cost of one extra channel; Option B
keeps the value type tighter at the cost of an internally-tense
"opaque scrim" wording in the spec.

---

## Summary of decisions

| ID | Topic | Recommendation |
|---|---|---|
| DD-M3-P2-001 | Box IR node form + child-layout contract | Option A across sub-issues — per-kind tag `WidgetKind::Box`; 0-child valid; **single-child-only with 2+ rejected at `wasamoc check`**; child measure passes Box bounds through unchanged; child alignment centred (no per-child override); overflow clips child to Box bounds |
| DD-M3-P2-002 | `aspect: <ratio>` value type | Option A — rational pair `aspect: <num>:<den>`; new `IrLiteral::Ratio { num, den }` + `PropertyValue::Ratio`; no new `IrType` (constant-only per DD-004); ABI value-conversion arms folded into the same step |
| DD-M3-P2-003 | `fill: <color>` value type | Option A — alpha-yes; new `PropertyValue::Color(u32)` + `IrLiteral::Color(u32)`; surface forms `#RRGGBB` and `#RRGGBBAA` only; ABI value-conversion arms folded into the same step |
| DD-M3-P2-004 | Bindable surface for `aspect` / `fill` | Option A — both attributes **constant-only** in Phase 2; no new per-type writer seam built; F5 deferral structurally preserved |
| DD-M3-P2-005 | Aspect measure-arrange algorithm | Option A across sub-issues — inscribed fit (bounded parent), bounded-axis-wins (unbounded on one axis), load-time error (unbounded on both axes), explicit width/height wins + warning (conflict), child intrinsic shrink-to-fit + parent-bounds fallback (no aspect), `wasamoc check` rejects non-positive sides |
| DD-M3-P2-006 | Placeholder pattern (Box + Text) | Option A — normative spec convention in `docs/dsl_spec.md` Box chapter; Phase 3 / Phase 6 cite it; M4 Image-widget ADR supersedes it cleanly |

Implementation task list: belongs in the Phase 2 progress file
`docs/plans/progress/m3-phase-2-progress.md` (created when this ADR
is Accepted and Phase 2 starts execution); not in this ADR and not
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
design-spec draft):

- [docs/dsl_spec.md](../dsl_spec.md) — extensions in three regions:
  - **DSL surface** — new Box chapter under the widget catalog
    documenting the IR node, attributes (`aspect`, `fill`), child-
    layout contract (single-child, centred, clipped), and the
    image-placeholder pattern subsection (DD-006). Section marker
    `**Phase status:** M3-Phase 2 ADR-accepted design draft; pending
    implementation re-sync` at the chapter top.
  - **DSL surface lexer / literal grammar** — `aspect: <num>:<den>`
    ratio literal; `fill: #RRGGBB` and `fill: #RRGGBBAA` color
    literals.
  - **IR text grammar** (§8) — `IrLiteral::Ratio` and
    `IrLiteral::Color` productions.
- [docs/architecture.md](../architecture.md) §6 — Box entry under
  the M2-revised IR section if structural placement warrants;
  short paragraph noting the per-type binding seam is *not*
  extended by Phase 2 (`aspect` / `fill` constant-only) so the F5
  deferral is unpressured.
- [docs/abi_spec.md](../abi_spec.md) — **no new ABI public function
  added**, no new `WASAMO_VALUE_*` tag added (both pursuant to
  DD-004 constant-only). The `read_property_value` /
  `write_property_value` / `property_value_to_owned` machinery in
  `abi.rs` does gain `Ratio` and `Color` arms in the per-type
  conversion match, but these are private internal arms not visible
  across the C ABI boundary; the existing ABI function signatures
  and value-tag numeric assignments are untouched.
- [docs/plans/m3-plan.md](../plans/m3-plan.md) — Progress section's
  Phase 2 row populated (Status: `in progress`; ADR link; progress
  file link).
- [docs/notes/retrospectives.md](../notes/retrospectives.md) —
  per framing decision E (a), the step-retrospective checklist's
  item 3 (clean rebuild) is amended to require `cargo fmt --all --
  --check` against the post-commit state explicitly, with "green"
  interpreted as the `--check` form. CI YAML (framing decision E
  (b)) is **not** updated in Phase 2 — deferred per CLAUDE.md §CI
  rules.

The **Moment 2** commit set (framing decision D — Phase close /
implementation re-sync) lands at phase end; the Box-chapter spec
marker flips to
`**Phase status:** M3-Phase 2 closed; implementation-synced`, any
divergence between the design-spec draft and the implementation is
corrected in the same commit, and earlier-phase spec gaps surfaced
during re-sync may fold per
[predoc-inputs.md §6](../notes/m3-phase-2/predoc-inputs.md#6-retroactive-spec-gap-fold-は最小範囲で同じ-phase-に折り込む)
with owner confirmation. The Phase 2 progress file is retired in
the same commit per the standard `active → closing → retired →
archived` lifecycle.

No ROADMAP revision is anticipated — A6 is already explicit, this
ADR operationalises it.

## Phase 2 verification closure (what counts as A6 evidence)

This section is not a DD — it records the agreed shape of the
proof that closes Phase 2 per framing decision C, so the
implementation plan inherits a concrete target rather than
re-litigating "what does Box's verification mean here?".

A6 (Box layout primitive + image-placeholder pattern) is considered
satisfied when **all four** of the following are observed:

1. **Unit-test evidence (host-independent).** Pure-logic tests in
   `wasamoc` (parse + check + lower) and in `wasamo-runtime`
   non-Windows-bound modules (aspect measure-arrange resolver,
   IR-loader handling of `IrLiteral::Ratio` / `IrLiteral::Color`,
   `PropertyValue::Ratio` / `PropertyValue::Color` plumbing) cover:
   ratio literal parsing; color literal parsing (both `#RRGGBB` and
   `#RRGGBBAA`); DD-005 measure-arrange edge cases (bounded
   parent inscribed fit; unbounded-on-one-axis bounded-axis-wins;
   unbounded-on-both-axes load-time error; explicit width/height
   conflict; child intrinsic shrink-to-fit when aspect absent;
   non-positive ratio sides rejected); `wasamoc check` diagnostics
   for `bind aspect: ...` and `bind fill: ...` (rejected per DD-004)
   and for 2+ children under Box (rejected per DD-001). These run
   on any CI runner.

2. **IR text round-trip evidence.** `wasamoc` emits,
   `wasamo-runtime` loads, and an in-process test reads back:
   `Box { aspect: 16:9; fill: #00000080; Text { text: "Photo 12" } }`
   as the corresponding `WidgetKind::Box` IR node with
   `IrLiteral::Ratio { num: 16, den: 9 }` and
   `IrLiteral::Color(<packed>)`. Tests the DD-001 / 002 / 003 / 006
   surfaces together.

3. **Windows-runtime layout evidence (CI-gated).** A mock-free
   integration test (per CLAUDE.md "Testing rules") on the Windows
   CI runner: a `.ui` fixture declares an aspect-fixed Box with a
   Text child inside a parent of known size. The test loads the
   IR, runs the layout pass, and asserts the Box's resolved
   rectangle matches the inscribed-fit calculation, that the Text
   child is centred, and that the Box's `PROP_BOX_FILL` reflects
   the color literal. Fails (not skips) on a runner that cannot
   create the Compositor — the test gates A6 evidence in CI, not
   local convenience.

4. **End-to-end host evidence (visible smoke).**
   `examples/gallery/gallery.ui` is seeded with the Phase 2
   sub-screen (a Box + Text placeholder against a trivial frame),
   and `examples/gallery-rust/` (newly created) is the
   workspace-member host that builds and runs it. `Start-Process`
   launch is recorded as successful by the assistant; visual
   correctness of `aspect: <ratio>` (rendered rectangle has the
   right ratio) and `fill: <color>` (rendered rectangle is the
   right color, including alpha) is **owner-manual GUI smoke** per
   framing decision G — the assistant does not assert on pixel- or
   eyeball-level correctness.

Items (1)–(3) are required for A6 acceptance; item (4) ties the
evidence back to the m3-plan target-app trajectory and seeds the
gallery directory that every subsequent M3 phase grows. C and
Zig hosts for the Box sub-screen are explicitly **not** required
in Phase 2 (per framing decision F and the Out of scope list);
Phase 8 broadens the full gallery to all three.

The acceptance / non-acceptance of test items (1)–(4) is the
operational form of "Phase 2 done"; the corresponding
implementation checklist (which crate / which test file / which
fixture) belongs in the Phase 2 progress file, not here.
