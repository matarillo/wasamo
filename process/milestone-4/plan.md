---
milestone: M4
status: agreed
roadmap-anchor: process/_roadmap.md#m4-interaction-stack
adrs: []  # populated as each phase's ADR set opens
created: 2026-07-28
agreed: 2026-07-28
---

# M4 Plan — Interaction stack milestone

## Frozen agreement

### Purpose

M3 closed the DSL surface: layout primitives, conditional rendering,
iteration, the `bool` scalar, parent-interpreted placement, and a
public draft of `docs/dsl_spec.md`. The Gallery renders, but nothing
in it responds — there is no focus, no hit-testing, no text, no
second window.

M4's purpose is to **settle the focus model once**, and to ship
together everything that shares it. Per
[framing.md](requirements/framing.md) §M4 主眼（thesis）の読み (F1),
input handling, single-line text editing, IME, multi-window, and
accessibility are not independent features; they are five consumers
of one model. The intake criterion for this milestone is therefore
not schedule but *"does this threaten the quality of settling the
focus model once."*

Concretely, M4 ships:

- An **event routing model and focus model**, including generic click
  handling on non-`Button` widgets, per-item handlers inside
  repetition, and a **structure-independent modal focus scope** —
  attachable to any subtree, so a root `ZStack` branch and a
  top-layer overlay are both consumers of one concept.
- A **single-line editable text widget** with IME via TSF, proving
  that Japanese input rides the same focus model rather than
  importing borrowed focus behaviour.
- **Multi-window**, **top-layer overlays**, and **window config
  properties** — the three surfaces where the focus model has to hold
  across a structural boundary.
- The **host state boundary** (host-supplied initial state, host
  writes to displayed state, write-back from an edited widget) and
  **predicate expressions**, both promoted from the
  [candidate pool](../candidate-pool.md) at M4 planning.
- **Per-monitor DPI awareness**, **Mica / Acrylic backdrop and accent
  follow-through**, and **two showcase applications** — the matured
  photo gallery (A) as the outward-facing banner and the quick
  capture inbox (B4) as the moving proof
  ([spec.md](requirements/spec.md) §採択).

M4 is explicitly **not** a widget-catalogue milestone. The official
widget set (CheckBox / ComboBox / Menu / Dialog), full theming,
anchored popovers, `TypedValue` / structured item data, and developer
tooling remain outside it, per
[framing.md](requirements/framing.md) §M4 に入れないもの and
[spec.md](requirements/spec.md) §範囲外. Menus and dialogs in B4 are
**author compositions** of existing widgets, not new products.

### Phase numbering

Phase numbers are **local to M4** (M4-Phase 1, 2, …), following the
M2 / M3 convention. ADR identifiers use the scope `M4-P<n>` (e.g.
`DD-M4-P2-001`); see [process/README.md](../README.md).

Three pre-plan steps were discharged before this plan opened and are
not numbered as M4 phases, because they produced no implementation:
the milestone framing ([requirements/framing.md](requirements/framing.md),
accepted 2026-07-07..27), the target-app adoption
([requirements/spec.md](requirements/spec.md), accepted 2026-07-28),
and the acceptance-criteria revision that followed it
([_roadmap.md §M4](../_roadmap.md#m4-interaction-stack), revised
2026-07-28).

### Acceptance criteria

[process/_roadmap.md §M4](../_roadmap.md#m4-interaction-stack) is the
SSOT; mirrored here with plan-local IDs for phase mapping. The IDs are
spelled `AC<n>` rather than M3's `A<n>` because M4's vocabulary already
uses **A** for the photo-gallery target app.

- **AC1.** Input handling: keyboard, mouse, touch; focus model and
  event routing. Includes click handling on non-`Button` widgets (with
  per-item handlers inside repetition) and a **structure-independent
  modal focus scope** — attachable to any subtree, so a root `ZStack`
  branch and a top-layer overlay are both consumers of one concept.
- **AC2.** Multi-window support (per-window state, cross-window focus).
- **AC3.** TextField widget — minimum editable text widget,
  **single-line**; required by IME verification. Multi-line editing is
  outside this criterion.
- **AC4.** IME via TSF (Japanese / CJK input).
- **AC5.** AccessKit / UIA integration.
- **AC6.** Mica / Acrylic root-window backdrop; system accent colour
  follow-through (initial — the full theming surface is M5).
- **AC7.** Per-monitor DPI awareness: declare process / window DPI
  awareness, render crisply on high-DPI displays without DWM bitmap
  scaling, and handle DPI changes across monitors
  ([DD-V-022](../cross-milestone/decisions/dpi-awareness-m4-deferral.md)).
- **AC8.** Host state boundary: host-supplied initial state, host
  writes to displayed state, and write-back from an edited widget
  (in-out binding). ABI-bearing.
- **AC9.** Expression predicates: reading a collection from outside the
  repetition (count, emptiness, index access), per-item conditional
  rendering, and equality-based selection. String concatenation and
  general arithmetic stay outside M4.
- **AC10.** Top-layer overlays: the top-layer structure itself plus the
  focus rule set that binds to it — click-away close, Esc, focus
  containment, focus restoration on close, and screen-reader order.
  Widget-anchored placement is **not** included.
- **AC11.** Window config properties: dynamic window title, initial
  window size, `WindowConfig`.
- **AC12.** First showcase applications — **two**: the matured photo
  gallery (A, the outward-facing banner) and the quick capture inbox
  (B4, the moving proof). Per-app scope and the feature-to-app split
  are in [requirements/spec.md](requirements/spec.md).
- **AC13.** Author-controllable sizing (Problem B) design spike, with
  the disposition recorded at M4 planning per the
  [author-controllable sizing VDR](../cross-milestone/decisions/author-controllable-sizing-surface.md).
  The disposition is recorded below in §Cross-phase dispositions.

### Phase breakdown

The phases below are working hypotheses; each one's design questions
become an ADR at pre-doc time. Each phase ships one (or a tightly
coupled pair of) surface(s) against A or B4 per the
[spec.md](requirements/spec.md) §2 本の役割分担 split, updates
`docs/dsl_spec.md` (and `docs/abi_spec.md` where ABI-bearing) for
what it shipped, and exercises that surface in `examples/`.

[framing.md](requirements/framing.md) §初期フェーズ分割の仮説 supplies
the dependency reading; phase count, boundaries and order are this
plan's decision. The one place this breakdown departs from the framing
hypothesis is **DPI**, which the framing grouped with the late showcase
cluster and this plan moves to the front — see M4-Phase 1's rationale
and §Phase dependencies.

- **M4-Phase 1 — Per-monitor DPI awareness and the coordinate-space
  boundary.** Declare process / window DPI awareness, localize the
  DIP ↔ physical-pixel conversion at the layout / render / input
  boundary, and handle `WM_DPICHANGED` across monitors. Answers
  [layout-engine.md §3.1](../../docs/notes/layout-engine.md) ("should
  the engine be aware of physical pixels"), whose stated reconsider
  trigger (Grid / ScrollView) fired inside M3 and was carried forward.
  No new DSL surface: the consumer is the existing Gallery, and the
  M3 residual *"runtime DPI-awareness and DPI-localized layout
  evidence"* ([M3 handoff](../milestone-3/handoff.md) §M4 Residual
  Cluster) is discharged here.

  **Why first rather than last.** Every later phase in this milestone
  consumes a coordinate conversion: hit-testing maps pointer messages
  to layout coordinates (Phase 2), the caret and composition rectangles
  map layout coordinates to screen coordinates for the IME candidate
  window (Phases 5–6), the top layer places content at window level
  (Phase 9), and each window can sit on a monitor with a different
  scale factor (Phase 8). Building those against a 1:1 DWM-scaled world
  and introducing the scale factor afterwards would re-open every one
  of them and invalidate their GUI evidence. The phase is small and
  self-contained precisely because it is placed before the work that
  would otherwise entangle it.

- **M4-Phase 2 — Event routing, focus model, and generic click
  handling.** The milestone's centre of gravity. Chooses the routing
  model (capture / target / bubble versus target-only plus high-level
  signals — intake case 6), settles focus location and traversal, and
  defines the **structure-independent modal focus scope**. Ships
  keyboard and mouse input into that model, generic click handling on
  non-`Button` widgets, and per-item handlers (`item` / `index`
  reference in handler position — the M3-deferred surface whose reopen
  condition fires here).

  The phase ADR carries these as **mandatory topics**, per
  [framing.md](requirements/framing.md) §粒度の定義（フォーカスモデル）:
  the routing model itself; the relation between the existing
  `Button.clicked` signal and generic click handling, including
  hit-test eligibility (intake obligation 4); `ZStack` sibling
  occlusion (a full-bleed scrim `Box` blocks clicks to lower
  siblings); Esc consumption by a modal scope; and the rule that
  **screen-reader modality attaches to the focus scope, not to the
  layer** — fixed here, implemented in Phase 11.

  **Pre-ADR spike (required before the ADR is Accepted).** The focus
  semantics that M5's official widget set will consume — (a) arrow-key
  movement within a group where Tab treats the whole group as one stop,
  and (b) separation of focus from the active item in an open list —
  must be demonstrated on something that runs, not only on paper: the
  traversal core (tree + scope annotations → next focus target) is
  extracted as Win32-independent pure logic under unit test, and the
  event path is exercised through a **mechanism fixture** composed from
  existing material. The fixture is not published as an official widget
  (not in `docs/dsl_spec.md`, not in the widget list; its spelling is
  fixture-local and non-normative). No provisional RadioButton /
  ComboBox is built — group spelling is M5's decision and M4 does not
  create a fait accompli.

  Consumer (A): thumbnail click opens the lightbox, Tab traversal, Esc
  closes, ← → step between photos, and modal containment inside the
  lightbox **while it stays a root `ZStack` branch** (intake obligation
  3; [spec.md](requirements/spec.md) §アプリ仕様 A).

- **M4-Phase 3 — Predicate expressions.** Collection reads from
  outside the repetition (count, emptiness, index access), per-item
  conditional rendering, and equality-based selection. Novel normative
  DSL content, so the spec-drafting load is real. Absorbs the M3
  deferred DSL surfaces recorded in
  [framing.md](requirements/framing.md) §粒度の定義（M3 先送り事項の吸収）:
  the collection-read axis, the per-item conditional axis, and the
  **equality-selection axis only** of the M3 selected-state deferral
  ([M3 handoff](../milestone-3/handoff.md) §Selected-State Deferred
  Axes — the group-surface, widget-owned-state and generic-toggle axes
  stay with M5, the two-way-binding axis with Phase 7).

  Per-item conditional rendering is a cross-layer deliverable: Phase 3 owns the
  loop context used by condition evaluation and subtree re-materialisation, and
  integrates creation and disposal through the existing effect, handler,
  focus / hover and layout lifecycles. This responsibility does not create a
  separate structural writer or change the positional iteration baseline.

  Consumer (A): the status-bar photo count, the lightbox caption
  (index read), and exclusive selection of the current thumbnail.
  String concatenation stays out — the count is authored as a static
  `Text` beside a bound one.

- **M4-Phase 4 — Gallery completion: scrolling, scrollbar, and real
  images.** Wheel and drag scrolling with a real scrollbar (composed
  from existing widgets, not a new official widget), the `Image`
  widget, direct-value `fill` outside `Box`, and real image content
  replacing the M3 `Box` + `Text` placeholders. Discharges the rest of
  the [M3 handoff](../milestone-3/handoff.md) §M4 Residual Cluster
  apart from the host-supplied collection (Phase 7).

  `Image` and direct-value `fill` are the two **stretch** intakes
  (AC-exempt) from [framing.md](requirements/framing.md)
  §二層取り込み方針; withdrawing them costs a one-line disposition back
  to the candidate pool.

  **This phase also specifies what a `Row` / `HStack` does when its
  children do not fit** (owner-settled 2026-08-08; Revision 2). Today it
  overlaps — the toolbar's two groups overflow their `Grid` columns
  toward each other at a narrow client — and no decision stands behind
  that: wrapping, clipping, shrinking and scrolling were all available
  and none was chosen. **The overlap is not only visual.** A non-clipping
  container is itself a hit-test candidate across its whole arranged
  rectangle ([dsl_spec.md §4.19](../../docs/dsl_spec.md)), so the
  overflowing group takes the clicks aimed at the widgets it covers:
  M4-Phase 2 T10 measured every gallery tab `ToggleButton`'s own centre
  resolving to a scroll `Button` instead, with `checked` unchanged after
  a real click. Routing behaves as specified; what is undecided is the
  layout rule. **"Overlapping is acceptable" remains an available
  answer** — what this phase owes is not a particular fix but a rule
  **written into [dsl_spec.md](../../docs/dsl_spec.md)**, where every
  other layout primitive's behaviour already is (§4.9 `Box`, §4.10
  `WrapPanel`, §4.11 `ScrollView`, §4.12 `Grid`, §4.13 `ZStack`). An
  internal decision does not discharge it: today's overlap *is* a
  decision of a kind — the implementation makes it every time — and the
  gap is that no document states it.
  `gallery_slice_integration.rs`'s `g7_the_overflowing_toolbar_swallows_clicks_aimed_at_the_tab_buttons`
  is the tripwire and says in its own failure message that a red there
  means the overflow was changed rather than broken.

- **M4-Phase 5 — Single-line text editing.** The editable text widget:
  caret (click positioning, ← → / Home / End, rendering, follow-scroll
  on overflow), the **internal selection model** plus its user-facing
  operations, clipboard (`Ctrl+C` / `X` / `V`, `CF_UNICODETEXT`), and
  integration as a single focus stop. Undo, password masking and rich
  text are out ([framing.md](requirements/framing.md)
  §粒度の定義（文字編集）; the depth material is in
  [single-line-text-editing.md](requirements/single-line-text-editing.md)).

  Binding is one-way plus a handler in this phase, matching the M3
  `ToggleButton.checked` precedent; in-out binding is Phase 7. The ADR
  must fix the **text-store-facing internal model** that TSF will
  require — text content, get/set selection range, range replacement,
  and the caret / composition rectangle in screen coordinates — so
  Phase 6 is wiring rather than redesign.

  **The handler half of that precedent does not exist for strings yet,
  and this phase is where it lands** (measured at M4-Phase 2 T9; revision
  1 below). `ToggleButton.checked` works because a handler can write a
  `bool` state — the write M3-Phase 1 added for exactly that purpose. A
  handler cannot write a scalar `string` state at all, for any
  right-hand side: `wasamoc check` accepts `s = "abc"`, the compiler
  emits it, and the evaluator rejects it at invocation, because
  `dsl_spec.md` §8.9 marks the string forms binding-only and the runtime
  has no string write. `string` is the only declarable state type a
  handler cannot write. Two things follow for this phase: the capability
  itself, and the evaluator change it needs — assignment currently picks
  its typed path from the **right-hand side's shape**, which cannot type
  an `item` read (whose type comes from the collection, not the
  expression), so a `for`-body handler writing a string binder needs
  dispatch on the left-hand side's declared type. Whether handler-body
  assignments gain type checking at the same time is a separable, wider
  question. `dsl_spec.md` §4.6 and §8.9 both move.

  Consumer (B4): the list window's first runnable form — a one-line
  entry field, an Add button, and the item rows.

- **M4-Phase 6 — IME via TSF.** `ITextStoreACP`-class integration over
  the Phase 5 text model: in-place display of the composition string,
  candidate-window placement (`GetTextExt`-class screen rectangles,
  crossing the Phase 1 DPI bridge), and range replacement. Also settles
  the precedence between text input and keyboard shortcuts that the
  earlier phases left open (intake case 5).

  Consumer (B4): Japanese entry into the item field.

- **M4-Phase 7 — Host state boundary and in-out binding.** The three
  capabilities of AC8 in one design: host-supplied initial state, host
  writes to displayed state, and write-back from an edited widget.
  ABI-bearing, so `docs/abi_spec.md` moves with it and the host bindings
  move with the ABI. Fires the M3-deferred **two-way binding** axis
  ([M3 handoff](../milestone-3/handoff.md) §Selected-State Deferred
  Axes) and M-expr5 of
  [expression-language-roadmap.md](../../docs/notes/expression-language-roadmap.md);
  the open questions of
  [host-state-boundary.md](../../docs/notes/host-state-boundary.md)
  (type model, write conflicts, generated bindings) are its ADR's topic
  list. Reopen condition 3 of
  [architectural-family.md](../../docs/notes/architectural-family.md)
  (binding shapes that do not fit `BindingTarget`) is monitored here.

  Consumers: A supplies and replaces the photo collection from the
  host; B4 writes item additions and renames back.

- **M4-Phase 8 — Multi-window and window config properties.** Per-window
  state and cross-window focus, plus the window attribute family —
  dynamic title, initial size, `WindowConfig` — which is the grammar
  entry point that
  [dsl-grammar.md](../../docs/notes/dsl-grammar.md) Q2 asks for. The
  ADR decides where the dynamic title's string is composed (binding
  versus host composition), since M4's expression language stops at
  predicates and cannot concatenate
  ([framing.md](requirements/framing.md) §採択後・各フェーズへ送る決定事項 4).

  Consumer (B4): the list index window plus one window per list, each
  titled with its list name and item count. B4's second upper bound —
  *no item count on the index rows* — is a spec constraint this phase
  must not quietly break ([spec.md](requirements/spec.md)
  §B4 の 2 つの上限).

- **M4-Phase 9 — Top-layer overlays and the modal dialog combination.**
  The top-layer structure (an element declared in place is realized at
  window level, escaping clip and stacking boundaries) plus the full
  focus rule set bound to it: click-away close, Esc, focus containment,
  focus restoration on close, screen-reader order. Placement is coarse
  and manual; widget-anchored placement is **not** in M4, which is what
  keeps [dsl-grammar.md](../../docs/notes/dsl-grammar.md) Q1 (widget
  name references) closed for this milestone. Design space:
  [top-layer-overlays.md](../../docs/notes/top-layer-overlays.md).

  Consumers (B4): the item row's "…" menu, opened from inside the list
  (so the top layer is structurally required, not decorative); and the
  rename dialog opened from that menu, which proves **containment ×
  top layer × text entry × IME in one place**. The dialog is an author
  composition of `Box` / `Text` / `Button` / the text field — no
  official `Dialog` widget is created.

  This is the phase where the modal focus scope defined in Phase 2
  meets its **second structure**. If the concept needs to change shape
  to fit, that is a Phase 2 ADR supersede, not a local patch — the
  thesis is that one concept covers both.

- **M4-Phase 10 — Author-controllable sizing design spike.** The AC13
  spike. Deliverable is an **impact audit, not an implementation**:
  grammar / parser / checker impact, IR and runtime layout impact,
  C ABI / host-construction impact, interaction with `aspect`,
  Fill / Shrink defaults, Grid tracks, ZStack alignment, ScrollView
  viewport and WrapPanel item sizing, diagnostics for
  under-constrained combinations, and a narrowing pass over the seven
  design families in the
  [VDR](../cross-milestone/decisions/author-controllable-sizing-surface.md)
  §Design-space inventory. Its first task is to **re-establish that the
  Problem B repro cases still fire on current `main`** (a checker fix
  may have masked the collapse) rather than assuming it. Its output
  concludes either "implement in M5" or "return to the M6 ABI-freeze
  disposition", and it must record whether the parent-owned-layout-data
  family fires the PM-2 Grid wrapper-rule reopen trigger
  ([M3 handoff](../milestone-3/handoff.md) §PM-2 Grid Wrapper Rule).
  Placement rationale: §Cross-phase dispositions.

- **M4-Phase 11 — AccessKit / UIA integration.** Supply of the
  semantic tree and reading order, the text-state patterns the entry
  field owes, and the **modality rule fixed in Phase 2** — background
  content is hidden from the screen reader by focus scope, not by
  layer. Settles the layout → UIA synchronization timing question of
  [layout-engine.md §3.2](../../docs/notes/layout-engine.md) (bulk
  update at layout completion versus per-node lazy propagation).

  Placed after Phase 9 because its verification density comes from
  crossing heterogeneous widgets — list, entry field, button, item row,
  menu, dialog — which only all exist once B4 is complete
  ([spec.md](requirements/spec.md) §2 本の役割分担).

- **M4-Phase 12 — Mica / Acrylic identity and showcase close.** The
  root-window backdrop and system accent follow-through (initial
  surface only; the full theming surface is M5), applied to both apps,
  plus milestone close: both showcase applications assembled and
  running, the editorial pass over `docs/dsl_spec.md` /
  `docs/abi_spec.md`, the CHANGELOG entry, and the milestone handoff.
  Mica is deliberately last among the AC surfaces: it is
  app-independent and non-differentiating
  ([framing.md](requirements/framing.md) §機能リスト, P2 rows), so it
  neither blocks nor is blocked by anything else, and closing on it
  makes the identity claim visible on the final state of both apps.

### Phase dependencies

```
M4-Phase 1 (DPI / coordinate space)
      │
      ▼
M4-Phase 2 (event routing + focus model + generic click)   ◄── stretch
      │                                                        re-evaluation
      ├──────────────► M4-Phase 3 (predicates)                 checkpoint
      │                        │                               (Phase 2 ADR
      │                        ▼                                Accepted)
      ├──────────────► M4-Phase 4 (gallery completion)
      │
      ▼
M4-Phase 5 (single-line text editing)
      │
      ├──────────────► M4-Phase 6 (IME via TSF)
      │                        │
      ▼                        │
M4-Phase 7 (host state boundary + in-out binding)
      │                        │
      ▼                        │
M4-Phase 8 (multi-window + window config)                     │
      │                        │                              │
      ▼                        ▼                              │
M4-Phase 9 (top-layer overlays + modal dialog) ◄──────────────┘
      │
      ├──────────────► M4-Phase 10 (sizing design spike)
      │
      ▼
M4-Phase 11 (AccessKit / UIA)
      │
      ▼
M4-Phase 12 (Mica / Acrylic + showcase close)
```

Phase 1 (DPI) precedes everything because it fixes the coordinate
space that hit-testing, caret / composition rectangles, top-layer
placement and per-window scale factors all convert through. It is a
hard prerequisite in the practical sense — later phases could be
written against a 1:1 world and retrofitted, but their GUI evidence
would have to be recaptured.

Phase 2 is a hard prerequisite for every interaction phase after it
(4, 5, 6, 9, 11): each ships a consumer of the focus model or the
routing model. It is deliberately the second phase so the ADR that
carries the milestone thesis is written while the interaction surface
is still small enough to reason about, not after five widgets have
accumulated implicit expectations.

Phases 3 and 4 are independent of each other and both depend only on
Phase 2. Phase 3 owns predicate checking / lowering / evaluation plus the
runtime structural integration required by per-item conditional rendering;
Phase 4 owns scrolling, gallery widgets and image presentation. They are
sequenced 3 → 4 because
Phase 4's status-bar and caption evidence reads better once predicates
exist, and because Phase 3's novel normative spec section benefits from
landing before the milestone's heavier OS integration begins.

Phases 5 → 6 are strictly ordered: TSF integrates over the text model
Phase 5 delivers. Phase 7 depends on Phase 5 (write-back needs an
editable widget as its consumer) but not on Phase 6.

Phase 8 depends on Phase 7 because the dynamic window title consumes
whatever the host state boundary settles about where composed strings
come from. Phase 9 depends on Phase 8 (the top layer is defined
per-window, so the window model should be final first) and on Phase 6
(the rename dialog's proof is containment × top layer × text × IME
together).

Phase 10 (sizing spike) depends on Phase 9 only for its *inputs* — the
concrete screens that would want explicit sizing, and the settled
host-construction story from Phase 7. It ships no runtime surface, so
nothing depends on it inside M4; its consumer is M5 planning.

Phase 11 depends on Phase 9 because its verification density requires
the full heterogeneous widget set. Phase 12 depends on every preceding
phase (it assembles both apps and closes the milestone).

### Acceptance ↔ phase mapping

| Acceptance | Phase(s) |
|---|---|
| AC1 (input, focus model, event routing, generic click, modal focus scope) | M4-Phase 2 (model + `ZStack` consumer); second structure in M4-Phase 9; touch evidence in M4-Phase 2 |
| AC2 (multi-window, cross-window focus) | M4-Phase 8 |
| AC3 (single-line TextField) | M4-Phase 5 |
| AC4 (IME via TSF) | M4-Phase 6 |
| AC5 (AccessKit / UIA) | M4-Phase 11; modality rule fixed in M4-Phase 2 |
| AC6 (Mica / Acrylic + accent) | M4-Phase 12 |
| AC7 (per-monitor DPI) | M4-Phase 1 |
| AC8 (host state boundary) | M4-Phase 7 |
| AC9 (predicate expressions) | M4-Phase 3 |
| AC10 (top-layer overlays + focus rule set) | M4-Phase 9 |
| AC11 (window config properties) | M4-Phase 8 |
| AC12 (two showcase applications) | M4-Phase 12 (assembly); A grown incrementally in Phases 1–4 and 7, B4 in Phases 5–9 |
| AC13 (sizing spike + recorded disposition) | Disposition recorded by this plan (§Cross-phase dispositions); spike executed in M4-Phase 10 |

Every phase also carries the per-phase synchronization rule
(`.ui` → IR → runtime → `docs/dsl_spec.md`, plus `docs/abi_spec.md`
where ABI-bearing) that M3 carried as A11. M4 does not mirror it as a
separate criterion; it is a phase-end gate below.

### Cross-phase dispositions

Decisions this plan is required to make, each with the obligation that
demanded it.

#### 1. Focus containment / modal / accessibility — one phase or split

[m4-interaction-intake.md](../../docs/notes/m4-interaction-intake.md)
obligation 5 asks whether focus trap, modal semantics and accessibility
are handled together or split across phases, and names this plan as the
landing site.

**Disposition: one design, three implementations.**

- The **concept is settled once**, in the M4-Phase 2 ADR: a modal focus
  scope is a structure-independent property attachable to any subtree,
  and screen-reader modality attaches to that scope rather than to a
  layer. Both statements are Phase 2 ADR content and binding on later
  phases.
- The **`ZStack` consumer** (A's lightbox) is implemented in
  M4-Phase 2.
- The **top-layer consumer** (B4's rename dialog) is implemented in
  M4-Phase 9.
- The **screen-reader consumer** (background hidden from the reading
  tree) is implemented in M4-Phase 11.

Splitting the *implementations* is what makes the concept testable
against two structures and one accessibility surface; splitting the
*design* would reintroduce exactly the failure the milestone thesis
exists to prevent. If Phase 9 or Phase 11 cannot be built on the
Phase 2 concept without changing it, the correct response is a Phase 2
ADR supersede — recorded, not absorbed locally.

#### 2. Author-controllable sizing spike disposition (AC13)

Required as an auditable artifact by the
[sizing VDR](../cross-milestone/decisions/author-controllable-sizing-surface.md)
§M4/M5 activation mechanism. Default is "spike in M4"; deferral to M5
requires a positively recorded reason.

**Disposition: spike in M4, at M4-Phase 10.** The VDR's decision inputs,
answered:

| Input | M4 planning answer | Pull |
|---|---|---|
| A real M4 screen cannot be expressed without explicit sizing | Not established at planning time. The likely candidates are B4's rename dialog, item menu and entry field — content on the top layer that must not stretch to the layer's extent — but the top-layer placement model is not designed until Phase 9 | Weak pull to M4; pulls the spike **later within** M4 |
| M4 capacity | Enumerated, not asserted: 13 criteria across 12 phases, listed in §Phase breakdown. The spike produces impact tables and one repro re-establishment, so it fits as a bounded phase rather than displacing surface work | M4 |
| Can M5 absorb both spike and implementation | M5 already owns the official widget set, full theming and VS Code LSP, with M6 ABI freeze immediately after. Spike-plus-implementation compressed into M5 just before freeze is the backstop collapse the VDR names | M4 |
| Does a promising family depend on M4's interaction context | Partly yes: hit-area sizing (Phase 2), host construction (Phase 7) and top-layer placement (Phase 9) all bear on it; the pure-DSL families (direct attribute, wrapper) do not | Later-M4, not early-M4 |
| Design-space family width | Seven families recorded in the VDR. Narrowing is the spike's job, so width bounds its scope, not its placement | Neutral |
| Are the Problem B repro cases still reproducible on `main` | Not assumed. Re-establishing them is the spike's first task; if they have drifted or been masked, re-establishment precedes the audit | Verify-first |
| Runway to M6 ABI freeze | M5, then freeze. Shrinking | M4 |

No input supports deferral to M5, and two (M5 absorption, runway)
argue against it. The interaction-dependency input is what places the
spike at Phase 10 rather than early in the milestone: by then the
C ABI / host-construction story (Phase 7) and the concrete screens
(Phase 9) are both in hand, which is when the audit can be answered
rather than guessed.

#### 3. Stretch re-evaluation checkpoint

[framing.md](requirements/framing.md) §二層取り込み方針 sets the
re-evaluation point for the stretch (AC-exempt) intakes at *"the moment
the focus-model core ADR is Accepted"*. In this plan that is the
**M4-Phase 2 ADR Accepted flip**. At that point the remaining stretch
volume is re-read against what Phase 2 actually cost:

- `Image` widget and direct-value `fill` (M4-Phase 4) — withdrawal
  costs a one-line disposition back to the
  [candidate pool](../candidate-pool.md).
- Multi-line text editing — not scheduled as a phase. It rides
  M4-Phase 5 or 6 only if there is room; withdrawal costs a carry-
  forward line to M5, since it is not even a candidate-pool item
  ([framing.md](requirements/framing.md) §粒度の定義（文字編集）).

Core intakes (AC8–AC11) are **not** in scope for this checkpoint;
dropping one of those is a
[DD-V-026](../cross-milestone/decisions/plan-revision-discipline.md)
tier-2 reduction with a reopen-condition table.

#### 4. Decisions routed out of framing

[framing.md](requirements/framing.md)
§採択後・各フェーズへ送る決定事項, with landing sites now fixed:

| Sent decision | Landing site |
|---|---|
| Event routing model (three-phase capture / target / bubble versus target-only plus high-level signals) | M4-Phase 2 ADR (mandatory topic) |
| Touch verification environment — which environment and which form of evidence discharges the touch half of AC1 | M4-Phase 2 phase plan, recorded against the taxonomy in [verification-environments.md](../../docs/notes/verification-environments.md) |
| Sizing spike placement | This plan, §Cross-phase dispositions 2 → M4-Phase 10 |
| Dynamic title's value source (binding versus host composition) | M4-Phase 8 ADR |

#### 5. Host-language parity for the second app

M3's A1 required the Gallery to run on all three example hosts
(Rust / C / Zig). M4 keeps that for A, which already has three hosts.
For B4 this plan requires:

- B4 ships on the **Rust host** as its primary form; and
- every **ABI-bearing** surface (M4-Phase 7 above all, plus window
  creation changes from M4-Phase 8) is exercised from **at least one
  non-Rust host**, so an ABI regression cannot hide behind Rust's
  in-workspace build.

Whether B4 itself gains full C and Zig hosts is decided at
**M4-Phase 7**, when the shape of the ABI change is known, and recorded
there. This plan does not commit the full three-host matrix for B4 in
advance, because the cost is a function of an ABI surface that has not
been designed yet.

### Out of scope

Surfaces explicitly excluded by
[framing.md](requirements/framing.md) §M4 に入れないもの and
[spec.md](requirements/spec.md) §範囲外. Recorded here so no phase
absorbs them silently:

- Full theming — light / dark themes, accent propagation into widgets,
  a type-scale system → M5. M4 ships the root backdrop and accent
  follow-through only.
- The official widget set (CheckBox / ComboBox / Menu / Dialog and
  siblings) → M5. M4's menus and dialogs are author compositions; the
  Phase 2 focus fixture is explicitly not a published widget.
- Anchored popovers — declarative anchoring to a widget, coordinate
  transformation, placement rules → M4 explicit defer; recorded in the
  [candidate pool](../candidate-pool.md), buyer is M5's widget set.
- Widget name references
  ([dsl-grammar.md](../../docs/notes/dsl-grammar.md) Q1) → M4 explicit
  defer, as a consequence of excluding anchored popovers.
- `TypedValue` / structured item data (M-expr2b/3) → held. B4's first
  upper bound (items stay a sequence of strings) exists to keep this
  hold intact.
- String concatenation and general arithmetic in expressions → M4's
  expression language stops at predicates.
- Theme-aware widget background colours, reactive `fill` → held.
- Developer debugging support → held (M5 tooling).
- Widget extension mechanism → held (post-1.0, re-checked at the M6
  freeze).
- Distribution — artifacts, channels, signing, versioning → held. M4
  needs only "a contributor can obtain and run the showcase", which
  clone + build instructions satisfy.
- Hot reload, animation DSL → post-1.0.
- Undo / redo, password masking, rich text in the entry field → not in
  M4 ([framing.md](requirements/framing.md) §粒度の定義（文字編集）).

### Verification strategy

Verification means are partitioned by what the code touches, per
[CLAUDE.md §Testing rules](../../CLAUDE.md). Each phase chooses from
this menu and states which it uses in its ADR / implementation plan.
M4 leans harder on the OS-integration side than M3 did, so the
partition matters more, not less.

- **Pure-logic unit tests** (`cargo test`, no Win32 / WinRT). Used
  for: the focus traversal core (tree + scope annotations → next focus
  target, including group traversal and focus / active-item
  separation); predicate evaluation and its lowering; the text buffer,
  caret index and selection-range arithmetic; DPI scale-factor
  arithmetic and coordinate conversion; scroll offset and scrollbar
  thumb geometry; IR types and serialization for every new surface.
- **Windows-only mock-free integration tests** against the real OS
  runtime surface. Used for: focus and event delivery through live
  widget state, TSF text-store round trips, per-window state and
  cross-window focus, top-layer realization, and UIA tree shape.
  CI-gated; these **fail rather than silently skip** on GitHub Actions
  when the required runtime capability is unavailable, and any skip
  guard is verified on an environment that actually lacks the
  capability before it lands
  ([verification-environments.md](../../docs/notes/verification-environments.md)).
- **Target-app evidence.** Each phase wires its surface into A or B4
  per the [spec.md](requirements/spec.md) §2 本の役割分担 split and
  runs it through `.ui` → IR → runtime, not host-imperative
  construction. Visual fidelity is judged against
  [gallery-wireframe.html](../milestone-3/requirements/gallery-wireframe.html)
  for A and
  [target-app-wireframes.additional.html](requirements/target-app-wireframes.additional.html)
  for B4.
- **GUI evidence with a positive control.** Because M4's surfaces are
  interactive, a single static frame is weak evidence — a wrong
  implementation can produce the same picture. Every phase whose
  evidence is "the GUI actually did the thing" captures screenshots and
  the assistant analyses them, and the capture set must include a
  control that distinguishes the intended behaviour from a
  coincidental look-alike: focus moved *to the expected next stop* and
  not merely somewhere; the modal scope *blocked* the background rather
  than the background happening to be empty; the composition string
  *replaced* the selected range. This is the assistant's pre-owner
  baseline and does not replace the owner's human-visible smoke.
- **Spec drafting** is verification of a different kind:
  `docs/dsl_spec.md` (and `docs/abi_spec.md` for ABI-bearing phases) is
  updated within the same phase, and the phase-end check asks whether
  the text would let an external implementor reproduce the surface.

### Phase-end criteria

A phase closes when **all** of the following hold:

1. **ADR Accepted.** Every design decision in the phase's
   `decisions/` set is `Accepted`, with no `Proposed` DD left open.
2. **Implementation landed** across `.ui`, `wasamo-ir`, `wasamoc`,
   `wasamo-runtime` and any host glue, with
   `cargo build --release --workspace` and `cargo test --workspace`
   green locally and on GitHub Actions CI.
3. **Verification evidence recorded.** Pure-logic and / or
   Windows-only integration tests covering the phase's surface pass on
   CI; the implementation log records which means was used and links
   the CI run. Phases with a GUI-visible result additionally record
   screenshot evidence with its positive control.
4. **Spec synchronized.** `docs/dsl_spec.md` reflects the surface the
   phase ships; ABI-bearing phases also move `docs/abi_spec.md`. A
   phase whose spec text is "TODO" does not close.
5. **Target-app slice runs.** The relevant slice of A or B4 exercises
   the new surface end-to-end from at least one example host. Phases
   that ship no author-facing surface (M4-Phase 1) satisfy this by
   demonstrating the existing app under the new runtime property.
6. **Out-of-phase residuals filed** in a live note or the phase
   handoff, pointed to from the ADR — not silently carried.
7. **Phase-end retrospective recorded** at
   `phase-N/retrospectives/phase-end.md`, with the merge and push gates
   per [retrospectives.md](../procedures/retrospectives.md).

### Milestone-end criteria

M4 is complete when **all** of the following hold:

1. **Every acceptance criterion AC1–AC13 is discharged**, with the
   discharge recorded in the corresponding ADR and the Progress row
   marked complete.
2. **Both showcase applications run** (AC12): A on all three example
   hosts, B4 per the host-parity disposition fixed at M4-Phase 7, with
   the outward-facing banner role assigned to A per
   [spec.md](requirements/spec.md) §「見本アプリ」AC の単数前提の審査.
3. **The focus model is settled once, demonstrably.** The modal focus
   scope defined in M4-Phase 2 is exercised by all three of its
   consumers — `ZStack` branch, top layer, screen-reader modality —
   without a per-consumer special case. A special case that survived is
   recorded as a residual, not glossed.
4. **Spec and ABI sync is auditable**: every phase ADR set names the
   `docs/dsl_spec.md` section it modified, and every ABI-bearing phase
   names its `docs/abi_spec.md` change.
5. **AC13's spike output exists** as a durable artifact with a
   conclusion ("implement in M5" or "return to the M6 disposition"),
   per the sizing VDR.
6. **CHANGELOG entry** for M4 lands, linking each phase ADR.
7. **No silently deferred M4 surface.** Anything in
   [spec.md](requirements/spec.md) §M4 が開く機能面 is either shipped
   or recorded as a deviation in this plan's Revision log — including
   the stretch rows, whose withdrawal is cheap but must still be
   written down.
8. **Clean rebuild green on CI** for the merge commit on `main`.
9. **Milestone handoff recorded** at `milestone-4/handoff.md`.

### Risks

- **M4-Phase 2 over-scope.** The phase carries the milestone thesis and
  a long mandatory-topic list; an ADR that tries to settle routing,
  traversal, modality, hit-test eligibility and the `Button.clicked`
  relation at once can stall. Mitigation: the pre-ADR fixture spike
  forces the hardest semantics (group traversal, focus / active-item
  separation) to be answered by running code before the ADR closes, and
  the ADR is allowed to *fix rules for later phases without
  implementing them* — the modality rule is the model case.

- **TSF is the heaviest OS integration in the milestone.** M4-Phase 6
  has no pure-logic escape hatch; it is mock-free Windows-only by
  construction, and a wrong text-store model surfaces late. Mitigation:
  M4-Phase 5's ADR must fix the text-store-facing internal model
  explicitly, so Phase 6 is wiring; and Phase 5's selection model is
  built to TSF's requirements even where the user-facing operation
  would not need it.

- **ABI churn across three hosts.** M4-Phase 7 is the milestone's only
  ABI-bearing phase, and M6 freezes the ABI. A boundary designed
  against one consumer will not hold. Mitigation: the phase designs all
  three directions together, exercises at least one non-Rust host, and
  discharges the `TypedValue` ABI-impact homework
  ([framing.md](requirements/framing.md) §検討ノートのケース分類と M4
  期間中の宿題) rather than leaving "ABI impact: unknown" standing.

- **DPI-first ordering could stall the thesis work.** If M4-Phase 1's
  coordinate localization turns out to be a layout-unit redesign rather
  than a boundary conversion, the milestone's centre of gravity starts
  late. Mitigation: Phase 1's scope is awareness declaration,
  conversion at the boundary and `WM_DPICHANGED` — not a change to how
  layout computes. If it grows past that, the correct response is a
  tier-2 plan revision, not silent expansion.

- **B4 has no runnable form until M4-Phase 5.** Five phases pass before
  the second showcase app exists at all, so a slip in the text phases
  puts AC12 at risk late. Mitigation: Phase 5's evidence *is* B4's
  minimal list window — the app starts existing at the earliest phase
  that can carry it, and Phases 6–9 grow it rather than assembling it
  at the end.

- **Two apps plus twelve phases is a long milestone.** Mitigation: the
  stretch re-evaluation checkpoint at the Phase 2 ADR flip is the
  scheduled place to shed volume, with the cheap withdrawals
  (stretch rows) separated in advance from the expensive ones (core
  intakes under tier-2 reduction).

- **The second structure could break the one-concept claim.** If
  M4-Phase 9's top layer cannot use the Phase 2 modal scope unchanged,
  the milestone thesis is materially weakened even if both apps work.
  Mitigation: this is called out as a Phase 2 ADR supersede path in
  §Cross-phase dispositions 1, and as milestone-end criterion 3, so it
  cannot be settled by adding a quiet special case.

### Revision log

Revisions to this `Frozen agreement` follow the plan-revision discipline
— [workflow.md](../procedures/workflow.md),
[DD-V-026](../cross-milestone/decisions/plan-revision-discipline.md):
owner-authorised, critically checked, proportionally recorded.

**Revision 1 (2026-08-08) — M4-Phase 5 gains a named prerequisite: the
handler-side scalar `string` write.**

- **What changed.** One paragraph added to the M4-Phase 5 description.
  No scope, acceptance criterion, phase boundary or dependency moves;
  the phase already owned "how a typed string reaches author-visible
  state" through its one-way-binding-plus-handler approach.
- **The premise that changed.** The phase's stated approach is "matching
  the M3 `ToggleButton.checked` precedent" — one-way binding plus the
  author's handler writing the state. M4-Phase 2 T9 measured that the
  handler half of that precedent is **unavailable for strings**: there
  is no scalar string write in the runtime, and `dsl_spec.md` §8.9
  already marks the string expression forms binding-only. When this plan
  was drafted (2026-07-28) nobody had checked whether the bool
  precedent's mechanism had a string twin; it does not.
- **Critical check.** The revision was proposed by the agent after the
  measurement and authorised by the owner on 2026-08-08. The premise was
  verified rather than assumed: the compiler was run against a probe
  `.ui` (all three string right-hand sides — literal, state read,
  interpolation — pass `check` and emit IR that §8.9 marks binding-only),
  and the evaluator was run against each of the three (all reject at
  invocation). The alternative placements were checked and rejected on
  the phases' own stated scope: M4-Phase 3 says "String concatenation
  stays out" and its consumers need reads, not writes; M4-Phase 4 is
  scrolling / `Image` / `fill`; M4-Phase 7 is two phases past the point
  where the one-way form is needed.
- **Why the milestone plan rather than a phase handoff.** A phase
  handoff carries one hop (phase N → phase N+1's `constraints.md`).
  Three hops would have to survive Phase 3 and Phase 4 forwarding it,
  and this plan is the only document M4-Phase 5's framing is guaranteed
  to read.
- **Not included.** The *interim* half of the same finding — that the
  unsupported form is accepted silently today rather than diagnosed — is
  not a milestone-plan item. It is an M4-Phase 3 pre-doc intake
  (owner-settled the same day), because enforcing §8.9's existing
  binding-only statement is a defect fix against normative text already
  in force, not a new surface.

**Revision 2 (2026-08-08) — M4-Phase 4 gains a named deliverable: what a
`Row` / `HStack` does when its children do not fit.**

- **What changed.** One paragraph added to the M4-Phase 4 description. No
  acceptance criterion, phase boundary or dependency moves, and no
  particular layout rule is chosen here — the phase owes **a rule stated
  in `dsl_spec.md`**, and "overlapping is acceptable" is one of its
  available contents. The obligation is the *stating*: the runtime
  already behaves one way, and what is missing is a document that says
  so, which is why "we decided internally to leave it" would not
  discharge this.
- **The premise that changed.** M4-Phase 2's framing agreement ④
  (owner-approved 2026-08-05) already sent the question here, on the
  basis of an M4-Phase 1 observation recorded as a **visual** overlap.
  M4-Phase 2 T10 measured a second half nobody had: the overflowing group
  also **takes the clicks** aimed at the widgets it covers, so the
  overlapped controls stop working rather than merely looking wrong. That
  changes what the receiving phase is deciding — a cosmetic question
  became a functional one.
- **Critical check.** Measured, not inferred, and by a fixture that
  prints its evidence: each gallery tab `ToggleButton`'s own centre
  resolves to a scroll `Button` (`All` → `Scroll down`, `Albums` →
  `Scroll down`, `Favorites` → `Scroll up`), and a real click at
  `Albums`' centre leaves all three `checked` values unchanged. The cause
  was checked against the specification rather than assumed to be a
  defect: [dsl_spec.md §4.19](../../docs/dsl_spec.md) makes every widget
  with a visual a candidate, a layout container included, and the runtime
  does exactly that. Authorised by the owner on 2026-08-08 after the
  measurement was presented with that reading.
- **Why the milestone plan rather than a phase handoff.** The same test
  Revision 1 applied: a handoff carries one hop, and M4-Phase 4 is two
  from the phase that measured this. It would have to survive M4-Phase 3
  forwarding a finding that is not M4-Phase 3's. Two documents a reader
  might follow point elsewhere — this plan's M4-Phase 4 entry did not
  mention it at all, and
  [M4-Phase 1's handoff](phase-1/implementation/handoff.md) still names
  **M4-Phase 2** as the landing place, which framing ④ superseded. That
  row is not edited: a closed phase's handoff is its record of what was
  known then, and this entry is the authoritative placement.
- **Not included.** What the rule should say. Also not included: any
  change to hit-testing, which behaves as specified.

**Revision 3 (2026-08-11) — Correct M4-Phase 3's cross-layer
responsibility.**

- **What / tier.** Tier 2 additive/refining. The Phase 3 description now
  explicitly owns the loop context and runtime structural integration required
  by its already-planned per-item conditional. The dependency prose no longer
  calls Phase 3 compiler-side or contrasts it with Phase 4 as runtime-side. No
  new author-facing capability was added by this revision.
- **Initiator.** Agent, from the Phase 3 source audit and independent framing
  review.
- **Old premise.** Checker, lowering and evaluator work could deliver the
  planned per-item conditional inside a compiler-side phase boundary.
- **New evidence.** The existing conditional effect uses an ordinary
  `BindingEvalContext`, and false-to-true re-materialisation calls
  `build_node(...)`; iteration uses item / index context and
  `build_node_with_loop_context(...)`. A condition inside a `for` can therefore
  lose its binder both when evaluated and when its subtree is rebuilt. That
  subtree also owns effects and handlers and crosses the Phase 1 / 2 focus,
  hover, handler-registry and layout lifecycles when inserted or removed.
- **Why the old plan no longer holds.** A compiler-only delivery could accept
  and lower the authored form without preserving the runtime binder and
  structural lifecycle that give it meaning. The old responsibility statement
  was insufficient for the deliverable already in the plan.
- **No-change option considered.** Leave the runtime work implicit. Rejected
  because the hidden dependency would misstate ADR responsibility, call-site
  review and GUI evidence during implementation planning.
- **Critical check.** Owner-completed 2026-08-11: the evidence and proposed
  change boundary were judged valid and proportionate.
- **Owner authorisation.** Authorised 2026-08-11 for the exact proposal recorded
  in the Phase 3 plan-revision artifact.
- **Impact check.** No AC meaning, phase order, completed-phase evaluation,
  retrospective / merge gate or ROADMAP text changed. Phases 3 and 4 remain
  independent after Phase 2. Phase 3's later evidence must cover the structural
  consumer it already promised.

## Progress

The Progress section is a compact milestone index. Detailed live task
tracking belongs in each phase's `implementation/plan.md` (with
`implementation/log.md`).

| Phase | Status | Progress file | ADR | Notes |
|---|---|---|---|---|
| M4-Phase 1 — Per-monitor DPI awareness | phase-end recorded; phase→main owner approval pending | [plan.md](phase-1/implementation/plan.md) | [preamble.md](phase-1/implementation/preamble.md) | Discharges AC7 and the M3 runtime-DPI residual. Framing owner-aligned 2026-07-28; four decisions Accepted 2026-07-28 (Per-Monitor-Aware V2 declared by the runtime in `wasamo_init`; layout in DIP with conversion confined to the seams and crispness bought at the rasterization surface; per-window scale with a fixed `WM_DPICHANGED` order; DIP as the outward unit with no new ABI function). Framing agreements ①/②/③ each tested inside the slate — no stage-2 plan revision required, and "Phase 7 is the milestone's only ABI-bearing phase" survives. Moment 1 spec sync landed 2026-07-28 ([architecture.md §12](../../docs/architecture.md#coordinate-spaces), [dsl_spec.md §1 units](../../docs/dsl_spec.md), [abi_spec.md §4.1 / §4.2](../../docs/abi_spec.md), [layout-engine.md §3.1](../../docs/notes/layout-engine.md) answered); Moment 2 implementation sync landed at T12, including the corrected `verification-environments.md` Observation 4 procedure. Initial phase-end CI runs `30873359437` and `30873615639` failed the DPI matrix test; repair code commit `1f162dc` passed [run 30878747516](https://github.com/matarillo/wasamo/actions/runs/30878747516) on the experimental branch. After the review-remediation test-target change, [phase-branch run 30881324493](https://github.com/matarillo/wasamo/actions/runs/30881324493) passed and directly verified the current code tree; the CI criterion is discharged without erasing the failed-run history |
| M4-Phase 2 — Event routing + focus model + generic click | phase-end recorded; phase→main owner approval pending | [plan.md](phase-2/implementation/plan.md) | [preamble.md](phase-2/decisions/preamble.md) | Milestone core. Framing owner-aligned 2026-08-05 (seven agreements, six of them explicitly as revisable hypotheses; touch = synthesized injection with stated limits, no hardware available). Pre-ADR spike discharged — [focus-traversal-spike.md](phase-2/decisions/exploration/focus-traversal-spike.md): traversal core as Win32-independent pure logic under unit test, plus a mechanism fixture driving it from the real `.ui` → IR → runtime path. Five decisions Accepted 2026-08-06: DD-001 (target-then-bubble with no capture phase and consume-on-handle; an unconsumed key reaches the default window procedure; touch on `WM_POINTER*` without changing the host process's input mode), DD-002 (layout-derived DIP hit rectangles retained beside the Visual write by the one lockstep pass; one topmost target bounded by ancestor clips, from which occlusion follows rather than needing a scrim rule), DD-003 (one `FocusState` per window with tree-order traversal, `enabled: false` removing a stop, click focusing the nearest focusable widget at or above the target, and group traversal and focus / active-item separation settled for M5 without building widgets), DD-004 (a modal scope as an annotated subtree whose presence is the entry — materialisation captures the restore target, not derivable from the tree afterwards, and moves focus inside), and DD-005 (per-item handlers admitted inside `for` bodies together with the registration lifecycle and the position-not-item identity consequence M3-Phase 7 routed here; `focus-group` and `modal-scope`; a `dismiss` request admitted beside `modal-scope` only, whose sources grow without the spelling changing; and one `key-down("<key>")` command signal, the physical-key half with non-character key names only). Moment 1 spec sync ([dsl_spec.md §4.19](../../docs/dsl_spec.md), [architecture.md §13](../../docs/architecture.md), [m4-interaction-intake.md](../../docs/notes/m4-interaction-intake.md)) re-syncs at the Accepted flip. **Stretch re-evaluation discharged 2026-08-05: both stretch intakes retained** (`Image` / direct-value `fill` stay at M4-Phase 4; multi-line editing stays ride-if-room — §Cross-phase dispositions 3). T1–T12 and the owner-visible smoke are complete; T13 synchronized the specs and drafted the phase-end retrospective / handoff. Its first cold suite fired CF-T7-1: allocator reuse retained a focus record on a fresh row without repainting its indicator. Owner-authorized T13a repaired that divergence through the existing focus writer, added an allocator-independent red/green fixture, completed full independent review and passed a replacement cold suite plus all consumer checks. Pointer-address identity remains a bounded handoff residual. T13/T13a were merged no-ff as `b42a212`; phase-branch [CI run 31302529054](https://github.com/matarillo/wasamo/actions/runs/31302529054) passed on `4b1076f`; phase→main merge remains a separate owner gate |
| M4-Phase 3 — Predicate expressions | not started | — | — | Novel normative DSL content; absorbs three M3-deferred DSL axes |
| M4-Phase 4 — Gallery completion | not started | — | — | Scroll / scrollbar / `Image` / direct `fill`; two stretch intakes |
| M4-Phase 5 — Single-line text editing | not started | — | — | Must fix the text-store-facing internal model for Phase 6 |
| M4-Phase 6 — IME via TSF | not started | — | — | Heaviest OS integration; crosses the Phase 1 DPI bridge |
| M4-Phase 7 — Host state boundary + in-out binding | not started | — | — | Only ABI-bearing phase; fixes B4 host-parity disposition |
| M4-Phase 8 — Multi-window + window config | not started | — | — | Decides the dynamic title's value source |
| M4-Phase 9 — Top-layer overlays + modal dialog | not started | — | — | Second structure for the modal focus scope |
| M4-Phase 10 — Author-controllable sizing spike | not started | — | — | AC13; audit deliverable, no runtime surface |
| M4-Phase 11 — AccessKit / UIA | not started | — | — | Implements the Phase 2 modality rule |
| M4-Phase 12 — Mica / Acrylic + showcase close | not started | — | — | AC6 + AC12 assembly + milestone close |

### Owner-facing resume note

This plan was drafted from the accepted M4 framing, the accepted
target-app spec, and the 2026-07-28 acceptance-criteria revision, and
**owner-agreed on 2026-07-28**. M4-Phase 1 (per-monitor DPI awareness)
opens from here.

The `Frozen agreement` section is revisable under the
plan-revision discipline
([workflow.md](../procedures/workflow.md), DD-V-026) — owner-authorised,
critically checked, proportionally recorded.
