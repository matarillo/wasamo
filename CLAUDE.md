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

Adding unit tests to a phase checklist is only warranted when that phase introduces testable pure logic. Do not add unit test checklist items to phases whose work is entirely Win32/WinRT (e.g. Phase 2, Phase 5).

## CI rules

Add a "update CI" checklist item only when a phase introduces a **new language or build system** (e.g. Zig, CMake/C). Phases that add Rust code to existing crates need no CI update — `cargo build --release --workspace` and `cargo test --workspace` already cover them.
