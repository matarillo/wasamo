# Vision Decision Record — Plan revision discipline

**Status:** Proposed

**Scope:** `process/procedures/workflow.md`, `process/README.md`,
`AGENTS.md` (its always-loaded plan-freeze wording),
`process/cross-milestone/decisions/` (this file, its supersede of the
read-only clause of DD-V-015, and its refinement of DD-V-019), the
`milestone-N/plan.md` agreement-edit rules, and the stale
cross-references in `process/milestone-3/plan.md` and
`process/milestone-2/plan.md` that point at dropped rule text.

## Context (shared)

While preparing the M3-Phase 7b insertion, the rules that govern *what
may be edited in a frozen `in-progress` plan* were found to be
**orphaned**. Two narrow exceptions to the "Frozen agreement is
read-only under `in-progress`" rule had been codified historically —
the **Acceptance criteria revision** exception (commit `d2deadf`,
2026-05-08) and the **Factual correction** exception (commit
`1de87a0`, 2026-05-21). Both lived in the Status-lifecycle section of
`docs/plans/README.md`; commit `13488b9` (2026-05-28, "make process
docs the SSOT") replaced that README with the slimmer
`process/README.md` and did not carry the exception prose forward, so
the rules now survive only in git history and the links at
`process/milestone-3/plan.md:482` and `process/milestone-2/plan.md:233`
dangle.

Recovering those exceptions surfaced a deeper question than where to
re-file them: whether the **read-only-by-default** model they patch is
the right model at all. The governing principle is that every plan is a
planning-time hypothesis, revisable through proper procedure. What a
plan-revision discipline must protect is not the *immutability* of the
agreement but two things: that no change is **unilateral or
unscrutinised**, and that every change is **auditable**. Concretely,
the agent must never revise the agreement on its own authority —
authority rests with the owner; but the agent is *expected to propose* a
revision whenever a premise the plan rested on has changed, and, when
the owner initiates a change, to **critically check that the premise
genuinely changed** rather than comply by default. Once a change has
cleared that gate, recording it should be light. Pre-1.0, Wasamo is
authored under a single owner (BDFL) plus AI, so the plan's "agreement"
is the owner's own evolving intent rather than a contract between
parties; the protection that matters is the critical-check gate, not a
freeze that forbids change unless an exception fires.

This vision decision record sets the plan-revision discipline
accordingly (DD-V-026) and re-homes it (DD-V-027). Both decisions
follow the ADR shape (Options → Comparison → Recommendation) and remain
`Proposed` pending owner review.

## DD-V-026 — Plan revision discipline: gated revision with proportional recording

**Status:** Proposed

**Context:**
[DD-V-015](./doc-system.md#dd-v-015--plan-two-layer-structure-frozen-agreement--live-progress)
split each plan into a `## Frozen agreement` section and a `## Progress`
section, and made the agreement section **read-only** once the plan is
`in-progress`, routing substantive changes through a vision ADR. The
two narrow exceptions above were later bolted onto that read-only
default. The default forces every agreement change to be classified
into an exception channel or escalated to a full vision ADR.

The value DD-V-015 actually protected was twofold: (i) scope changes
must be **visible and deliberate** — not slipped in by silent or
unilateral edit; and (ii) the plan's acceptance criteria must stay
**mirrored** with `process/_roadmap.md`, the AC SSOT. Immutability of
the agreement was a side effect of the chosen mechanism, not the goal —
and a read-only wall is only one way to force deliberateness.

**Options:**

- **Option 1 — Read-only-by-default, with the narrow exceptions
  restored.** Keep the agreement section read-only under `in-progress`;
  permit only the Acceptance-criteria-revision and Factual-correction
  exceptions in place; route everything else through a vision ADR.
  - Gain: maximal friction against silent scope creep; a stable
    baseline preserved by construction.
  - Give up: ceremony out of proportion to a single-author pre-1.0
    project; the default contradicts the governing principle that
    plans are revisable hypotheses; it inverts the default against the
    very author whose intent the agreement records; a routine
    corrective change must be shoe-horned into a taxonomy or escalated
    to a full vision ADR.

- **Option 2 — Revisable agreement behind an authority-and-critical-
  check gate, with proportional light recording.** The agreement
  section is revisable while the plan is `draft` or `in-progress`, but
  a substantive revision must clear a gate before it is recorded: (i) the **owner authorises** it —
  the agent never changes the agreement on its own authority; and
  (ii) it rests on a **premise change that has been critically
  checked** — the agent *proposes* such revisions when it observes a
  premise shift, and *critically verifies* the premise (rather than
  complying by default) when the owner initiates. Once the gate is
  cleared, recording is light and proportional: a mandatory
  Revision-log entry whose weight scales with the change, appended
  (never silently overwritten) so before/after stays legible, with
  `process/_roadmap.md` mirrored whenever acceptance criteria change.
  - Gain: the deliberateness a hard freeze enforced by a wall is
    instead supplied by the critical-check gate — which also directly
    catches the failure the owner most wants to prevent (the agent
    moving scope on its own); the former AC-revision and
    Factual-correction exceptions become *examples* of proportional
    recording rather than special cases; ceremony is right-sized to a
    BDFL pre-1.0 project.
  - Give up: the gate's value depends on the critical check being done
    in earnest — the agent genuinely challenging a weak premise, the
    owner genuinely weighing an agent proposal — rather than degrading
    into mutual rubber-stamping.

- **Option 3 — Revisable with no mandated trail.** Drop the freeze and
  require nothing.
  - Gain: zero ceremony.
  - Give up: re-opens silent drift, ROADMAP divergence, and unchecked
    unilateral change — discards everything the freeze was buying.
    Rejected.

**Comparison:**
Option 1 over-protects for the project's actual structure; Option 3
under-protects and throws away both the AC mirror and any check on who
changes what. Option 2 relocates the deliberateness from a freeze wall
to an authority-and-critical-check gate: the agent cannot change the
agreement alone, every substantive change must survive scrutiny of its
premise from whichever side did not initiate it, and only then is the
(light) recording made. This keeps the protection that earns its weight
while making revisability the default.

**Recommendation:** **Option 2.** The agreement section ceases to be
read-only; it is revisable behind the gate, scoped **pre-1.0** with a
re-evaluation trigger at 1.0 (when external contributors make the plan
a multi-party contract, a firmer freeze may be reintroduced). This
**supersedes the read-only-under-`in-progress` clause of DD-V-015**;
the two-section `Frozen agreement` / `Progress` *structure* of DD-V-015
is retained — only its read-only clause is superseded. (The section may
keep the name `Frozen agreement` for continuity, or be renamed
`Agreement`; that is an editorial follow-up, not part of this decision.)

Three rules hold across **all** tiers:

- **The agent never revises the agreed agreement body on its own
  authority** — it proposes; the owner authorises. This binds the
  *agreed* agreement section of a plan; the binding attaches to whatever
  the owner has agreed, not to a status label — content **not yet
  owner-agreed** (an unreviewed draft, or a still-open part of a plan
  under review) is freely editable. A proposal is a **separate
  artifact** — a drafted Revision-log entry, not an in-place edit of the
  agreement body — and the body edit lands only after owner
  authorisation.
- **Proposing a revision when a premise has shifted is a positive
  duty**, not optional caution: withholding such a proposal is a
  failure mode, the same kind as making an unauthorised change.
- **The critical check is performed by the side that did not initiate
  the change, and is never self-administered by the initiator.** When
  the agent proposes, the owner is the check; when the owner initiates,
  the agent is the check. An initiator approving its own premise is the
  rubber-stamping failure the gate exists to prevent.

The proportional recording then has three tiers:

1. **Editorial / factual** — wording, a moved file path, a
   cross-reference target. A one-line Revision-log entry; no
   premise-check (there is no premise to re-examine). This tier is
   available **only** for a mechanical correction that changes no
   identifier, no reference graph, and no normative meaning — it
   corrects the record to match what was already meant. Renaming an AC,
   phase, or decision *identifier* is presumptively **tier 2** (it
   touches the reference graph), as is any change where it is in doubt
   whether it is factual or substantive.
2. **Scope / AC / phase-structure** — adding, refining, or superseding
   acceptance criteria; inserting or reordering phases; changing
   dependencies, the acceptance ↔ phase mapping, or out-of-scope. All
   of tier 2 requires a **critically-checked premise change** (checked
   by the non-initiating side) and a Revision-log entry with rationale;
   existing AC IDs are preserved (no silent renumbering);
   `process/_roadmap.md` is mirrored whenever acceptance criteria
   change. Tier 2 is **asymmetric by direction**:
   - **Additive / refining** (add an AC, insert a phase, refine
     wording, reorder phases that have not executed): lighter, because
     these are self-correcting and visible — but not free. The entry
     must still carry a one-line **impact check**: the change's effect
     (if any) on existing AC meaning, dependency order, the evaluation
     of completed phases, the retro / merge gate, and whether a ROADMAP
     mirror is required. "Additive" describes the edit, not its blast
     radius; the impact check is what confirms the edit is genuinely
     additive in effect.
   - **Retracting / narrowing** (supersede or remove an AC, move
     in-scope work to out-of-scope, defer a committed deliverable,
     reorder or alter a *completed* phase): heavier, because a
     retraction can silently erase a commitment and its premise is the
     most self-serving to assert. In addition to the above, the
     retracted item must be recorded as a **deferral with an activation
     trigger** — naming where its responsibility now lives and what
     re-opens it — using the deferred-item table pattern already used
     in phase framings, not merely dropped from the text. This keeps
     the milestone-end "no silently deferred surface" guarantee
     enforceable.
3. **Thesis / purpose reversal** — a change to what the milestone is
   for, not merely how it is met. Same gate, plus a Revision-log entry
   **and** a vision decision record. Because this is the heaviest and
   most self-serving class, the critical check should be **genuinely
   independent** where feasible (a separate review pass), not only the
   non-initiating side's judgement.

**Status scope.** The gate and its light tiers apply while a plan is
`draft` or `in-progress`. A `completed` plan's agreement is **not**
lightly rewritten: a factual error is fixed as an archival correction
(tier 1), but any substantive re-interpretation of completed work —
re-scoping a shipped phase, re-reading what an AC was discharged by — is
not a planning change and does not ride the additive path; it goes
through postmortem / ROADMAP history at retraction weight (independent
check plus a durable record of the original and the re-reading). That
durable record lives in one of: the milestone `handoff.md`, a phase
retrospective / postmortem, a `process/_roadmap.md` revision note, or —
if the re-reading changes a normative decision — a vision decision
record.

**Enforcement tier:** the Revision-log entry is **Forcing** — for
tier 2/3 it must be filled out to the template below, so the *gate* (not
merely the edit) is auditable against ground truth. Tier 2 retractions
additionally produce the **deferral-with-trigger artifact**; tier 2
changes touching acceptance criteria produce the **ROADMAP-mirror
artifact**; tier 3 adds the **decision-record artifact**. Two things are
checkable on their face: an agreement edit with no recorded owner
authorisation, and a premise-check recorded by the same side that
initiated the change — each is a violation.

**Revision-log entry — minimal template.** To keep "light" from
decaying into thinly-grounded after-the-fact assent, each tier-2/3 entry
is a fixed fill-in (a few lines, not an essay):

- **What / tier** — the edit and its tier (2-additive, 2-retracting, 3).
- **Initiator** — owner or agent.
- **Old premise** — the assumption the prior plan rested on.
- **New evidence** — what changed, concretely (not "things changed").
- **Why the old plan no longer holds** — how it is now insufficient,
  incorrect, or lower-confidence given the evidence (it need not be
  "invalid"; over-claiming a clean break is itself a failure mode).
- **No-change option considered** — what keeping the plan as-is would
  cost, and why that was rejected.
- **Critical check** — the non-initiating side's assessment (the
  owner's, when the agent proposed; the agent's, when the owner
  initiated). The initiator may not write this field for itself.
- **Owner authorisation** — recorded; absent it, the edit is a
  violation.
- *(additive)* **Impact check** — the one-liner from the additive tier.
- *(retracting)* **Deferral** — the activation-trigger row.

At proposal time a proposer leaves the fields it cannot yet fill —
**Critical check** and **Owner authorisation** — as `pending`; the
agreement body edit lands only once both are filled. (When the owner
initiates, the owner's initiation *is* the authorisation, recorded as
such, and the agent supplies the critical check.)

The owner's authorisation is the **root of trust**; these fields exist
so that authorisation is *informed* rather than reflexive, and so an
agent's "the premise changed" is a falsifiable record rather than an
assertion.

**Worked consequence (M3-Phase 7b):** inserting Phase 7b is a tier-2
**additive** change (a phase insertion). Its premise change — the
Grid/ZStack placement asymmetry that became legible only after Phases 5
and 6 shipped — is owner-raised, so the agent is the non-initiating
critical check; that check has been done here (the finding that the
new-AC question is *contingent* on DD-M3-P7b-001, rather than assumed
either way). Because the insertion is additive, the standard tier-2
recording suffices: a Revision-log entry with rationale, plus a ROADMAP
mirror *only if* DD-M3-P7b-001 changes acceptance criteria. The
deferral-with-trigger artifact would attach only if DD-M3-P7b-001 ended
up retracting or deferring an already-committed surface. No bespoke
phase-insertion channel and no standalone authorising VDR are needed;
the earlier "does a corrective phase with no new AC need its own VDR"
question dissolves.

## DD-V-027 — Home of the plan-revision discipline

**Status:** Proposed

**Context:**
[DD-V-019](./process-rule-ssot.md#dd-v-019--process-rule-ssot-distribution)
splits ownership so that `process/README.md` owns "structural
conventions: folder roles, **lifecycles, mutability**" while
`process/procedures/workflow.md` owns "development workflow:
milestone/phase stages, **document lifecycle**, glossary". A rule about
what may be edited in a plan at a given status is simultaneously a
*mutability* rule (README's word) and a *document-lifecycle* rule
(workflow's word), so the boundary is ambiguous. In practice the
concrete status machinery for `plan.md` (`draft → in-progress →
completed`, the 凍結文書/継続文書 classification) already lives in
workflow.md §"ドキュメントのライフサイクル", workflow.md §1.4 already
states the plan-freeze rule operationally, and `process/README.md`'s
Folder-conventions table has no row for `milestone-N/plan.md` at all.

**Options:**

- **Option 1 — `process/README.md`.** Restore the discipline to a
  Status-lifecycle section of the README, its historical home.
  - Gain: continuity with where the rules used to live.
  - Give up: the README was deliberately slimmed in `13488b9` to
    folder/structure conventions and carries no `plan.md` lifecycle
    today; re-inflating it re-splits lifecycle prose across two files.

- **Option 2 — `process/procedures/workflow.md`.** Land the discipline
  beside the plan-status machinery it modifies, in workflow.md
  §"ドキュメントのライフサイクル" (adjacent to §1.4).
  - Gain: `plan.md`'s status lifecycle and 凍結/継続 classification
    already live here; the rule sits where a reader about to revise a
    plan already looks; no new lifecycle fragment elsewhere.
  - Give up: requires a one-line tightening of DD-V-019's boundary so
    README and workflow do not both appear to claim this rule class.

- **Option 3 — new dedicated file / `AGENTS.md`.** A new
  `process/procedures/plan-revision.md`, or folded into `AGENTS.md`.
  - Gain: a single unambiguous home with no boundary negotiation.
  - Give up: a dedicated file is over-fragmentation for this rule set;
    `AGENTS.md` owns enforceable rule *tiers*, not procedural rule
    *bodies*.

**Comparison:**
`plan.md`'s lifecycle is already a workflow.md responsibility (its
status table is there; README has no `plan.md` row), so Option 2 places
the discipline where it is already discovered and modified. Option 1
contradicts the post-`13488b9` slimming; Option 3 buys unambiguity at
disproportionate fragmentation.

**Recommendation:** **Option 2 — workflow.md**, with a one-line
tightening of DD-V-019's ownership row (README = folder roles +
mutability *of doc categories*; workflow = document *status* lifecycle
including the plan-revision discipline) and a one-line pointer left in
`process/README.md` so the rule is discoverable from the
mutability/Folder-conventions surface. Because this refines DD-V-019,
that edit lands in the same commit batch as this VDR's Accepted-flip,
per [DD-V-020](./process-rule-ssot.md#dd-v-020--process-rule-change-lifecycle).

## Landing tasks (on Accepted-flip)

Per [DD-V-020](./process-rule-ssot.md#dd-v-020--process-rule-change-lifecycle),
this is a structural change (it touches workflow.md / README.md /
AGENTS.md and supersedes the read-only clause of DD-V-015), so the SSOT
edits land in the same commit batch that flips this VDR to `Accepted`:

- Write the gated plan-revision discipline into
  `process/procedures/workflow.md` §document lifecycle: the
  authority-and-critical-check gate, the agent's propose-and-verify
  duties, the three recording tiers, the status scope, and the
  Revision-log entry template (DD-V-026).
- Replace the always-loaded plan-freeze wording in `AGENTS.md` (the
  `plan.md` "Frozen once `status: in-progress`" line) with a one-line
  pointer to the new discipline, so the always-loaded contract does not
  contradict the SSOT.
- Tighten the DD-V-019 ownership row (DD-V-027) and leave a pointer in
  `process/README.md`.
- Mark the read-only clause of DD-V-015 superseded, with a pointer
  here.
- Repair the stale references at `process/milestone-3/plan.md:482` and
  `process/milestone-2/plan.md:233` to the new home, and reconcile each
  plan's `in-progress` read-only wording with the new default.

## Revision history

| Version | Date | Notes |
|---------|------|-------|
| 0.1 | 2026-06-19 | Initial draft. Proposed; pending owner review. |
