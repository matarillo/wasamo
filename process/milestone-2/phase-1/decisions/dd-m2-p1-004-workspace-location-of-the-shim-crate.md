### DD-M2-P1-004 — Workspace location of the shim crate

**Status:** Accepted

**Context:**
The workspace currently has crates at top level (`wasamo-runtime/`,
`wasamoc/`) and grouped under `bindings/` and `examples/`. The shim
crate needs a home. A secondary question arose during review: should
the project adopt a `crates/` root directory to follow a pattern used
by some larger Rust workspaces?

**Options:**

Option A — Top-level `wasamo-dll/` (recommended)
- Sibling of `wasamo-runtime/`. Top-level placement matches the
  other "produces a build artifact at the project's name level"
  crates.

  - What you gain: Discoverability; consistent with `wasamo-runtime/`
    and `wasamoc/`.
  - What you give up: One more entry at the workspace root. The
    root listing is not yet so crowded that one more matters.

Option B — Nested under `wasamo-runtime/dll-shim/`
  - What you gain: Physical containment expresses the dependency.
  - What you give up: Unusual in cargo workspaces; inversion of the
    dependency direction (shim depends on runtime, not vice versa).

Option C — `crates/wasamo-dll/`
- Introduce a `crates/` directory and put the shim there.

  - What you gain: Conventional in some Rust monorepos; signals
    "internal crates" vs. `bindings/` and `examples/`.
  - What you give up: Inconsistent with the existing layout; either
    requires moving all other crates into `crates/` (broad churn
    out of M2-Phase 1 scope) or leaves `wasamo-dll` as the sole
    resident of a new directory (asymmetry). Deciding this now
    entangles a workspace-layout open question with an unrelated
    phase.

**Decision:** Option A — Accepted conditionally (2026-05-03). The
`crates/` pattern (Option C) would only make sense if all crates
migrated together. That is a separate workspace-layout decision,
not part of this phase. The open question — whether a future
`crates/` reorganisation is warranted — is recorded in
[`docs/notes/workspace-layout.md`](../../../../docs/notes/workspace-layout.md)
as a live note. If the project reaches the point where it decides to
adopt `crates/`, M2-Phase 1's placement of `wasamo-dll/` can be
addressed in the same migration commit.

---
