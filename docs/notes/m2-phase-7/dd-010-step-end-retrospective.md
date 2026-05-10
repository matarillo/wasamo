---
title: M2-Phase 7 / DD-M2-P6-010 step-end retrospective
status: recorded
created: 2026-05-09
scope: step-end
dd: DD-M2-P6-010
---

# M2-Phase 7 / DD-M2-P6-010 step-end retrospective

## Scope

Step-end retrospective for DD-M2-P6-010 (`dirty_effects` topological
sort fidelity), after replacing the `EffectId` numeric-order approximation
in `wasamo-runtime/src/reactive.rs`.

This is a step-end retrospective, not a phase-end retrospective. It is the
gate before merging this DD-010 step into the Phase 7 branch.

## Main Learning

The decisive implementation fact was that the existing `ReactiveGraph`
contained read dependencies only:

- `forward: SignalId -> EffectId set` says which Effects read a Signal.
- `back: EffectId -> SignalId set` says which Signals an Effect reads.

That pair is sufficient for invalidation, but not sufficient to derive the
topological edge needed by DD-010: "Effect A writes Signal X, Effect B reads
Signal X, therefore A must run before B when both are dirty." The shipped
runtime therefore needed an explicit `writes: EffectId -> SignalId set`
edge map.

The useful pattern was to preserve the DD's main architectural intent
(a free function over graph borrows, no ABI / Win32 / Compositor state)
while expanding the graph input set to include the missing write edge.
The implementation is now structurally ordered by the runtime dependency
graph rather than by Effect allocation order.

## Checklist

1. **本作業の主要な学び:** あり。Read graph と write graph は別物であり、
   topological dirty-Effect ordering needs explicit write tracking.
2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:**
   なし。
3. **ローカル clean rebuild:** green.
   - `cargo clean`: green.
   - `cargo build --workspace`: green.
   - `cargo build --release --workspace`: green.
   - `cargo test --workspace`: green.
   - Observed warnings: existing `wasamo` / `wasamo_sys` link-target /
     import-library ordering warnings; no build or test failure.
4. **PO に相談すべき設計判断・トレードオフ:** あり -> resolved.
   - ADR の required form names `&forward` / `&back` / dirty set (or
     equivalent borrows). Implementation adds `&writes` because the
     existing maps cannot encode writer-to-reader edges. This looks like
     a faithful implementation of the dependency-graph walk, but it is
     still a design-visible adjustment and should be owner-reviewed
     before fast-forward merge.
   - Owner disposition: accept path 2, with the condition that the ADR
     records the minor update according to existing decision-record
     practice. The ADR now records the write-edge borrow as a minor
     implementation clarification under DD-M2-P6-010.
5. **plan/ADR step 目的から外れた「ついで」のリファクタ・構造変更:** なし.
   - `ReactiveGraph::writes` is in-scope support for the accepted DD-010
     implementation, not an opportunistic cleanup.
6. **現在の phase ADR への追加 DD 必要性:** なし.
   - No new decision topic was discovered beyond the already-recorded
     M3 residuals for cycle policy, ordering ties, and fan-out /
     `MUTATION_CAP`.
7. **既存 ADR の Proposed 項目の新規追加、または Proposed -> Accepted 昇格:**
   なし.
8. **`m2-plan.md` の AC 追加・変更、または Phase 構成変更:** なし.
9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:** なし.
   - Cycle handling remains the DD-010 documented M3 residual, not a new
     step-created obligation.
   - No new `dead_code` warning was observed.
10. **タスクリストの後続 step 見直し:** 不要.
    - DD-012 and DD-011 sequencing remains as recorded in the Phase 7
      progress file.

## Fast-Track Judgment

Fast-track criteria were **not satisfied** at first because checklist
item 4 was "あり". The owner reviewed the issue and accepted path 2:
keep the implementation and reconcile the ADR by recording a minor
implementation clarification.

The reviewed item was narrow: whether adding `ReactiveGraph::writes` as
a graph borrow is acceptable as the implementation of DD-010's required
free-function form, or whether the ADR text should be reconciled to
mention write edges explicitly. The chosen disposition is ADR/code
alignment: DD-010's required implementation form now names the write-edge
map explicitly.

## Verification Notes

The DD-010 implementation added both pure-logic tests for the extracted
topological walk and a production-path regression:

- chain: `a -> b -> c`.
- diamond: `a -> {b, c} -> d`.
- fan-out wider than `MUTATION_CAP`.
- out-of-ID-order dependency: larger-ID upstream writer before smaller-ID
  downstream reader.
- production path: dirty downstream Effect has smaller ID than upstream
  writer and must not observe the stale intermediate value.

Commands run for this retrospective:

```text
cargo clean
cargo build --workspace
cargo build --release --workspace
cargo test --workspace
```

All completed successfully.

## Follow-Up

The `writes`-edge adjustment has been owner-reviewed and reconciled in
the Phase 7 ADR as a minor implementation clarification. No additional
DD is needed unless later work changes the ordering contract itself.
