---
title: M3-Phase 1 phase-end retrospective
status: recorded
created: 2026-05-19
scope: phase-end
phase: M3-Phase 1
---

# M3-Phase 1 phase-end retrospective

## Scope

M3-Phase 1 (`bool` scalar binding) の phase-end retrospective。対象は
`feat/m3-phase-1` から `main` への no-ff merge 前判定であり、
[docs/notes/retrospectives.md](../retrospectives.md) の phase-end
checklist に沿って、A9 の達成、上位文書との整合、CI、次 phase
(M3-Phase 6 / M3-Phase 8 を含む `bool` 依存 phase) への送り込み材料を
確認する。

対象 HEAD:

- branch: `wip/step` (→ `feat/m3-phase-1` への ff merge 候補)
- HEAD commit: `1129aea style: apply cargo fmt across T6-T8 …`
  以降に本 retrospective コミット
- CI: `workflow_dispatch` 後に追記 (本文 §15 参照)

## Current Judgment

2026-05-19 時点の判定は、**M3-Phase 1 の implementation/verification
criteria はローカルで達成済み。owner-review findings (1–4) はいずれも
本 retrospective commit (および同 commit の関連 doc-edit) で discharge
された。main merge gate としては GitHub Actions CI green 確認と owner
明示承認が残っている**、である。

owner-review findings の処理結果は [progress file §Owner-review follow-ups
(closed at T12 phase-end)](../../plans/progress/m3-phase-1-progress.md#owner-review-follow-ups-closed-at-t12-phase-end)
に記録した。要点:

- **Finding 1** (A9 evidence wording / ADR item 3 未達) → T13 を新設し
  binding-pipeline-inclusive な mock-free Windows-only integration test
  ([wasamo-runtime/tests/bool_binding_live_propagation.rs](../../../wasamo-runtime/tests/bool_binding_live_propagation.rs))
  で discharge。本 retrospective §Checklist 11 の A9 evidence 表現も
  T13 を main anchor、T6 widget-setter slice を補助 evidence として
  訂正済み。
- **Finding 2** (m3-plan §Phase-end criteria item 5 と ADR §Verification
  item 4 の乖離) → option (a) を採用。m3-plan §Phase-end criteria
  item 5 に foundational-phase exception 句を追加し、Phase 1 が
  `examples/bool-demo/` + `examples/bool-demo-rust/` で gallery
  sub-screen を代替したことを文書上整合させた。Phase 2 以降は item 5
  原則どおり `examples/gallery/` sub-screen を成長させる。
- **Finding 3** (checklist item 16 GUI smoke 欠落) → 本 §Checklist
  phase-end 固有 に item 16 (human-visible GUI smoke) を追加し
  「不要」判定を記録、旧 item 16 (CI YAML sanity check) を item 17 に
  renumber。
- **Finding 4** (progress file lifecycle) → progress file frontmatter
  `status: active` → `closing` に変更、本 retrospective の checkbox を
  tick 済み (本 commit で進捗反映)。

具体的には、

- A9 (`bool` admitted as the third scalar binding type alongside `i32`
  and `String`) は実装・テストの両面で discharged。
  - ADR §Verification item 3 (unified `.ui → load → click → state →
    bound widget property` chain) は T13 の mock-free Windows-only
    integration test
    ([wasamo-runtime/tests/bool_binding_live_propagation.rs](../../../wasamo-runtime/tests/bool_binding_live_propagation.rs))
    で証拠化。
  - T6 の widget-setter slice
    ([wasamo-runtime/tests/button_enabled.rs](../../../wasamo-runtime/tests/button_enabled.rs))
    は file-level comment 通り binding pipeline を bypass する slice
    test で、DD-M3-P1-005 (`PROP_BUTTON_ENABLED` ABI dispatch + visual
    flip + click suppression) の補助 evidence。
  - visible-window proof は `.ui`-driven
    [examples/bool-demo-rust/](../../../examples/bool-demo-rust/)
    の owner-manual smoke でカバー。
- `TypedValue` (F5) は本 phase でも引き続き deferred。DD-M3-P1-007 は
  「per-type evaluator/writer pair を ir_loader の call site で選ぶ」
  というかたちで seam を確立し、reactive engine 自体は依然
  type-agnostic に保たれている。
- local clean rebuild (`cargo clean` → release+debug workspace build
  → `cargo test --workspace`) は green。
- A11 (per-phase spec sync) は T10 で `docs/dsl_spec.md` 0.4 → 0.5、
  `docs/architecture.md` §6.8.7 更新済み。
- 仕様文書の external-implementor reproducibility は owner 確認済み
  (T10 の smoke check、commit `55b904b`)。
- merge / push は phase-end ファストトラック対象外。owner 明示承認後
  に no-ff merge、push はさらに別 gate。

## Main Learning

本 phase の主要な学びは三点ある。

### 1. 既存の type-suffix pattern は第三 scalar まで自然に伸びる

M2 で `i32` / `String` を支えていた **type-suffixed variant** 設計
(`HandlerExpr::{IntLit, StrLit, PropRead, StrPropRead}` /
`EvalContext::{get_i32, get_string, read_i32_tracked, …}`) は、
`bool` を第三 scalar として追加する局面でも、特別な抽象化を導入せず
に `BoolLit` / `BoolPropRead` / `get_bool` / `read_bool_tracked` /
`set_bool` を並べるだけで通った。F5 (`TypedValue`) の早期導入を
回避できたのは、Phase 7 (反復文法) で TypedValue 圧力を再評価する
という mid-M3 の判断軸を活かす形になっている。

DD-M3-P1-007 で確立した **per-type binding writer seam** は、
`ir_loader::build_node` の `match prop_ty` で
`evaluate_bool_binding` + `widget_write_property_bool` /
`register_bool_binding` の組を選ぶことで、reactive engine の
write-time に `IrType` dispatch を持ち込まずに済んだ。これは F5
deferral の構造的裏付けでもある。

### 2. 値型導入 = exhaustive match 強制 → 隣接ステップを巻き込む

`PropertyValue::Bool` を `wasamo-runtime/src/widget.rs` に足した
瞬間、`abi::{read_property_value, write_property_value,
property_value_to_owned}` と `emit::owned_to_value` (および
`OwnedArg::Bool`) の exhaustive `match` が mechanically 拡張を
要求した。T9 (C ABI value-conversion arms) を独立 commit にせず、
T6 part 1 commit `a550bd9` に fold した判断は CLAUDE.md §Commit rules
の「implementation reveals a tighter ordering」に該当する。

学び: **新しい `IrType` / `PropertyValue` variant を入れる step は、
ABI side の value-conversion arm を独立 step として温存しても
意味がない**。ABI 側に新規 public 関数を増やす step
(DD-M3-P1-008 Option B 系) のときだけ独立 step として温存する。

### 3. process gap: 各 step retrospective の "cargo fmt — green" は不十分

T12 の phase-end gate で `cargo fmt --all -- --check` を初めて
workspace 全体に走らせたところ、T6 part 1–3、T7 part 2、T8 part 2 に
跨る fmt drift (6 ファイル) を検出した。各 step retrospective は
`cargo fmt — green` を記録していたが、実際には commit 直前の
fmt 再実行が抜けていた可能性が高い。CI が `cargo fmt --check` を
強制していないため、その drift は phase-end まで未検出だった。

応急処置は本 phase の commit `1129aea style: apply cargo fmt across
T6-T8 …` (fmt-only、no semantic change) で済ませた。恒久対策の
候補は二つ:

- **(a)** step retrospective の checklist 項目 3 (clean rebuild) を、
  `cargo fmt --all -- --check` 単独でも検証する形に明示化する
  (現状は「実行 + green」だが、commit 後の状態に対する `--check` 形で
  記録するルールが暗黙的)。
- **(b)** CI に `cargo fmt --all -- --check` を追加する (CLAUDE.md
  §CI rules の「新言語/新ビルド系を追加しない限り CI 更新不要」原則
  の例外に当たるので、ADR/owner agreement が必要)。

本 retrospective は (a) を `docs/notes/retrospectives.md` の
step-end checklist 改訂提案として M3-Phase 2 開始時に owner と
協議する材料に残す。(b) は CI 変更を伴うので M3-Phase 2 以降の
pre-doc で別途扱う。**Phase 1 の close 自体は (a)/(b) のどちらかが
入る前であっても問題ない**: A9 達成と spec sync が gate であり、
fmt drift は本 phase の commit `1129aea` で解消済み。

## Step Artifacts Reviewed

本 phase-end 判定では、`docs/notes/m3-phase-1/` 以下の各 step
retrospective を phase-level evidence として読み直した。

- `t1-step-end-retrospective.md`: `IrType::Bool` / `IrLiteral::Bool` /
  `HandlerExpr::{BoolLit, BoolPropRead}` の wasamo-ir 拡張が、
  既存 exhaustive `match` をコンパイラ強制で網羅する形で
  入ったことを確認。
- `t2-step-end-retrospective.md`: `true` / `false` keyword 予約と
  lexer/parser 拡張、AST `BoolLit` 追加、`wasamoc::emit` の bool
  literal 出力が end-to-end でカバーされたことを確認。
- `t3-step-end-retrospective.md`: parse-time `Namespace` を再利用し、
  soft な widget-property catalog (`Text.text` / `Button.text` /
  `Button.enabled`) に対して `bind` LHS を型検査する設計を確認。
  DD-M3-P1-010 の accept/reject 表が unit test の rows と
  対応している。
- `t4-step-end-retrospective.md`: state-type table を見て identifier
  を `BoolPropRead` / `PropRead` / `StrPropRead` に分岐させる
  lowering を確認。非 state ident は静的 `IrLiteral::Ident` のままで、
  M2-era `.ui` corpus を壊していない。
- `t5-step-end-retrospective.md`: IR text loader が bool productions
  を受理することと、`wasamoc` emit → `wasamo-runtime` parse の
  round-trip test が cross-crate seam を守ることを確認。
- `t6-step-end-retrospective.md`: `PropertyValue::Bool` /
  `resolve_prop_key` の `IrType` 拡張 / `Button.enabled` runtime
  contract / Windows-only live integration test の四点が同じ step
  に集約された経緯と、T9 fold (commit `a550bd9`) の合理性を確認。
- `t7-step-end-retrospective.md`: `EvalContext` の bool 既定実装が
  M2 String shape (`get_bool` 既定 = `UnknownProperty`、
  `read_bool_tracked` 既定 = `get_bool` forward) に倣ったため、既存
  `EvalContext` 実装が壊れずに済んだ判断を確認。`Assign` arm が
  `Result<i32, _>` 返り値契約のもとで `Ok(0)` を返す妥当性も確認。
- `t8-step-end-retrospective.md`: per-type binding writer seam の
  位置 (engine 内部ではなく ir_loader call site) と、I32 / Str が
  M2 stringified path を継続使用する判断 (現行 widget catalog rows に
  i32-bound セマンティクスを持つ property が無いため) を確認。
- `t9-step-end-retrospective.md`: T9 が独立 commit を持たないこと、
  ABI value-conversion arms が T6 part 1 に fold されたことを確認。
  ABI 側 spec (`abi_spec.md`) は wire-format に変更がないため未更新で
  正しい。
- `t10-step-end-retrospective.md`: `dsl_spec.md` 0.4 → 0.5 と
  `architecture.md` §6.8.7 の拡張、および retroactive な `state`
  surface entry 追加 (M2-Phase 6 の文書漏れを最小範囲で fold) の
  owner-agreed scope expansion を確認。
- `t11-step-end-retrospective.md`: `examples/bool-demo-rust/` を
  既存 `counter-rust` の拡張ではなく sibling example として置いた
  判断 (M2 Hello Counter の証拠を汚さない) と、GUI smoke は owner
  manual 領域であることを Codex 側で明示記録した process correction
  を確認。

これらの step メモから見て、Phase 1 は「各 task の実装が入った」
だけでなく、設計判断 (per-type seam の位置 / T9 fold / state surface
retroactive fill) と process correction (GUI smoke は owner 領域)
が phase-level 判定に蒸留されている。

## Checklist

### 共通

1. **本作業の主要な学び:** あり。
   - 第三 scalar 導入は既存 type-suffix pattern の枠で閉じた。
   - `PropertyValue` variant 追加 = ABI value-conversion arm 強制で、
     T9 fold は構造的に必然だった。
   - `cargo fmt` の process gap (各 step retrospective の "green"
     が commit 後状態を保証していなかった) が phase-end gate で
     初めて検出された。詳細は §Main Learning。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`)
   の変更:** あり。
   - `dsl_spec.md` 0.4 → 0.5 (T10 part 1, commit `ed93d5e`)。
   - `architecture.md` §6.8.7 (T10 part 2, commit `b7f91ce`)。
   - これらは A11 (per-phase spec sync) の正規な phase 内更新で、
     未承認の仕様逸脱ではない。
   - `abi_spec.md` は意図的に未更新 (Phase 1 はワイヤフォーマット変更
     なし; `WASAMO_VALUE_BOOL = 3` と `v_bool` は M2 reserved)。

3. **ローカル clean rebuild:** green。
   - `cargo clean` — green (`Removed 3834 files, 973.9MiB total`)。
   - `cargo fmt --all -- --check` — green (commit `1129aea` 後)。
   - `cargo build --release --workspace` — green (43.73s)。
   - `cargo build --workspace` — green (38.14s)。
   - `cargo test --workspace` — green (165 + 98 + 7 + 6 unit/roundtrip
     + 8 integration tests; 0 failed, 0 ignored)。
   - 既知 warning (M2 由来): `wasamo` crate "provides no linkable
     target"、`wasamo-sys` import-library ordering note。いずれも
     build/test failure ではない。

4. **PO に相談すべき設計判断・トレードオフ:** あり。
   - phase-end は main への no-ff merge gate であり、owner 明示承認が
     必要。技術的には A9 達成と local CI gate を満たしているが、
     GitHub Actions CI green の確認と push timing は owner 判断。
   - §Main Learning #3 の `cargo fmt` process gap 対策 (step
     checklist 改訂 vs CI 追加) は本 phase の close 後に M3-Phase 2
     開始時に協議する候補。本 phase 内では応急処置のみ。

### phase-end 固有

11. **acceptance criteria (Ax) が本当に達成されているか:** 達成。
   - A9: `bool` は `wasamo-ir` の `IrType` / `IrLiteral` /
     `HandlerExpr` 三層、`wasamoc` の lex / parse / check / lower /
     emit、`wasamo-runtime` の IR loader / `PropertyValue` /
     `resolve_prop_key` / `EvalContext` / binding writer seam の
     すべてを通る。ADR §Verification item 3 (unified `.ui → load →
     click → state → bound widget property` chain) は T13 の mock-free
     Windows-only integration test
     `bool_binding_propagates_state_write_through_inline_handler_to_widget_property`
     ([wasamo-runtime/tests/bool_binding_live_propagation.rs](../../../wasamo-runtime/tests/bool_binding_live_propagation.rs))
     で discharge。T6 の `button_enabled_property_flips_visual_and_
     suppresses_click` ([wasamo-runtime/tests/button_enabled.rs](../../../wasamo-runtime/tests/button_enabled.rs))
     は file-level comment 通り binding pipeline を bypass する
     widget-setter slice であり、DD-M3-P1-005 の補助 evidence。
     visible-window proof は `.ui`-driven
     [examples/bool-demo-rust/](../../../examples/bool-demo-rust/)
     の owner-manual smoke。
   - A11 (per-phase spec sync): T10 で `dsl_spec.md` / `architecture.md`
     が同 phase 内に更新済み。

12. **`CHANGELOG.md` / `ROADMAP.md` の記述と実装の整合:** あり。
   - `CHANGELOG.md` に `[Unreleased] — M3: DSL surface (in progress)`
     section と `M3-Phase 1 — bool scalar binding (2026-05-19)`
     entry を本 retrospective commit に併せて追加する。
   - `ROADMAP.md` M3 AC は SSOT として変更なし。M3 自体は in-progress
     のままで、本 phase で discharged されたのは A9 のみ。M3 milestone
     marker (shipped 表示) は Phase 8 close 時に切り替わる。

13. **`VISION.md` / thesis-level claim への影響:** 影響なし。
   - 第三 scalar の追加は M3 Plan §Phase breakdown が前提とした増分で
     あり、`VISION.md` の thesis (External DSL × C ABI × Visual Layer)
     を変更しない。F5 deferral も M2 / M3 plan の既定路線。

14. **次 phase の pre-doc への送り込み材料を `docs/notes/` に
    整理したか:** 達成。
   - 既存 M3 横断材料: `docs/notes/m3/m3-start-framing.md` /
     `m3-target-app-predoc.md` / `m2-to-m3-handover.md` /
     `typed-value-evaluator.md` / `reactive-drain-cascade-policy.md` /
     `verification-environments.md` が引き続き M3 後続 phase の
     pre-doc に対する出発点として有効。
   - 本 phase の蒸留先: 本 phase-end retrospective §Main Learning
     #1–#3 が phase-level の主な学びの一次記録であり、step-end
     retrospective `docs/notes/m3-phase-1/t1-…/t11-*.md` 11 本は
     execution-level の細部を引き続き保持する。
   - **次 phase pre-doc input の書き起こし (本 phase close 内で完了):**
     [docs/notes/m3-phase-2/predoc-inputs.md](../m3-phase-2/predoc-inputs.md)
     に M3-Phase 2 (Box layout primitive) 視点で書き起こし済み。
     §1–§7 が以下を網羅:
     - §1: 新規 `PropertyValue` variant 追加 = ABI value-conversion
       arm を同一 step に fold する規律 (Main Learning #2)。
     - §2: 新規 bindable property の per-type writer seam を
       ir_loader call site で選ぶ規律 (Main Learning #1 /
       DD-M3-P1-007)。
     - §3: `cargo fmt` process gap への対策候補
       (step checklist 改訂 vs CI 強制) を Phase 2 開始時 DD として
       提示 (Main Learning #3)。
     - §4: 可視 proof は既存 canonical example を太らせず sibling
       example を立てる規律 (T11)。
     - §5: GUI smoke は owner manual、Codex は launch command 成功
       までを記録する process correction (T11 §Follow-Up)。
     - §6: spec sync 中の retroactive earlier-phase docs gap fold
       規律 (T10、memory `feedback_retroactive_spec_gap_fold` と同期)。
     - §7: Box の `aspect` 属性で float type を IR に入れるかの
       再評価論点 (Phase 1 defensive fallback)。
   - これは
     [docs/notes/retrospectives.md §Retrospective Main Learning の前送り](../retrospectives.md#retrospective-main-learning-の前送り)
     の「phase close 内で書き起こす」要件への適合。次 phase pre-doc
     起草時は M3-Phase 2 owner-agreed framing に合わせて §1–§7 を
     取り込む / 並べ替える / 削減することは Phase 2 内の判断だが、
     **本ノートの存在自体は本 phase の close 内で確定**。

15. **CI green 確認:** 確認済み。
   - GitHub Actions `workflow_dispatch` run
     [26094510225](https://github.com/matarillo/wasamo/actions/runs/26094510225)
     on `feat/m3-phase-1` — green (cargo build job: success)。
     `cargo test --workspace` 内で T6 `button_enabled` と T13
     `bool_binding_live_propagation` の mock-free Windows integration
     test が skip せず pass していることをこの run が証明する。
   - 本 phase は `cargo fmt --check` を CI が enforce していないが、
     §Main Learning #3 の通り、応急処置の fmt-only commit `1129aea`
     を入れた上で release/debug build と test が CI 上で green に
     なった。
   - 本 retrospective および関連 phase-close commit (`f6b6d74`) が
     phase ブランチ HEAD に含まれた状態での CI green であり、
     owner の main no-ff merge 承認に進める。

16. **human-visible GUI smoke:** 不要。
   - retrospectives.md §checklist item 16 は「runtime / ABI / binding /
     wasamoc lowering / examples 等、ユーザー可視の挙動に影響しうる
     phase では必要」と定めている。Phase 1 はそのすべてに該当する
     ので、本来は `counter-c` / `counter-rust` / `counter-zig` を
     [human-visible GUI smoke](../human-visible-smoke.md) に従って
     確認すべき位置にある。
   - **不要判定の根拠:** Phase 1 の人間可視領域は次の三層で既に
     カバー済み。
     1. T6 の mock-free Windows-only integration test
        `button_enabled_property_flips_visual_and_suppresses_click`
        が `CompositionColorBrush::Color()` flip と click 抑制を
        headless で assert。
     2. T13 の mock-free Windows-only integration test
        `bool_binding_propagates_state_write_through_inline_handler_to_widget_property`
        が `.ui → IR → click → state → bound widget property` chain を
        headless で assert (新規 binding-pipeline-inclusive evidence)。
     3. T11 で `examples/bool-demo-rust` を owner-manual smoke
        (`Start-Process .\target\release\bool-demo-rust.exe`、visible
        window 確認) で実機検証済み。
   - `counter-c` / `counter-rust` / `counter-zig` 自体は Phase 1 で
     surface 拡張を受けておらず (Hello Counter は `i32` / `String` の
     既存路線のみで動作)、新 surface の人間可視 smoke として走らせる
     意義は薄い。`examples/bool-demo-rust` が Phase 1 の新 surface を
     人間可視で検証する正規の host である点は §14 / Finding 2 の
     foundational-phase exception と整合する。

17. **CI YAML 変更要否の sanity check:** 不要。
   - Phase 1 は新言語・新ビルド系を追加していない (Rust crate
     `examples/bool-demo-rust` は既存 Rust workspace の追加 member
     であり、CLAUDE.md §CI rules の「Rust コードを既存 crate に
     追加する phase は CI 更新不要」に該当)。
   - §Main Learning #3 の CI に `cargo fmt --all -- --check` を
     追加するかどうかは本 phase の close 後に協議。本 phase 内では
     CI YAML を触らない。

## Phase-End Gate

**Merge readiness:** ready for owner no-ff merge approval **after**
GitHub Actions CI green confirmation on `feat/m3-phase-1`. push remains
a separate owner-approved gate per
[docs/notes/retrospectives.md](../retrospectives.md) と
`feedback_phase_end_merge` memory。

phase-end retrospective としては、以下を owner に報告する。

- A9 は達成済み (実装 + spec sync + visible proof + Windows-only live
  test)。
- local clean rebuild は green。fmt drift は本 phase 内で fmt-only
  commit `1129aea` により解消済み。
- A11 spec sync (T10) は external-implementor smoke check を owner
  確認済み。
- 新 CI YAML 作業は不要。
- main merge 前に GitHub Actions CI green の確認が必要。
- no-ff merge と push は別 gate であり、どちらも owner の明示承認が
  必要。

## Out-of-Phase Residuals

Phase 1 内で新たに発生した out-of-phase residual はなし。M2-to-M3
の引き継ぎ residual (cycle detection / dependency-tie observable
contract / `MUTATION_CAP` × fan-out interaction) は
[m2-to-m3-handover.md §3](../m2-to-m3-handover.md) に既記載のままで、
本 phase は触れていない。

§Main Learning #3 の `cargo fmt` process gap は M3-Phase 1 内で応急
処置済み (fmt-only commit `1129aea`)。恒久対策は M3-Phase 2 pre-doc
で扱う候補として記録するに留め、本 phase の residual には数えない
(対策が次 phase の preparation 範囲のため)。

## Commands Run

```text
git status --short --branch
git log --oneline -20
git log main..HEAD --oneline
cargo fmt --all -- --check
cargo fmt --all
cargo clean
cargo build --release --workspace
cargo build --workspace
cargo test --workspace
```

All local build/test commands completed successfully. The GitHub
Actions run for the phase branch will be triggered via
`workflow_dispatch` after this retrospective lands and the wip/step
branch is ff-merged into `feat/m3-phase-1`; its link will be appended
to [`docs/plans/progress/m3-phase-1-progress.md` §CI / verification
log](../../plans/progress/m3-phase-1-progress.md#ci--verification-log)
before owner no-ff merge approval.
