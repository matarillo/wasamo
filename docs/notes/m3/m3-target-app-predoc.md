---
title: M3 target app pre-doc — Photo Gallery (案 Z)
status: drafting
created: 2026-05-14
related:
  - ROADMAP.md
  - docs/notes/m3/m3-start-framing.md
  - docs/notes/m3/m3-target-app-wireframes.html
  - docs/plans/m2-plan.md
---

# M3 target app pre-doc — Photo Gallery (案 Z)

このノートは M3 の最初の設計対象である **target app / E2E proof** を確定する
owner-agreed pre-doc である。[m3-start-framing.md](m3-start-framing.md) §F2 / F7
に従い、M3 plan drafting と ROADMAP acceptance の見直しに先立って、
M3 で作る画面・必要 surface・out-of-scope・spec 同期ルールを固定する。

候補比較とワイヤーフレームの一次資料は
[m3-target-app-wireframes.html](m3-target-app-wireframes.html) に置く。
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

[m3-target-app-wireframes.html](m3-target-app-wireframes.html) の「オーナー加重を反映した
素読み」節に記録した加重後の比較結果より、Z は次の構造で単独首位となる。

- 重み付けが軽い軸（軸 2 執筆量 / 軸 4 射程 / 軸 5 漏出）が Z の固有弱点と一致しており減点されない。
- 重み付けが通常の軸（軸 1 出荷可能性 / 軸 3 M2 拡張可観測性）で Z は最上位（良・強/離散/高）。
- W (Explorer) は軸 3 で Z と同点だが、軸 1 でわずかに劣り（中〜良 vs 良）、二番手に留まる。
- X (Mail) / Y (Music) は重み付けの効いた軸（X: 軸 1 悪、Y: 軸 3 連続/低 再現性）で致命的減点を受け脱落。

加重そのものはオーナーから直接与えられた:

- 軸 1 通常 / 軸 2 低 / 軸 3 通常 / 軸 4 中低 / 軸 5 中低。

### 2. Framing レイヤ — 「実用的な画面構造」を優先する立場の明示

[m3-start-framing.md](m3-start-framing.md#L41) は M3 を **DSL surface milestone** と読む
立場を取り、その thesis を「実用的な画面構造を DSL で書けるだけの surface を増やし、
その surface を外部読者が参照できる public draft として文書化すること」と置く。
本採択はこの framing に直結する次の立場を表明する:

- 「**実用的な画面構造**」を優先する。Z は WrapPanel + ZStack + 条件レンダリング +
  繰り返し生成 という構成で、現実の photo gallery アプリの骨格を提供する。これは
  Hello Counter を超え、複数 surface が同時に成立する実用画面である ([m3-start-framing.md](m3-start-framing.md#L348) §F7)。
- Z 固有の弱点であった「**grammar surface（条件レンダリング・繰り返し生成）を M3 で
  前倒し public spec 化する含意**」を許容する。これは加重判断とは独立した、framing 上の
  立場表明である（加重では消えない構造的論点として
  [HTML](m3-target-app-wireframes.html) 加重節末尾に明記）。

### 3. 結論レイヤ — Z 採用の含意

上の 2 層を合わせた結果、M3 thesis の射程は次のように整理される。

- M3 は「**layout primitive + grammar surface**」の二軸構えで thesis 化する。
  単一の primitive list ではなく、layout primitive と grammar surface が同居する DSL を
  M3 で公開する。
- 現 ROADMAP の M3 AC（Grid / ScrollView / List / DSL public draft）は本採択により
  見直し対象となる ([m3-start-framing.md](m3-start-framing.md#L315) §F2「ROADMAP revision を起こす」)。
  ROADMAP revision の単位と内容は本 pre-doc 末尾「ROADMAP との同期」節で扱う。

---

## 必要 surface

Z の wireframe から逆算した M3 surface は次のとおり。各項目は
[m3-start-framing.md](m3-start-framing.md#L60) の「M3 pre-doc 成果物 2 / 3 項」に対応する。

### Layout primitive

| Primitive | 役割 | 検証する thesis |
|---|---|---|
| **WrapPanel** | 子要素を主軸方向に並べ、viewport 主軸サイズ超過で副軸方向の次行へ折り返す | 主軸 / 副軸の measure-arrange 二段階解決を持つ layout primitive を DSL で書けること |
| **ZStack** | 子要素を重ね合わせる（lightbox overlay の構造） | 兄弟要素の Z 順 + 部分透過レイヤを DSL で書けること |

WrapPanel の measure-arrange 仕様は novel な normative 執筆を要求する。これは
[m3-target-app-wireframes.html](m3-target-app-wireframes.html) の軸 2 評価で「重」と評価された
内訳の主要部分であり、本 pre-doc 採択時点で許容済み（採用理由 §2 参照）。

### Grammar surface

| Surface | 役割 | 検証する thesis |
|---|---|---|
| **条件レンダリング構文** | lightbox open / close のような binding 駆動の present/absent 切り替え | binding が boolean-ish な意味で widget の存否を制御する文法を DSL が持てること |
| **繰り返し生成構文** | gallery item の列挙（コレクション driven） | binding が列を生み、その各要素から widget tree が生成される文法を DSL が持てること |

両 grammar surface は M3 で **public spec として normative に書く**。M4 以降への
syntax reservation で済ませない。

### Binding / value surface

- スカラー型は M2 までの範囲（`i32` + `String`）で閉じる。
- 第三 scalar 型（特に `bool`）は本採択では導入しない。条件レンダリングは
  bool 値 binding を要求しないかたちで grammar 設計する（具体的な構文選択は
  M3 phase 単位の pre-doc で決める）。これは
  [m3-start-framing.md](m3-start-framing.md#L335) §F5（`TypedValue` 無条件導入 defer）と整合する。

---

## 各 surface が検証する thesis

各 primitive / grammar が M3 thesis に対して何を proof するかを明示する。

- **WrapPanel** — primitive の自前 measure-arrange を DSL が記述・実装・spec 化できることを示す。
  Grid（線形 row/column）よりも主軸 / 副軸の reflow ロジックが非自明であり、
  layout primitive の M3 spec drafting に対する切れ味として強い。
- **ZStack** — 兄弟要素が重なる layout の意味論を DSL が記述できることを示す。
  Hello Counter までは linear tree しか出現せず、本 primitive は M2 surface の真の拡張になる。
- **条件レンダリング** — binding が widget tree の構造そのもの（subtree の存否）を駆動できる
  ことを示す。M2 の binding は property 値の駆動のみだったため、本構文は M2 不可能性として
  ([m3-target-app-wireframes.html](m3-target-app-wireframes.html) 軸 3 評価で「離散 / 高」)
  明確に観察される。
- **繰り返し生成** — binding が widget tree の **数** そのものを駆動できることを示す。
  コレクション binding の M3 plan に直結し、後続 M4 以降の動的 UI surface
  （filter, sort, virtualization）の foundation になる。

---

## Out-of-scope

Z 採択により、wireframe 上には存在するが M3 surface としては明示的に扱わない項目を
固定する。[m3-target-app-wireframes.html](m3-target-app-wireframes.html) 軸 5（M4–M5 漏出）
評価で「回避可」とした項目を中心に出す。

### Visual / styling

- **Tabs の selected styling** — wireframe では gallery / album 等の切替に Tabs を描いているが、
  選択中タブの装飾 styling は M3 では扱わない。Tabs そのものを描く場合も装飾なしの
  最小表現に落とす（あるいは Tabs 自体を落とす wireframe バリアントを採用する）。
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
  M3 では focus model に踏み込まない（[m3-start-framing.md](m3-start-framing.md#L264) §「M3 に入れないもの」
  の input / focus model defer と整合）。

### Value / type

- **bool binding** — 上記「必要 surface」§Binding 通り、本採択では bool 型を導入しない。
  条件レンダリングは bool 値 binding に依存しないかたちで grammar 設計する。
- **第三 scalar 型一般** — `TypedValue` 無条件導入は defer
  ([m3-start-framing.md](m3-start-framing.md#L335) §F5)。本 target app は `TypedValue`
  圧力を構造的に避ける設計に寄せる。

### Platform

- **focus / AccessKit / multi-window / hot reload / C ABI** — いずれも Z は要求しない
  ([m3-target-app-wireframes.html](m3-target-app-wireframes.html) 軸 5 platform 漏出評価より）。
  これらは [m3-start-framing.md](m3-start-framing.md#L255) §「M3 に入れないもの」で
  既に defer されている。

---

## spec / implementation / E2E proof の同期ルール

[m3-start-framing.md](m3-start-framing.md#L341) §F6 に従い、本 target app の実装過程では
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
- **HTML pre-doc 視覚 input**: [m3-target-app-wireframes.html](m3-target-app-wireframes.html) は
  wireframe / 候補比較 artifact として M3 plan / phase pre-doc から視覚 input に参照される。
  pre-doc が HTML 内容を上書き・更新する関係ではない（HTML は exploratory artifact のまま
  維持する。本ファイル末尾「HTML との参照関係」節も参照）。

---

## ROADMAP との同期

本採択により、現 ROADMAP の M3 acceptance criteria（Grid / ScrollView / List /
DSL public draft）は見直し対象となる。[m3-start-framing.md](m3-start-framing.md#L315) §F2
の「必要なら ROADMAP revision を起こす」がここで具体化する。

### 現 ROADMAP M3 AC と Z 採択の差分

| 項 | 現 ROADMAP | Z 採択後 |
|---|---|---|
| Layout primitive A | Grid layout primitive | **WrapPanel** layout primitive |
| Layout primitive B | ScrollView primitive | （Z は本格的 viewport / overflow を要求しない。WrapPanel の wrap が overflow handling を兼ねる範囲で扱う。完全な ScrollView surface は M4 以降へ defer 候補） |
| Layout primitive C | List primitive | **ZStack** layout primitive + **繰り返し生成 grammar**（List の責務をこの 2 surface へ分解） |
| Spec | DSL specification first public draft | DSL specification first public draft（**条件レンダリング + 繰り返し生成を含む**） |

### ROADMAP revision の単位

ROADMAP revision は次の単位で起こす想定:

1. 上表の差分そのものを M3 AC の revision として記述する。
2. revision の根拠として本 pre-doc と
   [m3-target-app-wireframes.html](m3-target-app-wireframes.html) を参照する。
3. revision の意思決定単位は vision-level に届くか pre-doc レベルで閉じるかを別途判断する
   ([m3-start-framing.md](m3-start-framing.md#L322) §F3 の判定基準: public draft 誠実性 /
   visible proof 必要性 / grammar・IR 破壊的変更リスクのいずれか）。

ROADMAP revision の具体的執筆作業は本 pre-doc の owner approval を経た後に着手する。

---

## HTML との参照関係

[m3-target-app-wireframes.html](m3-target-app-wireframes.html) との関係を明示する。

- HTML は **exploratory artifact**（候補比較 + wireframe + 語彙批判的検討 + 加重判断記録）
  として維持する。本 pre-doc の採択に伴って HTML を縮小再構成しない。
- HTML 末尾「位置づけと将来再構成トリガー」節（[m3-target-app-wireframes.html L1470-1490](m3-target-app-wireframes.html#L1470-L1490)）
  で予告されている再構成（wireframe-only + 語彙整理 markdown 抽出）は、
  本採択時点では発火条件を満たさない。発火条件のいずれかが満たされた段階で再評価する。
- 本 pre-doc は HTML から候補選定結果と加重判断を受け、採用理由・必要 surface・out-of-scope・
  spec 同期ルール・ROADMAP との同期 を明文化する役割を持つ。HTML は本 pre-doc から
  視覚 input として参照される。
- 本 pre-doc が status: accepted になった後、HTML の「採択」記述は本 pre-doc を pointer 参照で
  指す状態のままにする（HTML 側に採択結論を duplicate しない、本 pre-doc が SSOT）。

---

## Next step

本 pre-doc が owner approval を得たら、次の順で進める。

1. 本ファイルの `status:` を `drafting` から `accepted` へ更新する。
2. ROADMAP M3 acceptance の revision を起こす（本 pre-doc「ROADMAP との同期」節の差分を反映）。
3. `docs/plans/m3-plan.md` を `status: drafting` で作成し、phase breakdown を本採択に基づき drafting する。
4. M3 最初の implementation phase の pre-doc を開く前に、本ファイルと m3-plan の owner agreement を確認する。
