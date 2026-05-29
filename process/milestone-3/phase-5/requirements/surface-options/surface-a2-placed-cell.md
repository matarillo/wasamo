---
title: M3-Phase 5 Grid Surface A2 — placed Cell wrapper
status: draft
target-phase: M3-Phase 5
role: supplemental requirement note
---

# Surface A2 — track-list + placed Cell wrapper

This is a supplemental owner-alignment note for
[../framing.md](../framing.md). It expands the `.ui` writing style,
ecosystem contrast, and future-extension implications for one candidate
Grid surface. It is not an ADR recommendation.

Surface A2 は、`Grid` が shared track list を持ち、Grid direct child の
`Cell` wrapper が `row` / `column` / span / alignment を持つ案です。
Surface A と同じく shared track sizing は `columns:` / `rows:` で一元管理しますが、
content widget には Grid-specific metadata を付けません。

## 書き味イメージ

### 最小の 2 column Grid

```wasamo-ui
Grid {
  columns: 180 1*
  rows: 48

  Cell {
    row: 0
    column: 0
    Text {
      text: "Name"
    }
  }

  Cell {
    row: 0
    column: 1
    TextInput {
      value: {profile.name}
    }
  }
}
```

書き味の特徴:

- shared column sizing は Surface A と同じく `columns:` 1 箇所で決まる。
- `row` / `column` は `Text` や `TextInput` ではなく `Cell` に閉じる。
- row structure は Surface B / D / C ほど document tree に現れない。
- `Cell` が layout-only single-child wrapper であることを仕様化する必要がある。

### Weighted star と spanning

```wasamo-ui
Grid {
  columns: 96 1* 2*
  rows: 64 1* 48

  Cell {
    row: 0
    column: 0
    column-span: 3
    Text { text: "Project" }
  }

  Cell {
    row: 1
    column: 0
    Box { fill: #334455 }
  }

  Cell {
    row: 1
    column: 1
    Box { fill: #557799 }
  }

  Cell {
    row: 1
    column: 2
    Box { fill: #88aacc }
  }

  Cell {
    row: 2
    column: 0
    column-span: 3
    Text { text: "Ready" }
  }
}
```

spanning の情報は `Cell` に集まります。content widget は、その cell の中身として
だけ扱われます。

### Alignment を含む例

```wasamo-ui
Grid {
  columns: 180 1* 120
  rows: 40 40

  Cell {
    row: 0
    column: 0
    h-align: end
    v-align: center
    Text { text: "Email" }
  }

  Cell {
    row: 0
    column: 1
    column-span: 2
    TextInput { value: {account.email} }
  }

  Cell {
    row: 1
    column: 2
    h-align: end
    v-align: center
    Button { text: "Save" }
  }
}
```

alignment の carrier は `Cell` です。これは Surface B / D / C と同じで、Surface A との
最大の違いです。

### Gallery proof slice

```wasamo-ui
Grid {
  columns: 96 1* 96
  rows: 64 1* 48

  Cell {
    row: 0
    column: 0
    column-span: 3
    h-align: center
    v-align: center
    Text { text: "Grid proof" }
  }

  Cell {
    row: 1
    column: 0
    Box { fill: #263340 }
  }

  Cell {
    row: 1
    column: 1
    Box { fill: #426985 }
  }

  Cell {
    row: 1
    column: 2
    Button { text: "Open" }
  }

  Cell {
    row: 2
    column: 0
    column-span: 3
    Text { text: "fixed + star + span" }
  }
}
```

visible proof の要件は Surface A と同じように満たせます。
書き味は A より 1 階層深く、B / D / C より row structure が弱い、という位置です。

## Invalid shape examples

### Duplicate cell claim

```wasamo-ui
Grid {
  columns: 1*
  rows: 1*

  Cell {
    row: 0
    column: 0
    Box { fill: #333333 }
  }

  Cell {
    row: 0
    column: 0
    Box { fill: #666666 }
  }
}
```

Same-cell overlap は Surface A と同じく reject。intentional overlay は Phase 6
ZStack の責務です。

### Multiple children in one Cell

```wasamo-ui
Grid {
  columns: 1*
  rows: 1*

  Cell {
    row: 0
    column: 0
    Box { fill: #333333 }
    Box { fill: #666666 }
  }
}
```

M3 A2 は 1 cell 1 child を要求しています。Surface A2 を採るなら、`Cell` は
single-child wrapper として定義するのが自然です。

### Span exceeds declared track count

```wasamo-ui
Grid {
  columns: 1* 1*
  rows: 1*

  Cell {
    row: 0
    column: 1
    column-span: 2
    Box { fill: #336699cc }
  }
}
```

`column + column-span <= column count` を満たさないので reject。

## Ecosystem contrast

### WPF

WPF は child に `Grid.Row` / `Grid.Column` attached property を書きます。
Surface A2 は座標指定の考え方は WPF に近いですが、metadata を arbitrary child ではなく
`Cell` wrapper に閉じる点が違います。

```xml
<TextBlock Grid.Row="0" Grid.Column="0" Text="Name" />
```

Surface A2 なら:

```wasamo-ui
Cell {
  row: 0
  column: 0
  Text { text: "Name" }
}
```

### CSS Grid

CSS Grid は placed child がそのまま element です。Surface A2 は placed child を
`Cell` に固定するため、CSS Grid より wrapper-oriented です。
CSS の自由度は下がりますが、Grid-specific metadata の所在は明確になります。

### HTML table

HTML table の `td` は content を包む cell wrapper です。
Surface A2 は `Cell` wrapper という点では table に近いですが、`Row` は持たず、
cell の位置は `row` / `column` metadata で決まります。つまり table-like wrapper と
Grid-like coordinate placement の hybrid です。

### SwiftUI / Compose

SwiftUI / Compose では builder scope や modifier によって layout metadata を
content から少し離して扱えます。Surface A2 は scope / modifier system を導入せずに、
その「layout contract carrier」を `Cell` という明示 wrapper で表現する案です。

## 将来の拡張性

### Component extension model

Surface A2 は、**Grid-owned child contract wrapper** の最初の built-in precedent になります。

良い点:

- `row` / `column` が Text / Box / Button の通常 property に見えない。
- future custom layout が `Pane`, `Slot`, `Item`, `Region` のような wrapper を
  持つ設計と接続しやすい。
- Surface A と同じく shared tracks は Grid parent に一元化できる。

注意点:

- parent-scoped child metadata の一般化ではなく、wrapper node kind の一般化へ寄る。
- `Cell` が Grid 専用 node なのか、将来他 layout でも使える概念なのかを決める必要がある。
- `Cell` wrapper が増えるため、小さい Grid では Surface A より verbose。

### Iteration

**前提**: Grid は M3 の iteration 対象ではありません。採択済み target-app
pre-doc ([spec.md](../../../requirements/spec.md)) は collection-driven な
「List 責務」を WrapPanel + ZStack + 繰り返し生成 grammar に分解し、Grid を
そこに含めていません。Phase 7 iteration の M3 対象は WrapPanel-backed な
thumbnail collection であり、Phase 7 が Grid children を生成することは
ありません。以下は M3 では発火しない post-M3 の可能性として、surface 比較の
foreclosure check(将来 iteration を構造的に塞がないか)の材料に留めます。

post-M3 で仮に Surface A2 を iterate するなら、iteration template は `Cell` を生成します。

```wasamo-ui
Grid {
  columns: 1* 1* 1*
  rows: 120 120 120

  for item in items {
    Cell {
      row: {item.index / 3}
      column: {item.index % 3}
      Card { title: {item.title} }
    }
  }
}
```

Surface A と同じく auto-placement に依存しないため predictable です。
ただし author は row / column 計算を `Cell` に書く必要があります。

### Track syntax

Surface A2 は Surface A と同じく `columns:` / `rows:` の track-list grammar を
使います。first-class track-list value を採るなら、`auto`, `minmax`, named lines,
bindable track pieces の将来拡張は Surface A と同じように扱えます。

## Row spanning consideration

M3 で row-span を admit するかどうかは
[framing.md DD-M3-P5-003 per-axis admission sub-issue](../framing.md)
で決まる scope decision で、ここでは確定させません。Surface A2 も Surface A と
同じく coordinate based のため、column-span と row-span は対称です。違いは
span metadata の carrier が content widget ではなく `Cell` wrapper である
ことだけです。

admit する場合の `.ui` 例:

```wasamo-ui
Grid {
  columns: 180 1*
  rows: 1* 1*

  Cell {
    row: 0
    column: 0
    row-span: 2
    Box { fill: #243447ff Text { text: "Sidebar" } }
  }

  Cell { row: 0 column: 1 Text { text: "Header" } }
  Cell { row: 1 column: 1 Text { text: "Body" } }
}
```

`Cell` の rectangle が `(row, column, row-span, column-span)` で決まり、
Surface A と同じ conflict check が再利用できます。

含意:

- 新しい surface 概念は発生しない。Surface A と同じく row-span 用の追加
  rule は不要。
- M3 で defer する場合も、`Cell` の `row-span:` を予約 + reject するだけで
  surface 構造は無傷。
- iteration template でも `Cell { row-span: ... ... }` の形で出せる。
- content widget には引き続き Grid metadata が乗らないため、row-span 拡張が
  content widget API へ波及することは Surface A 以上にない。

structural family (B / D / C) と比べたとき、A2 は A と同等に row-span に
対する surface restructure が不要な側に立ちます。

## 判断材料

Surface A2 は、Surface A の shared track sizing と irregular placement の強さを
保ちながら、content widget に `row` / `column` が生える不安をかなり減らします。
その代わり、`Cell` wrapper が必須になり、Surface B / D / C のような row-structure の
読みやすさは得られません。

「track は親に一元化したいが、Grid-specific metadata を arbitrary child に置きたくない」
なら、A2 は A / B / D / C と並べて flat に比較する価値が高い案です。
もし explicit coordinates より structural rows の読みやすさを重視するなら、Surface D が
近い比較対象になります。
