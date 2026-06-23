---
phase: M3-Phase 7b
task: T6b
title: Grid-as-ZStack-child slot.* checker fix
date: 2026-06-24
scope: task-end
merge_target: feat/m3-phase-7b
---

# T6b レトロスペクティブ

タスクブランチ: `feat/m3-phase-7b-t6b`

T5/T6 で発見された「ZStack 直下の Grid に `slot.h-align`/`slot.v-align` を
書くと checker が `inside Grid` と誤って弾く」制約のうち、**checker 側
（問題A）のみ**を修正した。`check_grid` が Grid 自身のメンバにある
`slot.*`（= 親向けデータ）を消費して reject していたのを、generic walk
（親 ZStack が検証）に委譲する形に変更。layout 側の 0×0（問題B）は
`width`/`height` author surface 欠落の症状として carry-forward に記録し、
本タスクでは触らない、と境界を切った。

コミット:

- `5907a29` docs(notes): author-controllable sizing の future-surface ノート
  （問題B の証拠ベース / Vision DR 予定地）。
- `6b28832` fix(wasamoc): Grid-as-ZStack-child の `slot.*` を accept（T6b
  本体 + plan/log の start/close gate artifact）。
- `b56ab73` test(m3-phase-7b): Codex review 1 巡目対応（accept を
  `!has_errors()` 両軸化、unknown-key / component-level / lower 側 positive
  control 追加、close-gate branch map 更新）。
- （本コミット）test(m3-phase-7b): Codex review 2 巡目対応（unknown-key
  テストに「`inside Grid` を出さない」discriminator + unknown-key count==1
  を追加、log verification 主表を `cargo test -p wasamoc --lib` 388 に差し替え、
  本 retro を最終状態に更新）。

## チェックリスト（task-end、項目 1〜11）

1. **今回の主な学び。** 「制約発見」が実は **独立2問題（checker の誤
   reject ＝ A / layout の Fill→0 collapse ＝ B）** で、scope・原因・締切が
   違った。テストコメントが両者を1つの「deferred」として束ねていたため、
   批判的に分解しないと「(b) を作業コストで退けた」ように見える罠があった。
   git で「7b は layout の measure 数学を1行も変えていない」と実証して、
   B が slot 設計と無関係な Phase 5/6 由来であることを確定できたのが要。
2. **仕様文書の変更。** なし。DD-M3-P7b-001 は既に「`slot.*` は ZStack 直下の
   子で有効」と述べており、Grid もウィジェットなので本修正は DD 意図への
   整合であって normative spec の変更を伴わない。`docs/notes/` 追加は
   exploratory ノートで規範文書ではない（項目 2 の判定対象外）。
3. **post-commit 検証。** task-end 範囲で最終的に green 確認：
   `cargo test -p wasamoc --lib` **388 passed**（T6b 6 テスト含む）、
   `cargo test --workspace` 全 green、`cargo fmt --all -- --check` exit 0、
   `cargo build -p wasamoc` に新規 warning なし。Codex review は 2 巡実施し、
   指摘修正を branch に additive に積んだ（`b56ab73` + 本コミット）。
   **clean rebuild（`cargo clean` → release+debug → workspace test）は
   T7 step-end / phase-end が所有する gate** なので本 task では未実施
   （incremental の proxy）。
4. **PO に相談すべき設計判断・トレードオフ。** あり（実施済み）。(a) 修正
   範囲、(b) 問題B の責務帰属・締切・記録メカニズムを着手前にチャットで
   合意した。残る判断（Vision DR の milestone home）は Phase 8 framing 送り
   で合意済み。
5. **タスク外のリファクタ・構造変更。** なし。production の変更は
   `check_grid` の `Member::PropertyBind` 1 アームのみ。「ついで」の
   整理はしていない（非 admitting 親の重複診断が副次的に消えたのは同一
   修正の論理的帰結で、別変更ではない）。
6. **追加 DD の要否。** なし（本 phase の ADR）。ただし phase 外の
   **Vision DR**（author-controllable sizing の milestone home）が将来
   必要で、Phase 8 framing 起票予定として記録済み。
7. **新規 Proposed ADR 項目 / 昇格。** なし。両 DD は Accepted のまま不変。
8. **マイルストーン AC / フェーズ構成の追加・変更。** Phase 構成に
   **T6b を mid-phase 挿入**（plan revision、owner 承認）。AC（A13 等）の
   accept-set は不変（むしろ実装を A13/DD 意図に整合させた）。
9. **後続へ持ち越す仮実装・近似・新規 `dead_code`。** 仮実装・近似なし。
   `dead_code` なし（`check_slot_property_outside_parent` は generic walk で
   継続使用）。**問題B（layout 0×0 / sizing）を carry-forward** として
   持ち越す（項目 10 参照）。
10. **新規の cross-task / cross-phase 設計制約。** あり。
    - **制約**: 「Fill 既定のコンテナ（Grid/ZStack）は Shrink 祖先軸の中で
      0×0 に潰れる。`slot.*` on Grid-in-ZStack は accept されるが、ZStack に
      確定サイズがある場合のみ描画される」。
    - **エビデンス**: T5 デモ（VStack-wrap 回避）、T6b の問題A/B 分解、
      git による measure 数学不変の実証。
    - **配置先（`carry-forward`）**: retro 項目 10 entry として記録。
      docs/notes ホーム（`docs/notes/author-controllable-sizing.md`）は
      landed。最終 handoff 対象化と Vision DR 起票は T7 candidate ledger →
      Phase 8 で確定（本 task では `implementation/handoff.md` を確定
      更新しない）。
11. **後続タスクリストの見直し。** 必要・実施済み。T6b を plan.md に挿入
    し、T7 は T6b merge を start-gate 前提に持つ。T7 の candidate ledger
    bullet が問題B の最終 owner（Vision DR 帰属）を引き取る。phase-end
    所有項目（status flip / CI run id / handoff finalization）は従来どおり
    phase-end owned で不変。

## 目標・前提・計画仮説の再点検

> 後続タスクの retrospective でも流用するダブルループ振り返り欄。
> 実行の巧拙（シングルループ）でなく、**設定した目標・前提・事前計画
> （= 検証すべき仮説）自体が妥当だったか**を、反証可能な signal とともに
> オーナーと問い直す。下の 5 サブセクションがひな型。

### 観測された事実（反証可能な signal）

- T5 のテストコメントは制約を1つの「accept-vs-reject deferred」として
  記述していたが、コード精読で**checker の reject と layout の 0×0 は
  別経路・別原因**だった（`check_grid` の重複評価 vs `measure_grid`
  Fill→0）。
- `git log -L :measure_grid:` と `git log -L :axis_is_stretchy:` が、
  問題B を生む関数本体は Phase 5/6 が最後の変更で **7b は未変更**と示した
  （= B は slot 設計変更と無関係、という反証可能な根拠）。
- 修正後、3 つの positive-control テスト（accept / value-validation /
  非 admitting reject）が同時に green。「blanket accept」や「blanket
  reject」ならどれか1つが落ちる構成にした。

### 自己分析（どの層が外れたか）

- 外れていたのは「実行」ではなく、もとの**前提（T5 が1問題として
  deferred 記録した枠組み）**。そのまま受けると「T7 triage で accept か
  spec-note か」の二択に縛られ、(a) checker fix を Phase 7b で打てる、
  という第三の解を見落としかけた。批判的再考の指示で前提を割り直せた。
- 「(b) を退ける理由が作業コストでは不可」というオーナー指摘が、
  merit ベース（B は別フェーズの layout 契約 + star-Grid intrinsic-size の
  未解決設計）への言語化を強制し、評価軸（product merit 主軸）に整合した。

### 計画仮説・前提の問い直し（オーナーと）

- 「制約 = 単一の deferred 項目」という前提を捨て、**2問題に分解 →
  A は in-scope バグ、B は未割当 future surface** と置き直したのは妥当
  だったか → 実装と git 実証で支持された。
- 問題B を「Phase 8 で Vision DR・pre-1.0 backstop」に置く設計は、
  「どの AC も sizing を所有しない」「M6 ABI freeze が backstop」という
  roadmap の現物に基づく。前提の出所を推測でなく現物確認にした点を
  今後も守る。

### 反証可能な是正テスト

- 「制約発見」を carry-forward する際は、**(1) 原因経路を機械出力で特定、
  (2) 既存設計変更との因果を git で反証可能に切り分け、(3) scope/締切/
  責務先を分けて記録**する、を手順化。単一 deferred ラベルで束ねない。
- checker の accept 変更は **accept 単発でなく、value-validation と
  非-admitting reject の保持を同コミットでピン留め**する（blanket
  accept/reject を反証する positive control）。

### 後続タスク / オーナーへ共有すること

- 問題B（author-controllable `width`/`height` sizing）は
  `docs/notes/author-controllable-sizing.md` が live home。T7 は candidate
  ledger に「責務先 = Phase 8 Vision DR / trigger / ABI 波及 pending」を
  1 行で積む。Vision DR は Phase 8 framing で起票（cross-milestone
  ガバナンスであり phase ではない）。
- `slot.*` on Grid-in-ZStack は今後 compile するが、ZStack に確定サイズが
  ある構造でのみ描画。gallery は VStack-wrap のまま（Grid-in-ZStack を
  足すと問題B を踏むので追加しない）。

## マージゲート

- **Review lane（implementation-gates.md §4）: branch/test-focused review。**
  T6b は `wasamoc check` の diagnostic/reject/accept ブランチ変更であり、
  schema/IR migration・runtime structural・GUI-render evidence のいずれにも
  当たらない。レビューは trap-#4 branch/test map（accept + value-validation
  reject + 非-admitting reject の 3 positive control）を確認する。
- 本 retrospective の後、**オーナーが別エージェントによるレビューを実施**。
  レビュー通過とオーナー明示承認の後に `feat/m3-phase-7b` へ no-ff merge
  （checklist 完了 = merge 許可ではない）。
- phase-end 所有項目（clean rebuild / CI run id / status flip / handoff
  finalization）は本 task では `[ ]` のまま、phase-end が所有。
