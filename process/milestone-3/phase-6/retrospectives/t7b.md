---
phase: M3-Phase 6
task: T7b
title: Component-root window-attribute boundary
date: 2026-06-08
scope: task-end
merge_target: feat/m3-phase-6
---

# T7b Retrospective

Task branch: `feat/m3-phase-6-T7b` (to be merged to
`feat/m3-phase-6` after explicit owner approval).

Refs (branch `feat/m3-phase-6-T7b`):

- `22dd09e` — T7b A2a `IrComponent.host_props` / `host_bindings` migration,
  compiler host catalog, runtime mirror validation, old root-squatted IR
  rejection, and canonical tests (**codex**).
- `c361a3a` — review-response: host diagnostic precision (compiler) +
  `host_bindings` emit pin + gallery validate-through-loader + audit table
  (**claude**).
- (this branch tip) — full-independent-review (Codex) response: runtime host
  catalog **value-shape mirror** (`backdrop` / `theme` typed-literal reject +
  ABI malformed coverage) and review-gate disposition (**claude**).

## Checklist (task-end, items 1-11)

1. **Main learning.** The root-cause fix was smaller once framed as
   "content root purity" rather than "ZStack exception handling": after
   `IrComponent` gained a host-owned surface, the temporary ZStack root
   allowlist disappeared, title resolution moved to the component surface,
   and the same catalog rule served both the compiler and runtime gates.
   The useful guardrail was making old root-squatted IR reject explicitly;
   otherwise the migration could have silently preserved the debt it was
   meant to remove.
2. **Specification document changes:** none in this task. DD-M3-P6-008
   explicitly assigns `docs/dsl_spec.md` / `docs/architecture.md` sync to T9
   Moment 2; `docs/abi_spec.md` remains untouched because the change is an
   internal compiler-IR / textual-IR representation change, not a C ABI
   surface.
3. **Post-change verification:** green (recorded in
   [log.md](../implementation/log.md)).
   - `cargo fmt --all -- --check` — green.
   - `cargo clean` — completed (`5603 files, 1.5GiB` removed).
   - `cargo build --release --workspace` — green.
   - `cargo build --workspace` — green.
   - `cargo test --workspace` — green.
   - Scoped checks: `cargo test -p wasamo-ir`, `cargo test -p wasamoc
     --lib`, `cargo test -p wasamo-runtime --lib ir_loader::tests`,
     `cargo test -p wasamo-runtime --test ir_loader_roundtrip`, `cargo test
     -p wasamo-runtime --test abi_load_ui`, `cargo run -p wasamoc -- check
     examples\gallery\gallery.ui`, and `cargo run -p wasamoc -- build
     examples\gallery\gallery.ui` — green.
   - Existing Cargo warnings about the `wasamo` linkable target /
     `wasamo-sys` import-library ordering were observed.
4. **Design judgments / trade-offs to consult PO:** none. The owner already
   selected DD-M3-P6-008 A2a and the Phase-6 host-binding policy (structural
   surface present; no bindable host attributes admitted this phase).
5. **Out-of-task refactors or structural changes:** none. The changes are
   the planned schema / textual-IR migration and its tests.
6. **Need for an additional DD:** none. DD-M3-P6-008 already owns this
   boundary and is Accepted.
7. **Existing-ADR Proposed item / Proposed -> Accepted promotion:** none.
8. **Milestone-plan AC / phase-shape additions or changes:** none. T7b
   closes the inserted DD-008 implementation slot; T8 still owns the
   owner-visible smoke after this final validator / gallery behavior lands.
9. **Carry-over temporary implementation / approximations / new
   `dead_code`:** none. `host_bindings` is deliberately structural but
   rejected by the Phase-6 host catalog; that is the accepted policy, not a
   temporary fallback. No new `dead_code` warning was introduced.
10. **New cross-task / cross-phase design constraints:** **yes —
    `phase-sync`.**
    - **Constraint:** host-owned attributes must remain structurally
      separated from the content root; M4 may replace the carrier, but should
      preserve that separation.
    - **Evidence:** T7b's migration moved `title` / `backdrop` / `theme` to
      `IrComponent.host_props`, removed the ZStack root exemption, and added
      runtime rejects for old root-squatted host props / bindings.
    - **Placement:** `phase-sync` — T9 Moment 2 already owns the
      `docs/dsl_spec.md` / `docs/architecture.md` fold for
      `host_props` / `host_bindings` and the host-owned-attributes vs
      content-root separation. Re-trigger criterion: any M4/M5 work adding
      host/base attributes, dynamic host bindings, base-name validation, or
      an ABI-facing window descriptor must re-check that host attributes are
      not stored on the content root.
11. **Downstream-task revision:** no new revision needed. Existing plan
    ownership is correct: T8 runs after T7b and owns owner-visible smoke /
    additive visible-correctness fixes; T9 owns Moment 2 spec / architecture
    sync and phase-close gates.

## Implementation-Gate Closure

- **#1 semantic migration:** call-site audit and classifications are recorded
  in [log.md](../implementation/log.md). The migration sites were
  `wasamo-ir` schema, `wasamoc` lower / emit / check, runtime parse /
  validate / title resolution / ZStack validation, ABI malformed-title test,
  and cross-crate IR roundtrip tests.
- **#2 missed side effects:** static title resolution now reads
  `host_props`; root-squatted host props / bindings are rejected; the ZStack
  root no longer has a Window-attribute exemption; ABI signatures stayed
  unchanged. **Review-found side effect (F1, fixed):** the runtime defensive
  mirror must mirror the catalog's *value shape*, not only its attribute-name
  set — `validate_host_surface` now rejects a typed-scalar literal on
  `backdrop` / `theme` (keyword identifiers only), closing the
  `host prop backdrop = 3` direct-textual-IR hole the first cut left open.
- **#4 branch tests:** compiler host accept / unknown-host reject /
  host-binding reject, lower no-splice, emit/parse canonical shape, runtime
  catalog mirror, unknown host prop, host binding, non-string host title,
  old root-squatted prop/binding rejection, and ABI malformed canonical
  host-title cases are all directly tested. Review-response additions
  (narrow branch/test tier): a wrong-typed static literal on a host attribute
  now reports a literal-type diagnostic distinct from the dynamic
  "not bindable" one (`component_level_host_title_non_string_literal_reports_string_requirement`,
  `component_level_host_backdrop_typed_literal_rejected`); the `host_bindings`
  emit half is pinned (`host_binding_emitted_on_component_surface`); and the
  real gallery IR is re-validated through the runtime loader headlessly
  (`gallery_ui_emits_and_validates_through_runtime_loader`) since T7b rewrote
  the validator T7's GUI evidence depended on. See
  [log.md](../implementation/log.md) 2026-06-08 review-response entry.
  Full-independent-review (Codex) addition: runtime value-shape mirror reject
  (`host_surface_rejects_typed_literal_backdrop` / `..._theme`), its positive
  control (`host_surface_accepts_keyword_backdrop_and_theme`), and the ABI
  malformed `host prop backdrop = 3` case.
- **#5 carry-forward:** item 10 records the M4-facing separation invariant as
  `phase-sync`; T9 Moment 2 is the owning fold point.
- **#6 deterministic-failure disposition:** no recurring or vanished runtime
  failure occurred. Expected red tests from old assertions were updated to
  the A2a canonical shape rather than re-run to green.

**Review lane — full independent review, performed.** Because this is a
schema / textual-IR migration the lane is full independent review. It was
performed by **Codex** (independent agent) plus the in-session Claude review;
disposition recorded in [log.md](../implementation/log.md) under
"2026-06-08 / T7b full independent review (Codex) — disposition" (finding F1
runtime value-shape mirror fixed in-task; F2 gate-closure made explicit; F3
Refs updated). The "narrow branch/test review tier" noted earlier applied
only to the diagnostic/test delta, not to the migration. Merge remains
gated on explicit owner approval.

## Evidence Pointers

- DD: [DD-M3-P6-008](../decisions/dd-m3-p6-008-component-root-window-attribute-boundary.md).
- Implementation plan: [plan.md](../implementation/plan.md) T7b.
- Verification index and gate artifacts:
  [implementation/log.md](../implementation/log.md).

## Merge Readiness

Checklist complete; local fmt / build / workspace tests are green. Per
[retrospectives.md §進行手順](../../../procedures/retrospectives.md),
checklist completion is **not** merge authorization — the no-ff merge to
`feat/m3-phase-6` awaits explicit owner approval. Push remains a separate
phase-end gate.
