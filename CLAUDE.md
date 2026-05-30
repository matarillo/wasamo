# Wasamo — Project Conventions for Claude

## Language rules

- All files under `docs/` must be written in **English**, with one
  exception: `docs/notes/` may be in Japanese (owner-authored exploratory
  notes — see `docs/notes/README.md`).
- Conversation with the project owner (chat) is in Japanese.
- Code comments: English only.
- Commit messages: English only.

## Document structure

### `process/` — Development process artifacts

Organized by milestone and phase. See `process/README.md` for full
structure.

- `process/_roadmap.md` — Overall milestone roadmap (SSOT for
  acceptance criteria).
- `process/cross-milestone/decisions/` — Vision decision records
  (covers vision / governance / policy / roadmap): doc system,
  RFC policy, DSL policy, process rules.
- `process/milestone-N/` — Per-milestone artifacts:
  - `plan.md` — Milestone execution plan. Frozen once
    `status: in-progress`.
  - `handoff.md` — Cross-phase design prerequisites and residuals;
    written at milestone close.
  - `requirements/` — Milestone-level scope, spec, and wireframes.
  - `phase-M/requirements/` — Phase scope and constraints
    (`framing.md`, `constraints.md`); produced before ADR drafting.
  - `phase-M/decisions/` — ADRs. `preamble.md` holds Context/Summary/
    Out of scope/Revisions; `dd-NNN-*.md` per decision. Immutable
    (revisions follow the supersede rule).
  - `phase-M/implementation/` — `plan.md` (task checklist), `log.md`
    (decisions + CI log), `handoff.md` (items to carry to next phase).
    Mutable during the phase.
  - `phase-M/retrospectives/` — `phase-end.md` + `tN.md` per task.

When information settles into a decision, it moves: notes →
`decisions/`. When a milestone is committed, structure moves: plan →
`_roadmap.md`. Each document type has a single role; do not duplicate
content across them.

### `docs/` — Technical reference and exploratory notes

- `docs/notes/` — Owner-authored exploratory notes and live open
  questions. Japanese allowed. See its README.
- `docs/architecture.md`, `docs/abi_spec.md`, `docs/dsl_spec.md` —
  Normative technical specifications. English only.

## Testing rules

Unit tests are only appropriate for logic that has **no Win32/WinRT FFI dependencies**.

- Pure Rust logic (parsers, layout algorithms, coordinate math): write unit tests.
- Win32/WinRT code (window creation, Compositor, Visual Layer, DirectWrite): do **not** mock the OS API surface. Correctness is verified by the CI Windows runner building and running the code.
- Mock-free Windows-only integration tests that use the real OS runtime
  surface are allowed. They are not unit tests. When such a test is used
  as CI-gated evidence, it should fail rather than silently skip on GitHub
  Actions if the required runtime capability is unavailable.
- Before landing such a test, verify its skip-guard actually triggers on
  an environment where the required runtime capability is missing (e.g.
  an SSH dev box where `wasamo_init` returns `0x80070005`). Local
  "passed without skip" only proves the guard's happy path doesn't break
  the test — it does not prove the guard works. See
  [docs/notes/verification-environments.md](docs/notes/verification-environments.md)
  for the environment taxonomy.

Adding unit tests to a phase checklist is only warranted when that phase introduces testable pure logic. Do not add unit test checklist items to phases whose work is entirely Win32/WinRT (e.g. Phase 2, Phase 5).

When pure logic is entangled with a Win32/WinRT-bound type (e.g. a struct whose constructor requires a live `Compositor`), you **may** introduce a test-module-only mirror struct that replicates the index/state logic without the OS dependency. Use this sparingly — only when the logic is non-trivial enough to warrant a test and cannot be exercised through pure free functions. Prefer extracting the logic into a free function first; reach for the mirror pattern only when extraction would distort the production type's API.

When a task's evidence is that a **GUI host actually rendered** (not just
that pure logic or headless runtime state is correct), the assistant's
automated evidence must be **launch + screenshot capture + assistant
analysis of the captured image** — not merely that the launched process
stays alive. `Start-Process` survival is a supporting "no early crash"
signal only; it cannot show the screen rendered non-blank or that the
intended sub-screen is in view. This assistant baseline is a pre-owner
check and does **not** replace the owner's human-visible GUI smoke (see
[docs/notes/human-visible-smoke.md](docs/notes/human-visible-smoke.md)).
Capture mechanics and the environment requirement (visible desktop
session; per-monitor-DPI-aware capture; `Graphics.CopyFromScreen`, not
`PrintWindow`, because the DirectComposition client area reads back blank
under `PrintWindow`) are recorded in
[docs/notes/verification-environments.md](docs/notes/verification-environments.md).

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

**Doc-side commits are scoped by review concern, not by file.** Files
that share a single review concern (e.g. a plan and its sibling ADR
touched in one tracking-table update) may land together; documents
whose review cycles converge at different rates (e.g. an owner-pre-
approved ADR status flip, a normative spec draft, and a project-wide
process change) must not. Draft and revision are typically separate
commits so the review diff stays legible — but multiple small
revisions may bundle when they share a review concern, and reviewers
may also batch several commits into one review pass.

Multi-document "Moment" or analogous bundle constructs introduced by
a framing decision are milestone labels, not commit units. See the
M3-Phase 2 framing decision D Postmortem in
[process/milestone-3/phase-2/requirements/framing.md](process/milestone-3/phase-2/requirements/framing.md)
for the originating failure.

## Retrospective rules

Run a retrospective before every merge. Scope is determined by the merge
target:

- merge target = phase branch → **task retrospective**
- merge target = main → **phase retrospective**

The merge gate requires explicit owner approval after the retrospective
checklist is complete. Push is a separate gate from merge.

Full procedure (checklist, doc-set, forward-carry discipline):
[process/procedures/retrospectives.md](process/procedures/retrospectives.md).

## Process rule lifecycle

Process knowledge has four SSOTs ([process/README.md §SSOT distribution](process/README.md#ssot-distribution)).
Changes flow as:

- **Minor edits** (wording, additional examples, clarifications) — edit
  the owning SSOT directly.
- **Structural changes** (new enforceable rule, policy reversal, new
  category) — file a vision decision record under `process/cross-milestone/decisions/` first, then
  update the SSOT in the same commit batch that flips the ADR to
  `Accepted`.

Boundary test: a change is *structural* if it requires touching another
SSOT or supersedes a prior decision. If both are no, edit in place.

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
