---
title: 開発者デバッグ支援 — 検討メモと未解決事項
status: live
created: 2026-05-28
last-updated: 2026-05-28
related:
  - docs/notes/layout-engine.md
  - docs/notes/dsl-grammar.md
  - docs/notes/human-visible-smoke.md
  - process/milestone-3/phase-5/requirements/framing.md
---

# 開発者デバッグ支援 — 検討メモと未解決事項

Wasamo で「先行 UI フレームワークの debug mode に近いこと」をやりたい、
という前提の live note。まだ決定は無く、論点の棚卸しと Open Questions を
残すための出発点。確定したら ADR へ蒸留し、ここからは外す。

---

## 1. 背景・動機

直接のきっかけは M3-Phase 5 Grid pre-doc の FD-D
([framing.md](../../process/milestone-3/phase-5/requirements/framing.md))。
non-root の Shrink container が Fill 子を持つと、Fill 子は高さ 0 に
潰れる(`degenerate_fill_in_shrink_parent_clamps_to_zero`)。これは
**決定的だが silent** な挙動で、作者から見ると「Grid が出ているはずなのに
画面に何も出ない」という形でしか観測できない。Phase 4 T6 の ScrollView
failure mode A も同型の罠だった。

より一般化すると、Wasamo には今のところ **開発者向けの debug mode が無い**。
一方で主要 UI フレームワークはどれも、レイアウト異常・再描画・ツリー構造を
可視化する debug 機構を持つ。Wasamo でも相当のものが欲しい、というのが
この note の前提。

重要な切り分け: FD-D の **意味論**(潰すか/潰さないか)は別問題で、現状維持
(silent collapse のまま)で確定している。本 note が扱うのは「その silent な
事象を **開発時に観測・検出できる手段**」という観測性 (observability) の軸。
両者は直交する。

---

## 2. 先行 UI フレームワークのデバッグ支援(棚卸し)

カテゴリ別に整理する。Wasamo に効きそうな順ではなく、概念別。

| カテゴリ | 代表例 | 何をするか |
|---|---|---|
| **レイアウト異常の警告** | Flutter の overflow 黄黒ストライプ + console `RenderFlex overflowed by N px` / `unbounded constraints` assertion | overflow・unbounded・0 潰れ等の異常を debug build で明示 |
| **Visual/Layout tree インスペクタ** | WPF Live Visual Tree / Snoop、Android Layout Inspector、Compose Layout Inspector、ブラウザ DevTools の Elements | 実行中のツリーと各ノードの実サイズ・属性を覗く |
| **レイアウト境界の可視化** | Flutter `debugPaintSizeEnabled`、Android「レイアウト境界を表示」開発者オプション、CSS の `outline` デバッグ | 各ノードの矩形・余白・baseline を画面に重ねて描く |
| **再描画/再レイアウトの可視化** | Flutter repaint rainbow / performance overlay、React DevTools highlight updates、Compose recomposition counts | どこが・何回 再計算されたかを色や数で見せる |
| **ログ/トレース** | `tracing`、Chrome DevTools / Flutter DevTools timeline、Android systrace | フレーム単位の計測・イベント列をタイムラインで追う |
| **静的診断 (lint)** | ESLint/stylelint、各種 IDE inspection | 実行前にソース上の問題箇所を warning で指摘 |
| **ホットリロード** | Flutter hot reload、各種 HMR | デバッグというより dev ループ短縮(隣接領域) |

観察: フレームワークはほぼ例外なく **「debug build でだけ noisy にし、
release build は静かに保つ」** という分離をしている。production 挙動
(= Wasamo の場合 silent collapse)は変えず、debug 時だけ検出を足す、という
のが定石。

---

## 3. Wasamo のアーキ上、どこで何ができそうか

Wasamo の経路は `.ui` → (wasamoc) → IR → (runtime) → Compositor/Visual Layer。
検出ポイントは大きく 3 つ。

1. **コンパイル時 (`wasamoc check`)**
   - 既に `Severity::Warning` + `filename:line:col` 付きの診断基盤がある
     ([wasamoc/src/diagnostic.rs](../../wasamoc/src/diagnostic.rs))。
   - 構造的に静的検出できる異常(例: Fill 子を持つ非 root Shrink container)は、
     既存の warning 基盤に lint として載せられる可能性が高い。**ソース位置を
     指せる**のが最大の強み。
   - 限界: 実際のサイズは実行時の window サイズ依存なので、「潰れる**かもしれない**」
     までしか言えない構造もある。

2. **実行時 (debug build の runtime)**
   - レイアウト結果(実サイズ 0、unbounded 検出など)は runtime でしか分からない。
   - 限界: 現状 IR (`wasamo-ir`) は **ソース span を持たない**ため、runtime 警告は
     「どの `.ui` 行か」を指せない。指したいなら IR に span を流す設計判断が要る
     (dsl-grammar の diagnostics 論点と接続)。

3. **Compositor / Visual Layer**
   - レイアウト境界の可視化 (overlay 描画) はここ。Flutter の
     `debugPaintSizeEnabled` 相当。Visual を 1 層足す形になりうる。

---

## 4. Open Questions

### OQ1. レイアウト潰れ (degenerate size) をどう検出するか

FD-D の silent 0-collapse が直接の対象。candidate:

- **(a) `wasamoc check` 静的 lint**: 「非 root の Shrink container が Fill 子を
  持つ」を構造的に warning。ソース位置を指せる。ただし root 判定(component root の
  Fill/Fill 上書き)と、実行時サイズ依存で必ずしも潰れないケースの扱いが論点。
- **(b) runtime debug 警告**: レイアウト後に size 0(かつ Fill 制約)を検出して
  log/stderr。実サイズに基づくので誤検出が少ない代わりに `.ui` 位置を指せない
  (OQ5/OQ6)。
- **(c) 視覚オーバーレイ**: 潰れたノードの想定矩形を debug 描画。
- 組合せも可。どれを最初に入れるか。

### OQ2. ∞ と 0 の非対称をどう扱うか

DD-M3-P5-004 で star の **unbounded (∞)** は **error** に倒した。一方 FD-D の
**0 collapse** は silent。同じ「definite bounded space が無い」事象なのに
扱いが割れている。debug 検出を入れると、production 挙動は変えずに「0 の側も
debug では見える」状態にできて、非対称を実務上やわらげられる。これを狙うか、
それとも将来 layout DD で意味論ごと揃える(0 も error 化)方向を別途検討するか。

### OQ3. ツリーインスペクションの手段

実行中の layout/visual tree を覗く手段が無い。candidate: tree dump (テキスト/
JSON を stderr やファイルへ) / 外部インスペクタ / IDE 連携。Wasamo は独自 IR +
Compositor なので既製インスペクタは流用できず、最小は **dump 機能**になりそう。
どの粒度(IR ノード / LayoutNode / Visual)で、いつ(毎フレーム / オンデマンド)
出すか。

### OQ4. debug build と release build の分離方針

「release は silent のまま、debug でのみ検出」を貫くための仕組み。
`cfg(debug_assertions)` / Cargo feature / 環境変数のどれを軸にするか。検出を
runtime に入れる場合、release の hot path にコストを残さない設計が要る。

### OQ5. ホスト言語をまたぐ debug 情報の届け方

Wasamo host は C / Zig / Rust。runtime の警告をどの経路でホスト開発者に届けるか。
stderr 直書き / ログコールバックを ABI に足す / debug build 限定の診断 API。
ABI 表面を増やすかどうかが論点(M3 は host-facing ABI を増やさない方針と整合
させる必要)。

### OQ6. `.ui` ソース位置を runtime まで運ぶか

runtime 警告に「`gallery.ui:42` の Grid が潰れた」と書きたいなら、IR に source
span を載せる必要がある。現状 IR は span を持たない。span を足すと IR サイズ・
ABI・lowering に波及する。debug build だけ span を載せる二系統 IR も選択肢。
dsl-grammar の token-level diagnostics 論点と一体で考える。

### OQ7. 検出対象の射程

レイアウト潰れ以外にも debug で見たい事象がある(overflow のはみ出し量、
unbounded、再レイアウト回数、star 解決結果、z-order 衝突など)。最初の 1 つを
どこに絞り、拡張可能な枠だけ先に決めるか。

---

## 5. 当面のスコープ感

- これは **Grid 固有でも Phase 5 thesis でもない**、横断的な layout/tooling 関心。
  FD-D を Grid に閉じて例外則化しないのと同じ理由で、debug 支援も Grid に隠さず
  独立の subsystem として育てる。
- FD-D の **意味論は (1) 現状維持**で確定。本 note はその silent 事象を将来
  観測可能にするための設計の種であって、Phase 5 の意味論を変えるものではない。
- 最小の一歩を選ぶなら、既存の `wasamoc check` warning 基盤に載る **静的 lint
  (OQ1-a)** が、新インフラ最小・ソース位置を指せる、という点で入口として有力。
  runtime/overlay/インスペクタはその後の段階。
- いずれ着手する際は、どの milestone/phase が owning するか、release への
  コスト境界、ABI を増やすか、を ADR で確定する。

---

## 6. 関連ノート

- [layout-engine.md](./layout-engine.md) — 2-pass measure/arrange、cache
  invalidation、collapse 挙動の背景。
- [dsl-grammar.md](./dsl-grammar.md) — diagnostics / source span を IR まで
  運ぶ論点 (OQ6)。
- [human-visible-smoke.md](./human-visible-smoke.md) — 自動検証と owner-visible
  GUI smoke の分離。debug 支援が入っても両者の役割分担は残る。
- [framing.md (M3-Phase 5)](../../process/milestone-3/phase-5/requirements/framing.md)
  — FD-D(non-root Shrink × Fill 子)と DD-M3-P5-004(unbounded-star error)。
