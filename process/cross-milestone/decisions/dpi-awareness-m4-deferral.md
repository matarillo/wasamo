# Vision Decision Record — Per-monitor DPI awareness placement, and the README vision-state framing

**Status:** Accepted 2026-05-30

**Scope:** `process/_roadmap.md` (M4 acceptance criteria — DD-V-022) and
`README.md` (lead vision note — DD-V-023 Option B). Surfaced by
M3-Phase 5 (Grid) T6 owner smoke.

This vision decision record settles where per-monitor DPI awareness is
owned (it places it in M4), and decides whether the `README.md` lead
paragraph needs an explicit signal that the capabilities it lists are
**target** capabilities rather than current-state guarantees. The
runtime gap it records is **pre-existing** (the runtime has been
DPI-unaware since M1); M3-Phase 5 only surfaced it, on a 125% high-DPI
display during the T6 owner smoke.

The discipline applied follows the post-M2 roadmap restructuring
([post-m2-roadmap.md](./post-m2-roadmap.md)): each milestone carries a
single **thesis**, and an acceptance criterion is added to a milestone
only when it serves that milestone's thesis. DPI awareness is placed in
M4 because it is a precondition for M4's identity-showcase thesis, not
because it is loosely "interaction-adjacent".

## Context

### The README describes the v1 target, not the current state

`README.md`'s lead paragraph claims rendering goes through
Windows.UI.Composition "so Mica/Acrylic, system theming, and high-DPI
composition all work out of the box." Cross-referencing
`process/_roadmap.md`:

- **Mica/Acrylic** is an **M4** acceptance criterion ("becomes
  demonstrable from this milestone"); in M3 it is only partial.
- **Full system theming** is an **M5** acceptance criterion.
- **High-DPI** appears in **no milestone's** acceptance criteria.

All three are therefore capabilities that are **not** fully realised
today; the README lists them anyway. The consistent reading is that the
README is a **vision / target description (the v1 picture)**, not a
current-state guarantee — if it were current-state, the Mica/Acrylic and
theming clauses would be equally "false", which no one treats them as.
High-DPI is one such target capability, on the same footing as its
Mica/Acrylic and theming siblings. (An earlier draft of this record
mis-read the high-DPI clause as a current-state claim and proposed
"correcting" it; that framing is withdrawn — singling out high-DPI for
correction while leaving its equally-unfinished siblings would be
incoherent.)

### The actual gap: a vision capability with no milestone home

What *is* a genuine gap is that high-DPI, unlike Mica/Acrylic (M4) and
theming (M5), has **no owning milestone**, while the runtime is
currently DPI-unaware. During M3-Phase 5 T6 owner smoke (2026-05-30, on
a 125% high-DPI display), code inspection established that
`wasamo-runtime` declares no DPI awareness:

- `window.rs`'s `create_hwnd` makes no DPI API call and ships no app
  manifest;
- layout consumes `GetClientRect` client pixels as logical units 1:1,
  with no DPI scale factor applied;
- there is no `WM_DPICHANGED` handler.

On a high-DPI monitor the DWM therefore bitmap-scales the whole window
(125% → a logical 800×600 window rendered as physical 1000×750, with
every element uniformly blurred). This is a **runtime** property, not a
`gallery-rust`-specific one, so it affects every host. [docs/notes/layout-engine.md §3.1](../../../docs/notes/layout-engine.md#31-dpi-スケーリングの局所化)
already flagged that DPI-scaling localization "needs reconsideration when
Grid / ScrollView land"; that trigger (M3 ScrollView / Grid) fired but
was not acted on inside M3.

## DD-V-022 — Per-monitor DPI awareness milestone placement

**Status:** Accepted

**Context:** Crisp high-DPI rendering is a README-promised (vision)
capability, but it has no owning milestone — `process/_roadmap.md` lists
it in no milestone's acceptance criteria — and the runtime is currently
DPI-unaware. It needs a single owner so the promised capability does not
silently slip to 1.0, exactly as Mica/Acrylic (M4) and full theming (M5)
each have an owning milestone.

**Options:**

Option A — M3 (resolve now, inside M3-Phase 5 or a follow-up M3 phase)
- What you gain: the capability ships sooner.
- What you give up: DPI is outside M3's thesis (the DSL surface is
  expressive enough to write real layouts); it is unplanned scope with
  no M3 phase owning it; it would distort the Grid close.

Option B — M4 (add as an M4 acceptance criterion)
- What you gain: aligns with M4's "Mica/Acrylic becomes demonstrable /
  first showcase ships" thesis — crisp rendering is a precondition for
  that identity demonstration, and the window/runtime surface DPI
  awareness touches is the same surface M4's input / focus / multi-window
  work touches.
- What you give up: M4 grows by one acceptance criterion.

Option C — post-1.0
- What you gain: the smallest pre-1.0 surface.
- What you give up: ships a 1.0 whose identity feature (Mica/Acrylic) is
  visibly undermined by blur; directly contradicts the M4 showcase
  thesis; defers a runtime-quality defect past the freeze.

**Decision:** Option B — M4. Per-monitor DPI awareness is added as an M4
acceptance criterion with the following text:

> **Per-monitor DPI awareness:** declare process / window DPI awareness,
> render crisply on high-DPI displays without DWM bitmap scaling, and
> handle DPI changes across monitors.

M3 does **not** address DPI: Grid (and the other M3 primitives) compute
correctly in logical pixels, and DPI is an orthogonal runtime-quality
concern that does not block any M3 thesis.

## DD-V-023 — Whether the README needs a vision-state note

**Status:** Accepted

**Context:** Because the README is a vision / target description (above),
high-DPI's not-yet-working state is **not**, on its own, a README defect
— Mica/Acrylic and theming are equally unfinished and equally listed.
The genuine question this episode surfaced is one of **readability**:
the present-tense "all work out of the box" can be mis-read as a
current-state guarantee (it was, in the first analysis of this very
record). Should the README explicitly signal that these are target
capabilities, so a reader does not mistake the vision for shipped state?

**Options:**

Option A — no note
- What you gain: the README stays as conventional vision prose; a pre-1.0
  repo with a visible roadmap is conventionally read as describing the
  target; no clutter in the lead; `process/_roadmap.md` remains the
  single status SSOT.
- What you give up: the present-tense "all work out of the box" keeps
  inviting the current-state mis-read.

Option B — a light vision note with a roadmap pointer
- What you gain: an honest, one-line signal (e.g. "Wasamo is pre-1.0;
  some capabilities described here are on the roadmap, not yet shipped —
  see the roadmap") sets expectations and forecloses the mis-read /
  future re-litigation, at near-zero cost.
- What you give up: one extra line in the lead; a faint hedge on the
  pitch.

Option C — soften the verb in place
- What you gain: changing "all work out of the box" to read as
  architectural intent (e.g. "are designed to work out of the box via the
  Visual Layer"), applied to all three clauses together (no singling-out),
  removes the present-tense guarantee with the smallest edit.
- What you give up: no roadmap pointer; the change is subtle and a reader
  may still over-read it.

**Recommendation:** Option B — a light vision note with a roadmap
pointer. It is the cheapest durable fix for the readability gap that this
episode demonstrated is real, and it generalises across all three listed
capabilities rather than singling out high-DPI. (Option C is an
acceptable lighter-touch alternative; Option A leaves the demonstrated
mis-read in place.)

**Decision:** Option B — a light vision note with a roadmap pointer is
added below the `README.md` lead. Final form (owner + Codex review,
pre-commit), verbatim from `README.md`:

```md
_Pre-1.0 note: some capabilities described here are roadmap targets rather than shipped guarantees; see [process/_roadmap.md](process/_roadmap.md)._
```

Three points fixed in review: (1) it is rendered as an **italic line,
not a blockquote**, because the README tagline is already a blockquote
and a second one collided with it visually/semantically (Codex); (2) it
stays **general** ("some capabilities") and **delegates the specifics to
the roadmap SSOT**, so it does not duplicate roadmap content or go stale
as capabilities ship; (3) it does **not single out high-DPI** — it
covers the lead's target capabilities uniformly.

## Consequent edits

Per [CLAUDE.md §Process rule lifecycle], SSOT edits land in the same
commit batch that flips this record to `Accepted`:

- `process/_roadmap.md` M4 — add the per-monitor DPI awareness acceptance
  criterion fixed in DD-V-022 (in this commit; DD-V-022 is accepted).
- `README.md` — add the DD-V-023 Option B vision note to the lead
  (wording owner-reviewed in this commit's README step).

This governance commit is **separate from the M3-Phase 5 Moment-2 spec
sync** (review-concern separation): it touches cross-milestone vision /
roadmap SSOTs, not the Grid implementation-sync.

**Out of this VDR's scope — a separate phase-end deliverable, not
replaced by this record:** the M3-Phase 5 *engineering* carry-forward
(the runtime DPI gap as an input to M4 framing) is written separately to
[`process/milestone-3/phase-5/implementation/handoff.md`](../../milestone-3/phase-5/implementation/handoff.md)
per the M3-Phase 5 phase-end retro (retrospectives.md item 15 / §6.3).
This record fixes the **vision / roadmap** decision; it is not the phase
handoff and does not discharge it.
