# Milestone Plans

This directory holds **milestone-level plans**: per-milestone breakdowns of
phases, dependencies, and scope, used as the agreement artifact between the
project owner and Claude before a milestone begins implementation, and as
the live progress tracker while the milestone is in flight.

## Relation to ROADMAP, CHANGELOG, and ADRs

Plans sit between ROADMAP (acceptance criteria SSOT) and ADRs (per-phase
decisions). They own the phase structure and the live progress for the
milestone they cover
([DD-V-016](../decisions/vision-doc-system.md#dd-v-016--plan--roadmap-commit-flow-redefinition)):

```
docs/plans/<M>-plan.md  ─┬─►  ROADMAP.md commits acceptance criteria
                         ├─►  plan tracks progress for the active milestone
                         └─►  docs/decisions/phase-*.md records per-phase decisions
                                                ↓
                                          CHANGELOG.md (on milestone completion)
```

A plan proposes the structure of a milestone (which phases, in what order,
serving what acceptance criterion). On owner agreement, ROADMAP commits the
**acceptance criteria** and links to the plan for phase detail. ROADMAP does
**not** transcribe the phase list. As phases begin, each one's design
decisions go into a phase ADR. When the milestone ships, the entry moves to
CHANGELOG.

## File naming

```
docs/plans/<milestone>-plan.md       e.g. m2-plan.md, m3-plan.md
```

## Frontmatter

Every plan must carry the following frontmatter:

```yaml
---
milestone: M2
status: drafting | agreed | in-progress | completed
roadmap-anchor: ROADMAP.md#m2-foundation
adrs:
  - docs/decisions/phase-N-xxx.md   # filled in as ADRs are written
created: YYYY-MM-DD
---
```

## Plan structure: frozen agreement + live progress

Each plan has two top-level sections
([DD-V-015](../decisions/vision-doc-system.md#dd-v-015--plan-two-layer-structure-frozen-agreement--live-progress)):

```markdown
## Frozen agreement
   purpose, acceptance criteria, phase breakdown, dependencies,
   acceptance ↔ phase mapping, out-of-scope, risks

## Progress
   per-phase checklist, owner-facing notes, links to landed ADRs
   and commits as phases complete
```

The **Frozen agreement** section follows the status lifecycle below. The
**Progress** section is live until the milestone reaches `completed`; it
takes over the role that ROADMAP phase checklists used to play.

## Status lifecycle

| Status | Meaning | Editing rule |
|---|---|---|
| `drafting` | Pre-agreement. Owner and Claude iterate freely. | Free updates to both sections |
| `agreed` | Owner has signed off the agreement section. ROADMAP update in progress. | Frozen agreement: only to reflect agreement. Progress: free |
| `in-progress` | ROADMAP carries the commitment. Phases are executing. | **Frozen agreement: read-only.** Progress: live (update as phases land) |
| `completed` | All phases done. Ready for archival decision. | Read-only |

The key discipline: once `in-progress`, the **Frozen agreement** section is
read-only — substantive scope changes go through a vision ADR, not by
editing the plan. The **Progress** section is the active workspace for the
milestone.

### Acceptance criteria revision (in-progress only)

The Frozen agreement is read-only while a milestone is `in-progress`,
with **one narrow exception**: the **Acceptance criteria** subsection
may be revised in place if owner and Claude agree that the existing
criteria are structurally insufficient to satisfy the milestone's
stated purpose (for example, a foundation milestone whose criteria
cover only pipeline wiring and miss the runtime guarantees that
distinguish "it runs" from "it is a Foundation"). This exception
exists because pre-doc cannot always anticipate which guarantees a
milestone needs to commit to; once implementation is underway, the
shape of the gap may only become legible against concrete code.

Revisions under this exception are subject to the following rules:

- **Existing AC IDs are preserved.** New criteria are **added** with
  fresh IDs (e.g. A5, A6). Existing criteria may be **refined** in
  wording or marked **superseded** with a pointer to the replacement,
  but **silent renumbering is forbidden** — readers and ROADMAP
  references must remain valid.
- **A *Revision log* subsection is mandatory.** It lives at the end
  of the Frozen agreement (after `### Resolved deferrals`, before
  `## Progress`) and records, for each revision: date, motivation,
  added/refined/superseded AC IDs.
- **Phase breakdown / dependencies / acceptance ↔ phase mapping /
  out-of-scope remain read-only under this exception.** Changes to
  those parts still go through a vision ADR. New phases added to
  cover newly-added criteria are a structural change to the plan and
  are recorded both in the Revision log and in the Phase breakdown
  table; this is the one place the exception spills past the AC
  subsection, and only because new AC without a phase is meaningless.
- **ROADMAP must be updated to mirror the revision.** ROADMAP is the
  acceptance-criteria SSOT; the plan's mirror cannot diverge.

If the disagreement is about the milestone's purpose itself rather
than the criteria that satisfy it, that is a vision-level question
and goes through a vision ADR — not this exception.

## Scope rule (plan vs ADR)

The Frozen agreement section contains:
- Phase breakdown and ordering
- Dependencies between phases
- Acceptance criteria mapping (which phase satisfies which ROADMAP criterion)
- Out-of-scope list for the milestone

The Frozen agreement section does **not** contain:
- Per-phase design decisions (those go into phase ADRs)
- Implementation details

The Progress section contains:
- A top-level checklist of phases
- Per-phase implementation task lists (the ordered steps to carry out
  each phase's accepted decisions)
- Owner-facing notes ("where did we leave off")
- Links to landed ADRs, commits, or pull requests as phases complete

Per-phase task lists live here — not in the phase ADR — because task
lists remain hypotheses even after pre-doc agreement. Build failures,
linker errors, CI surprises, and direct application verification can all
require steps to be added, split, reordered, or dropped after the design
decisions themselves are settled. Progress is explicitly mutable
throughout implementation; the ADR's DD entries are not.

The Progress section does **not** contain:
- Design decisions (still ADR-shaped)
- Acceptance criteria (still in ROADMAP)

If pre-doc surfaces a phase-level design decision while the plan is being
written, file a phase ADR for it — do not bury it in the plan's Frozen
agreement.

## Archival policy

When a milestone reaches `status: completed`, follow this flow:

1. **CHANGELOG entry** — Add a milestone entry to `CHANGELOG.md` linking to
   the per-phase ADRs and the GitHub Release.

2. **Distill** — Check whether the plan contains any unique residual
   information not already captured in ADRs, ROADMAP, CHANGELOG, or git
   history. If yes, transcribe it into the appropriate place (an ADR or
   `docs/notes/`).

3. **Delete** — By default, delete the completed plan. The git log
   preserves it; `git show` can recover its full content. Commit message:
   `chore(plans): retire <M>-plan.md (<M> complete; content distilled into ADRs and CHANGELOG)`

4. **Exception** — Move to `docs/plans/archive/` (create the folder on
   first use) only if there is a documented reason to keep the plan
   accessible without going through git. Add to frontmatter:
   ```yaml
   status: archived
   archived-reason: <why this plan must remain accessible>
   ```
   "I might want to look at it later" is not a reason; git can serve that.

The default is **delete**. Files that linger consume context and invite
confusion about whether they are still authoritative.
