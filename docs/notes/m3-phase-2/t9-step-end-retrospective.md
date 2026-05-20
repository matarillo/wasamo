---
title: M3-Phase 2 / T9 step-end retrospective
status: recorded
created: 2026-05-20
scope: step-end
task: T9 — Pure-logic unit tests (ADR §Phase 2 verification closure item 1)
---

# M3-Phase 2 / T9 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-2-progress.md` の **T9**
(ADR §Phase 2 verification closure item 1 が要求する pure-logic
unit test 群が discharge されていることを示す) の step-end
retrospective。T9 が discharge する材料は次:

- Ratio literal の accept shapes と zero side reject — T1
  (`wasamo-ir`) と T2 (`wasamoc` lexer / parser) と T3
  (`wasamoc check`) と T4 (lower) と T5 (emit) の既存テストで
  完全カバー。
- Color literal の `#RRGGBB` / `#RRGGBBAA` accept と malformed
  reject — 同 layer 群 + T7 (`wasamo-runtime` IR loader lex /
  pack) の既存テストで完全カバー。
- Aspect measure-arrange resolver の DD-M3-P2-005 case enumeration
  — T8 で `wasamo-runtime/src/layout.rs::tests` に 13 件として
  既に着地済み。
- `wasamoc check` diagnostics (`bind aspect:` / `bind fill:` /
  2+ children) — T3 (compile-time gate) と T7 (`ir_loader::build
  _node` defense-in-depth) の既存テストで完全カバー。

対象コミット:

- `1e42d85 docs(m3-phase-2): T9 pure-logic test inventory and
  checklist close`

これは step-end の gate であり、phase-end retrospective ではない。
本 step (T9) は単一 step = 単一 task 構造で、merge 先は phase
ブランチ `feat/m3-phase-2` (step→phase は ff)。

## Current Judgment

2026-05-20 時点で T9 step-end 基準は **達成済み**。

- 進捗 file の T9 checklist 4 項目すべてに対し、既存テストの
  `crate::module::tests::名前` 形式の cross-link を progress doc に
  記載した。新規テストの追加は行っていない (該当箇所はすべて T1–T5
  / T7 / T8 で実装済みのため、追加が必要なテストは存在しない)。
- ADR §verification closure item 1 enumeration に含まれる
  "explicit width/height conflict" は **Phase 2 scope 外**
  (DD-M3-P2-005 §"Phase 2 scope note": `width` / `height` 属性は
  M3-Phase 2 DSL surface に登場しない) であることを progress doc T9
  本文で明示し、T9 の verification 対象から外したことを記録した。
- `cargo fmt --all -- --check` (post-commit state): zero exit。
- `cargo build --workspace` (debug): green。
- `cargo test --workspace --lib` の per-crate test count は T8
  終了時点と同一 (wasamo-ir 12 / wasamo-runtime 200 / wasamoc 153)。
  T9 はコードを変更していないので変化なし。

T9 の blocker は残っていない。

## Main Learning

中心的な学びは「**verification closure 系の checklist item は、
phase 中盤の build-up step が既に discharge していることがある —
T9 のような後段「unit test まとめ」item は、新規テスト追加では
なく既存テストの cross-link 作業として discharge される場合があり、
その discharge 形態は ADR / progress doc の文面上、明示的に許容
できる**」。

T9 の checklist 文面は

> - [ ] Ratio literal: accept shapes; zero side rejected at check.
> - [ ] Color literal: ... rejected at lex / parse.
> - [ ] Aspect measure-arrange resolver: each DD-M3-P2-005 case
>       enumerated in T8.
> - [ ] `wasamoc check` diagnostics: ...

の形で書かれており、第 3 項目には明示的に "enumerated in T8"
と書かれていた一方、第 1 / 2 / 4 項目は「T1–T5 で landed」とは
明示されていなかった。実装に入ってみると、第 1 / 2 / 4 項目も
build-up phase で discharge 済みで、T9 は inventory + cross-link
の形でしか discharge できなかった。

これは「T9 が薄かった」ではなく、**M3-Phase 2 が build-up step
(T1–T8) で per-layer testing を厳格に実施した結果として、verification
closure step (T9) が naturally collapse した**、というのが正しい
読み方。Phase 1 では verification closure step (T8 in M3-Phase 1)
で実際に新規 test を多数追加していた — Phase 2 は build-up step
で per-DD pinning を徹底したので、closure step で追加すべき
gap が残らなかった。

副次的な学びとして、**closure step の output は "新規テスト 0 件"
だが、verification closure としての value は cross-link map
そのもの** であること。M3-Phase 2 phase-end (T13) で
[dsl_spec.md §4.9](../../dsl_spec.md#49-box-layout-primitive-m3-phase-2)
の Phase status marker を flip するとき、または将来 ADR §verification
closure item 1 を audit するとき、「どの test がどの ADR statement
を pin しているか」を progress doc T9 本文から逆引きできる構造を
作ったこと自体が T9 の deliverable。

これらは spec / ADR の文面を新たに足す必要はなく、progress doc 内
の文面構造として記録した。後続 phase の verification closure step
(M3-Phase 3 以降に similar T が立つ場合) では、**closure step に
入る前に build-up step での per-DD test pinning が完了しているかを
確認し、完了していれば closure step は inventory commit として
discharge する** — という pattern を踏襲してよい。

## Checklist

1. **本作業の主要な学び:** あり。
   - verification closure step (T9) は build-up step (T1–T8) の
     per-layer test pinning が厳格な場合、inventory + cross-link
     のみで discharge できる (Main Learning に展開)。
   - Phase 2 scope 外 enumeration ("explicit width/height conflict")
     は progress doc T9 本文で明示的に scope 外と記録した。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`)
   の変更:** **なし**
   - T9 はテスト inventory commit。spec doc には触れない。
   - dsl_spec.md §4.9 の Phase status marker flip は T13
     (Moment 2 spec re-sync) の責任範囲。

3. **ローカル clean rebuild:** **green**
   - `cargo fmt --all -- --check` (post-commit state): zero exit。
   - `cargo build --workspace`: green (debug)。
   - `cargo test --workspace --lib`: failure 0 件、test count
     は T8 終了時点と同一 (wasamo-ir 12 / wasamo-runtime 200 /
     wasamoc 153)。T9 はコードを変更していないので変化なし。
   - clean release rebuild (`cargo clean` → `cargo build
     --release --workspace`) は本 retro 時点で未実行。phase-end
     gate (T13) で改めて回す。
   - GitHub Actions 上の clean rebuild も phase-end gate (T13)
     で確認。

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - T9 範囲はすべて inventory 作業。新規 design 判断は発生して
     いない。
   - 「explicit width/height conflict」を T9 で扱わなかった
     ことは ADR DD-M3-P2-005 の "Phase 2 scope note" を **追従**
     しただけで、scope 判断の新規発生ではない。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - 変更は `docs/plans/progress/m3-phase-2-progress.md` の T9
     section 本文のみ。コード変更はゼロ。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - T9 で発見された論点はゼロ。ADR Accepted DD は不変。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed →
   Accepted への昇格:** **なし**
   - 当該 ADR は全 DD Accepted 済み (T8 時点で確認)。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**
   - A6 / A11 文言変更なし。Phase 構成変更なし。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **あり (`WidgetData::Box.fill` の `#[allow(dead_code)]` のみ
   T8 から継続)**
   - T9 で **新規** に追加した dead_code 警告は無い。
   - T8 から継続している `WidgetData::Box.fill` の
     `#[allow(dead_code)]` は T11 で test-only accessor が
     入った時点で外す前提 (T8 retro item 9 参照)。
   - `unimplemented!` / `todo!` stub は T9 でも追加していない。

10. **タスクリストの後続 step 見直し:** **なし (進行通り)**
    - T10–T13 の構成・順序・依存関係に T9 から見て調整すべき
      点は出ていない。
    - T10 は IR text round-trip evidence (emit → load の cross-
      crate fixture)、T11 は Windows-runtime integration test
      (Compositor 必須)、T12 は examples/gallery seed、T13 は
      phase-end gates — それぞれ独立に進める前提に変化なし。

## Fast-Track Judgment

Fast-track criteria は **満たさない** (item 9 で
`WidgetData::Box.fill` の `#[allow(dead_code)]` を T11 まで持ち
越すため):

- item 2 (spec doc 変更): なし
- item 3 (local clean rebuild): green
- item 4 (PO 相談事項): なし
- item 5 (ついでリファクタ): なし
- item 6 (追加 DD): なし
- item 7 (Proposed 増加/昇格): なし
- item 8 (m3-plan AC 変更): なし
- item 9 (仮実装・近似・新規 dead_code 警告): **あり** —
  `WidgetData::Box.fill` の `#[allow(dead_code)]` を T8 から継続
  (T9 で **新規** 導入した placeholder は無い)。T11 が解消する
  予定。
- item 10 (タスクリスト見直し): なし

step→phase ブランチへの ff merge はオーナー明示確認後に実行する
(retrospectives.md §3 のファストトラック基準は item 2–8 (FT 印
つき) が全て「なし」を要求し、本 step は item 9 で "あり" の
ためファストトラック不適格 — T6 / T7 / T8 と同じく item 9 起因の
不適格)。

## Verification Notes

T9 は新規テスト 0 件で、既存テストの cross-link map を progress
doc に記録した step。inventory 対象テスト count を crate / module
別に集計する:

- `wasamo-ir/src/lib.rs::tests` (T1): Ratio / Color literal の
  IR variant pinning 5 件
  (`ir_literal_ratio_round_trip_values`,
  `ir_literal_ratio_distinct_by_components`,
  `ir_literal_color_round_trip_value`,
  `ir_literal_color_distinct_by_packed_value`,
  `ir_literal_ratio_and_color_distinct_from_other_variants`)。
- `wasamoc/src/lexer.rs::tests` (T2): Ratio 8 件 + Color 7 件 =
  15 件 (lex accept / disambiguation / reject)。
- `wasamoc/src/parser.rs::tests` (T2): 4 件
  (`property_bind_ratio_literal`,
  `property_bind_color_literal_six_hex`,
  `property_bind_color_literal_eight_hex`,
  `box_image_placeholder_shape`)。
- `wasamoc/src/check.rs::tests` (T3): 21 件 (Box accept 6 件 +
  multi-child reject 3 件 + value validity reject 3 件 +
  bind reject 6 件 + positional reject 5 件、`box_one_child
  _accepted` は重複参照)。
- `wasamoc/src/lower.rs::tests` (T4): 5 件
  (`box_aspect_only_lowered_to_ir_ratio`,
  `box_fill_only_opaque_lowered_to_ir_color`,
  `box_fill_with_alpha_lowered_to_ir_color`,
  `box_aspect_and_fill_lowered_together`,
  `box_with_text_child_placeholder_shape_lowered`)。
- `wasamoc/src/emit.rs::tests` (T5): Box / Ratio / Color emit
  関連 5 件
  (`box_aspect_ratio_emitted_in_surface_form`,
  `box_fill_opaque_color_emitted_in_short_form`,
  `box_fill_color_with_alpha_emitted_in_full_form`,
  `color_emit_normalises_alpha_ff_input_to_short_form`,
  `box_phase2_placeholder_widget_node_shape_emitted`,
  `box_phase2_ir_text_emit_fixture`)。
- `wasamo-runtime/src/ir_loader.rs::tests` (T7): Box / Ratio /
  Color load 関連 8 件
  (`box_phase2_load_side_fixture`,
  `box_with_single_child_is_valid`,
  `box_with_zero_children_is_valid`,
  `color_literal_short_form_packs_implicit_alpha_ff`,
  `color_literal_long_form_carries_explicit_alpha`,
  `color_literal_long_form_with_full_rgba`,
  `color_must_be_six_or_eight_hex_digits`,
  `malformed_box_with_two_children`,
  `malformed_color_on_box_wrong_prop_name`,
  `malformed_color_outside_box_fill_on_text`,
  `malformed_ratio_in_nested_node`,
  `malformed_ratio_on_box_wrong_prop_name`,
  `malformed_ratio_outside_box_aspect_on_vstack`)。
- `wasamo-runtime/src/layout.rs::tests` (T8): 13 件 (numeric
  contract 3 件、bounded-axis-wins / both-axes error 3 件、
  no-aspect 4 件、child centred / clipped 2 件、container
  integration / zero-child rectangle 2 件; 重複 1 件分は
  `box_aspect_in_vstack_uses_inscribed_via_bounded_axis_wins`
  と `box_zero_child_still_has_size` の cross-link 数え)。

実行コマンド (post-commit `1e42d85` 状態):

```text
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace --lib
```

いずれも green。clean release rebuild (`cargo clean` →
`cargo build --release --workspace`) は phase-end gate (T13)
で改めて確認する。

## Follow-Up

T9 から後続 task への明示的な引き渡し:

- **T10 (IR text round-trip evidence, ADR §Phase 2 verification
  closure item 2):** T9 で cross-link した emit-side / load-side
  tests は in-crate testing のみ。T10 は emit → parse → runtime
  state の cross-crate test を Box fixture (`Box { aspect: 16:9;
  fill: #00000080; Text { ... } }`) で追加する。emit-side は
  `wasamoc::emit::box_phase2_ir_text_emit_fixture` の output 文字列、
  load-side は `wasamo-runtime::ir_loader::box_phase2_load_side
  _fixture` の subtree 検査の **接合点** を T10 で作る (現状は
  各 crate 内で独立に fixture を assert している)。
- **T11 (Windows-runtime layout integration test, ADR §Phase 2
  verification closure item 3, CI-gated):** T9 は変化なし。
  T8 retro Follow-Up の通り、`fill` の SpriteVisual brush peek
  と `WidgetData::Box.fill` の `#[allow(dead_code)]` 解除は T11
  着地時。
- **T13 (Phase-end gates):** T9 は phase-end Out-of-phase scan
  に追加項目を出していない。T8 で記録した `WASAMO_ERR_*` 拡張の
  residual のみが scope 内。

これらはすべて progress file の T10 / T11 / T13 として既に列挙
済みで、T9 から新たに発生した out-of-phase 項目は無い。
