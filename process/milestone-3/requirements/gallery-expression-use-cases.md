---
title: Gallery 具体ユースケース × 式言語プリミティブ × M-expr 段階（仮説・要求整理）
status: exploratory
created: 2026-06-29
related:
  - examples/gallery/gallery.ui
  - docs/dsl_spec.md
  - docs/notes/expression-language-roadmap.md
  - docs/notes/dsl-grammar.md
---

# Gallery 具体ユースケース × 式言語プリミティブ × M-expr 段階

## このメモの位置づけ

**意思決定ではなく、仮説・要求整理。** 「将来 Gallery でやりたい具体的な
操作」を 4 つに絞り、それぞれを実現するのに `.ui` の **式言語に必要な
プリミティブ**、**必要な M-expr 段階**、そして **式言語の外で要る前提**
(M-expr 段階の拡張では供給されない、式言語とは別の前提)を、実例つきで整理する。

基準言語は現状の `.ui`([dsl_spec.md](../../../docs/dsl_spec.md)):

- 式はリテラルと変数参照だけ(**演算子ゼロ**、§4.6)。
- **要素アクセス `xs[i]` は無い**(コレクション操作は `for` 反復と handler の
  `append`/`drop-last`/リテラルのみ)。
- `for` の loop binder(`item`/`index`)は **式(プロパティ/補間)位置だけ**で
  読める。**handler 位置・`if` 条件位置では読めない**(§4.6)。
- `if` の条件は真偽 state か真偽リテラルのみ。
- `Box.fill` などの見た目は **定数のみ**(§4.9、binding 不可)。
- クリックを受けるのは `Button` だけ(`Box` 等は不可)。

M-expr 段階の定義は [expression-language-roadmap.md](../../../docs/notes/expression-language-roadmap.md)
を正本とする。要点だけ再掲:

- **M-expr1 述語**: 比較・論理・否定(`== < && !` 等)。結果は真偽だけ。値の
  持ち方は変えない。
- **M-expr2 計算束縛**: 値を生む式。`TypedValue` を入れるかで 2 つに分かれる:
  - **M-expr2a** — 要素アクセス `xs[i]` や既存スカラ同士の算術・連結で、結果が
    `i32`/`string`/`bool` に収まるもの。`TypedValue` を**導入しない**軽い側。
  - **M-expr2b** — 結果が新しい種類の値(色・寸法)になる、または式が任意型の値を
    運ぶ一般化。`TypedValue` を**導入する**重い側。
- **M-expr3 構造化データ**: struct(レコード)型 + メンバアクセス。

---

## spec.md との整合性(重要)

**本メモは M3 スコープを定義する文書ではない。** 同じ `requirements/` にある
[spec.md](spec.md)(M3 target app pre-doc、accepted 2026-05-16)が M3 の SSOT で
あり、本メモはそれを **改訂しない**。

spec.md のスコープは **レイアウト部品だけではない**。§必要 surface は Layout
primitive / Grammar surface / Widget・content surface / **Binding・value surface**
の 4 区分を持ち、§Out-of-scope §Value / type で **値・型の対象範囲の上限を
明示的に固定**している:

- スカラは `i32` + `String` + `bool` の 3 種で閉じ、**第四 scalar 以降 /
  `TypedValue`(generic value union)は M3 では導入しない**
  ([spec.md](spec.md) §必要 surface §Binding / value surface)。
- **「本 target app は `TypedValue` 圧力を構造的に避ける設計に寄せる」**
  ([spec.md](spec.md) §Out-of-scope §Value / type)。つまり gallery は、本メモが
  挙げるユースケースが *構造的に発生しないように* 意図して設計されている。
- lightbox の close/prev/next は **Button click の handler binding** で表現し、
  swipe/keyboard や focus model には踏み込まない([spec.md](spec.md) §Out-of-scope §Interaction)。
- `Box.fill` 等の見た目は装飾なし最小・定数のみ。selected 時の visual も最小表現
  ([spec.md](spec.md) §Out-of-scope §Visual / styling)。

**本メモの 4 ユースケースは、この対象範囲の外（＝M3 では扱わない領域）にある**:

| UC | spec.md の該当 out-of-scope | 関係 |
|---|---|---|
| 1 強調 | 見た目は装飾なし最小・`fill` は定数(§Visual/styling, §4.9) | 見た目 binding は M3 外 |
| 2-1 | 値を生む式(要素アクセス)は value surface 外 | M-expr2a、M3 外 |
| 2-2 案A/B | §Value/type「`TypedValue` 圧力を構造的に避ける」 | M3 外(案A=2a、案B の struct は `TypedValue` を要する要因そのもの=M-expr3) |
| 3 押して選択 | §Interaction(`Box` クリック=M4、handler-binder=Q8 未決) | M3 外 |

したがって本メモと spec.md は **矛盾せず、補完関係**にある。spec.md が「M3 は
`TypedValue` 圧力を避ける」と *対象範囲の上限を定め*、本メモは「`TypedValue` 等が
必要になるのはどのユースケースで、どの M-expr 段階で初めて表現可能になるか」を
*その範囲の外として* 列挙する、将来の拡張を先取りして分類した資料である。

**特に紛らわしい一点**: spec.md が M3 で開ける **Button `selected` 属性
(bool 駆動)**([spec.md](spec.md) §必要 surface §Widget / content surface)と、本メモ
**UC3 の「サムネを押して per-item で選択」**は **別物**である。前者は真偽 state が
widget 属性を駆動することの実証(M3 内)。後者は loop 内の各セルがクリックを受け、
handler で `index` を読んで選択を *設定* する操作で、`Box` クリック(M4
interaction)と handler-binder 読み(Q8 未決)を要する **M3 外**の話。両者を
同一視しないこと。

---

## 一覧表

| ユースケース | 式言語に必要なプリミティブ | 必要な M-expr 段階 | 式言語の外で要る前提(M-expr 段階の拡張では供給されない) |
|---|---|---|---|
| **1. 1枚強調** | 比較 `index == 強調idx`(真偽を生む) | **M-expr1** | bindable な見た目(現状 `fill` は定数のみ)。条件オーバーレイで出すなら **`if` 条件位置での binder 読み解禁**(現状 §4.6 で禁止) |
| **2-1. ラベルだけ切替** | 要素アクセス `labels[current]`(string を生む)＋補間 placeholder の一般式化。矢印 `current += 1`/`-= 1` は既存 | **M-expr2a** | なし(端で止めるなら比較=M-expr1 を追加) |
| **2-2. 相関フィールドを一緒に** | **(A) 並列スカラ配列 + 要素アクセス** `captions[current]` 等、または **(B) struct + メンバアクセス** `photos[current].caption` | **(A) M-expr2a / (B) M-expr3** | (B) で日付を本物の日付型にするなら新スカラ型=`TypedValue` 拡張(表示用 string なら不要) |
| **3. 押して選択** | 選択保持・強調は比較 `index == selected`(M-expr1)。設定は handler で `selected = index` | **M-expr1**(選択ロジック本体) | **本体はここ**: ①`Box` への click(現状 `clicked` は `Button` のみ=M4 interaction)、②**handler 位置での binder `index` 読み**(未決 admission、[dsl-grammar Q8](../../../docs/notes/dsl-grammar.md)) |

以下、各ユースケースを実例で示す。**コードブロックはすべて「将来こう書けると
したら」の仮イメージ**であり、現行の spec ではない。

---

## 1. サムネイルセル内の1枚強調

「いまフォーカス中の 1 枚だけ枠を変える/光らせる」。

**いま光らせたい番号を i32 で持つ**:

```
state highlighted: i32 = 0
```

**M-expr1(比較)が入ると** — どのセルが強調対象かを `index == highlighted`
で判定できる。強調を「条件オーバーレイ」で出す書き方:

```
for label, index in labels {
    Box {
        aspect: 1:1
        fill: #336699cc

        if index == highlighted {        // ← M-expr1: 比較で真偽を作る
            Box { fill: #ffcc0080 }      //   強調用の半透明オーバーレイ
        }

        Text { text: "\{label}" }
    }
}
```

**ただし式言語だけでは足りない点が 2 つ**:

1. 上の `if index == highlighted` は **条件位置で loop binder `index` を読んで
   いる**。これは現状 §4.6 で **禁止**。M-expr1(演算子追加)とは別に、
   「条件位置での binder 読み」を解禁する admission 判断が要る。
2. オーバーレイを使わず「枠色を変える」形にするなら、`Box.fill` が現状
   **定数のみ**なので、**bindable な見た目プロパティ**(色やボーダーの binding)
   が要る。これは widget surface 側の話で、M-expr 段階の拡張では供給されない。

→ **1枚強調のロジックは M-expr1 で表せるが、見せ方(条件位置 binder/見た目
binding)は式言語の外の前提に依存する。**

---

## 2. lightbox 矢印ナビ

「現在位置を矢印で前後に動かす」基礎部分は **今でも書ける**。位置を i32 で持ち、
矢印 Button が増減するだけ:

```
state current: i32 = 0
...
Button { text: "<"  clicked => { root.current -= 1; } }
Button { text: ">"  clicked => { root.current += 1; } }
```

問題は「`current` に応じて中身を出す」側。ここで必要な段階が分かれる。

### 2-1. ラベルのテキストだけ切り替えられればよい

ラベル配列を持ち、**現在位置の要素を引いて表示**する:

```
state labels: string[] = ["S01", "S02", "S03", "S04", "S05", "S06"]
state current: i32 = 0
...
Text { text: "\{labels[current]}" }     // ← 要素アクセス labels[current]
```

- 必要プリミティブは **要素アクセス `labels[current]`**。これは演算子ではない
  が、**束縛に「値を生む式」**(名前付き read を超える初の形)なので、真偽しか
  作れない M-expr1 を越え、**M-expr2** に該当する。
- ただし結果が string(既存スカラ)に収まるので、**`TypedValue` ユニオンは
  要らない=M-expr2 の軽い側 = M-expr2a**(roadmap の内訳参照)。
- **補間 placeholder の拡張も伴う点に注意。** 現行 spec の `"\{…}"` の中身は
  `qualified_name`(`root.count` 等)に限られる([dsl_spec.md](../../../docs/dsl_spec.md) §2.4)。
  `"\{labels[current]}"` を書くには、補間 placeholder を **一般式へ広げる**
  拡張が M-expr2a の一部として必要になる(プロパティ RHS だけでなく補間位置も
  同時に育つ、という spec の「全位置で一斉に育つ」前提どおり)。
- 端で止めたい(先頭で `<` を無効化)なら比較を足す — これは M-expr1:

```
Button { text: "<"  enabled: current > 0          clicked => { root.current -= 1; } }
Button { text: ">"  enabled: current < labels_max clicked => { root.current += 1; } }
```

> 補足: `current < labels_max` の `labels_max` を「配列長」で出したい場合、
> 長さ取得 `labels.length` は **値を生む式**(結果は i32)なので **M-expr2a** が
> 要る。固定上限を別 state で持つなら M-expr1 だけで済む。

### 2-2. 写真としての相関フィールドを一緒に出したい(画像ID＋キャプション＋日付)

**重要: struct(レコード)は必須ではない。** 2 通りある。

#### 案A — 並列スカラ配列 + 要素アクセス(struct 不要・**M-expr2a で足りる**)

フィールドごとに配列を持ち、**同じ `current` で全部を引く**:

```
state ids:      i32[]    = [101, 102, 103]
state captions: string[] = ["朝の海", "町並み", "夕暮れ"]
state dates:    string[] = ["2026-01-05", "2026-02-11", "2026-03-20"]
state current:  i32      = 0
...
VStack {
    Text { text: "ID: \{ids[current]}" }
    Text { text: "\{captions[current]}" }
    Text { text: "\{dates[current]}" }
}
```

- **これは struct なしで実現できる。** 必要なのは要素アクセスだけ=**M-expr2a**
  (結果は既存スカラに収まり、`TypedValue` を要さない)。
- 「相関」は *「同じ index を 3 配列に使う」という規約* で担保される。
- 弱点: 配列の長さや並びがずれても **気づかないまま壊れる**(`ids` だけ 1 件多い等)。
  Add/Remove するときは 3 配列を **揃えて操作する責任が作者に残る**:

```
// 1 枚追加 = 3 配列すべてに append する責任
clicked => {
    ids = ids.append(104);
    captions = captions.append("新規");
    dates = dates.append("2026-04-01");
}
```

#### 案B — struct + メンバアクセス(**M-expr3**、相関を型で保証)

1 レコードにまとめる:

```
struct Photo { id: i32, caption: string, date: string }

state photos: Photo[] = [
    { id: 101, caption: "朝の海",   date: "2026-01-05" },
    { id: 102, caption: "町並み",   date: "2026-02-11" },
    { id: 103, caption: "夕暮れ",   date: "2026-03-20" },
]
state current: i32 = 0
...
VStack {
    Text { text: "ID: \{photos[current].id}" }     // ← メンバアクセス .id
    Text { text: "\{photos[current].caption}" }
    Text { text: "\{photos[current].date}" }
}
```

- 必要プリミティブは **struct 型 + メンバアクセス**=**M-expr3**。
- 相関が *型で保証* され、Add/Remove も **1 単位**で済む:

```
clicked => { photos = photos.append({ id: 104, caption: "新規", date: "2026-04-01" }); }
```

#### 案A と案B の関係(要点)

- **2-2 は struct 必須ではない。並列スカラ配列 + 要素アクセス(M-expr2a)で
  代替できる。**
- struct(M-expr3)は「相関を **規約でなく型で** 保証し、**まとめて足し引き**
  できる」という改善であって、**ユースケースの要件ではない**。
- 段階の繰り上げ(M-expr2a → M-expr3)が正当化されるのは、フィールド数が増えて
  並列配列の同期コスト/壊れやすさが無視できなくなったときや、要素を「1 枚=
  1 レコード」として handler で扱いたくなったとき。
- 日付を本物の日付型(比較・整形ができる値)にしたい場合は、案 A/B いずれでも
  **新スカラ型=`TypedValue` 拡張**が別途要る。**表示用の文字列で足りるなら
  不要。**

---

## 3. サムネイルを押して選択(インタラクティブ)

「サムネをクリックしたら、それを選択状態にして強調する」。

```
state selected: i32 = -1     // -1 = 未選択
...
for label, index in labels {
    Box {
        aspect: 1:1
        fill: #336699cc

        clicked => { root.selected = index; }   // (A)(B) 下記参照

        if index == selected {                   // (C) M-expr1: 比較
            Box { fill: #ffcc0080 }
        }

        Text { text: "\{label}" }
    }
}
```

この 1 例の中に、性質の違う 3 つの要求が混ざっている:

- **(A) `Box` がクリックを受ける** — 現状 `clicked` は `Button` だけ。これは
  **M4 の interaction surface**で、M-expr 段階とは無関係。
- **(B) handler で `index` を読む** — `selected = index` は handler 位置で loop
  binder を読む。現状 **禁止**で、しかも [dsl-grammar Q8](../../../docs/notes/dsl-grammar.md)
  で「handler 位置から `item`/`index` を読めるか」は **未決 admission**。
  これも演算子の段階では供給されない別判断。
- **(C) 選択の強調表示** — `index == selected` は **M-expr1**(比較)。ただし
  ユースケース 1 と同じく、条件位置 binder/見た目 binding という式言語外の前提を
  抱える。

→ **「押して選択」の式言語側の負担は小さい(選択は i32 を 1 つ持って比較するだけ
= M-expr1)。難しいのはむしろ式言語の外**(クリック可能化と handler-binder 読み)に
ある。

選択を「写真レコードごと」(選択中の caption/date も即座に使う)に拡張するなら、
2-2 案 B と同様に **M-expr3** へ繰り上がる。

---

## まとめ（要点）

- **M-expr1 で表現できる**: 1 の強調判定、3 の選択・強調ロジック本体。
- **M-expr2a(要素アクセス・既存スカラに収まる)で表現できる**: 2-1、および
  **2-2 を並列スカラ配列でやる案A**。`TypedValue` は要らない。
- **M-expr3(struct)が要るのは改善目的**: 2-2 を相関保証つきでやる案B、
  「レコードごと選択」に拡張した 3。**2-2 自体の必須要件ではない。**
- **どの M-expr 段階でも対応できない、式言語とは別の前提**: 1/3 の見た目
  binding と条件位置 binder、3 の `Box` クリック(M4 interaction)、handler-binder
  読み(Q8 未決)。これらは M-expr 段階の外にある前提なので、段階の議論と
  **混ぜない**こと。
