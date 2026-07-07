---
title: 開発プロセスのすすめかた — マイルストーン計画から実装、クロージングまで
status: SSOT
created: 2026-05-27
---

# 開発プロセスのすすめかた

本ドキュメントは Wasamo の開発をどう進めるかを定める SSOT。
主な対象読者は **AI エージェント**（オーナーも参照）。
個別のルールや手順は他の SSOT を参照する：

- フォルダ構造・命名規約 → [process/README.md](../README.md)
- 強制力のあるルール（テスト・コミット・CI 等） → [AGENTS.md](../../AGENTS.md)
- レトロスペクティブとマージゲート → [process/procedures/retrospectives.md](retrospectives.md)
- プロセスルールの変更ライフサイクル（規範） → [AGENTS.md §Process rule lifecycle](../../AGENTS.md#process-rule-lifecycle)。SSOT 分散と由来 → [process/cross-milestone/decisions/process-rule-ssot.md](../cross-milestone/decisions/process-rule-ssot.md)

---

## 全体フロー

開発は **マイルストーン** > **フェーズ** の階層で進む。マイルストーンの
内部でフェーズが繰り返される。

```
[マイルストーン階層]
   §1 マイルストーン計画
        ↓
   ┌──── [フェーズ階層] (フェーズごとに繰り返し) ────┐
   │   §2 フェーズ計画                                │
   │        ↓                                         │
   │   §3 設計判断                                    │
   │        ↓                                         │
   │   §4 実装計画                                    │
   │        ↓                                         │
   │   §5 実装                                        │
   │        ↓                                         │
   │   §6 フェーズクロージング                        │
   └──────────────────────────────────────────────────┘
        ↓
   §7 マイルストーンクロージング
        ↓
   (次のマイルストーン §1 へ)
```

各段階のアウトプットは `process/milestone-N/` 配下の決まった場所に書く
（→ [process/README.md](../README.md) の Structure 参照）。

`process/_roadmap.md` は全マイルストーンの acceptance criteria を集約する
SSOT。マイルストーン計画・設計同期・実装同期・クロージングの各段階で
アウトプットが流入する **ハブ文書** として機能する。

---

## ドキュメントのライフサイクル

各段階のアウトプットは以下の 3 ステップを辿る：

1. **ドラフト** — AI エージェントが下書きを書く
2. **オーナーレビュー** — オーナーが評価し、修正要求があればドラフトに戻る
3. **Accepted** — オーナー承認で確定

確定後の扱いは文書の性格による。frontmatter の `status` フィールドで
遷移を明示する。

> **表記の例外：** ADR (`decisions/preamble.md`、`dd-NNN-*.md`) と vision
> decision record は frontmatter ではなく本文先頭の `**Status:** Accepted`
> 形式で記述する。これは ADR の表記慣習を踏襲したもので、capitalization も
> `Accepted` / `Proposed` / `Superseded`（大文字始まり）を用いる。その他の
> 文書は frontmatter `status: accepted`（小文字）。

### 凍結文書

合意後は revisions セクションで明示的に管理する以外は変更しない。

| 文書 | ステータス遷移 |
|---|---|
| `milestone-N/requirements/framing.md`, `spec.md` | `draft` → `accepted` |
| `phase-M/requirements/framing.md`, `constraints.md` | `draft` → `accepted` |
| `decisions/preamble.md`, `dd-NNN-*.md` | `Proposed` → `Accepted` → (`Superseded`) |
| `retrospectives/tN.md`, `phase-end.md` | `draft` → `recorded` |
| `implementation/handoff.md` | `draft` → `recorded`（フェーズクローズ時） |
| `milestone-N/handoff.md` | `draft` → `recorded`（マイルストーンクローズ時） |

### 継続文書

合意後も実装中の更新を受け付け、最終的に閉じる。

| 文書 | ステータス遷移 |
|---|---|
| `milestone-N/plan.md` | `draft` → `in-progress` → `completed` |
| `implementation/plan.md` | `draft` → `in-progress` → `completed` |
| `implementation/log.md` | （ステータスなし、append-only。フェーズクローズで実質凍結） |

### 探索文書

ADR や決定に昇格して終了、または却下されて消化される。

| 文書 | ステータス遷移 |
|---|---|
| `cross-milestone/decisions/exploration/*.md` | `open` → `resolved` |
| `docs/notes/*.md` | `live` → `resolved`（または削除） |

### 進化文書 (living documents)

ロードマップと技術仕様書は **閉じない**。設計同期 / 実装同期のたびに継続的に
更新され、常に最新版だけが SSOT。過去版は git 履歴で参照する。

| 文書 | 性格 |
|---|---|
| `process/_roadmap.md` | 全マイルストーンの acceptance criteria を集約するハブ |
| `docs/architecture.md` | 全体アーキテクチャの解説 |
| `docs/abi_spec.md` | C ABI の規範記述 |
| `docs/dsl_spec.md` | `.ui` DSL の規範記述 |

これらの文書では他カテゴリと異なり `Status` フィールドは凍結度ではなく
**到達点**（どのマイルストーン / フェーズ実装まで反映されているか）を示す：

- **Status**: 反映済みの最新フェーズ（例: `M3-Phase 4 closed (implementation-synced)`）
- **Last updated**（任意）: 最終更新日
- **Document version**（任意）: 細粒度の改訂を追いたい場合の文書バージョン

設計同期 / 実装同期のタイミングで本文と一緒にヘッダの Status も最新化する。

### 計画(plan)改訂の規律

`milestone-N/plan.md` の `## Frozen agreement` は `status: in-progress` でも
**read-only ではない**。計画は計画時点の仮説であり、前提が変われば適切な手順で
改訂できる。守るのは agreement の不変性ではなく、変更が **単独・無吟味でない**
ことと **監査可能** であること。由来と論拠は
[plan-revision-discipline.md（DD-V-026）](../cross-milestone/decisions/plan-revision-discipline.md)。

**ゲート（全 tier 共通）:**

- **エージェントは合意済み agreement 本文を単独で改訂しない。** 提案し、オーナーが
  authorise する。binding は「オーナーが合意したもの」に係り status ラベルではない
  ——未合意の内容（未レビュー draft、レビュー中の未確定部）は自由に編集してよい。
  提案は別 artifact（Revision-log エントリ草案）として出し、本文 land は承認後。
- **前提が変わったと気づいたら改訂を提案するのはエージェントの積極的義務。** 黙認は
  無断変更と同種の失敗。
- **批判的チェックは起点でない側が行い、起点の自己採点は禁止。** エージェント提案→
  オーナーが check、オーナー起点→エージェントが check。

**記録（比例3 tier）:**

1. **Editorial / factual** — 文言・移動した path・cross-reference。Revision-log 1 行。
   識別子・参照グラフ・規範的意味を変えない機械的修正のみ（識別子 rename は原則 tier 2）。
2. **Scope / AC / phase 構成** — AC の追加/refine/supersede、phase の挿入/並べ替え、
   依存・acceptance↔phase mapping・out-of-scope の変更。批判チェック済み前提＋根拠付き
   Revision-log。既存 AC ID 保持（silent renumber 禁止）、AC 変更時は `process/_roadmap.md`
   を mirror。方向で非対称：
   - **追加/refine** — 一行 impact check（既存 AC 意味・依存順・完了 phase 評価・
     retro/merge gate・ROADMAP mirror 要否）。
   - **撤回/narrowing** — 加えて deferral-with-trigger 表（責務の置き先＋activation
     trigger）。silent drop 禁止。
3. **Thesis / purpose 反転** — Revision-log ＋ vision decision record。批判チェックは
   可能なら独立レビュー。

**status scope:** ゲートと軽量 tier は `draft` / `in-progress` の間のみ。`completed`
plan の agreement は軽く書き換えない——factual 修正 (tier 1) は archival correction、
完了済みの substantive 再解釈は milestone `handoff.md` / phase retrospective・postmortem /
`process/_roadmap.md` revision note /（規範変更なら）VDR に retraction weight で記録する。

**Revision-log 最小テンプレ（tier 2/3）:** what/tier・initiator・old premise・
new evidence・why the old plan no longer holds（insufficient / incorrect / lower-confidence）・
no-change option・critical check（非起点側、自己記入禁止）・owner authorisation。提案時は
critical check と owner authorisation を `pending` とし、本文 land は両充足後。

以下の各段階では、特記なき限りこのライフサイクルに従う。

---

## このフローからの逸脱

以下の場合は本ワークフローから逸脱する判断がありうる。逸脱する場合は
[AGENTS.md §Process rule lifecycle](../../AGENTS.md#process-rule-lifecycle)
の構造的変更フローに従い、vision decision record を立てて記録する：

- 新しい段階の追加（例：フェーズ内に design exploration 段階を独立させる）
- 既存段階のスキップ（例：設計判断が不要な小さなフェーズ）
- 段階の意味再定義

軽微な調整（プロセス記述の言い回し変更等）は本ドキュメントを直接編集する
（→ [process/cross-milestone/decisions/process-rule-ssot.md DD-V-020](../cross-milestone/decisions/process-rule-ssot.md#dd-v-020--process-rule-change-lifecycle)）。

---

## 1. マイルストーン計画

新規マイルストーンを開始するときの段階。アウトプットは
`process/milestone-N/requirements/` と `process/milestone-N/plan.md`。

4 つのサブ活動に分かれる：

### 1.1 引き継ぎ確認

以下の 3 つを確認する：

1. **前マイルストーンの `handoff.md`** — `process/milestone-N-1/handoff.md`
   を読み、本マイルストーンに効く制約・未解決事項を確認する。
2. **`docs/notes/` の live ノート** — 各ノートの "Re-evaluation triggers"
   を確認し、本マイルストーンで発火するものがないか点検する。発火している
   ものは取り込み判断（本マイルストーンで解決するか、引き続き live のまま
   置くか）。
3. **Pre-1.0 candidate pool
   （[process/candidate-pool.md](../candidate-pool.md)）** — 各 item に
   処遇（`take (milestone N)` / `hold` / `retire`）を判断し、同ファイルの
   disposition log に日付付きで記録する
   （[DD-V-028](../cross-milestone/decisions/pre-1.0-candidate-pool.md)
   の Forcing artifact）。`take` / `retire` は着地先リンク必須
   （destination-link rule）。`ABI-bearing: unknown` の item は M6 直前の
   マイルストーン計画までに `yes` / `no` へ解消する。

1 と 2 はマイルストーンレベルでは内部判断として消化し、明示的な文書化は
不要。3 の pool 処遇記録は必須（上記）。
取り込みが構造的決定を要する場合は個別の vision decision record を立て、
解決した live ノートは `status: resolved` に遷移させて
`process/cross-milestone/decisions/exploration/` への移動を検討する。

### 1.2 目標策定

「何を作れば本マイルストーンを達成したと言えるか」を具体化する。

- **アウトプット**:
  - `requirements/spec.md` — target app の仕様
  - `requirements/*-wireframes.html` — UI ワイヤーフレーム（必要なら）

### 1.3 方向性とスコープの確定

マイルストーン全体の方向性（thesis 解釈、target app、framing decisions）と
境界線（含むもの・含まないもの）を確定する。acceptance criteria 自体は
`_roadmap.md` に集約されているので、framing.md では AC の読み方や revise
方針を記録する。

- **アウトプット**:
  - `requirements/framing.md` — thesis 解釈、含む/含まないもの、初期 phase 構成仮説、owner-agreed framing decisions
  - `process/_roadmap.md` の該当マイルストーン項を最新化（AC の SSOT）

### 1.4 計画策定

スコープを実行可能な単位（フェーズ）に分解し、順序と依存を確定する。

- **アウトプット**:
  - `milestone-N/plan.md` — phase breakdown、各 phase の目標と依存

`plan.md` の `## Frozen agreement` は `status: in-progress` 以降も read-only では
なく、下記「計画(plan)改訂の規律」のゲートと比例記録に従って改訂できる
（[DD-V-026](../cross-milestone/decisions/plan-revision-discipline.md)）。

---

## 2. フェーズ計画

新規フェーズを開始するときの段階。**設計判断という重い段階が後続する**
ため、task breakdown はここでは行わない（→ §4 実装計画）。

アウトプットは `process/milestone-N/phase-M/requirements/`。
本段階の中心活動は **論点とスコープの確定**（§2.2〜§2.4）で、その前に
**制約引き継ぎ**（§2.1）が走る。4 つのサブ活動に分かれる：

### 2.1 制約引き継ぎ

前フェーズの `phase-M-1/implementation/handoff.md` から、本フェーズに効く
制約を切り出す。`handoff.md` は前 phase 側の永続記録であり、
`requirements/constraints.md` は本 phase 専用の解釈結果である。
単純コピーを目的にせず、本 phase の論点 (§2.2) / スコープ (§2.3) /
検証方針 (§2.4) と関係づけて再構成する。採用しない handoff 項目がある
場合は、なぜ本 phase の制約にしないかを `constraints.md` か
`requirements/framing.md` の revisions に短く残す。

- **アウトプット**:
  - `requirements/constraints.md` — 本フェーズで前提となる制約

### 2.2 論点設定

本フェーズで判断すべき問い（DD = Design Decision）を列挙し、オーナーと合意
する。DD 番号もここで予約する。

- **アウトプット**:
  - `requirements/framing.md` の DD slate セクション
  - DD-MX-PY-NNN 番号の予約

### 2.3 スコープ確定

含むもの・Out of scope を確定し、マイルストーンスコープとの対応を明示する。

- **アウトプット**:
  - `requirements/framing.md` のスコープセクション

### 2.4 検証方針確認

各 DD と acceptance criteria を、どの test / example / CI run で discharge
する方針かを仮確定する（具体化は実装フェーズで）。

- **アウトプット**:
  - `requirements/framing.md` の verification セクション

フェーズ計画段階の成果物 `requirements/framing.md` は、後段で凍結。実装中に
追記する場合は revisions セクションで管理。

---

## 3. 設計判断

フェーズ計画で確定した DD slate に従い、各 DD について Options を列挙し、
批判的評価のうえで Recommendation を選ぶ。

- **アウトプット**:
  - `decisions/preamble.md` — フェーズ全体の Context、Summary、Out of scope、Revisions
  - `decisions/dd-NNN-<slug>.md`（DD ごとに 1 ファイル）

DD の構造：Context → Options（複数案） → Comparison → Recommendation →
Forward-compat exposure → Technical risk re-evaluation。

DD を Accepted に進める前の複数視点レビューには、おすすめ手順とプロンプト
テンプレート集がある（強制ゲートではない）：
[design-decision-review.md](./design-decision-review.md)。

Accepted フリップのコミットに続けて **設計同期** を行う（次節参照）。

### 3.1 設計同期 (Moment 1)

ADR が Accepted になったタイミングで、上流文書を ADR の決定内容に合わせて
同期更新する。スコープは設計時点で確定したものに限る（実装結果ではない）。

同期対象の例：
- `_roadmap.md` の該当 phase 項
- `docs/dsl_spec.md` などの normative spec
- VISION.md（vision 影響があれば）
- 他の関連 ADR（クロスリファレンス）

同期対象の文書群は review concern 単位で分割し、**1 つの review concern を
共有する文書は 1 commit にまとめる**（review concern が異なる文書は別
commit）。doc-side commit のルールは
[AGENTS.md §Commit rules](../../AGENTS.md#commit-rules) 参照。

Accepted + 設計同期完了後、§4 実装計画に進む。

---

## 4. 実装計画

設計判断（ADR Accepted）の後、設計内容を実装可能な task に分解する。

- **アウトプット**:
  - `implementation/preamble.md` — フェーズ実装の intro、task 順序の根拠
  - `implementation/plan.md` — task list（チェックリスト形式）

task 順序は依存方向に従って組み、各 task が green workspace を維持する
（→ [AGENTS.md §Commit rules](../../AGENTS.md#commit-rules) の 1 commit per
task-list item 原則）。

実装計画段階の成果物 `plan.md` は **mutable**。実装中に task の分割・追加・
順序変更が起きたら反映する（plan changes mid-implementation are normal）。

---

## 5. 実装

`implementation/plan.md` の task を 1 つずつ実行する。

### 5.1 task 実行

各 task を 1 commit にまとめるのが基本（→ [AGENTS.md §Commit rules](../../AGENTS.md#commit-rules)）。
bundling が必要な場合（中間状態が build を壊す等）は plan を更新したうえで
合理化する。

### 5.2 task retrospective

各 task の完了時に task-end retrospective を回す。手順は
[process/procedures/retrospectives.md](retrospectives.md) 参照。

- **アウトプット**:
  - `retrospectives/tN.md` — task N の振り返り（または `dd-NNN.md` 形式）

task retrospective の通過後にオーナー承認を経て task branch を phase branch に
no-ff merge する。

### 5.3 log への記録

実装中に surface した追加判断・CI 実行結果は `implementation/log.md` に追記
する。`log.md` は Decisions log（追加判断）と CI / verification log（検証
記録）の混合。

### 5.4 evidence の保存

スクリーンショット、CI run の証拠、smoke 動画等は
`implementation/evidence/` に保存する。命名は `tN-<purpose>.<ext>` 形式。

---

## 6. フェーズクロージング

全 task が完了したタイミングで、フェーズを締める。

### 6.1 phase-end retrospective

フェーズ全体を振り返り、acceptance criteria の最終 mapping を確定する。
task retrospective の項目 10 で蓄積した `carry-forward` 候補を一覧し、
`doc-folded` 相当 / `carry-forward` / `local-only` の最終分類もここで
確定する。確定した handoff 対象は §6.3 で清書する。
手順は [process/procedures/retrospectives.md](retrospectives.md) 参照。

- **アウトプット**:
  - `retrospectives/phase-end.md`
  - 最重要セクション: `## Phase-End Gate` に検証クロージャの最終 mapping
    （どの DD verification criterion が どの test/example/CI run で
    discharge されたか）

### 6.2 実装同期 (Moment 2)

フェーズ実装の結果を上流文書に反映する。スコープは実装中に確定したことに
限る（設計時に確定したものは設計同期で反映済み）。

同期対象の例：
- `_roadmap.md` の phase 完了マーク、AC 達成記録
- 関連 spec の最終確認
- CHANGELOG への追記（適用される場合）

実装同期も設計同期と同じく、review concern 単位で分割し、1 つの review
concern を共有する文書は 1 commit にまとめる（review concern が異なる文書は
別 commit）。

### 6.3 handoff の整理

phase-end retrospective で確定した次フェーズ向けの材料を
`implementation/handoff.md` に清書する。retrospective 中は
`implementation/handoff.md` を確定成果物として更新しない。対象は
out-of-scope items だけでなく、次 phase に効く Main Learning、未決論点、
引き継ぎ制約を含む。主な内容源は task retrospective の項目 10 に記録
された `carry-forward` 候補と、phase-end retrospective で最終的に
`carry-forward` と確定した設計制約
（→ [retrospectives.md §Main Learning と設計制約の前送り](retrospectives.md)）。

清書では、複数 task に分散した同種の制約を統合し、次 phase の §2.1
制約引き継ぎで読みやすい構造に整える。`doc-folded` 相当または
`local-only` に閉じた候補は `implementation/handoff.md` の本文には
含めない（必要なら pointer のみ）。

- **アウトプット**:
  - `implementation/handoff.md` — 次フェーズの §2.1 制約引き継ぎで
    `requirements/constraints.md` の原典として読まれる永続記録。
    `constraints.md` は次 phase 側で論点・スコープ・検証方針に合わせて
    再構成する

### 6.4 phase branch のマージ

phase retrospective の通過とオーナー承認で phase branch を main に no-ff
merge する。push は別ゲート（→ [retrospectives.md](retrospectives.md)）。

---

## 7. マイルストーンクロージング

マイルストーン内の全フェーズが完了したタイミングで、マイルストーンを締める。

### 7.1 マイルストーン全体の振り返り

各フェーズの phase-end retrospective を読み返し、マイルストーン acceptance
criteria の達成を確認する。

### 7.2 マイルストーン handoff

次マイルストーンに引き継ぐ設計前提と残課題を整理する。

- **アウトプット**:
  - `milestone-N/handoff.md` — 次マイルストーンの計画段階で取り込まれる

### 7.3 ROADMAP 更新

`process/_roadmap.md` の該当マイルストーン項を「完了」状態に更新し、
acceptance evidence と最終アウトプットへのリンクを残す。

### 7.4 リリース（該当する場合）

v0.X.0 タグ付けと CHANGELOG 整備。手順は別途確立予定。

---

## 用語集

### DR の階層

| 用語 | 意味 |
|---|---|
| **DR** | Decision Record。決定の永続記録。**ADR** と **vision decision record** の 2 種類 |
| **ADR** | Architecture Decision Record。フェーズの設計判断を記録する DR。`process/milestone-N/phase-M/decisions/` 配下 |
| **vision decision record** | マイルストーン横断の判断を記録する DR。実際には vision だけでなく、governance / policy / roadmap に関する決定も含む。`process/cross-milestone/decisions/` 配下 |
| **DD** | Design Decision。DR 内の個別決定。ID は uppercase（ADR では `DD-MX-PY-NNN`、vision decision record では `DD-V-NNN`）、対応ファイル名は kebab-case（`dd-MX-PY-NNN-<slug>.md`） |

### DR の状態

| 用語 | 意味 |
|---|---|
| **Proposed** | DR がドラフト中で未確定の状態 |
| **Accepted** | DR がオーナー承認で確定した状態 |
| **Superseded** | DR が後続の決定に置き換えられた状態（履歴として保存） |

### DR の構成要素

| 用語 | 意味 |
|---|---|
| **論点** | 判断すべき問い。DD 1 つが 1 論点に対応 |
| **スコープ** | 何を含み、何を含まないかの境界 |
| **framing** | DR drafting 前段階の中心活動。アウトプットは `requirements/framing.md`。フェーズでは「論点とスコープの確定」（§2）、マイルストーンでは「方向性とスコープの確定」（§1.3）と呼ぶ |

### 段階間の同期操作

| 用語 | 意味 |
|---|---|
| **設計同期** | ADR Accepted フリップ後、決定内容を上流文書に反映する操作（Moment 1） |
| **実装同期** | フェーズ終了時、実装結果を上流文書に反映する操作（Moment 2） |

### 実装単位

| 用語 | 意味 |
|---|---|
| **task** | 実装計画における作業単位。`implementation/plan.md` のチェックリスト 1 項目に対応 |
| **task branch** | 1 task に対応する作業ブランチ。task retrospective 経由で phase branch に no-ff merge |

### 引き継ぎ関連

| 用語 | 意味 |
|---|---|
| **handoff** | 次フェーズ / 次マイルストーンに引き継ぐ材料（carry-forward 制約、Main Learning、未決論点、out-of-scope residual）。`implementation/handoff.md` または `milestone-N/handoff.md` |
| **carry-forward** | task / phase-end retrospective 項目 10 / 15 の分類タグの 1 つ。次 phase に前送りすべき設計制約を示す |

### 文書性質

| 用語 | 意味 |
|---|---|
| **living document** | 閉じない文書。`process/_roadmap.md` および技術仕様書（`docs/architecture.md`、`docs/abi_spec.md`、`docs/dsl_spec.md`）。設計同期 / 実装同期のたびに更新 |

### コミット規約

| 用語 | 意味 |
|---|---|
| **review concern** | 1 つのレビューで一緒に評価される文書群の単位。同じ concern を共有する文書は 1 commit にまとめる |
