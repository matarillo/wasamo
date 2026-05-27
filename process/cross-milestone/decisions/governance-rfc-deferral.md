# Vision Decision Record — Governance: RFC adoption deferred to post-1.0

**Status:** Accepted 2026-05-25

**Scope:** VISION.md §9.2 and §11; docs/decisions/README.md §Scope and
relation to RFCs; cross-references in docs/notes/m2-to-m3-handover.md,
docs/notes/m3/m3-start-framing.md, and docs/notes/typed-value-evaluator.md.

This vision decision record resolves the governance drift flagged in
[m3-phase-3-wrap-panel.md §Governance note](./m3-phase-3-wrap-panel.md#governance-note-m3-phase-adr-vs-rfc-transition).
The original VISION text — "M3 onward. Gradual transition to RFC-based
consensus" — predates the post-M2 roadmap restructuring
([vision-post-m2-roadmap.md](./vision-post-m2-roadmap.md), 2026-05-02)
that redefined M3 as the *DSL surface* milestone and pushed 1.0 out
to M6. Under the original milestone scheme (M2 Alpha / M3 Beta /
M4 1.0) the M3 transition pointed at a near-1.0 maturity boundary;
under the current scheme it points at the *first DSL-surface
milestone*, which is far too early to be a governance turnover
point. The text was not updated when the milestones were renumbered.

The observed practice confirms the same gap: M3 Phase 1, Phase 2, and
Phase 3 all ran as ADRs without invoking any RFC process, and
`docs/rfcs/` does not exist in the repository (the link in
VISION.md §11 is dead). This ADR aligns the documented governance
schedule with reality.

## Context

VISION.md §9.2 currently describes a three-stage trajectory:

1. **Early stages (M1-M2).** BDFL, ADRs.
2. **M3 onward.** Gradual transition to RFC-based consensus,
   `docs/rfcs/`.
3. **Post-1.0.** Fully open governance, possible TSC.

Stage 2 is the stale layer. It carries two assumptions that no
longer hold:

- That M3 is close enough to 1.0 to begin community governance
  turnover. Under the current ROADMAP, M3 is the first of four
  pre-1.0 milestones; 1.0 ships at M6.
- That `docs/rfcs/` is set up and routable. It is not, and Phase 1
  through Phase 3 of M3 have all run successfully on ADRs
  without it.

Two resolution paths exist:

- **Path A (chosen).** Revise VISION to defer RFC adoption to
  post-1.0. The pre-1.0 phase remains BDFL with phase / vision decision records;
  RFC machinery is set up only when fully open governance begins.
- **Path B (rejected).** Stand up `docs/rfcs/` now and re-route
  M3 phase governance through it. Rejected because (a) M3 Phase 1
  through Phase 3 already ran on ADRs and re-routing
  retroactively would reframe completed work, (b) the project is
  still pre-alpha and the contributor base is the BDFL alone, so
  the RFC machinery has no community to serve yet, and (c) setting
  up RFC process is itself a non-trivial design exercise (template,
  lifecycle, acceptance rule) that pulls effort away from M3
  DSL-surface work.

## DD-V-018 — Defer RFC adoption to post-1.0

**Status:** Accepted

**Context:** The "M3 onward = RFC transition" wording in VISION.md
§9.2 and §11 dates from the pre-restructuring milestone scheme
(M3=Beta, M4=1.0). After the 2026-05-02 restructuring (1.0 → M6,
M3 → DSL surface), the wording points at a milestone four releases
earlier than 1.0, which is structurally too early to be a
governance turnover point. Observed practice (M3 Phases 1–3 all on
ADRs, `docs/rfcs/` absent) confirms the schedule never
matched reality.

**Options:**

Option A — Collapse to two stages: pre-1.0 BDFL, post-1.0 open
governance (with RFC machinery introduced at the post-1.0 boundary)
- What you gain: schedule matches observed practice; one fewer
  transition for the project to navigate before 1.0; no broken
  forward references to a `docs/rfcs/` directory that does not
  exist; aligns governance turnover with the same milestone where
  ABI is frozen and SemVer commitments take effect.
- What you give up: no signposted intermediate "warming up" stage
  for community RFC practice; the jump from BDFL to fully open
  governance is taken in one step at 1.0.
- Technical risk: Low. Doc-only change.

Option B — Move the transition to a later pre-1.0 milestone (M5)
- What you gain: preserves a three-stage trajectory; gives RFC
  practice one milestone of runway before 1.0.
- What you give up: still commits to standing up `docs/rfcs/` and
  the RFC process before 1.0 ships; no evidence yet that the
  project will have a contributor base large enough to need it
  before then; introduces a second governance transition close to
  1.0, where attention should be on ABI freeze and showcase.
- Technical risk: Low. Doc-only change, but a commitment to do
  process work later.

Option C — Leave the existing text but add a "currently deferred"
note
- What you gain: preserves the original aspiration verbatim.
- What you give up: keeps a dead link (`docs/rfcs/`) live in
  VISION; carries a commitment the project has no evidence it can
  meet on the stated schedule; future readers must reconcile the
  aspiration text with the deferral note.
- Technical risk: Low.

**Decision:** Option A — collapse to two governance stages.
Pre-1.0 (M1 through M6) remains BDFL with ADRs and vision
ADRs. Post-1.0 introduces fully open governance and RFC machinery
together at the same boundary. The `docs/rfcs/` directory is not
created until the post-1.0 governance work begins; references to
RFC in pre-1.0 documents are either rewritten as "ADR" or framed
as future-tense post-1.0 process.

**Forward-compat exposure:** Low. Option A makes no commitment
about post-1.0 governance shape beyond "RFC and open governance
begin together"; the actual RFC template, lifecycle, and
acceptance rule are deferred to a post-1.0 vision decision record. If
contributor activity grows faster than expected during M4 or M5
and an earlier RFC transition becomes desirable, this DD can be
superseded with a new vision decision record pulling the transition forward
— that is the same revision affordance available to any vision
DD.

## Doc-side edits required

| Document | Change | Rationale |
|---|---|---|
| VISION.md §9.2 | Replace the three-bullet trajectory with a two-bullet one: "Pre-1.0 (M1–M6)" BDFL + ADRs; "Post-1.0" fully open governance + RFC machinery introduced together | Implements DD-V-018 |
| VISION.md §11 | Drop the "From M3 onward, substantial feature proposals follow the RFC process in `docs/rfcs/`" sentence; replace with pre-1.0 ADR guidance and a forward-tense note that RFC process begins post-1.0 | Removes the dead `docs/rfcs/` link and matches DD-V-018 |
| docs/decisions/README.md §Scope and relation to RFCs | Rewrite the "From M3 onward" sentence to "From post-1.0 onward"; keep the ADR-as-authoritative-record framing for the entire pre-1.0 period | Matches DD-V-018; aligns with realised M1–M3 phase-ADR flow |
| docs/notes/m2-to-m3-handover.md line ~193 | "binding ADR or RFC" → "binding ADR" (pre-1.0 timeframe) | Consistency with DD-V-018 |
| docs/notes/m3/m3-start-framing.md lines 15–16, 167 | "ADR / RFC" enumeration and "M3-era RFC" mention → ADR-only framing; M3 is pre-1.0 so no RFC process is in effect | Consistency with DD-V-018 |
| docs/notes/typed-value-evaluator.md line ~90 | "ADR または RFC" → "ADR" (note is owner-authored Japanese; RFC reference is pre-1.0 so collapses to ADR per DD-V-018) | Consistency with DD-V-018 |
| m3-phase-3-wrap-panel.md §Governance note | Add a closing line pointing at this vision decision record as the resolution; do not rewrite the historical note itself (per supersede rule) | Resolves the governance gap the Phase 3 ADR flagged |

## Out of scope

- Defining the RFC template, lifecycle, or acceptance rule. Those
  are post-1.0 work and belong in a separate vision decision record when the
  RFC machinery is actually being set up.
- Creating `docs/rfcs/` or any placeholder structure for it.
- Revisiting the post-1.0 TSC framing in VISION §9.2.
- Adjusting CONTRIBUTING.md or related guides — they do not
  currently reference the RFC schedule.

## Summary

| DD | Decision | Forward-compat exposure |
|---|---|---|
| DD-V-018 | Collapse to two governance stages: pre-1.0 BDFL + ADRs, post-1.0 open governance + RFC introduced together | Low |
