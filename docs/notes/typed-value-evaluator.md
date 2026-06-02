---
title: TypedValue evaluator unification — 検討メモと未解決事項
status: live
created: 2026-05-10
related-adrs:
  - process/milestone-2/phase-7/decisions/preamble.md
related-notes:
  - docs/notes/m2-phase-7/dd-011-pre-doc-framing.md
  - docs/notes/m2-to-m3-handover.md
---

# TypedValue evaluator unification — 検討メモと未解決事項

## 背景

DD-M2-P6-011 は、`.ui` の String property binding を
`BindingEvalContext` / `HandlerExpr` / binding evaluator 経由でどう
visible widget まで流すかを扱う。Phase 6 draft では共通の typed-value
evaluator surface、すなわち `read_typed(path) -> TypedValue` のような
形も検討したが、M2 の実装候補としてはより狭い String 経路を推奨していた。

Phase 7 の DD-011 framing では、オーナー合意として次の読みを採った。

- M2 の A6 acceptance は **demonstrative** に読む。つまり reactive binding
  path が `i32` 専用でないことを、`.ui` の String property を
  `Signal<String>` から visible widget まで通すことで実証できればよい。
- M2 close の条件として、evaluator API 全体を `TypedValue` enum の背後に
  一般化する必要はない。
- ただし `TypedValue` 案は却下ではない。DSL surface や tooling が実際に
  type-system pressure を生んだ時点で再検討する post-M2 の open question
  として残す。

このノートは、その open question を `m2-to-m3-handover.md` から分離して
記録する。handover note は M3 が継承する前提・残余を見る場所であり、
本ノートは M3 が最短の再検討契機になりうるものの、実際の解決は
M4 / M5 / post-1.0 にずれ込んでもよい cross-milestone の検討メモである。

---

## Open Question

Wasamo は将来的に、現在の parallel typed-read evaluator shape
（`get_i32`, `read_i32_tracked`, `get_string`, `read_string_tracked`,
および将来の同種メソッド）を、共通の typed-value surface に置き換えるべきか。

候補となる形は例えば:

```text
read_typed(path) -> TypedValue
```

ここで `TypedValue` は runtime / IR が共有する expression value の表現になる。

この決定は M2 では行わない。次に実際の圧力を生む milestone / phase が来た時点で、
以下のいずれかを選ぶ。

- narrow typed path を追加し続ける。
- binding evaluator の中だけに `TypedValue` を導入する。
- より広く typed-expression IR / evaluator surface を導入する。
- 既存の型集合がまだ小さいため、再度 defer する。

---

## `TypedValue` 導入に賛成する理由

- DSL に `bool`, `f32`, dimension, color などの scalar property type が増えた時、
  単一の evaluator value model のほうが拡張しやすい可能性がある。
- public `.ui` spec が expression type system を normative に書く段階では、
  typed-value IR によって spec と runtime の関係を明示しやすい。
- diagnostics / LSP などの tooling は、多数の ad hoc evaluator method より、
  単一の expression-value 語彙を持つほうが扱いやすい可能性がある。
- "type-agnostic" が、実態としては型ごとに手作業で method と IR variant を
  増やすだけの "manually type-enumerated" に drift するのを防げる。

## 早期導入に反対する理由

- M2 で必要なのは `i32` と `String` のみであり、完全に generic な value surface は
  現時点の acceptance pressure より広い。
- handler evaluation、binding evaluation、`EvalContext` implementor、test stub、
  IR tooling にまたがって blast radius が大きい。
- M3 の public DSL spec や M5 の tooling から十分な証拠が出る前に central
  `TypedValue` を導入すると、早すぎる type-system shape を固定してしまう。
- 型集合が小さい間は、parallel typed-read pattern のほうが読みやすく、
  実装意図も明確である。

---

## 再評価トリガ

次のいずれかが起きたら本ノートを再読し、必要なら ADR を起こす。

1. **第3の scalar property type が導入される。**
   `bool`, `f32`, dimension, color など、`i32` / `String` 以外の型を追加する時は、
   さらに parallel typed-read path を増やす案と、共通 `TypedValue` 表現へ移る案を
   比較する。

2. **List/Grid item context binding が typed value を必要とする。**
   M3 の List item template や Grid cell expression が `item`, `index`,
   row / column coordinate などの context value を導入する場合、それらを既存の
   `EvalContext` method family に入れるのか、typed expression-value surface が
   必要なのかを判断する。

3. **binding result を String 化せず runtime value として返す必要が出る。**
   numeric layout value、boolean visibility flag、color、dimension など、
   binding expression が runtime value を直接生成する必要が出た時は、
   `evaluate_binding() -> String` が中心でよいか再検討する。

4. **public DSL spec または LSP が型システムを必要とする。**
   M3 の public spec draft、または M5 の LSP 作業で、単一の normative
   expression type system が必要になった場合、`TypedValue` は runtime cleanup
   ではなく specification work になる可能性がある。

5. **parallel method family がノイズになり始める。**
   新しい `get_<T>` / `read_<T>_tracked` pair を足すことが意味のある設計ではなく
   機械的な増殖に見え始めたら、その違和感を再検討のシグナルとして扱う。

---

## Current Standing

M2 では `TypedValue` を実装しない。DD-M2-P6-011 は、narrow な String binding
path で `.ui` String binding の end-to-end 実証ができ、既存の integer binding
behavior を壊さないなら Accepted に進めてよい。

M2 以降は、本件を live open question として扱う。最初に real expression-typing
pressure を生む milestone が、正式に解決するか、理由を添えて defer するかを
このノートに反映する。
