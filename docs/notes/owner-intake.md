---
title: Owner intake — 未分化の要望・使用フィードバックの受け皿
status: live
created: 2026-07-07
last-updated: 2026-07-07
related-roadmap:
  - process/_roadmap.md
related-notes:
  - docs/notes/m4-interaction-intake.md
---

# Owner intake — 未分化の要望・使用フィードバックの受け皿

オーナーが「やれればやりたい」と思ったこと、および wasamo を実際に
使ってみてのフィードバックを、**分解・分類する前の未分化なまま**
書き溜めるための常設 note。

書き込みコストを最小にすることが最優先。1 行から書いてよい。分解
（どの milestone / レーンに流すか、ABI に触れるか）は書く時点では
不要で、後からエージェントが行う。

## エントリの書き方

1 エントリ = 見出し 1 つ。最小構成:

- **日付** — 記入日
- **本文** — 1〜2 行。例・動機は任意で追記
- **triage** — `untriaged` で始め、振り分けたら遷移先を記す

## Triage の遷移先

エントリは点検（milestone 計画の §1.1、またはオーナーが指示した
任意の時点）で分解され、次のいずれかに流れる:

- **milestone intake** — 未着手 milestone の設計空間に属するもの。
  per-topic intake note（例:
  [m4-interaction-intake.md](./m4-interaction-intake.md)）または
  当該 milestone の framing 入力へ
- **per-topic note 切り出し** — 同一テーマのエントリが溜まったら
  専用 note に分離
- **reject** — 見送り。理由 1 行を残してエントリを閉じる
- **fix-now** — バグ・明白な不整合。通常の修正作業へ
- **Pre-1.0 candidate pool** — 分解・タグ付け済みで、どの milestone
  にも未割当の item。`_roadmap.md` の
  [Pre-1.0 candidate pool](../../process/_roadmap.md#pre-10-candidate-pool)
  節へ（ルールは
  [DD-V-028](../../process/cross-milestone/decisions/pre-1.0-candidate-pool.md)）
- **fast lane**（計画中） — 小粒 additive item の比例縮小レーン。
  VDR 未確定の構想。具体的な小粒 item が最初に出た時点で VDR を
  起こす予定

triage 済みエントリは遷移先を記して閉じる（削除でもよい。履歴は
git に残る）。

---

## エントリ

### 2026-07-07 — M3 実装済みウィジェットの機能拡張

M3 までに実装したウィジェットの機能を拡張したい。例: Box 以外の
要素で背景色（`fill`）を設定できること。

- **triage:** triaged (2026-07-07) → 3 item に分解し
  [Pre-1.0 candidate pool](../../process/_roadmap.md#pre-10-candidate-pool)
  へ（DD-V-028 の Accept と同時に有効）: (a) レイアウトコンテナへの
  リテラル `fill` 拡張（`ABI-bearing: no`・小粒）、(b) themed widget
  （Button 等）の背景色（M5 theming の設計空間・`unknown`）、
  (c) reactive な `fill`（binding type / TypedValue 問題・`unknown`）
