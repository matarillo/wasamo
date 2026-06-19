---
title: M3-Phase 7b framing — parent-interpreted placement attributes
status: draft
created: 2026-06-19
target-phase: M3-Phase 7b
-------------------------

# M3-Phase 7b framing

**Status:** draft
**Targets phase:** M3-Phase 7b (parent-interpreted placement attributes)

プロジェクトの開発プロセス（[workflow.md §2](../../../procedures/workflow.md)）に従い、
本 note は設計判断（ADR）を書く前に **owner とフレーミングを合意**するための入力資料。
個別 DD の Options / 比較 / Recommendation は本 note では確定しない。ここで確定するのは、
Phase 7b が何を証明する phase なのか、どの論点を DD として予約するのか、どこまでを
Phase 7b scope とし、どこからを trigger 付きで後続へ送るのか、そして検証方針である。

先行 M3 phase から本 framing が継承する規律（再導出しない）:

* **Two-moment spec-sync**: Moment 1 = ADR-Accepted commit での design-spec draft、
  Moment 2 = phase close での implementation re-sync。
* **Moment is not a commit unit / review-concern 単位 commit**。
* **No fast-track**: 全 merge は owner 明示承認を要する。
* **Final-task retrospective split**: 最終 task の task-end retro と phase-end retro / CI run-id
  所有を最初から分ける。
* **Implementation gates**: semantic migration / side effects / parallel data drift / GUI positive
  control は、実装開始時に選択し、close-gate artifact で閉じる。
* **Formal constraints handoff はなし**: 本 phase は当初計画にない owner-inserted phase であるため、
  phase-to-phase constraints 文書は存在しない。ただし、Phase 6 / Phase 7 の設計文書から
  parent-interpreted placement に関する入力は本 framing の “Inputs absorbed” に明示的に吸収する。

---

## 今回オーナーに決めてほしいこと（Owner alignment packet）

**この節だけ読めば合意判断できる。** 推奨でよければ「OK」、変えたい項目だけ指示してください。
詳細な根拠は後続の各節に置く。

| ID | 決めてほしいこと                         | 推奨                                                                                                                                            | 詳細                                                                                                                                                                                 |
| -- | -------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ①  | Phase 7b を挿入する目的                 | **Grid / ZStack の parent-interpreted placement surface と内部 child-slot model を、M3 public draft 前に整理する corrective phase** とする                   | 新 layout primitive を足す phase ではない。既存 placement surface の意味論・author surface・内部 storage・将来 code-construction boundary を揃える phase。                                                    |
| ②  | Phase 7b thesis                  | **親が解釈する child placement は widget property ではなく child-slot metadata である**、というモデルを DSL / IR / runtime / future API 方針に通す                       | `Text.text` や `Button.enabled` とは違い、`h-align`, `row`, `column` などは直近親 container が子をどう配置するかの情報である。                                                                                  |
| ③  | DD slate                         | **2 DD 構成**にする                                                                                                                                | DD-001 = author-facing DSL surface。DD-002 = child-slot internal model and future construction boundary。migration / compatibility / diagnostics は各 DD の sub-issue として扱い、独立 DD にしない。 |
| ④  | DSL surface の設計空間                | DD-001 で **edge wrapper / prefix なし / fixed prefix / XAML-style attached property / parent-declared namespace** を比較対象に入れる                     | Recommendation は ADR で決める。ただし framing では、最低限この設計空間を確保する。                                                                                                                           |
| ⑤  | 内部設計の設計空間                        | DD-002 で **widget property model / parent parallel metadata / encapsulated SoA / child-slot-carried metadata / keyed metadata map** を比較対象に入れる | 既存 DD-M3-P7-006 の ST2 を入力として扱う。ただし author surface を変える可能性があるため、Phase 7b で再度 contract を明文化する。                                                                                       |
| ⑥  | Architectural family             | **vision decision record は新設しない**。tree-with-bindings 仮説内の整理として扱う                                                                              | SwiftUI / Compose 的な view-function scope modifier へ pivot する phase ではない。外部 DSL の tree member / child-slot metadata として整理する。                                                        |
| ⑦  | Future code-construction API/ABI | **Phase 7b では API/ABI を追加しないが、将来 API の非コミット制約を記録する**                                                                                          | 例: placement は child property setter ではなく、parent-scoped insertion / child-slot builder で表す方向を推奨制約として残す。具体 API signature は後続 phase へ送る。                                             |
| ⑧  | 実装範囲                             | ADR が Accepted された場合、Phase 7b は docs だけでなく、選ばれた surface に必要な parser / checker / lowering / runtime / examples の同期まで行う                         | ただし新 layout algorithm は作らない。既存 Grid / ZStack の placement 表現と検証を整理する。                                                                                                               |

**返事チェックリスト:**

* ① Phase 7b 目的: ☐
* ② Phase 7b thesis: ☐
* ③ DD slate 2 本構成: ☐
* ④ DSL surface 設計空間: ☐
* ⑤ 内部設計 設計空間: ☐
* ⑥ Architectural family 扱い: ☐
* ⑦ Future code-construction API/ABI は非コミット制約のみ: ☐
* ⑧ 実装範囲: ☐

---

## オーナー合意の記録

TBD.

---

## Phase 7b acceptance criteria (restated)

SSOT は [process/_roadmap.md M3](../../../_roadmap.md) / [plan.md §Acceptance criteria](../../plan.md)。
本 phase は当初計画にない owner-inserted phase であるため、計画時点の独立 AC を持たない。

ただし **「Phase 7b は新 AC を要しない」とは断定しない。** 新 AC の要否は
DD-M3-P7b-001（author surface）の結論に **contingent** であり、その DD より前に
ここで決め打ちしてはならない。理由は、既存 AC が placement の author surface を
一度も名指していないからである。A2 は Grid を "1 cell 1 child, star sizing + spanning"、
A4 は ZStack を "sibling z-order by document order" としか約束しておらず、
*親解釈 placement をどう書くか*、ましてや *コンテナ横断で一貫した書き方* は
どの既存 AC の thesis も担っていない。Phase 7b の届ける「cross-container で coherent な
parent-interpreted placement モデル」は、A2 と A4 の隙間に落ちている。

したがって AC 上の扱いは、DD-M3-P7b-001 の出口で次のいずれかに確定する。
**この判断と plan 改訂の経路は ADR Accepted 時点で固定し、plan の Revision log に記録する。**

* **(a) DD-001 が public author-facing placement surface を変える場合**
  （例: `slot.*` prefix、`Layer` 導入、`Cell` → `slot.row` 移行）。
  A2/A4 が一度も名指していない新しい author-facing 契約が生まれ、外部読者がそれに依存する。
  この場合は M3 acceptance-criteria revision 例外のもとで、新 AC を追加するか
  A2 / A4 / A12 の wording を refine する。phase 挿入は「新 AC を伴う phase」として
  AC-revision 例外ルート（Revision log + Phase breakdown 更新 + ROADMAP mirror）で処理する。
* **(b) DD-001 が surface を保ったまま非対称を原則として明文化するに留まる場合**。
  新しい author surface は生まれず、純粋に spec / doc hygiene なので、Phase 7b は
  既存の **A11 / A12** のもとで discharge する。phase 挿入は新 AC を伴わないため、
  vision-ADR（phase-insertion）ルートで処理する。

どちらに転んでも効く既存 obligation:

* **A11**: `.ui`, `wasamo-ir`, `wasamoc`, `wasamo-runtime`, `docs/dsl_spec.md`, `examples/gallery/`
  が同期して進むこと。Phase 7b が placement surface を変える場合、この同期対象に含まれる。
* **A12**: DSL public draft が外部読者に説明可能であること。Grid と ZStack の placement surface が
  意味論上は近いのに author surface / storage model / future API story が分裂していると、
  public draft で説明負債になる。Phase 7b はその負債を M3 内で整理する。

---

## Phase 7b thesis — parent-interpreted placement attributes

Phase 7b の thesis は:

> 親 container が解釈する child placement 情報を、widget 自身の通常 property ではなく、
> **parent-interpreted child-slot metadata** として明文化し、Grid / ZStack の author-facing DSL surface、
> IR / runtime storage、将来の code-construction API 方針を同じ概念に揃える。

Phase 7b の設計 thesis は次の 4 点に分解できる。

1. **Placement is not an intrinsic widget property.**
   `Text.text` や `Button.enabled` は widget 自身の属性である。一方、Grid の `row` / `column`、
   ZStack の `h-align` / `v-align` は、その child が直近の親 container の中でどう扱われるかを
   指定する情報である。したがって、意味論上は child widget property ではなく child-slot metadata である。

2. **Author surface must not obscure the model.**
   Grid は `Cell` wrapper、ZStack は direct child 上の `h-align` / `v-align` という surface を持っている。
   現状はどちらも parent-interpreted placement だが、見た目が違う。Phase 7b はこのブレを、
   edge wrapper / fixed prefix / prefix なし sugar などの選択肢として明示比較する。

3. **Runtime storage must remove drift by construction.**
   Parent-owned placement metadata を `children` と並列 vector で持つと、insert / remove / range splice /
   future reorder で child list と placement metadata がずれる危険が残る。Phase 7b は、placement を
   child-slot record に載せる内部 contract を再確認または改訂し、DSL surface と接続する。

4. **Future code-construction API must remain natural.**
   Wasamo は現時点でコードから UI tree を組む public API/ABI を持たないが、将来導入する可能性がある。
   そのとき placement は `child.set_property("h-align", ...)` ではなく、
   `parent.append_child(child, slot_metadata)` または parent-scoped builder のように表現される方が自然である。
   Phase 7b は API を設計しないが、その方向を塞がない DSL / IR / runtime model にする。

この thesis は、M3 の新 feature breadth を増やすものではない。むしろ、M3 public draft に入る前に、
既に入った Grid / ZStack placement surface の意味論を揃え、将来の iteration / construction API で破綻しない
概念境界を確定するための整理 phase である。

---

## Architectural-family trigger confirmation

Phase 7b では、`architectural-family.md` の re-evaluation trigger が発火していると扱う。
具体的には、M3 DSL spec drafting 中に author-facing grammar / placement surface を再整理しており、
これは DSL grammar が public-contract layer に影響するためである。

ただし本 framing は、Phase 7b のために vision decision record を新設しない。理由は次のとおり。

* Phase 7b は view-function with re-execution family への pivot ではない。
* SwiftUI / Compose 的な host-language scope modifier を導入する phase ではない。
* `.ui` は引き続き tree description であり、placement は tree の親子 slot に載る metadata として扱う。
* C ABI は引き続き handle-based / tree-mutation-compatible な方向であり、embedded scripting runtime や view-body re-execution primitive を必要としない。
* したがって、現行の tree-with-bindings working hypothesis 内で整理できる。

Architectural-family note には、Moment 1 または Moment 2 で次の扱いを記録する:

* Phase 7b re-read result: confirm within family (1).
* Placement surface は family (1) の tree / child-slot metadata として吸収。
* No DD-V vision decision record.

---

## Settled premises and open edges

### 決定済みとして再審議しない premise

* Wasamo は `.ui` external DSL を canonical declarative form とし、host language API は派生形として扱う。
* Phase 7b は新 layout primitive を導入しない。
* Grid と ZStack の layout algorithm 自体は再設計しない。
* Placement-like attributes は、親が解釈しなければ意味を持たない。
* `Cell` は runtime widget として materialise する widget ではなく、Grid placement を運ぶ構造的 wrapper として扱われている。
* ZStack の `h-align` / `v-align` は、ZStack direct child にだけ許される parent-consumed placement annotation であり、child の通常 prop set には残らない。
* 既存 DD-M3-P7-006 は、parallel-vector drift を観測済み failure mode として扱い、child-carried placement を推奨している。
* Phase 7b は public C ABI / Rust safe API / Zig API に新規 code-construction surface を追加しない。

### 本 framing で open DD に送る edge

* Grid と ZStack の author-facing placement surface を、edge wrapper に寄せるか、fixed prefix に寄せるか、prefix なし sugar を維持するか。
* Fixed prefix を採用する場合、`slot.` / `placement.` / `parent.` / `layout.` のどれを使うか。
* Grid の `Cell` を維持するか、`slot.row` / `slot.column` 的な direct-child surface に移すか。
* ZStack の direct child `h-align` / `v-align` を維持するか、`slot.h-align` / `Layer` 的な形へ移すか。
* Existing examples / gallery `.ui` を即時更新するか、compat alias を短期的に許すか。
* IR / textual IR で placement を child prop として表現するか、child-slot metadata として表現するか。
* Runtime storage を DD-M3-P7-006 ST2 として進めるか、Phase 7b の DSL choice に合わせて再定義するか。
* Future code-construction API に対し、どの程度まで非コミット制約を記録するか。

---

## 論点 slate（DD questions — 番号予約のみ）

本 phase の ADR set（`decisions/preamble.md` + DD ごとに 1 ファイル）が担う論点を列挙し、
**DD-M3-P7b-NNN 番号を予約**する。各 DD の options / 比較 / 推奨は §3 設計判断で書く。
ここでは「何を判断するか」と「なぜ Phase 7b の問いか」だけを固定する。

### DD-M3-P7b-001 — Parent-interpreted placement authoring surface

**問い:**
作者は parent-interpreted placement metadata を `.ui` でどう書くべきか。

**Phase 7b の問いである理由:**
Grid は `Cell` wrapper、ZStack は direct child 上の `h-align` / `v-align` で placement を表しており、
author-facing surface に一貫性がない。M3 public draft にこのまま入れると、
外部読者に「これは widget property なのか、親子関係の属性なのか」を説明しづらい。
また、将来 code-construction API を入れる場合、child property setter に見える surface は誤った API 方向を誘導しうる。

**sub-issues:**

* Surface form:

  * Option 1: Grid `Cell` / ZStack `Layer` のような edge wrapper に寄せる。
  * Option 2: prefix は置かず、意味論だけ child-edge / child-slot に統一する。
  * Option 3: fixed prefix を置く。
  * Option 4: XAML-style attached property。
  * Option 5: parent-declared modifier namespace。
* Fixed prefix の語彙:

  * `slot.`
  * `placement.`
  * `parent.`
  * `layout.`
* Grid surface:

  * `Cell` 維持。
  * `slot.row` / `slot.column` に移行。
  * `Cell` を sugar / legacy form にする。
* ZStack surface:

  * direct child `h-align` / `v-align` 維持。
  * `slot.h-align` / `slot.v-align` へ移行。
  * `Layer` wrapper を導入。
* Diagnostics:

  * placement attr が許される parent context。
  * stray placement attr の reject。
  * unknown widget prop との切り分け。
* Migration:

  * existing examples / gallery update。
  * pre-1.0 compatibility alias の有無。
  * dsl_spec の stale prose sweep。

### DD-M3-P7b-002 — Child-slot placement model and construction boundary

**問い:**
DSL で書かれた parent-interpreted placement metadata を、IR / textual IR / runtime storage / structural mutation /
future code-construction API 方針でどう表すべきか。

**Phase 7b の問いである理由:**
DD-M3-P7-006 は storage model と structural side-effect atomicity を扱ったが、
その spec content seed は author surface unchanged を前提としていた。Phase 7b は author surface 自体を
変える可能性があるため、child-slot model を architecture contract として再接続する必要がある。
また、将来 code-construction API を入れるなら、placement を child property setter で表すのか、
parent insertion / child-slot builder で表すのかを、少なくとも方向として記録しておく必要がある。

**sub-issues:**

* Internal model:

  * Widget property model。
  * Parent parallel metadata。
  * Encapsulated SoA + splice-only mutation。
  * Child-slot-carried placement。
  * Keyed metadata map。
* IR / textual IR:

  * Existing child `IrProp` consumption。
  * Explicit child-slot record。
  * Parent-specific placement payload。
  * Loader validation policy。
* Runtime storage:

  * `Vec<WidgetNode>` + side metadata。
  * `Vec<ChildSlot>` conceptual model。
  * container-specific child-entry type。
  * common placement enum vs per-container placement payload。
* Structural mutation:

  * DD-M3-P7-006 splice primitive をそのまま採用するか。
  * splice side effects を Phase 7b で再列挙するか。
  * Grid migration trigger を維持するか、Phase 7b で Grid も移すか。
* Future code-construction boundary:

  * No new API in Phase 7b。
  * Future API should prefer parent-scoped insertion / child-slot builder。
  * Avoid generic child property setter for placement。
  * Concrete API sketches are non-normative and must not freeze ABI shape。
* Documentation:

  * `docs/architecture.md` に child-slot model を記述。
  * `docs/dsl_spec.md` に author-facing placement surface と diagnostics を記述。
  * `docs/abi_spec.md` は原則 no-touch。ただし future compatibility note が必要かは DD で判断。

---

## Phase 7b scope

### In scope

Phase 7b で扱う範囲は次のとおり。ここでは、具体的な options / recommendation はまだ決めないが、
ADR が判断すべき面積は固定する。

* Grid / ZStack の parent-interpreted placement surface の統一方針。
* `Cell`, `Layer`, `slot.*`, `placement.*`, no-prefix direct child attr などの author-facing DSL 比較。
* Placement-like attributes と ordinary widget properties の境界。
* Parser / checker / lowering / textual IR / loader / runtime storage における placement の表現方針。
* ZStack `h-align` / `v-align` と Grid `row` / `column` / `span` / alignment の admission / rejection rules。
* Existing examples / gallery `.ui` の更新方針。
* `docs/dsl_spec.md` の placement surface chapter / invalid examples。
* `docs/architecture.md` の child-slot model / structural mutation contract。
* DD-M3-P7-006 との関係整理:

  * consume as premise,
  * revise,
  * supersede,
  * or split responsibilities.
* Future code-construction API/ABI との非コミット互換制約。
* Verification tests:

  * parser/check diagnostics,
  * lowering / loader,
  * runtime storage / layout correctness,
  * GUI positive control where necessary.

### Out of scope（activation trigger 付きで carry するもの）

次の表を deferred items の**正本**とする。ADR の forward-compat と implementation handoff は、
この表をコピーまたは精密化して使う。別の表を作り直して責務先や trigger を分散させない。

| Deferred item                                             | 責務を置く先                                                               | activation trigger                                                                                            | 理由                                                                                                                                                                |
| --------------------------------------------------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Public code-construction API / ABI                        | Future code-construction phase / M6 ABI freeze preparation           | Widget constructors or tree-building APIs are promoted beyond experimental / internal use                     | Phase 7b は API shape を固定しない。placement is child-slot metadata という方向だけ記録する。                                                                                         |
| Generic modifier system                                   | Future DSL ergonomics / styling phase                                | Non-placement modifier use case が concrete app から出る                                                           | Phase 7b は placement に限定する。styling / behavior / accessibility modifier を同じ構文に混ぜない。                                                                                |
| User-defined containers and custom slot attributes        | Component / custom-layout phase                                      | User-defined layout container が child placement keys を定義する必要が出る                                               | Phase 7b は built-in Grid / ZStack の parent-interpreted placement に限定する。                                                                                           |
| Non-layout parent-data                                    | M4+ input / accessibility / modal behavior phase                     | Hit-test capture, focus grouping, accessibility relationship など、layout 以外の parent-interpreted metadata が必要になる | `slot.*` を採用する場合でも、Phase 7b は layout placement のみを規範化する。                                                                                                          |
| Keyed child metadata / retained identity                  | Future keyed identity / reorder phase                                | list reorder, retained subtree state, `key:` surface が開く                                                      | Phase 7b の child-slot は structural parent-child edge であり、element identity / keyed diff とは別問題。                                                                     |
| Grid structural mutation under Cell                       | Future Grid mutation phase, or Phase 7b if DD explicitly pulls it in | direct `for` of `Cell`, conditional `Cell`, or structural mutation under Grid is admitted                     | Current Phase 7 DD deferred Grid storage migration because direct `for` under Grid is rejected. Phase 7b may revise this, but if not revised the trigger remains. |
| Layout algorithm changes                                  | Future layout primitive refinement phase                             | Grid / ZStack geometry itselfが不足する concrete app case が出る                                                      | Phase 7b は representation / surface の整理であり、新しい measure-arrange semantics は作らない。                                                                                   |
| Backward compatibility guarantee for old placement syntax | Pre-1.0 compatibility policy / public draft stabilization            | External users depend on shipped syntax or public docs declare stability                                      | Pre-1.0 では必要最小限の migration に留める。compat alias を置くかは DD-001 で判断する。                                                                                                  |

### Acceptance mapping

| Acceptance / obligation       | Phase 7b での discharge                                                                                                                        |
| ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| New-AC question (contingent)  | §"Phase 7b acceptance criteria" のとおり、DD-001 が author surface を変える (a) なら AC-revision 例外で AC 追加 / A2・A4・A12 refine、変えない (b) なら A11/A12 で discharge。ADR Accepted 時点で確定し plan Revision log に記録。 |
| M3 A11 synchronization        | DD-001 / DD-002 の採択後、`.ui`, `wasamo-ir`, `wasamoc`, `wasamo-runtime`, `docs/dsl_spec.md`, `docs/architecture.md`, `examples/gallery/` を同期する。 |
| M3 A12 public draft quality   | DD-001 で author-facing placement surface と invalid examples を決め、`docs/dsl_spec.md` に説明可能な形で反映する。                                             |
| Phase 7 DD-M3-P7-006 overlap  | DD-002 で consume / revise / supersede の関係を明記する。                                                                                              |
| Future code-construction risk | DD-002 で API を追加せず、placement を child property setter として扱わない future-compat constraint を記録する。                                                 |
| Parallel data drift risk      | DD-002 と implementation gates #2 / #3 で structural side-effect enumeration と storage invariant を閉じる。                                         |

---

## Verification strategy

Phase 7b の検証は、**新しい見た目を作ること**ではなく、**同じ見た目が新しい placement model で一貫して表現・検証・実行されること**を示す。

### Visible E2E proof

* Gallery / lightbox / overlay で、ZStack child placement が旧 surface と同等の位置に出ること。
* Grid を使う既存 gallery sub-screen で、cell placement が旧 surface と同等の位置に出ること。
* ZStack positive control:

  * `slot.h-align: end` / equivalent accepted syntax の child が右寄せになる。
  * 対照として alignment を変えた frame が異なる位置に出る。
* Grid positive control:

  * row / column / span / alignment が visual に反映される。
  * stray / omitted placement の default behavior が期待通りになる。
* GUI rendering task では screenshot + analysis + positive control を close artifact とする。

### Pure-logic tests

* Parser accepts the chosen syntax.
* Parser / checker rejects all non-chosen or deferred forms where applicable.
* Placement attr is rejected under parent containers that do not admit it.
* Ordinary widget property checking does not accidentally accept placement attr as widget property.
* Grid / ZStack placement defaults are preserved.
* Migration / compatibility alias tests, if aliases are accepted.
* Roundtrip / textual IR tests:

  * chosen DSL surface lowers to child-slot metadata;
  * textual IR loader re-rejects malformed placement metadata;
  * stale old IR form is either accepted via migration or rejected with named diagnostic, depending on DD.

### Windows-headless / runtime integration tests

* ZStack layout reads placement from child-slot storage, not parallel vector.
* Grid layout reads placement from chosen storage path if Grid is migrated in Phase 7b.
* Structural insert / remove under ZStack preserves:

  * child order,
  * Visual sibling order,
  * placement,
  * layout invalidation.
* If `for` is already implemented before Phase 7b lands, range-generated ZStack children carry placement through staging → commit.
* Destroy / detach path does not leak placement metadata.
* No-placement containers carry `None` / equivalent and do not pay placement-specific logic except through the generic child-slot path.

### Implementation gate expectations

Phase 7b の多くの implementation task では、次の traps が適用される見込みである。

* **#1 semantic migration**: `IrProp`, placement extraction, widget child traversal, textual IR loader, validator, roundtrip, layout arrange loops の call-site audit が必要。
* **#2 missed side effects**: child placement migration は layout invalidation, Visual sibling order, registry teardown, effect ownership, placement metadata を同時に扱う。
* **#3 parallel data drift**: 本 phase の中心。child list と placement metadata のズレを構造的に消す、または残る path を明示的に閉じる。
* **#4 untested authored branch**: 新 syntax / reject diagnostics / compatibility aliases は直接 firing test を持つ。
* **#5 carry-forward**: future code-construction API, generic modifier, custom container, keyed identity, Grid structural mutation trigger を handoff に記録する。
* **#7 GUI positive control**: ZStack / Grid placement の visible evidence は screenshot + positive control を伴う。

---

## Risks

### R1. Scope creep into a general modifier system

`slot.*` や `placement.*` を議論すると、styling / input / accessibility / behavior modifier まで同じ構文で扱いたくなる。
Phase 7b は parent-interpreted layout placement に限定する。generic modifier system は concrete driver が出るまで開かない。

### R2. Surface churn before public draft

Grid / ZStack の syntax を変えると、既存 examples / docs / owner mental model が揺れる。
ただし M3 public draft 前に整理する方が、public draft 後に breaking change するより小さい。
DD-001 は pre-1.0 compatibility alias の有無を明示的に扱う。

### R3. DD-M3-P7-006 と責務が重複する

DD-M3-P7-006 は storage / splice primitive の DD として既に存在する。
Phase 7b が内部 model を再度扱うと、同じ decision を二重に持つ危険がある。
DD-002 は、DD-M3-P7-006 を consume / revise / supersede のどれで扱うかを最初に明記する。

### R4. Future API を今決めすぎる

コード構築 API との親和性を意識しすぎると、まだ driver のない ABI shape を先に固定してしまう。
Phase 7b では API shape を決めない。決めるのは「placement は child property ではなく parent-scoped child-slot metadata」
という方向だけである。

### R5. Implementation larger than corrective phase

DSL syntax, IR, runtime storage, examples, docs を同時に触ると、Phase 7b が想定より大きくなる。
回避策として、DD slate は 2 本に限定し、layout algorithm / generic modifier / public API を明示的に out of scope とする。

---

## Owner-agreed framing decisions

### FD-7b-A. Phase 7b thesis

Phase 7b は、parent-interpreted placement attributes を child-slot metadata として整理する corrective phase である。
新 layout primitive や新 app feature を追加する phase ではない。

**Status:** draft

### FD-7b-B. DD slate

Phase 7b の ADR set は次の 2 DD を持つ。

* DD-M3-P7b-001 — Parent-interpreted placement authoring surface
* DD-M3-P7b-002 — Child-slot placement model and construction boundary

**Status:** draft

### FD-7b-C. Architectural family

Phase 7b は architectural-family re-evaluation trigger を踏むが、vision decision record は新設しない。
tree-with-bindings working hypothesis 内で吸収する。

**Status:** draft

### FD-7b-D. Future code-construction boundary

Phase 7b は public code-construction API / ABI を追加しない。
ただし、placement は child property setter ではなく parent-scoped child-slot metadata として将来 API に渡るべき、
という非コミット制約を DD-002 に記録する。

**Status:** draft

### FD-7b-E. Scope containment

Generic modifier system、custom layout container、non-layout parent-data、keyed identity、public API design は Phase 7b scope 外とし、
activation trigger 付きで carry する。

**Status:** draft

---

## Inputs absorbed

### From DD-M3-P7-006 — Placement storage model and structural side-effect atomicity

次の内容を Phase 7b framing の入力として吸収した。

* Current storage uses parent-owned per-child placement metadata parallel to `children`.
* Parallel data drift was observed in Phase 6 and is not hypothetical.
* ST2 child-carried placement is the proposed structural fix.
* The child slot carries node plus optional parent-interpreted placement kind.
* The concrete value space may be common enum or per-container child-entry type; the important contract is parent-interpreted child-slot-carried shape.
* The splice primitive side-effect enumeration is the baseline for structural mutation audit.
* The original DD assumed author surface unchanged; Phase 7b reopens that assumption.

### From docs/architecture.md — ZStack and C ABI current state

次の内容を Phase 7b framing の入力として吸収した。

* ZStack currently takes children directly in document order.
* ZStack `h-align` / `v-align` are authored as child props but consumed by the parent context as placement annotations.
* ZStack placement metadata is carried parallel to children in the current architecture prose.
* The C ABI is handle-based and has tree mutation in the stable core.
* M1 experimental constructors existed, but constructor promotion is deferred.
* Phase 7b must not accidentally commit a future public code-construction API shape.

### From docs/dsl_spec.md — Grid / Cell and ZStack author surface

次の内容を Phase 7b framing の入力として吸収する。

* Grid uses `Cell` as a Grid-specific wrapper / placement carrier rather than arbitrary attached properties on any child widget.
* `Cell` is not a free-standing runtime widget.
* ZStack uses direct child placement annotations rather than a `Layer` / `Cell`-style wrapper.
* This creates an author-facing inconsistency even though the meaning is parent-interpreted placement in both cases.

### From architectural-family.md

次の内容を Phase 7b framing の入力として吸収した。

* Wasamo currently fits tree-with-bindings best, but this is a live working hypothesis, not a ratified long-term commitment.
* M3 DSL grammar choices are a re-evaluation trigger.
* C ABI as handle-based, textual IR as tree description, and DSL grammar are family-coupled at the public-contract boundary.
* Phase 7b confirms within the current family rather than opening a vision decision record.

---

## Next session — handoff

Owner がこの framing に alignment したら、次の段階では Phase 7b ADR set を draft する。

* `process/milestone-3/phase-7b/decisions/preamble.md`
* `process/milestone-3/phase-7b/decisions/dd-m3-p7b-001-parent-interpreted-placement-authoring-surface.md`
* `process/milestone-3/phase-7b/decisions/dd-m3-p7b-002-child-slot-placement-model-and-construction-boundary.md`

ADR draft は `Status: Proposed` から始める。その後、owner review を経て `Status: Accepted` に進め、
続けて Moment 1 design-spec sync を行う。
