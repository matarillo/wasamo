# DD-M4-P1-004 — The outward unit contract and its wording in the three specs

**Status:** Accepted
**Phase:** M4-Phase 1
**AC:** Not named by AC7's wording. Carried by plan phase-end criterion
4 (spec synchronization), and by the fact that
[docs/abi_spec.md](../../../../docs/abi_spec.md) freezes at M6 — after
which a unit change is a compatibility break.

## Context

While the scale factor was always 1, "what does `width: 800` mean" had
no observable answer and no cost to leaving unanswered. It now has one.

None of the three normative specs currently defines a unit
([constraints §5](../requirements/constraints.md), re-verified at
drafting time):

- `docs/abi_spec.md` §4.2 declares
  `wasamo_window_create(title_utf8, title_len, width, height, out)` and
  says nothing about what `width` counts. It is the **only** ABI
  function that carries a coordinate — `WindowState` holds pointer and
  resize callback slots, but no ABI entry point sets or reads them.
- `docs/dsl_spec.md` says dimension attributes are "pixel extents in the
  layout coordinate system" and never says what that coordinate system
  measures.
- `docs/architecture.md` §12 carries the DPI question as `Open`.

Two consequences make this a decision rather than an editing task.
First, **the ABI freeze at M6**: a `width` whose unit is stated after
hosts have shipped against it cannot be corrected. Second,
[DD-M4-P1-002](./dd-m4-p1-002-coordinate-space-and-conversion-boundary.md)
has now defined the internal space, so the outward statement either
matches it or introduces a second translation for no reason.

## Decision dependency summary

Consumes DD-002's space definition. Feeds the Moment 1 design-sync
commit set (preamble §Upstream document revisions). Tests framing
agreement ③.

## Sub-issues

- **The unit of the ABI window size**, and what the arguments denote.
- **The unit of DSL dimension values and the typography ramp.**
- **Whether a host needs to be able to query the scale factor** — the
  question agreement ③ reserved.
- **Which document says what**, and the accompanying note updates.

## The unit

### Options

- **W1 — DIP everywhere outward-facing.** `wasamo_window_create`'s
  `width` / `height`, every DSL dimension literal, and the typography
  ramp are DIP — the same unit DD-002 gave the layout space.
- **W2 — Physical pixels outward-facing**, with the runtime converting
  inward.
  - Rejected on merit: it makes every authored `.ui` and every host
    window size resolution-dependent, which is the property the whole
    phase exists to remove. A host would have to query the scale before
    it could ask for a window of a sensible size, which manufactures
    exactly the ABI addition agreement ③ is trying to avoid — and it
    would make an author's `.ui` render at a different physical size on
    different machines, which is the DPI bug, restated as a contract.
- **W3 — Split: DIP for DSL, physical for the ABI.**
  - Rejected on merit: two units at one boundary, with the ABI's unit
    being the one a host is least equipped to reason about. It buys
    nothing — there is no host use case for "a window exactly 800
    device pixels wide" that the DIP form cannot express once the scale
    is known, and none that M4 has.

### Comparison

W1 is the only option under which an author's and a host's numbers mean
the same thing as each other and as the engine's, and the only one that
keeps a `.ui` file resolution-independent. W2 and W3 both push the scale
factor outward to parties that would then need it — which is a
product-merit argument, and separately would have forced the ABI
addition below.

### Recommendation

**W1 — DIP.**

**Backward compatibility.** At 100% every existing host is bit-identical
under the new statement, so nothing breaks. The honest framing is that
this states a semantics that was previously *unstated*, not that it
changes one: the runtime had exactly one behaviour, and DIP is the
description of it that stays true at other scales.

### What `width` / `height` denote

They are passed straight to `CreateWindowExW`'s `nWidth` / `nHeight`,
which size the **outer window rectangle** — frame and caption included —
not the client area. That has always been true and is not restated here
as a change; it is written down because a spec that says "DIP" without
saying "of what rectangle" is not reproducible by an external reader.

Redefining them as a **client** size was considered and rejected: it
would silently change every existing host's window size at 100%, which
is a real behaviour break in exchange for a nicer definition. The
client-size form belongs with the `WindowConfig` surface (AC11,
M4-Phase 8), where it can arrive as a new, explicitly-named attribute
rather than as a reinterpretation of an existing argument.

### Which DSL values this covers

Every dimension-bearing literal already in the language, stated once
normatively and referenced rather than repeated at each site: WrapPanel
`item-cross-size` / `item-spacing` / `line-spacing`, Grid fixed track
sizes, ScrollView `offset-y`, and Box's dimension surface. The
typography ramp — 12 / 14 / 20 / 28 — is DIP font size, which is what
`TypographyStyle::size_sp` already feeds to
`IDWriteFactory::CreateTextFormat` under a 96-DPI context, so this too
is a statement of the existing behaviour rather than a change to it.

No IR change and no `wasamoc` change: the unit is a semantic statement
about existing `i32` / `f64` literals, not a new encoding. (This is why
the schema-migration implementation gate is judged non-applicable for
the phase.)

## Does the host need the scale factor?

This is the question framing agreement ③ reserved, and it decides
whether a stage-2 plan revision must be proposed before this phase can
proceed.

### The test applied

A host needs the scale factor only if it must express something in
device pixels. Working through what a host does today and through M4:

- **Creating a window** — DIP, by W1. No.
- **Loading a `.ui`** — no coordinates cross the boundary. No.
- **Getting and setting properties** — `wasamo_get_property` /
  `wasamo_set_property` carry authored values, which are DIP. No.
- **Receiving signals** — `clicked` carries no coordinates.
- **Pointer and resize callbacks** — the slots exist on `WindowState`
  but **no ABI function installs them**; they are runtime-internal.
  Nothing crosses.
- **M4-Phase 2 (input, focus)** — the routing and focus surfaces are
  authored in `.ui`; if a host-visible pointer coordinate is ever
  exposed, it is exposed in DIP by this contract, not by a scale query.
- **M4-Phase 5 / 6 (TextField, IME)** — the caret and composition
  rectangles are computed *inside* the runtime and handed to TSF. The
  host is not in that path.
- **M4-Phase 7 (host state boundary)** — a host supplies and replaces
  *values*. If a host ever supplies a **dimension**, it supplies it in
  DIP.
- **M4-Phase 8 (multi-window, `WindowConfig`)** — the most plausible
  origin. A host that wants "as large as this monitor allows" needs
  either the scale or a DIP-denominated work-area query.
- **M4-Phase 9 (top layer)** — placement is runtime-internal this
  milestone (widget-anchored placement is explicitly excluded from
  AC10).

### Conclusion

**No new ABI function in M4-Phase 1, and none needed in M4 as
currently planned.** Framing agreement ③ holds, and **no stage-2 plan
revision is proposed** — the plan's "M4-Phase 7 is the milestone's only
ABI-bearing phase" survives untouched, because stating the semantics of
existing arguments moves no signature.

**Recorded trigger.** If a host must express or receive a length that is
*not* expressible in DIP — a device-pixel budget, a monitor work area,
a screen coordinate — the surface lands in the **M4-Phase 7 ABI wave**
(or M4-Phase 8 if it arrives with `WindowConfig`), as a scale or
work-area query designed with the rest of that wave. It does not get
retrofitted here, and it does not get pre-built on a prediction: DD-001
already establishes that the runtime is correct under any effective
awareness without the host knowing anything, so there is no correctness
motive for the query, only a convenience one that no concrete case has
yet asked for.

## Which document says what

Division of labour, so no statement lives in two places
([AGENTS.md](../../../../AGENTS.md): each document type has one role):

- **`docs/architecture.md`** — the **normative coordinate-space
  section**: the two spaces and their definitions, the conversion seams
  as a class, the text-surface resolution contract, and the invariant
  that layout results do not depend on the scale factor. This is the
  runtime-architecture statement, and it is the one an external
  implementer would reproduce the behaviour from. §12's DPI
  open-questions row moves from `Open` to resolved, pointing here.
- **`docs/dsl_spec.md`** — the **author-facing unit**: dimension values
  and font sizes are DIP, `1 DIP = 1/96 inch`, and an authored layout
  is therefore identical at every scale factor. Stated once
  normatively; the dimension-bearing sections reference it instead of
  repeating it, and the existing "pixel extents in the layout
  coordinate system" wording is replaced rather than annotated.
- **`docs/abi_spec.md`** — the **ABI argument unit**: §4.2 states that
  `width` / `height` are DIP of the outer window rectangle. §4.1 gains
  one further fact a host genuinely needs: **`wasamo_init` declares the
  process's DPI awareness**, and what happens if the host already
  declared its own (DD-001's tolerant path). A host cannot reason about
  its own manifest without this.

**Vocabulary discipline.** No DD identifiers, option labels, or
decision-summary phrasing enter spec prose; provenance is a hyperlink to
this ADR set only. Revision-history tables are **appended to**, never
edited in place
([constraints §8](../requirements/constraints.md)).

**The external-reader bar.** The wording is at the phase-end standard
when an implementer could reproduce the unit behaviour from the specs
alone: what a length means, what rectangle a window size denotes, what
stays fixed across scales, and what the runtime declares on the host's
behalf.

## Note updates

- **[`docs/notes/layout-engine.md`](../../../../docs/notes/layout-engine.md)
  §3.1** — answered. The M1-era question ("should the engine be aware of
  physical pixels — it affects DirectWrite hinting precision") gets both
  halves of its answer: **no**, the engine stays in DIP, and the hinting
  precision it was actually asking about is bought at the rasterization
  surface (DD-002). Revised in place with a pointer to this ADR set.
  **The note stays `live`** — §3.2 (AccessKit / UIA sync) belongs to
  M4-Phase 11, and §§3.3–3.5 remain open.
- **[`docs/notes/verification-environments.md`](../../../../docs/notes/verification-environments.md)
  Observation 4** — revised at **Moment 2, not Moment 1**. Its premise
  ("the host is DPI-unaware, so DWM stretches logical 800×600 to
  physical 1000×750; a DPI-unaware capture tool gets the crop and the
  scale wrong") is **falsified by this phase's implementation**, and the
  corrected capture coordinates that later phases will rely on can only
  be derived against the running surface. Revising it at design time
  would put an untested claim into the document other phases read as
  procedure. This is framing risk R5 and
  [constraints §7](../requirements/constraints.md)'s second direction:
  the phase is not only constrained by the capture discipline, it
  changes it.

## Forward-compat exposure

1. **Host-visible scale or work-area query** — deferred with the trigger
   above; lands in the M4-Phase 7 ABI wave or with `WindowConfig` at
   M4-Phase 8.
2. **Client-size window semantics** — M4-Phase 8 / AC11, as a new named
   attribute rather than a reinterpretation of `width` / `height`.
3. **Physical-pixel-denominated author values** (if a case ever demands
   exact device-pixel control) — would arrive as an explicitly-united
   literal form, never as a change to the meaning of the existing bare
   number.
4. **Integer pixel snapping** — deferred; would change how a DIP value
   is projected, not what a DIP value means, so this contract survives
   it.
5. **`docs/abi_spec.md` freeze at M6** — this contract is written to be
   the frozen one. That is the reason the phase states it now rather
   than when a host first asks.

## Technical risk re-evaluation

- **Stating a unit that the implementation then diverges from.** The
  Moment 1 spec text is written from accepted design, before code
  exists; Moment 2's divergence-correction pass re-verifies each
  statement against what landed. The specific statements at risk are the
  outer-window-rectangle claim and the font-size unit — both are checked
  against running behaviour at phase close, not assumed.
- **The DIP claim for `width` / `height` is unverifiable at 100%.** It
  is discharged by the development machine at 125%: a window created at
  800×600 DIP must measure 1000×750 physical. That is a concrete,
  cheap assertion and it belongs in the phase's evidence.
- **Over-reading agreement ③.** The conclusion here is "no ABI addition
  is *needed*", supported by walking every M4 phase, not "an ABI
  addition is forbidden." If the trigger fires mid-implementation, the
  correct response is the stage-2 plan revision proposal, not a
  workaround — the plan is a hypothesis to be revised, not routed
  around.
- **Note-update timing.** Deferring `verification-environments.md` to
  Moment 2 leaves a window in which the note's stated premise is stale
  relative to accepted design. Accepted deliberately: no other phase
  captures evidence during this phase's implementation, and a note that
  is *provably* correct one commit late is better than one that is
  speculatively wrong from the moment the ADR is accepted.

## Revision history

- 2026-07-28: Initial draft (Status: Proposed).
- 2026-07-28: Accepted flip following owner approval of the phase slate; no
  change requested to the recommendations or their comparisons.
