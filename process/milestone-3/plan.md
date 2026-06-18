---
milestone: M3
status: in-progress
roadmap-anchor: ROADMAP.md#m3-dsl-surface
adrs:
  - process/milestone-3/phase-1/decisions/preamble.md  # Phase 1 (bool scalar)
  - process/milestone-3/phase-2/decisions/preamble.md  # Phase 2 (Box layout primitive)
  - process/milestone-3/phase-3/decisions/preamble.md  # Phase 3 (WrapPanel layout primitive)
  - process/milestone-3/phase-4/decisions/preamble.md  # Phase 4 (ScrollView minimal)
  - process/milestone-3/phase-5/decisions/preamble.md  # Phase 5 (Grid layout primitive)
  - process/milestone-3/phase-6/decisions/preamble.md  # Phase 6 (ZStack + conditional rendering)
created: 2026-05-16
agreed: 2026-05-16
in-progress: 2026-05-19
---

# M3 Plan — DSL surface milestone

## Frozen agreement

### Purpose

M2 closed the loop on `.ui → IR → runtime` with a reactive
foundation: Hello Counter is driven by the DSL in C, Rust, and Zig,
reactive propagation is type-agnostic (i32 + String), and the
execution-order and re-entrancy invariants of the reactive engine
are settled at the ADR level.

M3's purpose is to **grow the DSL surface to where it can express
real layouts**, not just one counter, and to **publish the resulting
surface as a stable public draft** so that external readers and
downstream tooling tracks (M5 VS Code LSP in particular) have a
normative document to build against. Concretely, M3 ships:

- The **Photo Gallery** target app
  ([docs/notes/m3/m3-target-app-predoc.md](requirements/spec.md),
  accepted 2026-05-16; visual contract in
  [docs/references/m3-gallery-wireframe.html](./requirements/gallery-wireframe.html))
  as the visible proof and per-phase acceptance basis.
- A layout primitive set (**Grid, WrapPanel, ZStack, ScrollView,
  Box**) sized to that target app, alongside the M2 linear
  primitives (HStack / VStack).
- Two **grammar surfaces** — conditional rendering and iteration —
  raising the binding system from "drives property values" to
  "drives subtree presence and cardinality".
- A third **scalar** (`bool`) and a **Button `selected` state**
  surface, so subtree presence and widget attribute state can both
  ride a scalar binding without forcing `TypedValue` adoption.
- A **first public draft of `docs/dsl_spec.md`**, written *per
  phase* alongside the surface it ships — not collected at the end
  of the milestone.

M3 is explicitly **not** a feature-breadth milestone: input / focus
model, IME, multi-window, AccessKit, Mica/Acrylic rendering
semantics, full theming, VS Code LSP acceptance, hot reload, and
C ABI freeze remain deferred to M4–M6 per
[docs/notes/m3/m3-start-framing.md](requirements/framing.md)
§"M3 に入れないもの" and confirmed in the target-app pre-doc's
Out-of-scope section.

### Phase numbering

Phase numbers in this plan are **local to M3** (M3-Phase 1, 2, …),
following the M2 convention. ADR identifiers use the scope
`M3-P<n>` (e.g. `DD-M3-P2-001`); see
[process/README.md](../README.md).

A pre-plan **target-app framing** step was discharged before this
plan opened, in
[docs/notes/m3/m3-target-app-predoc.md](requirements/spec.md);
it is not numbered as an M3 phase because it produced no
implementation, only the agreed surface contract that this plan
mirrors.

### Acceptance criteria

ROADMAP is the SSOT (see
[process/_roadmap.md §M3](../_roadmap.md#m3-dsl-surface)); mirrored here
with stable IDs for phase mapping:

- **A1.** `examples/gallery/gallery.ui` drives the Photo Gallery
  target app end-to-end on the M2 reactive foundation, exercising
  every M3 layout primitive (A2–A6), both grammar surfaces (A7–A8),
  the `bool` scalar (A9), and the Button `selected` state surface
  (A10) through the `.ui → IR → runtime` path.
- **A2.** Grid layout primitive (1 cell 1 child, star sizing +
  spanning; same-cell overlap is **not** provided — overlay is
  ZStack's responsibility).
- **A3.** WrapPanel layout primitive (linear main-axis placement
  plus cross-axis wrap on main-axis overflow).
- **A4.** ZStack layout primitive (sibling z-order by document
  order).
- **A5.** ScrollView primitive (minimal: inner unbounded measure +
  viewport clip + content offset binding; scrollbar widget, wheel
  handler, and drag are deferred to M4).
- **A6.** Box layout primitive (0+ child container; `aspect:
  <ratio>` attribute subsumes a standalone AspectRatio; minimal
  `fill: <color>` attribute for scrim use). Image-widget deferral
  is carried by Box + Text-child placeholders.
- **A7.** Conditional rendering grammar: binding drives the
  present / absent state of a subtree.
- **A8.** Iteration grammar: collection binding drives widget-tree
  generation.
- **A9.** `bool` admitted as the third scalar binding type
  alongside `i32` and `String`. The `TypedValue` generic value
  union remains deferred.
- **A10.** Button `selected` state surface admitted. The concrete
  construct (`selected: bool` attribute vs separate `ToggleButton`
  primitive vs theming binding) is settled in the phase that owns
  it.
- **A11.** Per-phase synchronization of `.ui`, `wasamo-ir`,
  `wasamoc`, `wasamo-runtime`, `docs/dsl_spec.md`, and an
  `examples/gallery/` E2E proof. No phase closes leaving one side
  ahead of the others.
- **A12.** DSL specification first public draft. Covers the M2
  surface plus A2–A10; the novel normative content is the
  measure-arrange specifications for WrapPanel and Grid and the
  grammar surface for A7–A8.

### Phase breakdown

The phases below are working hypotheses; each one's design
questions become a ADR at pre-doc time, per
[the decisions README](../README.md). Each phase ships
one (or a tightly coupled pair of) surface(s) on the gallery
target app, updates `docs/dsl_spec.md` for what it shipped, and
exercises that surface in `examples/gallery/`.

- **M3-Phase 1 — `bool` scalar binding.** Add `bool` to the
  scalar set in `wasamo-ir`, the evaluator
  (`EvalContext` / `HandlerExpr`), `wasamoc` lowering, and the
  runtime read path. Foundational: A7 (conditional rendering) and
  A10 (selected state) both need a `bool` binding to ride on.
  Ships the M2 String-binding evidence pattern extended to `bool`
  (live `WidgetNode` propagation of a `bool`-bound attribute on a
  trivial widget that already exists — no new layout primitive is
  required for the phase to close). Origin:
  [m3-target-app-predoc.md §Binding / value surface](requirements/spec.md#binding--value-surface).

- **M3-Phase 2 — Box layout primitive.** 0+ child container with
  `aspect: <ratio>` and minimal `fill: <color>` attributes. Pure
  primitive — no novel measure-arrange algorithm, just aspect
  constraint resolution and child layout. Establishes the
  placeholder pattern (Box + Text child) that carries the M3
  Image-widget deferral. Origin:
  [m3-target-app-predoc.md — AspectRatio attribute and Image-widget deferral closures](requirements/spec.md#保留点の決着).

- **M3-Phase 3 — WrapPanel layout primitive.** Two-stage
  measure-arrange: linear main-axis placement plus cross-axis
  wrap on main-axis overflow. First M3 phase to introduce
  **novel normative measure-arrange spec** in `docs/dsl_spec.md`,
  so the spec-drafting discipline gets exercised early.

- **M3-Phase 4 — ScrollView primitive (minimal).** Inner
  unbounded measure + viewport clip + content offset binding.
  Pairs structurally with WrapPanel: the gallery's thumbnail strip
  becomes a `ScrollView { WrapPanel { … } }`. Scrollbar widget,
  wheel handler, and drag are out of M3 scope.

- **M3-Phase 5 — Grid layout primitive.** 2D measure-arrange with
  star sizing, row/column spanning, 1 cell 1 child. Second
  novel-normative-spec phase. Star sizing's cross-axis dependency
  resolution is the central algorithmic content. Same-cell overlap
  is explicitly **not** in scope — overlay is ZStack's
  responsibility (Phase 6).

- **M3-Phase 6 — ZStack primitive + conditional rendering
  grammar.** Two surfaces shipped together because the gallery's
  lightbox uses them as a unit: ZStack provides the layered
  overlay structure, conditional rendering provides the
  `bool`-driven present / absent toggle. First grammar surface;
  binding now reaches into widget-tree shape, not just property
  values.

- **M3-Phase 7 — Iteration grammar.** Collection-driven widget
  tree generation. The gallery's thumbnail set is generated from a
  collection binding. The concrete shape of the per-item context
  (identifier naming such as `item` / `index`, whether it rides on
  the unified `HandlerExpr` enum per the M2-to-M3 handover
  ([m2-to-m3-handover.md](../milestone-2/handoff.md) §2) or
  requires a separate context type, and how the collection type
  is exposed) is settled in the ADR — the plan commits only
  to the surface identity, not the syntactic form. The same phase
  ADR also decides whether iteration pressures `TypedValue`
  adoption; the plan's working assumption is that scalar `bool`
  + existing `i32` / `String` are sufficient and `TypedValue`
  remains deferred. If pressure surfaces, the ADR opens it
  as an explicit DD.

- **M3-Phase 8 — Button `selected` state + Gallery E2E + DSL
  spec public draft.** Three workstreams that close M3 together:
  (i) settle the concrete construct for A10 (`selected: bool`
  attribute vs `ToggleButton` vs theming binding — per-phase
  ADR question) and ship it driving a tab-like section of the
  gallery; (ii) assemble the full `examples/gallery/gallery.ui`
  exercising every M3 surface end-to-end (A1); (iii) editorial
  pass on `docs/dsl_spec.md` to promote it to **first public
  draft** (A12). Per-phase spec updates from Phases 1–7 mean
  Phase 8's spec work is editorial, not writing the spec from
  scratch. The framing-level permission to reserve syntax for
  M4 material in the public draft
  (per the framing's ROADMAP-acceptance discussion in
  [m3-start-framing.md](requirements/framing.md),
  owner-agreed 2026-05-11) is
  **not** carried as an M3 acceptance criterion; the Phase 8
  phase-ADR is where it is either exercised as an explicit DD
  or recorded as declined. The pre-doc's silence on material
  reservation is the default state — Phase 8 must act
  affirmatively to exercise the permission.

### Phase dependencies

```
M3-Phase 1 (bool scalar)
        │
        ├──► M3-Phase 6 (ZStack + conditional rendering)
        │           │
        │           └──► M3-Phase 8 (selected state + E2E + spec draft)
        │                              ▲
M3-Phase 2 (Box) ──► M3-Phase 3 (WrapPanel) ──► M3-Phase 4 (ScrollView) ──┤
                                                                          │
                            M3-Phase 5 (Grid) ────────────────────────────┤
                                                                          │
                            M3-Phase 7 (iteration grammar) ───────────────┘
```

Phase 1 (`bool`) is a hard prerequisite for Phase 6 (conditional
rendering needs `bool`) and Phase 8 (selected state needs `bool`).

Phases 2 → 3 → 4 form the thumbnail-strip chain: Box gives the
placeholder shape, WrapPanel gives the wrap, ScrollView puts the
wrapped row inside a viewport. They are listed in order because
each phase's E2E proof reuses the prior phase's result.

Phase 5 (Grid) is independent of the WrapPanel chain and can
proceed in parallel once Phase 1 lands, but it is sequenced after
the WrapPanel chain in this plan because Grid is the
heaviest single primitive (star sizing) and benefits from the
spec-drafting discipline rehearsed on WrapPanel.

Phase 6 (ZStack + conditional rendering) depends on Phase 1
(`bool`). It does not depend on the WrapPanel chain or Grid —
the lightbox structure can be proven against a minimal frame.

Phase 7 (iteration grammar) is independent of layout-primitive
phases at the IR / evaluator level, but its E2E proof
(thumbnails generated from a collection) reuses the WrapPanel +
ScrollView combination from Phase 4. Sequencing after Phase 4
keeps the E2E proof a strict superset rather than a re-do.

Phase 8 depends on every preceding phase (it assembles the full
gallery and promotes the cumulative spec).

### Acceptance ↔ phase mapping

| Acceptance | Phase(s) |
|---|---|
| A1 (gallery.ui drives full M3 surface E2E) | M3-Phase 8 (assembly); incrementally exercised by every prior phase's sub-screen proof |
| A2 (Grid) | M3-Phase 5 |
| A3 (WrapPanel) | M3-Phase 3 |
| A4 (ZStack) | M3-Phase 6 |
| A5 (ScrollView minimal) | M3-Phase 4 |
| A6 (Box + aspect) | M3-Phase 2 |
| A7 (conditional rendering grammar) | M3-Phase 6 |
| A8 (iteration grammar) | M3-Phase 7 |
| A9 (`bool` scalar) | M3-Phase 1 |
| A10 (Button selected state) | M3-Phase 8 |
| A11 (per-phase spec / impl / E2E sync) | Every phase (operational rule, not a single-phase deliverable) |
| A12 (DSL spec first public draft) | M3-Phase 8 (promotion); written incrementally in M3-Phase 1–7 |

### Out of scope (deferred to later milestones)

Surfaces explicitly excluded by the M3 target-app pre-doc and the
M3 start framing. Allocation to post-M3 milestones is recorded in
[process/_roadmap.md](../_roadmap.md) where the roadmap commits to a specific
milestone; where ROADMAP is silent, the pre-doc's "M4 以降" / "later"
wording is preserved here as "M4 or later" rather than tightened:

- Image widget surface, asset pipeline, icon font, image decoder
  → M4 or later (pre-doc: "M4 以降"; ROADMAP M4 does not name it
  explicitly)
- Button content other than text (e.g. Image inside Button)
  → M4 or later (tied to the Image-widget deferral)
- Scrollbar widget, wheel handler, drag-to-scroll → M4 or later
  (pre-doc: "M4 へ defer"; ROADMAP M4 does not name scrollbar
  explicitly — the M4 Input AC may absorb wheel/drag handling but
  the scrollbar *widget* is not yet committed)
- Splitter drag and any drag-driven layout resize → M4 or later
- lightbox swipe / pinch / keyboard-shortcut gestures → M4
  (covered by M4 Input handling AC)
- hit-testing, focus capture, modal focus trap → M4 (covered by
  M4 focus model AC)
- Input handling (kbd / mouse / touch), focus model → M4
- TextField, IME (TSF, Japanese / CJK) → M4
- Multi-window → M4
- AccessKit / UIA → M4
- Mica / Acrylic backdrop, system accent, full theming → M4 / M5
- VS Code LSP acceptance → M5 (parallel-track start permitted
  once M3 spec public draft is agreed)
- Hot reload → post-1.0
- C ABI freeze → M6
- `TypedValue` generic value union → reassessed mid-M3 if Phase 7
  surfaces type-system pressure; otherwise deferred. M3 ships
  only the third scalar (`bool`), not a value union.
- Cycle detection / dependency-tie observable contract / fan-out ×
  `MUTATION_CAP` interaction in the reactive drain — residuals
  from DD-M2-P6-010, listed in
  [m2-to-m3-handover.md §3](../milestone-2/handoff.md). Touched
  by an M3 phase only if its multi-binding work surfaces a concrete
  failure; otherwise carried forward.

### Verification strategy

Verification means are partitioned by what the code touches, per
[CLAUDE.md §Testing rules](../../CLAUDE.md). Each M3 phase chooses
from this menu and states which it uses in its ADR / progress
file:

- **Pure-logic unit tests** (`cargo test`, no Win32/WinRT). Used
  for: IR types and serialization; `HandlerExpr` evaluator
  extensions; `wasamoc` lowering passes; pure measure-arrange
  algorithms (WrapPanel main/cross-axis solver, Grid star sizing
  resolver, ScrollView clip arithmetic, Box aspect resolution);
  conditional-rendering subtree-presence reducer; iteration
  expander.
- **Windows-only headless integration tests** (mock-free, against
  the real OS runtime surface). Used for: end-to-end property
  propagation through live `WidgetNode` state where logic is
  entangled with a Compositor-bound type; CI-gated; **fail rather
  than silently skip** on GitHub Actions if the required runtime
  capability is unavailable.
- **Gallery E2E proof** in `examples/gallery/`. Each phase ships
  a sub-screen of the gallery wired to `.ui → IR → runtime` and
  invoked from the example host. M3-Phase 8 assembles the full
  gallery from these sub-screens. Visual fidelity is judged
  against
  [docs/references/m3-gallery-wireframe.html](./requirements/gallery-wireframe.html).
- **Spec drafting** is verification of a different kind:
  `docs/dsl_spec.md` is updated within the same phase, and the
  phase-end check (below) asks whether the spec text would let an
  external implementor reproduce the surface. This is not a test
  in the executable sense, but it is a phase-completion gate per
  A11.

Pure-logic vs Windows-headless distinction matters for CI cost
and reliability: prefer pure-logic where the algorithm permits;
fall back to a Windows-headless integration test only when the
property under test is entangled with a `Compositor`-bound type
(per CLAUDE.md, the "test-module-only mirror struct" escape is
permitted sparingly).

### Phase-end criteria

A phase closes when **all** of the following hold:

1. **ADR Accepted.** The ADR (`process/m3-phase-N-*.md`)
   has all its design decisions in `Accepted` status, with no
   `Proposed` DDs remaining open for the phase.
2. **Implementation landed.** The surface the phase owns is
   implemented across `.ui`, `wasamo-ir`, `wasamoc`,
   `wasamo-runtime`, and any host-side glue, with `cargo build
   --release --workspace` and `cargo test --workspace` green
   locally and on GitHub Actions CI.
3. **Verification evidence recorded.** Pure-logic unit tests
   and / or Windows-headless integration tests covering the
   phase's surface pass on CI. The phase progress file records
   which means was used and links the CI run.
4. **Spec synchronized.** `docs/dsl_spec.md` reflects the
   surface this phase ships. The text is sufficient for an
   external implementor to reproduce the surface (the same bar
   the M3 public draft will be held to in Phase 8).
5. **Gallery sub-screen runs.** The relevant slice of
   `examples/gallery/` exercises the new surface through
   `.ui → IR → runtime` end-to-end, not via host-imperative
   construction. The sub-screen is invoked from at least one
   example host (C / Rust / Zig — Phase 8 broadens this to all
   three for the full gallery).
   *Foundational-phase exception:* Phase 1 closes before
   `examples/gallery/` exists, so per its ADR
   ([m3-phase-1-bool-scalar.md §Verification closure item 4](phase-1/decisions/preamble.md#phase-1-verification-closure-what-counts-as-a9-evidence)),
   a dedicated minimal example under `examples/` (Phase 1 chose
   `examples/bool-demo/` + `examples/bool-demo-rust/`) is
   acceptable as substitute. The exception is scoped to Phase 1
   only; Phase 2 onward follows this item as written, with
   `examples/gallery/` being grown sub-screen by sub-screen.
6. **Out-of-phase residuals filed.** Anything discovered during
   the phase that is real but not in the phase's scope is recorded
   in a live note under `docs/notes/m3/` and pointed to from the
   ADR's residual / handover section, not silently carried.
7. **Phase-end retrospective recorded.** A short phase-end
   retrospective entry lands at
   `docs/notes/m3-phase-N/phase-end-retrospective.md` (the
   per-phase durable record, following Phase 1 / Phase 2
   practice) covering what the phase shipped, what slipped (if
   anything), and the merge / push gate per the *phase-end merge
   and push gating* discipline recorded in
   [docs/notes/retrospectives.md](../procedures/retrospectives.md)
   (the procedure document; durable per-phase entries are filed
   under the phase's own `docs/notes/m3-phase-N/` directory, not
   appended to the procedure document itself).

### Milestone-end criteria

M3 is complete when **all** of the following hold:

1. **Every acceptance criterion A1–A12 is discharged**, with the
   discharge recorded in the corresponding ADR and the plan's
   Progress section's row marked completed.
2. **Phase 8 outputs all three deliverables**: A10 (Button
   `selected` state) shipped; A1 (full `gallery.ui` running on
   all three example hosts) demonstrated; A12 (DSL spec first
   public draft) promoted in `docs/dsl_spec.md` with a
   `status: public-draft` (or equivalent) frontmatter marker and
   a CHANGELOG entry.
3. **CHANGELOG entry** for M3 lands, linking each ADR and
   the public-draft anchor in `docs/dsl_spec.md`.
4. **Per-phase spec sync (A11) is auditable**: every ADR
   from M3-Phase 1 onwards has a "spec section updated" reference
   pointing at the section of `docs/dsl_spec.md` it modified.
   This is what the M3 public draft *means* in practice — that
   the spec was written alongside the code rather than after it.
5. **External-reader smoke check on the public draft.** Phase 8
   asks: could a reader who has only `docs/dsl_spec.md` reproduce
   the M3 surface against a hypothetical host that already
   provides the C ABI? If the answer is "not yet" for any M3
   surface, Phase 8 has spec editorial work remaining.
6. **No silently deferred M3 surface.** Anything that appeared
   in the M3 target-app pre-doc's "必要 surface" section is
   either shipped or explicitly recorded as a deviation in this
   plan's Revision log.
7. **Clean rebuild green on CI** for the merge commit on `main`
   (or whichever branch carries the milestone close), matching
   the M2 close discipline.

### Risks

- **Spec-drafting drift.** A11 / A12 require per-phase spec
  updates, but spec text is easier to defer than code. If a phase
  closes with code green and spec text "to be written next phase",
  M3 silently turns into "implementation now, public draft at the
  end" — exactly the failure mode the framing rejected
  ([m3-start-framing.md §F6](requirements/framing.md#f6--dsl-spec-draft-は各-phase-の副産物ではなく-acceptance-の一部)).
  Mitigation: the phase-end criterion 4 ("spec synchronized") is a
  hard gate, not a soft one. A phase whose spec text is "TODO"
  does not close.

- **WrapPanel / Grid measure-arrange spec complexity.** These two
  primitives carry novel normative content. If either spec
  section bogs down in pre-doc, the phase stalls. Mitigation:
  spec drafting starts in pre-doc, not at phase close — the phase
  ADR is allowed to land a "spec draft v0" alongside its
  Accepted DDs, with Phase 8 doing the editorial pass.

- **Iteration grammar surfacing `TypedValue` pressure.** Phase 7
  is the most likely point at which the M2-deferred `TypedValue`
  generic value union becomes hard to avoid (per-item context,
  collection element types). Mitigation: ADR opens this as
  an explicit DD; if the answer is "we need `TypedValue`", M3
  acceptance gains a revision under the README "Acceptance
  criteria revision" exception rather than smuggling the change in.

- **Reactive-drain residuals (DD-M2-P6-010 follow-ons).** Phases
  6 and 7 are the most likely to touch the dirty-Effect drain
  residuals (cycle detection, dependency ties, fan-out ×
  `MUTATION_CAP`). Mitigation: the phase pre-doc is required to
  reference [m2-to-m3-handover.md §3](../milestone-2/handoff.md)
  and decide whether to fix or carry forward. Silent carry-forward
  is not acceptable.

- **Gallery example growing un-CI-able.** The M3 gallery is
  bigger than Hello Counter; the Windows-headless integration
  tests that exercise it must still terminate predictably on CI.
  Mitigation: per-phase headless tests target the phase's
  sub-screen, not the full gallery. The full-gallery E2E in
  Phase 8 is a single fixture, not a matrix.

### Revision log

- **2026-05-21 — Phase-end criterion 7 wording aligned to
  Phase 1 / Phase 2 practice.** Routed under
  [plans/README.md §Factual correction](../README.md).
  The original wording named `docs/notes/retrospectives.md` as
  the landing site for the per-phase retrospective entry. In
  practice, both Phase 1
  (`docs/notes/m3-phase-1/phase-end-retrospective.md`) and
  Phase 2 (`docs/notes/m3-phase-2/phase-end-retrospective.md`)
  filed their durable phase-end retrospectives under the
  phase's own notes directory, treating
  `docs/notes/retrospectives.md` as the procedure document
  only. Criterion 7 was reworded in place to reflect that
  established practice. The gate itself (existence of a
  phase-end retrospective entry, covering ship / slip / merge
  + push) is unchanged; no acceptance criterion is affected;
  no phase scope is changed.

## Progress

The Progress section is a compact milestone index. Detailed live
task tracking belongs in phase progress files under
`docs/plans/progress/`; completed phase logs are distilled into
ADRs, CHANGELOG, notes, and git history, then deleted by default.

| Phase | Status | Progress file | ADR | Notes |
|---|---|---|---|---|
| M3-Phase 1 — `bool` scalar binding | complete | [plan.md](phase-1/implementation/plan.md) | [preamble.md](phase-1/decisions/preamble.md) | ADR Accepted 2026-05-19; execution opened 2026-05-19; A9 discharged 2026-05-19 |
| M3-Phase 2 — Box layout primitive | complete | [plan.md](phase-2/implementation/plan.md) | [preamble.md](phase-2/decisions/preamble.md) | ADR Accepted 2026-05-20; execution opened 2026-05-20; A6 discharged 2026-05-20 |
| M3-Phase 3 — WrapPanel layout primitive | complete | [plan.md](phase-3/implementation/plan.md) | [preamble.md](phase-3/decisions/preamble.md) | ADR Accepted 2026-05-21; execution opened 2026-05-21; WrapPanel constituent of A3 discharged 2026-05-22; first novel-normative-spec phase |
| M3-Phase 4 — ScrollView (minimal) | complete | [plan.md](phase-4/implementation/plan.md) | [preamble.md](phase-4/decisions/preamble.md) | ADR Accepted 2026-05-25; execution opened 2026-05-25; A5 discharged 2026-05-25; A11 gallery owner-acceptance 2026-05-25 |
| M3-Phase 5 — Grid layout primitive | complete | [plan.md](phase-5/implementation/plan.md) | [preamble.md](phase-5/decisions/preamble.md) | ADR Accepted 2026-05-28; execution opened 2026-05-29; A2 (Grid) discharged + A11 gallery owner-acceptance 2026-05-30; Moment-2 docs synced + phase-end CI green + merged to main 2026-05-30; second novel-normative-spec phase; star sizing |
| M3-Phase 6 — ZStack + conditional rendering | complete | [plan.md](phase-6/implementation/plan.md) | [preamble.md](phase-6/decisions/preamble.md) | ADR Accepted 2026-06-02; A4 + A7 discharged 2026-06-09; first grammar surface (binding drives subtree present/absent); `bool` prereq landed in Phase 1; **M3-Phase 4 R1 (Window-title wiring) closed** via static `title:` host-wiring (DD-M3-P6-006); Moment-2 docs synced + phase-end CI green run 27149254110 |
| M3-Phase 7 — Iteration grammar | implementation complete; phase-end pending | [plan.md](phase-7/implementation/plan.md) | [preamble.md](phase-7/decisions/preamble.md) | ADR Accepted 2026-06-13; execution opened 2026-06-13; A8 discharged by collection-driven `for` generation with runtime append / remove / clear / reset positive controls; A11/A12 Moment 2 docs synced 2026-06-18; `TypedValue` pressure judged — **not adopted**, with per-item richness triggers recorded for phase-end handoff; phase-branch CI run-id / final handoff / phase-end retro remain phase-end-owned |
| M3-Phase 8 — `selected` state + Gallery E2E + DSL spec public draft | not started | — | — | A1, A10, A12 discharge |

### Owner-facing resume note

M3 plan is `in-progress` as of 2026-05-19. M3-Phase 1 (`bool`
scalar) closed 2026-05-19. M3-Phase 2 (Box layout primitive) closed
2026-05-20 and discharged A6. M3-Phase 3 (WrapPanel layout primitive)
closed 2026-05-22 and discharged the WrapPanel constituent of A3
(gallery overflow / wrapping evidence). M3-Phase 4 (ScrollView
minimal) closed 2026-05-25 and discharged A5 (ScrollView minimal) and
the A11 gallery proof's owner-acceptance half (owner-manual GUI smoke
on the rebuilt gallery host after the T6 window-root Fill/Fill fix).
The Frozen agreement section remains read-only under the
`in-progress` lifecycle (acceptance-criteria revision exception
aside).
