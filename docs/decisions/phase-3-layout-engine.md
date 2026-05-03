# Phase 3 — Layout Engine: Architecture Decisions

**Phase:** 3 (Layout Engine)
**Date:** 2026-04-28
**Status:** Accepted and implemented

---

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

### DD-P3-002 — LayoutNode ownership and handle model

**Status:** Accepted

**Context:**
The layout engine builds an internal tree of `LayoutNode` structs. The C ABI
must give the host language a stable reference to each node so it can update
properties and trigger re-layout. The question is who owns the node memory
and what the host receives.

**Options:**

Option A — Engine owns nodes; host receives opaque handles (`WasamoWidget*`)
- What you gain: Memory management is entirely within the runtime. The host
  cannot corrupt the tree by misusing a raw pointer. Consistent with the
  opaque pointer model already planned for Phase 6 (C ABI).
- What you give up: The host must call an explicit destroy function. Language
  bindings must wrap the handle to trigger destroy on drop (Rust RAII, Zig
  `defer`, etc.).

Option B — Nodes are value types allocated by the host
- What you gain: Host language controls lifetime without a destroy call.
- What you give up: Impossible to implement correctly across a C ABI boundary
  — the runtime must walk the tree internally, which requires stable addresses
  under its own control.

**Decision:** Option A — engine owns node memory; host holds opaque
`WasamoWidget*` handles. No new ABI type is introduced beyond what Phase 6
already plans.

---

### DD-P3-003 — Size constraint model

**Status:** Accepted

**Context:**
Each widget must declare how it occupies space on each axis. The model
determines how VStack/HStack distribute space among children, and how
Rectangle declares its size.

**Options:**

Option A — Three-value enum: `Fixed(f32)`, `Fill`, `Shrink`
- `Fixed(f32)`: explicit pixel size on the given axis.
- `Fill`: expand to consume remaining space after fixed-size children are
  placed. Multiple `Fill` siblings divide remaining space equally.
- `Shrink`: wrap to content (sum of children + spacing + padding for stacks;
  explicit dimensions for Rectangle).
- What you gain: Simple to implement. Covers all M1 DSL examples. Maps
  naturally onto the SwiftUI/Flutter size-model mental model.
- What you give up: No min/max constraints. A `Fill` child inside a `Shrink`
  parent resolves to zero size — a degenerate case that must be documented.

Option B — Min/max constraint system (CSS-style)
- What you gain: Expressive; handles edge cases gracefully.
- What you give up: Significantly more complex. All M1 use cases are
  expressible with Option A.

**Decision:** Option A — `Fixed / Fill / Shrink` three-value model for M1.
Min/max constraints deferred to M2 or later.

**Implementation note (post-implementation):**

In the `measure()` pass, `Fill` returns `0.0` on that axis — it signals "I will take whatever
the parent allocates" rather than declaring a demand. The parent resolves the final size during
`arrange()` by dividing remaining space equally among `Fill` siblings. This means a `Fill`
child inside a `Shrink` parent receives `0.0` (remaining = 0, clamped; per DD-P3-005). This
is documented behaviour, not an error.

Default size values per widget type:

| Widget | Width default | Height default |
|---|---|---|
| `VStack` | `Fill` | `Shrink` |
| `HStack` | `Shrink` | `Fill` |
| `Rectangle` | `Fixed` (caller must specify) | `Fixed` (caller must specify) |

A `Rectangle` with no explicit dimension is treated as an API error
(see DD-P3-005).

---

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

### DD-P3-005 — Error handling strategy

**Status:** Accepted

**Context:**
The layout engine can encounter two categories of failure:

1. **API errors**: the host calls an API incorrectly (null handle, invalid
   parent/child relationship, Rectangle with no explicit dimension).
2. **Layout errors**: a size constraint is degenerate (e.g., `Fill` child
   inside a zero-size `Shrink` parent), producing a zero or negative extent.

**Options:**

Option A — All errors are fatal (return error code; abort layout on any failure)
- What you gain: Deterministic — no silent fallbacks.
- What you give up: A single bad widget crashes the entire tree's layout.
  Unacceptable for a UI runtime.

Option B — Split strategy: API errors strict; layout errors resilient
- API errors: return an error code immediately. The host is responsible.
- Layout errors (degenerate constraints, zero-size fill): clamp to 0.0,
  no error returned. The affected subtree renders at zero size; the rest
  of the tree is unaffected.
- What you gain: Matches how WPF, UWP, and SwiftUI handle bad constraints —
  graceful degradation rather than a process crash.
- What you give up: Degenerate layouts are silent in M1 (no runtime warning).

**Decision:** Option B — split strategy.
API error codes reuse the `int` return convention from Phase 2.
Degenerate layout dimensions clamp to 0.0 without surfacing an error.
