# Vision ADR — Document system redesign

**Status:** Accepted 2026-05-02

**Scope:** `README.md`, `VISION.md`, `ROADMAP.md`, new `CHANGELOG.md`,
`docs/plans/README.md`, `docs/plans/<M>-plan.md` structure.

This vision ADR records the decisions that restructure the project's
document system after M1 shipped. The motivation, alternatives
considered, and the discussion that produced these decisions are in
[docs/notes/doc-system-redesign.md](../notes/doc-system-redesign.md).

The core problem: `ROADMAP.md` had grown into a three-role document
(acceptance-criteria SSOT, phase-level task tracker, completed
milestone history). The roles drift apart over time and the
document cannot be frozen for any of them. This ADR splits those
roles across documents whose lifecycles match their content.

## DD-V-010 — Acceptance criteria SSOT

**Status:** Accepted

**Context:** Acceptance criteria appeared in both `VISION.md §7` and
`ROADMAP.md`. The "ROADMAP is authoritative" rule was a convention,
not a structural guarantee, and drifted in practice.

**Options:**

Option A — Consolidate in ROADMAP; reduce VISION §7 to thesis-level
framing
- What you gain: structural guarantee (only one place to update);
  matches the existing "ROADMAP is authoritative" convention.
- What you give up: VISION readers need to follow a link for the
  full criteria.

Option B — Retire ROADMAP entirely; criteria live in VISION + plans
- What you gain: fewer top-level documents.
- What you give up: VISION conflates "why" with "what ships when";
  external readers lose a single place to see milestone shape.

Option C — Status quo (convention-based)
- What you gain: zero change.
- What you give up: drift recurs.

**Decision:** Option A — `ROADMAP.md` is the SSOT for acceptance
criteria. `VISION.md §7` keeps thesis-level framing (one paragraph
per milestone) and links to ROADMAP for criteria.

## DD-V-011 — ROADMAP role narrowing

**Status:** Accepted

**Context:** ROADMAP carried (a) acceptance criteria, (b) per-phase
task checklists, (c) completed milestone history. The three roles
have incompatible lifecycles — (a) is mostly stable, (b) churns
during execution, (c) is append-only. Holding all three forces the
document to grow without bound and prevents freezing any section.

**Options:**

Option A — ROADMAP carries acceptance criteria only; phase tracking
moves elsewhere; completed history moves to CHANGELOG
- What you gain: ROADMAP becomes a small, stable document; each
  role lives where its lifecycle fits.
- What you give up: more documents to navigate.

Option B — ROADMAP becomes a "current status" board; acceptance
criteria move to a separate document
- What you gain: a single place to see "what's happening now."
- What you give up: criteria drift into yet another location;
  ROADMAP semantics become non-standard.

Option C — Status quo (three roles).
- What you gain: zero change.
- What you give up: continued bloat.

**Decision:** Option A. ROADMAP holds acceptance criteria for
**all** milestones (future and past), but completed milestones are
compressed to a one-line entry pointing at the relevant ADRs and
the CHANGELOG. Per-phase checklists are removed entirely; their
purpose is taken over by the in-progress plan's progress section
(DD-V-015).

## DD-V-012 — Completed milestone history → CHANGELOG.md

**Status:** Accepted

**Context:** Completed milestones accumulated as full phase
checklists in ROADMAP, and a parallel revision-history table lived
in `VISION.md` Appendix B. Both are append-only history of what
shipped, both drift from the actual git tag history.

**Options:**

Option A — Introduce a `CHANGELOG.md` (Keep a Changelog format)
absorbing both ROADMAP's "completed" history and VISION's revision
table
- What you gain: industry-standard format for external readers;
  past and future are time-axis separated; VISION revision table
  (a long-standing duplication with git log) resolves at the same
  time.
- What you give up: one new top-level document.

Option B — Move completed milestones to `docs/history/` instead
- What you gain: avoids a top-level file.
- What you give up: invents a non-standard location for what is
  industry-standard CHANGELOG content; doesn't address the VISION
  Appendix B duplication.

Option C — Keep completed milestones in ROADMAP, accept bloat.
- What you gain: zero change.
- What you give up: the role-narrowing of DD-V-011 is impossible.

**Decision:** Option A — introduce `CHANGELOG.md`.

## DD-V-013 — CHANGELOG granularity and length control

**Status:** Accepted

**Context:** A CHANGELOG that records every phase or every commit
grows unboundedly and external readers stop reading. The
introduction of CHANGELOG (DD-V-012) is only viable if length is
controlled structurally, not by discipline.

**Options:**

Option A — Milestone-level entries only; each entry is a short
summary plus links to the milestone's ADRs and GitHub Release
- What you gain: 1.0 reaches roughly 70 lines total; each entry is
  scannable; phase-level detail lives in ADRs and release notes.
- What you give up: phase-level changes are not directly visible
  in CHANGELOG.

Option B — `Added/Changed/Fixed/Removed` sections per release with
sub-milestone granularity
- What you gain: standard Keep a Changelog format with full detail.
- What you give up: length grows unboundedly; bloat returns under a
  new name.

Option C — Detailed CHANGELOG with a separate archive file holding
older versions
- What you gain: visible portion stays short.
- What you give up: two more files to maintain than Option A.

**Decision:** Option A — milestone-level summary entries (3–5
lines) plus links. Older versions are folded with `<details>` if
the visible top of the file ever exceeds about a screen.

## DD-V-014 — "Current status" placement

**Status:** Accepted

**Context:** Once ROADMAP no longer carries phase checklists
(DD-V-011), an external reader has no 30-second answer to "what is
this project working on right now?"

**Options:**

Option A — Add a `## Status` line to `README.md`: one line of
prose plus links to the in-progress plan and the most recent
release
- What you gain: zero new documents; arrives where external
  readers already land.
- What you give up: requires maintenance discipline at milestone
  start / end.

Option B — Introduce `STATUS.md`
- What you gain: a dedicated file for status.
- What you give up: adds a document; risks growing into ROADMAP's
  old "current state" role.

Option C — Reintroduce a "Now" section in ROADMAP
- What you gain: status near the criteria.
- What you give up: re-entangles roles ROADMAP just narrowed
  (DD-V-011).

**Decision:** Option A — README has a one-line `## Status` block
linking to the in-progress plan and the latest CHANGELOG entry.

## DD-V-015 — Plan two-layer structure (frozen agreement + live progress)

**Status:** Accepted

**Context:** With phase checklists removed from ROADMAP
(DD-V-011), per-phase progress tracking still has a use: the
project owner wants a between-session memory aid, and Claude
benefits from a checked-state map. The existing `docs/plans/`
discipline freezes the plan once `status: in-progress` so the
plan cannot serve this. Three shapes were considered.

**Options:**

Option A — Owner-facing progress note in `docs/notes/progress-<M>.md`
- What you gain: respects existing plan/README rules; uses an
  established document type; Japanese-OK matches owner preference.
- What you give up: progress is divorced from the plan it tracks;
  Claude has to load two files to see "what was the plan, and how
  far have we got."

Option B — Plan stays one document, becomes live (drop the
`in-progress = read-only` rule)
- What you gain: zero new documents; nothing to split.
- What you give up: re-creates ROADMAP's role-mixing problem
  inside the plan — the same document holds the frozen agreement
  and the live progress, with no structural separator.

Option C — Plan splits internally into `frozen-agreement` and
`progress` sections; the read-only rule applies to the former only
- What you gain: agreement and progress live where they belong
  (same file, different sections); plan/README's discipline is
  preserved with one rule revision.
- What you give up: plan is no longer a single-purpose document;
  the section boundary requires editorial discipline.

Option D — GitHub Milestones / Issues
- What you gain: GitHub-native UI; Claude can read via `gh`.
- What you give up: ties documents to GitHub; harder to read in
  bulk; the project does not currently use Issues / Milestones.

**Decision:** Option C — plan documents gain two top-level
sections, `## Frozen agreement` and `## Progress`. The
in-progress freeze rule applies to `Frozen agreement` only;
`Progress` remains live until the milestone completes. The plan
is still a single file; archival policy is unchanged (deleted on
completion by default).

## DD-V-016 — Plan → ROADMAP commit flow redefinition

**Status:** Accepted

**Context:** `docs/plans/README.md` defined the flow as
"plan(proposal) → ROADMAP(commitment) → ADRs". Under DD-V-011
ROADMAP no longer carries phase structure, so "commitment" can no
longer mean "transcribe phases into ROADMAP."

**Options:**

Option A — Plan still owns phase structure and dependencies;
ROADMAP only commits **acceptance criteria** for the milestone;
ROADMAP links to the plan for phase detail
- What you gain: each document carries one kind of content;
  ROADMAP stays small.
- What you give up: external readers who want to see phase shape
  for the active milestone follow a link.

Option B — Replicate phase structure in ROADMAP as before
- What you gain: ROADMAP self-contained for the active milestone.
- What you give up: contradicts DD-V-011.

**Decision:** Option A. The flow becomes "plan(proposal) →
ROADMAP commits acceptance criteria → plan tracks progress →
ADRs record per-phase decisions". `docs/plans/README.md` is
revised to match.

## DD-V-017 — Revision-history sections in non-normative docs

**Status:** Accepted

**Context:** The same duplication-with-git problem that motivated
removing VISION's Appendix B (DD-V-012) recurs in other top-of-file
revision-history tables. A critical re-examination distinguished
three cases:

1. `docs/architecture.md` — internal design explanation, not a
   normative specification. Carries a 15-row revision history that
   duplicates git log + ADRs.
2. `docs/dsl_spec.md` — the normative `.ui` DSL specification, kept
   deliberately separate from the reference implementation
   ([VISION §9.4](../../VISION.md#94-independence-of-the-core-specification))
   so future third-party implementations can target it. Same applies
   to the future `docs/abi_spec.md` after the M6 freeze.
3. ADR-internal "Revision history" sections (e.g. `phase-8-hello-counter.md`)
   — amendment trail within a single living ADR, not a duplicate of
   git log per se.

**Options:**

Option A — Apply DD-V-012's logic uniformly: drop revision-history
tables from all three
- What you gain: full structural consistency.
- What you give up: third-party implementers of the DSL spec lose
  the in-document version trail that is industry-standard for
  normative specifications (W3C, IETF, TC39).

Option B — Distinguish by document role: drop only from
non-normative docs
- What you gain: normative specs keep the version trail their
  external consumers expect; non-normative docs stop duplicating
  git.
- What you give up: introduces a per-document rule that has to be
  remembered.

Option C — Status quo
- What you gain: zero change.
- What you give up: duplication continues in `architecture.md`.

**Decision:** Option B.

- `docs/architecture.md` — revision history table and document
  version header removed. It is internal design documentation;
  per-phase changes are recorded in the phase ADRs.
- `docs/dsl_spec.md`, future `docs/abi_spec.md` — revision history
  retained. These are normative specifications maintained
  independently of the reference implementation
  ([VISION §9.4](../../VISION.md#94-independence-of-the-core-specification)),
  and external implementers need an in-spec version trail.
- ADR-internal "Revision history" sections — out of scope for this
  decision; they serve a different purpose (single-ADR amendment
  trail) than duplicating project history.

## Cross-references

- [docs/notes/doc-system-redesign.md](../notes/doc-system-redesign.md)
  — origin of these decisions; will be marked resolved once this
  ADR is filed.
- [docs/decisions/vision-post-m2-roadmap.md](./vision-post-m2-roadmap.md)
  — the previous vision ADR (DD-V-005..009); this ADR continues
  the V numbering.
