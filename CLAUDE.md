# Wasamo — Project Conventions for Claude

## Language rules

- All files under `docs/` must be written in **English**, with one
  exception: `docs/notes/` may be in Japanese (owner-authored exploratory
  notes — see `docs/notes/README.md`).
- Conversation with the project owner (chat) is in Japanese.
- Code comments: English only.
- Commit messages: English only.

## Document categories under `docs/`

- `docs/decisions/` — ADRs. Per-phase design decisions, agreed and
  immutable (revisions follow the supersede rule). See its README.
- `docs/plans/` — Milestone plans. Upstream agreement artifacts that
  feed into ROADMAP and ADRs. Frozen once `status: in-progress`. See its
  README for lifecycle and archival policy.
- `docs/notes/` — Owner-authored exploratory notes and live open
  questions. Japanese allowed. See its README.

When information settles into a decision, it moves: notes → ADR. When a
milestone is committed, structure moves: plan → ROADMAP. Each category
has a single role; do not duplicate content across them.

## Testing rules

Unit tests are only appropriate for logic that has **no Win32/WinRT FFI dependencies**.

- Pure Rust logic (parsers, layout algorithms, coordinate math): write unit tests.
- Win32/WinRT code (window creation, Compositor, Visual Layer, DirectWrite): do **not** mock the OS API surface. Correctness is verified by the CI Windows runner building and running the code.
- Mock-free Windows-only integration tests that use the real OS runtime
  surface are allowed. They are not unit tests. When such a test is used
  as CI-gated evidence, it should fail rather than silently skip on GitHub
  Actions if the required runtime capability is unavailable.

Adding unit tests to a phase checklist is only warranted when that phase introduces testable pure logic. Do not add unit test checklist items to phases whose work is entirely Win32/WinRT (e.g. Phase 2, Phase 5).

When pure logic is entangled with a Win32/WinRT-bound type (e.g. a struct whose constructor requires a live `Compositor`), you **may** introduce a test-module-only mirror struct that replicates the index/state logic without the OS dependency. Use this sparingly — only when the logic is non-trivial enough to warrant a test and cannot be exercised through pure free functions. Prefer extracting the logic into a free function first; reach for the mirror pattern only when extraction would distort the production type's API.

## Commit rules

Default to one commit per task-list item in the active ADR / plan. This
default may be deviated from when:

- Bundling is required to keep the build/tests passing at every commit
  (a single item that spans multiple files where intermediate states
  do not build).
- Implementation reveals that an item should be split or reordered
  (e.g. a sub-issue surfaces, CI reports a new failure mode, a
  dependency between items is discovered).

When deviating, update the task list in the ADR/plan to reflect what
actually happened, so the document remains an accurate record rather
than a frozen prediction. Plan changes mid-implementation are normal
and expected; the rule exists to keep history reviewable, not to
freeze the plan.

## CI rules

Add a "update CI" checklist item only when a phase introduces a **new language or build system** (e.g. Zig, CMake/C). Phases that add Rust code to existing crates need no CI update — `cargo build --release --workspace` and `cargo test --workspace` already cover them.

## Build ordering requirements

The DSL examples consume IR text emitted by `wasamoc` at host build
time. Builds in the C and Zig host build systems shell out to
`wasamoc.exe`, so:

- **`cargo build -p wasamoc` must succeed before building
  `examples/counter-c/` (CMake) or `examples/counter-zig/`
  (`zig build`).** Both build systems expect `wasamoc.exe` to be
  available at a known location (typically `target/release/wasamoc.exe`
  or `target/debug/wasamoc.exe`).
- **`cargo build -p counter-rust` does not require a separate
  `wasamoc` build step**: its `build.rs` declares a workspace-internal
  build dependency on `wasamoc` and uses `cargo:rerun-if-changed` to
  recompile when `examples/counter/counter.ui` or `wasamoc` itself
  changes.
- **Workspace-wide builds** (`cargo build --release --workspace`)
  build `wasamoc` as part of the graph, so they implicitly satisfy
  the ordering for `counter-rust`. C and Zig hosts still need an
  explicit prior `wasamoc` build because they live outside the cargo
  workspace's build graph.

This pipeline is provisional — see
[docs/architecture.md §1 "DSL build pipeline"](docs/architecture.md#dsl-build-pipeline-m2-phase-6-onward)
for the re-evaluation triggers (M3 multi-`.ui` host, post-1.0 hot
reload).
