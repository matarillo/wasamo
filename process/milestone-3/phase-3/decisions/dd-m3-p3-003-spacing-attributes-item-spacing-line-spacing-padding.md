### DD-M3-P3-003 — Spacing attributes (item-spacing, line-spacing, padding)

**Status:** Accepted

**Context:**
Whether Phase 3 exposes item spacing (main-axis gap between siblings
within a line), line spacing (cross-axis gap between lines), padding
(inset between WrapPanel bounds and the line set), or none.

Spacing is the *gap between* items, not the *size of* items. The
item-size source is DD-004's question; once items have a size, spacing
decides what visible gap separates them. The wireframe's 12px gap
(see [docs/references/m3-gallery-wireframe.html](../references/m3-gallery-wireframe.html))
is therefore a DD-003 question conditional on DD-004 settling the
88×88 thumbnail extent. If DD-004 ships zero item-sizing attribute,
DD-003's spacing has no thumbnails to space.

**Options (item-spacing and line-spacing — surface scope):**

Option A — Ship `item-spacing: <i32>` and `line-spacing: <i32>` as
constant-only attributes, default `0` (recommended)
- Two new optional attributes on the WrapPanel IR node, both
  carrying `i32` pixel values. `item-spacing` separates siblings
  within a line on the main axis; `line-spacing` separates lines
  on the cross axis. Defaults: 0 (touching items / lines if
  unset). `wasamoc check` rejects negative literals; `validate()`
  rejects negative IR (DD-006). No new `IrType`, no new
  `IrLiteral` variant — `i32` plumbing already exists from M2 /
  Phase 1.

  - What you gain: Reuses existing `i32` plumbing — lex, parse,
    check, IR literal form, runtime decode all unchanged from M2.
    The Phase 3 gallery sub-screen can express the wireframe's 12px
    gap (`item-spacing: 12; line-spacing: 12`). Default 0 is
    visible-by-construction (touching items if unset, not silent
    invisibility).
  - What you give up: Two new attributes — but each is an additive
    optional attribute, not a structural surface change.
  - **Technical risk:** Low.

Option B — Ship neither; Phase 3 sub-screen accepts touching thumbnails
- WrapPanel has no spacing attributes. The gallery sub-screen
  visually deviates from the wireframe (no gaps between thumbnails)
  but the layout primitive ships smaller.

  - What you give up: Visible deviation from the wireframe in the
    sub-screen. The deviation is recoverable (a later phase adds
    spacing), but the gallery proof becomes "WrapPanel wraps" rather
    than "WrapPanel produces the wireframe's layout". The
    presence-of-gap question is structural to author intuition about
    "what does a WrapPanel look like"; deferring it creates an
    expectations-mismatch for any reader coming from CSS / WPF / XAML
    where gap is table stakes.

Option C — Ship `item-spacing` only; defer `line-spacing` to a later phase
- One attribute, not two. The wireframe's cross-axis gap between
  lines is unreachable in Phase 3.

  - What you give up: Visual asymmetry — main-axis gaps present,
    cross-axis gaps absent. The wireframe shows both. No spec
    saving relative to Option A — the lex / parse / check for the
    second attribute is mechanical.

Option D — Single combined `spacing: <i32>` attribute
(applies to both axes uniformly)
- One attribute setting the same value on both axes.

  - What you give up: Forces authors with mixed-axis gap intent
    (e.g. tight horizontal grid, looser vertical separation) to
    pick one. A later phase that splits the attribute then has a
    deprecation problem.

**Options (surface form, conditional on Option A):**

Option A — Bare integer literal `item-spacing: 12` (recommended)
- Per Phase 2 DD-M3-P2-002 / DD-M3-P2-003 discipline, no new
  `PropertyValue` variant unless the attribute is bindable —
  integer pixel spacing reuses `IrLiteral::Int` and the existing
  `i32` plumbing.

  - **Technical risk:** Low.

Option B — Pair literal `spacing: 12 12` (main cross)
- A structural pair literal, akin to DD-002's ratio form.

  - What you give up: New literal form for one attribute that has
    no obvious "the two values are inseparable" reading. The
    wireframe's gaps are equal but conceptually two different
    distances. Reject as over-engineering for the value shape.

**Options (bindable surface, conditional on Option A):**

Option A — Constant-only in Phase 3 (recommended)
- Phase 3 mirrors DD-002's stance and Phase 1 / Phase 2 seam-
  building discipline. Phase 3 sub-screen has no animated spacing
  use case.

  - What you gain: No new per-type writer pair built. Phase 3 reuses
    the existing `i32` literal plumbing; a future bindable-spacing
    phase either reuses the M2 string-baked `register_binding` path
    that `IrType::I32` properties currently dispatch to, or opens a
    typed-`i32` evaluator/writer pair if that phase warrants it.
    Phase 3 itself adds no engine plumbing.
  - **Technical risk:** Low.

Option B — Admit bindable spacing in Phase 3
- `bind item-spacing: <state-of-int>` works.

  - What you give up: Speculative seam *registration* for an
    attribute no Phase 3 sub-screen exercises reactively. Phase 1
    discipline argues against.

**Options (padding):**

Option A — Defer padding to a later phase (recommended)
- WrapPanel has no padding attribute in Phase 3. The Phase 3
  sub-screen accepts whatever left-edge behaviour the bare WrapPanel
  default produces (children flush with WrapPanel's main-axis-start
  edge).

  - What you gain: Smaller Phase 3 surface. Padding is a parent /
    container concept that interacts with Phase 4 ScrollView's clip
    surface and any future M4+ layout-with-margin work; settling
    its semantics deserves its own phase. The wireframe's left-edge
    margin (x=36 in a 20-padded frame) is achievable in Phase 3 by
    wrapping the WrapPanel in an outer HStack with a spacer — not
    elegant, but it ships A3 without expanding scope.
  - What you give up: The sub-screen has no first-class way to
    express the wireframe's left-edge margin; either it visually
    deviates or it composes with an outer wrapper. Framing decision E
    accepts the visual deviation as in-scope.

Option B — Ship `padding: <i32>` (uniform inset) in Phase 3
- One scalar inset applied to all four edges.

  - What you give up: Commits Phase 3 to a uniform-inset reading;
    a later phase that needs per-edge padding (`padding-left`,
    `padding-top`, …) either re-spec's the attribute or coexists
    awkwardly. Better to defer the whole question.

Option C — Ship 4-tuple padding `padding: <top> <right> <bottom> <left>`
- CSS-style four-edge padding.

  - What you give up: New literal form (a 4-tuple) for an attribute
    no Phase 3 sub-screen requires. Defer.

**Recommendation:** Option A for every sub-issue —

- Ship `item-spacing: <i32>` and `line-spacing: <i32>` as
  constant-only attributes; default 0 for both.
- Surface form: bare integer literal; reuses existing `i32`
  plumbing; no new `PropertyValue` variant.
- Bindable: constant-only in Phase 3.
- Padding: defer.

The spacing question turns on the wireframe-fidelity tension and the
surface-cost asymmetry. Option A pays a small attribute-plumbing cost
(two `i32` attributes, both reusing existing plumbing) for full
wireframe fidelity; Option B saves the plumbing at the cost of a
visibly-degraded sub-screen. The framing's working direction is Option
A, but the trade-off is a value judgement; **see Owner-agreement
checkpoint 1**.

**Forward-compat exposure:** Option A is dual-compatible with both
foreseeable future events:

- A future bindable-spacing phase admits binding for the attribute
  at that point. It can reuse the M2 string-baked `register_binding`
  path that `IrType::I32` properties currently dispatch to, or open
  a typed-`i32` evaluator/writer pair if that phase warrants it;
  no revision of the Phase 3 IR shape or the spacing semantics is
  required.
- A future padding-introducing phase adds a separate attribute (or
  attribute group); the absence of padding in Phase 3 does not
  pressure the eventual padding surface to be backward-compatible
  with any Phase-3 convention.

Option B / C / D would have committed Phase 3 to surface shapes
(no spacing / one-axis-only spacing / combined spacing) that either
demand visible-deviation acceptance or constrain a later split.

---
