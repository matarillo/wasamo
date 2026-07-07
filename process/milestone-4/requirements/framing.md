---
title: M4 start framing — Interaction stack
status: draft
created: 2026-07-07
related:
  - process/_roadmap.md
  - process/milestone-3/handoff.md
  - process/candidate-pool.md
  - docs/notes/m4-interaction-intake.md
  - process/milestone-4/requirements/target-app-wireframes.html
---

# M4 start framing — Interaction stack

M4 開始時のマイルストーンレベル framing（workflow §1.3）。M3 の前例
（[M3 framing.md](../../milestone-3/requirements/framing.md)）と同じく、
target app 採択・ROADMAP acceptance 見直し・plan drafting に先立って、
thesis の読み・取り込み方針・判断基準を固定する。

§1.1 引き継ぎ確認（2026-07-07 実施、オーナー合意済）の結論を取り込み、
target app 採択（A/B、未確定）を本 framing の中心の問いとして扱う。

---

## M4 thesis の読み

[_roadmap.md](../../_roadmap.md) §M4 の thesis:

> input, multi-window, text input, and accessibility share a focus
> model; they ship together so the focus model is settled once.

これを次のように読む:

- **M4 の中心 deliverable は focus model の一回確定**である。input handling・
  TextField・IME・AccessKit・multi-window は個別 feature ではなく、
  同一の focus model を共有する消費者として一括で設計・出荷する。
- したがって M4 への取り込み判断の基準は、スケジュールの都合ではなく
  「**focus model settlement の質を脅かすか**」に置く（F1）。
- Mica / Acrylic + accent + per-monitor DPI は「Wasamo identity が
  demonstrable になる」ための土台であり、first showcase（contributor
  outreach）が visible proof を担う。

## §1.1 引き継ぎ確認の結果（取り込み）

3 入力の消化結果。詳細判断はオーナー承認済（2026-07-07 チャット）。

### M3 handoff から

- **M4 residual cluster**（実画像・thumbnail hit-testing・wheel/drag
  scroll・lightbox modal focus・dynamic title/status・runtime DPI）は
  target app 採択に依存して消化先が変わる（→ §target app）。DPI と
  modal focus 系は AC / intake ケースとして app 非依存に扱う。
- **Author-controllable sizing spike** — [VDR](../../cross-milestone/decisions/author-controllable-sizing-surface.md)
  により M4 計画は spike の振り分け（デフォルト M4、M5 送りは積極的
  正当化が必要）を記録する義務を負う。→ §最初に決めるべき問い 7。
- TextField の two-way binding を設計する際、M3 selected-state 決定の
  deferred 軸「Two-way binding」の reopen トリガー（family-consistent
  two-way binding design opens）が**発火する**。M4 の該当 phase の DD で
  明示的に扱う。
- PM-2 Grid wrapper rule・default alignment・placement spelling は
  M4 では reopen トリガーが発火しない見込み。持ち越し。

### live notes から（トリガー点検）

M4 で発火するノート 6 本。いずれも `status: live` のまま、M4 の
framing / 各 phase の入力として消費する:

| ノート | 発火内容 / M4 での扱い |
|---|---|
| [m4-interaction-intake.md](../../../docs/notes/m4-interaction-intake.md) | ノート全体が M4 framing 必須入力。具体ケース 1〜6 を `M4 scope / M4 explicit defer / M5+ / 1.0 required` に分類する義務（→ §open question の扱い） |
| [host-state-boundary.md](../../../docs/notes/host-state-boundary.md) | 「M4 input / TextField / focus model 設計時」トリガー発火。pool take (core) と一体 |
| [dsl-grammar.md](../../../docs/notes/dsl-grammar.md) Q2 | 「M4 の Mica / Acrylic 導入」トリガー発火。Window-prop family（backdrop / theme / dynamic title / WindowConfig）を pool take (core) と一体で扱う |
| [top-layer-overlays.md](../../../docs/notes/top-layer-overlays.md) | 「M4 input / focus model の pre-doc」トリガー発火。pool take (core) と一体 |
| [expression-language-roadmap.md](../../../docs/notes/expression-language-roadmap.md) | M-expr1 述語（M4 早期仮説）+ M-expr5 双方向束縛（TextField）が発火見込み。pool take (core) と一体 |
| [layout-engine.md](../../../docs/notes/layout-engine.md) §3.1 / §3.2 | DPI 局所化・AccessKit 同期 — M4 AC そのものの設計入力 |

発火しないノートは全件 live 維持（resolve なし）。ただし
[architectural-family.md](../../../docs/notes/architectural-family.md) の
トリガー 3（`BindingTarget` に収まらない binding shape）は TextField
two-way binding の設計中に発火しうるため、該当 phase で監視する。
[release-distribution.md](../../../docs/notes/release-distribution.md) G 節の
「M4 = 1.0 cut の準備」は roadmap 変遷（M6 = 1.0）に対して stale な
記述（ノート側の小残件、本 framing では扱わない）。

### Pre-1.0 candidate pool から（処遇合意）

12 item の処遇をオーナー合意済（2026-07-07）。
[candidate-pool.md](../../candidate-pool.md) の disposition log への記入は、
destination-link rule により **本 framing の確定（着地先の成立）後**に行う
（同一 planning 内）。

| 処遇 | Item |
|---|---|
| take (M4 **core**) — AC 昇格 | host state boundary（ABI-bearing: yes）/ expression predicates（M-expr1/2a）/ top-layer overlays / window config props |
| take (M4 **stretch**) — target-app feature、AC 非昇格。**A 採択が前提** | `Image` widget / literal `fill`（Box 以外） |
| hold | themed-widget 背景色（M5 theming）/ reactive `fill` / `TypedValue`（M-expr2b/3）/ developer debug support（M5）/ release distribution（narrow slice のみ本 framing 入力、→ §open question の扱い）/ component extension model |

item count = 12（DD-V-028 の growth falsifier 記録用）。

## 二層取り込み方針（owner-agreed）

オーナー意向「pool の M4 関連 item はなるべく M4 で実施したい」を framing の
出発点として明文化する（F3）。ただし無条件の最大取り込みは、(a) 撤回時の
手続きコスト非対称（AC 化した item の削除は DD-V-026 tier 2 narrowing）、
(b) target app の feature 袋化による thesis 検証の希薄化、を招くため、
取り込みを二層に分ける（F2）:

- **M4 core（AC 昇格）** — focus model と設計が結合する item。
  ROADMAP M4 AC に昇格し、落とす場合は DD-V-026 tier 2 narrowing
  （deferral-with-trigger 表つき）を踏む。
- **M4 stretch（target-app feature、AC 非昇格）** — thesis と独立で
  切り離し可能な葉。`requirements/spec.md` の app 仕様には入れるが
  ROADMAP AC にはしない。落とす場合は pool へ戻す 1 行 disposition で
  済む（安価・監査可能な escape valve）。
- **checkpoint** — focus model の中核 ADR が Accepted になった時点で
  stretch の残量を再評価する（「どうしても収まらない」の判定を感覚では
  なく事前に決めた時点で行う）。

## M4 target app を framing で決める

M3 の実績順序（framing で基準 → wireframes で候補比較 → spec.md で採択
明文化）にならい、target app 採択を本 framing の中心の問いとする。
候補と軸別素読み（均等加重）の正本は
[target-app-wireframes.html](target-app-wireframes.html)。

- **候補 slate**: A（Photo Gallery 成熟 / M3 継続）、B1（付箋）、
  B2（設定画面クローン）、B3（チャット / ローカル echo bot）。
- **判断の構図**（均等加重の読み）: 単独首位なし。B3 = thesis 中心性
  （IME / host state）最強、B2 = 検証密度（focus / overlay）最強だが
  multi-window に穴、B1 = multi-window / identity 最強だが複数行編集
  リスク高、A = 工数・M3 residual 消化最強だが text input が周辺的。
  **軸の加重（オーナー）が決定打になる。**
- オーナーの現時点の傾き: B 系（2026-07-07 チャット、未確定）。

### 採択に依存して変わるもの（app-dependent 帰結）

- **stretch take（Image widget / literal `fill`）は A 前提の判断。**
  B 系採択時は Image widget は pool へ戻す（hold、M5 widget set lean）。
  literal `fill` は B1 なら note 色として残留、B2/B3 なら再判定。
- **M3 handoff の M4 residual cluster** は A なら app がそのまま消化。
  B 系採択時は M5 への carry を本 framing の revisions で明示 disposition
  する（silent drop 禁止）。
- **multi-window AC の discharge 方法**: B2 採択時のみ、app 内に自然な
  出番がないため、最小 fixture の別建て等の discharge 方法を追加で決める。

## ROADMAP acceptance の扱い

- AC の SSOT は [_roadmap.md](../../_roadmap.md)。M4 AC は target app
  採択後に「app に照らして審査」し、core take 4 件の AC 昇格を含む
  revision を起こす（DD-V-026 tier 2 追加/refine: 一行 impact check +
  Revision-log + `_roadmap.md` mirror）。
- 既存 AC（input / multi-window / TextField / IME / AccessKit / Mica /
  DPI / showcase / sizing spike disposition）は削らない。revision は
  追加・具体化の方向のみを想定する。
- sizing spike の振り分け記録（デフォルト M4）は plan.md 策定（§1.4）で
  phase 割り当てとともに確定する。

## docs/notes open question の扱い

- **intake ケース 1〜6 の分類**（thumbnail click / click-away / generic
  hit handler / focus trap / keyboard close / event routing）は target app
  採択後に本 framing の revisions で確定する。分類枠は intake note の
  規定どおり `M4 scope / M4 explicit defer / M5+ / 1.0 required`。
  ただし ケース 6（event routing model）は app に依らず M4 scope
  （focus model の土台）である見込みが強い。
- **`TypedValue` の `ABI-bearing: unknown` 解消**（abi_spec 照合 —
  ホストが計算済み値を set/get するか）を M4 期間中の宿題として置く。
  B3 採択時は並行配列圧力の点検と併せて前倒しする。
- **release distribution の narrow slice**: M4 の contributor outreach が
  必要とする最小回答は「contributor が showcase を入手・実行する手順」
  であり、clone + build 手順の整備で足りる見込み。artifact 配布・署名・
  channel 等の note 本体は開かない（hold 維持）。

## M4 に入れないもの

- **Full theming surface**（light/dark、accent の widget 伝播、type ramp）
  — M5。M4 は root backdrop + accent follow-through の initial まで。
- **Official widget set**（CheckBox / ComboBox / Menu 等）— M5。
  top-layer overlays の take は「overlay 面の設計と最小 proof」であり、
  Menu / ComboBox という widget 製品面の完成を意味しない。
- **`TypedValue` / structured item data**（M-expr2b/3）— hold。
  独立 pre-1.0 slot の判断は M5 close の判断点（DD-V-028 Out of scope）。
- **themed-widget 背景色 / reactive `fill`** — hold（M5 theming /
  TypedValue 設計空間）。
- **developer debug support** — hold（M5 tooling lean）。
- **component extension model** — hold(post-1.0 + M6 freeze check)。
- **release distribution の note 本体**（artifact / channel / signing /
  versioning）— hold。上記 narrow slice のみ。
- **hot reload / アニメーション DSL** — post-1.0（既存 roadmap どおり）。

## M4 で最初に決めるべき問い

1. **target app 採択** — 候補 slate と軸は上記。オーナー加重待ち。
2. **TextField の最小面** — 単一行で閉じるか、複数行（折返し・選択・
   クリップボード）まで含むか。B1 採択なら必須の前提問題。A/B2/B3 なら
   単一行で自然に閉じる。IME（TSF）検証に必要な最小面もここで確定。
3. **event routing model** — capture/target/bubble 三相か target-only +
   high-level signals か（intake ケース 6）。focus model の土台として
   最初期の phase で決める。
4. **multi-window の discharge 形**（特に B2 採択時）— app 内 proof か
   最小 fixture 別建てか。
5. **touch の検証環境** — touch AC をどの環境・どの証拠形で discharge
   するか（[verification-environments.md](../../../docs/notes/verification-environments.md)
   の taxonomy に追記が要るか）。
6. **M3 residual cluster の disposition**（B 系採択時）— M5 carry の
   明示記録。
7. **sizing spike の置き場** — どの phase に置くか（デフォルト M4 実施。
   M5 送りにするなら VDR の求める積極的正当化を記録）。

## Owner-agreed framing decisions

### F1 — M4 は focus-model milestone と読む（2026-07-07 合意）

M4 の中心 deliverable は focus model の一回確定。追加スコープの取り込み
判断基準は「focus model settlement の質を脅かすか」に置く。

### F2 — 取り込みは二層（core / stretch）で行う（2026-07-07 合意）

core = focus model 結合 item の AC 昇格。stretch = 切り離し可能な葉の
target-app feature（AC 非昇格）。stretch の再評価 checkpoint = focus model
中核 ADR の Accepted 時点。

### F3 — 「なるべく M4 で実施」を framing の出発点とする（2026-07-07 合意）

pool の M4 関連 item はオーナー意向により最大限 target app に取り込む
方向で framing する。収まらない場合の escape valve は F2 の二層構造が
提供する（stretch は 1 行 disposition、core は正規 narrowing）。

### F4 — target app は候補比較で採択する（open）

A / B1 / B2 / B3 の slate から、オーナー加重を反映した素読みで採択する。
採択の明文化は `requirements/spec.md`（§1.2 成果物）。**未確定。**

## 初期 phase breakdown 仮説

target app 採択後に記入する（placeholder）。現時点の依存方向の見立てのみ
置く: event routing / focus model（問い 3）が最初期、TextField + IME と
host state boundary がその上、multi-window と top-layer overlays が
focus model 確定後、showcase 統合と AccessKit / Mica / DPI の evidence が
後段。sizing spike は独立で任意の位置に置ける（問い 7）。

## Next step

1. オーナー: 軸 1〜9 の加重と slate の増減 →
   [target-app-wireframes.html](target-app-wireframes.html) に加重後の
   読みを記録。
2. target app 採択 → `requirements/spec.md`（M4 target app pre-doc）で
   明文化 + 本 framing F4 を確定・app-dependent 帰結を revisions で確定。
3. [candidate-pool.md](../../candidate-pool.md) disposition log に着地先
   リンク付きで記入（item count = 12 を併記）。
4. ROADMAP M4 AC revision（core take 4 件の昇格 + app 照らし審査）。
5. `milestone-4/plan.md`（§1.4）で phase breakdown + sizing spike 振り分け
   記録。
