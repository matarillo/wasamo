---
title: 開発文書体系の再検討
status: resolved
created: 2026-05-02
resolved: 2026-05-02
related-adrs:
  - docs/decisions/vision-doc-system.md
---

# 開発文書体系の再検討

> **Resolved 2026-05-02.** 本ノートで議論した論点 A〜G は
> [docs/decisions/vision-doc-system.md](../decisions/vision-doc-system.md)
> (DD-V-010..016) として確定済み。CHANGELOG.md 導入(C-4)+ plan
> 2層化(F-5)を含む合意内容は ADR を参照。本ファイルは経緯記録として
> 残す(議論の出発点と却下された選択肢を含む)。

M1 完了直後・M2 計画策定中の現時点で、プロジェクト文書群
(`VISION.md` / `ROADMAP.md` / `README.md` / `docs/plans/` /
`docs/decisions/` / `docs/notes/`) の役割分担と運用ルールを
見直したい。本ノートは検討の出発点であり、合意に至った事項は
ADR (DD-V-010 相当) に昇格させる。

## 動機

オーナーが疑問視している点:

1. **各マイルストーンの acceptance criteria の SSOT がどこか曖昧**。
   プロジェクトルートに置くべきか、`docs/` 配下でよいか。
2. **`ROADMAP.md` の位置づけの不整合**。VISION.md にもロードマップが
   あり重複している。「開発の最新状況を示す文書」が ROADMAP なのか
   別物なのか。完了マイルストーンの詳細タスクリストを残す意味は何か。

Claude による批判的レビューで、上記に加えて以下が浮上した:

3. ROADMAP は (a) acceptance criteria の SSOT、(b) phase 単位の
   タスクトラッカー、(c) 完了マイルストーンの履歴、の **3 役を兼業**
   しており、これが肥大化と役割不明瞭の根本原因。
4. `docs/plans/` の README が定義した
   「plan(提案) → ROADMAP(コミット) → ADR」フローと、
   現状の M2 セクションの実態(plan へのリンクのみで commit 内容を
   ROADMAP 側に転記していない)に**乖離**がある。
5. 「**いま何をやっているか**」を 30 秒で把握できる場所が
   どこにもない。
6. ROADMAP 内の Phase 5/6/7/8 プロローグ散文は ADR の決定要約の
   転記であり、**ADR と二重化**している。supersede が起きると
   ROADMAP 側の追記が必要になる(DD-P5-001..003 の例)。
7. (隣接論点) VISION.md 末尾のバージョン履歴表も git log と
   二重管理。今回のスコープには入れないが同種の問題として記録。

## 現状マッピング

| 情報項目 | VISION §7 | ROADMAP | docs/plans/ | docs/decisions/ |
|---|---|---|---|---|
| M1 acceptance criteria | あり(要約) | あり(正本扱い) | — | DD-V-001 が補足 |
| M2-M6 acceptance criteria | あり(要約) | あり(正本扱い) | M2 のみ詳細 | — |
| Phase 内タスク | — | あり(チェックボックス) | — | ADR が事後的に書き換え |
| 設計決定 | — | プロローグ散文 | — | ADR(正本) |
| 「いま何やってるか」 | — | チェックボックスから推測 | in-progress プラン | — |

## 検討する論点と選択肢

### 論点 A: acceptance criteria の SSOT

- **A-1**: VISION §7 を thesis のみに削り、acceptance criteria は
  ROADMAP に一本化(現状の "ROADMAP is authoritative" 慣習を
  構造的に保証)。**推奨**。
- **A-2**: 逆に ROADMAP を廃止し VISION + plans + decisions で代替。
  → README からの導線を含めて影響大。
- **A-3**: 現状維持(慣習で運用)。→ ドリフト事故が再発する構造。

### 論点 B: ROADMAP.md の役割再定義

- **B-1**: ROADMAP は **acceptance criteria のみ** に痩せる。
  完了マイルストーンは数行+ADR/タグへのリンクに圧縮。
  Phase チェックリストは廃止し、進行中の作業状況は in-progress の
  `docs/plans/<M>-plan.md` が SSOT になる。**推奨**。
- **B-2**: ROADMAP を「現在の状況ボード」と再定義し
  acceptance criteria は別文書に出す。→ 役割がさらに変則的になる。
- **B-3**: 現状維持(3 役兼業)。

### 論点 C: 完了マイルストーンの phase チェックリスト

- **C-1**: 圧縮(数行+ADR/タグへのリンクのみ残す)。git log と
  ADR で詳細は復元可能。`docs/plans/` の archival policy
  (デフォルト delete)と思想を揃える。**推奨**。
- **C-2**: 温存(歴史記録としての価値)。→ ROADMAP が継続的に
  肥大化、frozen にできない。
- **C-3**: 別ファイルに退避(`docs/history/m1.md` 等)。
  → 文書種別を増やす負債。

### 論点 D: 「現在の開発状況」の置き場所

- **D-1**: `README.md` に "Status" セクション 1〜2 行
  (例: `M1 shipped (v0.1.0). Currently planning M2.`)+
  進行中の plan / 直近 ADR へのリンク。**推奨**。軽量。
- **D-2**: 専用 `STATUS.md` を新設。→ 文書種別を増やす。
- **D-3**: ROADMAP 冒頭に "Now" セクション。→ ROADMAP の
  3 役兼業を温存することになる。

### 論点 E: plans → ROADMAP の転記運用

論点 B-1 を採るなら、plan は **acceptance criteria を持たず**、
phase 分解と依存関係のみを担う。ROADMAP は plan へリンクするのみで
phase 構造を転記しない。これで `docs/plans/README.md` の
"plan → ROADMAP commit" フローを **「acceptance criteria の commit」**
に再定義する必要がある。plans/README の改訂が伴う。

### 論点 F: タスクトラッカーの必要性と置き場所

論点 B-1 で ROADMAP からチェックリストを廃止する場合、
**phase 内タスクの進捗トラッキングをそもそも文書として残す
必要があるか**を別途決める必要がある。オーナーの整理:

- **外部公開する意味はない**(GitHub の閲覧者は acceptance criteria
  と現在のマイルストーンが分かれば十分。phase 内タスクの
  チェック状態は外部価値が低い)。
- **オーナー自身の備忘録としては欲しい**(次のセッションで
  「どこまで進んだか」を思い出すため)。
- **Claude の SSOT としては、auto memory や CLAUDE.md で
  十分な可能性がある**(in-progress plan + memory で代替可)。

選択肢:

- **F-1**: タスクトラッカー文書を持たない。in-progress の
  `docs/plans/<M>-plan.md` に phase 一覧があり、進捗は
  git log + auto memory + CLAUDE.md で追跡。オーナーの備忘録は
  auto memory に蓄積させる(`project_wasamo.md` を進捗で更新)。
  → 文書数最小。Claude には memory が効くが、オーナーが手元で
  俯瞰したい時に GitHub UI からは見えない。
- **F-2**: `docs/notes/` 配下にオーナー専用の進捗ノート
  (例: `progress-m2.md`)を置く。Japanese OK、live ドキュメント。
  完了時 archival policy で削除。→ オーナーの備忘録ニーズと
  外部非公開の中間。Claude も読める。
- **F-3**: in-progress の plan 文書を **frozen にせず**、
  phase チェックボックスを残して進捗トラッカーとして使う。
  → 現状の `docs/plans/README.md` の「in-progress = read-only」
  ルールを改める必要あり。plan が「合意 artifact」と
  「進捗トラッカー」の 2 役を兼ねるリスク(ROADMAP の轍)。
- **F-4**: GitHub Issues / Projects に逃がす。→ 外部ツール依存、
  Claude が読みづらくなる、オーナーの好みと合うか別途確認。

**初期推奨**: F-2(`docs/notes/progress-<M>.md`)。理由:
- オーナーの備忘録ニーズを満たす
- `docs/notes/` の既存ルール(Japanese OK / live / 完了時削除)に
  そのまま乗る
- Claude も auto memory と併用して読める
- plan を frozen に保てる(plan/README の規律を温存)

ただし F-1(memory のみ)も検証に値する。auto memory の
`project_wasamo.md` を進捗で更新する運用が回るかどうかは、
M2 の最初の数 phase で試してから決めても良い。

### 論点 G: ADR プロローグ散文の扱い

ROADMAP の Phase 5/6/7/8 プロローグ散文(ADR の決定要約)は
ROADMAP からは削除し、ADR 側に statement of work として吸収。
ROADMAP からは ADR へのリンクのみ残す。論点 B-1 を採るなら
phase チェックリスト自体が消えるので、自然に解決する。

## 推奨案(たたき台)

| 文書 | 役割 |
|---|---|
| `VISION.md` | なぜ・何を作るか。thesis のみ。acceptance criteria は持たない |
| `ROADMAP.md` | 全マイルストーンの **acceptance criteria のみ**。完了は数行+リンク |
| `README.md` | エレベーターピッチ + **現在のステータス 1 行** + 主要文書への導線 |
| `docs/plans/<M>-plan.md` | 進行前の合意 artifact。in-progress 中は「現在の作業」の SSOT。完了で archival policy 通り削除 |
| `docs/decisions/` | 設計決定の SSOT(プロローグ散文も含めここに集約) |
| `docs/notes/` | 現状のまま(オーナーの探索記録) |

## 合意したい順序

1. **論点 A**(acceptance criteria の SSOT)→ 全体設計の起点
2. **論点 B**(ROADMAP の役割再定義)→ A の帰結
3. **論点 C**(完了 phase チェックリストの扱い)→ B の帰結
4. **論点 D**(現在状況の置き場所)→ B-1 を採るなら必要
5. **論点 E**(plans → ROADMAP 転記運用)→ B の運用詳細
6. **論点 F**(タスクトラッカーの必要性と置き場所)→ B-1 を採るなら必要
7. **論点 G**(ADR プロローグ散文の扱い)→ B の帰結で自然解消の可能性

## Open Questions

- VISION.md の Roadmap セクション(§7)は **完全削除**まで踏み込むか、
  thesis 1 行だけ残すか。
- 完了マイルストーンの ROADMAP 上の表記フォーマット
  (例: `## M1 ✅ Proof of Concept — shipped v0.1.0 (2026-05-01). See [decisions/](...)`)
  の正規形を決める必要がある。
- `docs/plans/README.md` の「plan → ROADMAP commit」フローの
  再定義文言。
- VISION.md のバージョン履歴表(隣接論点)を本検討に含めるか
  別途扱うか。
