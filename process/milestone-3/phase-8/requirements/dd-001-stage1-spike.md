---
title: DD-M3-P8-001 事前の実現性スパイク — 段階 1（pre-DD・`selected` 実装前）
status: done
created: 2026-06-26
feeds: dd-m3-p8-001-button-selected-state-surface (未ドラフト)
---

# DD-001 段階 1 スパイク — 排他「1 つだけ選択中」状態の実現性

[framing.md §DD-M3-P8-001](framing.md) の「フレーミングの次にやること」手順 1–2。
**DD-001 の比較・採択より前**に、出荷済みサーフェスだけで *3 タブ相当の排他
状態* が表現できるかを裏取りし、選択肢集合を立て直すための spike。`selected` の
見た目はこの段階では検証しない（段階 2＝A10 実装後に回す）。

owner review を受けて、問いを二層に分けて確定した:

- **(1) author が「選択中」を `.ui` でどう書くか**（書き方の候補。S1〜S6 で棚卸し）。
- **(2) その「選択中」を表す真偽値を、どう作って 1 つに絞るか**（採択方向
  `Button { selected: 真偽値 }` への状態の供給・排他のしかた。案 α/β/γ/δ）。

採択候補 α は、コンパイルが通るだけでなく**runtime 上で動かして**まで確かめた —
`.ui` をロードし、タブのクリックで真偽値が一斉に書き換わり、表示要素が 1 つだけに
切り替わる（click → 一括代入 → 反映 → `if` の出し入れ）ところまで headless 実行で
確認した（GUI 表示・人間目視は段階 2 へ繰延べ）。

## 目的（段階 1 のスコープ）

- タブごとに独立 bool state を持ち、各 `clicked` のブロック代入で「1 つだけ
  true・他は false」に**できるか**を確認する。
- `selected` 属性はまだ無いので、状態の観測は **conditional `if`** で行う
  （タブ状態に応じてマーカー subtree を出し入れ）。
- 出荷済みサーフェスの事実（`==` 無し / handler はブロック代入のみ / host state
  書込 API 無し）が各案（α/β/γ/δ）をどう制約するかを、grep でなく**実物のコンパイル**
  で確定する（spike 規律の適用）。

## 方法

1. 後続が着地する全ファイルを実読（IR の式型、`if` 文法、runtime evaluator、
   host state 境界）。
2. **確かめ方を 2 種に分ける**:
   * **書き方の候補（S1–S6）** は、**仕様と実装の事実（出荷済み機能の有無、新しい
     ウィジェット / 描画 / 値の書き戻しが要るか）で実現性を分類**する机上監査。
     コンパイル証拠は持たない（実際にコンパイルしたのは下の α/β/δ 系だけ）。
   * **状態の供給手段（案 α/β/δ）** は、それを体現する使い捨て `.ui` を書いて
     `wasamoc check` → `wasamoc build` の**全パイプライン**（lex→parse→check→lower
     →IR emit）に通す（書ければ成功・書けなければ診断メッセージで裏取り）。
3. 採択候補（案 α）は **runtime loader まで実行**して live 経路を押さえる
   — throwaway IR を `parse_ir`→`build_widget_tree` で実体化し、`Button clicked`
   の inline handler（3 bool ブロック代入）→ reactive drain → sibling `if` 出入り
   を headless integration で評価し、各時点で marker subtree が **1 つだけ**残ること
   を確認する（陽性対照）。GUI screenshot は段階 2 へ繰延べ（§検証しないこと）。

> 念のため: 以下の α/β/δ は「**書き方**」ではなく、S1（`Button { selected: 真偽値 }`）を
> 選んだ場合に**その真偽値をどう作って 1 つに絞るか**の話。書き方そのものの棚卸しは
> 次節（書き方の候補 S1–S6）に分けてある。

## 確定した事実（出荷済みサーフェス）

| 事実 | 正本 | 含意 |
|------|------|------|
| 式に比較・等値演算子 `==` が無い。`HandlerExpr` の演算子は `CompoundOp = Add/Sub/Mul/Div` のみ | [wasamo-ir/src/lib.rs](../../../../wasamo-ir/src/lib.rs) `HandlerExpr` / `CompoundOp` | 「選択中タブ＝この値」を式で書けない → 排他は状態の組で表すしかない |
| `if` の条件は `BOOL_LIT \| IDENT`（bool 型 state へ解決）のみ。**演算子なし**（`!`・比較・論理）| [docs/dsl_spec.md](../../../../docs/dsl_spec.md) §4.6/§4.14、[gallery.ui](../../../../examples/gallery/gallery.ui) `if is_lightbox_open` | 観測は「タブごとに 1 つの bool state ＋ それぞれの `if`」になる |
| handler は状態へのブロック代入のみ（`clicked => { … }`、複文可）| [examples](../../../../examples/gallery/gallery.ui)、runtime [handler.rs](../../../../wasamo-runtime/src/handler.rs) `HandlerExpr::Block` / bool `Assign` | 各 `clicked` が「自分 true・他 false」を**手書き**で代入できる（案 α の核）|
| 表示中に host が component state を書く public API が無い | [host-state-boundary.md](../../../../docs/notes/host-state-boundary.md) | 排他を host 側ロジックへ逃がせない → 旧「ホスト側ロジック vs DSL グループ概念」の 2 択は廃。selected は binding 駆動で閉じる |

## 書き方の候補監査（S1–S6。owner review 指摘）

「選択中」を author が `.ui` でどう書くか、の候補空間を DD-001 の手前で棚卸しする。
次節の α/β/δ は「書き方」ではなく真偽値の作り方・絞り方の話なので、ここには含めない。

**結論（表を精読しなくてもよいよう先出し）: DD 本命＝S1、比較に残す＝S2、
明示 却下＝S3、対象外（理由付き）＝S4–S6。** 以下は各候補の根拠。

| # | 候補構文 | 書き手にとっての利点 | M3 実現性（出荷済み機能基準） | DD 配置 |
|---|----------|----------------------|-------------------------------|---------|
| S1 | `Button { selected: <bool> }` | 既存の真偽値バインディングに乗る・新部品不要・最小 | `selected` 属性追加＝A10 のスコープ。横断実装は要るが既存の真偽値バインディング経路に乗る | **本命**（DD で確定する書き方） |
| S2 | `ToggleButton { checked: <bool> }` / `SelectableButton` | トグル/選択の意味が型で明示・将来の書き戻しの自然な置き場 | 新部品＋新しい描画＋（双方向なら）書き戻し＝重い | DD に**意味比較として残す**（M3 では過大の公算大。S1 と意味差を対比） |
| S3 | 見た目テーマ経由（`style: accent` を条件分岐で差し替え／`style: cond ? accent : default`）| 低い | 三項演算子は式に無く不能。`if` で Button 自体を差し替える回避策は理屈上可だが handler 重複・選択の意味づけ不在で脆い | **明示 却下**（framing が「見た目テーマ経由」を候補にしていたため棚卸しして落とす） |
| S4 | 親/グループ部品（`TabBar` / `RadioGroup` / `SegmentedControl`）| 書き手に `==` を書かせず親が排他を管理＝最も簡潔 | 出荷済みに無い。`==` とは無関係に、値の書き戻し・選択値・子の値・操作の意味づけを開く必要がある | **M3 対象外**。きっかけは **`==` ではなく** 書き戻し/選択値/操作の意味づけ。等値演算子系（γ・δ）とは**別の先送り軸** |
| S5 | 単一 discriminant：i32 **および string / enum-like**（`state tab: string = "all"` ＋ `selected: tab == "all"`）| 単一の真実源・書き手に自然 | i32 同様 `==`（string では文字列等値）が無く詰む。δ は i32 を検証したが**結論は string/enum も同族** | **M3 対象外**・等値演算子系（δ を string/enum へ一般化して明記） |
| S6 | データ駆動タブ（collection ＋ `for` で tab を生成）| データ駆動・タブ数可変に自然 | `for` 内 handler は M3 繰延べ・`if` 条件で binder/item を読めない・index 等値も無く、選択中表示に届かない | **M3 対象外**（M3 の `for` 制約による不採。gallery 統合フェーズで自然に疑われるため一行残す） |

## 結果 — `Button.selected` の真偽値の作り方・絞り方（案 α/β/γ/δ）

採択方向 S1 の `selected` の真偽値を、出荷済み機能だけでどう作り・1 つに絞るか。

### 案 α（手書きブロック代入）= **実現可能（live 経路まで実証）**

体現 spike を全パイプラインに通した（`check` exit 0 / `build` exit 0）。emit
された IR が排他構造をそのまま示す:

- `state tab_all/tab_albums/tab_favorites: bool`（独立 3 状態）
- 各 `clicked` → `(block (assign tab_all true) (assign tab_albums false) (assign tab_favorites false))`
  ＝ 複文ブロックで「1 つ true・他 false」
- 観測 → `if (bool-prop-read tab_all) { node Text … }` を 3 本

**live 経路の実証（owner review 指摘の追試）**: コンパイルだけでなく runtime まで
実行した。throwaway IR を `parse_ir`→`build_widget_tree`（active registry を設置）
で実体化し、各タブ Button の inline `clicked` を `hit_test_click` で発火させて
headless integration を回した（[conditional_toggle_integration.rs](../../../../wasamo-runtime/tests/conditional_toggle_integration.rs)
／[bool_binding_live_propagation.rs](../../../../wasamo-runtime/tests/bool_binding_live_propagation.rs)
と同じ live 経路）。結果:

- 初期（tab_all=true）→ VStack 子は `HStack ＋ marker 1 本`で marker = "ALL"。
- "Albums" click → ブロック代入（all=false/albums=true/favorites=false）→ reactive
  drain → "ALL" 除去・"ALBUMS" 挿入。子は依然 **HStack ＋ marker 1 本のみ**。
- "Favorites" → "FAVORITES" 1 本。"All" → "ALL" 1 本。
- **各時点で marker subtree はちょうど 1 つ**（`children.len()==2` を毎ステップ
  assert）＝排他が live 経路で成立。
- **陽性対照**: 期待値の 1 つを `"WRONG_NEG_CONTROL"` に壊すとテストは
  `left: "ALBUMS"` / `right: "WRONG_…"` で**失敗**——assertion が skip-as-pass でなく
  実際にクリック後の live 状態を観測していることを確認（戻して green 再確認）。
- skip ガード未発火（compositor 利用可の本環境で実走。skip 時は "skipping … runtime
  compositor unavailable" を出すが未出力）。

runtime のプリミティブ正本: ブロックは
[handler.rs](../../../../wasamo-runtime/src/handler.rs) `HandlerExpr::Block` で逐次
評価、bool `Assign{rhs:BoolLit}` は `set_bool` で signal へ書く。conditional `if`
の reactive 再描画は Phase 6 出荷済み。案 α で**新しい surface はゼロ**——出荷済み
単一 bool プリミティブの N 重適用にすぎないことが live 経路でも確認できた。

- **コスト**: 3 タブで 3×3＝9 代入の手書き（O(N²)）。タブ追加で増殖し脆い。
- このテストは throwaway（spike 用に作成→evidence 記録後に削除）。恒久 regression
  test 化は DD-001 採択後の実装フェーズの判断（landing 時に skip ガード発火検証が要る
  ——AGENTS.md §Testing rules）。

### 案 β（A10 の実証手段を最小化）= **実現可能（ただし「単一 Button 自己トグル」は不能・形を修正）**

owner review 指摘で精査した。framing が言う「単一トグル」を素直に取った
**1 つの Button が自分で on/off を反転する**形（`clicked => { root.sel = !root.sel; }`）は
**不能**——`!`（否定）が式文法に無く parse error（`expected expression, found !`）。
[bool-demo.ui](../../../../examples/bool-demo/bool-demo.ui) も実体は `ready = false` の
**一方向**更新＋自分を disabled 化で、自己トグルではない。

したがって β の実現可能な最小形は**「単一 `selected` bool を 2 ボタンで on/off」**
（`Select`→`sel=true` / `Clear`→`sel=false`、観測 `if sel`）。これは compile 成功
（exit 0）。陽性対照は「Select で marker 出現・Clear で消滅」の 2 コマで、selected の
binding 駆動を**排他を見せずに**最小実証する。タブ帯は静的強調に留める。

> 要するに β は「単一*状態*の selected 遷移を 2 ボタンで作る」形であって、「単一
> *Button* のトグル」ではない。framing の β 記述（単一 Button on/off）はこの spike で
> 訂正された。

### 案 γ（排他を楽にする最小サーフェス：`==` 等値式 / discriminant 派生）= **出荷済みでは実現不能 → M3 out（`==` trigger で defer）**

γ は **`==` 等値式（および discriminant からの bool 派生）** に縮める。`==` が IR /
文法に存在しない（上表）ため不能。導入は式文法への等値演算子追加＝機能幅。M3 で
採れないことを spike が確定したので、DD-001 §Out of scope に **`==` trigger** で
defer する（実在しない案を比較に並べない）。

> **group surface は γ ではない**: framing の γ は当初「`==` / 単一選択グループ」を
> 束ねていたが、**グループ概念（RadioGroup / TabBar 等）は §書き方の候補監査
> の S4 として独立**させる。S4 の trigger は `==` ではなく write-back / selected
> value / child value / interaction semantics であり、`==`-family（γ・δ）とは**別の
> defer 軸**。両者を一本化しない（owner review 指摘）。

### 案 δ（i32 `tabIndex` 1 個で保持）= **保持は可・A10 の観測に到達できず不採**

owner 指摘で追加検証（bool 3 個でなく i32 単一変数 `tab_index` に index を持たせ
切り替える方式）。3 variant をコンパイルして裏取りした:

| variant | 結果 | 含意 |
|---------|------|------|
| `if tab_index == 0 { … }` | **parse error**（`expected {, found =`）| `==` が文法に無い |
| `if tab_index { … }`（裸の i32）| **check error**「`if` condition must be `bool`; state `tab_index` is declared `i32`」| i32 を条件に使えない（暗黙 i32→bool も無し）|
| `Text { text: "…\{root.tab_index}" }` | **exit 0** | i32 の観測は文字列補間でしか到達できない |

つまり i32 `tabIndex` は**保持側は α より明らかにきれい**（単一 state・各 clicked
は単一代入で O(N)・本質的に排他）が、**観測側で詰む**。`if` は bool しか取れず
（`==` 無し）、index から「このタブが選択中」という bool を導けない。観測は index
の数値補間に限られ、これは A10 が対象とする**ボタン単位の `selected` 見た目に接続
しない**——将来 `selected: bool` を binding 駆動するにも `tab_index == n` 形の bool
派生が要り、これがまさに `==`（案 γ／繰延べ式文法拡張）。したがって A10 の趣旨
（bool binding が*属性*を駆動）に**到達できる**のは bool-per-tab（α）の方で、α の
O(N²) は missing `==` の代償として**構造上不可避**である。i32 で O(N) にできるが
selected/`if` へ繋がらない、という非対称が確定した。

### 不採用候補の将来復活（wasamo 利用方針）— 先送りの軸は 2 本

今回不採用の候補は、**互いに別のきっかけで復活する 2 本の軸**に分かれる。DD-001
§Out of scope はこの 2 本を**別項目**として書き、`==` 一本に束ねない。

| 先送りの軸 | 該当候補 | 再検討のきっかけ（trigger） | 復活後の書き方 |
|------------|----------|------------------------------|----------------|
| **等値演算子系（`==`-family）** | γ（`==`／discriminant 派生）＋ δ（i32／string／enum の単一 discriminant） | 式に**等値演算子 `==`** が入るとき | 単一の discriminant state ＋ `selected: tab == 値`。O(N)・単一代入・本質的に排他で、α の O(N²) 手書きを置き換える |
| **グループ部品系（S4）** | RadioGroup / TabBar / SegmentedControl | **`==` ではなく**、値の書き戻し・選択値・子の値・操作の意味づけ（write-back / selected value / child value / interaction semantics）を開くとき | 親が排他を管理し、author は `==` を書かない。等値演算子系より重い |

これは wasamo の**利用方針（author が排他をどう書くか）の将来像**であって、
`examples/gallery/` を後から書き換えるかの**具体作業とは別レイヤ**（gallery を α 据え置き
か δ 系へ移すかは `==` 実装フェーズの独立判断）。

## 選択肢集合の確定（手順 2 のアウトプット）

**書き方（DD-001 が確定する本体。§書き方の候補監査）**:

- **本命＝S1 `Button { selected: <bool> }`**。比較に残す＝**S2 `ToggleButton`**
  （意味差の対比）。明示 却下＝**S3 見た目テーマ経由**。対象外に
  理由付き＝**S4 グループ部品 / S5 discriminant＋等値 / S6 for 駆動**。

**S1 の真偽値の作り方・絞り方（書き方そのものではない）**:

- **M3 で採れるのは α（タブ帯で排他を live に見せる・live 経路実証済み）と β
  （単一 selected を 2 ボタンで on/off）の 2 つ**。β の「単一 Button 自己トグル」は
  `!` 不在で不能ゆえ 2 ボタン形に訂正済み。
- α と β の取捨は spike が決める事項ではない——**product merit の trade-off
  （排他の振る舞いを gallery タブ帯で見せる価値 vs O(N²) 手書きの脆さ）として
  DD-001 が確定**する（評価軸は product merit 主軸・改訂コストは tie-breaker）。
- **γ・δ は今回いずれも不採用**（理由は同一＝`==` 繰延べ）。**`==` 実装で復活する
  discriminant 系の排他表現の将来候補**として束ねて残す（前節の利用方針）。実現不能
  ゆえ M3 の比較には並べない。**S4（group surface）は trigger が `==` ではなく
  write-back / interaction semantics なので、`==`-family とは別の defer 軸として
  書き分ける**。

## 段階 1 で**検証しないこと**（exit 時に後続へ割付済み）

- selected の**見た目**の連動（陽性対照 2 コマ）→ **段階 2（A10 実装後）**。
  `selected: bool` 追加後に、選んだ実証手段で binding 駆動の見た目連動を撮る。
- selected の parser→check→lower→IR→runtime 横断実装と伝播監査 → **DD-001 採択後の
  実装フェーズ**（framing §R6 / §検証方針 の selected 伝播監査）。
- 最小の見た目合格線（背景色だけ / 枠線だけ / 色＋枠）→ **DD-001 で owner に見せて
  確定**（構文だけでは決めない）。

## 監査用アーティファクト（再現可能性）

すべて throwaway（spike 用に作成し evidence 記録後に削除。`.ui` は scratchpad、
α の live test は `wasamo-runtime/tests/` に一時作成して実行後削除）。同じ結論に
あとから辿れる粒度で、入力・コマンド・exit・診断を残す。

### 1. コンパイル結果一覧（`wasamoc check` / `build`、release バイナリ）

| 入力 `.ui`（要旨） | コマンド | exit | 観測 |
|--------------------|----------|------|------|
| α: 3 bool state、各 clicked が 3 代入ブロック、`if tab_*` 3 本 | `wasamoc check` / `build` | **0 / 0** | 下記 IR を emit |
| δ-1: i32 `tab_index`、`if tab_index == 0` | `wasamoc check` | **1** | parse error: `expected {, found =`（`==` 不在） |
| δ-2: i32、`if tab_index`（裸） | `wasamoc check` | **1** | check error: ``if` condition must be `bool`; state `tab_index` is declared `i32` (dsl_spec §4.14)`` |
| δ-3: i32、`Text { text: "…\{root.tab_index}" }` | `wasamoc check` | **0** | i32 観測は文字列補間のみ到達 |
| β-1: 単一 Button `clicked => { root.sel = !root.sel; }` | `wasamoc check` | **1** | parse error: `expected expression, found !`（`!` 不在＝自己トグル不能） |
| β-2: 2 ボタン `Select`→`sel=true` / `Clear`→`sel=false`、`if sel` | `wasamoc check` | **0** | 最小 β の実現可能形 |

### 2. α の emit IR（排他の核）

```
state tab_all: bool = true
state tab_albums: bool = false
state tab_favorites: bool = false
...
on clicked {                                  ; "Albums" タブの handler
    (block (assign tab_all false) (assign tab_albums true) (assign tab_favorites false))
}
...
if (bool-prop-read tab_all)       { child { node Text { prop text = "ALL is selected" } } }
if (bool-prop-read tab_albums)    { child { node Text { prop text = "ALBUMS is selected" } } }
if (bool-prop-read tab_favorites) { child { node Text { prop text = "FAVORITES is selected" } } }
```

### 3. α の live runtime integration（headless）

- 経路: `lower → emit IR → parse_ir → build_widget_tree`（active registry 設置）
  → 各タブ Button visual を (100×40) に pin → `hit_test_click(50,20)` で inline
  `clicked` 発火 → reactive drain → sibling `if` 出入り。
- コマンド: `cargo test -p wasamo-runtime --test spike_alpha_exclusive -- --nocapture`
  → `test result: ok. 1 passed`（skip メッセージ無し＝実走）。
- 毎ステップ `built.root.children.len() == 2`（HStack＋marker 1 本）を assert。
  click 後 marker 文字列: ALL → ALBUMS → FAVORITES → ALL。
- 陽性対照（負）: 期待値を `"WRONG_NEG_CONTROL"` に壊すと
  `test result: FAILED`、`left: "ALBUMS"` / `right: "WRONG_NEG_CONTROL"`
  （assertion が live 状態を実観測。skip-as-pass でない）。戻して green 再確認。
