---
title: M3-Phase 3 / T5 step-end retrospective
status: recorded
created: 2026-05-21
scope: step-end
task: T5 — wasamo-runtime widget catalog (WrapPanel)
---

# M3-Phase 3 / T5 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-3-progress.md` の **T5**
(`wasamo-runtime` widget catalog) の step-end retrospective。T5 が
discharge する材料は次:

- DD-M3-P3-001 の IR-node-shape 半分 — per-kind tag
  `WidgetKind::WrapPanel` (Option A) と
  `WidgetData::WrapPanel { item_cross_size, item_spacing, line_spacing }`
  の data slot。
- DD-M3-P3-003 の constant-only `i32` storage 半分 —
  `item_spacing` / `line_spacing` の field を `i32` で持つ
  (`PropertyValue` には乗らず ABI 表面も足さない)。
- DD-M3-P3-004 の `item_cross_size` storage 半分 — Option (a)
  parent-cross passthrough を `None` でエンコードする。

対象コミット (4 件):

- `5d43f00 feat(wasamo-runtime): add WrapPanel widget catalog (M3-Phase 3 T5)`
- `fd75ad3 docs(m3-phase-3): flip T5 checkboxes (wasamo-runtime widget catalog)`
- `5784241 docs(m3-phase-3): record T5 step-end retrospective` (本ファイル初稿)
- `bb5b2fd feat(wasamo-runtime): own WrapPanel absent-to-default mapping at T5 (M3-Phase 3 T5)`
  — オーナーレビュー指摘 (progress doc T3 / T5 が「defaults are applied
  at the runtime layer in T5, not at the IR layer」と明記しているのに
  対し、初稿の constructor は `(Option<i32>, i32, i32)` を受けて
  default 適用を T6 loader の `unwrap_or(0)` に委譲していた) に対する
  gap fix。`apply_wrap_panel_defaults` pure-logic free function を
  追加し、`wrap_panel` constructor signature を 3 つとも `Option<i32>`
  に変更、default policy を catalog layer 内に閉じた。

step-end の gate であり phase-end retrospective ではない。merge 先は
phase ブランチ `feat/m3-phase-3` (ff)。本 step (T5) は単一 task =
単一 step 構造で、現在のブランチは `feat/m3-phase-3-t5`。

## Current Judgment

2026-05-21 時点で T5 step-end 基準は **達成済み**。fast-track 判定は
**不適格** (checklist item 9 に boundary placeholder 該当)。

- `WidgetData::WrapPanel { item_cross_size: Option<i32>,
  item_spacing: i32, line_spacing: i32 }` variant を `widget.rs`
  に追加。children は既存の `WidgetNode.children: Vec<Box<WidgetNode>>`
  に乗せ (Phase 2 Box と同じ shape)、0+ children を許容
  (DD-M3-P3-001 の "no upper bound"; single-child invariant は意図的に
  非適用)。defaults は `item_cross_size: None` / `item_spacing: 0` /
  `line_spacing: 0` (DD-M3-P3-004 Option (a) + DD-M3-P3-003 touching
  items / lines)。
- `WidgetNode::wrap_panel(compositor, item_cross_size, item_spacing,
  line_spacing)` constructor: 3 属性は **すべて `Option<i32>`** で受け、
  内部で `apply_wrap_panel_defaults` (新規 pure-logic free function;
  下記参照) を呼んで absent→default mapping を行う (rev 2 で初稿
  signature `(Option<i32>, i32, i32)` から修正)。`width =
  SizeConstraint::Fill` (DD-M3-P3-005 outer main-axis =
  parent_main_bound)、`height = SizeConstraint::Shrink` (cross-axis は
  T7 で line extent の sum に collapse する想定)、background brush は
  付けない (VStack / HStack と同じ container 扱い)。`pub(crate)` で
  IR loader 経由のみ呼び出される shape を pin。
- `apply_wrap_panel_defaults(Option<i32>, Option<i32>, Option<i32>) →
  (Option<i32>, i32, i32)` を `widget.rs` の自由関数として追加
  (rev 2 で新設)。**DD-M3-P3-003 / DD-M3-P3-004 の default policy を
  catalog layer 内の単一権威 site に閉じる**ことで、progress doc
  T3 / T5 の「defaults are applied at the runtime layer in T5, not at
  the IR layer」を実装側で満たす。`item_cross_size` 不在 → `None`
  (parent-cross passthrough)、`item_spacing` / `line_spacing` 不在 →
  `0` (touching items / lines)。T6 IR loader は presence / absence
  を `Option<i32>` のまま渡すだけになり、loader 側に default
  knowledge を持ち込まない。pure-logic なので Compositor 不要で
  unit test から直接 exercise できる。
- `WidgetKind::WrapPanel` + `LayoutNode::wrap_panel(item_cross_size,
  item_spacing, line_spacing)` を `layout.rs` に追加。`LayoutNode`
  struct には `item_cross_size: Option<f32>` / `item_spacing: f32` /
  `line_spacing: f32` の 3 field を新設し、既存 4 constructor
  (`rectangle` / `vstack` / `hstack` / `box_`) で defaults (`None` /
  `0.0` / `0.0`) を埋めた。i32 → f32 の cast は
  `build_layout_tree` 境界で発生 (VStack / HStack の `spacing` /
  `padding` と同じ pattern)。
- `measure_wrap_panel` arm は `Ok((0.0, 0.0))` placeholder、
  `arrange` の `WidgetKind::WrapPanel` arm は parent-allocated cell を
  そのまま `node.offset` / `node.size` に記録する no-op。T7 が
  DD-M3-P3-005 line-breaker + arrange で置き換える前提の boundary
  placeholder で、コードコメントで T7 への forward-pointer を入れている。
- `build_layout_tree` の `WidgetData::WrapPanel { .. }` arm が
  `LayoutNode::wrap_panel` を生成し、`WidgetNode.children` を再帰展開
  して layout tree 側にも乗せる。
- `#[allow(dead_code)]` を以下 5 箇所に付け、forward-pointer コメントを
  併記して T6 (IR loader) / T7 (measure-arrange) への hand-off marker と
  する:
  - `WidgetData::WrapPanel` variant (constructor は `wrap_panel` のみ、
    現状未使用)。
  - `WidgetNode::wrap_panel` 関数 (caller は T6 で出現)。
  - `LayoutNode::item_cross_size` / `item_spacing` / `line_spacing`
    field (reader は T7 で出現)。
- 新規 unit test 7 件 (`wasamo-runtime/src/widget.rs::tests`;
  初稿 3 件 + rev 2 で追加した 4 件):
  - **variant 形状 (初稿 3 件):**
    - `wrap_panel_variant_carries_three_attributes` — `Some(96)` /
      `8` / `12` を carry できることを assert。
    - `wrap_panel_variant_defaults_match_constructor_defaults` —
      `None` / `0` / `0` の data shape を assert。
    - `wrap_panel_variant_accepts_zero_item_cross_size` —
      DD-M3-P3-006 の zero-handling (`Some(0)` が `None` と distinct な
      legal carrier) を pin。
  - **absent→default mapping (rev 2 で追加 4 件):**
    - `apply_wrap_panel_defaults_maps_all_absent_to_runtime_defaults` —
      `(None, None, None) → (None, 0, 0)` (DD-M3-P3-003 /
      DD-M3-P3-004 の core mapping)。
    - `apply_wrap_panel_defaults_passes_through_present_values` —
      `(Some(96), Some(8), Some(12)) → (Some(96), 8, 12)` (identity
      modulo unwrap; Phase 3 はこの層では clamp しない、reject は
      `wasamoc check` T1 + T6 `validate()` 側の責務)。
    - `apply_wrap_panel_defaults_handles_each_attribute_independently` —
      `(None, Some(5), None) → (None, 5, 0)` (属性間の独立性)。
    - `apply_wrap_panel_defaults_preserves_some_zero_distinct_from_none` —
      `(Some(0), Some(0), Some(0)) → (Some(0), 0, 0)` (DD-M3-P3-006
      の zero-handling を default boundary 上で再確認; `Some(0)` は
      `None` に collapse しない)。
- **Clean rebuild gate (rev 2; gap-fix commit `bb5b2fd` 後に再実行):**
  値は本 retrospective 末尾の "Verification Notes" 節に記録。初稿
  rebuild (commit `fd75ad3` 時点) も green で、その時点での
  `wasamo-runtime` lib は 203 passed (T4 の 200 から +3)、rev 2 で
  207 passed に進む。

T5 の blocker は残っていない。T6 (`wasamo-runtime` IR loader +
`validate()` defense-in-depth) へ進める。

## Main Learning

最も load-bearing な学びは **「『defaults at the runtime layer in
T5』という progress doc の文言は、catalog constructor が absent→default
を applied する責務を持つ、ということ」** (rev 2 でオーナーレビュー
指摘を受けて析出)。初稿実装は `WidgetNode::wrap_panel` を
`(Option<i32>, i32, i32)` で受けて、`Option<i32> → i32` の unwrap を
T6 IR loader の `unwrap_or(0)` に委譲する設計だった。これは技術的には
動くが、progress doc T3 / T5 の「defaults are applied at the runtime
layer in T5, not at the IR layer」を素直に読むと **未達**。

- 「runtime layer」は wasamo-runtime crate のことであり、その内部で
  catalog (T5) と loader (T6) のどちらが default policy を持つかは
  独立な設計判断。progress doc は明示的に **T5** を指している。
- catalog constructor が default を持つほうが natural — constructor
  はその variant のすべての field invariant に責任を持つ entry point
  で、「absent → default」もそこに含まれる。loader は単に DSL surface
  の presence / absence を Option として transport する。
- 副産物として、catalog 側に default policy を閉じることで
  pure-logic test (`apply_wrap_panel_defaults`) が成立する。
  loader-driven の defaults は Compositor が要る integration test
  でしか pin できないが、free function に切り出された default mapping
  は Compositor 不要で 4 個の small test に分解できる。
- **memory `feedback_revise_dont_workaround` の体現**: progress doc を
  ad-hoc に「実は T6 が apply するつもりでした」と reinterpret する
  のではなく、実装を doc 文言に合わせて修正する道を選んだ。doc は
  仮説で、実装と合わなければ doc を直すか実装を直すかのいずれかで
  あり、両者を不整合に放置しない。

次に load-bearing な学びは **「Phase 2 T6 の `#[allow(dead_code)]` +
forward-pointer comment の hand-off pattern は Phase 3 T5 にもそのまま
適用できる」** という再確認。Phase 2 T6 の Main Learning がそのまま
T5 にも当てはまる:

- `WidgetData::WrapPanel` の variant は writer (T6 IR loader) も
  reader (T7 layout) もまだ存在しない時点で立てる。Rust の `dead_code`
  lint は基本的に「読まれない field / 構築されない variant」を検出
  するので、現時点ではすべて検出対象 — `#[allow(dead_code)]` で
  抑制しないと build が warning だらけになる。
- `#[allow(dead_code)]` は安全網ではなく **forward-pointer marker** —
  T6 / T7 の作業者は progress file の T6 / T7 行とコメントの
  "until T6 (IR loader) and T7 (measure-arrange) close them out" を
  読んで wiring を入れる必要がある。lint は補助にならない。
- T6 / T7 が完了して production caller / reader が入った時点で
  `#[allow(dead_code)]` を外す (= 本来の wiring の一部であり、別途
  ついでリファクタではない)。`#[allow]` を外したときに reader 側が
  まだ繋がっていなければそこで lint が再点灯する可能性がある — その
  範囲では allow を外す瞬間が遅延した check point になる。

副次的な学び:

- **「Box の `aspect: Option<Ratio>` のように per-kind field を
  LayoutNode に直接持たせる pattern が、複数 field の場合にも
  そのまま scale する」**。当初 `wrap_params: Option<WrapPanelParams>`
  という bundle 型を検討したが、最終的には `aspect` と同じ
  「LayoutNode struct に per-kind field を平坦に並べる」shape を採用。
  メモリオーバヘッドは Option<f32> + f32 + f32 = 16 bytes 弱で
  bundle と実質同じ、しかし bundle にすると T7 で
  `node.wrap_params.as_ref().map(|p| p.item_spacing)` のように
  追加の hop が出る。layout engine 側は `node.item_spacing` で直に
  読めるほうが arrange の loop が読みやすい (per-line 折り返し計算で
  spacing は per-iteration で読む値)。Phase 2 T6 が Box `aspect` で
  既に validate 済みの pattern を素直に再利用した形。
- **「main-axis vs cross-axis の default size constraint は ADR を
  読まないと一意に決まらない」**。VStack は `width = Fill, height =
  Shrink`、HStack は逆の `width = Shrink, height = Fill`、Box は
  `Shrink/Shrink` (aspect で派生)。WrapPanel は DD-M3-P3-005 の
  "outer main-axis equals `parent_main_bound`" 規定から `width = Fill`、
  cross-axis は "sum of line extents + line_spacing × (line_count − 1)"
  から `height = Shrink` が直接導かれる。HStack と width だけ反対
  (HStack は Shrink) なのは、HStack が intrinsic-sized children を
  並べる流儀なのに対し WrapPanel は parent bound を line break に
  使う流儀という、line breaker の有無からくる差。T7 の measure /
  arrange を書くときに「main-axis が Fill であること」が前提に
  なるので、T5 でこれを正しく置くのは load-bearing。

## Checklist

1. **本作業の主要な学び:** あり (記述項目)。
   - **(rev 2 で析出)** progress doc の「defaults are applied at the
     runtime layer in T5」は catalog constructor が absent→default
     を適用する責務を持つ意味であり、loader (T6) の `unwrap_or(0)` に
     委譲しない (Main Learning 第 1 項)。
   - Phase 2 T6 の `#[allow(dead_code)]` + forward-pointer comment
     pattern を Phase 3 T5 にも同形で適用 (variant + constructor +
     3 fields + apply_wrap_panel_defaults helper、計 6 箇所)。
   - LayoutNode に per-kind field を平坦に並べる pattern が複数 field
     にも scale すること、その際に bundle struct を挟まないほうが
     T7 arrange loop の readability が高いこと。
   - WrapPanel default size constraint (`Fill / Shrink`) は
     DD-M3-P3-005 の outer-bounds 規定から直接導出されるため、
     HStack の `Shrink / Fill` ではなく VStack の `Fill / Shrink`
     と一致する。これは line breaker の有無に起因する構造的な差で、
     T7 の measure / arrange の前提として load-bearing。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:**
   **なし**
   - T5 は `wasamo-runtime` 内部 catalog のみ。dsl_spec §4.10 の
     WrapPanel attribute 表面は T1–T4 で draft 済み、本 step での
     再記述は不要 (Moment 2 spec re-sync は T10 の責任範囲)。
     `PropertyValue` / ABI 表面は変えていないので abi_spec も触らない。
     `architecture.md` への影響もなし。

3. **ローカル clean rebuild:** **green** (rev 2; gap-fix commit
   `bb5b2fd` 後に再実行した値)
   - `cargo fmt --all -- --check` (post-commit state; commit
     `bb5b2fd`): zero exit。
   - `cargo clean`: 2923 files removed, 941.9 MiB total。
   - `cargo build --release --workspace`: green (release, 47.86s)。
   - `cargo build --workspace`: green (debug, 46.92s)。
   - `cargo test --workspace`: failure 0 件。
     - `wasamo-runtime` lib: **207 passed** (T4 の 200 から +7:
       WrapPanel variant 形状 3 件 + apply_wrap_panel_defaults 4 件)。
     - `wasamoc` lib: 202 passed (T4 と同じ)。
     - `wasamo-ir`: 12 passed (変化なし)。
     - `wasamo-runtime` integration `ir_loader_roundtrip`: 6 passed。
     - ABI / DLL / binding / counter-rust / gallery-rust crate 群も
       全 green。
   - 初稿 rebuild (commit `fd75ad3` 時点) も同じく green で、
     `wasamo-runtime` lib 203 passed (T4 の 200 から +3) を観測して
     いた。

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - T5 範囲はすべて DD-M3-P3-001 / DD-M3-P3-003 / DD-M3-P3-004 /
     DD-M3-P3-005 の Option A 採択から機械的に降りる。実装細目
     (子の格納位置、constructor の default size constraint、
     `LayoutNode` 平坦 field 採用) はすべて ADR の言葉ないし既存
     widget convention に対応がついており、新たな設計判断を要しない。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - 変更は `wasamo-runtime/src/{layout.rs,widget.rs}` の 2 file
     のみ。既存 widget の constructor / property dispatch /
     hit-testing / mutation API / binding writer / subtree teardown
     には一切触れていない。既存 4 LayoutNode constructor の
     default 値追加 (`item_cross_size: None`, `item_spacing: 0.0`,
     `line_spacing: 0.0`) は新規 field の必然的な機械的拡張で、
     ついでリファクタではない。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - 既存 DD-M3-P3-001 / DD-M3-P3-003 / DD-M3-P3-004 / DD-M3-P3-005 /
     DD-M3-P3-006 で T5 範囲は完全にカバー。実装細目 (子の格納位置、
     constructor の default size constraint、`LayoutNode` 平坦 field
     の choice、`#[allow(dead_code)]` の取り扱い) は spec/DD レベルの
     判断ではない。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格:** **なし**
   - 当該 ADR は全 DD Accepted 済み。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**
   - A3 文言変更なし。Phase 構成変更なし。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **あり (T6 / T7 boundary placeholder)**
   - `cargo build` 出力に新規の dead_code 警告は出ていない
     (`#[allow(dead_code)]` で抑制済み)。
   - ただし以下の **仮実装・近似** が残っている:
     - `layout.rs` の `measure_wrap_panel` は `Ok((0.0, 0.0))` を
       返すだけの placeholder。T7 (DD-M3-P3-005 line-breaker) が
       置き換える。
     - `layout.rs` の `arrange` の `WidgetKind::WrapPanel` arm は
       parent-allocated cell を offset / size に書くだけで、children
       の arrange は呼ばない。T7 が置き換える。
     - `#[allow(dead_code)]` の forward-pointer marker が 6 箇所
       (`WidgetData::WrapPanel` variant / `WidgetNode::wrap_panel`
       関数 / `apply_wrap_panel_defaults` 自由関数 / `LayoutNode` の
       3 field; rev 2 で `apply_wrap_panel_defaults` が増えて 5 →
       6)。T6 (IR loader) と T7 (layout) が完了したら外す。
   - `unimplemented!` / `todo!` stub は置いていない (build / test は
     panic せずに通る)。Phase 2 T6 retro と同じ判断基準: retrospective
     rule の文言 "仮実装・近似" に該当するため **あり** で記録する。

10. **タスクリストの後続 step 見直し:** **なし (進行通り)**
    - T6 / T7 / T8 / T9 / T10 の構成・順序・依存関係に T5 実装から
      見て調整すべき点は出ていない。
    - T6 (IR loader + `validate()`) への follow-up は下記 "Follow-Up"
      節に明示。

## Fast-Track Judgment

Fast-track criteria は **満たさない** (item 9 の "仮実装" 該当)。
memory `feedback_step_end_gate_discipline.md` の規律
("item 2–8 すべて『なし』+ item 3 green に厳密") に従い、item 9 が
「あり」の本 step は fast-track 対象外:

- item 2 (spec doc 変更): なし
- item 3 (local clean rebuild): green
- item 4 (PO 相談事項): なし
- item 5 (ついでリファクタ): なし
- item 6 (追加 DD): なし
- item 7 (Proposed 増加/昇格): なし
- item 8 (m3-plan AC 変更): なし
- item 9 (仮実装・近似・新規 dead_code 警告): **あり** —
  `measure_wrap_panel` placeholder / `arrange` WrapPanel arm no-op /
  6 箇所の `#[allow(dead_code)]` boundary marker (rev 2 で
  `apply_wrap_panel_defaults` が増えて 5 → 6)。Phase 2 T6 と
  同じく T7 (および T6) で置き換えられる boundary placeholder だが、
  retrospective rule の "仮実装・近似" に該当する。
- item 10 (タスクリスト見直し): なし

step→phase ブランチへの ff merge はオーナー明示確認後に実行する。

## Verification Notes

T5 で追加したテストと、走らせた command を記録する (rev 2 状態)。

新規テスト (widget, wasamo-runtime): 7 件
(`wasamo-runtime/src/widget.rs::tests`; 初稿 3 件 + rev 2 で
追加した 4 件):

variant 形状 (初稿 3 件):

- `wrap_panel_variant_carries_three_attributes`
  (`Some(96)` / `8` / `12` の carrier 動作)
- `wrap_panel_variant_defaults_match_constructor_defaults`
  (`None` / `0` / `0` の data shape)
- `wrap_panel_variant_accepts_zero_item_cross_size`
  (DD-M3-P3-006 の zero-handling: `Some(0)` ≠ `None`)

absent→default mapping (rev 2 で追加 4 件):

- `apply_wrap_panel_defaults_maps_all_absent_to_runtime_defaults`
  (`(None, None, None) → (None, 0, 0)`)
- `apply_wrap_panel_defaults_passes_through_present_values`
  (`(Some(96), Some(8), Some(12)) → (Some(96), 8, 12)`)
- `apply_wrap_panel_defaults_handles_each_attribute_independently`
  (`(None, Some(5), None) → (None, 5, 0)`)
- `apply_wrap_panel_defaults_preserves_some_zero_distinct_from_none`
  (`(Some(0), Some(0), Some(0)) → (Some(0), 0, 0)`)

実行コマンド (rev 2; commit `bb5b2fd` 時点):

```text
cargo fmt --all -- --check                 (post-commit state; zero exit)
cargo clean                                (2923 files, 941.9 MiB)
cargo build --release --workspace          (47.86s, green)
cargo build --workspace                    (debug; 46.92s, green)
cargo test --workspace                     (failure 0)
```

いずれも green。`wasamo-runtime` lib test は **207 passed**
(T4 の 200 から +7)、他 crate の test count は T4 と同じ。

初稿時点 (commit `fd75ad3` 後) の rebuild も同じく green で、
`wasamo-runtime` lib 203 passed (T4 の 200 から +3) を観測していた。
初稿 retrospective には `+3` (variant 形状 test のみ) と記録して
いたが、オーナーレビュー指摘を契機に gap fix commit と併せて
`+7` (variant 形状 3 件 + absent→default mapping 4 件) に伸ばし、
本 retrospective を rev 2 として整合させた。

## Follow-Up

T5 から後続 task への明示的な引き渡し:

- **T6 (`wasamo-runtime` IR loader + `validate()` defense-in-depth):**
  `ir_loader::construct_widget` に `"WrapPanel"` arm を追加し、
  3 属性の **presence / absence を `Option<i32>` のまま** 取得して
  `WidgetNode::wrap_panel(compositor, item_cross_size, item_spacing,
  line_spacing)` に渡す (default 適用は T5 で catalog 内に閉じ込め
  済み、loader 側に `unwrap_or(0)` などの default knowledge を持ち
  込まない)。既存 `extract_int_prop` (default 込み) ではなく
  presence を保つ抽出 helper を新設する (T4 emit retrospective でも
  同じ非対称性に言及)。`validate()` の側で `< 0` を
  `WASAMO_ERR_IR_MALFORMED` で reject する arm を追加。T6 完了時点で
  `WidgetData::WrapPanel` variant、`wrap_panel` constructor、
  `apply_wrap_panel_defaults` helper の `#[allow(dead_code)]` が
  自然に外れる。
- **T7 (Layout engine: line-breaker + arrange):** `measure_wrap_panel`
  placeholder (`Ok((0.0, 0.0))`) を DD-M3-P3-005 の bounded /
  unbounded main-axis line-breaker に置き換える。`arrange` の
  `WidgetKind::WrapPanel` arm を children の per-line arrange に
  置き換える。`LayoutNode.item_cross_size` / `item_spacing` /
  `line_spacing` を読むため、これら 3 field の `#[allow(dead_code)]`
  が自然に外れる。free-function 抽出 (per CLAUDE.md §Testing rules)
  か test-only mirror かは T7 内で判断 (`LayoutNode` 自体は
  Win32/WinRT-free なので free-function で書ける見込み)。
- **T8 (Windows-runtime integration test):** `WidgetNode::wrap_panel`
  は `SpriteVisual` を生成するが brush を付けない。container として
  のみ機能する shape は T8 の wrap-path / oversized-child fixture
  で確認する。

これらはすべて progress file の T6 / T7 / T8 として既に列挙済み。
T5 単体で新たに発見された follow-up は無し。
