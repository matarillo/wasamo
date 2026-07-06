# Vision Decision Record — Author-controllable sizing surface (Problem B)

**Status:** Accepted

**Scope:** roadmap ownership and public-draft positioning for explicit
author-controllable widget sizing (`width` / `height` class of surface).
Surfaced by M3-Phase 7b and raised during M3-Phase 8 framing. This record
does **not** implement sizing and does **not** change M3 acceptance criteria.

## Context

M3 proved that the DSL can express real layout structure, but it also exposed
a gap: authors cannot say "make this widget 200 px wide" or "give this
placeholder a 4:3 area at this size." Widget size is currently determined by
kind defaults such as Fill, Shrink, and internal Fixed values. Some defaults
are author-reachable indirectly through layout primitives, but there is no
general author surface for explicit widget width / height.

This is **Problem B** from M3-Phase 7b:

- A Fill-default container such as Grid / ZStack can collapse to 0 when a
  Shrink-axis ancestor asks it for an intrinsic size.
- `Box aspect` inside an unconstrained cell can fail to arrange because the
  author has no way to provide a bounding size.
- The issue is not a placement regression. Phase 7b fixed a checker bug that
  blocked Grid-in-ZStack authorship (Problem A); Problem B is the older
  missing sizing surface becoming visible.

The live note
[docs/notes/author-controllable-sizing.md](../../../docs/notes/author-controllable-sizing.md)
records the evidence and says the roadmap responsibility must be assigned by
a cross-milestone Vision DR. M3-Phase 8 is the forcing point because A12
publishes the first DSL public draft: if the draft presents kind-default
sizing as final, readers will infer a stability promise that the project has
not made.

## Decision question

Where is the explicit sizing problem owned, and what does accepting this VDR
mean before the concrete syntax / IR / runtime design is done?

The decision must keep two layers separate:

- **Roadmap responsibility:** when the project must decide and, if required,
  implement explicit sizing before it becomes expensive or impossible to add.
- **Surface design:** whether sizing is a direct widget attribute, a modifier,
  parent-owned layout data, a runtime/host construction API, a reactive value,
  or something else.

This VDR settles the first layer and records the open design space for the
second. It deliberately avoids picking a grammar spelling before the owning
design phase compares the consequences.

## Options

### Option A — M3 / Phase 8 implementation

Implement explicit sizing now, before the public draft closes.

What you gain:

- The public draft could describe a complete sizing story immediately.
- The gallery can avoid Problem B workarounds.
- No future migration note is needed for width / height.

What you give up:

- M3's thesis is already full: selected state, integrated gallery, and public
  draft closure. Adding sizing would reopen grammar, IR, runtime layout, and
  possibly host construction at the end of the milestone.
- The design would be made under pressure from a visible bug, not from a full
  comparison of authoring models.
- It risks coupling sizing to today's implementation defaults instead of the
  long-term layout model.

Assessment: too early. The issue is real, but M3 close is the wrong design
moment for a cross-cutting surface.

### Option B — M4 interaction-stack ownership

Add explicit sizing to M4 because M4 owns input, focus, multi-window, and the
first showcase application.

What you gain:

- M4 is the next concrete application milestone; real interaction screens may
  require explicit hit areas, placeholder sizes, and resizable regions.
- Sizing decisions can be tested against an actual showcase instead of an
  isolated layout demo.

What you give up:

- M4's thesis is the interaction stack and focus model. Explicit sizing is
  adjacent but not inherently part of input / focus.
- Assigning implementation to M4 now may crowd the interaction work before the
  ABI impact is known.
- A fixed M4 assignment may force a grammar decision even if the next M4
  screens can still be expressed with existing layout primitives.

Assessment: plausible trigger, not the right unconditional owner.

### Option C — M5 identity / tooling ownership

Own explicit sizing in M5, alongside full theming and editor tooling.

What you gain:

- Sizing is author-experience surface; M5's VS Code tooling and theming work
  would benefit from a stable grammar and diagnostics.
- It avoids growing M4.

What you give up:

- If explicit sizing has C ABI or host-construction implications, waiting
  until M5 may leave too little room before M6 freeze.
- The first showcase may already have needed the surface in M4.
- The sizing issue is more fundamental than identity polish.

Assessment: reasonable for tooling polish, too late as the first
responsibility gate.

### Option D — M6 ABI-freeze gate only

Do not assign implementation to a feature milestone now. Instead, require an
explicit sizing **disposition before M6 ABI freeze**: audit grammar / IR /
runtime / C ABI impact, then either implement pre-freeze if ABI-bearing or
record why post-freeze append-only addition is safe.

What you gain:

- Aligns with the real irreversibility: once C ABI freeze and SemVer apply,
  ABI-bearing sizing becomes harder to add.
- Keeps M4/M5 design space open until a concrete application or ABI audit
  proves the need.
- Does not silently defer beyond 1.0.

What you give up:

- It is a gate, not a product milestone. Without an earlier trigger, the
  actual design could still arrive late.
- The public draft must carry an honest future note until the disposition
  exists.
- It does not solve gallery workarounds now.

Assessment: the best baseline responsibility rule, but it needs an earlier
activation trigger so it does not become "think about it at the last minute."

### Option E — dedicated pre-1.0 layout / DSL ergonomics phase

Create or reserve a dedicated pre-1.0 phase between M4/M5 and M6 for explicit
sizing and related layout ergonomics.

What you gain:

- Gives the sizing surface a clear home without stealing scope from M4 input
  or M5 identity.
- Encourages a full design pass over grammar, IR, runtime layout, host
  construction, diagnostics, and tooling.

What you give up:

- Adds roadmap structure before there is enough evidence that sizing needs a
  whole phase.
- Risks over-designing a surface that might be small once the ABI audit is
  done.
- Could turn every future layout question into a holding pen for this phase.

Assessment: a valid alternate if future evidence accumulates, but too strong
for this VDR.

### Option F — post-1.0

Defer explicit sizing until after 1.0.

What you gain:

- Keeps all pre-1.0 milestones smaller.
- Lets real users supply evidence.

What you give up:

- If the surface is ABI-bearing, post-1.0 addition is constrained by the
  compatibility policy.
- The 1.0 DSL may ship without a basic author sizing affordance.
- It contradicts the Phase 2 / 4 / 7b evidence that this is already a known
  missing surface, not a speculative future idea.

Assessment: too weak unless the M6 audit proves the surface is safely
append-only and not required by the 1.0 showcase.

## Design-space inventory for the owning phase

This VDR does not choose syntax, but the later design must compare at least
these families. Recording them here prevents the roadmap decision from
collapsing prematurely into today's easiest implementation.

| Family | Shape | Merit | Cost / risk |
|---|---|---|---|
| Direct widget attributes | `Box { width: 200; height: 150 }` or equivalent | Simple author mental model; solves the visible Problem B cases directly | Adds cross-widget attributes and precedence rules against kind defaults, `aspect`, and parent constraints |
| Modifier-like sizing | `Box { size { width: 200 } }` or future modifier syntax | Could generalize with other styling / layout modifiers | Opens a generic modifier system before it is otherwise accepted |
| Parent-owned layout data | size supplied as slot / parent data | Keeps layout authority with the parent container | Does not solve all "widget wants a size" cases and may confuse sizing with placement |
| Layout primitive wrapper | `Frame { width: 200; child ... }` style wrapper | Clear node in the tree; avoids every widget gaining size attrs | Adds a new container / node and can make common authoring verbose |
| Runtime / host construction API | host sets size constraints imperatively | Useful for generated UI or ABI construction | Does not solve author-facing `.ui` alone; risks splitting DSL and host semantics |
| Reactive sizing | width / height can be bound to state or expressions | Powerful for resizable / adaptive UI | Pulls sizing into reactive invalidation and expression semantics; may exceed 1.0 needs |
| Layout algorithm change only | change Fill/Shrink measurement defaults | Can reduce collapse symptoms without new grammar | Treats a missing author control as a hidden runtime behavior change; may break existing layout expectations |

The later design may add more families, but it must not skip these without
recording why.

## Recommendation

Adopt **Option D as the irreversibility backstop, plus a scheduled M4/M5
activation trigger** layered on top of it.

This is a deliberate strengthening of the bare Option D, not Option D alone.
The earlier trigger is **no longer left demand-driven** ("act only if some
application happens to need sizing"). A purely conditional trigger can
silently never fire and dump the whole design onto the last moment before M6
— the weakness Option D's own assessment names. Explicit sizing is already a
*known-missing* surface (Phase 2 / 4 / 7b evidence), not a speculative one, so
it warrants a scheduled investigation rather than wait-and-see. Because of
this, the earlier claim that a bare Option D "keeps the M4/M5 design space
open" no longer applies as stated: this recommendation deliberately closes
part of that openness by committing a spike on a schedule. What stays open is
only the *syntax / IR / runtime / ABI shape* of the surface — the spike, not
this VDR, settles that.

Concretely:

- Explicit sizing is **not implemented in M3** and is not added as an M3
  acceptance criterion.
- The first public draft must carry a **future note** for author-controllable
  sizing; it must not present kind-default sizing as the final language model.
  The public draft carries the *future-work* framing **only** — it does
  **not** publish the M4/M5 schedule, which is a process commitment recorded
  here and in the roadmap, not an external promise (see DD-M3-P8-002).
- **A design spike lands no later than M5, preferred in M4.** Preferring M4
  leaves M5 room to implement. The spike's concrete targets are the
  already-documented Problem B repro cases (Grid / ZStack collapse under a
  Shrink ancestor; `Box aspect` arrange failure) plus any further case
  surfaced by M4's own screen selection (see mechanism below). The spike is
  scoped to producing the impact audit the M6 disposition needs anyway
  (grammar / parser / checker, IR + runtime layout, C ABI / host
  construction) — not to building out all the design-space families.
- **Implementation is conditional on the spike's conclusion.** If the spike
  finds the surface is ABI-bearing or required by a concrete screen, it is
  implemented in M5. If the spike concludes the surface is safely
  post-freeze append-only or warrants a dedicated later phase, it returns to
  the M6 disposition below.
- **The M6 ABI-freeze disposition is retained as the backstop** if the
  M4/M5 schedule slips: before freeze the project must still either implement
  the surface or record why append-only post-1.0 addition is safe.

### M4/M5 activation mechanism

To keep the schedule from silently collapsing back to "decide at the last
minute," **each of M4 and M5 planning must record a sizing-spike
disposition**: either "spike in this milestone (phase X)" or "defer to the
next milestone, because …". The disposition is an auditable artifact, not a
bare assertion. The **default is: spike in M4**; deferral to M5 requires a
positively recorded reason (see inputs below), because absence of a pull
toward M4 is not, by itself, licence to defer a known-missing surface.

Because M4 framing selects real screens via wireframes (as M3 did) and
deliberately does not force layout back into M3's expressible range, **M4
framing is the natural moment to run this decision**: the wireframes are the
live input for whether a real screen cannot be expressed without explicit
sizing.

Decision inputs for the M4 (and M5) planning disposition:

| Input | Knowable at planning? | Pushes toward | Caveat |
|---|---|---|---|
| A real M4 screen (from the wireframes) cannot be expressed without explicit sizing | Yes, once M4 wireframes exist | M4 (a live validation target beyond the synthetic repro cases) | Absence of such a screen is *not* permission to defer — this input only pulls toward M4 |
| M4 capacity: is the interaction stack already full, or is there room for a bounded spike | Yes | slack → M4; full → lean M5 | "Full" is the easiest excuse; a spike only produces impact tables, so require the load to be enumerated, not asserted |
| Can M5 absorb *both* spike and implementation if deferred, with M6 freeze immediately after | Roughly | M5 cannot absorb both → M4; can → defer OK | The backstop-collapse guard: spike + implement both in M5-just-before-freeze is the dangerous compression |
| Does a promising design family depend on M4's interaction context (host construction, reactive / bound sizing, hit-area sizing) | Partially (architectural judgment) | depends on it → later-M4 or M5; pure-DSL families (attribute / wrapper) → early-M4 | Double-edged and easy to rationalise either way; pin *which* families are in play and whether they touch input |
| How narrow the design-space family set is (the seven families in the Design-space inventory above) | Yes, from prior notes | narrow → small spike fits M4; wide → larger spike | "Still wide" is not a deferral reason — narrowing is the spike's job; width bounds the spike's scope instead |
| Are the Problem B repro cases still reproducible on current main and sufficient as spike targets | Yes, testable now | reproducible → M4 anytime; drifted / masked → re-establish first | Verify; do not assume the repro still fires (a checker fix may have masked the collapse) |
| Runway to M6 ABI freeze | Yes | shrinking → pull to M4; ample → M5 OK | The hard-deadline reality answering "why not just wait" |

Explicitly **not** inputs: the surface's *actual* ABI impact (that is the
spike's output — gating the spike on it is circular; only a prior estimate
counts), and re-litigating whether users want sizing (already settled as
known-missing by Phase 2 / 4 / 7b).

**Default rule:** spike in M4; defer to M5 only when M4 has no room *or* M5
can absorb both spike and implementation is positively shown. A real M4
screen needing sizing strengthens M4 further; the family-width,
interaction-dependency, and repro-readiness inputs tune the spike's timing and
scope rather than justifying deferral on their own; a shrinking runway raises
the bar for deferral.

### Costs this recommendation accepts

- **M5 scope growth.** If the spike concludes implementation is warranted, M5
  (identity / theming / tooling) gains a sizing implementation it did not
  originally scope. M5's phase structure is not yet defined, so this is not
  fatal, but it is a real added load, not free.
- **Some design commitment ahead of a real showcase.** Scheduling the spike
  guarantees progress but may run partly ahead of a driving application; the
  Problem B repro cases and M4 wireframes anchor it, and full ergonomic
  validation against a real app is carried as a spike residual, not a spike
  precondition.

Accepted meaning, if this recommendation is accepted:

- The project has **not** chosen `width` / `height` syntax.
- The project has **not** chosen whether sizing is a widget attribute,
  modifier, wrapper, parent data, runtime API, or reactive binding.
- The project **has** chosen that a sizing design spike lands no later than
  M5 (preferred M4), that implementation follows in M5 if the spike so
  concludes, and that explicit sizing cannot silently slip past M6 ABI freeze
  without a recorded disposition.
- Phase 8's only implementation-side responsibility is mitigation /
  documentation: work around gallery sizing within shipped surface where
  reasonable, and record unresolved cases in M3 handoff.

## Consequent edits if Accepted

- `docs/dsl_spec.md` (via DD-M3-P8-002): add a future note saying M3 uses
  kind-default sizing and that author-controllable sizing is an unresolved
  pre-1.0 item. Do not reserve exact syntax, and do **not** state the M4/M5
  schedule — the public draft carries future-work framing only, not the
  process schedule.
- `process/_roadmap.md`: **required at Accept.** Record the scheduled trigger
  — a sizing design spike no later than M5 (preferred M4) and implementation
  in M5 if the spike so concludes — alongside the existing M6 "C ABI freeze"
  disposition as the backstop. Naming M4/M5 as owners is a roadmap
  commitment, so this edit is no longer optional.
- `process/milestone-3/handoff.md` (at M3 close): carry both the explicit
  sizing disposition to the pre-M6 gate **and** the M4/M5 schedule, and
  record any concrete gallery workaround left behind.
- M4 planning (and M5 planning if deferred): record the sizing-spike
  disposition per the M4/M5 activation mechanism above (default M4; deferral
  positively justified) as an auditable artifact.
- `docs/notes/author-controllable-sizing.md`: after this VDR is Accepted,
  replace the open-question home with a pointer to this decision and keep any
  remaining design sketches there only as non-normative notes.

## Out of scope

- Implementing explicit sizing in M3.
- Changing Grid / ZStack / Box measure-arrange semantics in this VDR.
- Choosing syntax, precedence, units, constraints, or binding behavior.
- Deciding the PM-2 wrapper rule.
- Adding a generic modifier system.
- Defining public C ABI additions.

## Verification / review expectations

The later owning design must provide:

- a grammar / parser / checker impact table;
- IR and runtime layout invalidation impact;
- C ABI / host-construction impact audit, including whether post-freeze
  append-only addition is sufficient;
- interaction with `aspect`, Fill/Shrink defaults, Grid tracks, ZStack
  alignment, ScrollView viewport, and WrapPanel item sizing;
- diagnostics for impossible or under-constrained combinations;
- GUI evidence for at least one case that previously collapsed or aborted,
  with a positive control distinguishing fixed sizing from coincidental
  rendering.

## Revision history

| Date | Change |
|---|---|
| 2026-07-01 | Initial Proposed draft raised from M3-Phase 8 framing / Problem B. |
| 2026-07-02 | Revised recommendation from bare "Option D + demand-driven trigger" to "Option D backstop + scheduled M4/M5 trigger": design spike no later than M5 (preferred M4), implementation conditional on the spike's outcome, M6 disposition retained as the backstop. Added the M4/M5 activation mechanism (default-M4 rule + decision-input table), a costs subsection (M5 scope growth; design ahead of showcase), made the `_roadmap.md` edit required, and kept the M4/M5 schedule out of the public draft. Still Proposed. |
| 2026-07-02 | **Accepted** after owner review. Consequent edits landed in the same commit: `_roadmap.md` records the scheduled trigger under M4 / M5 / M6, and `docs/notes/author-controllable-sizing.md` is repointed to this decision. |
