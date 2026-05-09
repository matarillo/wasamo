---
milestone: M2
status: in-progress
roadmap-anchor: ROADMAP.md#m2-foundation
adrs:
  - docs/decisions/vision-post-m2-roadmap.md
  - docs/decisions/m2-phase-1-cdylib-shim.md
  - docs/decisions/m2-phase-2-wasamoc-output-format.md
  - docs/decisions/m2-phase-3-handler-exec-location.md
  - docs/decisions/m2-phase-4-tree-mutation-abi.md
  - docs/decisions/m2-phase-5-reactive-engine.md
created: 2026-05-02
---

# M2 Plan — Foundation Milestone

## Frozen agreement

### Purpose

M1 proved the core hypothesis (external DSL × C ABI × Visual Layer) by
exercising the runtime side end-to-end, with hosts constructing widget
trees imperatively through the experimental C ABI layer. M2's purpose
is to **close the loop on the DSL side**: make `.ui` files actually
drive the runtime, with reactive state propagation, so that Hello
Counter in each language is written against the DSL rather than
reproducing the DSL by hand.

This redefines M2 from the original "Alpha" feature wishlist (Grid /
ScrollView / List / input / IME / AccessKit / VS Code / DSL spec
public draft) into a **foundation milestone** whose acceptance is
structural, not feature breadth. The Alpha-style feature work has
been redistributed across M3–M6 (see
[ROADMAP.md](../../ROADMAP.md) and
[docs/decisions/vision-post-m2-roadmap.md](../decisions/vision-post-m2-roadmap.md)).

### Phase numbering

Phase numbers in this plan are **local to M2** (M2-Phase 1, 2, …).
M1's global Phase 1–8 numbering is not continued. ADR identifiers
from M2 onward use the scope `M<N>-P<n>` (e.g. `DD-M2-P2-001`); see
[docs/decisions/README.md](../decisions/README.md#file-naming).
M1 phase ADRs (`DD-P3-001` etc.) remain as historical records and
are not renumbered.

### Acceptance criteria

ROADMAP is the SSOT; mirrored here for ergonomics:

- **A1.** `examples/counter/counter.ui` drives the running Hello
  Counter in C, Rust, and Zig — the M1 host-imperative trees in
  `examples/counter-{c,rust,zig}/` are replaced by hosts that load
  the DSL through the agreed wasamoc pipeline.
- **A2.** Reactive state propagation works without host-side
  property-set plumbing: `count++` in the host updates the visible
  label through the M2 reactive path, not through a manual
  `wasamo_set_property` call written by the application.
- **A3.** `wasamo-runtime` and the `wasamo` safe wrapper no longer
  share an rlib filename through the cdylib-shim split; the post-M1
  cleanup flagged in
  [DD-P7-002](../decisions/phase-7-language-bindings.md) is
  discharged.
- **A4.** The C ABI gains the tree-mutation primitives required by
  the reactive engine; the experimental layer's all-at-once
  constructors remain available but are no longer the only way to
  construct UI.

- **A5.** Reactive Foundation Hardening. The reactive engine's
  execution-order guarantees and the runtime's re-entrancy/guard
  placement principle are settled at design level (Accepted ADRs)
  and reflected in implementation. Specifically:
  - DD-M2-P6-010 (topological sort of the dirty Effect drain) is
    Accepted and the implementation no longer relies on the counter
    case happening to converge.
  - DD-M2-P6-012 (re-entrancy / safety-guard placement principle)
    is Accepted and the principle is recorded in
    `docs/architecture.md` as a global runtime invariant that future
    M3+ entry paths must observe.

- **A6.** Type-Agnostic Reactive Binding. The reactive binding path
  is demonstrated end-to-end with a non-`i32` property type
  (`String`), proving the `EvalContext` / `HandlerExpr` / IR design
  is not silently `i32`-specialized.
  - DD-M2-P6-011 is Accepted; `.ui` String property bound to
    `Signal<String>` propagates to the visible widget.

### Phase breakdown

The phases below are working hypotheses; each one's design questions
become a phase ADR at pre-doc time, per
[the decisions README](../decisions/README.md).

- **M2-Phase 1 — cdylib-shim cleanup.** Split DLL output from the
  rlib so `wasamo-runtime` can be renamed cleanly without the
  cargo#6313 filename collision. Pure infra; independent of the DSL
  track. Origin:
  [DD-P7-002 post-M1 implementation note](../decisions/phase-7-language-bindings.md).

- **M2-Phase 2 — wasamoc output format decision.** Resolve the
  question Phase 6 pre-doc explicitly deferred to M2: host-language
  codegen vs IR + runtime interpretation. Includes implications for
  binding-author workload and the feasibility of post-M2 hot-reload
  (the latter is out of M2 scope but is constrained by this
  decision).

- **M2-Phase 3 — Handler execution location.** Resolve the second
  Phase 6-deferred question: where DSL inline handler bodies
  (`clicked => { ... }`) execute. The decision interacts with
  M2-Phase 2 and with M2-Phase 4's ABI surface.

- **M2-Phase 4 — Tree-mutation primitives at the ABI surface.**
  Promote the operations the reactive engine needs (insert / remove
  / replace child; property batching) from runtime-internal to the
  stable-core C ABI. M1 deliberately deferred this — see
  [DD-P8 "Out of scope"](../decisions/phase-8-hello-counter.md).

- **M2-Phase 5 — Reactive engine.** State change → invalidate →
  relayout → render path, building on the queued-emission machinery
  from Phase 6 and the layout invalidation hooks from
  [DD-P8-002](../decisions/phase-8-hello-counter.md).
  Subtree-vs-root dirty granularity is in scope only insofar as M2
  acceptance demands; large-tree optimization stays an open question
  in [layout-engine note §3.4](../notes/layout-engine.md).

- **M2-Phase 6 — `.ui → runtime` lowering.** The end-to-end pipeline
  that consumes M2-Phases 2 / 3 / 5 and produces a running Hello
  Counter from `counter.ui`. Replaces the imperative tree
  construction in `examples/counter-{c,rust,zig}/`.

- **M2-Phase 7 — Reactive Foundation Hardening & Contract
  Finalization.** Discharge the three DDs deferred from Phase 6
  closing (DD-M2-P6-010 / 011 / 012). Phase 6 establishes the
  pipeline (counter `.ui` → runtime, A1/A2); Phase 7 establishes the
  foundation guarantees that distinguish "it runs" from "it is a
  Foundation" (A5/A6). Order of work: 010 (topo sort) → 012 (guard
  placement principle, including `architecture.md` update) → 011
  (String binding end-to-end). The phase closes when all three DDs
  are Accepted and their implementation lands.

### Phase dependencies

```
M2-Phase 1   ── independent infra; lands any time

M2-Phase 2 ─┐
M2-Phase 3 ─┤
            ├─ M2-Phase 4 ─ M2-Phase 5 ─ M2-Phase 6 ─ M2-Phase 7
```

M2-Phases 2 and 3 are decision phases and can run in parallel; both
gate M2-Phase 4. M2-Phase 5 depends on 4. M2-Phase 6 depends on the
decisions (2, 3) and on 5. M2-Phase 7 depends on M2-Phase 6.

### Acceptance ↔ phase mapping

| Acceptance | Phase(s) |
|---|---|
| A1 (`counter.ui` drives all three hosts) | M2-Phase 6 |
| A2 (reactive propagation, no host wiring) | M2-Phase 5, M2-Phase 6 |
| A3 (cdylib-shim cleanup) | M2-Phase 1 |
| A4 (tree-mutation ABI primitives) | M2-Phase 4 |
| A5 (Reactive Foundation Hardening) | M2-Phase 7 |
| A6 (Type-Agnostic Reactive Binding) | M2-Phase 7 |

M2-Phases 2 and 3 are **decision phases** without a direct acceptance
hook; their outputs are ADR-shaped and feed M2-Phases 4 / 6.

### Out of scope (deferred to later milestones)

Items that originally appeared in the M2 Alpha paragraph but are
**not** in M2-as-foundation. Allocation to post-M2 milestones is
recorded in [ROADMAP.md](../../ROADMAP.md):

- Grid / ScrollView / List layout primitives → M3
- DSL spec public draft → M3
- Input handling (kbd / mouse / touch + focus model) → M4
- Multi-window support → M4 (pre-1.0 because of cross-cutting ABI)
- TextField widget → M4 (required by IME verification)
- IME via TSF (Japanese / CJK input) → M4
- AccessKit / UIA accessibility integration → M4
- Mica / Acrylic root-window backdrop, system accent → M4
- VS Code extension (LSP / highlighting / diagnostics) → M5 (parallel
  track may begin once M3 spec draft is agreed)
- Full theming surface, official widget set beyond TextField → M5
- Performance target verification (<100 ms startup, <30 MB memory) → M6
- Polished showcase + ABI freeze + C/Rust/Zig bindings mature → M6
- Hot reload (interpreter mode) → post-1.0; feasibility depends on
  M2-Phase 2's wasamoc output format decision
- Higher-level animation DSL → post-1.0
- Swift / Go bindings → post-1.0 community track

### Risks

- **Decision phases (M2-Phase 2, M2-Phase 3) blocking the DSL
  track.** If 2 / 3 do not converge in pre-doc, M2-Phase 6 cannot
  start. Mitigation: each decision phase is timeboxed to a single
  ADR review cycle; if the question does not converge, escalate to a
  VISION-level ADR (analogous to DD-V-001) rather than re-opening
  pre-doc indefinitely.

- **Reactive engine coupling with layout invalidation.**
  [DD-P8-002](../decisions/phase-8-hello-counter.md) installed a
  coarse "whole-window dirty" path. If M2-Phase 5 demands finer
  granularity for correctness (not performance), the layout-engine
  changes ripple beyond M2-Phase 5's nominal scope.

### Resolved deferrals

The post-M2 questions raised alongside the M2 redefinition were
resolved on 2026-05-02 and are now recorded in
[ROADMAP.md](../../ROADMAP.md), [VISION.md §7](../../VISION.md#7-roadmap),
and [docs/decisions/vision-post-m2-roadmap.md](../decisions/vision-post-m2-roadmap.md)
(DD-V-005..009). Summary:

- Grid / DSL spec public draft → M3
- Post-M2 structure: thesis-driven milestones M3 (DSL surface) /
  M4 (Interaction stack) / M5 (Identity & tooling) / M6 (1.0);
  Alpha / Beta labels dropped
- Multi-window → M4 (pre-1.0, ABI cross-cutting)
- Mica / Acrylic + first showcase → M4 (identity feature
  demonstrable from M4)
- VS Code LSP → M5 acceptance, parallel track from M3 spec draft
- Hot reload → post-1.0
- 1.0 binding list → C / Rust / Zig; Swift / Go → post-1.0 community
- ADR identifier scope `M<N>-P<n>` from M2 onward (see Phase
  numbering above)

### Revision log

- **2026-05-08** — Acceptance criteria revision under the
  README.md "Acceptance criteria revision" exception.
  - Motivation: A1–A4 cover pipeline wiring and structural cleanup
    but do not cover the runtime guarantees (execution order,
    re-entrancy/guard placement, type-agnostic binding) required to
    call M2 a Foundation milestone in a non-trivial sense.
  - Added: A5 (Reactive Foundation Hardening), A6 (Type-Agnostic
    Reactive Binding).
  - Added: M2-Phase 7 (Reactive Foundation Hardening & Contract
    Finalization), depending on M2-Phase 6.
  - DD-M2-P6-010 / 011 / 012 status remains Proposed; their
    discharge is now scoped to Phase 7.

- **2026-05-09** — Phase 7 progress (no acceptance-criteria change).
  - DD-M2-P6-010 (`dirty_effects` topological sort fidelity)
    Accepted in [m2-phase-7-reactive-foundation.md](../decisions/m2-phase-7-reactive-foundation.md#dd-m2-p6-010--dirty_effects-topological-sort-fidelity)
    — Option A (true topological walk in M2; pure-logic unit tests
    on synthetic dependency graphs; single drain code path). M3
    residuals (cycle detection, ordering ties, fan-out × MUTATION_CAP)
    recorded in [m2-to-m3-handover.md](../notes/m2-to-m3-handover.md)
    §3. Implementation step pending; A5 first clause (DD-010 Accepted)
    discharged at design level; second clause (implementation
    reflects it) opens the next step.

## Progress

The progress section is live until M2 reaches `completed`; it tracks
sub-task state for each phase and the owner-facing "where did we
leave off" memory. ADR links and commit references are added as
phases land.

- [x] **M2-Phase 1 — cdylib-shim cleanup**
  - ADR: [docs/decisions/m2-phase-1-cdylib-shim.md](../decisions/m2-phase-1-cdylib-shim.md) (Accepted 2026-05-03)
  - [x] `docs/decisions/m2-phase-1-cdylib-shim.md` — owner agreement (status "Accepted")
  - [x] `docs/notes/workspace-layout.md` — new live note: workspace layout open question (`crates/` migration) per DD-M2-P1-004
  - [x] `wasamo-runtime/Cargo.toml`: `[lib].name = "wasamo_runtime"`, `crate-type = ["rlib"]`. Comment update.
  - [x] **Intermediate verification (after rlib rename only):** `cargo build --release --workspace` passes.
  - [x] New `wasamo-dll/` crate: `Cargo.toml` (`[lib] name = "wasamo" crate-type = ["cdylib"]`), `build.rs` with MSVC `/WHOLEARCHIVE:wasamo_runtime` link arg, `src/lib.rs`. Workspace `Cargo.toml` `members += ["wasamo-dll"]`. Bundled with dep-edge step below (DD-M2-P1-006: shim without the edge reproduces the LNK1181 race).
  - [x] `bindings/rust-sys/build.rs` and any other consumer: cdylib build output path verified unchanged.
  - [x] `bindings/rust-sys/Cargo.toml`: `wasamo-dll = { path = "../../wasamo-dll" }` added to `[dependencies]` for build-order edge (DD-M2-P1-006). `no linkable target` warning accepted per linked note.
  - [x] `docs/notes/cdylib-shim-build-graph.md` — new live note: `no linkable target` deferral and re-evaluation triggers (DD-M2-P1-006)
  - [x] **Final verification:** `cargo clean && cargo build --release --workspace` passes; `dumpbin /exports target/release/wasamo.dll` shows all 19 `wasamo_*` symbols; `cargo run -p counter-rust --release` works end-to-end.
  - [x] `docs/architecture.md`: §1 workspace layout and crate responsibilities table updated; §11.4 replaced.
  - [x] `docs/plans/m2-plan.md` Progress: phase ticked, ADR linked.
  - [x] `CHANGELOG.md`: cdylib-shim split entry added.
  - Experimental branch (after main landed):
    - [x] Create branch `exp/m2-p1-poc-examples` from M2-Phase 1 tip.
    - [x] Recover Phase 2-5 examples from git history; place under `wasamo-poc/`; add to workspace. Update their `wasamo` dep to `wasamo-runtime`.
    - [x] Verify they compile and run on the SSH dev box.
    - [x] Do not merge to main; branch serves as resurrection reference.
- [x] **M2-Phase 2 — wasamoc output format decision**
  - ADR: [docs/decisions/m2-phase-2-wasamoc-output-format.md](../decisions/m2-phase-2-wasamoc-output-format.md) — **Accepted 2026-05-04** (spike passed; Option B adopted)
  - [x] `docs/decisions/m2-phase-2-wasamoc-output-format.md` — pre-doc filed (DD-M2-P2-001..004); status "Pre-doc", agreement gated on feasibility spike
  - [x] Owner agreement on DD-M2-P2-001 (Option B: IR + interpreter), DD-M2-P2-002 (textual IR), DD-M2-P2-003 (activities 1–7 in `wasamoc`), DD-M2-P2-004 (sequential sequencing vs Phase 3)
  - Feasibility spike (`exp/m2-p2-ir-loader-spike`, commit `b7ab4dc`):
    - [x] Branch `exp/m2-p2-ir-loader-spike` created from M2-Phase 1 tip
    - [x] `experimental_ir_loader` module added to `wasamo-runtime` (feature-gated `experimental-ir`)
    - [x] `experiments/ir-spike/counter.uic` hand-written in throwaway s-expression IR form
    - [x] ~200-line loader (tokenizer + tree walker) implemented in `wasamo-runtime`
    - [x] `experiments/ir-spike/` driver crate renders counter window end-to-end
    - [x] Pass criteria confirmed: internal builder API (`WidgetNode::vstack`, `text`, `button`, `append_child`, `set_clicked`) driven without modification; tagged-value `PropertyValue` sufficient; GUI renders identically to M1 hand-written example
  - [x] Spike result appended to ADR; status → **Accepted**
  - [x] `docs/plans/m2-plan.md` Progress: phase ticked, ADR linked, task list written
- [x] **M2-Phase 3 — Handler execution location**
  - ADR: [docs/decisions/m2-phase-3-handler-exec-location.md](../decisions/m2-phase-3-handler-exec-location.md) — **Accepted 2026-05-04**
  - [x] `docs/decisions/m2-phase-3-handler-exec-location.md` — pre-doc filed (DD-M2-P3-001..004); status "Proposed"
  - [x] Owner agreement on DD-M2-P3-001 (Option A: runtime-side interpreter), DD-M2-P3-002 (Option B: separate paths, inline first), DD-M2-P3-003 (Option A: catch_unwind + stderr), DD-M2-P3-004 (Option B: IR reserves optional span; coarse identifiers in M2)
  - [x] ADR status → **Accepted**
  - [x] `docs/notes/headless-verification.md` — new live note: examining the need for a headless-verification mechanism (drafted on the Phase 3 verification gap; M2 closes with a pure-logic test fixture strategy without building one)
  - [x] `docs/plans/m2-plan.md` Progress: phase still **open** — task list expanded below; coding work begins next session
  - **Implementation scope (this phase):**
    - [x] New `wasamo-runtime/src/handler.rs` — `HandlerExpr` enum (assign / `+=` `-=` `*=` `/=` / property read+write / int literal / block) + `EvalContext` trait + `evaluate()` + 14 unit tests (assign / compound / wrapping overflow / nested block / empty block / division-by-zero / prop-read-unknown)
    - [x] Added `inline_handlers` slot + `set_inline_handler()` API to `WidgetNode`; signal emit path reworked to run inline evaluation before host listener iteration (DD-M2-P3-002 Option B); ordering verified by `inline_before_host_ordering` unit test; `NullEvalContext` placeholder (replaced with the real context in Phase 5)
    - [x] `invoke_handler()` — handler invocation wrapped in `std::panic::catch_unwind`, logged to stderr in the format `wasamo: handler error in <location>: <message>` (DD-M2-P3-003); 3 unit tests covering panic injection / eval-error / success
    - [x] `format_handler_location()` + `WidgetPathSegment` — coarse identifier `<component>.<widget-path>.<signal>` formatter (DD-M2-P3-004 Option B); 5 pure-logic unit tests
    - [x] `cargo build --release --workspace` passes / `cargo test --workspace` passes (30 wasamo-runtime tests + 36 wasamoc tests + 1 wasamo-sys smoke test)
  - **Boundary with adjacent phases:**
    - vs Phase 4: handlers stay on internal `set_property` (no C ABI crossing — the essence of DD-M2-P3-001 Option A). Phase 4's C ABI promotion does not touch the handler path.
    - vs Phase 5: the `HandlerExpr` evaluator is implemented for the handler axis only. Sharing a common base with the binding evaluator happens in Phase 5 (the handler evaluator is Phase 5's starting point).
    - vs Phase 6: `HandlerExpr` is defined as an in-memory enum. Wiring textual IR ↔ `HandlerExpr` serialization is done in Phase 6. Phase 3 does not touch the throwaway IR in `experiments/ir-spike/` (it is fully redesigned in Phase 6).
  - **No GUI verification in this phase.** It will be confirmed retroactively when Phase 5 (reactive integration) completes and the counter's click → label update works end-to-end. See [docs/notes/headless-verification.md](../notes/headless-verification.md) for the rationale.
- [x] **M2-Phase 4 — Tree-mutation ABI primitives**
  - ADR: [docs/decisions/m2-phase-4-tree-mutation-abi.md](../decisions/m2-phase-4-tree-mutation-abi.md) — **Accepted 2026-05-05** (DD-M2-P4-001..004 all Option A)
  - **Implementation scope (this phase):**
    - [x] `wasamo-runtime/src/widget.rs` — added `attached: bool` field to `WidgetNode` (DD-M2-P4-003). Set to `true` on `append_child` attach; reset to `false` on detach.
    - [x] `wasamo-runtime/src/widget.rs` — added 4 internal Rust mutation APIs: `insert_child(&mut self, index: usize, child: Box<WidgetNode>) -> Result<(), MutationError>` / `remove_child(&mut self, index: usize) -> Result<Box<WidgetNode>, MutationError>` / `replace_child(&mut self, index: usize, new_child: Box<WidgetNode>) -> Result<Box<WidgetNode>, MutationError>` / `child_count(&self) -> usize`. `MutationError` represents `IndexOutOfBounds` / `AlreadyAttached` (the new child is already attached elsewhere).
    - [x] `wasamo-runtime/src/widget.rs` — `widget_destroy(node: Box<WidgetNode>)` internal helper: walks the subtree, severs signal-handler / observer-registry entries, and drops the `Box`. Shared with the `wasamo_window_destroy` subtree-teardown path (factored out into a shared sweep helper).
    - [x] **Pure-logic unit tests** (`widget.rs`): 16 tests (Slot/Children mirror verifies index bounds / attached transitions; no Win32/WinRT required)
    - [x] `wasamo-runtime/src/abi.rs` — added 5 stable-core C ABI functions + promoted `wasamo_widget_append_child` (DD-M2-P4-001 = A / DD-M2-P4-002 = A)
    - [x] `wasamo_widget_destroy` precondition handling: NULL → `WASAMO_OK` idempotent; attached → `WASAMO_ERR_INVALID_ARG` + last-error message (per the DD-M2-P4-003 ADR).
    - [x] Added the 6 new symbols to `wasamo.h` as §4.6; `wasamo_widget_append_child` moved into stable-core.
    - [x] `wasamo-runtime/src/reactive.rs` — created `with_batched_writes` skeleton (pub(crate); implementation in Phase 5). No C ABI symbol added per DD-M2-P4-004 = A.
    - [x] Revised `docs/abi_spec.md`: added §4.6 Tree mutation; updated §5 ownership wording; added §6 batching contract paragraph.
    - [x] `cargo build --release --workspace` passes / `cargo test -p wasamo-runtime -p wasamoc --release` passes (45 + 36 + 1 = 82 tests).
    - [x] **Link/export verification:** all 6 new symbols confirmed via `dumpbin /exports target/release/wasamo.dll` (wasamo_widget_append/insert/remove/replace_child / child_count / destroy).
    - [x] CI smoke test — no CI config change required (`cargo build --release --workspace` covers the new symbols).
    - [x] `docs/architecture.md` — added a note that stable core now covers 6 areas.
    - [x] `CHANGELOG.md` — added Phase 4 entry.
  - **Boundary with adjacent phases:**
    - vs Phase 3: handlers were implemented in Phase 3 to call internal `set_property` directly, and the internal path is preserved after Phase 4 C ABI promotion (avoids re-entrancy + retains the benefit of DD-M2-P3-001 Option A). Phase 4's mutation primitives do not touch the handler path.
    - vs Phase 5: the reactive engine's invalidation cascade is implemented on top of Phase 4's `with_batched_writes` skeleton (internal Rust). No host-visible batching API ships in M2 (DD-M2-P4-004 = A). When the Phase 5 binding evaluator calls the internal mutation API (`insert_child` / `remove_child`, etc.), it must obey this phase's attached-state invariants.
    - vs Phase 6: `wasamo_load_ui` (Phase 6) is one new C ABI entry, but tree construction itself uses this phase's internal Rust mutation API from inside the runtime (it does not go through the C ABI — same pattern as the Phase 2 spike's `experimental_ir_loader`).
  - **Verification kinds:** unit tests (pure logic: index bounds / attached-state transitions / subtree teardown — self-contained in `widget.rs`) + build (`cargo build --release --workspace`) + link/export (`dumpbin /exports`) + ABI smoke (CI header consistency). **No GUI verification in this phase alone** — mutation primitives are exercised retroactively by Phase 5's reactive behaviour and Phase 6's `.ui`-driven counter. Same policy as Phase 3 ([docs/notes/headless-verification.md](../notes/headless-verification.md)).
- [x] **M2-Phase 5 — Reactive engine**
  - ADR: [docs/decisions/m2-phase-5-reactive-engine.md](../decisions/m2-phase-5-reactive-engine.md) — **Accepted 2026-05-05** (DD-M2-P5-001..006)
  - Live note: [docs/notes/architectural-family.md](../notes/architectural-family.md) — records the tree-with-bindings family as a working hypothesis (not a long-term commitment); re-evaluation triggers documented for M3 DSL spec drafting and post-1.0 hot reload.
  - Pre-aligned design axes: [docs/notes/m2-phase-5-design-axes.md](../notes/m2-phase-5-design-axes.md) — owner direction (2026-05-05) on dependency-tracker depth and Option A verification, recorded before pre-doc.
  - **Risk concentration (the risk this phase must absorb within M2):**
    - This is the only technical-thesis validation point in M2 (A2: reactive propagation without host wiring). Every other phase stays in structural goals (A3) / ABI surface extension (A4) / integration (A1); the M2 foundation hypothesis — "a dependency tracker on top of DD-P8-002's whole-window dirty + queued emission" — is exercised here for the first time.
    - Decisions punted by other phases accumulate here: settling the `with_batched_writes` shape from Phase 4 (DD-M2-P4-004 = A); deciding whether Phase 3's `HandlerExpr` evaluator and the binding evaluator share a common core; the subtree-granularity open question left by DD-P8-002 ([layout-engine note §3.4](../notes/layout-engine.md)); re-evaluation of headless verification ([headless-verification note](../notes/headless-verification.md)).
    - Downstream rework costs (Phase 6 / M3+) depend heavily on this phase's shape. Phase 6 introduces no new mechanisms and just consumes Phase 5's evaluator output (the typed-IR representation of binding statements) — getting the shape wrong here forces a regression all the way back to Phase 2's textual IR normative grammar. M3+ binding-feature extensions (Grid cell bindings, List per-item context) also stack on top of this phase's dependency-tracker design.
    - Two risk-taking axes: (a) dependency-tracker design depth — settling for minimum viable (counter's single binding working is enough) means a rewrite in M3. Decide at pre-doc time whether to adopt the Solid / Vue signals prior-art pattern from the start. (b) commitment to headless verification — test in this phase whether the pure-logic fixture policy actually holds; if not, file an independent ADR for a no-Compositor mode. Discovering at Phase 6 (GUI manual verification mandatory) that headless is needed is the worst-case trajectory, so decide at the start of this phase.
  - **Implementation scope (this phase, settled at Phase 4 granularity per the Accepted ADR):**
    - [x] **`reactive.rs`: core primitives — `Signal<T>` + `Effect` + thread-local current-effect stack + dependency graph** (DD-M2-P5-001 = B / DD-M2-P5-002 = B). Forward edges `HashMap<SignalId, HashSet<EffectId>>`; back-edges `HashMap<EffectId, HashSet<SignalId>>` (needed for disposal). Thread-local `Vec<EffectId>` populated during `Effect::run()`. `get_untracked()` escape hatch for the rare reads outside dependency collection.
      - **Verification:** pure-logic unit tests — `Signal::set` invalidates dependents; re-running an Effect repopulates its dependency set (a binding may pick up different Signals each pass); nested Effect tracking; `get_untracked` does not record dependency.
      - **Technical risk: Low.** Solid.js / Vue ref / MobX prior art is canonical; the data-structure choice is HashMap/HashSet; thread-local stack is a standard pattern.
      - **Failure mode:** ID allocator (monotonic `u64` vs `slotmap`) may need a swap. Absorbable as DD-M2-P5-007 or a private refactor.
      - **Implementation note:** `Signal::set` does immediate Effect re-execution in this task. `with_batched_writes` (next task) will replace that with a dirty-set enqueue path.
    - [x] **`reactive.rs`: `with_batched_writes` body — fill in the Phase 4 skeleton.** Thread-local depth counter; outermost exit drains the dirty Effect set with an iteration cap (e.g. 16) to detect runaway re-entry.
      - **Verification:** pure-logic test — N writes inside the closure produce one Effect re-run; a re-entrant write inside the Effect re-runs but is bounded; iteration-cap exhaustion logs an error and breaks the loop.
      - **Technical risk: Low.** Phase 4's skeleton already established the entry/exit shape; bodywork is bookkeeping.
      - **Failure mode:** cap value or the diagnostics format may shift. Absorbable.
    - [x] **`reactive.rs`: Effect disposal via owner Drop** (DD-M2-P5-003). `EffectHandle` is the owner; its `Drop` walks the back-edge map, removes the EffectId from every Signal's dependent set, then frees the closure.
      - **Verification:** pure-logic test — register binding, drop the handle, write to the previously-tracked Signal, assert the closure does not run; assert no leaked entries in either edge map.
      - **Technical risk: Low.** Disposal is O(deps); the back-edge map exists to make it so.
      - **Failure mode:** disposal that fires while the Effect is mid-run (rare: a binding writes to a property that destroys its owning widget) may need a "currently-running" guard. Absorbable as DD-M2-P5-007.
      - **Implementation note:** `Drop` body landed bundled with step 1 (closure `Rc` must outlive the initial `run_effect` call). This step adds the leak-free edge-map verification to `dropped_handle_stops_effect`.
    - [x] **`handler.rs` / `reactive.rs`: `BindingEvalContext` — read-only `EvalContext` variant that wraps property storage and reports reads to the current Effect.** Reuses Phase 3's `EvalContext` trait (extends with a `read_property_tracked` path) and `HandlerExpr::evaluate()` (read-only subset; rejects `set` / compound-assign nodes at evaluation time per DD-M2-P5-006). Includes string-interpolation evaluation for `"Count: \{root.count}"`.
      - **Verification:** extend handler.rs unit tests — evaluator with `BindingEvalContext` records the expected Signal reads; rejects write expressions with a typed error.
      - **Technical risk: Low.** `EvalContext` is already mock-friendly (`NullEvalContext` exists from Phase 3). The variant is essentially a property-store adapter with a `track_read()` hook.
      - **Failure mode:** trait method shape may need splitting (`read_property` vs `read_property_tracked`). Absorbable.
    - [x] **`widget.rs`: `WidgetNode.bindings: Vec<EffectHandle>` field + binding disposal in the Phase 4 `widget_destroy` sweep.** Append handle disposal at the start of the sweep so reactive bookkeeping clears before signal-handler / observer registry severance — Effects may capture widget references that the existing teardown invalidates.
      - **Verification:** pure-logic test using the Slot/Children mirror (Phase 4 pattern) — register binding on a node, run `widget_destroy`, write to the corresponding Signal, assert the binding does not fire and edge maps are clean.
      - **Technical risk: Low–Medium.** `WidgetNode` is Win32/WinRT-coupled (Compositor-bound constructor); the mirror pattern is required. Drop ordering vs attached/detached state needs care.
      - **Failure mode:** may need an explicit `dispose_bindings()` call inside `widget_destroy` rather than relying on the field's natural `Drop` ordering. Refinement of DD-M2-P5-003 wording; absorbable.
    - [x] **`reactive.rs`: `register_binding(target: BindingTarget, expr: HandlerExpr) -> EffectHandle`** (DD-M2-P5-005 = A). `BindingTarget::WidgetProperty { node: WidgetId, prop: PropertyKey }` is the sole variant in M2. Internally constructs the Effect (closure that evaluates `expr` against `BindingEvalContext` and writes the result via internal `set_property`), registers it, returns the handle.
      - **Verification:** 2 pure-logic tests in `reactive::tests` — `register_binding_writes_initial_and_updates` (Signal write → new value propagated via writer); `register_binding_writer_called_for_size_affecting_prop` (writer called on initial run and on each Signal update, representing the `set_property` + DD-P8-002 dirty-mark path).
      - **Implementation notes (deviations from plan wording, absorbable):**
        - `WidgetId` is `*mut ()` (type-erased) in `reactive.rs` to avoid a circular import with `widget.rs`. Production callers in `widget.rs` cast `*mut WidgetNode` to `*mut ()` when constructing `BindingTarget`.
        - `register_binding` takes a `write_fn: fn(WidgetId, PropertyKey, &str)` parameter (not in the ADR signature); the production implementation (`widget_write_property`) lives in `widget.rs`. The function-pointer argument is the seam that keeps the modules decoupled.
        - An internal `register_binding_with_writer(writer: Box<dyn FnMut(String)>, expr, props)` is the testable core; tests call it directly with mock writers.
        - `props: Rc<HashMap<String, Signal<i32>>>` is passed at call-site rather than inferred from ambient state. Shape is provisional; will be revisited at the Phase 5 close GUI checkpoint.
      - **Failure mode:** `BindingTarget` may need a second variant (e.g. for diagnostics path or for the Phase 6 IR ↔ binding wiring). `pub(crate)` so internal refactor; absorbable.
    - [x] **`emit.rs`: insert reactive drain pass into `drain_if_outermost`** (DD-M2-P5-004 = B). Order: observer drain → reactive drain (run dirty Effects under `with_batched_writes`) → layout drain. Reactive writes go through `set_property` which marks layout-dirty under DD-P8-002, so the reactive pass must precede the layout drain.
      - **Verification:** pure-logic test asserting drain ordering observer → reactive → layout; integration test that a binding write triggers layout invalidation within the same outermost-frame cycle. The Phase 5-close GUI checkpoint exercises the real path end-to-end.
      - **Technical risk: Medium — the riskiest step in the phase.** Composes with three existing rules: DD-P6-003 queued emission (must not bypass), DD-P8-002 layout invalidation (must precede the layout drain), and re-entrancy (a reactive Effect writing a Signal that another Effect depends on creates within-pass cascades). The iteration-cap mechanism bounds runaway, but glitch-style "stale read mid-cascade" edge cases can hide here, and the M2 acceptance scenario (counter, single binding) does not exercise multi-Effect cascades — coverage leans on synthetic fixtures rather than the acceptance flow.
      - **Failure mode:** DD-M2-P5-004 = B may prove insufficient for a multi-binding scenario surfaced during Phase 6 implementation. **Most likely DD-addition source** — refinement would be DD-M2-P5-007/008 with explicit cycle-detection or sub-iteration semantics. Still inside the "DD absorbable" envelope as long as the family-level commitment (Signal + Effect 2-layer; tree-with-bindings) holds.
    - [x] **Pure-logic unit test suite** (DD-M2-P5-006 = A). Covers Signal/Effect propagation; dependency repopulation across re-runs; disposal cleanup; `with_batched_writes` coalescing; iteration-cap bound; `BindingEvalContext` read tracking; `register_binding` end-to-end via the Slot mirror; drain-ordering invariants. Fake Effect closures only; no Win32/WinRT in tests; no new mirror types beyond Phase 4's Slot/Children pattern.
      - **Verification:** this *is* the verification surface for the items above except for the drain pass's interaction with the real `emit.rs` machinery (which lacks a test fixture today; covered by the GUI checkpoint below).
      - **Technical risk: Low.** Test-only.
      - **Failure mode:** if a behaviour cannot be expressed against fake closures alone (e.g. drain ordering can only be tested with a real `WindowState`), the headless-verification re-evaluation trigger fires — file an independent ADR for a no-Compositor mode. **This is the one path that would exceed DD-addition into a sibling ADR.** Pre-aligned design axes (DD-M2-P5-006 = A) make it unlikely; the actual reveal point is more likely the GUI checkpoint than the pure-logic suite.
    - [x] **Phase 5 close GUI checkpoint.** Build a small experimental harness (parallel to the Phase 2 IR-loader spike at `experiments/`) that constructs a counter using `register_binding` directly — no `.ui` parser yet, no host-side property-set — and verify on real hardware that clicking the button updates the visible label through the reactive path. Acceptance A2 is *fully* discharged at Phase 6; this checkpoint isolates Phase 5-internal bugs before Phase 6 conflates failure modes.
      - **Verification:** GUI manual on local Windows or RDP-attached desktop ([verification-environments note](../notes/verification-environments.md)). Counter increments visibly without any `wasamo_set_property` call from the host.
      - **Technical risk: Low.** Reuses the Phase 2 spike harness pattern.
      - **Failure mode:** the most likely surprise is thread-affinity (current-effect stack must live on the GUI thread). M2 is single-threaded GUI so this is unlikely to bite. If it does, fix is local; absorbable.
      - Spike (`exp/m2-p5-reactive-checkpoint`, commit `fdc1545`): same shape as Phase 2 IR-loader spike. Feature `experimental-reactive-spike` added to `wasamo-runtime`; `experimental_reactive_spike` module exposes `SpikeSignal`, `SpikeBindingHandle`, and `register_counter_binding`. Driver crate `experiments/reactive-spike/` constructs `VStack{Text, Button}`, wires `Signal<i32>` → `PROP_TEXT_CONTENT` via `register_binding`, click handler calls `count.set(count.get()+1)` only. **Pass criteria confirmed:** button click updates label visibly through the reactive path; host-side `set_property` call count: zero. Thread-affinity failure mode did not bite (M2 single-threaded GUI as anticipated). Phase 6 acceptance A2 takes over as the permanent verification path.
    - [x] **`docs/architecture.md` revision** — add a §6.x reactive-engine subsection (Signal/Effect/Binding model; drain ordering; disposal contract).
    - [x] **`CHANGELOG.md`** — Phase 5 entry.
    - [x] **No C ABI symbols added** (per DD-M2-P4-004 = A). All Phase 5 types stay `pub(crate)` in `wasamo-runtime/src/reactive.rs`. `wasamo.h` and `abi_spec.md` untouched. Verified: `git diff 9329f5a..feat/m2-phase-5 --name-only` shows `wasamo-runtime/src/abi.rs` / `bindings/c/wasamo.h` / `docs/abi_spec.md` all untouched; `dumpbin /exports wasamo.dll` reports the same 26 `wasamo_*` symbols as Phase 4 close.
  - **Per-step risk verdict — does any step push beyond "additional DD absorbable"?**
    - Eight of the nine implementation steps (core primitives / `with_batched_writes` body / disposal / `BindingEvalContext` / `WidgetNode.bindings` / `register_binding` / unit tests / GUI checkpoint) carry **Low** or **Low–Medium** technical risk. Their failure modes are local refinements — ID allocator swap, trait method split, Drop ordering tweak, `BindingTarget` variant addition — that fit cleanly into a DD-M2-P5-007/008 amendment without disturbing the six accepted DDs.
    - The one **Medium**-risk step is the reactive drain pass insertion. It composes with three existing rules (queued emission, layout invalidation, re-entrancy) that the M2 acceptance scenario does not stress-exercise. If a multi-binding cascade requirement surfaces during Phase 6, the refinement is still DD-shaped (cycle detection / sub-iteration semantics) and stays inside the accepted family-level frame (Signal + Effect 2-layer; tree-with-bindings).
    - The single path that would *exceed* DD-addition is the headless-verification trigger surfacing during the unit-test step or the GUI checkpoint — if any behaviour fundamentally requires a Compositor-free test backend, a sibling ADR (no-Compositor mode) is needed. Pre-aligned design axes (DD-M2-P5-006 = A) make this unlikely; the realistic reveal point is the GUI checkpoint, where any failure that pure-logic tests missed signals that headless coverage was actually load-bearing.
    - **Verdict: all currently-foreseen failure modes are absorbable as additional DDs within the existing ADR.** No item required pre-doc reinforcement; on this read the ADR was flipped to Accepted (2026-05-05). The single watch-item is the drain-pass ordering — the right time to escalate (DD-007/008 or sibling ADR) is during Phase 5 implementation review.
  - **Boundary with adjacent phases:**
    - vs Phase 3: consumes Phase 3's `HandlerExpr` and shares the evaluator core. When Phase 3 triggers a property write, Phase 5's dependency tracker fires invalidation via a hook.
    - vs Phase 4: implemented on top of Phase 4's `with_batched_writes` skeleton. Without Phase 4 batching, re-evaluation cascades degrade performance.
    - vs Phase 6: Phase 6 lowers `.ui` binding statements into typed IR, which Phase 5's binding expression evaluator consumes.
  - **Verification kinds:** unit tests (dependency tracker and binding evaluator are pure logic; Option A verification per the design-axes note uses fake Effect closures with no new mirrors and no headless backend) + GUI manual (verify reactive linkage of the counter on real hardware — acceptance A2). **The phase most likely to surface a need for a headless-verification mechanism**; on entering Phase 5, re-evaluate [docs/notes/headless-verification.md](../notes/headless-verification.md) and, if needed, file an independent ADR for a "no-Compositor" mode.
- [x] **M2-Phase 6 — `.ui → runtime` lowering**
  - ADR: [docs/decisions/m2-phase-6-ui-lowering.md](../decisions/m2-phase-6-ui-lowering.md) — **Accepted 2026-05-07** (DD-M2-P6-001..009 Accepted; DD-M2-P6-010/011/012 Proposed and deferred to M2-Phase 7 per the 2026-05-08 acceptance-criteria revision; housing migrated to [m2-phase-7-reactive-foundation.md](../decisions/m2-phase-7-reactive-foundation.md) on 2026-05-08, stubs retained at the Phase 6 ADR section anchors)
  - Pre-doc framing: [docs/notes/m2-phase-6-pre-doc-framing.md](../notes/m2-phase-6-pre-doc-framing.md) — owner-aligned slate / scope / upstream-update bundling.
  - **Risk concentration (the risk this phase must absorb within M2):**
    - Phase 6 introduces *no* new core mechanism (Phases 3/4/5 supplied them) but is the first place Phase 5's reactive engine, Phase 4's mutation ABI, and Phase 3's `HandlerExpr` interpreter meet at the host surface. A1/A2 are only *fully* discharged here.
    - The drain transaction DD (DD-M2-P6-001 = D) supersedes DD-M2-P5-004's three-stage shape and rewrites the runtime's observer model to "post-commit pure effect". This is the single non-additive shape change in M2; if Option D's structural cost (MVVM/KVO patterns become unwriteable) surfaces as an M2 acceptance blocker — which is unlikely since the counter has no observer — the fallback is Option C, recoverable mid-phase.
    - Counter-only acceptance does not stress-exercise Phase 1 ordering rules (FIFO + topological + last-wins) or MUTATION_CAP exhaustion. Synthetic pure-logic fixtures cover those; the GUI checkpoint validates only the single-binding path.
  - **Implementation scope (this phase, settled by the proposed ADR):**
    - [x] **Owner agreement on DD-M2-P6-001..009.** ADR flips to Accepted once agreement on all nine DDs is recorded; coding work begins after.
    - [x] **DD-M2-P6-002 / DD-M2-P6-003: Normative textual IR grammar + handler/binding expression form.** Add an IR chapter to `docs/dsl_spec.md`. Specify a header line (`;wasamo-ir v0` style) and productions for tree nodes / properties / bindings / handler bodies / `state` declarations. Promote the Phase 2 spike's tagged-value form for `HandlerExpr` (`(assign root.count (add (read root.count) 1))`) shared across bindings and handlers; permit bare literals in unambiguous positions.
      - **Verification:** spec-only step; round-trip regression deferred to the loader/emitter rewrite below.
    - [x] **DD-M2-P6-001: Drain transaction (Option D, declarative + post-commit pure observer).** In `wasamo-runtime/src/emit.rs` (or sibling), replace Phase 5's three-stage drain with the three-phase form:
      - Phase 1 — mutation convergence loop (signal_queue + dirty_effects until quiescent or MUTATION_CAP).
      - Phase 2 — layout pass (terminal, read-only).
      - Phase 3 — post-commit observer drain with `IN_OBSERVER_CALLBACK` TLS flag; state-mutating ABI returns `WASAMO_ERR_OBSERVER_MUTATION`.
      - Phase 1 ordering rules: FIFO `signal_queue`, topological-by-dependency-graph `dirty_effects`, last-wins per-Signal write semantics.
      - Phase 1 re-entrancy: state mutation permitted; structure-changing ABI (e.g. `wasamo_load_ui`) returns `WASAMO_ERR_REENTRANT_LOAD`.
      - Divergence: MUTATION_CAP exhaustion drives an irreversible `Healthy → Diverged` transition; all subsequent ABI calls return `WASAMO_ERR_REACTIVE_DIVERGED` except `wasamo_runtime_destroy`. Diagnostics payload (offending Effect ID + iteration count + last-iteration dirty Signal IDs) lands in `wasamo_last_error_message`.
      - Reuse the existing TLS used by DD-P6-003's queued-emission `IN_DRAIN` flag; same TLS underpins thread affinity (see DD-M2-P6-005).
      - **Verification:** pure-logic tests for Phase 1 ordering (FIFO emission order; topological resolution across two dependent Effects; last-wins reduces observer entries to one); pure-logic tests for Phase 3 mutation guard (state-mutating call inside observer returns the error); pure-logic tests for divergence state machine (cap break → Diverged → no-op except destroy; diagnostics payload populated). Drain-pass interaction with real `WindowState` is covered retroactively by the GUI checkpoint.
      - **Technical risk: Medium.** The riskiest step in the phase. Failure modes are local (cap value tuning, ordering edge cases for the M3 multi-binding case); structural retreat path is Option C of the ADR.
    - [x] **DD-M2-P6-007: `SignalRegistry` per-type struct.** In `wasamo-runtime/src/reactive.rs`, replace DD-M2-P5-005's provisional `properties: Rc<HashMap<String, Signal<i32>>>` with `SignalRegistry { i32s: HashMap<String, Signal<i32>>, strings: HashMap<String, Signal<String>> }`. `register_binding(target, expr, registry: &SignalRegistry)` becomes the final signature. Keys are `wasamoc`-resolved names per DD-M2-P6-004's name-resolution rules.
      - **Verification:** existing reactive unit tests adapted; new pure-logic tests for string-typed Signal registration and binding.
      - **Technical risk: Low.** Mechanical rewrite of the spike's single-type map.
    - [x] **DD-M2-P6-004: `wasamoc` lowering activities (parse, check, property-binding lowering, handler-body lowering, IR emit, file write-out).** Extend `wasamoc` from M1's parse+check shape:
      - Parse `state count: i32` declarations; emit Signal nodes in the IR (Signal ownership in `.ui`).
      - Restricted type inference: `i32` and string only; reject other types at check time.
      - Property-binding lowering: lower DSL binding expressions (`text: "Count: \{root.count}"`) to `HandlerExpr` AST in the IR.
      - Handler-body lowering: lower DSL handler bodies (`clicked => { count += 1 }`) to `HandlerExpr` AST.
      - Compile-time name resolution: flat namespace per `.ui`; reject undefined references and duplicate `state` names; IR carries already-resolved names.
      - IR emit + file write-out per DD-M2-P6-002 grammar.
      - **Verification:** unit tests for each lowering pass (pure logic); end-to-end round trip — `wasamoc counter.ui` produces IR that the loader (next step) constructs into the expected widget tree. `cargo test -p wasamoc` green.
      - **Technical risk: Low–medium.** Lowering shape is small (counter exercises one binding + one handler); type inference is two cases.
    - [x] **DD-M2-P6-006: Productionise the IR loader.** New `wasamo-runtime/src/ir_loader.rs` consuming the DD-M2-P6-002 normative grammar; builds via Phase 4 internal mutation API (`append_child` / `set_property`); constructs `SignalRegistry` from IR `state` nodes; calls `register_binding` for each binding; wires handler bodies via `set_inline_handler` (Phase 3 path). Activates the registry through a thread-local handoff (`reactive::set_active_registry`) so click-handler dispatch (`widget::hit_test_click_inner`) reaches it via the new `HandlerEvalContext` (read/write adapter; replaces the Phase-3-era `NullEvalContext` placeholder).
      - **Verification (round-trip):** 13 `ir_loader` parser unit tests (covering header validation, all production forms, full counter round-trip via a hand-rolled mini-emitter); 3 cross-crate integration tests in `wasamo-runtime/tests/ir_loader_roundtrip.rs` that feed `wasamoc::emit` output through `parse_ir` and assert structural `IrComponent` equality. `wasamoc` was added as a dev-dependency for this purpose.
      - **Verification (GUI):** `exp/m2-p6-ir-loader-checkpoint` (commit `5a19446`) — driver crate `experiments/ir-loader-checkpoint/` runs `wasamoc` → `parse_ir` → `build_widget_tree` → `window_set_root` and renders the counter identically to the Phase 5 reactive spike (commit `fdc1545`). Owner manual confirmation 2026-05-07: title-font "Count: N" label updates on Increment click through the reactive path. Branch is verification-only and does not merge to `main`.
      - **Plan deviations (record for Phase 6 retrospective):**
        - **C1.5 — `HandlerExpr` unification (commit `00246ce`).** The Phase-3-era `wasamo_runtime::handler::HandlerExpr` was structurally identical to the DD-M2-P6-003 `wasamo_ir::HandlerExpr` (variant set + payload shapes), with cosmetic differences only (`PropRead.name`/`.path`; `CompoundOp::PlusEq`/`Add`). DD-M2-P6-006 was the first place the two types met; consolidating into a single `wasamo-ir`-resident definition (with `PropRead { path }` and `CompoundOp::Add/Sub/Mul/Div`) avoided a pointless conversion pass in the loader and a duplicate type to maintain. The IR text-grammar surface (DD-M2-P6-002 normative form: `+= -= *= /=`) is unchanged. Worth re-examining at Phase 6 close as a small structural change that landed inside an implementation step rather than as a standalone DD.
        - **`wasamo-ir` crate extraction (commit `f8f7d3d`).** Moving IR types from `wasamoc/src/ir.rs` into a dedicated `wasamo-ir` crate fixed the dependency direction (compiler → wasamo-ir ← runtime, instead of runtime → wasamoc). Same retrospective note as above — landed under DD-M2-P6-006 rather than as a standalone restructuring DD.
        - **Spike-decommission sub-bullets were no-ops on the integration branch.** DD-M2-P6-006's wording assumes the Phase 2 spike (`exp/m2-p2-ir-loader-spike`, commit `b7ab4dc`) had landed on `main`; in fact the spike never merged out of its exp branch, so the literal "move", "remove feature flag", and "decommission `experiments/ir-spike/counter.uic`" sub-bullets had no targets. The runtime gained a fresh `ir_loader.rs` rather than a renamed `experimental_ir_loader.rs`.
        - **Public API additions for IR-loader callers.** Added `wasamo_runtime::get_text_renderer` (commit `05837ba`) and `wasamo_runtime::window_set_root` (commit `a0483dc`) — both symmetric with existing accessors — to give external loader callers (the GUI checkpoint harness now, the C ABI later in DD-M2-P6-005) the surface they need without reaching into private modules.
        - **`DD-M2-P6-006` complete in commits `f8f7d3d` (wasamo-ir extract) → `00246ce` (HandlerExpr unify) → `779a6af` (ir_loader.rs + active-registry wiring + tests) → `05837ba` (get_text_renderer) → `a0483dc` (window_set_root). GUI checkpoint at exp commit `5a19446`.**
    - [x] **DD-M2-P6-009: Defense-in-depth IR loader validation.** Within `ir_loader.rs`:
      - Verify magic + version header line; mismatch → `WASAMO_ERR_IR_MALFORMED`.
      - Verify reference resolution (every binding/handler name resolves to a declared `state` or widget); failure → `WASAMO_ERR_IR_MALFORMED`.
      - Verify top-level document structure.
      - Trust emitter-side type integrity (no per-node re-validation).
      - **Verification:** pure-logic tests with hand-crafted malformed IR fragments — each failure path returns `WASAMO_ERR_IR_MALFORMED` with a populated `wasamo_last_error_message`.
      - **Plan deviation (record for Phase 6 retrospective):** widget-instance references are not yet representable in the IR — DSL `widget_decl` carries a type name only, with no per-instance `id`. Reference resolution is therefore restricted to `state` names in M2. The `or widget` clause above is forward-looking to a future grammar extension; the open question is captured in [docs/notes/dsl-grammar.md](../notes/dsl-grammar.md) Q1 to be revisited when M3 introduces Grid/List per-item context (or earlier if a concrete need surfaces).
      - **Defense-in-depth surface implemented in commit `476a1ea`:** `IrLoadError::Validate` variant + `is_malformed()` helper for DD-M2-P6-005 to map onto `WASAMO_ERR_IR_MALFORMED`; post-parse `validate()` pass enforcing unique `state` names and reference resolution; 13 new pure-logic tests covering all malformed paths (header, document structure, undeclared references in `PropRead`/`Assign`/`CompoundAssign` including nested in `Block`/`Interpolation` and child nodes, duplicate `state` names).
    - [x] **DD-M2-P6-005: `wasamo_load_ui` C ABI + error infrastructure.** In `wasamo-runtime/src/abi.rs` and `bindings/c/wasamo.h`:
      - Add `WasamoStatus wasamo_load_ui(WasamoLoadType type, const void* data, size_t data_len, WasamoWindow** out_root)` — single-function loader (Option α). The `type` discriminant chooses between `WASAMO_LOAD_PATH` and `WASAMO_LOAD_MEMORY` (sub-decisions A and C). The `(data, data_len)` shape (instead of a NUL-terminated `const char*`) follows the existing `(ptr, len)` convention used by every other string-bearing wasamo ABI and admits a future binary IR without ABI breakage.
      - Add `const char* wasamo_last_error_message(void)` — thread-local last-error string (sub-decision (i)).
      - Add error codes: `WASAMO_ERR_OBSERVER_MUTATION`, `WASAMO_ERR_REACTIVE_DIVERGED`, `WASAMO_ERR_REENTRANT_LOAD`, `WASAMO_ERR_WRONG_THREAD`, `WASAMO_ERR_IR_MALFORMED`.
      - Thread affinity: every `wasamo_*` ABI (except `wasamo_init` / `wasamo_last_error_message`) checks the runtime's owning thread; cross-thread call returns `WASAMO_ERR_WRONG_THREAD` without side effect. **Owning thread is fixed at `wasamo_init` time** per existing abi_spec §6 (which DD-M2-P6-005 inherits and extends with the explicit error-return rule); `WasamoWindowHandle` was a synonym for the existing `WasamoWindow*` type used by `wasamo_window_create` and friends, so the same handle type is reused.
      - Handle ownership: `WasamoWindow*` returned by `wasamo_load_ui` is runtime-owned; valid for runtime lifetime (no per-window destroy in M2).
      - Update `docs/abi_spec.md`: §3.1 status codes, §5.2 for `wasamo_load_ui`, §6 thread-affinity (init-time fix and error-return rule).
      - **Carry-over from DD-M2-P6-006:** wire the existing `reactive::divergence_diagnostics()` accessor into the `wasamo_last_error_message` payload (or delete it if the structured form is not used by any caller after this task lands). Currently dead-code; the warning is held open intentionally until this task resolves it before main merge.
      - **Verification:** build + `dumpbin /exports` confirms new symbols (`wasamo_load_ui` exported); pure-logic test exercising `WASAMO_ERR_WRONG_THREAD` (spawn thread, call ABI, assert error) plus argument validation and resource-resolution dispatch for both `WASAMO_LOAD_PATH` and `WASAMO_LOAD_MEMORY` modes; `cargo build --workspace` produces no `divergence_diagnostics is never used` warning.
      - **Plan deviation (record for Phase 6 retrospective):** the original task description used the placeholder names `WasamoWindowHandle` and `uint32_t flags` and asserted "owning thread fixed at `wasamo_load_ui` time". The implemented signature uses the explicit `(WasamoLoadType type, const void* data, size_t data_len, WasamoWindow** out_root)` shape — a NUL-terminated `const char* resource` was rejected during implementation review because it cannot carry an in-memory blob safely (binary IR contents may include `0x00`, and a missing NUL terminator on an embedded-string would silently overrun). Owning-thread capture stays at `wasamo_init` time to match the existing abi_spec §6 contract; the docs/decisions ADR's prose already implied a `(pointer, length)` blob, so this is an internal consistency fix rather than a substantive change. The pure-logic test seam is exposed via `wasamo_runtime::ffi::__install_owning_thread_for_test` so the cross-thread test runs without standing up a live Compositor.
    - [x] **DD-M2-P6-008: Counter examples migration (acceptance A1/A2).** `examples/counter/counter.ui` (single shared file) is now the source-of-truth for all three host examples; imperative tree construction in `examples/counter-{c,rust,zig}/` has been replaced.
      - **counter-rust** (commit `de7f26a`): `build.rs` runs the in-process `wasamoc` lib (parser + check + lower + emit) on `examples/counter/counter.ui` and writes `counter.uic` to `OUT_DIR`; `main.rs` hands the absolute path to `wasamo_load_ui` (`WASAMO_LOAD_PATH`) via the raw `wasamo-sys` binding. Cargo dependency switched from the safe `wasamo` wrapper to `wasamo-sys` per the directive.
      - **counter-c** (commit `3435cb4`): CMake's `add_custom_command` runs `wasamoc.exe build counter.ui counter.uic`, then `embed_uic.cmake` (a small CMake-builtin script using `file(READ ... HEX)`) generates `counter_uic.h` containing the IR bytes as a `static const unsigned char[]`; `main.c` includes that header and passes the `(pointer, length)` blob to `wasamo_load_ui` (`WASAMO_LOAD_MEMORY`).
      - **counter-zig** (commit `ca3d987`): `build.zig` invokes `wasamoc.exe` via `addSystemCommand`/`addOutputFileArg`; the resulting `LazyPath` is exposed to `main.zig` as the anonymous import `counter_uic`, which `@embedFile` inlines at compile time. `main.zig` hands the blob to `wasamo_load_ui` (`WASAMO_LOAD_MEMORY`) through the raw `wasamo.c` extern surface. Required adding `wasamo_load_ui`, `WasamoLoadType`, and `WASAMO_LOAD_PATH/MEMORY` constants to `bindings/zig/wasamo.zig` (the C and Rust bindings already had them from DD-M2-P6-005).
      - **Direct ABI calls only.** Each host contains zero imperative widget construction and zero `wasamo_set_property` calls — A2 structurally enforced.
      - **Verification:** **GUI manual confirmed 2026-05-08 on owner's local Windows desktop** (per [docs/notes/verification-environments.md](../notes/verification-environments.md), kind = "GUI/interactive verification on local or RDP-attached desktop"). All three counters launched from `counter.ui`, "Increment" click visibly updated the label through the reactive path, no host-side property writes. Screenshots archived under `private/screenshot counter-{rust,c,zig} 2026-05-08 *.png` (owner-local, not committed).
      - **Plan deviations (record for Phase 6 retrospective):**
        - **counter.ui already existed.** It was committed during a prior step (Phase 5/6 spike) and matched the wasamoc-acceptable DSL surface; no re-creation was necessary. The plan wording "Create `examples/counter/counter.ui`" was vestigial.
        - **Embed-at-compile-time means embed the wasamoc-emitted IR (`.uic`), not the DSL source (`.ui`).** The plan says "embed `counter.ui` at compile time" but `wasamo_load_ui` only accepts the IR text grammar (`;wasamo-ir v0` header). Resolved as Approach B (each build system invokes `wasamoc` at build time) over Approach A (commit `.uic` alongside `.ui`) to preserve a single source-of-truth. The pipeline is recorded as provisional in `docs/architecture.md` §1 ("DSL build pipeline") with re-evaluation triggers (M3 multi-`.ui` host, post-1.0 hot reload). Operational rule (`cargo build -p wasamoc` must precede C/Zig host builds) recorded in `CLAUDE.md` "Build ordering requirements".
        - **Window-level DSL props (`title`, `backdrop`, `theme`) silently dropped at the runtime boundary.** `wasamoc` correctly lowers them onto root-widget IR props, but `ir_loader::construct_widget` only honors widget-recognized props and `wasamo_load_ui` hardcodes the window title to `"Wasamo"`. M2-side window title is therefore `"Wasamo"` rather than M1's `"Counter"`. A1/A2 acceptance is unaffected (no title-text requirement in either criterion). Wiring window-level DSL props would require ABI extension (new `wasamo_window_set_title` or `wasamo_load_ui` config arg), which is out of Phase 6's "counter examples migration" scope. Captured as `docs/notes/dsl-grammar.md` Q2 with re-evaluation triggers (M3 DSL spec drafting, M4 Mica/Acrylic introduction).
      - **Technical risk: Low.** Embedding ergonomics are binding-side; the runtime path is identical to the Phase 5 spike harness.
    - **DD-M2-P6-011 (String-typed property binding) — moved to M2-Phase 7.** The 2026-05-08 acceptance-criteria revision split this DD off the Phase 6 task list and made it part of A6 under the new Phase 7 (`.ui` String property bound to `Signal<String>`). Phase 6 closes with `i32`-typed binding only, which suffices for A1/A2.
    - [x] **Upstream-document bundle (landed alongside ADR Accepted flip 2026-05-07; commit `d5415fe` and predecessors):**
      - [x] `VISION.md §4 Principle 2` supplement text appended.
      - [x] `docs/architecture.md` §6.8.3 rewritten to the three-phase + terminal drain form; mutation boundary documented; DD-M2-P3-002 side obligation discharged.
      - [x] `docs/decisions/m2-phase-5-reactive-engine.md` DD-M2-P5-004 Status flipped to "Superseded in part by DD-M2-P6-001"; deferred-dispatch trigger contract preserved.
      - [x] `docs/decisions/m2-phase-5-reactive-engine.md` DD-M2-P5-005 Status flipped to "Superseded in part by DD-M2-P6-007"; registration API preserved.
      - [x] `docs/notes/dd-m2-p6-drain-transaction.md` Status flipped to `superseded` (file retained as draft history; content folded into the ADR).
      - [x] `docs/notes/reactive-drain-cascade-policy.md` Status flipped to `superseded` (frontmatter; superseded-by points to DD-M2-P6-001).
    - [x] **Local clean rebuild green (Phase 6 closing, 2026-05-08):** `cargo clean` → `cargo build --release --workspace` → `cargo build --workspace` → `cargo test --workspace` — 177 passed, 0 failed across 16 test binaries.
    - [x] **Link/export verification (Phase 6 closing, 2026-05-08):** `dumpbin /exports target/release/wasamo.dll` confirms `wasamo_load_ui` and `wasamo_last_error_message` among the 27 exported `wasamo_*` symbols.
    - [x] `CHANGELOG.md` — Phase 6 entry added at closing.
    - [x] `docs/plans/m2-plan.md` Progress: phase ticked, ADR linked.
  - **Boundary with adjacent phases:**
    - vs Phase 3: Phase 6 wires the IR-side serialisation of `HandlerExpr` (DD-M2-P6-003 promotes the spike's tagged-value form). The handler evaluator itself is unchanged from Phase 3.
    - vs Phase 4: tree construction during `wasamo_load_ui` uses Phase 4's *internal* Rust mutation API (`insert_child` / `set_property` / etc.), not the C ABI — same pattern as the Phase 2 spike's loader. The new `wasamo_load_ui` is the only C ABI entry this phase adds beyond Phase 4's six tree-mutation symbols, plus `wasamo_last_error_message`.
    - vs Phase 5: Phase 6 supersedes DD-M2-P5-004's three-stage drain (Option D's three-phase form replaces it) and supersedes DD-M2-P5-005's `properties` parameter shape (`SignalRegistry` replaces it). The Signal/Effect/Binding primitives themselves and the dependency-tracker are unchanged.
    - Per-target-language codegen is **not required** (per DD-M2-P2-001 = B, a single runtime-side path suffices).
  - **Verification kinds:** unit tests (lowering passes / IR loader validation / drain ordering rules / divergence state machine — all pure logic) + build (`cargo build --release --workspace`) + link/export (`dumpbin /exports`) + ABI smoke + **GUI manual (RDP / physical) of all three language counters on real hardware — this is acceptance A1/A2 itself**. A green CI build alone does not satisfy A1/A2 (see [verification-environments.md](../notes/verification-environments.md) Observation 1).
- [ ] **M2-Phase 7 — Reactive Foundation Hardening & Contract Finalization**
  - ADR: [docs/decisions/m2-phase-7-reactive-foundation.md](../decisions/m2-phase-7-reactive-foundation.md) houses the three DDs (DD-M2-P6-010 / 011 / 012) — **Status: Proposed** for all three; pre-doc cycles re-run at Phase 7 entry per DD before any are flipped to Accepted. The DDs retain their `DD-M2-P6-NNN` numbering as historical surface-time identifiers; the Phase 6 ADR retains stub anchors that forward to the Phase 7 ADR. (ADR scaffolding migration committed 2026-05-08.)
  - **Scope.** Discharge the three DDs deferred from Phase 6 closing. Phase 6 established the `.ui → runtime` pipeline (A1/A2); Phase 7 establishes the foundation guarantees that distinguish "it runs" from "it is a Foundation" (A5/A6).
  - **Order of work (agreed 2026-05-08):**
    1. **DD-M2-P6-010** (dirty_effects topological sort fidelity) — re-run pre-doc with full Phase 6 implementation evidence; agreement → Accepted; replace EffectId-numeric-order approximation with true graph walk.
    2. **DD-M2-P6-012** (re-entrancy / safety-guard placement principle) — re-run pre-doc with full Phase 6 implementation evidence (now available; ADR Context section already reflects the Phase 5 retrospective bug shape); agreement → Accepted; record the principle in `docs/architecture.md` as a global runtime invariant (per A5).
    3. **DD-M2-P6-011** (String-typed property binding) — re-run pre-doc; agreement → Accepted; implement until a `.ui` String property bound to `Signal<String>` propagates through `BindingEvalContext` / `HandlerExpr::PropRead` to the visible widget (per A6).
  - **Acceptance discharged here:**
    - **A5** (Reactive Foundation Hardening) — both DD-M2-P6-010 and DD-M2-P6-012 must be Accepted with implementation landed; the guard-placement principle must be recorded in `docs/architecture.md`.
    - **A6** (Type-Agnostic Reactive Binding) — DD-M2-P6-011 Accepted with end-to-end `Signal<String>` binding demonstrated.
  - **Verification kinds:** unit tests (topo sort fidelity / guard-placement enforcement / String binding propagation — pure logic) + the existing Phase 6 GUI counter (regression check; no new GUI fixture mandated unless a DD demands it).
  - **Closing items:** `cargo build --release --workspace` + `cargo test --workspace` green; `CHANGELOG.md` Phase 7 entry; ROADMAP M2 marked shipped; M2-plan status → `completed`.
- **Out of M2 scope (restated):**
  - Headless-verification backend — examined critically in [docs/notes/headless-verification.md](../notes/headless-verification.md); the policy is to close M2 with a pure-logic test fixture strategy without building one. Re-evaluation trigger in Phase 5.

### Notes

_Empty._
