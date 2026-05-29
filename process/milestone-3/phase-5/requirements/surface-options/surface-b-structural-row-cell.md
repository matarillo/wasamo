---
title: M3-Phase 5 Grid Surface B — pure structural Row / Cell
status: draft
target-phase: M3-Phase 5
role: supplemental requirement note
---

# Surface B — pure structural Row / Cell

This is a supplemental owner-alignment note for
[../framing.md](../framing.md). It expands the `.ui` writing style,
ecosystem contrast, and future-extension implications for one candidate
Grid surface. It is not an ADR recommendation.

Surface B は、`Grid { Row { Cell { ... } } }` という構造で visible rows /
cells をそのまま書く案です。`Grid` は `columns:` / `rows:` を持たず、content
widget も `row` / `column` を持ちません。

このファイルでは、比較可能にするため **B-reject** variant を仮定します。
B-reject は canonical non-spanning row から shared column widths を推定し、
他の row が矛盾する width を宣言した場合に reject する規則です。
これは Surface B の recommendation ではなく、A / A2 / D / C と flat に比較するための
最小規則です。

## 書き味イメージ

### 最小の 2 column Grid

```wasamo-ui
Grid {
  Row {
    height: 48

    Cell {
      width: 180
      Text { text: "Name" }
    }

    Cell {
      width: 1*
      TextInput { value: {profile.name} }
    }
  }
}
```

書き味の特徴:

- `.ui` の nesting が visible structure を直接 mirror する。
- content widget に `row` / `column` metadata が付かない。
- shared column sizing は各 row の `Cell { width: ... }` から推定される。

### Weighted star と spanning

```wasamo-ui
Grid {
  Row {
    height: 64
    Cell {
      column-span: 3
      Text { text: "Project" }
    }
  }

  Row {
    height: 1*

    Cell {
      width: 96
      Box { fill: #334455 }
    }

    Cell {
      width: 1*
      Box { fill: #557799 }
    }

    Cell {
      width: 2*
      Box { fill: #88aacc }
    }
  }

  Row {
    height: 48
    Cell {
      column-span: 3
      Text { text: "Ready" }
    }
  }
}
```

この例では middle row が canonical non-spanning row として列幅を定義します。
header / footer の spanning `Cell` は `width` を持たず、推定済みの 3 columns を
またぎます。

### Alignment を含む例

```wasamo-ui
Grid {
  Row {
    height: 40

    Cell {
      width: 180
      h-align: end
      v-align: center
      Text { text: "Email" }
    }

    Cell {
      width: 1*
      column-span: 2
      TextInput { value: {account.email} }
    }
  }

  Row {
    height: 40

    Cell { width: 180 }
    Cell { width: 1* }
    Cell {
      width: 120
      h-align: end
      v-align: center
      Button { text: "Save" }
    }
  }
}
```

alignment の carrier は `Cell` になります。content widget 側に layout metadata を
増やさない点が Surface A との大きな違いです。

### Gallery proof slice

```wasamo-ui
Grid {
  Row {
    height: 64
    Cell {
      column-span: 3
      Text { text: "Grid proof" }
    }
  }

  Row {
    height: 1*

    Cell {
      width: 96
      Box { fill: #263340 }
    }

    Cell {
      width: 1*
      Box { fill: #426985 }
    }

    Cell {
      width: 96
      Button { text: "Open" }
    }
  }

  Row {
    height: 48
    Cell {
      column-span: 3
      Text { text: "fixed + star + span" }
    }
  }
}
```

3 rows x 3 columns / 5 children の visible proof は自然に書けます。
ただし column vector は middle row から推定されるため、この row が Grid の
構造上かなり重要になります。

## Invalid shape examples

### Shared track sizing mismatch

```wasamo-ui
Grid {
  Row {
    Cell { width: 180 Text { text: "Name" } }
    Cell { width: 1* TextInput { value: {name} } }
  }

  Row {
    Cell { width: 200 Text { text: "Email" } }
    Cell { width: 1* TextInput { value: {email} } }
  }
}
```

B-reject では、row 0 の column 0 は `180`、row 1 の column 0 は `200` なので
reject します。Surface A / A2 / D / C では column width を 1 箇所で定義するため、この
種類の mismatch は表現できません。

### Spanning-only rows cannot define columns

```wasamo-ui
Grid {
  Row {
    Cell {
      column-span: 2
      Text { text: "Only spanning row" }
    }
  }
}
```

canonical non-spanning row がない場合、column vector を推定できません。
この形を許すなら、別の column-count / default-width rule が必要になります。

### Multiple children in one Cell

```wasamo-ui
Grid {
  Row {
    Cell {
      Box { fill: #333333 }
      Box { fill: #666666 }
    }
  }
}
```

M3 A2 は 1 cell 1 child を要求しています。`Cell` を single-child wrapper として
定義しないと、ここに stacking rule が必要になり、ZStack との境界が曖昧になります。

## Ecosystem contrast

### HTML table

Surface B は HTML table の markup に最も近いです。

```html
<table>
  <tr>
    <td colspan="2">Header</td>
  </tr>
  <tr>
    <td>Name</td>
    <td><input /></td>
  </tr>
</table>
```

visible structure と document structure が一致する点は強いです。
一方で、HTML table は intrinsic sizing と spanning の歴史的複雑さを持ちます。
Wasamo が Surface B を採る場合、B-reject のように sizing rule をかなり絞らないと
table layout 的な複雑さに近づきます。

### SwiftUI Grid

SwiftUI の `Grid` は structural row に近い書き味を持ちます。

```swift
Grid {
    GridRow {
        Text("Name")
        TextField("Name", text: $name)
    }
}
```

Surface B はこの読み味に近いです。ただし SwiftUI には alignment guides や
view builder の型システムがあり、Wasamo M3 の `.ui` とは支えている言語機構が違います。

### Compose

Compose の uniform grid は `LazyVerticalGrid` のように data-driven collection
API に寄ります。一方、custom layout / Row / Column の組み合わせで structural に
組むこともできます。

Surface B は Compose の builder 的な読み味に近いですが、Wasamo では Phase 7 まで
iteration がないため、まず fixed structural children をどう扱うかが中心になります。

### WPF / CSS Grid

WPF / CSS Grid は parent が tracks を持ち、child が placement metadata を持つため、
Surface B とはかなり違います。Surface A2 はこの track + placement model を保ちつつ
metadata を `Cell` wrapper に閉じる中間案です。

Surface B は「親が座標系を持ち、子がそこへ配置される」よりも、
「document tree そのものが table-like 2D structure である」という model です。

## 将来の拡張性

### Component extension model

Surface B は、**structural child node kind の最初の built-in precedent** になります。

良い点:

- content widget に parent-specific metadata を足さずに済む。
- 将来 custom layout が `Item`, `Slot`, `Pane`, `Section` のような structural
  child node を持つ model へ伸ばしやすい。
- `.ui` の nesting が owner の意図を直接表すため、authoring の心理的負荷が低い。

注意点:

- custom component が自分専用の structural child node kind を定義できる仕組みを
  将来どう作るかが問題になる。
- `Row` / `Cell` が Grid 専用 node なのか、一般 DSL node kind なのかを曖昧にすると
  name resolution が重くなる。
- parent-scoped child metadata の precedent は作らないため、DockPanel 的な
  `dock: top` surface へは別の設計が必要になる。

### Iteration

**前提**: Grid は M3 の iteration 対象ではありません。採択済み target-app
pre-doc ([spec.md](../../../requirements/spec.md)) は collection-driven な
「List 責務」を WrapPanel + ZStack + 繰り返し生成 grammar に分解し、Grid を
そこに含めていません。Phase 7 iteration の M3 対象は WrapPanel-backed な
thumbnail collection であり、Phase 7 が Grid children を生成することは
ありません。以下は M3 では発火しない post-M3 の可能性として、surface 比較の
foreclosure check(将来 iteration を構造的に塞がないか)の材料に留めます。

post-M3 で仮に Surface B を iterate するなら、生成単位が問題になります。

Row を生成する例:

```wasamo-ui
Grid {
  for item in items {
    Row {
      Cell { width: 180 Text { text: {item.label} } }
      Cell { width: 1* TextInput { value: {item.value} } }
    }
  }
}
```

Cell を生成する例:

```wasamo-ui
Grid {
  Row {
    for item in items {
      Cell {
        width: 1*
        Card { title: {item.title} }
      }
    }
  }
}
```

Surface B は structural iteration と相性が良い一方、shared track sizing の
reconciliation と iteration が絡むと診断が難しくなる可能性があります。
たとえば generated rows が互いに異なる `width` を出した時、どの時点で
reject するのかは、post-M3 が Grid iteration を入れる場合に考える必要が
あります(M3 では発生しません)。

### Track sizing

Surface B は `columns:` を持たないため、track-list grammar を導入しません。
その代わり、track sizing は `Cell { width: ... }` / `Row { height: ... }` へ分散します。

良い点:

- 既存の attribute value plumbing に乗せやすい。
- string-encoded track-list の問題は発生しない。

注意点:

- shared column width が複数 row の `Cell` attribute に分散する。
- B-reject 以外の rule を選ぶと、Grid としての shared track identity が弱くなる。
- `auto`, `minmax`, bindable width を足す場合、各 `Cell` の value と
  shared track identity の関係を再確認する必要がある。

## Row spanning consideration

M3 で row-span を admit するかどうかは
[framing.md DD-M3-P5-003 per-axis admission sub-issue](../framing.md)
で決まる scope decision で、ここでは確定させません。Surface B は column-span
は intra-`Row` の document order で自然に閉じますが、row-span は document
tree を跨ぐため、coordinate family と違って per-surface の追加 rule が
必要になります。

`Cell { row-span: 2 }` を `Row[i]` に書くと、`Row[i+1]` の特定列が上から
占有されます。次の `Row` の children をどう書くかで 2 つの rule があり、
どちらも Surface B の核である "document structure mirrors visible structure"
に異なる影響を与えます。

**Option B-implicit (HTML rowspan-like skip):**

```wasamo-ui
Grid {
  Row {
    Cell { width: 180 row-span: 2 Box { fill: #243447ff Text { text: "Sidebar" } } }
    Cell { width: 1* Text { text: "Header" } }
  }
  Row {
    Cell { width: 1* Text { text: "Body" } }
  }
}
```

`Row[1]` の `Cell` は実は column 1 に着地します。`Row[1]` だけを読んでも
visible 列位置がわからず、author / reader は上 row の row-span を把握する
必要があります。

**Option B-explicit (placeholder):**

```wasamo-ui
Grid {
  Row {
    Cell { width: 180 row-span: 2 Box { fill: #243447ff Text { text: "Sidebar" } } }
    Cell { width: 1* Text { text: "Header" } }
  }
  Row {
    Cell { covered }
    Cell { width: 1* Text { text: "Body" } }
  }
}
```

local readability は保たれますが、author が上 row の coverage を手動で
tracking する必要があります。iteration template が `Row` を生成する場合は
template 側に coverage 計算を持つことになります。

加えて、B-reject の canonical non-spanning row 推論が row-span と干渉します。
row-span を持つ row は non-spanning 扱いしづらいため、canonical row の決定
アルゴリズムは「column-span も row-span も持たない最初の row」へ複雑化
します。

含意:

- M3 で defer する場合、Surface B の structural readability と canonical-row
  推論の単純さは維持できる。
- M3 で admit する場合、implicit / explicit のどちらの rule にも記述上の
  コストがあり、Surface B の "document structure mirrors visible structure"
  という強みは片側の料金を払う形になる。
- deferral は問題を消すのではなく先送りする (将来 admit する時に同じ rule
  choice が再浮上する)。

## 判断材料

Surface B は、オーナーが示した `Grid { Row { Cell { ... } } }` の書き味に
最も近いです。visible structure をそのまま書けることは強い魅力です。

ただし Grid として重要な shared track sizing は、Surface B では自然発生しません。
B-reject のような reconciliation rule を受け入れられるかが、Surface B を
本当に Grid として採れるかの焦点です。これを嫌うなら、親 Grid の track-list を
受け入れられる場合は D、explicit coordinate placement も欲しい場合は A2、
structural authoring と shared tracks の対称性を重視する場合は C の方が安定します。
