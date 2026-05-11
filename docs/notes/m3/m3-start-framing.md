---
title: M3 start framing — DSL surface
status: draft
created: 2026-05-11
related:
  - ROADMAP.md
  - docs/plans/m2-plan.md
  - docs/notes/m2-to-m3-handover.md
  - docs/notes/typed-value-evaluator.md
---

# M3 start framing — DSL surface

このノートは M3 の milestone plan と最初の設計文書を書く前に、
M3 の読み方を揃えるための framing draft である。ADR / RFC そのものではなく、
M3 の phase breakdown、各 phase の pre-doc、必要なら M3-era RFC の入力 artefact
として扱う。

M2 の pre-doc は「Foundation milestone をどう閉じるか」を中心にした。
M3 では同じ規律を使うが、焦点は少し違う。ここで決めたいのは個別 DD の
option ではなく、ROADMAP の M3 thesis をどう読むか、そして M3 に入れないものを
どこまで明示的に切るかである。

---

## M3 thesis の読み

ROADMAP の M3 thesis は次である。

> the DSL is expressive enough to write real layouts, and is published as a stable public draft.

この framing では、M3 を **DSL surface milestone** と読む。つまり M3 の目的は、
M2 で通った `.ui -> IR -> runtime` 基盤の上に、実用的な画面構造を DSL で書ける
だけの surface を増やし、その surface を外部読者が参照できる public draft として
文書化することである。

したがって M3 の主語は「runtime foundation の再設計」でも「Windows identity
feature」でも「editor tooling」でもない。M3 は Grid / ScrollView / List と DSL
spec draft を通じて、Wasamo の DSL が Hello Counter を超えた画面を表現できることを
閉じる。

---

## ROADMAP acceptance の再確認

ROADMAP は acceptance criteria の SSOT であり、M3 は現時点で次の 4 項目を持つ。

- Grid layout primitive
- ScrollView primitive
- List primitive
- DSL specification first public draft
  - M2 + M3 surface をカバーする。
  - M4 の material syntax を予約してよい。
  - Mica / Acrylic 等の rendering semantics にはコミットしない。

M3 plan はこの 4 項目を phase structure に落とす。acceptance criteria を増やす場合は、
「M3 thesis を満たすために構造的に不足している」ことを説明できる必要がある。

---

## M2 から継承する前提

M3 は M2 の決定を再訴訟しない。特に次の前提は、M3 の pre-doc が消費する入力であり、
暗黙に上書きしてはいけない。

### `wasamo-ir` は共有 IR crate

M2 で `wasamoc -> wasamo-ir <- wasamo-runtime` の依存方向が確立した。Grid /
ScrollView / List が新しい IR node form を必要とする場合、変更単位は少なくとも
次の 3 点の組である。

- `docs/dsl_spec.md` の grammar / IR spec
- `wasamo-ir` の in-memory representation
- `wasamoc` emitter と `wasamo-runtime` loader の wiring

compiler だけ、または runtime だけに新 surface を足すのは M3 の設計として不正である。

### `HandlerExpr` は handler / binding で統一

M2 で handler body と binding expression は単一の `HandlerExpr` に寄せられた。
M3 の List item context、Grid coordinate、template binding などが expression を増やす場合、
まずはこの共有 expression model にどう乗るかを検討する。binding 用と handler 用の
別言語を作る場合は、それ自体が明示的な設計判断になる。

### dirty Effect ordering には M3 residual がある

DD-M2-P6-010 により dirty Effect drain はトポロジカル walk になったが、M3 multi-binding
が触りうる residual は残っている。

- cycle detection policy
- dependency tie の observable contract
- fan-out と `MUTATION_CAP` の関係

これらは M3 の acceptance criteria そのものではないが、List / template / multi-binding
の設計が実際に触れるなら、その phase の pre-doc で処理する。

### `TypedValue` は open question であり、M3 AC ではない

M2 の DD-M2-P6-011 は `String` binding を通すことで、binding path が `i32` 専用でない
ことを実証した。一方で、evaluator API 全体を `TypedValue` に一般化する決定はしていない。

M3 は最初の再評価契機になりうる。特に List item context、layout value、boolean /
dimension / color のような第三の scalar type が入るなら、`TypedValue` を ADR / RFC
対象にする圧力が生まれる。ただし `TypedValue` を M3 開始時点の acceptance criterion
として先取りしない。実際の type-system pressure が出ないなら、M4 / M5 / post-1.0 へ
defer してよい。

---

## M3 で最初に決めるべき問い

### 1. Phase order

Grid / ScrollView / List / DSL spec draft をどの順に進めるか。素朴には Grid →
ScrollView → List → spec finalization が考えられるが、List が item-template と
binding-context を要求するなら、最も設計圧力が高いのは List である可能性もある。

M3 plan は「実装しやすい順」だけではなく、「後続 surface の設計を支配する順」を見る。

### 2. M3 の E2E サンプル

M2 の Hello Counter に相当する、M3 の目に見える到達点を決める必要がある。候補は、
設定画面、簡易ファイルブラウザ風リスト、サイドバー + detail layout など。

よいサンプルは次を同時に使う。

- Grid で全体構造を作る。
- ScrollView で viewport を持つ。
- List で繰り返し要素を表示する。
- `.ui` から runtime まで通り、M2 の手動 host wiring に戻らない。

### 3. DSL spec draft の進め方

DSL spec draft を最後にまとめるだけにすると、各 phase の設計判断が実装に埋もれやすい。
逆に最初に全部書こうとすると、未検証の surface を固定しやすい。

暫定方針としては、各 M3 phase で `docs/dsl_spec.md` を同時更新し、M3 の最後に
public draft として整える形が最も自然である。

### 4. List item context と TypedValue 再評価

List が `item`, `index`, selected state, template-local binding などを導入する場合、
既存の `EvalContext` method family に足すだけでよいのか、共通 typed-value surface が
必要になるのかを判断する。

ここは M3 の最初から結論を固定しない。List pre-doc の主要論点として扱う。

### 5. Grid / ScrollView の責務境界

Grid は layout constraint solver の問題であり、ScrollView は clipping / viewport /
coordinate transform の問題である。両者は組み合わせて使われるが、同じ phase にまとめると
layout engine の変更範囲が膨らむ可能性がある。

M3 plan は、Grid と ScrollView を分けるか、最小 ScrollView を Grid と同時に入れるかを
明示する必要がある。

---

## M3 に入れないもの

次は M3 の thesis から外す。必要なら syntax reservation や future hook は置けるが、
acceptance や実装完了条件にはしない。

- Mica / Acrylic rendering semantics
- full theming surface
- system accent propagation
- input / focus model
- TextField / IME
- AccessKit / UIA
- multi-window
- VS Code LSP acceptance
- hot reload
- C ABI freeze
- `TypedValue` の無条件導入

特に Theming / Mica / Acrylic は M4 / M5 の thesis に近い。M3 の DSL spec は material
syntax を予約してよいが、レンダリング契約まで M3 で閉じようとすると、M3 が
"DSL surface" ではなく "feature breadth" milestone に戻ってしまう。

---

## 初期 phase breakdown 仮説

以下は plan ではなく、plan drafting の出発点である。

| Phase | 仮名 | 主な問い | Acceptance hook |
|---|---|---|---|
| M3-Phase 1 | Grid layout primitive | row / column / span / measure-arrange の DSL・IR・runtime 表現 | Grid |
| M3-Phase 2 | ScrollView primitive | viewport、clip、content size、layout invalidation 境界 | ScrollView |
| M3-Phase 3 | List primitive | item template、item context、binding、rebuild / reuse scope | List |
| M3-Phase 4 | Public DSL draft | M2 + M3 surface の grammar / IR / semantics 整理 | DSL spec draft |

この順序は保守的である。List が最も大きな設計圧力を持つなら、List pre-doc を早めに開いて
TypedValue / item context の判断だけ先に済ませる alternative もありうる。

---

## Draft framing decisions

### F1 — M3 は DSL surface milestone と読む

M3 の目的は、Grid / ScrollView / List を通じて DSL が実用的な画面構造を表現できることを
示し、その surface を public draft として文書化することである。

### F2 — M3 は feature breadth milestone に戻さない

M2 開始時に original Alpha wishlist を Foundation milestone へ絞ったのと同じ理由で、
M3 でも Theming、input、LSP、hot reload、ABI freeze を同時に抱え込まない。

### F3 — `TypedValue` は再評価候補だが、開始時点の M3 acceptance ではない

M3 の実装が第三の scalar type、typed item context、runtime value binding を必要とするなら
`TypedValue` を設計判断として開く。そうでなければ defer してよい。

### F4 — DSL spec draft は各 phase の副産物ではなく acceptance の一部

M3 の最後に spec を慌てて書くのではなく、各 phase で surface 変更を spec に反映し、
最後に public draft として整える。spec は tooling / external implementation の参照元になるため、
実装後メモではなく M3 の成果物として扱う。

### F5 — M3 の E2E proof は Hello Counter を超える

M3 は単一 counter の延長では閉じない。Grid / ScrollView / List を同時に使う小さな実用画面を
M3 の visible proof として置き、`.ui -> IR -> runtime` の path が M2 基盤の上で拡張されたことを
確認する。

---

## Next step

この framing を owner とすり合わせた後、`docs/plans/m3-plan.md` を `status: drafting`
で作成する。M3 plan では ROADMAP acceptance を mirror し、phase breakdown、
dependencies、acceptance ↔ phase mapping、out-of-scope、risks を frozen agreement として
整理する。

最初の phase に進む前に、phase-specific pre-doc で DSL / IR / runtime / spec の変更単位を
明示する。
