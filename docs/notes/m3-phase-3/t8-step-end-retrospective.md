---
title: M3-Phase 3 / T8 step-end retrospective
status: recorded
created: 2026-05-22
scope: step-end
task: T8 — Windows-runtime integration test (ADR §Phase 3 verification closure evidence item 4)
---

# M3-Phase 3 / T8 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-3-progress.md` の **T8**
(ADR §Phase 3 verification closure evidence item 4 — Windows-runtime
layout integration evidence; CI-gated Compositor pipeline) の
step-end retrospective。

対象コミット (2 件; 本 doc 含む):

- `2a812e3 test(wasamo-runtime): add WrapPanel layout integration evidence (M3-Phase 3 T8)`
- (this commit) `docs(m3-phase-3): close T8 layout integration step` (本
  retrospective + progress checkbox flip)

T8 が discharge した材料は次:

- `wasamo-runtime/tests/wrap_panel_layout_integration.rs` を追加し、
  `.ui → wasamoc → IR → build_widget_tree → live WidgetNode →
  run_layout` の production path を mock-free に駆動する 2 fixture を
  実装。
  - `wrap_path_fixture_lays_out_multi_line_thumbnails` — 88×88
    の Box thumbnail 6 枚 を `item-cross-size: 88; item-spacing:
    12; line-spacing: 12` の WrapPanel に並べ、300×400 parent で
    2 行 3 列にラップすることを assert (outer 300×188、各 child
    88×88、各 child の (x, y) 配置)。
  - `oversized_child_fixture_paints_visible_overflow` —
    `aspect: 4:1` で `item-cross-size: 50` の Box を main bound
    100 の WrapPanel に置き、WrapPanel rect が main = 100 に
    留まること、child rect が width = 200 で `x + width > 100`
    の visible overflow を持つこと、WrapPanel の Composition
    visual に Clip が設定されない (`Visual::Clip()` が `Err`)
    ことを assert。
- ADR §Phase 3 verification closure evidence item 4 は **2 fixture
  両方の存在** を要求しているため、1 file 内に 2 `#[test]` を置く
  分割を採用 (helper / skip-guard / runtime init の共有あり)。
- skip-guard は Phase 2 T11 と同じ pattern: local では
  `0x80070005` を runtime-compositor-unavailable として skip、
  GitHub Actions では skip path に入ったら fail する `assert!`
  を維持。

step-end の gate であり phase-end retrospective ではない。merge 先は
phase ブランチ `feat/m3-phase-3` (ff)。本 step (T8) は単一 task =
単一 step 構造で、現在のブランチは `feat/m3-phase-3-t8`。

## Current Judgment

2026-05-22 時点で T8 step-end 基準は **達成済み**。

- progress file T8 checklist 3 項目はすべて `[x]` に flip 済み。
- 2 fixture とも本 Windows 開発環境 (Compositor available) で
  skip ではなく実行され、pass:
  `test result: ok. 2 passed; 0 failed`。
- clean rebuild gate (post-commit HEAD = `2a812e3`) も green:
  `cargo clean` → `cargo build --release --workspace` →
  `cargo build --workspace` → `cargo test --workspace` →
  `cargo fmt --all -- --check`。既存の `wasamo-sys` import library
  order warning は表示されたが T8 由来の warning / failure はなし。
- SSH dev box 上で skip-guard の triggering half が
  `0x80070005` (`アクセスが拒否されました。`) を検出して両 test を
  skip path で pass させることを観測済み (詳細は Verification Notes)。

## Main Learning

中心的な学びは「**WrapPanel の visible-overflow 規約は pure layout
契約だけでなく Composition visual の Clip 非設置によっても定義され、
その 'Clip 不在' は production の visual tree から読み戻して観測する
ことで end-to-end 性が成り立つ**」。

ADR の verification closure item 4 は「**WrapPanel installs no clip
surface on its Composition visual**」を spec'd "visible overflow,
parent clips" 規約の runtime-side 立証として要求していた。pure-logic
の line-breaker + arrange test (item 2) は数値上の overflow は十分に
pin できる (`child.x + child.w > wp.x + wp.w`) が、Composition
visual に Clip が後段で installed されないことの保証は別の観測点が
必要で、`Visual::Clip()` が `Err` を返す ('no clip installed') こと
を assert することで初めて item 2 と item 4 が独立した evidence と
して機能した。

T11 (Phase 2) の Box fill assertion が
`CompositionColorBrush.Color()` を読み戻したのと同じ精神で、本 step
では「visible overflow を許す側」も visual tree の負の事実
('Clip がない') で確認する pattern を初導入した。今後の Phase で
WrapPanel が ScrollView や clip-aware container の中に置かれた際の
回帰は、同じ `Visual::Clip()` 読み戻しを WrapPanel 親側に拡張する
ことで自然に検出できる。

副次的に、DSL syntax の child separator は `;` ではなく改行であり、
複合 widget literal を 1 行に圧縮するときの落とし穴を 1 回踏んだ
(初稿の `Box { aspect: 1:1; fill: #336699cc; ... }` は parser に
"expected member, found `;`" で reject された)。同じ literal を
`box_layout_integration.rs` の既存 pattern に揃え、複数行構文へ
書き直して green に到達。

## Checklist

1. **本作業の主要な学び:** あり。
   - WrapPanel の visible-overflow 規約は visual tree の
     "Clip 不在" を含めて初めて end-to-end 立証され、
     `Visual::Clip()` の `Err` 観測が item 2 (pure layout) と
     item 4 (visual tree) を独立 evidence として接続する
     (Main Learning に展開)。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`)
   の変更:** **なし** **(FT)**
   - T8 は test evidence + progress / retrospective 記録のみ。
   - dsl_spec.md §4.10 の Phase status marker flip および
     architecture.md §6.8 reconcile は T10 (Phase-end gates) の
     責務。

3. **ローカル clean rebuild:** **green** **(FT)**
   - `cargo fmt --all -- --check`: zero exit (post-commit HEAD
     `2a812e3` で確認)。
   - `cargo clean`: success (3423 files / 1.1GiB 削除)。
   - `cargo build --release --workspace`: success (54.41s)。
   - `cargo build --workspace`: success (49.99s; 既存の
     `wasamo-sys` import library order warning のみ)。
   - `cargo test --workspace`: success (workspace 全 test green)。
   - 追加 test 単体:
     `cargo test -p wasamo-runtime --release --test wrap_panel_layout_integration`:
     2 passed。

4. **PO に相談すべき設計判断・トレードオフ:** **なし** **(FT)**
   - T8 checklist の範囲内で実装。新規 design decision は発生
     なし (Clip 不在 assertion は ADR §verification closure item 4
     の文言が直接要求していたため、解釈余地なし)。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし** **(FT)**
   - 新規 test file (`wrap_panel_layout_integration.rs`) と
     progress / retrospective 記録のみ。

6. **現在の phase ADR への追加 DD 必要性:** **なし** **(FT)**
   - Accepted DD の解釈変更なし。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed →
   Accepted への昇格:** **なし** **(FT)**
   - 当該 ADR は T6 closing 時点で全 DD Accepted 済み。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし** **(FT)**
   - A3 文言変更なし。Phase 構成変更なし。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **なし** **(FT)**
   - `todo!` / `unimplemented!` / 新規 `#[allow(dead_code)]` は
     導入していない。

10. **タスクリストの後続 step 見直し:** **不要**
    - T9 (gallery sub-screen additive growth) と T10 (phase-end
      gates) は現行順序で進行可。

## Fast-Track Judgment

Fast-track criteria は **満たす**:

- item 2: なし
- item 3: green
- item 4: なし
- item 5: なし
- item 6: なし
- item 7: なし
- item 8: なし
- item 9: なし
- item 10: 不要

本作業では merge は実行していない。step→phase の扱いはオーナーの
次アクションに従う。

## Verification Notes

実行コマンド (post-commit HEAD `2a812e3` 上):

```text
cargo test -p wasamo-runtime --release --test wrap_panel_layout_integration
cargo fmt --all -- --check
cargo clean
cargo build --release --workspace
cargo build --workspace
cargo test --workspace
cargo fmt --all -- --check
```

すべて green。`cargo build --release --workspace` /
`cargo build --workspace` / `cargo test --workspace` では既存の
`wasamo-sys` import library order warning が表示されたが、T8 由来の
warning / failure はなし。

skip-guard の動作:

- 本 Windows 開発環境 (Compositor available) では `wasamo_init` が
  `WASAMO_OK` を返し、両 test の body が実行されて pass。
- SSH dev box 上で
  `cargo test -p wasamo-runtime --test wrap_panel_layout_integration -- --nocapture`
  を実行し、
  `wasamo_init: アクセスが拒否されました。 (0x80070005)` を
  runtime compositor unavailable として検出して
  `skipping WrapPanel wrap-path integration test: runtime compositor unavailable (...)`
  および
  `skipping WrapPanel oversized-child integration test: runtime compositor unavailable (...)`
  を stderr に出し、両 test が skip path で pass
  (`test result: ok. 2 passed; 0 failed`) することを観測。これにより
  skip-guard の triggering half (Compositor 不在環境で実際に skip path
  に入る) が立証された。
- GitHub Actions では同じ skip path を skip せず fail する assert を
  維持 (`!github_actions()` ガードによる; CI green は phase-end T10
  で `workflow_dispatch` から改めて確認する)。

## Follow-Up

T8 から新たに発生した out-of-phase residual はなし。

- **T9:** gallery example sub-screen の additive growth に進める
  (WrapPanel × 1:1 Box thumbnails 5–10 枚; framing decision E)。
- **T10:** phase-end gates で full workspace / CI / spec status
  marker flip / architecture.md §6.8 reconcile を実施。
