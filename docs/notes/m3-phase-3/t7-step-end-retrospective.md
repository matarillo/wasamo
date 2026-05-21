---
title: M3-Phase 3 / T7 step-end retrospective
status: recorded
created: 2026-05-22
scope: step-end
task: T7 — Layout engine: WrapPanel line-breaker and arrange
---

# M3-Phase 3 / T7 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-3-progress.md` の **T7**
(Layout engine: WrapPanel line-breaker and arrange) の step-end
retrospective。T7 が discharge する材料は次:

- DD-M3-P3-005 — novel normative measure-arrange algorithm
  (bounded / unbounded main-axis, first-child unconditional
  placement, oversized-line visible-overflow, cross-axis line
  sizing per DD-M3-P3-004, spacing-aware overflow `<=` inequality)。
- ADR の Phase 3 verification closure **evidence item 2**
  (line-breaker + arrange unit-test evidence — host-independent
  pure-logic tests against `wasamo-runtime/src/layout.rs`)。
- T5 retrospective Follow-Up: `LayoutNode.item_cross_size` /
  `item_spacing` / `line_spacing` の 3 件 `#[allow(dead_code)]`
  forward-pointer を、measure-arrange の reader が入った瞬間に
  lift する。

対象コミット (2 件):

- `c6a0625 feat(wasamo-runtime): WrapPanel measure-arrange line breaker (M3-Phase 3 T7)`
- `6b1b0f5 docs(m3-phase-3): flip T7 checkboxes (WrapPanel measure-arrange)`

step-end の gate であり phase-end retrospective ではない。merge 先は
phase ブランチ `feat/m3-phase-3` (ff)。本 step (T7) は単一 task =
単一 step 構造で、現在のブランチは `feat/m3-phase-3-t7`。

## Current Judgment

2026-05-22 時点で T7 step-end 基準は **達成済み**。fast-track 判定は
**適格** (checklist item 2–8 が「なし」、item 3 green、item 9 は
T7 自体が introduce した placeholder / dead_code は無し)。

- `measure_wrap_panel` を placeholder `Ok((0.0, 0.0))` から
  DD-M3-P3-005 の本実装に置換。outer cross = 各 line の
  cross_extent 和 + `line_spacing × (line_count − 1)`、outer main
  は `width` 制約に従って解決 (Fill → bounded 親では `0.0`
  anchor (HStack/VStack convention); unbounded 親では cumulative
  line main; Fixed → `v`; Shrink → max per-line main)。
  - **Fill / Shrink 分岐の根拠**: HStack/VStack の measure が
    Fill width に対して `0.0` を返す既存規約 (intrinsic 0、
    arrange-time に親から受け取った値を使う) に合わせた。
    Shrink width の WrapPanel が **oversized first-child の主軸
    extent を親に surface する** (max per-line main を返す) のは
    DD-M3-P3-005 visible-overflow の親側 (= ScrollView 等) が
    "Shrink WrapPanel" を直接 wrap することはほぼないとしても、
    pure-logic test で挙動が一致するように作っておく安全網。
  - **`avail_w` が `INFINITY` で Fill** の corner case: `0.0` を
    返すと親が `resolve_axis` で `available` (= `INFINITY`) を採用
    し、後段の arrange で `w = INFINITY` を受け取る可能性がある
    (line breaker は `main_bounded == false` を見て one-line flow
    にする)。これを避けるため、Fill + 不bounded時のみ
    `max_line_main` を返す軽い anchor 化を入れた。
- `arrange_wrap_panel` を新規追加し、`arrange` dispatch の
  `WidgetKind::WrapPanel` arm から呼ぶ (T5 の inline placeholder
  arm を関数化)。compute_wrap_lines を再走させて親割当て `(w, h)`
  に対し children を per-line 配置する。
  - 各 child は `(main_size, cross_size)` で arrange され、
    cross-axis は `cur_cross + (line.cross_extent - cross_size) / 2`
    で line 内 centered (DD-M3-P3-001 Option A)。
  - line spacing は line 間にのみ加算、no trailing margin
    (HStack/VStack の spacing 規約と symmetric)。
- `compute_wrap_lines` を free function として抽出。
  Win32/WinRT-free な `LayoutNode` のみを引数に取り、`measure` を
  per-child 呼ぶだけなので test-only mirror pattern は **不要**
  (CLAUDE.md §Testing rules の優先順 "free function 抽出を先、
  mirror は最後の手段" に従う)。
- T5-era `#[allow(dead_code)]` 3 件 (`LayoutNode.item_cross_size`
  / `item_spacing` / `line_spacing`) を lift。これで Phase 3 の
  `#[allow(dead_code)]` forward-pointer は全件解消 (T5 の 6 件 →
  T6 で 3 件残 → T7 で 0 件)。
- 新規 unit test 17 件 (`wasamo-runtime/src/layout.rs::tests`):
  - `wrap_panel_zero_children_measures_zero` —
    DD-M3-P3-001 0-child shape。
  - `wrap_panel_bounded_single_line_no_wrap` — happy path
    (3 thumbnails, no wrap)。
  - `wrap_panel_bounded_multi_line_wraps` — wrap firing path。
  - `wrap_panel_spacing_aware_inequality_uses_less_equal` —
    `50+10+50 == 110` boundary fit (`<=` per DD-M3-P3-001)。
  - `wrap_panel_no_trailing_item_spacing` — no trailing margin。
  - `wrap_panel_unbounded_main_axis_one_line_flow` —
    DD-M3-P3-005 unbounded-main Option A。
  - `wrap_panel_oversized_first_child_placed_unconditionally`
    — DD-M3-P3-005 oversized-first-child Option A の measure 半。
  - `wrap_panel_arrange_visible_overflow_for_oversized_child`
    — DD-M3-P3-005 oversized-line Option A の arrange 半
    (ADR evidence item 2 の "arrange-pass evidence of visible
    overflow" を pure-data 観測 `child.x + child.w > wp.x + wp.w`
    で pin)。
  - `wrap_panel_oversized_first_child_then_normal_children` —
    oversized first-child が line を 1 child で閉じ、後続が
    新しい line で unconditional rule を再適用する pattern。
  - `wrap_panel_cross_axis_uniform_when_item_cross_size_set` —
    DD-M3-P3-004 per-line uniform。
  - `wrap_panel_cross_axis_max_of_children_when_item_cross_size_unset`
    — DD-M3-P3-004 default (a) max-of-children。
  - `wrap_panel_cross_axis_center_alignment_within_line` —
    DD-M3-P3-001 centred (offset = `(line_extent − child_cross)/2`)。
  - `wrap_panel_zero_item_spacing_touching_items` —
    DD-M3-P3-006 zero-handling (主軸 0 spacing)。
  - `wrap_panel_zero_line_spacing_touching_lines` —
    DD-M3-P3-006 zero-handling (交差軸 0 spacing)。
  - `wrap_panel_zero_item_cross_size_degenerate_layout` —
    DD-M3-P3-006 author-requested degenerate (`item_cross_size = 0`
    → each line zero cross)。
  - `wrap_panel_unbounded_cross_with_aspect_child_propagates_box_error`
    — DD-M3-P3-005 unbounded-cross Option A (Phase 2 の
    `LayoutError::BoxAspectUnboundedBoth` 伝播)。
  - `wrap_panel_gallery_subscreen_shape` — gallery sub-screen
    形状 sanity (5 thumbs / 2-per-line / 3 lines, 250-wide,
    `item-cross-size: 88; item-spacing: 12; line-spacing: 12`)。
- **Clean rebuild gate (post-commit; commit `6b1b0f5`):**
  値は本 retrospective 末尾の "Verification Notes" 節に記録。
  `wasamo-runtime` lib は **233 passed** (T6 の 216 から +17)。

T7 の blocker は残っていない。T8 (Windows-runtime integration test) へ
進める。

## Main Learning

最も load-bearing な学びは **「pure-logic 線分け器を free function
で切り出すと、measure と arrange の整合が "同じ helper を 2 回呼ぶ"
だけで保たれる」**。

- WrapPanel は HStack/VStack/Box と違い、measure 段で line break
  を計算しないと outer cross が決まらず、arrange 段でも親割当て
  `(w, h)` に対して再度 line break を走らせて children を配置
  する必要がある。線分け器がもし `measure_wrap_panel` 内部に
  inline 実装されていたら、arrange 側で「同じロジックを書き直す」
  か「measure 結果を state として保持して arrange に渡す」かの
  2 択になる。前者は drift の温床、後者は `LayoutNode` の API
  変更 (per-WrapPanel 計算結果 cache field) を要する。
- 解は `compute_wrap_lines(node, main_bound, cross_bound)` を
  free function として切り出し、measure と arrange の双方が
  同一引数規約で呼ぶこと。これは Phase 2 の `measure_box` /
  `arrange_box` が `inscribed_fit` / `derive_height` /
  `derive_width` を共有する pattern と structurally 同じだが、
  WrapPanel は state-bearing (line list を作る) ので
  `Vec<WrapLine>` 戻り値で結合する。
- 結果として `LayoutNode` の API は変えず (per-WrapPanel cache
  field は不要)、measure と arrange の整合は "同じ helper" で
  自動保証される。test 戦略も `compute_wrap_lines` 単独で書ける
  (line break ロジックだけを exercise する) し、`run_layout`
  経由で end-to-end も走れる (本 T7 では後者を採用; 前者を加える
  なら helper を `pub(crate)` 化する必要があるが現時点で
  追加価値はない)。

次に load-bearing な学びは **「Fill width の WrapPanel が
unbounded main 親に出会ったときの `0.0` vs cumulative-main の
分岐は、HStack/VStack 規約 vs WrapPanel-specific 必要性の
tension を抱えている」**:

- HStack/VStack の measure は Fill width に対して常に `0.0` を
  返す。これは "親 arrange が `inner_w` を渡してくれるから
  measure 段で intrinsic を主張しない" という規約。
- WrapPanel も基本この規約に従いたいが、unbounded main 親
  (intrinsic-sizing 文脈) に置かれた場合、`0.0` を返すと親が
  `resolve_axis` で `available = INFINITY` を採用してしまい、
  後段の arrange が `w = INFINITY` で呼ばれる。compute_wrap_lines
  は `!main_bounded` で one-line flow に縮退するので bug には
  ならないが、`f32::INFINITY` を親が rectangle として記録する
  のは意味論的に汚い (subsequent rendering / clip / visual layer
  への波及が読みにくい)。
- 折衷として、Fill + unbounded のときだけ `max_line_main`
  (= cumulative one-line main) を返す軽い anchor を入れた。
  これは "親が WrapPanel を ScrollView 等で wrap せず直接
  intrinsic-sizing context に置く" 非通常パスを想定する保険で
  あり、gallery sub-screen / typical usage では一切呼ばれない
  (Fill + finite main の最頻路、または Shrink の代替)。
- この分岐は spec text に直接出てこない (DD-M3-P3-005 は
  "outer main = parent_main_bound when bounded" としか言わない)。
  spec text vs impl の差分は Phase 3 closing Moment 2 (T10) の
  spec re-sync で吸収判定する。Out-of-phase residual ではないが、
  forward-pointer として本 retrospective に明示する。

副次的な学び:

- **`#[allow(dead_code)]` forward-pointer の lift サイクルは
  T5 → T6 → T7 で完結した**。T5 が 6 件 (variant + constructor
  + helper + 3 fields) を導入し、T6 が catalog 側 3 件 (variant +
  constructor + helper) を lift し、T7 が layout 側 3 件 (fields)
  を lift する形が綺麗に揃った。Phase 2 の `WidgetData::Box`
  単一 marker → T7 lift と比較すると markers が多い分 lift
  span が長い (T5 → T7) が、各 step で機械的に lift できる
  構造になっていた (= 各 step が forward-pointer を 1 種類だけ
  解消する design)。
- **Visible-overflow の "arrange-pass evidence" は pure-data の
  rectangle 比較で書ける**。ADR evidence item 2 は "child.x +
  child.w > wp.x + wp.w" を arrange 後の `LayoutNode.offset` /
  `size` から直接読めることを要求する。これは pure-logic test で
  完全に exercise できる (Compositor を要さない)。Windows-runtime
  integration test (T8) の "absence of clip surface" assertion
  とは別物 (T8 は Visual layer での clip 非設置を観測する)。
  本 T7 で pure-data 半分を完全にカバーし、T8 で WinRT 半分を
  別途 cover する分業が ADR 通り。
- **「test 戦略は ADR の verification closure 列挙を逐語に
  従って書く」** が当てはまる。17 件のテストは ADR evidence
  item 2 の列挙 ("bounded main-axis happy path with multi-line
  wrap; bounded main-axis happy path with single-line fit; ...
  spacing-aware overflow inequality for `line_empty == false`;
  `item-spacing: 0` and `line-spacing: 0` degenerate layouts;
  `item-cross-size: 0` author-requested degenerate layout;
  unbounded-cross-axis-with-aspect-child propagating to Phase 2's
  `LayoutError::BoxAspectUnboundedBoth`") と一対一対応する
  形に整理した。これは progress doc T7 が "DD-M3-P3-005
  Recommendation の列挙に従う" と書いていることへの素直な
  応答であり、test scope の妥当性は ADR レビュー時に既に決着
  済み (= scope 判断は本 step 内では発生しない)。

## Checklist

1. **本作業の主要な学び:** あり (記述項目)。
   - pure-logic 線分け器を free function で切り出すと measure /
     arrange の整合が自動保証される (per-WrapPanel cache field
     不要)。
   - Fill width + unbounded main の corner case で `0.0` anchor を
     返すと `w = INFINITY` が arrange に流れる、`max_line_main`
     anchor の折衷を採用 (spec text に直接出てこない impl 細部、
     T10 spec re-sync で吸収判定)。
   - `#[allow(dead_code)]` lift サイクルが T5 → T6 → T7 で
     完結 (Phase 3 forward-pointers ゼロに)。
   - visible-overflow の arrange-pass evidence は pure-data の
     rectangle 比較で完全に書ける (Compositor 不要)。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:**
   **なし**
   - T7 は `wasamo-runtime/src/layout.rs` 内部の pure-logic 実装
     のみ。`dsl_spec §4.10` は T1–T4 で draft 済み、Moment 2 spec
     re-sync は T10 の責任。`abi_spec` への影響は無し
     (DD-M3-P3-005 は新規 `LayoutError` variant を導入しないため
     ABI 表面は変わらない)。`architecture.md` への影響なし
     (Win32/WinRT-free 境界は変わらず、§6 の WrapPanel 言及は
     T10 で flip)。
   - Fill + unbounded main の `max_line_main` anchor 折衷は
     spec text に出てこないが、これは impl 細部であり spec
     invariant (outer main = parent_main_bound when bounded) と
     矛盾しない。T10 spec re-sync で「impl note を追加するか、
     暗黙とするか」を判定。

3. **ローカル clean rebuild:** **green**
   - `cargo fmt --all -- --check` (post-commit state; commit `6b1b0f5`):
     zero exit。
   - `cargo clean`: 3408 files, 995.6 MiB removed。
   - `cargo build --release --workspace`: green (48.20s)。
   - `cargo build --workspace`: green (debug, 43.16s)。
   - `cargo test --workspace`: failure 0 件。
     - `wasamo-runtime` lib: **233 passed** (T6 の 216 から +17:
       layout WrapPanel test 17 件)。
     - `wasamoc` lib: 202 passed (T6 と同じ)。
     - `wasamo-ir`: 12 passed。
     - `wasamo-runtime` integration `ir_loader_roundtrip`: 6 passed。
     - 他 crate (ABI / DLL / binding / counter-rust / gallery-rust /
       bool-demo-rust / counter-c) 全 green。

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - T7 範囲は DD-M3-P3-005 と DD-M3-P3-001 から機械的に降りる。
     Fill + unbounded main の `max_line_main` anchor は impl 細部
     (spec invariant と矛盾しない) であり、設計判断ではない
     (Main Learning 第 2 項参照、T10 spec re-sync で吸収判定)。
   - free function `compute_wrap_lines` vs test-only mirror
     pattern の選択は CLAUDE.md §Testing rules の優先順 (free
     function 抽出を先、mirror は最後) の素直な適用で、
     `LayoutNode` が Win32/WinRT-free なため mirror は不要。
     これも設計判断ではない。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - 変更 file は 1 件 (`wasamo-runtime/src/layout.rs`)。
     `#[allow(dead_code)]` 3 件の lifting は T5 が forward-pointer
     として置いた marker の自然な完了であり、ついでリファクタ
     ではない (T5 / T6 retrospective Follow-Up にも明記済み)。
   - inline placeholder arm (T5 の `WidgetKind::WrapPanel` arm
     in `arrange`) を `arrange_wrap_panel` 関数化したのも、
     T5 placeholder の置換であってリファクタではない。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - DD-M3-P3-005 / DD-M3-P3-001 / DD-M3-P3-004 で T7 範囲は完全に
     カバー。Fill + unbounded anchor の impl 細部は新規 DD 化を
     要する設計判断ではなく、既存 DD と矛盾しない実装裁量。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格:** **なし**

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **なし (本 step 内で新規 introduce はゼロ)**
   - T7 自体は placeholder / approximation / 新規 `dead_code` を
     introduce していない。むしろ T5-era markers 残り 3 件を
     完全 lift した (Phase 3 forward-pointers 全件消去)。
   - Fill + unbounded `max_line_main` anchor は仮実装ではなく
     恒久実装の corner-case 折衷 (Main Learning 第 2 項; T10 spec
     re-sync で吸収判定する点を Follow-Up に明示)。

10. **タスクリストの後続 step 見直し:** **なし (進行通り)**
    - T8 / T9 / T10 の構成・順序・依存関係に T7 実装から見て
      調整すべき点は出ていない。
    - T8 への follow-up は下記 "Follow-Up" 節に明示。

## Fast-Track Judgment

Fast-track criteria を **満たす**:

- item 2 (spec doc 変更): なし
- item 3 (local clean rebuild): green
- item 4 (PO 相談事項): なし
- item 5 (ついでリファクタ): なし
- item 6 (追加 DD): なし
- item 7 (Proposed 増加/昇格): なし
- item 8 (m3-plan AC 変更): なし
- (item 9 は (FT) ではないが、本 step では「なし」)

memory `feedback_step_end_gate_discipline.md` の規律
("item 2–8 すべて『なし』+ item 3 green に厳密") を満たす。
step→phase ブランチへの ff merge をオーナー明示確認後に実行する
(retrospectives.md §進行手順 step 3 の fast-track 規定では事後通知
可だが、本プロジェクトの運用ルール上は最低 1 回のオーナー確認を
保持する慣行に従う)。

## Verification Notes

T7 で追加したテストと、走らせた command を記録する。

新規テスト (layout, wasamo-runtime): 17 件
(`wasamo-runtime/src/layout.rs::tests`):

- `wrap_panel_zero_children_measures_zero`
- `wrap_panel_bounded_single_line_no_wrap`
- `wrap_panel_bounded_multi_line_wraps`
- `wrap_panel_spacing_aware_inequality_uses_less_equal`
- `wrap_panel_no_trailing_item_spacing`
- `wrap_panel_unbounded_main_axis_one_line_flow`
- `wrap_panel_oversized_first_child_placed_unconditionally`
- `wrap_panel_arrange_visible_overflow_for_oversized_child`
- `wrap_panel_oversized_first_child_then_normal_children`
- `wrap_panel_cross_axis_uniform_when_item_cross_size_set`
- `wrap_panel_cross_axis_max_of_children_when_item_cross_size_unset`
- `wrap_panel_cross_axis_center_alignment_within_line`
- `wrap_panel_zero_item_spacing_touching_items`
- `wrap_panel_zero_line_spacing_touching_lines`
- `wrap_panel_zero_item_cross_size_degenerate_layout`
- `wrap_panel_unbounded_cross_with_aspect_child_propagates_box_error`
- `wrap_panel_gallery_subscreen_shape`

実行コマンド (post-commit; commit `6b1b0f5` 時点):

```text
cargo fmt --all -- --check                 (post-commit state; zero exit)
cargo clean                                (3408 files, 995.6 MiB)
cargo build --release --workspace          (48.20s, green)
cargo build --workspace                    (debug; 43.16s, green)
cargo test --workspace                     (failure 0)
```

いずれも green。`wasamo-runtime` lib test は **233 passed**
(T6 の 216 から +17)、他 crate の test count は T6 と同じ。

## Follow-Up

T7 から後続 task への明示的な引き渡し:

- **T8 (Windows-runtime integration test):** T7 が pure-data
  layout の完全実装を提供したので、T8 は Compositor / Visual layer
  との end-to-end (`.ui` → IR → `validate()` → construct_widget →
  `run_layout` → SpriteVisual rectangle) を回す責務に集中できる。
  ADR evidence item 4 の 2 fixtures (wrap-path / oversized-child)
  は T7 の `wrap_panel_gallery_subscreen_shape` /
  `wrap_panel_oversized_first_child_placed_unconditionally` と
  形状が一致しており、T8 fixture を組むときに pure-logic 期待値を
  そのまま流用できる。skip-guard は Phase 1 T6 / T13 / Phase 2 T11
  pattern (`0x80070005` from `wasamo_init`)。
- **T9 (gallery sub-screen growth):** `examples/gallery/gallery.ui`
  に 5–10 個の `Box { aspect: 1:1; fill: …; Text { … } }` を
  `WrapPanel { item-cross-size: 88; item-spacing: 12;
  line-spacing: 12 }` で包む形に追加。`gallery-rust` は workspace
  member として既に存在 (Phase 2)、`Start-Process` 起動を assistant
  が確認、visual correctness は owner-manual GUI smoke。
- **T10 (Phase-end gates) — Moment 2 spec re-sync:**
  本 retrospective Main Learning 第 2 項で明示した
  "Fill width + unbounded main の `max_line_main` anchor" を
  `dsl_spec §4.10` の implementation re-sync で吸収するか
  (impl note 追記)、暗黙とするか (spec invariant と矛盾しないので
  spec 側は無変更) を判定する。
- **Out-of-phase residual:** なし。Fill + unbounded anchor は
  Phase 3 内の Moment 2 spec re-sync で吸収判定する範囲内であり、
  cross-phase / cross-cutting 残件ではない。

これらはすべて progress file の T8 / T9 / T10 として既に列挙済み。
T7 単体で新たに発見された Out-of-phase residual は無し。
