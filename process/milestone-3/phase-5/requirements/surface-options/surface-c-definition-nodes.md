---
title: M3-Phase 5 Grid Surface C — definition nodes + structural rows
status: draft
target-phase: M3-Phase 5
role: supplemental requirement note
---

# Surface C — definition nodes + structural rows

This is a supplemental owner-alignment note for
[../framing.md](../framing.md). It expands the `.ui` writing style,
ecosystem contrast, and future-extension implications for one candidate
Grid surface. It is not an ADR recommendation.

Surface C は、`ColumnDefs` / `RowDefs` で shared tracks を先に定義し、
content は `Row` / `Cell` で structural に書く案です。

Surface A / A2 の「parent tracks + placed children」と Surface B の
「document structure mirrors visible structure」の中間です。shared track sizing は
definition nodes によって construction で解決し、content widget に `row` / `column`
metadata は付けません。

近接案として Surface D があります。D は columns だけを Grid-level に置き、rows は
structural `Row` に残します。C は rows / columns の両方を definition nodes に寄せる
より対称的な案です。

## 書き味イメージ

### 最小の 2 column Grid

```wasamo-ui
Grid {
  ColumnDefs {
    ColumnDef { width: 180 }
    ColumnDef { width: 1* }
  }

  RowDefs {
    RowDef { height: 48 }
  }

  Row {
    Cell {
      Text { text: "Name" }
    }

    Cell {
      TextInput { value: {profile.name} }
    }
  }
}
```

書き味の特徴:

- shared column sizing は `ColumnDefs` 1 箇所で決まる。
- content は `Row` / `Cell` nesting で visible structure を mirror する。
- 小さい Grid では definition section がやや重い。

### Weighted star と spanning

```wasamo-ui
Grid {
  ColumnDefs {
    ColumnDef { width: 96 }
    ColumnDef { width: 1* }
    ColumnDef { width: 2* }
  }

  RowDefs {
    RowDef { height: 64 }
    RowDef { height: 1* }
    RowDef { height: 48 }
  }

  Row {
    Cell {
      column-span: 3
      Text { text: "Project" }
    }
  }

  Row {
    Cell { Box { fill: #334455 } }
    Cell { Box { fill: #557799 } }
    Cell { Box { fill: #88aacc } }
  }

  Row {
    Cell {
      column-span: 3
      Text { text: "Ready" }
    }
  }
}
```

Surface B と同じ structural readability を持ちながら、column widths は
`ColumnDefs` から一意に決まります。

### Alignment を含む例

```wasamo-ui
Grid {
  ColumnDefs {
    ColumnDef { width: 180 }
    ColumnDef { width: 1* }
    ColumnDef { width: 120 }
  }

  RowDefs {
    RowDef { height: 40 }
    RowDef { height: 40 }
  }

  Row {
    Cell {
      h-align: end
      v-align: center
      Text { text: "Email" }
    }

    Cell {
      column-span: 2
      TextInput { value: {account.email} }
    }
  }

  Row {
    Cell { }
    Cell { }
    Cell {
      h-align: end
      v-align: center
      Button { text: "Save" }
    }
  }
}
```

alignment の carrier は Surface B と同じく `Cell` です。

### Gallery proof slice

```wasamo-ui
Grid {
  ColumnDefs {
    ColumnDef { width: 96 }
    ColumnDef { width: 1* }
    ColumnDef { width: 96 }
  }

  RowDefs {
    RowDef { height: 64 }
    RowDef { height: 1* }
    RowDef { height: 48 }
  }

  Row {
    Cell {
      column-span: 3
      Text { text: "Grid proof" }
    }
  }

  Row {
    Cell { Box { fill: #263340 } }
    Cell { Box { fill: #426985 } }
    Cell { Button { text: "Open" } }
  }

  Row {
    Cell {
      column-span: 3
      Text { text: "fixed + star + span" }
    }
  }
}
```

visible proof の構造は長くなりますが、track definitions と content rows の
役割分担はかなり明確です。

## Invalid shape examples

### Too many cells in a row

```wasamo-ui
Grid {
  ColumnDefs {
    ColumnDef { width: 1* }
    ColumnDef { width: 1* }
  }

  RowDefs {
    RowDef { height: 48 }
  }

  Row {
    Cell { Text { text: "A" } }
    Cell { Text { text: "B" } }
    Cell { Text { text: "C" } }
  }
}
```

2 columns しか定義されていないため、3 つ目の `Cell` は reject。

### Span exceeds declared column count

```wasamo-ui
Grid {
  ColumnDefs {
    ColumnDef { width: 1* }
    ColumnDef { width: 1* }
  }

  RowDefs {
    RowDef { height: 48 }
  }

  Row {
    Cell {
      column-span: 3
      Text { text: "too wide" }
    }
  }
}
```

`column-span: 3` は declared column count を超えるので reject。

### Multiple children in one Cell

```wasamo-ui
Grid {
  ColumnDefs {
    ColumnDef { width: 1* }
  }

  RowDefs {
    RowDef { height: 48 }
  }

  Row {
    Cell {
      Box { fill: #333333 }
      Box { fill: #666666 }
    }
  }
}
```

Surface B と同じく、`Cell` は single-child wrapper として扱うのが
M3 A2 の 1 cell 1 child と整合します。

## Ecosystem contrast

### WPF

WPF は `Grid.RowDefinitions` / `Grid.ColumnDefinitions` を持ち、content child には
`Grid.Row` / `Grid.Column` attached property を書きます。

Surface C は definition nodes の部分では WPF に近いです。
ただし content placement は attached property ではなく document structure で決まるため、
WPF の full Grid model とは違います。

```xml
<Grid>
  <Grid.ColumnDefinitions>
    <ColumnDefinition Width="180" />
    <ColumnDefinition Width="*" />
  </Grid.ColumnDefinitions>
  <TextBlock Grid.Column="0" Text="Name" />
  <TextBox Grid.Column="1" />
</Grid>
```

Wasamo Surface C なら:

```wasamo-ui
Grid {
  ColumnDefs {
    ColumnDef { width: 180 }
    ColumnDef { width: 1* }
  }
  RowDefs {
    RowDef { height: 48 }
  }
  Row {
    Cell { Text { text: "Name" } }
    Cell { TextInput { value: {name} } }
  }
}
```

### HTML table + colgroup

Surface C は HTML table の `colgroup` + rows にも近いです。

```html
<table>
  <colgroup>
    <col style="width: 180px" />
    <col />
  </colgroup>
  <tr>
    <td>Name</td>
    <td><input /></td>
  </tr>
</table>
```

shared column sizing は definition section に寄せ、content は rows/cells で書く、
という分担が似ています。

### SwiftUI / Compose

SwiftUI / Compose の structural builder と似た読み味はありますが、
Surface C は track definitions を explicit node として hoist する点が特徴です。
SwiftUI / Compose では API parameter や layout object として columns を渡すことが多く、
Wasamo Surface C のように definition child node として書くかは別設計です。

### CSS Grid

CSS Grid は definition を parent style (`grid-template-columns`) に書き、content
は child placement metadata で配置します。Surface C は shared tracks を親側で
一元化する点は似ていますが、placement は structural rows/cells で行うため、
CSS Grid の named line / grid area 的な model とは違います。

## 将来の拡張性

### Component extension model

Surface C は、**definition nodes + structural child node kind** の最初の
built-in precedent になります。

良い点:

- content widget に parent-specific metadata を足さない。
- shared container shape を definition section として明示できる。
- 将来 custom layout が `Slots`, `Areas`, `Regions`, `Breakpoints` のような
  definition nodes を持つ model へ伸ばしやすい。

注意点:

- DSL に「container の child だが visible child ではない definition node」という
  pattern が入る。
- custom component が definition nodes を定義できるようにする場合、child kind の
  namespace / ordering / validation を設計する必要がある。
- Surface A 的な parent-scoped child metadata や Surface A2 的な placed wrapper の
  precedent は作らないため、DockPanel の `dock: top` のような短い child contract には
  別の surface が必要になる。

### Iteration

**前提**: Grid は M3 の iteration 対象ではありません。採択済み target-app
pre-doc ([spec.md](../../../requirements/spec.md)) は collection-driven な
「List 責務」を WrapPanel + ZStack + 繰り返し生成 grammar に分解し、Grid を
そこに含めていません。Phase 7 iteration の M3 対象は WrapPanel-backed な
thumbnail collection であり、Phase 7 が Grid children を生成することは
ありません。以下は M3 では発火しない post-M3 の可能性として、surface 比較の
foreclosure check(将来 iteration を構造的に塞がないか)の材料に留めます。

post-M3 で仮に Surface C を iterate するなら、track definitions は固定し、content rows
だけを生成できます。

```wasamo-ui
Grid {
  ColumnDefs {
    ColumnDef { width: 180 }
    ColumnDef { width: 1* }
  }

  for field in fields {
    Row {
      Cell { Text { text: {field.label} } }
      Cell { TextInput { value: {field.value} } }
    }
  }
}
```

これは form / settings pane と相性が良いです。
一方で, thumbnail grid のように cells を row-major に大量生成したい場合は、
iteration が row chunking をどう表現するかが課題になります。

### Track syntax

Surface C では `ColumnDef { width: ... }` / `RowDef { height: ... }` が track value の
carrier になります。first-class track-list grammar は不要です。

良い点:

- track ごとの future metadata を足しやすい。
- `auto`, `minmax`, bindable width などを `width:` value 側に自然に追加できる。
- source location / diagnostics は definition node 単位で出しやすい。

注意点:

- Grid 1 つに対して definition nodes が必須になるため、small Grid の boilerplate が増える。
- content rows と definition rows の ordering rule を決める必要がある。
- `RowDefs` と content `Row` が別物であることを author に理解させる必要がある。

## Row spanning consideration

M3 で row-span を admit するかどうかは
[framing.md DD-M3-P5-003 per-axis admission sub-issue](../framing.md)
で決まる scope decision で、ここでは確定させません。Surface C は
`ColumnDefs` / `RowDefs` で tracks を hoist するため、row count も
definition から確定します。これは Surface B / D の row-span 議論と次の差を
もたらします:

- shared column reconciliation 問題は Surface B / D と同じく無関係 (column
  widths は `ColumnDefs` で確定)。
- row-span の bound check (`row + row-span <= row count`) は `RowDefs` の
  数と比較するだけで済むため、generated content rows の数に依存しない。
  Surface B / D では content `Row` 数を数える必要があり、iteration 経由の
  Grid では実行時依存になりやすい。

implicit vs explicit の `.ui` 上の rule choice は Surface B / D と同じく
必要です。

**Option C-implicit:**

```wasamo-ui
Grid {
  ColumnDefs {
    ColumnDef { width: 180 }
    ColumnDef { width: 1* }
  }
  RowDefs {
    RowDef { height: 1* }
    RowDef { height: 1* }
  }

  Row {
    Cell { row-span: 2 Box { fill: #243447ff Text { text: "Sidebar" } } }
    Cell { Text { text: "Header" } }
  }
  Row {
    Cell { Text { text: "Body" } }
  }
}
```

**Option C-explicit:**

```wasamo-ui
Grid {
  ColumnDefs {
    ColumnDef { width: 180 }
    ColumnDef { width: 1* }
  }
  RowDefs {
    RowDef { height: 1* }
    RowDef { height: 1* }
  }

  Row {
    Cell { row-span: 2 Box { fill: #243447ff Text { text: "Sidebar" } } }
    Cell { Text { text: "Header" } }
  }
  Row {
    Cell { covered }
    Cell { Text { text: "Body" } }
  }
}
```

含意:

- M3 で defer する場合、Surface C の structural readability は維持できる。
- M3 で admit する場合、implicit / explicit の rule choice は Surface B / D
  と同じだが、bound check の declarative さは Surface C の方が強い
  (`RowDefs` 数で静的に閉じる)。
- definition nodes の boilerplate 負担を払う見返りに、row-span を含む
  validation surface は他 structural surface より静的に閉じやすい。

## 判断材料

Surface C は、shared track sizing を重視しつつ structural authoring も残したい場合の
安定寄りの選択肢です。Surface B の reconciliation fragility を避けられる一方、
definition nodes という新しい DSL pattern と boilerplate を受け入れる必要があります。

「Grid は多少 verbose でも、rows / columns を対称な definition model で扱いたい」なら
Surface C は候補になります。「小さい Grid を軽く書きたい」「definition section を
増やしたくない」なら、A / A2 / B / D の方が書き味は軽くなります。特に D は
shared columns と structural rows の両立を C より軽く試す案です。
