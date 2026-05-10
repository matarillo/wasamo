---
title: M2-Phase 7 / DD-M2-P6-012 step-end retrospective
status: recorded
created: 2026-05-10
scope: step-end
dd: DD-M2-P6-012
---

# M2-Phase 7 / DD-M2-P6-012 step-end retrospective

## Scope

Step-end retrospective for DD-M2-P6-012 (re-entrancy and safety-guard
placement principle), after aligning the existing runtime guards with the
accepted Option C split:

- ABI boundary = diagnostic boundary.
- Internal runtime boundary = invariant boundary.
- Runtime-owned non-ABI entries are first-class runtime entries.
- Cleanup / destroy paths remain explicit exceptions.

This is a step-end retrospective, not a phase-end retrospective. It is the
gate before merging the DD-012 implementation step into the Phase 7 branch.

## Main Learning

The useful implementation fact was that most of the accepted Option C shape
already existed in code, but it was not fully fixed by tests:

- `abi.rs` already concentrated caller-facing status codes and
  `wasamo_last_error_message` text at exported `wasamo_*` entry points.
- `emit::drain_if_outermost()` already served as the internal boundary for
  non-ABI message-loop entry, suppressing nested drains while `IN_DRAIN` is
  set and suppressing all phases after `RuntimeHealth::Diverged`.

The step therefore stayed small: preserve the existing structure, add focused
guard-placement tests, and close the one visible ABI-boundary gap. The gap was
the void lifecycle entries `wasamo_run` and `wasamo_quit`: because they cannot
return a `WasamoStatus`, they now follow the same pattern as wrong-thread
void calls by recording diagnostics in last-error and returning as a no-op
when the runtime has diverged.

The cleanup-exception rule was also testable without creating real Win32 /
Composition objects: null `wasamo_window_destroy` and null
`wasamo_widget_destroy` remain allowed after divergence, which fixes the
exception policy without relying on OS-side destruction.

## Checklist

1. **本作業の主要な学び:** あり。DD-012 Option C was mostly an alignment and
   verification step, not a broad guard-token rewrite. The one code change
   needed was to make void lifecycle ABI entries (`wasamo_run` /
   `wasamo_quit`) participate in the divergence diagnostic boundary.
2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:**
   なし。
   - `architecture.md` had already recorded the DD-012 principle at
     acceptance time.
   - This step only updated `docs/plans/progress/m2-phase-7-progress.md`.
3. **ローカル clean rebuild:** green.
   - `cargo clean`: green.
   - `cargo build --release --workspace`: green.
   - `cargo build --workspace`: green.
   - `cargo test --workspace`: green.
   - Observed warnings: existing `wasamo` crate-type warning and existing
     `wasamo-sys` import-library ordering warning after clean builds; no
     build or test failure.
4. **PO に相談すべき設計判断・トレードオフ:** なし。
   - The implementation stayed inside accepted Option C.
   - No new placement rule or cleanup exception was introduced.
5. **plan/ADR step 目的から外れた「ついで」のリファクタ・構造変更:** なし.
   - Changes were limited to guard-placement tests, the void lifecycle
     divergence guard, a test-only health setter, and Phase 7 progress
     bookkeeping.
6. **現在の phase ADR への追加 DD 必要性:** なし.
   - Option D typed guard tokens remain a recorded M3+ revisit trigger; this
     step did not create a new decision topic.
7. **既存 ADR の Proposed 項目の新規追加、または Proposed -> Accepted 昇格:**
   なし.
   - DD-M2-P6-012 was already Accepted before implementation.
   - DD-M2-P6-011 remains Proposed.
8. **`m2-plan.md` の AC 追加・変更、または Phase 構成変更:** なし.
9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:** なし.
   - `wasamo_run` / `wasamo_quit` now reject post-divergence entry as void
     no-ops with diagnostics.
   - `drain_if_outermost` divergence suppression and re-entrant no-op
     behavior are covered by unit tests.
   - No new `dead_code` warning was observed.
10. **タスクリストの後続 step 見直し:** 不要.
    - DD-010 and DD-012 are now implemented.
    - The next step remains DD-M2-P6-011 pre-doc / acceptance work for A6.

## Fast-Track Judgment

Fast-track criteria were **satisfied** for this step:

- Item 2: none.
- Item 3: green.
- Item 4: none.
- Item 5: none.
- Item 6: none.
- Item 7: none.
- Item 8: none.
- Item 9: none.

No owner-facing design decision, specification change, AC change, or new
technical debt was created by the implementation step.

## Verification Notes

The DD-012 implementation added focused guard-placement coverage:

- Internal invariant boundary:
  - `drain_if_outermost()` suppresses all phases after divergence and leaves
    queued callback work untouched.
  - Re-entrant `drain_if_outermost()` while `IN_DRAIN` is set is a no-op and
    leaves work for the outer drain.
- ABI diagnostic boundary:
  - `wasamo_run()` and `wasamo_quit()` produce last-error diagnostics and
    return as no-ops after divergence.
  - Null `wasamo_window_destroy()` and null `wasamo_widget_destroy()` remain
    explicit cleanup exceptions after divergence.

Commands run for this retrospective:

```text
cargo clean
cargo build --release --workspace
cargo build --workspace
cargo test --workspace
```

All completed successfully.

## Follow-Up

DD-M2-P6-011 can resume on top of the accepted and implemented DD-010 /
DD-012 foundation. New evaluator or binding code added for A6 should follow
the DD-012 split: public ABI entry points own diagnostics, while any future
runtime-owned non-ABI entry must cross an internal invariant boundary.
