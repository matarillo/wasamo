---
title: Selected / toggle-state authoring surface
status: Proposed
phase: M3-Phase 8
ac: A10 (existing) — selected-state is reserved by A10 from the start (unlike 7b's newly-minted A13), so no new AC is expected; discharges under A10 + A11 + A12. Any new public-promise AC would come from DD-002, not this DD. Confirmed at the Accepted flip (framing FD-8-F).
date: 2026-06-26
related:
  - ./preamble.md
  - ./dd-m3-p8-002-dsl-spec-public-draft-promotion.md
  - ../requirements/framing.md
  - ../requirements/constraints.md
  - ../requirements/dd-001-stage1-spike.md
---

# DD-M3-P8-001 — Selected / toggle-state authoring surface

**Status:** Proposed

## Context

How should an author write a **selected / toggle** state in `.ui` — on a
dedicated toggle widget, or as an attribute on the generic Button?

A10 is the last new authoring surface M3 delivers: it shows that a
**boolean binding (Phase 1) can drive a widget *attribute***, not just a
widget's text or its presence under an `if`. The Photo Gallery's tab band
(several buttons, one selected) is the candidate place to exercise it
([gallery-wireframe.html](../../requirements/gallery-wireframe.html)). A
selected/toggle state appears in neither the spec nor the gallery today,
so this DD defines the surface for the first time.

Owner review of the stage-1 spike
([../requirements/dd-001-stage1-spike.md](../requirements/dd-001-stage1-spike.md))
split the question into **two layers**, and this DD decides them in that
order:

- **Layer 1 — how the author *writes* "selected".** The authoring form
  itself (the spike's S1–S6 audit). This is the load-bearing decision.
- **Layer 2 — how the driving boolean is *produced and narrowed to one*.**
  Given the Layer-1 form, how the driving boolean is supplied and how
  "exactly one selected" exclusion is expressed with shipped surface (the
  spike's α/β/γ/δ).

**Settled floor (not re-litigated).** Three things are fixed by the
framing and the stage-1 spike and are *not* reopened here:

- The driving value is a **boolean riding the existing boolean binding** —
  packet C, owner-aligned 2026-06-25; this DD does **not** re-decide
  whether to use a boolean at all. *Which surface carries it* is the
  load-bearing choice below. The starting point is recorded honestly:
  packet C **recommended** the `selected: bool` *attribute* with **no new
  widget** (framing line 48; owner-checked ☑ line 58), and the stage-1
  spike **concluded S1 as the lead, S2 as semantic comparison** (spike
  §結論). Packet C also left the alternatives **un-rejected** for this DD
  to decide (framing §66). The S2a recommendation below therefore
  **reverses** that prior S1 lead on owner intent — it is **not** a neutral
  pick among equally-open options; the reversal weight and its Accept-time
  re-sync are carried in §Recommendation (Layer 1) / §Accepted-time
  re-sync.
- Selected-state visuals are **minimal** in M3 (a colour/border
  difference), with the full theme surface owned by M5
  ([spec.md](../../requirements/spec.md) Out-of-scope §Visual).
- No new layout primitive and no new measure/arrange is introduced
  (framing §再検討しない前提; constraints §1). A dedicated toggle widget
  reuses Button's existing leaf measure/arrange; it is a new *node*, not a
  new layout primitive.

Per the owner prior, every option below is compared on **product merit /
thesis fit first**; revision cost is a tie-breaker, never a rejection
ground, and the over-engineering brake stays in force.

## Dependencies

- **Consumes** framing FD-8-A (M3-closing thesis: A10 + A1 + A12, no new
  primitive), FD-8-B (two-DD slate), FD-8-C (boolean-on-existing-binding
  direction; exclusion feasibility settled by the pre-DD spike, not
  asserted here; the carrying *widget surface* left open for this DD per
  framing §66), FD-8-E (implement-not-docs scope), and FD-8-F (no new AC
  expected unless DD-002 adds a public promise).
- **Fed by** the stage-1 feasibility spike
  ([../requirements/dd-001-stage1-spike.md](../requirements/dd-001-stage1-spike.md)),
  which is the authority for which Layer-2 options are *real* on shipped
  surface. The spike compiled each candidate (`wasamoc check` / `build`)
  and ran the adopted candidate (α) on the live runtime path. This DD does
  not re-derive those facts; it cites them.
- **Couples to** [DD-M3-P8-002](./dd-m3-p8-002-dsl-spec-public-draft-promotion.md)
  only at the documentation seam: DD-002 positions any deferred selection
  surface (the Out-of-scope axes below) in the public draft. DD-002
  does not decide this DD's authoring form. Because that positioning is
  where the owner sees how the reserved axes read as *public contract*,
  **DD-001 should not Accept before at least a DD-002 + `preamble.md`
  skeleton exists** enumerating this DD's adopted form, its reserved axes,
  and their public-draft representation (drafted in parallel per the
  Phase-8 plan; the `related` links above resolve when those skeletons
  land).
- **Shipped-surface facts that bound the space** (spike §確定した事実,
  not re-litigated): the expression grammar has **no `==`** (`HandlerExpr`
  /`CompoundOp` = `Add/Sub/Mul/Div` only,
  [wasamo-ir/src/lib.rs](../../../../wasamo-ir/src/lib.rs)); `if` conditions
  are `BOOL_LIT | IDENT` resolving to a `bool` state, **no operators**
  (no `!`, no comparison); handlers are block assignments only; and there
  is **no public API for the host (or a widget) to write component state
  while it is displayed** ([host-state-boundary.md](../../../../docs/notes/host-state-boundary.md)).
  Together these mean exclusion can only be expressed as a *combination of
  boolean states*, never as an expression — which is what shapes Layer 2 —
  and they are what put **two-way binding** (state write-back — the SwiftUI
  model) and **widget-owned mutable state** (the WPF/Qt model) out of M3's
  reach, which is what defers **S2b** and **S2c** below.

## Main decision A — authoring form (Layer 1)

The load-bearing decision: on what surface an author writes that something
is selected. The stage-1 spike audited the candidate space S1–S6 against
shipped-surface facts. The dedicated-toggle-widget idea splits into **three
distinct models**, named after established frameworks by *where the
canonical state lives* and *who writes it on a click* — **S2a (Compose
model)**, **S2b (SwiftUI model)**, **S2c (WPF/Qt model)** — only the first
of which is real on M3's shipped surface. S2a is weighed head-to-head
against the same boolean attribute on the generic Button (**S1**). The
disposition this DD proposes is **S2a lead / S1 minimal alternative / S2b +
S2c deferred (two-way binding / widget-owned state exceed Phase 8) / S3
explicit reject / S4–S6 out of scope (per-reason)**.

The three toggle models differ on two questions (used throughout, also in
§Forward-compat):

- **Where does the canonical state live?** *lifted* into a `state`
  declaration (S2a, S2b), or *owned by the widget* (S2c).
- **After a click, who writes that state?** the **author's code** (S2a —
  one-way, controlled: a `clicked` handler, or in Compose a value-carrying
  `onCheckedChange` callback the caller commits); the **widget, through a
  two-way binding** to the lifted state (S2b — SwiftUI `Toggle(isOn: $x)`);
  or the **widget into its own internal state** (S2c — WPF/Qt
  self-toggling).

M3's shipped surface offers only the **S2a** cell: lifted state + the
author writing it (handlers are block assignments; there is no host/widget
state write-back — §Dependencies). **S2b** needs two-way binding; **S2c**
needs widget-owned mutable state — both out of M3, and S2c additionally
strains wasamo's current lifted-state *hypothesis* — a family-level call
the owner may still take (§Forward-compat).

**Lexical naming is a separate sub-issue (SI-4), not part of this choice.**
The option labels below use `ToggleButton` / `checked` as **running
placeholders**. The load-bearing Layer-1 decision is *attribute on the
existing Button* (S1) vs *a dedicated toggle widget* (S2a–c) and, within
the latter, the binding model — **not** the concrete type/attribute
lexeme. If a dedicated widget (**any** S2 model) is chosen, the actual
type name (`ToggleButton` / `SelectableButton` / `TabButton` / …) and
attribute name (`checked` / `selected` / …) is an owner-presented pick
recorded in **SI-4**, common to all S2 models and not settled by picking
S2 over S1.

### Options

1. **S2a — `ToggleButton { checked: <bool> }` (controlled + one-way)**
   *(recommended).* A dedicated widget whose **type names the toggle/
   select purpose**, carrying a boolean `checked` attribute on the same
   one-way boolean binding the generic Button would use. The toggle
   behaviour comes from a **handwritten `clicked` handler writing state**
   (Layer 2) — exactly S1's mechanism, and (like Compose's toggles)
   **controlled + one-way**: the state is lifted into a `state` declaration
   and the author writes the transition. S2a adds **no** write-back
   (that is S2b) and **no** self-toggle (that is S2c), and not even the
   typed value-changed callback affordance Compose has — just a plain
   `clicked`.
   - What you gain — *and only this*: the selected/toggle purpose is
     **named in the type**. Benchmark frameworks model a selectable tab /
     item as a *dedicated typed component* (WinUI `ToggleButton` /
     `TabView`, SwiftUI `Toggle` / `Picker`, Compose `Tab` / `FilterChip`,
     Flutter `ChoiceChip`) rather than as a flag on a momentary-action
     button — that **type-naming** is the borrowed idiom. **The idiom is
     the type name, not a binding mechanism:** Compose's `Switch(checked,
     onCheckedChange)` is itself **controlled + one-way** (state hoisted;
     the caller writes it in the callback — unidirectional data flow), i.e.
     the **same cell as S2a**, so it lends S2a no two-way advantage. The
     one thing S2a lacks versus Compose is a *typed value-changed callback*
     (S2a uses a plain `clicked` handler) — an **ergonomic refinement
     within one-way control**, orthogonal to binding direction, **not** a
     two-way feature. Genuine two-way (write-back) is S2b; self-toggle is
     S2c. S2a is a *possible* type-home for later toggle/group growth — but
     only if those models arrive as same-type binding modes, not as separate
     widgets (an undecided fork — §Forward-compat); the benefit is therefore
     **contingent**
     ([architectural-family.md](../../../../docs/notes/architectural-family.md)).
   - What you give up: a **new dedicated widget node end-to-end** (parser →
     check → lower → IR → runtime loader) **plus its rendering path** —
     wider than S1, which adds only an attribute to an existing widget. The
     **real S1-vs-S2a delta is precisely** *"attribute on the existing
     Button"* vs *"new node + `checked` attribute + a Button-like click
     handler"* — not external full-toggle ergonomics. S2a also **consumes a
     new dedicated widget name** (its concrete lexeme is SI-4) **+ its
     initial (controlled, one-way) contract**, a forward cost S1 avoids by
     leaving Button's namespace untouched (§Forward-compat).
   - Technical risk: a new widget node plus a selected visual; each step
     rides the existing single-boolean primitive and Button's existing
     leaf measure/arrange (no new layout primitive), but the widget-node
     surface is wider than S1's attribute-only change.
2. **S1 — `Button { selected: <bool> }` (controlled + one-way)** *(minimal
   alternative).* The same one-way boolean attribute, placed on the
   **existing** generic Button instead of a new widget.
   - What you gain: **no new widget** — the smallest possible surface; the
     change is one attribute riding the boolean-binding path Phase 1 ships;
     A10's thesis (a binding drives an *attribute*) is expressed directly.
   - What you give up: a momentary-action `Button` now also carries a
     persistent selected flag, which **does not name the toggle/select
     purpose** the way the benchmark frameworks do; the select role lives
     only in author convention, not the type.
   - Technical risk: cross-cutting (parser → check → lower → IR emit →
     runtime loader → widget visual → cross-host parity), but narrower than
     S2a — an attribute on an existing widget, no new node.
3. **S2b — SwiftUI model: two-way binding to lifted state** *(deferred —
   exceeds Phase 8).* The state stays in a `state` declaration (lifted, as
   in S2a), but `checked` is bound **two-way**, so a click writes the new
   value back through the binding with **no author handler** (SwiftUI
   `Toggle(isOn: $isOn)`).
   - What you gain: removes the per-toggle `clicked` boilerplate while
     keeping state lifted — the declarative-family-consistent ergonomic
     upgrade of S2a.
   - Why deferred: it requires **two-way binding (state write-back)**,
     which M3 has no shipped surface for (§Dependencies) and which is a
     binding-direction feature in its own right — **implementing it now
     exceeds M3-Phase 8's responsibility** (A10 is "a *one-way* boolean
     binding drives an attribute", not "introduce two-way binding"). Not
     weighed as a buildable M3 form; reserved on the two-way-binding axis
     (§Forward-compat / §Out of scope). Choosing S2a does **not** foreclose
     it — two-way binding is added later beside S2a's type, opt-in
     (§Forward-compat (a)). *Note:* two-way binding alone does **not** make
     the tab band auto-exclusive — exclusion is still a group concern (S4 /
     Axis 2); it only removes the per-toggle write.
   - Technical risk: n/a in M3 (deferred); when revived, a two-way binding
     path end-to-end.
4. **S2c — WPF/Qt model: widget-owned self-toggling state** *(deferred —
   exceeds Phase 8, and a family-level (Vision-DR-scale) call).* The widget **owns** its
   `checked` state internally and **flips it itself** on click (WPF/Qt
   `ToggleButton.IsChecked`; HTML uncontrolled `<input type=checkbox>`).
   - What you gain: most ergonomic for the author — the widget manages its
     own on/off, with no external `state` required at all.
   - Why deferred: it needs **widget-owned mutable state**, which M3 does
     not have, plus (to be useful for exclusion / observation) a way to
     read that internal state back out — again write-back. Beyond cost, it
     **strains wasamo's current architectural *hypothesis***:
     tree-with-bindings keeps state *explicit and lifted* into `state`, not
     hidden inside widgets — widget-owned state is the imperative WPF/WinUI
     family
     ([architectural-family.md](../../../../docs/notes/architectural-family.md),
     a **live working hypothesis, not a ratified commitment**). So S2c is
     not merely "later"; adopting it would be a **family-level decision at
     Vision-DR scale**, which the owner retains full latitude to take — it
     is deferred here, not foreclosed. Reserved on the widget-owned-state
     axis (§Out of scope), kept for completeness so it is not silently
     dropped.
   - Technical risk: n/a in M3 (deferred); when revived, widget-owned state
     + a sync-out path, plus a family-fit re-evaluation.
5. **S3 — visual-theme-via-binding** (`style: accent` swapped
   conditionally, or `style: cond ? accent : default`) *(explicit
   reject).* Express selection by swapping a visual style.
   - What you gain: would reuse a styling surface rather than add a toggle
     attribute — *if one existed*.
   - What you give up / why rejected: the ternary form is **not in the
     expression grammar**; the only workaround (swap the whole widget via
     `if`) duplicates the handler per branch and leaves selection
     semantically unrepresented — fragile. Framing listed "theme-via-
     binding" as a candidate, so it is audited and dropped here rather
     than left implicit.
   - **Scope of the reject:** S3 is rejected as the *selection-authoring
     surface* for M3 — i.e. as a way to **write** "this is selected". It
     does **not** foreclose future **theme customization of selected
     visuals**, which stays open under the M5 theme surface
     (§Forward-compat (d) / §Out of scope). M3 selection is authored via
     the chosen `checked` (S2a) / `selected` (S1) attribute; how that
     attribute is *styled* later is an M5 concern, not closed here.
   - Technical risk: n/a (rejected).

S4–S6 are **out of scope for M3, each for a different reason**, and are
recorded in §Out of scope so they are not silently dropped:

- **S4 — group/parent widget** (`TabBar` / `RadioGroup` /
  `SegmentedControl`): the parent manages exclusion so the author writes
  no `==`. Out because it opens value write-back / selected value / child
  value / interaction semantics — **not** because of `==`. This is the
  *group-surface axis*.
- **S5 — single discriminant + equality** (`state tab: string = "all"` +
  `checked: tab == "all"`, i32 / string / enum): stalls on the missing
  `==`. This is the *equality-operator axis*.
- **S6 — data-driven tabs** (`for`-generated): blocked by M3's `for`
  constraints (handlers inside `for` deferred; `if` cannot read the
  binder/item; no index equality). Out under M3's existing `for` limits;
  noted because gallery assembly naturally raises it.

### Recommendation (Layer 1)

**S2a — `ToggleButton { checked: <bool> }` (controlled + one-way).** Of
the two forms real on M3's shipped surface, S2a names the toggle/select
purpose in the type — matching how the benchmark frameworks wasamo aligns
with model a selectable tab — while keeping exactly the one-way boolean-
binding mechanism A10 exists to demonstrate. S1 demonstrates the identical
mechanism (a binding drives an attribute); the difference is the surface,
not the binding. S1 is recorded as the **minimal alternative**: it adds no
new widget, and the owner may weight that minimality above S2a's
named-purpose / idiom fit. S2b (SwiftUI two-way) and S2c (WPF/Qt
widget-owned) are **deferred** — two-way binding and widget-owned state are
out of Phase 8's scope (S2c additionally **strains the current lifted-state
hypothesis**, Vision-DR-scale if revived) — not because they lack merit.
Recorded as **Proposed**: S2a is the recommended form. **The Accept is
two-stage:** (1) the load-bearing **S1 (attribute on Button) vs S2
(dedicated widget)** product-merit call here; and, if S2, (2) the
**concrete type/attribute lexeme** — recommended `ToggleButton { checked }`,
confirmed as the lighter second-stage pick in **SI-4**. **By default this
single Accept decides both** — the S1/S2 direction and the recommended
lexeme `ToggleButton { checked }`; only if the owner *explicitly* holds the
name does SI-4 stay an **Accepted-blocking unresolved sub-issue** until it
is fixed (§SI-4). Because
S2a **reverses** the framing/spike S1 recommendation, that call is an
**eyes-open reversal**, not a neutral pick — the cost and re-sync are
enumerated next.

### Accepted-time re-sync (this recommendation reverses the S1 lead)

S2a is a **reversal** of a prior owner-checked direction, and Accepting it
carries a re-sync obligation. Recorded so the owner Accepts with the full
cost visible:

- **What reverses:** framing packet C recommended the `selected: bool`
  *attribute* with **no new widget** (framing line 48; owner-checked ☑
  line 58); the stage-1 spike concluded **S1 as the lead, S2 as semantic
  comparison** (spike §結論). S2a adopts a **new `ToggleButton` widget**,
  reversing both, and directly contradicts packet C's "no new widget"
  property.
- **Why (owner intent):** the gallery is a public showcase meant to read
  as a credible UI framework; benchmark frameworks model a selectable
  tab/item as a *dedicated typed widget*, not a flag on a momentary Button,
  and naming the purpose in the type is the value being bought — owner
  call, this conversation 2026-06-27.
- **Re-sync targets at Accept (enumerated, not summarised away).** The
  concrete type/attribute lexeme used in these re-syncs is the **SI-4**
  pick (`ToggleButton` / `checked` as currently drafted):
  - `process/_roadmap.md` — A9 ("widget attribute state (Button
    selected)"), A10 (candidate list; already "concrete construct settled
    per phase"), A12 ("selected state surface") reworded to
    `ToggleButton` / `checked` where they name the concrete construct.
  - `framing.md` — packet C recommendation + checklist line 58, and the
    A1 feature-mapping table row ("タブ風セクション … `selected`" /
    "Button selected state") re-pointed to `ToggleButton` / `checked`.
  - the stage-1 spike's S1-lead conclusion noted as **superseded by this
    DD** (the spike stays an immutable record; the supersession is recorded
    here, not by editing the spike).
  - `docs/dsl_spec.md` / `docs/architecture.md` per §Spec impact.
- **If the owner instead keeps S1**, none of the above re-syncs: the
  framing/spike direction stands and `ToggleButton` is left un-consumed.

## Main decision B — driving-boolean production & exclusion (Layer 2)

Given the chosen controlled + one-way surface (S2a's `checked`, or S1's
`selected`), how is that boolean *produced*, and how is "exactly one
selected" expressed with shipped surface? This is **unchanged by the
Layer-1 choice** — the production / exclusion mechanism is identical
whether the boolean rides `ToggleButton { checked }` or `Button { selected
}`; only the attribute / type name in the examples differs. The spike
established that only two forms are real in M3 (α, β); the other two (γ,
δ) are not expressible without `==` and are deferred. This DD's open call
is **α vs β**, decided on product merit.

### Options

1. **α — handwritten block assignment** *(live-proven in the spike).*
   Each tab owns an independent `bool` state; each `clicked` sets its own
   `true` and the others `false` in one block. Observation is via
   conditional `if` (and, once the attribute lands, via `checked:` itself).
   - What you gain: **zero new Layer-2 surface** — it is the shipped
     single-boolean primitive applied N times; it shows the **exclusion
     *behaviour*** (one on, others off) live in the gallery tab band,
     faithful to the wireframe's tab strip. The spike ran this end-to-end
     on the runtime (click → block assign → reactive drain → exactly one
     marker subtree), with a negative positive-control confirming the
     assertion observes live state.
   - What you give up: **O(N²) handwritten assignment** (3 tabs → 9
     assignments); it grows and is brittle as tabs are added — an awkward
     shape for an author to write by hand (framing R8).
   - Technical risk: none beyond the Layer-1 surface's cross-layer change;
     the exclusion itself is already proven on the live path.
2. **β — single bool toggled by two buttons** *(minimal, compiles).* One
   `checked` state, a `Select` button → `true` and a `Clear` button →
   `false`; observe with `if checked` / `checked:`. The tab band stays a
   **static** highlight (exclusion not shown).
   - What you gain: minimal and clean; proves the binding-driven `checked`
     attribute with a two-frame positive control (marker appears on
     `Select`, disappears on `Clear`) without any O(N²) cost.
   - What you give up: it does **not** demonstrate exclusion at all; the
     gallery tab band becomes a static highlight, less faithful to the
     wireframe. (Note: an author writing a *single*-button self-toggle —
     `clicked => { x = !x }` — is **impossible**, since `!` is absent from
     the grammar, so β is a *two-button* on/off, corrected by the spike.
     This is distinct from S2c's *widget-owned* self-toggle (WPF/Qt model),
     deferred on the widget-owned-state axis.)
   - Technical risk: none beyond the Layer-1 surface's cross-layer change.

γ and δ are **not options for M3** — both require `==` to connect a
discriminant to a per-button `checked` boolean, and `==` is absent from
the grammar (spike §案 γ / §案 δ). They are deferred under the
equality-operator axis (§Out of scope), **not** placed in the M3
comparison, so unbuildable forms are not weighed against buildable ones.

### Recommendation (Layer 2)

**α — show exclusion live in the tab band — recommended, with β as the
documented live alternative.** On product merit, α delivers what A10 is
strongest demonstrating in the gallery: a boolean binding driving a
widget attribute *under a realistic multi-button exclusion*, matching the
wireframe's tab strip, and the spike already proved it works on the live
runtime path. β satisfies A10's core thesis too (binding drives an
attribute) but trades away the exclusion demonstration for minimality.

The owner merit call is therefore **"demonstrate exclusion behaviour in
the gallery (wireframe fidelity)" vs "avoid the O(N²) handwritten
assignment as an unnatural authoring shape" (framing R8).** A **third,
independent axis** weighs in because Phase 8 ships a *public* gallery:
whatever the example does becomes a **pattern future authors copy**, so
α's O(N²) one-true-others-false block is not merely awkward to write once —
it risks teaching an anti-pattern as the canonical way to express tab
exclusion. Two owner-facing ways discharge that risk: **(α + a provisional
note)** — ship the live exclusion but mark it in the spec / gallery as the
*M3-era* shape, pointing forward to the future discriminant form (Axis 1)
so readers do not read O(N²) as the intended long-term idiom. This note is
**not self-discharging in DD-001**: the DD-002 / `preamble.md` skeleton
(§Couples-to gate) must carry its concrete form — **who authors the
public-draft note, how strong the gallery comment / spec note is, and the
migration trigger to the future discriminant form** — so the owner can
confirm the mitigation is real before Accepting α. Or **(β +
static approximation)** — prove `checked` minimally with the two-button
toggle and render the tab band as a static highlight, recording the
static-approximation accounting in the A1 table / plan (SI-2). α is
recommended because the exclusion behaviour is the more informative thing
to show and is already de-risked, **and the teaching-risk is mitigable by
the provisional note** rather than by dropping the demonstration; β is the
retreat if the owner judges 3×3 handwritten assignment too
brittle/unnatural to be the first public example. Recorded as
**Proposed**; this is exactly the product-merit choice the owner is
invited to make at the Accepted flip.

A **fourth path — make exclusion *easy* by adding `==` (a single
discriminant + `checked: tab == value`, O(N) and intrinsically
exclusive)** — would most directly fix α's teaching-risk, so it is weighed
as an explicit **scope-expanding option**, not dismissed as merely
unimplemented. Why it is held to a later phase (and *not* on FD-8-A's
clause, which forbids a new **layout** primitive — framing line 46 — and so
does not by itself cover an expression operator): an equality operator is a
**new expression-grammar / IR / checker / spec feature**, whose cost spans —
(i) lexer + `HandlerExpr` / `if`-condition grammar (today `BOOL_LIT |
IDENT`, no operators); (ii) IR `CompoundOp` (today `Add/Sub/Mul/Div`) + a
comparison node; (iii) checker typing and **typed comparison semantics**
for the discriminant's i32 / string / enum-like operands — plausibly a
*narrow per-type* comparison rather than a generic `TypedValue`, but still
expression-typing + public-spec work;
(iv) a new **diagnostics** surface (type-mismatch / non-comparable
operands); and (v) **a new public promise in the A12 draft** — `==` would
ship as author-facing surface that DD-002 must then position. That is a
binding/expression-language expansion landing in **M3's final,
draft-publishing phase**, with a non-trivial plan revision, so it is held to
its own later phase (Axis 1's revisit trigger), not bolted on here to
sidestep the α/β call. The owner may overrule this on product-merit grounds
— but as an **eyes-open M3-scope expansion** with the cost above visible,
not a feasibility default.

## Sub-issues

### SI-1 — Minimal visual pass line *(owner-presented; not pre-decided)*

M3 selected visuals are minimal, but the *specific* minimal form must be
**shown to the owner**, not chosen on syntax alone
([spec.md](../../requirements/spec.md) Out-of-scope §Visual: "minimal,
undecorated; concrete form pinned in phase pre-doc"). The candidates:

- **V-a — background colour only** (selected cell gets a fill).
- **V-b — border only** (selected cell gets an outline).
- **V-c — colour + border** (both).

Recommendation noted but **deferred to the owner viewing a rendered
candidate**: V-c (colour + border) reads least ambiguously for the
§Verification two-frame positive control (a wrong static frame is hardest
to mistake for the real toggle when two cues move together); V-a is the
most minimal.

**What this DD fixes vs. what the implementation checkpoint fixes.** A
rendered selected visual does **not** exist before the attribute is
implemented — the spike's stage 1 explicitly does *not* verify selected
visuals (they are stage 2, after A10 lands). So the pass line is **not** an
Accepted-flip condition for this DD (drafting it from prose, or building a
throwaway mock just to flip the DD, would be the wrong order). This DD
fixes the **candidate set** (V-a / V-b / V-c) and the **judgement
criterion** (the chosen visual must be distinguishable across the
two-frame positive control, not a static look-alike). The **final pick**
is confirmed at the implementation-plan **owner checkpoint** — spike
stage 2 / framing FD-8-G(3), where the owner sees the actual rendered
candidate after the attribute lands. The plan records the pick; this DD
Accepts with the candidate set and criterion settled.

### SI-2 — Application target: tabs vs thumbnail highlight

The wireframe carries selected-state in **two** places: the tab band
("All / Albums / Favorites") and a **highlighted thumbnail** (row 2,
col 3). This DD must say which the binding-driven `checked` proof covers.

- Recommendation: the **tab band is the A10 binding-driven surface**; the
  highlighted thumbnail is rendered as a **static minimal approximation**
  in M3 (not a second `checked`-driven instance), deferring real
  thumbnail *selection interaction* (hit-testing / focus) to M4. This
  keeps A10's proof to one surface and avoids scope creep.
- This is an initial hypothesis: per framing FD-8-G(1) the wireframe-
  fidelity / placeholder agreement updates the A1 feature-mapping table,
  and may revise this assignment in the implementation plan.
- **If Layer-2 β is chosen**, the tab band shows a *static* highlight (no
  live exclusion), so A10's binding-driven exclusion proof rests on the
  two-button toggle, **not** the tab strip. How that is accounted — i.e.
  that the tab-band exclusion is a **static approximation** and A10's
  demonstration is discharged by the β toggle — must be **recorded in the
  A1 feature-mapping table / plan** at the FD-8-G(1) checkpoint, the same
  place the tab/thumbnail assignment is fixed. (Under α the tab band
  carries the live exclusion and no such note is needed.)

### SI-3 — Diagnostics (`checked` admission)

`checked` is admitted on `ToggleButton` (under S2a) and **rejected on
widgets that do not support it** (e.g. `Button { checked: … }`,
`Text { checked: … }`), as a named check error with a firing test. (Under
the S1 alternative the same trap applies to `selected` on Button.) This
falls out of the chosen surface and is the authored-branch evidence
(impl-gates trap #4); detail in §Spec impact.

### SI-4 — Lexical naming of the toggle surface *(owner-presented; not pre-decided; S2-general)*

§Main decision A and the rest of this DD use `ToggleButton` / `checked` as
the **recommended lexeme**, not an already-settled name: it is confirmed as
the **second-stage Accept call** (§Recommendation (Layer 1)), separable from
the Layer-1 S1-vs-S2 decision. The concrete **type name** and **attribute
name** do **not** change the Layer-1 product-merit call (attribute-on-Button
S1 vs dedicated widget S2): the type-name half becomes live **only once a
dedicated widget (any S2 model) is chosen**, and is **common to S2a / S2b /
S2c**, not specific to the recommended S2a binding model. The owner may
adopt S2 while deferring the exact lexeme to this sub-issue.

- **Type name** (S2 only): `ToggleButton` / `SelectableButton` /
  `TabButton` / a `ChoiceChip`-style name — each leans the surface toward
  *toggle* vs *selection* vs *tab/group* reading, so the pick interacts
  with the reserved group-surface axis (S4 / Axis 2) and is worth an
  explicit owner look rather than inheriting `ToggleButton` by default.
- **Attribute name** (S1 or S2): `checked` / `selected` / `on`. Under the
  S1 alternative only this half applies — Button gains the attribute and
  there is no new type to name.

This is recorded as a sub-issue, **not** modelled as extra Layer-1
options. Splitting each candidate name into its own top-level option (e.g.
`SelectableButton { selected }`, `TabButton { selected }`,
`Button { role: tab selected }`) was **considered and rejected**: it
re-entangles the lexeme with the binding-model and role semantics the
S2a/S2b/S2c split already separates cleanly, and inflates the option set
the owner must navigate against the over-engineering brake. **By default
the lexeme is fixed at this Accept** (the recommended `ToggleButton` /
`checked`), flowing into §Accepted-time re-sync and §Spec impact. **If the
owner explicitly defers the name**, this sub-issue is carried as
**Accepted-blocking unresolved** — the DD is not fully Accepted on the
naming axis and the re-sync / spec wording waits on it — rather than being
silently settled later in the implementation plan the way SI-1's visual
pick is.

## Forward-compat impact

`ToggleButton { checked: bool }` (controlled + one-way, the **S2a / Compose
model**) is M3's **minimal** selected/toggle surface, not "the one and only
selection model forever". The richer models from §Main decision A stay
reserved, on different triggers.

**What "extension" means here (an honest note).** The three models are
**per-instance mutually exclusive**: a given toggle is controlled (S2a),
*or* two-way (S2b), *or* self-toggling (S2c) — never several at once. So
"add S2b/S2c later" is **not** "make S2a's widget also do them" in the same
instance; it takes one of **two distinct shapes, and this DD does not
decide which**:

- **(i) same type, mode selected by binding** — `checked` gains an opt-in
  two-way / uncontrolled form on the *same* `ToggleButton`, the per-instance
  wiring picking the mode (precedent: WPF `IsChecked` OneWay/TwoWay/unbound;
  HTML/React controlled-vs-uncontrolled). Genuinely additive on the type.
- **(ii) a separate named widget** — a distinct widget bakes in the other
  model (precedent: Compose and SwiftUI each bake one model into their
  toggle type's API). This is **not** additive on `ToggleButton`; it is a
  *new widget added beside it*.

Which shape wasamo takes is an undecided design fork. **This qualifies
S2a's "type-home for growth" benefit:** it pays off only under (i); under
(ii) the future modes are new widgets and S2a merely consumed the
`ToggleButton` name for the controlled-only variant — a cost S1 avoids. The
reservations below therefore say *that* a model is reserved and on what
trigger, **not** that it will necessarily live inside `ToggleButton`.

- (a) **S2b (SwiftUI model) — two-way binding stays open, *conditionally
  additive*.** The `checked` attribute is one-way in M3; a future two-way
  binding to the lifted state can arrive as shape (i) (an opt-in two-way
  form on `checked`) or shape (ii) (a separate widget). **If (i), there is
  a design condition for it to be genuinely additive:** the two-way form
  must be **opt-in** (e.g. a distinct binding sigil) and must **leave the
  M3 controlled contract unchanged** — plain `checked: <state>` stays
  one-way and the author's `clicked` handler stays the thing that writes
  state. If `checked` instead
  silently became writable-on-click, it would **conflict** with M3's
  `clicked => { all = true; albums = false; … }` code. This DD does **not**
  pre-design that opt-in syntax; it records the condition. **Backstop:**
  wasamo is **pre-1.0** (BDFL + ADRs,
  [governance-rfc-deferral.md](../../../cross-milestone/decisions/governance-rfc-deferral.md)),
  so if the condition cannot be met cleanly a **breaking revision of the
  `ToggleButton` contract is permitted pre-1.0** — the owner Accepts S2a
  knowing the name/contract is fixed now and *may* cost a pre-1.0 breaking
  revision later.
- (b) **S2c (WPF/Qt model) — widget-owned self-toggle stays open but is a
  *family-level (Vision-DR) call*.** A self-toggling widget that owns its
  own `checked` could be added later, but it imports widget-owned mutable
  state, which is the imperative WPF/WinUI family, **not** the lifted-state
  tree-with-bindings shape wasamo *currently* sits in
  ([architectural-family.md](../../../../docs/notes/architectural-family.md)
  — a live hypothesis, not a ratified direction). So S2c is reserved as a
  *family-level* option, not a routine additive one: choosing S2a's
  controlled form now is the hypothesis-consistent default and does not
  foreclose S2c, but adopting S2c would be a deliberate family-level shift
  the owner takes with the design force visible (Vision-DR scale).
- (c) **Group semantics (S4) stay free to design.** Exclusion is expressed
  in M3 only with shipped surface (α handwritten assignment, or β's
  minimization), leaving future `RadioGroup` / `TabBar` /
  `SegmentedControl` parents (the natural home of self-toggling + group
  exclusion) free.
- (d) M3 selected visuals are provisional and can be absorbed/overridden by
  the M5 theme surface.
- (e) accessibility / focus / input semantics are re-designable in M4+.

The thing M3 *consumes* is a **new dedicated widget name** (the SI-4 lexeme,
`ToggleButton` recommended) **+ its initial (controlled, one-way)
contract**. The later path is **one of the two shapes
above** — same-type opt-in (i) or a separate widget (ii) — and which one is
undecided; under (i) it can stay non-breaking subject to the opt-in
condition, with pre-1.0 breaking revision as the honest backstop, while
under (ii) the new model simply lives in a new widget and the `ToggleButton`
name remains controlled-only. Either way this is the **owner-owned forward
cost** of S2a that S1 avoids (S1 leaves `ToggleButton` un-consumed). None of
(a)–(e) is built here; they are reservation conditions, and the §Out of
scope rows hold their triggers.

## Spec impact

`docs/dsl_spec.md` (author-facing, external-reader bar, **no DD/option
labels** per the living-spec vocabulary rule; provenance via ADR hyperlink
only):

- **`ToggleButton` with a `checked` attribute** (the SI-4 lexeme —
  `ToggleButton` / `checked` recommended; under the S1 alternative,
  `selected` on Button) — a boolean attribute, driven by the
  existing one-way boolean binding, with the chosen minimal visual (SI-1).
  Stated as a widget attribute distinct from placement (`slot.*`) and from
  intrinsic text/enabled props; the toggle is **controlled** (the click →
  value → state write is the author's handler), with the two-way (S2b) and
  widget-owned (S2c) models noted as out of scope.
- **Admission / rejection table (forcing artifact, not summarised away):**
  `checked` is admitted on `ToggleButton`; on any other widget it is a
  **named check error**, re-checked by the loader, each with a firing test.
  A paired accept/reject fixture pins the checked-attr-vs-unknown-prop
  distinction in both directions.
- **Exclusion expression** — documented as a *composition of boolean
  states* (the chosen Layer-2 form), with no claim that the language has a
  selection/group construct. If the owner picks α, the spec shows the
  handwritten one-true-others-false pattern as the M3 way to express tab
  exclusion; if β, the single-bool two-button toggle. The fact that
  "exactly one selected" is **author-composed, not a built-in** is stated
  honestly (it bears on DD-002's public-draft positioning of the deferred
  group surface).
- Stale prose swept in the same touch. Selected/toggle visuals documented
  as **minimal / provisional**, pointing forward to the M5 theme surface.

`docs/architecture.md`: the new `ToggleButton` node and its `checked`
attribute's representation through lower / IR / runtime loader / widget
visual, consistent with the existing single-boolean binding model (no new
binding-target class, no new measure/arrange — it reuses Button's leaf
layout).

## Risk mitigation

- **The toggle surface is a cross-cutting change (framing R6).** It crosses
  parser → check → lower → IR emit → runtime loader → widget visual →
  cross-host parity. Under S2a it also adds a **new `ToggleButton` widget
  node** (wider than S1's attribute-only change, narrower than a new layout
  primitive — it reuses Button's leaf measure/arrange). Beyond the impl-
  gates call-site audit table (trap #1), the **checked-propagation audit**
  is the central A10 evidence, pinned by firing tests / positive controls:
  (i) **`checked` on a non-supporting widget rejects**, (ii) **a bool-
  binding change reaches the visual**, (iii) **C / Rust / Zig render the
  same** (cross-host parity).
- **Over-build guard (framing R3).** No two-way binding, no widget-owned
  state, no dedicated group widget, and no full theme are built; the toggle
  stays controlled + one-way, visuals stay minimal (SI-1), and exclusion
  stays author-composed. S2b / S2c / S4 are kept on the table as
  *reservations*, not built.
- **Demonstration-vehicle feasibility (framing R8).** Whether the tab-band
  exclusion (α) is too heavy/unnatural was the explicit reason for the
  pre-DD spike, which proved α real on the live path and fixed β as the
  retreat. This DD therefore does not gamble on an unbuildable vehicle.
- **Positive control (AGENTS.md §Testing rules).** A single static frame a
  wrong implementation could equally produce is **not** evidence: the
  selected visual must be shown changing across a **two-frame** toggle
  (and, under α, the exclusion — one on, others off — in the same two
  frames). This is the same positive-control shape as Phase 6 show/hide
  and Phase 7 iteration. Assistant evidence = launch + DPI-aware
  screenshot + analysis; owner human-visible smoke is a separate gate.

## Out of scope (deferral axes — kept distinct)

The spike and the Layer-1 analysis establish that the not-chosen
candidates split into **axes that revive on different triggers**; this DD
records them as **separate items**, not bundled.

- **Axis 1 — equality-operator family (`==`-family): γ + δ.** A single
  discriminant state (i32 / string / enum-like) with `checked: tab ==
  value`. Not buildable in M3 because `==` is absent from the expression
  grammar (and `if` cannot derive a per-button bool from an index without
  it). **Re-visit trigger:** an equality operator `==` enters the
  expression grammar. **Revived form:** one discriminant state + `checked:
  tab == value` — O(N), single assignment, intrinsically exclusive —
  replacing α's O(N²) handwritten assignment. Whether `examples/gallery/`
  is later migrated from α to a discriminant form is an **independent**
  decision for the `==` phase, not implied here.
- **Axis 2 — group-surface family: S4.** Parent/group widgets
  (`RadioGroup` / `TabBar` / `SegmentedControl`) that manage exclusion so
  the author writes no `==`. **Re-visit trigger:** **not `==`** — rather
  value write-back / selected value / child value / interaction semantics.
  **Revived form:** the parent manages exclusion; the author writes no
  comparison. Heavier than the equality-operator axis, and the natural
  home for the toggle/select role plus group exclusion. Do **not** fold
  this into Axis 1 (spike owner-review correction).
- **Axis 3 — two-way binding (SwiftUI model): S2b.** `checked` bound
  two-way to the lifted state, so a click writes the bound state without a
  handler (SwiftUI `Toggle(isOn: $x)`). **Re-visit trigger:** two-way
  binding (state write-back) enters the binding grammar. **Revived form:**
  an opt-in two-way binding on `checked` — either on the same `ToggleButton`
  (shape (i)) or as a separate widget (shape (ii)); **undecided**
  (§Forward-compat). Distinct from Axis 1 (`==`) and Axis 2 (group): this
  axis is about binding *direction*, and keeps state lifted
  (family-consistent). Note: two-way binding alone does not auto-exclude the
  tab band (that is Axis 2).
- **Axis 4 — widget-owned state (WPF/Qt model): S2c.** A self-toggling
  widget that owns its `checked` internally and flips it on click.
  **Re-visit trigger:** **not a binding feature** — a *family-level*
  decision (Vision-DR scale) to admit widget-owned mutable state (the
  imperative WPF/WinUI family), which the tree-with-bindings shape wasamo
  *currently* sits in avoids
  ([architectural-family.md](../../../../docs/notes/architectural-family.md)
  — a live working hypothesis the owner may revisit at any phase boundary,
  not a ratified commitment). **Revived form:** widget-owned `checked` + a
  sync-out path (as shape (i) uncontrolled mode or shape (ii) a separate
  widget — §Forward-compat); a deliberate family-level shift, not a routine
  extension. Kept distinct from Axis 3 — S2b stays hypothesis-consistent,
  S2c would re-open the family question.

Also out of M3 scope (existing triggers hold):

- **Full theme / rich selected visuals** — M5 (the component theme
  surface; M3 visuals are minimal, SI-1).
- **Accessibility / focus / input semantics for selection** — M4+.
- **Real thumbnail selection (hit-testing / focus / gesture)** — M4; M3
  renders the highlighted thumbnail as a static minimal approximation
  (SI-2).
- **Data-driven tabs (`for`-generated, S6)** — blocked by M3's `for`
  constraints; lands when `for`-internal handlers / binder-reading `if`
  are designed.

## Revision history

- 2026-06-26 — Initial draft (Status: Proposed). Two-layer structure from
  the stage-1 feasibility spike: Layer-1 authoring form and Layer-2
  driving-boolean production (α recommended with β as the live
  alternative, γ/δ deferred). Minimal visual pass line recorded as an
  owner-presented sub-issue (SI-1); application target (SI-2) and
  diagnostics (SI-3) sub-issues.
- 2026-06-26 — Codex review folds (Status: Proposed; no recommendation
  reversed). SI-1: moved the visual pass-line **off the Accepted-flip
  condition** onto the implementation-plan owner checkpoint (spike stage 2
  / FD-8-G(3)), since a rendered selected visual does not exist before the
  attribute lands; the DD now fixes only the candidate set + judgement
  criterion. S3: added a **scope-of-reject** note — S3 is rejected as the
  M3 *selection-authoring* surface only; future theme customization of
  selected visuals stays open under M5 (consistent with framing.md §66 and
  §Forward-compat (d)).
- 2026-06-27 — Layer-1 recommendation set to **S2a — `ToggleButton {
  checked }` (controlled + one-way)**, with **S1 — `Button { selected }`**
  recast as the minimal alternative and **S2b** (self-toggling + two-way)
  split out and deferred (two-way binding is out of Phase 8's scope). The
  S1/S2 comparison is framed on two axes (Axis A click-behaviour; Axis B
  binding-direction); §Forward-compat and §Out of scope rewritten on those
  axes, with write-back / self-toggling reserved as Axis 3. Decision B
  (Layer 2 α/β) unchanged except the attribute / type naming in the
  examples (`selected` / Button → `checked` / `ToggleButton`); the
  production / exclusion mechanism is identical under either Layer-1
  surface. Status remains Proposed.
- 2026-06-27 — Codex re-review folds (Status: Proposed; recommendation
  unchanged). Named the S2a recommendation as a **reversal** of the
  framing packet C (`selected: bool`, no new widget; owner-checked) and the
  spike's S1-lead conclusion, and added **§Accepted-time re-sync**
  enumerating the roadmap (A9/A10/A12) / framing (packet C + A1 table) /
  spike / spec re-sync targets (Finding 1). Narrowed S2a's stated benefit
  to **type-naming only**, clarifying that the benchmark *controlled*
  toggles (Compose `Switch` / `onCheckedChange`) are themselves **one-way
  controlled** — the same cell as S2a, lending no two-way advantage — while
  genuine two-way (SwiftUI `isOn: $x`) / self-toggle is S2b, and restating
  the real S1-vs-S2a delta (Finding 3). Axis A re-stated as "who *writes*
  the boolean after a click" so a value-carrying callback does not read as
  self-toggling.
- 2026-06-27 — Split the dedicated-toggle-widget option into **three named
  models** at owner request: **S2a (Compose** — controlled, one-way, lifted
  state; recommended**)**, **S2b (SwiftUI** — two-way binding to lifted
  state; deferred**)**, **S2c (WPF/Qt** — widget-owned self-toggling state;
  deferred and *against the lifted-state family***)**. Decision A lists all
  three; §Out of scope splits the former write-back axis into **Axis 3**
  (S2b — two-way binding, family-consistent) and **Axis 4** (S2c —
  widget-owned state, a family-level shift); §Forward-compat (a)/(b)
  relabelled accordingly. Recommendation (S2a) and Decision B unchanged.
  Status remains Proposed.
- 2026-06-27 — Owner forward-compat critique fold (Status: Proposed;
  recommendation unchanged). Corrected the overclaim that S2b/S2c are
  "additive on the same `ToggleButton` type": the three models are
  **per-instance mutually exclusive**, and "extension" takes one of two
  **undecided** shapes — (i) same type, mode selected by binding (WPF/HTML
  precedent) or (ii) a separate named widget (Compose/SwiftUI precedent).
  §Forward-compat now states this fork; S2a's "type-home for growth"
  benefit is marked **contingent** (pays off only under (i)); §Out of scope
  Axis 3/4 "revived form" no longer asserts same-type. Qualified
  §Forward-compat (a): future two-way / self-toggle is additive **only
  under an opt-in condition** preserving the M3 controlled contract, with
  pre-1.0 breaking revision as the honest backstop, and named the
  name/contract consumption as the owner-owned forward cost (Finding 2).
  SI-2: added the β-fallback accounting condition (record the static
  approximation in the A1 table / plan).
- 2026-06-27 — Strategic / owner-alignment review folds (Status: Proposed;
  recommendation unchanged). **Finding 1** folded as a *restructure*:
  type/attribute **lexical naming extracted to SI-4** — an owner-presented,
  S2-general sub-issue (common to S2a/S2b/S2c, not S2a-specific), with
  `ToggleButton` / `checked` demoted to running placeholders; the review's
  alternative remedy (modelling each candidate name as a separate Layer-1
  option) is **rejected** as re-entangling lexeme with binding-model/role
  against the over-engineering brake. **Finding 2** folded: the `==` /
  discriminant path is now rejected as a **deliberate M3-close scope
  decision** (FD-8-A, no new primitive), not a feasibility default (Layer-2
  recommendation + Axis 1). **Finding 3** folded: added the **public-example
  teaching-risk axis** to the α/β call with two owner mitigations (α + a
  provisional spec/gallery note; or β + SI-2 static approximation).
  **Finding 4** folded: S2c "against the family" softened to a
  **family-level (Vision-DR-scale) call** straining a *live hypothesis*,
  owner latitude preserved (option S2c, §Forward-compat (b), Axis 4).
  **Finding 5 deferred**: the DD-002 / `preamble.md` skeletons are drafted
  in parallel per the Phase-8 plan, and §Couples-to now states DD-001's
  Accept depends on at least those skeletons existing — not a DD-001
  design-space gap. Status remains Proposed.
- 2026-06-27 — Codex re-review folds (Status: Proposed; recommendation
  unchanged). **Finding 1**: made the Accept **two-stage** (S1-vs-S2 first;
  SI-4 lexeme second) and reframed SI-4 / Spec impact / Forward-compat from
  `ToggleButton` / `checked` as *placeholder* to the **recommended lexeme
  confirmed at Accept**, so no other section silently consumes the name.
  **Finding 2**: the `==` rejection no longer leans on FD-8-A's
  *layout*-primitive clause — replaced with an explicit **scope-expanding
  cost frame** (grammar / IR `CompoundOp` / checker `TypedValue` equality /
  diagnostics / new A12 public promise / plan-revision weight),
  owner-overridable. **Finding 3**: α's teaching-risk note now names the
  concrete DD-002 / `preamble.md` responsibilities it depends on (note
  authorship, gallery/spec note strength, discriminant-migration trigger).
  **Finding 4**: softened the stale "against the lifted-state family" in
  §Recommendation to "strains the current hypothesis; Vision-DR-scale if
  revived". Status remains Proposed.
- 2026-06-27 — Codex re-review (3rd pass) folds (Status: Proposed;
  recommendation unchanged). **Finding 1**: removed the SI-4 double-read
  ("adopt S2, name later" vs "fixed at Accept") — the rule is now **one
  call decides S1/S2 + the recommended lexeme by default**, and only an
  *explicit* owner hold leaves SI-4 an **Accepted-blocking unresolved
  sub-issue** (harmonised §Recommendation (Layer 1) ↔ §SI-4). **Finding 2**:
  softened the `==` cost item from "`TypedValue` equality semantics" to
  "typed comparison semantics (plausibly narrow per-type, not necessarily a
  generic `TypedValue`)" so the scope-expansion compares fairly. **Finding
  3** is an Accept-gate (DD-002 / `preamble.md` skeleton), not a DD-001
  edit. Status remains Proposed.
