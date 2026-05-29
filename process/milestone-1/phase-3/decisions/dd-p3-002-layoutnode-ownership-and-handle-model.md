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
