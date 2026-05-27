### DD-P3-004 — Cross-axis alignment

**Status:** Accepted

**Context:**
Stacks have a main axis (VStack: vertical, HStack: horizontal) and a cross
axis. The `alignment` property controls how children are positioned on the
cross axis.

**Options:**

Option A — `Stretch` only; no runtime property in M1
- Children on the cross axis expand to fill the stack's cross-axis size.
- What you gain: No new API surface for M1.
- What you give up: Cannot center or trailing-align children without nesting
  workarounds. Forces a refactor in Phase 4 when Text and Button will
  commonly need centered layout.

Option B — Expose `alignment: Leading | Center | Trailing | Stretch`
- What you gain: Covers the common centering use case. Consistent with
  `spacing` and `padding` already in the Phase 3 API surface.
- What you give up: Small additional implementation surface.

**Decision:** Option B — expose `alignment` with four values.
`Stretch` is the default when not specified. This avoids a forced refactor
when Phase 4 introduces Text and Button.

---
