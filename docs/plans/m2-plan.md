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
- [ ] **M2-Phase 3 — Handler execution location**
  - ADR: [docs/decisions/m2-phase-3-handler-exec-location.md](../decisions/m2-phase-3-handler-exec-location.md) — **Accepted 2026-05-04**
  - [x] `docs/decisions/m2-phase-3-handler-exec-location.md` — pre-doc filed (DD-M2-P3-001..004); status "Proposed"
  - [x] Owner agreement on DD-M2-P3-001 (Option A: runtime-side interpreter), DD-M2-P3-002 (Option B: separate paths, inline first), DD-M2-P3-003 (Option A: catch_unwind + stderr), DD-M2-P3-004 (Option B: IR reserves optional span; coarse identifiers in M2)
  - [x] ADR status → **Accepted**
  - [x] `docs/notes/headless-verification.md` — new live note: ヘッドレス検証機構の必要性検討 (Phase 3 verification gap を契機に起草; M2 内では構築せず pure-logic test fixture 戦略で閉じる)
  - [x] `docs/plans/m2-plan.md` Progress: phase still **open** — task list expanded below; coding work begins next session
  - **Implementation scope (this phase, scheduled for next session):**
    - [ ] `wasamo-runtime/src/handler.rs` 新規 — `HandlerExpr` enum (assign / `+=` `-=` `*=` `/=` / property read+write / int literal / block) + `EvalContext` trait + `evaluate()` + 単体テスト (assign / compound / wrapping overflow / nested block)
    - [ ] `WidgetNode` に inline-handler slot 追加 + signal emit 経路を「inline 評価 → host listener iter」順に改造 (DD-M2-P3-002 Option B); fake listener list で順序検証 unit test
    - [ ] handler invoke を `std::panic::catch_unwind` で wrap + 書式 `wasamo: handler error in <component>.<widget-path>.<signal>: <message>` で stderr ログ (DD-M2-P3-003); panic injection unit test
    - [ ] coarse identifier `<component>.<widget-path>.<signal>` formatter (DD-M2-P3-004 Option B); pure logic として単体テスト
    - [ ] `cargo build --release --workspace` + `cargo test --workspace` 緑、push、CI Windows runner 緑
  - **Boundary with adjacent phases:**
    - vs Phase 4: handler は internal `set_property` のまま (C ABI 越えない — DD-M2-P3-001 Option A の本質)。Phase 4 の C ABI 化は handler 経路を再触しない。
    - vs Phase 5: `HandlerExpr` evaluator は handler 軸のみ実装。binding 評価器との共通基盤化は Phase 5 で実施 (handler evaluator が Phase 5 の出発点)。
    - vs Phase 6: `HandlerExpr` は in-memory enum として定義。textual IR ↔ `HandlerExpr` の serialization 接続は Phase 6 で実施。Phase 3 では `experiments/ir-spike/` の throwaway IR は触らない (Phase 6 で全面再設計)。
  - **GUI 検証は本フェーズでは実施しない.** Phase 5 (reactive 統合) 完了時に counter の click → label 更新が e2e で動くことで遡及的に確認。理由は [docs/notes/headless-verification.md](../notes/headless-verification.md) 参照。
- [ ] **M2-Phase 4 — Tree-mutation ABI primitives**
  - ADR: _not yet filed_
  - **Implementation scope (provisional, settled at pre-doc time):**
    - 既存 internal builder (insert / remove / replace child; property set) を C ABI に昇格。
    - 複数 property write のバッチ化 (Phase 5 invalidation cascade の amortize 用)。
    - `wasamo.dll` の export 表に新シンボル追加 (`dumpbin /exports` で検証)。
  - **Boundary with adjacent phases:**
    - vs Phase 3: handler は Phase 3 で internal `set_property` を直接呼ぶ実装にしてあり、Phase 4 C ABI 化後も internal 経路を維持 (re-entrancy 回避 + DD-M2-P3-001 Option A の利点保持)。
    - vs Phase 5: reactive engine の invalidation cascade は Phase 4 の batching primitive に乗る。Phase 4 が batching API を出さないと Phase 5 が大量 write を amortize できない。
    - vs Phase 6: `wasamo_load_ui` (Phase 6) は新 C ABI 1 本だが、tree 構築自体は Phase 4 の primitive を runtime 内部から使う想定。
  - **検証種別:** build (`cargo build --release --workspace`) + link/export (`dumpbin /exports target/release/wasamo.dll`) + 単体テスト (pure logic 部分のみ; ABI surface は CI build 緑で代替)。GUI 検証は本フェーズ単独では不要。
- [ ] **M2-Phase 5 — Reactive engine**
  - ADR: _not yet filed_
  - **Implementation scope (provisional, settled at pre-doc time):**
    - property → binding の依存グラフ (dependency tracker) — Solid / Vue 系の signals パターンを参考に数百行規模。
    - property write 観測 → 依存 binding の invalidate → 再評価 → widget property 書込 → 必要に応じて relayout/render 起動。
    - binding expression evaluator (read-only / 文字列補間 `"Count: \{root.count}"` 含む) — Phase 3 の `HandlerExpr` evaluator を共通基盤に格上げ (handler evaluator は side-effecting / binding evaluator は pure read; 評価器の core を共有)。
    - DD-P8-002 の "whole-window dirty" 経路に上乗せ。subtree 粒度は acceptance demand があれば検討、なければ live note に open question として残す。
  - **Boundary with adjacent phases:**
    - vs Phase 3: Phase 3 の `HandlerExpr` を読込み、評価器 core を共通化。Phase 3 で property write が呼ばれた時に Phase 5 の dependency tracker が hook 経由で invalidate を起動する。
    - vs Phase 4: Phase 4 の batching primitive 上で実装。Phase 4 が batching を提供しないと再評価カスケードが性能悪化。
    - vs Phase 6: Phase 6 が `.ui` の binding 文を typed IR に降下し、Phase 5 の binding expression evaluator が消費する。
  - **検証種別:** unit test (dependency tracker, binding evaluator は pure logic) + GUI 手動 (実機で counter の reactive 連動を確認 — acceptance A2)。**ヘッドレス検証機構の必要性が顕在化する可能性の高いフェーズ**; Phase 5 着手時に [docs/notes/headless-verification.md](../notes/headless-verification.md) を再評価し、必要なら "no-Compositor" mode の独立 ADR を起こす。
- [ ] **M2-Phase 6 — `.ui → runtime` lowering**
  - ADR: _not yet filed_
  - **Implementation scope (provisional, settled at pre-doc time):**
    - `wasamoc` typed IR emit — DD-M2-P2-003 activities 1-7 を実装 (parse → check → 型推論 → property binding lowering → handler body lowering → textual IR 出力)。
    - textual IR の normative grammar 起草 (DD-M2-P2-002 Option B; s-expression 風)。
    - `wasamo-runtime` 側 textual IR parser — Phase 2 spike の `experimental_ir_loader` を production 化、`HandlerExpr` (Phase 3) と binding expression (Phase 5) を生成する。
    - 新 C ABI 1 本: `wasamo_load_ui(path, &out_root)` 系。
    - `examples/counter-{c,rust,zig}/` を `.ui` 駆動 host に置換 — acceptance A1。
    - `architecture.md` 改訂: §6 (or M2 改訂版) に signal dispatch 順序の runtime contract を記載 (DD-M2-P3-002 末尾の指示; 実物が Phase 6 で揃うのでここで記述)。
  - **Boundary with adjacent phases:**
    - 新規 core 機構なし。Phase 3 の `HandlerExpr`、Phase 4 の C ABI primitive、Phase 5 の binding evaluator を消費する純統合フェーズ。
    - 出力先言語別の codegen は **不要** (DD-M2-P2-001 Option B により runtime 側 1 本で足りる)。
  - **検証種別:** build + 単体テスト (textual IR parser は pure logic) + **GUI 手動 (RDP / 物理) で 3 言語すべての counter を実機確認 — acceptance A1/A2 そのもの**。CI build 緑だけでは A1/A2 は満たせない (verification-environments.md Observation 1 参照)。
- **Out of M2 scope (再掲):**
  - ヘッドレス検証 backend — [docs/notes/headless-verification.md](../notes/headless-verification.md) で批判的に検討、構築せず pure-logic test fixture 戦略で M2 を閉じる方針。Phase 5 で再評価トリガあり。

### Notes

_Empty._
