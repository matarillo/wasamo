---
title: M3-Phase 2 / T8 step-end retrospective
status: recorded
created: 2026-05-20
scope: step-end
task: T8 — wasamo-runtime layout aspect measure-arrange
---

# M3-Phase 2 / T8 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-2-progress.md` の **T8**
(`wasamo-runtime` layout pass が DD-M3-P2-005 inscribed-fit
measure-arrange と DD-M3-P2-001 child centring / clip overflow を
実装する) の step-end retrospective。T8 が discharge する材料は
次:

- DD-M3-P2-005 すべて — 「bounded parent 下の inscribed-fit
  (rounding contract 込み)」「one-axis unbounded 下の
  bounded-axis-wins」「both-axis unbounded 下の layout-time
  runtime error (aspect 有 / 無 共通の structural error class)」
  「no-aspect Box の shrink-to-fit / parent bounds fallback」
  「aspect value validity の compile-time gate 起点」。
- DD-M3-P2-001 の child-layout portion — 「Box 内の child は
  Box bounds に対して measure、centred alignment、Box bounds で
  clip」。
- `fill` の rendering 着地 — DD-M3-P2-003 で Box-internal
  `Color` を `SpriteVisual` brush に paint する path。T11 の
  evidence path がここから読む。

対象コミット:

- `5021936 feat(wasamo-runtime): Box aspect measure-arrange
  (M3-Phase 2 T8)`

これは step-end の gate であり、phase-end retrospective ではない。
本 step (T8) は単一 step = 単一 task 構造で、merge 先は phase
ブランチ `feat/m3-phase-2` (step→phase は ff)。

## Current Judgment

2026-05-20 時点で T8 step-end 基準は **達成済み**。

- `layout.rs`:
  - `Ratio { num, den }` を layout-engine local に追加。
    `box_values::Ratio` の structural mirror。layout.rs を
    Win32/WinRT-free に保つための boundary 型で、conversion は
    `WidgetNode::build_layout_tree` の Box arm でだけ走る。
  - `LayoutNode::box_(aspect)` constructor: `width/height` default
    が `Shrink/Shrink` に変更。これにより parent stack /
    window root は measure 出力 (inscribed-fit / bounded-axis-wins)
    を尊重する。
  - `measure_box`:
    - `aspect` 有: `(bounded, bounded)` → `inscribed_fit`,
      `(bounded, unbounded)` → `(W, derive_height(W, ratio))`,
      `(unbounded, bounded)` → `(derive_width(H, ratio), H)`,
      `(unbounded, unbounded)` →
      `Err(LayoutError::BoxAspectUnboundedBoth)`.
    - `aspect` 無 + children 空: `(bounded, bounded)` →
      `(W, H)`, `(bounded, unbounded)` → `(W, 0)`,
      `(unbounded, bounded)` → `(0, H)`,
      `(unbounded, unbounded)` → `Err(LayoutError::BoxNoExtent)`。
    - `aspect` 無 + child 有: child の measure 結果を直接返す
      (shrink-to-fit)。`wasamoc check` (T3) と
      `ir_loader::build_node` validate (T7) で single-child
      invariant が保証されているので `node.children[0]` 直接読みで
      安全。
  - `inscribed_fit`: 分岐選択を `f64` の cross multiplication
    (`W*den` vs `H*num`) で行い、`f32` round-off に依存しない
    こと (DD-M3-P2-005 numeric contract) を満たす。一致時は
    width branch (`<=`)、これは `Box(16:9)` in `1600×900` の
    happy path と一致。derived axis は `f32`。
  - `arrange_box`: parent から受けた cell (w, h) から **再度**
    inscribed-fit を計算する。これは parent が Stretch alignment 等
    で aspect を無視した cell を allocate しても Box が aspect を
    honour するため (DD-M3-P2-005 の「Box の resolved 矩形は
    always inscribed-fit」契約)。child が居れば Box bounds で
    再 measure → axis ごとに `cw.min(rw)` で clip (Fill child は
    full extent を取る) → 中央配置で arrange。
  - `LayoutError` enum (`BoxAspectUnboundedBoth` / `BoxNoExtent`)
    を追加。`measure` / `arrange` / `run_layout` の signature を
    `Result<…, LayoutError>` に変更。VStack / HStack 内の child
    measure は `.collect::<Result<Vec<_>, _>>()?` で uniform に
    伝播する。
- `widget.rs`:
  - `WidgetNode::box_`: `fill: Option<box_values::Color>` を
    `Color { A, R, G, B }` に unpack (`0xAARRGGBB` → 各 8bit)、
    `CompositionColorBrush` を構築して `SpriteVisual` に
    `SetBrush`。`fill = None` の場合 brush 未設定 (= 透明)。
    default `width/height` も `Shrink/Shrink` に flip。
  - `build_layout_tree` の Box arm: `aspect.map(|r| layout::Ratio
    { num: r.num, den: r.den })` で domain 境界を渡る。
    `node.width` / `node.height` も上書きで thread (既存の
    VStack / HStack arm と同じ shape)。
  - `WidgetNode::run_layout`: `layout::run_layout` の `Result`
    を `windows::core::Error(E_FAIL, …)` に map。message は
    DD-M3-P2-005 への pointer を含む短い文字列。`WM_SIZE` 側の
    既存 `let _ = …` callsites は touched なし。
  - `WidgetData::Box.aspect` の `#[allow(dead_code)]` を外した
    (T8 で reader 着地)。`fill` 側は変わらず field-read としては
    dead (constructor 内で読んでから捨てる構造)、ここは保持。
- 新規テスト 13 件 (`layout::tests`):
  - Numeric contract (3 件): `box_aspect_inscribed_width
    _constrained`, `box_aspect_inscribed_height_constrained`,
    `box_aspect_equal_touch_takes_width_branch`。
  - One-axis bounded / both-axis error (3 件):
    `box_aspect_unbounded_height_uses_bounded_axis_wins`,
    `box_aspect_unbounded_width_uses_bounded_axis_wins`,
    `box_aspect_unbounded_both_axes_is_runtime_error`。
  - No-aspect cases (4 件):
    `box_no_aspect_empty_matches_parent_bounds`,
    `box_no_aspect_empty_unbounded_both_is_runtime_error`,
    `box_no_aspect_empty_one_axis_unbounded_collapses_to_zero`,
    `box_no_aspect_shrinks_to_fit_child`。
  - Single child centred + clipped (2 件):
    `box_aspect_child_measured_centred_and_intrinsic_kept`,
    `box_aspect_oversize_child_clipped_to_box_bounds`。
  - Container integration (1 件):
    `box_aspect_in_vstack_uses_inscribed_via_bounded_axis_wins`
    (HStack-passes-INF + VStack-passes-INF と同型の
    intrinsic-sizing path をテスト)。
  - 「zero-child Box 矩形」確認 (1 件): `box_zero_child_still
    _has_size` — 4:3 in 600×400 の height-branch 計算で
    inscribed 矩形が立つことだけを確認。`fill` の SpriteVisual
    brush 着地 (rendering-side) は T11 が verify する。
- `cargo fmt --all -- --check` (post-commit state) zero exit。
- `cargo build --workspace` debug green、`cargo test --workspace`
  すべて green。`wasamo-runtime`: 186 → **200 passed** (+13
  layout::tests; box_values / ir_loader / widget の T6–T7 tests
  は数値そのまま)。`wasamo-ir` 12, `wasamoc` 153 変化なし。

T8 の blocker は残っていない。

## Main Learning

中心的な学びは「**layout engine の Win32/WinRT-free contract を
維持するために、Box-internal domain type を layout 側に mirror した
構造を許容したこと**」。

`box_values::Ratio` は `wasamo-runtime` Box-internal の domain type
(DD-M3-P2-002 Option A)。一方 layout engine (`layout.rs`) は
crate 内で意図的に Win32/WinRT-free を保っており、`Compositor` /
`SpriteVisual` / `windows::*` import を 0 件にすることでテスト時の
依存をゼロにしている (Phase 3 layout-engine ADR の design
intent)。この 2 つの制約の交点で取れる選択肢は次の 3 つ:

- (A) `box_values::Ratio` を pub に上げて `layout::LayoutNode` に
  そのまま埋め込む。
- (B) `LayoutNode` を `<num, den>` の生 `i32` ペアで持つ (型を
  捨てる)。
- (C) `layout::Ratio` を独立に切り、`build_layout_tree` で変換する
  (採用)。

(A) は visibility を不必要に広げ、しかも `box_values` が現状
`pub(crate)` で閉じている (T6 で確定) ことと逆行する。
(B) は型情報のロスが地味に痛い — `measure_box` / `inscribed_fit`
の signature が `Ratio` を取ることで「これは aspect ratio 専用の
分岐選択」という intent が型から読める。
(C) は同形 `{ num: i32, den: i32 }` を 2 箇所に持つコストはあるが、
layout engine の crate-internal 境界をきれいに保てる。境界変換は
`build_layout_tree` の 1 行で済み、両 type は `Clone + Copy +
PartialEq + Eq` で同形なので diverge する余地が小さい。

これは CLAUDE.md の「test-module-only mirror struct」よりも
production-mirror に近い構造で、明示的にドキュメント化する価値が
ある: **layout engine が Win32/WinRT-free な「pure-logic
sub-crate-relative module」として閉じている前提を、Box-internal
domain type の mirror で支える**。後続 phase (Phase 3 WrapPanel /
Phase 4 ScrollView) で同種の per-widget Box-internal type が
layout engine 側に現れる場合は同じ mirror 形を踏む。

副次的な学びとして、**`measure` / `arrange` の signature を
`Result<…, LayoutError>` に揃えたことで、layout-time runtime
error の伝播路が VStack / HStack arrangement を含めて uniform に
なった**。VStack / HStack 内の child measure は
`.collect::<Result<Vec<_>, _>>()?` 1 行で書ける形で T8 の error
class を透過的に通過する。これにより「将来 ScrollView /
WrapPanel が、**各 phase ADR で明示的に layout-time runtime error
を採択する** 場合」は、`LayoutError` を拡張するだけで既存の伝播路
がそのまま使える (個別 panic / silent zero に逃げず、構造的に
error path を保つ)。

ここで「明示的に採択する場合」と限定する理由は、Phase 3 layout
engine ADR ([phase-3-layout-engine.md](../../decisions/phase-3-layout-engine.md))
が **degenerate layout は clamp して error を返さない** という
方針を default として持っているため。T8 が `LayoutError` を導入
できたのは M3-Phase 2 ADR DD-M3-P2-005 が「Box(aspect) で parent
両軸 unbounded の場合は layout-time runtime error」を明示的に
上書き採択した結果であり、この採択 **無し** に WrapPanel /
ScrollView の degenerate path を runtime error 化すると Phase 3
default を破る。`LayoutError` の伝播路が「型として使える」と
「ある phase で使ってよい」は別の判断であり、後者は当該 phase の
ADR が明示するまで保留 — という理解で残す。

もう一つ副次的な学びは、**`arrange_box` で inscribed-fit を **
**再計算する設計判断**。parent allocator (例えば Stretch
alignment 下の VStack cross axis) が aspect を honour しない
cell を渡してきたとき、Box の resolved 矩形が parent cell に
追従するか aspect を保つかの選択がある。DD-M3-P2-005 の
本文 (「Box の resolved 矩形は the largest aspect-correct
rectangle」) は後者を求めており、`arrange_box` で再度
`inscribed_fit(w, h, ratio)` を呼ぶことでこれを保証している。
測定経路 (measure) と arrange 経路の両方で同じ算式を通すと、
parent の allocation policy がどうであれ Box の painted
rectangle は aspect を持つ。

これらはいずれも spec / ADR の文面を新たに足す必要はなく、
実装層のローカル最適化 (mirror 構造の維持、Result の uniform
伝播、arrange 時の再計算) として記録に残せば足りる。

## Checklist

1. **本作業の主要な学び:** あり。
   - layout engine の Win32/WinRT-free 契約を守るために、Box-
     internal domain type を layout 側に mirror した (Main
     Learning に展開)。
   - `Result<…, LayoutError>` への refactor で layout-time
     error 伝播が uniform になり、後続 phase の layout error
     拡張点が type 駆動で増設可能になった。
   - `arrange_box` は parent からの allocation に依存せず
     inscribed-fit を再計算する (DD-M3-P2-005 の resolved
     矩形契約)。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`)
   の変更:** **なし**
   - T8 は `wasamo-runtime` 内の pure-logic 実装。DD-M3-P2-005
     / DD-M3-P2-001 のいずれも既に ADR で Accepted 済みで、
     spec への転記は Moment 2 spec re-sync (T13) の責任範囲。
   - `WASAMO_ERR_*` ABI 拡張は T8 では入れず、Out-of-phase
     residual として進捗 file に記録した (後続 phase の pre-doc
     input scan で拾われる)。これは `abi_spec.md` 変更ではなく、
     `abi_spec.md` 変更を **しない** ことの記録。

3. **ローカル clean rebuild:** **green**
   - `cargo fmt --all -- --check` (post-commit state): zero exit。
   - `cargo build --workspace`: green (debug)。
   - `cargo test --workspace`: failure 0 件。
     - `wasamo-runtime`: 200 passed (T7 で 186 → T8 で +13)。
     - `wasamo-ir`: 12 passed (変化なし)。
     - `wasamoc`: 153 passed (変化なし)。
     - 他 crate 変化なし。
   - clean release rebuild (`cargo clean` → `cargo build
     --release --workspace`) は本 retro 時点で未実行。phase-end
     gate (T13) で改めて回す。
   - GitHub Actions 上の clean rebuild も phase-end gate (T13)
     で確認。

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - T8 範囲はすべて DD-M3-P2-001 / DD-M3-P2-005 の Option A
     採択から機械的に降りる。`layout::Ratio` の mirror 構造、
     `Result<…, LayoutError>` への refactor、`arrange_box` の
     inscribed-fit 再計算、`WidgetNode::box_` の `Shrink/Shrink`
     default + brush 着地 はいずれも実装層のローカル判断で、
     ADR / spec の boundary は動かしていない。
   - `WASAMO_ERR_*` の ABI 拡張を保留したことは scope 判断
     (T8 内に call site が無い) であり、Out-of-phase residual
     として明示記録した。phase-end / 次 phase pre-doc input
     review で改めて判断される。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - 変更は `wasamo-runtime/src/layout.rs` (algorithm + tests)
     と `wasamo-runtime/src/widget.rs` (Box constructor + tree
     builder + error mapping) のみ。`measure` / `arrange` /
     `run_layout` の signature 変更は T8 task 内 (layout-time
     error の伝播路) の一部であり、別途のついではない。
   - VStack / HStack の arrange 経路にも `?` を入れたが、これは
     `Result<…, LayoutError>` への signature 変更に伴う必要最小
     編集で、ロジック自体は変えていない。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - 既存 DD-M3-P2-001 / DD-M3-P2-005 で T8 範囲は完全にカバー。
     `LayoutError` enum / `arrange_box` 再計算 / `layout::Ratio`
     mirror はいずれも実装層判断で、ADR レベルの新規論点ではない。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed →
   Accepted への昇格:** **なし**
   - 当該 ADR は全 DD Accepted 済み。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**
   - A6 / A11 文言変更なし。Phase 構成変更なし。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **あり (`WidgetData::Box.fill` の `#[allow(dead_code)]` のみ
   継続)**
   - `cargo build` 出力に T8 で **新規** の dead_code 警告は出て
     いない。
   - T7 までで継続していた次の placeholder は T8 で解消:
     - `layout.rs` の `WidgetKind::Box` arm: `measure_box` /
       `arrange_box` が DD-M3-P2-005 を実装し、leaf 扱いの T6
       placeholder は消えた。
     - `WidgetData::Box.aspect` の `#[allow(dead_code)]`: T8 で
       reader (`build_layout_tree` Box arm) が着地し、attribute を
       除去した。
   - 残置している placeholder:
     - `WidgetData::Box.fill` の `#[allow(dead_code)]`: field
       自体は constructor で読まれて `Color` brush の構築に使われ
       (T8 で着地)、その後は `WidgetData::Box.fill` として
       struct に保存される。**フィールドとして再読される path
       は無い** ため Rust の dead_code lint は read を検出せず、
       `#[allow]` を維持する。Phase 2 内では DD-M3-P2-004 で
       fill 定数前提なので mutator も追加されない。将来 phase
       が bindable fill / fill animation を導入する時点で reader
       が増えるが、その時に外す方が intent と合う。
     - T11 の Windows-runtime integration test も `fill` の
       brush 着地を render-side で verify するため、`fill` field
       を accessor 経由 (test-only) で peek する path が追加され
       る可能性がある。その場合 `#[allow(dead_code)]` を外す
       タイミングは T11 着地時。
   - `unimplemented!` / `todo!` stub は T8 でも追加していない。

10. **タスクリストの後続 step 見直し:** **なし (進行通り)**
    - T9–T13 の構成・順序・依存関係に T8 実装から見て調整すべき
      点は出ていない。
    - T11 への follow-up (fill brush の verify path) と、T13 で
      閉じる `WASAMO_ERR_*` 拡張判断 (Out-of-phase residual) は
      下記 "Follow-Up" 節に明示。

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
  `WidgetData::Box.fill` の `#[allow(dead_code)]` を T11 まで
  持ち越す (T8 で reader を増やすには struct field の再読 path
  が現状不要; T11 が test-only accessor で peek する path を
  入れる)。T8 で **新規** 導入した placeholder は無い。
- item 10 (タスクリスト見直し): なし

step→phase ブランチへの ff merge はオーナー明示確認後に実行する
(retrospectives.md §3 のファストトラック基準は item 2–8 (FT 印
つき) が全て「なし」を要求し、本 step は item 9 で "あり" の
ためファストトラック不適格 — T6 / T7 と同じく item 9 起因の
不適格)。

## Verification Notes

T8 で追加したテストと、走らせた command を記録する。

新規 `layout::tests` テスト 13 件:

- Numeric / inscribed-fit (3 件):
  - `box_aspect_inscribed_width_constrained`
  - `box_aspect_inscribed_height_constrained`
  - `box_aspect_equal_touch_takes_width_branch`
- One-axis bounded-axis-wins / both-axis error (3 件):
  - `box_aspect_unbounded_height_uses_bounded_axis_wins`
  - `box_aspect_unbounded_width_uses_bounded_axis_wins`
  - `box_aspect_unbounded_both_axes_is_runtime_error`
- No-aspect cases (4 件):
  - `box_no_aspect_empty_matches_parent_bounds`
  - `box_no_aspect_empty_unbounded_both_is_runtime_error`
  - `box_no_aspect_empty_one_axis_unbounded_collapses_to_zero`
  - `box_no_aspect_shrinks_to_fit_child`
- Child centred + clipped (2 件):
  - `box_aspect_child_measured_centred_and_intrinsic_kept`
  - `box_aspect_oversize_child_clipped_to_box_bounds`
- Container integration / zero-child rectangle (2 件):
  - `box_aspect_in_vstack_uses_inscribed_via_bounded_axis_wins`
  - `box_zero_child_still_has_size`

実行コマンド (post-commit `5021936` 状態):

```text
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
```

いずれも green。clean release rebuild (`cargo clean` →
`cargo build --release --workspace`) は phase-end gate (T13)
で改めて確認する。

## Follow-Up

T8 から後続 task への明示的な引き渡し:

- **T9 (Pure-logic unit tests, ADR §Phase 2 verification closure
  item 1):** T8 で `layout::tests` の 13 件は DD-M3-P2-005
  enumeration を実質網羅した (numeric contract、one-axis bounded-
  axis-wins、both-axis runtime error、no-aspect shrink-to-fit、
  child centring / clipping)。T9 の checklist 第 3 項目
  (「Aspect measure-arrange resolver: each DD-M3-P2-005 case
  enumerated in T8」) は T8 内で landed しているので、T9 では
  Ratio / Color literal / `wasamoc check` diagnostic の 3 項目を
  T1–T3 既存 test の inventory + 不足分の補強で discharge する形
  になる。T9 着手時に T8 layout テストの一覧を引用する形で
  cross-link するのが自然。
- **T10 (IR text round-trip evidence, ADR §Phase 2 verification
  closure item 2):** T8 の layout 実装は `LayoutNode` 上で
  完結しており、IR text round-trip の load-side end-to-end は
  T7 で landed 済み。T10 では emit → parse → runtime state の
  cross-crate test を Box fixture (`Box { aspect: 16:9; fill:
  #00000080; Text { ... } }`) で追加する。runtime state 検査は
  `WidgetData::Box` field の値を test-only accessor 越しに
  peek する形か、`build_node` で構築した `WidgetNode` の subtree
  を直接 inspect する形のいずれか。Compositor 必須なので
  Windows-only path として T11 と統合する選択肢がある。
- **T11 (Windows-runtime layout integration test, ADR §Phase 2
  verification closure item 3, CI-gated):** T8 で landed した
  `arrange_box` の resolved 矩形 と child centred + clipped
  geometry を、live Compositor 環境で `WidgetNode.visual` の
  offset / size から verify する。`fill` の SpriteVisual brush
  も同じ test で peek する (DD-M3-P2-003 verification path)。
  `WidgetData::Box.fill` の `#[allow(dead_code)]` を外すのは
  ここで test-only accessor を追加した時点。Skip-guard (Phase 1
  T6 / T13 と同じ shape) は CI fail / non-CI skip の二段構え。
- **T13 (Phase-end gates):** Out-of-phase residual として記録した
  「`WASAMO_ERR_*` 拡張」を Phase-end の Out-of-phase scan で
  改めて確認する。`emit.rs::mark_layout_dirty_for` /
  `window.rs::WM_SIZE` の `let _ = r.run_layout(…)` 形は T8 で
  変えていない (`LayoutError → E_FAIL` の map により signature
  互換)。phase-end で `WASAMO_ERR_*` 拡張を実行するか defer
  するかはオーナー判断の余地あり。

これらはすべて progress file の T9 / T10 / T11 / T13 として既に
列挙済みで、T8 から新たに発生した out-of-phase 項目は
「`WASAMO_ERR_*` 拡張」のみ (進捗 file の Out-of-phase residuals
に landed)。
