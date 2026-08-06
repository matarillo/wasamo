# M4-Phase 2 — Implementation log

Append-only mixed log: decisions log (mid-implementation judgments) +
CI / verification log (evidence pointers, run ids). Per-task
implementation-gate selections (start) and close artifacts land here per
[implementation-gates.md](../../../procedures/implementation-gates.md)
and the re-decision obligation in [plan.md](./plan.md).

Entries this phase carries by obligation, named here rather than
discovered:

- **T2's call-site audit table** — zero `visual_rect` readers left on
  the input path, both readers named.
- **T3's and T9's structural side-effect enumerations** — what a
  handler's state write pulls in; what subtree removal releases.
- **T7's materialisation-seam enumeration** — every path that
  materialises or removes a subtree, each shown to run the entry / exit
  seam.
- **Stated limits**, recorded with their reason rather than elided —
  the synthesized-touch limit (T11) in the shape of Phase 1's
  synthesized-`WM_DPICHANGED` limit.

---

## T1 — Layout-derived hit rectangles

### Start gate (recorded 2026-08-06, before any source edit)

Read before selecting:
[AGENTS.md](../../../../AGENTS.md),
[implementation-gates.md](../../../procedures/implementation-gates.md),
[plan.md](./plan.md) §T1 and §Cross-task obligations,
[preamble.md](./preamble.md),
[DD-M4-P2-002](../decisions/dd-m4-p2-002-hit-testing-and-generic-click.md),
[constraints.md](../requirements/constraints.md) §1 / §4 / §8, the
[focus spike](../decisions/exploration/focus-traversal-spike.md) close
gate, and the landed source (`wasamo-runtime/src/widget.rs`
`sync_visuals` / `visual_rect_dip` / the ten constructors,
`wasamo-runtime/src/emit.rs` drain layout entry,
`wasamo-runtime/src/lib.rs::window_add_widget`).

**Trap selection.**

| # | Trap | Applies | Reason |
|---|---|---|---|
| 1 | Semantic-migration miss | **yes** | `WidgetNode` gains a field, so every struct literal that builds one is a call-site that must initialise it, and the clip predicate is an exhaustive `WidgetData` match a future widget kind must be forced to classify. Rust makes both compile-error-forcing; the close artifact is the call-site table naming each site with its classification. |
| 2 | Missed side effects | **yes** | A second store lands inside the walk that already writes Composition geometry. Derived effects to enumerate before writing: whether the walk's audited single-Composition-write property survives a node-side write, what a partially failed pass leaves behind, and which tree-mutation paths can leave a stored rectangle out of step with the tree it belongs to. |
| 3 | Parallel/derived data drift | **yes** | Two parallel pairs land at once. The stored DIP rectangle is derived from the same `LayoutNode` value the Composition offset / size are derived from, so it must be written in that same primitive; and the clip predicate restates, in a match, what three constructors state by calling `SetClip`. The second pair cannot be made atomic, so it is pinned by a test that compares the predicate against the live `Visual.Clip()` for **every** widget kind. |
| 4 | Untested authored branch | **yes**, narrow | T1 authors no reject / diagnostic / size branch. It does author the clip predicate's per-kind arms, and an arm that silently answers "does not clip" is exactly the failure T2's clip bound would inherit. Each arm is fired directly by the per-kind cross-check rather than incidentally. |
| 5 | Carry-forward underweighted | **yes** | T2 is the store's first reader and T5's focus indicator is the next writer-adjacent change. The single-writer property, the "no rectangle ⇒ not hit-testable" failure mode, and the partial-pass ordering are invariants later tasks must preserve; each is recorded with a re-trigger criterion rather than left tacit. |
| 6 | Symptom taken at face value | **yes**, low expectation | T1 deliberately runs mutations that must go red (below). A red that is *not* the expected signature, and any failure in the existing suite, is root-caused rather than re-rolled. |
| 7 | Weak GUI evidence | no | T1 renders nothing and launches no host: the deliverable is a node-side store with no consumer, and no Composition write changes, so there is no frame that could differ. **Re-decide if** T1 ends up altering a Composition geometry write or adding a visual. |

**Review lane.** **Full independent review**, as
[preamble.md §Review lanes](./preamble.md) predicts for T1 — a runtime
structural change: a second store written inside the audited lockstep
walk. The lane is confirmed rather than inherited: the change does touch
`sync_visuals`, the pass whose single-write property DD-M4-P1-002's
audit closed. The trap-#4 branch/test check composes into it for the
clip predicate's arms.

**Red-test disposition (discretionary, recorded per
[constraints.md §8](../requirements/constraints.md)).**
[DD-V-029](../../../cross-milestone/decisions/dd-v-029-pure-logic-red-test-obligation.md)'s
narrowed obligation does not name this task — T1 adds no rounding, no
unit conversion and no boundary condition; it stores a value layout
already produced. But the two wrong implementations it could hide are
precisely the ones
[preamble.md §What "green" is worth](./preamble.md) says a green suite
cannot see: storing the **physical** rectangle instead of the DIP one
(invisible at 100%), and storing the **parent-relative** offset instead
of the absolute one (invisible at the root). Mutation witnesses are
therefore run for both, and the run is recorded below rather than
claimed.

**Planned proof obligations** (each closed at the T1 close gate):

1. The call-site table for the new field: every site that constructs or
   writes it, with its classification.
2. The structural side-effect enumeration for adding the store to
   `sync_visuals`, including the partial-pass ordering decision.
3. The clip predicate pinned against `Visual.Clip()` for every widget
   kind, each arm fired directly.
4. Integration assertions that the retained rectangle equals the
   arranged result — at 96 DPI **and** at a scale ≠ 1, since the DIP
   claim is unfalsifiable at 100%.
5. The never-laid-out case asserted, not assumed.
6. The two mutation witnesses shown red.
7. The whole task list re-read at the close gate (the re-audit
   discipline, [plan.md](./plan.md) §Cross-task obligations).

### Close gate (recorded 2026-08-06)

Landed: `WidgetNode::arranged_rect: Option<DipRect>` written in
`sync_visuals`; `WidgetNode::clips_children`; the three `__*_for_test`
accessors; `wasamo-runtime/tests/arranged_rect_integration.rs` (five
tests).

#### #1 — Call-site audit table

Query: `rg "arranged_rect" wasamo-runtime/src/widget.rs` plus
`rg "WidgetNode \{" --include=*.rs` across every crate, to catch a
struct literal outside the widget module that a field addition could
have missed. Every literal that builds a `WidgetNode` is a `Self { … }`
inside `impl WidgetNode` in `widget.rs`; there is none elsewhere, and
the type has no `Default` impl and no `..` shorthand at any site, so
the compiler enumerated the breakage rather than a grep. Each of the
ten failed to compile with `missing field \`arranged_rect\`` until it
was edited.

| Site | `widget.rs` line | Classification | Reason |
|---|---|---|---|
| `rectangle` | 514 | must-initialise → `None` | Constructor; no layout has run |
| `vstack` | 540 | must-initialise → `None` | Constructor |
| `hstack` | 566 | must-initialise → `None` | Constructor |
| `text` | 608 | must-initialise → `None` | Constructor |
| `box_` | 657 | must-initialise → `None` | Constructor |
| `wrap_panel` | 701 | must-initialise → `None` | Constructor |
| `scroll_view` | 761 | must-initialise → `None` | Constructor |
| `grid` | 798 | must-initialise → `None` | Constructor |
| `zstack` | 823 | must-initialise → `None` | Constructor |
| `button_family` | 995 | must-initialise → `None` | Constructor behind `button` / `toggle_button`, which delegate rather than build their own literal |
| `sync_visuals` | 2258 | **the writer** | The only site that assigns a value; DD-002's single writer |

Filtering helpers that could absorb the field silently: none — there is
no `Default`, no `..` update syntax, and no builder. The second
exhaustive surface added by this task, `clips_children`, and the kind
accessor `__kind_name_for_test` both match `WidgetData` with **no `_`
arm**, so a new widget kind cannot reach either as an unclassified
default.

Tests added: the five in `arranged_rect_integration.rs`. Tests
deliberately not added: none for the constructors' `None` beyond case
(c), because `None` is what every literal already had to state.

#### #2 — Structural side-effect enumeration

| Derived effect | Disposition |
|---|---|
| Composition geometry single-pass property (DD-M4-P1-002 audit) | **Preserved.** `SetOffset` / `SetSize` in `wasamo-runtime/src` remain the same six calls, all inside `sync_visuals` (widget.rs 2235 / 2240 outer, 2284 / 2290 button label, 2320 / 2326 ScrollView intermediate); `dip_scale.rs`'s two mentions are doc comments. A node-side field write is not a Composition write, so the audited property is unchanged rather than re-argued. |
| Partial-pass consistency | **Decided, not inherited.** The store trails this node's two Composition writes, so "the node has a rectangle" implies its Visual was written from the same layout result; a pass that fails mid-tree leaves the visited nodes consistent across both stores. |
| Layout invalidation / re-layout triggers | Unchanged — the store rides the existing walk and adds no invalidation path. |
| Visual sibling order, parent-owned slot metadata | Untouched. |
| Tree mutation — attach / detach / re-parent / replace | **No invalidation writer added, deliberately**, because DD-002 fixes exactly one writer. The gap it leaves is closed by reachability rather than by a second write: hit resolution descends `children` from the window root, so a detached node is not reachable at all; every production mutation path re-enters layout inside the same call (`emit.rs` drain → `run_layout_as_window_root_at_scale`; `window::set_root` on install), so no message is dispatched against a tree whose rectangles predate its last mutation; and `lib.rs::window_add_widget` attaches a Visual without ever making the widget a `WidgetNode` child, so a widget attached that way never enters the walk — DD-002's "not hit-testable is the better failure", now true by construction. |
| `ButtonData.label_size`'s three-point write ([constraints §4](../requirements/constraints.md)) | Not touched; T1 writes no label geometry. |
| Per-node `scale` / `raster_scale` caches | Untouched; `commit_scale_recursive` still runs after `sync_visuals`. |

#### #3 — Parallel-data sync

Two parallel pairs, closed differently because only one of them can be
made atomic.

- **Rectangle ↔ Composition geometry.** Both derive from the same
  `computed.offset` / `computed.size` in the same node's section of the
  same walk — the physical write subtracts the parent's absolute offset
  and multiplies, the node store retains the value un-subtracted. One
  primitive, so no drift is expressible.
- **`clips_children` ↔ the constructors' `SetClip`.** Not atomic: one
  is a match, the other is three constructor bodies. Pinned by
  `the_clip_predicate_agrees_with_the_live_visual_for_every_widget_kind`,
  which compares the predicate against the live `Visual.Clip()` for one
  instance of all eleven kinds. The instances identify themselves
  through `__kind_name_for_test` and the covered set is asserted equal
  to the expected set, so an entry that is not the kind it was pulled
  out as fails by name — a positional label would have let the
  predicate and the clip agree on the wrong node.

#### #4 — Branch tests

`clips_children`'s two arms, each fired directly by the per-kind loop:
the clipping arm by the `ScrollView` / `Grid` / `ZStack` instances, the
non-clipping arm by the other eight. Neither is covered incidentally —
every kind is asserted individually with its own name in the message.

#### #6 — Deterministic-failure disposition

No failure was re-rolled. Three reds were produced deliberately and
each was reverted:

| Witness | Edit | Went red | Reading |
|---|---|---|---|
| M1 — physical instead of DIP | multiply each stored component by `target.factor()` | `the_retained_rectangle_is_dip_not_physical` (`width: expected 500, got 750`) | The other four run at 96 DPI where the mutation is the identity, which is the preamble's point: a suite taken only at 100% cannot see this. |
| M2 — parent-relative instead of absolute | subtract `parent_abs_offset` from the stored offset | `every_laid_out_node_retains_its_absolute_dip_rectangle` (`x: expected 35, got 15`) **and** `a_scrollview_child_retains_the_position_it_paints_at` (`expected -50, got 0`) | The second red was not predicted and is correct: the ScrollView content child reaches the same corruption through the translated `child_parent_abs`. |
| M3 — a kind dropped from the coverage table | pull `remove_child(0)` twice on the first iteration so `Box` never enters the set | `the_clip_predicate_agrees_with_the_live_visual_for_every_widget_kind`, naming `Box` as the missing member | Shows the coverage assertion can fail. Its predecessor — `assert_eq!(kinds.len(), 11)` — could not: the table has eleven entries by construction. Found in review, before the test landed. |

Suite state after the reverts: `cargo fmt --all -- --check` clean,
`cargo build --workspace` clean, `cargo test --workspace` 40 binaries
all `ok`, zero `FAILED`. The release clean rebuild is the
retrospective's item 3.

#### #5 — Carry-forward

| Constraint | Evidence | Placement | Re-trigger criterion |
|---|---|---|---|
| **Three existing test files hit-test against a hand-pinned Visual rectangle, not a laid-out tree.** `button_enabled.rs` (three `hit_test_click` calls), `bool_binding_live_propagation.rs` (one), and `togglebutton_runtime_integration.rs` (two, through its `pin_hit_rect` helper) each write `SetOffset` / `SetSize` directly so today's readback lands inside. Those nodes have never been through `sync_visuals`, so their retained rectangle is `None` and the clicks will resolve to nothing the moment the readers switch. | Read at this task's re-audit; each file states the workaround in its own comment | `carry-forward` → recorded in [plan.md](./plan.md) §T2 | T2, unconditionally. Converting them to lay the tree out is part of the migration: re-pinning the store or ignoring the tests would reintroduce the mixed path the complete-migration obligation exists to prevent. |
| **The store's implication is ordering-dependent.** "Has a rectangle" implies "its Visual was written from the same layout result" only while the store trails this node's two Composition writes. | The ordering decision above | `carry-forward` | Any task that adds a geometry write to `sync_visuals` or moves the store — T5's focus indicator is the next candidate, since DD-003 requires the indicator to be drawn through this pass and nowhere else. |
| **A subtree must reach layout before its rectangle is trusted, and there is no invalidation writer to fall back on.** | The reachability argument in the #2 enumeration | `carry-forward` | T7's materialisation seam and T9's `for` regeneration — the two tasks that add paths which create or replace subtrees. If either introduces a path that mutates the tree without re-entering layout inside the same call, the reachability argument stops holding and the gap becomes real. |

#### #7 — Re-decided at close

Still **not applicable**. The task added no Composition write, no
visual, and no host launch; nothing it wrote reaches the screen, so
there is no frame a positive control could compare.

#### Re-audit of the whole task list

Per [plan.md](./plan.md) §Cross-task obligations, the full list was
re-read at this close gate rather than only T1's item.

- **T2** — inherits the hand-pinned-fixture obligation above, becomes
  `clips_children`'s first production caller (at which point
  `__clips_children_for_test` is redundant and should go rather than
  linger as a second entry point), and should decide whether
  `sync_visuals`'s `children.iter_mut().zip(computed.children.iter())`
  deserves a length assertion: the two are built 1:1 by
  `build_layout_child_slots` in the same pass, so a truncation is not
  currently reachable, but `zip` would absorb one silently and the
  result would be a node with no rectangle rather than a compile error.
- **T5** — the `SetOffset` / `SetSize` enumeration DD-003 asks for
  inherits the M4-Phase 1 T3 set unchanged; T1 added none.
- **T13** — [architecture.md §13.1](../../../../docs/architecture.md)
  matches what landed (absolute DIP on the node, one walk two stores,
  no rectangle ⇒ not hit-testable). No divergence to record from this
  task.
- **Cross-task obligation "no new ABI function"** — held. The three
  additions are `#[doc(hidden)]` Rust accessors, not C ABI surface.
- T3, T4, T6 through T12 are unaffected by what this task built.

#### Verification means

The test file reuses `tests/common/mod.rs`'s skip guard **unchanged**,
so the standing obligation to verify a newly authored guard against an
environment that actually lacks the capability
([CLAUDE.md §Testing rules](../../../../CLAUDE.md)) is discharged by the
existing helper rather than re-opened. `tests/common/mod.rs` was not
touched, so the `0x80070005` two-conjunct check
([constraints §8](../requirements/constraints.md)) is intact.

The 144-DPI assertion uses
`__run_layout_as_window_root_at_dpi_for_test`, which drives a
standalone `WidgetNode` tree at an explicit DPI with no window and no
monitor query — so it introduces none of the desktop-range dependency
[constraints §10](../requirements/constraints.md) keeps out of this
phase.

---

## T2 — Single-target hit resolution and the complete geometry migration

### Start gate (recorded 2026-08-06, before any source edit)

Read before selecting:
[AGENTS.md](../../../../AGENTS.md),
[implementation-gates.md](../../../procedures/implementation-gates.md),
[plan.md](./plan.md) §T2 and §Cross-task obligations,
[preamble.md](./preamble.md) (§What "green" is worth, §The migration
obligation, §Review lanes),
[DD-M4-P2-002](../decisions/dd-m4-p2-002-hit-testing-and-generic-click.md),
[DD-M4-P2-001](../decisions/dd-m4-p2-001-event-routing-model.md)
§Recommendation (for the T2/T3 boundary),
[constraints.md](../requirements/constraints.md) §1 / §7 / §8 / §10,
the [T1 close gate](#t1--layout-derived-hit-rectangles) above and the
[T1 retrospective](../retrospectives/t1.md), and the landed source
(`widget.rs` `hit_test_click` / `update_hover` / `visual_rect_dip` /
`clips_children` / `sync_visuals`, `window.rs` the four pointer
message arms, and every test that drives a click).

**Scope re-decided against the code, not inherited from the plan.**
The plan's T2 hypothesis holds, and reading the source added three
items it does not name:

1. **A fourth and fifth existing test stand on the old geometry
   source**, beyond the three T1 named.
   `dpi_scale_matrix_integration.rs::a_stale_descendant_scale_still_hit_tests_where_the_widget_is`
   pins the *one-divisor traversal property* (Phase 1 F-37) — a
   property of the readback path this task deletes. It will stay green
   and its stated reason will be false, which is the trap-#1 shape a
   compiler cannot catch. `iteration_mutation_integration.rs` derives
   its click point from a `Visual` readback; that is a test-side
   physical coordinate, valid at scale 1, and is classified rather than
   changed.
2. **`visual_rect_dip` and the free `visual_rect` lose their last
   callers** when both readers switch. Leaving them is what a
   "complete migration" audit would have to explain away, so their
   removal is the audit's strongest evidence and is part of this task.
3. **The `zip` length question T1's re-audit handed to T2** is decided
   here, because T2 is the store's first reader: a truncated
   `computed.children` would silently produce a node with no rectangle,
   i.e. a silently unhittable widget.

**Trap selection.**

| # | Trap | Applies | Reason |
|---|---|---|---|
| 1 | Semantic-migration miss | **yes** | The geometry *source* migrates. Rust enumerates almost none of it: deleting a private helper compiles, and a test that pinned the readback property stays green. The audit is therefore a grep table over `visual_rect` / `hit_test_click` / `update_hover` across `src`, `tests`, `examples` and `wasamo-dll`, with every site classified — including the tests, which are call-sites of the migrated behaviour even when they still compile. |
| 2 | Missed side effects | **yes** | Changing *which* node a click resolves to changes what the click's effects can be. To enumerate before writing: the disabled-Button arm (today it descends, DD-002 makes it a target that stops the walk), the hover walk's descent (semantics stay T4's), the drain → re-layout → rectangle ordering that decides whether a click is resolved against fresh geometry, entry on a subtree (the readback's precondition row disappears), and `ButtonData.label_size`'s three-point write ([constraints §4](../requirements/constraints.md)), which this task must not touch. |
| 3 | Parallel/derived data drift | **yes** | The resolver consumes two derived facts per node — `arranged_rect` and `clips_children` — the second of which restates three constructors' `SetClip`. T1 pinned that pair with a per-kind test reached through `__clips_children_for_test`; T2's plan predicts that accessor becomes redundant. Whether the pin survives its removal is decided in this task, not assumed. |
| 4 | Untested authored branch | **yes** | Four new arms: reverse-order first-hit, the clip-bounded descent, "no rectangle ⇒ not a candidate" (and its fail-closed form for a clipping node), and edge containment. The last is a **boundary condition**, so [DD-V-029](../../../cross-milestone/decisions/dd-v-029-pure-logic-red-test-obligation.md)'s red-test obligation applies by name, not by discretion. |
| 5 | Carry-forward underweighted | **yes** | T3 inherits the resolution result shape (it needs the ancestor chain), T4 inherits a hover walk deliberately left semantically unchanged, and T5 / T7 / T9 inherit "a rectangle is only trustworthy after the mutation's own layout pass". Each is recorded with a re-trigger criterion. |
| 6 | Symptom taken at face value | **yes** | Converting the fixtures is *expected* to produce reds. Every red is dispositioned as "this fixture stood on the deleted geometry source" with the mechanism named — not re-pinned to make it green, which is the exact move the complete-migration obligation exists to forbid. |
| 7 | Weak GUI evidence | **no** | T2 adds no Composition write and no visual, and its behavioural difference from today — occlusion — is unobservable in the gallery until the lightbox is wired at T10 ([preamble.md](./preamble.md)). A gallery frame taken now would be produced identically by the old and the new resolver, which is the definition of a non-discriminating frame. **Re-decide if** T2 ends up changing a Composition write or an observable gallery click. |

**Review lane.** **Full independent review**, as
[preamble.md §Review lanes](./preamble.md) predicts and as the change
confirms: a runtime structural change (the complete geometry
migration plus a new dispatch shape). The trap-#4 branch/test check
composes into it for the four new arms.

**The T1 corrective, applied.** The T1 retrospective added a one-line
start-gate test: *does this task introduce a new store / unit /
coordinate system, and is its correctness observable at 100%?* T2
introduces none — but it **removes the cancellation** that made the
pointer conversion unobservable at every scale, so the same answer
falls out: a wrong conversion is invisible at 100% and wrong at any
other scale. A non-unit-scale leg is therefore evidence, not garnish,
and it is the plan's own requirement rather than a discretionary
addition.

**Planned proof obligations** (each closed at the T2 close gate):

1. The call-site audit table showing **zero** `visual_rect` readers on
   the input path, covering `src` and the tests that drive clicks.
2. The structural side-effect enumeration, including the disabled-Button
   behaviour change and the fixture-conversion consequences.
3. Pure-logic tests over a constructed **overlapping** tree, with a
   clip case and its agreement leg (the same tree under a non-clipping
   ancestor resolves).
4. The edge-containment red-test witness (DD-V-029), plus witnesses for
   the ordering and clip arms.
5. The staleness fixture: a click resolved correctly *after* a property
   write triggered re-layout.
6. The non-unit-scale fixture: a click at **physical** coordinates
   resolving to the widget whose DIP rectangle contains the converted
   point, with the un-converted point as the negative leg.
7. The `clips_children` accessor question decided and recorded either
   way.
8. The whole task list re-read at the close gate (the re-audit
   discipline, [plan.md](./plan.md) §Cross-task obligations).

### Close gate (recorded 2026-08-06)

Landed: `wasamo-runtime/src/hit.rs` (`DipPoint`, the `HitTree` view,
`contains`, `resolve_topmost`, nine unit tests); `HitTree for
WidgetNode`; `hit_test_click` rewritten as resolve-then-dispatch;
`update_hover` switched to the same store; `visual_rect_dip` and the
free `visual_rect` deleted; the `sync_visuals` child-count assertion;
three fixtures converted from hand-pinned rectangles to real layout;
the DPI-matrix one-divisor test re-documented;
`wasamo-runtime/tests/hit_resolution_integration.rs` (two fixtures).

#### #1 — Call-site audit table

Queries:
`rg "visual_rect" wasamo-runtime/src bindings wasamo-dll wasamoc examples`
and
`rg "hit_test_click|update_hover|clear_hover" wasamo-runtime/src wasamo-runtime/tests examples bindings wasamo-dll`.

**Input path — zero `visual_rect` readers remain.** The single surviving
occurrence anywhere in `wasamo-runtime/src` is `widget.rs:2037`, inside
the doc comment of `__set_geometry_scale_dpi_for_test`, describing the
property that no longer exists. Both helpers were deleted rather than
left unused, so the audit is closed by absence of the symbol, not by an
argument about who calls it.

| Call site | Classification | Reason |
|---|---|---|
| `widget.rs::hit_test_click` | **migrated** | Resolves through `hit::resolve_topmost` over `arranged_rect`; dispatches on the single target |
| `widget.rs::update_hover` / `update_hover_inner` | **migrated** | `inside` now reads `arranged_rect`; walk and semantics unchanged (T4 owns those) |
| `widget.rs::visual_rect_dip` | **deleted** | Last caller gone; it was the readback conversion itself |
| `widget.rs::visual_rect` (free fn) | **deleted** | Only `visual_rect_dip` called it |
| `widget.rs::clear_hover` | ignore-OK | Reads no geometry — it resets state unconditionally on `WM_MOUSELEAVE` |
| `window.rs` × 4 (`WM_MOUSEMOVE`, `WM_MOUSELEAVE`, `WM_LBUTTONDOWN`, `WM_LBUTTONUP`) | ignore-OK | The message arms convert the pointer to DIP and call in; the conversion is unchanged and is now load-bearing rather than cancelling |
| `tests/button_enabled.rs` (3 clicks) | **converted** | Was hand-pinning `SetOffset`/`SetSize`; now lays the Button out and clicks its arranged centre |
| `tests/bool_binding_live_propagation.rs` (1 click) | **converted** | Same |
| `tests/togglebutton_runtime_integration.rs` (2 clicks) | **converted** | Same; `pin_hit_rect` deleted, and the subtree entry now dispatches through the laid-out root |
| `tests/dpi_scale_matrix_integration.rs::a_stale_descendant_scale_still_hit_tests_where_the_widget_is` | **re-documented, assertions kept** | Pinned the deleted one-divisor property. It stays green because the mechanism it measured is gone, so its doc comment and two assertion messages were rewritten to state what it now demonstrates (see #6 for what it still discriminates) |
| `tests/iteration_mutation_integration.rs` (1 click) | ignore-OK | Derives the click point from a `Visual` readback, which is a **test-side physical coordinate** at a scale-1 tree, not a runtime input-path reader. Left unchanged |
| `tests/box_layout_integration.rs`, `tests/wrap_panel_layout_integration.rs`, `tests/dpi_scale_matrix_integration.rs` local `visual_rect` helpers | ignore-OK | Layout assertions against the Composition tree — the Visual is the physical truth and is the right source for those |

**What the compiler enumerated and what it did not.** Deleting the two
helpers forced every *runtime* caller to be dealt with. It forced
nothing on the test side: all five test files above compiled unchanged
after the migration, and three of them would have silently stopped
exercising a click (`arranged_rect == None` ⇒ resolves to nothing, so
the assertions "the callback fired" would have gone red only because
the click no longer landed — a red with a misleading cause). This is
the trap-#1 shape recorded at the start gate, and the grep table is the
artifact because the compiler is not.

#### #2 — Structural side-effect enumeration

| Derived effect | Disposition |
|---|---|
| **Which node receives a click** | Changed by design: one target, the topmost. For the gallery this is behaviour-preserving (no two interactive widgets overlap, only Buttons carry handlers) |
| **Disabled Button** | Behaviour change DD-002 names: it is now a target that stops the walk instead of recursing into children. It occludes what is behind it and dispatches nothing |
| **A Button-family widget with `WidgetNode` children** | **Reachable, unexercised, and narrowed between T2 and T3.** Neither `wasamoc`'s checker nor the IR loader restricts Button children, and [dsl_spec.md §4.16](../../../../docs/dsl_spec.md) shows `Button { slot.row: 0 slot.column: 1 Text { text: "ok" } }` as a legal placement example. A click over that child now resolves to the child and dispatches nothing, where the pre-T2 recursion fired the Button. No `.ui`, example or test in the repo builds one, so nothing observes it — but it is a real narrowing that **T3's bubbling closes**, carried forward below rather than left to be rediscovered |
| **Hover** | Geometry source only. The whole-tree walk, the disabled arm's descent and the transition/animation code are untouched — T4 owns the semantics |
| **`clear_hover`** | Untouched; reads no geometry |
| **Composition geometry writes** | Untouched. `SetOffset`/`SetSize` remain the same six calls inside `sync_visuals`, so DD-M4-P1-002's single-pass audit is preserved and not re-argued |
| **`ButtonData.label_size`'s three-point write** ([constraints §4](../requirements/constraints.md)) | Not touched; T2 writes no label geometry |
| **Entry on a subtree** | The readback's precondition row is deleted: absolute DIP rectangles make a subtree entry well-defined for geometry. What a subtree entry does *not* get is the clip bound of ancestors above the entry point — stated in `hit_test_click`'s doc comment rather than left implicit |
| **Rectangle freshness relative to a state write** | **Measured, not assumed** — see the drain-boundary finding below |
| **`sync_visuals` child-count** | T1's re-audit question closed with a `debug_assert_eq!`: `zip` would absorb a truncation as a node with no rectangle, which under T2 is a silently unhittable widget. It fires nowhere in the suite (1,019 tests, dev profile, `debug_assertions` on) |

**The drain-boundary finding (measured).** `wnd_proc`'s `WM_LBUTTONUP`
arm never calls `emit::drain_if_outermost`; the production call site is
the line after `DispatchMessageW` in `wasamo_runtime::run`
([lib.rs](../../../../wasamo-runtime/src/lib.rs)). So a click delivered
by a direct `SendMessageW` runs the handler and `Signal::set`'s
synchronous reactive drain — the structural rebuild is visible
immediately — but **not** the layout phase, and every `arranged_rect`
keeps its pre-click value. Measured directly with a throwaway probe:
after a `SendMessageW`'d toggle click the target's rectangle was
bit-for-bit unchanged even though the tree had grown a child. Two
consequences, both recorded rather than absorbed:

- The staleness fixture posts its state-writing click and pumps the
  production `run` loop, which is where the drain actually is. It
  exercises the production mechanism instead of a test-side layout call.
- **In production the store cannot be stale when a later message is
  resolved**, because the drain runs at every loop iteration boundary,
  i.e. between the message that wrote state and the next input message.
  That is the mechanism behind DD-002's staleness mitigation, now stated
  rather than assumed — and it is the same boundary DD-001's "one drain
  per dispatch" will have to be reconciled with at T3.

#### #3 — Parallel-data sync

Both derived facts the resolver reads come from the same two sources T1
pinned, and neither gained a second writer.

- **`arranged_rect`** — still written only by `sync_visuals`. T2 adds
  readers, not writers.
- **`clips_children` ↔ the three constructors' `SetClip`** — still not
  atomic, still pinned by T1's per-kind test against the live
  `Visual.Clip()`.

**`__clips_children_for_test` is retained — a deviation from the plan's
T2 bullet, recorded rather than silently taken.** The plan predicted the
accessor becomes redundant once `clips_children` gains a production
caller. Half of that is true: `hit::resolve_topmost` now calls it, so it
is no longer the predicate's only entry point and the "second entry
point" hazard is gone. But the accessor's *evidence* role survives the
prediction: eight of the eleven widget kinds cannot have children, so no
production click can ever exercise their arm of the predicate, and
removing the accessor would have deleted T1's per-kind agreement pin for
those eight rather than replacing it. The alternative considered — 
re-pointing the per-kind test at resolution behaviour — is not available
for exactly those kinds. The accessor's doc comment now states this role;
[plan.md](./plan.md) §T2 records the deviation.

#### #4 — Branch tests, each fired directly

| Authored arm | Test that fires it |
|---|---|
| Reverse-order first-hit (occlusion) | `hit::tests::the_later_of_two_overlapping_siblings_wins_occlusion` |
| Child over parent | `a_child_wins_over_its_parent_when_both_contain_the_point` |
| Container as target | `a_container_is_the_target_when_the_point_is_inside_it_but_in_none_of_its_children` |
| Clip bound, **with its agreement leg** | `a_clip_excludes_an_overflowing_childs_rect_and_the_same_tree_without_clip_resolves_it` — the same rectangles with `clips` false resolve, so the `None` is the clip and not a coordinate error |
| Fail-closed clip with no rectangle | `a_clipping_node_with_no_rectangle_makes_its_subtree_unreachable` |
| No rectangle ⇒ not a candidate, subtree still reachable | `a_node_with_no_rectangle_is_not_a_candidate_but_its_non_clipping_subtree_is_still_reachable` |
| Edge containment (**boundary condition**) | `edge_containment_includes_the_left_and_top_edges_and_excludes_the_right_and_bottom` |
| Path identity (per-item resolution) | `the_returned_path_identifies_which_of_two_identically_shaped_sibling_subtrees_was_hit` |
| Nothing under the point | `nothing_under_the_point_yields_none` |

#### #6 — Deterministic-failure disposition and the mutation witnesses

No failure was re-rolled; the suite never went red except where a
mutation was deliberately introduced. Five witnesses were run and
reverted. The first three were run by the task lead directly rather than
taken from the implementing agent's report.

| Witness | Mutation | Went red | Reading |
|---|---|---|---|
| **W1 — the conversion the cancellation used to hide** | `window.rs` `WM_LBUTTONUP` passes the raw physical pointer instead of the DIP-converted one | `a_click_at_non_unit_scale_resolves_the_widget_whose_dip_rectangle_contains_the_converted_point`, and **nothing else in the workspace** (`--no-fail-fast`) — "positive leg: a click at the converted point must resolve to the right Button exactly once" | This is the phase's central claim made concrete. Phase 1 recorded that *no* test could distinguish a correct conversion from a missing one; after T2 exactly one can, and it is the one written for it. Every other test stays green because they all run at scale 1 |
| **W2 — a stale store** | `sync_visuals` writes `arranged_rect` only when it is `None` | Three: `a_click_after_a_relayout_triggering_state_write_resolves_at_the_new_rectangle`, `a_click_at_non_unit_scale_...`, and `a_stale_descendant_scale_still_hit_tests_where_the_widget_is` | The staleness fixture reddens on its **guard** ("the property write must have moved the target Button"), not on the discriminating leg, because a frozen store makes before and after identical before the legs are reached. Recorded honestly: the guard is what detects this mutation; the leg is what pins the behaviour once the guard passes. That the DPI-matrix test also reddens is the answer to whether it is now vacuous — it is not: it still requires a *second* layout pass to have updated the store |
| **W3 — edge containment** ([DD-V-029](../../../cross-milestone/decisions/dd-v-029-pure-logic-red-test-obligation.md)) | `contains` uses `<=` on the right edge | `edge_containment_...` alone — "right edge must be excluded" | The named obligation for the boundary condition, discharged by name |
| **W4 — paint order** | Children visited in forward instead of reverse order | `the_later_of_two_overlapping_siblings_wins_occlusion` alone | Occlusion is unobservable in the gallery until T10; this is what stands in for it |
| **W5 — the clip bound** | The clip check disabled | `a_clip_excludes_an_overflowing_childs_rect_...` and `a_clipping_node_with_no_rectangle_...` | Both arms of the bound |

**A methodological finding, recorded because it nearly produced a false
artifact.** W3 was first applied through a shell heredoc that never ran
(the interpreter was absent; the shell's own error text was mistaken for
output), and the subsequent test run came back green — a "the mutation
did not go red" result that would have been recorded as a finding about
the test. Reading the file back showed the mutation had never been
written. **A green mutation witness is only evidence once the mutation
is confirmed present in the file**; a witness run must verify the edit
landed, not just that a command exited. This generalises T1's corrective
(an assertion needs an input that reddens it) to the tooling that
produces the red.

Suite state after all reverts, on the post-commit tree:
`cargo fmt --all -- --check` zero exit, `git diff --check` clean,
`cargo test --workspace --no-fail-fast` **41 binaries, 1,019 passed, 0
failed, 0 skipped** (skip 0 means the Compositor was available, so the
integration assertions actually ran). T1's baseline was 1,008; the
eleven added are `hit.rs`'s nine unit tests and the two new fixtures.
The release clean rebuild is the retrospective's item 3.

#### #5 — Carry-forward

| Constraint | Evidence | Placement | Re-trigger criterion |
|---|---|---|---|
| **A click over a Button-family widget's `WidgetNode` child dispatches nothing until bubbling lands.** The shape is authorable today (checker and loader both admit it; [dsl_spec.md §4.16](../../../../docs/dsl_spec.md) shows one) and unexercised by any fixture, so no test observes the narrowing | The §2 enumeration; found by auditing what the single-target rule changes, not by a failing test | `carry-forward` → recorded in [plan.md](./plan.md) §T3 as a required evidence item | **T3, unconditionally.** T3's evidence must include a click on a Button's child activating the Button through the ancestor walk — otherwise the narrowing ships |
| **The reactive drain is at the message-loop boundary, not in the message arm.** DD-001's "one drain per dispatch, after the walk completes" has to be reconciled with a drain that today runs after *every* dispatched message, from `run` | Measured (the throwaway probe and the fixture that needs `run` to re-layout); §2 | `carry-forward` → [plan.md](./plan.md) §T3 | **T3**, which owns the drain boundary. Also any task that writes an integration fixture expecting a synthesised message to re-layout — it will not, unless it pumps `run` |
| **A test that drives a click must lay its tree out.** Hand-pinning a Visual rectangle no longer puts anything inside the hit target; the click silently resolves to nothing | The three converted fixtures | `carry-forward` | Any later task adding a click-driving fixture — T3, T4, T8, T9, T11 all will |
| **`hit_test_click` on a subtree skips the clip bound of ancestors above the entry point.** Geometry is well-defined there now, but confinement is not | `hit_test_click`'s rewritten doc comment | `carry-forward` | T7's modal scopes and any task that resolves from other than the window root |

#### #7 — Re-decided at close

Still **not applicable**. T2 added no Composition write, no visual and
no host launch; the difference from the pre-T2 resolver is occlusion,
which nothing in the gallery can show until the lightbox is wired at
T10. A frame captured now would be produced identically by both
resolvers, which is the definition of a non-discriminating frame.

#### Re-audit of the whole task list

Per [plan.md](./plan.md) §Cross-task obligations, the full list was
re-read at this close gate rather than only T2's item.

- **T3** — gains two required items from the carry-forward above (the
  Button-child bubbling case; the drain-boundary reconciliation). Also
  inherits the resolution result shape: `resolve_topmost` returns the
  **path of child indices**, so the ancestor chain DD-001 wants is the
  set of prefixes of that path and needs no second traversal.
- **T4** — hover is still a whole-tree walk with T2's geometry. When it
  moves to enter/leave against the resolved target, `resolve_topmost` is
  already the function to call, and the disabled arm's descent is the
  behaviour to re-decide against the topmost rule.
- **T5** — `sync_visuals` is unchanged as a geometry writer, so T1's
  ordering carry-forward is intact and still applies to the focus
  indicator.
- **T7 / T9** — T1's "a subtree must reach layout before its rectangle
  is trusted" is now load-bearing rather than latent: T2 made the
  rectangle the hit source. The drain-boundary finding is the mechanism
  that keeps it true, so a materialisation path that does not reach the
  drain would break hit-testing, not only rendering.
- **T10** — the clip bound is landed and unit-tested, so the gallery's
  scrolled hit-testing has its rule; what T10 adds is the first
  production tree where it is observable.
- **T13** — [architecture.md §13.1 / §13.2](../../../../docs/architecture.md)
  match what landed (layout-derived DIP source, one target, reverse
  order, clip bound, every widget a candidate). Two statements in §13.2
  describe behaviour this phase has not reached yet — hover computed
  against the resolved target (T4) and one drain per dispatch (T3) — so
  they are not divergences to record from T2, but they are exactly what
  the phase-close re-verification must check against the landed runtime.
- **Cross-task obligation "no new ABI function"** — held. `hit.rs` is a
  private module; nothing new is exported.
- T6, T8, T11, T12 are unaffected by what this task built.

#### Verification means

Both new fixtures reuse `tests/common/mod.rs`'s skip guard **unchanged**,
so the standing obligation to verify a newly authored guard on an
environment that lacks the capability
([CLAUDE.md §Testing rules](../../../../CLAUDE.md)) is discharged by the
existing helper. `tests/common/mod.rs` was not touched, so the
`0x80070005` two-conjunct check
([constraints §8](../requirements/constraints.md)) is intact.

The non-unit-scale fixture touches the DPI-fixture environment, so
[constraints §10](../requirements/constraints.md)'s recorded
desktop-range dependency was read first (M4-Phase 1 T8: a hosted CI
desktop failed a 1440×960 request twice). The fixture's largest physical
request is **480×320**, below the 720×480 the T8 repair settled on, and
it prints the screen and max-track metrics into the failing assertion so
a runner that cannot honour the rectangle explains itself. It also
follows F-47: it normalises to 96 DPI with an explicitly chosen client
and **asserts** both the realised extent and the committed scale rather
than assuming the developer's monitor. Both fixtures derive their scale
factor from the value the runtime committed, not from the constant they
requested — written as the constant, the fixture's own
`assert_ne!(factor, 1.0)` would have been true by construction, which is
the tautological-assertion shape T1's retrospective flagged.
