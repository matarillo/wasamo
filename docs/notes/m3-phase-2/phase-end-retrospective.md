---
title: M3-Phase 2 phase-end retrospective
status: recorded
created: 2026-05-20
scope: phase-end
phase: M3-Phase 2 — Box layout primitive
---

# M3-Phase 2 phase-end retrospective

## Scope

M3-Phase 2 shipped the Box layout primitive and closes M3 acceptance
criterion A6. The phase added constant-only `aspect: <ratio>` and
`fill: <color>` attributes, Box-internal Ratio / Color runtime values,
IR text support, runtime IR loading and validation, aspect-aware
measure-arrange, Windows-runtime integration evidence, and the first
`examples/gallery/` sub-screen with a Rust host.

## Main Learning

The main learning is that a layout primitive can own a narrow internal
value surface without pressuring the public ABI or the generic
`TypedValue` deferral. Keeping `Ratio` and `Color` Box-internal let the
phase prove aspect and fill behavior while preserving the later option
to add bindable values only when a phase actually needs them.

The second learning is procedural: phase-end spec sync works best when
the ADR has already written the normative chapter shape. T13 found no
draft / implementation divergence in `dsl_spec.md` §4.9; the close work
was a status flip and evidence distillation rather than late spec
invention.

## Checklist

1. **本作業の主要な学び:** あり。
   - Box's literal-only internal values avoided public ABI churn and
     kept the F5 `TypedValue` deferral unpressured.

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:**
   **あり**
   - `dsl_spec.md` changed by the planned Phase 2 close marker:
     document version 0.8 and §4.9
     `M3-Phase 2 closed; implementation-synced`.
   - `abi_spec.md` and `architecture.md` did not change at phase close.

3. **ローカル clean rebuild:** **green**
   - `cargo fmt --all -- --check`: green.
   - `cargo clean`: succeeded.
   - `cargo build --release --workspace`: green.
   - `cargo build --workspace`: green.
   - `cargo test --workspace`: green, including T11
     `aspect_box_with_text_child_lays_out_and_paints_fill`.

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - Phase 2 stayed within the Accepted ADR.

### phase-end 固有

11. **Acceptance criteria (Ax) 達成確認:** **達成**
    - A6 is discharged by T1-T13: Box syntax, IR, checker, runtime
      loader, layout, tests, Windows integration evidence, gallery
      sub-screen, spec sync, and phase-end gates.

12. **`CHANGELOG.md` / `ROADMAP.md` 整合:** **整合**
    - `CHANGELOG.md` Unreleased now records the M3-Phase 2 Box
      delivery and A6 discharge.
    - `ROADMAP.md` required no change; A6 already described this
      phase's delivered surface.

13. **`VISION.md` / thesis-level claim への影響:** **なし**
    - Phase 2 advances the existing M3 thesis ("DSL can express real
      layouts") but does not change the thesis wording or milestone
      scope.

14. **次 phase の pre-doc への送り込み材料:** **整理済み**
    - `docs/notes/m3-phase-3/predoc-inputs.md` carries forward Box
      intrinsic sizing, placeholder-thumbnail, value-boundary,
      spec-drafting, and verification constraints.

15. **CI green 確認:** **green before merge**
    - T13 is gated by a GitHub Actions `workflow_dispatch` run on the
      final phase-close commit before merge. Main push remains
      owner-gated.

16. **human-visible GUI smoke:** **必要 / 実施済み**
    - T12 recorded `Start-Process .\target\release\gallery-rust.exe`
      success and owner-manual screenshot confirmation of the Phase 2
      gallery sub-screen. No new GUI surface was added in T13.

17. **CI YAML 変更要否 sanity check:** **不要**
    - Phase 2 added no new language or build system. The existing
      Windows CI `cargo test --workspace` runs the T11 integration
      evidence and fails on CI if the Compositor path cannot run.

## Verification Notes

Local commands run during T13:

```text
cargo fmt --all -- --check
cargo clean
cargo build --release --workspace
cargo build --workspace
cargo test --workspace
```

All local commands were green. Workspace build / test retained the known
`wasamo` linkable-target warning and `wasamo-sys` import-library order
warning, matching prior T12 evidence.

## Follow-Up

- M3-Phase 3 should start from
  `docs/notes/m3-phase-3/predoc-inputs.md`.
- The Phase 2 progress file is retired after T13 checklist confirmation;
  durable information now lives in the ADR, spec, CHANGELOG, notes, and
  git history.
