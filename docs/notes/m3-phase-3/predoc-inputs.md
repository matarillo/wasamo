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
