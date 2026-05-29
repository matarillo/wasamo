### DD-P3-001 — Layout algorithm

**Status:** Accepted

**Context:**
Phase 3 introduces the layout engine responsible for computing the position
and size of each widget in the Visual Layer. The engine must support VStack,
HStack, and Rectangle for M1. Two credible approaches exist: a custom
measure/arrange two-pass model, and adopting an existing Rust layout crate.

**Options:**

Option A — Custom two-pass measure/arrange
- What you gain: No new runtime DLL dependency (consistent with the policy
  in `architecture.md §4`). Algorithm is well-understood; WPF, UWP, SwiftUI,
  and Flutter all use this model. M1's layout primitives (two stack types and
  a rectangle) require only a small subset of the full algorithm — the
  implementation surface is bounded and auditable.
- What you give up: Custom code that must be maintained. When M2 introduces
  Grid and ScrollView, complexity will grow.

Option B — Taffy (Rust-native flexbox/grid crate)
- What you gain: Proven algorithm covering flexbox, grid, and block layout.
  Reduces the amount of layout logic to maintain. Rust-native with no C FFI.
- What you give up: Adds a dependency to `wasamo` (runtime DLL), which
  requires explicit case-by-case approval per `architecture.md §4`. Taffy's
  flexbox model does not map idiomatically to VStack/HStack — a translation
  layer is needed. Its full capabilities are unused in M1.

**Decision:** Option A — custom two-pass measure/arrange for M1.
Taffy is a credible candidate for M2 when Grid and ScrollView are introduced;
that adoption decision will be made in the M2 pre-document.

---
