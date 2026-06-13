---
phase: M3-Phase 7
title: Iteration grammar
status: active
adr: process/milestone-3/phase-7/decisions/preamble.md
plan: process/milestone-3/plan.md
opened: 2026-06-13
---

# M3-Phase 7 — Iteration grammar: Implementation

This is the live task list and execution framing for M3-Phase 7. The
design decisions are frozen in the ADR set under
[../decisions/](../decisions/preamble.md) (preamble + DD-M3-P7-001
through DD-M3-P7-007, Status: Accepted on 2026-06-13). This file and
its sibling [plan.md](./plan.md) are mutable during the phase per
[../../../README.md §SSOT distribution](../../../README.md#ssot-distribution);
the in-flight decisions log and CI evidence land in
[log.md](./log.md) as the phase progresses. Cross-phase residuals
land in [handoff.md](./handoff.md) at phase close.

## Phase 7 scope

Phase 7 ships the **iteration grammar**: a runtime-mutable collection
`state` drives the number of generated widget subtrees (A8 — the
cardinality generalisation of Phase 6's presence construct), advancing
every side per **A11** and landing the normative iteration chapter per
**A12**, with the WrapPanel + ScrollView-backed thumbnail sub-screen as
the **A1** gallery slice. The decisions and their rationale are frozen
in the [ADR §Decisions table](../decisions/preamble.md#decisions); this
list is only *what each task builds* (DD pointers, not a
re-derivation):

- **Author surface** — `for <binder> ("," <index-binder>)? in
  <collection> { <one widget child> }`; `in` reserved; per-container
  admission sweep (DD-M3-P7-001).
- **Collection value & mutation surface** — `i32[]` / `string[]` /
  `bool[]` state types + list literals; `IrStateType::Scalar |
  Collection` + `IrLiteral::List` (an IR-schema change); whole-value
  per-element-type collection signals; whole-value assignment over
  pure `append` / `drop-last` expressions + static-literal reset /
  clear; `TypedValue` judged not-adopted (DD-M3-P7-002).
- **Loop-local context** — author-named binders, expression-position
  reads only, flat scope, collision errors; handlers inside a `for`
  body rejected (DD-M3-P7-003).
- **IR / traversal** — `ControlFlowNode::For`; textual-IR `for`
  member; the **canonized member-expansion seam** (prefix-sum over
  per-member live cardinality, shared by static load and reactive
  mutation; the Phase 6 conditional path migrates onto it);
  `BindingTarget::ForLoopSubtree`; trap-#1 call-site audit
  (DD-M3-P7-004).
- **Runtime semantics** — positional un-keyed identity (normative);
  tail insert / remove plans; live positional item reads with the
  out-of-range guard; stage-then-commit insertion; tail-first
  disposal; the preserved synchronous drain contract (DD-M3-P7-005).
- **Placement storage** — child-carried placement (ZStack migrates
  this phase; Grid deferred with trigger); the single placement-aware
  splice seam owning the structural side-effect bundle (DD-M3-P7-006).
- **Validation & drain disposition** — the full dual-gate reject
  matrix; cap accounting fixed (depth, not breadth) with the ≫N
  convergence fixture; reactive-drain residuals 1–3 carried with
  record, item 4 held (DD-M3-P7-007).

Out of scope — keyed identity / retained state, reorder, structured
item fields / `TypedValue`, `f64[]`, host state boundary, loop-external
collection reads, per-item handlers, binder-in-`if`-condition, nested
`for` / template scope, member-range bodies, general collection
expressions, Grid / Box / ScrollView direct `for`, large-N
performance, Image widget, per-monitor DPI:
[../decisions/preamble.md §Out of scope](../decisions/preamble.md#out-of-scope);
the deferred-items 正本 with activation triggers is the framing scope
table ([../requirements/framing.md](../requirements/framing.md)).

## Verification closure

The ADR's
[Phase 7 verification closure](../decisions/preamble.md#phase-7-verification-closure-what-counts-as-a8-evidence)
fixes the seven evidence lines and their content, including the
positive controls (a fixed-N single frame is **not** evidence). This
plan adds only the *where* (task mapping); the per-item sub-assertions
(prefix-pointer retention, same-batch doomed-binding guard, declared
order with `if` / `for` interleaving, handler-return observability,
≫N cap convergence) live in the ADR items and are referenced — not
re-stated — by the owning task:

| ADR evidence item | Task(s) |
|---|---|
| (1) `wasamoc check` compile-time (positive + full reject matrix) | T3; gallery positive control T8 |
| (2) pure-logic cardinality planner / expansion seam | T4 |
| (3) lowering / textual-IR roundtrip / loader rejection | T3 (emit) + T6 (loader) |
| (4) Windows-runtime integration (CI-gated, fail-not-skip) | T7 |
| (5) assistant-visible GUI (2+ frames: N → append → remove) | T8 |
| (6) E2E visible smoke | T8 (assistant build/launch) + T9 (owner GUI smoke) |
| (7) A12 spec-closure gate | T10 |

The Windows-runtime fixtures (item 4) fail — not skip — on a runner
that cannot create the Compositor; the skip-guard inherits the Phase
2–6 pattern (`0x80070005` from `wasamo_init`), and multi-test binaries
reuse the Phase 6 keep-alive apartment helper.

**Positive-control discipline:** the A8 proof is a **2+ frame
mutation pair** (initial N → append N+1 → tail-remove), driven by
body-external text Buttons; prefix-subtree-pointer retention is the
positional-identity positive control; the same-length static reset and
the empty-`drop-last` no-dirty case are tested, not assumed.

## Obligations carried from the ADR (represented in this plan from the start)

Per
[../decisions/preamble.md §Obligations carried to the implementation plan](../decisions/preamble.md#obligations-carried-to-the-implementation-plan):

1. **First task: instantiation-context design** — T1 designs the
   instantiation context type (element tag, signal reference,
   position, live / out-of-range guard); the DD variant spellings stay
   intentionally adjustable.
2. **Sequencing for bisectability** — T1 fixes (and records in
   [log.md](./log.md)) the order of the I2 schema migration, the C1
   seam canonization, the ST2 placement migration, the splice
   primitive, and the `for` runtime work so intermediate commits stay
   bisectable. The default order encoded in this plan is
   T2 (I2 schema) → T3 (wasamoc surface) → T4 (C1 seam) → T5 (ST2) →
   T6 (loader static path) → T7 (reactive range mutation); T1 may
   revise it with reasons.
3. **Load-path test refinement** — T6 proves static materialisation
   plus the initial `for` effect does **not** double-create children.

## Step-end / phase-end retrospective split (final-task ownership)

Per [../requirements/constraints.md §10](../requirements/constraints.md)
(the inherited final-step ownership split) and the Phase 6 phase-end
learning, the final-task split is represented from the start:

- The **final-task (T10) step-end retrospective**
  ([retrospectives.md](../../../procedures/retrospectives.md) checklist
  items 1–11; step → phase merge gate) is **owned by T10** and is a T10
  deliverable. Local `cargo fmt` / clean-rebuild evidence returns to
  T10 ownership if T10 changes production Rust (the Phase 6 phase-end
  sharpening).
- The **phase-end retrospective** (checklist items 12–18; phase → main
  merge gate), the **phase-branch CI run id**, the **handoff
  finalization**, and this file's **status flip** are **NOT owned by
  T10**. They land on the phase branch after T10 merges in, by
  separate phase-end commits, and are the precondition for the
  phase → main merge gate. The corresponding T10 plan bullets **stay
  `[ ]` at T10 close** and are checked by the phase-end commits.

Before the final task closes, the T0-frozen task list is cross-checked
against any mid-phase owner decisions and the mutable phase plan is
revised where they diverge (revise, do not work around).

## Lifecycle transition

This implementation file opens at `status: active` and transitions to
`status: closing` at the **phase-end batch commit** — the phase-branch
commit that lands the CI-verified gates (local `cargo fmt`, plus CI
`build` / `test` / Windows integration) and the spec / architecture /
plan status flips + log.md — **not** at T10 step-close. The on-CI gates
are phase-end-owned and verified only after the phase branch runs
`workflow_dispatch` CI, so **T10 step-close itself leaves
`status: active`**. The phase-end retrospective is a separate commit in
the same phase-end cluster and is recorded under
`process/milestone-3/phase-7/retrospectives/phase-end.md` (with sibling
`tN.md` per-step retros). The `closing` → `retired` transition belongs
to the phase → main merge / post-merge distillation.

Per
[../../../procedures/retrospectives.md](../../../procedures/retrospectives.md),
implementation start is gated on Moment 1 commit-set completion (ADR
Accepted; dsl_spec §4.15 iteration chapter; architecture §6.7.10 /
§6.8.5; m3-plan.md Phase 7 row; this implementation/preamble.md +
plan.md). At T0 land time, the Moment 1 commit set closes and T1 may
open.

## Implementation gates

Every implementation task runs
[implementation-gates.md](../../../procedures/implementation-gates.md)
at **task start and task close**: record the selected failure-mode
gates (with reasons for non-applicable ones) in [log.md](./log.md)
*before* choosing an approach, and close with the auditable artifacts.
Known phase-wide gate load (from the ADR):

- **Trap #1 (semantic migration / call-site audit)** — the I2
  `IrStateType` schema change (T2), the `HandlerExpr` widening
  (T2/T3), and `ControlFlowNode::For` / `ForLoopSubtree` (T2/T7): the
  close artifact is the `rg`-enumerated match-site table over
  `IrMember` / `ControlFlowNode` / `BindingTarget` / `HandlerExpr`,
  with `IrNode::widget_children()` and every widget-only filter
  classified (the exact Phase 6 failure mode).
- **Trap #2 (structural side-effect enumeration)** — the splice seam
  (T7): the six-item side-effect bundle in DD-M3-P7-006 checked off
  per change.
- **Trap #3 (parallel data drift)** — closed structurally by ST2 for
  ZStack (T5); the close artifact is "no parallel placement vectors
  remain on mutated paths" (greppable) + the Grid static-only pointer.
- **Trap #4 (untested reject branch)** — every DD-007 matrix row
  fires a direct test (T3 / T6 / T7).
- **Review tiers** — T2 (schema / IR migration) and T5 / T7 (runtime
  structural change) take the full independent review; diagnostic /
  reject-branch additions (T3, T6) take the branch/test-focused
  review.

## Technical risks (planning-time recon)

The per-DD §Technical risk re-evaluation sections carry the full set;
the plan-level top risks and their mitigating tasks:

| ID | Risk | Mitigation |
|---|---|---|
| R-A | **I2 is a compile-error-forcing schema migration** — `IrState.ty: IrStateType` breaks every `IrState` construction / match site across `wasamoc`, textual-IR emit / load, and the runtime registry; the workspace will not build until all sites migrate together. | T2 bundles the schema change with emit + loader + registry in one buildable commit (per AGENTS.md §Commit rules); call-site audit artifact at close. |
| R-B | **The C1 canonization touches the shipped conditional mutation path.** | T4 is its own task/commit, separate from `for` runtime work; the seam is pure logic with its own unit suite; the Phase 6 declared-order Windows fixtures run unchanged as regressions. |
| R-C | **The ST2 ZStack migration touches shipped arrange / loader code.** | T5 is its own commit preceding the range primitive; Phase 6 ZStack fixtures (union sizing, alignment, conditional-under-ZStack placement) are the regression gate. |
| R-D | **Reject-matrix breadth (~20 dual-gated branches).** | Table-driven fixtures; the matrix lives in DD-M3-P7-007 and is checklisted by T3 / T6, not rediscovered per-crate. |
| R-E | **Stale-index / doomed-binding hazards under same-batch tail removal.** | The guarded `ItemRead` (DD-005) plus T7's directly-fired same-batch removal fixture; drain tie order must not turn tail removal into a panic. |
| R-F | **Cap surprise at scale.** | T7's ≫N convergence fixture (e.g. 64 > `MUTATION_CAP`) positively demonstrates breadth ≠ depth; the fixture records which setup path it uses (many tail-append assignments in one batch, or headless direct signal setup). |

Revisit and close each risk in [log.md](./log.md) as it lands or
evolves; T1 sharpens this table against the current source before T2
opens.

## Cross-references

- ADR set: [../decisions/preamble.md](../decisions/preamble.md) +
  DD-M3-P7-001 through DD-M3-P7-007.
- Phase 7 framing decisions:
  [../requirements/framing.md](../requirements/framing.md)
  (FD-P / FD-Q / FD-A … FD-G; deferred-items 正本 scope table).
- Phase 7 constraints:
  [../requirements/constraints.md](../requirements/constraints.md).
- Specification chapters (design-spec draft):
  [`docs/dsl_spec.md`](../../../../docs/dsl_spec.md) §4.15 iteration +
  §2 / §3 / §5 / §8 supporting additions (Phase status: `M3-Phase 7
  design accepted; implementation pending` until the T10 Moment 2
  flip).
- Architecture entries:
  [`docs/architecture.md`](../../../../docs/architecture.md) §6.7.10
  iteration runtime shape; §6.8.5 child-carried placement; §9
  declared-tree / entity-tree generalisation; top Status flips to
  `M3-Phase 7 closed (implementation-synced)` at T10 Moment 2.
- ABI: [`docs/abi_spec.md`](../../../../docs/abi_spec.md) — **no
  touch** per the ADR preamble (collection state is runtime-owned; no
  host API). If implementation surfaces an unavoidable ABI need, it is
  recorded at Moment 2 with owner confirmation.
- `docs/notes/architectural-family.md` — the FD-Q trigger-1/-3 confirm
  entry lands at **Moment 2** (T10), revise-in-place.
