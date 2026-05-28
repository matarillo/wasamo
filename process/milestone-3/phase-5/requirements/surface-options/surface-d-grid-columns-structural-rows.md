---
title: M3-Phase 5 Grid Surface D — Grid columns + structural rows
status: draft
target-phase: M3-Phase 5
role: supplemental requirement note
---

# Surface D — Grid columns + structural rows

This is a supplemental owner-alignment note for
[../framing.md](../framing.md). It expands the `.ui` writing style,
ecosystem contrast, and future-extension implications for one candidate
Grid surface. It is not an ADR recommendation.

Surface D は、`Grid` が shared column track list を持ち、rows / cells は
structural に書く案です。Surface B の `Row` / `Cell` の読み味を残しつつ、
Surface B の最大の弱点である shared column sizing reconciliation を避けます。

## 書き味イメージ

### 最小の 2 column Grid

```wasamo-ui
Grid {
  columns: 180 1*

  Row {
    height: 48

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

- column widths は `columns:` 1 箇所で決まる。
- row membership と cell membership は document structure で見える。
- row heights は `Row { height: ... }` が持つ。
- `Cell` は column document order に従って Grid-level columns を消費する。
- surface は非対称です。columns は parent-level、rows は structural。

### Weighted star と spanning

```wasamo-ui
Grid {
  columns: 96 1* 2*

  Row {
    height: 64

    Cell {
      column-span: 3
      Text { text: "Project" }
    }
  }

  Row {
    height: 1*

    Cell { Box { fill: #334455 } }
    Cell { Box { fill: #557799 } }
    Cell { Box { fill: #88aacc } }
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

Surface B と違い、spanning-only header row があっても column vector は
`columns:` から分かります。canonical non-spanning row は不要です。

### Alignment を含む例

```wasamo-ui
Grid {
  columns: 180 1* 120

  Row {
    height: 40

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
    height: 40

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

alignment の carrier は Surface B / C と同じく `Cell` です。
content widget に Grid-specific metadata は付きません。

### Gallery proof slice

```wasamo-ui
Grid {
  columns: 96 1* 96

  Row {
    height: 64

    Cell {
      column-span: 3
      h-align: center
      v-align: center
      Text { text: "Grid proof" }
    }
  }

  Row {
    height: 1*

    Cell { Box { fill: #263340 } }
    Cell { Box { fill: #426985 } }
    Cell { Button { text: "Open" } }
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

visible proof は Surface B / C と同じくらい読みやすく、column sharing は
Surface A / A2 と同じく construction で解決します。

## Invalid shape examples

### Too many cells in a row

```wasamo-ui
Grid {
  columns: 1* 1*

  Row {
    height: 48
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
  columns: 1* 1*

  Row {
    height: 48
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
  columns: 1*

  Row {
    height: 48
    Cell {
      Box { fill: #333333 }
      Box { fill: #666666 }
    }
  }
}
```

`Cell` は single-child wrapper として扱うのが M3 A2 の 1 cell 1 child と整合します。

## Ecosystem contrast

### HTML table / form builders

Surface D は table / form の「rows and cells」読み味に近いですが、column sizing は
`colgroup` 的に parent-level へ寄せます。

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

Wasamo Surface D はこの `colgroup + tr/td` の役割分担にかなり近いです。

### WPF / CSS Grid

WPF / CSS Grid は parent-level tracks と child placement metadata を持ちます。
Surface D は parent-level columns だけを借り、placement は document structure に
戻します。そのため irregular coordinate placement は A / A2 より弱いですが、forms や
settings panes の書き味はかなり自然です。

### SwiftUI / Compose

SwiftUI / Compose の structural builders は row/cell の読み味を作りやすいです。
Surface D はそこに parent-level column definition を足した形です。
lazy / adaptive grid ではありませんが、fixed structural UI の authoring には合います。

### Surface C との違い

Surface C は `ColumnDefs` / `RowDefs` を両方 definition nodes に寄せます。
Surface D は columns だけ Grid attribute に寄せ、rows は structural `Row` に残します。
つまり C より軽い代わりに、track definition model は非対称です。

## 将来の拡張性

### Component extension model

Surface D は、**parent config + structural content** の最初の built-in precedent になります。

良い点:

- content widget に `row` / `column` を付けない。
- shared columns は parent config として一元管理できる。
- future custom layout が「親に global config、子は structural sections」という
  model を採る余地を作る。
- Surface B の reconciliation rule を custom extension 側へ持ち込まなくて済む。

注意点:

- Surface A / A2 のような explicit coordinate placement precedent は弱い。
- Surface C のように rows / columns を対称な definition model としては扱わない。
- custom layout が row-like child nodes と parent-level config を同時に持つ場合の
  ordering / validation model が将来必要になる。

### Iteration

Surface D で Phase 7 iteration を入れる場合、rows を生成する形が自然です。

```wasamo-ui
Grid {
  columns: 180 1*

  for field in fields {
    Row {
      height: 48
      Cell { Text { text: {field.label} } }
      Cell { TextInput { value: {field.value} } }
    }
  }
}
```

Form / settings pane にはかなり相性が良いです。
一方で thumbnail grid のように items を row-major に流し込みたい場合は、iteration が
row chunking をどう表現するかが課題になります。

### Track syntax

Surface D は `columns:` の first-class track-list grammar を使います。
`rows:` は使わず、row height は `Row { height: ... }` に置きます。

良い点:

- shared column syntax は A / A2 と共通化できる。
- row height は structural row に近く、author の視線と合いやすい。
- `auto`, `minmax`, bindable column pieces は A / A2 と同じ path で拡張できる。

注意点:

- columns と rows の surface が非対称になる。
- row definitions を一元管理したい use case では C の方が整って見える。
- `rows:` を将来足すと D が A2 / C に近づき、surface 境界が曖昧になる可能性がある。

## Row spanning consideration

M3 で row-span を admit するかどうかは
[framing.md DD-M3-P5-003 per-axis admission sub-issue](../framing.md)
で決まる scope decision で、ここでは確定させません。Surface D は columns を
parent-level、rows を structural に置くため、column-span は intra-`Row` で
自然に閉じる一方、row-span は cross-`Row` の問題を Surface B と同じ形で
抱えます。

Surface B との違いは canonical non-spanning row 推論を持たない (column
widths は `columns:` で確定) ため、row-span が shared column reconciliation
に干渉しないことです。それでも implicit vs explicit の `.ui` 上の rule
choice は同じく必要になります。

**Option D-implicit:**

```wasamo-ui
Grid {
  columns: 180 1*

  Row {
    Cell { row-span: 2 Box { fill: #243447ff Text { text: "Sidebar" } } }
    Cell { Text { text: "Header" } }
  }
  Row {
    Cell { Text { text: "Body" } }
  }
}
```

`Row[1]` の `Cell` は column 1 に着地。上の row-span 情報を読まないと位置が
わからない点は Surface B と同じです。

**Option D-explicit:**

```wasamo-ui
Grid {
  columns: 180 1*

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

- M3 で defer する場合、Surface D の structural readability は無傷。
  "shared columns はそのまま、rows は読みやすく" という D の strength が
  そのまま保てる。
- M3 で admit する場合、implicit / explicit の rule choice は Surface B と
  同じく必要。Surface D の strength の片側 (structural rows の読みやすさ)
  が一部削られる。
- column-span が Surface D で軽く扱える分、row-span の cross-Row 問題は
  相対的に目立つ surface でもある。

## 判断材料

Surface D は、Surface B の structural authoring を保ちつつ、shared column sizing の
fragility を避ける中間案です。Surface C より軽く、A / A2 より row structure が読みやすい。

弱点は、columns と rows の扱いが非対称になること、そして irregular coordinate placement は
A / A2 より弱いことです。Wasamo v1.0 の主な Grid use case が forms / settings panes /
gallery slices なら候補になります。freeform 2D placement を重視するなら A / A2 の方が
自然です。
