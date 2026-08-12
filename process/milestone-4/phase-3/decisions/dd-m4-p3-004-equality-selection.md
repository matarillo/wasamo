# DD-M4-P3-004 — Equality-based selection from a single discriminant

**Status:** Proposed
**Phase:** M4-Phase 3
**AC:** AC9 (equality-based selection); AC12 as an incremental
consumer only; phase-end criterion 4 (spec synchronization)

## Context

M3 shipped selection as a hand-composed pattern and said so in the spec:
one `bool` state per option, every handler writing every state.
[dsl_spec.md §4.17](../../../../docs/dsl_spec.md) records that this
"grows as O(N²) hand-written assignments", that it "must **not** be read
as a reserved or long-term idiom", and that the intended replacement is
"a discriminant state with `checked: tab == value`, once an equality
operator enters the expression grammar". The gallery's tab strip is that
pattern in the shipped app: three `bool` states, three handlers, nine
assignments.

M3's handoff split selection into five axes and sent four onward. This
record owns exactly one: **one discriminant, equality, exclusive
display**. Group widgets, widget-owned state and generic toggle
appearance stay with M5; two-way binding stays with M4-Phase 7.

The operator itself is DD-001's. What is left for this record is the
part an operator does not answer: which surface the resulting `bool`
drives, what the click writes, how selection stays distinct from focus,
and what an out-of-collection discriminant means.

### What exists to build on (measured)

- **The per-item bool binding path already exists.**
  `register_for_item_bool_binding` and
  `evaluate_bool_binding_optional` were added at M4-Phase 2 so a
  `ToggleButton.checked` inside a `for` body can be bound per item. What
  `checked: index == selected_index` needs is the expression, not the
  writer.
- **`checked` is admitted on `ToggleButton` only** (§4.17), re-checked
  by the loader, and its selected visual is a background-colour change
  the spec calls "minimal and provisional".
- **`Box.fill` is constant-only.** `check_box_const_only_bind` rejects a
  non-literal right-hand side, and
  [dsl_spec.md §8.12](../../../../docs/dsl_spec.md) defers a bindable
  `fill` to "a future phase that first needs reactive aspect or fill".
- **The gallery's thumbnails are `Box`.**
  `examples/gallery/gallery.ui` builds each as
  `Box { aspect: 1:1 fill: #4f6272 clicked => { … } Text { … } }`, and
  clicking one sets `selected_index` and opens the lightbox — it is a
  navigation target, not a toggle.
- **`ToggleButton` is a focus stop.** `WidgetNode::focus_role` maps both
  `Button` and `ToggleButton` to `FocusRole::Stop`; every container maps
  to `Container` / `Group` / `ModalScope`. Eighteen `ToggleButton`
  thumbnails would therefore be eighteen new Tab stops in the gallery
  window.
- **Selection and focus are already separate states** in the runtime,
  and Phase 2 fixed and visually confirmed their combined appearance.

## Sub-issues

- **Which display surface an equality result may drive.**
- **Which surface the shipped gallery uses**, which is a product choice
  and not the same question.
- **What the click writes**, and where the boundary against widget-owned
  state and two-way binding is drawn.
- **Selection versus focus** as an author-visible contract.
- **Index equality versus value equality.**
- **A discriminant that matches no item.**

## Options

### Projection surface (the language rule)

- **V1 — `checked:` only.** Equality drives `ToggleButton.checked`; a
  non-`ToggleButton` cannot show selection.
- **V2 — a conditional marker only.** Equality drives a per-item
  conditional subtree (DD-003), so the selected item gets an extra
  child.
- **V3 — both, because neither is a rule.** A `bool`-valued comparison
  is admissible wherever a `bool` expression is admissible, which
  follows from DD-001's uniform admission and needs no rule here.
- **V4 — make `Box.fill` bindable**, so the thumbnail's own fill can
  change.

### The shipped gallery consumer (the product choice)

- **A — the thumbnails become `ToggleButton`s** with
  `checked: index == selected_index`.
- **B — the thumbnails stay `Box`** and the selected one gets a marker
  subtree through DD-003.
- **C — neither; the selection proof moves to a named mechanism
  fixture** and the gallery is not changed.

### Discriminant that matches no item

- **N-1 — zero items selected**, silently. Equality is simply false
  everywhere.
- **N-2 — a diagnostic**: an out-of-collection discriminant is reported.
- **N-3 — the discriminant is clamped** so something is always
  selected.

## Comparison

### The language rule: V3 is what DD-001 already decided

V1 and V2 are both *rules about where a `bool` may be used*, and DD-001
chose admission by type rather than by position precisely so that no
such rule is written twice. Under DD-001's P1 a comparison is a `bool`
expression; `ToggleButton.checked` takes a `bool`; an `if` condition
takes a `bool`. Both projections follow. Writing V1 or V2 as a rule here
would create the position table DD-001 rejected, and would do it in the
one record whose subject is a value, not a position.

V4 is a different kind of option: it is a scope addition, deferred by
§8.12 to the phase that first needs reactive fill, and adopting it here
would take that decision from its owner. It is listed because it is the
option an author would reach for first — "make the selected tile look
different" is naturally a fill change — and because rejecting it is what
forces the choice between A and B below. Its cost is not just scope: a
bindable `fill` needs a `Color` value in the binding type system, which
is exactly the `IrType::Ratio` / `IrType::Color` deferral §8.12 records
alongside `TypedValue`.

### The product choice: A versus B is a real trade, and B wins on the app

This is the part of the record that is not settled by DD-001, and it is
the point where the framing's illustrative example
(`ToggleButton { checked: index == selected_index }`) and the shipped
app disagree — the framing's example is explicitly a provisional
spelling, and the gallery's thumbnails are `Box`.

**A's arguments.** It matches the spec's own anticipated form (§4.17
names `checked: tab == value`); it proves the M3 selected-state axis at
the surface M3 named; it uses a runtime path that already exists; and it
makes the "focus moved but selection did not" leg of the verification
matrix a *discriminating* leg, because with `Box` thumbnails Tab never
reaches a thumbnail and that leg is trivially true.

**A's costs.** Three of them, and the first is the serious one.

1. **It changes Phase 2's focus behaviour in the shipped app.**
   Eighteen thumbnails become eighteen Tab stops in a window that
   currently has a handful. Recovering the previous traversal would mean
   annotating the `WrapPanel` as a focus group — a Phase 2 surface, used
   here to undo a side effect Phase 3 introduced. The framing puts
   "Phase 2 の routing / focus / modal-scope の再設計" out of scope; this
   is not a redesign, but it is a Phase 2-visible behaviour change
   arriving through a Phase 3 expression record, which is the shape of
   thing the phase boundaries exist to prevent.
2. **It asserts a semantics the thumbnail does not have.** A
   `ToggleButton` is a control the user toggles. The gallery's thumbnail
   opens the lightbox; it is a navigation target that happens to also be
   the current one. Making it a toggle to obtain a background tint
   misrepresents it in the app that is the milestone's outward-facing
   banner.
3. **The visual it buys is the wrong one.** §4.17's selected visual is a
   background-colour change, and a thumbnail's background is covered by
   its own `fill`. The strongest form of A would want the fill to change
   — which is V4, deferred.

**B's arguments.** The thumbnail stays what it is; the marker is an
additive child that says "this one"; it exercises DD-003 and DD-004
together in the shipped app, which is the composition AC9 actually
names; and it needs no widget-kind change, no new focus stops and no
deferred bindable property.

**B's costs.** The `checked:` projection is then not exercised by the
shipped app, and the focus-versus-selection separation leg is weaker,
because no thumbnail is focusable. Both are answerable without changing
the app: `checked: <comparison>` is exercised by the tab strip, which
**is** `ToggleButton` and **is** the O(N²) pattern §4.17 asks to be
replaced — three `bool` states and nine assignments collapse to one
discriminant and three comparisons, in the shipped app, with no widget
change. And the focus-versus-selection leg then runs on the tab strip,
where the buttons are already focus stops and already inside a
`focus-group`, which makes it a genuinely discriminating leg rather than
a vacuous one.

That reframing is what decides it: **B does not give up the `checked:`
proof, it moves it to the consumer that was already asking for it.** The
gallery then demonstrates both projections — `checked:` on the tab strip,
conditional marker on the thumbnails — with one widget-kind change fewer
than A, not one more.

C is left available and is not recommended: the gallery has two natural
consumers, so pushing selection into a fixture would be avoiding the app
rather than protecting it.

### What the click writes

The click keeps writing an author-owned discriminant. `ToggleButton`
does not flip its own `checked`; `checked` stays one-way and controlled,
exactly as §4.17 fixed it. This is not a new decision — it is the M3
contract surviving contact with a new expression — and stating it is how
the record makes clear that equality selection is **not** a step toward
widget-owned state or two-way binding, both of which have owners.

The tab strip's handlers therefore go from three assignments each to
one, and the thumbnail's `clicked` keeps writing `selected_index` as it
does today.

### Selection versus focus

They stay separate states with separate causes: focus moves on Tab and
on click; selection moves only when a handler writes the discriminant.
Tab across the tab strip does not change which tab is selected. A widget
that is both selected and focused keeps the composed appearance Phase 2
fixed, and this record must not introduce a second appearance rule.

### Index equality versus value equality

Both are admitted by DD-001's typing (`i32` and `string` equality are
both in the set). They differ in a way the author must be told: with
value equality over a collection containing duplicates, more than one
item compares equal, so "exactly one" is not a property of equality — it
is a property of comparing against a **positionally unique** value. The
gallery's exclusivity is proved with index equality for that reason, and
the spec should say plainly that value equality selects *every* matching
item, which is sometimes what the author wants and is never exclusivity.

### A discriminant that matches no item: N-1

N-3 is clamping under another name and is rejected for the reason DD-002
rejects it: it invents a selection the author did not write, and it has
no answer for an empty collection.

N-2 asks the runtime to decide that `-1` is wrong. It is not wrong —
"nothing selected" is a legitimate state, and `-1` is a conventional
spelling for it, which DD-002 has already adopted for `last-index()` on
an empty collection. Making it a diagnostic would forbid the one value
that expresses the empty selection.

N-1 falls out of the semantics rather than being chosen: equality is
false for every item, so zero markers appear and zero `checked` are
true. The only thing this record adds is the explicit statement that
this is **not** DD-002's out-of-range contract — no collection is
indexed by an equality comparison, so no read fails, nothing is retained
and nothing is reported. The two must not be conflated, because the same
`-1` produces a silent empty selection here and a reported failed read
there, and an author who does not know that will read one as the other.

## Recommendation

- **V3** — no new admission rule. A comparison is a `bool` expression;
  `ToggleButton.checked` and a DD-003 conditional condition both take
  `bool`; both projections follow from DD-001.
- **B for the shipped gallery**, with both projections demonstrated:
  - the **tab strip** replaces its three `bool` states and nine
    assignments with one `i32` discriminant and
    `checked: <discriminant> == <value>` per tab — the exact collapse
    §4.17 asks for, in the shipped app, with no widget-kind change;
  - the **thumbnails stay `Box`** and the selected one gains a marker
    through DD-003's per-item conditional.
- **The click writes an author-owned discriminant.** `checked` stays
  one-way and controlled; no widget-owned state, no two-way binding, no
  self-toggling.
- **Selection and focus stay separate**, with the Phase 2 composed
  appearance unchanged and no second appearance rule.
- **Index equality is what proves exclusivity.** Value equality is
  admitted and selects every matching item; the spec says so rather than
  leaving the author to discover it with duplicates.
- **N-1** — a discriminant matching no item means zero items selected.
  It is not a diagnostic, it is not clamped, and it is explicitly
  distinguished from DD-002's out-of-range read contract.
- **No new widget, no new selection ownership model, no group surface.**
- **`docs/dsl_spec.md` moves**: §4.17 (the O(N²) pattern is replaced by
  the single-discriminant form; the "future direction" row for equality
  selection becomes shipped surface; the duplicate-value caveat is
  added), §4.6 / §4.14 as DD-001 and DD-003 move them.

## Forward-compat exposure

- **Group and exclusive-selection widgets (M5) are unconstrained.** A
  `RadioGroup` that manages exclusion internally does not have to be
  built on this record's discriminant, and this record reserves no
  spelling it would want.
- **Two-way binding (M4-Phase 7) is unconstrained**, because nothing
  here makes `checked` writable. If Phase 7 makes it two-way, the
  discriminant form still works and simply stops being the only way.
- **Widget-owned selection state remains a reopening, not an addition**,
  for the same reason it was in M3: it changes who owns the value, and
  every existing `.ui` written against the controlled contract would
  have to be re-read.
- **A bindable `Box.fill` would make the marker unnecessary** for the
  gallery's thumbnails. That is the honest statement of what B costs: it
  is the best available projection **given** that reactive fill is
  deferred, not the projection this record would choose in a language
  that had one. Recording it here is what lets the phase that lands
  reactive fill revisit the gallery's thumbnail on purpose rather than
  by accident.
- **Multi-selection is not addressed.** A discriminant is singular by
  construction; a set-valued selection would be a different design, not
  an extension of this one, and nothing here reserves a spelling for it.

## Technical risk re-evaluation

- **The tab-strip change is the only part of this record that alters
  behaviour in shipped code.** Three `bool` states become one `i32`, and
  three handlers each lose two assignments. The failure mode is a strip
  where zero or two tabs appear selected, which is visible; the subtler
  one is a strip where the *initial* selection is wrong because the
  discriminant's default does not correspond to the tab that was
  `true` before. That is a one-line default and exactly the kind of
  detail a diff review passes over.
- **"Exactly one" is not provable from a single frame.** A marker that
  is always present on item 0 and a correct marker on the selected item
  produce the same picture until the selection moves. The positive
  control is moving the discriminant to a second value and showing the
  marker moved **and** that only one exists — the count matters as much
  as the position.
- **The focus-versus-selection leg is only discriminating where the
  widget is focusable.** Under recommendation B that leg belongs to the
  tab strip, not the thumbnails, and a test that runs it on the
  thumbnails would be vacuous rather than wrong — which is worse,
  because it passes.
- **Value equality with duplicates is a reachable authoring mistake**
  that this record makes into documented behaviour rather than a defect.
  A reject test is not appropriate; a spec sentence and a unit case that
  pins "two items match" are.
- **Nothing here is ABI-bearing**, and the only runtime path involved
  (`register_for_item_bool_binding`) already exists and already has
  Phase 2 evidence.
