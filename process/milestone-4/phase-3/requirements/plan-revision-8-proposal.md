---
title: M4 plan Revision 8 proposal — Phase 3 settles the prefix set's membership
status: proposed
created: 2026-08-12
authorised: pending
proposal-target:
  - process/milestone-4/phase-3/requirements/framing.md
  - process/milestone-4/plan.md
  - process/candidate-pool.md
workflow-tier: tier 2 refining
initiator: agent
related:
  - process/procedures/workflow.md
  - process/cross-milestone/decisions/plan-revision-discipline.md
  - process/milestone-4/phase-3/requirements/framing.md
  - process/milestone-4/phase-3/requirements/plan-revision-7-proposal.md
  - process/milestone-4/phase-3/decisions/dd-m4-p3-007-dot-meaning-and-prefix-set.md
  - process/milestone-4/plan.md
  - process/candidate-pool.md
  - docs/dsl_spec.md
---

# M4 plan Revision 8 proposal — Phase 3 settles the prefix set's membership

**State:** Proposed 2026-08-12. Owner authorisation and the owner's critical
check are outstanding; no agreement body is edited until both are filled.

This proposal exists because
[DD-M4-P3-007](../decisions/dd-m4-p3-007-dot-meaning-and-prefix-set.md)
reaches a recommendation its own framing does not authorise. It is filed as a
separate artifact so the owner decides the two questions separately: whether
the dot's defect is repaired the way DD-007 argues, and whether the retirement
question agreed on the same day to be M4-Phase 7's is pulled forward.

- **What / tier.** **Tier 2 refining.** Move one half of the prefix set's final
  form — **membership**, whether the expression-side set is emptied and the
  `root.` prefix retired — from framing §含まないもの, which gives it to
  M4-Phase 7, into Phase 3, where DD-007 settles it. The other half —
  **spelling**, whether host state is written with a prefix and whether members
  carry a reserved symbol — stays M4-Phase 7's and keeps its
  [candidate pool](../../../candidate-pool.md) row. AC9 wording, phase order,
  phase dependencies and the acceptance↔phase mapping are unchanged, and no
  other record's question moves.

- **Initiator.** Agent, 2026-08-12, during DD-007 drafting. The critical check
  is therefore the **owner's**, per
  [DD-V-026](../../../cross-milestone/decisions/plan-revision-discipline.md).

- **Old premise.**
  [Revision 7](./plan-revision-7-proposal.md) split the question: DD-007
  settles the rule, and whether the set is emptied goes onward to M4-Phase 7
  with a candidate-pool row as the backstop. Its impact check states the
  consequence of that split plainly — "`root` stays a valid prefix; the 26
  occurrences continue to compile" — and its critical check names the split as
  what keeps the widening proportionate. The premise underneath is that the
  rule and the membership are **separable**: that DD-007 could state what a dot
  means and leave the member in place, untouched and unstated, until Phase 7's
  own boundary design gave a reason to look at it.

- **New evidence.** Five findings produced during DD-007's drafting, by running
  the release `wasamoc` and by reading `docs/dsl_spec.md`. Each is an
  observation about what the language and its documents already do. None was
  available to Revision 7, whose evidence was confined to how the prefix is
  resolved, that it is optional, that no document defines it, that §2.4 and §3
  disagree, that it never reaches the IR, and that 26 occurrences exist.

  - **The language already requires the unprefixed form where it has any
    opinion.** §4.15 ships two named diagnostics that reject the prefix
    outright: `for x in root.xs` fails with "the loop collection must be a
    local state name", and `root.xs = root.xs.append(1)` with "collection
    mutation requires a local state name".
  - **Both spellings already ship, in one file.** Beside the 26 prefixed
    occurrences, five property bindings read state unprefixed — `checked:
    tab_all_selected` three times and `offset-y: scroll_y` in `gallery.ui`,
    `enabled: ready` in `bool-demo.ui` — so `gallery.ui` writes both.
  - **No name in the language is ambiguous without the prefix.** A binder can
    never share a name with a state (§4.15 rejects the collision as a name
    conflict) and a placement value does not shadow a state of the same name
    (§4.16). There is no position where writing the prefix changes which name
    is found.
  - **`slot` and `root` behave differently in a measurable way.** `slot.` is
    required, resolves its right side against a closed placement-keyword set
    rather than against state, and lowers to a placement record. `root.` is
    optional, changes what the right side resolves against in no position, and
    is discarded at lowering.
  - **Retention is not inert in the spec.** Under a retained member, §4.15's
    two positions become a carve-out **stricter** than membership checking —
    the member is admissible in expressions, and not there — and the spec
    sentence describing the member says it labels the space an unprefixed name
    already reaches.

- **Design inference — for the critical check, not an observation.** From the
  measured `slot` / `root` difference above, DD-007 generalises a criterion:
  **a prefix belongs where it changes what the right-hand side resolves
  against**, so `root` does not belong. That step is a value judgement about
  what the prefix set is *for*, and it is the load-bearing one — the five
  observations are compatible with the opposite conclusion. K2's case is
  precisely that a prefix which changes no resolution can still earn its place
  as a marker saying "this read is component state", and as a place already
  held for whatever the language later wants a prefix for. **The question this
  proposal asks the owner to check is whether to adopt the criterion**, not
  whether the observations are correct.

- **Why the old plan no longer holds — the split is not free.** The old plan is
  not wrong, and deferring remains workable; what the evidence removes is the
  assumption that deferring is **costless**.

  - **Phase 3 cannot leave the member unstated.** DD-007's spec sync writes
    §2.4, §3, §4.16 and §5 whatever it decides. Under retention it publishes a
    sentence defining a member that decides nothing, plus the §4.15 carve-out —
    into a public draft, carried through Phases 4–6. The deferral does not
    postpone the statement; it only fixes which statement is made.
  - **The deferred trigger is the weaker of the two.** Revision 7 recorded this
    itself: the row "would otherwise lapse" because its firing condition is a
    **negative** outcome — Phase 7 declining to use a prefix — which produces no
    deliverable. Settling membership now leaves Phase 7 a question its own
    boundary design cannot avoid answering, and leaves the pool row carrying
    only the spelling question, which fires either way.
  - **The break is strictly rising.** What retirement costs that retention does
    not is the break itself: the repository's 24 authored occurrences stop
    compiling until they are migrated, and any `.ui` written outside the
    repository against the public draft's examples stops compiling too. Both
    are at their smallest now. After this record's spec sync the prefixed
    spelling is published normative surface carried through Phases 4–6, and
    every later phase widens the exposed surface rather than narrowing it.

- **No-change option considered.** Keep the framing as accepted and let DD-007
  answer with **retention**: `root` retained as a checked member, written
  optionally with the unprefixed form named canonical, the member recorded as
  provisional pending M4-Phase 7, `state root` staying legal, and the existing
  occurrences left compiling. It still moves the teaching surface — naming the
  unprefixed form canonical and then teaching the other one would be
  incoherent — so the 33 places move under this option too, without breaking.

  This is a complete answer, not a placeholder. It satisfies the rule DD-007
  states, and it **frees `photos.count` for DD-001 exactly as retirement
  does** — the spelling is freed by closing the set, not by emptying it — so
  nothing downstream in Phase 3 is blocked by taking it. What it costs is a
  spec sentence describing a member that decides nothing; the §4.15 carve-out
  stated and taught; two synonymous spellings for one state read carried toward
  the 1.0 freeze; the identifier `root` carrying two roles told apart by
  position, since it keeps a contextual meaning as a dotted head while staying
  an ordinary state name everywhere else; and a larger, later break if Phase 7
  retires it after all.
  It is rejected here because the record publishes a statement about the
  member either way, and retention publishes the one that would have to be
  withdrawn.

- **Critical check.** **Pending — the owner's.** The agent is the initiator and
  may not write this field for itself. The substance to check is the design
  inference above — the criterion, not the five observations. Two further
  points are offered for that check rather than asserted as its outcome.
  First, DD-007's merits argument stands
  on its own and does not need this revision: if the owner finds the retirement
  case persuasive but the pull-forward unwelcome, declining this proposal costs
  Phase 3 nothing it needs. Second, the strongest counter-argument is that
  Revision 7 was authorised on the same day, on a proportionality claim that
  named this exact split — so a revision reversing it a few hours later is the
  shape a scope-creep failure would also take. What distinguishes it is that
  the five findings above were produced by the ADR investigation Revision 7
  authorised, which is the mechanism by which a deferral is supposed to be
  re-examined; the owner is the judge of whether that distinction holds.

- **Owner authorisation.** **Pending.**

- **Impact check.** Existing AC meanings are unchanged — AC9's text is not
  touched and no AC ID is added, renumbered or superseded, so
  `process/_roadmap.md` needs no mirror. Phase dependencies and order are
  unchanged: DD-007 stays upstream of DD-001 inside Phase 3 and no phase
  boundary moves. Completed-phase evaluation is unaffected, and M4-Phase 1's
  evidence artifact keeps its two prefixed occurrences as the record of what was
  run, so no closed phase's artifact is rewritten. Retro and merge gates are
  unchanged. Two things do move and are the reason this is not a wording edit:
  **authored `.ui`** — the 24 occurrences in `examples/` and the nine example
  lines in `dsl_spec.md` stop compiling until they are rewritten, reversing
  Revision 7's "the 26 occurrences continue to compile" — and the **DD-007
  positive control**, which becomes the migrated `examples/` compiling without
  the prefix. The rewrite itself is not what this revision buys: under the
  no-change option those same 33 places still move, because W3 names the
  unprefixed form canonical. What the revision changes is that they **break**
  if they do not. `docs/abi_spec.md` still does not move: the prefix never
  reaches the IR or the C ABI.

## What stays deferred

This revision discharges one row of Revision 7's deferral table and leaves the
rest standing. Nothing is silently dropped.

| Question | Owner after this revision | Activation trigger |
|---|---|---|
| Whether the expression-side set is emptied, retiring `root.` | **Discharged in Phase 3** by DD-007 | — |
| The host-state prefix spelling itself | M4-Phase 7 | Unchanged — its own host state boundary design, which cannot close without spelling host state somehow |
| Whether members of the set carry a reserved symbol | M4-Phase 7, backed by the [candidate pool](../../../candidate-pool.md) row | Unchanged — Phase 7's boundary decision; [DD-V-028](../../../cross-milestone/decisions/pre-1.0-candidate-pool.md)'s per-planning disposition duty is the backstop and the 1.0 compatibility commitment the deadline |

The row that Revision 7 identified as the one that would otherwise lapse — the
one whose trigger depends on a negative outcome — is the row this revision
discharges. The two that remain have triggers Phase 7's own deliverables fire.

## Edits

The section-level record of what this revision would touch. `proposal-target`
names the documents; this table names the sections. All of it lands only after
the critical check and owner authorisation are filled.

| Target | Edit |
|---|---|
| framing §含まないもの, prefix-set bullet | Membership is settled in Phase 3 by DD-007. What stays M4-Phase 7's is the host-state prefix spelling and whether members carry a reserved symbol; the candidate-pool row carries the second |
| framing §DD と検証手段の対応, DD-007 row | The positive control becomes the migrated `examples/` compiling without the prefix, replacing the 26 surviving `root.` occurrences |
| framing §Revisions | One dated entry |
| `plan.md` §Revision log | Revision 8 entry in the tier 2 template |
| [candidate pool](../../../candidate-pool.md), prefix-set row | Part (1), membership, recorded as discharged with its decision reference; part (2), spelling, stays open with its trigger unchanged |
