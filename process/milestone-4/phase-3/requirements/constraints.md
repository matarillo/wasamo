---
title: M4-Phase 3 制約引き継ぎ — 述語式
status: accepted
created: 2026-08-11
accepted: 2026-08-11
source-phase: M4-Phase 2
target-phase: M4-Phase 3
related:
  - process/milestone-4/phase-1/implementation/handoff.md
  - process/milestone-4/phase-2/implementation/handoff.md
  - process/milestone-4/plan.md
  - process/milestone-4/requirements/framing.md
  - process/milestone-4/requirements/spec.md
  - process/milestone-3/handoff.md
  - docs/dsl_spec.md
---

# M4-Phase 3 制約引き継ぎ

ワークフロー [§2.1](../../../procedures/workflow.md) の成果物。
[M4-Phase 2 handoff](../../phase-2/implementation/handoff.md) を直接の原典とし、
[M4-Phase 1 handoff](../../phase-1/implementation/handoff.md) も 2 phase 越しに
再点検して、Phase 3（**述語式**）に効く制約だけを論点・範囲・検証方針に
合わせて組み直す。単純な転記ではなく、各項目について「Phase 3 で何を
守るか」と採否を明示する。

この accepted 文書の判断も、その時点で得られた情報に基づく仮説である。
後続の owner agreement、plan revision、設計・実装上の新しい証拠が前提を
変えた場合は、旧前提と影響を Revisions に残したうえで本文を更新する。

## 判断要約

| 入力 | 判断 | Phase 3 で必要な対応 |
|---|---|---|
| 範囲外の collection 添字読み | **採用（result / failure contract は未決）** | runtime diagnostic / fallback / clamp と、失敗時の旧表示・対象 effect・後続 effect を DD-M4-P3-002 で比較する。Phase 2 の runtime diagnostic 期待は有力な入力だが制約にはしない |
| ハンドラ内で書き込みを条件付きにする手段 | **採用（Revision 4 landed）** | 四つの Gallery producer を全境界で止める、小さい再利用可能な handler control-flow surface を Phase 3 で届ける。正式構文・IR・評価方式は DD-M4-P3-005 で決める |
| handler assignment の admission / 型適合不足 | **採用（Revision 5 landed）** | 全 handler assignment を実行前に position capability と LHS / RHS type compatibility へ通す。scalar `string` write capability は Phase 5 のまま |
| per-item conditional の runtime structural responsibility | **採用（Revision 3 landed）** | condition evaluation と subtree 再実体化で loop context を保ち、生成・破棄を既存 effect / handler / focus / hover / layout lifecycle へ統合する |
| 陽性対照の比較自体を失敗させる証明の義務化 | **不採用** | 新しい常設ルールは作らない。既存の GUI 陽性対照と DD-V-029 の限定的な red-test 義務を適用する |
| M3 で先送りした選択状態の 5 軸 | **等値選択だけ採用** | group 部品・部品所有状態・汎用 Toggle は M5、双方向束縛は Phase 7 のまま |
| Phase 1 の layout / visual 単一書き手 | **条件付き採用** | per-item 条件表示の再実体化は既存の production layout 境界へ戻し、新しい cache / Composition geometry writer を作らない |

## Phase 3 の現行基準線

[M4 plan](../../plan.md) と [M4 framing](../../requirements/framing.md) が
Phase 3 に割り当てた成果は次の 6 点である。

1. `for` の外からの collection 読み取り（件数、空判定、添字読み）。
2. `for` の各項目を条件にした構造的な表示と、その cross-layer lifecycle。
3. 等値比較による、1 つの識別値を使った排他選択。
4. collection 境界で state write を guard できる、小さい再利用可能な handler
   control-flow surface。
5. すべての handler assignment に対する、実行前の position admission と
   LHS / RHS type compatibility の検査。
6. 上記を A（写真ギャラリー）の件数表示、ライトボックスのキャプション、
   選択中サムネイル、四つの navigation producer で `.ui` → IR → runtime まで
   実証し、`docs/dsl_spec.md` に規範として同期すること。

この境界から、次も固定される。

- **文字列連結と一般算術は入れない。** 件数表示は静的な `Text` と値を
  表示する `Text` を並べて作る。
- **`TypedValue` と構造化された項目データは入れない。** 読み取り結果は
  既存の `i32` / `string` / `bool` に閉じる。
- **ABI を増やさない。** Phase 3 は checker / lowering / evaluator と、
  per-item conditional に不可欠な runtime structural integration を既存の
  DSL / textual IR / loaded IR / runtime 内部経路で閉じる。新しい C ABI entry
  point、value carrier、host callback が不可避と分かった場合は、暗黙に吸収せず
  framing / plan の再確認へ戻す。
- **新しい部品や選択状態の所有モデルを作らない。** 等値選択をサムネイルの
  区別へどう投影するか（既存 property の束縛か、条件表示による印か）は
  ADR で比較する。

**採否:** **採用（スコープの外枠）**。

## 1. 範囲外の添字読みは、結果と failure containment を明示的に決める

Phase 2 のギャラリーでは `selected_index` が 0 未満または要素数以上に
なり得る。Phase 3 でキャプションを collection の添字読みに置き換えると、
この状態を避けて通れない。

Phase 2 handoff は runtime diagnostic を期待し、`EvalError::ItemOutOfRange` の
先例も持つ。しかし、後続の framing 合意⑤は、その期待を ADR 前の結論には
しないと決めた。DD-M4-P3-002 は少なくとも次を比較する。

- runtime error / fallback / clamp のどれを author contract にするか。
- 範囲外 read が起きたとき、旧表示、対象 effect、同じ drain の後続 effect を
  どう扱うか。
- guard による範囲外 state の予防（DD-M4-P3-005）と、既に範囲外になった state
  を読む契約をどう分離するか。

ここで守る制約は、どの案を採るかではなく、範囲外を未定義動作や偶然の evaluator
結果にせず、作者が予測できる一つの契約として決め、正常系と識別可能に検証する
ことである。

この分岐は純ロジックの**境界条件**なので、実装時は
[DD-V-029](../../../cross-milestone/decisions/dd-v-029-pure-logic-red-test-obligation.md)
に従い、対象テスト名と、そのテストを実際に赤くした誤実装を close artifact
に記録する。

**採否:** **採用（判断義務 + 検証制約。result は DD-002 で未決）**。

## 2. ハンドラの境界 guard は Phase 3 の必須成果とする

オーナーが Phase 2 close で示した期待は、Phase 3 が範囲外読みの contract を
決めるだけでなく、作者が `selected_index` の更新を条件付きにして、先頭・末尾で
範囲外状態を作らないようにできることである。

旧 plan からはこの能力を導出できなかったため、
[Revision 4](./plan-revision-proposal.md#proposed-revision-4--add-small-reusable-handler-control-flow)
が AC9、Phase 3 行、ROADMAP を additive に改訂した。Phase 3 は Left / Right key と
previous / next button の四 producer を、empty / 1 件 / 複数件の両端で止められる
surface を届ける。

固定するのは成果と狭さであり、方式ではない。正式構文、IR、predicate の最小集合、
false 時の event consumption、複数 statement での評価時点は DD-M4-P3-005 で比較する。
一般関数、handler loop、一般命令言語まで広げず、gallery 専用命令にも閉じない。

**採否:** **採用（Revision 4 による必須成果。能力の形は未決）**。

## 3. 全 handler assignment を検査するが、文字列書き込みは作らない

`docs/dsl_spec.md` §8.9 は、`StrLit`、`StrPropRead`、`Interpolation` を
binding-only と定めている。しかし現在は checker・lowering・loader を通過し、
ハンドラ実行時に evaluator が拒否するまで作者へ伝わらない。

Phase 2 の実測は個別の string 診断漏れではなく、assignment 全体に expected type と
position capability を与える仕組みの欠落を示した。
[Revision 5](./plan-revision-proposal.md#proposed-revision-5--require-complete-handler-assignment-admission-and-type-checking)
により、Phase 3 は次を守る。

- すべての handler assignment を、実行前に position capability と LHS 宣言型に
  対する RHS type compatibility の両方へ通す。
- binding-only の文字列式を handler assignment の右辺に置いた形だけでなく、
  `i32 = "abc"`、`string = 5` など方向の異なる mismatch も同じ枠組みで拒否する。
- §8.9 の規範を実装不足に合わせて弱めない。
- `string` state をハンドラから書ける能力、左辺の宣言型に基づく evaluator /
  writer の拡張、TextField の one-way binding + handler は Phase 5 に残す。
- 全 `HandlerExpr` variant と assignment form の分類・call-site audit によって
  完全性を ground truth と照合し、既知 case の追加だけで完了としない。

**採否:** **採用（Revision 5 による全 assignment の検査。能力は Phase 5）**。

## 4. per-item 条件表示は、既存の反復モデルを変えない

M3-Phase 7 と Phase 2 が確定した反復の基準線を維持する。

- `for` は位置ベース・key なしであり、項目 identity や並べ替え保持を追加しない。
- element binder は collection の要素型、index binder はゼロ始まりの `i32`
  のまま。
- Phase 3 が開くのは、既存 `if` の条件位置から item / index を使った述語を
  読めること。複数 widget の body、nested `for`、shadowing、member-range body
  は開かない。
- 同じ長さの collection 差し替えでは、保持された位置が現在値を読み直すという
  既存の positional semantics を壊さない。
- equality は**等値選択の軸だけ**を開く。group surface、widget-owned state、
  generic Toggle / appearance、two-way binding は先送り先を変えない。

**採否:** **採用（既存仕様との整合制約）**。

## 5. 構造変化は Phase 2 の入力・フォーカス状態を壊し得る

項目ごとの条件表示は、述語の値が変わると `for` の中で部分木を追加・除去する。
これは compiler-only に見えても、Phase 2 handoff の次の再発条件に触れる。

- **hover の位置パス（CF-T4-1）:** sibling の追加・除去で、範囲内の古い
  path が別ノードを指す可能性がある。bounds check だけでは誤表示を防げない。
- **同期 drain 中の registry 再照会（CF-6）:** handler の state write が
  条件部分木を作り直してから handler registry を再照会する経路では、解放済み
  pointer の再利用リスクが発火する。
- **handler binder の実行時解決（CF-T9-4）:** handler の attachment、
  loop scope、共通 expression evaluator に触れる場合、binder は attachment
  時ではなく invocation 時に解決するという識別テストを維持する。
- **focus identity（CF-T7-1 / CF-T9-1 の残件）:** focus anchor は node
  address のままであり、解放・再利用された同一 address の新しい node を
  focus target と誤認し得る。条件部分木の再生成が focusable node を含む、
  または既存 focus path をずらす場合に再発条件を点検する。pointer anchor が
  残る間は、自然な allocator observer と deterministic presentation fixture の
  2 本を削除・弱化しない。
- **focus presentation（修復済みの不変条件）:** 構造更新後の focus target
  表示は既存の単一 focus writer を通す。別の表示更新経路を作らない。

Phase 3 framing / implementation plan は、各再発条件について「実際に踏むか」を
呼び出し経路で判定する。踏むものは修復または回帰テストを同じ task に置き、
踏まないものは理由を記録する。stable logical identity が必要になった場合、または
誤った focus target が観測された場合だけ identity policy を再決定する。その他は
Phase 2 の routing、focus scope、identity policy を再設計しない。変更が必要なら
Phase 2 ADR の successor が要る。

**採否:** **条件付き採用（構造更新または共通 evaluator に触れる task の
開始ゲート）**。

## 6. 再実体化した部分木は、既存の layout / visual 更新経路に戻す

Phase 1 handoff の構造面で Phase 3 に直接効くのは、per-item 条件表示が
部分木を再実体化する点である。Phase 2 までの現物では、reactive な `if` / `for`
の追加・除去は layout dirty を立て、`emit::flush_layout` から
`run_layout_as_window_root_at_scale` へ入り直す。この経路を維持する。

- per-item 条件表示の追加・除去を、layout を通らない direct child mutation
  として実装しない。初期 build は `window::set_root`、reactive mutation は
  `mark_layout_dirty_for` → `flush_layout` という既存 2 seam のどちらかを通す。
- per-node scale cache の writer は `commit_scale_recursive` 1 つ、Phase 2 が
  追加した layout-derived `arranged_rect` の writer は `sync_visuals` 1 つの
  ままにする。条件 effect や evaluator から直接 cache を書かない。
- Composition の geometry write は `sync_visuals` 1 pass のままにする。
  条件分岐の present / absent を実現するために `SetOffset` / `SetSize` を別経路へ
  足さない。
- 新しい collection / predicate binding の property writer が Button-family の
  label を更新する場合は、`label_text`、`label_size`、node の
  `SizeConstraint::Fixed` pair を同じ経路で更新する。既存 writer を再利用できる
  場合は writer を増やさない。

Phase 1 が残した「reactive drain が plain `run_layout` を使う」欠陥は、Phase 2
T3 の F-23 で修復済みであり、現在の `flush_layout` は
`run_layout_as_window_root_at_scale` を呼ぶ。これは Phase 3 の未解決事項として
再掲せず、**修復済みの基準線**として退行させない。

**採否:** **採用（Phase 1 からの cross-hop 実装制約）**。

## 7. 検証は純ロジックと出荷アプリの両方で閉じる

Phase 3 の中心は OS 非依存の式処理なので、checker・lowering・IR・evaluator の
純ロジックは unit test で固定する。一方、完了条件は、Phase 3 が開く author
surface と validation contract を、それぞれの production gate まで識別可能に
通すことを要求する。

1. collection の件数が status bar に出る。
2. collection が空かどうかで異なる結果になる。
3. 有効な添字のとき、lightbox caption が対応する要素を表示する。
4. 1 つの `selected_index` と equality で、選択中 thumbnail が 1 つだけになる。
5. `for` の item / index を使う条件が、項目ごとの部分木を present / absent に
   する。
6. Left / Right key と previous / next button の四 producer が、empty / 1 件 /
   複数件の両端で範囲外 state を作らない。
7. 全 handler assignment が `wasamoc check` で position capability と LHS / RHS
   type compatibility を検査され、正当な既存 assignment は維持される。

1・3・4・6 は plan が名指す A（写真ギャラリー）の消費者で行う。2・5 も A の
自然な状態として示せるなら同じ slice に置く。アプリへ不自然な表示を足す必要が
ある場合は、framing で理由を記録したうえで、出荷と同じ `.ui` → IR → runtime
経路を通る named mechanism fixture に分ける。純ロジック test だけで 2・5 の
author-facing surface を閉じない。7 は全 variant / assignment form の call-site
audit と admit / reject matrix を主証拠とし、GUI を完了条件にしない。

GUI の表示を成果の証拠にする場合は、AGENTS.md の規則どおり launch + screenshot
capture + assistant analysis を行い、誤実装でも同じに見える静止画を除外する
陽性対照を含める。runtime を変更した後の捕捉前には
`cargo build --release --workspace` を行う。狭い client 幅で起きる toolbar
overflow は Phase 4 の判断事項なので、Phase 3 の証拠は内容が収まる幅で撮るか、
既知事象として明記する。

Phase 1 から継承する証拠規律も、該当する証拠形に限って適用する。

- frame 差分を使う場合、committed frame 1 枚を基準線にせず、変更の両側で
  複数回捕捉する。比較する frame 集合は同じ client-rectangle 形に揃える。
- 外部 script / tool が window geometry、screen coordinates、cursor position を
  読む場合、観測 process 自身を Per-Monitor-Aware V2 にしてから読む。
- 「A と B が違う」を陽性対照にする場合、対象性質が効かない条件で A と B が
  一致する脚も含める。
- cold target で `cargo test --workspace` を行う場合は、同じ profile の primary
  workspace build を先に行う。
- Windows integration test が shared skip guard より前に OS state を触る場合、
  その設定成功を `expect` しない。設定を試みた後の実状態を読み戻し、assert は
  guard の内側へ置く。

**採否:** **採用（検証方針の前提）**。

## 8. 陽性対照を「失敗させて見せる」新ルールは作らない

Phase 2 handoff の CF-T12-5 は、新しいルールを採る意図を持たない未決の問い
として送られた。Phase 3 では**新しい常設義務を設けない**。

理由は次のとおり。

- GUI 証拠には、既に「意図した動作を偶然の見た目から区別する陽性対照」が
  AGENTS.md で義務付けられている。
- Phase 3 で新たに必須となる範囲外添字の境界分岐は、DD-V-029 の狭い
  red-test 義務で直接覆われる。
- 「すべての green / identical observation を一度失敗させる」という広い案は
  DD-V-029 で明示的に退けられている。これを狭く再提案するだけの新しい実測は、
  現時点ではない。

比較 script を新設した task が、比較結果や判定帯の自己検査を必要と判断する
ことは妨げない。ただし task 固有の検証と、全 project の process rule を区別する。
将来ルール化する場合は DD-V-029 の successor が先に必要である。

**採否:** **不採用（CF-T12-5 を「no rule」で閉じる）**。

## 9. Phase 1 handoff の cross-hop 監査

Phase 1 handoff の全行を、Phase 3 の surface と再発条件に照らして分類した。

| Phase 1 項目群 | Phase 3 での扱い | 理由 |
|---|---|---|
| subtree の attach / re-parent / re-materialise、geometry / scale cache の単一 writer、Composition geometry の単一 pass、Button label の 3 点同期 | **採用** — §6 | per-item 条件表示が再実体化を行い、式の property writer が label 経路へ届き得る |
| stale uplifted rlib、cold test-only build、frame baseline、DPI-aware observer、陽性対照の一致脚、pre-guard OS work | **採用または条件付き採用** — §7 | runtime / GUI 証拠、cold test、Windows fixture を使う場合に再発する |
| reactive drain の誤った layout entry | **修復済み基準線** — §6 | Phase 2 T3 F-23 で修復済み。残件として再度開かない |
| layout-derived hit rectangle、hit-test の相殺変換、DIP callback、scale-change 後の pointer update | **Phase 2 で消化済み** | Phase 3 は hit-test / pointer surface を変更しない。構造更新後の hover 残件だけを §5 で継承 |
| toolbar overflow | **Phase 4** — §7 では証拠の既知事象だけ | layout policy の持ち主を Phase 3 へ変えない |
| host scale / work-area、per-window scale、client-size semantics、screen-coordinate mapping、resolution-dependent image、`WM_GETDPISCALEDSIZE` | **各 named later phase** | Phase 3 は ABI、window、IME / top-layer、image、sizing を開かない |
| pixel snapping、text quality、custom title bar、non-zero clip inset、scale-dependent measure、length newtype | **不発** | Phase 3 は geometry unit、raster quality、frame、clip、measure の意味を変えない |
| `wasamo_init` / DPI declaration ordering、last-error、DPI change fixture / step order、cross-posture capture、portable host delivery | **不発** | Phase 3 は DPI initialization / transition や配布経路を変更しない |
| misleading `wasamo-sys` warning と build-graph cleanup | **未スケジュールのまま** | Phase 3 の仕事に便乗させない。F-5 / F-21 の実行上の注意だけ §7 で守る |

## 10. 今回の制約にしない Phase 2 handoff 項目

| handoff 項目 | 持ち主 / 再発条件 | Phase 3 で不採用の理由 |
|---|---|---|
| `Row` / `HStack` overflow と重なった入力 | M4-Phase 4 | layout policy であり、Phase 3 は変更しない。証拠の既知基準線としてだけ §7 で参照 |
| pointer capture、drag、gesture、multi-contact | M4-Phase 4 以降 | Phase 3 の式面は新しい pointer gesture を持たない |
| Button の Space / Enter activation | M5 widget set | equality selection は Button の keyboard contract を決めない |
| scalar `string` を handler から書く能力 | M4-Phase 5 | Phase 3 は §3 の全 assignment validation を持つが、この writer capability は作らない |
| host-listener の `key-down` 実体化 | M4-Phase 7 または最初の host consumer | Phase 3 は ABI / host listener を開かない |
| nested modal scope、per-item modal scope | M4-Phase 9 | per-item 条件表示は modal scope の組合せを要求しない |
| 半透明 cover 越しの no-change sensor | M4-Phase 9 | overlay を使わない |
| `focus-group` + `modal-scope` の組合せ | candidate pool / M6 | Phase 3 は focus annotation を増やさない |
| direct-ABI child mutator の focus rebase 欠落 | production `focused_path` reader または focus annotation ABI | Phase 3 の runtime 経路では direct-ABI child mutator を使わない |
| unknown signal 名の診断 | 4 番目の signal または不具合報告 | Phase 3 の assignment validation は signal 語彙や signal-name admission を変えない |
| `pointer_physical` の座標型 | 3 番目の caller または単位欠陥 | Phase 3 は座標を扱わない |

## 先行フェーズからの境界確認

| 出所 | 項目 | Phase 3 での扱い |
|---|---|---|
| M3 handoff | loop 外 collection 読み取り | **採用** — 件数、空判定、添字読みとして本フェーズに着地 |
| M3 handoff | per-item conditional presence | **採用** — binder を使う条件表示として本フェーズに着地 |
| M3 handoff | selected-state の 5 軸 | **部分採用** — equality / single-discriminant selection だけ。残りの送り先は維持 |
| M3-Phase 7 | positional / un-keyed iteration | **採用** — Phase 3 の条件表示でも identity policy を変えない |
| M3-Phase 7 | nested `for`、shadowing、member-range body | **不採用** — 今回の 3 消費者に不要 |
| M3-Phase 7 | Grid 下の構造変化 | **不採用** — Phase 3 の gallery 消費者は、未移行の Grid placement を条件生成しない構成に保つ。必要になれば placement decision を先に再開 |
| M3-Phase 7 | reactive drain の cycle / ordering / cap 残件 | **原則不発** — 条件表示の effect が state を書く設計に広げない。発火した場合は黙って吸収せず ADR の論点へ戻す |

---

この文書は制約の引き継ぎだけを行う。述語の綴り、型規則、IR 表現、依存追跡、
診断文言、handler guard の具体形は、次段階の Phase 3 framing と ADR で決める。

## Revisions

- **2026-08-11 — Accepted with re-evaluation allowance.** オーナー受入済み。
  本文の判断は現時点で得られている情報と知識に基づく仮説であり、不変の事実
  ではない。新しい情報・知識・実測によって前提が変わった場合は再評価してよい。
  内容を変更するときは、根拠と影響を本節に記録し、凍結文書の改訂として扱う。
- **2026-08-12 — Reconciled with owner agreement and plan Revisions 3–5.**
  当初の compiler-side 責務、範囲外 read = runtime diagnostic、handler guard の
  scope gate、既知 string RHS だけの診断という四つの仮説を更新した。新情報は、
  per-item conditional が loop context と runtime structural lifecycle を横断する現物、
  四 producer を全境界で止める owner requirement、assignment 全体の expected-type /
  position-admission 欠落、および範囲外 result を DD-002 の比較へ戻した framing 合意⑤。
  Revision 3 (`7763555`)、Revision 4 (`4afa204`)、Revision 5 (`1499241`) は個別に
  owner-authorised / landed 済みである。ABI 非変更、既存 iteration semantics、
  layout / visual の単一 writer、検証規律、後続 phase の capability 所有は維持する。
