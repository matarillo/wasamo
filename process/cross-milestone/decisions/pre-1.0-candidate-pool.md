# Vision Decision Record — Pre-1.0 candidate pool (DD-V-028)

**Status:** Proposed

**Scope:** adds a non-AC section to `process/_roadmap.md` that holds
triaged pre-1.0 candidate items not (yet) assigned to a milestone, and
adds a per-item disposition duty to milestone planning (workflow §1.1).
This record does **not** create a small-item fast lane, a maintenance
lane, or an owner-directed milestone ("M5.5") — those are related but
separate decisions (see Out of scope).

## Context

Owner-side requirement seeds live in `docs/notes/` (exploratory,
Japanese allowed, zero ceremony). The roadmap is the SSOT for
acceptance criteria ([DD-V-010](./doc-system.md#dd-v-010--acceptance-criteria-ssot)),
so a desire that does not map cleanly onto an existing milestone
thesis has no schedule-visible home: it is either invisible to
planning (stays a note) or misfiled (the Post-1.0 section is
semantically wrong for items that **must** land before the M6 C ABI
freeze because the post-freeze surface is append-only).

The intake path itself exists — workflow §1.1 checks live notes at
milestone planning, and per-milestone intake notes
(e.g. [m4-interaction-intake.md](../../../docs/notes/m4-interaction-intake.md))
feed a specific milestone's framing. What is missing is a home for
items that are **triaged but unassigned**: decomposed enough to carry
scheduling-relevant tags, yet belonging to no milestone.

The [author-controllable sizing VDR](./author-controllable-sizing-surface.md)
already demonstrated the needed pattern — roadmap pressure without an
AC (scheduled trigger + M6 backstop) — but at full-VDR weight. That
weight is right for a cross-cutting surface; it is disproportionate
for small owner desires (e.g. "background `fill` on containers other
than `Box`"), which then pile up untracked. Origin discussion:
owner/agent session of 2026-07-07, seeded by widget-extension wishes
not yet in notes.

## Decision question

Where do triaged-but-unassigned pre-1.0 candidate items live so that
they are visible to milestone planning and to the M6 freeze decision,
without becoming acceptance criteria and without per-item VDR cost?

## Options

### Option A — status quo (notes + Post-1.0 section)

What you gain: no new structure; notes stay the single informal
surface. Not zero-mechanism: §1.1 already mandates a live-note
trigger check, and the pool is **re-derivable on demand** — the
2026-07-07 sweep produced the 12-item inventory from the existing
notes in one session, so nothing is permanently lost under A.

What you give up: the re-derivation is a repeated cost with no
persistent artifact — `ABI-bearing` verdicts and hold/retire
decisions leave no audit trail, so each planning re-does the sweep
and can silently reach different conclusions; unassigned desires are
invisible between sweeps; pre-freeze-required items have no surface
that confronts M6 planning with them; the Post-1.0 section absorbs
items it semantically cannot hold.

Assessment: fails not on capability but on auditability and repeat
cost. The current friction is evidence this is insufficient.

### Option B — per-item VDR (the sizing precedent generalized)

What you gain: every scheduled desire gets full options analysis and
an explicit trigger/backstop; maximum auditability; the
[sizing VDR](./author-controllable-sizing-surface.md) proves the
pattern works end-to-end (scheduled trigger, planning disposition,
M6 backstop).

What you give up: fixed cost per item is far above what small desires
justify — the exact failure mode that keeps them in notes today; the
day-one seed alone would add 12 records to
`cross-milestone/decisions/`, burying the genuinely heavy decisions
in the directory.

Assessment: wrong as the general mechanism, but **retained as the
complementary escalation path**: a pool row whose triage reveals a
cross-cutting surface (e.g. the `TypedValue` row) graduates to its
own VDR, exactly as sizing did. The pool does not replace B for
heavy items; it prevents B's cost being the entry fee for small
ones.

### Option C — AC-less "Pre-1.0 candidate pool" section in `_roadmap.md`

A dedicated roadmap section between M5 and M6 holding triaged items
with scheduling tags; milestone planning must record a per-item
disposition each time it runs.

What you gain: desires become schedule-visible in the hub document
planning already reads; the ABI-bearing tag makes the M6 freeze
confrontation structural rather than memory-dependent; entry cost is
one row, not one VDR. Two gains that mirror Option D's weaknesses:
**ambient visibility** — the pool sits physically between M5 and M6
in the file agents open at every design sync, implementation sync,
and phase framing, so stale rows and mid-milestone relevance (e.g. an
M4 TextField phase noticing the host-state-boundary row) are caught
incidentally; and **single-file `take` diffs** — adopting an item is
an intra-file move whose two halves (row removed, AC added) are
adjacent in one diff, leaving no silent-drop channel.

What you give up: the roadmap gains a section whose entries are not
commitments — readers must not mistake pool items for ACs; the pool
can silt up if the disposition duty is not enforced; and the
high-churn pool mixes into the low-churn AC SSOT, so "what changed in
the roadmap" reviews and git history get noisier. **Measured weight
(2026-07-07 seeding experiment):** a full `docs/notes/` sweep yielded
12 items on day one; even at one table row per item the section is 37
lines — already tying the largest active milestone section (M4, 35
lines). The growth direction is asymmetric: milestone sections shrink
to stubs when shipped, while the pool grows as notes and dogfooding
feedback accumulate.

**Owner counter-position (2026-07-07 review):** growth is not
unbounded in expectation. The notes corpus is finite, plannings
consume items, and the owner's prior is that the pool converges
around **twice the day-one seed (~25 items)**. Under that prior,
Option C's cost plateaus at "largest stable section in the file"
rather than growing without limit, and the size argument loses most
of its force.

Assessment: viable under the owner's bounded-growth prior; strained
if that prior breaks. The prior is falsifiable cheaply: each §1.1
disposition pass records the item count, and the pool exceeding ~25
items (or growing across two consecutive plannings despite takes) is
the signal to reassess.

### Option D — hybrid: item table in a dedicated file, stub in the roadmap

The item table and the disposition log live in
`process/candidate-pool.md`; `_roadmap.md` keeps a short AC-less stub
between M5 and M6 (definition + link — no item count, so the stub
cannot drift).

What you gain: the roadmap stays the AC SSOT plus one pointer,
regardless of how large the pool grows; the pool file can hold the
full item table without a size escape hatch; **churn-rate
separation** — the pool changes far more often than ACs, and D keeps
that churn out of the roadmap's git history and review passes; the
M5–M6 stub placement still confronts M6 planning with the pool's
existence (though not its contents — see below).

What you give up:

- **Ambient visibility is lost — structural, not fully closable.**
  `candidate-pool.md` is opened only when §1.1 fires (once per
  milestone); between plannings nobody sees the rows, so staleness
  detection is delayed and mid-milestone serendipity (a phase framing
  noticing a relevant row in passing) disappears. The stub confronts
  readers with the pool's existence, but a link is not a scroll —
  contents go unseen. The partial mitigation is operational, Soft
  tier: most mid-milestone pulls are owner-initiated (the desires are
  the owner's own), so the primary consumer does not depend on
  ambient exposure.
- **`take` dispositions span two files — a silent-drop channel.** The
  pool-row removal and the milestone-side landing are separate diffs;
  the exact failure DD-V-026's narrowing rules guard against.
  Mitigated to Forcing by the destination-link rule in the
  Recommendation below.
- **Link fan-out (minor):** four documents point at
  `candidate-pool.md`; renames or restructures touch more places.

Assessment: recommended by the agent. The measured seeding weight
shows Option C's size cost is immediate, while D's costs are
probabilistic and two of the three are boundable; but the visibility
loss is real and permanent, and under the owner's bounded-growth
prior (see Option C) the size argument against C weakens
substantially. C and D are closer than the day-one numbers suggest.

## Recommendation

Adopt **Option D (hybrid)** — the agent's recommendation. The initial
draft recommended Option C; the 2026-07-07 seeding experiment flipped
it to D on the measured size; the same-day critical pass then
surfaced D's visibility and two-file costs and the owner's
bounded-growth prior, which pull back toward C.

**Owner review position (2026-07-07): leans C.** The Accept decision
resolves the C-vs-D choice; both shapes are fully specified (rules
below are option-neutral except the Placement and size-hatch bullets,
and the Consequent edits section carries both variants). Whichever is chosen, the falsifier is
recorded: C is reassessed if the pool exceeds ~25 items or grows
across two consecutive plannings; D is reassessed if a planning finds
stale rows that ambient visibility would have caught, or a `take`
loses its landing half despite the destination-link rule.

Rules of the pool:

- **Placement:** item table + disposition log in
  `process/candidate-pool.md`; `_roadmap.md` keeps a short AC-less
  stub between M5 and M6 — directly before the freeze the pool guards
  — containing the definition and the link only (no item count, so
  the stub cannot drift).
- **Entry criterion:** an item must be **triaged** — decomposed enough
  to carry the tags below — and must be a **capability desire or a
  freeze-relevant disposition duty**, not an open design question
  (design questions — e.g. widget id syntax, `else` / `switch` shape —
  stay in their notes until a desire pulls them). Undifferentiated
  wishes stay in `docs/notes/` (the owner-intake note) until an
  agent-assisted triage splits them. One owner wish may yield several
  pool items.
- **Item format:** one table row per item — what (one cell) +
  `ABI-bearing: yes | no | unknown` + a "leans" hint (which milestone
  or design space it gravitates toward, if any) + origin link (note /
  discussion / VDR). One row per item is a hard format bound: an item
  that needs more than a row needs its own note, linked from the row.
- **No size escape hatch needed:** the dedicated file absorbs pool
  growth; the one-row-per-item bound is about triage discipline, not
  space.
- **Lifecycle:** an item leaves the pool only by (a) adoption into a
  milestone — at planning (§1.1) or mid-milestone via
  [DD-V-026](./plan-revision-discipline.md) tier 2, (b) explicit move
  to the Post-1.0 section, or (c) rejection with a one-line reason.
  Never silently. **Destination-link rule (option-neutral):** every
  `take` / `retire` disposition line links its landing — the AC or
  plan section, the Post-1.0 entry, or the rejection reason — so a
  removal whose landing half is missing is auditable rather than a
  silent drop.
- **Freeze backstop:** `ABI-bearing: unknown` must be resolved to
  `yes` or `no` no later than the last milestone planning before M6;
  M6 planning must record a disposition for every item still tagged
  `yes` or `unknown` (implement pre-freeze, or record why append-only
  post-freeze addition is safe — the same disposition shape as the
  sizing VDR's M6 gate).

**Enforcement tier: Forcing.** Each run of milestone planning (§1.1)
records a per-item disposition — `take (milestone N)` / `hold` /
`retire (Post-1.0 | rejected)` — as a dated line in the disposition
log in `process/candidate-pool.md`. The recorded disposition is the
auditable artifact: a reviewer can check it against the pool's
contents and the milestone's framing.
A Hard tier (CI check) is not feasible for a judgment-bearing list; a
Soft tier (prose only) is exactly what lets pools silt up.

## Consequent edits if Accepted

Common to both options (same commit batch as the Accept flip, per
AGENTS.md §Process rule lifecycle):

- `process/procedures/workflow.md` §1.1: add the pool as a third
  mandatory planning input with the disposition-recording duty,
  pointing at wherever the item table lives.
- `docs/notes/owner-intake.md`: repoint its triage-destination list
  from "candidate pool (planned)" to this decision and the pool's
  location.
- The pool is seeded with the 12 items from the 2026-07-07 full-notes
  triage sweep (background-color decomposition + one item per live
  note with an unassigned pre-1.0 desire).

If **Option D** is chosen at Accept:

- `process/candidate-pool.md`: new file — rules summary + item table
  + disposition log.
- `process/_roadmap.md`: the "Pre-1.0 candidate pool" section becomes
  a stub between M5 and M6 (definition + link only).

If **Option C** is chosen at Accept:

- `process/_roadmap.md`: the full section (rules summary + item table
  + disposition log) stays between M5 and M6 — the current working
  state; no new file.

## Out of scope

- Small-item fast lane and maintenance lane (proportional process for
  implementing small items / bug fixes). Deliberately deferred until a
  concrete small item exercises the need; the pool only *holds* items,
  it does not define how they are implemented.
- An owner-directed milestone ("M5.5"). Deferred to M5 close: if
  ABI-bearing items remain in the pool then, that decision is taken
  with meta-ACs and a close condition, per the 2026-07-07 discussion.
- Any change to AC semantics or to DD-V-010.
- Triage verdicts for specific items beyond the initial seeding.

## Revision history

| Date | Change |
|---|---|
| 2026-07-07 | Initial Proposed draft from the owner intake / candidate pool discussion. |
| 2026-07-07 | Seeding experiment (full `docs/notes/` sweep, 12 items) run against the owner's roadmap-balance concern. Item format tightened to one table row (hard bound); entry criterion refined to exclude open design questions; growth valve added (pre-authorised split to `process/candidate-pool.md` at ~20 items). Still Proposed. |
| 2026-07-07 | Recommendation flipped **Option C → Option D (hybrid)** on the seeding evidence: 12 items at one row each already tie the M4 section (37 vs 35 lines) and the growth direction is asymmetric. Option D rewritten from "separate backlog file" to the hybrid shape (item table + disposition log in `process/candidate-pool.md`, AC-less stub in `_roadmap.md`); C/D assessments rebalanced with the measured weight; growth valve retired (absorbed by D). Still Proposed. |
| 2026-07-07 | Full-slate critical pass (owner-requested). D's give-ups made explicit: ambient-visibility loss (structural, partially mitigable only at Soft tier), two-file `take` silent-drop channel (closed to Forcing by the new option-neutral destination-link rule), link fan-out; D gains churn-rate separation. C gains the mirrored strengths (ambient visibility, single-file `take` diffs) and the **owner's bounded-growth prior** (~2× the day-one seed) recorded as a counter to the size argument, with a cheap falsifier (item count per §1.1 pass). A sharpened to "re-derivable but non-auditable"; B retained as the escalation path for heavy items. Recommendation stays D (agent); owner review position leans C; Accept resolves. Consequent edits split into option variants. Still Proposed. |
