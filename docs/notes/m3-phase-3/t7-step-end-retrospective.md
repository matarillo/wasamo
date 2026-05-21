---
title: M3-Phase 3 / T7 step-end retrospective
status: recorded
created: 2026-05-22
revised: 2026-05-22 (rev 4 — metadata sync + item 9 (FT) wording; doc consistency on top of rev 3)
scope: step-end
task: T7 — Layout engine: WrapPanel line-breaker and arrange
---

# M3-Phase 3 / T7 step-end retrospective

> **Revision history**
>
> - **rev 1 (initial, commit `46f706b` / 後に `984c5b8`):** Initial
>   record claimed fast-track 適格、blocker なし。
> - **rev 2 (commit `ed930b2`):** レビューで `item-cross-size`-unset
>   path の measure→arrange drift bug を指摘され、fix commit
>   `253b207` を追加。Current Judgment / Fast-Track Judgment /
>   Main Learning / Verification Notes / Follow-Up を rev 2 で改訂。
>   rev 1 の "blocker なし / fast-track 適格" の判定は **誤りだった**
>   と明示し、本版で取り消した。
> - **rev 3 (commit `981c63e`):** rev 2 の再 review で 2 件の指摘:
>   (1) clean rebuild が rev 1 を proxy にしており post-fix HEAD で
>   未実施 → post-fix `253b207` 上で `cargo clean` → release+debug
>   build → `cargo test --workspace` を物理実施し、Checklist item 3 /
>   Verification Notes に証跡を記録。(2) Main Learning 末尾に rev 1
>   由来の重複ブロック (Fill width 学び / 副次的な学び / "17 件 ADR
>   一対一対応で scope は決着済み") が残っていたため削除し、survive
>   側の "ADR 列挙 sufficiency" 段落を rev 2 framing と整合する形に
>   書き直した。
> - **rev 4 (本版; commits `25dd993` + 後続 metadata 完成 commit):**
>   再々 review で 2 件の Low 指摘: (a) Revision history / 対象
>   コミット list が rev 3 を反映していなかったので本版で同期。
>   (b) Fast-Track Judgment の item 9 を「(FT) ではない」と書いて
>   いたが `retrospectives.md` 上は **(FT) 付与あり**。文言を
>   process 文書に合わせて修正 (結論 = fast-track 不適格は維持)。
>   本版は code / gate 実態の変更を伴わない doc consistency 修正。
>   **note (rev 4 内部分割):** rev 4 は最初の commit `25dd993` で
>   Revision history 更新 + item 9 (FT) 修正までを行ったが、
>   frontmatter `revised:` と 対象コミット list を実体としては
>   rev 3 のまま放置していた (rev 4 doc が「同期した」と書いた
>   内容と乖離していた)。次の review で同 Low 指摘を受け、本 rev 4
>   の二段目 commit で frontmatter / 対象コミット list を実際に
>   rev 4 へ揃え直した。rev を bump しない判断はオーナー指示
>   (「rev を上げないと決めて rev 4 で揃える」option) に従う。

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

対象コミット (8 件; rev 4 が `25dd993` と本 commit の 2 件に分割):

- `c6a0625 feat(wasamo-runtime): WrapPanel measure-arrange line breaker (M3-Phase 3 T7)`
- `6b1b0f5 docs(m3-phase-3): flip T7 checkboxes (WrapPanel measure-arrange)`
- `984c5b8 docs(m3-phase-3): record T7 step-end retrospective` (rev 1; amended from `46f706b` to translate the Japanese commit body to English)
- `253b207 fix(wasamo-runtime): cache WrapPanel cross-bound across measure→arrange` (review-found drift bug)
- `ed930b2 docs(m3-phase-3): T7 retrospective revision — review-found bug + fix` (rev 2 retrospective body update)
- `981c63e docs(m3-phase-3): T7 retrospective rev 3 — physical clean rebuild + dedup` (rev 3 retrospective body update)
- `25dd993 docs(m3-phase-3): T7 retrospective rev 4 — metadata sync + item 9 (FT) wording` (rev 4 first commit: Revision history 拡充 + item 9 (FT) 文言修正; frontmatter / 対象コミット list の同期は漏れた)
- (this commit) `docs(m3-phase-3): T7 retrospective rev 4 metadata completion` (rev 4 second commit: 上記漏れを完成させて frontmatter / 対象コミット list を rev 4 へ揃え直す; rev は bump しない)

step-end の gate であり phase-end retrospective ではない。merge 先は
phase ブランチ `feat/m3-phase-3` (ff)。本 step (T7) は単一 task =
単一 step 構造で、現在のブランチは `feat/m3-phase-3-t7`。

## Current Judgment

2026-05-22 時点 (rev 2; commit `253b207` 反映後) で T7 step-end
基準は **達成済み**。fast-track 判定は **不適格** (item 5 「ついで
リファクタ」は引き続き「なし」だが、item 9 「後続 step に持ち越す
近似」は rev 1 で誤って「なし」と書いていた — 実態は rev 1 commit
時点で `item-cross-size`-unset path に measure→arrange drift が
残っており、rev 2 の fix commit `253b207` で解消する必要があった。
したがって T7 全体としては fast-track 規定の「item 9 なし」を
満たさない流れであり、ff merge はオーナー明示確認後に実行する)。

**rev 1 の誤り (review で取り消し):**

- rev 1 は「`compute_wrap_lines` を free function で切り出すと
  measure / arrange の整合が "同じ helper を 2 回呼ぶ" だけで
  自動保証される」と Main Learning 第 1 項で主張していた。
  これは **`item-cross-size` が set の場合のみ正しい** 部分
  真理であって、unset path では成立しない。理由は spec
  (DD-M3-P3-004 Option (a)) が "child cross input = parent of
  WrapPanel's cross bound" を要求するのに対し、`compute_wrap_lines`
  が rev 1 で受け取った `cross_bound` 引数は measure では
  `avail_h` (= parent of WrapPanel)、arrange では `h`
  (= WrapPanel 自身の allocated cross = 通常 `desired_h` ≠
  `avail_h` under `height: Shrink`) という別物だったため。
- rev 1 のテスト suite は `item-cross-size` unset の line sizing を
  Fixed-size `Rectangle` 子のみで確認しており、子の cross-bound
  依存性 (Box{aspect} のような cross input から main size を導出
  する子) を踏まない fixture だけだった。これが unset path の
  drift を test で捕捉できなかった原因。
- Review 指摘 fixture: WrapPanel(no item-cross-size, 3× Box{1:1},
  parent 250×100) で
  - measure(250, 100): 子 cross bound = 100 → 子 (100, 100); 2 lines;
    outer (250, 200)
  - arrange(250, 200): 子 cross bound = h = 200 → 子 (200, 200);
    1 per line; 3 lines stacked to cross 600, but allocated h=200。
  - 実際の `LayoutNode.children[i]` の size/offset が measure 想定と
    乖離する。

**rev 2 で fix した内容 (commit `253b207`):**

- `LayoutNode` に `wrap_measured_cross_bound: Cell<f32>` フィールド
  追加 (`pub(crate)`, sentinel `f32::NAN`)。
- `measure_wrap_panel` が `child_cross_input = item_cross_size
  .unwrap_or(avail_h)` を解決し、cell に store。
- `arrange_wrap_panel` が `item_cross_size.unwrap_or_else(||
  cached_if_not_nan else h)` で再解決し、measure と同じ
  `child_cross_input` を `compute_wrap_lines` に渡す。
- `compute_wrap_lines` のシグネチャを `(node, main_bound,
  cross_bound)` → `(node, main_bound, child_cross_input)` に
  変更 (cross-bound resolution を caller に移動)。
- `item_cross_size` が `Some` の path は cache を読まないので
  gallery sub-screen の happy path は影響なし。
- 直接 `arrange` を呼ぶ stand-alone path は cache=NAN → `h`
  fallback で self-consistent。
- 新規 regression test 2 件:
  - `wrap_panel_unset_item_cross_size_measure_arrange_consistent`
    — review fixture を pin (pre-fix で fail することは
    `wp.children[0].size` を `(100, 100)` で assert する形で確認;
    pre-fix なら `(200, 200)` になる)。
  - `wrap_panel_arrange_without_prior_measure_falls_back_to_h`
    — NaN fallback contract を pin。

**rev 2 時点の実装サマリ:**

- `measure_wrap_panel` を placeholder `Ok((0.0, 0.0))` から
  DD-M3-P3-005 の本実装に置換。outer cross = 各 line の
  cross_extent 和 + `line_spacing × (line_count − 1)`、outer main
  は `width` 制約に従って解決 (Fill → bounded 親では `0.0`
  anchor (HStack/VStack convention); unbounded 親では cumulative
  line main; Fixed → `v`; Shrink → max per-line main)。
  rev 2 で **`child_cross_input = item_cross_size.unwrap_or(avail_h)`
  を `wrap_measured_cross_bound: Cell<f32>` に store** する step を
  追加し、arrange 側が同じ value で `compute_wrap_lines` を回せる
  ようにした。
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
  に対し children を per-line 配置する。**rev 2 で**
  `child_cross_input = item_cross_size.unwrap_or_else(|| cached_or_h)`
  に変更し、measure と同じ cross input で line breaker を回す。
  - 各 child は `(main_size, cross_size)` で arrange され、
    cross-axis は `cur_cross + (line.cross_extent - cross_size) / 2`
    で line 内 centered (DD-M3-P3-001 Option A)。
  - line spacing は line 間にのみ加算、no trailing margin
    (HStack/VStack の spacing 規約と symmetric)。
- `compute_wrap_lines` を free function として抽出。**rev 2 で
  signature を `(node, main_bound, child_cross_input)` に変更**
  (rev 1 は `cross_bound` という曖昧名で、`item_cross_size.unwrap_or
  (cross_bound)` を helper 内部で解決していたのが drift の温床
  だった)。Win32/WinRT-free な `LayoutNode` のみを引数に取り、
  `measure` を per-child 呼ぶだけなので test-only mirror pattern
  は **不要** (CLAUDE.md §Testing rules の優先順 "free function
  抽出を先、mirror は最後の手段" に従う)。
- T5-era `#[allow(dead_code)]` 3 件 (`LayoutNode.item_cross_size`
  / `item_spacing` / `line_spacing`) を lift。これで Phase 3 の
  `#[allow(dead_code)]` forward-pointer は全件解消 (T5 の 6 件 →
  T6 で 3 件残 → T7 で 0 件)。
- 新規 unit test 19 件 (`wasamo-runtime/src/layout.rs::tests`;
  rev 1 で 17、rev 2 fix で +2):
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
  - **(rev 2 追加)** `wrap_panel_unset_item_cross_size_measure_arrange_consistent`
    — review fixture (WrapPanel + 3× Box{1:1}, 250×100, no
    item-cross-size) を pin。pre-fix では子 size が `(200, 200)`
    になる drift を `(100, 100)` で assert する形で fail させる。
  - **(rev 2 追加)** `wrap_panel_arrange_without_prior_measure_falls_back_to_h`
    — `f32::NAN` sentinel fallback (stand-alone arrange) の contract
    を pin。
- **Clean rebuild gate (post-fix; commit `253b207`):**
  値は本 retrospective 末尾の "Verification Notes" 節に記録。
  `wasamo-runtime` lib は **235 passed** (rev 1 の 233 から +2:
  regression test 2 件)。

rev 2 commit `253b207` 反映後、T7 の blocker は残っていない。
T8 (Windows-runtime integration test) へ進める。

## Main Learning

(rev 2 で **大幅に書き直し**。rev 1 の Main Learning 第 1 項
「free function 抽出だけで measure / arrange 整合が自動保証
される」は **部分真理であり、unset path では成立しない** ことが
review で露呈した。rev 2 の Main Learning は drift bug の発生機構
と fix の構造を中心に据える。)

最も load-bearing な学びは **「`compute_wrap_lines` を free
function 化しても、helper の引数が caller 側で意味の違う値に
バインドされていれば measure / arrange は drift する。整合性
保証は helper シグネチャ単独では足りず、caller 側の引数解決
規約まで一致させる必要がある」**。

- rev 1 の `compute_wrap_lines(node, main_bound, cross_bound)`
  は helper 内部で `node.item_cross_size.unwrap_or(cross_bound)`
  を解決していた。**測れる事実: rev 1 では caller が異なる
  `cross_bound` を渡していた** — measure は `avail_h`
  (= parent of WrapPanel's cross-axis bound, spec 通り)、arrange
  は `h` (= WrapPanel 自身の allocated cross)。`item_cross_size`
  が `Some` のときは helper 内部で override されるので一致するが、
  `None` のときは `cross_bound` がそのまま child cross input に
  化けるため、measure と arrange で異なる child measure を生む。
- **rev 2 の fix の構造的要点は 2 つ**:
  1. helper の引数を `cross_bound` (曖昧名) から
     `child_cross_input` (resolve 済み値) に rename + 解決責任を
     caller に移動。これにより「helper に何を渡せば measure と
     一致するか」が caller の責務として可視化される。
  2. arrange が measure-time の resolved value を knowing できる
     よう、`LayoutNode` に `wrap_measured_cross_bound: Cell<f32>`
     を追加して measure が store・arrange が read する。
     `Cell<f32>` の interior mutability で `&LayoutNode` 受け取り
     (existing API) を壊さず caching を実現。sentinel `f32::NAN`
     で「直接 arrange されたが先行 measure なし」path を
     `h` fallback に分岐させ、stand-alone arrange の self-
     consistency も担保。
- **より深い教訓**: pure-data 線分け器の自動整合性は、helper を
  free function で抽出するだけでは保証されない。**caller が
  渡す引数の意味論的等価性** (= measure と arrange で「同じ概念
  の値」を渡しているか) が成り立たないと崩れる。WrapPanel の
  unset path のように **caller side でしか resolve できない値**
  (= parent of WrapPanel's cross bound; arrange 時には parent が
  既に立ち去っており、WrapPanel 自身からは取れない) がある場合、
  measure→arrange の state hand-off が **必須**。`Cell<f32>` で
  既存 API を破らず実装できることを confirm したのは副次の収穫。

次に load-bearing な学びは **「テスト fixture の cross-bound-
sensitivity を意識しないと、unset path の drift は検出不能」**:

- rev 1 で `wrap_panel_cross_axis_max_of_children_when_item_cross_size_unset`
  は `Rectangle::rectangle(Fixed(40, 30), Fixed(50, 80))` を使って
  いた。`Rectangle` の `measure_leaf` は `node.width` / `height` を
  そのまま返すだけで、`avail_w` / `avail_h` 引数を無視する。
  したがって child の measure 結果が cross bound に依存せず、
  drift があっても気付かない。
- 検出には **cross input から main size を導出する子** (= Box{aspect}
  の bounded-axis-wins, あるいは Text widget の wrap measure) が
  必要。rev 2 の regression test は Box{1:1} を使うので、cross input
  が 100 → 200 に変わると main size も 100 → 200 に変わり、line
  break が変動する。**反例で fixture を組まないと不変式が見えない**
  という基本則を改めて踏んだ形。
- 一般化: pure-logic test の fixture は「テスト対象の関数が
  consumed する引数 axis すべて」に対して sensitivity を持つ
  data を選ぶべき。Phase 4 / Phase 5 で同じ pattern (helper が
  context-dependent な引数を取る) が出るときは、fixture 選定の
  時点で sensitivity を意識する。

次に load-bearing な学びは **「Fill width の WrapPanel が
unbounded main 親に出会ったときの `0.0` vs cumulative-main の
分岐は、HStack/VStack 規約 vs WrapPanel-specific 必要性の
tension を抱えている」** (rev 1 から carryover):

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
  T5 → T6 → T7 で完結した** (rev 1 から carryover)。T5 が 6 件
  (variant + constructor + helper + 3 fields) を導入し、T6 が
  catalog 側 3 件 (variant + constructor + helper) を lift し、
  T7 が layout 側 3 件 (fields) を lift する形が綺麗に揃った。
  Phase 2 の `WidgetData::Box` 単一 marker → T7 lift と比較すると
  markers が多い分 lift span が長い (T5 → T7) が、各 step で
  機械的に lift できる構造になっていた (= 各 step が
  forward-pointer を 1 種類だけ解消する design)。
- **Visible-overflow の "arrange-pass evidence" は pure-data の
  rectangle 比較で書ける** (rev 1 から carryover)。ADR evidence
  item 2 は "child.x + child.w > wp.x + wp.w" を arrange 後の
  `LayoutNode.offset` / `size` から直接読めることを要求する。
  これは pure-logic test で完全に exercise できる (Compositor を
  要さない)。Windows-runtime integration test (T8) の "absence of
  clip surface" assertion とは別物 (T8 は Visual layer での clip
  非設置を観測する)。本 T7 で pure-data 半分を完全にカバーし、
  T8 で WinRT 半分を別途 cover する分業が ADR 通り。
- **「test 戦略は ADR の verification closure 列挙を逐語に
  従って書けば十分」** が rev 1 で当てはまる **と思っていた** が、
  ADR の文言は "cross-axis line sizing when `item-cross-size` is
  unset (max of children's reported sizes)" を要求するだけで、
  **「unset path で measure と arrange が consistent であること」
  という不変式を明示していない**。rev 1 は ADR 言及通りに fixture
  を組んだが (17 件すべて DD-M3-P3-005 Recommendation の列挙と
  一対一対応)、ADR 列挙の sufficiency は別問題だった。これは ADR
  レビュー時に owner が見抜くべき性質の問題ではなく、実装者の
  自助義務 (implementor's diligence) として「helper の引数 axis
  すべてに対する sensitivity test」を組むべきだった、という反省。
  **rev 1 の retrospective が「test scope の妥当性は ADR レビュー
  時に既に決着済み」と書いていたのは誤り** — review fixture の
  外側 (= ADR 言及外の不変式) を test で守る責任は実装者側に
  残っており、本 step ではそれを果たせなかった。

## Checklist

(rev 2: item 9 を「あり」に訂正、それに伴う説明を追加。他項目は
内容変化なし。)

1. **本作業の主要な学び:** あり (記述項目)。
   - **(rev 2 改訂)** helper を free function で抽出しても、caller
     が異なる引数 binding を持てば measure / arrange は drift する
     (rev 1 の「自動整合」主張は部分真理だった)。整合性保証は
     **caller side 引数解決規約の一致** まで含めて初めて成り立つ。
     fix の構造は (i) helper 引数 rename + 解決責任の caller 化、
     (ii) `Cell<f32>` で measure→arrange の state hand-off。
   - **(rev 2 追加)** テスト fixture の cross-bound-sensitivity を
     意識しないと unset path の drift は検出不能。rev 1 は
     `Rectangle::Fixed` を使ったため child measure が `avail_h` に
     非感応で、bug を pure-logic test で素通りした。Box{aspect} 等の
     cross input → main size 導出 child が必要だった。
   - Fill width + unbounded main の corner case で `0.0` anchor を
     返すと `w = INFINITY` が arrange に流れる、`max_line_main`
     anchor の折衷を採用 (spec text に直接出てこない impl 細部、
     T10 spec re-sync で吸収判定)。(rev 1 から carryover)
   - `#[allow(dead_code)]` lift サイクルが T5 → T6 → T7 で
     完結 (Phase 3 forward-pointers ゼロに)。(rev 1 から carryover)
   - visible-overflow の arrange-pass evidence は pure-data の
     rectangle 比較で完全に書ける (Compositor 不要)。(rev 1 から
     carryover)

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:**
   **なし**
   - T7 は `wasamo-runtime/src/layout.rs` 内部の pure-logic 実装
     のみ。`dsl_spec §4.10` は T1–T4 で draft 済み、Moment 2 spec
     re-sync は T10 の責任。`abi_spec` への影響は無し
     (DD-M3-P3-005 は新規 `LayoutError` variant を導入しないため
     ABI 表面は変わらない)。`architecture.md` への影響なし
     (Win32/WinRT-free 境界は変わらず、§6 の WrapPanel 言及は
     T10 で flip)。
   - Fill + unbounded main の `max_line_main` anchor 折衷、および
     rev 2 で追加した `wrap_measured_cross_bound` cache は
     どちらも spec text に出てこないが、これは impl 細部であり
     spec invariant (outer main = parent_main_bound when bounded;
     child cross input = parent cross bound when item-cross-size
     unset) と矛盾しない (むしろ後者は spec invariant を保つための
     必要 mechanism)。T10 spec re-sync で「impl note を追加するか、
     暗黙とするか」を判定。

3. **ローカル clean rebuild:** **green** (post-fix state で物理実施)
   - **rev 2 review 指摘 (clean rebuild が proxy では不足)** を受け
     **post-fix HEAD (`253b207`) で再実施した**:
     - `cargo fmt --all -- --check` (post-fix state): zero exit。
     - `cargo clean`: 3318 files, 997.3 MiB removed。
     - `cargo build --release --workspace`: green (46.80s)。
     - `cargo build --workspace`: green (debug, 39.89s)。
     - `cargo test --workspace`: failure 0 件。
       - `wasamo-runtime` lib: **235 passed** (rev 1 の 233 から +2:
         regression test 2 件)。
       - `wasamoc` lib: 202 passed (rev 1 / T6 と同じ)。
       - `wasamo-ir`: 12 passed。
       - `wasamo-runtime` integration `ir_loader_roundtrip`:
         6 passed。
       - 他 crate (ABI / DLL / binding / counter-rust / gallery-rust /
         bool-demo-rust / counter-c) 全 green。
   - rev 1 の clean rebuild log は "Verification Notes" 節下記に
     歴史的記録として残す。

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - T7 範囲は DD-M3-P3-005 と DD-M3-P3-001 から機械的に降りる。
   - rev 2 fix の `Cell<f32>` cache は実装裁量 (spec invariant を
     満たすために必要な mechanism であり、設計判断ではない)。
   - Fill + unbounded main の `max_line_main` anchor も impl 細部
     (T10 spec re-sync で吸収判定する点を Follow-Up に明示)。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - 変更 file は 1 件 (`wasamo-runtime/src/layout.rs`)。
     `#[allow(dead_code)]` 3 件の lifting は T5 が forward-pointer
     として置いた marker の自然な完了。inline placeholder arm の
     `arrange_wrap_panel` 関数化も T5 placeholder の置換。
   - rev 2 fix で追加した `wrap_measured_cross_bound: Cell<f32>`
     フィールドは DD-M3-P3-004 / DD-M3-P3-005 が要求する spec
     invariant を満たすためのもので、ついでリファクタではない
     (review で必要性が露呈した bug-fix)。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - DD-M3-P3-005 / DD-M3-P3-001 / DD-M3-P3-004 で T7 範囲は完全に
     カバー。`Cell<f32>` cache は ADR の "child cross input =
     parent cross bound when unset" を妥当に実装するための impl
     mechanism。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格:** **なし**

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **rev 1 時点では「あり (見落とし)」**、**rev 2 fix 反映後は
   「なし」**
   - **rev 1 commit (`c6a0625`) は `item-cross-size`-unset path に
     measure→arrange drift bug を残していた** (= 後続 step に
     持ち越す仮実装相当)。rev 1 retrospective ではこれを「なし」と
     記録したが、これは誤り (test fixture が cross-bound-sensitive
     child を踏んでいなかったため自己検出できなかった)。
   - rev 2 fix commit `253b207` で drift bug を解消し、regression
     test 2 件を追加。**rev 2 反映後の現在状態では新規 placeholder
     / approximation / `dead_code` ゼロ**。
   - Fill + unbounded `max_line_main` anchor は仮実装ではなく
     恒久実装の corner-case 折衷 (Main Learning 第 3 項; T10 spec
     re-sync で吸収判定する点を Follow-Up に明示) — rev 1 と
     同判定。

10. **タスクリストの後続 step 見直し:** **なし (進行通り)**
    - T8 / T9 / T10 の構成・順序・依存関係に T7 実装から見て
      調整すべき点は出ていない。
    - T8 への follow-up は下記 "Follow-Up" 節に明示。

## Fast-Track Judgment

(rev 2 で **判定を撤回**。rev 4 で item 9 の (FT) 文言を process
文書に合わせて修正。)

rev 1 時点では fast-track 適格と判定したが、これは item 9 を誤って
「なし」と書いた前提によるもので、**実態としては rev 1 commit
`c6a0625` 時点で drift bug が後続に持ち越し相当だった**ため、本来は
item 9 「あり」→ fast-track 不適格。rev 2 fix commit `253b207`
反映後は item 9 「なし」に戻るが、**step-end 全体としては「rev 1
で merge していたら bug 入りで phase ブランチに進んでいた」状況で
あり、本 step の fast-track 規定での扱いとしては「不適格」を採る**:

- item 2 (spec doc 変更) **(FT)**: なし
- item 3 (local clean rebuild) **(FT)**: green (post-fix state)
- item 4 (PO 相談事項) **(FT)**: なし
- item 5 (ついでリファクタ) **(FT)**: なし
- item 6 (追加 DD) **(FT)**: なし
- item 7 (Proposed 増加/昇格) **(FT)**: なし
- item 8 (m3-plan AC 変更) **(FT)**: なし
- item 9 (後続に持ち越す仮実装 / 新規 `dead_code`) **(FT)**: rev 1
  時点では誤って「なし」と記録、rev 2 fix 反映後は「なし」だが、
  **rev 1 commit のまま phase ブランチに進んでいたら "あり" 相当
  だった**。step 全体の fast-track 適否判断としては「不適格」を採る
  のが正しい意思決定 — 規定の趣旨は「step 内で発見できた問題は
  step 内で解決する」だが、本 step の bug は review で発見されており、
  自己内発見の保証は最終的に有しなかった。

memory `feedback_step_end_gate_discipline.md` の規律
("item 2–8 すべて『なし』+ item 3 green に厳密") は item 9 の
誤判定を rev 2 で訂正することで item 2–8 part に限れば満たすが、
**「session 内合意は item 4 flip の根拠にならない」(同 memory)
と symmetric に、rev 1 retrospective の自己判定だけでは
gate を通せない**。step→phase ブランチへの ff merge は
**オーナー明示確認を待つ** (fast-track による事後通知では
進めない)。

## Verification Notes

T7 で追加したテストと、走らせた command を記録する。

新規テスト (layout, wasamo-runtime): 19 件 (rev 1 で 17、rev 2 fix
で +2 件; いずれも `wasamo-runtime/src/layout.rs::tests`):

**rev 1 (commit `c6a0625`) で追加:**

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

**rev 2 fix (commit `253b207`) で追加 (regression):**

- `wrap_panel_unset_item_cross_size_measure_arrange_consistent`
- `wrap_panel_arrange_without_prior_measure_falls_back_to_h`

**実行コマンド (rev 1; post-commit `6b1b0f5` 時点 — clean rebuild
gate, 歴史的記録):**

```text
cargo fmt --all -- --check                 (post-commit state; zero exit)
cargo clean                                (3408 files, 995.6 MiB)
cargo build --release --workspace          (48.20s, green)
cargo build --workspace                    (debug; 43.16s, green)
cargo test --workspace                     (failure 0)
```

`wasamo-runtime` lib test: **233 passed** (T6 の 216 から +17)。

**実行コマンド (rev 2; post-fix `253b207` 時点 — full clean rebuild
gate, 現行 step-end gate 証跡):**

```text
cargo fmt --all -- --check                 (post-fix state; zero exit)
cargo clean                                (3318 files, 997.3 MiB)
cargo build --release --workspace          (46.80s, green)
cargo build --workspace                    (debug; 39.89s, green)
cargo test --workspace                     (failure 0)
```

`wasamo-runtime` lib test: **235 passed** (rev 1 の 233 から +2:
regression test 2 件)、他 crate の test count は rev 1 と同じ
(`wasamoc` 202 / `wasamo-ir` 12 / `ir_loader_roundtrip` 6)。
**rev 2 review 指摘 (rev 1 を proxy にして clean rebuild を省略
していた)** を受け、本 retrospective 修正と同じタイミングで
post-fix HEAD に対して full clean rebuild を物理実施した結果。

**rev 2 fix の core assertion を物理 dry-run で確認済み:**
`arrange_wrap_panel` の cache lookup を `let child_cross_input =
node.item_cross_size.unwrap_or(h);` (pre-fix 相当) に一時置換し、
`cargo test -p wasamo-runtime --lib
wrap_panel_unset_item_cross_size_measure_arrange_consistent` を
走らせると以下で fail することを確認した:

```text
thread '...wrap_panel_unset_item_cross_size_measure_arrange_consistent'
panicked at wasamo-runtime\src\layout.rs:1479:
assertion `left == right` failed
  left: (200.0, 200.0)
 right: (100.0, 100.0)
```

`(200, 200)` は pre-fix で arrange-time cross bound = `h = 200`
として子 Box(1:1) を measure し直した結果、`(100, 100)` は
measure-time の cross bound = `avail_h = 100` で測った結果。
確認後、cache lookup を rev 2 fix の状態に戻し、`cargo test -p
wasamo-runtime --lib wrap_panel` で 35 passed を再確認。

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
  本 retrospective Main Learning 第 3 項で明示した
  "Fill width + unbounded main の `max_line_main` anchor" と
  **rev 2 fix で追加した `wrap_measured_cross_bound: Cell<f32>`
  cache** を、`dsl_spec §4.10` の implementation re-sync で吸収
  するか (impl note 追記)、暗黙とするか (spec invariant と矛盾
  しないので spec 側は無変更) を判定する。後者の cache は
  「spec invariant を impl で保つために必要」な mechanism なので、
  spec text には出さずに rationale を architecture.md §6 の
  WrapPanel paragraph で言及する方向が筋。
- **Phase-end retrospective (T10 closing) への持ち越し:**
  rev 2 の Main Learning「テスト fixture の cross-bound-
  sensitivity を意識しないと unset path の drift は検出不能」は、
  T7 単独の学びを超えて Phase 3 / 次 phase 共通の implementor
  discipline。**Phase 3 closing retrospective の Main Learning
  に forward distillation** すべき (forward distillation rule
  per [[feedback_retro_forward_distillation]]; M3-Phase 4
  predoc-inputs.md に "helper の引数 axis sensitivity を test
  fixture で意識的にカバーする" として書き起こす)。
- **Out-of-phase residual:** なし。Fill + unbounded anchor も
  rev 2 cache も Phase 3 内の Moment 2 spec re-sync で吸収判定する
  範囲内であり、cross-phase / cross-cutting 残件ではない。

これらはすべて progress file の T8 / T9 / T10 として既に列挙済み
(forward distillation rule は phase-end retro 側で扱う)。
T7 単体で新たに発見された Out-of-phase residual は無し。
