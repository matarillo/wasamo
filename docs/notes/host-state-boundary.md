---
title: Host state boundary notes
status: live
created: 2026-06-11
related:
  - ../dsl_spec.md
  - ../architecture.md
  - ../../process/milestone-3/phase-1/decisions/dd-m3-p1-008-mutation-source-for-the-phase-1-live-propagation-evidence.md
---

# Host state boundary notes

このノートは、Wasamo v1 までに検討したい host と runtime の state 境界について、
owner intent と未設計事項を残す parking lot である。

ここに書く surface 例は API 決定ではない。C ABI、Rust / C / Zig binding、
DSL surface、IR shape、generated binding のどこに責務を置くかは、将来の ADR で
決める。

## 現状

M3 時点の `.ui` の `state` は、runtime-owned な `SignalRegistry` に
ロード時に作られる。

たとえば `state count: i32 = 0` の `0` は `.ui` / IR に含まれる初期値であり、
host が `wasamo_load_ui` に `{ count: 123 }` のような initial state bag を渡す
public API はない。

表示中に host から `state` を直接更新する public API もない。
`wasamo_set_property` や Rust binding の `Widget::set_property` は widget property を
直接書く API であり、`state` signal を名前で更新する API ではない。

`in-out property` 構文は残っているが、現状では parser surface に近い。
runtime の reactive `state`、`SignalRegistry`、public ABI と統合された
in-out state channel ではない。

Phase 6 で `IrComponent.host_props` / `host_bindings` は導入されたが、これは
Window title / backdrop / theme などの host-owned attributes と content root を
分離するための構造である。dynamic host bindings は M3 時点では rejected / deferred であり、
component state の host read/write channel ではない。

## v1 までに欲しい能力

owner intent として、v1 までに何らかの形で次を扱いたい。

- **host-supplied initial state**:
  `.ui` が宣言した state に対し、host が load / create 時に初期値を渡せる。
- **host replace / host write**:
  表示中に host が state を更新し、通常の reactive binding と同じ経路で UI が更新される。
- **in-out write-back**:
  runtime 側で確定した値、たとえば TextField の編集値、ScrollView の入力由来 offset、
  選択状態などを、author が宣言した state または host 側の observable value へ戻せる。

これらは collection 専用の要求ではない。`state count: i32` のような単値でも必要になる
general host state boundary である。

## Expected surface sketches, not decisions

以下は期待する能力を説明するためのスケッチであり、API 形状の決定ではない。

```rust
// host-supplied initial state sketch
let ui = wasamo::load_ui("gallery.ui")
    .with_state("count", 10)
    .with_state("items", vec!["a", "b", "c"])
    .build()?;

// host replace sketch
ui.set_state("count", 11)?;
ui.set_state("items", vec!["x", "y"])?;

// in-out sketch
ui.bind_state("search_text", &model.search_text)?;
```

C ABI では、名前指定 API、component handle、typed value bag、generated binding API など
複数の形がありうる。ここでは選ばない。

例を残す目的は、「host state boundary」が Window host attributes の話なのか、
widget property set の話なのか、component state の話なのかを後続作業で混同しないためである。
一方で、例を厚くしすぎると Rust builder API や名前指定 API が既成事実に見えるため、
このノートでは能力の輪郭だけに留める。

## M3 boundary

M3 中の `for` / collection binding 検討では、まず runtime-owned collection state で
cardinality-driven subtree generation を証明する方向を基本線にする。

host から collection 全体を set / replace する API は、単値 state の host write API も
未設計であるため、M3 の必須範囲には含めない。

ただし、M3 中の collection 設計は将来の host state boundary を塞がないこと。
特に、collection state の型、copy / ownership、element identity、batching、
reactive drain との関係は、将来の host replace と矛盾しない形で記録する。

## Open questions

- **Component / window identity**:
  host がどの component instance の state を指定するのか。
- **Type model**:
  `WasamoValue` を拡張するのか、typed state API を分けるのか、
  `TypedValue` を導入するのか。
- **Collection ownership**:
  host-supplied array は copy か borrow か。更新時の diff は runtime が取るのか、
  replace-only から始めるのか。
- **Scheduler semantics**:
  host write は即時 drain か、batched write か、event-loop boundary で反映か。
- **Write-back semantics**:
  in-out の runtime-origin write と host-origin write が衝突した場合の優先順位。
- **Generated bindings**:
  C ABI primitive だけを公開するのか、Rust / C / Zig binding が typed wrapper を
  生成するのか。

## Revisit triggers

- M4 input / TextField / focus model を設計するとき。
- ScrollView の wheel / drag / write-back offset を開くとき。
- M3 collection state を host から初期化・差し替えたくなったとき。
- dynamic Window title / host bindings を開くとき。
- M6 ABI freeze の前。
