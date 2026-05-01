# Milestone Plans

This directory holds **milestone-level plans**: per-milestone breakdowns of
phases, dependencies, and scope, used as the agreement artifact between the
project owner and Claude before a milestone begins implementation.

## Relation to ROADMAP and ADRs

Plans are an **upstream artifact** that feeds into ROADMAP and ADRs:

```
docs/plans/<M>-plan.md   →  ROADMAP.md   →  docs/decisions/phase-*.md
   (proposal)              (commitment)       (per-phase decisions)
```

A plan proposes the structure of a milestone (which phases, in what order,
serving what acceptance criterion). Once agreed, ROADMAP is updated to
reflect the committed structure. As phases begin, each one's design
decisions go into a phase ADR — **not** back into the plan.

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
roadmap-anchor: ROADMAP.md#m2-alpha
adrs:
  - docs/decisions/phase-N-xxx.md   # filled in as ADRs are written
created: YYYY-MM-DD
last-aligned-with-roadmap: YYYY-MM-DD   # required once status >= in-progress
---
```

## Status lifecycle

| Status | Meaning | Editing rule |
|---|---|---|
| `drafting` | Pre-agreement. Owner and Claude iterate freely. | Free updates |
| `agreed` | Owner has signed off. ROADMAP update in progress. | Updates only to reflect agreement |
| `in-progress` | ROADMAP carries the commitment. Phases are executing. | **Read-only.** Update ROADMAP/ADRs instead |
| `completed` | All phases done. Ready for archival decision. | Read-only |

The key discipline: **once `in-progress`, the plan is frozen**. Execution
state lives in ROADMAP and ADRs, not in the plan. This prevents the plan
from becoming a competing source of truth.

## Scope rule (plan vs ADR)

Plans contain:
- Phase breakdown and ordering
- Dependencies between phases
- Acceptance criteria mapping (which phase satisfies which VISION criterion)
- Out-of-scope list for the milestone

Plans do **not** contain:
- Per-phase design decisions (those go into phase ADRs as `DD-P<N>-<seq>`)
- Implementation details
- Execution tracking (that lives in ROADMAP checkboxes)

If pre-doc surfaces a phase-level design decision while the plan is being
written, file a phase ADR for it — do not bury it in the plan.

## Archival policy

When a milestone reaches `status: completed`, follow this flow:

1. **Distill** — Check whether the plan contains any unique residual
   information not already captured in ADRs, ROADMAP, or git history. If
   yes, transcribe it into the appropriate place (an ADR, `docs/notes/`,
   or ROADMAP).

2. **Delete** — By default, delete the completed plan. The git log
   preserves it; `git show` can recover its full content. Commit message:
   `chore(plans): retire <M>-plan.md (<M> complete; content distilled into ADRs)`

3. **Exception** — Move to `docs/plans/archive/` (create the folder on
   first use) only if there is a documented reason to keep the plan
   accessible without going through git. Add to frontmatter:
   ```yaml
   status: archived
   archived-reason: <why this plan must remain accessible>
   ```
   "I might want to look at it later" is not a reason; git can serve that.

The default is **delete**. Files that linger consume context and invite
confusion about whether they are still authoritative.
