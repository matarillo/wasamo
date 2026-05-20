---
title: M3-Phase 3 pre-doc inputs — M3-Phase 2 close からの前送り
status: live
created: 2026-05-20
source-phase: M3-Phase 2
target-phase: M3-Phase 3
---

# M3-Phase 3 pre-doc inputs

この note は、Phase 2 close の学びを M3-Phase 3 (WrapPanel layout
primitive) の pre-doc へ前送りするもの。単なる retrospective ではなく、
Phase 3 が Phase 2 の全 commit を読み直さなくても、この制約群から
着手できるように action-oriented に書く。

## 1. WrapPanel は Box intrinsic sizing を consume し、再定義しない

Phase 2 では、parent axis の片方が bounded の場合に
`Box { aspect: <ratio> }` が defined intrinsic size を持つようにした:
bounded axis が勝ち、もう片方の axis は ratio から導出される。Phase 3 の
WrapPanel ADR はこの behavior について `docs/dsl_spec.md` §4.9 を cite
し、WrapPanel が child に main-axis / cross-axis constraint をどう渡すかを
定義する。Box aspect algorithm を再記述しない。

具体的な pre-doc question:

- thumbnail strip で、WrapPanel は child を fixed main-axis item slot で
  measure するのか、max main-axis constraint で measure するのか、
  unbounded main-axis constraint + later arrange で扱うのか。この答えで
  `Box { aspect: 1:1 }` が thumbnail size をどう得るかが決まる。

## 2. Placeholder thumbnail は gallery asset shape の normative 形になった

Phase 2 では `Box { aspect: <ratio>; fill: <color>; Text { ... } }` を
normative pre-Image placeholder pattern にした。Phase 3 は Image-like
surface、asset pipeline、host-imperative fixture を導入せず、この shape から
gallery sub-screen を構築する。

具体的な pre-doc question:

- Phase 3 の visible proof には、どの minimal thumbnail item shape を
  使うべきか: square placeholder (`1:1`) か、mixed aspect placeholder
  か、その両方を含む fixed set か。mixed aspect item は、WrapPanel
  contract が main axis の variable child extent を意図的に support する
  場合にだけ wrapping の良い evidence になる。

## 3. Multi-child overlap は Box scope 外のまま

Phase 2 は Box の 2+ children を意図的に reject し、overlap は ZStack に
向けた。Phase 3 は thumbnail 上の label や image 上の badge のために Box
へ依存しない。ZStack が ship する前に WrapPanel item が composite
thumbnail を必要とする場合でも、Phase 3 ADR は overlap semantics を
必要としない plain child tree として保つ。

具体的な pre-doc question:

- Phase 3 gallery proof は placeholder 内の label を必要とするか。それとも
  ZStack / Image が後で composition を広げるまで、visible item は single
  Box + centred Text placeholder でよいか。

## 4. Phase 3 では spec-drafting bar が上がる

Phase 2 の spec close は、ADR-written な Box chapter の re-sync が中心
だった。Phase 3 は、acceptance text が novel normative measure-arrange
algorithm を明示する最初の M3 phase になる。pre-doc では implementation
開始前に WrapPanel spec outline の draft を置く。少なくとも次を含める:

- line formation の input / output;
- main-axis overflow behavior;
- cross-axis line sizing;
- spacing / padding treatment、またはそれらの attribute が Phase 3 scope
  外であるという明示;
- unbounded-parent behavior。特に後続の ScrollView phase 内での扱い。

## 5. Phase 3 が必要としない限り constant-only value boundary を保つ

Phase 2 は `Ratio` と `Color` を Box-internal に保ち、新しい
`PropertyValue` / ABI arm を避けた。Phase 3 も、WrapPanel 自身が
bindable value type を導入しない限り、この boundary を保つ。新しい
property が binding を必要とする場合は、per-type runtime writer、IR
literal/type surface、ABI conversion story を phase 跨ぎにせず、同じ
step で決める。

## 6. 引き継ぐ verification shape

Phase 3 は Phase 2 の分割を引き継ぐ:

- WrapPanel line breaker には pure-logic measure-arrange test;
- IR / loader test は実際に追加する new widget / property だけを対象にする;
- visible behavior が real compositor-backed widget state に依存する場合は、
  Windows-only integration test を 1 つ置く;
- gallery sub-screen は host-imperative construction ではなく
  `.ui -> wasamoc -> IR text -> wasamo_load_ui` で育てる。

Phase 2 T11 の skip guard pattern は引き続き正しい model。local developer
machine では Compositor creation が unavailable な場合に skip してよいが、
GitHub Actions では silent skip ではなく fail しなければならない。

## 7. IR-loader の defense-in-depth gate は pure validation に寄せる

Phase 2 T7 では、progress file の文面が `ir_loader::build_node` を名指し
していた invariant (Box の 2+ children reject、Ratio / Color literal の
位置 reject) を、実装上は WinRT-bound な `build_node` ではなく pure logic
の `validate()` に置いた。重要なのは「どの関数名で reject するか」では
なく、「IR load → runtime materialise 境界に入る前に同じ error class で
必ず reject されるか」だった。

Phase 3 で WrapPanel の IR-loader invariant を追加する場合も、この判断を
引き継ぐ:

- `IrNode` / `IrLiteral` だけで判定できる invariant は `validate()` 側に
  置く。Compositor が必要な `build_node` 側へ散らさない。
- C ABI から見える error class は `WASAMO_ERR_IR_MALFORMED` で揃える。
- 将来 `validate()` 抜きで widget tree を組み立てる入口を作るなら、その
  入口が同じ invariant を再保証する責務を持つ。

具体的な pre-doc question:

- WrapPanel が Phase 3 で持つ defense-in-depth invariant は何か。子数、
  property の値域、main-axis / cross-axis attribute の組み合わせなどを、
  `wasamoc check` と runtime `validate()` の両方でどこまで重ねるか。

## 8. layout engine は Win32/WinRT-free のまま保つ

Phase 2 T8 では、runtime の Box-internal `box_values::Ratio` をそのまま
layout engine に渡さず、`layout::Ratio` に mirror した。理由は
`layout.rs` を Win32/WinRT-free に保ち、measure-arrange algorithm を
pure-logic tests で直接 pin するため。

Phase 3 の WrapPanel line breaker は、Phase 2 よりも spec/algorithm の比重が
高い。したがって layout engine 側の入力型は、runtime domain type や
Compositor-bound object を引きずらず、pure data として閉じるのを既定にする。
もし WrapPanel 固有の domain type が必要なら、runtime 側の型を pub に広げる
前に、layout-local mirror 型で十分かを検討する。

具体的な pre-doc question:

- WrapPanel の line item / line result / spacing / padding / axis mode は、
  `layout.rs` 内の pure struct で表現できるか。runtime widget state から
  layout input へ変換する境界はどこか。

## 9. layout-time error surface は内部 `LayoutError` と ABI surface を分ける

Phase 2 T8 では `LayoutError::{BoxAspectUnboundedBoth, BoxNoExtent}` を
導入し、layout-time runtime error を type で伝播できるようにした。一方で
dedicated `WASAMO_ERR_*` ABI code、IR location plumbing、layout-error
callback / `wasamo_run_layout` のような public surface は Phase 2 scope 外と
して残した。現状の WM_SIZE / layout-dirty call site では Result が
observable ではないため、ABI を広げても使い道がない。

Phase 3 で WrapPanel 固有の no-extent / invalid-constraint / overflow policy
error が必要になった場合も、まず `LayoutError` を拡張する。public ABI error
code は、host がその error を観測できる surface を同じ phase で開く場合だけ
検討する。

具体的な pre-doc question:

- WrapPanel に layout-time author error はあるか。それは internal
  `LayoutError` で十分か、host-visible diagnostic surface を必要とするか。
  必要なら `WASAMO_ERR_*` だけでなく、error の観測 API も同時に設計する。

## 10. verification item は infrastructure 共有だけで統合しない

Phase 2 T10 では、emit-side と load-side の in-crate fixture だけでは
IR text round-trip evidence として弱いことが分かり、実際の emitted text を
runtime parse / build 側へ流す cross-crate driver を置いた。また T10 / T11 は
accessor や Compositor guard helper を共有できても、ADR verification item が
別である以上、同じ step に統合しなかった。

Phase 3 でも、verification closure item は evidence の意味で分ける:

- in-crate algorithm tests は line breaker の数値契約を pin する。
- emit / parse / load の境界を証明する場合は、実出力を流す cross-crate
  driver を置く。
- Windows-runtime integration evidence が必要な場合、内部 accessor だけで
  足りるか、production visual / render object まで読むべきかを closure item
  の性質で選ぶ。
- helper の重複削減を理由に、ADR 上で別 item の evidence を無理に統合しない。

## 11. gallery は additive growth path として扱う

Phase 2 T12 の学びは、gallery seed を canonical examples の延長ではなく、
以後の M3 surface を足していく専用の受け皿として作ることだった。
Phase 3 は既存の `examples/gallery/` を捨てて作り直すのではなく、
Box placeholder sub-screen を WrapPanel-of-Boxes へ育てる。

具体的な pre-doc question:

- Phase 3 の gallery proof は Phase 2 sub-screen を直接拡張するか、同じ
  gallery 内に Phase 3 sub-screen として並べるか。どちらにしても
  `counter-*` / `bool-demo-*` の canonical examples は太らせない。
- C / Zig host parity は Phase 8 の full gallery で広げる前提を保つか。
  Phase 3 は最低 1 host の gallery proof で十分かを ADR で確認する。

## 12. Box の future-width/height rule を WrapPanel item sizing と混ぜない

Phase 2 の ADR / spec には、将来 Box に `width` / `height` が入る場合の
forward-looking rule がある: explicit dimensions が `aspect` に勝ち、
`aspect` は informational になる。ただし `width` / `height` は
M3-Phase 2 DSL surface には存在しない。

Phase 3 が thumbnail item size を必要とするとき、これを Box の
`width` / `height` introduction と混ぜない。WrapPanel が child に与える
constraint、WrapPanel 側の item slot / spacing / padding、または gallery
fixture 側の parent bounds として決めるのかを明確にする。Box の
future-width/height rule を Phase 3 の都合で暗黙に ship しない。

具体的な pre-doc question:

- thumbnail size は WrapPanel property で決めるのか、parent constraint から
  導くのか、child intrinsic size に任せるのか。Box 自身に新しい dimension
  attribute を足す必要が本当にあるか。
