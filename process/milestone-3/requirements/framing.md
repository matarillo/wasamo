---
title: M3 start framing — DSL surface
status: accepted
created: 2026-05-11
related:
  - ROADMAP.md
  - docs/plans/m2-plan.md
  - docs/notes/m2-to-m3-handover.md
  - docs/notes/typed-value-evaluator.md
---

# M3 start framing — DSL surface

このノートは M3 の milestone plan と最初の設計文書を書く前に、
M3 の読み方を揃えるための owner-agreed framing である。ADR そのものではなく、
M3 の phase breakdown、各 phase の pre-doc、必要なら追加の vision decision record の入力 artefact
として扱う。

M2 の pre-doc は「Foundation milestone をどう閉じるか」を中心にした。
M3 では同じ規律を使うが、焦点は少し違う。ここで決めたいのは個別 DD の
option ではなく、ROADMAP の M3 thesis をどう読むか、そして M3 に入れないものを
どこまで明示的に切るかである。

Owner agreement (2026-05-11): ROADMAP の M3 acceptance criteria は VISION の
thesis 表現をもとにした仮設定であり、M3 開始時点で見直す価値がある。
特に Grid / ScrollView / List を primitives として選ぶことには、まだ十分な
根拠があるとまでは言えない。さらに、primitives の妥当性は単体では判断できない。
M3 で実際に作る E2E proof の画面を先に固定し、その画面が要求する surface から
Grid / ScrollView / List の採否を逆算する。また `docs/notes/` 以下の live notes は
オーナー要求事項の種であり、open question の一部は M3 に入れる候補として棚卸する。

---

## M3 thesis の読み

ROADMAP の M3 thesis は次である。

> the DSL is expressive enough to write real layouts, and is published as a stable public draft.

この framing では、M3 を **DSL surface milestone** と読む。つまり M3 の目的は、
M2 で通った `.ui -> IR -> runtime` 基盤の上に、実用的な画面構造を DSL で書ける
だけの surface を増やし、その surface を外部読者が参照できる public draft として
文書化することである。

したがって M3 の主語は「runtime foundation の再設計」でも「Windows identity
feature」でも「editor tooling」でもない。M3 は、選定した DSL surface と public
spec draft を通じて、Wasamo の DSL が Hello Counter を超えた画面を表現できることを
閉じる。Grid / ScrollView / List はその現在の候補であり、ここで審査する。

---

## M3 target app を先に決める

M3 の最初の設計対象は primitive list ではなく、M3 で作る **target app / E2E proof**
である。M3 thesis が "real layouts" なら、まず「どんな実用画面を DSL で書けるように
するか」を固定し、その画面から必要な primitives、binding context、layout constraint、
spec 記述を逆算する。

target app が未定義のまま Grid / ScrollView / List の妥当性を議論すると、判断基準が
設計美学に寄りやすい。M3 pre-doc は少なくとも次を成果物として持つ。

1. M3 で作る E2E 画面のワイヤーフレーム、または動く最小プロトタイプ。
2. その画面に必要な primitives と binding / layout capability の一覧。
3. 各 primitive が検証する thesis。
4. M3 では明示的に扱わない機能のリスト。
5. spec / implementation / E2E proof の同期ルール。

候補画面は、設定画面、簡易ファイルブラウザ風リスト、サイドバー + detail layout などで
よい。ただし Hello Counter の延長ではなく、複数の layout primitives と複数の
binding / layout constraint が同じ画面内で同時に成立するものを選ぶ。

---

## ROADMAP acceptance の扱い

ROADMAP は acceptance criteria の SSOT であり、M3 は現時点で次の 4 項目を持つ。

- Grid layout primitive
- ScrollView primitive
- List primitive
- DSL specification first public draft
  - M2 + M3 surface をカバーする。
  - M4 の material syntax を予約してよい。
  - Mica / Acrylic 等の rendering semantics にはコミットしない。

ただし、この framing では上記 4 項目を **確定済みの実装リストとして扱わない**。
これらは VISION の "real layouts + public draft" thesis を具体化するための
初期仮説であり、M3 plan を作る前に次を問い直す。

- Grid / ScrollView / List は、M3 thesis を検証する最小かつ十分な primitives か。
- 他の primitive や substrate work のほうが、"real layouts" の検証として強いか。
- `docs/notes/` の open question のうち、M3 で解かないと public draft が
  不誠実になるものはないか。
- ROADMAP の M3 acceptance を見直す必要があるなら、どの単位で revision するか。

従って M3 plan は、単にこの 4 項目を phase structure に落とすのではなく、
まずこの 4 項目を採用・修正・置換する根拠を持つ必要がある。
その根拠は、先に固定した target app / E2E proof に対して各 primitive が何を
証明するかで説明されるべきである。

---

## `docs/notes` open question 棚卸

`docs/notes/` 以下の live notes は、単なる周辺メモではなく、オーナー要求事項の
種である。M3 開始時点では、少なくとも次を棚卸対象にする。

| Note | M3 との関係 | 初期判定 |
|---|---|---|
| `docs/notes/dsl-grammar.md` | widget id、Window 由来 component-level prop、名前解決。M3 public DSL draft と直接接続する。 | **M3候補**。少なくとも spec drafting 前に判断が必要。 |
| `docs/notes/layout-engine.md` | DPI scaling、layout cache invalidation、user-defined layout、非同期 measure。Grid / ScrollView を採るなら直撃する。 | **M3候補**。Grid/ScrollView選定の根拠にもなる。 |
| `docs/notes/architectural-family.md` | M3 DSL grammar が tree-with-bindings family を明示的に確定するか、別 family へ寄るか。 | **M3入口で再読必須**。必要なら vision decision。 |
| `docs/notes/m2-to-m3-handover.md` | shared IR crate、unified `HandlerExpr`、dirty Effect topo residual、TypedValue pointer。 | **M3前提 + residual**。acceptance 候補ではなく設計制約。 |
| `docs/notes/typed-value-evaluator.md` | 第3の scalar type、item context、typed binding result が出るなら再評価。 | **条件付きM3候補**。List / typed layout values 次第。 |
| `docs/notes/headless-verification.md` | M3+ DSL surface 拡大で pure-logic fixture が足りるか。 | **条件付きM3候補**。ACにするより verification risk として扱う。 |
| `docs/notes/m2-phase-6/dd-m2-p6-drain-transaction.md` | observer から state mutation を要求する場合の post_event / deferred mutation API。 | **保留寄り**。M3 E2E sample が外部 integration / analytics を要求するなら再浮上。 |
| `docs/notes/workspace-layout.md` | crate / workspace placement の open question。 | **M3本体では低優先度**。新 crate が必要ならその時に見る。 |

この表は初期棚卸であり、M3 plan の frozen agreement ではない。目的は、
"Grid / ScrollView / List を作る" というタスク表に入る前に、M3 が本当に解くべき
要求事項を見落とさないことである。

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
dimension / color のような第三の scalar type が入るなら、`TypedValue` を ADR
対象にする圧力が生まれる。ただし `TypedValue` を M3 開始時点の acceptance criterion
として先取りしない。実際の type-system pressure が出ないなら、M4 / M5 / post-1.0 へ
defer してよい。

---

## M3 で最初に決めるべき問い

### 1. M3 target app / E2E proof

Grid / ScrollView / List の実装順を決める前に、M3 で実際に作る画面を決める。
この画面は単なるデモではなく、M3 acceptance criteria の判断基準に近い位置づけを持つ。

よい target app は次を同時に満たす。

- 採用候補の layout primitive を少なくとも一度は使う。
- viewport / overflow を扱う。
- 繰り返し要素、またはそれに相当する実用データ表示を含む。
- 複数の binding / layout constraint が同時に成立する。
- `.ui` から runtime まで通り、M2 の手動 host wiring に戻らない。

この target app を先に選ぶと、Grid / ScrollView / List が本当に M3 AC として妥当かを
検証しやすい。逆に primitive 名を先に固定すると、サンプルが後付けのデモに落ちる。

### 2. M3 acceptance criteria revision

target app を固定したうえで、Grid / ScrollView / List / DSL spec draft をそのまま採るか、
修正するか、置換するかを判断する。M3 thesis が "real layouts + public draft" なら、
primitive の選定は「作りたい widget 名」ではなく「どんな UI を実際に書けるようにするか」
から逆算する。

例えば、List は `item context` や `TypedValue` を引きずり込む可能性が高い一方で、
M3 の visible proof にとって強い説得力を持つ。ScrollView は実用画面にはほぼ必須だが、
最初の M3 で独立 primitive として acceptance に置くべきか、List/Grid の supporting
primitive として扱うべきかはまだ未確定である。

### 3. Phase order

acceptance を見直したうえで、Grid / ScrollView / List / DSL spec draft をどの順に進めるか。
素朴には Grid → ScrollView → List → spec finalization が考えられるが、List が item-template と
binding-context を要求するなら、最も設計圧力が高いのは List である可能性もある。

M3 plan は「実装しやすい順」だけではなく、「後続 surface の設計を支配する順」を見る。

### 4. DSL spec draft の進め方

DSL spec draft を最後にまとめるだけにすると、各 phase の設計判断が実装に埋もれやすい。
逆に最初に全部書こうとすると、未検証の surface を固定しやすい。

暫定方針としては、各 M3 phase で `docs/dsl_spec.md` を同時更新し、M3 の最後に
public draft として整える形が最も自然である。implementation が変わったら spec と
E2E proof も同じ phase 内で追随させ、spec を実装後のメモに落とさない。

### 5. `docs/notes` open question の昇格基準

open question を M3 に入れる基準を明示する必要がある。初期案としては、
次のいずれかに当たるものだけを M3 acceptance / phase scope 候補にする。

- M3 の public DSL draft に normative に書かないと、外部読者が実装できない。
- M3 の visible proof を作るために実装上必要になる。
- M3 で選ぶ grammar / IR shape が、後から変えると破壊的になる。
- M4 以降の thesis に明確に属するが、M3 で syntax reservation が必要である。

この基準を満たさないものは、たとえ重要でも M3 の out-of-scope または future hook に残す。

### 6. List item context と TypedValue 再評価

List が `item`, `index`, selected state, template-local binding などを導入する場合、
既存の `EvalContext` method family に足すだけでよいのか、共通 typed-value surface が
必要になるのかを判断する。

ここは M3 の最初から結論を固定しない。List / Grid の設計レビューでは、
「この設計は将来 `TypedValue` に自然に接続できるか」を明示的なチェック項目として扱う。
`TypedValue` を実装しない場合でも、後から導入する際のリファクタリング範囲を無用に
広げない。

### 7. Grid / ScrollView の責務境界

Grid は layout constraint solver の問題であり、ScrollView は clipping / viewport /
coordinate transform の問題である。両者は組み合わせて使われるが、同じ phase にまとめると
layout engine の変更範囲が膨らむ可能性がある。

M3 plan は、Grid と ScrollView を分けるか、最小 ScrollView を Grid と同時に入れるかを
明示する必要がある。

---

## M3 に入れないもの

次は現時点では M3 の thesis から外す候補である。必要なら syntax reservation や future hook は置けるが、
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

ただし、LSP / diagnostics を実装対象外にすることは、tooling を設計制約から外すことを
意味しない。M3 の DSL は人間が書けるだけでなく、後続の静的解析・診断・補完の対象として
破綻しない構文と意味論を保つ必要がある。これは M3 の acceptance ではなく、
surface design の制約として扱う。

なお、この out-of-scope list も棚卸後に確定する。たとえば Window title のような
component-level prop は、Mica / Acrylic rendering semantics とは別に、M3 public DSL draft の
誠実性に関わる可能性がある。

---

## 初期 phase breakdown 仮説

以下は plan ではなく、plan drafting の出発点である。ROADMAP AC を見直した結果、
この表自体が変わる可能性がある。特に M3-Phase 0 は、実装 phase ではなく
acceptance を選ぶための pre-doc phase である。

| Phase | 仮名 | 主な問い | Acceptance hook |
|---|---|---|---|
| M3-Phase 0 | Target app framing | E2E proof の画面、必要 surface、out-of-scope、spec 同期ルールを固定する | AC 選定の根拠 |
| M3-Phase 1 | Grid layout primitive | row / column / span / measure-arrange の DSL・IR・runtime 表現 | Grid |
| M3-Phase 2 | ScrollView primitive | viewport、clip、content size、layout invalidation 境界 | ScrollView |
| M3-Phase 3 | List primitive | item template、item context、binding、rebuild / reuse scope | List |
| M3-Phase 4 | Public DSL draft | M2 + M3 surface の grammar / IR / semantics 整理 | DSL spec draft |

Phase 1 以降の順序は保守的である。Phase 0 で選ぶ target app によっては、
List pre-doc を早めに開いて TypedValue / item context の判断だけ先に済ませる、
または ScrollView を List/Grid の supporting primitive として扱う alternative もありうる。

---

## Owner-agreed framing decisions

### F1 — M3 は DSL surface milestone と読む

M3 の目的は、DSL が実用的な画面構造を表現できることを示し、その surface を public draft
として文書化することである。Grid / ScrollView / List はそのための現在の候補であり、
無批判に確定済みとは扱わない。

### F2 — ROADMAP の M3 acceptance は target app に照らして審査する

ROADMAP は SSOT だが、現在の M3 acceptance は VISION 由来の仮説を含む。
M3 plan drafting の前に target app / E2E proof を定義し、その画面に対して
Grid / ScrollView / List / spec draft が M3 thesis を最もよく検証する組み合わせかを
審査する。必要なら ROADMAP revision を起こす。

### F3 — `docs/notes` の open question を棚卸してから M3 scope を切る

`docs/notes/` 以下の live notes はオーナー要求事項の種として扱う。M3 に入れるかは、
public draft の誠実性、visible proof への必要性、grammar / IR の破壊的変更リスクを
基準に判断する。

### F4 — M3 は feature breadth milestone に戻さない

M2 開始時に original Alpha wishlist を Foundation milestone へ絞ったのと同じ理由で、
M3 でも Theming、input、LSP、hot reload、ABI freeze を同時に抱え込まない。
ただし DSL surface は、後続の diagnostics / completion / static analysis が扱える形を
設計制約として維持する。

### F5 — `TypedValue` は再評価候補だが、開始時点の M3 acceptance ではない

M3 の実装が第三の scalar type、typed item context、runtime value binding を必要とするなら
`TypedValue` を設計判断として開く。そうでなければ defer してよい。ただし List / Grid の
設計レビューでは、将来 `TypedValue` に自然に接続できるかを明示的に確認する。

### F6 — DSL spec draft は各 phase の副産物ではなく acceptance の一部

M3 の最後に spec を慌てて書くのではなく、各 phase で surface 変更を spec に反映し、
最後に public draft として整える。spec は tooling / external implementation の参照元になるため、
実装後メモではなく M3 の成果物として扱う。implementation / spec / E2E proof は同じ phase で
同期させる。

### F7 — M3 の E2E proof は Hello Counter を超える

M3 は単一 counter の延長では閉じない。採用した M3 surface を同時に使う小さな実用画面を
M3 の visible proof として置き、`.ui -> IR -> runtime` の path が M2 基盤の上で拡張された
ことを確認する。この proof はデモではなく、M3 acceptance criteria を選ぶための基準でもある。

---

## Next step

この framing は owner-agreed として扱う。次に、まず M3 target app / E2E proof を決める。
その pre-doc では、ワイヤーフレームまたは最小プロトタイプ、必要 primitives、
各 primitive が検証する thesis、out-of-scope、spec / implementation / E2E proof の
同期ルールを明示する。

target app の候補は [m3-target-app-wireframes.html](target-app-wireframes.html) に
4 案（Mail Reader / Music Player / Photo Gallery / File Explorer）のワイヤーフレームと
layout surface の批判的検討を整理してある。target app pre-doc は本ファイルから候補を
選定した上で起こす。

そのうえで M3 acceptance candidates を整理し、必要なら ROADMAP revision のための
vision decision を起こす。その後 `docs/plans/m3-plan.md` を `status: drafting` で
作成する。M3 plan では確定した ROADMAP acceptance を mirror し、phase breakdown、
dependencies、acceptance ↔ phase mapping、out-of-scope、risks を frozen agreement として
整理する。

最初の implementation phase に進む前に、phase-specific pre-doc で DSL / IR / runtime /
spec / E2E proof の変更単位を明示する。
