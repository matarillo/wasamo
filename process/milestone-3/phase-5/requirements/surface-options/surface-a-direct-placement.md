---
title: M3-Phase 5 Grid Surface A — direct child placement
status: draft
target-phase: M3-Phase 5
role: supplemental requirement note
---

# Surface A — track-list + direct child placement

This is a supplemental owner-alignment note for
[../framing.md](../framing.md). It expands the `.ui` writing style,
ecosystem contrast, and future-extension implications for one candidate
Grid surface. It is not an ADR recommendation.

Surface A は、`Grid` が shared track list を持ち、direct content child が
`row` / `column` / span / alignment を parent-scoped metadata として持つ案です。
WPF / CSS Grid に近い mental model ですが、Wasamo では full attached-property
machinery や CSS cascade を入れるわけではありません。

近接案として Surface A2 があります。A2 は `columns:` / `rows:` は同じですが、
placement / span / alignment を content widget ではなく `Cell` wrapper に置きます。
このファイルは、あえて wrapper を置かず content child に直接 metadata を置く案だけを
扱います。

## 書き味イメージ

### 最小の 2 column Grid

```wasamo-ui
Grid {
  columns: 180 1*
  rows: 48

  Text {
    row: 0
    column: 0
    text: "Name"
  }

  TextInput {
    row: 0
    column: 1
    value: {profile.name}
  }
}
```

書き味の特徴:

- shared column sizing は `columns:` 1 箇所で決まる。
- child の visible position は `row` / `column` metadata を読まないと分からない。
- `Text` / `TextInput` 自体に layout-specific metadata が乗って見える。

### Weighted star と spanning

```wasamo-ui
Grid {
  columns: 96 1* 2*
  rows: 64 1* 48

  Text {
    row: 0
    column: 0
    column-span: 3
    text: "Project"
  }

  Box {
    row: 1
    column: 0
    fill: #334455
  }

  Box {
    row: 1
    column: 1
    fill: #557799
  }

  Box {
    row: 1
    column: 2
    fill: #88aacc
  }

  Text {
    row: 2
    column: 0
    column-span: 3
    text: "Ready"
  }
}
```

この surface では spanning が child metadata として自然に書けます。
header / footer のような横断要素は短いです。

### Alignment を含む例

```wasamo-ui
Grid {
  columns: 180 1* 120
  rows: 40 40

  Text {
    row: 0
    column: 0
    h-align: end
    v-align: center
    text: "Email"
  }

  TextInput {
    row: 0
    column: 1
    column-span: 2
    value: {account.email}
  }

  Button {
    row: 1
    column: 2
    h-align: end
    v-align: center
    text: "Save"
  }
}
```

alignment の carrier は content widget 側になります。ただしこれは通常 widget
catalog property ではなく、Grid parent の direct child にだけ意味を持つ
metadata として扱うのが安全です。

### Gallery proof slice

```wasamo-ui
Grid {
  columns: 96 1* 96
  rows: 64 1* 48

  Text {
    row: 0
    column: 0
    column-span: 3
    text: "Grid proof"
  }

  Box {
    row: 1
    column: 0
    fill: #263340
  }

  Box {
    row: 1
    column: 1
    fill: #426985
  }

  Button {
    row: 1
    column: 2
    text: "Open"
  }

  Text {
    row: 2
    column: 0
    column-span: 3
    text: "fixed + star + span"
  }
}
```

Phase 5 visible proof の最低線である 3 rows x 3 columns / 5 children を
短く表現できます。

## Invalid shape examples

### Duplicate cell claim

```wasamo-ui
Grid {
  columns: 1*
  rows: 1*

  Box { row: 0 column: 0 fill: #333333 }
  Box { row: 0 column: 0 fill: #666666 }
}
```

Same-cell overlap は reject。intentional overlay は Phase 6 ZStack の責務です。

### Span exceeds declared track count

```wasamo-ui
Grid {
  columns: 1* 1*
  rows: 1*

  Text {
    row: 0
    column: 1
    column-span: 2
    text: "too wide"
  }
}
```

`column + column-span <= column count` を満たさないので reject。

### Missing placement in multi-child Grid

```wasamo-ui
Grid {
  columns: 1* 1*
  rows: 1*

  Text { text: "A" }
  Text { row: 0 column: 1 text: "B" }
}
```

Surface A で auto-placement を出さないなら、multi-child Grid では explicit
placement required にするのが一貫します。

## Ecosystem contrast

### WPF

WPF は `RowDefinition` / `ColumnDefinition` を親 Grid に置き、child には
`Grid.Row` / `Grid.Column` attached property を書きます。

```xml
<Grid>
  <Grid.ColumnDefinitions>
    <ColumnDefinition Width="180" />
    <ColumnDefinition Width="*" />
  </Grid.ColumnDefinitions>
  <TextBlock Grid.Row="0" Grid.Column="0" Text="Name" />
  <TextBox Grid.Row="0" Grid.Column="1" />
</Grid>
```

Surface A は child-side placement という意味では WPF に近いです。
違いは、Wasamo Phase 5 では full attached-property system を作らず、
Grid direct child にだけ効く built-in metadata として始める点です。

### CSS Grid

CSS Grid は parent に `grid-template-columns` を置き、child に `grid-row` /
`grid-column` を style property として置きます。

```css
.form {
  display: grid;
  grid-template-columns: 180px 1fr;
}
.name-label {
  grid-column: 1;
}
.name-input {
  grid-column: 2;
}
```

Surface A は track-list + placed children という構造が似ています。
ただし Wasamo には CSS cascade / selector / formatting context がないため、
「どの element にでも grid-column を書けるが、効くかは親次第」という model は
そのまま採りません。

### QML / Slint

QML の `GridLayout` では child に `Layout.row` / `Layout.column` のような
layout-specific property を書けます。

```qml
GridLayout {
    columns: 2
    Text { text: "Name"; Layout.row: 0; Layout.column: 0 }
    TextField { Layout.row: 0; Layout.column: 1 }
}
```

Surface A はこの文化にも近いです。違いは、Wasamo では `Layout.*` のような
general attached / grouped property namespace をまだ導入しない点です。

### SwiftUI / Compose

SwiftUI / Compose は structural builder や scope / modifier に寄ります。
子の placement metadata を parent scope が提供する形にできますが、Wasamo M3 には
scope receiver や modifier chain がまだありません。

Surface A は、そうした future scope system を待たずに短く書ける一方、
将来 scope / modifier model を入れる場合には、Grid metadata をどう移行するかを
ADR で意識しておく必要があります。

## 将来の拡張性

### Component extension model

Surface A は、**parent-scoped child metadata の最初の built-in precedent** になります。

良い点:

- 将来 custom layout が `slot`, `dock`, `area`, `order` などの child contract を
  持つ道を想像しやすい。
- WPF attached property 的な意味論へ発展できる。
- irregular layout や named-area 的な拡張に強い。

注意点:

- `row` / `column` を通常 widget property として Text / Box / Button に追加すると、
  component-extension-model と相性が悪くなる。
- name collision を避けるため、metadata の scope を parent kind に結びつける必要がある。
- custom layout が同じ mechanism を使えるか、built-in special case に留めるかを
  将来決める必要がある。

### Iteration

Surface A で Phase 7 iteration を入れる場合、iteration template は placed child を
生成します。

```wasamo-ui
Grid {
  columns: 1* 1* 1*
  rows: 120 120 120

  for item in items {
    Card {
      row: {item.index / 3}
      column: {item.index % 3}
      title: {item.title}
    }
  }
}
```

auto-placement を入れないなら、iteration 側が row / column を計算する必要があります。
これは明示的で predictable ですが、authoring はやや重くなります。

### Track syntax

Surface A では `columns:` / `rows:` の track-list grammar が中心になります。
first-class track-list value を採るなら、将来の `auto`, `minmax`, named lines,
bindable track pieces を足しやすいです。

```wasamo-ui
Grid {
  columns: auto minmax(180, 1*) 96
}
```

string-encoded にすると、この拡張は string mini-language に積み上がります。
v1.0 framework の tooling / diagnostics を重視するなら、first-class grammar の方が
説明しやすいです。

## Row spanning consideration

M3 で row-span を admit するかどうかは
[framing.md DD-M3-P5-003 per-axis admission sub-issue](../framing.md)
で決まる scope decision で、ここでは確定させません。Surface A は coordinate
based のため、column-span と row-span は完全に対称です。

admit する場合の `.ui` 例:

```wasamo-ui
Grid {
  columns: 180 1*
  rows: 1* 1*

  Box {
    row: 0
    column: 0
    row-span: 2
    fill: #243447ff
    Text { text: "Sidebar" }
  }

  Text { row: 0 column: 1 text: "Header" }
  Text { row: 1 column: 1 text: "Body" }
}
```

`(row, column, row-span, column-span)` の rectangle conflict check が
そのまま両軸に効きます。

含意:

- 新しい surface 概念は発生しない。column-span 用の validation / arrange
  ロジックがそのまま row-span に転用できる。
- M3 で defer する場合も、`row-span:` 属性名を予約し `wasamoc check` /
  runtime validation で reject するだけで surface 構造は変わらない。
- iteration template でも `row-span: {item.height}` のように child metadata
  1 行で出せる。
- 将来 named lines や `auto` row tracks を入れた場合も、span 処理は
  axis-uniform に保てる。

row-span 周りの追加 surface コストが coordinate family ではほぼゼロである
ことが、structural family (B / D / C) との対比点になります。

## 判断材料

Surface A は、irregular placement、spanning、shared track sizing の明快さでは
最も軽いです。一方で、Wasamo に parent-scoped child metadata という precedent を
作ります。この precedent を component-extension-model の足場として意識的に置けるなら
有力です。

逆に、content widget に `row` / `column` が生えることを避けたいが、親 Grid の
track-list は維持したいなら A2 が近い比較対象です。`.ui` の document structure が
visible structure と一致することを最優先するなら、B / D / C の方が読み味は近くなります。
