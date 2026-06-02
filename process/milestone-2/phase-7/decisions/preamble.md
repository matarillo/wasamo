# M2-Phase 7 — Reactive Foundation Hardening & Contract Finalization: Architecture Decisions

**Phase:** M2-Phase 7 (Reactive Foundation Hardening & Contract Finalization)
**Date:** 2026-05-08 (ADR opened; DDs remain Proposed pending per-DD pre-doc cycles); 2026-05-09 (DD-M2-P6-010 Accepted; DD-M2-P6-010 minor implementation clarification recorded); 2026-05-10 (DD-M2-P6-012 Accepted; DD-M2-P6-011 Accepted); 2026-05-11 (M2 completed)
**Status:** Accepted (DD-M2-P6-010 / 011 / 012)

## Context

M2 acceptance criteria **A5** (Reactive Foundation Hardening) and **A6**
(Type-Agnostic Reactive Binding) — added by the 2026-05-08 acceptance-
criteria revision recorded in
[m2-plan.md](../../plan.md#progress) — are discharged by this
phase. Phase 6 closed A1/A2 (counter `.ui` drives the running counter
through the reactive path end-to-end); Phase 7 closes A5/A6 by
upgrading the foundation guarantees that distinguish "the pipeline
runs" from "the pipeline is a Foundation other layers can rely on".

This ADR houses three DDs that were drafted as part of the Phase 6 ADR
slate but deferred at Phase 6 closing because their resolution depends
on full Phase 6 implementation evidence rather than mid-implementation
judgement:

- **DD-M2-P6-010** — `dirty_effects` topological sort fidelity. Surfaced
  during the DD-M2-P6-001 implementation retrospective (2026-05-07).
- **DD-M2-P6-011** — String-typed property binding. Surfaced during the
  DD-M2-P6-007 implementation step (2026-05-07).
- **DD-M2-P6-012** — Re-entrancy and safety-guard placement principle.
  Surfaced from the Phase 5 retrospective; deliberately deferred so the
  full Phase 6 set of re-entrancy states (Diverged / IN_DRAIN /
  IN_OBSERVER_CALLBACK / UI-thread confinement) is available as
  evidence.

The DDs retain their original `DD-M2-P6-NNN` numbering. The number is a
historical identifier that records when the issue surfaced; the ADR
file is the housing for resolution. Decoupling the two avoids
renumbering churn across `docs/plans/m2-plan.md`, prior memory, and
git history.

### Order of work (agreed 2026-05-08)

Per the Phase 7 entry in [m2-plan.md](../../plan.md), the three
DDs are processed sequentially as independent pre-doc cycles, in the
order:

1. **DD-M2-P6-010** (topo sort fidelity) — pre-doc → agreement →
   Accepted → implementation. Discharges the Phase 6 closing
   constraint that any multi-binding work must replace the
   `EffectId`-numeric-order approximation.
2. **DD-M2-P6-012** (guard placement principle) — pre-doc → agreement
   → Accepted → `architecture.md` update; the principle is recorded
   as a global runtime invariant. Discharges A5.
3. **DD-M2-P6-011** (String-typed property binding) — pre-doc →
   agreement → Accepted → implementation. Discharges A6 with an
   end-to-end `Signal<String>` binding demonstration.

The order reflects a deliberate sequencing choice: 010 settles the
ordering primitive that A6's binding evaluator depends on; 012
establishes the placement principle that any new evaluator code in
011 should follow; 011 lands last so it benefits from both. The
sequence is not implied by the DD numbers and is not reversible
without re-pre-doc.

### Side obligations carried in

- DD-M2-P6-012 acceptance must update `docs/architecture.md` to record
  the chosen guard-placement principle as a global runtime invariant.
  This update lands in the same commit that flips DD-M2-P6-012 to
  `Accepted`.
- DD-M2-P6-010's mandatory-pre-condition language (currently in the
  Phase 6 ADR's Forward-compat exposure paragraph) is reconciled at
  010 acceptance. Under the recommended Option A, the constraint is
  discharged at acceptance time (the walk lands in M2; M3 inherits it).
  The reconciled text replaces the Phase 6 paragraph in the same
  commit that flips DD-010 to `Accepted`.

---

## Summary of accepted decisions

| ID | Topic | Recommendation | Impl risk | Forward-compat exposure |
|---|---|---|---|---|
| DD-M2-P6-010 *(Accepted 2026-05-09)* | `dirty_effects` topological sort fidelity | **Option A** — true topological walk in M2, extracted as a free function with pure-logic unit tests. A5 (literal reading) discharged by implementation; M3 inherits the verified primitive. Options B / C / C-lite recorded as considered, not recommended. | Low–medium (Option A) | Discharged at acceptance; M3 residuals (cycle / ties / fan-out) recorded in m2-to-m3-handover.md |
| DD-M2-P6-011 *(Accepted 2026-05-10)* | String-typed property binding | **Option B** — `StrPropRead` HandlerExpr variant. A6 is discharged demonstratively: M2 proves `.ui` String binding through runtime widget property state while preserving existing integer `PropRead` behavior. Option C `TypedValue` unification is deferred to a post-M2 open question. | Low (Option B) | Low for M2; later typed-expression pressure tracked in typed-value-evaluator.md |
| DD-M2-P6-012 *(Accepted 2026-05-10)* | Re-entrancy and safety-guard placement principle | **Option C** — role-specified defense in depth. ABI boundary owns caller-facing diagnostics; internal runtime boundary owns invariant enforcement for ABI-bypassing entries; cleanup exceptions are explicit. Option D typed tokens deferred as a M3+ revisit trigger. | Low-medium (focused implementation alignment and tests) | Low-medium; M3 timer / async-I/O / windowproc surfaces inherit the rule |

**Aggregate shipped picture.** The three DDs stayed narrowly scoped:
010 replaced the production dirty-Effect ordering path with a true graph
walk; 011 added an additive `HandlerExpr` variant plus String read methods
while preserving the existing integer path; 012 settled the guard-placement
principle and aligned the visible M2 gaps with focused tests. With these
implemented, M2's A5/A6 acceptance criteria are discharged. The broader
typed-value rewrite remains outside M2 unless later DSL/tooling evidence
reopens it.

**Aggregate forward-compat exposure.** All three DDs have explicit
successor work or revisit triggers — 010's M3 residuals after the
topological primitive is discharged, 011's post-M2 typed-value open
question, and 012's application across timer / async-I/O / windowproc
surfaces in M3.

## Out of scope

- **General topological-graph diagnostics tooling.** DD-010's true
  graph walk is in scope (under Option A) or scheduled (under Option
  B); a tool that visualises the dependency graph or its SCCs is
  post-M2.
- **`f32` / `bool` / aggregate-typed property binding.** DD-011 covers
  `i32` and `String` only. Additional scalar types are post-M2 work.
  They are one possible trigger for revisiting Option C, but they are
  not forced into M3 unless M3's DSL surface or public spec draft
  creates real type-system pressure.
- **`wasamo_post_event` and timer / async-I/O ABI.** DD-012's option
  set considers M3 timer and async-I/O entry paths as constraints on
  the principle; designing those entry paths themselves is M3.
- **Compile-time guard-token enforcement across the entire runtime.**
  DD-012 Option D is in scope as an option to evaluate; if rejected,
  it is post-M2. If accepted, its rollout sequencing across
  `reactive`, `emit`, `registry`, `window` is task-level detail
  recorded in `m2-plan.md`'s Progress section, not in this ADR.
- **Multi-Effect-per-Signal `.ui` constructs.** Out of scope for M2
  regardless of which DD-010 option is chosen; their introduction is
  M3's multi-binding work item.

## Provenance

This ADR was opened on 2026-05-08 to house the three DDs that the
Phase 6 ADR
([m2-phase-6-ui-lowering.md](../../phase-6/decisions/preamble.md)) carried in
its draft slate but did not Accept. The Phase 6 ADR retains stub
entries at the DD section anchors that forward to this file; the
Phase 6 ADR itself remains `Accepted` for DD-M2-P6-001..009.

The acceptance-criteria revision that introduced A5/A6 and scoped them
to Phase 7 is recorded in the Progress section of
[m2-plan.md](../../plan.md) under the 2026-05-08 entry; the
Phase 7 entry there names this ADR as the housing for DD-010/011/012.

### Minor implementation clarifications

- **2026-05-09 — DD-M2-P6-010 write-edge borrow.** During DD-010
  implementation, `ReactiveGraph::forward` / `back` were confirmed to
  encode read dependencies only. They are sufficient for invalidation
  but not for deriving the Effect-to-Effect ordering edge "writer runs
  before reader when both are dirty." The required implementation form
  was therefore clarified to name the write-edge map as an explicit
  graph borrow. This does not supersede DD-010 or change Option A; it
  records the concrete graph input needed to implement the accepted
  topological walk faithfully.
