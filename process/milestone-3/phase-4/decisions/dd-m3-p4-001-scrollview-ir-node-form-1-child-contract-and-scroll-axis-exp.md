### DD-M3-P4-001 — ScrollView IR node form, 1-child contract, and scroll-axis exposure

**Status:** Accepted

**Context:** ScrollView is a new widget in `wasamo-ir` and
`wasamo-runtime`. Phase 4 must commit to (i) the IR node shape
(per-kind tag vs structural variant), (ii) the child-count contract
(exactly 1 vs 0/1 vs 0+), and (iii) whether the scroll axis is an
attribute (`scroll-axis: vertical|horizontal|both`) or hardcoded.
The Phase 4 gallery sub-screen and the wireframe overflow strip
both use vertical scroll only.

**Options (IR node shape):**

- **Option A — Per-kind tag (recommended).** ScrollView appears
  as a new `widget_type: "ScrollView"` value on the generic
  `IrNode`, parallel to `HStack` / `VStack` / `Rectangle` / `Box`
  / `WrapPanel`. Phase 2 / Phase 3 settled this pattern.
  - What you gain: consistency with all prior layout primitives;
    no IR-level structural change; the parser already accepts the
    generic shape unchanged.
  - What you give up: nothing relative to the established pattern.
- Option B — Structural variant in `IrLayout`. ScrollView
  participates as a layout-flavour discriminator rather than a
  widget kind.
  - What you gain: arguably cleaner separation of "container that
    arranges children" from "leaf widget".
  - What you give up: contradicts Phase 2 / Phase 3 precedent;
    rewires the IR's existing categorisation; cost without
    visible benefit at Phase 4 scope.

**Options (child-count contract):**

- **Option A — Exactly 1 child (recommended).** `validate()`
  rejects 0-child and >1-child ScrollView with
  `WASAMO_ERR_IR_MALFORMED`.
  - What you gain: smallest spec surface; ScrollView's single-
    content semantics are unambiguous; matches WPF
    `ScrollViewer.Content` (single-child) precedent.
  - What you give up: authors must wrap N children in an explicit
    container (`ScrollView { VStack { … } }`) rather than relying
    on synthesis.
- Option B — Accept 0 or 1 child. 0 = empty viewport.
  - What you gain: parallel to Phase 3 WrapPanel's 0-child
    admission.
  - What you give up: 0-child ScrollView has no meaning beyond
    "Box-shaped viewport with nothing in it", which Box already
    provides; adds a spec edge case for no acceptance gain.
- Option C — Accept N children with implicit synthetic wrapper
  (e.g. lower to `ScrollView { VStack { N children } }` at IR-load
  time).
  - What you gain: ergonomic shorthand.
  - What you give up: requires additional normative spec
    (which wrapper, how its attributes default, how it composes
    with the author's explicit children) for no acceptance
    criterion in A5; postponable without surface compatibility
    cost.

**Options (scroll-axis exposure):**

- **Option A — Hardcode vertical-only (recommended).** No
  `scroll-axis` attribute in Phase 4; the scroll axis is a fixed
  property of ScrollView.
  - What you gain: smallest spec surface; matches the wireframe
    overflow strip exactly; mirrors Phase 3 DD-002's
    hardcode-horizontal-main-axis decision and the additive-
    extension principle (a later phase that needs horizontal or
    bidirectional scroll opens its own DD and adds the attribute
    additively).
  - What you give up: a future horizontal-only ScrollView use
    case must wait for the attribute to ship.
- Option B — Expose `scroll-axis: vertical|horizontal`. Constant-
  only, no binding.
  - What you gain: future horizontal use case is one attribute
    flip away.
  - What you give up: doubles the measure-arrange spec content
    (DD-005 must spec both axis orientations); no A5 requirement;
    bindable / constant-only sub-issue surfaces for no gallery
    use.
- Option C — Expose `scroll-axis: vertical|horizontal|both`.
  - What you gain: complete two-axis surface.
  - What you give up: significant additional measure-arrange spec
    (diagonal scrolling, two-dimensional clamping); contradicts
    A5's "minimal" phrasing outright.

**Decision:** Option A (per-kind tag) + Option A (exactly 1 child)
+ Option A (hardcode vertical-only). Smallest IR / spec surface
that satisfies A5 verbatim. Each Phase-4-non-shipped surface
(N-child synthesis, horizontal axis, both-axis) is additively
expressible in a later phase without breaking the Phase 4
contract.

**Layering with DD-002 / DD-005.** The 1-child rule names *what*
a ScrollView contains; DD-002 settles the viewport bound; DD-005
names *how* the content is measured against that bound. An Option
that admitted N>1 children with implicit synthesis would not
contradict DD-005 structurally (the synthetic wrapper would
become the single content), but would push wrapper-defaulting
semantics into the spec for no A5 requirement. An Option that
inferred scroll axis from parent shape would contradict DD-005's
normative requirement that the scroll axis is a static IR
property.
