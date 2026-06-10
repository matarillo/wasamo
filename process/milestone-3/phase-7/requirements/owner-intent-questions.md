---
title: M3-Phase 7 owner-intent questions — イテレーション grammar
status: draft
created: 2026-06-10
source-phase: M3-Phase 6
target-phase: M3-Phase 7
---

# M3-Phase 7 owner-intent questions（framing 入力）

> **この文書の位置づけ.** これは workflow §2.2 の成果物（framing thesis /
> DD slate）**ではない**。§2.2 に入る前段の **elicitation 作業文書**である。
> イテレーションが新規に提起するオーナー意図の問いのうち、既存文書がまだ
> 答えていないものを抽出し、各問いを「pre-framing で潰すべきか / packet で
> provisional FD として運べるか」に分類する。**ここでは自答・推奨をしない**
> （§3 の選択肢空間は含意の見える化であって推奨ではない）。framing.md /
> dsl-grammar.md への書き込みもしない。
>
> 既決 premise（For variant / `ForLoopSubtree`、fresh 規範 + opt-in 形状、
> `HandlerExpr` 統一、式 grammar 一斉拡張規律、`for` 予約）は §1 にマップし、
> **再審議しない**。

---

## 0. Owner-supplied prior — 評価軸の重み付け（2026-06-10 合意）

本 phase 以降の比較・判断に効く cross-cutting なオーナー意図。下記の各問いの
選択肢評価、および §2.2 framing / DD slate / レビューはこの prior の下で行う。

- **過去合意は仮説.** 過去の合意（acceptance / spec / DD）は仮説であり、**適切な
  手順を踏めば改訂可能**。本文書が A8 / §9 等の改訂経路を中立に併記するのはこの
  方針の運用（[[feedback_revise_dont_workaround]] と整合）。
- **比較の主軸は product merit.** 実用性・thesis 整合が主軸。**実装・改訂コストは
  独立の評価軸ではなく tie-breaker** に留める。選択肢を却下するときは merit で
  却下する。
- **歯止め（同時に合意）.**
  - 「コスト非重視」は「常に大きい案を選ぶ」ではない。design-decision-review 段階 2
    の **過剰設計（将来拡張性を理由とした over-engineering）観点は存続**する。
  - オーナー条件「**マイルストーンの目的を達成するためであれば**」により、コストが
    効く唯一の経路は「他 M3 AC（特に Phase 8 / public draft）を脅かすか」。すなわち
    コストは消えず、**置き場が「DD の比較軸」→「plan / framing §Risks（schedule
    リスク管理）」へ移る**。
- **耐久記録.** この prior は §2.2 が開いたとき framing.md の **owner-agreed FD
  （評価軸の重み付け）**として蒸留し、以後の DD・レビュー全体から参照させる
  （本文書は framing 入力につき、ここでは prior の記録に留める）。

---

## 1. 既回答・既決マップ（出典つき）

「決定」と「先行 phase が記録した期待」を区別する。**期待は決定ではない** —
§2 で confirm-or-revise の問いに変換する。

### 1a. 決定済み（再質問・再審議しない）

| 項目 | 内容 | 出典 |
|---|---|---|
| 制御フロー family の出発点 | iteration は `IrMember` / `ControlFlowNode` family の拡張として実装する。widget として materialise しない | [constraints §1](./constraints.md); [phase-6 handoff §Phase 7 targets](../../phase-6/implementation/handoff.md) |
| `For` 着地点 | `ControlFlowNode::For { binding, body }` が `BindingTarget::ForLoopSubtree`（予約済み slot）を埋める | [DD-004 forward-compat 3](../../phase-6/decisions/dd-m3-p6-004-conditional-ir-and-runtime-present-absent.md); [constraints §2](./constraints.md) |
| range target の ownership 問題 | declared slot identity / 挿入・削除 atomicity / registry teardown / effect drain timing / failure reporting は `ConditionalSubtree` の問題を再利用 | [constraints §2](./constraints.md); [phase-6 handoff](../../phase-6/implementation/handoff.md) |
| fresh-on-return 規範（**Phase 6 base case として**） | absent→present は fresh rebuild、state リセット。これは規範的 author-visible semantics で spec に fold 済み | [DD-004 ID-1](../../phase-6/decisions/dd-m3-p6-004-conditional-ir-and-runtime-present-absent.md); [DD-005 LA-1](../../phase-6/decisions/dd-m3-p6-005-conditional-effect-lifecycle-and-drain-contract.md) |
| retention は opt-in 互換形状 | 将来の state retention は **keyed / explicit opt-in** として入り、既存 destroy/rebuild default を silent に変えない | [DD-004 ID-1 Recommendation](../../phase-6/decisions/dd-m3-p6-004-conditional-ir-and-runtime-present-absent.md) |
| `HandlerExpr` 統一 | per-item 式（`item.foo`）は unified `HandlerExpr` enum に乗る。enum 分割不可 | [M2 handoff §2](../../../milestone-2/handoff.md) |
| 式 grammar の一斉拡張規律 | operators（`!` / 比較 / 論理）は condition-only ではなく全 `expr` 位置に uniform に育てる | [DD-004 forward-compat 3](../../phase-6/decisions/dd-m3-p6-004-conditional-ir-and-runtime-present-absent.md); [dsl-grammar Q5](../../../../docs/notes/dsl-grammar.md) |
| `for` 予約 / sketch | `if`/`else`/`switch`/`for` は §2.1 で予約済み。spec seed に `for item in items { … }` のスケッチ。`in` の予約状態は本 phase で確定 | [DD-004 spec seed](../../phase-6/decisions/dd-m3-p6-004-conditional-ir-and-runtime-present-absent.md); [phase-6 handoff forward-compat 2](../../phase-6/implementation/handoff.md) |
| B1 single-widget-child 先例 | conditional body = single widget child（IG-1）。`for` body の単数/range は open だが IG-1/IG-2 の枠は既存 | [DD-003](../../phase-6/decisions/preamble.md); [DD-004 IG-1](../../phase-6/decisions/dd-m3-p6-004-conditional-ir-and-runtime-present-absent.md) |
| widget id ≠ item key 規律 | item identity / key surface を widget instance id 導入問題と混同しない | [dsl-grammar Q1](../../../../docs/notes/dsl-grammar.md) |
| 接地事実: collection 型が存在しない | `state_type ::= "i32" \| "string" \| "bool"`（[§3 Grammar](../../../../docs/dsl_spec.md) の `state_type` 産生規則 + [§4.7 State declarations](../../../../docs/dsl_spec.md) の supported-types 表）。collection / list 型は無い。collection 変更演算も [§4.5 Signal handler](../../../../docs/dsl_spec.md) / [§4.6 Expressions](../../../../docs/dsl_spec.md)（scalar compound-assign のみ）に**ない** | dsl_spec §3 / §4.5 / §4.6 / §4.7 |

### 1b. 先行 phase が記録した「期待」（決定ではない → §2 で confirm-or-revise）

| 期待 | 記録 | 注意 |
|---|---|---|
| 「Phase 7 = keyed identity を足す」 | [DD-004 forward-compat 2](../../phase-6/decisions/dd-m3-p6-004-conditional-ir-and-runtime-present-absent.md):「Phase 7 adds keyed identity / state retention」 | [constraints §3](./constraints.md) が「positional / keyed / fresh のどれかを**決めなければならない**」と open DD に正しく降格済み。→ **Q2** |
| 「`for` が ID-2 reconciler の first real driver」 | [DD-004](../../phase-6/decisions/dd-m3-p6-004-conditional-ir-and-runtime-present-absent.md) | `for` が何を**必要とするか**の予測。v1 が実際にそれを必要とするかは open |
| 「data-driven reorder は Phase 7 の ordering-contract driver」 | [DD-005 (c) item 2](../../phase-6/decisions/dd-m3-p6-005-conditional-effect-lifecycle-and-drain-contract.md):「a keyed `for` reorder, where present-set order is data-driven, not declared」 | SM-2/SM-3 を Phase 6 が defer した条件付き予測。v1 が reorder を許すかが再点火条件。→ **Q3** |
| 「iteration の M3 target は WrapPanel-backed thumbnail collections」 | [dsl_spec §4.12 deferral](../../../../docs/dsl_spec.md) | E2E target は固いが、**scale の v1 commitment** は未確定。→ **Q4** |
| member emission を canonize しない留保 | [DD-007 Deferred design space](../../phase-6/decisions/dd-m3-p6-007-scrollview-conditional-content-policy.md) | `for` の 0..N emission が「imperative member emission を content model として確定する」方向へ押す。態度決定は §2.2 slate（付録）へ |

---

## 2. 未回答の問いリスト（優先度順）

各問い: **問い**（プロダクト/思想レベル）/ **gate**（どの DD 候補・スコープ・
検証方針を、constraints § 番号と対応づけて）/ **既存文書の部分回答** /
**選択肢空間と含意**（推奨しない）/ **分類**【pre-framing 必須 / packet 可】。

**選択肢の読み方（誘導回避）.** 各問いの選択肢 (i)/(ii)/(iii) の **並び順に
優先順位の含意はない**。最小性（実装が小さい）は評価軸ではない。各選択肢には
「**買うもの**（product merit）」と「**諦めるもの／要求するもの**」を対称に
記す。比較の主軸は product merit であり、実装・改訂コストは選択肢を product
merit で却下する判断の **tie-breaker** に留める（独立軸ではない）。

**推奨回答順序（依存マップ）.** 問いには依存がある。手戻りを避けるための順序:
**Q1**（基底: collection の意味）→ Q1 の答えに応じて **Q7**（変更経路の要否）/
**Q3**（reorder 可否）→ **Q2**（identity; Q3 と結合）→ **Q4 / Q5a / Q5b / Q6**
（scale / doctrine / 露出 / scope）。Q4 (i) は Q1=(ii)/(iii) を前提にする。

### Q1. v1 の「collection」とは何か — 静的反復か、実行時可変か（**最優先・基底**）

- **問い.** v1 イテレーションが回す「collection」は、ソースに書かれた固定長を
  単に展開する**コンパイル時固定の反復**か、それとも cardinality が実行時に
  変わる**reactive な可変 collection**か。これは phase 全体の野心を決める基底
  の分岐。
- **gate.** collection-surface DD（型・初期値・要素型・変更手段）のスコープ全体
  （constraints の thesis bullet「collection 型の露出方法は ADR で確定」）;
  検証方針 [§2.4 / §9](./constraints.md)。
- **部分回答.** 2 つの既存制約がこの分岐に当たる。(1) **凍結済み acceptance A8**
  は「collection **binding** drives widget-tree generation」（[plan.md A8](../../plan.md);
  ROADMAP が SSOT）— binding を介する点が文言に入っている。(2) [constraints §9](./constraints.md)
  の陽性対照は「collection を **変化させて item 数が連動して増減する** 2 frame
  以上」を要求する。両者を前提にすると、純粋に静的な固定長 `for`（hardcode した
  tree と区別不能、binding 不介在）は A8 文言・Phase 7 thesis・§9 陽性対照のいずれ
  とも緊張する。これは検証規律や AC を動かせない制約ではなく、改訂経路が中立に
  存在する（選択肢 (i) 参照）。plan の working assumption は「scalar で足り
  `TypedValue` 不要」。
- **選択肢空間と含意.**
  - (i) **静的反復のみ**（固定 literal collection を展開）— 買うもの: 機構が
    小さく、collection 型・変更経路・identity を v1 で開かない。諦める／要求する
    もの: **凍結済み acceptance A8 の文言「collection *binding* drives
    widget-tree generation」と正面から緊張する** — binding を介さないコンパイル時
    固定展開は A8 と Phase 7 thesis（「cardinality 駆動へ拡張」）が要求する
    性質を v1 で証明しない。整合には (1) A8 を改訂する（[plan.md A8](../../plan.md);
    README acceptance-revision exception 経由 — 中立の手続き注記）か、(2) §9 陽性
    対照を改訂する（constraints.md は status: accepted の §2.1 成果物につき
    workflow §2.1 revisions 規律で理由を残す。陽性対照原理自体は
    [verification-environments.md Obs 4](../../../../docs/notes/verification-environments.md)
    / AGENTS.md に fold 済みの milestone 横断規律）か、いずれかが要る。改訂方向は
    予断しない: A8 を**緩める**（binding 不要化）改訂も、**強める**（変更経路・規模
    要件の AC 昇格）改訂も対称に開いている。
  - (ii) **実行時可変 collection + 最小変更経路**（Q7 の mutation path と連動）—
    買うもの: §9 陽性対照と A8「binding drives」に直接乗る; 実用 UI の collection
    は実行時に変わるという性質を v1 で証明する。要求するもの: collection 型 +
    初期値 + 要素型（→ Q5b）+ 変更手段（→ Q7）の surface を新設。
  - (iii) **full collection 演算**（任意挿入/削除/reorder）— 買うもの: sort /
    filter / 並べ替えまで含む実用 collection UI の表現力。開くもの:
    ordering-contract（§6）・cap（Q4）・identity（Q2）を一気に。
- **分類.** 【pre-framing 必須】。phase の scope ambition そのもの。A8 / §9 の
  どちらをどう動かすかは owner 判断。

### Q2. identity の confirm-or-revise — keyed か、fresh/positional v1 か

- **問い.** 先行記録の「Phase 7 = keyed identity」期待を **confirm** するか、
  明示的に **revise** して v1 を fresh/positional（Phase 6 base case の
  collection 一般化）とし、keyed retention を defer するか。**silent な乖離は
  禁止**（confirm でも revise でも明文化する）。
- **gate.** identity DD [constraints §3](./constraints.md); 検証方針（identity が
  state/effect/disposal/Visual 順とどう相互作用するか）。
- **部分回答.** Phase 6 は fresh-on-return を **un-keyed base case** として出荷
  ([DD-004 ID-1](../../phase-6/decisions/dd-m3-p6-004-conditional-ir-and-runtime-present-absent.md))。
  retention は opt-in 互換形状で予約済み。declared tree が安定 anchor なので
  reconciler は **IR 変更なし**で後付け可能。つまり「keyed を今やる」技術的
  強制力はなく、期待は予測に留まる。
- **選択肢空間と含意.**
  - (i) **fresh/positional v1**（key なし、collection 変化で full rebuild）—
    買うもの: Phase 6 base の素直な一般化、IR 変更なし、最小機構。諦めるもの:
    item ごとの in-progress state / focus / scroll / 入力中の値は absent→present
    で保持されない（M3 placeholder regimen では item は stateless なので v1 の
    痛みは小さいが、実用 list UI では効く）。
  - (ii) **keyed identity v1**（`key:` + retention + reconciler）— 買うもの:
    実用 list UI が要する **item 単位の state / focus / 入力保持**; SwiftUI /
    Flutter / Slint が揃って key/identity を持つのはこの実需による。data 再評価で
    item が「同じもの」として残る（reorder 時の入力欄が消えない等）。開く／要求
    するもの: ID-2 Element-level reconciliation（DD-004 が「no M3 driver の大型
    subsystem」と Phase 6 文脈で評した。その proportionality 判断が本 phase で
    なお当てはまるかは owner 判断）; keys / diffing / Element lifetime; reorder
    （Q3）と結合。
  - (iii) **positional-stable, retention なし** — 中間。買うもの: 位置安定（再
    insert 位置が決定的）。諦めるもの: state 保持はせず。DD-004 が ID-1.5 として
    「Phase 6 では dead weight」と退けた seam を collection 文脈で再評価する形。
- **分類.** 【pre-framing 必須】（confirm-or-revise を明示する義務、silent 乖離
  禁止）。

### Q3. v1 で data-driven reorder を許すか

- **問い.** v1 collection 変化が **item の順序変更**（append/truncate を超えた
  中間挿入・並べ替え）を含むか。reorder を許すと、Phase 6 が defer した
  structural-ordering contract（SM-2/SM-3）の再点火条件に触れる。
- **gate.** reactive-drain residual DD [constraints §6 item 2](./constraints.md)
  （ordering ties）; identity DD [constraints §3](./constraints.md)（reorder は
  key がないと意味を持ちにくい → Q2 と結合）。
- **部分回答.** [DD-005 (c) item 2](../../phase-6/decisions/dd-m3-p6-005-conditional-effect-lifecycle-and-drain-contract.md):
  quiescent child-order invariant は「**declared** 順」で order を固定するが、
  `for` では declared 順が **data 順**になる。「present-set order が data-driven
  になる keyed reorder こそ contracted mutation order の real driver で Phase 7
  に属す」と明記済み（条件付き予測）。
- **選択肢空間と含意.**
  - (i) **append/truncate-only v1**（順序 = collection 順、中間 reorder なし）—
    買うもの: SM-1 status quo を維持できる公算、機構が小さい。諦めるもの:
    並べ替えを伴う操作は v1 で表現不能。
  - (ii) **任意 reorder** — 買うもの: **sort / filter / 並べ替えという collection
    UI の基本操作**を v1 で表現できる。要求するもの: ordering contract（Phase 6
    が defer した SM-2/SM-3 の再評価。drain 順とは独立に観測可能な順序保証）+
    keyed identity（Q2 (ii); reorder は key がないと「同一 item の移動」を
    表せない）。
- **分類.** 【pre-framing 必須】（Q2 identity と §6 drain を結合する分岐）。

### Q4. v1 の iteration はどの規模で「実用」と言えるか（cap は従属変数）

- **問い.** 主問は「v1 iteration が実用と言える **target 規模**は何か」。cap を
  carry できるかは従属変数として扱う（規模 commitment が cap 機構の要否を決める、
  逆ではない）。gallery 規模（[gallery-wireframe.html](../../requirements/gallery-wireframe.html)
  の status 行 "218 photos"、grid 可視 ~18 枚）を v1 commitment とするか、v1 は
  機構を小 N で証明し大 N を documented backstop に留めるか。
- **gate.** fan-out × `MUTATION_CAP` [constraints §6 item 3](./constraints.md)
  （Phase 7 直撃と明記）; structural failure observability
  [constraints §5](./constraints.md); 検証方針。
- **部分回答.** [DD-005 (b)/(c)](../../phase-6/decisions/dd-m3-p6-005-conditional-effect-lifecycle-and-drain-contract.md):
  現 `MUTATION_CAP` = 16。N child 生成は N effect を current drain に fan-out
  させ、cap 超過は既存 divergence guard で surface。SM-4 separate insertion
  budget は Phase 6 で declined（`for`-era requirement を当て推量しないため）。
  **接地懸念（要確認）.** DD-005 は lightbox を「N ≪ 16」と*比較*しているが、
  数十枚の thumbnail 生成は **16 と最初から同オーダー**にある。cap 周りは
  「pathological case の carry」ではなく Phase 7 の target 自体が触りうる。したがって
  **cap の正確な機構**（cap が数えるのは drain 反復数か effect 数か; N item 一括
  挿入が 1 mutation でどう数えられるか）の検証を framing / DD の **必須確認事項**
  とする（数値が分かるまで (i)/(ii) の実コストは確定しない）。
- **選択肢空間と含意.**
  - (i) **gallery-scale を v1 commitment** — 買うもの: 実用 photo gallery として
    通る規模。要求するもの: cap 成長 / SM-4 budget の再評価、性能・収束保証を本
    phase の設計対象に（Q1=(ii)/(iii) を前提）。
  - (ii) **小 N で機構証明、cap は carry** — 買うもの: 機構の正しさを最小コストで
    証明（§9 は枚数でなく「変化」を要求）。諦めるもの: 大 N は divergence
    backstop 任せで、実用規模の収束は v1 では未保証。cap interaction は named
    re-ignition point として次へ。
- **分類.** 【packet 可】（推奨つき FD として provisional 化しうる。ただし cap
  機構の検証結果次第で pre-framing へ昇格しうる — 上記接地懸念参照）。

### Q5a. `item` / `index` は「全参照 state 経由」主義の例外か（【pre-framing 必須】）

- **問い.** iteration は `item` / `index` を、式から解決可能な **初の非 state 名**
  として持ち込む。オーナーは `item`/`index` を「全参照を state 経由に揃える」
  主義（dsl-grammar Q1）への **限定的例外**（loop-local read-only binding）として
  受け入れるか。さらに `item` の可読性を **式位置と handler 位置で分けるか**
  （例: binding 式では読めるが handler 本体では読めない／両方で読める）— これは
  Q5b の露出型とは別の admission 判断。
- **gate.** per-item context DD（constraints thesis bullet）; 名前解決規律（Q6）。
- **部分回答.** [M2 handoff §2](../../../milestone-2/handoff.md): `item.foo` は
  unified `HandlerExpr` に乗る（premise; enum 分割不可）。`HandlerExpr` は binding
  式（read-only subset）と handler 本体（read/write）の両モードで評価されるため、
  `item` の可読性をモード別に絞るかは EvalContext 側の admission。[dsl-grammar Q1](../../../../docs/notes/dsl-grammar.md):
  「全参照を state 経由に揃える（Elm/SwiftUI 的 UI=state の純関数）」は魅力ある
  未決方針。
- **選択肢空間と含意.**
  - (i) **`item`/`index` は state 経由主義の限定例外**（loop-local read-only）—
    買うもの: per-item 表現が直截。諦めるもの: 「UI=state の純関数」の純度が
    一段下がる（loop-local binding という第二の名前種を導入）。handler 位置でも
    読めるかは別途決める。
  - (ii) **handler 位置では `item` 不可、式位置のみ**（中間）— 将来の
    select-this-item 等を v1 では開かない（Phase 6 FD-D 型の「問いを閉じない」
    defer も可）。
- **分類.** 【pre-framing 必須】（doctrine。state 経由主義の境界を引く）。

### Q5b. `item` が露出するのは scalar か fields か（【packet 可】）

- **問い.** `item` が露出するのは **opaque scalar**（String/i32）か、**fields**
  （`item.filename` 等）か。後者は collection 要素型を複合にし `TypedValue` 圧力を
  生む。（問うのは露出の意味論であって enum 形状ではない。）
- **gate.** `TypedValue` DD [constraints §7](./constraints.md); collection 要素型
  （Q1 の collection-surface DD）。
- **部分回答.** M3 placeholder regimen では thumbnail は Box+Text で、caption
  （IMG_0237 / date / dims、[gallery-wireframe.html](../../requirements/gallery-wireframe.html)
  lightbox state）は lightbox 側 = 選択 item の fields を要する。
- **選択肢空間と含意.**
  - (i) **scalar item のみ**（String/i32）— 買うもの: `TypedValue` 圧力 surface
    せず plan working assumption を維持。諦めるもの: caption fields は item から
    直接表現不能（placeholder regimen が thumbnail を Box+Text に留めるため v1
    E2E は回避可能）。
  - (ii) **structured item（fields）** — 買うもの: `item.filename` 等の実用的な
    per-item データアクセス。開くもの: collection 要素型が複合になり `TypedValue`
    圧力が surface（plan の acceptance-revision exception 経由でしか入らない）。
- **分類.** 【packet 可】（placeholder regimen を背景に scalar-only 案を packet で
  扱える。どちらを採るかは owner 判断）。

### Q6. per-item scope — shadowing / nesting / 衝突規則

- **問い.** `item` / `index` が state 名と衝突したとき、`for` が `for` の内側に
  nest したときの名前解決規則は何か。shadowing 許容 / error / scoped のどれか。
  nesting を v1 で許すか。
- **gate.** per-item context DD（constraints thesis bullet）; 名前解決
  （[dsl-grammar Q1](../../../../docs/notes/dsl-grammar.md) の未決「名前解決スコープ
  — component-local フラットか、ネスト可能か」）。
- **部分回答.** Phase 6 は sibling + descendant conditional を in-scope とし、
  bare nested `if`-as-immediate-body のみ defer（[DD-003 B1](../../phase-6/decisions/preamble.md)）。
  for-in-for は iteration analog。現 resolver は `HashMap<&str, IrType>` で名前→型
  を引く（[DD-004 loader 拡張](../../phase-6/decisions/dd-m3-p6-004-conditional-ir-and-runtime-present-absent.md)）。
- **選択肢空間と含意.**
  - (i) **flat / 衝突 = error / v1 nesting なし** — 最小。`item` 名固定、state 名
    と衝突を reject。
  - (ii) **lexical scope + shadowing + nesting 許容** — 表現力高いが、scope 解決を
    resolver に持ち込む（inner `item` が outer を shadow）。
- **分類.** 【packet 可】（推奨つき provisional FD。ただしオーナーが「全参照
  state 経由」主義との関係で doctrine を問いたい場合は Q5a と束ねて pre-framing）。

### Q7. E2E 達成証拠の形 — 陽性対照の mutation 経路

- **問い.** §9 陽性対照は item 数が 2 frame 間で変化することを要求する。E2E で
  collection が実行時に変わる **authored 経路**は何か。collection を変更する
  Button-click handler（DSL handler の collection 演算が要る）か、host-side ABI
  mutation か、test-only seam か。陽性対照は「変更経路が存在する」ことを前提に
  する（Q1 / (d+g) と連動）。
- **gate.** collection-surface DD の **変更手段** arm（DSL handler 演算 / host ABI /
  test seam）; 検証方針 [§2.4](./constraints.md)。
- **部分回答.** Phase 6 の陽性対照は Button-click → bool toggle（event handler →
  state → conditional subtree）。iteration analog は Button-click →
  collection-mutate。だが現 handler grammar は scalar compound-assign（i32 の
  `+=` 等）のみで、**collection 変更演算は存在しない**
  （[dsl_spec §4.5 Signal handler](../../../../docs/dsl_spec.md) /
  [§4.6 Expressions](../../../../docs/dsl_spec.md)）。
- **選択肢空間と含意.**
  - (i) **最小 DSL handler collection 演算を新設**（append/remove 相当）— 陽性
    対照を `.ui` 内で自己完結。handler grammar 拡張 = 本 phase scope 増。
  - (ii) **host-ABI 駆動 mutation** — `.ui` を汚さず host から collection を変える。
    ABI surface（M3 は ABI freeze せずだが新 export 是非）を開く。
  - (iii) **2 静的 collection を bool で切替**（mutation 回避）— 既存機構のみ。
    だが「cardinality が data から駆動される」証明力が弱く、§9 陽性対照の趣旨を
    満たすか微妙（Q1 (i) と同じ弱さ）。
- **分類.** 【pre-framing 必須】（handler/ABI mutation surface を新設するか否かは
  大きな scope 判断。Q1 と結合）。

---

## 3. 独立導出 → 種リスト突合（増減を理由つきで報告）

owner プロンプトの種リスト (a)–(f/g) に対し、上記 §2 の独立導出を突合する。
**結論: 種の和集合から落ちた項目はゼロ。owner 問いレベルの新規追加もゼロ
（設計レベルでは 1 件追加 — `for` body 内 handler の admission / handler 内
`item` 可読性。付録 8）。** ただし product-fork の線で **再分割**した（mutation の
有無は A8/§9 が押す分岐なので reorder 可否から切り出すべき、等）。

| 種 | §2 での扱い | 増減と理由 |
|---|---|---|
| **(a)** collection 変化モデル（特に v1 で data-driven reorder を許すか、DD-005 item 2 再点火条件との対応） | **Q1 + Q3** に分割 | **分割**。現行 §9 陽性対照は「実行時に cardinality が変わる経路」を押す（Q1）が、reorder までは押さない（Q3）。両者は gate する DD が異なる（Q1=collection-surface 全体 / Q3=§6 ordering + §3 identity）ため product-fork で切り出した。DD-005 item 2 の再点火条件は Q3 に明示対応 |
| **(b)** identity（「Phase 7=keyed」期待を confirm するか、明示 revise して fresh/positional v1 とするか。silent 乖離禁止） | **Q2** | **一致**。期待→confirm-or-revise への変換を保持 |
| **(c)** scale 期待（gallery 規模を v1 commitment とし cap を carry するか） | **Q4** | **一致** |
| **(d+g)** collection surface 一式（型・初期値・要素型 TypedValue 圧力・変更手段。陽性対照が変更経路を前提にする点） | **Q1（型・実行時可変）/ Q5b（要素型・TypedValue）/ Q7（変更手段）** に分散 | **分散（落とさず）**。monolithic な「collection surface」1 問にせず、product-level fork ごとに gate 先の異なる 3 問へ展開。陽性対照が変更経路を前提にする点は Q7 と Q1 に明示。初期値の構文は §2.2 slate（付録）へ送る機構寄り項目 |
| **(e)** per-item context の名前解決・スコープ（衝突・shadowing・ネスト、全参照 state 経由主義との関係。HandlerExpr 搭載は premise） | **Q5a（主義・式/handler 位置）+ Q5b（露出型）+ Q6（shadowing/nesting/衝突）** に分割 | **分割**。doctrine（state 経由主義の例外可否 = Q5a, pre-framing 必須）/ 露出型（Q5b, packet 可）/ mechanics（衝突規則 = Q6, packet 可）は分類が異なるため切り分け。HandlerExpr 搭載 premise は再審議せず明記。**設計レベルで 1 件追加**: `for` body 内 handler の admission と handler 内 `item` 可読性（下記 §3 末尾・付録 8 参照） |
| **(f)** E2E 達成証拠の形（陽性対照の見せ方、(d+g) 変更経路の選択と連動） | **Q7** | **一致**。mutation-path との連動を明示し、Phase 6 の bool-toggle analog として接地 |

**独立導出が種の外に出さなかったことの確認.** いくつか誘惑された候補
（per-container の direct-`for` 許容、collection 空状態と DD-007 reject の干渉、
body 形状の単数/range）は **owner 問いに昇格させず**付録の §2.2 slate 候補に
留めた（プロンプトの「避けること」に従う）。これは種リストが well-formed で
あったことの傍証であり、独立導出は問いの**再分割**（mutation 有無を §9 強制から
切り出す等）に価値を出した。

---

## 付録: owner 問いに昇格しないが §2.2 slate に入るべき設計論点候補

以下は **オーナー意図の問いではなく**、framing thesis 確定後に §2.2 DD slate で
扱う設計レベル論点。ここでは列挙に留める（態度決定はしない）。

1. **per-container × `for` 許容スイープ.** WrapPanel / ScrollView / Grid / Box /
   ZStack のそれぞれで direct-`for` を許すか、wrapper を要求するか。Phase 6 が
   conditional × 各 container cardinality で行った sweep
   （[DD-007](../../phase-6/decisions/dd-m3-p6-007-scrollview-conditional-content-policy.md)、
   Cell/ScrollView=cardinality reject / Grid=structural reject / Box=tolerate）の
   iteration 版。E2E target は WrapPanel-backed なので WrapPanel direct-`for` が
   中心。
2. **DD-007 留保への態度.** member emission を content model として canonize するか。
   `for` の 0..N emission は「imperative member emission」方向へ押す
   （[DD-007 Deferred design space](../../phase-6/decisions/dd-m3-p6-007-scrollview-conditional-content-policy.md)
   の 3 base model のうち 1 つ）。空 collection（0 child）と DD-007 の
   conditionally-empty container 留保の干渉もここ。
3. **body 形状（single vs range, IG-1 → IG-2）.** `for` body が single widget child
   か `structural_member*` か。range insert/remove machinery（DD-004 IG-2）は
   `for` が driver と名指しされている。
4. **collection 式の語彙・位置.** `for item in items` の `items` 式が取りうる位置
   （state read のみか、式か）、`in` の予約、collection literal / 初期値の構文形。
5. **`ForLoopSubtree` の slot bookkeeping.** conditional の recompute-not-cache
   index 規律（DD-004）を range（N child、preceding live range 数で base index を
   再計算）へ一般化する機構。
6. **placement 格納モデル（SoA/AoS/keyed map）.** range mutation が育つ前の Phase 7
   決定（[constraints §4](./constraints.md)、handoff が "Phase 7 decision" と名指し）。
   機構レベルにつき §2.2 DD の領分。
7. **architectural-family トリガー発火の扱い.** M3 DSL grammar は最も committal な
   family-level 選択で、iteration grammar はその核
   （[architectural-family 再評価トリガー 1/3](../../../../docs/notes/architectural-family.md)）。
   tree-with-bindings family 内で収まるか、VDR 昇格を要するかは framing で扱う
   メタ判断。
8. **`for` body 内 handler の admission.** Phase 6 conditional body は handler 持ち
   widget（lightbox の `x` Button）を内包した。`for` body も既定で N item × N
   handler を持ちうる。handler を `for` body に admit するか、handler 本体から
   `item` を参照できるか（将来の select-this-item 等）は Q5a の式露出とは **別の
   admission 判断**で、spec が明示しないと A12 external-reader bar に響く。owner の
   実用性志向の下では v1 で実装しなくても **設計評価には載せる**（Phase 6 FD-D の
   「問いを閉じない」型）価値がある。Q5a に式位置/handler 位置の分離問いとして
   1 本足済み; ここは body admission（handler を body に許すか）側の機構論点。

---

> **回答の蒸留先.** 本文書で得たオーナー回答の**耐久部分**は
> [docs/notes/dsl-grammar.md](../../../../docs/notes/dsl-grammar.md) の新 Q
> または Q3 / Q6 への追記として蒸留し、framing.md がそれを参照する（本作業
> 文書は framing 入力であって SSOT ではない）。
