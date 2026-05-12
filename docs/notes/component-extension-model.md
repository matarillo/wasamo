---
title: Component extension model — future note
status: live
created: 2026-05-12
related:
  - docs/notes/dsl-grammar.md
  - docs/notes/m3/m3-target-app-wireframes.html
---

# Component extension model — future note

このノートは、Wasamo の built-in component set の外側に custom component を
追加するための将来モデルを残す parking lot である。

M3 の target app framing では `layout primitive` という語を使っているが、
これは M3 public DSL draft で Wasamo が標準提供する built-in layout component を
指す。Wasamo の component set が将来それらだけに閉じるという意味ではない。

ただし、component extension model 自体は M3 の acceptance scope には含めない。
M3 では、将来の extension model を妨げない terminology / name resolution /
reserved syntax を保つことに留める。

---

## Motivation

Wasamo が実用 UI DSL になるなら、標準 component だけで全ての layout / widget /
rendering need を覆うことはできない。アプリ開発者やサードパーティライブラリ提供者が
custom component を定義し、DSL から import して使える経路が将来的には必要になる。

想定される custom component は、単なる visual wrapper だけではない。子要素構造、
measure / arrange、rendering、event handling、state exposure を持つ component も
将来候補になる。

---

## Non-goals for M3

次は M3 の scope には含めない。

- custom component の実装 API
- `.uic` などの component definition file format
- measure / arrange override protocol
- native component ABI
- C / Rust / 他言語 binding
- component registry
- package / library distribution
- import resolution
- versioning / compatibility policy
- sandboxing / safety policy

M3 が扱うのは built-in component の DSL surface と public draft である。
custom component model は future hook として残す。

---

## Conceptual Model

将来モデルは少なくとも次の概念を持ちうる。

- **built-in component**: Wasamo が標準提供し、public DSL draft に normative に
  記載する component。
- **custom component**: アプリまたはサードパーティライブラリが提供する component。
- **component implementation**: measure / arrange / render / event / state exposure を
  実装する本体。
- **child contract**: custom component が子要素を受け取るか、どの slot / collection に
  受け取るかを定義する契約。
- **component registry**: runtime が component name から implementation を引く仕組み。
- **DSL import surface**: `.ui` から built-in / custom component を名前解決する仕組み。
- **native binding**: C ABI または他言語 binding で component implementation を登録する境界。

---

## Open Questions

### Component Identity

- built-in component と custom component は同じ名前空間に置くか。
- 名前衝突時は built-in を優先するか、import alias を必須にするか。
- component name は package name / module path を含むか。

### Definition Surface

- custom component を `.ui` で定義できるようにするか。
- `.uic` のような別ファイル形式を導入するか。
- DSL-defined component と native component を同じ import surface で扱うか。

### Layout Protocol

- custom component は measure / arrange を override できるか。
- 子要素の desired size / arranged rect をどの型で受け渡すか。
- DPI scaling、layout invalidation、async measure をどう扱うか。

### Runtime / ABI

- component implementation は runtime にいつ登録されるか。
- C ABI を primary boundary にするか、Rust / 他言語 binding を別途持つか。
- custom component から state mutation / event emission を許すか。
- safety / lifetime / threading をどう扱うか。

### Packaging

- component library はどの単位で配布されるか。
- version compatibility を DSL spec version とどう結びつけるか。
- public draft は extension model のどこまでを normative にするか。

---

## M3 Implications

M3 では component extension model を解かない。ただし、次の点は M3 surface 設計時に
邪魔しないよう注意する。

- `layout primitive` は built-in component を指す用語として使い、DSL の表現力が
  それらだけに閉じるとは書かない。
- built-in component 名は、将来の import / namespace と衝突しにくい形にする。
- M3 で import 構文を予約する必要があるかは、DSL grammar の open question として
  別途扱う。
- M3 public draft は custom component model を normative に書かない。

---

## Revisit Triggers

- M4+ で standard component set を拡張するだけでは足りない concrete app / library
  use case が出た時。
- user-defined layout / custom measure-arrange が必要になった時。
- native binding / C ABI freeze / package distribution を扱う milestone に入る時。
- DSL import / name resolution を public draft で予約する必要が出た時。
