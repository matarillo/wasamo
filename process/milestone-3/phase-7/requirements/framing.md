---
title: M3-Phase 7 framing — イテレーション grammar
status: aligned
created: 2026-06-11
aligned: 2026-06-11
target-phase: M3-Phase 7
---

# M3-Phase 7 framing

**Status:** aligned (owner alignment 2026-06-11; ⓪〜⑧ 全項目 OK)  
**Targets phase:** M3-Phase 7 (iteration grammar / collection-driven widget-tree generation)

プロジェクトの開発プロセス（[workflow.md §2](../../../procedures/workflow.md)）に従い、
本 note は設計判断（ADR）を書く前に **owner とフレーミングを合意**するための入力資料。
個別 DD の Options / 比較 / Recommendation は本 note では確定しない。ここで確定するのは、
Phase 7 が何を証明する phase なのか、どの論点を DD として予約するのか、どこまでを
Phase 7 scope とし、どこからを trigger 付きで後続へ送るのか、そして検証方針である。

先行 M3 phase から本 framing が継承する規律（再導出しない）:

- **Two-moment spec-sync**: Moment 1 = ADR-Accepted commit での design-spec draft、
  Moment 2 = phase close での implementation re-sync。
- **Moment is not a commit unit / review-concern 単位 commit**。
- **No fast-track**: 全 merge は owner 明示承認を要する。
- **Final-task retrospective split**: 最終 task の task-end retro と phase-end retro / CI run-id
  所有を最初から分ける。
- **Implementation gates**: semantic migration / side effects / parallel data drift / GUI positive
  control は、実装開始時に選択し、close-gate artifact で閉じる。
- **制約引き継ぎ**: 本 framing は [constraints.md](./constraints.md)（§2.1 アウトプット、
  accepted）を前提として読む。ControlFlowNode family、ForLoopSubtree landing point、
  identity open issue、placement storage decision、range failure observability、reactive-drain
  fix-or-carry、TypedValue 圧力判断、visible E2E 陽性対照はそこで Phase 7 入力として確定済み。

---

## 今回オーナーに決めてほしいこと（Owner alignment packet）

**この節だけ読めば合意判断できる。** 推奨でよければ「OK」、変えたい項目だけ指示してください。
詳細な根拠は後続の各節に置く。

| ID | 決めてほしいこと | 推奨 | 詳細 |
|---|---|---|---|
| ⓪ | **Phase 7 以降の比較軸** | owner-intent answers §0 を framing FD として採用する。過去合意は仮説として扱い、product merit / thesis 整合を主軸に比較する。実装・改訂コストは独立軸ではなく tie-breaker。ただし「コスト非重視」は maximalism ではない。将来拡張性を理由とした過剰設計は、引き続き design-decision review の失敗モードとして扱う。M3 の他 AC、特に Phase 8 / public draft を脅かす schedule risk は framing §Risks で扱う。 | [FD-P](#fd-p-phase-7-以降の比較軸product-merit-主軸) |
| ① | **architectural-family trigger の confirm-or-strain** | architectural-family.md の trigger 1（M3 DSL spec 起草開始）と trigger 3（`BindingTarget` に収まらない binding feature の提案）が Phase 7 で発火していることを認める。そのうえで、Phase 7 iteration は tree-with-bindings family 内の `BindingTarget` / `ControlFlowNode` 拡張で吸収でき、view-function re-execution family への pivot や VDR 昇格は不要と confirm する。 | [FD-Q](#fd-q-architectural-family-トリガーの-confirm-or-strain) |
| ② | **Phase 7 thesis** | Phase 7 は `if` から `for` への structural control-flow family 拡張であり、静的展開ではなく、**実行時可変 collection binding が generated subtree の cardinality を駆動する**ことを証明する。A8 の改訂は不要。 | [FD-A](#fd-a-phase-7-thesiscollection-binding-が-cardinality-を駆動する) |
| ③ | **visible proof / 陽性対照** | gallery thumbnail set を collection から生成し、body 外の text Button 等による最小 handler mutation（append / truncate 相当）で item 数を増減させる。初回 proof の基本線は runtime-owned collection state であり、host set / replace API は必須範囲に含めない。固定 N の単発 screenshot は不可。増えた frame / 減った frame の 2 frame 以上で証明する。 | [FD-B](#fd-b-visible-proof固定-n-ではなく増減-2-frame-で証明する) |
| ④ | **初回 surface の境界** | 初回 surface は runtime-owned collection state、un-keyed base、append/truncate-only、scalar item、flat scope を基本線にする。host-supplied initial state / host replace / in-out write-back は v1 までに欲しい能力として肯定的に残すが、general host state boundary は iteration cardinality proof とは別 thesis であるため Phase 7 必須範囲には含めない。単値 state の host write API もまだ無いことは、これが collection 固有ではない一般問題であることの傍証として扱う。keyed identity / retained state、data-driven reorder、structured item fields、`f64[]`、大規模 list 性能も条件ベース trigger 付きで後続へ送る。 | [FD-C](#fd-c-初回surfaceの境界un-keyed--appendtruncate--scalar--flat) |
| ⑤ | **`item` / `index` の位置づけ** | `item` / `index` は「全参照を state 経由に揃える」主義への初の明文例外として **loop-local read-only binding** として扱う。ただし例外は式（binding）位置に限る。handler 位置から `item` を読めるか、per-item handler を許すかは Phase 7 ADR の admission 判断に載せる。 | [FD-D](#fd-d-item--index-は-loop-local-read-only-binding) |
| ⑥ | **先行 phase の keyed expectation の扱い** | Phase 6 forward-compat の「Phase 7 = keyed identity / ID-2 reconciler driver」という期待は、Phase 7 framing で明示的に revise する。accepted ADR は遡及編集せず、Phase 7 framing + live 文書 sync で stale expectation を正す。 | [FD-E](#fd-e-先行phaseのkeyed期待はreviseとして扱う) |
| ⑦ | **DD slate の粒度** | Phase 7 ADR は 7 DD を予約する。grammar surface、collection value / mutation / TypedValue、item context / scope、IR / textual IR、runtime identity / range mutation、placement storage、validation / diagnostics / reactive-drain cap を分ける。 | [FD-G](#fd-g-dd-slate-粒度7-dd-予約) |
| ⑧ | **Deferred items の正本** | owner-intent answers §4 の deferred table は、本 framing の scope / out-of-scope 節を正本にする。dsl-grammar.md には思想と trigger だけを置き、責務先は framing / ADR / handoff に置く。 | [FD-F](#fd-f-deferred-items-の正本は本-framing-scope-節に置く) |

**返事チェックリスト（owner 回答 2026-06-11）:**

- ⓪ 比較軸 prior: ☑ OK
- ① architectural-family trigger confirm-or-strain: ☑ OK
- ② Phase 7 thesis: ☑ OK
- ③ visible proof: ☑ OK
- ④ 初回 surface 境界: ☑ OK
- ⑤ `item` / `index` の扱い: ☑ OK
- ⑥ keyed expectation の revise: ☑ OK
- ⑦ DD slate 粒度: ☑ OK
- ⑧ Deferred items 正本: ☑ OK

---

## オーナー合意の記録（Owner alignment outcome）

**2026-06-11 完了。** Owner は alignment packet ⓪〜⑧ の全項目を推奨どおり
承認した（変更指示なし）。これにより FD-P / FD-Q / FD-A / FD-B / FD-C /
FD-D / FD-E / FD-G / FD-F は **owner-agreed framing decisions** として確定し、
DD slate（DD-M3-P7-001〜007 の 7 DD 予約）、scope / out-of-scope（deferred
items 正本テーブルを含む）、verification strategy が ADR drafting の入力と
して凍結された。次段階は Phase 7 ADR set の draft（§Next session — handoff）。

追記: 2026-06-11 ADR strategic review fold により、loop-external collection reads 行を
FD-F の正本テーブルへ追加した。

---

## Phase 7 acceptance criteria (restated)

SSOT は [process/_roadmap.md M3](../../../_roadmap.md) / [plan.md §Acceptance criteria](../../plan.md)。
本 phase が直接負う AC は **A8** であり、A11 / A12 が operational obligation として同時に効く。

- **A8 — Iteration grammar.**
  > collection binding drives widget-tree generation.

  Phase 6 の conditional rendering は binding が subtree の **present / absent** を駆動した。
  Phase 7 は同じ structural control-flow family を **0..N cardinality** へ拡張する。
  重要なのは、固定個数の静的展開ではなく、実行時に変わる collection binding が生成される
  widget subtree の個数を変えること。owner-intent answers はこの読みを採り、A8 の改訂を不要にした。

- **A11 — per-phase synchronization obligation.** `.ui` / `wasamo-ir` / `wasamoc` /
  `wasamo-runtime` / `docs/dsl_spec.md` / `examples/gallery/` の sub-screen が本 phase 内で
  同時に前進する。Iteration は grammar surface なので、parser / lowering / textual IR /
  runtime / validator / example の片側だけを先行させない。

- **A12 — DSL specification first public draft obligation.** Phase 7 は `docs/dsl_spec.md` に
  iteration grammar の normative section を追加する。外部読者が spec だけで、`for` の構文、
  collection 型 / 初期値 / mutation surface、item / index scope、body shape、identity baseline、
  validation / diagnostics、runtime mutation timing を再現できる水準を phase close の bar とする。

- **A1 への incremental proof.** Full gallery は Phase 8 が assembled proof として閉じるが、
  Phase 7 は WrapPanel + ScrollView-backed thumbnail collection を collection から生成する sub-screen を
  ship する。これは A1 のうち A8 に関係する増分 proof である。

---

## Phase 7 thesis — Wasamo structural iteration model

Phase 7 は「List widget」を入れる phase ではない。Wasamo の external `.ui` DSL に
**構造的反復生成**を入れる phase である。Phase 6 の `if` と同じく、iteration は widget として
materialise しない。`IrMember::ControlFlow(ControlFlowNode::For)`（命名は DD で確定）として、
widget member と control-flow member を明示的に dispatch する family に属する。

Phase 7 の設計 thesis は次の 5 点に分解できる。

1. **Cardinality is reactive.** Collection binding の値が変わると、生成される subtree 数が変わる。
   固定長 template expansion ではなく、runtime cardinality 変化が visible proof で観測される必要がある。
2. **Iteration is the sibling of conditional rendering.** `if` は 0/1、`for` は 0..N。
   どちらも property toggling ではなく widget-tree shape を変える structural control flow である。
3. **Initial identity is un-keyed.** 初回 surface は item ごとの retained state を証明しない。
   Fresh / positional-stable の normative wording は DD で精密化するが、keyed retention を silent baseline にしない。
4. **Loop locals are scoped read-only bindings.** `item` / `index` は state 宣言ではないが、
   binding expression から読める loop-local name として導入する。これは「全参照 state 経由」主義の
   限定的な例外であり、handler 位置の可読性は別の admission 判断である。
5. **TypedValue pressure must be explicit.** Scalar item で閉じるなら、その理由と trigger を残す。
   Structured item fields を採るなら、M3 acceptance revision を含む `TypedValue` 採用判断を隠さず開く。

この thesis は Wasamo Vision の「UI structure in the DSL, logic in the host language」および
「external DSL + C ABI」の方向性に沿う。Host language の `for` を借りるのではなく `.ui` 側に
canonical iteration grammar を持つことで、C / Rust / Zig / Swift / Go から同じ declarative surface を共有できる。

---

## Architectural-family trigger confirmation

Phase 7 では、`architectural-family.md` の re-evaluation trigger が発火していると扱う。
具体的には、(1) M3 DSL spec の normative section として iteration grammar を起草するため trigger 1
（M3 DSL spec drafting begins）に触れる。また、(2) `ForLoopSubtree` または同等の range-style
structural target を `BindingTarget` family に追加するため、trigger 3（`BindingTarget` に収まらない、
または収まり方を確認すべき binding feature）を再読対象にする。

再読後の framing 判断は **confirm** である。Phase 7 iteration は、family (1) tree-with-bindings の
内部拡張として扱える。すなわち、tree は `.ui` / IR に declarative に記述され、binding は tree member に
紐づき、tree-shape change は `ControlFlowNode::For` と `BindingTarget::ForLoopSubtree` 相当の
local structural mutation として処理される。この形は Phase 6 の `if` と同じ structural control-flow family の
0..N 拡張であり、view-function re-execution family への pivot を要求しない。

したがって本 framing は、Phase 7 のために vision decision record を新設しない。理由は、public C ABI を
view-function / scope re-execution 型へ変える必要がなく、textual IR を tree description から別 family へ
改訂する必要もなく、`BindingTarget` / `ControlFlowNode` の内部拡張で足りるからである。ただし、これは
family (1) の長期 ratification ではない。`BindingTarget` に自然に収まらない derivation、host-language
view function を canonical surface にする proposal、または DSL spec が re-execution semantics を必要とする
提案が出た場合は、architectural-family.md の trigger に従って再度 strain として扱う。

---

## Settled premises and open edges

### 決定済みとして再審議しない premise

- Iteration は `IrMember` / `ControlFlowNode` family の拡張であり、widget として materialise しない。
- Range-style structural target は `BindingTarget::ConditionalSubtree` が開いた ownership 問題を再利用する。
- `HandlerExpr` enum は統一する。per-item expression のために別 enum を生やさない。
- Operators / expression grammar は condition-only ではなく全 expression position に uniform に育てる。Phase 7 が collection 専用 mutation statement を置くなら、それは expression ではなく handler statement として線引きする。
- `for` / `in` の予約状態、body shape、textual IR 表現は Phase 7 ADR で確定する。
- Widget id と item key は混同しない。

### 本 framing で open DD に送る edge

- Fresh vs positional-stable の normative wording。
- `for` body を single widget child に限るか、range / multiple members を許すか。
- Per-item handler を許すか。許す場合、handler body から `item` / `index` を読めるか。
- Scalar collection の型名・初期値・mutation syntax。
- Structured item fields を Phase 7 で開くか、`TypedValue` と一体で defer するか。
- Parent-owned placement metadata を SoA のまま維持するか、child record へ移すか、keyed metadata map にするか。
- Range mutation の partial failure を log-only に留めるか、rollback / terminal error / stronger runtime error を持つか。
- `MUTATION_CAP` の会計モデル。N item materialisation が drain iteration / effect count / structural mutation count のどれに数えられるか。

---

## 論点 slate（DD questions — 番号予約のみ）

本 phase の ADR set（`decisions/preamble.md` + DD ごとに 1 ファイル）が担う論点を列挙し、
**DD-M3-P7-NNN 番号を予約**する。各 DD の options / 比較 / 推奨は §3 設計判断で書く。
ここでは「何を判断するか」と「なぜ Phase 7 の問いか」だけを固定する。

### DD-M3-P7-001 — Iteration author-facing grammar surface

**問い:** `.ui` で collection-driven iteration をどう書くか。`for item in items { Widget { ... } }` 形を採るか、
structural directive / attribute 形を採るか、`in` を予約語にするか、body を single widget child に限るか、
range of members を許すか。

**Phase 7 の問いである理由:** A8 の直接 surface。Phase 6 の structural `if` と同じ control-flow family に
自然につながる grammar でなければ、Wasamo structural rendering model が `if` 単発で止まる。

**sub-issues:** `for` keyword、`in` reservation、collection expression の位置、body cardinality、nested control-flow の扱い、
per-container direct-`for` admission sweep（WrapPanel / ScrollView / Grid / Box / ZStack のどこで direct `for` を許すか、wrapper を要するか）、
invalid syntax diagnostics、`docs/dsl_spec.md` の normative examples / invalid examples。

### DD-M3-P7-002 — Collection value surface, mutation statements, and `TypedValue` pressure

**問い:** Phase 7 の collection binding を DSL / IR / runtime にどう露出するか。基本線を runtime-owned collection state とするか、host-provided collection binding slot まで開くか、または両者を分けるかを明示する。collection state type、初期値、scalar element type、append / truncate 相当の minimal mutation path、既存 `+=` との関係、collection 専用 handler statement の可否、host state boundary との将来互換性、`TypedValue` を採るか defer するかをどう判断するか。

**Phase 7 の問いである理由:** A8 は collection binding を要求するが、現 `state_type` は `i32` / `string` / `bool` だけで、collection 型も collection mutation も存在しない。さらに plan は Phase 7 を `TypedValue` 圧力判断地点として名指ししている。host-state-boundary.md は、M3 中の `for` / collection binding ではまず runtime-owned collection state で cardinality-driven subtree generation を証明する方向を基本線にし、host set / replace API は Phase 7 必須範囲に含めないとしている。これは単なる未設計回避ではなく、general host state boundary が iteration cardinality とは別 thesis であり、M4 input / host binding / M6 ABI freeze の波で扱う方が設計判断の質が上がるという sequencing 判断である。その境界を DD-002 で明文化する必要がある。

**sub-issues:** collection 型名、literal / initial value、element scalar set、`i32[]` / `bool[]` / `string[]` の homogeneous collection、`f64[]` deferral、runtime-owned collection state と host-provided collection binding slot の境界、host set / replace API を Phase 7 必須にしない場合の future-compat record、将来の host-supplied initial state / host replace を塞がない collection value representation（copy / ownership / element identity）、batching、reactive drain との関係、append/truncate statement、operator uniformity rule との関係、structured fields の defer trigger、M3 acceptance revision が必要になる条件。

### DD-M3-P7-003 — Loop-local context, scope, and handler admission

**問い:** `item` / `index` をどの名前で、どの scope に、どの expression position へ露出するか。state 名との collision、
shadowing、nested loop、handler body からの `item` 可読性、per-item handler を Phase 7 で許すかを決める。

**Phase 7 の問いである理由:** Iteration は template-local name を初めて DSL に入れる。これは widget id ではなく、
loop-local read-only binding である。ここを曖昧にすると external-reader bar と future nested structural control flow の両方に響く。

**sub-issues:** `item` / `index` default name、author-specified name の可否、flat scope、collision = error、nested scope defer、handler admission、
`item` と widget id / item key の境界。

### DD-M3-P7-004 — IR / textual IR representation and structural traversal model

**問い:** `wasamo-ir` と textual IR に `for` をどう表現し、loader / validator / roundtrip / traversal が widget member と
control-flow member をどう dispatch するか。`ControlFlowNode::For`、`BindingTarget::ForLoopSubtree`、declared slot、body template、
member emission canonization の扱いを決める。

**Phase 7 の問いである理由:** Phase 6 が `IrMember::ControlFlow(If)` で開いた family を拡張する最初の test である。
Traversal helper が widget-only filter として control-flow を落とす failure mode は Phase 6 で実際に出ており、semantic migration audit が load-bearing になる。

**sub-issues:** IR node shape、textual IR syntax、body template storage、static load-time presence、roundtrip tests、call-site audit、
`for` body を declared member range と見るか single template child と見るか、member emission を content model として canonize するか。

### DD-M3-P7-005 — Runtime identity baseline and range mutation semantics

**問い:** Collection 変化時に generated subtree をどう insert/remove/rebuild するか。Fresh / positional-stable の normative wording、
registry teardown、effect disposal、Visual order、setter return 前の drain、partial failure の観測可能性を決める。

**Phase 7 の問いである理由:** Conditional は 0/1 subtree mutation だったが、iteration は 0..N mutation である。
Single-child conditional では許容できた log-only diagnostics が、multi-child range では partial state を残しうる。

**sub-issues:** declared slot identity、entity identity、fresh rebuild baseline、append/truncate atomicity、remove order、effect drain timing、
将来の host-origin write / batched replace と矛盾しない drain・identity 契約、rollback / terminal error / log-only の比較、failure reporting、observable contract。

### DD-M3-P7-006 — Placement storage model and structural side-effect atomicity

**問い:** Range mutation が育つ前に、parent-owned placement metadata をどこに保持するかを決める。現行 SoA parallel vector を維持するか、
child record AoS へ移すか、keyed metadata map を採るか。新しい parent-owned per-child metadata と range insertion の atomicity をどう保つか。

**Phase 7 の問いである理由:** Phase 6 ZStack conditional path は child list / placement metadata / live Visual sibling order を同時更新する必要があった。
Iteration は 1 mutation で複数 child を増減させるため、parallel data drift がより発生しやすい。

**sub-issues:** ZStack placements、Grid `Cell` / parent-owned metadata との境界、ScrollView / WrapPanel での no-placement path、container sweep、
atomic primitive、layout invalidation、Visual sibling order、implementation-gates trap #2 / #3 close artifact。

### DD-M3-P7-007 — Validation, diagnostics, cap accounting, and reactive-drain residual disposition

**問い:** `wasamoc check` / runtime validation / textual IR loader がどの invalid shape を拒否するか。
また、reactive-drain residual（cycle detection、ordering ties、fan-out × `MUTATION_CAP`、synchronous non-batched drain proof contract）を
Phase 7 で fix するか carry するかを明示する。特に N item materialisation が `MUTATION_CAP` にどう数えられるかを確定する。

**Phase 7 の問いである理由:** Iteration は collection cardinality 変化が複数 dependent effect / generated subtree に fan-out する初の grammar。
Silent carry-forward は plan-level risk と constraints §6 で禁止されている。

**sub-issues:** non-collection target rejection、non-scalar item rejection、name collision diagnostics、nested unsupported diagnostics、
per-container direct-`for` validation、empty collection（0 generated child）の扱い、Phase 6 DD-007 の conditionally-empty ScrollView reject との干渉、
invalid mutation statement、cap accounting model、small-N proof が cap に触れない evidence、fix-or-carry record、branch tests。

---

## Phase 7 scope

### In scope

Phase 7 で扱う範囲は次のとおり。ここでは、具体的な options / recommendation はまだ決めないが、ADR が判断すべき面積は固定する。

- Structural `for` grammar を member-level control-flow form として導入する。
- Per-container direct-`for` admission（どの container 直下で `for` を許すか、どこで wrapper を要するか）を ADR で決める。
- Cardinality 変化を証明するのに十分な runtime-mutable collection binding を導入する。基本線は runtime-owned collection state とし、host set / replace API は Phase 7 必須範囲にしない。
- Gallery proof のために、append / truncate または同等の最小 author-visible mutation path を持つ。
- Binding expression 位置で、loop-local read-only `item` / `index` を読めるようにする。
- Scalar item baseline を基本線にしつつ、`TypedValue` 圧力は DD で明示判断する。`i32[]` / `bool[]` / `string[]` を中心に扱い、`f64[]` は Phase 7 必須範囲から defer する。
- 初回 scope は flat とし、state 名との collision は reject する。
- Identity baseline は un-keyed とする。ただし fresh と positional の normative wording は ADR で決める。
- Ordering model は append/truncate-only とし、初回 proof では data-driven reorder を扱わない。
- Runtime range mutation path について、disposal / drain / Visual order semantics を定義する。
- Range mutation 実装に入る前に、placement storage model を決める。
- `docs/dsl_spec.md` に iteration grammar の normative section を追加し、`examples/gallery/` の sub-screen で proof する。

### Out of scope（activation trigger 付きで carry するもの）

次の表を deferred items の**正本**とする。ADR の forward-compat と implementation handoff は、この表をコピーまたは精密化して使う。別の表を作り直して責務先や trigger を分散させない。

| Deferred item | 責務を置く先 | activation trigger | 理由 |
|---|---|---|---|
| keyed identity / retained state | **M4 input / focus / TextField pre-doc で必ず再評価**する。LazyList / 大規模 list は **M5 or later** に送る | repeated subtree 内に focus / input / selected / user-editable state が入る。あるいは reorder を許す | Retention が本当に問題になるのは、「同じ item の入力中状態・focus・selection を保持したい」時である。M4 が input を開くため、そこが自然な再点火点になる。大規模 list 性能は LazyList / performance thesis に寄せる |
| data-driven reorder | 原則として **M5 / collection UX DD** に送る。Phase 7 で reorder を入れる判断をするなら、その時点で即時に開く | sort / filter / drag reorder / user-authored order mutation / keyed diff を要求する UI | Reorder は input stack ではなく、collection UX / reconciler / ordering contract の問題である。Phase 7 の cardinality proof とは別 thesis に属する |
| structured item fields / `TypedValue` | **Phase 7 ADR で明示判断**する。不要と判断する場合は、trigger 付きで **M4 showcase spec または M5 widget/data-surface DD** へ carry する | `item.filename`、caption fields、image metadata、record-like state、scalar で足りない concrete app case | Plan は Phase 7 を `TypedValue` 圧力の最有力点と名指ししている。「不要」とする場合でも、再評価 trigger を残す |
| host state boundary（host-supplied initial state / host replace / in-out write-back） | **M4 input / TextField / focus model、dynamic Window title / host bindings、または M6 ABI freeze 前**に再評価する。M3 中に collection state を host から初期化・差し替えたくなった場合は即時に開く | host が `.ui` state に initial value を渡す、表示中に state を set / replace する、TextField / ScrollView offset / selection など runtime-origin value を host 側へ write-back する | これは collection 専用ではなく general host state boundary の問題であり、iteration cardinality proof とは別 thesis に属する。単値 state にすら host write channel が無いことは、その一般性を示す傍証である。M4 input / host bindings / M6 ABI freeze と同じ波で開く方が設計判断の質が上がる。ただし collection state の型、copy / ownership、element identity、batching、reactive drain との関係は将来 host replace と矛盾しないよう ADR に記録する |
| loop-external collection reads（length / empty check / element index read） | **DD-002 forward-compat と DD-007 diagnostics**に deferral / reject を置き、正本として本表に追加する（FD-F 正本テーブル追記） | gallery caption の `N items`、Remove disable の empty check、body 外の element access、host/state expression で collection を読む concrete app case | Phase 7 の collection read は `for` header と loop-local binder 経由に限る。外側の read surface は Q5 の uniform expression/reference extension と一緒に開く方が、参照形と operator pocket を分裂させない |
| `f64[]` / 第四 scalar collection element | **DD-002 の element-scalar-set 判断で defer を明示**し、必要なら **M5 value-surface DD または `TypedValue` / scalar expansion DD** へ送る | 座標、比率、metrics、opacity、animation value、image metadata など、`f64` 要素を要する concrete app case が出た時 | `f64[]` は structured fields ではないが、第四 scalar / value-surface 拡張である。Phase 7 の cardinality proof は `i32[]` / `bool[]` / `string[]` の homogeneous scalar collection で足りるため、`f64[]` は肯定的に残しつつ trigger 付きで defer する |
| per-item handler / handler 内 `item` 参照 | **Phase 7 ADR で admission を明示判断**する。Reject するなら trigger 付き defer として記録する | select-this-item / delete-this-item 等の per-item interaction が要る UI。自然には M4 input で到来する | Q5a の例外は式（binding）位置だけである。`for` body 内の handler 持ち widget を許すか、handler から `item` を読めるかは別判断である |
| nested template scope / shadowing | **次に nested structural control flow を開く phase** に送る（暫定 M4+ grammar residual） | nested `for`、`else` / `switch`、bare nested control flow、template-local named scope が必要になった時 | Scope 規則は、structural control-flow family の拡張と一緒に設計する方が一貫する |
| gallery-scale / cap / fan-out | **Phase 7 framing §Risks + reactive-drain residual carry** で扱う。大規模化は **M5+ LazyList / performance DD** に送る | N item 生成が `MUTATION_CAP` に触れる。Visible list が数十〜数百 item を acceptance として要求する。CI / E2E で convergence failure が出る | Constraints §6 は fan-out × cap を Phase 7 直撃の論点とし、fix-or-carry の明示記録を義務化している。Silent carry-forward は不可 |
| item key と widget id の境界 | **Phase 7 ADR で確定的に記録**する。Widget id 自体は top-layer / anchor / concrete app case まで carry する | `key:` 導入、top-layer anchor 参照、state 経由では表現力が足りない concrete case | Widget id と item key を混同しない規律をここで明文化する |

### Acceptance mapping

| Acceptance / obligation | Phase 7 での discharge |
|---|---|
| A8 iteration grammar | DD-001〜DD-005 で扱う。`for` grammar + collection binding + runtime range mutation を通す |
| A11 per-phase sync | `.ui` parser / IR / wasamoc / runtime / docs / gallery sub-screen を同じ phase 内で更新する |
| A12 DSL public draft | `docs/dsl_spec.md` の iteration section を Moment 1 / Moment 2 で同期する |
| A1 incremental gallery proof | WrapPanel + ScrollView-backed thumbnail set を collection から生成し、add/remove proof を出す |
| Plan risk: `TypedValue` | DD-002 で explicit judgment を行う |
| Plan risk: reactive-drain residual | DD-007 で fix-or-carry judgment を行う |

---

## Verification strategy

Phase 7 の検証は、「固定 tree と区別できるか」を中心に組む。Hardcoded された N 個の thumbnails を表示するだけでは、iteration grammar の proof にならない。

### Visible E2E proof

- `examples/gallery/` に collection-backed thumbnail sub-screen を置く。
- 初期状態 N、append 後 N+1、truncate 後 N または N-1 のように、collection mutation に応じて item 数が変わることを 2 frame 以上で示す。
- Mutation trigger は body 外の text Button でよい。Per-item handler は proof には不要である。
- Screenshot evidence は launch / process survival だけでは足りない。Screenshot、assistant による analysis、陽性対照を含める。Owner human-visible smoke は別枠で行う。
- DPI blur 等の既知 M4 runtime-quality residual は、この phase の failure とはしない。ただし evidence analysis には既知事項として注記する。

### Pure-logic tests

- Parser は選択された `for` syntax を accept し、invalid forms を reject する。
- Lowering は `ControlFlowNode::For` / collection binding target shape を生成する。
- Textual IR roundtrip は `for` を保持し、control-flow members を落とさない。
- Validator は、non-collection target、container ごとの direct-`for` admission、empty collection が生む 0-child 形状、Phase 6 ScrollView conditional-content reject との干渉、name collision、unsupported nested scope、invalid mutation statement、scalar-only 選択時の non-scalar item を reject する。
- Runtime range reducer / pure mutation planner は、WinRT なしで可能な範囲を test する。Append / truncate が declared insertion/removal range を計算し、order を保ち、disposal / insertion effects を schedule することを確認する。

### Windows-headless / runtime integration tests

- Collection mutation 後、live `WidgetNode` / Visual order が collection cardinality を反映することを確認する。
- Setter / handler return 時点で、選択された contract に従い、fresh generated subtree の effects が drain 済みとして観測できることを確認する。
- Parent-owned placement を持つ container に range mutation が触れる場合、placement metadata と live Visual sibling order が atomically に更新されることを確認する。少なくとも ZStack に触れる経路ではこれを検証する。
- 追加した diagnostic / reject branch については、それぞれを直接 fire する failure-path test を置く。

### Implementation gate expectations

Phase 7 の多くの implementation task では、次の traps が適用される見込みである。

- **#1 semantic migration**: IR / control-flow enum に `For` を追加する場合、traversal call-site audit が必要になる。
- **#2 missed side effects**: Tree insertion/removal では、layout dirty、Visual order、registry、effects、parent metadata を列挙する必要がある。
- **#3 parallel data drift**: Placement metadata は child list mutation と同じ primitive の中で atomically に更新する必要がある。
- **#4 untested authored branch**: 新しい reject / diagnostic branch は、それを直接 fire する test を必要とする。
- **#5 carry-forward**: keyed identity、TypedValue、`f64[]`、host state boundary、cap、reorder、handler-item reads を defer する場合、trigger-backed carry record が必要になる。特に host state boundary は、今設計しない一方で将来 host replace を塞がない representation / drain / identity 制約を ADR に残す必要がある。
- **#7 GUI positive control**: Gallery proof は、mutation-backed な 2+ frame screenshot evidence を必要とする。

---

## Risks

### R1. `TypedValue` による scope inflation

Phase 7 は、collection element type が自然に `TypedValue` を圧迫する最初の地点である。ここで structured item fields を admit すると、phase の中心が iteration grammar から value-system 設計へ移る可能性がある。

これは product merit 上必要なら許容できる。ただし、その場合は acceptance-revision path を明示する必要がある。そうでなければ、scalar item baseline に閉じ、trigger-backed defer として後続へ送る方が thesis sequence として安全である。

### R2. Keyed identity expectation の silent drift

Phase 6 の forward-compat text は、Phase 7 を keyed identity / reconciler driver と予測していた。Owner answers はこの期待を revise している。リスクは revise すること自体ではなく、un-keyed baseline を実装しながら、retained identity を示唆する stale docs を残すことである。

Mitigation は、revise を本 framing に記録し、accepted Phase 6 ADR を遡及編集せず、Phase 7 Moment 2 で live documents を更新することである。

### R3. `MUTATION_CAP` accounting の曖昧さ

N 個の generated items を append / truncate する操作は、1 mutation と数えるのか、N structural edits と数えるのか、N effect drains と数えるのか、あるいはそれらの組み合わせなのかを明確にする必要がある。

この accounting が曖昧なままだと、小さい N で偶然通る proof を出荷してしまうおそれがある。DD-007 は accounting model を定義し、選択した gallery N が cap に触れないことを示す。もし cap に触れるなら carry-forward は選べず、fix が必要になる。

### R4. Placement metadata drift

Range insertion は、materialised child list、placement metadata、live Visual order の invariant を壊しやすい。Phase 6 でも ZStack conditional path でこの failure mode が見えており、Phase 7 では一度の mutation で複数 child が増減するため、危険度が上がる。

Mitigation は、実装前に DD-006 で storage model を決め、implementation-gates #2 / #3 の close artifacts を必須にすることである。

### R5. Spec drafting drift

Iteration grammar は A12 の normative content である。Code だけが先に land し、external-reader-grade の spec text が遅れると、M3 は「今は実装、public draft は後で」という形に退行する。

Mitigation は、ADR Accepted 時点の Moment 1 design-spec draft と、phase close 時点の Moment 2 implementation re-sync を必ず行うことである。Invalid examples も spec に含める。

### R6. Schedule risk の置き場

Owner prior により、implementation / revision cost は独立の DD comparison axis ではない。ただし、候補案が Phase 8 / public draft closure を脅かす場合、それは framing risk として扱う必要がある。

したがって DD は options を product / thesis merit に基づいて reject し、schedule impact はこの §Risks で phase 横断のリスクとして管理する。同時に、将来拡張性を理由として過剰設計を正当化してはならない。Product merit は「大きい設計を常に選ぶ」ための口実ではなく、Phase 7 thesis に対して比例した設計を選ぶための評価軸である。

### R7. Architectural-family confirmation の過不足

Phase 7 は M3 DSL spec と range-style structural binding target の両方に触れるため、architectural-family trigger を無視すると、tree-with-bindings family を暗黙に ratify したように見えるリスクがある。

Mitigation は、本 framing で trigger 発火を明示し、今回の判断を「family (1) 内に収まるため VDR 不要」と confirm して記録し、さらに Phase 7 Moment 2 で `architectural-family.md`（family 仮説と trigger の SSOT）へ revise-in-place で反映すること（FD-Q 参照）である。ただしこの confirm は長期 ratification ではない。将来 `BindingTarget` に収まらない binding feature や view-function re-execution semantics を要求する surface が出た場合は、あらためて strain として扱う。

---

## Owner-agreed framing decisions

### FD-P. Phase 7 以降の比較軸（product merit 主軸）

Phase 7 以降の DD 比較では、product merit / thesis 整合を主軸にする。実装・改訂コストは独立の評価軸ではなく、tie-breaker としてだけ扱う。

ただし「コスト非重視」は maximalism ではない。将来拡張性を理由とした過剰設計は、引き続き design-decision review の失敗モードとして扱う。Product merit は、Phase 7 thesis に比例した設計を選ぶための軸であり、大きい案を自動的に正当化する軸ではない。

M3 の他 AC、特に Phase 8 / public draft を脅かす場合は、DD comparison table ではなく framing / plan の schedule risk として扱う。

**Status:** owner-aligned (2026-06-11).

### FD-Q. Architectural-family トリガーの confirm-or-strain

Phase 7 では architectural-family.md の trigger 1（M3 DSL spec drafting begins）と trigger 3（`BindingTarget` に収まらない、または収まり方を確認すべき binding feature）が発火していると扱う。

本 framing の判断は **confirm** である。Iteration は tree-with-bindings family 内の structural control-flow 拡張として扱い、`ControlFlowNode::For` と `BindingTarget::ForLoopSubtree` 相当の内部拡張で吸収する。Public C ABI、textual IR の tree-description contract、`.ui` を canonical DSL とする方針を view-function re-execution family へ pivot させる必要はない。

したがって Phase 7 のための新規 VDR は不要である。ただしこれは family (1) の長期 ratification ではない。将来 `BindingTarget` に自然に収まらない binding feature、host-language view function を canonical surface にする proposal、または re-execution semantics を必要とする DSL surface が出た場合は、再度 strain として扱う。

この confirm は framing にのみ留めず、SSOT へ反映する。Phase 7 Moment 2 で `architectural-family.md` の alignment table / re-evaluation triggers に「Phase 7（M3 DSL spec drafting）で trigger 1 / 3 を再読し、family (1) 内に収まると confirm、新規 VDR 不要」を revise-in-place で追記する（同 note 自身の「revising in place の場合は alignment table と re-evaluation triggers を更新」指示に従う）。これにより、次に同 note を読む者が trigger 1 を未処理と誤認しない。

**Status:** owner-aligned (2026-06-11).

### FD-A. Phase 7 thesis（collection binding が cardinality を駆動する）

Phase 7 は static template expansion ではなく、runtime-mutable collection binding が generated widget subtree の個数を駆動することを証明する。

これは A8 の “collection binding drives widget-tree generation” と整合するため、acceptance revision は不要である。

**Status:** owner-aligned (2026-06-11).

### FD-B. Visible proof（固定 N ではなく増減 2 frame で証明する）

Fixed N thumbnails の単発 screenshot は、hardcoded tree と区別できない。Phase 7 proof では collection を変化させ、item 数が連動して増減する 2 frame 以上を示す必要がある。

Mutation は body 外の Button でよく、per-item handler は proof には不要である。基本線は runtime-owned collection state を `.ui` / runtime 内で変化させる proof であり、host から collection 全体を set / replace する public API は Phase 7 の必須 proof path にしない。

**Status:** owner-aligned (2026-06-11).

### FD-C. 初回 surface の境界（un-keyed / append-truncate / scalar / flat）

Initial surface は runtime-owned collection state、un-keyed base、append/truncate-only、scalar item、flat scope を基本線にする。

Scalar item は Phase 7 では `i32` / `bool` / `string` の homogeneous collection を中心に扱う。`f64[]` は第四 scalar / value surface の拡張になるため、Phase 7 必須範囲からは defer する。

Host-supplied initial state、host replace / host write、in-out write-back は v1 までに欲しい能力として肯定的に残す。ただしこれは collection 専用の要求ではなく、general host state boundary という別 thesis に属する。単値 state の host write API もまだ無いことは、Phase 7 で切る理由ではなく、この論点が collection 固有ではないことの傍証である。したがって Phase 7 では必須 surface にせず、M4 input / host bindings / M6 ABI freeze の波で再評価する。Phase 7 の collection 設計は、将来の host state boundary を塞がないよう、型、copy / ownership、element identity、batching、reactive drain との関係を ADR に記録する。

Keyed retention、reorder、structured fields、`f64[]`、host state boundary、large-N performance は肯定的に残す。ただし、Phase 7 では silently 先送りせず、activation trigger 付きで後続へ送る。

**Status:** owner-aligned (2026-06-11).

### FD-D. `item` / `index` は loop-local read-only binding

`item` / `index` は state 宣言ではないが、binding expression から読める loop-local read-only binding として扱う。

これは state 経由主義への限定的例外である。Handler 位置から読めるかどうかは、別の admission 判断とする。

**Status:** owner-aligned (2026-06-11).

### FD-E. 先行 phase の keyed 期待は revise として扱う

Phase 6 の forward-compat expectation は、Phase 7 framing で revise として扱う。

Accepted ADR は遡及編集しない。Phase 7 framing / ADR / live-document sync によって stale expectation を正す。

**Status:** owner-aligned (2026-06-11).

### FD-G. DD slate 粒度（7 DD 予約）

Phase 7 ADR は 7 DD を予約する。DD-001 は author-facing grammar、DD-002 は collection value / mutation / `TypedValue` pressure、DD-003 は loop-local context / scope / handler admission、DD-004 は IR / textual IR / traversal、DD-005 は runtime identity / range mutation、DD-006 は placement storage / structural side-effect atomicity、DD-007 は validation / diagnostics / cap accounting / reactive-drain residual disposition を担当する。

この粒度は、owner alignment packet の判断項目として扱う。ADR drafting 中に統合・分割が必要になった場合は、owner に「どの FD を更新するか」が分かる形で framing revision または ADR preamble に理由を記録する。

**Status:** owner-aligned (2026-06-11).

### FD-F. Deferred items の正本は本 framing scope 節に置く

owner-intent answers §4 の deferred items は、本 framing の scope / out-of-scope 節を正本にする。

`dsl-grammar.md` には思想と condition-based trigger だけを残し、責務先は framing / ADR / handoff へ流す。

**Status:** owner-aligned (2026-06-11).

---

## Inputs absorbed

### From [constraints.md](./constraints.md)

次の内容を Phase 7 framing の入力として吸収した。

- Control-flow family extension は `IrMember` / `ControlFlowNode` から始める。
- `BindingTarget::ConditionalSubtree` は range-style structural target の landing point になる。
- Declared-tree / entity-tree identity は fresh-on-return を超える部分が open のまま残っている。
- Placement storage model は、range mutation が育つ前に Phase 7 で決める必要がある。
- Structural failure observability は range mutation に合わせて見直す必要がある。
- Reactive-drain residuals については fix-or-carry judgment が必要であり、silent carry-forward は禁止される。
- `TypedValue` 圧力は明示的に判断する必要がある。
- Semantic migration audit gate は既に codify 済みであり、Phase 7 にも適用される。
- Visible E2E proof には screenshot + analysis + positive control が必要である。
- Final-step ownership split と mutable phase plan の運用を継承する。

### From [owner-intent-questions.md](./owner-intent-questions.md)

次の内容を Phase 7 framing の入力として吸収した。

- Existing premises と prior-phase expectations を分離する。
- Keyed identity expectation は decision ではなく、confirm-or-revise が必要な期待として扱う。
- Static iteration は、A8 / positive-control thesis と緊張する。採るなら acceptance または verification の改訂が必要になる。
- Q1–Q7 が提示した owner-intent branches を、FD-A〜FD-E と DD-001〜DD-007 に蒸留する。

### From [owner-intent-answers.md](./owner-intent-answers.md)

次の owner answer を Phase 7 framing の入力として吸収した。

- Product merit を primary comparison axis とする。
- Phase 7 thesis は runtime-mutable collection cardinality である。
- Initial surface は un-keyed base、append/truncate-only、scalar item、flat scope とする。
- `item` / `index` は expression position の loop-local read-only binding として扱う。
- Deferred items には activation trigger と stable responsibility landing が必要である。
- Accepted ADR を遡及編集せず、live document sync によって stale keyed-identity expectation を revise する。

### From [host-state-boundary.md](../../../../docs/notes/host-state-boundary.md)

次の内容を Phase 7 framing の入力として吸収した。

- M3 時点の `.ui` `state` は runtime-owned `SignalRegistry` に load 時に作られ、host-supplied initial state bag や host から state signal を直接更新する public API は存在しない。
- `wasamo_set_property` / `Widget::set_property` は widget property を直接書く API であり、component state の host read/write channel ではない。
- `IrComponent.host_props` / `host_bindings` は Window title / backdrop / theme など host-owned attributes と content root の分離であり、component state の host read/write channel ではない。
- v1 までに host-supplied initial state、host replace / host write、in-out write-back を扱いたい owner intent はあるが、これは collection 専用ではなく general host state boundary の問題である。
- M3 中の `for` / collection binding では、まず runtime-owned collection state で cardinality-driven subtree generation を証明する方向を基本線にする。
- Host から collection 全体を set / replace する API は、iteration cardinality proof とは別の general host state boundary thesis に属するため、M3 の必須範囲には含めない。単値 state の host write API もまだ無いことは、この論点が collection 固有ではないことの傍証として扱う。
- ただし、collection state の型、copy / ownership、element identity、batching、reactive drain との関係は、将来の host replace と矛盾しない形で記録する。
- 再訪 trigger は、M4 input / TextField / focus model、ScrollView wheel / drag / write-back offset、M3 collection state の host 初期化・差し替え要求、dynamic Window title / host bindings、M6 ABI freeze 前である。

### From [dsl-grammar.md](../../../../docs/notes/dsl-grammar.md)

次の内容を Phase 7 framing の入力として吸収した。

- Q1 は、widget id と item key を混同してはならないことを記録している。
- Q5 は expression grammar uniformity を記録している。Operators は condition-only pockets ではなく、すべての expression position に広げるべきである。
- Q6 は conditional rendering から来る structural control-flow family framing を与える。
- Q8 は Phase 7 iteration grammar thesis と condition-based re-evaluation triggers を与える。

### From [plan.md](../../plan.md)

次の内容を Phase 7 framing の入力として吸収した。

- A8 は Phase 7 が直接 owner となる acceptance である。
- A11 / A12 により、implementation / spec / example sync が phase ごとに必要である。
- Phase 7 は `TypedValue` 圧力と reactive-drain residual が最も出やすい地点である。
- Verification では、可能な限り pure-logic tests を優先し、runtime / Visual behavior が Compositor-bound types と絡む場合に Windows-headless tests を使う。

### From [spec.md](../../requirements/spec.md)

次の内容を Phase 7 framing の入力として吸収した。

- M3 target app は Photo Gallery である。
- Gallery item list を生成するために iteration grammar が必要である。
- Grammar surfaces は M3 の public-spec content であり、M4 向けの syntax reservation では済ませない。
- Image widget は deferred のままである。Thumbnails は Box + Text placeholder で表すため、ADR が妥当性を説明するなら scalar item proof は許容できる。

### From [architectural-family.md](../../../../docs/notes/architectural-family.md)

次の内容を Phase 7 framing の入力として吸収した。

- M3 DSL spec drafting は architectural-family re-evaluation trigger である。
- `BindingTarget` に収まらない、または収まり方を確認すべき binding feature は re-evaluation trigger である。
- Phase 7 iteration はこれらの trigger を発火させるが、今回の判断は tree-with-bindings family 内に収まるという confirm であり、新規 VDR は不要である。
- この confirm は長期 ratification ではなく、将来の strain trigger は残る。

### From [VISION.md](../../../../VISION.md)

次の内容を Phase 7 framing の入力として吸収した。

- `.ui` は host languages をまたぐ canonical declarative form であり続ける。
- External DSL は、特定 host language の `for` construct に依存しない。
- M3 の目的は DSL surface expressiveness と public draft quality であり、feature maximalism ではない。

---

## Next session — handoff

Owner がこの framing に alignment したら、次の段階では Phase 7 ADR set を draft する。

- `process/milestone-3/phase-7/decisions/preamble.md`
- `dd-m3-p7-001-iteration-author-facing-grammar.md`
- `dd-m3-p7-002-collection-value-surface-and-typedvalue-pressure.md`
- `dd-m3-p7-003-loop-local-context-scope-and-handler-admission.md`
- `dd-m3-p7-004-ir-textual-ir-and-structural-traversal.md`
- `dd-m3-p7-005-runtime-identity-and-range-mutation-semantics.md`
- `dd-m3-p7-006-placement-storage-and-structural-side-effects.md`
- `dd-m3-p7-007-validation-diagnostics-cap-and-reactive-drain.md`

ADR draft は `Status: Proposed` から始める。その後、owner review を経て `Status: Accepted` に進め、続けて Moment 1 design-spec sync を行う。
