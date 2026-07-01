---
title: Toggle / selected-state control surface
status: Proposed
phase: M3-Phase 8
ac: A10 (existing) — selected-state is reserved by A10 from the start (unlike 7b's newly-minted A13), so no new AC is expected; discharges under A10 + A11 + A12. Any new public-promise AC would come from DD-002, not this DD. Confirmed at the Accepted flip (framing FD-8-F).
date: 2026-06-28
related:
  - ./preamble.md
  - ./dd-m3-p8-002-dsl-spec-public-draft-promotion.md
  - ../requirements/framing.md
  - ../requirements/constraints.md
  - ../requirements/dd-001-stage1-spike.md
---

# DD-M3-P8-001 — Toggle / selected-state control surface

**Status:** Proposed

## Context

On what **control surface** does an author write a **persistent toggle /
selected** state in `.ui`?

A10 is the last new authoring surface M3 delivers: it shows that a
**boolean binding (Phase 1) can drive a widget *attribute***, not just a
widget's text or its presence under an `if`. The Photo Gallery's tab band
(several buttons, one selected) is the candidate place to exercise it
([gallery-wireframe.html](../../requirements/gallery-wireframe.html)). A
toggle/selected state appears in neither the spec nor the gallery today,
so this DD defines the surface for the first time.

**Why the decision was re-opened (under-specified Button option).** An
earlier pass put the load-bearing choice as *"a boolean attribute on the
generic `Button`"* (call it the "S1" reading) **vs** *"a dedicated toggle
widget"*. That framing was **under-specified on the Button side**: a bare
`Button { selected: bool }` reads as if **every** `Button` can carry a
selected/checked state, leaking persistent toggle semantics onto ordinary
action buttons. The option left unexamined is a **Button *capability*
surface** — a `Button` that holds a persistent `checked` state **only when
it is declared `checkable`** (the Slint / Qt model). Specifying the Button
side properly changes the product-merit comparison against a dedicated
widget materially, so the load-bearing decision is recast as a **control
taxonomy** and the comparison redone.

The question therefore splits into **three orthogonal axes**, decided in
order:

- **Main decision A — Control taxonomy (Layer 1).** *Where the persistent
  toggle capability lives:* a bare attribute on every Button (**B1**), a
  Button *capability* (**B2**), a dedicated toggle widget (**T1**), or a
  generic Toggle whose *appearance* is selected (**G1**). This is the
  load-bearing decision.
- **Main decision B — State ownership & write model (Layer 2).** *Who
  writes the state after a click:* controlled + one-way author code
  (**W1**), a two-way binding (**W2**), or the widget itself (**W3**).
  Independent of A.
- **Main decision C — Driving boolean & exclusion (Layer 3).** Given a
  controlled boolean, *how it is produced and narrowed to "exactly one
  selected"* with shipped surface (the spike's α / β; γ / δ deferred).

**Settled floor (not re-litigated).** Three things are fixed by the framing
and the stage-1 spike and are *not* reopened here:

- The driving value is a **boolean riding the existing boolean binding** —
  packet C, owner-aligned 2026-06-25; this DD does **not** re-decide whether
  to use a boolean at all. *Which control surface carries it* is the
  load-bearing choice (Main decision A).
- Selected-state visuals are **minimal** in M3 (a colour/border
  difference), with the full theme surface owned by M5
  ([spec.md](../../requirements/spec.md) Out-of-scope §Visual).
- No new layout primitive and no new measure/arrange is introduced
  (framing §再検討しない前提; constraints §1). A dedicated toggle widget
  (T1) or a checkable Button (B2) reuses Button's existing leaf
  measure/arrange; it is a new *node* / a new *capability*, not a new layout
  primitive.

Per the owner prior, every option below is compared on **product merit /
thesis fit first**; revision cost is a tie-breaker, never a rejection
ground, and the over-engineering brake stays in force.

## Dependencies

- **Consumes** framing FD-8-A (M3-closing thesis: A10 + A1 + A12, no new
  primitive), FD-8-B (two-DD slate), FD-8-C (boolean-on-existing-binding
  direction; exclusion feasibility settled by the pre-DD spike; the carrying
  *control surface* left open for this DD per framing §66), FD-8-E
  (implement-not-docs scope), and FD-8-F (no new AC expected unless DD-002
  adds a public promise).
- **Fed by** the stage-1 feasibility spike
  ([../requirements/dd-001-stage1-spike.md](../requirements/dd-001-stage1-spike.md)),
  the authority for which Layer-3 options are *real* on shipped surface. The
  spike compiled each candidate (`wasamoc check` / `build`) and ran the
  adopted candidate (α) on the live runtime path. This DD cites those facts,
  it does not re-derive them.
- **Couples to** [DD-M3-P8-002](./dd-m3-p8-002-dsl-spec-public-draft-promotion.md)
  only at the documentation seam: DD-002 positions any deferred selection
  surface (the Out-of-scope axes below) in the public draft. DD-002 does not
  decide this DD's control surface. Because that positioning is where the
  owner sees how the deferred (non-foreclosed) axes read as *public contract*, and because
  this DD's α recommendation (Main decision C) **leans on** that positioning
  as its teaching-risk mitigation, **DD-001 should not Accept before DD-002
  carries the α-mitigation items in *concrete, inspectable form*** — not
  merely a skeleton that names them. DD-002 §DD-001 coupling must hold,
  drafted (recommendations pending DD-002's own Accept): the public-draft
  note authorship, its wording strength, the `==`-migration forward pointer,
  and the deferred-axis representation. (DD-002's broader policy options
  A/B/C may stay skeletal; only the coupling items must be concrete.)
- **Shipped-surface facts that bound the space** (spike §確定した事実, not
  re-litigated): the expression grammar has **no `==`** (`HandlerExpr` /
  `CompoundOp` = `Add/Sub/Mul/Div` only,
  [wasamo-ir/src/lib.rs](../../../../wasamo-ir/src/lib.rs)); `if` conditions
  are `BOOL_LIT | IDENT` resolving to a `bool` state, **no operators**;
  handlers are block assignments only; and there is **no public API for the
  host (or a widget) to write component state while it is displayed**
  ([host-state-boundary.md](../../../../docs/notes/host-state-boundary.md)).
  Together these mean exclusion can only be expressed as a *combination of
  boolean states* (Main decision C), and they are what put **two-way
  binding** (W2) and **widget-owned mutable state** (W3) out of M3's reach.

## Main decision A — Control taxonomy (Layer 1)

The load-bearing decision: on what control surface a persistent toggle/
selected state lives. Four options, named by *where the capability lives*.

**The dispute, stated narrowly.** This is *not* whether wasamo adopts a
capability-typed component system in general. The narrower question is whether
wasamo's **author-facing button abstraction carries an optional checkability**.
B2 answers *yes* — a button-looking binary toggle is spelled as a **checkable
`Button`**. T1 answers *no* — persistent `checked` state is the responsibility
of a dedicated toggle control, so a button-looking toggle is spelled as a
**`ToggleButton`**. This is a judgement about the **author-facing surface /
concept vocabulary**, not about internal inheritance or shared-component
structure: T1 may still share a `ButtonBase`-style common path internally, and
B2 does **not** imply treating Switch / CheckBox / RadioButton / Tab / Picker /
SegmentedControl as Button modes. Under either choice, future Switch / CheckBox
/ RadioButton / Segment / Tab can be added as their own typed surfaces.

> **Axis A is *type structure*, independent of the Main-decision-B *write
> model*.** A given framework can sit on a point of *each* axis: WPF's
> `ToggleButton` is a *dedicated type* (T1 on axis A) **and** self-toggling
> (W3 on axis B); SwiftUI's `Toggle` is an *appearance-variant* control (G1)
> **and** two-way-bound (W2). The split is deliberate: it lets wasamo pick a
> control surface (A) and a write model (B) **separately** — and, in M3,
> wasamo's only buildable write model is the controlled W1 (§Main decision
> B), whatever A it picks.

### Options

1. **B1 — generic Button attribute.** `Button { selected: bool }` (or
   `Button { checked: bool }`) — a bare boolean on the **ordinary** Button.
   *(Strongly recommended reject — not finalized.)*
   - What you gain: the smallest possible surface; one attribute on an
     existing widget.
   - What you give up / why reject: it reads as if **every** `Button` can be
     selected/checked, leaking persistent toggle state onto momentary action
     buttons that should hold none. The select/toggle role lives only in
     author convention, with no surface marking *which* buttons are
     stateful. This is the under-specified reading that re-opened the
     decision; it is the form packet C originally recommended, and the
     properly-specified Button option is **B2**, not B1.
   - Technical risk: n/a (recommended reject).
2. **B2 — Button mode / capability.** `Button { checkable: true; checked:
   <bool> }` — an ordinary `Button` holds **no** persistent state; only a
   Button **declared with the toggle capability** carries a persistent
   `checked`. *(Live contender — co-equal with T1.)* (The capability's
   spelling — a boolean flag `checkable: true` (shown) vs a `mode: checkable`
   enum — is **SI-4 CF-1 / CF-2**; CF-1 is the illustration here, and CF-2
   carries extra cost — §B2-cost note.)
   - What you gain: the **smallest author surface** that closes B1's leak —
     **one widget type** (no catalog growth), reusing the existing Button's
     look, press behaviour, and implementation; the stateful-ness is
     **explicit at the use site** (`checkable: true`), without a new node. It
     is **not** an unusual shape: it has clear precedent in **Qt**
     (`QAbstractButton { checkable; checked }`), **Slint** (`Button { checkable;
     checked }`), and **ARIA** (`button` role + `aria-pressed` — a state
     annotation, so a weaker analogue than a component API). That lineage is
     clear, but it is a **minority** one and not, on its own, evidence of
     wasamo's preferred future direction
     ([architectural-family.md](../../../../docs/notes/architectural-family.md)).
   - What you give up / cost: a **cross-attribute dependency** the checker
     must specify and enforce — `checked` is meaningful **only** under
     `checkable: true` (reject otherwise? default? — **SI-5**). wasamo has
     not had an attribute whose validity depends on another attribute's
     value; this is new checker work and a new public rule DD-002 must
     document. The other cost is **semantic**: the toggle role is **not
     visible in the type name** — a reader cannot distinguish a stateful
     toggle from an ordinary action button until they see the `checkable` /
     `checked` attributes. So the reason to adopt B2, if any, is **surface
     economy / local minimality**, not a future-taxonomy argument. *Boundary:*
     B2 only fixes the **spelling of a button-looking binary toggle**; it is
     **not** a decision to treat Switch / CheckBox / RadioButton / Tab /
     Picker / SegmentedControl as Button modes — grouping, exclusive
     selection, value selection, two-way binding, and widget-owned state stay
     separate future decisions (§Out of scope). *Family note:* Slint/Qt
     checkable buttons **self-toggle** (W3 on axis B); wasamo would borrow
     their **type structure** (a capability on Button), **not** their write
     model — M3 uses the controlled W1, so the author writes `checked`, the
     Button does not flip itself.
   - Technical risk: cross-cutting (parser → check → lower → IR → runtime →
     widget visual → cross-host parity), plus the `checkable`-gated `checked`
     validation; no new node, no new layout primitive.
3. **T1 — dedicated ToggleButton.** `ToggleButton { checked: <bool> }` — a
   **distinct widget type** whose name carries the toggle/select purpose.
   *(Live contender — co-equal with B2.)*
   - What you gain: the toggle role is **named in the type** (the chief
     author-semantics merit); `Button` keeps a single momentary / action
     meaning and there is no cross-attribute dependency — a `ToggleButton`
     simply has `checked`. Matches the **dedicated-type family**: **WPF /
     WinUI**, where `ButtonBase` is the clickable / pressable / commandable
     base and `checked` is introduced at **`ToggleButton.IsChecked`** (with
     `CheckBox` / `RadioButton` specialising `ToggleButton` — internal /
     conceptual button-family sharing is allowed), plus **Radix `Toggle`** (a
     two-state-button primitive) and **MUI / Fluent UI React `ToggleButton`** —
     the model SI-1's WinUI visual lean already points at. (**Compose** and
     **Flutter** sit nearby but mixed: Compose pairs an action `Button` with
     typed `Switch` / `Checkbox` or the low-level `toggleable` modifier;
     Flutter uses purpose-specific `ToggleButtons` / `SegmentedButton` /
     `IconButton.isSelected`. Neither is a B2 capability-flag-on-Button
     precedent.)
   - What you give up / cost: it adds an **author-facing type** for a
     relatively small difference (a button-looking toggle) — a **new widget
     node end-to-end** (parser → check → lower → IR → runtime loader → widget
     visual), **catalog growth** wider than B2's attribute-on-existing-Button
     change, and it **consumes a new type name** (its concrete lexeme is
     SI-4), a forward cost B2 avoids by leaving the type catalog untouched.
     Adopting T1 does **not** fix the future shape of Switch / CheckBox /
     RadioButton / Picker / SegmentedControl — those stay separate decisions.
     *Family note:* WPF/WinUI `ToggleButton` self-toggles (W3); like B2,
     wasamo borrows the **type structure**, not the write model (W1 in M3).
   - Technical risk: a new widget node plus a selected visual; reuses
     Button's leaf measure/arrange (no new layout primitive); the node
     surface is wider than B2's.
4. **G1 — generic Toggle + appearance.** `Toggle { appearance: button;
   checked: <bool> }` — a single toggle control whose **appearance**
   (button / switch / checkbox) is selected by a property. *(Strongly
   recommended defer — not finalized; not reject.)*
   - What you gain: one control covers button / switch / checkbox looks —
     the SwiftUI `Toggle` + `.toggleStyle(.button / .switch / .checkbox)`
     idiom.
   - Why defer (not adopt): it imports an **`appearance` / control-family
     axis** far beyond a single selected state. The lineage is real but
     narrow: the pure form (appearance-only variation of a fixed toggle
     role) is essentially **SwiftUI alone** among modern declarative
     frameworks; the older natives that unify a control family by a
     flag — **Win32 `BS_PUSHLIKE`, WinForms `CheckBox.Appearance = Button`,
     AppKit `NSButton.buttonType`** — vary *role + appearance* together, an
     even heavier control-family-unification design space. The **modern
     declarative mainstream did not take this path** (Compose / Flutter /
     WinUI / most web use **separate types → T1**, or a **capability flag →
     B2**). Adopting G1 in M3 widens role/appearance/control-family well
     past A10's scope, so it is **deferred with a trigger** (a future
     appearance / control-family phase), **not rejected** — SwiftUI is a
     live precedent.
   - Technical risk: n/a in M3 (deferred).

### Recommendation (Layer 1)

- **B1 — reject (strong recommendation, not finalized).** The "every Button
  is selectable" leak is a real surface defect; the properly-specified
  Button option is B2. Recorded as the DD's strong recommendation; the owner
  has not hard-accepted it.
- **G1 — defer with trigger (strong recommendation, not finalized).** A real
  but minority lineage (SwiftUI; Win32/WinForms/AppKit family-unification
  ancestry); adopting it opens an appearance/control-family axis beyond M3.
  Deferred (non-foreclosed) on **Axis 5** (§Out of scope), revived by a future appearance /
  control-family phase.
- **B2 vs T1 — genuinely open (co-equal).** This DD does **not** pre-pick.
  The product-merit comparison is now possible for the first time and is the
  owner's call:

  | | **B2 — checkable Button** | **T1 — dedicated ToggleButton** |
  |---|---|---|
  | **Product taxonomy / author semantics** | toggle is a **mode of Button** — fewer types, the role is *implicit in a flag* at the use site | toggle is its **own kind** — the role is *explicit in the type name*, cleaner per-type semantics |
  | Catalog | **one type** (capability flag) | **new type** (catalog growth) |
  | New checker work | **`checkable`↔`checked` dependency** (SI-5) | none beyond admit-on-type (SI-3) |
  | Lexeme consumed (SI-4) | a capability form (CF-1 `checkable:true` / CF-2 `mode:checkable`) + `checked` — no type name | a new type name + `checked` |
  | **External precedent** *(a secondary consideration — see below)* | clear but **minority**: Qt / Slint checkable Button, ARIA `aria-pressed` | dedicated-type mainstream: WPF / WinUI / Radix / MUI (Compose / Flutter nearby) |
  | SI-1 visual lean | neutral | the WinUI `ToggleButton` look already chosen for the visual |

  The **substantive deciding axis is product taxonomy / author semantics** —
  *toggle as a mode of Button* (B2: fewer types, role implicit in a flag,
  but a new cross-attribute dependency and a toggle role invisible in the type
  name) vs *toggle as its own kind* (T1: explicit role in the type, cleaner
  per-type semantics, but an extra author-facing type for a small diff).
  **External precedent is a *secondary* consideration, deliberately not
  over-weighted:** both options have real lineage — Qt / Slint / ARIA for B2,
  WPF / WinUI / Radix / MUI for T1 — so neither precedent set is dispositive,
  and wasamo's own tree-with-bindings alignment is itself a **live working
  hypothesis, not a ratified long-term direction**
  ([architectural-family.md](../../../../docs/notes/architectural-family.md):
  "no accepted ADR names family (1) as the long-term selection"). So "B2 has
  precedent" must **not** be read as making B2 the design-principled option and
  T1 the merely-cosmetic one. Both are legitimate product taxonomies; T1's
  "explicit role in the type" is a genuine author-semantics merit, not just a
  visual one, and B2's value is **surface economy**, not a future-taxonomy
  claim.

  **B2's cost is *CF-dependent* (SI-4).** Under **CF-1** (`checkable: true`
  boolean flag) B2's new surface is just the `checkable`↔`checked`
  dependency. Under **CF-2** (`mode: checkable` enum) B2 *additionally* opens
  an **enum-valued `mode` attribute** (a new attribute kind) and a
  **future-mode axis** that edges into Axis 5 (control-family) territory — so
  CF-2 widens B2's floor toward T1's and beyond. The table row "new checker
  work" is the CF-1 reading; pick CF-2 and B2's cost grows. This keeps the
  B2-vs-T1 comparison honest about *which B2*. Recorded as
  **Proposed**; the B2/T1 pick is the product-merit call the owner makes at
  the Accepted flip.

## Main decision B — State ownership & write model (Layer 2)

Given a control surface (B2 or T1), **who writes the boolean after a
click?** This axis is **independent of A** (it applies to whichever surface
A picks) and reuses the model analysis from the prior draft. The three
models differ on two questions:

- **Where does the canonical state live?** *lifted* into a `state`
  declaration (W1, W2), or *owned by the widget* (W3).
- **After a click, who writes it?** the **author's code** (W1 —
  controlled, one-way: a `clicked` handler); the **widget through a two-way
  binding** to the lifted state (W2); or the **widget into its own internal
  state** (W3).

M3's shipped surface offers only the **W1** cell: lifted state + the author
writing it (handlers are block assignments; there is no host/widget state
write-back — §Dependencies).

### Options

1. **W1 — controlled + one-way lifted state** *(adopt — the only buildable
   M3 model).* State lives in a `state` declaration; a `clicked` handler
   writes the transition. The Compose model (`Switch(checked,
   onCheckedChange)` — state hoisted, the caller writes it; unidirectional
   data flow).
   - What you gain: it is exactly the shipped single-boolean binding path
     Phase 1 ships; A10's thesis (a binding drives an *attribute*) is
     expressed directly, on either B2 or T1.
   - What you give up: the author writes the per-toggle transition by hand
     (no write-back ergonomics — that is W2).
   - Technical risk: rides the existing single-boolean primitive.
2. **W2 — two-way binding to lifted state** *(defer — exceeds Phase 8).*
   State stays lifted, but `checked` is bound **two-way**, so a click writes
   the value back with **no author handler** (SwiftUI `Toggle(isOn: $x)`).
   - Why defer: requires **two-way binding (state write-back)**, which M3
     has no shipped surface for and which is a binding-direction feature in
     its own right — implementing it now exceeds Phase 8 (A10 is "a *one-way*
     boolean binding drives an attribute"). Deferred (non-foreclosed) on the two-way axis
     (Axis 3). Choosing W1 does **not** foreclose it.
   - Technical risk: n/a in M3 (deferred).
3. **W3 — widget-owned self-toggling state** *(defer — exceeds Phase 8; its
   *full* form is a family-level call, but a narrower opt-in form is lighter).*
   The widget **owns** its `checked` internally and **flips it itself** on
   click (WPF/Qt `ToggleButton`/ checkable `QPushButton`; HTML uncontrolled
   checkbox).
   - Why defer: needs **widget-owned mutable state**, which M3 does not have,
     plus a read-back path. Beyond cost it **strains wasamo's current
     architectural *hypothesis*** — tree-with-bindings keeps state *explicit
     and lifted*, not hidden inside widgets
     ([architectural-family.md](../../../../docs/notes/architectural-family.md),
     a live working hypothesis). **Two revival weights, not one:** a
     **narrow opt-in uncontrolled toggle** (a single widget offering a
     self-toggling mode beside the controlled one) is an **explicit
     uncontrolled-widget surface decision** — additive, not a family pivot;
     whereas making widget-owned state the **general** model would be a
     **family-level (Vision-DR-scale)** shift. W3 is deferred either way
     (Axis 4), but the lighter opt-in path keeps the deferral from reading as
     "only a full family pivot can revive it". *Note:* this is the
     model Slint/Qt/WPF actually use for their toggles — so wasamo's M3
     choice (W1, controlled) is honestly **the Compose/React write model,
     not the native-toolkit one**, even when A picks the native-toolkit
     *type structure* (B2/T1).
   - Technical risk: n/a in M3 (deferred).

### Recommendation (Layer 2)

**W1 — controlled + one-way.** It is the only model real on M3's shipped
surface, and it expresses A10's thesis directly on either B2 or T1. W2 and
W3 are **deferred** (two-way binding / widget-owned state are out of Phase
8; W3's *general* form is additionally a family-level call, though a narrow
opt-in uncontrolled toggle is lighter — option 3), not rejected. Recorded as
**Proposed**.

## Main decision C — Driving boolean & exclusion (Layer 3)

Given the controlled W1 surface (B2's `checked` or T1's `checked`), how is
that boolean *produced*, and how is "exactly one selected" expressed with
shipped surface? This is **unchanged by the Layer-1/2 choice** — the
mechanism is identical whether the boolean rides a checkable `Button` or a
`ToggleButton`; only the attribute/type lexeme in the examples differs. The
spike established that only two forms are real in M3 (α, β); γ / δ need `==`
and are deferred. The open call is **α vs β**, on product merit.

### Options

1. **α — handwritten block assignment** *(live-proven in the spike).* Each
   tab owns an independent `bool` state; each `clicked` sets its own `true`
   and the others `false` in one block. Observation via conditional `if`
   (and, once the attribute lands, via `checked:` itself).
   - What you gain: **zero new Layer-3 surface** — the shipped single-boolean
     primitive applied N times; it shows the **exclusion *behaviour*** (one
     on, others off) live in the gallery tab band, faithful to the
     wireframe. The spike ran this end-to-end on the runtime (click → block
     assign → reactive drain → exactly one marker), with a negative positive
     control confirming the assertion observes live state.
   - What you give up: **O(N²) handwritten assignment** (3 tabs → 9
     assignments); brittle as tabs are added (framing R8).
   - Technical risk: none beyond the Layer-1 surface's cross-layer change.
2. **β — single bool toggled by two buttons** *(minimal, compiles).* One
   `checked` state, a `Select` → `true` and a `Clear` → `false`; observe with
   `if checked` / `checked:`. The tab band stays a **static** highlight
   (exclusion not shown).
   - What you gain: minimal; proves the binding-driven `checked` attribute
     with a two-frame positive control without any O(N²) cost.
   - What you give up: does **not** demonstrate exclusion; the tab band is a
     static highlight, less faithful to the wireframe. (A *single*-button
     self-toggle `clicked => { x = !x }` is **impossible** — `!` is absent —
     so β is a *two-button* on/off, corrected by the spike. Distinct from W3's
     widget-owned self-toggle.)
   - Technical risk: none beyond the Layer-1 surface's cross-layer change.

γ and δ are **not options for M3** — both require `==` to connect a
discriminant to a per-button `checked` boolean, and `==` is absent from the
grammar (spike §案 γ / §案 δ). They are deferred under the equality-operator
axis (Axis 1), **not** placed in the M3 comparison.

### Recommendation (Layer 3)

**α — show exclusion live in the tab band — recommended, with β as the
documented live alternative.** On product merit, α delivers what A10 is
strongest demonstrating: a boolean binding driving a widget attribute under
a realistic multi-button exclusion, matching the wireframe, already proven
on the live runtime. β satisfies A10's core thesis too but trades away the
exclusion demonstration for minimality.

A **third axis** weighs in because Phase 8 ships a *public* gallery:
whatever the example does becomes a **pattern future authors copy**, so α's
O(N²) one-true-others-false block risks teaching an anti-pattern as the
canonical way to express tab exclusion. Two owner-facing ways discharge the
risk: **(α + a provisional note)** — ship the live exclusion but mark it in
the spec / gallery as the *M3-era* shape pointing forward to the future
discriminant form (Axis 1); this note is **not self-discharging in DD-001** —
the DD-002 §DD-001 coupling must carry its concrete form (note authorship,
note strength, migration trigger) so the owner can confirm the mitigation is
real before Accepting α. Or **(β + static approximation)** — prove `checked`
minimally and render the tab band static, recording the static-approximation
in the A1 table / plan (SI-2). α is recommended because the exclusion
behaviour is the more informative thing to show and is de-risked, **and the
teaching-risk is mitigable by the note** rather than by dropping the
demonstration; β is the retreat if the owner judges O(N²) too brittle to be
the first public example. Recorded as **Proposed**.

**α/β disposition at Accept (not a co-held pair).** If the owner Accepts α,
**α is the selected form and β is *not* adopted** — β is retained only as a
**defined fallback with a trigger**: it is substituted *only if* the
implementation / impl-checkpoint shows α unworkable (e.g. the live tab-band
exclusion cannot be rendered cleanly), at which point the substitution is
recorded in the A1 table / plan with the SI-2 static-approximation
accounting. β is **not** a still-open alternative the owner may pick later
without that trigger. (If the owner instead Accepts β outright, α is not
built and the tab band ships static — a different Accept, named explicitly
per §Accepted disposition.)

A **fourth path — make exclusion *easy* by adding `==`** (a single
discriminant + `checked: tab == value`, O(N), intrinsically exclusive) —
would most directly fix α's teaching-risk, so it is weighed as an explicit
**scope-expanding option**, not dismissed as unimplemented. It is held to a
later phase because an equality operator is a **new expression-grammar / IR /
checker / spec feature** — (i) lexer + `HandlerExpr` / `if`-condition grammar
(today `BOOL_LIT | IDENT`); (ii) IR `CompoundOp` + a comparison node; (iii)
checker typing + typed comparison semantics (plausibly narrow per-type, not
necessarily a generic `TypedValue`); (iv) a new diagnostics surface; and (v)
a new public promise in the A12 draft. That is a binding/expression-language
expansion landing in M3's final, draft-publishing phase, so it is held to its
own later phase (Axis 1's trigger), owner-overridable as an **eyes-open M3
scope expansion**, not a feasibility default.

## Accepted disposition (what the flip records)

A full Accept of this DD is **not** a single word — it closes all three axes
plus the sub-issues and the DD-002 gate (the gate's *weight* depends on the
Layer-3 choice — see item 5). The Accepted flip must record the **tuple**
explicitly (a bare "accepted" closes nothing auditable):

1. **A — control surface + full lexeme (SI-4), named together:** e.g. `T1 +
   ToggleButton/checked`, **or** `B2 + CF-1 checkable:true/checked` (the
   **capability form** CF-1/CF-2 **and** the state attribute, per SI-4). **B1
   = rejected** and **G1 = deferred (Axis 5)** are confirmed at the flip (the
   DD's strong recommendations become decisions only here).
2. **B — write model:** `W1` (controlled + one-way) adopted; `W2`, `W3`
   deferred (Axes 3 / 4).
3. **C — exclusion:** `α` selected (β retained only as the triggered fallback
   per §α/β disposition) **or** `β` selected outright (tab band static).
4. **Sub-issues:** SI-1 candidate set + criterion fixed (visual pick deferred
   to the impl checkpoint); SI-2 application target (tab band live / thumbnail
   static, subject to FD-8-G(1)); SI-3 diagnostics rule per the A outcome;
   **SI-5** dependency rule **iff** A = B2 (empty under T1).
5. **DD-002 gate satisfied — weight depends on the Layer-3 choice:**
   - **Under α:** the gate is the **full** one — DD-002 §DD-001 coupling
     must carry its **concrete, inspectable** items **1–3** (the α
     teaching-risk note: authorship, strength, `==`-migration trigger) plus
     item **4** (deferred-axis representation); items 1–3 are a **precondition
     of Accepting α** (§Dependencies / §Couples-to).
   - **Under β outright:** the α teaching-risk note is **not needed**, so
     coupling items 1–3 do **not** gate this Accept; **only item 4**
     (representation of the five deferred axes) applies. β's static-
     approximation accounting is recorded in the A1 table / plan (SI-2), not
     in the DD-002 α-note.
6. **Re-sync targets rebuilt per the A outcome** (the prior §Accepted-time
   re-sync was dropped pending A): **T1** → new-widget re-sync (roadmap A1 /
   A10 / A12, framing, `plan.md`, spec/architecture) in the chosen type name
   — A1's wording "the Button `selected` state surface" updates to the
   ToggleButton form;
   **B2** → keyword re-sync in the **chosen capability form** (CF-1
   `checkable:true` / CF-2 `mode:checkable`) + `checked`, no new type. The
   framing packet-C form (`Button { selected }`) is recorded as **B1,
   rejected** — annotated, not overwritten.

**Partial Accept is allowed and must name what is held.** If the owner fixes
A's *direction* but **explicitly holds the lexeme**, SI-4 is carried
**Accepted-blocking** (the DD is not fully Accepted on the naming axis; the
re-sync and Moment-1 spec sync wait on it) — recorded as e.g. `Accepted: T1;
lexeme held — SI-4 open`. Any element left open is named the same way, so the
record always says exactly which axes closed.

## Sub-issues

### SI-1 — Minimal visual pass line *(owner-presented; A-independent)*

M3 selected visuals are minimal, but the *specific* minimal form must be
**shown to the owner**, not chosen on syntax alone
([spec.md](../../requirements/spec.md) Out-of-scope §Visual). Candidates:

- **V-a — background colour only** (selected cell gets a fill).
- **V-b — border only** (selected cell gets an outline).
- **V-c — colour + border** (both).

A rendered selected visual does **not** exist before the surface is
implemented — the spike's stage 1 does *not* verify visuals (stage 2, after
the surface lands). So the pass line is **not** an Accepted-flip condition;
this DD fixes the **candidate set** (V-a / V-b / V-c) and the **judgement
criterion** (the chosen visual must be distinguishable across the two-frame
positive control, not a static look-alike). The **final pick** is confirmed
at the implementation-plan owner checkpoint (spike stage 2 / FD-8-G(3)),
where the owner sees the rendered candidate. **Verification note:** a
single-cue visual (V-a, fill only) must show the fill change unambiguously
across the two frames; V-c reads least ambiguously, V-a is most minimal.

### SI-2 — Application target: tabs vs thumbnail highlight *(A-independent)*

The wireframe carries selected-state in **two** places: the tab band and a
**highlighted thumbnail** (row 2, col 3). This DD says which the
binding-driven `checked` proof covers — and, because "static approximation"
kept standing in for an answer, **what an author can actually *write* for the
thumbnail highlight in M3, and how it *behaves*.**

**What the `for`-grid can express (the binding facts).** The thumbnails are
`for`-generated over a **scalar collection** — M3 collections are `i32[]` /
`string[]` / `bool[]`, **not** arrays of records. The loop binder is read in
**binding positions** inside the body (e.g. interpolated into Text,
`Text { text: "\{label} #\{index}" }`, gallery.ui), but to highlight *one*
cell you would need a **per-cell boolean** derived from the item/index — and
M3 has **no surface to derive one**: collections carry no per-item record
fields, `if` cannot read the binder (spike S6), handlers inside `for` are
deferred, and there is **no `==` and no indexed collection read**
(`xs[i]`) to compute "this index is the highlighted one". **Consequence: a
`for`-generated thumbnail highlight cannot be data-driven or interactive in
M3** — there is no author surface to make *one* generated cell look or behave
differently from the rest.

**So the thumbnail highlight, in M3, is one of:**

- **TH-a — no highlight (uniform grid).** Every `for`-cell renders
  identically; the wireframe's highlighted cell is not reproduced (this is the
  current `examples/gallery/gallery.ui` state). *Behavior:* none — no cell is
  distinguished.
- **TH-b — fixed decorative highlight (recommended static approximation).**
  The highlight is authored **outside the `for` data path** as one **explicit
  (non-`for`) cell** in the same WrapPanel, carrying the SI-1 visual
  **directly** — i.e. a `Box` with a distinct fill/border. If a frame-*over*-
  thumbnail look is wanted, **that one cell's own subtree is a `ZStack`**
  (placeholder `Box` + an overlay frame `Box`) — intentional overlay is
  **ZStack's** role; this is **not** a `slot.*` placement of a sibling over the
  flow, and **not** a Grid same-cell overlap (Grid rejects that,
  [dsl_spec.md §Grid](../../../../docs/dsl_spec.md)). Its **position follows
  the WrapPanel wrap flow**, not a pixel-precise "row 2, col 3". *Behavior:* a
  **static visual** — no click response, does **not** track a "selected"
  thumbnail, and is **not** an A10 `checked` instance (decoration, not a
  binding-driven attribute). It reproduces the wireframe *look* only.
- **(TH-live — real thumbnail selection: M4.)** Clicking a thumbnail to select
  it (exclusive, data-driven) needs, at minimum: a **click surface on the
  cell** (today `clicked` is `Button`-only — M4 interaction), **handler-position
  binder reads** (admission undecided —
  [dsl-grammar Q8](../../../../docs/notes/dsl-grammar.md)), and a **means to
  derive a per-cell boolean** (`index == selected` — the `==` / M-expr1
  family). **Record collections** are needed only for the *richer*
  per-photo-record variant (select-by-photo-record), **not** for index-based
  selection
  ([gallery-expression-use-cases.md UC3](../../requirements/gallery-expression-use-cases.md)).
  All M4-or-later.

**Recommendation:** the **tab band is the sole A10 binding-driven surface**;
the thumbnail highlight is **TH-b** (fixed decoration, wireframe fidelity) or,
if the owner prefers minimalism, **TH-a**. Under **either**, the thumbnail
highlight is **not** counted as an A10 instance — A10's binding-driven proof
rests on the tab band. This is **forced by the `for` binding facts above**,
not merely preferred.

- Initial hypothesis: per FD-8-G(1) the wireframe-fidelity / placeholder
  agreement updates the A1 feature-mapping table and may revise the TH-a/TH-b
  choice in the implementation plan.
- **If Layer-3 β is chosen**, the tab band *also* shows a *static* highlight
  (no live exclusion); A10's binding-driven proof then rests on the two-button
  toggle, and that accounting (static approximation) must be **recorded in the
  A1 table / plan** at the FD-8-G(1) checkpoint.

### SI-3 — Diagnostics *(A-conditional)*

The admission / rejection rule depends on the control surface:

- **Under T1:** `checked` is admitted on `ToggleButton` and **rejected on
  widgets that do not support it** (`Button { checked: … }`, `Text { checked:
  … }`), as a named check error with a firing test, both directions.
- **Under B2:** `checked` is admitted **only under the chosen capability
  form** (SI-4 CF-1 `checkable: true` / CF-2 `mode: checkable`) and
  **rejected on a plain `Button`** (and on non-supporting widgets); the
  capability-gated case is detailed in **SI-5**.

Either way this falls out of the chosen surface and is the authored-branch
evidence (impl-gates trap #4); detail in §Spec impact.

### SI-4 — Lexical surface *(owner-presented; A-conditional)*

The concrete lexemes depend on the Main-decision-A outcome and are an
owner-presented pick (the type-structure decision does not by itself fix the
lexeme):

- **Under T1 — a type name + an attribute name.**
  - **Type name:** `ToggleButton` / `SelectableButton` / `TabButton` / a
    `ChoiceChip`-style name. Owner reasoning (this conversation): the
    component is a **standalone toggle button with no grouping** (exclusion
    is author-composed, the group surface S4 is deferred), so `TabButton`
    (tab semantics) and `SelectableButton` (selection-in-a-set) **overclaim
    capability the component lacks**; **`ToggleButton`** names exactly "a
    button that toggles" and is recommended.
  - **Attribute name:** `checked` / `pressed` / `selected`. Analysis: for a
    standalone toggle button, `selected` (membership/grouping) is the weakest
    (the component has no set). The real contest is **`pressed` vs
    `checked`**. `pressed` is the ARIA-canonical toggle-button term
    (`aria-pressed`; shadcn/Radix), but in **toolkits** `pressed` is the
    *transient physical-press* word (Slint `Button.pressed`, Qt `down` /
    `pressed()`, WPF `IsPressed`) — the web avoids the clash only because its
    transient state is `:active`. wasamo is a Slint-family **toolkit** and
    M4 will add transient input states, so using `pressed` for the
    *persistent* toggle **double-books** the word M4 wants. **`checked`** is
    recommended: it keeps the family住み分け (toggle = `checked`, transient =
    `pressed`), aligns with WinUI (SI-1) and the multitude, and is value-
    flavored. `pressed` stays defensible **only** if wasamo decides to name
    M4's transient state otherwise.
- **Under B2 — capability expression form + state attribute (no type name).**
  The option space is **not just the attribute name**: *how the capability
  itself is expressed* is a fair choice that must be decidable, because it
  shapes the SI-5 dependency rule and the public surface.
  - **Capability expression form:**
    - **CF-1 — boolean capability flag:** `Button { checkable: true; checked }`
      — the **Slint / Qt** idiom (a boolean that enables the toggle
      capability). Simple, family-consistent with the lineage B2 borrows; two
      booleans (`checkable` enables, `checked` is state). SI-5 dependency =
      `checked` requires `checkable: true`.
    - **CF-2 — mode / role enum:** `Button { mode: checkable; checked }` —
      `mode ∈ { momentary (default), checkable, … }`. One property *names the
      Button's mode*, reads as "this Button is in checkable mode" (WinForms
      `Appearance`-like), and is **extensible to future modes**. Cost: it
      introduces an **enum-valued attribute** — a new attribute *kind* (wasamo
      attrs so far are bool / i32 / string / placement-keyword) — and the
      spelling **resembles G1's `appearance: <variant>`**; keep them distinct
      (`mode` = *behavior capability*, **not** *appearance*). SI-5 dependency =
      `checked` requires `mode: checkable`.
    - *(CF-3 — a toggle-specific flag `toggle: true` / `type: toggle`: a CF-1
      variant, noted not expanded.)*
  - **State attribute name:** `checked` / `pressed` / `selected` — same
    analysis as under T1 (`pressed` double-books the toolkit transient-press
    word + M4 input states; `selected` implies grouping the standalone toggle
    lacks; **`checked`** recommended).
  - *Lean (owner-presented, not pre-picked):* **CF-1 (`checkable: true`)** is
    the family-consistent default and keeps SI-5 a simple boolean dependency;
    **CF-2 (`mode:`)** buys future-mode extensibility at the cost of a new
    enum-attribute kind that edges toward G1's territory. The owner picks the
    **capability form**, not only the name.

*(G1's analogous lexeme — `appearance: button` — is intentionally **not**
enumerated here, since G1 is recommended-defer (Axis 5); it is recorded there,
not in this sub-issue.)*

This is **owner-presented**, not pre-decided. **Recording rule:** the
Accepted flip must name the **A outcome and its full lexeme together** — e.g.
`Accepted: T1 + ToggleButton/checked`, or `Accepted: B2 + CF-1
checkable:true/checked` (the capability form **and** the state attribute); a
bare "T1 accepted" / "B2 accepted" does **not** close SI-4. If the owner
explicitly holds any part of the lexeme, SI-4 is carried **Accepted-blocking**
(the DD is not fully Accepted on the naming axis; re-sync / spec wording wait
on it).

### SI-5 — `checkable` ↔ `checked` dependency model *(B2-only)*

Live **only if Main decision A = B2.** B2 introduces wasamo's first
**cross-attribute dependency**: `checked` is meaningful only under the chosen
capability form (**SI-4 CF-1** `checkable: true`, or **CF-2** `mode:
checkable`). The DD must specify (the rule's *shape* follows the CF choice):

- **`checked:` written without the capability** (no `checkable: true` under
  CF-1, or `mode` ≠ `checkable` under CF-2) → recommended a **named check
  error** ("`checked` requires `checkable: true`" / "… requires `mode:
  checkable`"), with a firing test — the cleanest way to keep B1's leak
  closed. (Alternative considered: implicitly enabling the capability when
  `checked` is present — rejected as it re-introduces the "any Button is
  implicitly checkable" ambiguity B2 exists to remove.)
- **Default value** of `checked` when the capability is on and no `checked:`
  binding given (recommended `false`).
- **Lowering / IR / runtime** representation of the capability (flag under
  CF-1 / enum value under CF-2) + the state, and how the checker enforces the
  dependency.

This is the concrete artifact of B2's "spec burden" cost (§Main decision A)
and feeds the B2-vs-T1 comparison. **Folds into SI-3** if the owner prefers
a single diagnostics sub-issue; kept separate here because the dependency is
the distinctive B2 cost. **Under T1 this sub-issue is empty** (no capability
flag).

## Forward-compat impact

The adopted M3 surface (a checkable Button **or** a `ToggleButton`, with a
controlled one-way `checked`) is M3's **minimal** toggle surface, not "the
one and only selection model forever". The richer models are kept
**non-foreclosed** — not built in M3, their design space left open on
different triggers (the §Out of scope axes hold the triggers). This is
**design non-foreclosure, not a public reservation**: each axis's
*public-draft* representation defaults to a **future-note with a trigger**,
**not** a reserved slot, and is promoted to a public reservation only if
**[DD-002](./dd-m3-p8-002-dsl-spec-public-draft-promotion.md)'s Main decision A
promotes it at DD-002's Accept** — the minimal-reservation default DD-002
item 4 already carries.

- (a) **Two-way binding (W2)** stays open, *conditionally additive*:
  `checked` is one-way in M3; a future two-way form must be **opt-in** (a
  distinct binding sigil) and **leave the M3 controlled contract unchanged**
  (plain `checked: <state>` stays one-way; the author's handler stays the
  writer). **Backstop:** wasamo is **pre-1.0** (BDFL + ADRs,
  [governance-rfc-deferral.md](../../../cross-milestone/decisions/governance-rfc-deferral.md)),
  so a breaking revision is permitted pre-1.0 if the opt-in cannot be met
  cleanly.
- (b) **Widget-owned self-toggle (W3)** stays open, at **two weights**: a
  **narrow opt-in uncontrolled toggle** (a self-toggling mode beside the
  controlled one) is an **explicit uncontrolled-widget surface decision**,
  additive — *not* a family pivot; making widget-owned state the **general**
  model would be a **family-level (Vision-DR-scale)** shift away from the
  lifted-state shape wasamo currently sits in (the imperative WPF/WinUI
  family). Either is deferred (Axis 4). (Note: this is how the native
  toolkits whose *type structure* B2/T1 borrow actually write their toggles;
  M3's controlled W1 deliberately does not.)
- (c) **Group semantics** stay free to design — exclusion is expressed in M3
  only with shipped surface (α / β), leaving future `RadioGroup` / `TabBar` /
  `SegmentedControl` parents free (Axis 2).
- (d) **Generic Toggle + appearance (G1)** and the broader **control-family
  unification** (Win32/WinForms/AppKit ancestry) stay **deferred (future-note,
  Axis 5)** for a future appearance / control-family phase; M3 does not open an
  `appearance` axis.
- (e) M3 selected visuals are provisional and absorbed/overridden by the M5
  theme surface; accessibility / focus / input semantics are re-designable
  in M4+.

**B2 vs T1 forward note.** The forward cost differs by A: **T1** consumes a
new **type name** (SI-4) whose later growth is either same-type binding
modes or new sibling widgets (undecided); **B2** consumes the **chosen
capability form** (CF-1 `checkable` / CF-2 `mode`) + `checked` **keywords**
and a cross-attribute-dependency rule, but leaves the type catalog untouched.
**Under CF-2, B2 additionally opens an enum-valued `mode` attribute and a
future-mode axis** that edges into Axis 5 (control-family) territory — a wider
forward cost than CF-1. Neither is built here beyond the M3 minimal surface.

## Spec impact

`docs/dsl_spec.md` (author-facing, external-reader bar, **no DD/option
labels** per the living-spec vocabulary rule; provenance via ADR hyperlink
only) — **A-conditional**:

- **Under T1:** a **`ToggleButton`** widget with a `checked` boolean
  attribute (SI-4 lexeme), driven by the existing one-way boolean binding.
- **Under B2:** the **chosen capability form** (SI-4 CF-1 `checkable: true`
  flag, or CF-2 `mode: checkable` enum) **+ `checked`** state on `Button`,
  with the capability-gated `checked` rule (SI-5) stated as the admission
  rule. **If CF-2,** the spec also documents the `mode` enum attribute and its
  value set (`momentary` / `checkable`), kept minimal (not the full
  future-mode axis, which is Axis 5).
- Common: the toggle is **controlled** (the click → value → state write is
  the author's handler), with the two-way (W2) and widget-owned (W3) models
  noted out of scope. The **chosen minimal visual** is **not** asserted at
  Moment 1 — the spec states it as *minimal, candidate set V-a/V-b/V-c, pick
  pending the implementation checkpoint*; the **concrete visual is pinned at
  Moment 2** (after the impl checkpoint), consistent with SI-1.
- **Admission / rejection table (forcing artifact):** the chosen rule
  (T1: `checked` on `ToggleButton` only; B2: `checked` under the chosen
  capability form only — CF-1 `checkable: true` / CF-2 `mode: checkable`),
  re-checked by the loader, each with a firing test and a paired accept/reject
  fixture.
- **Exclusion** documented as a *composition of boolean states* (the Layer-3
  form), stated honestly as **author-composed, not a built-in group
  construct** (bears on DD-002's public-draft positioning).
- Stale prose swept; selected/toggle visuals documented as **minimal /
  provisional**, pointing forward to the M5 theme surface.

`docs/architecture.md`: the chosen control surface (a `ToggleButton` node, or
a capability + `checked` on Button — CF-1 boolean flag / CF-2 `mode` enum) and
its representation through lower / IR / runtime loader / widget visual,
consistent with the existing
single-boolean binding model (no new binding-target class, no new
measure/arrange — reuses Button's leaf layout).

## Risk mitigation

- **The toggle surface is a cross-cutting change (framing R6).** It crosses
  parser → check → lower → IR emit → runtime loader → widget visual →
  cross-host parity. Under **T1** it adds a **new widget node**; under **B2**
  it adds a **cross-attribute dependency** (SI-5). Beyond the impl-gates
  call-site audit table (trap #1), the **checked-propagation audit** is the
  central A10 evidence, pinned by firing tests / positive controls: (i)
  `checked` rejected where unsupported (T1: non-`ToggleButton`; B2:
  capability-absent — no `checkable:true` / `mode:checkable`), (ii) a
  bool-binding change reaches the visual, (iii) C / Rust / Zig render the same
  (cross-host parity).
- **Over-build guard (framing R3).** No two-way binding (W2), no
  widget-owned state (W3), no dedicated group widget (Axis 2), no generic-
  appearance Toggle (G1), and no full theme are built; the toggle stays
  controlled + one-way (W1), visuals minimal (SI-1), exclusion author-
  composed (α/β). The richer models are kept **non-foreclosed** (deferred with
  triggers), not built.
  **If B2 + CF-2 is chosen,** the `mode` enum is kept **minimal**
  (`momentary` / `checkable` only); the full future-mode / appearance axis is
  **not** pre-opened (that is Axis 5 / G1) — CF-2 buys naming, not the control
  -family surface.
- **Demonstration-vehicle feasibility (framing R8).** Whether the tab-band
  exclusion (α) is too heavy was the explicit reason for the pre-DD spike,
  which proved α real on the live path and fixed β as the retreat.
- **Positive control (AGENTS.md §Testing rules).** A single static frame a
  wrong implementation could equally produce is **not** evidence: the
  selected visual must be shown changing across a **two-frame** toggle (and,
  under α, the exclusion — one on, others off — in the same two frames).
  Assistant evidence = launch + DPI-aware screenshot + analysis; owner
  human-visible smoke is a separate gate.

## Out of scope (deferral axes — kept distinct)

The not-chosen candidates split into **axes that revive on different
triggers**; recorded as **separate items**, not bundled.

- **Axis 1 — equality-operator family (`==`-family): γ + δ.** A single
  discriminant state (i32 / string / enum-like) with `checked: tab == value`.
  Not buildable in M3 (`==` absent). **Trigger:** an equality operator `==`
  enters the expression grammar. **Revived form:** one discriminant + `checked:
  tab == value` — O(N), intrinsically exclusive — replacing α's O(N²). Whether
  `examples/gallery/` is later migrated is an **independent** decision for the
  `==` phase.
- **Axis 2 — group-surface family: S4.** Parent/group widgets (`RadioGroup` /
  `TabBar` / `SegmentedControl`) that manage exclusion so the author writes no
  comparison. **Trigger:** **not `==`** — value write-back / selected value /
  child value / interaction semantics. The natural home for the toggle/select
  role *plus* group exclusion. Do **not** fold into Axis 1.
- **Axis 3 — two-way binding (W2 / SwiftUI model).** `checked` bound two-way,
  so a click writes the state without a handler. **Trigger:** two-way binding
  enters the binding grammar. About binding *direction*, keeps state lifted
  (family-consistent). Not auto-exclusive (that is Axis 2).
- **Axis 4 — widget-owned state (W3 / WPF/Qt model).** A self-toggling widget
  owning its `checked`. **Trigger (two weights):** a **narrow opt-in
  uncontrolled toggle** is an **explicit uncontrolled-widget surface
  decision** (additive — a self-toggling mode beside the controlled one);
  making widget-owned state the **general** model is a **family-level
  (Vision-DR-scale)** shift away from the lifted-state shape wasamo currently
  sits in. Not a binding feature either way. Kept distinct from Axis 3.
- **Axis 5 — generic-Toggle appearance / control-family unification (G1).** A
  single control whose appearance (button / switch / checkbox) is selected by
  a property (SwiftUI `Toggle` + `toggleStyle`), and the broader role+appearance
  unification (Win32 `BS_PUSHLIKE`, WinForms `Appearance`, AppKit
  `NSButton.buttonType`). **Trigger:** a future appearance / control-family /
  theming phase that deliberately opens the `appearance` axis. Deferred, not
  rejected (SwiftUI is a live precedent); kept distinct from the type-vs-
  capability choice (T1/B2), which is about *structure*, not appearance.

Also out of M3 scope (existing triggers hold):

- **Visual-theme-via-binding** (`style: cond ? accent : default`) as the
  *selection-authoring* surface — rejected: the ternary form is not in the
  grammar, and the `if`-swap workaround duplicates handlers and leaves
  selection unrepresented. Does **not** foreclose future **theme
  customization** of selected visuals (M5).
- **Full theme / rich selected visuals** — M5 (M3 visuals minimal, SI-1).
- **Accessibility / focus / input semantics for selection** — M4+ (and the
  transient-press / `pressed` state, SI-4).
- **Real thumbnail selection (hit-testing / focus / gesture)** — M4; M3
  renders the highlighted thumbnail as a static approximation (SI-2).
- **Data-driven tabs (`for`-generated, S6)** — blocked by M3's `for`
  constraints.

## Revision history

- 2026-06-26 — Initial draft (Status: Proposed). Two-layer structure from the
  stage-1 spike: Layer-1 authoring form and Layer-2 driving-boolean production
  (α recommended with β as the live alternative, γ/δ deferred). Minimal visual
  pass line (SI-1); application target (SI-2); diagnostics (SI-3).
- 2026-06-26 — Codex review folds. SI-1 moved off the Accepted-flip condition
  onto the implementation checkpoint; S3 scope-of-reject note added.
- 2026-06-27 — Layer-1 recommendation set to S2a (`ToggleButton { checked }`,
  controlled + one-way); S1 recast as minimal alternative; S2b/S2c split out
  and deferred.
- 2026-06-27 — Codex re-review folds. Named the S2a recommendation a reversal
  of packet C / the spike S1-lead; added §Accepted-time re-sync; narrowed
  S2a's benefit to type-naming.
- 2026-06-27 — Split the dedicated-widget option into three named models
  (S2a Compose / S2b SwiftUI / S2c WPF·Qt); §Out of scope split into write-back
  axes.
- 2026-06-27 — Owner forward-compat critique fold. Per-instance mutual
  exclusivity of the three models; "extension" takes one of two undecided
  shapes; type-home benefit marked contingent.
- 2026-06-27 — Strategic / owner-alignment review folds. Lexical naming
  extracted to SI-4; `==` recast as a deliberate M3-close scope decision;
  public-example teaching-risk axis added; S2c softened to a family-level call.
- 2026-06-27 — Codex re-review folds. Two-stage Accept (S1-vs-S2; SI-4 lexeme);
  `==` rejection re-based on a scope-expanding cost frame; α teaching-risk note
  named its DD-002 dependencies.
- 2026-06-27 — Codex re-review (3rd pass) folds. Removed the SI-4 double-read;
  softened the `==` cost item to typed comparison semantics.
- 2026-06-28 — Accept-discipline review folds. Re-sync auditability (added
  plan.md; framing packet C annotated-as-superseded vs the A1 table
  re-pointed); SI-4 silent-sweep recording rule; visual-write timing split
  (Moment 1 candidate set / Moment 2 pinned).
- 2026-06-28 — Gate-strength alignment. §Couples-to raised to "concrete,
  inspectable form".
- 2026-06-28 — **Major restructure to a control taxonomy (Status: Proposed;
  re-opens stage-1).** Reason: the prior "S1" (`Button { selected: bool }`)
  was **under-specified** — a bare boolean on the generic Button reads as if
  every Button is selectable; the properly-specified Button option (a
  **capability**, `Button { checkable; checked }`) was never compared, which
  changes the comparison against a dedicated widget. **Main decision A
  recast** from "S1 vs S2" into a **control taxonomy: B1 generic Button
  attribute (strong reject) / B2 Button capability / T1 dedicated
  ToggleButton / G1 generic Toggle + appearance (strong defer)**, with
  **B2 ↔ T1 co-equal and owner-open** (no DD pick). The old S2a/S2b/S2c
  state-ownership distinction **lifted out** into a new **Main decision B —
  write model (W1 controlled adopt / W2 two-way defer / W3 widget-owned
  defer)**, independent of A. The old Layer-2 (α/β) moved to **Main decision
  C**. The **S1-vs-S2a "reversal" narrative removed** (the packet-C form is
  simply B1, rejected as under-specified). **§Accepted-time re-sync** dropped
  pending the A outcome (T1 → new-widget re-sync; B2 → keyword re-sync;
  rebuilt at the Accepted flip). **Sub-issues re-organised:** SI-1 / SI-2
  unchanged (A-independent); SI-3 diagnostics A-conditional; SI-4 lexical
  surface A-conditional (carries the standalone-toggle reasoning and the
  `pressed` vs `checked` analysis: `pressed` double-books the toolkit
  transient-press word + M4 input states, so `checked` recommended); **SI-5
  new** — the B2 `checkable ↔ checked` dependency. **§Out of scope** gains
  **Axis 5** (G1 appearance / control-family unification, deferred with the
  SwiftUI / Win32 / WinForms / AppKit lineage recorded). B1-reject and
  G1-defer are the DD's **strong recommendations, not finalized**. Status
  remains Proposed.
- 2026-06-28 — Post-restructure review folds (Status: Proposed;
  recommendation unchanged). **B2/T1 fairness:** separated *product taxonomy /
  author semantics* (mode-of-Button vs own-type — the substantive deciding
  axis) from *current-impl / family fit* (explicitly a **live hypothesis, not
  a ratified direction**), so B2 is not framed as the principled option and
  T1 as merely cosmetic. **W3 deferral softened:** split into a *narrow opt-in
  uncontrolled toggle* (an explicit uncontrolled-widget surface decision,
  additive) vs the *general* widget-owned model (a family-level shift) — both
  deferred (Axis 4 / Forward-compat (b) / Main decision B updated). **Added
  §Accepted disposition** — the Accept records a tuple (A surface + lexeme /
  B write model / C exclusion / sub-issues / DD-002 gate / A-conditional
  re-sync), with partial-Accept naming what is held. **α/β disposition
  clarified:** on Accepting α, β is a *triggered fallback* (substituted only
  if the impl checkpoint shows α unworkable), **not** a co-held open
  alternative. Status remains Proposed.
- 2026-06-28 — Accepted-disposition β-gate clarification (Status: Proposed).
  §Accepted disposition item 5 made the **DD-002 gate weight Layer-3-
  conditional**: under α the full gate (coupling items 1–3 + 4, items 1–3 a
  precondition of Accepting α); under β-outright only item 4 (deferred-axis
  representation), the α teaching-risk note not needed. Recommendation
  unchanged.
- 2026-06-28 — SI-2 / SI-4 author-surface expansion (Status: Proposed;
  recommendations unchanged). **SI-2** now explains **what an author can write
  for the thumbnail highlight and how it behaves**, grounded in the `for`
  binding facts (scalar-array collections — no records; binder usable only via
  string interpolation; `if` can't read the binder; `for`-internal handlers
  deferred) → a `for`-generated highlight **cannot** be data-driven or
  interactive in M3. Enumerated **TH-a** (no highlight / uniform grid),
  **TH-b** (fixed decorative highlight via ZStack overlay or a hand-authored
  exceptional cell — static, non-interactive, **not** an A10 instance), and
  TH-live (M4); recommended TH-b (or TH-a), tab band the sole A10 surface.
  **SI-4** B2 branch expanded so the **capability *expression form*** is a
  first-class option, not just the attribute name: **CF-1** boolean flag
  `checkable: true` (Slint/Qt, simple, family-consistent) vs **CF-2** mode
  enum `mode: checkable` (extensible, but a new enum-attr kind edging toward
  G1) — fed into SI-5 (the dependency-rule shape follows the CF choice) and
  SI-3; the Accept lexeme now records the capability form too. (G1's
  `appearance:` analog intentionally omitted — G1 is recommended-defer.)
- 2026-06-28 — CF-2 downstream-sync + SI-2 validity folds (Status: Proposed;
  recommendations unchanged). **CF-2 made consistently live:** §Accepted
  disposition (re-sync), §Spec impact (surface + admission table + the `mode`
  enum), §architecture, §Forward-compat note, and §Risk now read **"chosen
  capability form (CF-1 `checkable:true` / CF-2 `mode:checkable`)"** instead of
  CF-1-hardcoded `checkable`. **CF-2 cost carried into the comparison:** the
  B2/T1 prose + over-build guard note that CF-2 additionally opens an
  enum-`mode` attribute kind + a future-mode axis (edging into Axis 5), kept
  minimal (`momentary`/`checkable`) — so B2's floor is honest about *which B2*.
  **SI-2 TH-b corrected to valid authoring shapes:** an explicit (non-`for`)
  cell beside the `for` block in the WrapPanel (static-sibling interleave is
  admitted, dsl_spec) carrying the SI-1 visual directly, or that one cell's
  subtree as a **ZStack** for an overlay frame (intentional overlay is ZStack's
  role; **not** `slot.*` over the flow, **not** a Grid same-cell overlap which
  Grid rejects). **SI-2 binding-facts corrected:** binders are read in binding
  positions generally (not "string interpolation only"); the real blocker is
  no per-cell boolean can be derived (no record fields, no `==`, no indexed
  read, `if` can't read the binder).
- 2026-06-29 — Re-pointing fold: dispute narrowed, gain/lose tone aligned
  (Status: Proposed; **B2/T1 stay co-equal — no T1 lean**; three-axis A/B/C
  structure unchanged, exclusion stays Main decision C). Main decision A gains
  a **"dispute, stated narrowly"** framing — the question is whether wasamo's
  author-facing **button abstraction carries optional checkability**, **not**
  whether wasamo adopts a capability-typed component system; the choice is
  author-surface vocabulary, not internal inheritance (T1 may share a
  `ButtonBase` path; B2 does not make Switch / CheckBox / RadioButton / Tab /
  Picker / SegmentedControl into Button modes). **B2** reframed: gain = the
  smallest surface, reusing Button look/press/impl, with **Qt / Slint / ARIA**
  as clear-but-minority precedent (not a wasamo-direction argument); added
  cost = the toggle role is invisible in the type name, so B2's case is
  **surface economy**, plus an explicit **boundary**. **T1** reframed: gain =
  role named in the type, with **WPF / WinUI** (`ButtonBase` → `ToggleButton`,
  `CheckBox` / `RadioButton` specialise), **Radix**, **MUI**; cost = an extra
  author-facing type for a small diff; **Compose / Flutter** noted as mixed,
  not B2 precedents. Comparison row "current-impl / family fit" →
  **"external precedent"** (secondary; both sides have lineage, neither
  dispositive). Recommendation unchanged.
- 2026-06-29 — Owner-review folds (Status: Proposed; recommendations
  unchanged). **(1) SI-2 TH-live weakened:** real thumbnail selection needs, at
  minimum, a cell click surface + handler-position binder reads (Q8) + a
  per-cell boolean (`index == selected`); **record collections are only the
  *richer* per-photo-record variant**, not the index-based baseline (per
  [gallery-expression-use-cases.md UC3](../../requirements/gallery-expression-use-cases.md)).
  **(2) Deferred-axis strength aligned to DD-002:** §Forward-compat reworded
  from "stay reserved" to **design non-foreclosure → public-draft default =
  future-note with a trigger, promoted to a reservation only at this DD's
  Accept** (resolving DD-002 item 4's "Main decision A's minimal-reservation
  policy" reference, which DD-001 had not stated); G1 (d) "stay reserved" →
  "deferred (future-note)". **(3) Re-sync target fixed:** T1 new-widget re-sync
  `A9 → A1` (A9 = bool scalar, untouched by a new type name; A1 carries the
  "Button `selected` state surface" wording that updates under T1).
- 2026-06-29 — Owner-review folds (round 2; Status: Proposed; recommendations
  unchanged). **(1) Promotion authority disambiguated:** §Forward-compat now
  says public-reservation promotion is **DD-002's Main decision A at DD-002's
  Accept** (not DD-001's control-taxonomy Main decision A). **(2) Residual
  "reservation" language swept** to match the non-foreclosure framing:
  §Couples-to "reserved axes" → "deferred (non-foreclosed) axes"; Layer-1 G1
  "Reserved on Axis 5" and Main-decision-B W2 "Reserved on the two-way axis" →
  "Deferred (non-foreclosed)"; §Risk over-build guard "kept as reservations" →
  "kept non-foreclosed (deferred with triggers)".
