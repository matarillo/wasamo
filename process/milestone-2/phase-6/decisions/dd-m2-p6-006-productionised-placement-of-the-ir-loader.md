### DD-M2-P6-006 — Productionised placement of the IR loader

**Status:** Accepted

**Context:**
The Phase 2 spike's `wasamo-runtime/src/experimental_ir_loader.rs`
is feature-gated and not part of the default build. Phase 6 makes
the loader load-bearing on M2 acceptance; the question is where
the loader lives in the workspace and what becomes of the
experimental flag.

The malformed-IR validation policy (how defensively the loader
treats input) is decided separately in DD-M2-P6-009 because it
has direct ABI-error-surface impact; this DD cross-references it.

**Options:**

Option A — Inside `wasamo-runtime`, replacing experimental loader (recommended)
- Move loader implementation to `wasamo-runtime/src/ir_loader.rs`
  (or split into a submodule). Remove the
  `experimental_ir_loader` feature flag.
- What you gain: smallest workspace change; the loader lives
  with the runtime types it constructs; single-crate build
  story unchanged for hosts.
- What you give up: future "load IR without instantiating
  runtime" use cases (hot reload pre-loading, IR
  pretty-printer) build into the runtime crate; acceptable
  for M2 since neither use case is in scope.

Option B — Split into `wasamo-loader` crate
- New crate; runtime depends on it.
- What you gain: loader can be used standalone (e.g. for
  diagnostic tools); separation of concerns.
- What you give up: an additional crate to version, build,
  and document; the loader and runtime types are tightly
  coupled (loader constructs runtime types directly), so the
  split would either leak runtime internals or force a thin
  abstraction layer with no current consumer.

**Recommendation:** **Option A**, removing the
`experimental_ir_loader` feature flag.

A single-crate placement matches every M2-acceptance use case
and keeps the loader colocated with the types it builds. The
feature flag was always temporary (Phase 2 spike); removing it
on production-ising is the simplest end state. If a standalone
loader becomes necessary (e.g. for a diagnostic tool), B is
additive.

**Forward-compat exposure:**

- Out-of-scope items engaged: post-1.0 hot reload, M5
  diagnostic tooling.
- A is additive on both: hot reload reuses the same loader
  through repeated calls; diagnostic tooling, when it lands,
  motivates the split (B) — at which point the cost is paid
  for a real consumer.
- B paid up-front carries the maintenance cost without an M2
  consumer.

**Technical-risk re-evaluation:** A is the lower-risk choice;
the experimental code is already present in the runtime crate.
Risk reinforces A.

---
