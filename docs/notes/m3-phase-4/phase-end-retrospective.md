---
title: M3-Phase 4 phase-end 振り返り
status: recorded
created: 2026-05-25
scope: phase-end
phase: M3-Phase 4 — ScrollView primitive (minimal)
---

# M3-Phase 4 phase-end 振り返り

## 対象範囲

M3-Phase 4 は ScrollView primitive (minimal) を shipped とし、M3
acceptance criterion A5 (ScrollView minimal: inner unbounded measure +
viewport clip + content offset binding) を close した。この phase では:

- `wasamoc check` の ScrollView surface / diagnostics (T1)
- pure-data layout engine の ScrollView measure-arrange と clamp
  semantics (T2)
- runtime IR loader / `validate()` defense-in-depth と widget
  materialisation (T3)
- Windows-runtime Visual tree evidence: outer clipped Visual +
  ScrollView-owned intermediate content Visual + `parent_abs_offset`
  shift / R2 closure (T4)
- gallery `.ui` sub-screen growth and assistant-side build / launch
  evidence (T5)
- owner-manual visible smoke and the T6 window-root Fill/Fill fix
  bundle after the first smoke failed (T6)
- Moment 2 spec / architecture / plan re-sync, T7 step-end retro, and
  Phase 5 pre-doc carry-forward material (T7)

## 主な学び

主な学びは、**runtime-boundary の root sizing と production root shape
を integration fixture で pin しないと、pure-layout / direct-root
fixture が green でも gallery path が崩れる**こと。T4 fixture は
ScrollView を component root に置いていたが、production `.ui` は
VStack root を踏む。T6 owner smoke で root VStack の default
`height: Shrink` が Fill ScrollView child を 0-height に collapse し、
viewport clip が空になる failure mode A が可視化された。対応として
`WidgetNode::run_layout_as_window_root` を導入し、window root の
LayoutNode を client rect Fill/Fill として扱う runtime-boundary 規約を
`architecture.md §6.3` に fold した。

二つ目の学びは、**ScrollView の intermediate Visual は
`sync_visuals()` の parent-relative offset convention に新しい局所規約
を要求する**こと。intermediate Visual が `(0, -applied_y, 0)` を
持つため、content subtree へ渡す `parent_abs_offset` も
`(scrollview_abs_x, scrollview_abs_y - applied_y)` に shift しないと、
layout tree の root-relative offsets と Composition tree の
parent-relative offsets が二重適用または欠落する。T7 でこの規約を
`architecture.md §6.5` に一般則として fold した。

三つ目の学びは、**step-end と phase-end は retrospective / progress
bullet を物理的に分離すると reviewer friction が下がる**こと。T7
step-end retro は checklist items 1-11、phase-end retro は items
12-18 であり、所有 branch / merge gate も違う。Phase 4 はこの分離を
明示した最初の phase になったため、Phase 5 pre-doc input §5 に
carry-forward した。

## チェックリスト

12. **Acceptance criteria (Ax) 達成確認:** **達成**
    - A5 は T1-T6 により discharged。ADR §Phase 4 verification
      closure の items 1-4 は automated / CI-gated evidence として
      landed: wasamoc surface (T1), layout engine (T2), IR loader /
      validate defense-in-depth (T3), Windows-runtime Visual evidence
      including R2 closure (T4)。item 5 は T5 (gallery `.ui` +
      assistant build / launch) と T6 (owner-manual smoke + fix
      iteration) で discharged。
    - A11 の Phase 4 slice は T7 Moment 2 re-sync と T6
      owner-acceptanceで discharged。`docs/dsl_spec.md` §4.11 は
      `M3-Phase 4 closed; implementation-synced`、`architecture.md`
      は Phase 4 complete と runtime-boundary / Visual-layer
      divergences を fold 済み。
    - ADR の "discharged" 表記と実装の乖離は無し。Phase 4 ADR 自体は
      DD slate / verification closure / out-of-scope をすでに保持して
      おり、retrospectives.md §phase-sync の ADR-touch three cases の
      うち、追加 touch が必須なケースは無い。R1 window-title wiring は
      progress file §Out-of-phase residuals と Phase 5 pre-doc input §4
      に登録済みで、ADR への追加 cross-reference は不要と判断。

13. **`CHANGELOG.md` / `ROADMAP.md` 整合:** **整合**
    - `CHANGELOG.md` Unreleased に
      `### M3-Phase 4 — ScrollView primitive (minimal) (2026-05-25)`
      entry を追加。entry は A5 discharge、generic IR / no-new-ABI
      surface、wasamoc / runtime / Visual-layer changes、gallery
      visible proof、T6 window-root fix、Moment 2 spec sync、R1
      residual を要約。
    - `ROADMAP.md` は変更不要。A5 は ScrollView minimal (inner
      unbounded measure + viewport clip + content offset binding) を
      既に明示しており、Phase 4 はそのまま operationalise した。

14. **`VISION.md` / thesis-level claim への影響:** **なし**
    - Phase 4 は既存の M3 thesis ("DSL can express real layouts") を
      前進させるが、VISION の thesis wording や milestone scope は
      変更しない。ScrollView の Phase 4 surface は M3 target-app
      scope 内であり、M4+ input-driven scrolling / scrollbar / write-
      back は既存の defer に収まる。

15. **次 phase の pre-doc への送り込み材料:** **整理済み**
    - `docs/notes/m3-phase-5/predoc-inputs.md` に Phase 4 close 由来
      の carry-forward を action-oriented に整理済み:
      production root shape を integration fixture parent として
      カバーする rule、non-root Shrink container + Fill child の設計
      空間、M4 handoff としての `scroll_y` Signal drift、R1
      Window-title wiring owning-phase assignment、phase-final step の
      step-end / phase-end retrospective 分離 rule。
    - T4/T5/T6 の phase-sync item は T7 Moment 2 で `architecture.md`
      / `dsl_spec.md` に fold 済み。open の `phase-sync` item は残って
      いない。

16. **CI green 確認:** **local clean rebuild green; GitHub Actions run pending replacement**
    - Local clean rebuild on `feat/m3-phase-4` after T7 merge and
      phase-end doc updates:
      `cargo fmt --all -- --check` green; `cargo clean` removed 2547
      files / 915.8 MiB; `cargo build --release --workspace` green
      (44.17 s); `cargo build --workspace` green (36.21 s);
      `cargo test --workspace` green (`wasamo-runtime` lib 258 passed,
      `scroll_view_layout_integration` 3 passed,
      `wrap_panel_layout_integration` 2 passed, `wasamoc` lib 227
      passed, workspace failure 0).
    - GitHub Actions `workflow_dispatch` evidence: **PENDING_CI_RUN**.
      This placeholder is replaced after the phase-end doc commit is
      pushed and the run completes on `feat/m3-phase-4`.
    - main no-ff merge / push remains a separate owner-gated action
      after this phase-end gate.

17. **human-visible GUI smoke:** **必要 / Phase 4 内で実施済み**
    - T6 owner-manual GUI smoke on the rebuilt `gallery-rust.exe`
      returned green on all four observation points: viewport clip
      sharp, +100 / -100 Buttons move content, clipped content remains
      hidden outside the ScrollView rect, and off-viewport thumbnails
      enter as `scroll_y` progresses. Evidence images are committed
      under `docs/references/m3-phase-4/`.
    - Phase-end work after T6 is doc / verification only and adds no new
      GUI surface, so no additional visible smoke is required at
      phase-end.

18. **CI YAML 変更要否 sanity check:** **不要**
    - Phase 4 adds no new language, external build system, or CI matrix
      dimension. Existing Windows CI `cargo test --workspace` covers
      the new ScrollView Windows-runtime integration tests, and the
      local phase-end clean rebuild confirmed those tests still pass.

## 検証メモ

Phase-end 中に実行した local command:

```text
cargo fmt --all -- --check
cargo clean
cargo build --release --workspace
cargo build --workspace
cargo test --workspace
```

すべて green。既知 warning (`wasamo` non-linkable target、
`wasamo-sys` import-library order) は Phase 2 / Phase 3 / T7 close と
同じ。

GitHub Actions:

```text
PENDING_CI_RUN
```

## フォローアップ

- Phase 5 pre-doc は
  [`docs/notes/m3-phase-5/predoc-inputs.md`](../m3-phase-5/predoc-inputs.md)
  から開始する。
- R1 Window-title wiring は Phase 4 内では closed しない。次 gate は
  M3-Phase 5 pre-doc framing で owning phase を assign すること。
- Phase 4 progress file は `status: closing` のまま維持する。per
  `docs/plans/README.md` lifecycle と T7 progress note の通り、
  `closing` → `retired` は phase → main merge commit / post-merge
  distillation の所有物であり、本 phase-end retrospective commit では
  実行しない。
