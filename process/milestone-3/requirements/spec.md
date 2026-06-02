---
title: M3 target app pre-doc — Photo Gallery (案 Z)
status: accepted
created: 2026-05-14
accepted: 2026-05-16
related:
  - ROADMAP.md
  - docs/notes/m3/m3-start-framing.md
  - docs/notes/m3/m3-target-app-wireframes.html
  - docs/plans/m2-plan.md
---

# M3 target app pre-doc — Photo Gallery (案 Z)

このノートは M3 の最初の設計対象である **target app / E2E proof** を確定する
owner-agreed pre-doc である。[m3-start-framing.md](framing.md) §F2 / F7
に従い、M3 plan drafting と ROADMAP acceptance の見直しに先立って、
M3 で作る画面・必要 surface・out-of-scope・spec 同期ルールを固定する。

候補比較とワイヤーフレームの一次資料は
[m3-target-app-wireframes.html](target-app-wireframes.html) に置く。
本 pre-doc は HTML を視覚 input として参照し、HTML が exploratory artifact
として持つ「均等加重時の素読み」「オーナー加重を反映した素読み」の上に立つ
「採択」を本ファイル側で明文化する役割を持つ。

---

## 採択

**M3 target app = 案 Z (Photo Gallery)** を採用する。

ワイヤーフレームの全 4 案（X Mail Reader / Y Music Player / Z Photo Gallery /
W File Explorer）から、Z を M3 の visible proof と pre-doc 5 軸審査の対象に固定する。
W (Explorer) は backup ではなく、本採択により候補集合から外れる（後述「採用理由」参照）。

---

## 採用理由

採用理由は次の 3 層からなる。下層から順に上層へ依存する。

### 1. 加重判断レイヤ — Z は加重後の素読みで単独首位

[m3-target-app-wireframes.html](target-app-wireframes.html) の「オーナー加重を反映した
素読み」節に記録した加重後の比較結果より、Z は次の構造で単独首位となる。

- 重み付けが軽い軸（軸 2 執筆量 / 軸 4 射程 / 軸 5 漏出）が Z の固有弱点と一致しており減点されない。
- 重み付けが通常の軸（軸 1 出荷可能性 / 軸 3 M2 拡張可観測性）で Z は最上位（良・強/離散/高）。
- W (Explorer) は軸 3 で Z と同点だが、軸 1 でわずかに劣り（中〜良 vs 良）、二番手に留まる。
- X (Mail) / Y (Music) は重み付けの効いた軸（X: 軸 1 悪、Y: 軸 3 連続/低 再現性）で致命的減点を受け脱落。

加重そのものはオーナーから直接与えられた:

- 軸 1 通常 / 軸 2 低 / 軸 3 通常 / 軸 4 中低 / 軸 5 中低。

### 2. Framing レイヤ — 「実用的な画面構造」を優先する立場の明示

[m3-start-framing.md](framing.md#L41) は M3 を **DSL surface milestone** と読む
立場を取り、その thesis を「実用的な画面構造を DSL で書けるだけの surface を増やし、
その surface を外部読者が参照できる public draft として文書化すること」と置く。
本採択はこの framing に直結する次の立場を表明する:

- 「**実用的な画面構造**」を優先する。Z は WrapPanel + ZStack + 条件レンダリング +
  繰り返し生成 という構成で、現実の photo gallery アプリの骨格を提供する。これは
  Hello Counter を超え、複数 surface が同時に成立する実用画面である ([m3-start-framing.md](framing.md#L348) §F7)。
- Z 固有の弱点であった「**grammar surface（条件レンダリング・繰り返し生成）を M3 で
  前倒し public spec 化する含意**」を許容する。これは加重判断とは独立した、framing 上の
  立場表明である（加重では消えない構造的論点として
  [HTML](target-app-wireframes.html) 加重節末尾に明記）。

### 3. 結論レイヤ — Z 採用の含意

上の 2 層を合わせた結果、M3 thesis の射程は次のように整理される。

- M3 は「**layout primitive + grammar surface**」の二軸構えで thesis 化する。
  単一の primitive list ではなく、layout primitive と grammar surface が同居する DSL を
  M3 で公開する。
- 現 ROADMAP の M3 AC（Grid / ScrollView / List / DSL public draft）は本採択により
  見直し対象となる ([m3-start-framing.md](framing.md#L315) §F2「ROADMAP revision を起こす」)。
  ROADMAP revision の単位と内容は本 pre-doc 末尾「ROADMAP との同期」節で扱う。

---

## 必要 surface

Z の wireframe から逆算した M3 surface は次のとおり。各項目は
[m3-start-framing.md](framing.md#L60) の「M3 pre-doc 成果物 2 / 3 項」に対応する。

状態列の凡例:

- **日付** (YYYY-MM-DD): 当該 surface 仕様が本 pre-doc 上で合意に到達したセッション日付。
- **既存 (M2)**: M2 までに合意済み、本 milestone では break しない。

### Layout primitive

| Primitive | 状態 | 役割 | 検証する thesis |
|---|---|---|---|
| **WrapPanel** | 2026-05-14 | 子要素を主軸方向に並べ、viewport 主軸サイズ超過で副軸方向の次行へ折り返す | 主軸 / 副軸の measure-arrange 二段階解決を持つ layout primitive を DSL で書けること |
| **Grid** | 2026-05-14 | 2D 座標系で子要素を配置、**1 cell 1 child** 制約、row/column sizing (auto / fixed / star) + spanning | 2D layout の measure-arrange (star sizing の他軸依存解決を含む) を DSL で書けること |
| **ZStack** | 2026-05-14 | 子要素を重ね合わせる（lightbox overlay の構造）。z-order は document order | 兄弟要素の Z 順 + 部分透過レイヤを DSL で書けること |
| **ScrollView** | 2026-05-14 (minimal) | 内側 unbounded measure + viewport clip + offset binding。scrollbar widget / wheel handler / drag は M4 へ defer | viewport 概念と content offset を持つ layout primitive を DSL で書けること |
| **Box** | 2026-05-15 | 0 個以上の子 widget を保持する汎用 container。属性として `aspect: <ratio>` (AspectRatio 兼任) と最小限の `fill: <color>` (scrim 用) を持つ | aspect 制約付き container を独立 primitive として DSL で書けること、および Image 未開封下での placeholder 表現 (Box + Text 子) を支えること |
| HStack / VStack | 既存 (M2) | 線形配置、Fill/Shrink sizing policy | M2 で確立済み、本 milestone では break しない |

**AspectRatio surface の扱い** (「保留点の決着」§保留 1 反映): AspectRatio は独立
primitive とせず Box の `aspect: <ratio>` attribute として畳む。thumbnail の正方形固定
(`Box { aspect: 1/1; ... }`) も lightbox 写真の比率固定 (`Box { aspect: 4/3; ... }`) も
Box 上で表現する。

WrapPanel と Grid の measure-arrange 仕様は novel な normative 執筆を要求する。これは
[m3-target-app-wireframes.html](target-app-wireframes.html) の軸 2 評価で「重」と評価された
内訳の主要部分であり、本 pre-doc 採択時点で許容済み（採用理由 §2 参照）。M3 を「DSL surface
milestone + first public spec draft」と framing する以上、spec 執筆量自体はコストではなく
成果物。

**Grid と ZStack の責務境界**: Grid は 2D 座標系での配置、ZStack は overlay (Z 順) の責務。
XAML / WPF が Grid に same-cell overlap を許す慣習は採用しない（XAML に ZStack が無い歴史的
事情によるもので、orthogonal primitive 思想を採る本 milestone では分離する）。SwiftUI の
modern Grid (1 cell 1 child) と Compose の Box (= ZStack) の分離方針と整合する。

**ZStack を Grid same-cell overlap で代替する案は不採用**: HTML 整理
[L928, L1011](target-app-wireframes.html#L928) の「Grid 同一セル overlap で ZStack を代替」
は star sizing と same-cell overlap という Grid の独立 2 機能を conflate した記述で、論理的
には独立。overlay 表現は ZStack 専管とし、Grid spec は z-order を扱わない。

### Grammar surface

| Surface | 状態 | 役割 | 検証する thesis |
|---|---|---|---|
| **条件レンダリング構文** | 2026-05-14 | lightbox open / close のような binding 駆動の present/absent 切り替え | binding が boolean-ish な意味で widget の存否を制御する文法を DSL が持てること |
| **繰り返し生成構文** | 2026-05-14 | gallery item の列挙（コレクション driven） | binding が列を生み、その各要素から widget tree が生成される文法を DSL が持てること |

両 grammar surface は M3 で **public spec として normative に書く**。M4 以降への
syntax reservation で済ませない。

### Widget / content surface

| Surface | 状態 | 役割 | 検証する thesis |
|---|---|---|---|
| **Button `selected` 系 surface** | 2026-05-15 | Tabs 切替 / トグル等で「選択中」を視覚的に区別する surface。bool binding を直接駆動する | bool scalar binding が widget attribute を駆動できることを示す |

具体的な construct (Button attribute `selected: bool` / 独立 `ToggleButton` primitive /
theming binding) のいずれを採るかは M3 phase pre-doc で詰める。本 pre-doc は selected
state surface を M3 で開けることのみ確定する。

Image widget surface および Button content の text 以外への拡張は M3 では開けない
(「保留点の決着」§保留 2 を参照、Out-of-scope §Value / type にも記載)。thumbnail /
lightbox photo / scrim 等の「写真らしき領域」は Box + Text 子要素で placeholder 表現する。

### Binding / value surface

- スカラー型は `i32` + `String` + **`bool`** の 3 種で閉じる。M2 までの範囲 (`i32` +
  `String`) に bool を 3 つ目の scalar として追加する (合意 2026-05-15)。
- bool の M3 採用は条件レンダリング (lightbox open/close 等の binding 駆動 present/absent
  切替) および Button selected state surface の自然な台座となる。bool を入れない場合の
  代替 (String 等価比較 / Option / int truthy / コレクション empty 判定 等) はいずれも
  bool より重く非直交になるため、bool 採用が最小コストの選択肢。
- bool 採用は [m3-start-framing.md](framing.md#L335) §F5 (`TypedValue` 無条件
  導入 defer) を覆さない。`TypedValue` は generic value union 機構であり、bool を 3 つ目
  の scalar として追加することは別議論として扱う。`TypedValue` defer は維持する
  (Out-of-scope §Value / type 参照)。
- 第四 scalar 以降 / generic value union (`TypedValue`) の導入は本 milestone では行わない。

---

## 各 surface が検証する thesis

各 primitive / grammar が M3 thesis に対して何を proof するかを明示する。

- **WrapPanel** — 主軸方向の連続配置と viewport 主軸サイズ超過時の副軸方向 wrap という
  二段階 reflow を持つ layout primitive の measure-arrange を DSL が記述・実装・spec
  化できることを示す。M2 までの線形配置 (HStack / VStack) を超えた non-trivial reflow
  ロジックの M3 spec drafting に対する切れ味として立つ。
- **Grid** — 2D 座標系での measure-arrange (star sizing の他軸依存解決を含む) と
  1 cell 1 child 制約下での spanning を DSL が記述できることを示す。WrapPanel の
  主軸 / 副軸 reflow とは独立した 2D layout 検証として M3 thesis に並列に立つ。
  same-cell overlap は持たず、overlay は ZStack 専管とすることで orthogonal primitive
  思想を proof する。
- **ZStack** — 兄弟要素が重なる layout の意味論を DSL が記述できることを示す。
  Hello Counter までは linear tree しか出現せず、本 primitive は M2 surface の真の拡張になる。
- **ScrollView** (minimal) — 内側 unbounded measure + viewport clip + content offset binding
  を持つ layout primitive を DSL が記述できることを示す。scrollbar widget / wheel handler /
  drag を M4 へ defer した minimal surface であり、proof の射程は「viewport 概念と offset
  binding が DSL / IR / runtime を通る」点に限定されるが、Photo Gallery の thumbnail 数十枚
  シナリオを成立させる構造的必要条件として M3 thesis に位置づく。
- **Box** — 任意の子要素を保持する汎用 container と aspect 制約を兼ねる layout primitive を
  DSL が記述できることを示す。AspectRatio を独立 primitive とせず attribute として畳む
  設計判断、および Image widget を M4 へ defer した上で placeholder 表現 (Box + Text 子)
  を成立させる構造の双方を proof する。
- **条件レンダリング** — binding が widget tree の構造そのもの（subtree の存否）を駆動できる
  ことを示す。M2 の binding は property 値の駆動のみだったため、本構文は M2 不可能性として
  ([m3-target-app-wireframes.html](target-app-wireframes.html) 軸 3 評価で「離散 / 高」)
  明確に観察される。
- **繰り返し生成** — binding が widget tree の **数** そのものを駆動できることを示す。
  コレクション binding の M3 plan に直結し、後続 M4 以降の動的 UI surface
  （filter, sort, virtualization）の foundation になる。
- **bool scalar binding** — i32 / String に並ぶ第三の scalar として、widget tree の subtree
  存否 (条件レンダリング) と widget attribute (Button selected state) の双方を駆動できる
  ことを示す。`TypedValue` 機構 (generic value union) を導入せずに scalar 拡張のみで
  grammar surface と widget surface の両方を支えられることが本 milestone の thesis 検証の
  一部となる。

---

## Out-of-scope

Z 採択により、wireframe 上には存在するが M3 surface としては明示的に扱わない項目を
固定する。[m3-target-app-wireframes.html](target-app-wireframes.html) 軸 5（M4–M5 漏出）
評価で「回避可」とした項目を中心に出す。

### Visual / styling

- **Tabs / Button selected state の visual styling 詳細** — selected state surface 自体は
  M3 で採用する (「必要 surface」§Widget / content surface 参照) が、selected 時の
  具体的な visual 表現 (border / background / typography 強調など) は M3 では装飾なし
  最小表現に落とす。具体形は M3 phase pre-doc で詰める。
- **scrim（lightbox 背景の半透明黒幕）の opacity** — lightbox は ZStack による overlay として
  描き、scrim の alpha 値 styling は M3 では扱わない。背景 dim が必要なら不透明 fill で代替する。
- **Breadcrumb 風の Button-not-Button styling** — そもそも Z には breadcrumb がないので
  本項は確認的記述（W 比較からの継承）。

### Interaction

- **Splitter drag** — Z には Splitter がなく、構造的に発生しない（これが Z の軸 5 強みの一つ）。
  M3 全体としても drag による layout resize は扱わない。
- **lightbox の close / prev / next ジェスチャ** — 該当 interaction は Button click の handler
  binding で表現する。swipe / pinch / keyboard shortcut は M3 では扱わない。
- **hit-testing / focus capture / modal focus trap** — lightbox は構造上 modal-ish だが、
  M3 では focus model に踏み込まない（[m3-start-framing.md](framing.md#L264) §「M3 に入れないもの」
  の input / focus model defer と整合）。

### Value / type

- **第四 scalar 以降 / generic value union (`TypedValue`)** — bool は M3 で採用する
  (「必要 surface」§Binding / value surface 参照) が、`TypedValue` 機構 (generic value
  union) の無条件導入は M3 では行わない ([m3-start-framing.md](framing.md#L335)
  §F5)。本 target app は `TypedValue` 圧力を構造的に避ける設計に寄せる。
- **Image widget surface** — 画像 / アイコン用の widget primitive は M3 では開けない
  (「保留点の決着」§保留 2 参照)。thumbnail / lightbox photo / scrim 等の「写真らしき
  領域」は Box + Text 子要素で placeholder 表現する。asset pipeline / icon font / image
  decoder の surface 化は M4 以降。
- **Button content の text 以外への拡張** — Image widget の M3 defer に従い、Button content
  は text 属性のみで閉じる (`Button { text: "×" }` 等)。Image / 任意 widget を Button content
  に入れる surface は M3 では開けない。

### Platform

- **focus / AccessKit / multi-window / hot reload / C ABI** — いずれも Z は要求しない
  ([m3-target-app-wireframes.html](target-app-wireframes.html) 軸 5 platform 漏出評価より）。
  これらは [m3-start-framing.md](framing.md#L255) §「M3 に入れないもの」で
  既に defer されている。

---

## spec / implementation / E2E proof の同期ルール

[m3-start-framing.md](framing.md#L341) §F6 に従い、本 target app の実装過程では
次の同期規律を維持する。

- **同一 phase 内同期**: 各 M3 phase で、`.ui` （DSL）、`wasamo-ir`、`wasamoc` emitter、
  `wasamo-runtime` loader / layout、`docs/dsl_spec.md` を **同じ phase 内で同期させる**。
  片側だけが先行する状態を phase 境界をまたいで残さない。
- **target app は acceptance 判定基準**: 本 pre-doc で採択した Z gallery が、各 phase の
  完了判定で `.ui -> IR -> runtime` の path を実際に通る最小ケースとして機能する。
  phase 別の sub-screen / 部分機能を E2E proof として置き、最終 phase で full app として
  閉じる。
- **spec drafting は副産物ではない**: WrapPanel measure-arrange と grammar surface（条件
  レンダリング・繰り返し生成）は normative spec の対象として、各 phase 完了時点で
  `docs/dsl_spec.md` に反映済みであることを要求する。M3 最終 phase でまとめて spec を書く
  運用にしない。
- **視覚 input**: M3 plan / phase pre-doc / dsl_spec drafting / E2E proof acceptance の視覚
  input は [docs/references/m3-gallery-wireframe.html](../../references/m3-gallery-wireframe.html)
  を参照する (採択済み Photo Gallery 視覚仕様を英語ラベルで固定した stable artifact)。元の
  探索アーティファクト [m3-target-app-wireframes.html](target-app-wireframes.html) は
  候補比較 / 加重判断記録 / 語彙批判的検討の歴史的記録として `docs/notes/` 配下に保持し、
  抽出版から "Source exploratory note" pointer で参照される。詳細は本ファイル末尾
  「視覚 artifacts との参照関係」節を参照。

---

## ROADMAP との同期

本採択により、現 ROADMAP の M3 acceptance criteria（Grid / ScrollView / List /
DSL public draft）は見直し対象となる。[m3-start-framing.md](framing.md#L315) §F2
の「必要なら ROADMAP revision を起こす」がここで具体化する。

### 現 ROADMAP M3 AC と Z 採択の差分

| 項 | 現 ROADMAP | Z 採択後 (pre-doc accepted 2026-05-16) |
|---|---|---|
| Layout primitive A | Grid layout primitive | **Grid** layout primitive (1 cell 1 child、star sizing + spanning、same-cell overlap は持たない) |
| Layout primitive B | ScrollView primitive | **ScrollView** layout primitive (minimal: clip + offset binding。scrollbar widget / wheel handler / drag は M4 へ defer) |
| Layout primitive C | List primitive | **WrapPanel** + **ZStack** + **繰り返し生成 grammar**（List の責務をこの 3 surface へ分解） |
| Layout primitive D | — | **Box** layout primitive (0+ 子 widget container、`aspect: <ratio>` attribute で AspectRatio 兼任、最小限の `fill: <color>` 属性) |
| Layout primitive 追加 | — | (上記合算で 5 layout primitive: Grid / WrapPanel / ZStack / ScrollView / Box。M2 既存の HStack / VStack は並存維持) |
| Scalar type | — (i32 + String) | **`bool` を 3 つ目の scalar として追加** (`i32` + `String` + `bool`)。`TypedValue` 機構の導入は引き続き defer |
| Widget surface | — | **Button selected state surface** を M3 で開封 (具体形は M3 phase pre-doc で確定) |
| Spec | DSL specification first public draft | DSL specification first public draft（**Grid / WrapPanel / ZStack / ScrollView / Box の normative spec + 条件レンダリング + 繰り返し生成 grammar + bool scalar + Button selected state surface を含む**） |

注: 旧案では「Grid を削除し WrapPanel で置換 / ScrollView を完全 defer / ZStack を独立
primitive 化」を検討したが、2026-05-14 セッションで以下が確認された:

- Grid は語彙の普遍性（XAML / MUI / Compose / Slint / QML / CSS Grid 全部にある）と表現力
  (star sizing) の点で M3 layout primitive set から外すと M3 thesis を弱める。
- ScrollView は WrapPanel の wrap だけでは副軸方向 overflow を扱えず、Photo Gallery の
  実用要件 (thumbnail 数十枚〜) に届かない。完全 defer は破綻するため minimal surface で残す。
- ZStack は overlay 専管として独立必要。Grid same-cell overlap での代替は star sizing と
  same-cell overlap を conflate した記述で、論理的に独立。

2026-05-15 セッションで追加:

- Box は AspectRatio (保留 1) と Image widget defer (保留 2) の解消過程で追加された。
  `aspect` 属性の置き場として AspectRatio を吸収し、Image を M4 へ defer する代わりに
  Box + Text 子要素による placeholder 表現 (β1) を担う。
- bool を 3 つ目の scalar として追加することで、条件レンダリングと Button selected state の
  両方が `TypedValue` 機構を導入せずに表現できるようになる。F5 (`TypedValue` defer) は
  維持。

### ROADMAP revision の単位

ROADMAP revision は次の単位で起こす想定:

1. 上表の差分そのものを M3 AC の revision として記述する。
2. revision の根拠として本 pre-doc と
   [m3-target-app-wireframes.html](target-app-wireframes.html) を参照する。
3. revision の意思決定単位は vision decision に届くか pre-doc レベルで閉じるかを別途判断する
   ([m3-start-framing.md](framing.md#L322) §F3 の判定基準: public draft 誠実性 /
   visible proof 必要性 / grammar・IR 破壊的変更リスクのいずれか）。

ROADMAP revision の具体的執筆作業は本 pre-doc の owner approval を経た後に着手する。

---

## 視覚 artifacts との参照関係

本 pre-doc は 2 つの視覚 artifact と参照関係を持つ。役割は明確に分かれる。

### 抽出版 — `docs/references/m3-gallery-wireframe.html` (forward reference)

- 採択済み Photo Gallery 視覚仕様のみを英語ラベルで固定した stable artifact。
- 候補比較 / 加重判断 / 日本語の批判的検討は持ち込まない。本 pre-doc が SSOT (採択判断 / 採用
  理由 / 必要 surface / out-of-scope / spec 同期ルール / ROADMAP との同期) として残り、抽出版は
  対応する視覚仕様部分のみを担う。
- M3 plan / phase pre-doc / dsl_spec drafting / E2E proof acceptance から視覚 input として参照
  される。
- 抽出版から元の探索ノートへ "Source exploratory note" pointer リンクを置き、歴史的議論経緯への
  遡及参照を残す。
- `docs/references/` カテゴリは新設。CLAUDE.md「Document categories under `docs/`」節に
  durable supporting reference artifacts の置き場所として狭く定義し、decisions や execution
  tracker は置かず、extracted で stable な artifact を期待する旨を追記する。

### 元の探索ノート — `docs/notes/m3/m3-target-app-wireframes.html` (historical record)

- **exploratory artifact**（候補比較 X/Y/Z/W + 4 wireframe + 語彙批判的検討 + 加重判断記録）
  として `docs/notes/` 配下に維持する。本 pre-doc 採択により内容を縮小再構成しない。
- 本 pre-doc の採択により、HTML 末尾「位置づけと将来再構成トリガー」節
  ([m3-target-app-wireframes.html L1470-L1490](target-app-wireframes.html#L1470-L1490)) で
  予告されていた再構成 trigger が **発火する**。発火形は予告された「wireframe-only + 語彙整理
  markdown 抽出」全面再構成ではなく、採択された Z (Gallery) wireframe のみを `docs/references/`
  へ抽出する **部分的再構成** とする。X / Y / W wireframe および候補比較 / 加重判断記録 / 語彙
  批判的検討は元 HTML にそのまま残す。
- 元 HTML の「採択」記述は本 pre-doc を pointer 参照で指す状態のままにする（HTML 側に採択結論
  を duplicate しない、本 pre-doc が SSOT）。

---

## Next step

本 pre-doc が owner approval を得たら、以下を進める。

1. 本ファイルの `status:` を `drafting` から `accepted` へ更新する。
2. **`docs/references/` カテゴリ新設と Gallery wireframe 抽出** (本 pre-doc「視覚 artifacts
   との参照関係」節に基づく):
   - CLAUDE.md「Document categories under `docs/`」節に `docs/references/` の役割を狭く
     定義して追記する (durable supporting reference artifacts の置き場所、decisions や
     execution tracker は置かない、extracted で stable な artifact を期待、等)。ファイル数が
     増えてきた段階で `docs/references/m3/` のようなサブディレクトリ化を検討する旨も併記。
   - `docs/references/m3-gallery-wireframe.html` を作成 (採択 Z wireframe のみ、英語ラベル、
     候補比較 / 日本語議論は持ち込まない、`docs/notes/m3/m3-target-app-wireframes.html` への
     "Source exploratory note" pointer リンクを置く)。
3. **ROADMAP M3 acceptance の revision** を起こす (本 pre-doc「ROADMAP との同期」節の差分を
   反映)。2 と独立、並行可。
4. **`docs/plans/m3-plan.md` を `status: drafting` で作成** し、phase breakdown を本採択に
   基づき drafting する。2 と 3 の両方が完了していると plan が抽出版 wireframe と revised AC
   を直接参照できて望ましいが、厳格な依存ではない。
5. M3 最初の implementation phase の pre-doc を開く前に、本ファイル / 抽出版 wireframe /
   revised ROADMAP / m3-plan の owner agreement を確認する。

---

## 保留点の決着

2026-05-14 セッションで開いた 3 保留は 2026-05-15 セッションで決着した。決着内容と
反映先を下に記録する。詳細議論経緯は本 pre-doc 上に残さず、決着の事実と該当節への
pointer のみ保持する。

### 保留 1 closure: AspectRatio surface の形式 → (b) Box attribute 化

AspectRatio は独立 primitive とせず、Box の `aspect: <ratio>` attribute として畳む。

- **反映先**: 「必要 surface」§Layout primitive (Box row + 直下の AspectRatio 補足段落)、
  「各 surface が検証する thesis」§Box、「ROADMAP との同期」§差分表 Layout primitive D。
- **連動**: 保留 2 closure と一体。Image を defer する代わりに Box が aspect 属性と
  placeholder container の両方を担う。

### 保留 2 closure: Image widget surface の M3 開封可否 → 不開封 (M4 へ defer)

Image widget surface は M3 では開けない。thumbnail / lightbox photo / scrim 等の
「写真らしき領域」は **Box + Text 子要素 (β1 形式)** で placeholder 表現する。Button
content は text 属性のみで閉じる。

- **反映先**: 「必要 surface」§Layout primitive (Box row)、「必要 surface」§Widget /
  content surface (Image / Button content 拡張は非採用)、「Out-of-scope」§Value / type
  (Image widget / Button content 拡張)、「ROADMAP との同期」§差分表 Layout primitive D。
- **採用しなかった代替案**: (i) Image widget を rendering 抜き surface として M3 開封、
  (β2) Box に text 属性を持たせる、(β3) Box が positional string content を取る。いずれも
  surface 圧力増 / 直交性低下のため不採用。

### 保留 3 closure: Tabs / Button 選択状態 surface → 採用、bool を 3 つ目の scalar として導入

Button selected state surface を M3 で開ける。bool を `i32` / `String` に並ぶ 3 つ目の
scalar として導入し、selected state は bool binding で表現する。`TypedValue` 機構の
無条件導入は引き続き defer (F5 維持)。selected 時の具体的 visual 表現 (border /
background / typography 強調等) は M3 では装飾なし最小表現に落とし、具体形は M3 phase
pre-doc で詰める。

- **反映先**: 「必要 surface」§Binding / value surface (bool 採用)、「必要 surface」
  §Widget / content surface (Button selected state surface)、「各 surface が検証する
  thesis」§bool scalar binding、「Out-of-scope」§Visual / styling (Tabs / Button selected
  state styling 詳細)、「Out-of-scope」§Value / type (`TypedValue` defer は維持)、
  「ROADMAP との同期」§差分表 Scalar type 行 / Widget surface 行。
- **F5 整合**: F5 で defer したのは `TypedValue` (generic value union) 機構であって、
  bool を 3 つ目の scalar として追加することはこれと別議論。F5 は維持。
- **bool 不採用で条件レンダリングをどう実現するつもりだったか** (owner 質問への回答):
  旧記述 ("bool 値 binding を要求しないかたちで grammar 設計する") は under-specified
  だった。代替 (String 等価比較 / Option / int truthy / コレクション empty 判定) は
  いずれも bool より重く非直交になるため、bool 採用が grammar / widget 双方で最小コストの
  選択肢として確認された。

