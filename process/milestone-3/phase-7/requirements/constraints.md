---
title: M3-Phase 7 制約引き継ぎ — イテレーション grammar
status: accepted
created: 2026-06-09
source-phase: M3-Phase 6
target-phase: M3-Phase 7
---

# M3-Phase 7 制約引き継ぎ

ワークフロー §2.1 のアウトプット。前フェーズの永続記録
[M3-Phase 6 handoff.md](../../phase-6/implementation/handoff.md) から本
フェーズ（**イテレーション grammar**）に効く制約を切り出し、本 phase の
論点・スコープ・検証方針に合わせて再構成する。単純コピーではなく、各項目
に「Phase 7 でどう効くか」と **採否**（本 phase の constraints とするか /
別 owner へ送るか）を付す。

Phase 7 thesis（[plan.md](../../plan.md) Phase 7 行）の前提:

- **collection binding が widget-tree を生成する**。Phase 6 で binding は
  「property 値駆動」から「subtree の present/absent 駆動」へ届いたが、
  Phase 7 は「subtree の **cardinality**（0..N 個の反復生成）駆動」へ拡張
  する。gallery の thumbnail set が collection から生成される。
- per-item context の具体形（`item` / `index` の識別子命名、unified
  `HandlerExpr` enum に乗るか別 context 型が要るか、collection 型の露出
  方法）は **ADR で確定**する。plan は surface identity のみ commit し、
  syntactic form は commit しない。
- 同じ phase ADR が **`TypedValue` 圧力**を判断する（working assumption:
  scalar `bool` + 既存 `i32` / `String` で足り `TypedValue` は deferred の
  まま。圧力が surface したら ADR が explicit DD として開く）。
- 依存: IR / evaluator レベルでは layout-primitive phase と独立。ただし
  E2E proof（collection から生成する thumbnails）は Phase 4 の
  WrapPanel + ScrollView を再利用する。

引き継ぎ源は phase-6 handoff だが、論点に効く milestone-plan レベルの義務
（reactive-drain residual / `TypedValue` 圧力判断）も併記する。さらに
Phase 2–5 の out-of-phase residual の disposition（closed / M4-owned /
non-Phase7 を含む）を末尾「prior-phase residual sweep」で確認する。

---

## 1. 制御フロー family の拡張は `IrMember` / `ControlFlowNode` から始める（**Phase 7 核心**）

[phase-6 handoff §Phase 7 handoff targets](../../phase-6/implementation/handoff.md)
第1項。Phase 6 は `if` を **structural member**
（`IrMember::ControlFlow(ControlFlowNode::If)`）として出荷し、lowering /
textual IR / validation / static load-time presence / reactive mutation の
すべてが widget member と control-flow member を **明示的に dispatch** する。

**Phase 7 への効き方:**

- iteration（`for` / `foreach` 等）は `else` / `switch` と同じく
  **control-flow family の拡張**として実装する。widget として
  materialise しない。これは Phase 7 の grammar / textual-IR / loader /
  validator / roundtrip / traversal 変更すべてに効く設計方針。
- re-trigger（handoff 由来）: control-flow form を足す grammar /
  textual-IR / loader / validator / roundtrip / traversal の変更すべて。
- **採否:** **採用（実装制約: 構造の出発点）**。Phase 7 の control-flow
  family 拡張はここから始まる。

## 2. range-style structural target の landing point は `BindingTarget::ConditionalSubtree`（**Phase 7 核心**）

[phase-6 handoff §Phase 7 handoff targets](../../phase-6/implementation/handoff.md)
第2項。Phase 6 conditional runtime は subtree を insert/remove し、旧
subtree を dispose し、宣言 Visual 順を保ち、setter が戻る前に fresh
subtree の Effects を drain する **初の structural binding target** を作った。

**Phase 7 への効き方:**

- iteration の `ForLoopSubtree`（または同等の range target）は
  `ConditionalSubtree` と **同じ ownership 問題**を再利用して設計する:
  declared slot identity / insertion・removal の atomicity / registry
  teardown / effect drain のタイミング / failure reporting。
- conditional が「0 or 1 child」だったのに対し range は「0..N child」で
  ある点が拡張差分。slot identity と挿入位置の決定が単純な present/absent
  から index 付き列へ一般化される（→ §3 identity 論点と直結）。
- re-trigger（handoff 由来）: multi-child structural mutation target すべて。
- **採否:** **採用（実装制約: runtime 設計の中核）**。ownership 問題の
  再利用先は確定、具体化は §2.2 / 実装。

## 3. declared-tree / entity-tree identity モデルの決定（**Phase 7 で open する DD 論点**）

[phase-6 handoff §Phase 7 handoff targets](../../phase-6/implementation/handoff.md)
第3項。Phase 6 は意図的に **absent=fresh-on-return / 保持なし**で、removed
後 reinsert された conditional subtree は fresh entity として rebuild する。
これは [docs/dsl_spec.md](../../../../docs/dsl_spec.md) /
[docs/architecture.md](../../../../docs/architecture.md) に fold 済み。

**Phase 7 への効き方:**

- iteration は identity が **positional / keyed / 意図的に fresh** の
  どれかを決めなければならない。さらに、その identity が state / effects /
  disposal / Visual 順とどう相互作用するかを定義する必要がある。
- これは constraints というより本 phase で **新規に判断すべき論点**
  （→ §2.2 DD slate）であり、Phase 6 が fresh-on-return 以上を解決して
  いない、という事実が引き継ぎの実体。`key:` syntax / list diffing /
  retained subtree state / entity identity model のいずれかに触れる時点で
  発火する。
- re-trigger（handoff 由来）: `key:` syntax / list diffing / retained
  subtree state / entity identity model のいずれか。
- **採否:** **採用（DD 論点入力 — 未結論）**。identity モデルは Phase 7 で
  新規に判断する。constraints では「前 phase が fresh-on-return 以上を
  解決していない」事実のみ記録し、決定は §2.2 DD slate へ送る。

## 4. placement 格納モデルは range mutation が育つ前の Phase 7 決定（**Phase 7 で open する DD 論点**）

[phase-6 handoff §Phase 7 handoff targets](../../phase-6/implementation/handoff.md)
第4項。handoff が明示的に「**Phase 7 decision before range mutation
grows**」と名指しした項目。現行モデルは parent-owned placement metadata を
parallel vector（特に ZStack placements）で持つ SoA 構造のため、structural
mutation は materialised child list / placement metadata / live Visual
sibling 順を **1 つの invariant** として同時更新しなければならない。Phase 6
T5 の ZStack conditional path は `insert_child_with_zstack_placement` +
placement removal を要した。

**Phase 7 への効き方:**

- range primitive を実装する前に、placement 格納を **(a) 現行 SoA
  parallel-vector を維持 / (b) placement を child record へ移す（AoS） /
  (c) keyed metadata map** のどれにするか決める（→ §2.2 DD slate）。
  range insertion は 1 mutation で複数 child の placement を増減させるため、
  この格納モデルが mutation の atomicity と複雑度を直接左右する。
- **placement-like surface を変える時は compiler gate / runtime validator
  / default・alignment extraction を一括更新する**。1 つを「diagnostic
  だけ」と扱うのは T1/T2/T3 ZStack placement follow-up が drift を出した
  失敗パターン（→ §8 semantic-migration audit と連動）。
- re-trigger（handoff 由来）: `ForLoopSubtree` / range insertion、新しい
  parent-owned per-child metadata 種、widget-only path 経由の child 挿入。
- **採否:** **採用（DD 論点入力 — 未結論）**。格納モデルの選択は Phase 7 で
  決定（§2.2 DD slate）。handoff が "Phase 7 decision" と名指しした項目で、
  constraints では結論を先取りしない。

## 5. range mutation の structural failure observability の見直し（採用 / §2.4 へ）

[phase-6 handoff §Phase 7 handoff targets](../../phase-6/implementation/handoff.md)
第5項。Phase 6 は conditional mutation path の build / insert / remove
失敗を validation 後の **log-only diagnostics** に留めた。single-child
conditional branch では許容だが、multi-child range edit は **部分失敗**
しうるため、より強い runtime error か rollback story が要るかもしれない。

**Phase 7 への効き方:**

- range insert/remove が 1 mutation で複数 child に及び、途中で失敗した
  ときの観測可能性（error 報告 / rollback / partial-state の扱い）を
  Phase 7 の検証方針（→ §2.4）と設計（→ §2.2）の判断対象に含める。
- re-trigger（handoff 由来）: 1 mutation で 2 つ以上の child を insert/
  remove しうる structural edit すべて。
- **採否:** **採用（DD 設計入力 + 検証方針 — 未結論）**。error / rollback /
  partial-state の扱いは §2.2 設計と §2.4 検証方針で決定する。

## 6. reactive-drain residual の fix-or-carry 判断義務（plan-level 制約）

引き継ぎ源は 2 つ:
(i) [phase-6 handoff §Phase 7 handoff targets](../../phase-6/implementation/handoff.md)
第6項（DD-M3-P6-005 SM-1 carry-forward = reactive-drain residual 1-3 が
依然 deferred）、(ii) [plan.md §Risks](../../plan.md) と
[M2 handoff §3](../../../milestone-2/handoff.md)（silent carry-forward 不可、
phase pre-doc が fix/carry を明示判断する義務）。M2 handoff §3 の 4 項目:

1. **cycle detection policy**。
2. **ordering ties**（observable contract か implementation-defined か）。
3. **fan-out × `MUTATION_CAP`**。
4. **synchronous non-batched drain proof contract（M3-Phase 1 addendum）** —
   `BATCH_DEPTH == 0` での write が戻りまでに dirty Effect を drain する
   observable contract。

**Phase 7 への効き方:**

- iteration は 1 つの collection 変化が **複数 dependent effect に fan-out**
  しうる初の grammar（N child 生成 → N 個の effect drain）。Phase 6 ADR と
  同じく、Phase 7 ADR / framing は M2 handoff §3 の 4 項目を参照し、本
  phase の range-mutation 経路が具体的 failure を surface するか判断し、
  fix または carry-forward を **明示記録**する（silent carry-forward 不可）。
- 特に **item 3（fan-out × `MUTATION_CAP`）は Phase 7 直撃**: collection
  cardinality が大きいと range 生成が `MUTATION_CAP` に当たりうる。
  handoff も「reactive scheduler 変更 / batch update / multi-effect
  ordering / fan-out しうる structural mutation」を re-trigger として
  名指ししている。
- **採否:** **採用（DD 論点入力 — 未結論）**。fix-or-carry の明示判断は
  §2.2 DD slate で行う（silent carry-forward 不可）。

## 7. `TypedValue` 圧力判断（plan-level 制約、Phase 7 を名指し）

[plan.md Phase 7 行](../../plan.md) と [plan.md §Risks](../../plan.md)。
plan は Phase 7 を「M2-deferred `TypedValue` generic value union が避け
にくくなる最有力点」と名指しし（per-item context / collection element
type）、ADR が explicit DD として開くことを要求する。working assumption
は「scalar `bool` + `i32` / `String` で足り `TypedValue` は deferred」。

**Phase 7 への効き方:**

- per-item context（§3 identity と並ぶ Phase 7 の中心設計）が collection
  要素型をどう露出するかで `TypedValue` 圧力が決まる。「要らない」を
  default としつつ、圧力が surface したら ADR が explicit DD として開く
  （`TypedValue` 採用は M3 acceptance の revision を要するため smuggle
  不可、README acceptance-revision exception 経由）。
- **採否:** **採用（DD 論点入力 — 未結論）**。`TypedValue` 採否は §2.2 で
  explicit DD として予約し、結論は ADR で出す（plan の working assumption は
  "不要" だが先取りしない）。

## 8. semantic-migration audit gate の Phase 7 適用（**既に codify 済み — 新規 VDR 不要**）

[phase-6 handoff §Main learnings carried forward](../../phase-6/implementation/handoff.md)
第3項は semantic-migration audit を「forcing artifact として VDR に codify
せよ」という未了 learning として書いているが、**handoff が指す codify は
起草より前の 2026-06-06 に既に完了している**:
[DD-V-025](../../../cross-milestone/decisions/agents-md-and-rule-enforcement.md)
（Accepted 2026-06-06）が semantic-migration traversal audit を **Forcing
tier の gate** として確立し、[implementation-gates.md](../../../procedures/implementation-gates.md)
の trap #1（start gate）+ close artifact #1（call-site classification
table: `rg` query / files / per-class reason / tests added-or-not）に landed
済み。したがって本項は「VDR を新設する」ではなく「**既存 gate を Phase 7 の
IR migration に適用する**」である。

**Phase 7 への効き方:**

- Phase 7 は control-flow family（§1）と range BindingTarget（§2）の追加で
  **新たな IR / semantic migration を伴う公算が高い**。この migration は
  [implementation-gates.md](../../../procedures/implementation-gates.md) の
  **start gate（trap #1 semantic-migration / #2 side effects / #3
  parallel-data drift を選択・記録）+ close gate（call-site audit table 等の
  auditable artifact）** と、[AGENTS.md §Process rule lifecycle](../../../../AGENTS.md)
  の schema/IR-migration **full review lane** に従う。新規 VDR は不要。
- 唯一の residual: handoff が併記した「**compile-error-forcing 構築機構を
  silent-absorb ヘルパ（filtering iterator 等）より優先**」は、現行 trap #1
  の close artifact（call-site table）に機構選好として明記されていない。
  ただしこれは [implementation-gates.md §5](../../../procedures/implementation-gates.md)
  の lifecycle boundary 上 **新規 gate ではなく trap #1 への
  concrete-example 追記（minor edit）相当**で、VDR を要しない。Phase 7 の
  migration が実際にこの機構を用いるなら、その時点で trap #1 に minor edit
  するのが proportionate。
- **採否:** **適用（既存プロセス gate = DD-V-025 / implementation-gates）**。
  新規 VDR 不要。trap #1 への minor edit 要否は Phase 7 migration 実施時に
  判断する。

## 9. visible E2E の verification 規律（screenshot + 解析 + 陽性対照）

規範核は [AGENTS.md §Testing rules](../../../../AGENTS.md)（DD-V-024 後、
`CLAUDE.md` は `@AGENTS.md` の import shim）、capture mechanics / DPI-aware
capture / 陽性対照原理は
[verification-environments.md Observation 4](../../../../docs/notes/verification-environments.md)
に fold 済み（Phase 5/6 で SSOT 確定済み。ここでは再導出せず参照のみ）。

**Phase 7 への効き方:**

- Phase 7 の E2E proof（collection 駆動 thumbnails）は visible surface。
  assistant-automated evidence は launch + screenshot（`CopyFromScreen`、
  per-monitor-DPI-aware）+ assistant 解析を含める。owner human-visible
  smoke は代替しない。
- **陽性対照が Phase 7 で load-bearing**: 固定 N 個の thumbnails の単発
  frame は **hardcode した widget tree でも同じ見た目**を出しうる。
  iteration が「collection が cardinality を駆動する」ことを証明するには、
  collection を **変化させて item 数が連動して増減する** 2 frame 以上を
  撮る（Phase 6 の conditional toggle 前後 2 frame に対応する、iteration
  版の陽性対照）。
- **採否:** **採用（検証方針の前提）**。§2.4 で各 visible task の最小
  evidence に組み込む。

## 10. プロセス学び（final-step ownership 分割 / T0-frozen task list の stale 化）

[phase-6 handoff §Main learnings](../../phase-6/implementation/handoff.md)
第1・2項。

- **final-step は local evidence と phase-branch CI を分離して所有する**:
  local clean rebuild は code を変えた step が所有、GitHub Actions run id /
  on-CI Windows evidence は phase branch が CI を回した後の phase-end が
  所有。Phase 7 の `implementation/plan.md` 最終 task checklist は最初から
  この分割（task-end retro = 最終 task `[x]` 可、phase-end CI/run-id =
  phase→main merge gate 所有で `[ ]` のまま）で書く。
- **T0 凍結 task list は mid-phase の owner 決定で stale 化しうる**
  （Phase 6 は T7b 挿入と Observation 5 status 変化の 2 例）。mutable phase
  plan を stale wording の work around でなく現 SSOT に対して revise する。
- 手順は [retrospectives.md](../../../procedures/retrospectives.md)。
- **採否:** **採用（プロセス前提）**。実装計画 §4 / クロージング §6 の
  進め方に反映する。

---

## prior-phase residual sweep（Phase 2–5）

§2.1 の原典は前 phase（Phase 6）handoff だが、それ以前の phase の
out-of-phase residual も Phase 7 に新規制約を持ち込まないことを監査する
ため、Phase 2–5 の disposition を closed 状態も含めて確認する。

| 出所 | residual | disposition |
|---|---|---|
| Phase 2 | layout-time runtime error 専用 `WASAMO_ERR_*` ABI code（`wasamo_run_layout` 相当の entry point 導入時に解決）| **non-Phase7・open-M3**。iteration は新 layout-error entry point を導入しない。§5（range mutation の structural failure observability）と主題は隣接するが別レイヤ（layout 失敗 vs structural-mutation 失敗）。Phase 7 制約にしない |
| Phase 3 R1 | `.gitignore` `*.uic` pattern | **non-Phase7**。build-hygiene の cross-cutting 項目で iteration thesis と無関係。任意の hygiene pass owned |
| Phase 3 R2 | `sync_visuals` ↔ pure-layout boundary test gap | **closed — Phase 4 T4**。Phase 4 framing decision F が「close R2 inside Phase 4」と決定し、Windows integration fixture `scroll_path_fixture_r2_three_level_visual_nesting_root_relative_math`（4 階層 Visual nesting の root-relative offset 一致）で closure 済み（commit `689d381`、[phase-4 t4.md](../../phase-4/retrospectives/t4.md)）。脱落ではない |
| Phase 4 R1 | Gallery host Window title wiring | **closed**。Phase 6 DD-M3-P6-006 で静的 `title:` host-wiring として解決済み（[phase-6 handoff §Closed items](../../phase-6/implementation/handoff.md)）|
| Phase 5 | runtime per-monitor DPI awareness 欠如 | **M4-owned**。DD-V-022/023 + roadmap M4 AC（下記 pointer 節と同じ扱い）|

Phase 2–5 の residual は Phase 7 に **新規制約を持ち込まない**（closed /
M4-owned / non-Phase7）。なお Phase 7 の range mutation は child insert/
remove に伴う **新たな Visual sibling order / sync side effects** を持ちうる
が、それは（既に閉じた）Phase 3 R2 の再オープンではなく、§2（range
structural target）/ §4（placement 格納モデル）/ §5（structural failure
observability）と implementation-gates trap #2（side effects）/ #3
（parallel-data drift）で扱う **Phase 7 固有の新規検証判断**である。

---

## 前送り対象に含めないもの（pointer のみ）

- **DPI runtime 修正**（M4 owned）— [DD-V-022 / DD-V-023](../../../cross-milestone/decisions/dpi-awareness-m4-deferral.md)
  + roadmap M4 AC が owner。iteration / placement 設計と直交する
  runtime-quality 軸。evidence 解析時に「既知の M4 残課題」として注記する
  に留める（Phase 6 §5 と同じ扱い）。
- **dynamic Window title / host bindings**（M4 owned）—
  [phase-6 handoff §M4 targets](../../phase-6/implementation/handoff.md)。
  Phase 7 が host attribute に触れる公算は低いが、触れる場合は
  host-owned-attributes vs content-root separation を保ち、`title` /
  `backdrop` / `theme` を `root.props` / `root.bindings` に戻さない。
- **lightbox modal 入力 / caption row 高さ / image・thumbnail-click 挙動**
  （いずれも M4 input / DPI / metrics owned）—
  [phase-6 handoff §M4 targets](../../phase-6/implementation/handoff.md)。
  ただし **Phase 7 は thumbnail set を生成する**ため接点はある: Phase 7 が
  生成するのは構造（cardinality）であって、thumbnail の click-to-open /
  real image widget / modal focus は M4 のまま。Phase 7 の E2E は Box +
  Text placeholder + plain text Button で構造経路を証明する（Phase 6 の
  gallery と同じ placeholder 規律）。
- **Observation 5 — closed**（root-caused + remediated）。Phase 7 として
  carry すべき残課題はない。ただし Phase 7 の E2E proof が WrapPanel +
  ScrollView を再利用し **2 つ以上の Compositor test を持つ mock-free
  Windows integration binary** を新設する場合は、Phase 6 の keep-alive
  apartment helper / `run_on_owning_runtime_thread_or_skip` を使う。これは
  通常の test-harness 入力であり open defect ではない（詳細は
  [verification-environments.md §Observation 5](../../../../docs/notes/verification-environments.md)）。
- **doc-folded 済み semantics**（spec を直接読む）— ZStack realised
  semantics / structural conditional semantics（`if` は control flow であり
  widget でない、absent subtree は fresh-on-return）/ ScrollView 直下の
  conditional member reject / component host surface separation はいずれも
  [docs/dsl_spec.md](../../../../docs/dsl_spec.md) /
  [docs/architecture.md](../../../../docs/architecture.md) に fold 済み。
  Phase 7 が拡張する base であり、constraints として転記せず spec を直接
  読む。
