---
title: M4-Phase 3 制約引き継ぎ — 述語式
status: draft
created: 2026-08-11
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

## 判断要約

| 入力 | 判断 | Phase 3 で必要な対応 |
|---|---|---|
| 範囲外の collection 添字読み | **採用** | 既定値・丸め・折返しでごまかさず、名前のある実行時診断で失敗させる |
| ハンドラ内で書き込みを条件付きにする手段 | **採用。ただしスコープ境界の合意が先** | オーナー期待は Phase 3 での提供。現行 plan から自明ではないため、framing で「Phase 3 の述語面に含む」か「plan 改訂が必要」かを明示的に決める |
| binding-only の文字列式をハンドラ代入が受理する欠陥 | **採用（診断のみ）** | Phase 3 は不正な形を早期に拒否する。文字列を書ける能力は Phase 5 のまま |
| 陽性対照の比較自体を失敗させる証明の義務化 | **不採用** | 新しい常設ルールは作らない。既存の GUI 陽性対照と DD-V-029 の限定的な red-test 義務を適用する |
| M3 で先送りした選択状態の 5 軸 | **等値選択だけ採用** | group 部品・部品所有状態・汎用 Toggle は M5、双方向束縛は Phase 7 のまま |
| Phase 1 の layout / visual 単一書き手 | **条件付き採用** | per-item 条件表示の再実体化は既存の production layout 境界へ戻し、新しい cache / Composition geometry writer を作らない |

## Phase 3 の固定された境界

[M4 plan](../../plan.md) と [M4 framing](../../requirements/framing.md) が
Phase 3 に割り当てた成果は次の 4 点である。

1. `for` の外からの collection 読み取り（件数、空判定、添字読み）。
2. `for` の各項目を条件にした構造的な表示。
3. 等値比較による、1 つの識別値を使った排他選択。
4. 上記を A（写真ギャラリー）の件数表示、ライトボックスのキャプション、
   選択中サムネイルで `.ui` → IR → runtime まで実証し、
   `docs/dsl_spec.md` に規範として同期すること。

この境界から、次も固定される。

- **文字列連結と一般算術は入れない。** 件数表示は静的な `Text` と値を
  表示する `Text` を並べて作る。
- **`TypedValue` と構造化された項目データは入れない。** 読み取り結果は
  既存の `i32` / `string` / `bool` に閉じる。
- **ABI を増やさない。** Phase 3 は checker / lowering / evaluator を中心と
  する compiler-side phase であり、M4 の ABI 面は Phase 7 が持つ。
- **新しい部品や選択状態の所有モデルを作らない。** 等値選択をサムネイルの
  区別へどう投影するか（既存 property の束縛か、条件表示による印か）は
  ADR で比較する。

**採否:** **採用（スコープの外枠）**。

## 1. 範囲外の添字読みは、実行時診断で失敗させる

Phase 2 のギャラリーでは `selected_index` が 0 未満または要素数以上に
なり得る。Phase 3 でキャプションを collection の添字読みに置き換えると、
この状態を避けて通れない。

Phase 3 は次を守る。

- 範囲外読みを空文字、0、`false` などの既定値に置き換えない。
- 先頭・末尾へ丸めず、折り返しもしない。
- 名前のある実行時診断として失敗させる。Phase 2 の
  `EvalError::ItemOutOfRange` は比較対象だが、同じ型や文言の採用までは
  この文書で決めない。
- 診断を観測可能にし、範囲外読みを正常な値が得られた成功として扱わない。

正確なエラー型、ログ面、旧値を保持するか、部分的な effect をどう扱うか、
式全体を失敗させる単位は Phase 3 の ADR で決める。ただし「安全そうな値へ
劣化させ、成功に見せる」案は選択肢に戻さない。

この分岐は純ロジックの**境界条件**なので、実装時は
[DD-V-029](../../../cross-milestone/decisions/dd-v-029-pure-logic-red-test-obligation.md)
に従い、対象テスト名と、そのテストを実際に赤くした誤実装を close artifact
に記録する。

**採否:** **採用（設計制約 + 検証制約）**。

## 2. ハンドラの書き込みを守る述語は、スコープ合意なしに実装しない

オーナーが Phase 2 close で示した期待は、Phase 3 が範囲外読みを診断する
だけでなく、作者が `selected_index` の更新を条件付きにして、先頭・末尾で
範囲外状態を作らないようにできることである。

一方、現行 plan の Phase 3 行が明記するのは「collection 読み取り、項目ごとの
条件**表示**、等値選択」であり、ハンドラ本体の条件分岐は明記されていない。
既存の構造用 `if` を、そのままハンドラの命令文と読むこともできない。

したがって Phase 3 framing は、ADR の論点を決める前に次を明示する。

1. 条件付き書き込みを、Phase 3 の述語面に含まれる狭い能力として定義できるか。
2. それとも handler control flow という別の surface であり、
   [DD-V-026](../../../cross-milestone/decisions/plan-revision-discipline.md)
   に沿った plan 改訂が必要か。
3. どちらの場合も、一般的な命令文・関数・M-expr4 まで広げずに、今回必要な
   guard をどこまで小さく保てるか。

この判断を「述語があるから当然使える」と暗黙に済ませない。plan 改訂が必要と
判定した場合は、オーナー承認と非起点側の critical check が揃うまで Frozen
agreement を変更しない。

**採否:** **採用（Phase 3 framing の必須ゲート。能力の形は未決）**。

## 3. 文字列代入の欠陥は診断するが、文字列書き込みは作らない

`docs/dsl_spec.md` §8.9 は、`StrLit`、`StrPropRead`、`Interpolation` を
binding-only と定めている。しかし現在は checker・lowering・loader を通過し、
ハンドラ実行時に evaluator が拒否するまで作者へ伝わらない。

Phase 3 が引き取るのは**診断の欠落**だけである。

- binding-only の文字列式を handler assignment の右辺に置いた形を、実行時
  呼び出しより前に作者向け診断として拒否する。
- §8.9 の規範を実装不足に合わせて弱めない。
- `string` state をハンドラから書ける能力、左辺の宣言型に基づく evaluator /
  writer の拡張、TextField の one-way binding + handler は Phase 5 に残す。
- 診断対象を handler assignment 全体の型検査へ広げるか、既知の
  binding-only 形だけに絞るかは ADR で比較する。無関係な型検査の全面刷新を
  この欠陥修正に同梱しない。

**採否:** **採用（Phase 3 の診断範囲。能力は Phase 5）**。

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
純ロジックは unit test で固定する。一方、完了条件は、Phase 3 が開く 5 つの
surface を `.ui` から実行時まで識別可能に通すことを要求する。

1. collection の件数が status bar に出る。
2. collection が空かどうかで異なる結果になる。
3. 有効な添字のとき、lightbox caption が対応する要素を表示する。
4. 1 つの `selected_index` と equality で、選択中 thumbnail が 1 つだけになる。
5. `for` の item / index を使う条件が、項目ごとの部分木を present / absent に
   する。

1・3・4 は plan が名指す A（写真ギャラリー）の消費者で行う。2・5 も A の
自然な状態として示せるなら同じ slice に置く。アプリへ不自然な表示を足す必要が
ある場合は、framing で理由を記録したうえで、出荷と同じ `.ui` → IR → runtime
経路を通る named mechanism fixture に分ける。純ロジック test だけで 2・5 の
author-facing surface を閉じない。

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
| scalar `string` を handler から書く能力 | M4-Phase 5 | Phase 3 は §3 の診断だけを持つ |
| host-listener の `key-down` 実体化 | M4-Phase 7 または最初の host consumer | Phase 3 は ABI / host listener を開かない |
| nested modal scope、per-item modal scope | M4-Phase 9 | per-item 条件表示は modal scope の組合せを要求しない |
| 半透明 cover 越しの no-change sensor | M4-Phase 9 | overlay を使わない |
| `focus-group` + `modal-scope` の組合せ | candidate pool / M6 | Phase 3 は focus annotation を増やさない |
| direct-ABI child mutator の focus rebase 欠落 | production `focused_path` reader または focus annotation ABI | Phase 3 の runtime 経路では direct-ABI child mutator を使わない |
| unknown signal 名の診断 | 4 番目の signal または不具合報告 | Phase 3 の診断 intake は handler RHS の string 形に限定し、signal 語彙を変えない |
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
