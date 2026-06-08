---
phase: M3-Phase 6
task: T4b
title: ScrollView conditional-content policy (DD-M3-P6-007) deliberation
date: 2026-06-04
scope: task-end
merge_target: feat/m3-phase-6
---

# T4b Retrospective

Task branch: `docs/m3-phase-6-t4b-dd-007` (to be merged to
`feat/m3-phase-6`). A deliberation-only task: it resolves the open
DD-M3-P6-007 (whether a ScrollView may be *conditionally empty*) and
flips it to `Accepted (a)`. The non-integer `T4b` label was chosen so an
(a) outcome (near-no-op) leaves the T5–T9 references untouched.

Refs:

- `e192125` (`docs(process): revise DD-M3-P6-007 per review passes;
  reconcile plan T4b`) — the multi-pass design-decision review revisions
  (strategic / recommendation-choice / implementation-readiness) folded
  while `Status: Proposed`.
- close-out commit(s) — DD-007 `Proposed → Accepted`, preamble §Decisions
  index + Revisions, `docs/dsl_spec.md` §4.11/§4.14 sync, plan.md T4b, this
  retro.

## Checklist (task-end, items 1–11)

1. **Main learning.**
   - The (a) decision is best read not as "ScrollView cannot be empty" but
     as **deferring the DSL's conditional-content model**. Phase 6 rejects
     the *current direct-conditional syntax* in exact-one containers; the
     conditionally-empty (b) direction stays open, to be reconsidered once
     the DSL picks a base content model (imperative member emission vs.
     optional typed content vs. explicit empty / fragment content). Keeping
     "reject the syntax" and "reject the direction" separate was the load
     -bearing framing the review surfaced.
   - Repo-grounded verification beat reviewer guesses twice: the prior-DD
     citation for ScrollView's exact-one contract was wrong in the stub
     (DD-M3-P4-003 → it is **DD-M3-P4-001**, loader gate DD-M3-P4-006), and
     two reviewer "assuming / pending" hedges (Box 0-child, Cell rationale,
     C-ABI error class) were resolved by reading the source rather than
     softening the text. A design-decision review that cites code should
     verify the citation in-repo before folding it.
   - A near-no-op task still has a real doc/process surface. Because (a) is
     a *code-path* no-op, the easy failure was dropping the dsl_spec sync;
     the plan's (a) bullet had in fact omitted it. Flagging then
     reconciling the plan (not just the DD) is what closed the gap.
2. **Specification document changes:** **yes (mechanical sync of a
   now-Accepted DD).** `docs/dsl_spec.md` §4.11 gains one sentence (a direct
   conditional member under ScrollView is rejected; wrap inside the content
   widget) and §4.14 gains one diagnostics-table row. Both are the
   mechanical transcription of the just-accepted DD-M3-P6-007 — no
   independent normative content. No `abi_spec.md` / `architecture.md` edit.
3. **Post-commit verification:** green. `cargo fmt --all -- --check` clean;
   clean rebuild (release + debug) + `cargo test --workspace` green. The
   task touches no Rust — the **code tree** is byte-identical to the phase
   tip `732afe2` (only docs/process changed), whose dual-gate evidence
   (`scrollview_conditional_member_*` /
   `validate_rejects_scrollview_with_conditional_*`) already passes. (CI log
   updated in [log.md](../implementation/log.md).)
4. **Design decisions / trade-offs for the PO:** **yes — resolved.** The
   a-vs-b choice was the task. The owner accepted **(a)** on 2026-06-04
   after a multi-pass design-decision review; the trade-off (wrapper is a
   real layout / future-accessibility node, not a neutral Fragment) and the
   deferred (b) direction are recorded in DD-007.
5. **Out-of-scope "while we're here" refactor:** none. Doc/process only.
6. **New DD needed for the current ADR:** none beyond DD-007 itself (this
   task's subject).
7. **Existing-ADR Proposed item / Proposed → Accepted promotion:** **yes.**
   DD-M3-P6-007 `Proposed → Accepted (a)`. Indexed in preamble §Decisions
   (seventh row, a mid-phase addition outside the original 1:1 framing
   slate) with a Revisions entry.
8. **Milestone-plan AC / phase-structure change:** none. A4/A7 unchanged.
9. **Carry-forward stubs / approximations / new `dead_code`:** none (no
   code change).
10. **New cross-task / cross-phase design constraint:** **yes —
    `doc-folded`.**
    - **Constraint:** an exact-one-cardinality container (ScrollView, and by
      precedent `Cell`) rejects a *direct* conditional member in Phase 6;
      conditionally-empty container content is deferred pending the DSL's
      conditional-content model choice.
    - **Evidence:** DD-M3-P6-007 deliberation + the T4-follow-up dual-gate
      tests.
    - **Placement:** `doc-folded` — DD-M3-P6-007 (§Deferred design space,
      §Accepted consequence) and `docs/dsl_spec.md` §4.11 / §4.14. The
      deferred (b) direction lives in the DD's §Deferred design space as the
      future-pickup record; a pointer suffices, no separate handoff entry.
11. **Downstream-task revision (no current-ADR impact):** **not needed.**
    The (a) outcome leaves T5–T9 unchanged — exactly what the non-integer
    `T4b` label was designed for; no renumber. No `[ ]` ADR / evidence item
    is left with unclear ownership: DD-007 is Accepted and its evidence is
    the already-shipped interim, owned by the T4 follow-up. The (b)-branch
    plan bullet is marked not-selected.

## Merge readiness

Checklist complete; build green; no blockers. Per
[retrospectives.md §進行手順](../../../procedures/retrospectives.md),
checklist completion is **not** merge authorization — the task-end no-ff
merge to `feat/m3-phase-6` awaits explicit owner approval. Push remains a
separate, later (phase-end) gate.
