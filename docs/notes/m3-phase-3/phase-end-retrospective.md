---
title: M3-Phase 3 phase-end 振り返り
status: recorded
created: 2026-05-22
scope: phase-end
phase: M3-Phase 3 — WrapPanel layout primitive
---

# M3-Phase 3 phase-end 振り返り

## 対象範囲

M3-Phase 3 は WrapPanel layout primitive を shipped とし、M3
acceptance criterion A3 (gallery overflow / wrapping evidence) の
WrapPanel 構成要素を close した。この phase では:

- kebab-case attribute name (`item-cross-size` / `item-spacing` /
  `line-spacing`) と negative `IntLit` を扱うための lexer 拡張 (T1)
- `wasamoc check` の WrapPanel 受理 + reject set + aspect-only-Box
  warning (T1 / T2)
- AST → IR lowering と IR text emit (T3 / T4) — production
  コード変更ゼロで generic 経路の汎用性を test として固定
- `WidgetData::WrapPanel` 追加と runtime IR loader / `validate()`
  defense-in-depth (T5 / T6)
- 二段階 measure-arrange の line-breaker / arrange 実装 (T7)
- Windows-runtime integration test (clip-absence assertion + visible
  overflow regulation) (T8)
- `examples/gallery/` を Box single-screen から WrapPanel of 10
  thumbnails (canonical 88 / 12 / 12) へ additive growth (T9)
- `sync_visuals` 境界の parent-relative offset 変換 bug fix (T9 内
  fix; T10 で architecture §6.5 に 1 行 fold)
- phase-end spec / architecture sync (T10)

## 主な学び

主な学びは、**WrapPanel が"新しい parser 文法を導入しない"という
framing は parser 層では正しかったが、lexer surface (kebab Ident、
negative IntLit) と Composition 境界 (parent-relative offset 変換)
の両方で暗黙の前提を要した**こと。前者は T1 で発覚して spec close
時に §2.2 へ fold、後者は T9 visible-smoke で発覚して T10 で
architecture §6.5 に 1 行 fold した。どちらも「framing decision の
裏側にある暗黙 surface を、phase-end の re-sync で明示化する」という
共通の構造を持つ。

二つ目の学びは、**novel-normative measure-arrange を ADR で先に書き
切ったことが T7 実装の drift 検出を後押しした**こと。DD-M3-P3-005 の
bounded / unbounded 分岐は ADR 上で明示されており、T7 の rev 2 で
発見された `compute_wrap_lines` の cross-axis-extent drift bug は、
helper 抽出だけでは整合保証されないことを reject test で pin する
形で fix した。free-function 抽出は call-site の責務を消さない、
という Phase 3 固有の戒め (Phase 4 pre-doc input §6 に前送り)。

三つ目の学びは、**verification closure item の evidence 形式は ADR
で先に分離して書く**こと。T8 の Windows-runtime integration test は
pure-logic line-breaker test (item 2) では証明できない "Composition
visual に Clip を installation しない" を立証するもので、ADR
verification closure item 4 が evidence 形式を spec 化していたため、
T8 は実装着手時に何を assert すれば close できるかが既に定まって
いた。

## チェックリスト

1. **本作業の主要な学び:** あり。
   - 上記「主な学び」3 点。Phase 4 pre-doc input への前送り済み
     ([m3-phase-4/pre-doc-inputs.md](../m3-phase-4/pre-doc-inputs.md))。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:**
   **あり**
   - `dsl_spec.md` は Phase 3 close marker + lexer-surface fold:
     document version 0.9 → 1.0、§4.10
     `M3-Phase 3 closed; implementation-synced`、§2.2 の `Ident`
     pattern を kebab-case 継続込みに、`IntLit` pattern に optional
     leading `-` を、表直下に「negative-sign surface は `IntLit`-only
     (FloatLit / measurement / RatioLit には及ばない、`-`
     operator も導入しない)」の 1 行 note。§5 AST shape は
     `IntLit { value: i64 }` で既に signed surface を保持していた
     ため変更なし。
   - `architecture.md` は Status を `M3-Phase 1, M3-Phase 2, and
     M3-Phase 3 complete` に flip。§6.5 (`WidgetNode` and Visual
     Layer sync) に `LayoutNode` 絶対座標と `sync_visuals` の
     parent-relative `Visual.Offset` 変換規約を 1 行追加 (T9 由来
     R3-A fold)。
   - `abi_spec.md` は phase close では変更なし (Phase 3 は新規
     `WASAMO_VALUE_*` / `WASAMO_LAYOUT_ERROR_*` tag を追加しない)。

3. **ローカル clean rebuild:** **green**
   - `cargo fmt --all -- --check`: green.
   - `cargo clean`: success (2482 files / 884.1 MiB removed).
   - `cargo build --release --workspace`: green.
   - `cargo build --workspace`: green.
   - `cargo test --workspace`: green (workspace 全 test 通過。T8
     Windows-runtime integration test を含む)。
   - 既知 warning: `wasamo` non-linkable target warning と
     `wasamo-sys` import-library ordering warning は Phase 2 close
     時点と同じ。

4. **PO に相談すべき設計判断・トレードオフ:** **あり**
   - T1 Decisions log で flag されていた「lexer change (kebab Ident
     + negative IntLit) を dsl_spec §2 / §5 に明記するか」を Moment
     2 spec re-sync 時 (本 T10) に inline 提示し、owner 選択
     "A' = §2.2 に両方明記、§5 触らず" を採用。
   - T9 由来の R3 候補 (architecture §6.5 への offset 規約 1 行
     追記) を residual に回すか fold するかを inline 提示し、
     owner 選択 "R3-A = T10 architecture commit で fold (別 commit
     で review 単位を分ける)" を採用。

### phase-end 固有

11. **Acceptance criteria (Ax) 達成確認:** **達成**
    - ADR §Phase 3 verification closure の evidence item 1–5 は
      T1–T9 で discharged: (1) sub-screen positive control =
      `examples/gallery/` 88 / 12 / 12 + 10 thumbs、(2) pure-logic
      line-breaker / arrange tests = T7 unit test、(3) IR loader
      defense-in-depth = T6 `validate()`、(4) Windows-runtime
      clip-absence + visible-overflow fixture = T8、(5) visible
      smoke = T9 `Start-Process .\target\release\gallery-rust.exe`
      success + owner-manual visual confirmation。

12. **`CHANGELOG.md` / `ROADMAP.md` 整合:** **保留**
    - `CHANGELOG.md` Unreleased への M3-Phase 3 delivery 記録は
      phase-end main-merge gate のタイミングで本 retrospective と
      合わせて owner レビューに含める前提。
    - `ROADMAP.md` は変更不要。A3 は WrapPanel を含む overflow /
      wrapping evidence として既に記述されており、Phase 3 はその
      構成要素 (WrapPanel 単独) を ship した。

13. **`VISION.md` / thesis-level claim への影響:** **なし**
    - Phase 3 は既存の M3 thesis ("DSL can express real layouts")
      を前進させるが、thesis wording や milestone scope は変更
      しない。WrapPanel は ADR-accepted scope 内で完結。

14. **次 phase の pre-doc への送り込み材料:** **整理済み**
    - `docs/notes/m3-phase-4/pre-doc-inputs.md` (13 sections) に
      Phase 3 T1–T9 の Main Learning を action-oriented に前送り
      済み。ScrollView (minimal) の novel-normative
      measure-arrange、no-new-IR-variant claim の test 固定、
      defaults の widget-catalog 層責務、free-function 抽出の
      call-site drift discipline、clip surface as WrapPanel-T8
      inverse、pure-layout / Composition offset 境界、gallery
      growth path、process continuity (inline option presentation、
      fast-track / merge gate)、live-note audit triggers を含む。

15. **CI green 確認:** **phase branch ff merge + push 後に
    `workflow_dispatch` で実行予定**
    - T10 commit set を `feat/m3-phase-3-t10` から
      `feat/m3-phase-3` へ ff merge し、push 後に
      `workflow_dispatch` から CI を回す。CI URL は green 確認後に
      本 retrospective へ folder。
    - main push は phase-end gate の owner 明示承認後 (本 retro と
      別 session) に行う。

16. **human-visible GUI smoke:** **不要 / Phase 3 内で実施済み**
    - T9 で `Start-Process .\target\release\gallery-rust.exe`
      success と owner-manual screenshot confirmation を記録済み
      (`sync_visuals` 修正後の rev 2 状態)。T10 では新しい GUI
      surface を追加していない (docs / spec sync のみ) ため再 smoke
      は不要。

17. **CI YAML 変更要否 sanity check:** **不要**
    - Phase 3 は新しい language / build system を追加していない。
      既存の Windows CI `cargo test --workspace` が T8 integration
      evidence を実行し、Compositor unavailable 時は skip ではなく
      fail する skip-guard を T8 で SSH dev box 上で verify 済み。

## 検証メモ

T10 中に実行した local command:

```text
cargo fmt --all -- --check
cargo clean
cargo build --release --workspace
cargo build --workspace
cargo test --workspace
```

すべて green。既知 warning (`wasamo` non-linkable target、
`wasamo-sys` import-library order) は Phase 2 close 時点と同じ。

## フォローアップ

- M3-Phase 4 (ScrollView minimal) は
  [`docs/notes/m3-phase-4/pre-doc-inputs.md`](../m3-phase-4/pre-doc-inputs.md)
  から開始する。
- Phase 3 progress file (`docs/plans/progress/m3-phase-3-progress.md`)
  は T10 checklist 完全 flip 後に `status: closing` →
  ([retired] phase-end main-merge gate 完了時に retired) に進める。
  durable information は ADR、spec、architecture、CHANGELOG、本
  retrospective、git history に移した。
- Phase 3 由来の out-of-phase residual:
  - **R1** `.gitignore` `*.uic` pattern — cross-cutting build
    hygiene; Phase 4 以降の任意 step で fold 可能。
  - **R2** `sync_visuals` ↔ pure-layout boundary test gap — Phase
    4 ScrollView の content offset 機構が同境界に依存するため、
    Phase 4 pre-doc input §8 で land 検討を明示している。
  - R3 候補 (architecture §6.5 への offset 規約追記) は T10 で
    R3-A として fold 完了 (residual ではない)。
- 次 session の入口は `feat/m3-phase-3` 上で CI green 確認 → owner
  への phase-end main-merge gate 報告。main no-ff merge + push は
  owner 明示承認後。
