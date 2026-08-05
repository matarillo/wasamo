# DD-M4-P2-002 — Hit testing, occlusion, and generic click handling

**Status:** Accepted
**Phase:** M4-Phase 2
**AC:** AC1 (click handling on non-`Button` widgets, per-item handlers)

## Context

Three questions that have to be answered together, because each one's
answer constrains the others: **where the geometry comes from**, **which
widget wins when several are under the pointer**, and **which widgets are
eligible at all**. The fourth, **what happens to `Button.clicked`**, is
the explicit obligation
[m4-interaction-intake.md](../../../../docs/notes/m4-interaction-intake.md)
requirement 4 sends here.

Today `hit_test_click_inner` reads each node's rectangle back off its
live `Visual`, converts to DIP by dividing by the traversal root's
scale, and fires **every** Button-family widget whose rectangle contains
the point as it recurses. That was correct in M3 because no two
interactive widgets overlapped and only Buttons reacted. Both premises
end in this phase: `ZStack` gains a scrim over live content, and any
widget becomes clickable.

Two measured facts from M4-Phase 1 frame the geometry question
([handoff](../../phase-1/implementation/handoff.md), T5):

- **The two conversions on this path cancel.** The pointer is divided by
  the window's scale in `wnd_proc`, and the readback is divided again.
  Because hit-testing sources its geometry from the visual tree, the two
  are symmetric — *"until then no test can distinguish a correct row 9
  from a missing one"*.
- **The single-divisor rule is a precondition on the public entry, not
  an invariant the runtime maintains.** `hit_test_click` takes its
  divisor from the receiver, so entering on a subtree compares two
  spaces; `scale` is private, so a caller cannot supply the right
  divisor even knowingly. Every production caller happens to enter at
  the window root.

The same handoff names this phase as the landing site for layout-derived
hit rectangles, and the owner has agreed that as the baseline
([framing.md](../requirements/framing.md) agreement 2) — as a working
hypothesis, revisable on new evidence, with the readback option kept in
the comparison.

## Sub-issues

- **Geometry source** — Visual readback or a layout-derived cache; and,
  if the cache, where the retained rectangle lives and who writes it.
- **Target selection** — how many widgets a pointer event resolves to.
- **Clipping** — what bounds containment when content extends past a
  clipping container.
- **Eligibility** — which widgets can be a target.
- **`Button.clicked`** — preserved, renamed, or generalised.
- **Per-item resolution** — how a click inside repetition names its item.
- **Minimum hit target** — whether M4 has one.

## Options

### Geometry source

- **H1 — keep the Visual readback.** Hit-testing continues to ask the
  Composition tree where each node is.
- **H2 — layout-derived DIP rectangles.** Layout retains each node's
  arranged rectangle; hit-testing reads that and never touches the
  Visual.

### Target selection

- **T1 — every containing widget fires** (today's behaviour).
- **T2 — one target: the topmost containing widget**, then DD-001's
  bubble walk up its ancestors.

### Eligibility

- **E1 — every widget with a visual is a candidate.**
- **E2 — an explicit attribute is required** to be hit-testable.
- **E3 — only widgets carrying a click handler** (plus Button-family)
  are candidates.

## Comparison

### Geometry source: H2 over H1

- **H1's correctness stops being free the moment anything else in this
  phase lands.** The cancellation above holds only while hit geometry
  *is* the visual geometry. The clip bound on containment, a
  DIP-denominated minimum target, or any other rule expressed in layout
  terms breaks the symmetry, and the residual error is proportional to
  `scale - 1` — it is invisible at 100% and wrong at 150%. That is the exact signature
  M4-Phase 1 spent a phase removing from rendering; H1 leaves it in
  input.
- **H1 carries an unenforceable precondition.** The single-divisor rule
  cannot be expressed in the type system and is already violated by a
  test in this repository. H2 deletes the row: with rectangles cached in
  DIP at layout time, there is no divisor and no traversal root to be
  wrong about.
- **H2 makes the hard part pure logic.** Occlusion, per-item resolution
  and any future minimum-target rule become "rectangles + order →
  target", testable with no Compositor. Under H1 the same rules are only
  reachable through the OS.
- **H1's advantage is that it cannot go stale**, because it reads the
  thing that is actually on screen. H2 introduces a cache with the same
  hazard class as the geometry-scale cache M4-Phase 1 spent T1–T6
  disciplining: it is only correct if it has exactly one writer and
  every path that changes geometry goes through it. That is a real cost
  and the reason the migration obligation below is part of the decision
  rather than an implementation detail.
- **Cost of getting it wrong is asymmetric.** A stale H2 cache produces
  a *visible, scale-independent* error (clicks land on the wrong widget
  everywhere). A wrong H1 conversion produces an error that only appears
  on scaled monitors. The first kind is caught by the fixture; the
  second is what shipped undetected for three milestones.

**H2, with a migration obligation that is part of the decision:** the
change is complete or it is not made. No intermediate state may have one
path reading the cache and another reading the Visual, because that is
where the cancellation argument silently stops applying to only some
rows. The input path has **two** readers today — `hit_test_click` and
`update_hover`, the latter entered from three window messages — and the
obligation covers both. The close artifact is a call-site audit showing
**zero** `visual_rect` readers on the input path.

**Where the retained rectangle lives and who writes it.** Layout
computes on a transient tree: the layout entry builds a `LayoutNode`
tree, arranges it, applies the results, and drops it. The arrange pass
therefore cannot be the retained rectangle's writer — its output does
not outlive the pass. The rectangle is retained **on the widget node**
and written by the **same lockstep walk that applies layout results to
the Visual** (`sync_visuals`): one walk, two stores — the physical
offset and size to the Composition visual, the absolute DIP rectangle to
the node. This inherits the discipline that walk already carries (every
Composition geometry write in the runtime happens there and nowhere
else — M4-Phase 1 T3) instead of inventing a second lockstep walk, which
would be the parallel-data shape the runtime keeps eliminating. Writing
a node-side field is not a Composition write, so the walk's audited
single-pass property is preserved; the enumeration that closes it now
covers both stores.

### Target selection: T2 over T1

T1 is not a considered design; it is what falls out of recursing without
a stopping rule, and it is wrong as soon as widgets overlap — a click on
a scrim would also fire the thumbnail underneath it.

T2's important property is what it makes *unnecessary*: **occlusion is
not a rule about scrims.** If hit-testing resolves exactly one target,
the topmost, then lower `ZStack` siblings are never candidates and
nothing has to block them. The framing's design condition 2 ("a scrim
blocks clicks to lower siblings") is discharged by target selection, not
by a scrim concept, and no widget needs to declare itself a blocker.
This is also what keeps DD-001 free of a capture phase.

"Topmost" is defined by paint order, which the runtime already fixes:
within a container, later children paint over earlier ones. Hit-testing
is therefore the reverse walk — children in reverse order, first
containing node wins — which is the same total order the screen shows.

**Containment is bounded by ancestor clips.** "Hit-testing resolves
what the screen shows" has a second half: a container that installs a
clip (`ScrollView`, `Grid`, `ZStack` — the kinds that install
`InsetClip{0,0,0,0}` on their outer Visual) paints nothing of its
subtree outside its own rectangle, so nothing of its subtree outside
that rectangle may be a target. The walk descends into a clipping
container only where the point lies inside it. A non-clipping container
(the stacks, `WrapPanel`) paints overflowing children — the toolbar's
measured overlap at narrow widths is exactly that — and its overflowing
children are accordingly hittable. Without this rule the gallery itself
misresolves: content scrolled above the `ScrollView` viewport retains
rectangles under the toolbar, and the content cell paints later than the
toolbar cell, so a toolbar click would resolve to an invisible
thumbnail. The clip bound follows from the widget kind plus the same
retained rectangle; it adds no second geometry store.

### Eligibility: E1 over E2 and E3

- **E3 is the tempting one and it is wrong**, because it conflates
  *being a target* with *having a handler*. Under E3 a scrim with no
  handler is not a candidate, so clicks pass through it to the content
  below, and the framing's condition 2 fails. Occlusion requires
  non-interactive widgets to be targets.
- **E2 makes the common case verbose** and creates an authoring trap
  where a scrim silently fails to block until someone remembers the
  attribute. It also adds a spelling DD-005 would have to design, for a
  behaviour E1 gives by default.
- **E1's cost** is that a widget cannot opt *out* of being hit — there
  is no click-through. Nothing in A or B4 wants it. Recorded as a
  forward exposure rather than pre-built.

Under E1 the eligible set is "every widget with a visual", which is
every widget: the target is always well-defined, and whether anything
*reacts* is DD-001's bubbling question, not this one.

### `Button.clicked` — generalised, not duplicated

The intake obligation asks for the relation to be made explicit. Three
readings were available: keep `clicked` Button-only and introduce a
second name for other widgets; keep both with different semantics; or
make `clicked` one signal that any widget can carry.

The last is chosen. A second name would be a distinction with no
semantic content — both mean "the user activated this" — and would
force every later consumer (M5's widget set, the accessibility tree) to
learn which spelling a given widget uses. Button's existing behaviour is
preserved exactly: `enabled: false` suppresses dispatch
(DD-M3-P1-005), hover / pressed background states are Button-family
presentation and do not generalise. What generalises is the signal, not
the appearance.

**Consequence for hit-test eligibility, stated because it is the one
place the two interact:** a disabled Button is still a *target* (it
occludes), and it does not dispatch. Under today's code the disabled arm
recurses into children instead; under T2 it stops the walk. That is a
behaviour change and is deliberate — a disabled control that lets clicks
through to whatever is behind it is a defect, not a feature.

### Per-item resolution

The target is a node in the live tree; repetition materialises one
subtree per item. The item identity is therefore already carried by the
tree structure, and per-item resolution needs no hit-testing support at
all — it is a question of what the handler can *read*, which is DD-005.
Recorded here so the reader does not look for it in the hit path.

### Minimum hit target

**Not adopted in M4.** A DIP-denominated minimum touch target is a real
accessibility obligation, but it interacts with layout (a widget whose
hit area exceeds its visual area overlaps its neighbours' hit areas),
and this phase does not open layout. Deferred with a trigger: the first
touch-primary surface, or M4-Phase 11's accessibility pass, whichever
first. Recorded rather than silently omitted because H2 is what makes it
cheap later — under H1 it was not expressible.

## Recommendation

- **H2** — layout retains each node's arranged **DIP** rectangle on the
  widget node, written by the same lockstep walk that applies layout
  results to the Visual (one walk, two stores); hit-testing reads that.
  **Complete migration:** zero `visual_rect` readers remain on the input
  path — `hit_test_click` and `update_hover` both — evidenced by a
  call-site audit.
- **T2** — one target, the topmost containing widget, found by a
  reverse-order walk **bounded by ancestor clips**: a clipping
  container's subtree is hittable only inside that container's
  rectangle, and a non-clipping container's overflowing children are
  hittable, matching paint. DD-001 then bubbles from the target.
  Occlusion is a consequence, not a rule.
- **E1** — every widget with a visual is a candidate; reacting is a
  separate question.
- **`clicked` is one signal on every widget.** Button keeps its
  `enabled` suppression and its hover / pressed presentation; a
  disabled Button occludes and does not dispatch.
- **No minimum hit target in M4**, deferred with a trigger.

## Forward-compat exposure

- **The cache has exactly one writer** — the lockstep walk that applies
  layout results — by the same discipline as the geometry-scale cache.
  Any path that attaches, re-parents or re-materialises a subtree must
  reach layout before its rectangles are trusted. `lib.rs::window_add_widget` is
  already outside that boundary (M4-Phase 1 T3, F-24) and stays a
  cleanup candidate; under H2 a widget attached that way is not
  hit-testable, which is a *better* failure than being hit-testable at a
  stale rectangle.
- **Click-through remains addable** as an opt-out attribute if a
  concrete case appears.
- **A minimum hit target remains addable** and lands inside the same
  rectangle cache, not in the space definition.
- **Screen-coordinate mapping is untouched.** The IME caret and
  top-layer placement continue to read the visual tree, which is
  physical, per M4-Phase 1's recorded payoff. H2 does not move that
  boundary.
- **M4-Phase 4's scrollbar drag** will want pointer capture (DD-001
  defers it) and hit rectangles that account for scroll offset. The
  cache stores arranged rectangles, which already include the offset a
  `ScrollView` applies, and the clip bound is what keeps scrolled-out
  content unhittable — so no additional contract is needed; recorded so
  it can be re-derived rather than rediscovered.

## Technical risk re-evaluation

- **Staleness is the real risk, and it is not hypothetical.** The
  identical hazard has been hit twice in this codebase (the
  geometry-scale cache, the uplifted rlib). Mitigation: single writer,
  the complete-migration obligation above, and a fixture assertion that
  a click resolves correctly *after* a property write triggers
  re-layout, which is the path where a cache would go stale.
- **The behaviour change on disabled Buttons is user-visible.** It is
  small and defensible, but it is a change, and it is recorded here
  rather than discovered in a retrospective.
- **The evidence for T2 cannot be a single frame.** "Clicking the scrim
  did not open the lightbox" is equally produced by a broken click path.
  The positive control pairs it with the same click at the same
  coordinates with the scrim absent, which must open it
  ([framing.md](../requirements/framing.md) §検証方針 control C).
