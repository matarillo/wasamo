---
title: M4-Phase 3 plan revision proposal
status: approved-for-landing
created: 2026-08-11
proposal-target: process/milestone-4/plan.md
framing-status: "§2.2 owner-agreed; §2.3 / §2.4 pending"
workflow-tier: tier 2 additive/refining
initiator: mixed; recorded per proposal
related:
  - process/procedures/workflow.md
  - process/milestone-4/plan.md
  - process/_roadmap.md
  - process/milestone-4/phase-3/requirements/framing.md
  - process/milestone-4/phase-2/implementation/handoff.md
---

# M4-Phase 3 plan revision proposal

**State:** Approved for landing on 2026-08-11. This artifact itself does not edit
the plan's Frozen agreement or activate DD-M4-P3-005. The exact edits below were
authorised independently; ADR drafting and implementation remain gated on their
landing in the plan and, for AC changes, the ROADMAP mirror.

The independent framing review found three changes that must not share one
approval:

1. correct the implementation responsibility of an already-planned deliverable;
2. add an owner-required author capability that the current plan does not imply;
3. close a systemic handler-assignment validation gap instead of individual cases.

They are therefore recorded as separate Revision-log entries. Revision numbers
remain provisional until landing. The required critical checks and exact-text
authorisations are recorded per proposal below.

## Proposed Revision 3 — Correct Phase 3's cross-layer responsibility

- **What / tier.** **Tier 2 additive/refining.** Correct the Phase 3 responsibility
  and dependency prose so per-item conditional rendering explicitly includes
  condition evaluation and runtime structural integration. Remove the statement
  that Phase 3 is compiler-side. Do not add a new author-facing capability in
  this revision.
- **Initiator.** Agent. The mismatch was found by the Phase 3 source audit and
  confirmed by the independent framing review.
- **Old premise.** The plan describes Phase 3 as checker / lowering / evaluator
  work and contrasts it with Phase 4 as runtime / widget-side. It assumes the
  planned per-item conditional can be delivered inside that boundary.
- **New evidence.** The current conditional effect uses the ordinary
  `BindingEvalContext`, while a condition inside a `for` must read the owning
  item / index context. The false-to-true mutation path rebuilds through
  `build_node(...)`, whereas iteration insertion uses
  `build_node_with_loop_context(...)`. A per-item condition can therefore lose
  loop context both when evaluating the condition and when re-materialising its
  subtree. That subtree may own effects and handlers, and its removal or
  re-insertion crosses the existing focus, hover, handler-registry and layout
  lifecycle seams established in Phases 1 and 2.
- **Why the old plan no longer holds.** The author-facing deliverable was already
  planned, but its stated ownership is insufficient. A compiler-only delivery
  could accept and lower the syntax while failing to preserve the binder or
  runtime structural lifecycle that gives the feature its meaning.
- **No-change option considered.** Keep the wording and treat runtime work as an
  implicit implementation detail. Rejected in this proposal because it hides a
  cross-layer dependency that affects ADR responsibility, call-site review and
  GUI evidence; it would recreate the false premise during task planning.
- **Critical check.** Owner check completed 2026-08-11: the evidence and change
  boundary are valid and proportionate.
- **Owner authorisation.** Authorised 2026-08-11 for the exact proposed text.
- **Impact check.** No AC meaning, phase order, completed-phase evaluation,
  retrospective / merge gate or ROADMAP text changes. Phases 3 and 4 remain
  independently sequenced after Phase 2; only the inaccurate compiler/runtime
  responsibility split is removed. Phase 3's later implementation evidence must
  cover the structural consumer it already promised.

### Proposed Frozen-agreement edits for Revision 3

In the M4-Phase 3 phase description, append after the existing surface list:

> Per-item conditional rendering is a cross-layer deliverable: Phase 3 owns the
> loop context used by condition evaluation and subtree re-materialisation, and
> integrates creation and disposal through the existing effect, handler,
> focus / hover and layout lifecycles. This responsibility does not create a
> separate structural writer or change the positional iteration baseline.

Replace the dependency sentence:

> Phase 3 is compiler-side (checker / lowering / evaluator); Phase 4 is runtime /
> widget-side.

with:

> Phase 3 owns predicate checking / lowering / evaluation plus the runtime
> structural integration required by per-item conditional rendering; Phase 4
> owns scrolling, gallery widgets and image presentation.

No `_roadmap.md` edit is proposed by Revision 3.

## Proposed Revision 4 — Add small reusable handler control flow

- **What / tier.** **Tier 2 additive/refining.** Refine AC9 and the Phase 3 entry
  with a reusable but small handler control-flow surface sufficient to guard
  state writes at collection boundaries. Mirror the AC9 refinement in
  `_roadmap.md`.
- **Initiator.** Owner for the product requirement, confirmed 2026-08-11 after
  independent review. The agent supplies the technical critical check below;
  the owner authorised the exact proposed text on 2026-08-11.
- **Old premise.** AC9 and the Phase 3 entry list collection count / emptiness /
  index access, per-item conditional rendering and equality selection. They do
  not provide handler control flow or another way to prevent the gallery's four
  selection writers from creating an invalid index.
- **New evidence.** The gallery writes `selected_index` unconditionally from
  Left and Right key handlers and from the `<` and `>` button handlers. Equality
  selection can show no selected thumbnail for an invalid value, but cannot by
  itself express both lower and upper write guards, including empty and
  one-item collections.
- **Why the old plan no longer holds.** It is insufficient for the owner's
  required Phase 3 outcome: all four gallery paths must stop at both ends, using
  a capability that is general enough to reuse outside the gallery.
- **No-change option considered.** Keep AC9 unchanged, define out-of-range reads
  in DD-M4-P3-002, and let all four writers create invalid state. Rejected by the
  owner because a runtime read contract does not let an author prevent the
  invalid write.
- **Critical check.** Agent check: the requested guard is not derivable from
  equality alone and is genuinely additional normative DSL scope. The narrow
  boundary remains technically coherent if the ADR requires only conditional
  handler execution and the minimum boundary predicates demonstrated by the
  gallery matrix; it need not admit general functions, loops, an `else` family,
  string concatenation or general arithmetic. The change does not require a new
  phase or reorder an executed phase.
- **Owner authorisation.** Authorised 2026-08-11 for the exact proposed AC9,
  Phase 3 and ROADMAP text.
- **Impact check.** AC9 is additively refined and therefore requires a mirrored
  `_roadmap.md` edit. AC1–AC8 and AC10–AC13 keep their meaning. Phase order and
  the evaluation of completed Phases 1 and 2 do not change. Phase 3's close gains
  the four-producer boundary outcome; its retrospective / merge gates are not
  moved. Phase 4 remains independent of Phase 3 in implementation order.

### Proposed Frozen-agreement edits for Revision 4

Refine AC9 in `process/milestone-4/plan.md` and mirror the same wording in
`process/_roadmap.md`:

> Expression predicates: reading a collection from outside the repetition
> (count, emptiness, index access), per-item conditional rendering,
> equality-based selection, and a small reusable handler-control-flow surface
> sufficient to guard a state write at collection boundaries. String
> concatenation and general arithmetic stay outside M4.

Append to the M4-Phase 3 phase description:

> Phase 3 also owns the small reusable handler-control-flow surface needed to
> keep the gallery selection index inside the collection at all four producers
> (Left / Right key and the two navigation buttons), including empty, one-item
> and multi-item collections.

## Proposed Revision 5 — Require complete handler-assignment admission and type checking

- **What / tier.** **Tier 2 additive/refining.** Refine AC9 and the Phase 3 entry
  so every handler assignment is checked before execution for expression-position
  admission and LHS / RHS type compatibility. Replace the Phase 5 paragraph that
  leaves this as a wider open question with an explicit dependency on the Phase 3
  invariant. Mirror the AC9 refinement in `_roadmap.md`. Keep scalar `string`
  write capability in Phase 5.
- **Initiator.** Owner for the completeness requirement, confirmed 2026-08-11
  after independent review. The agent supplies the technical critical check
  below; the owner authorised the exact proposed text on 2026-08-11.
- **Old premise.** Phase 3 owns the binding-only string diagnostic intake, while
  the plan leaves broader handler-assignment type checking as a separable open
  question in the Phase 5 scalar-string prerequisite.
- **New evidence.** Phase 2 measured that both `i32 = "abc"` and `string = 5`
  can pass `wasamoc check`; binding-only `StrLit`, `StrPropRead` and
  `Interpolation` are only visible instances of a handler-assignment admission
  gap. Adding new Phase 3 expression forms would create more call sites at which
  per-variant rejection can be omitted.
- **Why the old plan no longer holds.** It is lower-confidence than the owner's
  required completeness contract. Closing three known string cases would still
  permit other destination mismatches to reach invocation, and would not provide
  an auditable reason to believe every assignment form uses the same admission
  and compatibility rules.
- **No-change option considered.** Reject only the three binding-only string
  forms and leave other mismatch directions to later runtime errors. Rejected by
  the owner in intent because it treats symptoms ad hoc rather than establishing
  a mechanism whose coverage can be audited.
- **Critical check.** Agent check: the evidence demonstrates a systemic checker
  gap, not merely a missing string evaluator. Position admission and destination
  type compatibility are separable invariants: a type-correct scalar string
  assignment remains unavailable until Phase 5, while a type-incompatible
  assignment is invalid independently of that future capability. This can be
  made a Phase 3 correctness obligation without choosing a checker architecture
  in the plan or moving the Phase 5 writer.
- **Owner authorisation.** Authorised 2026-08-11 for the exact proposed AC9,
  Phase 3, Phase 5 and ROADMAP text.
- **Impact check.** AC9 is additively refined and therefore requires a mirrored
  `_roadmap.md` edit. AC1–AC8 and AC10–AC13 keep their meaning. Phase order and
  completed-phase evaluation do not change. Phase 3 gains a completeness
  obligation and later auditable call-site coverage; its retrospective / merge
  gates are not moved. Phase 5 continues to own scalar `string` writes and
  consumes, rather than reopens, the Phase 3 validation invariant.

### Proposed Frozen-agreement edits for Revision 5

Append to AC9 in `process/milestone-4/plan.md` and mirror the same wording in
`process/_roadmap.md`:

> Every handler assignment is checked before execution for expression-position
> admission and LHS / RHS type compatibility. Scalar `string` write capability
> remains with M4-Phase 5.

Append to the M4-Phase 3 phase description:

> The phase establishes a complete admission and destination-type-checking
> contract for every handler assignment rather than adding rejects for
> individual RHS variants. This validation does not make scalar `string` writes
> available; that capability remains in Phase 5.

Replace the final two sentences of the Phase 5 scalar-string prerequisite
paragraph:

> Whether handler-body assignments gain type checking at the same time is a
> separable, wider question. `dsl_spec.md` §4.6 and §8.9 both move.

with:

> Phase 3 has already established handler-assignment position admission and
> destination-type compatibility. Phase 5 consumes that invariant while adding
> the scalar `string` writer and its admitted RHS forms; `dsl_spec.md` §4.6 and
> §8.9 both move.

## Gate disposition

| Proposal | Critical check | Owner authorisation | Frozen agreement edit | ROADMAP mirror |
|---|---|---|---|---|
| Revision 3 — cross-layer responsibility | owner check completed 2026-08-11 | authorised 2026-08-11 | authorised to land | not required |
| Revision 4 — handler control flow | agent check recorded | authorised 2026-08-11 | authorised to land | required on landing |
| Revision 5 — assignment completeness | agent check recorded | authorised 2026-08-11 | authorised to land | required on landing |

The three proposals were authorised independently. None is treated as landed
until its Frozen-agreement edit, and any required ROADMAP mirror, is committed.
