---
title: M3-Phase 2 phase-end 振り返り
status: recorded
created: 2026-05-20
scope: phase-end
phase: M3-Phase 2 — Box layout primitive
---

# M3-Phase 2 phase-end 振り返り

## 対象範囲

M3-Phase 2 は Box layout primitive を shipped とし、M3 acceptance
criterion A6 を close した。この phase では constant-only な
`aspect: <ratio>` / `fill: <color>` attribute、Box-internal な Ratio /
Color runtime value、IR text support、runtime IR loading / validation、
aspect-aware measure-arrange、Windows-runtime integration evidence、
および Rust host を伴う最初の `examples/gallery/` sub-screen を追加した。

## 主な学び

主な学びは、layout primitive が狭い internal value surface を持っても、
public ABI や generic `TypedValue` deferral に圧をかけずに済む、という
こと。`Ratio` と `Color` を Box-internal に保ったことで、aspect / fill
behavior を証明しつつ、bindable value が本当に必要になった phase でだけ
拡張する余地を残せた。

二つ目の学びは手順面。phase-end spec sync は、ADR 側で normative
chapter の形を先に書いておくと一番うまく進む。T13 では
`dsl_spec.md` §4.9 に draft / implementation divergence は見つからず、
close 作業は late spec invention ではなく、status flip と evidence
distillation になった。

## チェックリスト

1. **本作業の主要な学び:** あり。
   - Box の literal-only internal value により、public ABI churn を避け、
     F5 `TypedValue` deferral への圧もかけずに済んだ。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:**
   **あり**
   - `dsl_spec.md` は planned Phase 2 close marker として変更:
     document version 0.8 および §4.9
     `M3-Phase 2 closed; implementation-synced`.
   - `abi_spec.md` は phase close では変更なし。
   - `architecture.md` は phase close 後、main merge 前の status refresh
     として先頭 status line を `M3-Phase 1 and M3-Phase 2 complete;
     M3-Phase 3 is next` に更新した。これは architecture contract の
     変更ではなく、上位文書の現在地表示の同期。

3. **ローカル clean rebuild:** **green**
   - `cargo fmt --all -- --check`: green.
   - `cargo clean`: success.
   - `cargo build --release --workspace`: green.
   - `cargo build --workspace`: green.
   - `cargo test --workspace`: green。T11
     `aspect_box_with_text_child_lays_out_and_paints_fill`.

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - Phase 2 は Accepted ADR の範囲内に収まった。

### phase-end 固有

11. **Acceptance criteria (Ax) 達成確認:** **達成**
    - A6 は T1-T13 により discharged: Box syntax、IR、checker、
      runtime loader、layout、tests、Windows integration evidence、
      gallery sub-screen、spec sync、phase-end gate。

12. **`CHANGELOG.md` / `ROADMAP.md` 整合:** **整合**
    - `CHANGELOG.md` Unreleased に M3-Phase 2 Box delivery と A6
      discharge を記録済み。
    - `ROADMAP.md` は変更不要。A6 はこの phase の delivered surface を
      既に記述している。

13. **`VISION.md` / thesis-level claim への影響:** **なし**
    - Phase 2 は既存の M3 thesis ("DSL can express real layouts") を
      前進させるが、thesis wording や milestone scope は変更しない。

14. **次 phase の pre-doc への送り込み材料:** **整理済み**
    - `docs/notes/m3-phase-3/predoc-inputs.md` に Box intrinsic sizing、
      placeholder-thumbnail、value-boundary、spec-drafting、
      verification constraint を前送り済み。
    - その後の `docs/notes` 直下 open question audit で、
      architectural-family / layout-engine / TypedValue / grammar /
      verification evidence placement の Phase 3 冒頭チェックも同じ
      file に追記済み。

15. **CI green 確認:** **merge 前 green**
    - T13 は merge 前に final phase-close commit 上の GitHub Actions
      `workflow_dispatch` run で gate 済み。
    - main merge 前の branch verification target
      `89939855129ed77ef9055d6774c5781367fdc317` でも
      `workflow_dispatch` run
      <https://github.com/matarillo/wasamo/actions/runs/26171511748>
      が green。この retrospective evidence update は docs-only なので、
      main push は引き続き owner-gated。

16. **human-visible GUI smoke:** **必要 / 実施済み**
    - T12 で `Start-Process .\target\release\gallery-rust.exe` success
      と、Phase 2 gallery sub-screen の owner-manual screenshot
      confirmation を記録済み。T13 では新しい GUI surface は追加していない。

17. **CI YAML 変更要否 sanity check:** **不要**
    - Phase 2 は新しい language / build system を追加していない。既存の
      Windows CI `cargo test --workspace` が T11 integration evidence を
      実行し、CI 上で Compositor path が走れない場合は fail する。

## 検証メモ

T13 中に実行した local command:

```text
cargo fmt --all -- --check
cargo clean
cargo build --release --workspace
cargo build --workspace
cargo test --workspace
```

すべて green。workspace build / test では既知の `wasamo`
linkable-target warning と `wasamo-sys` import-library order warning が
残ったが、これは prior T12 evidence と同じ。

## フォローアップ

- M3-Phase 3 は
  `docs/notes/m3-phase-3/predoc-inputs.md` から開始する。
- Phase 2 progress file は T13 checklist confirmation 後に retired。
  durable information は ADR、spec、CHANGELOG、notes、git history に
  移した。
