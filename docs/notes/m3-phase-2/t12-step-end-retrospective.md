---
title: M3-Phase 2 / T12 step-end retrospective
status: recorded
created: 2026-05-20
scope: step-end
task: T12 — Seed examples/gallery/ + examples/gallery-rust/ (ADR §Phase 2 verification closure item 4)
---

# M3-Phase 2 / T12 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-2-progress.md` の **T12**
(ADR §Phase 2 verification closure item 4 — gallery seed and visible
launch evidence) の step-end retrospective。

対象コミット:

- `83126b7 feat(examples): seed M3 Phase 2 gallery host`

T12 が discharge した材料は次:

- `examples/gallery/gallery.ui` を追加し、Phase 2 の最小 gallery
  sub-screen として `Box { aspect: 16:9; fill: #336699cc; Text { ... } }`
  を置いた。
- `examples/gallery-rust/` を workspace member として追加し、
  `build.rs` が `.ui` を `wasamoc` 経由で IR text に変換し、
  `main.rs` が `wasamo_load_ui` でロードする Phase 1
  `bool-demo-rust` と同じ build pipeline にした。
- `cargo build -p gallery-rust`、`cargo build --release -p gallery-rust`、
  `Start-Process .\target\release\gallery-rust.exe` が成功。
- C / Zig hosts は Phase 2 では追加しない、という framing decision F
  と ADR out-of-scope を progress checklist 上で明示的に閉じた。

これは step-end の gate であり、phase-end retrospective ではない。

## Current Judgment

2026-05-20 時点で T12 step-end 基準は **達成済み**。

- progress file T12 checklist 4 項目はすべて `[x]` に flip 済み。
- gallery UI は `Box` の aspect / fill / child Text を通るため、ADR
  verification closure item 4 の visible proof surface を持つ。
- Rust host は workspace member として登録され、Cargo.lock も
  `gallery-rust` package を含む。
- `Start-Process` は command 成功まで確認済み。visual correctness は
  pre-doc framing decision G の通り owner-manual GUI smoke。

## Main Learning

中心的な学びは「**gallery seed は canonical examples の延長ではなく、
以後の M3 surface を足していくための専用の受け皿として作ると、
phase ごとの visible proof が迷子になりにくい**」。

Phase 1 では foundational exception として `bool-demo-rust` が
gallery-sub-screen substitute だったが、Phase 2 では framing decision F
に従い、直接 `examples/gallery/` と `examples/gallery-rust/` を作った。
これにより、今後の Phase 3+ は `counter-*` や `bool-demo-*` の意味を
薄めずに、gallery に sub-screen を追加していける。

## Checklist

1. **本作業の主要な学び:** あり。
   - gallery は visible proof の累積場所として独立させるほうが、
     canonical examples の責務を広げずにすむ (Main Learning に展開)。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`)
   の変更:** **なし**
   - T12 は examples / progress / retrospective の追加のみ。
   - dsl_spec.md §4.9 の Phase status marker flip は T13 の責務。

3. **ローカル clean rebuild:** **green**
   - `cargo fmt --all -- --check`: zero exit。
   - `cargo clean`: initial run failed because the just-launched
     `gallery-rust.exe` held `target\release\gallery-rust.exe`; after
     the GUI process exited, rerun succeeded。
   - `cargo build --release --workspace`: success。
   - `cargo build --workspace`: success。
   - `cargo test --workspace`: success。
   - T12 追加 host 単体:
     `cargo build -p gallery-rust`: success。
   - T12 追加 host release:
     `cargo build --release -p gallery-rust`: success。

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - framing decision F / G と ADR out-of-scope に沿った実装。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - 新規 gallery fixture / Rust host / workspace registration のみ。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - Accepted DD の解釈変更なし。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed →
   Accepted への昇格:** **なし**
   - 当該 ADR は全 DD Accepted 済み。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**
   - A6 closure item 4 の evidence を追加しただけで、AC / phase
     構成は変更していない。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **なし**
   - `todo!` / `unimplemented!` / 新規 `#[allow(dead_code)]` は導入
     していない。

10. **タスクリストの後続 step 見直し:** **不要**
    - T13 (phase-end gates) は現行順序で進行可。

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
cargo build -p gallery-rust
cargo build --release -p gallery-rust
Start-Process .\target\release\gallery-rust.exe
cargo fmt --all -- --check
cargo clean
cargo clean
cargo build --release --workspace
cargo build --workspace
cargo test --workspace
cargo fmt --all -- --check
```

最初の `cargo clean` は起動中の `gallery-rust.exe` が release binary
を保持していたため失敗し、プロセス終了後の rerun は success。その後の
workspace build / test / fmt check はすべて green。workspace build /
test では既存の `wasamo` linkable target warning と `wasamo-sys`
import library order warning が表示されたが、T12 由来の warning /
failure はなし。

`Start-Process` は command 成功を確認済み。visual correctness は
owner-manual GUI smoke に委ねる。

## Follow-Up

T12 から新たに発生した out-of-phase residual はなし。

- **T13:** phase-end gates で full workspace / CI / spec status marker
  flip / phase-end retrospective に進む。
