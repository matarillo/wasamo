---
milestone: M2
status: in-progress
roadmap-anchor: ROADMAP.md#m2-foundation
adrs:
  - docs/decisions/vision-post-m2-roadmap.md
  - docs/decisions/m2-phase-1-cdylib-shim.md
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

### Phase dependencies

```
M2-Phase 1   ── independent infra; lands any time

M2-Phase 2 ─┐
M2-Phase 3 ─┤
            ├─ M2-Phase 4 ─ M2-Phase 5 ─ M2-Phase 6
```

M2-Phases 2 and 3 are decision phases and can run in parallel; both
gate M2-Phase 4. M2-Phase 5 depends on 4. M2-Phase 6 depends on the
decisions (2, 3) and on 5.

### Acceptance ↔ phase mapping

| Acceptance | Phase(s) |
|---|---|
| A1 (`counter.ui` drives all three hosts) | M2-Phase 6 |
| A2 (reactive propagation, no host wiring) | M2-Phase 5, M2-Phase 6 |
| A3 (cdylib-shim cleanup) | M2-Phase 1 |
| A4 (tree-mutation ABI primitives) | M2-Phase 4 |

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
- [ ] **M2-Phase 5 — Reactive engine**
  - ADR: [docs/decisions/m2-phase-5-reactive-engine.md](../decisions/m2-phase-5-reactive-engine.md) — **Proposed 2026-05-05** (DD-M2-P5-001..006; awaiting owner agreement)
  - Live note: [docs/notes/architectural-family.md](../notes/architectural-family.md) — records the tree-with-bindings family as a working hypothesis (not a long-term commitment); re-evaluation triggers documented for M3 DSL spec drafting and post-1.0 hot reload.
  - Pre-aligned design axes: [docs/notes/m2-phase-5-design-axes.md](../notes/m2-phase-5-design-axes.md) — owner direction (2026-05-05) on dependency-tracker depth and Option A verification, recorded before pre-doc.
  - **Risk concentration (the risk this phase must absorb within M2):**
    - This is the only technical-thesis validation point in M2 (A2: reactive propagation without host wiring). Every other phase stays in structural goals (A3) / ABI surface extension (A4) / integration (A1); the M2 foundation hypothesis — "a dependency tracker on top of DD-P8-002's whole-window dirty + queued emission" — is exercised here for the first time.
    - Decisions punted by other phases accumulate here: settling the `with_batched_writes` shape from Phase 4 (DD-M2-P4-004 = A); deciding whether Phase 3's `HandlerExpr` evaluator and the binding evaluator share a common core; the subtree-granularity open question left by DD-P8-002 ([layout-engine note §3.4](../notes/layout-engine.md)); re-evaluation of headless verification ([headless-verification note](../notes/headless-verification.md)).
    - Downstream rework costs (Phase 6 / M3+) depend heavily on this phase's shape. Phase 6 introduces no new mechanisms and just consumes Phase 5's evaluator output (the typed-IR representation of binding statements) — getting the shape wrong here forces a regression all the way back to Phase 2's textual IR normative grammar. M3+ binding-feature extensions (Grid cell bindings, List per-item context) also stack on top of this phase's dependency-tracker design.
    - Two risk-taking axes: (a) dependency-tracker design depth — settling for minimum viable (counter's single binding working is enough) means a rewrite in M3. Decide at pre-doc time whether to adopt the Solid / Vue signals prior-art pattern from the start. (b) commitment to headless verification — test in this phase whether the pure-logic fixture policy actually holds; if not, file an independent ADR for a no-Compositor mode. Discovering at Phase 6 (GUI manual verification mandatory) that headless is needed is the worst-case trajectory, so decide at the start of this phase.
  - **Implementation scope (provisional; pre-doc proposes the shape below per DD-M2-P5-001..006, settled when the ADR is Accepted):**
    - Signal + Effect 2-layer reactive primitive with read-time auto-tracking (DD-M2-P5-001 = B; DD-M2-P5-002 = B). `Signal<T>` is an observable storage cell; `Effect` is a re-runnable closure whose dependencies are auto-collected via a thread-local current-effect stack. No `Computed` layer in M2.
    - Effect lifetime tied to an owner Drop handle (DD-M2-P5-003); explicit disposal removes the effect from all dependent Signals' dependent sets.
    - Outermost-frame deferred dispatch (DD-M2-P5-004) — re-evaluation flush composes with `emit::drain_if_outermost`, not bypassing the queued-emission rule (DD-P6-003).
    - `BindingTarget` enum + `register_binding()` API (DD-M2-P5-005; `pub(crate)`) — encodes the "binding lives on a tree node" assumption; family-coupled but internal, revisable without C ABI churn (see architectural-family note).
    - Binding expression evaluator reusing Phase 3's `HandlerExpr` evaluator core through a read-only `EvalContext` variant that records reads (DD-M2-P5-006). Includes string interpolation like `"Count: \{root.count}"`.
    - Layered on top of DD-P8-002's "whole-window dirty" path; binding writes go through `set_property` and inherit existing layout-dirty marking. Subtree granularity stays out of scope (open question in [layout-engine note §3.4](../notes/layout-engine.md)).
    - No C ABI symbols added (per DD-M2-P4-004 = A). All Phase 5 types are `pub(crate)` in `wasamo-runtime/src/reactive.rs`.
  - **Boundary with adjacent phases:**
    - vs Phase 3: consumes Phase 3's `HandlerExpr` and shares the evaluator core. When Phase 3 triggers a property write, Phase 5's dependency tracker fires invalidation via a hook.
    - vs Phase 4: implemented on top of Phase 4's `with_batched_writes` skeleton. Without Phase 4 batching, re-evaluation cascades degrade performance.
    - vs Phase 6: Phase 6 lowers `.ui` binding statements into typed IR, which Phase 5's binding expression evaluator consumes.
  - **Verification kinds:** unit tests (dependency tracker and binding evaluator are pure logic; Option A verification per the design-axes note uses fake Effect closures with no new mirrors and no headless backend) + GUI manual (verify reactive linkage of the counter on real hardware — acceptance A2). **The phase most likely to surface a need for a headless-verification mechanism**; on entering Phase 5, re-evaluate [docs/notes/headless-verification.md](../notes/headless-verification.md) and, if needed, file an independent ADR for a "no-Compositor" mode.
- [ ] **M2-Phase 6 — `.ui → runtime` lowering**
  - ADR: _not yet filed_
  - **Implementation scope (provisional, settled at pre-doc time):**
    - `wasamoc` typed IR emit — implement DD-M2-P2-003 activities 1–7 (parse → check → type inference → property binding lowering → handler body lowering → textual IR output).
    - Draft the textual IR's normative grammar (DD-M2-P2-002 Option B; s-expression-flavoured).
    - `wasamo-runtime` textual IR parser — productionise the Phase 2 spike's `experimental_ir_loader`; emits `HandlerExpr` (Phase 3) and binding expressions (Phase 5).
    - One new C ABI: `wasamo_load_ui(path, &out_root)` and friends.
    - Replace `examples/counter-{c,rust,zig}/` with `.ui`-driven hosts — acceptance A1.
    - Revise `architecture.md`: document the signal-dispatch ordering runtime contract in §6 (or its M2-revised version) (per the closing instruction of DD-M2-P3-002; the real implementation lands in Phase 6, so the description goes here).
  - **Boundary with adjacent phases:**
    - No new core mechanisms. A pure integration phase consuming Phase 3's `HandlerExpr`, Phase 4's C ABI primitives, and Phase 5's binding evaluator.
    - Per-target-language codegen is **not required** (per DD-M2-P2-001 Option B, a single runtime-side path suffices).
  - **Verification kinds:** build + unit tests (the textual IR parser is pure logic) + **GUI manual (RDP / physical) verification of all three language counters on real hardware — this is acceptance A1/A2 itself**. A green CI build alone does not satisfy A1/A2 (see verification-environments.md Observation 1).
- **Out of M2 scope (restated):**
  - Headless-verification backend — examined critically in [docs/notes/headless-verification.md](../notes/headless-verification.md); the policy is to close M2 with a pure-logic test fixture strategy without building one. Re-evaluation trigger in Phase 5.

### Notes

_Empty._
