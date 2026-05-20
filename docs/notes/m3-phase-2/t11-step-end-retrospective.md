---
title: M3-Phase 2 / T11 step-end retrospective
status: recorded
created: 2026-05-20
scope: step-end
task: T11 — Windows-runtime layout integration test (ADR §Phase 2 verification closure item 3)
---

# M3-Phase 2 / T11 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-2-progress.md` の **T11**
(ADR §Phase 2 verification closure item 3 — Windows-runtime layout
integration evidence) の step-end retrospective。

対象コミット:

- `216cb5e test(wasamo-runtime): add Box layout integration evidence
  (M3-Phase 2 T11)`

T11 が discharge した材料は次:

- `wasamo-runtime/tests/box_layout_integration.rs` を追加し、
  `.ui → wasamoc → IR → build_widget_tree → live WidgetNode →
  run_layout` の production path を mock-free に駆動。
- `Box { aspect: 16:9; fill: #336699cc; Text { text: "Photo 12" } }`
  を既知の 800x800 parent extent に layout し、Box の
  `SpriteVisual` rectangle が 800x450、Text child がその矩形内で
  centered になることを assert。
- `fill` は `WidgetNode::__box_state_for_test` と `SpriteVisual`
  の `CompositionColorBrush` から観測し、`wasamo_get_property` は
  使わない。
- Compositor unavailable (`0x80070005`) は T10 と同じ local skip /
  GitHub Actions fail pattern。

これは step-end の gate であり、phase-end retrospective ではない。

## Current Judgment

2026-05-20 時点で T11 step-end 基準は **達成済み**。

- progress file T11 checklist 3 項目はすべて `[x]` に flip 済み。
- 追加 test はこの Windows 開発環境で skip ではなく実行され、
  `aspect_box_with_text_child_lays_out_and_paints_fill` が pass。
- `cargo test -p wasamo-runtime` も pass し、既存 Windows-only
  integration tests との干渉なし。
- clean rebuild gate も pass:
  `cargo clean` → `cargo build --release --workspace` →
  `cargo build --workspace` → `cargo test --workspace` →
  `cargo fmt --all -- --check`。
  build 時に既存の `wasamo` linkable target warning と
  `wasamo-sys` import library order warning は出たが、failure は 0。

## Main Learning

中心的な学びは「**layout integration evidence は pure layout result
だけでなく、production visual tree に sync された後の状態を読むと
ADR closure item と実際の runtime behavior がきれいに接続される**」。

T8 の pure-logic tests は `LayoutNode` の数値契約を十分に pin して
いたが、T11 の責務はその数値が live `WidgetNode` / `SpriteVisual`
に反映されることの確認だった。したがって、同じ 16:9 の
inscribed-fit を再assertするだけではなく、

- Box root の `Visual.Offset` / `Visual.Size`
- Text child の `Visual.Offset` / `Visual.Size`
- Box fill の `CompositionColorBrush.Color`

を読み戻すことで、`build_layout_tree` → `layout::run_layout` →
`sync_visuals` → Composition object state までを一本につないだ。

副次的に、T10 で追加した `__box_state_for_test` は T11 でも有効な
観測点だったが、T11 では render model 側の brush も確認するほうが
「fill が本当に visual に materialise された」ことを強く示せた。
今後の Windows-runtime closure item でも、内部 accessor だけで
足りるか、render object まで読むべきかを evidence の性質で選ぶ。

## Checklist

1. **本作業の主要な学び:** あり。
   - live visual tree から rectangle / brush を読むことで、pure
     layout と runtime rendering model の接続を executable にできる
     (Main Learning に展開)。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`)
   の変更:** **なし**
   - T11 は test evidence と progress / retrospective 記録のみ。
   - dsl_spec.md §4.9 の Phase status marker flip は T13 の責務。

3. **ローカル clean rebuild:** **green**
   - `cargo fmt --all -- --check`: zero exit。
   - `cargo clean`: success。
   - `cargo build --release --workspace`: success。
   - `cargo build --workspace`: success。
   - `cargo test --workspace`: success。
   - 追加 test 単体:
     `cargo test -p wasamo-runtime --test box_layout_integration`: pass。
   - runtime crate:
     `cargo test -p wasamo-runtime`: pass。

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - T11 checklist の範囲内で実装。新規 design decision は発生なし。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - 新規 test file と progress / retrospective 記録のみ。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - Accepted DD の解釈変更なし。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed →
   Accepted への昇格:** **なし**
   - 当該 ADR は全 DD Accepted 済み。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**
   - A6 / A11 文言変更なし。Phase 構成変更なし。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **なし**
   - `todo!` / `unimplemented!` / 新規 `#[allow(dead_code)]` は導入
     していない。

10. **タスクリストの後続 step 見直し:** **不要**
    - T12 (gallery seed) と T13 (phase-end gates) は現行順序で進行可。

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

実行コマンド:

```text
cargo test -p wasamo-runtime --test box_layout_integration
cargo fmt --all -- --check
cargo test -p wasamo-runtime
cargo clean
cargo build --release --workspace
cargo build --workspace
cargo test --workspace
cargo fmt --all -- --check
```

すべて green。`cargo build --release --workspace` /
`cargo build --workspace` / `cargo test --workspace` では既存の
`wasamo` linkable target warning と `wasamo-sys` import library
order warning が表示されたが、T11 由来の warning / failure はなし。

skip-guard の動作:

- 本 Windows 開発環境では `wasamo_init` が `WASAMO_OK` を返し、
  test body が実行されて pass。
- `0x80070005` の local skip branch は T10 で観測済みの pattern を
  同形で再利用。GitHub Actions では skip せず fail する assert を維持。

## Follow-Up

T11 から新たに発生した out-of-phase residual はなし。

- **T12:** gallery example seed に進める。
- **T13:** phase-end gates で full workspace / CI / spec status marker
  flip を改めて確認する。
