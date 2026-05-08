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
3. オーナーが merge 種別 (ff / no-ff) を承認
4. 承認後に merge を実行
5. (phase-end のみ) オーナーが push タイミングを別途承認
6. 承認後に push を実行
7. push 後 CI を確認 (項目 12)

## 共通項目 (両 scope で確認)

1. 本作業の主要な学び (計画時に想定しなかったこと)
2. 仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) への
   想定以上の変更が発生していないか
3. ローカル clean rebuild (`cargo clean` → release+debug build →
   `cargo test --workspace`) が green
4. その他、プロダクトオーナーに相談すべき事項

## step-end 固有 (merge → phase ブランチ)

5. タスクリストの後続 step を見直す必要があるか (現在の phase ADR には
   影響しない範囲)
6. 現在の phase ADR に追加の DD が必要か
7. 後続 phase に引き継ぐ制約が増えたか

CI green: 推奨 (PR を上げていれば PR CI、ローカルのみなら clean rebuild
が proxy)。CI YAML 変更は通常不要 (phase 内で発生したら ADR に補足 DD)。

## phase-end 固有 (merge → main)

8. acceptance criteria (Ax) が本当に達成されているか — ADR レベルの
   「discharged」表記と実装が乖離していないか
9. `CHANGELOG.md` / `ROADMAP.md` の記述が実装と整合しているか
10. `VISION.md` / thesis-level claim に影響を与えたか — 影響あれば本
    phase 内で更新するか、別 ADR に切るかを決める
11. 次 phase の pre-doc への送り込み材料を `docs/notes/` に整理したか
    (出発点になる設計軸、未決の論点、引き継ぎ制約など)
12. CI green 確認 — push 前は local clean rebuild green (項目 3) が
    CI の proxy。**push はオーナー明示承認後のみ**。push 後 GitHub
    Actions で main CI green を確認、失敗時の recovery (revert PR /
    force reset 等) はオーナー判断。
13. CI YAML 変更要否の sanity check — 本 phase で新言語/新ビルド系を
    追加していれば CI 更新済みであること (CLAUDE.md の CI rules)

CI green: **必須**。clean rebuild は **必須** (incremental cache の嘘を
main に持ち込まない)。
