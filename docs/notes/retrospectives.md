---
title: レトロスペクティブ — マージ前の振り返り手順
status: live
created: 2026-05-05
---

# レトロスペクティブ

ブランチをマージする前に retrospective を実施する。**scope はマージ先で
決まる**:

- merge 先 = phase ブランチ → **step retrospective** → オーナー確認後 ff merge
- merge 先 = main → **phase retrospective** → オーナー確認後 no-ff merge → オーナー確認後 push
- 1 step = 1 phase (step ブランチが phase 全体をカバー) の場合は
  step → phase merge 後に **続けて phase retrospective も回す**。
  ff (step→phase) と no-ff (phase→main) はそれぞれ独立した gate で、
  **オーナー確認は最低 2 回必要** (retrospective を 1 回にまとめて main
  まで一気に進めない)。push は no-ff merge とも別 gate。

## 進行手順

checklist 完了 = merge 許可ではない。順序を固定する:

1. 下記 checklist を実施
2. 結果をオーナーに報告 (CHANGELOG / plan diff、CI / rebuild 結果、
   未決事項、retrospective 所見)
3. **ファストトラック判定** (下記参照):
   - **scope = step-end** かつ checklist 項目 2–8 がすべて「なし」/
     項目 3 が green の場合に限り、報告と同時に ff merge を実行し、
     事後にオーナーへ通知してよい。
   - それ以外は従来通りオーナーの承認を待つ (4 へ)。
4. オーナーが merge 種別 (ff / no-ff) を承認
5. 承認後に merge を実行
6. (phase-end のみ) オーナーが push タイミングを別途承認
7. 承認後に push を実行
8. push 後 main CI green 確認 (項目 15)

**phase-end (main への no-ff merge) は常にオーナー明示承認が必要**で、
ファストトラックの対象外 (AC 達成判定・thesis 影響評価を含み、機械的
判定に委ねない)。

## checklist

各項目「あり/なし」(または green/fail、必要/不要) で答える。
**「あり」「fail」「必要」が 1 つでもあればオーナー報告 + 承認必須**。
ファストトラック対象項目は項目末尾に **(FT)** を付す。

### 共通 (両 scope)

1. 本作業の主要な学び (記述項目、判定対象外)
2. 仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の
   変更 — あり/なし **(FT)**。タイポ修正、または既に Accepted な DD の
   機械的転記は「なし」扱い。
3. プロジェクトルートで `cargo fmt` を実行した上で、ローカル clean
   rebuild (`cargo clean` → release+debug build →
   `cargo test --workspace`) — green/fail **(FT)**
4. PO に相談すべき設計判断・トレードオフ — あり/なし **(FT)**

### step-end 固有 (merge → phase ブランチ)

5. plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更 — あり/なし **(FT)**
6. 現在の phase ADR への追加 DD 必要性 — あり/なし **(FT)**
7. 既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格 — あり/なし **(FT)**
8. `m2-plan.md` の AC (Ax) 追加・変更、または Phase 構成の追加・統合・
   分割 — あり/なし **(FT)**
9. 後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告 — あり/なし
   **(FT)**
10. タスクリストの後続 step 見直し (現在の phase ADR に影響しない
    範囲) — 必要/不要

CI green: 推奨 (PR を上げていれば PR CI、ローカルのみなら項目 3 の
clean rebuild が proxy)。CI YAML 変更は通常不要 (phase 内で発生したら
ADR に補足 DD)。

### phase-end 固有 (merge → main、ファストトラック対象外)

11. acceptance criteria (Ax) が本当に達成されているか — ADR の
    「discharged」表記と実装の乖離
12. `CHANGELOG.md` / `ROADMAP.md` の記述と実装の整合
13. `VISION.md` / thesis-level claim への影響 — 影響あれば本 phase 内
    で更新するか、別 ADR に切るかを決める
14. 次 phase の pre-doc への送り込み材料を `docs/notes/` に整理したか
    (出発点になる設計軸、未決の論点、引き継ぎ制約など)
15. CI green 確認 — phase ブランチ上で `workflow_dispatch` から CI を
    走らせ、**merge 前に GitHub Actions green を確認する** (proxy では
    なく実 CI を gate にする)。**push はオーナー明示承認後のみ**。
    push 後は main 上で CI green を再確認 (push トリガで自動実行)、
    失敗時の recovery (revert PR / force reset 等) はオーナー判断。
16. human-visible GUI smoke — 必要/不要。runtime / ABI / binding /
    wasamoc lowering / examples 等、ユーザー可視の挙動に影響しうる
    phase では必要。必要な場合は
    [human-visible GUI smoke](./human-visible-smoke.md) に従い、
    `counter-c`, `counter-rust`, `counter-zig` を確認する。
17. CI YAML 変更要否の sanity check — 本 phase で新言語/新ビルド系を
    追加していれば CI 更新済みであること (CLAUDE.md の CI rules)

CI green: **必須**。clean rebuild は **必須** (incremental cache の嘘を
main に持ち込まない)。

## ファストトラック基準の根拠

各 (FT) 項目を独立に守る理由:

- **項目 2 (仕様変更)**: ABI シグネチャの微調整等は C/Zig ホストへの
  波及効果があり、型システム整合性のゲートを残す必要がある
  (cf. DD-M2-P6-005)。
- **項目 7 (Proposed 増加・昇格)**: 「とりあえず Proposed でマージ」は
  M3 以降での設計負債を蓄積させうる (cf. DD-M2-P6-010 トポロジカル
  ソート近似)。昇格もオーナー裁定が必要。
- **項目 8 (m2-plan.md)**: AC の追加 (例: A5 再入安全性、A6 String
  通電) は完了定義を変える重い判断で、リソース配分の再確認を含む。
- **項目 5 / 6 / 9**: 計画外の構造変更・新規 DD・引き継ぎ技術的負債は
  いずれも「次の step / phase の前提」を変えるため、機械判定に委ねず
  オーナーの可視化を経る。
