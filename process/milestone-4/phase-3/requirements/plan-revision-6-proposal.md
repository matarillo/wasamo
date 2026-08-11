---
title: M4 plan Revision 6 proposal — ABI-bearing phase hypothesis
status: authorised
created: 2026-08-12
authorised: 2026-08-12
landing: pending
proposal-target: process/milestone-4/plan.md
workflow-tier: tier 2 refining
initiator: owner
related:
  - process/procedures/workflow.md
  - process/milestone-4/plan.md
  - process/candidate-pool.md
  - process/milestone-4/phase-3/requirements/framing.md
---

# M4 plan Revision 6 proposal — ABI-bearing phase hypothesis

**State:** Owner-authorised on 2026-08-12; reflected in the current working
tree. Commit / landing is still pending.

This proposal arose during the independent review of the M4-Phase 3 §2.3 /
§2.4 owner-alignment packet. It changes no Phase 3 scope or AC9 wording, but
removes an internally contradictory premise consumed by the `TypedValue`
candidate-pool row.

- **What / tier.** **Tier 2 refining.** Qualify four M4 plan statements so
  Phase 7 remains the only acceptance surface explicitly classified ABI-bearing
  at planning, while Phase 8's window-creation ABI impact stays for its ADR.
  Keep the non-Rust-host evidence gate for any Phase 8 ABI change. No AC, phase
  scope, dependency or ordering changes.
- **Initiator.** Owner, 2026-08-12. The owner directed the initial-plan claim to
  be treated as a revisable hypothesis and requested a critical re-check rather
  than preservation as a constraint.
- **Old premise.** The plan called Phase 7 M4's only ABI-bearing phase in its
  risk and progress text, but its host-parity section simultaneously classified
  Phase 8 window creation as ABI-bearing.
- **New evidence.** Phase 7's ABI-bearing status is explicit in AC8 and its phase
  description. Phase 8 has not reached framing or ADR; the current plan does not
  establish whether its multi-window / `WindowConfig` design needs new ABI,
  reuses Phase 7's boundary or stays runtime-internal.
- **Why the old plan no longer holds.** The two unconditional readings are
  incompatible, and selecting either would pre-decide an unperformed Phase 8
  design. The known classification and the conditional evidence duty can be
  stated without doing so.
- **No-change option considered.** Leave the contradiction for Phase 8 framing.
  Rejected because it makes the current plan ambiguous and lets the `TypedValue`
  scheduling row inherit an arbitrary reading.
- **Critical check.** Agent-completed 2026-08-12. Current evidence proves only
  that Phase 7 is ABI-bearing; it proves neither outcome for Phase 8. Making the
  Phase 8 non-Rust-host gate conditional on an actual ABI change retains the
  vision's cross-host protection without selecting an architecture early.
- **Owner authorisation.** Authorised 2026-08-12 through the owner's
  review-remediation instruction.
- **Impact check.** AC meaning, phase order, completed-phase evaluation and
  retro / merge gates are unchanged. `_roadmap.md` needs no mirror. Phase 7
  still owns the `TypedValue` ABI-impact verdict; Phase 8 still owns
  multi-window / window config and must move `abi_spec.md` plus bindings if its
  ADR changes ABI.

The exact edits are present as Revision 6 in the current working-tree version
of [`process/milestone-4/plan.md`](../../plan.md). They do not alter the Phase 3
§2.3 / §2.4 scope or verification proposal, and are not described as landed
until their commit exists.
