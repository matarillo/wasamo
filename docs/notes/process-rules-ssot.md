---
title: プロセス記述の SSOT 化 — オープンクエスチョン
status: open
created: 2026-05-20
related-adrs:
  - docs/decisions/vision-doc-system.md
related-notes:
  - docs/notes/doc-system-redesign.md
---

# プロセス記述の SSOT 化

## 背景

M3-Phase 2 の Moment 1 を 1 commit にまとめた失敗の振り返りで、新ルール
「doc commit は review concern 単位」を
[CLAUDE.md §Commit rules](../../CLAUDE.md#commit-rules) に追記した
(commit `b11688b`)。

このとき問われたメタ論点：**プロジェクトの "process" は現状どこに住んで
いて、どこを SSOT にすべきか？** 既存の
[doc-system-redesign.md](./doc-system-redesign.md) は VISION / ROADMAP /
plans / decisions / notes の上位構造（"何の文書がどこに住むか"）を
整理したが、**運用ルールそのもの**（commit ルール、テストルール、retro
手順、lifecycle 規約 等）の住所と SSOT 化は別問題として残っている。

本ノートはこの問いを記録する。決定はまだしない。

## 現状のプロセス記述の散在マッピング

| 場所 | 主に扱うプロセス | 性格 |
|---|---|---|
| `CLAUDE.md` | Language / Document categories / Testing / Commit / CI / Build ordering | セッション開始時に毎回読まれる "活きたルール" |
| `docs/notes/retrospectives.md` | step-end / phase-end 振り返り手順と checklist | 手順書。retro 実施時に参照 |
| `docs/plans/README.md` | plan の lifecycle、Frozen agreement と Progress の構造、AC revision、progress file lifecycle、archival | plans/ ディレクトリ規約 |
| `docs/decisions/README.md` | ADR の規約（supersede 等） | decisions/ ディレクトリ規約 |
| `docs/notes/README.md` | notes/ の対象範囲、言語、lifecycle | notes/ ディレクトリ規約 |
| `docs/decisions/vision-doc-system.md` | doc system 自体の vision-level DDs | "Why" を記録する vision ADR |
| per-phase `docs/notes/<phase>/<phase>-pre-doc-framing.md` | 当該 phase の pre-doc workflow / framing decision | phase-local。次 phase が "前例として" 参照 |

ある程度の重複は既に存在する。例：

- "1 commit per task-list item" → `CLAUDE.md` §Commit rules
- "task list はあとから変えていい" → `CLAUDE.md` §Commit rules + `plans/README.md` §Phase progress file lifecycle
  ("task lists remain hypotheses..." の文脈)
- "pre-doc framing は draft → review → 修正 を繰り返す" → 暗黙に各 phase
  の framing note 冒頭で前例として継承されているのみで、明文化された SSOT
  はない

これらが drift した場合の検知機構は現状ない。

## オープンクエスチョン

### Q1. "process" とは何か？SSOT 化の対象範囲はどこか？

候補：

- **(a) 強制力のあるルール（毎回適用 / 違反は是正）だけ SSOT 化**。
  lifecycle・手順は別物として扱う。
- **(b) ルール + lifecycle + 手順の全部を SSOT 化対象**。
- **(c) "活きたルール"（毎回適用）と "経過的記録"（vision ADR、過去の
  framing 等）を分け、前者だけ SSOT 化**。後者は履歴として温存。

批判点：(a) は綺麗だが境界判定が曖昧。「lifecycle 記述」と「強制力のある
ルール」は連続しており、`retrospectives.md` §3 のような checklist は両者の
混合。

### Q2. SSOT の所在は CLAUDE.md か、それとも別ファイルか？

候補：

- **(a) `CLAUDE.md` を process SSOT として正式化**。他文書はすべて
  forward link で参照する形に整理。
- **(b) `CLAUDE.md` は "セッション初期化用の要約 + index" に留め、
  process SSOT は `docs/conventions/`（新設）等に分離**。
- **(c) 領域別に SSOT を分散**（commit ルール → CLAUDE.md、retro 手順
  → retrospectives.md、…）。現状の散在をそのまま追認するが、横断 index
  は別途設ける。

批判点：

- (a) は `CLAUDE.md` がふくらむ。毎セッション読まれる前提なのでサイズは
  無制限ではない（context budget）。
- (b) は新設の `docs/conventions/` への遷移コストが高い。既存の forward
  link を大量に書き換える必要。
- (c) は現状追認に見えるが、"横断 index" を維持しないと "どこに何が書いて
  あるか" が分からない問題は解決しない。

### Q3. 重複と forward link の運用ルール

SSOT を一箇所に決めたあと、他文書からは "短い要約 + リンク" 形式にするのか、
リンクのみにするのか。

要約を許すと SSOT との drift が再発する。リンクのみだと文脈の中で
読みづらい（リンクを開かないと意味が分からない）。中間案として
「**SSOT 側に anchor を増やし、forward link は anchor 単位**」がありうる
が、anchor の長期安定性を別途担保する必要がある。

### Q4. プロセス変更の lifecycle

新ルールはどのフローで追加・変更されるか：

- **(a) 直接 SSOT を編集してコミット**（現在の運用に近い。今回の `b11688b`
  がそう）。
- **(b) vision ADR を立てて昇格させる**（doc-system-redesign の
  DD-V-010..016 がそうだった）。
- **(c) retro で議論したものを framing → SSOT という多段昇格**。

(a) は身軽だが、合意の重み付けが弱い（"気がついたら CLAUDE.md にルールが
増えていた" 問題）。(b) は重いが履歴と "なぜ" が残る。`b11688b` で導入した
ルールはこの基準だと (b) で扱うべきだった可能性がある — 当該 commit は
postmortem を framing file に置くことで "なぜ" を一応残しているが、ルール
昇格の lifecycle としてはアドホック。

### Q5. 過去の散在記述の整理コスト

SSOT 化を決めた場合、現状の散在を整理する作業量は小さくない
（`retrospectives.md` / `plans/README.md` / `decisions/README.md` /
`vision-doc-system.md` の各論を SSOT に引き寄せる、または逆向きの link
整理）。**いつ・誰がやるか**。

候補：

- M3 完了後の整理 phase で一括。
- 触る都度に少しずつ（incremental）。
- 「触ったときに必ず SSOT 側も同期」というルールを立てる（drift 防止に
  強いが負担大）。

## 関連する既決事項

[vision-doc-system.md](../decisions/vision-doc-system.md) DD-V-010..016
で確定済の周辺事項：

- DD-V-013 ROADMAP は acceptance-criteria SSOT
- DD-V-015 plan の 2 層構造（Frozen agreement / Progress）
- DD-V-016 plan → ROADMAP → ADR の commit フロー

これらは**文書 SSOT**を扱う。本ノートが扱う**プロセスルール SSOT**とは
隣接論点で、結論を流用できる部分（"SSOT 化と forward link 運用" の発想）と
できない部分（プロセスルールは ADR 単位で扱うには細かすぎる場合が多い）が
ある。

## 未決のまま

本ノートは「決定するための材料を整理する」段階。Q1〜Q5 に対するオーナーの
方針が固まった時点で、別途 vision ADR または `CLAUDE.md` 直接更新の form
で結論を出す。それまでは：

- `CLAUDE.md` への新規 process ルール追記は、これまで通り **その都度判断**。
  `b11688b` のように retro で surface した重要ルールは `CLAUDE.md` に
  直書きする運用を継続。
- 散在記述同士の forward link は **既存のものは温存**、新規に書く際は
  SSOT 候補（`CLAUDE.md` か領域別 README）への link を貼る。
