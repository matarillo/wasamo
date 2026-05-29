### DD-P4-002 — Font property model

**Status:** Accepted

**Context:**
`Text` needs a `font` property. The DSL spec (`docs/dsl_spec.md`) already
uses `font: title` in the Counter example. The question is how far to
formalise the font API for M1.

**Options:**

Option A — Semantic enum (`TypographyStyle`) mapping to Windows type ramp
- Define `TypographyStyle` as an enum with four values for M1:
  `Caption` (12 sp, regular), `Body` (14 sp, regular),
  `Subtitle` (20 sp, semi-bold), `Title` (28 sp, semi-bold).
  Each variant maps to Segoe UI Variable with the corresponding size and
  weight, matching the WinUI 2 / WinApp SDK typography tokens and the DSL
  example syntax.
  The name `TypographyStyle` is preferred over `FontStyle` because
  `FontStyle` conventionally denotes the posture axis (Normal / Italic /
  Oblique), not the semantic size-and-weight scale.
- What you gain: DSL `font: title` maps directly to `TypographyStyle::Title`.
  DPI-aware sizing is managed by the type-ramp constants. Consistent with
  the platform visual language.
- What you give up: Custom font families and arbitrary point sizes are not
  expressible in M1. A larger font vocabulary must wait for M2.

Option B — Explicit font descriptor (`family: String, size: f32, weight: u16`)
- What you gain: Flexible; any system font is available.
- What you give up: More verbose; requires richer DSL syntax
  (`font: { family: "Segoe UI", size: 14 }`); DPI scaling becomes the
  caller's problem.

**Decision:** Option A — four-value `TypographyStyle` enum for M1.
Custom descriptors deferred to M2. This matches the DSL example and keeps
the API surface small.

---
