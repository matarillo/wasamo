# Vision Decision Record — Process rule SSOT distribution

**Status:** Accepted 2026-05-26

**Scope:** `CLAUDE.md`, `process/README.md`, `process/procedures/retrospectives.md`,
`process/cross-milestone/decisions/`, and the forward-link discipline
between them.

This vision decision record captures how process knowledge (enforceable rules,
structural conventions, procedures, and rationale) is distributed
across SSOTs after the `process/` restructure. The motivation,
alternatives considered, and the discussion that produced these
decisions are in
[process/cross-milestone/decisions/exploration/process-rules-ssot.md](./exploration/process-rules-ssot.md).

The core problem: process knowledge had drifted across `CLAUDE.md`,
`docs/notes/retrospectives.md`, the per-folder READMEs under
`process/` and `docs/plans/`, and the vision decision records themselves.
The same rule sometimes appeared in two places with subtly different
wording. There was no rule for *where* a new process rule should
land, so additions accumulated wherever the moment suggested.

This vision decision record splits process knowledge across multiple SSOTs
by category, and defines a lifecycle for changing process rules.

## DD-V-019 — Process rule SSOT distribution

**Status:** Accepted

**Context:**
After the `process/` restructure (DD-V-010..018 plus the unnumbered
folder reorganisation of 2026-05-26), the natural homes for process
knowledge have changed. `docs/plans/README.md` and
`process/README.md` were consolidated into `process/README.md`.
`docs/notes/doc-system-redesign.md` became
`process/cross-milestone/decisions/exploration/doc-system-redesign-note.md` (a pre-decision
note, kept outside `decisions/` as a historical record). The remaining
question is which SSOT owns which category of process knowledge,
and how cross-references work without re-creating drift.

**Decision:** Five SSOTs, each owning one category. Other files link
rather than duplicate.

| SSOT | Owns |
|---|---|
| `CLAUDE.md` | Enforceable rules: language, testing, commit, CI, build order, retrospective, process-rule lifecycle |
| `process/README.md` | Structural conventions: folder roles, lifecycles, mutability |
| `process/procedures/workflow.md` | Development workflow: milestone/phase stages, document lifecycle, glossary |
| `process/procedures/retrospectives.md` | Retrospective procedure: when to run, checklist, doc-set, forward-carry |
| `process/cross-milestone/decisions/` | Vision decision records: the *why* behind the conventions |

**Forward-link discipline:**

- Default: link only, no duplicated content.
- Exception: when a fragment of the SSOT must be quoted for local
  context (e.g. a one-line summary at the top of a section), copy
  the SSOT's wording verbatim rather than paraphrasing. Verbatim
  copies are detectable by string match; paraphrases drift silently.
- Anchor-level links are encouraged where the SSOT exposes stable
  anchors. Anchor stability is the SSOT owner's responsibility.

## DD-V-020 — Process rule change lifecycle

**Status:** Accepted

**Context:**
Process rules historically were added by direct SSOT edits, often
surfaced mid-implementation when a retrospective revealed the gap.
This produced rules that are real but underdocumented: no rationale,
no alternatives considered, no record of what triggered the change.
Commit `b11688b` (2026-05-20, "doc commit is review-concern scoped")
is the canonical example — the rule is correct, but the rationale
lives only in the originating phase's framing postmortem rather than
in a vision decision record.

**Decision:** Two-tier lifecycle for process rule changes.

- **Minor edits** — wording adjustments, additional examples,
  clarifications that do not change the rule's content. Edit the
  owning SSOT directly. No ADR required.
- **Structural changes** — new enforceable rule, reversal of a prior
  decision, new category of process knowledge, or any change that
  requires touching another SSOT. File a vision decision record under
  `process/cross-milestone/decisions/` first; update the SSOT in the same
  commit batch that flips the ADR to `Accepted`.

**Boundary test:** A change is *structural* if either is true:
1. It requires touching another SSOT to remain consistent.
2. It supersedes or contradicts a prior decision.

If both are false, edit in place.

**Rationale:** The two-tier rule keeps minor maintenance friction-free
while ensuring substantive process changes leave a rationale trail
that future readers (and future Claude sessions) can follow.

## DD-V-021 — Phase-end verification evidence placement

**Status:** Accepted

**Context:**
Q6 of [process-rules-ssot.md](./exploration/process-rules-ssot.md)
asked where ADR verification closure / acceptance evidence should
live. The `process/` restructure resolved most of the boundary
question by splitting `tasks/` into `implementation/` (mutable
during the phase) and `retrospectives/` (append-only post-phase).
The remaining question is which file holds the final mapping of
"ADR verification criterion ↔ test/example/CI run that discharged
it" at phase close.

**Decision:** `retrospectives/phase-end.md` owns the final
verification mapping under a dedicated `## Phase-End Gate`
section. Per-task evidence (smoke screenshots, individual test
results) lives in `implementation/evidence/` and `implementation/log.md`
during the phase; the phase-end retrospective distills them into
the final closure mapping.

**Rationale:** The phase-end retrospective is the natural anchor
for "what discharged this phase's acceptance criteria" because it
is the document a future phase consults to understand what the
prior phase actually shipped. Per-task evidence has a different
lifetime (it accumulates during the phase) and a different reader
(the implementer or reviewer of that specific task), so colocating
it with the task-level execution log in `implementation/` matches
its access pattern.

## Revision history

| Version | Date | Notes |
|---------|------|-------|
| 0.1 | 2026-05-26 | Initial draft, Accepted same session |
