---
title: M3-Phase 6 framing — ZStack + 条件レンダリング
status: draft
created: 2026-05-31
target-phase: M3-Phase 6
---

# M3-Phase 6 framing

**Status:** draft; pending owner alignment
**Targets phase:** M3-Phase 6 (ZStack primitive + conditional rendering grammar)

プロジェクトの開発プロセス（[workflow.md §2](../../../procedures/workflow.md)）に
従い、本 note は設計判断（ADR）を書く前に **owner とフレーミングを合意**する
ための入力資料。個別の設計判断の中身（案の比較・推奨）は本 note では負わない
（それは §3 設計判断）。ここで確定するのは **設計判断の論点一覧・スコープ・
検証方針** と、それらを貫く **オーナー合意事項（FD-\*）** である。冒頭の
「今回オーナーに決めてほしいこと」が判断用の入口、後続の各節がその根拠。
先行 M3 phase から本 framing が継承する規律（再導出しない）:

- **Two-moment spec-sync**（Moment 1 = ADR-Accepted commit での design-spec
  draft、Moment 2 = phase close での implementation re-sync）。
  [m3-phase-2 framing decision D](../../phase-2/requirements/framing.md) 由来。
- **Moment is not a commit unit / review-concern 単位 commit**
  （[CLAUDE.md §Commit rules](../../../../CLAUDE.md)）。
- **No fast-track**: 全 merge は owner 明示承認。
- **Final-task retrospective split**: 最終 task の task-end retro と phase-end
  retro を最初から別 bullet にする（[constraints.md §6](./constraints.md)）。
- **制約引き継ぎ**: 本 framing は
  [constraints.md](./constraints.md)（§2.1 アウトプット、accepted）を前提と
  して読む。R1 採用 / DPI 不採用 / 陽性対照 / reactive-drain 義務はそこで確定済み。

---

## 今回オーナーに決めてほしいこと（Owner alignment packet）

**この節だけ読めば合意判断できる。** 推奨でよければ「OK」、変えたい項目だけ
指示してください。詳細な根拠は後続の各節（リンク先）に置く。

| ID | 決めてほしいこと | 推奨 | 詳細 |
|---|---|---|---|
| ⓪ | **条件レンダリングの思想**（画面構造を状態でどう変えるか）| v1 は「テンプレート + 独自構文型」を採用。将来の「言語構文型（`if`/`switch`/loop をそのまま書く自由度）」を塞がない。「常に作っておいて表示/非表示で切替」は中心にしない。条件・分岐・繰り返しを**一つの「構造を変える制御構文ファミリー」の第一歩**として設計する | [FD-CR](#fd-cr-条件レンダリングの思想最重要cross-cutting) |
| ① | **画面で見せる達成証拠**（何を見せれば Phase 6 達成か）| lightbox を thumbnail 一覧の上に重ねて出す。**普通の text Button** click で `is_lightbox_open` を切替え、出した時／消した時の**2 枚**で証明 | [FD-B](#fd-b-画面で見せる達成証拠--lightbox) |
| ② | **半透明 scrim を使うか**（写真の背後の膜）| **半透明を使う**。既存の `fill: #RRGGBBAA` で表現でき追加コストなし（解決済み・確認のみ）| [FD-G](#fd-g-scrim-は既存-rrggbbaa-で半透明可新-alpha-surface-は不要) |
| ③ | **状態で変わる Window title を「評価対象」に載せるか** | 静的 title は必須で確定。動的（状態で変わる）title は**比較はするが実装は約束しない**（結論 defer 可）。問いを先に閉じない | [FD-D](#fd-d-window-title-の扱い) |
| ④ | **消えた subtree の中の binding の寿命**と **toggle 直後に画面状態を観測できる約束** | subtree が消えている間その中の binding をどう扱うか（止める/捨てる/再出現時に作り直す）の**最小ルールを Phase 6 で決める**。「切替えた直後に画面状態を観測できる」既存の約束は保つ | [FD-E](#fd-e-条件-subtree-の-binding-寿命と-toggle-直後の観測) |
| ⑤ | **ZStack と条件レンダリングを一体で出荷するか** | **一体**。lightbox が両方を一組で要求し、「重ね合わせ（layout）＋出し分け（grammar）が実用画面で噛み合う」ことを 1 枚で示せる | [FD-F](#fd-f-zstack--条件レンダリングを-unit-として出荷) |

**返事チェックリスト（これだけ埋めれば確定）:**

- ⓪ 条件レンダリングの思想: ☐ OK ／ ☐ 修正 → ____
- ① 画面で見せる達成証拠（lightbox・text Button 切替・2 枚）: ☐ OK ／ ☐ 修正 → ____
- ② 半透明 scrim 採用: ☐ OK ／ ☐ opaque で可
- ③ 動的 title を評価対象に: ☐ 載せる（推奨）／ ☐ 静的のみで閉じる
- ④ binding 寿命の最小ルールを Phase 6 で決める: ☐ 決める（推奨）／ ☐ carry-forward
- ⑤ ZStack + 条件を一体出荷: ☐ OK ／ ☐ 分割

> **合意不要（私が継承で進める機械的事項）:** FD-A（論点 6 件の過不足）/
> FD-C（検証方針）/ FD-H（上流文書の反映タイミング）/ FD-I（最終 task の
> retro 分割）。これらは先行 phase からの継承で、オーナー判断を要しない。

---

## オーナー合意の記録（Owner alignment outcome）

*（合意後にここへ ⓪〜⑤ の確定結果を記録し、status を `framing aligned` に
更新する。未記入＝未合意。）*

---

## Phase 6 acceptance criteria (restated)

SSOT は [process/_roadmap.md M3](../../../_roadmap.md) /
[plan.md §Acceptance criteria](../../plan.md)。本 phase が負う AC:

- **A4 — ZStack layout primitive.**
  > sibling z-order by document order.

  load-bearing な部分は **(i) 兄弟要素を重ねる measure/arrange**（各子が
  同じ重なり領域を占める）と **(ii) document order = paint order（Z 順）**。
  Phase 5 で「same-cell overlap は Grid ではなく ZStack の責務」と境界を
  引いた、その overlay 専管 primitive がここで出荷される。

- **A7 — conditional rendering grammar.**
  > binding drives the present / absent state of a subtree.

  **M3 初の grammar surface**。M2/M3-Phase 1-5 の binding は property 値の
  駆動に限られていた。本 AC は binding が **widget-tree の構造（subtree の
  存否）** を駆動する文法を DSL に入れる。台座の `bool` scalar は M3-Phase 1
  で landed 済み。

- **A11（operational obligation）.** `.ui` / `wasamo-ir` / `wasamoc` /
  `wasamo-runtime` / `docs/dsl_spec.md` / `examples/gallery/` の sub-screen が
  本 phase 内で同時に前進する。片側だけ先行させない。本 phase の gallery
  visible proof は **lightbox**（下記 FD-B）。

- **A12（DSL spec 漸進 draft obligation）.** ZStack と条件レンダリング grammar の
  normative section を `docs/dsl_spec.md` に **本 phase 内で** 追加する
  （Moment 1 design draft → Moment 2 close）。条件レンダリングは
  [target-app pre-doc](../../requirements/spec.md) が「M4 syntax reservation で
  済ませず M3 で public spec として normative に書く」と明示した surface。
  external-reader bar（spec だけで再現可能か）を phase close で適用する。
  **条件レンダリング grammar section は単発の構文解説ではなく、「Wasamo の
  structural rendering model」の最初の章として書く**（下記設計 thesis）:
  外部読者が「`if` の次に `else` / `switch` / `for` 相当が同じ文法
  ファミリーとして自然に来る」と理解できる水準を bar とする。

---

## 条件レンダリングの設計 thesis（Wasamo structural rendering model）

**owner の核心意向**（[docs/notes/dsl-grammar.md Q6](../../../../docs/notes/dsl-grammar.md)）。
条件レンダリングは「表示する / 隠す」の小機能ではなく、**UI DSL がどこまで
画面構造を状態に従わせられるかを決める中核文法**。Phase 6 はこれを単発機能
ではなく **structural control-flow grammar の第一歩**として定義する。

条件レンダリングには 3 つの思想があり、Wasamo の立場を固定する:

1. **プロパティ制御型** — tree を常に作っておき `visible`/`enabled` 等の
   property で命令的に出し分ける。**これは Phase 6 conditional rendering の
   中心案ではない**。Wasamo が証明したいのは property toggling ではなく
   **tree shape を宣言的に変えること**。
2. **テンプレート + 独自構文・属性型**（`if` 風 block / `when:` 風 attribute /
   構造的 directive）— **Wasamo v1 はこれを採用**。独立 `.ui` DSL を持つため
   自然な中心で、compiler/runtime が条件 subtree の範囲・依存 state・widget/
   effect の寿命を把握しやすく、public spec として説明しやすい。
3. **言語構文型 embedded DSL**（host 言語の `if`/`switch`/loop をそのまま
   使う）— 自由度は最大だが `.ui` の独立性・言語横断性を弱める。**v1 では
   採らないが、runtime/IR/grammar が将来この自由度へ拡張するのを妨げない**。

**Phase 6 が守る方向性（Q6 由来、設計スコープの requirement）:**

- subtree の存否を、表示 property ではなく **構造的な present / absent** として扱う。
- 条件レンダリング・複数分岐・繰り返し生成を、ばらばらの特殊機能ではなく
  **構造的制御構文ファミリー**として設計する（Phase 7 iteration は同じ
  family の別要素、将来 `else` / `switch` も同 family）。
- 初期の構文・IR・runtime が、将来 `else` / `switch` / loop へ広げるときに
  邪魔にならない形にする（場当たり専用 node にしない）。
- 条件 subtree 内の binding/effect の **寿命を曖昧にしない**（absent 時に
  effect は存在するのか / 止まるのか / 破棄か、再出現時に再利用か再生成か）
  → DD-005 で明文化。

**runtime identity（実体 tree の同一性・寿命管理）の観点（Q6「ランタイム
設計への含意」由来）:** この thesis は文法だけの問題ではなく
**runtime identity の問題**でもある。Q6 が参照点とする
Flutter の Widget / Element / RenderObject 分離のように、**軽量な「宣言 tree」
（状態変化で再生成されてよい）と、寿命を持つ「実体 tree」（state / effect /
layout 実体 / focus / 入力中の値などを identity で扱う）を分離**して考える。
v1 の表面構文がテンプレート + 独自構文型であっても、runtime 内部でこの分離を
保てば、将来 `.ui` 由来でも言語内 DSL 由来でも同じ runtime 機構に落とせる
余地が残る。**この分離を DD-004 の設計評価軸にする**（packet には出さない、
owner 判断でなく設計判断）。

**実装スコープと設計スコープの分離:** Phase 6 の **実装**は `if <bool>` の
最小単位で足りる（`else` / `switch` / loop を実装しない）。だが **設計
判断**（DD-003 grammar / DD-004 IR・runtime）は、上記 family への拡張可能性と
宣言 tree / 実体 tree 分離を評価軸に含める。実装を欲張らず、設計面積だけ広げる。

---

## 論点 slate（DD questions — 番号予約のみ）

本 phase の ADR set（`decisions/preamble.md` + DD ごとに 1 ファイル）が
担う論点を列挙し、**DD-M3-P6-NNN 番号を予約**する。各 DD の options /
比較 / 推奨は §3 設計判断で書く。ここでは「何を判断するか（問い）」と
「なぜ Phase 6 の問いか」だけを固定する。

### DD-M3-P6-001 — ZStack IR node 形と author-facing surface
**問い:** ZStack を `wasamo-ir` / `wasamo-runtime` にどの IR node 形で導入し、
`.ui` でどう書くか（Box / Grid / WrapPanel / ScrollView と並ぶ per-kind tag、
子は直接 child か）。
**Phase 6 の問いである理由:** 新 layout primitive の catalog 追加であり、
M3 primitive の per-kind tag パターンに乗るかを確定する必要がある。
**sub-issues（中身は §3）:** IR node tag、子の保持形、author surface の最小形。

### DD-M3-P6-002 — ZStack measure / arrange + z-order + clip 契約
**問い:** ZStack が自身のサイズをどう測り（子の union か、親割当 Fill か）、
各子をどこに配置するか（各子が ZStack 全域を占める / 子ごとの alignment）、
paint 順（document order = z-order）と outer-bounds clip の扱い。
**Phase 6 の問いである理由:** overlay の意味論そのもの。Phase 5 Grid で
確立した「document order = paint order」「outer-bounds clip in scope /
per-child clip out」の先例と整合させる判断。
**sub-issues（中身は §3）:** sizing policy、per-child alignment、clip 範囲。

### DD-M3-P6-003 — 条件レンダリングの author-facing grammar surface
**問い:** 「binding が真のとき subtree が present」を `.ui` でどの構文で
書くか。設計 thesis に従い、まず **3 大アプローチ（property 制御型 /
template structural directive 型 / language 構文型）を明示比較**し、
**Phase 6 は approach 2 を採るが approach 3 を塞がない**という評価軸で
構文形を選ぶ。approach 2 内の具体形（`if` 風 block / `when:` 風 attribute /
構造的 directive）はその下位比較。
**Phase 6 の問いである理由:** property 駆動を超えて構造駆動に踏み込む
**M3 初の grammar surface**。M4 reservation せず M3 で normative 化する
owner-agreed 方針（pre-doc）と、structural control-flow family の第一歩と
いう設計 thesis に直結。
**sub-issues（中身は §3）:**
- **grammar family 視点:** `if` 単発でなく、将来の `else` / `else if` /
  `switch` / `for`（Phase 7 iteration）と同じ文法ファミリーに育つ形か。
- **条件の式位置 grammar（[dsl-grammar.md Q5](../../../../docs/notes/dsl-grammar.md) 再訪点）:**
  条件式に bare bool identifier/literal だけを認めるか、`!ready` / comparison /
  logical operator まで本 phase で開くか（expression grammar 拡張の射程）。
- **type/shape validation and diagnostics:** 非 bool 条件の拒否、構文位置の
  制約、`docs/dsl_spec.md` の invalid examples。何を拒否し何を許すかを本
  phase で確定しないと A12 public draft に耐えない。
- binding 参照の書き方、構文形の最小単位。

### DD-M3-P6-004 — 条件レンダリングの IR 表現 + runtime present/absent 機構
**問い:** 条件 subtree を `wasamo-ir` でどう符号化し、`wasamo-runtime` が
bool 変化時に subtree をどう insert/remove するか（Visual layer / WidgetNode
への反映機構）。**設計 thesis の眼目（2 軸）:**
- **(i) control-flow family 拡張性:** `ConditionalSubtree` 専用の場当たり
  IR にすると Phase 7 iteration / 将来 `switch` で詰まる。**「structural
  control-flow node family（`if` / `for` / 将来 `switch`）に拡張できる IR/
  runtime か」を評価軸**にする。
- **(ii) runtime identity / 宣言 tree と実体 tree の分離（Q6「ランタイム
  設計への含意」由来）:** 軽量な「宣言 tree」（状態変化で再生成されてよい）と、
  寿命を持つ「実体 tree」（state / effect / layout 実体 / focus / 入力中の値を
  identity で扱う）を runtime 内部で分離できる設計か。Flutter の
  Widget / Element / RenderObject 分離が参照点。

実装は `if bool` だけでよいが、設計は上記 2 軸で評価する。
**Phase 6 の問いである理由:** grammar surface の runtime 実体。binding が
widget-tree shape を駆動する初の経路で、reactive 機構と接続する。Phase 7
iteration が同じ family に乗るか、将来の言語内 DSL が同じ runtime に落とせるか
は、ここの IR/runtime 設計で決まる。
**sub-issues（中身は §3）:** IR node 形（control-flow family 拡張性）、
**subtree identity（宣言 tree vs 寿命を持つ実体 tree の分離）**、
present/absent の runtime 操作、**state / effect / layout 実体の寿命を
表面構文から独立して扱えるか**、**IR loader / runtime が拒否すべき形の
diagnostics**（DD-003 の grammar-level validation の runtime/loader 側対応）。
**DD-M3-P6-005 と密結合**（effect lifetime は DD-005、実体 tree の identity 機構は
DD-004）。

### DD-M3-P6-005 — 条件 subtree の effect lifecycle + reactive-drain proof 契約
**問い:** 条件 subtree の present/absent が reactive 機構とどう接続するか。
2 つの面を持つ:
- **(a) effect lifecycle policy（Phase 6 で明文化）:** 条件 subtree の内側に
  binding/effect がある場合、subtree が absent のとき effect をどう扱うか
  （disabled/disconnected とみなす / subtree recreate 時に再登録する 等）。
  conditional rendering が作る lifecycle 境界を Phase 6 で **最小 policy
  として決める**（cycle/tie/fan-out の全面解決ではないが、境界の明文化は
  本 phase の核心）。設計 thesis（[dsl-grammar.md Q6](../../../../docs/notes/dsl-grammar.md)）
  が明示する「条件 subtree 内 binding/effect の寿命を曖昧にしない」要求に
  直結。
- **(b) drain proof contract:** [M2 handoff §3](../../../milestone-2/handoff.md)
  **item 4**（`BATCH_DEPTH==0` で write が drain を完了させる、M3-Phase 1
  T13 が依拠した observable proof contract）を条件レンダリングが直撃する:
  **bool を toggle した直後に host/test が subtree presence をいつ観測
  できるか** を保つか、観測境界を明示 revise するか。items 1-3
  （cycle / ties / fan-out）は (a) の lifecycle policy が触れるかを判断し、
  fix または carry-forward を明示記録する。
**Phase 6 の問いである理由:** plan §Risks と M2 handoff が conditional
rendering を名指しで「直撃」と書いた義務。effect lifecycle は条件
レンダリングの核心であり silent carry-forward 不可。
**sub-issues（中身は §3）:** absent subtree の effect 扱い、recreate 時の
再登録、item 4 の契約保持 vs 境界 revise、items 1-3 の fix/carry 判断。

### DD-M3-P6-006 — Window-title host-wiring (R1) surface
**問い:** component-level `title:` を native window に適用する host-wiring を
どの経路で実装するか。**静的 title の host-wiring は必須達成条件**
（[constraints.md §1](./constraints.md) の R1 解決条件）。加えて、本 phase は
binding が property → tree shape へ広がる phase なので、**`String` binding
駆動の動的 title を評価対象として比較**する（結論が defer でも、問いは閉じ
ずに DD で比較する）。
**Phase 6 の問いである理由:** R1 owning phase = Phase 6（Phase 5 FD-E）。
静的 host-wiring が必須要件である一方、動的 title は本 phase の grammar
拡張テーマ（binding → 構造/属性）と地続きの評価軸。
**sub-issues（中身は §3）:** host 経路、静的（必須）の実装、動的 title の
評価（採否・defer 理由）。

---

## Phase 6 スコープ

### In scope

- **ZStack** IR node（per-kind tag）+ author surface（DD-001）。
- ZStack の **重なり measure/arrange + document-order z-order + outer-bounds
  clip 契約**（DD-002）。
- **条件レンダリング grammar surface**（DD-003）と、その **IR 表現 +
  runtime present/absent 機構**（DD-004）。
- 条件 subtree の **effect lifecycle policy の明文化 + reactive-drain proof
  契約の処置**（DD-005、FD-CR の「寿命を曖昧にしない」要求の実体）。
- **R1 Window-title host-wiring**（静的 title が native window に乗る、DD-006）。
- **動的 Window title の *評価*（実装コミットはしない）**（DD-006、FD-D）。
- A11 gallery visible proof = **lightbox slice**（FD-B）。
- A12 `docs/dsl_spec.md` への ZStack section + 条件レンダリング grammar
  section の design-spec draft（Moment 1）。

### Out of scope

- **繰り返し生成 grammar の実装** — Phase 7。lightbox は単一 subtree の
  toggle で足り、collection driven な生成を要しない。**ただし Phase 6 の
  conditional grammar は Phase 7 iteration と独立ではなく、同じ structural
  grammar family の最初の要素として設計する**（設計 thesis）。Phase 6 が
  iteration を *実装* しないだけで、grammar/IR/runtime の設計判断は family
  拡張性を評価軸に持つ。`else` / `switch` も同 family の将来要素。
- **Button selected state surface** — Phase 8（A10）。tab strip / 選択
  thumbnail の selected styling は本 phase で開けない。
- **scrim の alpha *styling controls*（theming / named palette / dynamic
  alpha）** — M3 out of scope（pre-doc）。ただし **半透明 scrim 自体は既存
  `fill: #RRGGBBAA` literal で表現可能**で in scope（FD-G、dsl_spec §4.9 が
  scrim use case を名指しで admit 済み）。
- **lightbox の close/prev/next の swipe / pinch / keyboard ジェスチャ** —
  M4 input。close/nav は Button click handler の binding で表現
  （ただし nav の実 photo 切替の中身は M3 placeholder 範囲）。
- **hit-testing / focus capture / modal focus trap** — M4。lightbox は
  構造上 modal-ish だが focus model には踏み込まない。
- **explicit z-index / author-facing layering 属性** — paint 順は document
  order 固定（DD-002）。layering 属性は出さない。
- **動的 (binding 駆動) Window title の *実装コミット*** — 本 phase では
  確定しない。**評価そのものは DD-006 で in scope**（静的は必須、動的は比較）
  だが、動的を実装するという約束は本 phase でしない（採否/defer は ADR
  判断、FD-D）。
- **per-monitor DPI awareness（runtime）** — M4（[constraints.md §5](./constraints.md)）。
- **Image widget surface** — M4。lightbox photo は Box(aspect 4:3) + Text
  placeholder。

### Acceptance mapping

| AC | In-scope realization |
|---|---|
| A4 | DD-001 / DD-002 — ZStack IR node + 重なり measure/arrange + document-order z-order + outer-bounds clip |
| A7 | DD-003 / DD-004 — 条件レンダリング grammar surface + IR 表現 + runtime present/absent 機構（DD-005 で drain 契約を確定） |
| A11 | ZStack + 条件レンダリングが `.ui`/`wasamo-ir`/`wasamoc`/`wasamo-runtime`/`docs/dsl_spec.md` + lightbox gallery slice を本 phase 内で前進 |
| A12 | `docs/dsl_spec.md` の ZStack section + 条件レンダリング grammar section（Moment 1 → Moment 2）、external-reader bar を close で適用 |

---

## 合意事項の詳細（Owner-agreed framing decisions / FD-\*）

上の「今回オーナーに決めてほしいこと」packet の根拠を置く節。draft
recommendation で、**owner alignment session で確定**する。確定後に本
セクションが ADR draft の agreed agenda になる。

**owner 判断が要る項目**（packet ⓪〜⑤）= FD-CR / FD-B / FD-G / FD-D /
FD-E / FD-F。**継承・機械的で owner 判断不要な項目** = FD-A / FD-C /
FD-H / FD-I（packet の「合意不要」枠）。この区別を曖昧にしないため、各 FD
本文の冒頭に該当 packet ID を併記する。

### FD-CR. 条件レンダリングの思想（最重要・cross-cutting）
*packet ⓪｜owner 判断が要る。*
**owner-agreed framing decision**（[設計 thesis 節](#条件レンダリングの設計-thesiswasamo-structural-rendering-model) /
[dsl-grammar.md Q6](../../../../docs/notes/dsl-grammar.md)）:
- Wasamo v1 の条件レンダリングは **approach 2（テンプレート + 独自構文・
  属性型）を採用**。
- runtime / IR / grammar は **approach 3（言語構文型 embedded DSL の自由度、
  `if`/`switch`/loop 相当）への将来拡張を妨げない**。
- **approach 1（property 制御型 = node を常に作り visible/enabled で命令的に
  出し分ける）は中心案にしない**。証明したいのは property toggling ではなく
  **tree shape の宣言的変更**。
- 条件レンダリングは単発機能ではなく **structural control-flow grammar
  family の第一歩**として定義する（Phase 7 iteration・将来 `else`/`switch`
  は同 family）。

この FD が DD-003（grammar）/ DD-004（IR・runtime）/ DD-005（effect
lifecycle）/ A12（spec を structural rendering model の第一章として書く）を
貫く。**実装スコープは `if <bool>` 最小、設計スコープは family 拡張性と
runtime identity / 宣言 tree・実体 tree 分離まで**（後者は特に DD-004 の
評価軸）。

### FD-A. 論点 slate の過不足（DD slate completeness）
*合意不要（継承・機械的）。*
**recommendation:** 上記 6 DD（001 ZStack IR/surface・002 ZStack
measure/arrange・003 条件 grammar・004 条件 IR/runtime・005 effect
lifecycle + drain 契約・006 Window-title）で本 phase の論点を尽くす。
DD-003/004/005 は FD-CR の設計 thesis を評価軸として負う。**DD slate は
この 6 件で進める**。過不足があれば冒頭 packet のレビュー（owner が ⓪〜⑤ を
判断する過程）で検出されるので、ここで別途 owner 確認は求めない。

### FD-B. 画面で見せる達成証拠 = lightbox
*packet ①｜owner 判断が要る。*
**recommendation:** 本 phase の A11 visible proof は
[gallery-wireframe.html](../../requirements/gallery-wireframe.html) の
**lightbox state**:
- ZStack overlay = **scrim（半透明 fill Box `#RRGGBBAA`、FD-G）+ centered
  photo（Box aspect 4:3 + Text placeholder）+ caption（VStack: title /
  metadata）+ nav controls（`<` `>` `x` Button）**、document-order z-order。
- 条件レンダリング = lightbox subtree が `bool` binding（例 `is_lightbox_open`）
  真のとき present、偽で absent。
- overlay の **背景の第一候補は thumbnail gallery slice**（WrapPanel /
  ScrollView、Phase 3/4 で landed 済み）。M3 最終形の thumbnail gallery は
  WrapPanel/ScrollView + Phase 7 iteration に寄るため、その上に lightbox を
  載せておけば Phase 8 full-gallery close へ素直に積み上がる。**やむを得ない
  場合のみ既存 root の別 content**（Phase 5 Grid slice 等）を背景にする。
  いずれにせよ overlay は **既存 root を置換せず上に載せる**。

**陽性対照（[constraints.md §3](./constraints.md)）:** 単発 frame では
overlay/条件を証明できない。**`is_lightbox_open` を toggle した present →
absent（および逆）の 2 frame** を最小 evidence とする。z-order proof は
「**photo/caption/nav が scrim の上に painted され、scrim が背後の
thumbnail を覆う**（半透明 scrim なら dim、後述 FD-G）」という document-order
= paint-order の重なり関係で示す。

### FD-C. 検証方針（Verification strategy / §2.4）
*合意不要（継承・機械的）。*
**recommendation:** [CLAUDE.md §Testing rules](../../../../CLAUDE.md) の
partition に従う。grammar surface なので runtime/visual だけでなく
**lowering / IR / diagnostics の pure-logic 検証**も明示する:
- **Pure-logic unit tests:** ZStack measure/arrange（重なり領域・z-order・
  clip）、条件レンダリングの subtree-presence reducer（bool → present/absent
  の純ロジック）、**`wasamoc` lowering**（条件 grammar → IR）、**IR
  roundtrip / loader**（emit → load の往復）、**invalid syntax / type /
  shape diagnostics**（非 bool 条件・不正構文位置を `wasamoc check` /
  loader が拒否すること、DD-003/004）。
- **Windows-headless integration（mock-free, CI-gated, fail-not-skip）:**
  live `WidgetNode` で bool toggle → subtree が実際に insert/remove される
  経路（DD-004/005 の runtime 実体、Compositor-bound type と絡む部分）。
  **DD-005 item 4 の drain 契約**（toggle 直後の presence 観測）はここで
  pin する。**ZStack の z-order は pure-logic だけでは足りず、実 Visual の
  child order / paint order を確認する integration** も含める（document
  order が実描画の重なり順に効くことの pin）。
- **Gallery E2E proof:** lightbox slice が `.ui → IR → runtime` を通る
  （FD-B）。
- **Assistant-visible GUI evidence:** launch + `CopyFromScreen` screenshot +
  assistant 解析、per-monitor-DPI-aware capture、陽性対照 = toggle 前後
  2 frame（[constraints.md §2/§3/§4](./constraints.md)）。owner human-visible
  smoke は別途。
- **toggle はどの操作で起こすか:** `is_lightbox_open` を **通常の text
  Button click handler** が変える経路で proof する（`Open lightbox` 等の
  text Button で open、`x` text Button で close）。これにより条件レンダリング
  単体でなく **event handler → bool state → 条件 subtree** の実用経路を 1 本
  通せる（bool state の実用も兼ねる）。**thumbnail click で open にしない**:
  thumbnail は Box + Text placeholder であり、Box hit-testing / arbitrary
  content Button / image Button は M3 out of scope（toggle の起点に必要
  ない）。

### FD-D. Window title の扱い
*packet ③｜owner 判断が要る。（R1 Window-title scope）*
**recommendation:** 本質は component metadata / host-wiring。**静的 `title:`
が native window title bar に乗ることを必須達成条件**とする（R1 解決条件）。
加えて、本 phase は binding が property → tree shape へ広がる phase なので、
**動的 (`String` binding 駆動) title を DD-006 の評価対象として比較する**
（問いを先に閉じない）。結論が defer でも構わないが、owner 意向を要件化する
場として、static-only vs static+dynamic を DD で並べて評価し、採否と理由を
ADR に残す。

### FD-E. 条件 subtree の binding 寿命と toggle 直後の観測
*packet ④｜owner 判断が要る。（旧名 Reactive-drain residual の disposition）*
**recommendation:** 2 面で扱う（DD-005）:
- **(a) effect lifecycle policy を Phase 6 で明文化する**（防御的 carry では
  なく積極的決定）。条件 subtree の内側に binding/effect があるのは Phase 6 の
  核心ケースなので、absent subtree の effect を disabled/disconnected と
  みなすか、recreate 時に再登録するか等の **最小 lifecycle policy** を本 phase
  で決める。cycle/tie/fan-out（items 1-3）の全面解決は要らないが、conditional
  rendering が作る lifecycle 境界は明文化する。
- **(b) drain proof contract（item 4）は M3-Phase 1 の同期 drain を保持**する
  方向を推奨（条件レンダリングの verification が「toggle 直後に presence を
  観測できる」前提に依存）。観測境界を今 revise する必要が出た場合のみ明示
  DD で開く。
items 1-3 は (a) の lifecycle policy が具体的に触れる範囲のみ判断し、それ以外は
carry-forward。fix/carry の最終判断は ADR で記録。

### FD-F. ZStack + 条件レンダリングを unit として出荷
*packet ⑤｜owner 判断が要る。*
**recommendation:** plan の通り 2 surface を本 phase で一体出荷する。lightbox が
両者を unit として要求するため（overlay 構造 = ZStack、open/close = 条件
レンダリング）。技術的には ZStack は条件レンダリング無しでも成立しうるが、
visible proof を 1 つの lightbox slice に閉じる方が A11 の E2E が締まる。

### FD-G. scrim は既存 `#RRGGBBAA` で半透明可（新 alpha surface は不要）
*packet ②｜owner 判断が要る（実質は確認）。*
**recommendation:** **半透明 scrim は scope creep ではない。** dsl_spec
[§4.9](../../../../docs/dsl_spec.md) は `fill` の `#RRGGBBAA` 8桁 literal を
M3-Phase 2 で既に admit し、まさに *"the structural scrim use case
(`Box { fill: #00000080 }`) is expressible"* と scrim を名指しで理由にして
いる。したがって lightbox scrim は **既存の `fill: #RRGGBBAA` literal**
（例 `#00000080`）で wireframe 通りの dim を表現できる。
- **out of scope なのは alpha *styling controls*** のみ（theming / named
  palette / dynamic alpha） — これは pre-doc の out-of-scope と一致。
- どの literal 値を使うか（opaque vs 半透明、具体 hex）は DD-002 / 実装で
  確認する実装詳細。pre-doc が opacity を out にしたのは新 *styling surface* を
  開かない意味であって、lightbox 背景を visually useless にする意図ではない。

### FD-H. 上流文書への反映タイミング（Upstream-document revision timing / two sync moments）
*合意不要（継承・機械的）。*
**recommendation:** Moment 1（ADR-Accepted commit）で `docs/dsl_spec.md` に
ZStack section + 条件レンダリング grammar section の design draft を入れ、
Moment 2（phase close）で implementation re-sync。各 doc は review-concern
単位 commit。先行 phase の two-moment 構造を継承。
**sync 対象は `dsl_spec.md` だけに閉じない**: 条件 subtree の runtime 機構・
effect lifecycle・reactive-drain 観測境界（DD-004/005）は
[`docs/architecture.md`](../../../../docs/architecture.md)、Window-title
host-wiring（DD-006）は host/ABI 経路次第で
[`docs/abi_spec.md`](../../../../docs/abi_spec.md) に触れうる。**ADR は
`architecture.md` / `abi_spec.md` への touch / no-touch を明示判断**し
（「触れうる」で曖昧に残さない、後で「必要だったのに見落とし」を防ぐ）、
touch する doc は `dsl_spec.md` / `_roadmap.md` / `plan.md` と合わせて
review-concern 単位で sync する。

### FD-I. 最終 task の retrospective 分割（Final-task retrospective split）
*合意不要（継承・機械的）。*
**recommendation:** 本 phase の `implementation/plan.md` 最終 task checklist は
task-end retro と phase-end retro を最初から別 bullet にする
（[constraints.md §6](./constraints.md)）。

---

## Inputs absorbed

- **[constraints.md](./constraints.md)（§2.1, accepted）** — R1 採用（DD-006）、
  assistant-visible evidence + 陽性対照（FD-B/C）、DPI 不採用（out of scope）、
  reactive-drain 4 義務（DD-005/FD-E）、final-task retro split（FD-I）。
- **[plan.md](../../plan.md)** — Phase 6 = ZStack + 条件レンダリング一体
  出荷（FD-F）、`bool` prereq、R1 owning phase = Phase 6、§Risks の
  reactive-drain fix-or-carry 義務。
- **[docs/notes/dsl-grammar.md Q6](../../../../docs/notes/dsl-grammar.md)** —
  **owner の条件レンダリング思想（最重要）**。3 アプローチ、v1 = approach 2 /
  future = approach 3 拡張可 / approach 1 非中心、structural control-flow
  grammar family、effect 寿命の明文化、そして **Q6「ランタイム設計への含意」=
  runtime identity の観点**（Flutter Widget/Element/RenderObject 参照点、
  軽量な「宣言 tree」と寿命を持つ「実体 tree」の分離）= **FD-CR / 設計 thesis /
  DD-003/004/005** の原典（runtime identity / 宣言 tree・実体 tree 分離は
  特に DD-004 の設計評価軸）。併せて **Q5**（条件式位置の expression grammar
  拡張点 = DD-003）、**Q2**（Window 由来 prop の runtime 配線 = R1 / DD-006）。
- **[target-app pre-doc / spec.md](../../requirements/spec.md)** — 条件
  レンダリングを M3 で normative 化（M4 reservation せず、A12）、ZStack =
  overlay 専管・document order z-order。scrim の alpha *styling controls* は
  out of scope だが半透明 literal 自体は表現可（FD-G）。
- **[gallery-wireframe.html](../../requirements/gallery-wireframe.html)** —
  lightbox の視覚契約（scrim + photo + caption + nav、`is_lightbox_open`
  条件、背景 dimmed grid）= FD-B の視覚 input。
- **[dsl_spec.md §4.9](../../../../docs/dsl_spec.md)** — `fill: #RRGGBBAA`
  alpha literal は M3-Phase 2 で landed 済み、scrim use case を名指しで
  admit（FD-G）。
- **[M3-Phase 5 decisions](../../phase-5/decisions/preamble.md)** — Grid が
  same-cell overlap を持たず overlay を ZStack に委譲、document-order =
  paint-order の先例、outer-bounds clip in scope / per-child clip out の
  先例（DD-002 が継承）。
- **[M2 handoff §3](../../../milestone-2/handoff.md)** — reactive-drain の
  4 inherited obligation（DD-005）。

---

## Next step

1. owner は冒頭の **「今回オーナーに決めてほしいこと（Owner alignment
   packet）」** の ⓪〜⑤ を判断する（この節だけで足りる）。
2. 判断結果を **「オーナー合意の記録（Owner alignment outcome）」** 節に
   記録し、status を `framing aligned` に更新。
3. §3 設計判断: `decisions/preamble.md` + DD-M3-P6-001〜006 を一括 draft
   （`Proposed` → owner review → `Accepted`）。
