---
title: M3-Phase 7b framing — parent-interpreted placement attributes
status: aligned
created: 2026-06-19
target-phase: M3-Phase 7b
-------------------------

# M3-Phase 7b framing

**Status:** aligned (owner-aligned 2026-06-19)
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
各項目は合意後に右端の **確定先** へ転記される（1 項目 = 1 合意単位 = 1 確定先）。詳細な根拠は後続の各節に置く。

| ID | 決めてほしいこと | 推奨 | 確定先 |
| -- | -- | -- | -- |
| A | Phase 7b の目的と thesis | 既存 placement surface を揃える **corrective phase**。settled な床は「親が解釈する child placement は widget の **intrinsic property ではない**」まで。**container-specific sugar か generalizable parent-data か**（概念軸）と、**child-slot か parallel か**（storage 軸）は結論ではなく DD で開く。child-slot は観測済み drift ゆえ**有力仮説**として記すが既定ではない。新 layout primitive は足さない。 | FD-7b-A |
| B | DD slate（2 本構成と予約する比較空間） | **DD-001** = author-facing DSL surface（比較空間: edge wrapper / prefix なし / fixed prefix / XAML-style attached property / parent-declared namespace）。**DD-002** = placement internal model and construction boundary（比較空間: widget property model / parent parallel metadata / encapsulated SoA / child-slot-carried metadata / keyed metadata map）。migration / compatibility / diagnostics は各 DD の sub-issue とし、独立 DD にしない。 | FD-7b-B（比較空間の詳細は §論点 slate） |
| C | Architectural family の扱い | placement grammar の整理は architectural-family note の **re-evaluation trigger 1（M3 DSL spec drafting）を踏む**。各 surface option（attached property / parent-declared namespace を含む）が family に与える影響を DD-001 で**確認**し、family (1) 内に収まるなら **revise-in-place（VDR 不要）**、pivot 級なら **VDR へ昇格**する——という条件付き判断を記録する。surface option はいずれも tree-description grammar で view-function re-execution ではないため **family (1) 内 confirm が期待される**が、結論は DD の出口で確定する。これは DD-V-026 の**比例記録の原則**（重い artifact は thesis 反転 / family pivot 級にのみ留保）の application であり、Phase 7 Moment-2 の confirm-no-VDR と同型。 | FD-7b-C ＋ architectural-family.md alignment 表（Moment 1/2 で記録） |
| D | Future code-construction API/ABI | Phase 7b では API/ABI を追加しない。非コミット制約として記録するのは「**placement を generic child property setter で表さない**」（概念 thesis 由来）まで。正の形（parent-scoped insertion / child-slot builder 等）は**非規範**とし固定しない。具体 API signature は後続 phase へ送る。 | FD-7b-D |
| E | Scope（実装範囲と containment） | ADR が Accepted された場合、Phase 7b は docs だけでなく、選ばれた surface に必要な parser / checker / lowering / runtime / examples の同期まで実装する。新 layout algorithm は作らない。generic modifier system / custom layout container / non-layout parent-data / keyed identity / public API design は activation trigger 付きで carry する。 | FD-7b-E |
| F | 新 AC の要否（contingent） | いま決め打ちしない。**DD-M3-P7b-001 の出口**で、(a) author surface を変えるなら AC-revision 例外で新 AC 追加 or A2 / A4 / A12 refine、(b) 変えないなら A11 / A12 で discharge。ADR Accepted 時点で確定し plan Revision-log に記録する。 | §Phase 7b acceptance criteria ＋ plan Revision-log |

**返事チェックリスト:**

* A 目的と thesis: ☐
* B DD slate 2 本＋比較空間: ☐
* C Architectural family（条件付き confirm／pivot なら VDR 昇格）: ☐
* D Future code-construction は非コミット制約のみ: ☐
* E Scope（実装まで＋containment）: ☐
* F 新 AC は DD-001 contingent: ☐

---

## オーナー合意の記録

**2026-06-19**: owner が Owner alignment packet A〜F に正式合意。
A→FD-7b-A、B→FD-7b-B、C→FD-7b-C、D→FD-7b-D、E→FD-7b-E、
F→§Phase 7b acceptance criteria ＋ plan Revision-log に確定。

合意に至る経緯: framing draft（`355e23a`, pre-alignment）に対し owner との対話で packet を
case A 構造（A〜F・確定先列）へ再編し、FD-7b-C を「条件付き confirm／pivot なら VDR 昇格」に補正。
続けて Codex の 3 パスのレビューを批判的に検討して folding し、(i) thesis を「placement は intrinsic
widget property ではない」を床とし child-slot / generalizable を DD で開く軸に降格、(ii) DD-001 に
Option 0（principled asymmetry / status quo）・name-collision rule・control-flow surface・forward-compat
reservation を追加、(iii) DD-002 に bindability 方針・textual IR compatibility policy を追加し名称を
中立化（Placement internal model …）、(iv) stale fact（ZStack は Phase 7 で child-carried 実装済み・
Grid 未移行）と default 所有（Grid stretch / ZStack center）を訂正、(v) 用語を storage 中立へ磨き込み、
まで反映済み。

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
  新しい author-facing 契約は生まれない（DD-002 が Grid storage migration や textual IR change を採れば
  内部実装は変わりうるが、外部読者が依存する新しい author-facing 契約は生じない）ので、Phase 7b は
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

> 親 container が解釈する child placement 情報を、**widget 自身の intrinsic property と混同しない公開モデル**
> として確立し、Grid / ZStack の author-facing DSL surface、IR / runtime storage、将来の code-construction
> API 方針を、同じ概念境界の上で coherent に揃える。placement を container-specific sugar とみなすか
> generalizable parent-data とみなすか、また storage を child-slot 化するかは、この thesis の**結論ではなく
> DD で開く軸**である。

Phase 7b の設計 thesis は、settled な床と、DD で開く軸に分けて整理する。

**Settled な床（再審議しない）:**

1. **Placement is not an intrinsic widget property.**
   `Text.text` や `Button.enabled` は widget 自身の属性である。一方、Grid の `row` / `column`、
   ZStack の `h-align` / `v-align` は、その child が直近の親 container の中でどう扱われるかを指定する
   parent-interpreted な情報である。これは container-specific sugar 派・generalizable parent-data 派の
   どちらも合意する概念の床であり、ここまでが thesis である。

**DD で開く軸（結論ではない）:**

2. **Conceptual boundary — container-specific か generalizable か。**
   placement を「Grid 固有の `Cell` / ZStack 固有の annotation という container-specific な構文」とみなすか、
   「container 横断の generalizable な parent-data grammar（例 `slot.*`）」とみなすかは、surface と storage の
   手前にある上位の概念判断である。DD-001 冒頭で比較する。

3. **Author surface must not obscure the model.**
   Grid は `Cell` wrapper、ZStack は direct child 上の `h-align` / `v-align` という surface を持つ。
   現状はどちらも parent-interpreted placement だが見た目が違う。Phase 7b はこのブレを、edge wrapper /
   fixed prefix / prefix なし sugar / attached property / namespace、および **非対称を設計思想として
   明文化・維持する案（status quo）** として明示比較する。

4. **Runtime storage — drift を構造的に消すか。**
   parent-owned placement を `children` と並列 vector で持つと insert / remove / splice / reorder でずれる
   危険がある。child-slot record に載せる案は Phase 6 で観測された drift を構造的に消すため **有力仮説**だが、
   parallel / encapsulated SoA / keyed map も DD-002 の比較対象に残す。なお ZStack は Phase 7 で既に
   child-carried へ同期済み、Grid は parallel のまま——Phase 7b はこの非対称を author surface と合わせて再接続する。

5. **Future code-construction API must remain natural.**
   将来コードから UI tree を組む API を入れる場合、placement を generic child property setter
   （`child.set_property("h-align", …)`）で表さないことだけを非コミット制約として残す。正の形
   （parent-scoped insertion / builder 等）は非規範とし、Phase 7b では固定しない。

この thesis は、M3 の新 feature breadth を増やすものではない。むしろ、M3 public draft に入る前に、
既に入った Grid / ZStack placement surface の意味論を揃え、将来の iteration / construction API で破綻しない
概念境界を確定するための整理 phase である。

---

## Architectural-family trigger confirmation

Phase 7b では、`architectural-family.md` の re-evaluation trigger 1（M3 DSL spec drafting）が発火していると扱う。
具体的には、author-facing grammar / placement surface を再整理しており、これは DSL grammar が public-contract
layer に影響するためである。

本 framing は VDR 要否を**結論として先取りしない**。DD-001 の各 surface option が family に与える影響を確認
したうえで、family (1) 内に収まるなら revise-in-place（VDR 不要）、pivot 級なら VDR へ昇格する。現時点の
見立てでは、surface option はいずれも family (1) 内に収まると**期待される**。理由:

* surface option（edge wrapper / prefix / attached property / parent-declared namespace）はいずれも
  tree-**description** grammar であり、view-function with re-execution（family 2）ではない。
* SwiftUI / Compose 的な host-language scope modifier（view-body 再実行の単位）を導入する phase ではない。
* `.ui` は引き続き tree description であり、placement は tree の親子 slot に載る metadata として扱う。
* C ABI は引き続き handle-based / tree-mutation-compatible な方向であり、embedded scripting runtime や view-body re-execution primitive を必要としない。

この「trigger は踏むが、確認のうえ family (1) 内なら VDR 不要」という条件付き運用は、DD-V-026 の比例記録原則
（重い artifact は thesis 反転 / family pivot 級にのみ留保）の application である。

Architectural-family note には、Moment 1 または Moment 2 で次の扱いを記録する:

* Phase 7b re-read result: 各 surface option の family-impact を確認した結果（期待は confirm within family (1)）。
* family (1) 内なら placement surface を tree / parent-interpreted placement metadata として吸収、VDR なし。
* もし DD-001/002 が pivot 級の choice を採るなら VDR へ昇格。

---

## Settled premises and open edges

### 決定済みとして再審議しない premise

* Wasamo は `.ui` external DSL を canonical declarative form とし、host language API は派生形として扱う。
* Phase 7b は新 layout primitive を導入しない。
* Grid と ZStack の layout algorithm 自体は再設計しない。
* Placement-like attributes は、親が解釈しなければ意味を持たない。
* `Cell` は runtime widget として materialise する widget ではなく、Grid placement を運ぶ構造的 wrapper として扱われている。
* ZStack の `h-align` / `v-align` は、ZStack direct child にだけ許される parent-consumed placement annotation であり、child の通常 prop set には残らない。
* **storage の現状**: ZStack は M3-Phase 7 で **child-carried placement に実装同期済み**（architecture.md §6.8.5）。Grid は static-only ゆえ `cell_placements` を **parallel-vector のまま**保持し、structural-mutation path が開くときに child-carried へ移す trigger が記録されている。DD-M3-P7-006 は parallel-vector drift を観測済み failure mode として child-carried を推奨した DD だが、その推奨は **ZStack では実装済み・Grid では未適用**である。
* **default は parent container が所有する**: surface を揃えても default semantics は統一されない。現状 Grid `Cell` の `h-align` / `v-align` default は **stretch**、ZStack の default は **center**。surface 統一は default 統一を含意しない（default を変えるなら別個の layout-behavior change になる）。
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

* Conceptual boundary（先頭で扱う上位判断）:

  * placement は container-specific な DSL sugar（Grid 固有 `Cell` / ZStack 固有 annotation）か、
    container 横断の generalizable な parent-data grammar（`slot.*` 等）か。
  * この概念判断が以降の Surface form の評価軸を規定する。
* Surface form:

  * **Option 0: principled asymmetry / documented status quo** — Grid = structured `Cell` wrapper、
    ZStack = lightweight direct annotation の非対称を、「構造データを伴う placement のみ wrapper、単純な
    alignment は直付け」という設計思想として明文化し維持する。既存実装への追認ではなく意図されたモデルか
    を判断する一級 option。
  * Option 1: Grid `Cell` / ZStack `Layer` のような edge wrapper に統一して寄せる。
  * Option 2: prefix は置かず、意味論だけ child-edge / child-slot に統一する。
  * Option 3: fixed prefix を置く。
  * Option 4: XAML-style attached property。
  * Option 5: parent-declared modifier namespace。
  * 各 option には architectural family への影響（family (1) 内か pivot 級か）を 1 行で添える（FD-7b-C）。
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
* Name-collision rule（surface option の比較軸でもある）:

  * parent-consumed placement attr が将来 child widget の通常 prop と同名になった場合の規則
    （親が先に consume / `slot.*` 等の namespace で回避 / diagnostic）。
  * no-prefix 直書きは衝突リスク、prefix / namespace は回避——この差は surface option の優劣に直結する。
* Control-flow × placement の author surface:

  * `for` / `if` で生成される child に placement をどう書くか（body root child に付く / block に付く /
    生成各 child に複製）。runtime は Phase 7 で「生成 child が placement を staging→commit で運ぶ」まで
    実装済み（architecture.md §6.7.10 / §6.8.5）なので、ここで決めるのは author surface の見え方。
  * per-iteration で placement を変えられるか（= bindability、DD-002 と連結）。
* Placement の bindability（surface 側の見え方）:

  * 現状 placement は literal/constant。bindable 化の実装は Phase 7b でしないが、author surface 上
    「将来 bind できる concept か」を DD-002 の storage 方針と一貫させる。
* Forward-compat reservation（実装は defer・互換だけ今見る）:

  * 選んだ surface が、将来の custom layout container の custom slot attr や non-layout parent-data
    （hit-test / focus / accessibility）と**衝突しない予約条件**を満たすこと。将来 system は設計しないが、
    構文が将来拡張を偶然狭めない条件を名指す。
* Diagnostics:

  * placement attr が許される parent context。
  * stray placement attr の reject。
  * unknown widget prop との切り分け。
* Migration:

  * existing examples / gallery update。
  * pre-1.0 compatibility alias の有無。
  * dsl_spec の stale prose sweep。

### DD-M3-P7b-002 — Placement internal model and construction boundary

**問い:**
DSL で書かれた parent-interpreted placement metadata を、IR / textual IR / runtime storage / structural mutation /
future code-construction API 方針でどう表すべきか。

**Phase 7b の問いである理由:**
DD-M3-P7-006 は storage model と structural side-effect atomicity を扱ったが、
その spec content seed は author surface unchanged を前提としていた。**その child-carried 推奨は ZStack では
M3-Phase 7 で実装済み・Grid では未適用**であり、Phase 7b は author surface 自体を変える可能性があるため、
placement storage model を architecture contract として（ZStack の既存実装と Grid 未移行の非対称も含めて）再接続する必要がある。
また、将来 code-construction API を入れるなら、placement を child property setter で表すのか、
parent insertion / child-slot builder で表すのかを、少なくとも方向として記録しておく必要がある。

**sub-issues:**

* Internal model（child-slot は観測済み drift ゆえ**有力仮説**だが、既定ではなく比較する）:

  * Widget property model。
  * Parent parallel metadata。
  * Encapsulated SoA + splice-only mutation。
  * Child-slot-carried placement（leading hypothesis）。
  * Keyed metadata map。
* IR / textual IR:

  * Existing child `IrProp` consumption。
  * Explicit child-slot record。
  * Parent-specific placement payload。
  * Loader validation policy。
  * **Compatibility policy**: child-slot record 化が textual IR の破壊的変更になる場合、旧 form を loader で
    migrate するか、reject するか、IR schema revision として扱うか。pre-1.0 の textual IR は wasamoc が `.ui`
    から毎ビルド再生成する build-internal artifact ゆえ default は **reject + 再生成** 寄りだが、明示的に DD で決める。
* Placement の bindability / reactive mutation:

  * 現状 placement は literal/constant。Phase 7b で bindable にする実装はしないが、「placement は将来 bindable に
    なりうる public concept か」「layout 安定のため原則 constant-only か」を**方針 + trigger**として記録する
    （reactive architecture との境界。実装は defer）。
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
  * 非コミット制約は「generic child property setter で placement を表さない」まで（概念 thesis 由来）。
  * 正の形（parent-scoped insertion / child-slot builder 等）は非規範で、ABI shape を固定しない。
  * Concrete API sketches are non-normative and must not freeze ABI shape。
* Documentation:

  * `docs/architecture.md` に chosen placement model を記述（child-slot 採用時はその model）。
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
* `docs/architecture.md` の chosen placement model / structural mutation contract。
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
| Public code-construction API / ABI                        | Future code-construction phase / M6 ABI freeze preparation           | Widget constructors or tree-building APIs are promoted beyond experimental / internal use                     | Phase 7b は API shape を固定しない。placement を generic child property setter で表さない、という方向だけ記録する（正の形は非規範）。                                                                                         |
| Generic modifier system                                   | Future DSL ergonomics / styling phase                                | Non-placement modifier use case が concrete app から出る                                                           | Phase 7b は placement に限定する。styling / behavior / accessibility modifier を同じ構文に混ぜない。構文が将来の modifier 拡張を偶然狭めない予約条件は DD-001 で見る（実装は defer）。                                                                                |
| User-defined containers and custom slot attributes        | Component / custom-layout phase                                      | User-defined layout container が child placement keys を定義する必要が出る                                               | Phase 7b は built-in Grid / ZStack の parent-interpreted placement に限定する。選んだ surface が custom slot attr と衝突しない予約条件は DD-001 で見る（実装は defer）。                                                                                           |
| Non-layout parent-data                                    | M4+ input / accessibility / modal behavior phase                     | Hit-test capture, focus grouping, accessibility relationship など、layout 以外の parent-interpreted metadata が必要になる | `slot.*` を採用する場合でも、Phase 7b は layout placement のみを規範化する。選んだ surface が non-layout parent-data と衝突しない予約条件は DD-001 で見る（実装は defer）。                                                                                                          |
| Keyed child metadata / retained identity                  | Future keyed identity / reorder phase                                | list reorder, retained subtree state, `key:` surface が開く                                                      | Phase 7b の placement metadata は structural parent-child edge の問題であり、element identity / keyed diff とは別問題。                                                                     |
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
| Future code-construction risk | DD-002 で API を追加せず、placement を generic child property setter として扱わない future-compat constraint を記録する（正の形は非規範）。                                                 |
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

  * chosen DSL surface lowers to the chosen placement storage model;
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
* If a child-slot model is chosen, no-placement containers carry `None` / equivalent and pay no placement-specific logic except through the generic child-slot path.

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
Phase 7b では API shape を決めない。決めるのは「placement を generic child property setter で表さない」という
非コミット制約だけで、正の形（parent-scoped insertion / builder 等）は非規範とする。

### R5. Implementation larger than corrective phase

DSL syntax, IR, runtime storage, examples, docs を同時に触ると、Phase 7b が想定より大きくなる。
回避策として、DD slate は 2 本に限定し、layout algorithm / generic modifier / public API を明示的に out of scope とする。

---

## Owner-agreed framing decisions

### FD-7b-A. Phase 7b thesis

Phase 7b は、parent-interpreted placement attributes を **widget の intrinsic property と混同しない公開モデル**
として整理する corrective phase である。settled な床は「placement は parent-interpreted であり intrinsic widget
property ではない」まで。container-specific sugar か generalizable parent-data か、および storage を child-slot 化
するかは結論ではなく DD で開く（child-slot は観測済み drift ゆえ有力仮説）。新 layout primitive や新 app feature を
追加する phase ではない。

**Status:** agreed (owner-aligned 2026-06-19)

### FD-7b-B. DD slate

Phase 7b の ADR set は次の 2 DD を持つ。

* DD-M3-P7b-001 — Parent-interpreted placement authoring surface
* DD-M3-P7b-002 — Placement internal model and construction boundary

各 DD が比較する設計空間は §論点 slate に予約する。migration / compatibility / diagnostics は
独立 DD にせず各 DD の sub-issue として扱う。

**Status:** agreed (owner-aligned 2026-06-19)

### FD-7b-C. Architectural family

Phase 7b は architectural-family re-evaluation trigger 1（M3 DSL spec drafting）を踏む。VDR 要否は
**結論として先取りせず**、DD-001 の各 surface option（attached property / parent-declared namespace を含む）が
family に与える影響を確認したうえで、**family (1) 内に収まるなら revise-in-place（VDR 不要）、pivot 級なら
VDR へ昇格**する、という条件付き判断とする。現時点の見立てでは、surface option はいずれも tree-description
grammar であり view-function re-execution（family 2）ではないため、**family (1) 内 confirm が期待される**が、
確定は DD の出口で行う。記録は revise-in-place（architectural-family.md の alignment 表と re-evaluation triggers
を Moment 1/2 で更新し commit）で行い、pivot 級と判明した場合のみ VDR を起こす。

この条件付き運用は、
[plan-revision-discipline](../../../cross-milestone/decisions/plan-revision-discipline.md)（DD-V-026）が確立した
**比例記録の原則**——重い artifact は thesis 反転 / family pivot 級にのみ留保し、confirm / additive な変更は
軽い記録で足りる——の application である。architectural-family note の Phase 7 Moment-2 re-read が同様に
confirm-within-family を VDR なしで処理した前例がある。

**Status:** agreed (owner-aligned 2026-06-19)

### FD-7b-D. Future code-construction boundary

Phase 7b は public code-construction API / ABI を追加しない。
非コミット制約として記録するのは「placement を generic child property setter で表さない」（概念 thesis 由来）まで。
正の形（parent-scoped insertion / child-slot builder 等）は非規範とし、ABI shape を固定しない。具体 signature は
後続 phase へ送る。

**Status:** agreed (owner-aligned 2026-06-19)

### FD-7b-E. Scope（実装範囲と containment）

ADR が Accepted された場合、Phase 7b は docs だけでなく、選ばれた surface に必要な
parser / checker / lowering / runtime / examples の同期まで実装する。新 layout algorithm は作らない。

Generic modifier system、custom layout container、non-layout parent-data、keyed identity、public API design は
Phase 7b scope 外とし、activation trigger 付きで carry する。

**Status:** agreed (owner-aligned 2026-06-19)

---

## Inputs absorbed

### From DD-M3-P7-006 — Placement storage model and structural side-effect atomicity

次の内容を Phase 7b framing の入力として吸収した。

* Parallel data drift was observed in Phase 6 and is not hypothetical.
* ST2 child-carried placement was the proposed structural fix; it is **implemented for ZStack in M3-Phase 7**, while **Grid remains on the parallel `cell_placements` vector** (static-only; migration trigger held).
* The child slot carries node plus optional parent-interpreted placement kind.
* DD-M3-P7-006 framed the important contract as a parent-interpreted child-slot-carried shape (the concrete value space may be common enum or per-container child-entry type).
* The splice primitive side-effect enumeration is the baseline for structural mutation audit.
* The original DD assumed author surface unchanged; Phase 7b reopens that assumption and re-connects the ZStack-implemented model to Grid and the DSL surface.

### From docs/architecture.md — ZStack and C ABI current state

次の内容を Phase 7b framing の入力として吸収した。

* ZStack currently takes children directly in document order.
* ZStack `h-align` / `v-align` are authored as child props but consumed by the parent context as placement annotations.
* ZStack placement is **child-carried** in the current architecture prose (storage contract revised and implementation-synced in M3-Phase 7, §6.8.5); the earlier parallel `zstack_placements` vector was replaced. Grid's `cell_placements` is still parallel because Grid is static-only.
* Grid `Cell` alignment defaults to `stretch`; ZStack alignment defaults to `center` — defaults are owned per container even where the attribute name is shared.
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
* Phase 7b checks each surface option's family impact; it is expected to confirm within the current family (VDR unnecessary), but escalates to a vision decision record if a DD-001/002 choice turns out pivot-level.

---

## Next session — handoff

Owner がこの framing に alignment したら、次の段階では Phase 7b ADR set を draft する。

* `process/milestone-3/phase-7b/decisions/preamble.md`
* `process/milestone-3/phase-7b/decisions/dd-m3-p7b-001-parent-interpreted-placement-authoring-surface.md`
* `process/milestone-3/phase-7b/decisions/dd-m3-p7b-002-placement-internal-model-and-construction-boundary.md`

ADR draft は `Status: Proposed` から始める。その後、owner review を経て `Status: Accepted` に進め、
続けて Moment 1 design-spec sync を行う。
