### DD-M2-P6-008 — Migration shape for `examples/counter-{c,rust,zig}`

**Status:** Accepted

**Context:**
M2 acceptance A1 replaces the per-language imperative tree
construction in `examples/counter-{c,rust,zig}/` with hosts that
load `counter.ui` through the agreed pipeline. Two coupled
sub-questions: per-language wrapper API shape, and whether `.ui`
sources are shared or copied per language.

Resource-resolution form is decided in DD-M2-P6-005 (recommended:
absolute path or compile-time embedded blob); this DD picks how
each example uses it.

**Options:**

Option α — Per-language wrapper API: thin direct call
- Each example calls `wasamo_load_ui` directly through its
  language's existing C-ABI binding. No new helper.
- What you gain: smallest example surface; demonstrates the
  raw ABI; binding-author audience sees exactly what their
  binding must expose.
- What you give up: counter examples carry slightly more
  boilerplate (resource path setup) than a polished helper
  would.

Option β — Per-language wrapper API: language-idiomatic helper
- Each binding crate (`wasamo` for Rust, `wasamo.h` for C,
  `wasamo.zig` for Zig) provides a small idiomatic helper
  (e.g. Rust `Wasamo::load_ui_file(path)`).
- What you gain: examples read more naturally; community
  binding authors see a target shape.
- What you give up: helper API surface this ADR creates and
  Phase 6 must specify per language.

**Resource location:**

- (X) Single shared `examples/counter/counter.ui`, all three
  hosts load it (path resolved per Option A or C in
  DD-M2-P6-005).
- (Y) Per-language copies under `examples/counter-c/counter.ui`,
  etc.

**Recommendation:** **Option α with shared (X)**, plus
compile-time embedding (DD-M2-P6-005 = C) for the C and Zig
examples; absolute path (DD-M2-P6-005 = A) for the Rust
example.

α exposes the ABI cleanly to the binding-author audience this
M2 deliverable targets; idiomatic helpers (β) belong in M3
when the wrapper crates' broader API is being designed.
A single shared `.ui` (X) is the canonical "DSL drives all
hosts" demonstration and removes per-host drift risk in copies.

The Rust example uses path-loading because Rust's `cargo run`
ergonomics already point at the workspace; C and Zig use
embedded `.ui` because their build systems make embedding
ergonomic and the resulting binary is self-contained — exactly
the binding-style M3 community bindings will inherit.

**Forward-compat exposure:**

- Out-of-scope items engaged: M3 list/grid examples (additional
  examples), post-M2 search-path / resource-bundle features.
- α + X is additive on both axes: M3 examples add new
  directories; resource-bundle features (when they land)
  augment DD-M2-P6-005's resource-resolution choices, which
  these examples consume rather than define.
- β codifies a per-language helper API before M3 designs the
  full wrapper crates; revisitation likely.

**Technical-risk re-evaluation:** α + X is the lowest-risk
choice (no new API surface); risk reinforces it.

---
