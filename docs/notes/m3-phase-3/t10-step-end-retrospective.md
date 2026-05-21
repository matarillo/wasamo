---
title: M3-Phase 3 / T10 step-end retrospective
status: recorded
created: 2026-05-22
scope: step-end
task: T10 — Phase-end gates (Phase 3 close on the step branch; ff merge into phase branch follows)
---

# M3-Phase 3 / T10 step-end retrospective

## 対象範囲

T10 は M3-Phase 3 progress doc §T10 — Phase-end gates の checklist
を実行する step。step branch `feat/m3-phase-3-t10` 上で:

- ローカル `cargo fmt --all -- --check` + clean rebuild
  (release + debug build + `cargo test --workspace`) の green 確認
- `dsl_spec.md` の Moment 2 spec re-sync (§4.10 status flip、§2.2
  lexer-surface fold、version 0.9 → 1.0)
- `architecture.md` の Moment 2 architecture re-sync (top-level
  Status flip、§6.5 への T9 由来 offset 規約 1 行追加 = R3-A fold)
- ADR `m3-phase-3-wrap-panel.md` への "Phase 3 implementation
  residuals" subsection 追加 (R1 / R2 へのクロスリファレンス)
- progress doc `§Out-of-phase residuals` に R1 / R2 を記録、T10
  checklist 全 [x] flip、front-matter `status: active` → `closing`
- 次 phase 前送り note `docs/notes/m3-phase-4/pre-doc-inputs.md`
  (13 sections) 作成
- 本 phase の durable phase-end retrospective
  `docs/notes/m3-phase-3/phase-end-retrospective.md` 作成
- 本 step-end retrospective 作成

CI 実行は本 step branch ではなく phase branch (`feat/m3-phase-3`)
上で `workflow_dispatch` から行う方針 (本 retrospective 内で別途
記録)。

## 主な学び

中心的な学びは **「phase-end の docs-only step も、design decision
が 2 件混入する」**こと。T10 は実装変更を含まないが、Decisions log
が flag していた "lexer change を spec §2 に明記するか" と T9 由来の
"architecture §6.5 へ 1 行追記するか / 残件に回すか" の 2 件は
owner 確認が必要な選択肢付き判断であり、いずれも docs-only commit
として inline で options を提示する形を取った。

`feedback_design_choices.md` の rule は「options with pros/cons +
recommendation を提示し、document する」だが、T10 セッションの
直接学びは:

- owner は `AskUserQuestion` ではなく **inline-in-chat で options
  を並べる形** を明示的に preferred (T10 セッション中に一度
  `AskUserQuestion` を試行 → owner が停止指示)。
- 提示は selection だけでなく rationale を伴った recommendation
  まで書き、選んだ option を commit message + retrospective に明示
  する。

これは Phase 4 以降の design-choice surface (ScrollView の各
attribute、bindable surface 取り扱い等) でも継続して適用する。

二つ目の学びは **「retroactive spec-gap fold は doc category の
責務分離 (lexical / architecture / AST / IR) を維持したまま行う」**
こと。T1 由来 lexer-surface fold は §2.2 (lexical) に閉じ、§5
(AST i64) は触らず、§8.2 (IR INT) は既に signed surface だったので
変更不要とした。T9 由来 offset 規約 fold は §6.5 (Visual-Layer
sync) に 1 行限定。両 fold とも「現 phase 内で発覚した earlier-phase
の docs 漏れを最小範囲で同じ phase の sync commit に折り込む」
(`feedback_retroactive_spec_gap_fold.md`) の rule に従いつつ、
責務分離を守ったため fold の influence は局所に閉じた。

## チェックリスト

1. **本作業の主要な学び:** あり (上記 2 点)。phase-end docs-only
   step での design-choice 取扱いは Phase 4 へ前送り (pre-doc
   inputs §12 process continuity)。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`)
   の変更:** **あり** **(FT non-trivial)**
   - `dsl_spec.md` v0.9 → v1.0 (§4.10 status flip + §2.2 lexer
     surface fold + revision-history row)。
   - `architecture.md` top-level Status flip + §6.5 offset 規約
     1 行追加。
   - `abi_spec.md` 変更なし。
   - これは Phase 3 close 当然の作業範囲だが、fast-track 判定上は
     「あり」として明示する。

3. **ローカル clean rebuild:** **green** **(FT)**
   - `cargo fmt --all -- --check`: green (zero exit, post-commit
     state)。
   - `cargo clean`: success (2482 files / 884.1 MiB removed)。
   - `cargo build --release --workspace`: green (40.98s)。
   - `cargo build --workspace`: green (34.20s)。
   - `cargo test --workspace`: green (workspace 全 test 通過)。
   - 既知 warning は Phase 2 close 時点と同じ
     (`wasamo` non-linkable target、`wasamo-sys` import-library
     ordering)。

4. **PO に相談すべき設計判断・トレードオフ:** **あり** **(FT)**
   - **判断 1:** dsl_spec §2 / §5 に lexer change を明記するか
     (Decisions log forwarded)。owner 選択 = "A' = §2.2 に kebab
     Ident + signed IntLit を両方明記、§5 触らず"。
   - **判断 2:** T9 由来 architecture §6.5 への offset 規約 1 行
     追記を residual に回すか fold するか。owner 選択 = "R3-A =
     T10 architecture commit で fold (別 commit で review 単位
     分離)"。
   - 両判断とも inline-in-chat で options を提示し、owner が選択。
     commit message と本 retrospective に明示。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし** **(FT)**
   - T10 の commit set は全て progress doc §T10 checklist 範囲内。

6. **現在の phase ADR への追加 DD 必要性:** **なし** **(FT)**
   - 上記 2 件の owner 判断は DD level ではなく Decisions log
     forwarded 項目の closing。新規 DD 追加なし。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed →
   Accepted への昇格:** **なし** **(FT)**
   - 当該 ADR は T6 closing 時点で全 DD Accepted 済み。T10 で
     追加した "Phase 3 implementation residuals" subsection は
     handover 記録 (DD ではない)。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし** **(FT)**
   - m3-plan §Phase-end criteria item 6/7 (residuals / retro)
     の文言は触らず、本 T10 は規定どおりに実行のみ。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **なし** **(FT)**
   - T10 は docs / spec sync のみ。code 変更なし。

10. **タスクリストの後続 step 見直し:** **不要**
    - T10 は本 phase 最後の step。後続 step なし (phase-end
      main-merge gate は別ブランチ / 別 session)。

## ファストトラック判定

step-end fast-track の対象判定:

- 項目 2 = **あり** (spec / architecture re-sync は当然作業だが
  fast-track 判定上は明示「あり」)
- 項目 3 = green
- 項目 4 = **あり** (owner 確認 2 件)
- 項目 5 / 6 / 7 / 8 / 9 = なし

→ 項目 2 と項目 4 が「あり」のため **fast-track 不成立**。
T10 → `feat/m3-phase-3` への ff merge は owner 明示承認を経る。
本 retrospective は owner 報告材料として作成。

## 検証メモ

T10 commit 系列 (時系列):

1. `e423ece` — `docs(dsl_spec): M3-Phase 3 close — flip §4.10 +
   fold T1 lexer surface (v0.9 → v1.0)`
2. `e88bdd8` — `docs(architecture): flip Status to M3-Phase 3
   complete (Moment 2)`
3. `6b0b7f6` — `docs(architecture): fold T9 layout-vs-visual
   offset convention clarification`
4. `826d5b4` — `docs(m3-phase-3): file R1/R2 out-of-phase
   residuals + ADR cross-ref`
5. `92e1ded` — `docs(m3-phase-4): forward-distillation
   pre-doc-inputs from M3-Phase 3 close`
6. `08ec155` — `docs(m3-phase-3): T10 phase-close — phase-end
   retrospective + progress flip to closing`
7. (本 commit) — `docs(m3-phase-3): T10 step-end retrospective`

review-concern 単位の分割原則 (`CLAUDE.md §Commit rules`):

- spec re-sync (1) と architecture re-sync (2, 3) は別 doc / 別
  review cycle。
- architecture の Moment 2 flip (2) と T9 由来 R3-A fold (3) は
  diff の意図 / origin が異なるため別 commit。
- residual filing (4) は progress + ADR で 1 件の review concern
  (cross-ref が常に整合)。
- forward-distillation (5) と phase-end retrospective (6) は
  別 doc / 別 review concern。

## フォローアップ

T10 → `feat/m3-phase-3` ff merge は owner 明示承認後に実行。
merge 後に `feat/m3-phase-3` を push し、`workflow_dispatch` から
CI を回す。CI green 確認後:

- `docs/notes/m3-phase-3/phase-end-retrospective.md` item 15 に
  CI run URL を fold (small docs-only follow-up commit)。
- progress doc T10 checklist 2 つ目 / 3 つ目の bullet 末尾に
  CI URL を fold。

phase-end main-merge gate (`feat/m3-phase-3` → `main` no-ff) は
別 session で owner 明示承認後に実行。push もさらに別 gate
(`retrospectives.md` 項目 6 / 7)。

T10 から発生した out-of-phase residual: **なし**。R3 は本 step で
fold 完了 (R3-A) し residual ではない。R1 / R2 は T9 由来の継続
記録 (T10 で新規発見ではない)。

T10 から発生したプロセス側の継続事項:

- AskUserQuestion を design-choice presentation に使わない方針が
  T10 で確認された。次 phase 以降も inline options + recommendation
  + commit/retro での選択明示で進める。

## 引き継ぎ

次 session の入口:

- branch: `feat/m3-phase-3-t10` から ff merge 承認 → 
  `feat/m3-phase-3` 上で CI green 確認 → CI URL fold
  follow-up commit + push → owner へ phase-end main-merge gate
  報告
- phase-end main-merge: owner 明示承認 + no-ff merge + 別 gate
  での push + main CI green 再確認
- 次 phase 着手: `feat/m3-phase-4` 系列ブランチで
  `docs/notes/m3-phase-4/pre-doc-inputs.md` を入力に Phase 4 ADR
  pre-doc 起稿から開始
