---
title: M4-Phase 3 フレーミング — 述語式
status: draft
created: 2026-08-11
target-phase: M4-Phase 3
workflow-stage: "2.3 / 2.4 owner review"
related:
  - process/milestone-4/plan.md
  - process/milestone-4/requirements/framing.md
  - process/milestone-4/requirements/spec.md
  - process/milestone-4/phase-1/implementation/handoff.md
  - process/milestone-4/phase-2/requirements/framing.md
  - process/milestone-4/phase-2/implementation/handoff.md
  - process/milestone-4/phase-3/requirements/plan-revision-6-proposal.md
  - docs/dsl_spec.md
---

# M4-Phase 3 フレーミング — 述語式

**状態:** draft（§2.2 owner-agreed、§2.3 / §2.4 owner review 待ち）

**今回実施した段階:**
[workflow.md §2.2〜§2.4](../../../procedures/workflow.md) の framing draft

**まだ実施していない段階:** §2.3 / §2.4 の owner agreement、§3 設計判断

この文書は、M4-Phase 3 で**何を決める必要があるか**を分け、DD 番号を
予約し、オーナーとの合意点を明らかにするための資料である。構文、IR、
実行方式、エラー文言の結論はここでは選ばない。

前段の accepted
[constraints.md](./constraints.md) は現時点の情報に基づく仮説として読む。
新しい情報で前提が変わった場合は再検討できるが、変更理由と影響を記録する。
今回読み直した Phase 1・2 の implementation handoff / retrospective / 現物、
M4 の計画・要件、現行の DSL 仕様と `.ui` の実例から、constraints の主要な
境界は維持できる。一方、当初の plan の二つの前提は、そのままでは維持できなかった。
一つは、計画済みの per-item conditional が compiler-side だけでは閉じないという
現物上の訂正である。もう一つは、旧 plan からは導出できなかった handler guard を
Phase 3 の必須成果に加えるというスコープ変更である。この二つは別々に owner
authorise され、Revision 3〜5 として land 済みである。本 §2.3 / §2.4 は、改訂後の
plan、roadmap AC9、`docs/dsl_spec.md` 1.22 を基準線にする。

> **例の読み方:** 以下の `count(photos)`、`photos[selected_index]`、
> `index == selected_index`、handler 内の `if` は、作者が実現したいことを
> 説明するための**仮の書き方**である。Wasamo の構文として予約したもの
> ではない。正式な綴りは後続の ADR で比較する。

---

## §2.2 でオーナー合意済みのこと

この節は §2.2 の合意記録である。

| ID | 合意してほしいこと | 提案 | 状態 |
|---|---|---|---|
| ① | **DD の分け方と番号** | DD-M4-P3-001〜006 の 6 件を予約する。001 = 共通の述語式、002 = collection 読み取りと範囲外 failure contract、003 = 項目ごとの条件表示、004 = 等値選択、005 = 小さい一般的な handler control flow、006 = handler assignment admission / 型検査の完全性。DD-005 の plan-revision gate は完了した | owner-agreed 2026-08-11 |
| ② | **計画済み per-item conditional の責務訂正** | Phase 3 は compiler-only では閉じず、DD-003 が condition evaluation、subtree 再実体化、focus / hover / handler registry / layout lifecycle までを cross-layer に設計する。これは handler guard の採否とは独立した、既存 Phase 3 deliverable の実現条件である | owner-agreed; Revision 3 landed `7763555` |
| ③ | **handler control-flow の Phase 3 追加** | gallery の Left / Right key と左右 button の 4 経路すべてを、empty / 1 件 / 複数件で範囲外へ進ませないことを Phase 3 の必須成果とする。gallery 専用命令ではなく、今後も使える**一般的だが小さい surface**を DD-005 で設計する。正式構文・IR・評価方式はここでは選ばない | owner-agreed; Revision 4 landed `4afa204` |
| ④ | **handler assignment 検査の完全性** | 個別の不正 RHS をアドホックに塞ぐのではなく、全 handler assignment が漏れなく検査される仕組みを DD-006 の要求にする。RHS が handler position で許されるかという **capability / position admission** と、許された RHS の型が LHS 宣言型に合うかという **type compatibility** を分ける。scalar `string` write capability は Phase 5 のまま | owner-agreed; Revision 5 landed `1499241` |
| ⑤ | **範囲外 read の判断範囲** | runtime error / fallback / clamp のどれを契約にするか、失敗時に旧表示・対象 effect・後続 effect をどう扱うかを DD-002 の未決論点にする。Phase 2 の runtime diagnostic 期待は有力な入力だが、ADR 前の結論にはしない。DD-005 の guard は範囲外 state を予防する別責務であり、DD-002 の代替ではない | owner-agreed 2026-08-11 |
| ⑥ | **pre-ADR spike の要否** | Phase 3 全体には必須化しない。現物調査で per-item conditional の不足箇所は既に二つの loop-context seam として特定でき、未知の状態モデルを試作で発見する段階ではない。下記の発火条件が実際に成立した DD だけに、「何を観測できれば答えか」を限定した spike を ADR Accepted 前に提案する | owner-agreed 2026-08-11 |
| ⑦ | **現行仕様の矛盾の扱い** | `docs/dsl_spec.md` §8.11 に残っていた「`for` body の handler 禁止」「item/index read は binding だけ」という二行は、Phase 2 の実装同期漏れとして factual correction し、Phase 3 の新しい DD や禁止前提にはしない | owner-agreed; corrected in `6e3db4f` |

①〜⑦はいずれも、現時点の情報に基づく仮説として合意済みであり、新しい実測・
設計上の発見があれば見直せる。②〜④に関係する plan 改訂手続きも完了している。

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
owner authorisation を求めるべきだ、という**論点設定上の結論**であった。その gate
は [plan-revision-proposal.md](./plan-revision-proposal.md) の Revision 3〜5 として
完了し、改訂後の plan / roadmap を本節以降の固定入力にする。

## §2.3 / §2.4 でオーナーに合意してほしいこと

下記の詳細な scope / verification を要約した owner alignment packet である。
推奨どおりなら「⑧〜⑬ OK」、変更したい項目があればその番号を指定してほしい。

| ID | 合意してほしいこと | 提案 | 状態 |
|---|---|---|---|
| ⑧ | **Phase 3 の In scope** | 改訂後 AC9 の全要素を本フェーズで閉じる。DD-001〜006 の式・collection read・per-item conditional・equality selection・小さい handler control flow・全 handler assignment の admission / 型適合に加え、per-item conditional の runtime structural lifecycle と Gallery A の named consumers までを含む | owner review pending |
| ⑨ | **Out of scope** | 文字列連結、一般算術・一般命令言語、`TypedValue` / 構造化 item、keyed / nested iteration、新 widget / selection ownership、two-way binding、scalar `string` write、ABI、window / image / scrolling、focus model の再設計を含めない。DD が選ぶ正式構文・IR・範囲外 failure contract は「Phase 3 外」ではなく「framing では未決」と区別する | owner review pending |
| ⑩ | **M4 acceptance criteria との対応** | Phase 3 が discharge する milestone criterion は AC9。AC1 は構造更新で focus / hover / handler lifecycle を退行させない回帰対象、AC12 は Gallery A を段階的に成熟させる consumer だが、どちらも Phase 3 単独で完了を主張しない | owner review pending |
| ⑪ | **検証の三層** | OS 非依存の checker / lowering / IR / evaluator は unit test、実 `.ui` → IR → runtime と構造 lifecycle は mock-free Windows integration test、作者に見える Gallery 成果は launch + screenshot + assistant analysis で閉じる。出荷画面に不自然な検証 UI が必要なら named mechanism fixture に分ける | owner review pending |
| ⑫ | **識別可能な陽性対照と境界 matrix** | count / empty / per-item presence は collection の状態を変えて結果が反転する対照、caption / selection は異なる index へ移動する対照、guard は key / button の 4 producer を empty / 1 件 / 複数件の両端で試す。範囲外 read は DD-V-029 の red-test、assignment 完全性は全 variant の call-site audit + reject / admit matrix で証明する | owner review pending |
| ⑬ | **将来の追加を塞がない二つの設計義務** | Out of scope に置いた能力のうち二つは、Phase 3 が今決める形の下流にある。多段 `for` と入れ子の構造制御は DD-003 が選ぶ loop context の所有・寿命に依存し、Phase 5 の scalar `string` write と、既存スカラに収まる値を生む式が将来入る場合は DD-006 の position admission / 型適合の枠組みに乗る。**Phase 3 の scope はどちらにも広げない。**その選択が将来の追加を構造的に排除しないことだけを、両 DD の判断要件に加える | owner review pending |

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
- 条件評価と再実体化で loop context を所有・保持する形が、将来の多段 `for` と
  入れ子の構造制御を**構造的に排除しない**か。Phase 3 でそれらを開くことは
  求めない。ただし一段専用と分かる形（単一の context を上書きで持つ、binder 名を
  loop 外と同じ経路で解決する等）を選ぶなら、多段化に必要な変更が後から追加
  として入るのか、既存 `.ui` の意味を変えるのかを ADR が判定して記録する。

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
比較、専用の index-valid predicate など複数の設計方向があり、どれも改訂済み plan
が方式まで固定するものではない。`selected_index == 0` だけでは「0 でないとき」を書くための
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

**land 済み scope の下位の問い:**

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

**revision 前の no-change option の意味:** 旧 plan を維持すると、DD-002 で範囲外
read の契約は決められても、作者は Phase 3 の DSL だけで左右端の書き込みを guard
できなかった。gallery は key と click の 4 経路すべてで無効 state を作り得るままに
なり、2026-08-11 の owner requirement を満たさないため、この option は Revision 4
で退けられた。

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
- position capability と type compatibility の枠組みが、将来の消費者を
  **個別 case の追加ではなく同じ枠組みへの登録として**受け入れられるか。
  最初の消費者は Phase 5 の scalar `string` write（許可される RHS が増える）と、
  M4 の外に置かれている、既存スカラに収まる値を生む式（結果型を持つ RHS が
  増える）である。どちらも「新しい RHS 形ごとに checker へ分岐を足す」形に
  なるなら、DD-006 が主張する完全性は landing 時点のスナップショットにすぎず、
  次の追加のたびに全経路を洗い直すことになる。この点は DD-006 の完全性要求に
  既に含まれるが、判定できるよう将来の消費者を明示する。

---

## 論点の割り付け確認

| 必要な判断 | 担当 DD | 他 DD と混ぜない線 |
|---|---|---|
| 式の共通構文・型・式位置 | DD-001 | collection の範囲外時の挙動は DD-002 |
| count / empty / index read の reactive semantics と失敗 | DD-002 | per-item scope は DD-003 |
| loop binder を条件表示で読むことと構造 lifecycle | DD-003 | selected-state family 全体は DD-004 の範囲外 |
| equality で 1 state → 選択表示を作る author contract | DD-004 | group widget / two-way binding は後続へ残す |
| handler write の条件実行 | DD-005（owner-required、plan gate 完了） | gallery 専用にせず、framing で syntax / 方式を決めない |
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

## Phase 3 の対象範囲（§2.3 scope）

### 含むもの（In scope）

- **DD-001 — 共通 predicate surface。** Phase 3 で必要な式の構文候補、結果型、
  使用できる expression position、scope / type admission を決める。DD-002〜005 が
  同じ規則を消費できることまでを含むが、一般式言語を完成させる責任は負わない。
- **DD-002 — collection read。** `for` の外からの count、empty、index read、
  collection / index の reactive dependency、各既存 element type の結果型、範囲外
  result / failure contract と effect containment を決める。runtime error、fallback、
  clamp の比較を framing で先取りしない。
- **DD-003 — per-item conditional。** loop item / index を条件から読み、条件評価と
  false → true の subtree 再実体化の両方で現在位置の loop context を保つ。
  subtree の生成・破棄を既存 effect、handler registry、focus / hover、layout / visual
  lifecycle へ統合し、新しい構造 writer を作らない。
- **DD-004 — equality selection。** 一つの author-owned discriminant から既存の
  `checked` binding または条件部分木へ排他的な選択表示を投影し、focus と selection
  を別状態のまま保つ。
- **DD-005 — 小さい handler control flow。** Gallery A の Left / Right key と左右
  button の四つの producer を、empty / 1 件 / 複数件の両端で止められる、gallery
  専用ではない最小 surface を決めて届ける。guard と範囲外 read の failure contract
  は別責務のまま保つ。
- **DD-006 — assignment validation の完全性。** 全 handler assignment を実行前に
  position capability と LHS / RHS type compatibility の両方へ漏れなく通す。
  既存の正当な `string[]` whole-value mutation を維持しつつ、scalar `string` write
  は未提供として区別して診断する。
- **出荷 consumer と規範同期。** Gallery A の status-bar count、lightbox caption、
  current-thumbnail selection、四つの境界 guard を実 `.ui` → IR → runtime で通す。
  empty と per-item presence が Gallery に不自然な UI を要求する場合だけ、同じ production
  path を通る named mechanism fixture を補助 consumer にする。ADR Accepted 時と phase
  close 時には、選ばれた surface を `docs/dsl_spec.md` と必要な architecture 記述へ同期する。
- **ABI 非変更。** 新しい C ABI entry point、value carrier、host callback を作らず、
  既存の DSL / textual IR / loaded IR / runtime 内部経路で閉じる。もし ADR 調査で ABI が
  不可避と判明した場合は scope 内へ暗黙に吸収せず、framing / plan の再確認へ戻す。

accepted [constraints.md](./constraints.md) から後続判断で変わった箇所は明示的に
reconcile する。「compiler-side phase」は Revision 3 の cross-layer responsibility が
置き換える。§1 の「範囲外 read は runtime diagnostic」は後の §2.2 合意 ⑤により
DD-002 の比較へ戻り、§2 の handler guard gate と §3 の個別 string 診断は Revision 4 / 5
により、それぞれ DD-005 の必須 surface と DD-006 の全 assignment 完全性へ広がった。
ABI 非変更、既存 iteration semantics、単一 layout / visual writer、検証規律など、
その他の制約は維持する。

### 含まないもの（Out of scope）

- 文字列連結、汎用の `+ - * /` expression、任意関数、handler loop、一般的な
  `else` family、early return、任意 command からなる一般命令言語。DD-005 が Gallery
  の境界表に必要だと示す最小 predicate / control-flow はこの除外の例外だが、何を
  最小とするかは ADR が比較する。
- record / object collection、`TypedValue`、item field access、keyed identity、
  nested `for`、binder shadowing、複数-widget / member-range の `for` body。
- `RadioGroup`、`SegmentedControl`、新しい選択 widget、widget-owned selection、
  generic Toggle appearance、Button の Space / Enter activation。
- two-way binding と scalar `string` state を handler から書く能力。前者は M4-Phase 7、
  後者は M4-Phase 5 のままである。
- ABI / host state、window、TextField / IME、scrolling / scrollbar、`Image`、top layer、
  accessibility、theming、author-controllable sizing。
- Phase 2 の routing / focus / modal-scope / hit-test / identity policy の再設計。
  DD-003 の構造更新で退行が見つかった場合は既存契約を修復し、契約自体の変更が必要なら
  Phase 2 ADR の successor と scope 再確認を先に行う。
- `docs/dsl_spec.md` 1.22 で訂正済みの `for` body handler / handler-position binder
  admission を再審議すること、unknown signal 名や focus annotation の診断を便乗して
  広げること。

次は Out of scope ではなく、**framing では決めず §3 の ADR に残す事項**である：正式な
author syntax、AST / IR の形、evaluator / dependency-tracking 方式、範囲外時の具体的な
error と effect containment、診断文言と優先順位、DD-004 の表示投影方式、DD-005 の
control-flow 方式。ここで一案へ固定しない。

#### 別 phase が所有する除外項目を Phase 3 で読み直さない線

上の除外のうち他 phase が所有するものは「Phase 3 の外」であって「pre-1.0 の外」では
ない。Phase 3 がそれらの進捗を主張しないのと同様に、**pre-1.0 に着地するかどうかの
判断も Phase 3 では行わない。** 判断の所在は plan 側にあり、次の二つは撤回・遅延の
余地が明示されているぶん Phase 3 から触りたくなるが、いずれも既に所有者と判断点を
持つ。

- **`Image` と直値 `fill`** — M4-Phase 4 所有。AC を持たない予備段の取り込みだが、
  撤回を判断する stretch 再評価チェックポイントは M4-Phase 2 の ADR Accepted flip で
  発火し、**2026-08-05 に両方を保持する形で discharge 済み**である
  （[plan.md](../../plan.md) §Cross-phase dispositions 3）。未発火の gate は残って
  いないので、Phase 3 が再評価を提案する理由はない。
- **作者が制御する寸法** — M4-Phase 10 が調査、M5 が実装、M6 が凍結前 disposition と
  いう控えを持つ（[plan.md](../../plan.md) §Cross-phase dispositions 2、
  [sizing VDR](../../../cross-milestone/decisions/author-controllable-sizing-surface.md)）。
  同じく Phase 3 の論点ではない。

ただしこの二つを読み直す過程で、**plan がまだ持っていない観測**が一つ出た。M6 凍結の
前に M5（または M4 と M5 の間）へ入りうる pre-1.0 項目は、作者が制御する寸法の実装
だけではない。[candidate pool](../../../candidate-pool.md) の `TypedValue` / 構造化
データも「M4〜M5 の専用スロット」に傾いており、その ABI 影響判定は M4-Phase 7 が負う。
plan §Cross-phase dispositions 2 が spike を M4 に置いた判定入力の一つ「M5 は spike と
実装の両方を吸収できるか」は、M5 が同時に `TypedValue` スロットも吸収する可能性を
勘定に入れていない。これは Phase 3 が答える問いではなく、**M4-Phase 7 の ABI 判定が
出た時点で plan 側が持つべき入力**である。ここでは論点として記録するにとどめ、Phase 3
の scope にも DD にも入れない。

### M4 acceptance criteria との対応

| M4 criterion | Phase 3 での扱い | Phase 3 close で主張すること |
|---|---|---|
| **AC9 — Expression predicates** | **本フェーズの主 acceptance criterion。** 改訂後 AC9 の collection read、per-item conditional、equality selection、小さい handler control flow、全 assignment の admission / type compatibility と、明記された除外を DD-001〜006 で分担する | 下記 verification matrix の全行と Gallery / fixture の author-facing outcomes が揃い、AC9 の Phase 3 割当を discharge できる |
| **AC1 — input / focus** | DD-003 の subtree mutation が既存 focus / hover / handler lifecycle を通るための**回帰境界** | Phase 2 の契約を壊していないことだけ。AC1 全体を再度 discharge したとも、新しい input 能力を追加したとも主張しない |
| **AC12 — two showcases** | Gallery A を Phase 3 の consumer として段階的に成熟させる | Phase 3 の named Gallery slice を示すだけ。Gallery completion は Phase 4、二つの showcase 全体の完了は後続 phase / milestone close に残す |
| その他の AC | 本フェーズでは所有しない | 進捗・完了を主張しない |

---

## 検証方針（§2.4）

### 証拠レイヤー

1. **OS 非依存の unit test。** parser / checker / lowering / textual-IR emission、
   expression typing、純粋な evaluator / dependency bookkeeping は各 crate の unit test
   と `wasamoc` round-trip test で固定する。Win32 / WinRT-bound object は mock しない。
2. **mock-free Windows integration test。** 実 `.ui` を `wasamoc` で check / lower / emit
   し、loader と production runtime を通す named fixture で、reactivity、subtree
   lifecycle、input producer、focus / hover / handler regression を確認する。CI の必要
   runtime capability が無ければ黙って skip せず失敗する既存方針を維持する。
3. **出荷 Gallery の可視証拠。** GUI 表示を成果とする slice は launch + screenshot
   capture + assistant analysis を必須とし、単なる process survival は補助信号に留める。
   誤実装でも同じ一枚になり得ないよう、状態を変える前後脚と復帰 / no-change 脚を置く。
4. **spec / call-site audit。** ADR Accepted 時の design sync と phase close 時の
   implementation sync で `docs/dsl_spec.md` を現物へ合わせる。DD-003 は structural
   side-effect / writer enumeration、DD-006 は assignment variant と検査 call site の
   対応表を close artifact に残す。

### DD と検証手段の対応（仮確定）

| DD | 純ロジック / compiler 証拠 | 実経路 / consumer 証拠 | 固有の failure / review obligation |
|---|---|---|---|
| **DD-001** | 各採用 expression の parse、type、scope / position admission、lowering、emit / load round trip。隣接する不許可位置と型の reject case を対にする | DD-002〜005 の production-path fixture が同じ共通規則を消費する | IR schema を変更する案なら full independent review。prototype の通りやすさを ADR 選択理由にしない |
| **DD-002** | count / empty / valid index の全 element-type matrix、collection と index の dependency 再実行、範囲外と containment の決定済み matrix | Gallery の count / caption。empty の自然な出荷状態が無ければ named fixture で non-empty ↔ empty を反転させる | 範囲外分岐は DD-V-029 に従い、対象 test とそれを落とした誤実装を close artifact に記録する |
| **DD-003** | binder の型 / scope admission と lowering、nested / shadowing 等の非採用境界 reject | false ↔ true、same-length replacement、remove / reinsert を実 `.ui` 経路で行い、loop context、effect / handler 登録、layout、hover / focus の再発条件を確認。Gallery に自然な表示が無ければ named fixture | runtime structural change と GUI evidence は full independent review。既存 writer と lifecycle への call-site / side-effect audit を必須にする |
| **DD-004** | equality の admitted type pairs、result=`bool`、mismatch reject | Gallery で一つの `selected_index` から選択表示 1 件を作り、別 index への移動、focus-only 移動、範囲外 discriminant 時の ADR 決定を区別する | selected と focused の同時成立表示を Phase 2 基準線から退行させない |
| **DD-005** | 採用した guard / control-flow の parse、admission、lowering、IR、false=no-write と評価時点の unit cases | key Left / Right、button previous / next の四 producer × empty / 1 件 / 複数件（先頭・中間・末尾）matrix。少なくとも key と button の双方を production input path で通す | schema / runtime structural risk に応じ implementation gate の review level を選ぶ。DD-002 の rollback contract を重複して決めない |
| **DD-006** | 全 `HandlerExpr` variant を RHS-capable / non-RHS に分類し、全 assignment form について LHS declared type × position capability の admit / reject matrix を作る。`i32 = "many"`、`string = 5`、未許可 scalar-string write を reject し、既存 `string[]` append を positive control にする | `wasamoc check` を作者向け gate として通し、ADR が loader defense-in-depth を選ぶ場合は textual / memory IR の reject も実 loader で確認する。GUI は完了条件にしない | variant 一覧と checker / lowering / loader / evaluator の call-site audit を ground truth と照合する。個別 case 追加だけで「完全」としない |

### AC9 の discharge matrix

| AC9 の要素 | 最低限必要な証拠 |
|---|---|
| collection count / emptiness / index access | type・dependency・境界の unit matrix、non-empty ↔ empty で count / empty を反転する fixture、Gallery の status-bar count と二つ以上の index に対応する caption |
| per-item conditional rendering | binder 条件の compiler matrix、実 runtime の present ↔ absent ↔ present、same-length replacement、構造 lifecycle 回帰、識別可能な可視対照 |
| equality-based selection | type matrix、Gallery で exactly-one selection、異なる index への移動、focus-only 移動との分離 |
| small reusable handler control flow | 四 producer × empty / 1 / multiple matrix。内部へ一度動ける positive leg と、各端で余分な同方向入力が no-change になる boundary leg |
| every handler assignment checked | 全 variant の call-site audit、admit / type-mismatch / capability-mismatch matrix、既存 `string[]` mutation の non-regression |
| explicit exclusions | 文字列連結・一般算術が偶然 admission されていない negative cases、scalar `string` write の capability reject。一般言語全体の網羅テストは要求しない |
| shipped author path | `examples/gallery/gallery.ui` が workspace build の build script を通って check / lower / emit され、Gallery runtime と可視 evidence が同じ artifact を消費する |

### GUI 陽性対照と owner-visible smoke の範囲

- **count / empty / per-item presence:** collection を non-empty → empty → non-empty、
  または条件を false → true → false に変え、表示が対応して反転する。固定の件数文字列や
  最初から空の一枚だけでは証拠にしない。Gallery に不自然な mutation UI を足さず、
  必要なら named mechanism fixture を使う。
- **caption / equality selection:** 少なくとも二つの異なる index を選び、caption の値と
  exactly-one selection が一緒に移ることを示す。Tab だけで focus を移した脚では
  selection が移らないことを分離対照にする。
- **boundary guard:** 先頭から一度内側へ動ける脚を示してから先頭へ戻り、Left key と
  previous button を余分に入力しても caption / selection が変わらないことを示す。
  末尾も Right key / next button で対称に行う。empty / 1 件の全 matrix は自動 integration
  test を主証拠とし、静止画を大量に並べることは要求しない。
- assistant capture / analysis は実装 close 前の baseline であり、owner-visible smoke を
  置き換えない。owner smoke の具体手順は implementation plan で、採用された構文と
  Gallery UI に合わせて確定する。

### build / CI 方針

- 新しい言語・build system を導入しないため、**CI workflow 更新は予定しない**。
- cold debug verification は `cargo build --workspace` の後に
  `cargo test --workspace` を行う。Phase 3 の compiler / pure-logic test と mock-free
  Windows integration test は workspace suite に載せ、CI の既存 Windows runner で
  gate する。
- runtime または出荷 Gallery を変えた GUI evidence の前には
  `cargo build --release --workspace` を行う。これが `gallery-rust` の build script を
  通じて出荷 `gallery.ui` の check / lower / emit も行い、stale uplifted rlib を避ける。
- deterministic failure が出た場合は同一 command を一度 rerun して再現 / 非再現を記録し、
  close artifact で disposition する。実装 task の gate 選択と具体的な test 名は §4 で
  固定し、本 §2.4 では証拠の役割と最低 matrix だけを凍結する。

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

この回答時点では §2.2 だけで止めた。§2.3 の scope、§2.4 の verification、
ADR の選択肢・推奨・結論は、次の明示的な作業として別に残した。

**2026-08-11 — §2.3 / §2.4 owner review request:** 上記 ⑧〜⑬と、
§Phase 3 の対象範囲 / §検証方針を owner review へ提示した。scope と verification
以外の ADR 選択肢・推奨・結論は起草していない。回答は pending。

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
- **2026-08-11 — §2.3 / §2.4 owner-review draft.** 改訂後の M4 plan / roadmap
  AC9、land 済み plan-revision proposal、accepted constraints、`dsl_spec.md` 1.22 を
  突き合わせ、In scope / Out of scope / AC 対応と DD / AC9 discharge matrix を追加。
  純ロジック unit、mock-free Windows integration、状態遷移つき Gallery GUI evidence、
  DD-V-029 red-test、assignment call-site audit の役割を仮確定した。owner 回答までは
  `draft` を維持し、ADR 起草・実装へ進まない。
- **2026-08-12 — 将来の追加を塞がない設計義務の追加。** Out of scope の各能力を
  1.0 までの取り込み価値で並べ直したところ、二つが Phase 3 の選択の下流にあると
  分かった。多段 `for` / 入れ子の構造制御は DD-003 の loop context 所有・寿命に、
  Phase 5 の scalar `string` write と将来の値を生む式は DD-006 の admission /
  型適合の枠組みに依存する。scope は広げず、将来の追加を構造的に排除しないことを
  ADR の判断要件として合意項目 ⑬ と両 DD の下位の問いに加えた。
- **2026-08-12 — 別 phase 所有の除外項目を読み直さない線。** `Image` / 直値 `fill` と
  作者が制御する寸法について、撤回・遅延の判断点が既に plan 側にあること（前者は
  stretch 再評価が 2026-08-05 に discharge 済み、後者は M4-Phase 10 → M5 → M6 の控え）
  を §含まないもの に記録し、Phase 3 が再評価を提案しない線を引いた。あわせて、
  M5 の pre-1.0 スロットを作者制御寸法の実装と `TypedValue` が取り合いうるという、
  plan がまだ持たない観測を、M4-Phase 7 の ABI 判定後に plan が消費する入力として
  記録した。Phase 3 の scope と DD には入れていない。
- **2026-08-12 — Owner-alignment independent-review remediation.** accepted
  `constraints.md` を、その文書自身の再評価 allowance に従って Revision 3〜5 と
  framing 合意⑤へ同期した。候補プールでは Button keyboard activation の根拠を
  `dsl_spec.md` 1.21 と `key-down("Enter")` の互換性へ訂正し、focus annotation の
  pre-1.0 扱いを制約ではなく再評価可能な scheduling hypothesis として記述した。
  `TypedValue` については Phase 7 の decision ownership を維持しつつ、Phase 7 / 8 の
  ABI 影響を ADR 前に固定しないよう [M4 plan Revision 6 proposal](./plan-revision-6-proposal.md)
  と候補行を限定修正した。
  ⑧〜⑬の scope / verification proposal 自体は変更しておらず、owner agreement は
  引き続き pending である。
