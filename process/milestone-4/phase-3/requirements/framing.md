---
title: M4-Phase 3 フレーミング — 述語式（論点設定）
status: draft
created: 2026-08-11
target-phase: M4-Phase 3
workflow-stage: "2.2 issue framing"
related:
  - process/milestone-4/plan.md
  - process/milestone-4/requirements/framing.md
  - process/milestone-4/requirements/spec.md
  - process/milestone-4/phase-1/implementation/handoff.md
  - process/milestone-4/phase-2/requirements/framing.md
  - process/milestone-4/phase-2/implementation/handoff.md
  - docs/dsl_spec.md
---

# M4-Phase 3 フレーミング — 論点設定

**状態:** draft（§2.2 owner-agreed、§2.3 / §2.4 未実施）

**今回実施した段階:**
[workflow.md §2.2「論点設定」](../../../procedures/workflow.md) のみ

**まだ実施していない段階:** §2.3 スコープ確定、§2.4 検証方針確認、
§3 設計判断

この文書は、M4-Phase 3 で**何を決める必要があるか**を分け、DD 番号を
予約し、オーナーとの合意点を明らかにするための資料である。構文、IR、
実行方式、エラー文言の結論はここでは選ばない。

前段の accepted
[constraints.md](./constraints.md) は現時点の情報に基づく仮説として読む。
新しい情報で前提が変わった場合は再検討できるが、変更理由と影響を記録する。
今回読み直した Phase 1・2 の implementation handoff / retrospective / 現物、
M4 の計画・要件、現行の DSL 仕様と `.ui` の実例から、constraints の主要な
境界は維持できる。一方、**plan の二つの前提は、そのままでは維持できない**。
一つは、計画済みの per-item conditional が compiler-side だけでは閉じないという
現物上の訂正である。もう一つは、現行 plan からは導出できない handler guard を
Phase 3 の必須成果に加えるというスコープ変更である。この二つを一括承認せず、
下記の別々の問いと plan-revision gate で扱う。

> **例の読み方:** 以下の `count(photos)`、`photos[selected_index]`、
> `index == selected_index`、handler 内の `if` は、作者が実現したいことを
> 説明するための**仮の書き方**である。Wasamo の構文として予約したもの
> ではない。正式な綴りは後続の ADR で比較する。

---

## 今回オーナーに合意してほしいこと

この節だけで §2.2 の合意判断ができる。推奨どおりなら「①〜⑦ OK」、
変更したい項目があればその番号を指定してほしい。

| ID | 合意してほしいこと | 提案 | 状態 |
|---|---|---|---|
| ① | **DD の分け方と番号** | DD-M4-P3-001〜006 の 6 件を予約する。001 = 共通の述語式、002 = collection 読み取りと範囲外 failure contract、003 = 項目ごとの条件表示、004 = 等値選択、005 = 小さい一般的な handler control flow、006 = handler assignment admission / 型検査の完全性。DD-005 の plan-revision gate は完了した | owner-agreed 2026-08-11 |
| ② | **計画済み per-item conditional の責務訂正** | Phase 3 は compiler-only では閉じず、DD-003 が condition evaluation、subtree 再実体化、focus / hover / handler registry / layout lifecycle までを cross-layer に設計する。これは handler guard の採否とは独立した、既存 Phase 3 deliverable の実現条件である | owner-agreed; Revision 3 landed `7763555` |
| ③ | **handler control-flow の Phase 3 追加** | gallery の Left / Right key と左右 button の 4 経路すべてを、empty / 1 件 / 複数件で範囲外へ進ませないことを Phase 3 の必須成果とする。gallery 専用命令ではなく、今後も使える**一般的だが小さい surface**を DD-005 で設計する。正式構文・IR・評価方式はここでは選ばない | owner-agreed; Revision 4 landed `4afa204` |
| ④ | **handler assignment 検査の完全性** | 個別の不正 RHS をアドホックに塞ぐのではなく、全 handler assignment が漏れなく検査される仕組みを DD-006 の要求にする。RHS が handler position で許されるかという **capability / position admission** と、許された RHS の型が LHS 宣言型に合うかという **type compatibility** を分ける。scalar `string` write capability は Phase 5 のまま | owner-agreed; Revision 5 landed `1499241` |
| ⑤ | **範囲外 read の判断範囲** | runtime error / fallback / clamp のどれを契約にするか、失敗時に旧表示・対象 effect・後続 effect をどう扱うかを DD-002 の未決論点にする。Phase 2 の runtime diagnostic 期待は有力な入力だが、ADR 前の結論にはしない。DD-005 の guard は範囲外 state を予防する別責務であり、DD-002 の代替ではない | owner-agreed 2026-08-11 |
| ⑥ | **pre-ADR spike の要否** | Phase 3 全体には必須化しない。現物調査で per-item conditional の不足箇所は既に二つの loop-context seam として特定でき、未知の状態モデルを試作で発見する段階ではない。下記の発火条件が実際に成立した DD だけに、「何を観測できれば答えか」を限定した spike を ADR Accepted 前に提案する | owner-agreed 2026-08-11 |
| ⑦ | **現行仕様の矛盾の扱い** | `docs/dsl_spec.md` §8.11 に残っていた「`for` body の handler 禁止」「item/index read は binding だけ」という二行は、Phase 2 の実装同期漏れとして factual correction し、Phase 3 の新しい DD や禁止前提にはしない | owner-agreed; corrected in `6e3db4f` |

①〜⑦はいずれも、現時点の情報に基づく仮説としての合意を求める。
新しい実測・設計上の発見があれば見直せる。②・③の plan 改訂だけは、仮説として
合意した後も [workflow.md §計画(plan)改訂の規律](../../../procedures/workflow.md)
の手続きが別に必要である。

### Phase 1・2 の実施結果で plan 仮説を再評価した結論

| plan 時点の仮説 | 実施後に分かったこと | framing への反映 |
|---|---|---|
| Phase 3 は checker / lowering / evaluator 中心の **compiler-side** phase | per-item `if` は widget / effect / handler を生成・破棄する。Phase 2 で、同じ同期 drain 中に hover path のずれ、handler registry の再照会、focus の address identity が問題になり得ると判明。Phase 1 の layout dirty → `flush_layout` → `sync_visuals` も通る | Phase 3 を compiler-only と扱わない。DD-003 が runtime structural integration の責任を持つ。plan の phase 説明も改訂候補にする |
| equality と collection 読みがあれば gallery の前後移動も自然に閉じる | 現在の gallery は key 2 経路と button click 2 経路の計 4 か所で無条件に `selected_index` を増減する。equality だけでは「0 より大きい」「末尾より小さい」を自然に書けず、空・1 件も扱えない | 範囲外読みの診断（DD-002）と、範囲外状態を作らない guard（DD-005）を別責務として両方残す。後者の plan 追加を推奨する |
| predicate の追加は既存 expression pipeline の局所拡張 | Phase 2 で、handler assignment は `i32 = "abc"` や `string = 5` も check 時に見逃し、invocation 時まで失敗し得ると実測した。新しい expression position を増やすほど、case-by-case の reject では別経路の抜けが残る | 新しい Phase 3 式そのものの型・scope admission は DD-001、すべての handler assignment を LHS 宣言型と position capability に照らす完全性契約は DD-006 が持つ。検査を一つずつ足す方式はここで選ばず、ADR が漏れを構造的に防ぐ仕組みを比較する |
| per-item 条件は既存 `if` と `for` の単純な合成 | 現物では `append_static_member` が `loop_context` を受け取る一方、`register_conditional_binding` は通常の `BindingEvalContext` を使い、再挿入も `build_node(...)` で loop context を落とす | DD-003 は「構文を許可するか」だけでは閉じない。条件評価と再実体化の両方で現在位置の loop context を保つ問いを持つ。ただし不足箇所は source audit で特定済みなので、現時点で必須 spike は不要 |
| 選択表示は focus と同じ強調として扱ってもよい | Phase 2 が `checked` と `focused` を別状態として実装・可視確認し、同時成立時の合成も固定した | DD-004 は selection と focus を統合しない。Tab で focus が動いても、作者が `selected_index` を書かない限り selection は変わらない契約を保つ |

この再評価は、plan を無断で読み替えるためではない。旧前提、新情報、変更影響、
no-change の意味を明示した Revision-log 草案を別 artifact に作り、critical check と
owner authorisation を求めるべきだ、という**論点設定上の結論**である。plan 本文は
その二つが満たされるまで変更しない。

---

## 作者から見た、今回分けるべき問い

Phase 3 の目的を「式の内部実装を増やすこと」ではなく、Wasamo でアプリを
書く人が次の区別を自然に表せることとして捉える。

| アプリで実現したいこと | 説明用の仮記法 | 現在 | Phase 3 で問うこと |
|---|---|---|---|
| 写真の件数を表示する | `Text { text: count(photos) }` | `for` の外では collection を読めない。現在の gallery は `18` を静的文字列に埋めている | 件数読みの型、綴り、更新追随をどう定義するか（DD-001 / 002） |
| 写真が 0 件なら空表示を出す | `if empty(photos) { Text { text: "No photos" } }` | collection の空判定を `if` 条件にできない | 空判定を bool 述語としてどう扱うか（DD-001 / 002） |
| 選択中の写真名を caption に出す | `Text { text: photos[selected_index] }` | `for` の外から添字で要素を読めない。現在は `Photo #<index>` | 有効な添字の値と、範囲外をどう失敗させるか（DD-002） |
| 項目ごとに印を出し分ける | `for photo, index ... { if index == selected_index { ... } }` | loop binder は binding / handler では読めるが、`if` 条件では読めない | binder を条件から読む範囲と、部分木の作り直しをどう扱うか（DD-003） |
| 1 つの状態で 1 つだけ選択表示する | `checked: index == selected_index` | 3 tab なら bool state を 3 個持ち、各 click で 3 個を書き換える | equality を使う単一識別値の選択を、既存 property / 構造表示へどう投影するか（DD-004） |
| 先頭・末尾から範囲外へ進ませない | handler 内で「有効範囲なら更新」 | 現在は Left / Right key と `<` / `>` click の 4 経路が無条件に `selected_index` を増減し、−1 や件数と同じ値になれる | plan 改訂後、gallery 専用でない小さい handler control-flow surface をどう設けるか（DD-005） |
| handler assignment の誤りを実行前に知る | `count = "many"` / `title = 5` / `title = "New"` | LHS 型と RHS 型を照合しない経路があり、binding-only string RHS も invocation まで通る | 全 assignment を漏れなく検査しつつ、型適合と position capability をどう分離するか。scalar string write は Phase 5 に残す（DD-006） |

Phase 3 を終えても、少なくとも次は実現できない前提を問いの境界に置く。

- `"Photos: " + count(photos)` のような文字列連結。件数表示は静的な
  `Text` と値を表示する `Text` の並置で作る。
- 一般的な `+ - * /` 式、任意の関数、複合的な命令文。
- `RadioGroup`、`SegmentedControl`、自己 toggle、widget-owned selection。
- two-way binding。`checked` は引き続き controlled one-way である。
- record / object の collection、`TypedValue`、項目の key identity、
  nested `for`、loop binder の shadowing。
- handler から scalar `string` state を書く能力。これは Phase 5 の責任である。

上の一覧は §2.3 の scope 確定ではない。DD の問いが意図せず一般言語・
部品一式・型システム刷新へ広がらないようにする、論点の境界確認である。

---

## Phase 3 に pre-ADR spike は必要か

**判断:** Phase 3 全体の必須 pre-ADR spike は置かない。必要性が発火した
DD だけに、狭い spike を後から提案できる形にする。

### Phase 2 で必要だった理由との比較

`exp/m4-phase-2-focus-spike` の履歴では、Phase 2 の spike は小さな構文確認
ではなかった。純ロジックの focus core、実 `WidgetNode` 木からの投影、
`.ui` → IR → runtime の fixture を作り、次のような ADR 入力を実測した。

- group 内の直前位置には、focus とは別の記憶が必要だった。
- focus と active item には別の pointer が必要だった。
- modal containment は一つの名前の下にある二つの機構の合成だった。
- modal scope の復帰先は、閉じた後の木から導出できなかった。
- 当時の `WidgetData` は必要な 6 role のうち 2 role しか表せなかった。

つまり Phase 2 は、ADR が決めるべき**状態モデルそのものが実現可能か**、
また紙上で一つに見える概念が実装でも一つかを、動く木で確かめる必要が
あった。さらに M4 plan が M5 の将来部品まで意味論を先に固定するよう求め、
pre-ADR spike を明示的な gate にしていた。

Phase 3 との違いは次のとおりである。

| 観点 | Phase 2 | Phase 3 |
|---|---|---|
| 未知の中心 | focus / group / active item / modal scope の状態と所有 | 構文・型・評価規則と、既知の loop-context / structural seam の接続 |
| 既存の実行経路 | 必要な role と production focus core がまだ無かった | parser、checker、lowering、`HandlerExpr`、型別 evaluator、dependency tracking、reactive `if` がある。ただし per-item conditional は condition effect と再挿入の二か所で loop context が未接続 |
| 実測しないと分からなかったこと | 復帰情報を木から導けるか、別 pointer が要るか、実木から投影できるか | 現時点では同種の外部・状態モデル上の未知は見つかっていない |
| 主な消費者 | M5 の未実装部品まで含む将来意味論 | A の count / empty / caption / thumbnail selection という具体例 |
| prototype の害 | plan が意図的に許容し、非 production seam と cleanup 条件を置いた | syntax / AST / IR variant が ADR 前の既成事実になり、方式比較を狭めやすい |
| plan 上の扱い | pre-ADR spike が明記された必須 gate | spike の指定なし。novel normative DSL content とだけ記載 |

### DD ごとの判定

| DD | 必須 spike | 理由 |
|---|---|---|
| DD-001 共通式 surface | **不要** | 構文・型・式位置は規範判断であり、prototype の通りやすさで決めない。lexer が equality token をまだ持たないことも、実現不能の証拠ではなく設計対象そのもの |
| DD-002 collection 読み | **不要** | 既存に collection signal、型別 tracked read、`ItemOutOfRange` の先例がある。範囲外後の旧値 / effect の扱いは製品契約として ADR が決め、その後 unit test で固定できる |
| DD-003 per-item 条件表示 | **不要** | source audit で、現在の conditional effect が通常の `BindingEvalContext` を使い、再挿入が `build_node(...)` で loop context を落とすという二つの不足を特定できた。未知の状態モデルではなく、ADR が loop context の所有・寿命・再評価規則を決めるための具体的入力である。Phase 2 の focus / hover / registry 再発条件は call-path audit で判定する |
| DD-004 equality selection | **不要** | 新 widget / group state ではなく、1 state と既存 `checked` または条件表示の合成。author contract を先に決めるべきで、動く ToggleButton を先に作っても方式比較の根拠にならない |
| DD-005 handler control flow | **不要（targeted trigger only）** | tier 2 plan-revision gate は完了した。まず author example の境界表で必要な表現力を比較し、それだけでは IR / evaluator 候補の可否が決まらない場合に限って狭い spike を再検討する |
| DD-006 handler assignment 検査 | **不要** | Phase 2 で `i32 = "abc"` / `string = 5` と binding-only string forms の漏れを実測済み。中心は網羅的な admission / compatibility invariant の設計であり、個別 reject の prototype を増やしても完全性の判断材料にならない |

ここでいう call-path audit や author example の境界表は ADR の調査であり、
production code や暫定 IR を作る spike ではない。

### 狭い spike を再検討する発火条件

次のどれかが、推測ではなく source audit・既存 test・ADR の候補比較から
具体的に示された場合だけ再検討する。

1. **型表現の壁:** count / index / equality のいずれかが、既存の型別
   expression / evaluator を保ったまま表せず、constraints が除外した
   `TypedValue` または別の式木を導入しないと候補比較ができない。
2. **依存追跡の壁:** collection と index の二つを読む effect が、既存の
   tracked-read model で正しく再実行されるかを source と既存 unit test から
   判定できず、DD-002 の方式選択がその可否に依存する。
3. **構造更新の壁:** per-item conditional が既存の conditional mutation seam
   を使えるか、または Phase 2 の hover / focus / registry 契約を壊すかを
   call path から判定できず、DD-003 の候補がそこで分かれる。
4. **guard の表現力の壁:** DD-005 の plan 改訂が承認された後、左右端・空
   collection・1 件だけの collection を、候補となる狭い guard surface で
   書けるかが紙上の境界表では決まらず、IR / evaluator の可否で候補が分かれる。
5. **既存実測との矛盾:** binding-only string RHS の現在挙動が Phase 2 handoff
   と異なる、または許可形 `string[]` append との境界が source audit で
   確定できない。

なお DD-003 の二つの loop-context 不足は、発火条件 3 が既に spike を要求した
ことを意味しない。どこで context が失われるかと、既存の
`ForItemEvalContext` / `build_node_with_loop_context` という比較材料まで source から
特定できている。ADR の候補が「その seam では寿命を安全に表せない」などの理由で
実測なしに比較不能になった時点で、初めて発火する。

発火時も「Phase 3 spike」という大きな一本にはしない。対象 DD、比較する
候補、観測する問い、答えたと言える条件、prototype の廃棄 / production 化の
境界を先に記録する。結果が Accepted 済み ADR を揺らした場合は、Phase 2 の
合意と同じく既存の ADR 改訂規律に従う。

---

## 論点一覧（DD 番号の予約）

本フェーズの設計判断用に **DD-M4-P3-001〜006** を予約する。
ここでは各 DD の「問い」「分ける理由」「具体的に答える必要がある下位の問い」
だけを定める。選択肢の比較、推奨、結論は §3 の ADR で行う。

DD の依存関係は次のとおりである。

- DD-001 は共通の語彙・型・式位置を定める土台。
- DD-002〜004 は、DD-001 の共通規則を 3 つの利用場面へ適用する。
- DD-005 は owner-required deliverable であり、完了した plan-revision gate を
  基準線として DD-001 を消費する。ADR 起草は §2.3 / §2.4 完了後に進む。
- DD-006 は DD-001 の expression typing を利用しても、全 handler assignment の
  destination compatibility と position admission の完全性を独立に所有する。

### DD-M4-P3-001 — 述語式の共通 surface と型規則

**問い:** collection の件数・空判定・添字読みと equality を、`.ui` の
どの式位置で、どの型規則により書けるようにするか。構文、名前解決、
結果型、AST / IR の共通表現をどうそろえるか。

**独立した問いにする理由:** 現在の `expr` は、同じ見た目でも位置ごとに
許される形が狭い。たとえば `if` 条件は bool literal / bool state だけ、
property binding は対象 property の型に依存し、handler RHS は別の評価経路を
使う。collection 読みや `==` を 1 か所だけの特例にすると、作者は
「`checked:` では使えるが `if` では使えない」のような予測しにくい言語を
受け取る。反対に、全 expression position を無条件に一般化すると Phase 3 の
範囲を越える。共通線を先に決める必要がある。

**具体例（綴りは未決）:**

```wasamo
Text { text: count(photos) }
if empty(photos) { Text { text: "No photos" } }
ToggleButton { checked: index == selected_index }
```

**下位の問い:**

- 件数の結果は `i32`、空判定と equality の結果は `bool` とするか。
- 添字読みの結果型を collection の要素型からどう決めるか。
- equality を `i32` / `string` / `bool` のどこまで認めるか。同じ型同士だけか、
  暗黙変換を認めるか。
- local state、`root.` 付き state、loop binder をどの位置で解決できるか。
- operator の結合順位や括弧をどこまで定めるか。DD-005 の gate が閉じた
  基準線では不要な logical、relational、一般算術をどう明確に拒否するか。
  gate が開くなら、DD-005 が必要性を示した最小追加だけをどう共通規則へ
  戻すか。
- `.ui` → AST → IR の表現を位置ごとの専用品にするか、共通 expression として
  表すか。既存の `HandlerExpr` という名前と実際の binding 利用のずれを
  この機会にどう扱うか。
- Phase 3 が認める各式を、`if` 条件、property binding、handler guard など
  **認めた全位置**で、結果型と scope を `wasamoc check` が検査できるか。
  「実行経路が型を拒否する」ことを作者向け診断の代わりにしない。
- 不正な型・不正な式位置を `wasamoc check` と loader のどこまでで
  再検査するか。

### DD-M4-P3-002 — `for` の外からの collection 読み取りと失敗契約

**問い:** collection の件数、空判定、添字読みを、`for` の外から
どのように読み、どの変更に追随させ、添字が範囲外ならどの単位で失敗させるか。

**独立した問いにする理由:** 件数と空判定は全域で値を返せるが、添字読みは
実行時に失敗し得る。3 つを同じ「collection read」として構文だけ決めても、
reactive dependency と失敗後の画面状態が未決のまま残る。特に gallery では
`selected_index` が −1 または N になれるため、範囲外は架空の例ではない。

**具体例（綴りは未決）:**

```wasamo
// photos が差し替われば自動で更新される必要がある。
Text { text: count(photos) }

// selected_index = 2 なら 3 件目の名前。
Text { text: photos[selected_index] }

// selected_index = -1 または count(photos) のときの結果は DD-002 で決める。
```

**下位の問い:**

- collection 自体、添字 state、両方の reactive dependency をどう追跡するか。
- 同じ長さの collection 差し替えでも、添字先の値を読み直すことをどう保証するか。
- 負数、`index == len`、空 collection、読み取り後に collection が縮んだ場合を、
  runtime error、fallback、clamp などのどの契約で扱うか。Phase 2 handoff の
  `ItemOutOfRange` precedent を採用するか、別の作者契約を選ぶか。
- 範囲外時に、対象 binding の旧表示を保持するか、式 / effect を失敗状態に
  するか、後続 effect をどう扱うか。部分的な更新をどこまで許すか。
- runtime error を選ぶ場合、診断を作者がどこで観測できるようにするか。
  fallback / clamp を選ぶ場合、空文字、0、`false`、末尾値、丸め、折返しの
  どれを許し、作者が意図しない範囲外 read を正常に見せる不利益をどう評価するか。
- `i32[]` / `string[]` / `bool[]` の要素を、既存の型別 evaluator / writer と
  どうつなぐか。`TypedValue` は導入しない。

### DD-M4-P3-003 — loop binder を使う項目ごとの条件表示

**問い:** `for` の body 内にある `if` が element binder / index binder を
条件として読める範囲をどこまで開き、条件変化による部分木の追加・除去を
既存の positional iteration とどう整合させるか。

**独立した問いにする理由:** これは collection を外から読む話ではなく、
既存 `for` の局所 scope を既存 `if` の条件位置へ広げる話である。値が変わる
だけの binding と異なり、結果は widget / Visual / effect / handler の生成・
破棄になるため、Phase 2 の hover・focus・handler registration にも触れる。

**具体例（綴りは未決）:**

```wasamo
for photo, index in photos {
    VStack {
        Text { text: photo }
        if index == selected_index {
            Text { text: "Selected" }
        }
    }
}
```

上の例が実現するのは「各項目の下に選択印の部分木を置く」ことである。
非選択項目に透明な印を常駐させることではない。条件が false ならその部分木、
その effect、そこにある handler は存在しない。

現物には、単純合成を阻む既知の二点がある。`for` body を作る経路は
`loop_context` を持つが、現在の conditional effect は通常の binding context で
条件を評価する。また false → true の再挿入は loop context なしの `build_node`
を使う。このため、たとえば 3 番目の項目で `index == selected_index` が true に
なっても、条件評価が `index` を読めない、または挿入された `Text { text: photo }`
や handler が 3 番目の binder を失う可能性がある。どの seam を採るかは未決だが、
**条件 effect と再実体化の両方**を DD-003 が答える必要がある。

**下位の問い:**

- element binder / index binder のどちらを条件で読めるか。collection 要素型に
  応じてどの述語を許すか。
- binder と component state の両方を読む条件の dependency をどう持つか。
- per-item condition effect が、attachment 時の値を凍結せず、現在位置の item / index
  を invocation / re-evaluation 時に読むための context をどこが所有するか。
- 同じ位置の要素が差し替わったとき、現在値を読み直す既存 positional semantics
  をどう保つか。
- false → true で作り直した子、その binding、handler に、同じ loop context を
  どう渡すか。条件だけ直して再実体化側で binder を失う半分の修正を防ぐ。
- present / absent の切替時も、既存 `if` の fresh subtree、document order、
  effect disposal、同期 drain の契約をそのまま使えるか。
- 構造変更後に hover path、focus anchor、handler registry 再照会の再発条件を
  踏むか。踏むなら既存 Phase 2 契約の中でどう閉じ、踏まないならどの call path
  により不発と判断するか。
- 再実体化を既存 layout dirty → `flush_layout` → `sync_visuals` へ戻し、
  layout / arranged rectangle / Composition geometry の別 writer を作らないことを
  どう構造的に守るか。
- nested `for`、bare nested control flow、複数 widget body、key identity、
  shadowing を開かずに済む境界はどこか。

### DD-M4-P3-004 — equality による単一識別値の選択

**問い:** 1 つの識別値（gallery では `selected_index`）と equality を使って、
「現在選ばれている項目」を既存の表示 surface へどう投影するか。値が collection
内のどの項目にも対応しないとき、作者にどの契約を示すか。

**独立した問いにする理由:** equality 演算子の一般的な型規則は DD-001 が
持つが、M3 から送られた selected-state の責任は「演算子を足すこと」だけでは
閉じない。作者が O(N²) の bool state と一括代入をやめ、1 つの state から
排他的な見た目を作れることまで確認する必要がある。一方で group widget や
two-way binding を同時に設計してはならない。

**現在と目標の違い（綴りは未決）:**

```wasamo
// 現在: 選択肢ごとに bool state を持ち、各 click で全 state を書く。
state tab_all_selected: bool = true
state tab_albums_selected: bool = false

// Phase 3 が可能にしたい書き方の意味:
state selected_index: i32 = 0
for photo, index in photos {
    ToggleButton { checked: index == selected_index }
}
```

**下位の問い:**

- equality の bool 結果を既存 `checked` binding に直接使うか、DD-003 の
  条件部分木で選択印を出すか、両方を同じ規則で許すか。
- click が書くのは引き続き author-owned `selected_index` でよいか。
  `ToggleButton` の自己更新や two-way binding を混ぜない境界をどう示すか。
- `checked`（selected）と `focused` を別状態のまま保てるか。たとえば Tab で
  別の thumbnail に focus が移っても、handler が `selected_index` を書くまでは
  選択表示を移さないこと、同じ thumbnail が selected + focused なら Phase 2 の
  合成表示を退行させないことを、作者契約としてどう説明するか。
- 重複値を持つ collection で「値の equality」を使う場合と、index equality を
  使う場合の違いをどう説明するか。gallery の排他性は index equality で
  証明するか。
- `selected_index` が −1 / N のときは「選択表示 0 件」でよいのか、それとも
  選択 state 自体を不正と診断するのか。添字読みの失敗契約（DD-002）と
  混同しないこと。
- group surface、widget-owned state、generic Toggle / appearance を M5、
  two-way binding を Phase 7 に残したまま、どこまでを Phase 3 の選択契約とするか。

### DD-M4-P3-005 — 小さい一般的な handler control flow

**予約状態:** **owner-required、plan-revision gate 完了。** gallery の左右端を
key と button の両方で止めること、そのための能力を gallery 専用ではなく
一般的だが小さい surface とすることは、2026-08-11 の owner 回答で必須になった。
Revision 4 は `4afa204` で land 済み。ADR 起草は §2.3 / §2.4 完了後とし、実装は
さらに ADR Accepted と実装計画を待つ。

**問い:** イベント handler の state write を「条件が成り立つときだけ」
実行でき、gallery 以外にも再利用できる最小の author-facing control-flow surface
は何か。構造表示用 `if` と handler control flow をどう区別し、一般的な命令言語へ
広げずに Left / Right key と左右 button の 4 経路を安全に書けるようにするか。

**独立した問いにする理由:** 現在の `if` は widget body の structural member
であり、handler block の statement ではない。また Phase 3 の計画済み
predicate は count / empty / index read / equality である。先頭と末尾の両方を
守るには、equality だけでは通常足りない。Phase 2 close は「実行時診断」と
「handler で止める能力」の**両方**を Phase 3 へ送ったため、DD-002 の診断を
採れば DD-005 が不要になる、という関係でもない。

```wasamo
// 現在。先頭でも無条件に -1 へ進む。
key-down("ArrowLeft") => { root.selected_index -= 1; }

// 実現したい意味。これは仮記法であり、handler 内 if を予約しない。
key-down("ArrowLeft") => {
    if has_previous(photos, selected_index) {
        root.selected_index -= 1;
    }
}
```

`has_previous` 相当を表すには、たとえば relational comparison、件数との境界
比較、専用の index-valid predicate など複数の設計方向があり、どれも現行 plan
から自明ではない。`selected_index == 0` だけでは「0 でないとき」を書くための
否定または別分岐が要り、末尾判定には `count - 1` 相当も問題になる。

**plan-revision gate:**

**Disposition:** completed 2026-08-11. Revision 3 (`7763555`)、Revision 4
(`4afa204`)、Revision 5 (`1499241`) が個別に land した。以下は完了した gate の
監査手順として残す。

1. 本文書で、handler guard が既存 Phase 3 scope の暗黙の言い換えではなく、
   handler control flow と追加の境界 predicate を含み得る scope 追加であること、
   それでも Phase 2 の完了責任と gallery の自然な記述を閉じるため Phase 3 の
   必須成果にしたいという owner intent を、plan revision の入力として記録する。
2. Phase 3 に入れる場合は、[workflow.md §計画(plan)改訂の規律](../../../procedures/workflow.md)
   の **tier 2（scope / phase 構成）**として Revision-log entry 草案を別 artifact
   に起こす。handler guard の追加だけでなく、「Phase 3 は compiler-side」という
   現在の責務説明を compiler + runtime structural integration へ直す必要性も、
   同じ草案の別項目として監査する。initiator は agent、critical check と owner
   authorisation は `pending` から始める。
3. AC9 の意味を refine するなら `_roadmap.md` を mirror する。単なる phase
   supporting deliverable と整理できる場合でも、Phase 3 行の scope 追加と
   acceptance↔phase mapping への影響を点検する。
4. 既存 AC の意味、依存順、完了済み Phase 1・2 の評価、retro / merge gate、
   後続 Phase 5・7 との責任境界への impact check を行う。
5. critical check と owner authorisation がそろって plan 本文が land した後に、
   DD-005 を active にして設計判断へ進む。

**gate が開いた後の下位の問い:**

- guard の条件に DD-001 の全 predicate を使えるか、境界用の狭い形だけか。
- relational / logical / arithmetic のどれを本当に必要とするか。専用形で
  避ける場合、その専用形が将来の一般式を不自然に縛らないか。
- statement を条件実行する構文、guarded assignment、早期 return 相当などを
  どう比較するか。ここではどの綴りも予約しない。
- false のときは「何も書かない」でよいか。handler は実行済みとして event を
  consume するか。複数 statement のどこに guard の単位を置くか。
- Left / Right key と `<` / `>` click の 4 経路を同じ境界規則で書けるか。
  empty / 1 件 / 2 件以上の表を作り、片方だけ守れる surface を採らない。
- state write は handler return 後ではなく、その statement の中で reactive drain
  を同期的に起こし得る。guard を一度評価してから書く間、また複数 statement の
  前後で collection や構造が変わる場合、どの時点の predicate を読むか。
- guard 後の write が per-item conditional や caption を同期再評価し、その途中で
  範囲外 read が起きた場合の表示・effect containment は DD-002 の結論を消費する。
  DD-005 で別の rollback / transaction 契約を重ねて決めない。
- 一般関数、loop、`else` family、任意の command、M-expr4 全体を開かずに
  gallery の左右端を自然に書けるか。

**no-change option の意味:** 現行 plan を維持すると、DD-002 で範囲外 read の
契約は決められても、作者は Phase 3 の DSL だけで左右端の書き込みを guard
できない。gallery は key と click の 4 経路すべてで無効 state を作り得るままに
なり、2026-08-11 の owner requirement を満たさない。このため framing は
tier 2 revision を提案するが、plan 本文の変更自体は gate 完了まで行わない。

### DD-M4-P3-006 — handler assignment admission / 型検査の完全性

**問い:** 既知の不正 RHS を case-by-case に拒否するのではなく、すべての handler
assignment が、実行前に漏れなく position capability と LHS 宣言型へ照合される
完全性契約をどう作るか。その中で、現行仕様が binding-only とする string forms を
Phase 5 の書き込み能力より前に正しく診断するには、各 gate の責任をどう分けるか。

**独立した問いにする理由:** Phase 2 の実測は一つの string 診断漏れではなく、
handler assignment 全体に expected type を与える検査がないことを示した。
`i32 = "abc"` と `string = 5` の両方が `wasamoc check` を通り得るため、既知の
`StrLit` だけを拒否しても、別の式形・別方向の mismatch が残る。反対に、
scalar `string` write を Phase 3 で可能にすると Phase 5 の capability を先取りする。
したがって、**その RHS を handler position で使用できるか**と、**使用できる RHS の
型が LHS に適合するか**を二層に分け、どの assignment も両方の判定から漏れない
仕組みを設計する必要がある。

**Phase 3 で早期に拒否すべき異なる二種類の例:**

```wasamo
state count: i32 = 0
state caption: string = "Old"

Button {
    // 型不一致。string form の将来 capability とは無関係に不正。
    clicked => { root.count = "many"; }

    // 型は string 同士でも、Phase 3 では RHS が handler position に未許可。
    clicked => { root.caption = "New"; }

    // 逆方向の型不一致も同じ完全性契約で見逃さない。
    clicked => { root.caption = 5; }
}
```

作者にとって必要なのは、上の三例を invocation 時の evaluator error まで持ち越さず、
compile / load の早い段階で、型不一致なのか、この位置ではまだ使えない能力なのかを
区別して理解できることである。具体的な checker 構造や診断優先順位は ADR で決める。

**拒否してはいけない別の形:**

```wasamo
state archive: string[] = []

for label in labels {
    Button {
        clicked => { root.archive = root.archive.append(label); }
    }
}
```

これは scalar string assignment ではなく、`string[]` の whole-value collection
assignment であり、loop binder が append の要素値になる既存の許可形である。
「RHS のどこかに string があれば拒否」という検査では、完全性ではなく誤拒否を
増やしてしまう。

**下位の問い:**

- すべての scalar / collection handler assignment form が、LHS 宣言型を expected
  type とする判定を必ず通るという invariant を、どの境界で定義するか。
- `StrLit`、`StrPropRead`、`Interpolation` の binding-only rule を、型適合とは別の
  position-capability rule としてどう表すか。`Block` 等の再帰形でも漏れないか。
- Phase 3 で新設する expression forms と既存 forms を同じ完全性契約に乗せつつ、
  DD-001 の expression-result typing と DD-006 の assignment compatibility を
  二重実装しない責任分担は何か。
- `wasamoc check` を作者向け主 gate とするか。lowering は不正 AST を到達不能と
  みなすか、追加診断を持つか。
- textual / memory IR は `wasamoc` を通らないため、loader は type / capability
  invariant を defense-in-depth でどこまで再検査するか。
- evaluator の現在の拒否は最後の防御として残すか。残す場合、正常経路の
  作者向け診断とどの責任を分けるか。
- capability violation と type mismatch が同時に成立するとき、作者に最も
  根本的な修正を示す診断順位は何か。
- `string[]` append のような既存許可形、compound assignment、loop-local read を
  含む全 call site をどう列挙し、未検査経路がないことを ADR / implementation
  close で監査可能にするか。
- 診断は不正な RHS の source span、LHS 名、expected / actual type、または
  position capability のどれを示すか。Phase 5 という内部日程に依存せず、作者が
  現在できることを説明する文言にできるか。

---

## 論点の割り付け確認

| 必要な判断 | 担当 DD | 他 DD と混ぜない線 |
|---|---|---|
| 式の共通構文・型・式位置 | DD-001 | collection の範囲外時の挙動は DD-002 |
| count / empty / index read の reactive semantics と失敗 | DD-002 | per-item scope は DD-003 |
| loop binder を条件表示で読むことと構造 lifecycle | DD-003 | selected-state family 全体は DD-004 の範囲外 |
| equality で 1 state → 選択表示を作る author contract | DD-004 | group widget / two-way binding は後続へ残す |
| handler write の条件実行 | DD-005（owner-required、plan gate 待ち） | gallery 専用にせず、gate 前に syntax / 方式を決めない |
| 全 handler assignment の position admission と LHS 型適合 | DD-006 | expression-result typing は DD-001、scalar string write capability は Phase 5 |

この分離により、たとえば「collection 添字読みの範囲外診断を設計したから
handler guard も自動的に入った」「`==` を追加したから group widget まで決めた」
「文字列 RHS を早く拒否したから文字列を書けるようになった」という先取りを
防ぐ。

### DD にしない既知の基準線修正

`docs/dsl_spec.md` は revision 1.21 で Phase 2 implementation-synced と記録し、
§4.19 と landed implementation は `for` body の handler、および handler 内の
item / index read を認めている。しかし §8.11 の loader validation table には
現在も次の古い文言が残る。

- `no handler member inside a for body`
- item / index read は `binding positions` だけ

これは「Phase 3 で再び許可するか」という設計論点ではない。Accepted 済み
DD-M4-P2-005、Phase 2 の integration tests、loader の landed validation と答えが
既に一致しているため、Phase 2 の spec-sync 漏れとして `docs/dsl_spec.md` 1.22
（`6e3db4f`）で factual correction した。DD 番号は割り当てていない。
DD-001 / 003 / 006 は、訂正前の二行を制約として候補を狭めてはならない。

---

## オーナー回答の記録

**2026-08-11 — 独立レビュー後の owner intent 回答:**

- Phase 3 終了時に、gallery の左右端を Left / Right key と左右 button の両方で
  止められることを**必須**とする。
- その guard は gallery 専用能力ではなく、今後も使える**一般的だが小さい
  handler control-flow surface**とする。構文・IR・評価方式は未決である。
- handler assignment の型検査不足は全体として閉じたい。本質的な期待は、既知の
  case をアドホックに塞ぐことではなく、すべての assignment を漏れなく扱える
  仕組みを設けることである。Phase 5 の scalar string write capability は別である。
- 範囲外 index read の結果と、エラー時の表示 / effect containment は、framing で
  先に固定せず DD-002 の論点として判断する。

これらは owner intent と DD に課す必須成果の確認であり、正式構文・IR 方式、
範囲外 contract、診断方式を Accepted とする回答ではない。また handler control
flow と plan の cross-layer 責務記述を Frozen agreement に land する authorisation
でもない。別 artifact の tier 2 proposal に対する critical check と owner
authorisation は、具体的な old/new plan diff と impact を見た後に記録する。

**2026-08-11 — §2.2 agreement:** オーナーは改訂後の ①〜⑦ に合意した。
Revision 3 の根拠と変更範囲を critical check 済みとして authorise し、Revision 4
の AC9 / Phase 3 / ROADMAP 案、Revision 5 の AC9 / Phase 3 / Phase 5 / ROADMAP
案をそれぞれ authorise した。plan と ROADMAP の land は別コミットで記録する。

回答後も、今回は §2.2 だけで止める。§2.3 の scope、§2.4 の verification、
ADR の選択肢・推奨・結論は、次の明示的な作業として別に行う。

## Revisions

- **2026-08-11 — Initial §2.2 draft.** DD-M4-P3-001〜006 を提案。
  DD-005 は tier-2 plan-revision gate 付きの条件予約、DD-006 は診断責任だけで
  Phase 5 の string write capability を先取りしない形に分離した。
- **2026-08-11 — Pre-ADR spike assessment.**
  `exp/m4-phase-2-focus-spike` の履歴と成果物を Phase 3 の DD ごとに比較。
  Phase 3 全体には spike を必須化せず、5 つの具体的な発火条件が成立した
  DD だけに狭い spike を再提案する判断を、owner 合意項目 ⑥ として追加した。
- **2026-08-11 — Critical responsibility re-audit after Phase 1 / 2.**
  `plan.md` を固定前提ではなく仮説として再評価。compiler-only では閉じない
  runtime structural responsibility、Phase 3 に handler guard を入れる推奨方向、
  新規式 admission と既存 string RHS 診断の二層分離、selection / focus の独立、
  per-item conditional が loop context を失う二つの現行 seam、§8.11 の stale rows
  を記録した。plan / roadmap / normative spec 本文は本 §2.2 作業では変更していない。
- **2026-08-11 — Independent-review and owner-intent revision.** compiler-only の
  訂正と handler control-flow の追加を別の合意単位にした。gallery の 4 入力を
  全境界で止める owner requirement と、一般的だが小さい surface という境界を
  DD-005 に記録。DD-006 を binding-only string の個別診断から、position admission
  と LHS type compatibility を全 handler assignment に漏れなく適用する完全性問題へ
  広げた。範囲外 read の result / failure containment は DD-002 の未決論点に戻し、
  DD-005 が重複して rollback semantics を決めないよう責任を分離した。
- **2026-08-11 — §2.2 owner agreement and plan-gate authorisation.** 改訂後の
  ①〜⑦が owner-agreed。Revision 3 は owner critical check 済み、Revision 3〜5
  の具体案は個別に authorise された。§2.3 scope と §2.4 verification は未実施の
  ため、文書全体の status は `draft` のまま維持する。
- **2026-08-11 — Plan-gate and factual-correction landing sync.** Revision 3
  (`7763555`)、Revision 4 (`4afa204`)、Revision 5 (`1499241`) を個別に land。
  DD-005 の plan gate を完了扱いにした。`docs/dsl_spec.md` §8.11 の Phase 2
  spec-sync 漏れは version 1.22（`6e3db4f`）で訂正した。§2.3 / §2.4 と ADR は未着手。
