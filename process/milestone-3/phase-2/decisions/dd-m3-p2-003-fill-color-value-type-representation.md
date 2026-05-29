### DD-M3-P2-003 — `fill: <color>` value-type representation

**Status:** Accepted

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

Option A — Color value type carries alpha (recommended)
- The Phase 2 color value carries four 8-bit channels (r, g, b, a).
  The Phase 2 surface syntax (below) admits `#RRGGBBAA`. The value
  layer (a Box-internal `Color` domain type — see the variant-strategy
  sub-issue below) is built once and reused as bindable / alpha-styling
  controls land in later phases. M3 styling remains out of scope per
  the target-app pre-doc.

  - What you gain: A6's scrim use case is expressible. The
    target-app pre-doc's internal tension resolves: "M3 fills can
    carry alpha; M3 styling does not expose alpha control beyond the
    literal hex form" — a one-line spec statement.
  - What you give up: 32 bits per color value vs 24. Negligible.
  - **Technical risk:** Low.

Option B — Color value type excludes alpha; M3 scrim is opaque-by-spec
- The color value carries (r, g, b). Surface admits `#RRGGBB` only.

  - What you give up: A6's scrim wording is internally inconsistent
    as recorded above. M4+ alpha-styling adoption forces a value-
    layer revision (color widened from 3 to 4 channels at that point,
    with every existing `#RRGGBB` literal becoming an implied
    `alpha = 0xFF`). The revision is mechanically additive, but it's
    a revision Option A avoids.

**Options (variant strategy — consequent on Option A above):**

Option A — Box-internal `Color(u32)` domain type; **not** added to
`PropertyValue` in Phase 2 (recommended)
- A new private domain type `Color(u32)` lives in `wasamo-runtime`
  (packed `u32` in `0xAARRGGBB` layout, alpha in the most
  significant byte; recorded in
  [dsl_spec.md §8.2](../dsl_spec.md#82-notation) `COLOR` token).
  `WidgetData::Box` stores
  `fill: Option<Color>` as a Box-internal field. `IrLiteral::Color(u32)`
  parallel in `wasamo-ir`. `wasamoc` lexer accepts `#RRGGBB` (alpha
  implied 0xFF) and `#RRGGBBAA`. `ir_loader` materialises
  `IrLiteral::Color` directly into the Box's internal field; the
  value never traverses `PropertyValue`-mediated paths.
  `PropertyValue` is **not** widened with a `Color` variant in Phase
  2, and `WASAMO_VALUE_COLOR` is **not** added. The
  [predoc-inputs.md §1](../notes/m3-phase-2/predoc-inputs.md#1-box-が新規-propertyvalue-variant-を入れるなら-abi-value-conversion-arm-は同じ-step-に-fold-する)
  fold-in-same-step obligation triggers when `PropertyValue` gains a
  variant; Phase 2 satisfies it by not adding a variant. See DD-002's
  IR / runtime plumbing block for the symmetric `Ratio` treatment.

  - What you gain: Type-tagged at the runtime domain layer (not a
    coerced string). No string-parsing at the widget setter. Phase 2
    surface stays narrow: literal attribute exists but is not a
    property / binding / ABI surface, matching DD-004's
    constant-only intent. ABI surface (`read_property_value` /
    `write_property_value` / `property_value_to_owned` /
    `WASAMO_VALUE_*` tag space) is untouched.
  - What you give up: Two seams to build later (`PropertyValue::Color`
    + ABI tag) instead of one, **if and when** bindable `fill` lands.
    This is a deliberate phase-deferred cost: the seam is built in
    the phase that uses it (Phase 1 discipline; symmetric with
    DD-M3-P1-007's "build the seam for `Button.enabled`, not
    pre-built for unknown later callers").
  - **Technical risk:** Low.

Option B — New `PropertyValue::Color(u32)` + `WASAMO_VALUE_COLOR` ABI
tag in Phase 2 (rejected)
- `PropertyValue::Color` carries packed `u32`; `WASAMO_VALUE_COLOR`
  tag added to the C ABI's value union; `read_property_value` /
  `write_property_value` / `property_value_to_owned` arms folded into
  the same step per
  [predoc-inputs.md §1](../notes/m3-phase-2/predoc-inputs.md#1-box-が新規-propertyvalue-variant-を入れるなら-abi-value-conversion-arm-は同じ-step-に-fold-する);
  `abi_spec.md` updated.

  - What you give up: Widens the public ABI surface ahead of the
    bindable-`fill` surface that DD-004 explicitly defers to a later
    phase. The Phase 2 sub-screen exercises neither bindable nor
    ABI-visible `fill`, so the additional surface is speculative.
    Conflicts with DD-004's "constant-only, no new per-type writer
    seam" stance from the value-layer side.

Option C — Reuse `PropertyValue::Str` with `#RRGGBB[AA]` parsing at
the loader (rejected)
- No new variant; the loader parses the hex string when reading the
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

**Recommendation:** Alpha **included**; new `IrLiteral::Color(u32)`
plus a Box-internal `Color(u32)` domain type in `wasamo-runtime`
(stored on `WidgetData::Box`, **not** added to `PropertyValue` in
Phase 2; no `WASAMO_VALUE_COLOR` tag, no `abi.rs` changes); surface
forms `#RRGGBB` and `#RRGGBBAA` only. The alpha-yes decision is the
load-bearing one — without it, A6's scrim wording is internally
inconsistent. The "Box-internal, not PropertyValue" boundary is the
implementation-boundary consequence of pairing alpha-yes with DD-004's
constant-only stance: it keeps the ABI surface (DD-002 IR / runtime
plumbing block) untouched in Phase 2. This boundary is a consistency
choice, not a separate load-bearing yes/no for the owner — Checkpoint 2
asks only the alpha question.

The Phase 2 styling commitment remains: *the value layer carries
alpha; the M3 styling layer does not gain alpha-styling controls
beyond the literal hex form*. Theming, palette, and dynamic alpha
adjustment all remain M4+ work (per
[m3-plan.md §Out of scope](../plans/m3-plan.md#out-of-scope-deferred-to-later-milestones)
and target-app pre-doc Visual / styling Out-of-scope).

**Forward-compat exposure:** Option A (alpha-yes, Box-internal) is
dual-compatible with both possible future events:

- If M4+ theming admits alpha control, the value layer already
  carries alpha — theming binds in semi-transparent values without
  value-layer revision. The bindable-`fill` phase lands
  `PropertyValue::Color(Color)` + `IrType::Color` +
  `HandlerExpr::ColorLit` / `ColorPropRead` +
  `WASAMO_VALUE_COLOR` ABI tag + `abi.rs` arms together; the Phase 2
  `Color` domain type is the type that gets wrapped, not revised.
- If M4+ theming explicitly forbids alpha at the styling layer, the
  value layer still carries alpha for the structural scrim case
  Phase 2 needs; styling restricts at its layer without value-layer
  revision.

Option B (alpha-no) requires a value-layer revision under the
former event and is "scrim cannot be expressed" under either.

---
