# Notes

Owner-authored exploratory notes, sketches, and open questions that do not
fit the formal structure of an ADR (`process/milestone-N/phase-M/decisions/`)
or a milestone plan (`process/milestone-N/plan.md`).

## When to put something here

- Brainstorming or hypothesis dumps that may or may not become decisions
- Long-lived "open questions" tied to a subsystem (e.g. layout engine)
- Cross-phase design sketches that the owner wants Claude to see during
  related work

## When NOT to use this folder

- A concrete decision was reached → write it as an ADR under
  `process/milestone-N/phase-M/decisions/`
- The content is execution tracking for an active milestone → it belongs
  in `process/milestone-N/plan.md` or `process/_roadmap.md`
- The information is captured by code, ADRs, the roadmap, or git history

## Language

Notes in this folder may be written in **Japanese** (exception to the
English-only rule for `docs/`). They are primarily for the project owner's
own reference; Claude reads them as context but does not require them to
be in English.

## Lifecycle

Notes are living documents. When a note's open questions are resolved,
either:

- Distill the resolution into an ADR and remove the resolved section from
  the note, or
- Mark the note as `status: superseded` in its frontmatter and link to the
  ADR that replaced it.

Stale notes that have no remaining live content should be deleted (git
history preserves them).
