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

---

## T3 — Propagation and the drain boundary

### Start gate (recorded 2026-08-06, before any source edit)

Read before selecting:
[AGENTS.md](../../../../AGENTS.md),
[implementation-gates.md](../../../procedures/implementation-gates.md),
[plan.md](./plan.md) §T3 and §Cross-task obligations,
[preamble.md](./preamble.md) (§What "green" is worth, §The sequencing
thesis, §Review lanes),
[DD-M4-P2-001](../decisions/dd-m4-p2-001-event-routing-model.md),
[DD-M4-P2-002](../decisions/dd-m4-p2-002-hit-testing-and-generic-click.md)
§Recommendation,
[DD-M4-P2-005](../decisions/dd-m4-p2-005-dsl-handler-surface.md) §Which
keys the runtime keeps / §IR and compiler impact,
[constraints.md](../requirements/constraints.md) §3 / §4, the Moment-1
normative text ([architecture.md §13.2](../../../../docs/architecture.md),
[dsl_spec.md §4.19](../../../../docs/dsl_spec.md) and §4.8's disabled
contract), the [T2 close gate](#t2--single-target-hit-resolution-and-the-complete-geometry-migration)
and the [T2 retrospective](../retrospectives/t2.md), and the landed
source (`widget.rs` `hit_test_click` / `build_layout_tree` /
`sync_visuals`, `hit.rs`, `window.rs`'s pointer arms, `emit.rs`,
`registry.rs`, `reactive.rs::Signal::set`, `lib.rs::run`).

**Scope re-decided against the code, and three facts were measured
before the approach was chosen** (throwaway probes, run and discarded).
Two of them contradict what T2 handed forward, so they are recorded
first.

1. **The Button-child narrowing T2 carried forward does not exist, and
   the evidence item the plan derived from it cannot be built.**
   `build_layout_tree` maps `Button` / `ToggleButton` / `Text` /
   `Rectangle` to `LayoutNode::rectangle(…)`, which carries **no
   children**, so a Button's `WidgetNode` child never enters the layout
   tree. Measured on `Button { text: "outer" Text { text: "inner" } }`:
   `wasamoc check` accepts it and the loader builds the `Text` as a child
   node — and then

   - in the **release** profile the child's `arranged_rect` is `None`, so
     it is not a hit candidate at all, the click resolves to the
     **Button**, and the Button fires (measured: one dispatch, at a
     committed factor of 1.25 with the pointer conversion applied). There
     is therefore no pre-T3 narrowing to close;
   - in the **debug** profile — which is how `cargo test` runs — T2's
     `sync_visuals` child-count `debug_assert_eq!` fires during
     `wasamo_load_ui` (`widget.rs:2326`) and the process aborts
     (`STATUS_STACK_BUFFER_OVERRUN`, a non-unwinding panic across the FFI
     boundary). **A fixture that builds this shape cannot run**, so the
     plan's "a click on a Button's child activating the Button through
     the ancestor walk" is unbuildable as written.

   Disposition at the close gate; the plan's T3 evidence item is
   replaced, not dropped.

2. **`clicked` on a non-Button widget is already accepted end to end
   except for runtime dispatch.** `wasamoc check` accepts
   `Box { clicked => { … } }` (no per-kind signal admission rule exists),
   `lower` / `emit` carry it, and the IR loader attaches it — measured:
   the loaded `Box` node holds `inline_handlers = ["clicked"]` and a
   laid-out rectangle, so it is the resolved hit target. Clicking it does
   nothing today, because `hit_test_click` dispatches only where
   `button_data_mut()` is `Some`. **T3 is the piece that makes an
   already-authorable handler fire**, which makes the generic-dispatch
   fixture a red-before / green-after witness rather than a new surface.
   T8's "checker widening" is correspondingly smaller than predicted —
   carried to T8 in the close gate's re-audit.

3. **`wasamo_signal_connect` already admits any widget and any signal
   name** (`abi.rs`), so the host-facing generic `clicked` path needs no
   ABI change either — the cross-task "no new ABI function" obligation is
   untouched by this task.

**What T3 therefore is.** Not "add bubbling to a working generic
dispatch", but: replace the Button-family dispatch gate with the
handler-carrying test DD-001 defines, walk target-then-ancestors under
consumption, and reconcile the drain boundary T2 measured.

**Trap selection.**

| # | Trap | Applies | Reason |
|---|---|---|---|
| 1 | Semantic-migration miss | **yes** | No enum or schema gains a variant, so the compiler enumerates nothing here — but the *decision* "does this widget react to a click" migrates from one site (`button_data_mut()`) to a predicate over three producers (`clicked_fn`, `inline_handlers`, the signal registry). Every site that decides whether a click reacts, and every producer of a `clicked` handler, is audited as a call-site table: a producer left out of the predicate is a handler that silently never fires, and a producer left out of the invocation is a node that consumes without running anything |
| 2 | Missed side effects | **yes** | Dispatch shape change. To enumerate before writing: the synchronous reactive drain inside a handler (a structural rebuild mid-dispatch), the layout phase that does **not** run there (T2's drain-boundary finding), `update_hover` running after `hit_test_click` in the same message arm against a tree the click may have rebuilt, registry teardown on subtree removal, and the `ButtonData.label_size` three-point write ([constraints §4](../requirements/constraints.md)), which this task must not touch |
| 3 | Parallel/derived data drift | **yes** | The consumption predicate and the invocation are a derived pair over the same three producers: if they drift, a node either consumes without running a handler or runs one without consuming. They are made one snapshot value taken once per node, so the pair cannot be edited apart |
| 4 | Untested authored branch | **yes** | New arms: generic (non-Button) dispatch, consumption ending the walk, the disabled-Button arm that suppresses **without** consuming, and "no handler anywhere ⇒ nothing happens". Each ships with a test that fires it directly, and each is put under a deliberately wrong implementation shown to redden it. DD-V-029's named obligation is **not** triggered (no rounding, unit-conversion or boundary-condition branch is added — edge containment stayed T2's), so these witnesses are the trap-#4 / #6 artifact rather than that decision's |
| 5 | Carry-forward underweighted | **yes** | T4 inherits the hover-versus-target ordering inside the arm; T5 inherits the walk for the key path; T8 inherits finding 2 above and the disabled-Button assertions; T13 inherits the drain-boundary wording check and the §4.16 spec-example divergence from finding 1 |
| 6 | Symptom taken at face value | **yes** | Finding 1's abort is a deterministic crash on an authorable shape. It is dispositioned with its mechanism named, not worked around by silently avoiding the shape |
| 7 | Weak GUI evidence | **no** | T3 adds no Composition write, no visual and no host launch. Its behavioural difference — which handler runs — is observable only through state read-back, and the gallery has no ancestor handler and no non-Button handler until T10, so a frame captured now would be produced identically by the pre- and post-T3 dispatch. **Re-decide if** T3 ends up changing a Composition write or an observable gallery click |

**Review lane.** **Full independent review**, as
[preamble.md §Review lanes](./preamble.md) predicts and as the change
confirms: a runtime structural change (dispatch shape, consumption, the
drain boundary). The trap-#4 branch/test check composes into it.

**The T2 correctives, applied.** T2's retrospective added two start-gate
lines. Both are answered here rather than at the close:

- *Which tests pin a property this task deletes, and do they go red or
  stay green with a false reason?* The property being deleted is
  "**only** a Button-family target dispatches". The tests that pin it —
  `button_enabled.rs`, `bool_binding_live_propagation.rs`,
  `togglebutton_runtime_integration.rs`, `iteration_mutation_integration.rs`
  and `hit_resolution_integration.rs` — all click Buttons that carry a
  handler, so they stay green **and their stated reasons stay true**: a
  Button with a handler still consumes at the target. The one shape whose
  reason would change is a click on a Button with *no* handler inside a
  container that has one; no existing test builds it, and the new
  fixtures do.
- *A mutation witness is evidence only once the mutation is confirmed
  present in the file.* Every witness at the close gate is applied with
  an edit, read back, run, then reverted and re-read.

**Planned proof obligations** (each closed at the T3 close gate):

1. The call-site audit table over the three `clicked` producers and every
   dispatch-decision site.
2. The structural side-effect enumeration, including what the synchronous
   rebuild inside a handler does to the rest of the message arm.
3. Pure-logic tests for the dispatch chain's order (target first, root
   last).
4. Integration fixtures: generic dispatch on a non-Button widget; the
   ancestor walk with DD-001's named consumption control (difference and
   agreement legs); the disabled Button suppressing without consuming; a
   handler that removes its own subtree; a host-registered listener
   through `wasamo_signal_connect` consuming the walk.
5. Mutation witnesses for each new arm, each read back before it is run.
6. The drain-boundary reconciliation, decided and recorded either way.
7. Finding 1 dispositioned with an owner-visible route, and the plan's T3
   evidence item revised rather than quietly dropped.
8. The whole task list re-read at the close gate (the re-audit
   discipline, [plan.md](./plan.md) §Cross-task obligations).

### Close gate (recorded 2026-08-07)

Landed: `hit::dispatch_chain` (the target-first walk order, two unit
tests); `hit_test_click` rewritten as capture-chain → walk →
consume-at-the-first-node-that-runs-anything; `ClickDisposition` /
`ClickedHandlers` / `click_disposition_for` / `run_clicked_handlers` as
module items beside it; `WidgetNode::button_data` (the immutable
counterpart the read-only disposition needed); a residual note on
`set_clicked`; and `wasamo-runtime/tests/event_routing_integration.rs`
(five fixtures).

#### #1 — Call-site audit table

The migrating decision is **"does this widget react to a click"**, which
moved from one site (`button_data_mut()` at the resolved target) to a
predicate over three producers. The compiler enumerates none of that —
no type changed — so the artifact is the grep table.

Queries:
`rg '"clicked"' wasamo-runtime/src bindings wasamo-dll wasamoc/src examples`,
`rg "inline_handlers|set_inline_handler" wasamo-runtime/src`,
`rg "clicked_fn|set_clicked" wasamo-runtime/src bindings examples`,
`rg "enqueue_signal|signal_tokens_for" wasamo-runtime/src`,
`rg "hit_test_click" wasamo-runtime/src wasamo-runtime/tests examples bindings wasamo-dll`,
`rg "button_data_mut\(\)|button_data\(\)" wasamo-runtime/src`.

**The three producers, each shown to reach both halves of the decision.**
A producer in the predicate but not the invocation is a node that
consumes without running anything; a producer in the invocation but not
the predicate is a handler that silently never fires. Both halves read
one snapshot, so the table is over producers rather than over the two
sites.

| Producer | Written by | In the predicate | In the invocation |
|---|---|---|---|
| `ButtonData::clicked_fn` | `WidgetNode::set_clicked` (Rust-native) and `wasamo_button_set_clicked` → **no**: that ABI entry forwards to `wasamo_signal_connect`, so the C host path is the *registry* producer, not this one | `has_native`, Button-family only | `run_clicked_handlers` step 1 |
| `WidgetNode::inline_handlers` where the signal is `"clicked"` | `set_inline_handler`, whose only production caller is `ir_loader.rs:3092` — attached for **every** widget kind, with no per-kind filter | `inline`, cloned at snapshot time | `run_clicked_handlers` step 2 |
| A registry `Signal` entry named `"clicked"` | `registry::add_signal` via `wasamo_signal_connect` (and `wasamo_button_set_clicked`, which is a thin forwarder) — admits **any** widget and any name | `has_host_listener` | `run_clicked_handlers` step 3, which re-queries the registry rather than replaying captured tokens |

**Every site that decides whether a click reacts.**

| Call site | Classification | Reason |
|---|---|---|
| `widget.rs::hit_test_click` | **migrated** | The only decision site. Was "target is an enabled Button-family widget"; is now the chain walk over `click_disposition_for` |
| `widget.rs::click_disposition_for` | **new, sole predicate** | Read-only; the one place the three producers are tested |
| `widget.rs::run_clicked_handlers` | **new, sole invocation** | Reads the snapshot the predicate produced |
| `window.rs:959` (`WM_LBUTTONUP`) | ignore-OK | The only production caller of `hit_test_click`; unchanged, and it still converts the pointer to DIP before calling in |
| `widget.rs::update_hover` / `update_hover_inner` / `clear_hover` | ignore-OK | Hover is presentation state, not a `clicked` dispatch. Untouched — T4 owns its semantics |
| `widget.rs` five other `button_data_mut()` sites (`set_clicked`, `update_button_label`, `update_button_enabled`, `update_toggle_button_checked`, the hover arm) | ignore-OK | Property setters and hover; none of them decides whether a click dispatches |
| `emit.rs::enqueue_signal` | ignore-OK | Generic over signal name; the `"clicked"` caller is the one above. Its early return on an empty token list is what made the pre-T3 unconditional call a no-op |
| `abi.rs::wasamo_signal_connect` / `wasamo_button_set_clicked` | ignore-OK | Registration, not dispatch. Neither gained a constraint — `wasamo_signal_connect` already admitted any widget and any signal name, so **the phase's "no new ABI function" obligation is untouched** |
| `tests/button_enabled.rs`, `tests/bool_binding_live_propagation.rs`, `tests/togglebutton_runtime_integration.rs` | ignore-OK, re-read | These call `hit_test_click` **on the Button itself** as the entry root, so their dispatch chain is one node long and the walk is a no-op extension. All three still assert what their comments claim |
| `tests/iteration_mutation_integration.rs:334`, `tests/hit_resolution_integration.rs` | ignore-OK, re-read | Click Buttons that carry a handler, which still consume at the target |

**What the compiler enumerated: nothing.** No type changed, so every one
of the rows above compiles either way. The one shape whose *reason*
would have gone false — a click on a Button with no handler inside a
container that has one — is not built by any pre-existing test, and is
now built by `event_routing_integration.rs`.

#### #2 — Structural side-effect enumeration

| Derived effect | Disposition |
|---|---|
| **Which node's handler runs** | Changed by design: the target's if it has one, otherwise the nearest ancestor with one. Behaviour-preserving for the gallery, where every handler is on a Button that is itself the resolved target |
| **A widget with a handler and no Button data** | Now dispatches. This is not a new authored surface — `wasamoc check`, `lower`, `emit` and the IR loader already accepted and attached it (start gate finding 2) — it is the runtime half arriving |
| **A Button-family widget with no handler at all** | Now transparent to propagation instead of terminal. Pre-T3 it reached `enqueue_signal` unconditionally, which was a no-op with no listeners; post-T3 the walk continues past it |
| **A disabled Button-family widget** | Suppresses its own dispatch and does **not** consume (`docs/dsl_spec.md` §4.8 / §4.19). Its occlusion is unchanged: it is still the resolved target, so nothing beneath it is reachable |
| **The synchronous reactive drain inside a handler** | A handler's state write drains its effects synchronously at zero batch depth (`reactive::Signal::set`), so a conditional or `for` effect can rebuild or remove subtrees **during** dispatch. Handled structurally rather than defended against: the chain is captured before the first handler runs, the inline bodies are cloned into the snapshot, and the node is never dereferenced after the native closure is entered. Consumption is what makes the ancestor half unreachable — the walk returns at the first node that runs anything, so no ancestor is ever visited after user code has run |
| **`update_hover` after `hit_test_click` in the same message arm** | Untouched, and now runs against a tree the click may have structurally rebuilt but **not** re-laid-out (the drain's layout phase is at the message-loop boundary, below). A rebuilt-but-unarranged node has no `arranged_rect`, so it is not "inside" anything and the hover walk skips it — the T1/T2 intended failure, not a new one. Recorded for T4, which owns hover |
| **Registry teardown on subtree removal** | `widget_destroy` → `for_each_ptr` → `registry::remove_for_widget` severs registrations with the subtree. This is what makes step 3's re-query correct rather than merely safe: a node removed by its own handler enqueues nothing. Pinned by F4 |
| **`ButtonData.label_size`'s three-point write** ([constraints §4](../requirements/constraints.md)) | Not touched. T3 writes no label geometry |
| **Composition geometry writes** | Untouched. `SetOffset` / `SetSize` remain the same six calls inside `sync_visuals`, so DD-M4-P1-002's single-pass audit is preserved |
| **The reactive drain's position** | Unchanged, deliberately — see the reconciliation below |

**The drain-boundary reconciliation** (T2's carry-forward, and this
task's by the plan). DD-M4-P2-001 asks for **one drain per dispatch,
after propagation completes**; T2 measured that the production call site
is the line after `DispatchMessageW` in `wasamo_runtime::run`, i.e. one
drain per *message*, not per dispatch. **Resolution: the drain stays
where it is, and the ADR's requirement is met without moving it.**

- The requirement's stated purpose is that "an event must not be
  delivered into a tree the same event has already invalidated"
  ([architecture.md §13.2](../../../../docs/architecture.md)). What
  would violate it is a drain *between propagation steps*. There is
  none: the walk contains no drain point, and it ends at the first node
  that runs anything.
- One drain per message is a *superset* of one drain per dispatch. A
  message that dispatched nothing leaves the queue empty and no window
  dirty, so its drain is a no-op; a message that dispatched runs exactly
  one drain, after `hit_test_click` has returned.
- **Moving the drain into the `WM_LBUTTONUP` arm was considered and
  rejected on a concrete hazard, not on cost.** Phase 3 of the drain
  invokes host callbacks, and `abi_spec` §6 permits a callback to "freely
  call back into the ABI" — including `wasamo_window_destroy`. Inside
  `wnd_proc` the runtime holds a `&mut WindowState` derived from the
  window's user data and returns through it; draining there would let a
  host callback free that allocation mid-message and re-enter
  `DestroyWindow` from inside a message handler. The message-loop
  boundary is the point at which nothing is borrowed, which is what
  makes it the safe point abi_spec §6 names.
- **The consequence for fixtures is unchanged and stays a carry-forward**:
  a synthesised `SendMessageW` observes an inline handler's synchronous
  writes but not the layout phase or a host listener; a fixture that
  needs either pumps `run`. Both this file's fixtures and
  `hit_resolution_integration.rs` do so through the same helper.

No divergence to record against the normative text: "the reactive drain
runs once, after the walk completes" is true of the landed runtime.
`docs/dsl_spec.md` §4.19's sentence that a handler's writes are
"propagated to quiescence **once, after propagation completes**" is also
satisfied, but for a reason worth stating so T13 does not re-derive it:
the propagation that could still be in flight is empty, because the
handler that wrote the state consumed the event. The reactive
propagation itself is synchronous **inside** the handler, per
[constraints §3](../requirements/constraints.md)'s non-batched drain
contract, which this task does not change.

#### #3 — Parallel-data sync

The consumption decision and the invocation are the derived pair, and
they are made **one value taken once per node**: `click_disposition_for`
returns `ClickDisposition`, `hit_test_click` branches on it, and
`run_clicked_handlers` receives the same `ClickedHandlers` it carries.
Neither side re-derives "does this node have a handler" from the live
node, so they cannot be edited apart. `ClickedHandlers` is non-empty by
construction — the `NoHandler` variant exists so an empty snapshot is
unrepresentable.

The `inline` field is the second parallel-data point and is the same
shape: it is a **clone taken at snapshot time**, not a re-read, because
the node it came from may not exist by the time it is evaluated.

#### #4 — Branch tests, each fired directly

| Authored arm | Test that fires it |
|---|---|
| Generic dispatch: a non-Button target's handler runs | `a_click_on_a_widget_without_a_handler_stays_silent_while_a_click_on_a_sibling_with_one_runs_it` (positive leg) |
| No handler on the chain at all ⇒ nothing runs | the same test's negative leg |
| The ancestor walk: target runs nothing, ancestor does | `an_ancestor_handler_runs_only_until_a_nested_widget_gets_one_of_its_own` (first leg) |
| Consumption ends the walk | the same test's second leg (DD-M4-P2-001's named control) |
| Disabled Button-family: suppress **and** do not consume | `a_disabled_button_suppresses_its_own_handler_without_stopping_propagation` (difference leg) |
| An enabled Button in the same position consumes | the same test's agreement leg |
| A handler destroying its own node mid-dispatch | `a_handler_that_removes_its_own_widget_consumes_the_click_and_leaves_no_registration_to_fire` |
| The host-listener producer counts for consumption | `a_host_signal_listener_on_a_non_button_widget_consumes_the_walk_until_disconnected` (first leg) |
| …and stops counting once disconnected | the same test's agreement leg |
| Chain order: target first, root last | `hit::tests::the_dispatch_chain_starts_at_the_target_and_ends_at_the_root` |
| A target that is the root | `hit::tests::a_target_that_is_the_root_yields_only_the_root` |

DD-V-029's named red-test obligation is **not** triggered: no rounding,
unit-conversion or boundary-condition branch was added (edge containment
stayed T2's). The witnesses below are the trap-#4 / #6 artifact.

#### #6 — Deterministic-failure disposition and the mutation witnesses

**The deterministic failure this task found is the debug abort in start
gate finding 1**, and it is dispositioned rather than avoided. A
`Button` carrying a `WidgetNode` child — a shape `wasamoc check`
accepts, the IR loader builds, and
[dsl_spec.md §4.16](../../../../docs/dsl_spec.md) shows as an
example — aborts the process during `wasamo_load_ui` in any build with
`debug_assertions`, because `build_layout_tree` maps Button to a
childless `LayoutNode` and T2's `sync_visuals` child-count assertion
then fires. Root cause named, minimal repro run (both profiles), and
the disposition is:

- **The assertion is correct and stays.** It says "a `WidgetNode` exists
  that layout does not know about", which is exactly true here.
- **The defect is in the DSL surface, not in routing.** T3 does not fix
  it; it records the mechanism and routes the choice, which the owner
  settled the same day in favour of the reject — see §Owner disposition
  below.
- **The plan's T3 evidence item derived from it is replaced, not
  dropped** — see the plan revision below and carry-forward CF-1.

**Six mutation witnesses.** Every one was applied with an edit, **read
back from the file** to confirm the mutation was actually present before
the run (the T2 corrective), run, then reverted and the revert confirmed
by re-reading. No failure was re-rolled; the suite went red only where a
mutation was deliberately introduced.

| Witness | Mutation | Went red | Reading |
|---|---|---|---|
| **W1 — the disabled-Button suppression** | `click_disposition_for`'s `Suppressed` early return deleted | F3 alone, on "a disabled Button must suppress its own `clicked` dispatch (§4.8)" | Without the check the disabled Button's own handler runs — the §4.8 half |
| **W2 — suppression consumes** | `ClickDisposition::Suppressed => return` instead of `continue` | F3 alone, on "a disabled Button must not stop propagation … (§4.19)" | The two halves of §4.8's sentence are pinned by two different assertions on the same click, and each has its own witness |
| **W3 — handling no longer consumes** | the `Handlers` arm `continue`s after running | **Four**: F2 (DD-M4-P2-001's consumption control), F3 (agreement leg), F4 ("the walk must have consumed at the removed Button"), F5 (host listener) | Consumption is the rule the most evidence stands on. F4 reddening *without crashing* is itself informative: the walk continued past a node destroyed mid-dispatch and the ancestor pointer was still valid, because the chain was captured before dispatch |
| **W4 — chain order reversed** | `dispatch_chain` builds root-first | The unit test `the_dispatch_chain_starts_at_the_target_and_ends_at_the_root`, **and** F2 / F3 / F4 / F5 — each on "the nested widget's own handler must run exactly once" | The order is load-bearing at both levels: the pure test names the property, the fixtures show the consequence (the root's handler would consume before the widget the user touched) |
| **W5 — the pre-T3 Button-family gate restored** | `click_disposition_for` returns `NoHandler` for any non-Button node | F1 (the central claim), F2, F3, F5 | This is the before-state of this very task, reconstructed: a `Box`'s handler cannot fire. F1's message names it exactly |
| **W6 — the host listener dropped from the predicate** | the consumption test drops `has_host_listener` | F5 alone, on "a connected host listener on the Box must fire exactly once" | The third producer is pinned separately from the other two, so the predicate cannot silently lose one |

Suite state after all reverts, on the post-commit tree:
`cargo fmt --all -- --check` zero exit, `git diff --check` clean,
`cargo test --workspace --no-fail-fast` **42 binaries, 1,026 passed, 0
failed, 0 skipped**. T2's baseline was 1,019; the seven added are
`hit.rs`'s two `dispatch_chain` unit tests and
`event_routing_integration.rs`'s five fixtures. Skip 0 means the
Compositor was available, so the integration assertions actually ran.
The release clean rebuild is the retrospective's item 3.

#### #5 — Carry-forward

| Constraint | Evidence | Placement | Re-trigger criterion |
|---|---|---|---|
| **CF-1 — a `Button` with a `WidgetNode` child aborts a debug build at load and renders nothing in release.** The shape is accepted by `wasamoc check`, built by the loader, shown in `dsl_spec.md` §4.16, and unknown to layout | Start gate finding 1; measured in both profiles | `carry-forward` → [plan.md](./plan.md) §T3 (replacing the evidence item it invalidates) and §T8; the withheld capability has its own [candidate pool](../../../candidate-pool.md) row | **T8**, which rejects the shape at both gates and corrects §4.16's example (§Owner disposition below). **T13** then re-verifies §4.16 against the landed checker |
| **CF-2 — a literal `enabled: false` on a plain `Button` is silently dropped.** `ir_loader.rs`'s `"Button"` arm never reads an `enabled` prop; its `"ToggleButton"` sibling does. Measured: literal `Button` → `enabled == true`, state-bound `Button` → `false`, literal `ToggleButton` → `false` | The probe above, and F3, which had to bind `enabled` to a state to build a disabled Button at all | `carry-forward` → [plan.md](./plan.md) §T8 | **T8.** Any task asserting Button's `enabled` contract from `.ui` hits it first |
| **CF-3 — `clicked` needs no checker widening; only reject-side bounding.** `wasamoc check` has no per-kind signal admission rule, so DD-M4-P2-005's "the change is in `check` … and in the runtime's dispatch" is half already true | Start gate finding 2; F1 is the runtime half landing | `carry-forward` → [plan.md](./plan.md) §T8 | **T8**, whose reject tests are now the whole of its `clicked` work |
| **CF-4 — `hit_test_click` entered on a subtree gets neither ancestors nor ancestor clip bounds.** T2 recorded the clip half; propagation now has the same boundary, and the three fixtures that enter on a Button rely on it being a no-op | The audit table above; `hit_test_click`'s doc comment | `carry-forward` | **T7**'s modal scopes, and any task that resolves from other than the window root |
| **CF-5 — a native `set_clicked` closure that destroys its own node frees the closure it is running inside.** Pre-existing, not introduced here; the inline and host producers are both safe by construction | `set_clicked`'s doc comment | `carry-forward` | Any task that installs a native closure with structural side effects — no production caller exists today |
| **CF-6 — the registry keys widgets by raw pointer, so a freed node's address could in principle be reused before the enqueue step re-queries it.** Pre-existing in the registry design; T3 relies on the re-query for CF-safe behaviour | F4 pins the intended outcome; the ABA window is not reachable from any current path | `carry-forward` | Any task that allocates a widget inside a handler's synchronous drain |

#### #7 — Re-decided at close

Still **not applicable**. T3 added no Composition write, no visual and no
host launch, and the gallery has neither a non-Button handler nor an
ancestor handler until T10 — a frame captured now would be produced
identically by the pre- and post-T3 dispatch, which is the definition of
a non-discriminating frame. The evidence that distinguishes them is
state read back through the production message path, which is what the
five fixtures do.

#### Re-audit of the whole task list

Per [plan.md](./plan.md) §Cross-task obligations, the full list was
re-read at this close gate rather than only T3's item.

- **T4** — hover is still the whole-tree walk with T2's geometry.
  `resolve_topmost` is the function to call, and `hit_test_click`'s
  chain walk is the shape to mirror if hover ever needs an ancestor
  notion (it does not: hover is a target property). New for T4: the
  message arm now runs `update_hover` after a click that may have
  rebuilt the tree without re-arranging it (§2 above).
- **T5** — the key path is the same walk with a different starting node
  and a different signal name. `dispatch_chain` is reusable as-is; what
  T5 adds is the focused-node entry and the `DefWindowProc` fallthrough
  for an unconsumed key. `hit_test_click`'s structure deliberately keeps
  the "did anything run" answer local to the walk, which is the value
  T5's arm needs to decide whether to return `LRESULT(0)` or fall
  through.
- **T6** — unaffected; no IR or checker surface changed.
- **T7** — CF-4 is now load-bearing for modal scopes: a scope that
  resolves from other than the window root gets no ancestor chain above
  its entry point.
- **T8** — **three items land here** (CF-1, CF-2, CF-3). Its `clicked`
  work is smaller than the plan predicted (no widening needed) and its
  Button-family surface is larger (two loader defects). Recorded in the
  plan's T8 section.
- **T9** — per-item handlers ride this walk unchanged; the registration
  lifecycle F4 exercises (removal severs registrations before the
  enqueue) is the same mechanism T9's subtree-removal enumeration will
  have to state.
- **T10** — the gallery's first ancestor handler and first non-Button
  handler will be the first production consumers of everything in this
  task.
- **T11** — touch will enter the same `hit_test_click`, so the walk is
  message-family-agnostic already; only the DIP conversion seam is
  shared, as DD-M4-P2-001 says.
- **T12 / T13** — T13's re-verification list gains the §4.16 divergence
  (CF-1). The drain wording in
  [architecture.md §13.2](../../../../docs/architecture.md) is
  **not** a divergence — see the reconciliation in §2 — but the reason
  is recorded there so T13 checks it rather than re-deriving it.
- **Cross-task obligation "no new ABI function"** — held, and now
  positively evidenced: `wasamo_signal_connect` already admitted any
  widget and any signal name, so the generic host path needed nothing
  added.

#### Verification means

The five fixtures reuse `tests/common/mod.rs`'s skip guard **unchanged**,
so the standing obligation to verify a newly authored guard on an
environment that lacks the capability
([CLAUDE.md §Testing rules](../../../../CLAUDE.md)) is discharged by the
existing helper; `tests/common/mod.rs` was not touched, so the
`0x80070005` two-conjunct check
([constraints §8](../requirements/constraints.md)) is intact.

No fixture changes scale, so none of them touches the DPI-fixture
environment beyond normalising to 96 DPI at a 360x240 physical client —
below the 480x320 ceiling M4-Phase 1 T8 settled on
([constraints §10](../requirements/constraints.md)) — and each asserts
both the realised extent and the committed scale rather than assuming
the developer's monitor (Phase 1 F-47). Each derives its click
coordinates from the scale the runtime **committed**, not from the
constant it requested.

**What these fixtures cannot show, stated rather than implied.** They run
at scale 1, so they do not re-exercise the pointer conversion T2's
non-unit-scale fixture owns; the walk is geometry-independent, and that
fixture is unchanged and still green. The ancestor legs are one hop —
the multi-level prefix order is pinned by `dispatch_chain`'s unit test
rather than by a three-level fixture, which is the division of labour
DD-M4-P2-002 set up when it made resolution pure logic.

### Owner disposition of the two Button-family findings (2026-08-07)

Both findings the close gate routed rather than fixed are dispositioned.
Neither changes T3's landed code; both change what T8 owns.

**CF-1 — a `Button` carrying a `WidgetNode` child.** Rejected at both
gates: the `wasamoc check` admission rule **and** the IR loader's
re-check, which is the two-gate shape `dsl_spec.md` §4.9 / §4.16 already
use and is required here for the same reason — `wasamo_load_ui` admits
memory IR that never passed through `wasamoc`. The direct C path
(`wasamo_widget_append_child`) stays ungated, matching how `Box`'s
child-count rule is enforced; T2's `sync_visuals` child-count assertion
remains the tripwire there. §4.16's placement example is corrected in
the same change: it illustrates `slot.*` and needs no Button child to
do so. Landing: [plan.md](./plan.md) §T8.

The alternative — making Button a layout container — was **not** taken
inside this phase. It is not a defect fix but a widget-design decision
(what becomes of the label, how children arrange, what the
accessibility name is), and this phase does not open layout
([DD-M4-P2-002](../decisions/dd-m4-p2-002-hit-testing-and-generic-click.md)
§Minimum hit target). Because the reject **narrows an authored surface**,
the withheld capability is recorded as its own row in the
[candidate pool](../../../candidate-pool.md) — leaning M5, no milestone
claimed — so re-opening it is a milestone decision rather than a
rediscovery. That is the DD-V-028 lifecycle rule applied in the
direction the pool exists for.

**CF-2 — a literal `enabled: false` on a plain `Button`.** Fixed at T8,
by reading the prop the way the `"ToggleButton"` arm does. The test that
goes with it drives the literal through `.ui` → IR → loader and asserts
the widget constructs disabled — the half that was missing. The checker
already has a test that the literal is *accepted*
(`wasamoc/src/check.rs`), and the runtime already has a test that the
literal *takes effect* — but only for `ToggleButton`
(`togglebutton_runtime_integration.rs`). The pairing existed for one
widget and not the other, which is the shape that let a two-and-a-half
month old gap stay invisible.

## T4 — Hover and pressed behind the routing model

### Start gate (recorded 2026-08-07, before any source edit)

Read before selecting:
[AGENTS.md](../../../../AGENTS.md),
[implementation-gates.md](../../../procedures/implementation-gates.md),
[plan.md](./plan.md) §T4 and §Cross-task obligations,
[preamble.md](./preamble.md) (§What "green" is worth, §The migration
obligation, §Review lanes),
[DD-M4-P2-001](../decisions/dd-m4-p2-001-event-routing-model.md)
§Pointer capture, hover, pressed and §Recommendation,
[DD-M4-P2-002](../decisions/dd-m4-p2-002-hit-testing-and-generic-click.md)
§Recommendation,
[constraints.md](../requirements/constraints.md) §3 / §4 / §5 / §8 / §10,
the Moment-1 normative text
([architecture.md §13.2](../../../../docs/architecture.md) and §12.5,
[dsl_spec.md §4.19](../../../../docs/dsl_spec.md) and §4.8),
the [T3 close gate](#t3--propagation-and-the-drain-boundary) and the
[T3 retrospective](../retrospectives/t3.md), and the landed source
(`widget.rs` `update_hover` / `update_hover_inner` / `clear_hover` /
`hit_test_click` / `update_button_enabled` / `update_toggle_button_checked`
/ `effective_button_color`, `hit.rs`, `window.rs`'s four pointer arms and
`set_root`).

#### Normative statements that already answer this task's behaviour

Recorded per
[DD-V-031](../../../cross-milestone/decisions/dd-v-031-normative-answers-at-start-gate.md):
this phase synchronised its normative text at Moment 1, so the questions
below are **answered**, not open, and are not escalations.

| Question | Document | What it fixes |
|---|---|---|
| Who computes hover / pressed, and against what | [architecture.md §13.2](../../../../docs/architecture.md) | "computed as enter / leave transitions **against the resolved target** rather than by a whole-tree walk" — the walk is named as the thing being replaced, so a stateless re-walk that merely narrows to the target is **not** what is specified |
| Whether the painted node is the target or the node a release would dispatch to | the same sentence | The rule is "against the resolved target"; "so the widget that paints pressed is the widget a release would dispatch to" is its stated consequence, not a second rule. Measured below: the two coincide for every shape the widget set can build |
| Whether hover / pressed gain an authored surface | [architecture.md §13.2](../../../../docs/architecture.md), [dsl_spec.md §4.19](../../../../docs/dsl_spec.md) | No. They are Button-family presentation state; §4.19 says a Button "paints hover / pressed states; those are Button behaviours (§4.8), not part of the signal's meaning" |
| What a disabled Button does | [dsl_spec.md §4.8](../../../../docs/dsl_spec.md) | "Hover / press visual transitions are frozen; the background paints a flat disabled grey directly (no `ColorKeyFrameAnimation` runs)" |
| Whether a scale change synthesises a pointer update | [architecture.md §12.5](../../../../docs/architecture.md) | "It synthesises no pointer message either: the pointer may end up over a different widget after the accompanying resize, and the next real pointer message corrects the hover state." [constraints §5](../requirements/constraints.md)'s sub-issue is therefore already discharged and needs no code |
| Where hit geometry comes from | §13.2, §4.19 | Landed at T1 / T2; this task inherits it and changes no geometry |

No divergence between the ADR set and the normative text was found, so
nothing is carried to T13's re-verification from this row.

#### Scope re-decided against the code — four facts measured first

Throwaway probes and greps, run before the approach was chosen.

1. **Nothing in the suite exercises hover.**
   `rg "update_hover|clear_hover|ButtonState|hovered"` over
   `wasamo-runtime/tests` returns **zero** hits, and `update_hover` /
   `clear_hover` have no caller anywhere outside `window.rs`'s four pointer
   arms (`rg` over `wasamo-runtime`, `wasamo-dll`, `bindings`, `examples`).
   The whole-tree walk can therefore be deleted and replaced with anything
   at all and the suite stays green — this task's instance of
   [preamble.md §What "green" is worth](./preamble.md). Every property this
   task claims needs a test that did not exist before it.
2. **The gallery already contains a reachable overlap, and the pre-T4
   defect is visible in it today.** The `if is_lightbox_open` branch is a
   *later* child of the root `ZStack` than the main `Grid`, and its scrim
   `Box` is stretch/stretch, so an open lightbox covers the toolbar.
   Measured on the pre-T4 release build with
   [evidence/capture-t4-hover.ps1](./evidence/capture-t4-hover.ps1)
   (982x703 client at 120 DPI, mean over a 2,559-pixel mask taken from the
   checked "All" `ToggleButton`, two frames per side):

   | Side | Mean R / G / B over the mask |
   |---|---|
   | closed, cursor parked away | 46.99 / 117.65 / 213.41 |
   | closed, cursor over "All" | 74.06 / 138.90 / 224.06 |
   | open, cursor parked away | 22.39 / 42.88 / 68.30 |
   | open, cursor over "All" | 28.15 / 46.78 / 71.19 |

   Within-side frame-to-frame jitter is **0.00 on every channel**, so the
   sampled means carry no noise floor to clear. Hover-versus-no-hover is
   +27.07 / +21.25 / +10.65 with the lightbox closed and still
   **+5.76 / +3.90 / +2.89 with it open** — the fifth of the same signal
   that the scrim's `cc` alpha leaves. **The toolbar's ToggleButton hovers
   through the scrim today.** This contradicts
   [preamble.md](./preamble.md)'s "occlusion is unobservable until T10" for
   the hover half, so trap #7 is selected rather than waved off, and the
   same script re-run after the change is the close artifact.
3. **"The resolved target" and "the nearest Button-family node on the
   dispatch chain" coincide for every shape the widget set can build.**
   `build_layout_tree` maps `Button` / `ToggleButton` to a childless
   `LayoutNode` (T3 start gate finding 1), so a Button-family node is never
   an ancestor of a hit candidate: whenever a Button-family widget is on the
   chain at all it *is* the target. Hover therefore needs no ancestor walk,
   and §13.2's "against the resolved target" can be implemented literally
   with no ambiguity to resolve. Recorded because the coincidence is a
   property of today's layout mapping rather than of the routing model — if
   Button ever becomes a layout container the question reopens.
4. **`ButtonData::state` already has a third writer.**
   `update_button_enabled` sets `state = ButtonState::Normal` and paints the
   flat grey directly, on a *binding* write rather than a pointer message.
   Any retained "which node is painting hover" record is therefore derived
   data that a non-pointer path can invalidate — which is what puts trap #3
   on this task rather than off it.

**What T4 therefore is.** Not "narrow the whole-tree walk to the target",
which the normative text explicitly excludes, but: give the window a
retained record of *which node currently paints a non-Normal state*, and
make every pointer arm a leave / enter transition against the target
`hit::resolve_topmost` already resolves — with the invalidation paths of
fact 4 enumerated rather than hoped about.

#### Trap selection

| # | Trap | Applies | Reason |
|---|---|---|---|
| 1 | Semantic-migration miss | **yes** | No enum or schema gains a variant, so the compiler enumerates nothing — but the *decision* "does this widget paint hover / pressed" migrates from a per-node containment test evaluated at every node to a single resolved target. Every reader and writer of `ButtonData::state`, and every caller of the two hover entry points, is audited as a call-site table; a writer left out is a node that paints a state nobody owns |
| 2 | Missed side effects | **yes** | The derived effects to enumerate before writing: the colour animation `start_color_anim` starts (and the `if new_state != btn.state` guard that keeps a leave from overriding the disabled grey), `update_button_enabled`'s state reset, `update_toggle_button_checked`'s brush rebuild from `btn.state`, `window::set_root` replacing the tree the retained record indexes, a click handler's synchronous rebuild between `hit_test_click` and the `update_hover` that follows it in the same `WM_LBUTTONUP` arm (T3's carry-forward to this task), and the `ButtonData.label_size` three-point write ([constraints §4](../requirements/constraints.md)) which this task must not touch |
| 3 | Parallel/derived data drift | **yes** | The retained record and `ButtonData::state` are a derived pair: the invariant is "at most one node has `state != Normal`, and it is exactly the node the record names". They must be written in the same primitive, and fact 4's third writer is the path that can break the pair from outside |
| 4 | Untested authored branch | **yes** | New arms: the target is an enabled Button-family widget (paint and retain); the target is a **disabled** Button-family widget (paint nothing, retain nothing, and the previously painting node still leaves); the target is a non-Button widget or nothing at all (the same); previous equals next (no leave); `WM_MOUSELEAVE` with nothing retained. Each ships with a test that fires it directly, and each is put under a deliberately wrong implementation shown to redden it. DD-V-029's named obligation is **not** triggered — no rounding, unit-conversion or boundary-condition branch is added, edge containment stayed T2's — so the witnesses are the trap-#4 / #6 artifact rather than that decision's |
| 5 | Carry-forward underweighted | **yes** | T5 puts focus state beside this record on the same `WindowState` and adds a second painted state that must stay distinguishable from hover (DD-003); T7 materialises and removes subtrees under the pointer, which is what can shift the retained record's index; T10 is the first production consumer. Per the T3 retrospective's corrective, anything this task *requires as evidence of a later task* is built and run here first, or it is recorded as a finding with an owner instead of as a carry-forward |
| 6 | Symptom taken at face value | **conditional** | No deterministic failure is in hand at the start gate. Selected as armed rather than applicable: any failure that appears during implementation gets a minimal repro and a root cause, not a re-roll |
| 7 | Weak GUI evidence | **yes** | Selected on the measurement in fact 2, against the preamble's prediction. The deliverable is a painted state, and a frame that distinguishes the intended behaviour from the pre-T4 one is **buildable in the gallery today** — so the close artifact is the same capture script re-run, where the open-lightbox hover delta must collapse into the (measured, zero) within-side jitter while the closed-lightbox delta must survive |

```
- [x] #1 semantic migration   - [x] #2 side effects   - [x] #3 parallel data   - [x] #4 branch tests
- [x] #5 carry-forward        - [~] #6 root cause     - [x] #7 GUI positive control
```

#### Review lane

**Full independent review**, which is a **correction of
[preamble.md §Review lanes](./preamble.md)'s prediction** of
branch/test-focused. The prediction assumed T4 was "a state-ownership
change behind T2's already-reviewed structure". Facts 1, 2 and 4 change the
classification: the task adds **retained per-window state** with a
cross-path invalidation surface (a runtime structural change), and it
carries **GUI-render evidence** — two of the three high-risk classes in
[implementation-gates.md §4](../../../procedures/implementation-gates.md).
The trap-#4 branch/test check composes into it rather than replacing it.
Recorded here as the Phase 1 F-12 / T12 precedent requires: a lane found
stale at a task's start gate is corrected at that gate.

#### The T3 correctives, applied

T3's retrospective added two lines to later start gates. Both are answered
here rather than at the close:

- *Does the normative text already answer this task's semantics?* Yes — the
  table above, consulted before the ADRs were read for reasoning. The one
  place the ADR and the spec could have disagreed (target versus dispatch
  node) is measured in fact 3 to be a distinction without a difference in
  the current widget set.
- *Re-measure what this task actually adds rather than inheriting the plan's
  framing.* Done: the plan says "replacing the whole-tree walk", and fact 1
  shows the walk is entirely unpinned, so the risk is not that the
  replacement is hard but that **nothing would notice if it were wrong**.
  That moves the task's centre of gravity from the transition rule to the
  evidence.

#### Planned proof obligations

Each closed at the T4 close gate:

1. The call-site audit table over every reader and writer of
   `ButtonData::state` and both hover entry points.
2. The structural side-effect enumeration, including `set_root`, the
   binding-driven `enabled` reset, and the post-click `update_hover` in the
   same `WM_LBUTTONUP` arm.
3. The parallel-data statement for the retained record, naming the single
   primitive that writes both halves.
4. Pure-logic unit tests for the leave / enter transition rule.
5. Integration fixtures over real messages: enter / leave / press / release
   transitions read back from live widget state; the overlap case where only
   the topmost widget reacts; the disabled-Button arm; `WM_MOUSELEAVE`.
6. Mutation witnesses for each new arm, each read back from the file before
   it is run and re-read after the revert.
7. The GUI positive control re-run, with the pre-change numbers above as the
   before-state.
8. The whole task list re-read at the close gate (the re-audit discipline,
   [plan.md](./plan.md) §Cross-task obligations).

### Close gate (recorded 2026-08-07)

Landed: `hit::hover_leave_target` (the leave rule, five unit tests);
`widget::HoverState` (the window's retained "which node paints a
non-`Normal` state" record) with `WidgetNode::update_hover` /
`clear_hover` rewritten as leave-then-enter against
`hit::resolve_topmost`'s single target, both narrowed to `pub(crate)`;
`node_at_path_mut` (bounds-checked descent) and `set_button_state_at`
(the one guarded state-writing primitive) beside them;
`update_hover_inner` deleted; `WindowState::hover` with its reset in
`set_root`; `__button_state_for_test` / `ffi::__hover_target_for_test`;
and `wasamo-runtime/tests/hover_transition_integration.rs` (five
fixtures).

#### #1 — Call-site audit table

The migrating decision is **"does this widget paint hover / pressed"**,
which moved from a containment test evaluated at *every* node of a
whole-tree walk to a single resolved target. No type changed, so the
compiler enumerates none of it; the artifact is the grep table.

Queries:
`rg "update_hover|clear_hover" wasamo-runtime wasamo-dll bindings examples`,
`rg "ButtonState" wasamo-runtime/src`,
`rg "\.state" wasamo-runtime/src/widget.rs`,
`rg "effective_button_color|start_color_anim|transition_duration" wasamo-runtime/src`,
`rg "hover" wasamo-runtime/src wasamo-runtime/tests`.

**Every writer of `ButtonData::state`.** A writer left out of the pair
with `HoverState` is a node that paints a state nobody owns.

| Writer | Classification | Reason |
|---|---|---|
| `widget.rs::set_button_state_at` | **new, sole pointer-driven writer** | Both enter and leave go through it; it is the only place the transition and its animation are spelled |
| `widget.rs::update_button_enabled` | **pre-existing, out of the pair, enumerated** | Resets `state` to `Normal` on a *binding* write and paints the flat grey directly (`docs/dsl_spec.md` §4.8). It cannot reach `HoverState`, so the record may keep naming a node it has already reset; the leave that follows finds the states equal and is a no-op under `set_button_state_at`'s guard. Recorded rather than "fixed": making this a third writer of the record would put a hover producer on the binding path, which is the shape [DD-M4-P2-001](../decisions/dd-m4-p2-001-event-routing-model.md) refuses |
| `widget.rs::update_toggle_button_checked` | ignore-OK | *Reads* `btn.state` to recompute the brush; never writes it |
| `widget.rs` Button/ToggleButton constructors | ignore-OK | Initialise `state` to `Normal`, which is the invariant's own base case |
| `widget.rs::update_hover_inner` | **deleted** | The whole-tree walk this task replaces |
| `widget.rs::clear_hover` (old body) | **rewritten** | Was a whole-tree reset; is now the retained target's leave |

**Every caller of the two hover entry points.**

| Call site | Classification | Reason |
|---|---|---|
| `window.rs` `WM_MOUSEMOVE` | migrated | Passes `&mut state.hover`; `down` still `state.mouse_down` |
| `window.rs` `WM_LBUTTONDOWN` | migrated | `down: true` |
| `window.rs` `WM_LBUTTONUP` | migrated | `down: false`, and **still after `hit_test_click`** — see the enumeration below |
| `window.rs` `WM_MOUSELEAVE` | migrated | `clear_hover` |
| `window.rs::set_root` | **new** | Resets the record; the previous root is dropped just above it and its indices name nothing in the new tree |
| Anywhere else in `wasamo-runtime`, `wasamo-dll`, `bindings`, `examples` | **none exist** | Which is what makes the `pub` → `pub(crate)` narrowing behaviour-preserving. `HoverState` is crate-internal, so no external caller could supply the new argument in any case |

**What the compiler enumerated: the signature change only.** Both entry
points gained a parameter, so every call site had to be visited — but that
is four sites in one file, not the semantic surface. The semantic surface
is the writer table above, and nothing there is compiler-enforced.

#### #2 — Structural side-effect enumeration

| Derived effect | Disposition |
|---|---|
| **Which node paints hover / pressed** | Changed by design: exactly the resolved target, when it is an enabled Button-family node. Overlapping widgets no longer both paint |
| **A Button under a covering widget** | No longer paints. This is the gallery defect measured at the start gate and re-measured at #7 below |
| **A disabled Button-family widget** | Still the resolved target and still occludes (T2's rule, untouched), paints nothing, and **retains nothing** — so the previously painting node still leaves. `docs/dsl_spec.md` §4.8's "hover / press visual transitions are frozen" |
| **A Button-family widget's `WidgetNode` children** | The old walk descended into them under a disabled Button. That descent is **deleted rather than adapted**: `build_layout_tree` maps Button-family to a childless `LayoutNode`, so such a child has no rectangle, is never a hit candidate, and was never reachable by a correct containment test anyway (T3 start gate finding 1; the shape itself is rejected at T8) |
| **The colour animation** | Unchanged: same `effective_button_color` / `transition_duration` / `start_color_anim`, same `if new_state != btn.state` guard, now called from one place instead of per node |
| **`update_button_enabled`'s state reset** | Enumerated in #1. Its interaction with the guard is what lets the flat disabled grey survive a later leave |
| **`window::set_root`** | Resets the record. Without it a path from the dropped tree would be applied to the new one — bounds-checked, so not unsound, but able to write `Normal` onto an unrelated node |
| **A click handler's synchronous rebuild, mid-`WM_LBUTTONUP`** | T3's carry-forward, decided here. **The arm's order is kept**: `hit_test_click` first, `update_hover` second. The release must dispatch against the tree the user saw; a node materialised by the handler has no `arranged_rect` until the message-loop boundary's drain re-lays-out, so it is not a hit candidate and the hover resolved immediately after a click can be stale. That staleness is **not** corrected here — correcting it would need a second hover producer on the drain path. It self-corrects on the next real pointer message, and the #7 capture exercises exactly this path (the lightbox is opened by a click and the toolbar's hover is measured afterwards) |
| **A retained path that a structural change shifts** | `node_at_path_mut` descends with `children.get_mut`, never indexing, so a stale path is a no-op rather than a panic. A path that now names a *different* live node can leave a node that is still under the pointer painted until its next enter/leave. Carried forward (CF-T4-1) with the measurement that no M4 shape reaches it |
| **`ButtonData.label_size`'s three-point write** ([constraints §4](../requirements/constraints.md)) | Not touched. T4 writes no label geometry |
| **Composition geometry writes** | Untouched. `SetOffset` / `SetSize` remain the same six calls inside `sync_visuals`; DD-M4-P1-002's single-pass audit is preserved |
| **The synthesised pointer update after a scale change** | Still not adopted, and needs no code: [architecture.md §12.5](../../../../docs/architecture.md) already states that a scale change "synthesises no pointer message... and the next real pointer message corrects the hover state". [constraints §5](../requirements/constraints.md)'s sub-issue is discharged by that sentence rather than by this task |

#### #3 — Parallel-data sync

The retained record and the painted `ButtonState` are the derived pair.
They are written **in one function each**: `update_hover` computes the
paint target, runs the leave and the enter, and assigns
`hover.target = paint_target` in the same body; `clear_hover` leaves and
clears in the same body. `HoverState::target` is a private field with no
setter, and the only accessor is a read-only `target()` for the test seam,
so a future edit cannot write the record from anywhere else without adding
a writer to this module first.

The pair is asserted rather than assumed: **every fixture reads both**
`__button_state_for_test` and `ffi::__hover_target_for_test` after every
step where either could have changed. Witness W4 (the record write
deleted) reddens all five fixtures, F1 on the message that names this
trap.

The one path that can break the pair from outside is
`update_button_enabled`, enumerated in #1 and #2.

#### #4 — Branch tests, each fired directly

| Authored arm | Test that fires it |
|---|---|
| Enter: the target is an enabled Button-family node | `a_button_moves_through_hover_press_release_and_leave_in_one_pointer_sequence` step 1 |
| `down` selects `Pressed` over `Hovered` | the same fixture's steps 2 and 3 |
| Leave: the target moved off the retained node | the same fixture's step 4 |
| Only the topmost of two containing candidates paints | `only_the_topmost_of_two_overlapping_buttons_hovers_and_the_wide_one_still_hovers_where_uncovered` (difference leg) |
| …and the lower one is not simply unhoverable | the same fixture's agreement leg |
| The target is a **disabled** Button-family node: paint nothing, retain nothing, still leave the previous | `a_disabled_button_paints_nothing_but_still_occludes_and_the_previously_hovered_button_still_leaves` (difference leg) |
| …and that position is hoverable once enabled | the same fixture's agreement leg |
| The target is a non-Button widget: paint nothing, still leave | `hovering_a_non_button_target_paints_nothing_and_still_leaves_the_button_it_partly_covers` |
| `clear_hover` with something retained | `wm_mouseleave_clears_the_retained_hover_target_and_a_second_leave_is_a_no_op` (first leave) |
| `clear_hover` with nothing retained | the same fixture's second leave |
| The leave rule's five cases | `hit::tests::hover_leave_target_*` (five unit tests) |

DD-V-029's named red-test obligation is **not** triggered: no rounding,
unit-conversion or boundary-condition branch was added (edge containment
stayed T2's). The witnesses below are the trap-#4 / #6 artifact.

#### #6 — Deterministic-failure disposition and the mutation witnesses

No deterministic failure appeared during implementation, so trap #6 stays
armed rather than discharged against a defect. Nothing was re-rolled: the
suite went red only where a mutation was deliberately introduced.

**Seven mutation witnesses.** Every one was applied with an edit, **read
back from the file** to confirm the mutation was present before the run,
run, then reverted and the revert confirmed by re-reading (the T2
corrective, carried by T3).

| Witness | Mutation | Went red | Reading |
|---|---|---|---|
| **W1 — the disabled arm** | the paint target's `enabled` test always true | F3 alone, on "a disabled Button must paint nothing on entry" | §4.8's frozen-transition half is pinned by one fixture, separately from the leave half in the same test |
| **W2 — the leave guard** | `hover_leave_target` always returns `previous` | The unit test `hover_leave_target_is_none_when_previous_equals_next…` **alone**; the five fixtures stayed green when run separately | **The division of labour, measured.** A stationary mouse leaving-and-re-entering restarts the colour animation but ends in the same `ButtonState`, so no state read-back can see it. The pure test is the only thing that can, which is why it exists |
| **W3 — the leave half** | the `leave_hover_at` call dropped | F1, F2, F3, F5 — and **not** F4 | The leave reached through `update_hover` and the one reached through `clear_hover` are pinned by different fixtures, so deleting either is visible on its own |
| **W4 — the pair** | `hover.target = paint_target` dropped | **All five**, F1 on "the retained hover path must name the entered Button (trap #3…)" | The record and the paint cannot be edited apart without every fixture noticing |
| **W5 — the press mapping** | `Pressed` / `Hovered` swapped | All five, F1 and F2 on their own messages | |
| **W6 — the transition guard** | `if new_state != btn.state` removed | **Nothing.** `hover_transition_integration`, `button_enabled`, `togglebutton_runtime_integration`, `bool_binding_live_propagation` all stayed green | **A stated coverage limit, not a pass.** The guard's unique effects — not restarting the animation, and not overriding `update_button_enabled`'s flat grey with the *enabled* `Normal` colour — are `CompositionColorBrush` properties mid-animation, which no read-back in this suite can observe. The exposure is **unchanged from pre-T4** (the old `update_hover_inner` and `clear_hover` carried the identical guard with no `enabled` check either). Carried forward as CF-T4-2 |
| **W7 — the pre-T4 walk, restored** | the whole-tree walk re-added as an additive pass after the new logic | **F2's difference leg and F5's covered leg, and nothing else** | The decisive one: these fixtures catch the actual defect this task fixes, not merely a mutation of its own implementation. It is also what showed F5's first draft was not discriminating — its "Box is the target" point had been chosen *outside* the Button, where the pre-T4 walk would have left the Button too; corrected to a point both rectangles contain before this witness was run |

Suite state after all reverts, on the post-commit tree:
`cargo fmt --all -- --check` zero exit, `git diff --check` clean,
`cargo test --workspace --no-fail-fast` **43 binaries/sections, 1,036
passed, 0 failed, 0 ignored, 0 skipped** (T3's baseline was 1,026; the ten
added are `hit.rs`'s five `hover_leave_target` unit tests and
`hover_transition_integration.rs`'s five fixtures). Skip 0 means the
Compositor was available, so the integration assertions actually ran.

#### #7 — GUI evidence with a positive control

[evidence/capture-t4-hover.ps1](./evidence/capture-t4-hover.ps1), re-run
against the post-change `cargo build --release --workspace`, on the same
982x703 client at 120 DPI, with the sample mask recomputed from the same
frame and coming out **byte-identical** (2,559 px, bbox x[10..69]
y[13..56]) — so both runs measure the same pixels.

| Mean RGB over the mask | pre-T4 | post-T4 |
|---|---|---|
| closed, cursor parked away | 46.99 / 117.65 / 213.41 | 46.99 / 117.65 / 213.41 |
| closed, cursor over "All" | 74.06 / 138.90 / 224.06 | 74.06 / 138.90 / 224.06 |
| open, cursor parked away | 22.39 / 42.88 / 68.30 | 22.39 / 42.88 / 68.30 |
| open, cursor over "All" | 28.15 / 46.78 / 71.19 | **22.39 / 42.88 / 68.30** |
| **hover delta, lightbox closed** | **+27.07 / +21.25 / +10.65** | **+27.07 / +21.25 / +10.65** |
| **hover delta, lightbox open** | **+5.76 / +3.90 / +2.89** | **0.00 / 0.00 / 0.00** |
| within-side frame-to-frame jitter | 0.00 on every channel | 0.00 on every channel |

**Both legs are needed and both moved the right way.** The
closed-lightbox delta is the agreement leg — it is *unchanged*, so the
capture distinguishes "hover now respects occlusion" from "hover stopped
working", which a single frame could not. The open-lightbox delta is the
difference leg: it was a scrim-attenuated copy of the same signal
(ratio 0.21 / 0.18 / 0.27, consistent with the scrim's `cc` alpha) and is
now exactly the measured jitter floor. The capture states its scale
(125%), takes two frames per side, uses the **client** rectangle mapped
with `ClientToScreen`, and the tool declares Per-Monitor-Aware V2 **and
reads the posture back** before measuring (Phase 1 F-48).

This run also exercises the `WM_LBUTTONUP` ordering decided in #2: the
lightbox is opened by a real click, and the toolbar's hover is measured
after the pointer moves again — i.e. after the drain that the click's
own message could not wait for.

**Sampling a text-free solid-colour region gave zero frame-to-frame
jitter**, against the up-to-13/channel text-pixel jitter Phase 1 F-33
measured. Carried to T12, whose control B needs a tolerance only where it
samples text.

This is the assistant baseline and does **not** replace the owner's
human-visible smoke ([CLAUDE.md §Testing rules](../../../../CLAUDE.md)).

#### #5 — Carry-forward

| Constraint | Evidence | Placement | Re-trigger criterion |
|---|---|---|---|
| **CF-T4-1 — the retained hover path is index-based, so a structural change under the pointer can shift what it names.** `node_at_path_mut` is bounds-checked, so a stale path is a no-op rather than a panic and never unsound; what can happen is a node left painted hovered until its next enter/leave | The nearest shape was **built and run**, not reasoned about: the #7 capture opens the lightbox by a real click with the pointer over the toolbar, and the toolbar's hover clears correctly on the next move (the insert lands at a later sibling index, so the toolbar's path does not move). No M4 shape puts a Button-family widget inside a `for` body or inside a scope that reorders siblings | `carry-forward` → this ledger | **T7** (modal-scope materialisation and removal) and **T9** (per-item subtrees). The signal is a Button-family widget whose sibling index can change while the pointer is over it |
| **CF-T4-2 — `set_button_state_at`'s `if new_state != btn.state` guard is unpinned.** W6 deletes it and nothing in the suite goes red; its unique effects are mid-animation brush properties | Witness W6 above. Exposure is unchanged from pre-T4 — the deleted `update_hover_inner` and the old `clear_hover` carried the same guard with no `enabled` check | `carry-forward` → this ledger | **T5**, which adds a *third* background-painted state (the focus indicator, DD-M4-P2-003) and must keep it distinguishable from hover and from the ToggleButton selected state — the first task with a reason to read a brush colour back |
| **CF-T4-3 — `set_root`'s hover reset is not fired by any test.** It is an unconditional statement rather than a branch, so trap #4 does not bite, but no test or example replaces a root after a pointer message | `rg` over `wasamo-runtime/tests`, `examples`, `bindings`: `wasamo_window_set_root` is called once per window, before any message. `wasamo_load_ui` is the only production caller of `window::set_root` | `carry-forward` → this ledger | The first test, example or host that replaces a live window's root |
| **CF-T4-4 — hover is wired to the three mouse messages only.** T11's `WM_POINTER*` arms inherit nothing: a pointer arm that does not call `update_hover` leaves the record untouched | The audit table in #1: four call sites, all in `wnd_proc`'s mouse arms | `carry-forward` → this ledger | **T11.** Whether a touch contact should paint hover at all is a decision that task must make explicitly rather than inherit by omission |
| **CF-T4-5 — the preamble's "occlusion is unobservable until T10" is false, for the click half as well as the hover half.** The gallery's `if is_lightbox_open` branch is authorable, openable and closable today, and its stretch/stretch scrim covers the toolbar | The start-gate capture measured the pre-T4 hover-through-the-scrim delta directly | `finding` → [preamble.md](./preamble.md) §What "green" is worth, corrected in this task's commit batch | **T12**, whose control C is the phase-level version of the same shape. T2's occlusion claim rests on pure-logic tests and fixtures rather than a gallery frame; that is sound but was chosen against a prediction that has now been measured false, so the owner is told rather than the record being left to imply the prediction held |

#### Re-decided at close

The start gate selected traps 1–5 and 7 with 6 armed. **The selection
survived**, with one thing built that the gate did not predict: trap #7's
close artifact turned out to also exercise the `WM_LBUTTONUP` ordering
decision from trap #2, because the only way to open the lightbox is a real
click. Trap #6 stays armed and undischarged — no deterministic failure
appeared. The review lane correction made at the start gate (full
independent review) also stands: the landed change adds retained
per-window state and carries GUI-render evidence.

#### Re-audit of the whole task list

Per [plan.md](./plan.md) §Cross-task obligations, the full list was re-read
at this close gate rather than only T4's item.

- **T5** — inherits two things concretely. `FocusState` lands beside
  `HoverState` on the same `WindowState`, and `node_at_path_mut` is
  reusable for any path-addressed node. More importantly, the focus
  indicator is a **third** background-painted state after hover and the
  ToggleButton selected state, so DD-M4-P2-003's distinguishability
  requirement meets CF-T4-2: T5 is the first task with a reason to read a
  brush colour back, which is what would also pin the transition guard.
- **T6** — unaffected; no checker, IR or loader surface changed.
- **T7** — CF-T4-1 is its to preserve: the materialisation seam is exactly
  what can shift a retained sibling index while the pointer is over it.
- **T8** — unaffected by this task, and CF-2 (a literal `enabled: false` on
  a plain `Button` is dropped) is **re-confirmed still open**: F3 had to
  bind `enabled` to a state to build a disabled Button at all, and its
  guard assertion says so in its own message.
- **T9** — same index-shift exposure as T7, through `for` regeneration.
- **T10** — first production consumer. The gallery's hover-through-the-scrim
  defect is fixed by this task, so T10's lightbox work starts from a
  gallery whose hover already respects occlusion.
- **T11** — CF-T4-4: touch inherits no hover behaviour, and whether it
  should is that task's explicit decision.
- **T12** — two inheritances. Control C is the phase-level form of the
  control run here, and the zero-jitter measurement over a text-free
  solid-colour mask means its agreement legs need F-33's 13/channel
  tolerance only where they sample text.
- **T13** — the hover sentences in
  [architecture.md §13.2](../../../../docs/architecture.md) and
  [dsl_spec.md §4.19](../../../../docs/dsl_spec.md) match the landed
  runtime and are a **confirmation** rather than a re-derivation: §13.2's
  "the widget that paints pressed is the widget a release would dispatch
  to" holds because `build_layout_tree` maps Button-family to a childless
  `LayoutNode`, so a Button-family node on the dispatch chain is always the
  target. That reason is recorded here so T13 checks it instead of
  re-measuring it. No divergence to record.
- **Cross-task obligation "no new ABI function"** — held. The two new
  symbols are `__*_for_test` seams in the Rust-side `ffi` module, not C
  entry points, and no `extern "C"` function was added.

#### Verification means

The five fixtures reuse `tests/common/mod.rs`'s skip guard **unchanged**,
so the standing obligation to verify a newly authored guard on an
environment that lacks the capability
([CLAUDE.md §Testing rules](../../../../CLAUDE.md)) is discharged by the
existing helper; `tests/common/mod.rs` was not touched, so the
`0x80070005` two-conjunct check
([constraints §8](../requirements/constraints.md)) is intact.

No fixture changes scale. Each normalises to 96 DPI at a 360x240 physical
client — below the 480x320 ceiling M4-Phase 1 T8 settled on
([constraints §10](../requirements/constraints.md)) — and asserts both the
realised extent and the committed scale rather than assuming the
developer's monitor (Phase 1 F-47). Every move and click coordinate is
derived from `__arranged_rect_for_test()` and multiplied by the factor the
runtime **committed**, and every geometric relationship a fixture depends
on (containment, non-containment, disjointness) is asserted before the
point is used — so a layout change makes a fixture fail loudly rather than
quietly stop discriminating.

**What these fixtures cannot show, stated rather than implied.** They run
at scale 1, so they do not re-exercise the pointer conversion T2's
non-unit-scale fixture owns; the transition rule is geometry-independent
and that fixture is unchanged and still green. They read `ButtonState`,
not pixels, so the animation the state drives — its duration, its restart,
and the disabled grey the guard protects — is invisible to them (W2 and W6
measure exactly that boundary). The GUI control at #7 is what covers the
painted side, and it covers one overlap shape at one scale.

### Owner disposition of CF-T4-5 (2026-08-07)

The one finding the close gate routed rather than settled is
dispositioned. It changes no landed code and no T4 artifact; it fixes
where the residual claim lands.

**CF-T4-5 — the preamble's occlusion prediction was false, and T2's
occlusion claim has no gallery frame.** The owner accepted the close
gate's recommendation:

- **T2 is not reopened.** Its occlusion rule is pinned by pure-logic
  tests over a constructed overlapping tree and by integration fixtures,
  which bound the rule rather than one instance of it. Re-capturing a
  gallery frame against T2 would add no information that T4's measurement
  has not already produced on the same path.
- **T12's control C is what closes the residual at phase level.** That
  control is the same shape — with the lightbox open a background click
  does nothing, with it closed the same coordinate fires — so the gallery
  frame T2 did not take is taken there, once, for the phase.

The preamble correction (`880e68c`) stands as the record of the
prediction itself; this row is the record of what the phase does about
the gap the prediction left behind. T12's row in the control table now
names the discharge so the obligation is visible where it is executed
rather than only here.

## T5 — Per-window focus state and Tab traversal

### Start gate (recorded 2026-08-07, before any source edit)

Read before selecting:
[AGENTS.md](../../../../AGENTS.md),
[implementation-gates.md](../../../procedures/implementation-gates.md),
[plan.md](./plan.md) §T5 and §Cross-task obligations,
[preamble.md](./preamble.md) (§What "green" is worth, §The keyboard half
is two surfaces, §Review lanes),
[DD-M4-P2-003](../decisions/dd-m4-p2-003-focus-model-and-traversal.md) in
full,
[DD-M4-P2-001](../decisions/dd-m4-p2-001-event-routing-model.md)
§Recommendation (keyboard start point, unconsumed-key fallthrough),
[DD-M4-P2-002](../decisions/dd-m4-p2-002-hit-testing-and-generic-click.md)
§Recommendation (the resolved target a click focuses from),
[constraints.md](../requirements/constraints.md) §2 / §4 / §7 / §8 / §10,
the Moment-1 normative text
([architecture.md §13.2 / §13.3](../../../../docs/architecture.md),
[dsl_spec.md §4.19](../../../../docs/dsl_spec.md) §Focus / §Which keys the
runtime keeps, and §4.8's disabled contract), the
[T4 close gate](#t4--hover-and-pressed-behind-the-routing-model) and the
[T4 retrospective](../retrospectives/t4.md), and the landed source
(`focus_core.rs`, `focus_spike.rs`, `hit.rs`, `widget.rs`
`spike_focus_role` / `HoverState` / `update_hover` / `node_at_path_mut` /
`set_button_state_at` / `effective_button_color` / `update_button_enabled`
/ `update_toggle_button_checked` / `sync_visuals`, `window.rs`'s
`WM_KEYDOWN` and four pointer arms and `set_root`, `lib.rs`'s `ffi` seams
and the `__focus_spike` re-export, `examples/gallery/gallery.ui`).

#### Normative statements that already answer this task's behaviour

Recorded per
[DD-V-031](../../../cross-milestone/decisions/dd-v-031-normative-answers-at-start-gate.md).
This phase synchronised its normative text at Moment 1, so the questions
below are **answered**, not open, and none of them is an escalation.

| Question | Document | What it fixes |
|---|---|---|
| Where focus lives, and what it holds | [architecture.md §13.3](../../../../docs/architecture.md) | "Focus is per window. A `WindowState` owns one focus record holding the focused node and three derived stores" — the record is one object, not a field per store |
| What is focused at window open | §13.3, [dsl_spec.md §4.19](../../../../docs/dsl_spec.md) §Focus | Nothing. "No widget shows a focus indicator until the keyboard is used or a click places focus" |
| Traversal order and wrap | §4.19 §Focus | "Tab / Shift+Tab move focus in declaration order, wrapping at both ends; the first Tab lands on the first stop" |
| What a disabled Button does to traversal | §4.19 §Focus, [dsl_spec.md §4.8](../../../../docs/dsl_spec.md) | "A Button with `enabled: false` is **not** focusable — it is skipped by traversal and cannot be activated from the keyboard" |
| What a click does to focus | §4.19 §Focus, §13.3 | "A click moves focus to the nearest focusable widget at or above the widget it resolved to, and leaves focus unchanged when there is none — clicking background never clears focus" |
| What window activation does to focus | §4.19 §Focus, §13.3 | "Losing and regaining the window's activation does not change which widget is focused" |
| What happens to a key the runtime does not use | §13.2, §4.19 §Which keys the runtime keeps | "A key that reaches the end of the walk without a handler running is **not** consumed by the runtime: it continues to the window's default handling" |
| Which keys traversal takes | §4.19 §Which keys the runtime keeps | "`Tab` / `Shift+Tab` — Always the runtime; traversal cannot be overridden". Arrows and `Escape` are conditioned on a group / a scope, neither of which exists before T6/T7 |
| Where the indicator may be written | §13.3 | "presentation state applied by the same pass that writes visual geometry, **not a visual written at focus-change time**"; and it "shares its only means — a background change — with hover and the selected state, so the three must remain visually distinct" |
| Whether focus is authored | §4.19 §Not in this surface | "an attribute making a non-Button widget focusable" is outside this phase — M4 spells no opt-in, the derivation is the extension point (DD-003 F3) |

**One divergence is carried to T13 rather than resolved here**, and it is
in the indicator row. §13.3's "applied by the same pass that writes
visual geometry" cannot be implemented literally: `sync_visuals` runs
from a layout pass, a Tab press is not a state write and triggers no
layout, so an indicator applied there would not appear until something
else re-laid the tree out. Its second half — "not a visual written at
focus-change time" — *is* satisfiable and is what this task satisfies: no
`Visual` is created and no geometry is written, only the same
`CompositionColorBrush` colour transition hover already drives.
DD-M4-P2-003's own risk section names the same thing and names the
mitigation as an artifact rather than a prohibition — "the close artifact
for any task touching it is... every `SetOffset` / `SetSize` in the
runtime, with its pass" — which is #2 below. Recorded as a wording
divergence for the phase-close re-verification, not silently corrected.

#### Scope re-decided against the code — six facts measured first

Throwaway probes and greps, run before the approach was chosen.

1. **The whole key path is unexercised, and the host slot has no
   installer.** `rg "key_down_fn|key_down"` over the repo returns exactly
   three hits, all in `window.rs`: the field declaration, its `None`
   initialiser, and the invocation in the `WM_KEYDOWN` arm. `rg
   "WM_KEYDOWN"` over `wasamo-runtime/tests`, `examples` and `bindings`
   returns **zero** — no test anywhere sends a key message. The plan's
   description of the arm ("forwards to the uninstalled `key_down_fn` host
   slot and returns") is therefore measured true, and this task inherits
   T4's shape: the risk is not that the transition rule is hard but that
   **nothing in the suite would notice if it were wrong**.
2. **`focus_core` has no production caller and `focus_spike` has exactly
   one consumer.** `rg "focus_core|focus_spike|__focus_spike"` returns the
   module declarations, the `__focus_spike` re-export in `lib.rs`,
   `widget.rs::spike_focus_role`, and `tests/focus_mechanism_fixture.rs`.
   DD-M4-P2-003's "the spike's core is not yet load-bearing" is measured
   true, and this task is what makes it false.
3. **The six Composition geometry writes are unchanged and all inside
   `sync_visuals`** — `widget.rs` 2358 / 2363 (the node's own Visual),
   2407 / 2413 (the Button-family label), 2443 / 2449 (the `ScrollView`
   intermediate). `dip_scale.rs`'s two mentions are doc comments. This is
   the *before* half of DD-M4-P2-003's required close artifact; the task
   must not add a seventh.
4. **`effective_button_color` has five production call sites and eight in
   its own unit tests**, so widening its signature is compiler-forcing —
   the one part of this task's semantic migration Rust enumerates.
5. **The gallery's first Tab stop is a *checked* `ToggleButton`.**
   `gallery.ui`'s first Button-family widget in tree order is the "All"
   `ToggleButton`, bound `checked: tab_all_selected` whose initial state is
   `true`. So the first frame of any traversal evidence is the
   focused ∧ checked combination, which means the indicator's
   distinctness must hold against the **selected** colour and not only
   against `Normal` — the DD-M4-P2-003 requirement is fired by the
   gallery's very first stop rather than by a constructed case.
6. **The negative prediction this task depends on is about the
   fallthrough, and it is not yet measured** (the T4 retrospective's new
   start-gate line). The claim "an unconsumed key's fallthrough to
   `DefWindowProc` cannot be distinguished by the returned `LRESULT`,
   because `DefWindowProcW` returns 0 for `WM_KEYDOWN`" is a *prediction*
   about the OS, and T4's lesson is that a negative prediction nobody
   measures survives unchallenged. It is therefore measured during
   implementation over a candidate key set before the fixture's assertion
   shape is chosen, and the measurement is recorded either way.

**What T5 therefore is.** Not "wire the spike up", but four things with a
boundary each: a per-window focus record whose only writer paints in the
same primitive; a production projection that derives `Stop` / `Container`
from the widget kind; Tab / Shift+Tab consumed ahead of the host key slot
with **every other key falling through to `DefWindowProc`**; and
click-to-focus on the nearest focusable widget at or above the resolved
target.

Three boundaries are drawn deliberately and recorded in
[plan.md](./plan.md) §T5 rather than left implicit:

- **The key *walk* is not built here.** §13.2 has keyboard messages enter
  the propagation walk at the focused widget, but no authored key handler
  can exist until T8 adds `key-down("<key>")` — a walk built now would be
  a branch no test could fire, which is exactly what trap #4 forbids. T5
  lands the *consumption* half (traversal takes Tab) and the *fallthrough*
  half; T8 lands the dispatch between them, against handlers that exist.
- **Disabling a Button that holds focus applies the successor rule
  lazily, at the next traversal.** DD-M4-P2-003 requires that a widget
  disabled while focused "stops being a stop and the successor rule below
  applies". `update_button_enabled` is on the *binding* path with no
  window in reach, and T4 refused to give that path a second producer of
  the hover record for the same reason — so no eager writer is added.
  Nothing is lost that a test can see: `tab_stops` already excludes a
  disabled stop and `tab()` from a focus that is not in the stop list
  starts at the domain's first stop, which is the same answer
  `focus_after_removing` gives. The Backward case differs (it lands on the
  last stop, not the first) and is asserted rather than assumed.
- **Focus moves on `WM_LBUTTONUP`, before dispatch.** The same message
  that dispatches `clicked` (T3), so the widget that takes focus is the
  widget the click activates — the release-time analogue of §13.2's
  "the widget that paints pressed is the widget a release would dispatch
  to" — and *before* the handler runs, because a handler's synchronous
  rebuild can invalidate the resolved path (T3's §2 enumeration).

#### Trap selection

| # | Trap | Applies | Reason |
|---|---|---|---|
| 1 | Semantic-migration miss | **yes** | Two migrations in one task. `ButtonData` gains a field and `effective_button_color` gains a parameter — both compiler-forcing (fact 4). The `WM_KEYDOWN` arm's *return path* migrates from "always `LRESULT(0)`" to "consumed ⇒ 0, otherwise `DefWindowProcW`", and the compiler enumerates **none** of that. The audit table therefore covers every writer and reader of the new field, every caller of the colour function, and every site that decides whether a key is consumed |
| 2 | Missed side effects | **yes** | Retained per-window state with a cross-path invalidation surface, the same class T4 met. To enumerate before writing: `window::set_root` replacing the tree the record indexes; `update_button_enabled`'s binding-path state reset (fact above); `update_toggle_button_checked`'s brush rebuild, which must now read the focus flag or a focused ToggleButton loses its indicator on the next `checked` write; a click handler's synchronous rebuild between the focus write and the dispatch that follows it in the same arm; the interaction of the focus colour with the hover colour on the **same brush**; and the Composition geometry writes, which must stay at six (fact 3, and DD-M4-P2-003's named close artifact) |
| 3 | Parallel/derived data drift | **yes** | The window's focus record and the painted per-node flag are a derived pair with the same invariant shape T4's `HoverState` carries: at most one node is painted focused, and it is exactly the node the record names. They must be written in one primitive, and the record's own field must be unwritable from outside it. The focus record additionally carries `focus_core`'s group memory, whose single-writer discipline DD-M4-P2-003 adopts as a requirement — unexercised until T7, but the primitive that would break it is added here |
| 4 | Untested authored branch | **yes** | New arms: Tab forward; Shift+Tab backward; the wrap at each end; a disabled stop skipped; a key that is not traversal's falling through; the click that finds a focusable ancestor; the click that finds none (focus unchanged, **not** cleared); a focus change that leaves the previously focused node. Each ships with a test that fires it directly, and each is put under a deliberately wrong implementation shown to redden it. [DD-V-029](../../../cross-milestone/decisions/dd-v-029-pure-logic-red-test-obligation.md)'s **named** obligation is not triggered — no rounding, unit conversion or boundary condition is authored here; the traversal wrap is a boundary condition but it is `focus_core`'s, landed and unit-tested by the spike, and this task adds no arm to it |
| 5 | Carry-forward underweighted | **yes** | T6 gives the projection its authored roles; T7 retires `focus_spike` and owns the eager successor and the id-stability exposure; T8 adds the key walk between the consumption and the fallthrough; T12 inherits the indicator frames. Per the T3 retrospective's corrective, anything this task *requires as evidence of a later task* is built and run here first, or recorded as a finding with an owner rather than as a carry-forward |
| 6 | Symptom taken at face value | **conditional** | No deterministic failure is in hand at the start gate. Selected as armed: any failure during implementation gets a minimal repro and a root cause, not a re-roll |
| 7 | Weak GUI evidence | **yes** | The plan requires it by name — "the indicator must be **distinguishable**... Evidence includes a frame pair showing focused versus hovered/selected as visibly distinct states" — and fact 5 makes the hardest comparison (focused ∧ checked versus checked) the gallery's *first* stop. A state read-back cannot answer it: `__button_state_for_test` would report the same `"normal"` for a focused and an unfocused Button, so only a captured frame can say the three states differ |

```
- [x] #1 semantic migration   - [x] #2 side effects   - [x] #3 parallel data   - [x] #4 branch tests
- [x] #5 carry-forward        - [~] #6 root cause     - [x] #7 GUI positive control
```

#### Review lane

**Full independent review**, as
[preamble.md §Review lanes](./preamble.md) predicts and as the change
confirms: a runtime structural change (retained per-window focus state,
the first production caller of the traversal core, and the `WM_KEYDOWN`
return-path change) carrying GUI-render evidence — two of the three
high-risk classes in
[implementation-gates.md §4](../../../procedures/implementation-gates.md).
The trap-#4 branch/test check composes into it for the new arms.

#### The T4 correctives, applied

T4's retrospective added one line to later start gates and one to later
close gates. The start-gate line is answered here:

- *Which negative prediction of the phase documents does this task depend
  on, and has it been measured once?* Two. The plan's description of the
  `WM_KEYDOWN` arm and DD-M4-P2-003's "the spike's core has no production
  caller" are both measured true (facts 1 and 2). The third — that the
  fallthrough is invisible in the returned `LRESULT` — is an assumption
  about the OS rather than about this repo, is **not** measured yet, and
  fact 6 makes measuring it a precondition of choosing the fixture's
  assertion shape rather than a footnote to it.

The close-gate line ("the mutation witnesses must include one that
restores the pre-fix behaviour") is discharged at the close gate: for this
task the pre-T5 behaviour is *no focus at all*, so the restoring mutation
is the one that deletes the focus move and leaves the arm returning
`LRESULT(0)`.

#### Planned proof obligations

Each closed at the T5 close gate:

1. The call-site audit table over the new field, the widened colour
   function, and every site that decides whether a key is consumed.
2. The structural side-effect enumeration, including `set_root`, the
   binding-path `enabled` reset, the `checked` brush rebuild, and the
   click-handler rebuild inside the same message arm.
3. **The `SetOffset` / `SetSize` enumeration with each call's pass**
   (DD-M4-P2-003's named artifact), showing the set is still the six of
   fact 3.
4. The parallel-data statement naming the single primitive that writes
   the record and the painted flag together.
5. Pure-logic unit tests: the key-to-command mapping, the
   nearest-focusable-at-or-above walk, and the indicator's distinctness
   from `Normal`, `Hovered`, `Pressed` and the selected colour across
   every style / state combination.
6. Integration fixtures over real messages, each establishing its own
   initial focus state (Phase 1 F-47) and asserting the **expected next
   stop** rather than that focus moved.
7. The fallthrough measurement of fact 6, recorded with whichever
   assertion shape it licenses.
8. Mutation witnesses for each new arm — including the restoring one —
   each read back from the file before it is run and re-read after the
   revert.
9. The GUI evidence: focused versus hovered versus selected versus
   normal, with an agreement leg and a control that a wrong
   implementation would fail.
10. The whole task list re-read at the close gate (the re-audit
    discipline, [plan.md](./plan.md) §Cross-task obligations).

### Close gate (recorded 2026-08-07)

Landed: `wasamo-runtime/src/focus.rs` (`FocusProjection` with its per-id
tree paths, `WindowFocus`, the pure `tab_direction` / `nearest_focusable`,
the `move_focus` primitive, `traverse_on_key`, `focus_on_click`,
`discard_stale_focus`, `focused_path`, seven unit tests);
`ButtonData::focused` with `WidgetNode::set_button_focused_at` as its sole
writer; `effective_button_color` widened with the focus axis plus
`focus_indicator_color` / `FOCUS_INDICATOR_COLOR` /
`FOCUS_TRANSITION_TICKS` and two distinctness unit tests;
`WidgetNode::focus_role` (the production derivation) and
`__button_focused_for_test`; `WindowState::focus` with its `set_root`
reset, the rewritten `WM_KEYDOWN` arm and the focus write in
`WM_LBUTTONUP`; `ffi::__focus_path_for_test`;
`wasamo-runtime/tests/focus_traversal_integration.rs` (six fixtures);
`evidence/capture-t5-focus.ps1`.

#### #1 — Call-site audit table

Two migrations. Rust enumerates one of them and none of the other, so the
table covers both and says which is which.

Queries:
`rg "effective_button_color" wasamo-runtime/src`,
`rg "ButtonData \{" wasamo-runtime/src`,
`rg "focused" wasamo-runtime/src/widget.rs`,
`rg "WindowFocus|state\.focus" wasamo-runtime/src wasamo-runtime/tests bindings examples wasamo-dll`,
`rg "set_focus|set_button_focused_at|move_focus|traverse_on_key|focus_on_click|focused_path" wasamo-runtime/src`,
`rg "WM_KEYDOWN|key_down_fn" wasamo-runtime/src wasamo-runtime/tests examples bindings`.

**The compiler-forced half.** `ButtonData` has exactly one struct literal
(`widget.rs:1019`, inside `button_family`, behind which `button` and
`toggle_button` both delegate) and no `Default`, no `..` update syntax and
no builder, so the new field could not be missed at construction.
`effective_button_color`'s parameter addition broke all five production
call sites and all eight in its own unit tests.

| `effective_button_color` call site | Classification | Reason |
|---|---|---|
| `button_family` constructor | must-pass `false` | Construction precedes any focus; the invariant's base case |
| `update_button_style` | must-pass `btn.focused` | Rebuilds the brush from scratch; without the flag a focused Button loses its indicator on a style write |
| `update_button_enabled` | must-pass `btn.focused` | Same, and it is the path that disables a Button *while focused* — the flag must be carried so re-enabling repaints the indicator rather than dropping it |
| `update_toggle_button_checked` | must-pass `btn.focused` | Same. The gallery fires this on every toolbar click, so a dropped flag would be visible in the first frame of the GUI evidence |
| `set_button_state_at` | must-pass `btn.focused` | The hover transition and the focus indicator share one brush; the hover write has to carry the focus axis or hovering a focused Button would erase its indicator |

**The half the compiler enumerates nothing of.** No type changed for the
key path or for the focus record, so these rows are the artifact.

| Site | Classification | Reason |
|---|---|---|
| `focus.rs::move_focus` | **the sole writer of the pair** | The only function in the crate that calls both `set_button_focused_at` and `focus_core::FocusState::set_focus` |
| `widget.rs::set_button_focused_at` | **the sole writer of the painted flag** | `pub(crate)`; `rg` shows its only two call sites are inside `move_focus` |
| `focus_core::FocusState::set_focus` | **the sole writer of the focused id** | Production callers: `move_focus` and `discard_stale_focus`, both in `focus.rs`. Its other in-crate callers (`enter_modal`, `exit_modal`, `apply_arrow`) have no production caller until T7; the rest are `focus_core`'s own unit tests |
| `window.rs::set_root` | **new** | Resets the record; the previous root is dropped just above and its indices name nothing in the new tree — the same statement, for the same reason, as T4's hover reset beside it |
| `window.rs` `WM_KEYDOWN` arm | **migrated** | The return path changes from unconditional `LRESULT(0)` to "consumed ⇒ 0, otherwise fall through to `DefWindowProcW`". Nothing in the language forces this to be revisited, which is why it is here |
| `window.rs` `WM_LBUTTONUP` arm | **migrated** | Gains the focus write, ordered before `hit_test_click` |
| `focus.rs::tab_direction` | **new, sole key predicate** | The one place a virtual key is classified as traversal's |
| `focus.rs::nearest_focusable` | **new, sole focusability test on the click path** | Defined against `FocusTree::tab_stops`'s own output rather than as a second predicate |
| `widget.rs::focus_role` | **the sole role derivation** | Shared by the production projection and `focus_spike`'s override projection, so the two cannot disagree about what a widget kind is |
| `widget.rs::update_hover` / `clear_hover` / `set_button_state_at`'s state half | ignore-OK | Hover is a different axis on the same brush. Untouched except for passing the focus flag through |
| `lib.rs::ffi::__focus_path_for_test` | **new, read-only** | Projects on demand and maps the id to a path; holds no state of its own |
| `abi.rs` | ignore-OK, unchanged | No `extern "C"` function added or altered; the cross-task "no new ABI function" obligation holds |
| `tests/focus_mechanism_fixture.rs` | ignore-OK, re-read | Drives `focus_spike`'s override projection, which is untouched. Still passes; T7 retires it |

**What no existing test pinned.** `rg "WM_KEYDOWN"` over
`wasamo-runtime/tests`, `examples` and `bindings` returned **zero** before
this task, and `key_down_fn` had no installer anywhere. The entire key path
could have been written any way at all and the suite would have stayed
green — this task's instance of
[preamble.md §What "green" is worth](./preamble.md), and the reason the
evidence rather than the transition rule is where the work went.

#### #2 — Structural side-effect enumeration

| Derived effect | Disposition |
|---|---|
| **Which node paints the focus indicator** | New. Exactly the node the window's record names, written in the same primitive as the record |
| **The hover brush** | Shared, deliberately: focus and hover are two axes on one `CompositionColorBrush`. `effective_button_color` takes both, so every path that recomputes the colour carries both, and hovering a focused Button keeps the indicator |
| **`update_button_enabled` on a focused node** | Enumerated, not fixed. A binding write that disables a focused Button leaves the record naming it and the flag set, while the paint becomes the flat disabled grey (`effective_button_color` checks `enabled` before it reads `focused`). Traversal already excludes a disabled stop, so the **next** Tab lands on the domain's first stop — DD-M4-P2-003's successor rule applied lazily. Making this eager would put a focus producer on the binding path, which is the shape T4 refused for hover and DD-M4-P2-001 refuses generally. Asserted by F3's third leg |
| **`update_toggle_button_checked`** | Now carries the focus flag. Without it the gallery's own toolbar would drop the indicator on the first click, which is exactly the frame the GUI control captures |
| **`window::set_root`** | Resets the record. Without it a path from the dropped tree would be applied to the new one |
| **A click handler's synchronous rebuild, mid-`WM_LBUTTONUP`** | The reason the focus write is ordered **before** `hit_test_click`: the widget that takes focus must be the one the user touched, resolved against the tree that was on screen. Its consequence — a retained id outliving the projection that produced it — is the crash dispositioned in #6 |
| **The `WM_KEYDOWN` return path** | Changed by design. An unconsumed key reaches `DefWindowProcW`, which can dispatch further messages into `wnd_proc` synchronously; a nested frame derives its own `&mut WindowState` from `GWLP_USERDATA`. Sound because nothing uses the outer `state` after the fallthrough leaves the arm — a different argument from `WM_DPICHANGED`'s, which is hoisted above the borrow precisely because it *does* use `state` afterwards. Stated at the site |
| **The host key slot** | Reached only after traversal declines. Still uninstalled in production ([constraints §2](../requirements/constraints.md): the first installer fixes the callback unit as shipped API, and this phase is not it); F5 installs a recorder for the length of one fixture, which is what makes consumption observable at all |
| **Win32 activation** | Untouched: `wnd_proc` has no `WM_SETFOCUS` / `WM_KILLFOCUS` arm, so neither can reach the record. `docs/dsl_spec.md` §4.19's "losing and regaining the window's activation does not change which widget is focused" holds by construction, and F1 asserts it rather than leaving it implied |
| **`ButtonData.label_size`'s three-point write** ([constraints §4](../requirements/constraints.md)) | Not touched |
| **Composition geometry writes** | Unchanged — see #3 |
| **Layout invalidation** | None added. A focus change repaints a brush and triggers no layout pass, which is what makes the literal reading of §13.3's indicator sentence unimplementable (recorded at the start gate as a divergence for T13) |

#### #3 — Every `SetOffset` / `SetSize` in the runtime, with its pass

DD-M4-P2-003 names this as the required close artifact for any task
touching the indicator, because "drawing at focus-change time is the
obvious implementation" and a geometry write outside the single pass would
silently break what DD-M4-P1-002's audit closed.

Query: `rg "SetOffset|SetSize" wasamo-runtime/src`.

| Site | Pass | What it writes |
|---|---|---|
| `widget.rs:2452` / `:2457` | `sync_visuals` | The node's own Visual offset and size |
| `widget.rs:2501` / `:2507` | `sync_visuals` | The Button-family label Visual |
| `widget.rs:2537` / `:2543` | `sync_visuals` | The `ScrollView` intermediate content Visual |
| `dip_scale.rs:102` / `:112` | — | Doc comments naming the operations, not calls |

**Six calls, all inside `sync_visuals`, unchanged from T1 / T2 / T3 / T4.**
The focus indicator adds none: it is a `CompositionColorBrush` colour
transition through the same `start_color_anim` hover already drives, and
creates no `Visual`. The half of §13.3 that says "not a visual written at
focus-change time" is therefore satisfied literally; the half that says
"applied by the same pass that writes visual geometry" is the divergence
recorded at the start gate for T13's re-verification.

#### #4 — Parallel-data sync

The retained focused id and the painted `ButtonData::focused` flag are the
derived pair, with the invariant "at most one node paints the indicator,
and it is exactly the node the record names".

- **One primitive.** `move_focus` clears the previous node's paint, sets
  the next node's, and writes the record, in one body. `WindowFocus::core`
  is a private field with no setter and only a read-only `focused()`
  accessor, so no other module can write the record; `set_button_focused_at`
  is `pub(crate)` and has exactly two call sites, both inside `move_focus`.
- **Asserted, not assumed.** Every fixture reads both halves after every
  step where either could have changed — the node the record names *and*
  the node it previously named — through `assert_focused_stop`. Witness W2
  (the enter paint dropped) reddens all five of F1–F5 on the pairing
  message; W3 (the leave dropped) reddens F2 and F3 on the
  "previously focused node must have stopped painting" message.
- **`focus_core`'s own parallel store** — the per-group memory
  DD-M4-P2-003 requires be written by the same primitive as the focus
  pointer — is untouched and stays enforced by visibility inside
  `focus_core::FocusState::set_focus`. No `Group` role exists before T6/T7,
  so the map is empty this task; the discipline is inherited rather than
  re-implemented.
- **The one path that can break the pair from outside** is
  `update_button_enabled`, enumerated in #2.

#### #5 — Branch tests, each fired directly

| Authored arm | Test that fires it |
|---|---|
| `Tab` forward | `tab_walks_the_stops_in_declaration_order_and_wraps_at_both_ends` (steps 1–3) |
| `Shift+Tab` backward | the same fixture's last two steps, and `focus::tests::tab_with_shift_is_backward` |
| Wrap at the forward end | the same fixture, Tab from the last stop |
| Wrap at the backward end | the same fixture, Shift+Tab from the first stop |
| A key that is not traversal's | `focus::tests::a_non_tab_key_is_not_traversals_regardless_of_shift`, and `tab_is_consumed_by_traversal_while_an_unclaimed_key_is_not` leg 1 |
| `Tab` consumed with **no stop in the domain** | the same fixture's leg 2 (the recorder is what makes it discriminating) |
| A disabled stop skipped | `a_disabled_button_is_skipped_by_traversal_and_a_button_disabled_while_focused_loses_it_at_the_next_tab` (difference leg) |
| …and reachable when enabled | the same fixture's agreement leg |
| A stop disabled *while focused* | the same fixture's third leg |
| Click focuses the target | `a_click_focuses_the_nearest_focusable_widget_at_or_above_the_target_and_a_background_click_changes_nothing` leg A |
| Click on a non-focusable widget leaves focus unchanged | the same fixture's leg B |
| Click on no widget of its own leaves focus unchanged | the same fixture's leg C |
| `nearest_focusable`'s arms (target itself / an ancestor / none / empty chain) | `focus::tests::nearest_focusable_*` (four unit tests) |
| Nothing focused at window open | `nothing_is_focused_when_a_window_opens_and_window_activation_does_not_change_it` (whole-tree walk, with the visited count asserted so the walk cannot be vacuous) |
| Activation messages leave focus alone | the same fixture's second half |
| A retained id the projection cannot explain | `a_focus_record_naming_a_removed_stop_is_cleared_rather_than_read_back_stale` |
| The indicator's distinctness from Normal / Hovered / Pressed | `widget::tests::focused_normal_is_distinct_from_every_unfocused_hover_state` |
| …and from the selected colour | `widget::tests::focused_unchecked_is_distinct_from_unfocused_checked_the_selected_confusion` |
| A disabled widget's grey survives the focus flag | `widget::tests`'s disabled-colour test, looped over `focused` |

[DD-V-029](../../../cross-milestone/decisions/dd-v-029-pure-logic-red-test-obligation.md)'s
**named** obligation is not triggered: this task authors no rounding, no
unit conversion and no boundary condition. The traversal wrap *is* a
boundary condition, but it is `focus_core`'s, landed and unit-tested by
the spike, and no arm of it changed here. The witnesses below are the
trap-#4 / #6 artifact.

#### #6 — Deterministic-failure disposition and the mutation witnesses

**The deterministic failure this task found is a crash, and it was found
by review rather than by a red test.** Reading the landed `move_focus`
raised the question of what a retained `FocusId` means after the tree
shrinks; a throwaway probe answered it:

```
thread ... panicked at wasamo-runtime\src\focus.rs:63:20:
index out of bounds: the len is 2 but the index is 4
```

Minimal repro: a `Button` inside an `if` branch whose own `clicked`
handler clears the branch's condition. `focus_on_click` focuses it before
dispatch (id 4), the handler's synchronous drain removes the subtree, and
the next projection has two nodes. Root cause: `FocusId` is the pre-order
index of a projection rebuilt per operation, while the record outlives any
one projection. **The shipped gallery reaches it** — the lightbox is an
`if is_lightbox_open` branch and its Buttons are the highest pre-order
ids, so focusing the close control and then removing it is the same shape.
The first probe run measured nothing because it sent DIP coordinates where
`wnd_proc` expects physical; that is recorded because the null result was
*the probe's*, not the runtime's, and re-rolling it as "no crash here"
would have closed the question falsely.

Disposition: fixed in this task (`05d46f9`), not carried. Three call paths
could reach a stale id — `FocusProjection::path`, `focused_path`, and
`FocusTree::tab` through `focus_core`'s own node vec — and all three are
closed, the first by returning `Option` so the case is unrepresentable at
the call site. The regression fixture crashes rather than merely failing
if the fix regresses.

**Nine mutation witnesses.** Every one was applied with an edit, **read
back from the file** to confirm it was present before the run, run, then
reverted with the revert confirmed by re-reading (the T2 corrective,
carried by T3 and T4). No failure was re-rolled: the suite went red only
where a mutation was deliberately introduced.

| Witness | Mutation | Went red | Reading |
|---|---|---|---|
| **W1 — the pre-T5 arm restored** (the T4 corrective's required restoring witness) | `WM_KEYDOWN` swallows every key again: no traversal call, `return LRESULT(0)` after the host slot | F1, F2, F3, F5 — and **nothing else in the workspace** | The decisive one. It reconstructs the before-state of this task rather than mutating its implementation, so it answers "do these fixtures catch the absence of the feature", not merely "do they watch their own code". That F4 stays green is correct — click-to-focus does not travel `WM_KEYDOWN` — and that the other 43 binaries stay green is the measurement behind "the key path was entirely unpinned" |
| **W2 — the pair, enter half** | `move_focus` drops `set_button_focused_at(.., true)` | **All five** of F1–F5, F1 on the trap-#3 pairing message | The record and the paint cannot be edited apart without every fixture noticing |
| **W3 — the pair, leave half** | `move_focus` drops the previous node's clear | F2 and F3 alone, on "the previously focused node must have stopped painting" | Only the two fixtures that move focus twice can see it, which is why the leave is asserted separately from the enter |
| **W4 — Shift ignored** | `tab_direction` always returns `Forward` | `focus::tests::tab_with_shift_is_backward` **and** F2's two Shift+Tab legs | The division of labour: the unit test names the property, the fixture shows it survives the real message path — and incidentally confirms `SetKeyboardState` reaches `wnd_proc`'s own `GetKeyState` |
| **W5 — a disabled stop counts as enabled** | `focus_role` returns `enabled: true` for Button-family | F3's difference leg alone | The agreement leg stays green, which is exactly why the pair exists |
| **W6 — click-to-focus loses its stop filter** | `focus_on_click` focuses the resolved target whether or not it is a stop | F4 (leg B) and F3's third leg | "At or above the nearest **focusable**" is what fails, not "a click focuses something" |
| **W7 — the host slot runs before traversal** | `key_down_fn` invoked ahead of `traverse_on_key` | F5 alone, on "routing consumes ahead of the host key slot", reporting the recorder saw `[9]` | The mutation the fixture could not have caught as first written: the returned `LRESULT` is 0 either way, and the original leg asserted only that. The recorder is what makes the ordering claim falsifiable |
| **W8 — the indicator is not distinguishable** | `focus_indicator_color` returns `base` | `widget::tests::focused_normal_is_distinct_…` alone — **and none of the six fixtures** | A stated coverage boundary, measured rather than assumed: the fixtures read a boolean flag, so the indicator's *appearance* is invisible to them. Only the pure colour test and the GUI capture at #7 can see it, which is why trap #7 applies to this task |
| **W9 — the stale-id fix removed** | `discard_stale_focus` and the `Option`-returning `path` reverted, the new fixture kept | `a_focus_record_naming_a_removed_stop_is_cleared_…` **by panicking**, with the same `index out of bounds: the len is 2 but the index is 4` | The regression fixture fails by crashing, not by assertion, which is the failure mode the defect actually has |

Suite state after all reverts, on the post-commit tree:
`cargo fmt --all -- --check` zero exit, `git diff --check` clean,
`cargo test --workspace --no-fail-fast` **44 binaries/sections, 1,051
passed, 0 failed, 0 ignored, 0 skipped**. T4's baseline was 1,036; the
fifteen added are `focus.rs`'s seven unit tests, `widget.rs`'s two
distinctness tests, and `focus_traversal_integration.rs`'s six fixtures.
Skip 0 means the Compositor was available, so the integration assertions
actually ran.

#### #7 — GUI evidence with a positive control

[evidence/capture-t5-focus.ps1](./evidence/capture-t5-focus.ps1), run
against `cargo build --release --workspace` on a 982x703 client at 120 DPI
(**scale 125%**), two frames per side, the **client** rectangle mapped with
`ClientToScreen`, PMv2 declared **and read back** (Phase 1 F-48).

The sample is the gallery's first Tab stop, the **checked** "All"
`ToggleButton` — the hardest of DD-M4-P2-003's three comparisons rather
than a constructed one, because it is already painting the selected colour
before focus arrives. The mask is derived once from the baseline frame by
the same blue predicate T4's capture used, so no frame showing an effect
can influence where the effect is measured; it came out **byte-identical
to T4's** (2,559 px, bbox x[10..69] y[13..56]), which makes the two runs
directly comparable.

| Mean RGB over the mask | R | G | B |
|---|---|---|---|
| N — nothing focused, cursor away | 46.99 | 117.65 | 213.41 |
| H — hovered, nothing focused | 74.06 | 138.90 | 224.06 |
| FA — Tab ×1, focused, cursor away | 140.88 | 150.55 | 146.65 |
| FB — Tab ×4, focus moved on | 46.99 | 117.65 | 213.41 |
| **FA − N (focused vs selected)** | **+93.89** | **+32.90** | **−66.76** |
| **FA − H (focused vs hovered)** | **+66.82** | **+11.65** | **−77.41** |
| **FB − N (agreement leg)** | **0.00** | **0.00** | **0.00** |
| within-side jitter, every side | 0.00 | 0.00 | 0.00 |

**All three legs are needed.** The two difference legs are DD-M4-P2-003's
requirement made numeric, and they move in a direction hover cannot: hover
brightens (+27 / +21 / +11, every channel up), while focus shifts hue (R
strongly up, B strongly down), so the two are not two degrees of one
signal. **FB − N is the control**: when focus moves to the fourth stop the
sampled button returns to *exactly* its unfocused colour, so an
implementation that painted the indicator but never cleared it fails here
while passing every difference leg above. The script also carries a guard
that is not a leg — if FA were within the noise floor of N it throws,
because that would mean the key never arrived and every comparison below
it would be vacuous.

N and H reproduce T4's numbers for the same pixels to the last recorded
digit, and two independent runs of this script (separate process launches)
were bit-identical on all eight frames.

**Foreground activation is acquired before any key is sent, verified, and
retried.** Mouse input is routed by cursor position and needs no
activation, which is why T4's hover capture never met this; keyboard input
is routed to the focused window of the *foreground* thread. The script
therefore earns activation with a real click inside the client area (at
the parking point, which changes nothing it measures), reads
`GetForegroundWindow` back, and retries up to five times before giving up —
and records which input path it used. Both recorded runs used **real key
presses**.

An early version of the script asked for activation without earning it and
then, once it clicked, gave up after a single attempt; that attempt was
refused on a freshly created-shown-repositioned window. Reading one refusal
as an environment verdict would have been the wrong conclusion from a
sample with no retry in it. The rule this produced is not local to this
capture and is recorded where capture mechanics live
([verification-environments.md](../../../../docs/notes/verification-environments.md)
Observation 4): anything synthesizing keyboard input acquires foreground,
verifies it, and retries. The posted-`WM_KEYDOWN` fallback remains as the
**weaker claim** — it exercises the message loop and window procedure but
not the OS input stack that decides which window a key reaches — and has
consequently never been exercised.

This is the assistant baseline and does **not** replace the owner's
human-visible smoke ([CLAUDE.md §Testing rules](../../../../CLAUDE.md)).

#### #8 — Carry-forward

| Constraint | Evidence | Placement | Re-trigger criterion |
|---|---|---|---|
| **CF-T5-1 — a retained `FocusId` that is still *in range* but names a different node after a rebuild is not detected.** `discard_stale_focus` catches only the out-of-range case | The fix's own doc comment; the in-range case has no tree shape M4-Phase 2 can build, so no test fires it | `carry-forward` → this ledger | **T7** (modal-scope materialisation and removal) and **T9** (`for` regeneration) — the two tasks that add paths creating or replacing subtrees. This is the focus twin of T4's CF-T4-1 for hover; both are the same index-shift class |
| **CF-T5-2 — the eager successor rule is not implemented.** DD-M4-P2-003 requires the successor of a removed or disabled stop to be computed **before** the mutation; T5 applies it lazily at the next traversal, landing on the domain's first stop | F3's third leg and F6 assert the lazy behaviour by name | `carry-forward` → this ledger | **T7**, which owns the materialisation seam that can call in *before* a mutation. Until then the observable difference is that a removal falls to the first stop rather than to the structural successor |
| **CF-T5-3 — the fixtures cannot see the indicator's appearance.** W8 deletes the colour distinction and none of the six fixtures notices; they read a boolean flag | Witness W8 | `carry-forward` → this ledger | **T12**, whose control B reads traversal order off captured frames. Any later change to `effective_button_color` needs the pure colour tests or a capture, not the fixtures |
| **CF-T5-4 — anything synthesizing keyboard input must acquire foreground activation first, verify it, and retry.** Keyboard input is routed to the focused window of the foreground thread, so a key sent without it goes elsewhere; `SetForegroundWindow` is refused unless the caller is already foreground, so activation is earned with a real click and read back. A capture that instead posts `WM_KEYDOWN` supports a weaker claim and records that it did | This capture's own history: asking without earning failed, and a single earned attempt on a freshly shown window was refused | `doc-folded` → [verification-environments.md](../../../../docs/notes/verification-environments.md) Observation 4, beside the PMv2 read-back rule it is the sibling of | **T12** (control B is Tab-driven) and **T11** (synthesized pointer injection carries the same "earn the capability, verify it, record which path" shape). The rule is in the capture-mechanics SSOT, so a later task inherits it by reading that file rather than by finding this row |
| **CF-T5-5 — the host key slot is now load-bearing for evidence, and still uninstalled in production.** F5's discrimination depends on `key_down_fn` being reached only after traversal declines | F5 and witness W7 | `carry-forward` → this ledger | **T8**, which adds authored `key-down` handlers between the consumption and the fallthrough. A dispatch inserted on the wrong side of the slot breaks F5, which is the intended tripwire |
| **CF-T5-6 — §13.3's indicator sentence cannot be implemented literally.** "Applied by the same pass that writes visual geometry" would require a focus change to trigger a layout pass; the landed indicator is a brush colour written through the same primitive hover uses, and adds no geometry write | The #3 enumeration (six calls, unchanged) | `finding` → **T13's re-verification list** | **T13**, which re-verifies §13.3 against the landed runtime. The clause that *is* satisfied literally is the same sentence's second half, "not a visual written at focus-change time" |

#### Re-decided at close

The start gate selected traps 1–5 and 7 with 6 armed. **The selection
survived, and trap 6 fired.** It was armed against "a failure that appears
during implementation"; what happened instead is that a crash was found by
*reading* the landed code and then reproduced deliberately — the trap's
artifact obligation (minimal repro, root cause, disposition) applies
unchanged, and #6 discharges it against a real defect rather than leaving
it armed as T4's stayed.

Two things were built that the gate did not predict: a regression fixture
for that crash (F6, which the gate could not have named because the defect
was not known then), and a fallback input path in the capture tool that
the measured environment then did not need. Neither changes the review
lane: full independent review was predicted, confirmed at the start gate,
and is what the change still is.

#### Re-audit of the whole task list

Per [plan.md](./plan.md) §Cross-task obligations, the full list was re-read
at this close gate rather than only T5's item.

- **T6** — unaffected by what this task built; it adds the `focus-group` /
  `modal-scope` attributes the projection will read. What it inherits is
  the shape of the reader: `WidgetNode::focus_role` is the single
  derivation, so T6's attributes reach traversal by widening that function
  rather than by adding a second source of roles.
- **T7** — inherits three things concretely. CF-T5-1 and CF-T5-2 are its to
  close (the materialisation seam is the only place a successor can be
  computed before the mutation). It also retires `focus_spike`, the
  `__focus_spike` seam and the override map, at which point
  `tests/focus_mechanism_fixture.rs` re-points at the production
  projection — which now exists, so that retirement is a deletion rather
  than a rewrite. And `discard_stale_focus` is the function whose reason
  disappears once entry / exit compute the successor properly; it should be
  re-examined then rather than left as a permanent fallback.
- **T8** — inherits CF-T5-5. Its key dispatch lands **between** traversal's
  consumption and the `DefWindowProc` fallthrough this task built, and its
  assertions about the keys the runtime keeps (`Tab` always) run against
  `focus::tab_direction`, which is where that answer now lives. Its CF-2
  work is re-confirmed still open: F3 had to author its disabled stop as a
  `ToggleButton` literal or a state-bound `Button`, and says so.
- **T9** — the same index-shift exposure as T7, through `for` regeneration.
- **T10** — first production consumer of Tab traversal in a `.ui` the owner
  drives. The gallery already traverses correctly (the #7 capture is a
  gallery frame), so T10's work starts from a gallery whose keyboard
  already works rather than from one that needs wiring.
- **T11** — touch inherits nothing from focus: a `WM_POINTER*` arm would
  have to call `focus_on_click` explicitly, exactly as CF-T4-4 records for
  hover. Whether a touch contact should move focus is that task's explicit
  decision, not something to inherit by omission.
- **T12** — inherits CF-T5-3 and CF-T5-4. Control B (traversal order) is
  Tab-driven, so it needs the input-capability measurement this task's
  script makes; and its frames are the only thing that can see the
  indicator, which is what makes control B possible at all.
- **T13** — gains CF-T5-6 on its re-verification list. The §4.19 focus
  bullets otherwise match the landed runtime and are a **confirmation**
  rather than a re-derivation: nothing is focused at open, Tab / Shift+Tab
  walk declaration order and wrap at both ends, a disabled Button is
  skipped, a click focuses the nearest focusable at or above and never
  clears, and activation does not change focus. Each has a named fixture in
  the #5 table, so T13 can check the list rather than re-read the code.
- **Cross-task obligation "no new ABI function"** — held. `focus.rs` is a
  private module and the one new symbol on the `ffi` surface is a
  `__*_for_test` seam, not a C entry point; no `extern "C"` function was
  added or changed.

#### Verification means

The six fixtures reuse `tests/common/mod.rs`'s skip guard **unchanged**, so
the standing obligation to verify a newly authored guard on an environment
that lacks the capability
([CLAUDE.md §Testing rules](../../../../CLAUDE.md)) is discharged by the
existing helper; `tests/common/mod.rs` was not touched, so the
`0x80070005` two-conjunct check
([constraints §8](../requirements/constraints.md)) is intact.

No fixture changes scale. Each normalises to 96 DPI at a 360x240 physical
client — below the 480x320 ceiling M4-Phase 1 T8 settled on
([constraints §10](../requirements/constraints.md)) — and asserts both the
realised extent and the committed scale rather than assuming the
developer's monitor (Phase 1 F-47). Every click coordinate is derived from
`__arranged_rect_for_test()` and multiplied by the factor the runtime
**committed**, and every geometric relationship a fixture depends on is
asserted before the point is used.

**What these fixtures cannot show, stated rather than implied.** They run
at scale 1, so they do not re-exercise the pointer conversion T2's
non-unit-scale fixture owns; traversal is geometry-independent, and the
click legs inherit a conversion that fixture still pins. They read the
focus record and a boolean flag, so the indicator's *colour* is invisible
to them — W8 measures exactly that boundary, and the GUI control at #7 is
what covers the painted side, at one scale, on one widget. And the
`LRESULT` half of F5's fallthrough leg is a measured coincidence rather
than evidence: `DefWindowProcW` returns 0 for `WM_KEYDOWN` for every
candidate key tried, so what discriminates consumption there is the host
key slot, not the return value.

## T6 — DSL: `focus-group`, `modal-scope`, and `dismiss`

### Start gate (recorded 2026-08-07, before any source edit)

Read before selecting:
[AGENTS.md](../../../../AGENTS.md),
[implementation-gates.md](../../../procedures/implementation-gates.md),
[plan.md](./plan.md) §T6 / §T7 / §T8 / §T13 and §Cross-task obligations,
[preamble.md](./preamble.md) (§The sequencing thesis, §What "green" is
worth, §Review lanes),
[DD-M4-P2-005](../decisions/dd-m4-p2-005-dsl-handler-surface.md) in full,
[DD-M4-P2-003](../decisions/dd-m4-p2-003-focus-model-and-traversal.md)
§Eligibility F3 and §Group traversal,
[DD-M4-P2-004](../decisions/dd-m4-p2-004-modal-focus-scope.md)
§Recommendation,
[constraints.md](../requirements/constraints.md) §2 / §8 / §9,
the Moment-1 normative text
([dsl_spec.md §4.19](../../../../docs/dsl_spec.md) in full, §2.1 / §2.2 /
§3 grammar, §8.5 / §8.6 / §8.8 IR grammar;
[architecture.md §13.3 / §13.4 / §13.5](../../../../docs/architecture.md)),
the T5 close gate and the
[T5 retrospective](../retrospectives/t5.md), and the landed source
(`wasamoc/src/lexer.rs` ident rule, `parser.rs` member dispatch,
`check.rs` `check_members_inner` / `check_grid` / `check_cell` /
`check_zstack_unknown_attr` / `check_scrollview_unknown_attr` /
`check_togglebutton_property_name` / `check_host_property_bind`,
`lower.rs::lower_node_with_loop`, `emit.rs::emit_prop`;
`wasamo-runtime/src/ir_loader.rs` validate / `construct_widget` /
`build_node`, `widget.rs::focus_role` and the ten `WidgetNode`
constructors, `focus.rs::FocusProjection`, `focus_spike.rs`,
`focus_core.rs::FocusRole` / `tab_stops`).

#### Normative statements that already answer this task's behaviour

Recorded per
[DD-V-031](../../../cross-milestone/decisions/dd-v-031-normative-answers-at-start-gate.md).
The phase synchronised its normative text at Moment 1, so these are
**answers**, not open questions.

| Question | Document | What it fixes |
|---|---|---|
| How the two annotations are spelled | [dsl_spec.md §4.19](../../../../docs/dsl_spec.md) §`focus-group` / §`modal-scope` / §Attribute admission | `focus-group: true` and `modal-scope: true`, `bool`, default `false`, "admitted on **any container**" |
| Whether they may be bound | §4.19 §Attribute admission | "Both are **constant-only**: the value must be a `true` / `false` literal, and a binding-expression RHS is rejected — the same rule `Box.fill` and the `WrapPanel` attributes carry" |
| Whether they change layout | §4.19 §Attribute admission | "Neither attribute changes layout: an annotated container measures and arranges exactly as an unannotated one" |
| Where `dismiss` may be written | §4.19 §`dismiss` / §Attribute admission | "admitted **only on a container that carries `modal-scope: true`**. Written anywhere else it could never be raised, so it is rejected at `wasamoc check` rather than silently never firing" |
| What `dismiss` *is* | §4.19 §`dismiss`, [architecture.md §13.5](../../../../docs/architecture.md) | A request addressed to the innermost scope; "the runtime does not act on the request". Esc is its only source — and the Esc-to-request conversion is T7's, not this task's |
| What the group annotation means | §4.19 §`focus-group` | One Tab stop, arrows within, per-group memory — all **behaviour**, which [plan.md](./plan.md) §T7 owns |
| What the scope annotation means | §4.19 §`modal-scope`, [architecture.md §13.4](../../../../docs/architecture.md) | "Being there is being open" — presence is the entry, and entry runs "on the structural seam that materialises the subtree", which is T7's |
| Whether the role derivation is the extension point | [DD-M4-P2-003](../decisions/dd-m4-p2-003-focus-model-and-traversal.md) F3, and the T5 close gate re-audit | "the derivation is the extension point"; T5 recorded that T6's attributes "reach traversal by widening that function rather than by adding a second source of roles" — so `WidgetNode::focus_role` is the one place the annotation lands |
| Whether the IR needs a new carrier | §4.19 (no new value type), [dsl_spec.md §8.6](../../../../docs/dsl_spec.md) | `property_set ::= "prop" IDENT "=" literal` already has a `BOOL` alternative; `handler ::= "on" IDENT "{" expr "}"` already takes any signal name. No new token, `IrType`, `IrLiteral` or `PropertyValue` |

**One divergence is recorded rather than resolved here.** §4.19's
attribute table says "any container" and lists no exception, but
`check_grid`'s M3-Phase 5 arm rejects **every** signal handler on a
`Grid` (fact 4 below). This task admits `dismiss` on a `Grid` that
carries `modal-scope: true` so the normative sentence holds uniformly,
and leaves the rest of Grid's blanket handler rejection to
[plan.md](./plan.md) §T8, which owns widening `clicked`. The divergence
between §T8's premise and the landed checker is fact 4.

#### Scope re-decided against the code — five facts measured first

Throwaway probes (`wasamoc check` / `wasamoc build` over a constructed
`.ui` exercising the widget kinds), run before the approach was chosen.
The probe `.ui` and its emitted IR are not retained; the results are.

1. **The grammar, the lexer, `lower` and `emit` need no change at all.**
   `focus-group` and `modal-scope` already lex as one `Ident`
   (§2.2's `[A-Za-z_][A-Za-z0-9_]*(?:-[A-Za-z][A-Za-z0-9_]*)*`, the rule
   `item-cross-size` already uses), parse as `property_bind`, lower
   through the generic `Member::PropertyBind` arm to
   `IrProp { value: IrLiteral::Bool(_) }`, and emit as
   `prop focus-group = true`. `dismiss => { ... }` parses as
   `signal_handler` and emits as `on dismiss { ... }`. **Measured**: the
   probe built to valid IR text containing both lines with the compiler
   untouched. The plan's T6 heading ("Grammar, checker, IR, loader")
   over-predicts two of its four; the work is **checker + loader + role
   derivation + tests**.
2. **Nothing is rejected today that should be, and nothing accepted
   today stays accepted.** On `VStack` / `HStack` / `Box` / `WrapPanel` —
   and on `Text` and `Button` — `focus-group: true` is accepted with
   **zero diagnostics** and lowered into the IR, where the loader
   ignores it. `focus-group: <bool state>` is accepted and lowered to a
   **binding**. `modal-scope: 1` is accepted and lowered as an `Int`
   prop. `dismiss => { ... }` is accepted on any node, including at
   component level and on a container with no `modal-scope`. This is the
   silent-drop class T3's CF-2 recorded for `Button.enabled`, and it is
   what the three rejects exist to close.
3. **Three per-kind attribute gates already reject the new names, and
   each is a call site this task must pass through.** Measured
   diagnostics: "`modal-scope` is not a recognised ScrollView attribute"
   (`check_scrollview_unknown_attr`), "unknown ZStack attribute
   `focus-group`" (`check_zstack_unknown_attr`), "unknown Grid attribute
   `modal-scope`" (`check_grid`). A fourth,
   `check_togglebutton_property_name`, rejects them on `ToggleButton` —
   the **right** answer for a non-container, with the wrong diagnostic.
   These four are the semantic-migration audit's rows on the checker
   side, and Rust enumerates none of them.
4. **`plan.md` §T8's premise "`clicked` needs no checker widening —
   `check` has no per-kind signal admission rule" is false for one
   kind.** `check_grid`'s `SignalHandler` arm emits "`Grid` takes no
   signal handlers" — measured. `Box`, `ZStack`, `VStack` and the rest
   do accept handlers, so T3's CF-3 holds everywhere except `Grid`.
   Recorded as a finding for T8 rather than fixed here beyond the
   `dismiss` case this task owns.
5. **`WidgetNode` has ten struct-literal construction sites and no
   `Default` / `..` update syntax**, so adding a per-node annotation
   field is **compiler-forcing** — the one part of this task's migration
   Rust enumerates. `WidgetNode::focus_role` has two callers
   (`focus::FocusProjection::project`, `focus_spike::walk`); it is the
   single role derivation T5 left as the extension point.

**What T6 therefore is.** Not "add grammar and IR", but four things:

- **Checker admission** — the two attributes on the seven container
  kinds, constant-only `true` / `false`, routed **ahead of** the four
  per-kind gates of fact 3 so a `ZStack` / `ScrollView` / `Grid` can
  carry them and a `Text` / `Button` / `ToggleButton` / `Rectangle` /
  `Cell` cannot.
- **`dismiss` admission** — a checker rule over the enclosing body's
  sibling members: the handler is admitted only where
  `modal-scope: true` is written on the same node.
- **A loader gate for the same rules** (recorded deviation from the
  plan's checker-only list, argued below).
- **The node's focus role** — a per-node annotation the loader writes
  and `WidgetNode::focus_role` reads, so `Group` and `ModalScope` become
  reachable roles for the first time.

Three boundaries are drawn deliberately:

- **No behaviour is built.** Group traversal, per-group memory, scope
  entry / exit / restore and the Esc-to-`dismiss` conversion are all
  T7's. This task's tests assert the **role that reaches the node**,
  never what traversal then does with it — asserting traversal here
  would be asserting T7's unbuilt behaviour.
- **The intermediate state is named rather than discovered.** Between
  this task and T7 a `modal-scope: true` subtree projects as
  `FocusRole::ModalScope` with no entry seam, so `focus_core::tab_stops`
  skips it: a **present but un-entered scope**, which is exactly the
  state [DD-M4-P2-004](../decisions/dd-m4-p2-004-modal-focus-scope.md)
  says must not be reachable ("nothing present is un-entered"). It is
  unreachable from any shipped `.ui` — no example file carries the
  attribute, and T10 is what adds one, after T7 — but it is real for a
  test or a hand-written `.ui` in between. Recorded as a carry-forward
  to T7 rather than left for T7 to find.
- **The both-at-once case is expressible and single-valued.**
  DD-M4-P2-005 records "a container that is a group and a scope" as
  expressible under A1 and untested in M4; `focus_core::FocusRole` is
  one-of-six, so the projection cannot carry both. The derivation gives
  `modal-scope` precedence, states why at the site, tests it, and
  carries the composite question to T7.

**Why the loader gate is added** (deviation from
[plan.md](./plan.md) §T6, which names three checker rejects). The loader
must read these props anyway to write the annotation, so the gate is
adjacent rather than additional; `wasamo_load_ui` admits memory IR that
never passed through `wasamoc` (the two-gate shape §4.9 / §4.12 / §4.16
already use); and without it the failure mode is *silent* — an
annotation on a `Button`, or `prop modal-scope = 1`, is dropped with no
diagnostic, which is the defect class this phase already has on its
books twice (T3's CF-2, and fact 2 above). The plan's §T6 item is
revised to record what was actually built
([AGENTS.md §Commit rules](../../../../AGENTS.md#commit-rules)).

#### Trap selection

| # | Trap | Applies | Reason |
|---|---|---|---|
| 1 | Semantic-migration miss | **yes** | Two migrations, and Rust enumerates one. `WidgetNode` gains a field — compiler-forcing across ten construction sites (fact 5). The **checker's per-kind attribute dispatch** gains two admitted names, and the compiler enumerates nothing of it: four existing gates (fact 3) would each swallow a new name plausibly, and the generic `else` arm accepts it with no diagnostic at all. The audit table therefore covers every per-kind attribute gate, every signal-handler gate, every reader of the new field, and the IR-side `prop` / `on` paths |
| 2 | Missed side effects | **yes** | A per-node field on `WidgetNode` is read by a derivation with two callers, one of them the spike's override projection — a widened `focus_role` changes what `focus_spike::project` derives *before* overrides apply, so `tests/focus_mechanism_fixture.rs` is in the blast radius. Also enumerated before writing: whether the annotation must survive `set_root` / subtree re-materialisation (it is built-time state on the node, not window state); whether it can reach layout (it must not — §4.19 "neither attribute changes layout"); and whether it can reach the Composition geometry writes (it must not — the six-call enumeration DD-M4-P2-003 requires of any task touching focus presentation) |
| 3 | Parallel/derived data drift | **no** | The annotation has exactly one writer (the loader, at construction) and no derived copy: `focus_role` computes from it on demand rather than caching a role. Nothing is written twice. The trap-3 pair this phase carries — the focus record and the painted flag — is untouched, and the group **memory** whose single-writer discipline DD-M4-P2-003 requires is written by `focus_core::FocusState::set_focus`, which this task does not call |
| 4 | Untested authored branch | **yes** | The task is almost entirely branches. Accept side: each attribute on each admitting kind, both attributes together, `dismiss` beside `modal-scope: true`. Reject side: a binding RHS; a non-bool literal RHS; the attribute on each non-admitting kind and at component level; `dismiss` without `modal-scope`; `dismiss` beside `modal-scope: false`; and the same rules on the loader side. Every one ships with a test that fires it directly, named in the close artifact |
| 5 | Carry-forward underweighted | **yes** | Three named already: the present-but-un-entered intermediate (T7), the both-at-once role collapse (T7), and fact 4's Grid handler rejection (T8). Per the T3 retrospective's corrective, anything this task requires *as evidence of a later task* is built and run here, or recorded as a finding with an owner |
| 6 | Symptom taken at face value | **conditional** | No deterministic failure is in hand at the start gate. Armed: any failure during implementation gets a minimal repro and a root cause, not a re-roll |
| 7 | Weak GUI evidence | **no** | The deliverable is a compiler surface and inert node state. Nothing this task lands is painted: the annotation adds no `Visual`, no brush and no geometry write, and §4.19 fixes that it changes no layout. A captured frame could not distinguish an annotated container from an unannotated one, which is precisely why this is not a GUI-evidence task. The first frame that can see a group or a scope is T12's, after T7 gives them behaviour |

```
- [x] #1 semantic migration   - [x] #2 side effects   - [ ] #3 parallel data   - [x] #4 branch tests
- [x] #5 carry-forward        - [~] #6 root cause     - [ ] #7 GUI positive control
```

#### Review lane

**Branch/test-focused review**, as
[preamble.md §Review lanes](./preamble.md) predicts and as the change
confirms: checker and loader additions whose deliverable *is* the reject
branches, plus a compiler-forced field addition. It is **not** a runtime
structural change — no new store with a lifetime, no message-arm return
path, no second writer — and it carries no GUI-render evidence (trap 7).
The one judgment that would raise the lane if it were wrong is whether
widening `focus_role` counts as structural; it does not, because the
function's shape, its callers and its single-derivation property are
unchanged and only its *value set* widens, which fact 5 makes
compiler-visible at every construction site and which the fixture re-run
measures.

#### The T4 and T5 correctives, applied

Later start gates inherit one line each from T4 and T5:

- *Which negative prediction of the phase documents does this task depend
  on, and has it been measured once?* Three, and all three are measured
  (facts 1–4). The plan's "No new token, `IrType`, `IrLiteral` or
  `PropertyValue`" is measured true and stronger than written — no
  grammar, `lower` or `emit` change either. The plan's implicit premise
  that the checker is silent about these names is measured **half
  false**: four per-kind gates already reject them. And §T8's "check has
  no per-kind signal admission rule" is measured **false for `Grid`**.
- *What does this task retain across messages or frames, and is the
  identity stable?* (T5's new line.) **Nothing.** The annotation is
  per-node built-time state read on demand by a derivation that already
  runs fresh per operation; this task introduces no retained identifier
  and reads no `FocusId`. The identifier hazard T5 recorded (CF-T5-1)
  is untouched and stays T7's.

#### Planned proof obligations

Each closed at the T6 close gate:

1. The call-site audit table over every per-kind attribute gate, every
   signal-handler gate, the new field's writer and readers, and the
   IR-text `prop` / `on` paths.
2. The structural side-effect enumeration, including the spike
   projection's re-derivation and the "changes no layout, writes no
   geometry" statements with the `SetOffset` / `SetSize` count.
3. The branch table: one test per accept arm and one per reject arm, on
   both the checker and the loader side.
4. A round-trip assertion that the attributes survive `.ui` → IR text →
   loaded IR, and that the loaded node's `focus_role` is the annotated
   one.
5. Mutation witnesses, including at least one that is **not** a mutation
   of this task's own implementation (T5's close-gate line).
6. The whole task list re-read at the close gate (the re-audit
   discipline, [plan.md](./plan.md) §Cross-task obligations).

### Close gate (recorded 2026-08-07)

Landed: `wasamoc/src/check.rs` (`FOCUS_ANNOTATION_CONTAINERS`,
`FOCUS_ANNOTATION_ATTRS`, `check_focus_annotation_const_only_bind`,
`check_focus_annotation_admission`, the attribute dispatch arm ahead of
the per-kind gates, the `carries_modal_scope` predicate and the `dismiss`
rule, `check_grid`'s two relaxations, 23 unit tests);
`wasamo-runtime/src/widget.rs` (`FocusAnnotation`, the
`WidgetNode::focus_annotation` field at all ten construction sites,
`set_focus_annotation` as its sole writer, the widened `focus_role`
container arm, `__focus_role_for_test`);
`wasamo-runtime/src/ir_loader.rs` (`FOCUS_ANNOTATION_CONTAINERS`,
`validate_focus_annotation_invariants` with its member recursion wired
into `validate`, the `validate_phase6_zstack_node_invariants`
relaxation, the annotation write in `build_node_with_loop_context`,
22 unit tests);
`wasamo-runtime/tests/focus_annotation_integration.rs` (one fixture).

**Nothing was landed in `lexer.rs`, `parser.rs`, `lower.rs`,
`emit.rs`, `wasamo-ir`, `focus_core.rs`, `focus_spike.rs`, `focus.rs`,
`window.rs` or `layout.rs`** — the start gate's fact 1 predicted the
first four and the boundary held for the rest.

#### #1 — Call-site audit table

Two migrations. Rust enumerates one of them and none of the other, so
the table covers both and says which is which.

Queries:
`rg "focus-group|modal-scope|dismiss" wasamoc/src wasamo-runtime/src`,
`rg "focus_role|focus_annotation" wasamo-runtime/src`,
`rg "arranged_rect: None" wasamo-runtime/src/widget.rs` (the
construction-site census),
`rg "unknown .* attribute|takes no signal handlers|accepts no Phase-6"
wasamoc/src wasamo-runtime/src` (the per-kind gates that could swallow a
new name),
`rg "SetOffset|SetSize" wasamo-runtime/src`.

**The compiler-forced half.** `WidgetNode` has ten struct literals and
no `Default`, no `..` update syntax and no builder, so the new field
could not be missed at construction: `widget.rs` 594 / 621 / 648 / 691 /
741 / 786 / 847 / 885 / 911 / 1085. Nothing else in the workspace
constructs a `WidgetNode`.

**The half the compiler enumerates nothing of** — every gate that
decides whether an attribute name or a signal name is admitted. These
rows are the artifact.

| Site | Classification | Reason |
|---|---|---|
| `check.rs:2182` attribute dispatch arm | **new, must run early** | Placed after the `slot.*` / `CHILD_PLACEMENT_ATTRS` dispatch and after the component-level early return, and **before** the ZStack / ScrollView / ToggleButton arms. Witness W8 measures that the ordering is load-bearing rather than incidental |
| `check.rs:2203` `check_zstack_unknown_attr` | **migrated** | Rejected `focus-group` on a ZStack before this task (start gate fact 3). Now unreachable for the two names because the arm above claims them first |
| `check.rs` ScrollView arms (`offset-y`, then the catch-all) | **migrated** | Same shape: `modal-scope` on a `ScrollView` was a measured reject before this task |
| `check.rs` `check_togglebutton_property_name` | **migrated** | Still rejects the names — correctly, `ToggleButton` is not a container — but the diagnostic is now the admission one. `focus_group_true_on_togglebutton_rejected_as_admission_not_unknown_attr` is what pins which of the two fires |
| `check.rs:1366` `check_grid` attribute arm | **migrated** | Skips the two names so a `Grid` does not also report "unknown Grid attribute"; the generic dispatch owns them. Same shape as the `slot.*` skip immediately above it |
| `check.rs:1396` `check_grid` signal arm | **migrated** | Was a blanket "`Grid` takes no signal handlers"; now lets `dismiss` through and rejects every other name. `non_dismiss_handler_on_grid_still_rejected` bounds the relaxation |
| `check.rs:2385` `Member::SignalHandler` | **new, sole `dismiss` admission** | The one place a signal name is checked against its enclosing node in the checker |
| `check.rs:2088` `carries_modal_scope` | **new, the predicate** | "A **container** that carries `modal-scope: true`", both halves — see #2 |
| `check.rs` `check_host_property_bind` | ignore-OK, unchanged | Component-level `focus-group:` keeps its existing unknown-host-attribute diagnostic; `focus_group_true_at_component_level_rejected_as_unknown_host_attr` pins that it still reaches this gate rather than the new one |
| `check.rs` `check_cell` | ignore-OK, unchanged | A `Cell` is not a container; its pre-existing unknown-attribute diagnostic fires **alongside** the new admission one, the same dual-diagnostic shape a misplaced WrapPanel attribute already produces |
| `ir_loader.rs:251` `validate` chain | **new, ordered** | Runs immediately after `validate_phase2_node_invariants` and **before** the ZStack and ToggleButton gates, so a misplaced annotation reports admission rather than a per-kind "unknown attribute" |
| `ir_loader.rs:1296` `validate_focus_annotation_invariants` | **new, the four rejects** | Kind admission, non-`Bool` literal, either name on the binding path, `dismiss` without `modal-scope = true` |
| `ir_loader.rs:1358` its member recursion | **new** | Covers `IrMember::Widget`, `If` branch bodies and `For` bodies, copying the shape every earlier `validate_phaseN_member_invariants` uses |
| `ir_loader.rs:1164` ZStack prop gate | **migrated** | Let the two names through. **This one is not cosmetic**: without it the loader refuses IR the checker accepts, and witness W6 reddens the integration fixture to prove it |
| `ir_loader.rs:1197` ZStack handler gate | **migrated** | Was `!node.handlers.is_empty()`; now `any(|h| h.signal != "dismiss")`. `zstack_clicked_handler_still_rejected_after_relaxation` bounds it |
| `ir_loader.rs` `validate_phase8_togglebutton_node_invariants` | ignore-OK, ordering only | Its "unknown ToggleButton attribute" arm is now unreachable for the two names because the new gate runs first; the loader-side ToggleButton test asserts which message comes out |
| `ir_loader.rs:3189` the annotation write | **new, sole writer** | One kind-independent site in `build_node_with_loop_context`, not per-kind arms in `construct_widget` |
| `widget.rs:1152` `set_focus_annotation` | **sole writer of the field** | `pub(crate)`; `rg` shows exactly one call site, the one above |
| `widget.rs:1188` `focus_role` | **migrated, value set widened** | Same signature, same two callers, still total over `WidgetData`; only the container arm's value set grows |
| `widget.rs:1931` `__focus_role_for_test` | **new, read-only** | Returns `&'static str` rather than exporting `FocusRole`, the shape `__button_state_for_test` already uses |
| `focus_spike.rs::walk` | **second reader, re-derived** | Calls `focus_role()` and then applies its override map on top. Its fixture builds trees programmatically, so no node carries an annotation and the derived role is unchanged — measured by `focus_mechanism_fixture` staying green (4 tests) |
| `focus.rs::FocusProjection::project` | **first reader, unchanged** | Consumes whatever `focus_role` returns; no edit needed, which is the point of T5 leaving the derivation as the extension point |
| `abi.rs` | ignore-OK, unchanged | No `extern "C"` function added or altered; the cross-task "no new ABI function" obligation holds |

**What no existing test pinned.** `git grep "focus-group|modal-scope|
dismiss"` over `e3ff83a` (the pre-T6 tree) returns **exactly one hit in
the whole workspace** — a doc comment in `focus.rs` naming the two
annotations as the things arrows and `Escape` are conditioned on — and
**no test**. Witness W1a measures the consequence directly: with the
annotation read deleted from `focus_role`, exactly **one of 45** test
sections goes red, and it is the one this task added.

#### #2 — Structural side-effect enumeration

| Derived effect | Disposition |
|---|---|
| **The node's focus role** | New, and the only intended effect. Computed on demand by `focus_role` from the field; no role is cached, so there is no derived copy to keep in step (which is why trap #3 was recorded non-applicable at the start gate and stayed so) |
| **Layout** | Untouched, and required to be: dsl_spec §4.19 "neither attribute changes layout". The field is not read by `build_layout_tree`, `measure`, `arrange` or `sync_visuals` — `rg "focus_annotation" wasamo-runtime/src` returns the struct, the field, the writer and `focus_role` only |
| **Composition geometry** | Unchanged — see #3 |
| **`Visual` creation** | None. The annotation creates no `Visual` and no brush; it is not painted at all, which is why trap #7 is non-applicable and why no capture could witness this task |
| **The spike's override projection** | Re-derives through the same `focus_role`. Enumerated at the start gate as the blast radius; `focus_mechanism_fixture` (4) and `focus_traversal_integration` (6) both stay green |
| **`window::set_root` / subtree re-materialisation** | Not a concern by construction: the annotation is **built-time state on the node**, written by the loader from the IR the node was built from, so a re-materialised subtree gets it again from the same IR. Unlike T4's `HoverState` and T5's `WindowFocus` there is no window-level record to invalidate — this task retains no identifier across messages (the T5 start-gate line) |
| **The binding path** | Deliberately closed at both gates. Constant-only means a `bind focus-group = …` is not a shape the runtime has to support, so `resolve_prop_key` gains no entry and no `PropertyValue` variant exists |
| **`ButtonData.label_size`'s three-point write** ([constraints §4](../requirements/constraints.md)) | Not touched |
| **`focus_core`'s group memory** | Not touched. Its single-writer discipline lives inside `FocusState::set_focus`, which this task does not call. A `Group` role now exists, but nothing moves focus into one until T7 |
| **Traversal, for an unannotated tree** | Unchanged: with both flags `false` the container arm returns `FocusRole::Container`, which is exactly the pre-T6 value. Every existing focus test stays green without edit, which is the measurement |
| **Traversal, for an *annotated* tree** | Changed, and this is the one behavioural consequence — see the carry-forward CF-T6-1. A `modal-scope` subtree is skipped by `tab_stops` until T7 enters it |

#### #3 — Every `SetOffset` / `SetSize` in the runtime, with its pass

Carried from T5 because the field is per-node presentation-adjacent
state and DD-M4-P2-003 requires the enumeration of any task that could
add a geometry write.

Query: `rg "SetOffset|SetSize" wasamo-runtime/src`.

| Site | Pass | What it writes |
|---|---|---|
| `widget.rs` node Visual offset / size (2 calls) | `sync_visuals` | The node's own Visual |
| `widget.rs` Button-family label (2 calls) | `sync_visuals` | The label Visual |
| `widget.rs` `ScrollView` intermediate (2 calls) | `sync_visuals` | The content Visual |
| `dip_scale.rs` (2 mentions) | — | Doc comments naming the operations, not calls |

**Six calls, all inside `sync_visuals`, unchanged from T1 / T2 / T3 /
T4 / T5.** The annotation adds none.

#### #4 — Branch tests, each fired directly

Every branch authored by this task, with the test that fires it. 23 new
tests in `wasamoc`, 22 in `wasamo-runtime`'s loader, 1 integration
fixture.

**Checker — accept arms**

| Authored arm | Test that fires it |
|---|---|
| `focus-group: true` on each of the seven containers | `focus_group_true_accepted_on_every_admitting_container` (a table over all seven, each built validly) |
| `modal-scope: true` on the same seven | `modal_scope_true_accepted_on_every_admitting_container` |
| The `false` literal is also a constant | `focus_group_false_accepted`, `modal_scope_false_accepted` |
| Both attributes on one container | `focus_group_and_modal_scope_together_on_one_container_accepted` |
| `dismiss` beside `modal-scope: true` | `dismiss_handler_accepted_beside_modal_scope_true` |
| `dismiss` on a `Grid` (the relaxed arm) | `dismiss_handler_accepted_on_grid_carrying_modal_scope` |
| The gate that used to swallow the name — ZStack | `focus_group_true_on_zstack_produces_no_diagnostic` (asserts `diagnostics.is_empty()`, not merely no error, so a surviving *warning* would fail it) |
| …and ScrollView | `modal_scope_true_on_scrollview_produces_no_diagnostic` |

**Checker — reject arms**

| Authored arm | Test that fires it |
|---|---|
| Constant-only, state-ident RHS | `focus_group_state_ident_rejected` |
| Constant-only, non-bool literal RHS | `modal_scope_int_literal_rejected` |
| Admission — `Text` | `focus_group_true_on_text_rejected` |
| Admission — `Button` | `focus_group_true_on_button_rejected` |
| Admission — `ToggleButton`, **and which of two diagnostics fires** | `focus_group_true_on_togglebutton_rejected_as_admission_not_unknown_attr` |
| Admission — `Rectangle` | `focus_group_true_on_rectangle_rejected` |
| Admission — `Cell` | `focus_group_true_inside_cell_rejected` |
| Component level stays the host gate's | `focus_group_true_at_component_level_rejected_as_unknown_host_attr` |
| `dismiss` with no `modal-scope` | `dismiss_handler_without_modal_scope_sibling_rejected` |
| `dismiss` beside `modal-scope: false` | `dismiss_handler_beside_modal_scope_false_rejected` |
| `dismiss` at component level | `dismiss_handler_at_component_level_rejected` |
| The predicate's **kind** half | `dismiss_beside_a_modal_scope_on_a_non_container_is_still_rejected` (two diagnostics, both asserted) |
| The predicate's **position** half | `dismiss_beside_a_component_level_modal_scope_is_still_rejected` |
| The Grid relaxation is narrow | `non_dismiss_handler_on_grid_still_rejected` |

**Loader — accept arms**

`focus_group_true_accepted_on_every_admitting_container`,
`modal_scope_true_accepted_on_every_admitting_container`,
`focus_group_false_accepted`, `modal_scope_false_accepted`,
`focus_group_and_modal_scope_together_on_one_container_accepted`,
`dismiss_handler_accepted_beside_modal_scope_true`,
`dismiss_handler_accepted_on_zstack_carrying_modal_scope`,
`dismiss_handler_accepted_on_grid_carrying_modal_scope`.

**Loader — reject arms**

| Authored arm | Test that fires it |
|---|---|
| Kind admission | `focus_group_true_on_text_rejected`, `_on_button_rejected`, `_on_rectangle_rejected`, `modal_scope_true_on_text_rejected` |
| …and which diagnostic wins over the ToggleButton gate | `focus_group_true_on_togglebutton_rejected_as_admission_not_unknown_attr` |
| Non-`Bool` literal | `focus_group_non_bool_literal_rejected`, `modal_scope_non_bool_literal_rejected` |
| The binding path | `focus_group_binding_rejected`, `modal_scope_binding_rejected` |
| `dismiss` without / with `false` / on a non-container | `dismiss_handler_without_modal_scope_prop_rejected`, `dismiss_handler_beside_modal_scope_false_rejected`, `dismiss_handler_on_non_container_rejected` |
| The ZStack relaxation stayed narrow — props | `zstack_spacing_prop_still_rejected_after_relaxation` |
| …and handlers | `zstack_clicked_handler_still_rejected_after_relaxation` |

**The role derivation**

| Authored arm | Test that fires it |
|---|---|
| `focus-group: true` → `Group` | `authored_focus_annotation_reaches_the_loaded_node_as_its_focus_role` |
| `modal-scope: true` → `ModalScope` | the same fixture |
| unannotated container → `Container` | the same fixture (the control leg) |
| Button-family arm unchanged under an annotated ancestor | the same fixture |
| both-at-once → `ModalScope` (the documented precedence) | the same fixture |

[DD-V-029](../../../cross-milestone/decisions/dd-v-029-pure-logic-red-test-obligation.md)'s
**named** obligation is not triggered: this task authors no rounding, no
unit conversion and no boundary condition. The witnesses below are the
trap-#4 artifact instead.

#### #5 — Mutation witnesses

Eight witnesses. Every one was applied with an edit, **read back from
the file** to confirm it was present before the run, run, then reverted
with the revert confirmed by re-reading — and, for `check.rs`, by
`git diff --stat` returning empty against the committed state (the T2
corrective, carried by T3, T4 and T5). No failure was re-rolled: the
suite went red only where a mutation was deliberately introduced.

**Two of the eight are not mutations of this task's own implementation**
(the T5 close-gate line): W1a and W1b each *restore the pre-T6
behaviour* rather than break the new code, so they answer "do these
tests catch the absence of the feature", not merely "do they watch their
own code".

| Witness | Mutation | Went red | Reading |
|---|---|---|---|
| **W1a — the role read restored** (restoring) | `focus_role`'s container arm returns `FocusRole::Container` unconditionally | `focus_annotation_integration` — and **nothing else in the workspace**: 1 of 45 sections | The measurement behind "no existing test pinned this". The whole workspace has exactly one test that can see the annotation reach the node, and it is this task's |
| **W1b — the checker admission restored** (restoring) | `FOCUS_ANNOTATION_ATTRS` emptied, so the dispatch never fires and pre-T6 checker behaviour returns | 13 checker tests **and** the integration fixture (2 of 45 sections) | The integration fixture is the interesting one: it drives the real compiler, so restoring the checker makes the `.ui` fail `check` before the runtime is reached. That is the compiler-to-runtime path being end-to-end rather than two halves asserted apart |
| **W2 — precedence flipped** | `focus_role` prefers `Group` over `ModalScope` | the integration fixture alone, on the both-at-once leg | The precedence is asserted, not incidental |
| **W3 — loader kind admission deleted** | the `FOCUS_ANNOTATION_CONTAINERS` test short-circuited | 5 loader tests | Each non-container kind is fired by its own test, not by one representative |
| **W4 — loader constant-only deleted** | the `IrLiteral::Bool` test short-circuited | `focus_group_non_bool_literal_rejected`, `modal_scope_non_bool_literal_rejected` | The runtime half of the constant-only rule is separately pinned from the checker half |
| **W5 — loader `dismiss` rule deleted** | the `dismiss` arm short-circuited | 3 loader tests | Absent / `false` / non-container are three inputs to one branch, and all three are fired |
| **W6 — the ZStack relaxation reverted** | the two names no longer skip the ZStack prop gate | 5 loader tests **and** the integration fixture | The decisive one for the two-gate question: without the relaxation the loader refuses a `.ui` the compiler accepted, and the fixture is what notices, because it is the only test that runs both gates on one input |
| **W8 — the checker dispatch reordered** | the focus-annotation arm moved *after* the ZStack / ScrollView / ToggleButton gates | 5 checker tests, including both no-diagnostic controls and the ToggleButton which-diagnostic test | The ordering claim in the audit table is falsifiable rather than asserted |

W7 (the annotation write removed at the build site, leaving `focus_role`
intact) reddened the integration fixture alone and produced a
`method set_focus_annotation is never used` warning — recorded here
because the warning is a second, independent signal that the write site
is the only one.

#### #6 — Deterministic-failure disposition

**None arose.** Trap 6 was selected as *armed* rather than applying, and
it did not fire: no test failed except where a witness was deliberately
in place, and every witness failure disappeared on the confirmed revert.
There is no rerun history to record because there was no unexplained
failure to rerun.

#### #7 — Carry-forward

| Constraint | Evidence | Placement | Re-trigger criterion |
|---|---|---|---|
| **CF-T6-1 — between this task and T7, a *present* `modal-scope` subtree is *un-entered*.** `focus_role` now returns `FocusRole::ModalScope`, and `focus_core::tab_stops` skips a scope the state has not entered; entry is the materialisation seam T7 owns. That is precisely the state [DD-M4-P2-004](../decisions/dd-m4-p2-004-modal-focus-scope.md) argues must not be reachable ("nothing present is un-entered") | The derivation itself, plus `focus_core::tab_stops`'s `FocusRole::ModalScope if !state.is_entered(id) => return` arm | `carry-forward` → this ledger | **T7**, which adds the entry seam and closes it. Bounded meanwhile: no shipped `.ui` carries the attribute — T10 is what adds one, after T7 — so the state is reachable only from a test or a hand-written `.ui` in between. Named at the start gate rather than found here |
| **CF-T6-2 — a container carrying both annotations collapses to one role.** `focus_core::FocusRole` is one-of-six, so `modal-scope` takes precedence and the `focus-group` half has no effect. It is no longer *silent*: `wasamoc` warns, and the shape stays accepted | The precedence branch in `focus_role` and witness W2; the integration fixture's both-at-once leg pins the chosen answer; the checker warning pins that the author is told | `finding` → the [candidate pool](../../../candidate-pool.md), owner-settled 2026-08-07 | **No M4-Phase 2 task owns it.** T7's plan covers group traversal, scope entry / exit, the seam, dismissal and the spike retirement — a combined role is in none of them, so assigning it to T7 would have been an assignment nothing executes. It is a **surface** question rather than a projection one: if the combination is not wanted, §4.19's two booleans should become one enumerated attribute (DD-M4-P2-005's A3, rejected only because it made the combination inexpressible); if it is wanted, the traversal core needs a combined role. Re-trigger: the first app that wants a container to be a group **and** a scope, or the M6 freeze reading §4.19 |
| **CF-T6-3 — a per-kind signal admission rule *does* exist, in two places, and `plan.md` §T8's premise is false for them.** `check_grid` rejected every signal handler on a `Grid`, and `validate_phase6_zstack_node_invariants` rejects every handler on a `ZStack`. This task relaxed both **only** for `dismiss` | Measured at the start gate (fact 4) for the checker and while wiring the loader gate for the runtime; `non_dismiss_handler_on_grid_still_rejected` and `zstack_clicked_handler_still_rejected_after_relaxation` pin the current bound | `finding` → **T8** | **T8**, which widens `clicked` to any widget. Its plan text says "`clicked` needs no checker widening — `check` has no per-kind signal admission rule"; that is true for `Box` and the stacks and **false** for `Grid` and `ZStack`, on the checker and loader side respectively. T8 must either widen both gates or record the narrowing |
| **CF-T6-4 — the two `FOCUS_ANNOTATION_CONTAINERS` lists are duplicated across crates with no mechanical tie.** `wasamoc::check` and `wasamo_runtime::ir_loader` each hold the seven-name list; they agree today and nothing makes them stay in agreement | The two consts, and the fact that the integration fixture is the only test that runs an input through both | `carry-forward` → this ledger | **Any task that adds a container widget kind** — M4-Phase 4's `Image` is the nearest candidate, M5's widget set the larger one. This is the same shape as the pre-existing `resolve_prop_key` / `widget_prop_type` duplication the codebase already documents as deliberate (the compiler stays self-contained), so it is recorded as inherited rather than introduced |

#### Re-decided at close

The start gate selected traps 1, 2, 4 and 5, armed 6, and recorded 3 and
7 non-applicable with reasons. **The selection survived unchanged**, and
each non-applicable call is confirmed by what was built: no derived copy
of the role was cached (3), and nothing this task lands is painted (7).
Trap 6 stayed armed and did not fire.

Two things were built that the gate did not name explicitly: the
**loader-side relaxation of the ZStack gate** (the gate predicted a
loader *addition*, not a loader *relaxation* — the ZStack gate's
existence was discovered while wiring the new validate), and the second
restoring witness W1b. Neither changes the review lane. The lane stays
**branch/test-focused review** as predicted: the ZStack relaxation is a
reject-branch narrowing, not a structural change, and the compiler-forced
field addition is enumerated at all ten sites.

#### Re-audit of the whole task list

Per [plan.md](./plan.md) §Cross-task obligations, the full list was
re-read at this close gate rather than only T6's item.

- **T7** — inherits CF-T6-1 directly, and gains what it was
  waiting for: `focus_role` now yields `Group` and `ModalScope` from an
  authored source, so T7's projection work is entry / exit / memory
  rather than role plumbing. It does **not** inherit CF-T6-2: a combined
  group-and-scope role appears in none of T7's plan bullets, so that row
  goes to the candidate pool rather than to a task that would not execute
  it. Its plan bullet "the core's un-entered state
  has no production constructor" is now **false** — this task created
  exactly that constructor — so T7's reconciliation is a real branch with
  a real input rather than a hypothetical. Its `focus_spike` retirement is
  unaffected: the override map still overrides whatever `focus_role`
  derives.
- **T8** — inherits CF-T6-3. Its `clicked`-widening premise needs
  correcting against two measured gates. Its `key-down("<key>")` grammar
  work is untouched by this task; the start gate's fact 1 (no grammar
  change was needed for `focus-group` / `modal-scope` / `dismiss`) does
  **not** generalise to `key-down`, which needs the phase's one new
  production because a signal name carrying an argument is not an `Ident`.
- **T9** — unaffected. Per-item handlers inside `for` are the phase's
  only new IR content; this task added none. The `dismiss` admission
  predicate reads a node's own member list, so a handler inside a `for`
  body is checked against its own siblings, not the loop's.
- **T10** — first `.ui` that will carry the attributes. It must land
  **after** T7 for CF-T6-1's reason: annotating the gallery lightbox
  before the entry seam exists would make its buttons keyboard-
  unreachable. The plan's ordering already has this right; recorded so
  the reason is visible rather than incidental.
- **T11** — unaffected; touch inherits nothing from the annotation.
- **T12** — unaffected by this task's landing, but its control C
  (containment and occlusion) is the first *frame* that can see a scope,
  and it can only see one after T7.
- **T13** — gains two re-verification items. §4.19's attribute table
  ("admitted on any container") and its signal-admission table are now
  implemented on both gates and should be checked against the landed
  container list; and §4.19's `dismiss` admission sentence should be
  checked against the two-gate implementation rather than the checker
  alone. CF-T6-3's narrowing is T8's to close, not T13's, but if T8
  leaves it open T13 inherits a divergence.
- **Cross-task obligation "no new ABI function"** — held. No `extern "C"`
  function was added or changed; the one new public symbol is the
  `__focus_role_for_test` seam.
- **Cross-task obligation "every task that measures something re-reads
  the whole task list"** — discharged here.

#### Verification means

Run against the **final branch state**, after the review remediation
landed ([retrospectives.md](../../../procedures/retrospectives.md) item
3: a verification recorded before a later remediation commit is older
than the branch it claims to describe).

`cargo clean` (9,540 files / 2.6 GiB removed), then
`cargo build --release --workspace` 1m26s success,
`cargo build --workspace` 1m07s success,
`cargo test --workspace --no-fail-fast` **45 binaries/sections, 1,107
passed, 0 failed, 0 ignored**. T5's baseline was 1,051; the 56 added are
`check.rs`'s 27 unit tests, `ir_loader.rs`'s 28, and the one integration
fixture. `cargo fmt --all -- --check` zero exit and `git diff --check`
clean against the final state.

**The integration fixture ran rather than skipped**, verified by running
it with `--nocapture` and confirming the shared guard's
`skipping …: runtime compositor unavailable` line does **not** appear.
`tests/common/mod.rs` was not touched, so the `0x80070005` two-conjunct
check ([constraints §8](../requirements/constraints.md)) is intact and
the standing obligation to verify a newly authored guard does not apply —
no guard was authored.

**What this task's evidence cannot show, stated rather than implied.**
The fixture asserts the role that reaches the node; it does not assert
what traversal then does with that role, because the behaviour does not
exist until T7. It runs at one scale and touches no geometry, so it
inherits nothing from and adds nothing to the pointer-conversion
evidence T2 owns. And no captured frame exists for this task, by
construction: the annotation is not painted, so a frame could not
distinguish an annotated container from an unannotated one.

#### Independent review and its remediation (recorded 2026-08-07)

The review lane ([implementation-gates.md §4](../../../procedures/implementation-gates.md))
was executed as an **independent branch/test-focused review** by a second
agent that did not write the code, against `c839c17` / `f1314c9` /
`53c95f1`. It built its own branch-coverage table from the diff rather
than from the close gate's, probed ~15 constructed `.ui` files through
`wasamoc check` / `build` and ~18 hand-built IR-text cases through
`parse_ir` — including memory-IR shapes the checker can never emit — and
re-ran the suite and the integration fixture with `--nocapture`.

It confirmed the two ordering claims by tracing the source (the checker's
dispatch sits after the `slot.*` / placement routing and before the
ZStack / ScrollView / Box / ToggleButton / WrapPanel arms; the loader's
gate runs before the ZStack and ToggleButton gates), confirmed that
ScrollView and Grid have **no** loader-side unknown-attribute catch-all
so the narrower doc claim at that site is accurate, confirmed the
`unreachable!()` arm in `__focus_role_for_test` is genuinely unreachable,
and found **no disagreement between the two gates on any shape this task
introduces** — including the `if`-wrapped canonical example.

Four findings. Two changed the code, one sharpened a carry-forward, one
was accepted as recorded.

**F1 — the "no behaviour is added" claim needed narrowing, and the code
site needed the caveat.** True of layout, paint and geometry; **not**
true of routing. `focus_role`'s two callers are production message paths
(`focus::traverse_on_key` from `WM_KEYDOWN`, `focus::focus_on_click` from
`WM_LBUTTONUP`), so widening its value set changes what both can reach
the moment an author writes the attribute. The close gate already
carried the `modal-scope` half as CF-T6-1; the review added the
`focus-group` half, which was **not** recorded. Verified at the lead's
own reading, with one correction to the review's account:

- `FocusTree::tab` **does** call `resolve_stop` (`focus_core.rs` 321 /
  350), so **Tab into a group already lands on the group's first or
  remembered member** — that half is correct from this task onward.
- `focus::focus_on_click` builds its landing from `tab_stops` +
  `nearest_focusable` and **never** calls `resolve_stop`, so **a click on
  a widget inside a group moves focus to the group container**, not to
  the clicked widget, until T7.

Remediated by a doc-comment paragraph at `focus_role` (`6d77dae`) — a
reader of `widget.rs` alone could not otherwise infer any of it — and by
CF-T6-5 below. No behaviour was changed: the fix is T7's.

**F2 — a real trap-#4 gap, and the most representative shape in the
feature.** Every accept-side test wrote `dismiss` as a **flat** sibling
of `modal-scope: true`. Nothing wrapped it in an `if`, which is §4.19's
own worked example and the gallery lightbox's actual shape, so two newly
authored paths had no test firing them: `carries_modal_scope`'s
recomputation at the checker's `Conditional` / `For` recursion, and
`validate_focus_annotation_member_invariants`'s `If` / `For` arms — both
functions added whole by this task. The review hand-verified both were
*correct*, so this was a coverage defect rather than a live bug; a broken
version of either would have passed the entire committed suite.

Remediated in `6d77dae` with nine tests, the discriminating one being an
`if`-wrapped container carrying `dismiss` whose **enclosing** container
carries `modal-scope: true` and which is **still rejected** — a predicate
that leaked from the outer recursive call into the inner one would
wrongly accept it. Two measured facts are recorded at the tests rather
than smoothed over: in the checker a `dismiss` inside a `for` body draws
**two** diagnostics (the pre-existing deferred-handler gate and this
task's), both asserted; in the loader the same shape is intercepted by an
earlier `validate` pass before this gate runs, so the `For` arm is fired
by an admission violation instead and the pre-existing rejection is
asserted for what it is. Each new test was shown to redden under a
deliberate short-circuit of the arm it watches.

**F3 — the Grid handler rule is single-gated, and that is not this
task's to close.** `check_grid` rejects every non-`dismiss` handler on a
`Grid`; the loader has **no** Grid handler gate at all and never has
(verified at `e3ff83a`). So `Grid { clicked => … }` is rejected by
`wasamoc check` and **accepted** by `wasamo_load_ui`. Pre-existing,
unreachable through `wasamoc build` (check aborts first), and therefore
only a memory-IR concern — but it is exactly the checker/loader
divergence CF-T6-3 is about, and the close gate had recorded only the
checker-versus-spec half. Folded into CF-T6-3 below.

**F4 — a loader-side `Cell` admission test was missing** where the
checker had one. Added in `6d77dae`; low risk, since the same code path
was already covered through `Text` / `Button` / `Rectangle`.

Suite after remediation: `cargo fmt --all -- --check` zero exit,
`git diff --check` clean, `cargo test --workspace --no-fail-fast`
**45 binaries/sections, 1,107 passed, 0 failed, 0 ignored** (the ten
added are four checker tests and six loader tests).

#### Carry-forward, revised after the review

CF-T6-1 and CF-T6-3 are restated here in the form the review's evidence
supports; CF-T6-5 is new. CF-T6-2 and CF-T6-4 stand as recorded above.

| Constraint | Evidence | Placement | Re-trigger criterion |
|---|---|---|---|
| **CF-T6-1 (restated) — between this task and T7, a *present* `modal-scope` subtree is *un-entered*, and its subtree is reachable by neither Tab nor click-to-focus.** `focus_core::FocusState::enter_modal` has no production caller, and `collect_stops` returns early for an un-entered `ModalScope`; both `traverse_on_key` and `focus_on_click` read that stop list. The original entry named traversal only | `focus_core::collect_stops`'s `FocusRole::ModalScope if !state.is_entered(id) => return` arm; the two production callers traced in the review | `carry-forward` → this ledger, and `doc-folded` → the `focus_role` doc comment | **T7**, which adds the entry seam. Bounded meanwhile: no shipped `.ui` carries the attribute, and T10 — which adds the first one — lands after T7 |
| **CF-T6-5 (new) — a click on a widget inside a `focus-group` moves focus to the group container, not to the clicked widget.** `FocusTree::tab` resolves a group landing through `resolve_stop`, so Tab is already correct; `focus::focus_on_click` derives its landing from `tab_stops` + `nearest_focusable` and never calls `resolve_stop` | The two call sites, read at the lead's verification of the review's F1 | `carry-forward` → this ledger, and `doc-folded` → the `focus_role` doc comment | **T7**, which owns group traversal and the per-group memory `resolve_stop` reads. The asymmetry is the tripwire: any fix that makes the click path agree with Tab must go through the same primitive rather than adding a second landing resolver |
| **CF-T6-3 (restated) — a per-kind signal admission rule exists in three places with three different shapes, and `plan.md` §T8's premise is false for two kinds.** `check_grid` rejects every non-`dismiss` handler on a `Grid` (compiler only — the loader has **no** Grid handler gate and never has, so `Grid { clicked => … }` is rejected by `check` and accepted by `wasamo_load_ui`); `validate_phase6_zstack_node_invariants` rejects every non-`dismiss` handler on a `ZStack` (loader only — the checker admits them). `Box`, the stacks, `WrapPanel` and `ScrollView` have no rule on either side | Start-gate fact 4 for the checker; the review's probe for the absent loader Grid gate, verified against `e3ff83a`; `non_dismiss_handler_on_grid_still_rejected` and `zstack_clicked_handler_still_rejected_after_relaxation` pin the current bound | `finding` → **T8** | **T8**, which widens `clicked` to any widget. It must decide three things, not one: widen `check_grid`, widen the ZStack loader gate, and whether the Grid rule gains the loader half it never had. Leaving any of them narrows the authored surface against §4.19 and hands T13 a divergence |

#### Owner disposition of CF-T6-2 (2026-08-07)

The close gate assigned the both-annotations question to **T7**. Reading
T7's plan item shows it owns group traversal, scope entry / exit, the
materialisation seam, dismissal and the spike retirement — a **combined
group-and-scope role is in none of them**. An assignment nothing
executes is not a carry-forward, so the row is re-dispositioned:

- **Owner (2026-08-07): keep the shape accepted, and warn.** `wasamoc`
  emits one warning per container carrying both `focus-group: true` and
  `modal-scope: true`, naming `focus-group` as the half with no effect
  (`f96e1c4`). The surface DD-M4-P2-005 chose is not narrowed — rejecting
  the combination would withdraw the reason A1 was selected over A3 —
  but the state stops being *silent*, which is the failure mode this
  phase closes everywhere else (a `dismiss` that could never fire, a
  misspelled key name that never matches).
- **The surface question itself goes to the
  [candidate pool](../../../candidate-pool.md)**, no milestone claimed,
  carrying the owner's direction: **if the combination turns out not to
  be wanted, the two booleans should become one attribute with an
  enumerated value.** That is DD-M4-P2-005's option A3, rejected there
  *only* because it made the combination inexpressible — so dropping the
  requirement drops the objection. Deciding the other way means giving
  the traversal core a combined role.
- **A combined role is not built now.** No M4 app writes both, so its
  branches would be branches no test can fire (implementation-gates
  trap 4).

Two further witnesses close the new branch, applied and reverted with
the revert confirmed by re-reading:

| Witness | Mutation | Went red | Reading |
|---|---|---|---|
| **W9 — the warning removed** (restoring) | the diagnostic is not pushed, restoring the pre-warning behaviour | the two warning tests, and **nothing else** in `wasamoc` | The tests catch the absence of the warning, not only its wording |
| **W10 — the `true` requirement dropped** | the scan matches a `focus-group` bind of any value | `focus_group_false_modal_scope_true_no_warning` alone | `focus-group: false` beside a scope is an ordinary scope, and the warning must not fire there |

The message is author-facing rather than implementation-facing: an
earlier draft said "a node can hold only one focus role", which names a
runtime-internal enum a `.ui` author never sees and `docs/dsl_spec.md`
never uses. It reads, verbatim:

> this container carries both `focus-group: true` and `modal-scope: true`;
> a container can behave as one or the other, not both, and a modal scope
> wins, so `focus-group` has no effect here. Remove it, or move the group
> to a child container (dsl_spec §4.19).

#### Re-verification after the warning (recorded 2026-08-07)

Run against the **final branch state**, superseding the counts recorded
above (the branch gained the warning after they were taken):

`cargo fmt --all -- --check` zero exit, `git diff --check` clean,
`cargo clean` (8,293 files / 2.4 GiB removed), then
`cargo build --release --workspace` 1m25s success,
`cargo build --workspace` 53s success,
`cargo test --workspace --no-fail-fast` **45 binaries/sections, 1,111
passed, 0 failed, 0 ignored**. T5's baseline was 1,051; the 60 added are
`check.rs`'s 31 unit tests, `ir_loader.rs`'s 28, and the one integration
fixture. The integration fixture ran rather than skipped, verified with
`--nocapture`.

#### Disposition of the T8 finding (CF-T6-3), for the record

The Grid / ZStack handler asymmetry is **not** an owner decision this
task's merge waits on. It is a measurement that corrects §T8's premise,
recorded in the carry-forward table and written into
[plan.md](./plan.md) §T8; the decision belongs to T8's own start gate.
The T6 retrospective's owner-consultation item lists it no longer, for
that reason.

## T7 — Group traversal and modal scopes in the runtime

### Start gate (recorded 2026-08-07, before any source edit)

Read first: [AGENTS.md](../../../../AGENTS.md),
[implementation-gates.md](../../../procedures/implementation-gates.md),
[plan.md](./plan.md) §T7 and §Cross-task obligations,
[preamble.md](./preamble.md),
[DD-M4-P2-003](../decisions/dd-m4-p2-003-focus-model-and-traversal.md),
[DD-M4-P2-004](../decisions/dd-m4-p2-004-modal-focus-scope.md),
[constraints.md](../requirements/constraints.md), the T5 and T6 close
gates above, and the T6 retrospective.

#### Normative statements that already answer this task (DD-V-031)

This phase synchronises its normative text ahead of implementation, so
the questions below are **answered, not open**. Listed with what each
fixes, so a disagreement found while building is recorded as a T13
divergence rather than re-decided here.

| Question | Where it is answered | What it fixes |
|---|---|---|
| What a group is | [dsl_spec §4.19 §`focus-group`](../../../../docs/dsl_spec.md) | One Tab stop; Tab enters and leaves but does not step between members; arrows move within, wrapping; the group remembers the member last focused inside it; a group never entered lands on its first member |
| What entry is | [dsl_spec §4.19 §`modal-scope`](../../../../docs/dsl_spec.md), [architecture.md §13.4](../../../../docs/architecture.md) | "Being there is being open." Entry runs on the structural seam that materialises the subtree — the drain that makes a conditional true, iteration generating it, **or the initial build** — and does three things: push in materialisation order, capture the focused node as the restore target, move focus to the scope's first stop (or leave it unset when the scope has none, keys then starting at the scope) |
| What exit is | [architecture.md §13.4](../../../../docs/architecture.md) | Removal of the subtree; **restoration takes precedence over structural succession**; a removal's successor is computed before the mutation |
| What a scope confines | [dsl_spec §4.19 §What a scope does not do](../../../../docs/dsl_spec.md) | The keyboard only. Pointer confinement is the occlusion rule plus an authored covering widget; a scope with no covering child traps Tab and passes clicks through |
| What dismissal is | [dsl_spec §4.19 §`dismiss`](../../../../docs/dsl_spec.md), [architecture.md §13.5](../../../../docs/architecture.md) | A request **addressed** to the innermost scope, stopping there; the runtime delivers and never acts on it; writing no handler means the scope does not close by dismissal |
| Which keys the runtime keeps | [dsl_spec §4.19 §Which keys the runtime keeps](../../../../docs/dsl_spec.md) | `Tab` / `Shift+Tab` always; arrows **while focus is inside a `focus-group`**, otherwise the propagation walk; `Escape` **while a modal scope is present**, otherwise the propagation walk. A key that reaches the end of the walk without a handler continues to the window's default handling |
| Where the focus record lives, and its writer discipline | [architecture.md §13.3](../../../../docs/architecture.md) | One record per window holding the focused node and three derived stores (group memory, active-item pointers, the scope stack); the group memory is written by the **same primitive** that writes the focused node |
| Whether entry may enqueue further work | [architecture.md §13.4](../../../../docs/architecture.md) | "It writes runtime focus state only and enqueues no further drain work" — an invariant this task asserts rather than assumes |
| What a click lands on | [dsl_spec §4.19 §Focus](../../../../docs/dsl_spec.md), [DD-M4-P2-003 §Click-to-focus](../decisions/dd-m4-p2-003-focus-model-and-traversal.md) | The nearest focusable widget **at or above** the resolved target; unchanged when there is none — clicking background never clears focus |
| Whether the indicator may add a geometry write | [architecture.md §13.3](../../../../docs/architecture.md), [DD-M4-P2-003](../decisions/dd-m4-p2-003-focus-model-and-traversal.md) | It may not. The `SetOffset` / `SetSize` enumeration is the close artifact of any task that could add one (carried from T5) |

**One question the normative text does not answer, decided here and sent
to T13.** Whether a click *outside* an entered scope may move focus
outside it. §4.19 says the scope confines the keyboard and that clicks
pass through a scrim-less scope; it does not say what such a click does
to focus. **The landed T5 behaviour already answers it and is kept**:
`focus_on_click` enumerates stops from `traversal_root`, so with a scope
entered there is no candidate outside it and focus is left unchanged —
the same arm as a background click. Keeping confinement is the reading
consistent with "no widget outside it can be reached by the keyboard";
recorded as a T13 re-verification item rather than a silent choice.

#### Measured facts (probes run before choosing an approach)

**Fact 1 — the whole scope / arrow half of `focus_core` has zero
production callers.** A `rg` over `wasamo-runtime/src`,
`wasamo-runtime/tests`, `examples` and `bindings` for `enter_modal`,
`exit_modal`, `apply_arrow`, `focus_after_removing`, `esc_target`,
`initial_focus` and `.arrow(`, excluding `focus_core.rs` itself, returns
**one doc-comment mention** (`widget.rs:1199`, T6's reachability caveat)
and **nine hits in `tests/focus_mechanism_fixture.rs`** — the spike
fixture this task retires. So every behaviour T7 lands is a first
production caller, and no existing production test can regress it.

**Fact 2 — the structural seam is already centralised, and it is the
layout-invalidation seam.** `rg "mark_layout_dirty_for"` over
`wasamo-runtime/src` returns the definition plus **five** call sites:
four in `ir_loader.rs` (conditional insert 3541, conditional remove
3553, `for` range insert 3697, `for` range remove 3710) and one in
`widget.rs:1353` (a size-affecting property write). Every reactive
structural mutation marks its window dirty; `emit::flush_layout` —
Phase 2 of `drain_if_outermost` — then visits every dirty window with a
sound `&mut WindowState` **and** its root. The initial build is
`window::set_root`, which `wasamo_load_ui` also goes through
(`abi.rs:1281`). The seam is therefore **two places**, not one, and both
already exist: `set_root` and `flush_layout`.

**Fact 3 — the four direct-ABI structural entries do not reach that
seam, and cannot carry the annotation.**
`wasamo_widget_append_child` / `insert_child` / `remove_child` /
`replace_child` call `WidgetNode::insert_child` / `remove_child` /
`replace_child`, none of which calls `mark_layout_dirty_for`. That is the
"outside the layout boundary" classification DD-M4-P2-002 already
records, and the exact residual
[DD-M4-P2-004](../decisions/dd-m4-p2-004-modal-focus-scope.md) names
("a dangling stack entry requires a removal path that bypasses the
structural seam"). Bounded further by construction:
`set_focus_annotation` has exactly one caller
(`ir_loader::build_node_with_loop_context`, T6's audit), so a node
created through the C ABI can never carry `modal-scope`.

**Fact 4 — running the seam inside the effect would alias, running it in
Phase 2 does not.** `mutate_conditional_subtree` executes inside
`Signal::set`'s synchronous drain, which for a click runs **inside**
`hit_test_click`, which holds `&mut WidgetNode` on the window's root.
Forming `&mut WindowState` there to reach `state.focus` would alias that
borrow. `flush_layout` runs after `wnd_proc` returns, at the message-loop
boundary, where `&mut *wptr` is already the established pattern. This is
what places the seam in Phase 2 rather than at
`insert_structural_child` / `remove_structural_child`.

**Fact 5 — "the successor is computed before the mutation" needs no
pre-mutation tree walk, because the two pieces it needs are held
elsewhere.** `focus_core::FocusTree::focus_after_removing`'s structural
succession is "the first stop in the domain that is not inside the
removed subtree" — which is exactly what T5's lazy landing already
produces (`focus_traversal_integration` fixture 3's third leg). The half
that genuinely cannot be recovered after the mutation is the **restore
target**, and that is captured at entry and retained on the scope's stack
entry. So CF-T5-2's observable content is *restoration precedence*, not
succession, and a post-mutation reconciliation satisfies
DD-M4-P2-004 as long as the capture happened at entry.

**Fact 6 — a `FocusId` is a coordinate in a projection that is rebuilt
per operation, and T7 is the task that makes retaining one across a
mutation load-bearing.** `WindowFocus` holds `focus_core::FocusState`,
whose `focused`, `group_memory`, `active_item` and `modal_stack` are all
keyed by `FocusId` = the pre-order index of `FocusProjection`'s walk. A
structural insert or removal shifts every id at or after the mutation
point. Today only `focused` is retained across messages, and
`discard_stale_focus` catches only the **out-of-range** case (CF-T5-1). A
modal stack entry is retained for the whole life of the scope — the
longest-lived retained id in the runtime — so this task cannot leave the
in-range case open the way T5 could.

**Fact 7 — node addresses are stable and are already the runtime's
cross-mutation identity.** A `WidgetNode`'s children are
`Vec<ChildSlot>`, and `ChildSlot` owns a `Box<WidgetNode>`
(`widget.rs:207`), so a node's address does not move when its parent's
child vector reallocates. `registry`'s observer / signal entries,
`emit::mark_layout_dirty_for` and `focus_spike::Projection::id_of`
already identify widgets by that address. Pointer identity is therefore
an existing house mechanism, not an invention.

**Fact 8 — the click path's group defect (CF-T6-5) is not fixed by
`resolve_stop`.** `resolve_stop` returns a group's *remembered* member,
which is right for Tab and wrong for a click: clicking "Favorites" must
focus "Favorites", not the remembered "Albums". The landing rule a click
needs is per-node — the nearest **enabled `Stop`** at or above the
target, with a `Group` (reached when the click did not land on a member)
resolving through `resolve_stop`, and the whole walk bounded by the
current traversal domain. `focus::nearest_focusable` cannot express it,
because it is defined against `tab_stops`, in which a group's members do
not appear at all.

**Fact 9 — integration fixtures can reach Phase 2.**
`event_routing_integration.rs`'s `click_and_drain` posts the click and a
`WM_QUIT`, then pumps `wasamo_runtime::run()`, whose
post-`DispatchMessageW` `emit::drain_if_outermost` runs Phase 2. So a
Phase-2 seam is observable from a fixture without adding a production
call site that exists only for tests.

#### What this task turns out to be

The plan's T7 bullets survive, and the start gate adds one that was not
in them and is a precondition for two of them:

- **The retained focus record needs a coordinate system that survives a
  structural mutation.** Facts 6 / 7 make this the enabling change: ids
  are rebased through node addresses at every operation, which closes
  CF-T5-1 (in-range staleness is now *detected*, not only out-of-range)
  and lets a scope's stack entry outlive the mutations that happen while
  it is open. Without it, entry / exit is built on a key that silently
  renames nodes.
- Facts 2 / 4 / 5 place entry and exit at **`set_root` + `flush_layout`**
  rather than at `insert_structural_child` / `remove_structural_child`.
  The plan predicted "the seam" and asked for it to be enumerated before
  it is trusted; the enumeration says it is the layout-invalidation seam,
  which is one hop *later* than the plan's wording ("structural drain")
  suggests, and is the only place a sound `&mut WindowState` exists.
- Fact 8 makes CF-T6-5 a **new landing rule in `focus_core`**, not a
  re-use of `resolve_stop` — a correction to the plan's bullet, which
  reads as though calling the existing primitive were the whole fix. The
  plan's actual requirement — that the fix go through the primitive the
  memory is written by — is met either way: the landing still reaches
  focus through `move_focus` → `FocusState::set_focus`, which is the sole
  writer of `group_memory`.

**The un-entered state (CF-T6-1) is closed by the seam, and the branch
keeps its test rather than being narrowed away by the projection.**
`collect_stops`'s `FocusRole::ModalScope if !state.is_entered(id)` arm
stays reachable in principle — through fact 3's ABI paths — and is fired
by `focus_core`'s own unit test
`an_unentered_modal_scope_is_not_reachable_by_tab`. The plan allowed
either resolution and required that it be recorded; this is the recorded
one.

#### Selected traps

```
- [x] #1 semantic migration   - [x] #2 side effects   - [x] #3 parallel data   - [x] #4 branch tests
- [x] #5 carry-forward        - [ ] #6 root cause (armed)  - [x] #7 GUI positive control
```

| Trap | Applies | Why / why not |
|---|---|---|
| 1 | **yes** | Two migrations with no compiler enumeration. `FocusId`'s *meaning* changes — a coordinate that is now rebased rather than assumed stable — without its type changing, so nothing breaks at compile time; and `focus_core`'s six-variant `FocusRole` gains its first production producers for `Group` / `ModalScope`, so every traversal call-site that filters on role must be classified. The artifact is a call-site audit table over `focus_role`, the `FocusId`-consuming functions, and every reader of the stop list |
| 2 | **yes** | Entry and exit are state changes with derived effects: the painted indicator, the group memory, the scope stack, layout dirtiness, and — the one DD-M4-P2-004 names explicitly — whether entry enqueues further drain work. Enumerated at close **from the diff rather than from intent** (the T6 retrospective's corrective) |
| 3 | **yes** | Three parallel pairs. The focused id and the painted flag (T5's, preserved); the focused id and the group memory (DD-M4-P2-003 adopts the single-writer enforcement); and the new one — the retained ids and the coordinate system they are expressed in, which must be written together or the ids name nothing checkable |
| 4 | **yes** | New branches: the click landing arms (Stop / Group / disabled / outside the domain), the arrow arms (moved focus / not handled), the Escape arms (scope entered / not entered / scope without a `dismiss` handler), and the reconciliation arms (push / pop / restore / fall to first stop). Each ships with a test that fires it directly |
| 5 | **yes** | T8 inherits the keys this task consumes (arrows inside a group, Escape while a scope is entered) as the tripwire for its own dispatch; T9 inherits the same identity model through `for` regeneration; T10 is the first `.ui` to carry the annotations; T12's control C is the first frame that can see a scope. Anything this task *requires as evidence of a later task* is built and run here, or recorded as a finding with an owner (the T3 retrospective's corrective) |
| 6 | **armed, not applying** | No failure has been observed yet. Armed because this task changes an identity model underneath existing fixtures: if a T5 fixture goes red, the obligation is minimal repro → root cause → disposition, never a re-roll |
| 7 | **yes** | The focus indicator moving into a scope on entry, and back out on exit, is painted. The plan puts the gallery frames at T12, but this task's own evidence must include a frame pair with a positive control, because a scope that confines and a scope that is simply empty look identical in a single frame. The control is the **agreement leg**: the same Tab, with the scope absent, reaching the background |

Additions to the per-task start-gate line the earlier retrospectives
accumulated (T1 new store / unit / coordinate system; T2 tests pinning
the property being deleted; T3 build once, here, the evidence required of
the next task; T4 measure the negative prediction being relied on; T5 the
lifetime of identifiers retained across messages; T6 how many gates the
new rule has):

- **T1's line fires.** The new coordinate system is the anchor vector —
  a *new store*, in the same sense T1's rectangle was, and its writer
  must be single by construction.
- **T2's line fires.** `focus::nearest_focusable` and
  `focus::discard_stale_focus` are both deleted. Each has tests
  (`nearest_focusable`'s four unit tests; fixture 6 for
  `discard_stale_focus`). The unit tests go with the function; fixture 6
  **stays and must stay green under the replacement**, because what it
  pins is the observable behaviour, not the mechanism.
- **T5's line fires**, and is the task's centre — see fact 6.
- **T6's line does not fire**: this task adds no rule with a compiler
  gate. Its rules are runtime behaviour with exactly one gate each. The
  analogous check that *does* fire is the two-sided one — every rule this
  task lands about which keys the runtime keeps needs both the
  consumption leg **and** the fallthrough leg, since a key silently
  consumed and a key correctly consumed are indistinguishable from the
  consumption side alone (T5 fixture 5's shape, and CF-T5-5).

#### The seam enumeration ([plan.md](./plan.md) §T7, required before it is trusted)

| Path that materialises or removes a subtree | Runs the entry / exit seam | How |
|---|---|---|
| `window::set_root` (initial build; also every `wasamo_load_ui`) | **yes** | Directly, after the initial layout pass |
| `ir_loader::mutate_conditional_subtree` — insert | **yes** | `mark_layout_dirty_for` → `drain_if_outermost` Phase 2 → `flush_layout` |
| `ir_loader::mutate_conditional_subtree` — remove | **yes** | same |
| `ir_loader::mutate_for_loop_subtree` — tail-range insert | **yes** | same |
| `ir_loader::mutate_for_loop_subtree` — tail-range remove | **yes** | same |
| `wasamo_widget_append_child` / `insert_child` / `remove_child` / `replace_child` | **no** | Outside the layout boundary (DD-M4-P2-002); marks nothing dirty. Bounded by fact 3: a C-ABI-created node cannot carry the annotation, so it cannot introduce a scope. What it *can* do is shift the tree under a retained anchor, which the rebase detects and clears rather than mis-resolves |
| `lib.rs::window_add_widget` | **no** | Attaches a Visual without putting the widget in `root_widget`; it is not in the focus projection at all |

**If the seam were not one mechanism this would be a plan change.** It is
one mechanism reached from two entry points, which is what the plan's
"structural drain or initial build" already describes; the correction is
only *where in the drain* it runs (fact 4).

#### Review lane

**Full independent review**, as [preamble.md](./preamble.md) predicts.
Confirmed rather than inherited: this task is a runtime structural change
three times over (a new retained store and its rebase primitive, the
scope stack and restore state, and a new consumption arm in the
`WM_KEYDOWN` return path), and it carries GUI-render evidence. The trap-4
branch/test check composes in rather than replacing it
([implementation-gates.md §4](../../../procedures/implementation-gates.md)).

#### Boundaries this task does not cross

- **No new ABI function** (the phase's cross-task obligation). Any new
  symbol is a `__*_for_test` seam on the existing `ffi` module.
- **No authored key handler dispatch.** T8 owns `key-down("<key>")` and
  the generic key walk; T7 lands only the keys the *runtime* keeps, and
  an unconsumed key still falls through to the host key slot and then to
  `DefWindowProcW` (T5's arm, unchanged).
- **No combined group-and-scope role** (CF-T6-2 — owner-settled to the
  candidate pool; no M4-Phase 2 task owns it).
- **No `.ui` gains an annotation.** T10 is the first; annotating the
  gallery here would pre-empt it.
- **`ActiveItemList` / `ActiveItem` gain no production producer**, so
  focus / active-item separation keeps only its `focus_core` unit tests —
  the deliberate narrowing the plan records, restated at close.

### Close gate (recorded 2026-08-08)

Landed in `wasamo-runtime/src/focus.rs` (`FocusProjection::anchors` and its
two accessors, `WindowFocus::anchors` and `rebase`, `DroppedScope` /
`DroppedScopes`, `with_focus_write`, `sync_scopes_to_tree`,
`arrow_direction`, `arrow_on_key`, `dismiss_on_key`; `move_focus` reduced
to a wrapper; `discard_stale_focus` and `nearest_focusable` deleted);
`wasamo-runtime/src/focus_core.rs` (`FocusState::remap`, `modal_entries`,
`FocusTree::focus_landing`); `wasamo-runtime/src/widget.rs`
(`SignalHandlers`, `signal_handlers_for`, `run_signal_handlers`,
`WidgetNode::deliver_dismiss_at`; `ClickedHandlers` and
`run_clicked_handlers` refactored onto the shared helper);
`wasamo-runtime/src/window.rs` (`set_root`'s seam call, the `WM_KEYDOWN`
arm's two new consumers); `wasamo-runtime/src/emit.rs`
(`pending_counts`, `flush_layout`'s seam call);
`wasamo-runtime/src/lib.rs` (the `__focus_spike` module and the
`focus_spike` declaration deleted); `wasamo-runtime/src/focus_spike.rs`
**deleted**; `wasamo-runtime/tests/focus_identity_integration.rs` (new, 3
fixtures), `wasamo-runtime/tests/modal_scope_integration.rs` (new, 7
fixtures), `wasamo-runtime/tests/focus_mechanism_fixture.rs`
(re-pointed, 3 fixtures);
`process/milestone-4/phase-2/implementation/evidence/capture-t7-scope-entry.ps1`
(new).

**Nothing was landed in** `layout.rs`, `hit.rs`, `abi.rs`, `reactive.rs`,
`registry.rs`, `handler.rs`, `dip_scale.rs`, `runtime.rs`, `text.rs`,
`box_values.rs`, or anywhere in `wasamoc`, `wasamo-ir`, `docs/` or
`examples/`. The start gate predicted the compiler and the shipped `.ui`
would be untouched and both held.

#### Correction to the start gate

**Fact 1's count is nine, not ten.** The independent review re-ran the
census over `850cb64` and got nine hits for the seven-name pattern in
`tests/focus_mechanism_fixture.rs`. Nine is correct; the conclusion the
fact draws — that the whole scope / arrow half of `focus_core` had no
production caller, only that spike fixture — is unchanged either way.

#### #1 — Call-site audit table

Two migrations. Rust enumerates neither, because neither changes a type:
`FocusId`'s **meaning** changes (a coordinate that is rebased, where it
was previously assumed stable) and `FocusRole::Group` / `ModalScope` gain
their first production **producers** while the enum itself is untouched.
The rows are the artifact.

Queries: `rg "FocusProjection::project\(" wasamo-runtime/src`;
`rg "\.set_focus\(|\.enter_modal\(|\.apply_arrow\(|\.remap\(|exit_modal\(" wasamo-runtime/src`;
`rg "set_button_focused_at" wasamo-runtime/src`;
`rg "mark_layout_dirty_for" wasamo-runtime/src`;
`rg "insert_child|remove_child|replace_child" wasamo-runtime/src`;
`rg "SetOffset|SetSize" wasamo-runtime/src`;
`rg "focus_spike|__focus_spike"` (whole workspace, the retirement check);
`rg "focus_role|focus_annotation" wasamo-runtime/src`.

| Site | Classification | Reason |
|---|---|---|
| `focus.rs` `FocusProjection::project` — **six** call sites (`sync_scopes_to_tree`, `traverse_on_key`, `focus_on_click`, `arrow_on_key`, `dismiss_on_key`, `focused_path`) | **must-rebase, audited individually** | Every one builds a fresh projection whose ids a retained record must be re-expressed against. Five call `WindowFocus::rebase` before touching `focus.core`. The sixth, `focused_path`, **cannot** — it takes `&WindowFocus` — and is bounded by its only caller being the `__focus_path_for_test` seam; the residual is CF-T7-2 and is stated in its doc comment rather than implied |
| `focus.rs` `WindowFocus::rebase` | **new, sole writer of the pair** | Writes `core`'s remapped ids and the `anchors` they are expressed in, together. `anchors` is private with no setter, the discipline `core` already had |
| `focus_core.rs` `FocusState::remap` | **new, the whole of the migration** | Rewrites all four id-keyed stores — `focused`, `group_memory`, `active_item`, `modal_stack` — so no store is left expressed in the previous coordinate system. Each store has its own drop rule and its own test |
| `focus_core.rs` `FocusState::set_focus` | **migrated, still the sole writer of `focused`** | Production callers are now four, all inside `with_focus_write`'s closure argument: `move_focus`, `sync_scopes_to_tree`'s restoration step, its succession step, and — transitively — `enter_modal` and `apply_arrow`. The field stays private |
| `focus.rs` `with_focus_write` | **new, the one primitive** | The only function in the crate that calls `WidgetNode::set_button_focused_at` (`rg` shows two calls, both inside it). Every production write of the focused id runs through it, so the record and the painted indicator cannot be edited apart — the T4 / T5 trap-#3 shape, widened to cover the two new writers |
| `widget.rs` `set_button_focused_at` | **migrated, caller changed** | Its doc comment named `move_focus` as the only caller; that is now `with_focus_write`, which `move_focus` delegates to. Updated rather than left stale |
| `focus_core.rs` `FocusState::enter_modal` | **new production caller** | `sync_scopes_to_tree`'s entry step. Its role check (the spike's S-3 fix) is not bypassed: the loop's own `role(id) == ModalScope` filter is a pre-order selection, not a substitute, and the doc says so |
| `focus_core.rs` `FocusState::exit_modal` | **ignore-OK, deliberately not adopted** | Presence-driven exit makes it unreachable: a present scope is always entered, so the only exit is the subtree leaving, which `rebase` detects by the scope's anchor vanishing. Recorded as CF-T7-4 so a later phase does not read its dead state as an oversight |
| `focus_core.rs` `FocusTree::focus_after_removing` | **ignore-OK, deliberately not adopted** | Start-gate fact 5: its structural succession is the domain's first surviving stop, which `initial_focus` produces from the post-mutation tree. Also CF-T7-4 |
| `focus_core.rs` `FocusTree::focus_landing` | **new, the click landing** | Replaces `tab_stops` + `nearest_focusable`. `collect_stops` and this differ in exactly one place — a group's members — which is the rule, not drift, and the doc states the relationship |
| `focus.rs` `nearest_focusable` | **deleted** | Defined against `tab_stops`, in which a group's members never appear, so it could not express the landing rule. Its four unit tests go with it; the definition it protected now lives beside `collect_stops` |
| `focus.rs` `discard_stale_focus` | **deleted** | Its whole job — an out-of-range retained id — is the degenerate case of the rebase. Fixture 6 of `focus_traversal_integration.rs`, which was its regression test, stays green unchanged, which is the measurement that the observable behaviour survived the mechanism change |
| `window.rs` `set_root` | **new seam call site** | After the initial layout. Also the entry point for a scope present in the initial tree (DD-M4-P2-004) |
| `emit.rs` `flush_layout` | **new seam call site** | Phase 2 of `drain_if_outermost`, reached by every reactive structural mutation through `mark_layout_dirty_for`. Start-gate fact 4 is why it is here and not at `insert_structural_child` / `remove_structural_child` |
| `emit.rs` `mark_layout_dirty_for` — five call sites (four structural in `ir_loader.rs`, one property write in `widget.rs:1353`) | **unchanged, re-audited** | The four structural ones are the seam's reach. The fifth is a size-affecting property write; reconciling there is a no-op, since the scope set is unchanged |
| `abi.rs` `wasamo_widget_append_child` / `insert_child` / `remove_child` / `replace_child` | **ignore-OK, outside the seam** | None marks the window layout-dirty (DD-M4-P2-002's "outside the layout boundary"). They cannot introduce a scope — `set_focus_annotation` still has exactly one caller, the IR loader — but they can shift the tree under a retained anchor. CF-T7-2 |
| `window.rs` `wnd_proc` `WM_KEYDOWN` arm | **migrated, two consumers inserted** | Order is Tab → arrows → Escape → host key slot → `DefWindowProcW`. The arm's no-`return` fallthrough (T5's) is unchanged, which is what CF-T5-5's tripwire depends on |
| `widget.rs` `run_clicked_handlers` / `click_disposition_for` | **migrated, inline+host half extracted** | The native-closure step stays in `run_clicked_handlers`; ordering (native → inline → host enqueue) is unchanged. The snapshot-then-run split — the reason the walk is sound while a handler rebuilds the tree — is now stated once, in `run_signal_handlers`, and shared |
| `widget.rs` `deliver_dismiss_at` | **new, single addressee** | No propagation walk and no suppression check: DD-M4-P2-004 addresses the request to the innermost scope rather than walking to it |
| `widget.rs` `focus_role` | **unchanged code, corrected doc** | Its reachability caveat had gone stale in three ways and described two defects this task fixed as current. Found by the independent review, rewritten to the landed state |
| `focus_spike.rs`, `lib.rs::__focus_spike` | **deleted** | `rg "focus_spike|__focus_spike"` over `--include=*.rs` returns **zero hits** workspace-wide |
| `abi.rs` (as a whole) | ignore-OK, unchanged | No `extern "C"` function added or altered; the cross-task "no new ABI function" obligation holds |

#### #2 — Structural side-effect enumeration

Built from `git diff 850cb64..HEAD`, not from what the task set out to
add — the T6 retrospective's corrective, which is what caught two whole
recursion arms there.

| Derived effect | Disposition |
|---|---|
| **The painted focus indicator** | Written by `with_focus_write` in the same call as every focused-id write. Two `set_button_focused_at` calls, both inside it |
| **The per-group memory** | Unchanged mechanism: `FocusState::set_focus` writes it, and every path that writes focus reaches `set_focus`. It gains its first production exercise here (arrows inside a group, and `resolve_stop` reading it back on a Tab or click landing) |
| **The modal stack** | New retained state. Pushed only by `enter_modal` from the entry step; dropped only by `remap` when the scope's anchor is gone. There is no other mutator |
| **The anchor coordinate system** | New retained state, written only by `rebase`, together with the ids it explains |
| **Layout** | Untouched. The annotation was already inert for layout at T6, and nothing this task adds is read by `build_layout_tree`, `measure`, `arrange` or `sync_visuals` |
| **Composition geometry** | Unchanged — see #3 |
| **`Visual` creation** | None. The indicator is a brush colour on an existing `Visual`, which is what DD-M4-P2-003 requires and what §13.3's second clause states literally |
| **Drain work enqueued by entry** | **Asserted, not assumed**: `debug_assert_eq!` over `emit::pending_counts()` immediately before and after the entry loop, discharging §13.4's "writes runtime focus state only and enqueues no further drain work". Witness W8 shows the assertion fires when the invariant is broken |
| **`window::set_root`'s reset** | `state.focus = WindowFocus::default()` already cleared the record; the seam call now also gives it a coordinate system for the tree that exists, and enters any scope present in it |
| **The spike's second projection** | Deleted, so there is no second reader of `focus_role` to keep in step |
| **`ButtonData.label_size`'s three-point write** ([constraints §4](../requirements/constraints.md)) | Not touched |
| **The `clicked` dispatch order** | Unchanged by the shared-helper refactor: native closure, then inline bodies, then the host enqueue. Verified by reading the final `run_clicked_handlers`, and by `event_routing_integration.rs`'s five fixtures staying green |

#### #3 — Every `SetOffset` / `SetSize` in the runtime, with its pass

Carried from T5 because DD-M4-P2-003 requires the enumeration from any
task that could add a geometry write, and this task moves the focus
indicator.

Query: `rg "SetOffset|SetSize" wasamo-runtime/src` — eight hits.

| Site | Pass | What it writes |
|---|---|---|
| `widget.rs` node Visual offset / size (2 calls) | `sync_visuals` | The node's own Visual |
| `widget.rs` Button-family label (2 calls) | `sync_visuals` | The label Visual |
| `widget.rs` `ScrollView` intermediate (2 calls) | `sync_visuals` | The content Visual |
| `dip_scale.rs` (2 mentions) | — | Doc comments naming the operations, not calls |

**Six calls, all inside `sync_visuals`, unchanged from T1 / T2 / T3 / T4 /
T5.** Entry, exit, arrows and dismissal add none.

#### #4 — Branch tests, each fired directly

Built from the diff. 13 integration fixtures across three files, plus 25
new unit tests in `focus_core.rs` and 8 in `focus.rs`.

**The coordinate system**

| Authored arm | Test that fires it |
|---|---|
| `remap` — `focused` identity / shift / unmappable | `remap_with_the_identity_mapping_changes_nothing`, `remap_with_a_shifting_mapping_moves_focused`, `remap_drops_focused_when_it_is_unmappable` |
| `remap` — group memory, key unmappable / value unmappable | `remap_drops_a_group_memory_entry_whose_key_is_unmappable`, `…_whose_value_is_unmappable` |
| `remap` — active item | `remap_moves_an_active_item_entry` |
| `remap` — modal entry, scope unmappable / `restore_to` unmappable / order | `remap_drops_a_modal_entry_whose_scope_is_unmappable`, `remap_keeps_a_modal_entry_whose_restore_to_is_unmappable_with_restore_to_cleared`, `remap_preserves_modal_stack_order` |
| The in-range stale id, end to end | `a_retained_focus_record_survives_a_structural_removal_that_shifts_ids` — the branch T5 recorded as unbuildable (CF-T5-1's in-range half); the fixture pins the arithmetic (id 5 → 2, removal of 3 nodes) so it cannot silently degenerate into the out-of-range case |
| `DroppedScopes::outermost` — 0 / 1 / 2 entries | `outermost_of_no_entries_is_none`, `outermost_of_one_entry_is_that_entry`, `outermost_of_two_entries_is_the_first_not_the_innermost` |

**The click landing**

| Authored arm | Test that fires it |
|---|---|
| Target is itself an enabled `Stop` | `focus_landing_returns_the_target_itself_when_it_is_a_stop` |
| Target is a non-focusable container; an ancestor is a stop | `focus_landing_climbs_to_an_ancestor_stop_when_the_target_is_a_non_focusable_container` |
| A group **member** wins over its group | `focus_landing_on_a_group_member_focuses_that_member_not_the_remembered_one` (the memory is set to a different member first, so a `resolve_stop`-based implementation fails it) |
| The group container itself → `resolve_stop` | `focus_landing_on_the_group_container_itself_falls_back_to_the_remembered_member` |
| A disabled member → the group's first | `focus_landing_on_a_disabled_member_falls_back_to_the_groups_first_member` |
| Nothing legal → `None` | `focus_landing_over_a_chain_with_nothing_focusable_is_none` |
| Outside / inside an entered scope | `focus_landing_outside_an_entered_modal_scope_is_none`, `focus_landing_inside_an_entered_modal_scope_lands_normally` |
| End to end, through a real click | `a_click_inside_a_focus_group_focuses_the_clicked_member` (two legs: the second member, then the first) |

**Entry, exit and succession**

| Authored arm | Test that fires it |
|---|---|
| Entry through the production seam (a state write flips the `if`) | `entry_is_driven_by_the_production_seam` |
| Entry at the initial build | `entry_at_initial_build` |
| Entry skipped for an unannotated subtree (the spike's S-3 leg) | `a_present_but_unannotated_subtree_does_not_confine` |
| Exit restoration, `restore_to` present, beating succession | `exit_restores_and_restoration_beats_succession`, `a_thumbnail_click_enters_the_modal_scope_and_escape_restores_it` |
| Exit restoration, `restore_to` **absent** | `exit_with_no_restore_target_leaves_focus_unset` |
| Structural succession, through the real seam | `structural_succession_lands_on_the_domains_first_surviving_stop` |
| Nesting: stack order, innermost addressing, unwinding | `entering_nested_scopes_stacks_outer_then_inner`, `traversal_root_and_esc_target_name_the_innermost_entered_scope`, `exiting_both_nested_scopes_restores_each_entrys_own_capture` |
| A scope with no stop leaves focus unset | `entering_a_scope_with_no_focus_stop_leaves_focus_unset` |
| Group + scope in one tree, end to end | `tab_order_covers_the_group_and_the_ungrouped_stops_on_the_real_tree` |

**The keys the runtime keeps** — each with both legs, because a key
silently consumed and a key correctly consumed are indistinguishable from
the consumption side alone

| Authored arm | Test that fires it |
|---|---|
| `arrow_direction`'s four mappings and its `None` | `left_is_prev`, `up_is_prev`, `right_is_next`, `down_is_next`, `a_non_arrow_key_is_not_an_arrow` |
| Arrows inside a group consume and move; arrows outside reach the host slot | `arrow_keys_two_legs` (both legs), `arrows_move_inside_the_group_and_group_memory_survives_a_visit_outside` |
| Escape with a scope entered consumes and delivers; with none, it reaches the host slot; with a scope but no handler, it still consumes and does not close | `escapes_two_legs` (three legs) |

`MovedActiveItem` is **not** a separately authored arm: the consumption
test is one condition over the outcome, so the variant no M4 widget can
produce does not become a branch no test can fire
(implementation-gates trap #4, applied in its inverse direction — the
same reading the T6 retrospective recorded).

[DD-V-029](../../../cross-milestone/decisions/dd-v-029-pure-logic-red-test-obligation.md)'s
**named** obligation is not triggered: this task authors no rounding, no
unit conversion and no boundary condition. The witnesses below are the
trap-#4 artifact instead.

#### #5 — Mutation witnesses

Eight, all applied by the task lead, each **read back from the file** to
confirm the mutation was present before the run, run, then reverted with
the revert confirmed by re-reading **and** by `git diff --stat` returning
empty. The full suite was re-run green afterwards. No failure was
re-rolled: the suite went red only where a mutation was deliberately in
place.

**Three of the eight restore the pre-T7 behaviour** rather than breaking
the new code (the T5 close-gate line, carried by T6), so they answer "do
these tests catch the absence of the feature", not only "do they watch
their own code".

| Witness | Mutation | Went red | Reading |
|---|---|---|---|
| **W1 — the restore branch deleted** (the spike's M7, named by [plan.md](./plan.md) §T7) | `sync_scopes_to_tree`'s exit step writes `None` instead of the captured `restore_to` | `exit_restores_and_restoration_beats_succession` and `a_thumbnail_click_enters_the_modal_scope_and_escape_restores_it`, and **nothing else in the crate** | Restoration is asserted, and by exactly the two fixtures that claim it |
| **W2 — the entry walk removed** (restoring) | the entry loop's range emptied, restoring pre-T7 behaviour | 5 tests: both entry fixtures, both exit fixtures, and the mechanism fixture's scope test | `a_present_but_unannotated_subtree_does_not_confine` and `arrow_keys_two_legs` stayed **green**, which is the discriminating half — they do not depend on entry, and a test suite that reddened everywhere would not have shown that |
| **W3 — the rebase made the identity mapping** (restoring) | `remap`'s mapping returns `Some(old_id)` rather than looking the anchor up | `a_retained_focus_record_survives_a_structural_removal_that_shifts_ids`, plus both exit fixtures | The second half is the interesting one: with ids assumed stable, a removed scope's id stays "mappable", the stack entry is never dropped, and the exit never happens. The exit path genuinely rests on the anchor mapping rather than merely coexisting with it |
| **W4 — the T5-era click landing restored** (restoring) | `focus_landing` replaced by `tab_stops` + a nearest-stop search | 3 `focus_landing` unit tests and `a_click_inside_a_focus_group_focuses_the_clicked_member` | CF-T6-5's fix is pinned at both levels, and the mechanism fixture's Tab/arrow test stayed green — the defect was click-only, as the carry-forward recorded |
| **W5 — arrows never consumed** | `arrow_on_key` returns `false` immediately | `arrow_keys_two_legs` and `arrows_move_inside_the_group_and_group_memory_survives_a_visit_outside` | |
| **W6 — Escape never consumed** | `dismiss_on_key` returns `false` immediately | `escapes_two_legs`, `exit_restores_and_restoration_beats_succession`, and the mechanism fixture's scope test | |
| **W7 — Escape consumed with no scope entered** | `dismiss_on_key` returns `true` when `esc_target` is `None` | `escapes_two_legs` **alone** | The agreement leg is what catches it. Without leg B, "Escape is consumed" would be satisfied by a runtime that swallowed every Escape |
| **W8 — entry made to enqueue drain work** | the entry loop marks the scope's own node layout-dirty | the `debug_assert_eq!` fires, with its own message, in every entry fixture | §13.4's invariant is asserted rather than documented, and the assertion is shown to be load-bearing |

Two further witnesses were run by the review remediation, each shown red
before being left green: the structural-succession branch short-circuited
(`structural_succession_lands_on_the_domains_first_surviving_stop` red,
the file's other fixtures green) and the exit step filtered to
`restore_to.is_some()` (`exit_with_no_restore_target_leaves_focus_unset`
red, the `restore_to == Some` fixture green).

#### #6 — Deterministic-failure disposition

**None arose.** Trap 6 was selected as *armed* rather than applying, and
it did not fire: no test failed except where a witness was deliberately in
place, and every witness failure disappeared on the confirmed revert.
There is no rerun history to record because there was no unexplained
failure to rerun.

#### #7 — GUI evidence

Script:
[capture-t7-scope-entry.ps1](./evidence/capture-t7-scope-entry.ps1).
Frames are not committed (the evidence directory holds scripts); the
numbers are recorded here so they survive the frames.

**What it discriminates.** Every fixture above reads entry back as
*state* — a path and a boolean. None can show that the indicator actually
*paints* on a node the same drain created moments earlier, because
`set_button_focused_at` starts a colour animation on a brush belonging to
a `Visual` built during that message, and `__button_focused_for_test`
reports the same boolean whatever colour the brush reaches. That is the
same limit T5's capture header records for the traversal case (CF-T5-3).

**The two frame sets** are two builds of the same tree. E carries
`modal-scope: true` and a `dismiss` handler on the gallery lightbox's
outer `ZStack`; U has both lines removed. Neither line adds a node, and
the emitted IR differs by exactly the four lines they produce — checked by
`wasamoc build` on both variants and diffing — so the comparison is about
the annotation and nothing else.

- **Difference leg**: the `<` Button, the scope's first stop, differs
  between E and U by **86.04 / 66.65 / 33.86 per channel** (max-abs 86.04)
  against a tolerance of 3.0 (the observed within-side jitter was 0; the
  floor of 3.0 is the applied tolerance).
- **Agreement leg**: the `>` Button, unfocused in both, differs by
  **exactly 0**. Without this leg, "E and U differ" would be satisfied by
  any global shift — a different window position, a different backdrop
  tint — and would say nothing about focus.

Display scale 1.25 (120 DPI), client rectangle 982x703, two frames per
side, masks derived once from a single reference frame so no frame that
shows the effect decides where the effect is measured. Preceded by
`cargo build --release --workspace` for each variant
([AGENTS.md §Build ordering](../../../../AGENTS.md), Phase 1 F-21).

**Read as images, not only as numbers**, by the capturing agent and again
by the task lead: in E the `<` Button is a warm amber rectangle and the
`>` Button is neutral grey; in U both are neutral grey, and the lightbox
is open identically in both. **A second discriminator was visible in the
same pair and was not designed in**: the toolbar's accent "Open lightbox"
Button — the widget the click focused — is bright accent blue in E and
carries the focus tint in U. That is both halves of the entry transition
in one frame pair, focus leaving the opener and arriving at the scope's
first stop, which the sampled regions alone could not show. The
restore-target capture is what E's frame shows the *other* side of.

`examples/gallery/gallery.ui` is a **throwaway probe**: T10, not T7, owns
landing the annotation in it. The operator writes the variant, builds,
captures, and reverts with `git checkout --`; `git status` was confirmed
clean for that path afterwards. The script's header labels which steps are
the operator's and which are its own.

This is the assistant baseline and does **not** replace the owner's
human-visible smoke ([CLAUDE.md §Testing rules](../../../../CLAUDE.md)).

#### #8 — Carry-forward

| Constraint | Evidence | Placement | Re-trigger criterion |
|---|---|---|---|
| **CF-T7-1 — an anchor is a node address, and an address can be reused.** A node freed by `widget_destroy` can have its address handed back to a later allocation, so an anchor naming a removed node can in principle match a *different*, newly built node. The consequence is bounded to "focus lands on an unexpected widget", never an unsound read, because nothing dereferences an anchor | `FocusProjection::id_of_anchor`'s doc comment states the bound; the narrow window is that every structural mutation rebases at the end of the same drain | `carry-forward` → this ledger, and `doc-folded` → `id_of_anchor` | **T9**, whose `for` regeneration frees and allocates subtrees in one drain — the nearest shape that could produce a reuse inside one window. This is the residual that remains *after* CF-T5-1's in-range case is closed, not a restatement of it |
| **CF-T7-2 — the four direct-ABI child mutators reach no seam, and `focused_path` cannot rebase.** `wasamo_widget_append_child` / `insert_child` / `remove_child` / `replace_child` mark nothing layout-dirty, so an edit through one of them runs no rebase; `focused_path` takes `&WindowFocus` and can only read. An in-range id can then resolve to a different node rather than to `None` | The seam enumeration at the start gate; `focused_path`'s and `id_of_anchor`'s doc comments | `carry-forward` → this ledger, and `doc-folded` → `focused_path` | Bounded today by `focused_path`'s only caller being the `__focus_path_for_test` seam and by a C-ABI-created node being unable to carry the annotation (`set_focus_annotation` has one caller, the loader). The re-trigger is the **first production reader of `focused_path`**, or any task that gives an ABI-created node a focus annotation. This is DD-M4-P2-004's own recorded residual ("a removal path that bypasses the structural seam"), now measured rather than predicted |
| **CF-T7-3 — nesting is supported and unexercised, and its multi-entry paths are pinned only by pure logic.** `DroppedScopes::outermost`'s selection with two-plus entries, and `sync_scopes_to_tree`'s ordering when a whole nest vanishes at once, have no integration fixture, because no M4 `.ui` builds a scope inside a scope | Three `DroppedScopes` unit tests and four `focus_core` nesting tests; the independent review confirmed no fixture reaches the multi-entry path | `carry-forward` → this ledger | **M4-Phase 9**, whose dialog-from-a-menu is the first nested case. DD-M4-P2-004 records nesting as "supported, unexercised in M4"; this row says *which* code that leaves unexercised, so Phase 9 checks it rather than discovering it |
| **CF-T7-4 — the arrow axis mapping is this implementation's choice.** `docs/dsl_spec.md` §4.19 says "arrow keys move focus within the group, wrapping at its ends" without fixing which axis maps to which direction. The landed mapping is Left / Up → previous, Right / Down → next, and both axes are accepted | `arrow_direction` and its five unit tests | `finding` → **T13's re-verification list** | **T13**, which re-verifies §4.19 against the landed runtime. Either the spec gains the sentence or the mapping is recorded as unspecified |
| **CF-T7-5 — a click outside an entered scope leaves focus unchanged, and the normative text does not say so.** §4.19 says a scope confines the keyboard and that clicks pass through a scrim-less scope; it does not say what such a click does to focus. `focus_landing` bounds its walk to `traversal_root`, so there is no candidate outside the scope and the click takes the same arm as a background click | `focus_landing_outside_an_entered_modal_scope_is_none`; decided at the start gate rather than during implementation | `finding` → **T13's re-verification list** | **T13**. The alternative reading — a click may move focus out of a scope — would make confinement pointer-breakable, which is why the landed answer is the one consistent with "no widget outside it can be reached by the keyboard" |

**Two functions of the spike core have no production caller, and that is
`doc-folded` rather than a row above.** `FocusState::exit_modal` is
unreachable because presence-driven exit means a present scope is always
entered, so the only exit is the subtree leaving, which the rebase
detects; `FocusTree::focus_after_removing` is unused because its
structural succession is the domain's first surviving stop, which
`initial_focus` produces from the post-mutation tree (start-gate fact 5).
Both follow from DD-M4-P2-004's presence-entry rather than from an
oversight, and the invariant that matters — **the modal stack has one
writer pair, `enter_modal` from the entry step and `remap` dropping the
entry on exit** — is stated where the code is, in `focus_core.rs`'s
`allow(dead_code)` comment and at `enter_modal`. Restating it here would
be the documentation analogue of implementation-gates trap #3: a second
source of truth in derived prose for something the owning file already
says.

#### Re-decided at close

The start gate selected traps 1, 2, 3, 4, 5 and 7, and armed 6.
**The selection survived unchanged**, and each call is confirmed by what
was built: the two uncompiler-enumerated migrations are real and their
audit table is above (1); the side-effect enumeration found the drain-work
question the ADR named and turned it into an assertion (2); three parallel
pairs were kept single-writer (3); every authored arm has a firing test,
after the review found three that did not (4); five carry-forwards are
recorded with re-triggers (5); and the GUI control was taken and read
(7). Trap 6 stayed armed and did not fire.

**One thing was built that the gate did not name**: `with_focus_write`.
The gate predicted two new writers of the focused id and recorded the
trap-#3 obligation, but not that satisfying it would mean *reducing*
`move_focus` to a wrapper over a new primitive. That is a strengthening
rather than a deviation — `set_button_focused_at` still has exactly one
caller — and it does not change the review lane.

The lane stays **full independent review**, as predicted at both gates,
and was executed as one.

#### Re-audit of the whole task list

Per [plan.md](./plan.md) §Cross-task obligations, the full list was
re-read at this close gate rather than only T7's item.

- **T8** — inherits the keys this task consumes, and they are now
  falsifiable rather than predicted: `arrow_keys_two_legs` and
  `escapes_two_legs` each assert both the consumption leg and the
  fallthrough leg through the host key slot, so a `key-down` dispatch
  inserted on the wrong side of that slot breaks a named test. That is
  CF-T5-5's tripwire, armed. Its CF-T6-3 work — the `Grid` / `ZStack`
  handler asymmetry — is untouched by this task and still open at T8's
  own start gate. Its `key-down("ArrowLeft")` acceptance test will run
  against a runtime that keeps arrows **only** inside a group, which is
  the behaviour its plan item names.
- **T9** — inherits CF-T7-1 directly: `for` regeneration is the nearest
  shape that frees and allocates subtrees inside one drain. It also
  inherits the seam unchanged — its `mutate_for_loop_subtree` already
  reaches `flush_layout` through `mark_layout_dirty_for`, so per-item
  handlers need no new seam work. The structural side-effect enumeration
  its plan item owes (what subtree removal releases) now has a sibling to
  cite: the focus record's entry is dropped by the rebase, not by a
  release path.
- **T10** — is the first `.ui` to carry the annotations, and the capture
  above shows exactly what it will look like when it does. What T10 must
  *not* inherit is the throwaway probe: `examples/gallery/gallery.ui` is
  reverted, and landing `modal-scope: true` plus the `dismiss` handler is
  T10's work, not a diff to recover. Its restore-target sentence is now
  measured rather than predicted — the frame pair shows the opener losing
  focus as the scope takes it, which is the behaviour §4.19's
  "restores to whatever the keyboard was on beforehand" describes.
- **T11** — touch inherits nothing from focus. Whether a touch contact
  moves focus is still that task's explicit decision (T5's re-audit line,
  unchanged): a `WM_POINTER*` arm would have to call `focus_on_click`
  itself.
- **T12** — control C (containment and occlusion) is now buildable: a
  scope exists, it confines Tab, and the agreement leg it needs — the
  same Tab reaching the background with the scope absent — is the same
  shape `a_present_but_unannotated_subtree_does_not_confine` already
  asserts at the state level. It also inherits CF-T5-3 unchanged, and
  gains a second capture script to copy mechanics from.
- **T13** — gains three re-verification items: CF-T7-4 (the arrow axis
  mapping §4.19 does not fix), CF-T7-5 (what a click outside a scope does
  to focus), and one more from the seam's shape — **§13.4's "a removal's
  successor is computed before the mutation"** describes a runtime that
  does the restore capture at *entry* (before, as required) and derives
  structural succession *after* the mutation, because the answer is the
  domain's first surviving stop either way (start-gate fact 5). The
  sentence is satisfied in substance and not in sequence; T13 decides
  whether the wording narrows.
- **Cross-task obligation "no new ABI function"** — held. No `extern "C"`
  function was added or changed; the surface shrank, if anything, because
  the `__focus_spike` module was deleted.
- **Cross-task obligation "every task that measures something re-reads
  the whole task list"** — discharged here.

#### Verification means

Run against the **final branch state**, after the review remediation
landed ([retrospectives.md](../../../procedures/retrospectives.md) item
3).

`cargo fmt --all -- --check` zero exit, `git diff --check` clean,
`cargo clean` (11,719 files / 3.1 GiB removed), then
`cargo build --release --workspace` and `cargo build --workspace` both
successful, and `cargo test --workspace --no-fail-fast` **47
binaries/sections, 1,145 passed, 0 failed, 0 ignored**. T6's baseline was
1,111 across 45 sections; the two new integration binaries account for the
two extra sections, and the 34 added tests are 25 in `focus_core.rs`, 8 in
`focus.rs`, and 13 integration fixtures, less the 4 deleted
(`nearest_focusable`'s three plus one, and the mechanism fixture's fourth
test), less the spike module's own.

**The new fixtures ran rather than skipped**, verified by running each
with `--nocapture` and confirming the shared guard's
`skipping …: runtime compositor unavailable` line does **not** appear.
`tests/common/mod.rs` was not touched, so the `0x80070005` two-conjunct
check ([constraints §8](../requirements/constraints.md)) is intact and the
standing obligation to verify a newly authored guard does not apply — no
guard was authored.

**What this task's evidence cannot show, stated rather than implied.**
Every integration fixture runs at 96 DPI and scale 1, so none of them
re-exercises the pointer conversion T2's non-unit-scale fixture owns;
traversal and scope entry are geometry-independent, and the click legs
inherit a conversion that fixture still pins. The GUI capture ran at 1.25
and shows the indicator, not the conversion. No fixture builds a scope
inside a scope, so nesting ships pinned by pure logic alone (CF-T7-3). And
the `dismiss` request's **host-listener** delivery — a
`wasamo_signal_connect` listener rather than an inline handler — has no
test of its own; the branch it would fire is the one `clicked`'s host
fixture already fires in `run_signal_handlers`, which is why this is
recorded as a residual rather than as a trap-#4 gap.

#### Independent review and its remediation (recorded 2026-08-08)

The review lane
([implementation-gates.md §4](../../../procedures/implementation-gates.md))
was executed as a **full independent review** by a second agent that did
not write the code, against `623c835` / `1365695` / `f934d05` / `66bcb37`
/ `88e97c8`. It built its own branch-coverage table from the diff rather
than from this close gate's, re-ran the seam enumeration, traced the
composition cases (a drain that closes one scope and opens another; a
removal that takes the focused node and an entered scope together), and
checked the "only caller" / "sole writer" / "one primitive" claims the
diff adds with `rg`.

It confirmed as sound: the seam enumeration and the four ABI mutators'
classification; that a failed insert or remove leaves the tree unmutated,
so the record cannot be left out of step by a partial edit; that a second
`set_root` resets the record before syncing; that five of the six
`FocusProjection::project` call sites rebase before reading `focus.core`
and the sixth's pre-rebase read is a coordinate-independent boolean; the
`WM_KEYDOWN` arm's order against §4.19's table; that the dismissal
refactor preserves `run_clicked_handlers`'s safety argument unweakened and
leaves `clicked`'s producer order unchanged; and that the deleted fourth
mechanism-fixture test's scope-side property is genuinely carried by
`a_present_but_unannotated_subtree_does_not_confine`.

Seven findings. Four were comments claiming something the code does not
do; three were authored branches no test fired. All seven were remediated
in `73853da`; none changed production behaviour.

- **F1 — `FocusState::remap`'s doc claimed a fallback the caller does not
  have.** It said the restoration a vanished scope owes is "`restore_to`,
  or the domain's first stop when even that is gone". The exit step writes
  `restore_to` as captured, so a captured `None` leaves focus unset —
  which is the spec's answer (§4.19 "remembers the focused widget",
  possibly nothing), not a gap. The domain's first stop is the separate
  structural-succession branch's answer. Comment corrected; the code was
  right.
- **F2 — the structural-succession branch was never fired through the
  real seam.** Every drain-pumping fixture either removed an unrelated
  subtree or was a scope open/close; the one T5-era test that removes the
  focused stop uses raw `SendMessageW` and never pumps the loop, so
  `flush_layout` never ran during it. Closed by
  `structural_succession_lands_on_the_domains_first_surviving_stop`,
  shown red against a short-circuit of the branch it fires.
- **F3 — `DroppedScopes::outermost` had never run with more than one
  entry.** No fixture can build the case (nesting is unexercised in M4),
  but the selection is pure logic; closed by three unit tests and
  recorded as CF-T7-3 for the seam-level half that remains unbuildable.
- **F4 — `WidgetNode::focus_role`'s doc comment had gone stale in three
  ways**, in a file this task's diff never touched: it counted two
  production callers where the projection now has six entry points; its
  reachability caveat described the present-but-un-entered scope and the
  click-lands-on-the-group-container defects as current when this task
  fixed both; and it called the combined group-and-scope question T7's
  when the owner had sent it to the candidate pool. **This is the finding
  worth carrying**: a doc comment written at T6 to warn about a defect
  becomes a false statement the moment the defect is fixed, and nothing in
  the toolchain notices. Rewritten to the landed state.
- **F5 — `focused_path`'s doc overclaimed.** It said every seam has
  already rebased before it reads; it cannot rebase, and the four ABI
  mutators reach no seam. Narrowed, with the bound stated, and recorded as
  CF-T7-2.
- **F6 — the `dismiss` host-listener path has no test.** Recorded as a
  residual in Verification means: the branch is `clicked`'s, already
  fired; only the `dismiss` instantiation is untested.
- **F7 — the capture script's header read as though the script performed
  the variant swap, the builds and the revert.** It performs none of them.
  Header rewritten to label operator steps and script steps, and the
  numbers moved into #7 above so they survive the frames.

The review also found **Fact 1's count off by one** (nine, not ten),
corrected above.

**External-agent (codex) review is not performed**, per the owner's
standing disposition for this phase (T1–T6, re-confirmed at the T6 merge
approval on 2026-08-07).

## T8 — DSL: generic `clicked` and `key-down("<key>")`

### Start gate (recorded 2026-08-08, before any source edit)

Read first: [AGENTS.md](../../../../AGENTS.md),
[implementation-gates.md](../../../procedures/implementation-gates.md),
[plan.md](./plan.md) §T8 and §Cross-task obligations,
[preamble.md](./preamble.md),
[DD-M4-P2-005](../decisions/dd-m4-p2-005-dsl-handler-surface.md),
[DD-M4-P2-001](../decisions/dd-m4-p2-001-event-routing-model.md),
[constraints.md](../requirements/constraints.md), the T3 / T6 / T7 close
gates above with their owner dispositions, and the T7 retrospective.

#### Normative statements that already answer this task (DD-V-031)

| Question | Where it is answered | What it fixes |
|---|---|---|
| Which widgets may carry `clicked` | [dsl_spec §4.19 §Click handling on any widget](../../../../docs/dsl_spec.md) and its §Attribute admission table | **Every** widget. Not a per-kind list — the table's row reads "`clicked` \| any widget" |
| Which widgets may carry `key-down` | [dsl_spec §4.19 §Attribute admission](../../../../docs/dsl_spec.md) | Any widget, same as `clicked` |
| How `key-down` is spelled | [dsl_spec §4.19 §Keyboard input](../../../../docs/dsl_spec.md), [DD-M4-P2-005 §K3](../decisions/dd-m4-p2-005-dsl-handler-surface.md) | The key is named **in the declaration**, as a string: `key-down("ArrowLeft") => { … }`. Not one signal per key, not a body-filtered callback, not a structured key value |
| Which key names are recognised | [dsl_spec §4.19 §Keyboard input](../../../../docs/dsl_spec.md) | Exactly `"Escape"`, `"ArrowLeft"`, `"ArrowRight"`, `"ArrowUp"`, `"ArrowDown"`, `"Home"`, `"End"`, `"PageUp"`, `"PageDown"`, `"Enter"`, `"F1"`…`"F12"` — 22 names. An unrecognised name is a `wasamoc check` diagnostic rather than a handler that silently never fires |
| Where a key event starts | [dsl_spec §4.19 §Click handling](../../../../docs/dsl_spec.md), [architecture.md §13.2](../../../../docs/architecture.md) | At the focused widget; when nothing is focused, at the innermost focus scope. Then the same ancestor walk as a click, first match consuming |
| What happens to an unconsumed key | [dsl_spec §4.19 §Which keys the runtime keeps](../../../../docs/dsl_spec.md), [architecture.md §13.2](../../../../docs/architecture.md) | It is **not** swallowed: it continues to the window's default handling. T5 landed that arm; this task must dispatch **ahead of** it without changing it |
| Which keys never reach an authored handler | [dsl_spec §4.19 §Which keys the runtime keeps](../../../../docs/dsl_spec.md) | `Tab` / `Shift+Tab` always; arrows **while focus is inside a `focus-group`**; `Escape` **while a modal scope is present**. All three landed at T5 / T7, so this task asserts against real behaviour rather than against a prediction |
| Whether `key-down` is a text path | [dsl_spec §4.19](../../../../docs/dsl_spec.md), [architecture.md §13.2](../../../../docs/architecture.md) | It is the **command** half. Text never travels it; an active IME composition owns the keyboard (M4-Phase 6 implements that); auto-repeat **is** delivered, which on Win32 needs no code — `WM_KEYDOWN` already repeats |
| What `dismiss` admission is | [dsl_spec §4.19 §Attribute admission](../../../../docs/dsl_spec.md) | A container carrying `modal-scope: true`. Landed at T6 on both gates; this task must not widen or narrow it |
| The disabled-Button contract | [dsl_spec §4.8](../../../../docs/dsl_spec.md), [§4.19](../../../../docs/dsl_spec.md) | `enabled: false` suppresses **click**-handler dispatch, still occludes, does not end propagation, and is not a focus stop. Stated over clicks; §4.8 says nothing about `key-down` |
| Whether a new ABI function is allowed | [constraints §2](../requirements/constraints.md), [framing agreement ⑦](../requirements/framing.md) | No. Widening the **vocabulary** of the existing `wasamo_signal_connect` path is explicitly the permitted form, and `key-down("<key>")` is a signal name on that path |

#### Three places where the normative text does **not** answer, recorded rather than resolved here

Per DD-V-031 these are divergences for the phase-close re-verification,
not questions this task settles by editing normative prose.

- **§3's grammar has no production for the argument.**
  `signal_handler ::= IDENT "=>" block` — unchanged by the Moment 1
  sync, while §4.19 shows `key-down("ArrowLeft") => { … }` and the
  CHANGELOG's 1.19 row calls the argument "the one new grammar
  production". §3's §Disambiguation table likewise has no `IDENT` `(`
  row. The production lands in the parser here; the **spec text** is a
  T13 item.
- **§8.8's IR grammar has no argument either.**
  `handler ::= "on" IDENT "{" expr "}"`. The IR must carry the key name
  in a form the loader maps without re-parsing (DD-M4-P2-005 §IR and
  compiler impact), so the emitted text form gains an argument. Same
  disposition: code here, spec wording at T13.
- **§4.5 still reads "The only recognized signal name is `clicked`."**
  §4.19 adds `dismiss` (landed T6) and `key-down` (landing here), so the
  sentence was already false before this task. T13.

#### Measured facts (probes run before choosing an approach)

**Fact 1 — `key-down(…)` is a parse error today, and needs no new
token.** `wasamoc check` on
`Button { text: "x"  key-down("ArrowLeft") => { } }` reports
``unexpected token `(` after identifier`` (`parser.rs`'s member
dispatch, which routes a leading `IDENT` on the second token only for
`:` / `{` / `=>`). `Token::LParen` / `Token::RParen` already exist in the
lexer and `key-down` already lexes as **one** `Ident` under §2.2's hyphen
rule, so the change is a member-dispatch arm plus an AST field — matching
the CHANGELOG's "No new token".

**Fact 2 — there is no signal-name validation anywhere.**
`totally_unknown_signal => { }` on a `Box`, and a bare `key-down => { }`
with no argument, are **both accepted** by `check` today (probe, exit 0,
no diagnostics). The only per-signal rules that exist are T6's `dismiss`
admission and `check_grid`'s blanket arm. So "an unrecognised key name is
a diagnostic" has no existing machinery to extend — the key-name check is
new code, and a bare `key-down` must be rejected too or it becomes a
second silently-never-fires spelling.

**Fact 3 — the Grid / ZStack asymmetry is exactly as CF-T6-3 recorded,
re-measured.** `Grid { clicked => { } }` is **rejected by `check`**
(`check_grid`'s `signal != "dismiss"` arm, `check.rs:1440`) and the
loader has **no** Grid handler gate (`rg` over `ir_loader.rs` finds
none). `ZStack { clicked => { } }` is **accepted by `check`** and
rejected by the loader
(`validate_phase6_zstack_node_invariants`, `ir_loader.rs:1197`,
`h.signal != "dismiss"`). Both are literally the trap-#1 shape — a
filter helper keyed on one signal name that silently drops every new one.

**Fact 4 — the childless-layout defect spans four widget kinds, not
one.** `build_layout_tree` (`widget.rs:2467`) maps `Rectangle`, `Text`,
`Button` **and** `ToggleButton` to a childless `LayoutNode::rectangle`.
A probe confirms `check` accepts a `WidgetNode` child on **all four**
(`Button { Text {} }`, `Text { Button {} }`, `Rectangle { Button {} }`,
`ToggleButton { Text {} }` — exit 0, no diagnostics). CF-1 and the
[candidate pool](../../../candidate-pool.md) row both name the
**Button family**, and the pool row spells it "a `Button` / `ToggleButton`
holds an authored subtree" — so this task rejects the shape on those two.
`Text` and `Rectangle` carry the same defect and no disposition; recorded
as a finding rather than folded in, because rejecting them would narrow
an authored surface no owner decision covers.

**Fact 5 — a `Button` literal `enabled` is dropped, and the fix is one
read plus its binding guard.** `ir_loader.rs`'s `"Button"` arm
(line 3838) reads `text` and `style` only; its `"ToggleButton"` sibling
(line 3849) reads `enabled` with a `has_binding` guard that defers to the
binding's initial run. `WidgetNode::button` hard-codes `enabled = true`
into `button_family`. Confirms CF-2 against the current tree.

**Fact 6 — Button keyboard activation does not exist in the runtime.**
`rg "VK_RETURN|VK_SPACE"` over `wasamo-runtime/src` returns **nothing**,
and `run_clicked_handlers` has exactly **one** caller,
`WidgetNode::hit_test_click`. §4.19 ("A Button additionally raises it
from keyboard activation") and §4.8 ("cannot be reached or activated from
the keyboard", said of a *disabled* one) both describe a behaviour the
runtime has never had, and plan §T8's "Button keeps … its keyboard
activation" presupposes it. **Not built here** — see §Boundaries below —
and recorded as a finding with an owner.

**Fact 7 — this task is CF-T7-2's stated re-trigger, and the shape that
avoids firing it already exists.** CF-T7-2's re-trigger is "the first
production reader of `focused_path`", and a key walk is exactly that
reader. But `focused_path` takes `&WindowFocus` and therefore *cannot*
rebase, which is the whole content of the residual. The three landed key
consumers — `traverse_on_key`, `arrow_on_key`, `dismiss_on_key` — each
`FocusProjection::project` + `WindowFocus::rebase` + read instead, and
the key walk takes that same shape. So `focused_path` keeps its single
`__focus_path_for_test` caller and the residual stays bounded — but its
doc comment asserts "**the** exposure is bounded by this function's only
caller", and the close gate re-reads it (the T7 retrospective's finding
(d): claims this diff invalidates live outside the diff).

**Fact 8 — the CF-T5-5 tripwire is real and armed.**
`modal_scope_integration.rs`'s `escapes_two_legs` (line 849) and
`arrow_keys_two_legs` (line 985) each install a host-key-slot recorder
and assert **both** the consumption leg and the fallthrough leg. A
`key-down` dispatch inserted on the wrong side of `state.key_down_fn`
(`window.rs:983`) breaks a named test rather than silently swallowing the
key.

**Fact 9 — the schema change is compile-error-forcing.** `IrHandler` has
exactly **four** sites (`wasamo-ir/src/lib.rs:194` the definition,
`ir_loader.rs:2847` the IR-text parser, `ir_loader.rs:5747` a test,
`lower.rs:171` the compiler). `Member::SignalHandler` has five match
sites in `check.rs` / `lower.rs` plus the parser's producer. Adding a
field to each makes Rust enumerate the breakage, which is the shape
[implementation-gates §2](../../../procedures/implementation-gates.md)
prefers for a semantic migration — with the wildcard / filter grep still
required, because fact 3's two helpers absorb new signal names without a
compile error.

#### What this task turns out to be

The plan's §T8 opening — "this widens *who may carry a handler* — a
checker rule over the existing handler table — not how an event travels"
— is **true of `clicked` and false of `key-down`**. `clicked` is what the
plan describes. `key-down` is a five-layer addition ending in the
runtime:

1. **Parser + AST** — the member-dispatch arm for `IDENT` `(` `STRING` `)`
   `=>`, and `Member::SignalHandler` gaining the argument (fact 1).
2. **Checker** — the recognised-key table, an unrecognised name, a bare
   `key-down`, and an argument on a signal that takes none (fact 2).
3. **IR** — `IrHandler` gains the argument; `emit` writes it; the loader's
   `parse_handler` reads it (fact 9, and DD-M4-P2-005's "the IR carries a
   key name that the loader can map without re-parsing").
4. **Loader** — the second gate on the key name, and attaching the handler
   under a canonical name.
5. **Runtime** — the key walk itself. T5 deliberately deferred it:
   "T5 lands the consumption half and the fallthrough half; **T8 lands the
   dispatch between them**." It is a new consumption arm in `WM_KEYDOWN`,
   between `dismiss_on_key` and `state.key_down_fn`, reusing
   `hit::dispatch_chain` and `run_signal_handlers` rather than adding a
   second dispatcher.

The `clicked` half resolves CF-T6-3's three questions as **one** answer
rather than three: `check_grid`'s blanket signal arm and the ZStack
loader's handler arm are both **removed**, so per-kind signal admission
ceases to exist and admission is by signal name alone — `dismiss` needs a
`modal-scope: true` sibling (T6, both gates), `key-down` needs a
recognised key (here, both gates), everything else is admitted on every
kind. That answers CF-T6-3's third question ("does the Grid rule gain the
loader half it never had?") with **no**: there is no per-kind handler rule
left for either gate to hold. Widening the two rules instead — teaching
each to admit `clicked` and `key-down` too — would keep two per-kind
allow-lists that the *next* signal name has to be added to twice, which is
the drift CF-T6-3 exists to record.

Two consequences are named rather than left to be found:

- **`Grid { totally_unknown => { } }` becomes accepted**, because every
  other widget kind already accepts it (fact 2). That is a widening, and
  it is the *existing* behaviour of ten of the eleven kinds rather than a
  new permission. The uniform question — whether an unrecognised signal
  name should be a diagnostic anywhere — is a finding, not this task's.
- **`ZStack { clicked => { } }` becomes accepted at the loader**, which is
  what §4.19's admission table requires;
  `zstack_clicked_handler_still_rejected_after_relaxation` and
  `non_dismiss_handler_on_grid_still_rejected` are the two T6 tests that
  pinned the old bound, and they are replaced by tests pinning the new one.

#### Selected traps

```
- [x] #1 semantic migration   - [x] #2 side effects   - [x] #3 parallel data   - [x] #4 branch tests
- [x] #5 carry-forward        - [ ] #6 root cause (armed)  - [ ] #7 GUI positive control
```

- **#1 — applies.** Two schema types gain a field (`Member::SignalHandler`,
  `IrHandler`), and every reader of a handler's signal name must be
  classified. The two filter helpers of fact 3 are the exact failure this
  trap describes, so the audit table covers `rg` over `\.signal`,
  `signal ==`, `signal !=`, `"clicked"`, `"dismiss"` and
  `inline_handlers` in addition to the compiler's own enumeration.
- **#2 — applies.** A new consumption arm changes the `WM_KEYDOWN`
  ordering contract, and a `key-down` handler's state write drains
  synchronously exactly as a `clicked` handler's does. The enumeration
  states what a key dispatch pulls in (drain → re-layout → rectangle store
  → focus rebase) and what the walk must not observe mid-flight, reusing
  `hit_test_click`'s snapshot-then-run argument rather than restating it.
- **#3 — applies.** The 22 recognised key names would otherwise exist
  three times: the spec table, the checker's, and the runtime's virtual-key
  map. A name accepted by `check` with no virtual-key mapping is a handler
  that silently never fires — this phase's signature failure. One owner for
  the name list, in the crate both sides already depend on
  (`wasamo-ir`), plus a runtime test asserting every recognised name maps
  to a virtual key. The handler's canonical storage spelling is the second
  derived pair: the loader writes it and the dispatcher looks it up, so
  one shared function produces it.
- **#4 — applies**, and is the plan's stated artifact for this task. Every
  new reject arm gets a test that fires it directly, on **both** gates
  where the rule is two-gated.
- **#5 — applies.** CF-T7-2's re-trigger fires (fact 7), and the findings
  of facts 4 and 6 need owners.
- **#6 — armed, not selected.** No failure is in hand.
- **#7 — does not apply.** This task paints nothing. Its deliverables are
  diagnostics and a key-dispatch arm; the gallery carries no `key-down`
  handler and no ancestor handler until T10, so a frame captured now would
  be produced identically by the pre- and post-T8 runtime — a
  non-discriminating frame, which is what the trap exists to forbid. The
  evidence that discriminates is state read back through real window
  messages, which is what the fixtures do. Same reading T3 recorded and
  T12 executes.

#### Review lane

**Corrected at this start gate, from [preamble.md](./preamble.md)'s
predicted "Branch/test-focused review" to a full independent review** —
the Phase 1 F-12 / T12 precedent the preamble itself provides for. The
prediction stands on "checker widening plus one grammar production; the
reject tests are the artifact", and two of the five layers above are
neither:

- **Runtime structural change** — a new consumption arm inside
  `WM_KEYDOWN`'s ordering, and the runtime's **second** handler-dispatch
  walk beside `hit_test_click`'s.
- **Schema / IR migration** — `IrHandler` and the IR text grammar gain the
  argument, which is
  [implementation-gates §4](../../../procedures/implementation-gates.md)'s
  first named high-risk class.

The lanes compose: the full review **includes** the trap-#4
branch/test-focused check, which is the larger half of this task by count.

#### Boundaries this task does not cross

1. **No handler inside a `for` body.** Both gates keep rejecting it; T9
   owns admission, binder reads, lifecycle and identity together
   (DD-M4-P2-005).
2. **No change to `examples/gallery/gallery.ui` or any other shipped
   `.ui`.** Landing the first authored `key-down` and the first
   annotation is T10's.
3. **No GUI capture set.** T12's, per trap #7 above.
4. **No spec re-verification, no phase-status flip, and no new normative
   prose** beyond §4.16's example correction, which plan §T8 and the
   2026-08-07 owner disposition assign here explicitly. The three gaps
   above go to T13.
5. **Button does not become a layout container.** The reject narrows the
   authored surface deliberately; the withheld capability is
   [candidate pool](../../../candidate-pool.md) row "Button-family content
   children", already recorded at T3.
6. **Button keyboard activation is not built** (fact 6). It is in no
   sub-issue of DD-M4-P2-005, in no deliverable of
   [preamble.md](./preamble.md) §Phase scope, and building it would put
   `Enter` (and `Space`) into §4.19's "keys the runtime keeps" table,
   which that table does not list — so an authored `key-down("Enter")`
   would silently never fire while a Button is focused, the precise
   failure mode this surface exists to prevent. Deciding it is an owner
   disposition of the same kind CF-1 / CF-2 received; not building it
   leaves the additive direction open, building it does not.
7. **No global unknown-signal-name diagnostic.** §4.5's stale sentence
   would arguably license one, but it narrows a surface every widget kind
   accepts today (fact 2) and no decision in the set asks for it. Finding.
8. **No new ABI function**, and none is needed:
   `wasamo_signal_connect` already takes an arbitrary signal name, so a
   host listener on `key-down("ArrowLeft")` is the vocabulary widening
   [constraints §2](../requirements/constraints.md) permits.

#### The T7 correctives, applied

The T7 retrospective added one line to each gate. Both are answered here
rather than at close.

- **Start gate: "which of the carry-forwards this task closes are
  `doc-folded`, and is rewriting that prose a work item?"** This task
  closes **CF-T6-3**, whose placement is `finding` → T8 with no doc fold,
  and it *touches* **CF-T7-2**, which is `doc-folded` → `focused_path`'s
  doc comment. CF-T7-2 is not closed here (fact 7 keeps the residual
  bounded rather than removing it), but its doc comment's claim about
  "this function's only caller" is re-read at close as a work item. Two
  further prose sites are on the list for the same reason: `check_grid`'s
  signal arm carries a comment saying widening `clicked` "is out of scope
  for this task (T8)", and `ir_loader.rs`'s ZStack handler comment names
  the same bound — both become false statements the moment the rules are
  removed.
- **Close gate: "does each branch-table row name the production path the
  test reaches the branch through?"** Recorded as an obligation now so the
  close gate is written against it rather than reconstructed. For this
  task the distinction bites in one place: a `key-down` handler whose body
  writes state needs the message loop pumped (`key_and_drain`) for the
  drain's Phase 2 to run, exactly as `escapes_two_legs`'s leg A does —
  `SendMessageW` alone reaches the dispatch but not the re-layout.

### Close gate (recorded 2026-08-08)

Commits:

| commit | content |
|---|---|
| `36829ea` | start gate (normative-answer table, three normative gaps, nine measured facts, trap selection, corrected review lane, boundaries) |
| `ef56c68` | `clicked` on every widget — both per-kind blanket handler rules removed, both gates' tests re-pointed |
| `3d3785e` | the two Button-family defects (CF-1 / CF-2) and §4.16's example |
| `bc75589` | the `key-down("<key>")` compiler surface (grammar, AST, checker, IR, emit, loader parse + second gate) |
| `9c378e6` | the `key-down` runtime dispatch (`key_name_for_vk`, `key_down_on_key`, `deliver_key_down`, the `WM_KEYDOWN` arm, seven fixtures) |
| `7738882` | the `clicked`-widening bound — a disabled Button occluding a lower sibling |
| `d7c6147` | three claims outside the diff that the key walk made incomplete |

#### #1 — Call-site audit table

Two schema types gained a field. Neither derives `Default` and neither
construction site used `..`, so **the compiler enumerated the migration**;
the greps below are for the shapes a compiler cannot enumerate — filter
helpers keyed on a signal name, which absorb a new one silently.

**Compiler-forced sites** (every one of them changed):

| Site | Type | Classification |
|---|---|---|
| `wasamo-ir/src/lib.rs:194` | `IrHandler` definition | field added |
| `wasamoc/src/lower.rs:170` | `Member::SignalHandler` destructure → `IrHandler` construction | must-carry; `arg` threaded |
| `wasamoc/src/parser.rs:634` | `Member::SignalHandler` construction | must-produce; the new production writes it |
| `wasamoc/src/check.rs:2470` | `Member::SignalHandler` destructure (no `..`) | must-dispatch; the three new rules |
| `wasamo-runtime/src/ir_loader.rs:2943` | `parse_handler` → `IrHandler` construction | must-parse; optional `( STRING )` |
| `wasamo-runtime/src/ir_loader.rs:~5890` | hand-built `IrHandler` literal in the round-trip test | test fixture; `arg: None` |

**Sites the compiler did *not* force**, found by
`rg '\.signal\b|signal ==|signal !=|"clicked"|"dismiss"|"key-down"|inline_handlers|Member::SignalHandler'`
over `wasamoc/src`, `wasamo-runtime/src`, `wasamo-ir/src`, `wasamo-dll/src`,
`bindings`, `examples`:

| Site | What it does | Classification and reason |
|---|---|---|
| `check.rs:1440` (pre-T8) `signal != "dismiss"` in `check_grid` | rejected every non-`dismiss` handler on a `Grid` | **must-change** — the trap's own shape: a filter keyed on one name silently drops every new one. Removed (`ef56c68`) |
| `ir_loader.rs:1197` (pre-T8) `h.signal != "dismiss"` in `validate_phase6_zstack_node_invariants` | same, on the loader side | **must-change**. Removed (`ef56c68`) |
| `ir_loader.rs:1370` `h.signal == "dismiss"` | T6's `dismiss` admission gate | ignore-OK — keyed on `dismiss` **positively**, so a new signal name is not affected. Pinned by the two `dismiss` tests re-run on both gates |
| `ir_loader.rs:1428` `handler.signal == "key-down"` | T8's own second gate | new; its three arms each have a firing test (#5) |
| `ir_loader.rs:1727` `handler.signal` in a diagnostic | formats the name into a message | ignore-OK — display only. The message omits `arg`, which is a legibility residual rather than a correctness one (#8) |
| `check.rs:2489` `signal == "dismiss"` | T6's checker admission gate | ignore-OK, same reason as `ir_loader.rs:1370` |
| `check.rs:1330`, `check.rs:3262` `Member::SignalHandler { span, .. }` | the `if` / `for` body structural rejections | ignore-OK — they reject *any* handler in that position regardless of name, so a new signal cannot slip past |
| `check.rs:1495` `Member::SignalHandler { .. }` | `check_grid`'s now-inert arm | ignore-OK by construction — this is the removed rule's replacement |
| `lower.rs:53` `Member::SignalHandler { .. }` | the state/property collection pass | ignore-OK — handlers contribute no state |
| `widget.rs:1610` `set_inline_handler` | the one write of the storage key | must-change: the loader now passes `signal_key`'s composed form. Documented on the function (`d7c6147`) |
| `widget.rs:2957` `inline_handlers` filter in `signal_handlers_for` | the one read of the storage key | ignore-OK — it compares whole names, and both sides compose through `signal_key` (#4) |
| `widget.rs:1733`, `widget.rs:2907`, `widget.rs:3059` — `"dismiss"` / `"clicked"` literals at dispatch | the two pre-T8 dispatch sites | ignore-OK — `signal_key(name, None) == name`, so their keys are byte-identical to before |
| `abi.rs:1149` `let name = b"clicked"` | `wasamo_button_set_clicked`'s wrapper over `wasamo_signal_connect` | ignore-OK — a no-argument signal; the host path already admits any name, which is why no ABI function was needed |
| `wasamoc/src/emit.rs:240/245` | IR text emission | must-change; the `Some` arm added, the `None` arm byte-identical (pinned by `no_argument_handler_still_emits_unchanged_form`) |

**Tests deliberately not added**: none. Every must-change site above has a
firing test in #5.

#### #2 — Structural side-effect enumeration

The new `WM_KEYDOWN` arm is the structural change. What a key dispatch
pulls in, and what the walk must not observe mid-flight:

| Derived effect | How it is handled |
|---|---|
| **The dispatch chain across a handler's synchronous rebuild** | `deliver_key_down` resolves every chain node to a raw pointer **before any handler runs**, and returns as soon as one runs — so no ancestor is ever visited after user code has run. This is `hit_test_click`'s argument, inherited rather than restated (its doc comment says so explicitly) |
| **Inline bodies vs. the node they live on** | `signal_handlers_for` clones every inline body out of the node before any of them runs; `run_signal_handlers`'s host-enqueue step compares `widget_ptr` and never dereferences it. Both properties are the ones `clicked` and `dismiss` already rest on |
| **Reactive drain** | A handler's `Signal::set` drains its effects synchronously, exactly as a `clicked` handler's does. `key_down_on_key` writes no focus state and enqueues nothing of its own |
| **Layout invalidation → drain Phase 2** | A structural mutation inside the handler reaches `emit::flush_layout` at the message-loop boundary, the same seam a click's does. Fixture 7 asserts it by reading a `__arranged_rect_for_test()` that only exists once Phase 2 has run, and needs `key_and_drain` (not `SendMessageW`) to observe it |
| **Focus record vs. the mutation** | `flush_layout` runs `focus::sync_scopes_to_tree`, which rebases. `key_down_on_key` deliberately performs **no** focus write of its own — the same division `dismiss_on_key` records |
| **The retained focus id's coordinate system** | `key_down_on_key` projects and rebases **before** reading `focus.core`, the same first step the other three `*_on_key` functions take. It does **not** call `focused_path`, which cannot rebase — see #8 |
| **`WM_KEYDOWN`'s return path** | Unchanged for an unconsumed key: no `return`, so control still reaches `DefWindowProcW`. The consuming arm returns `LRESULT(0)` like its three siblings |
| **The host key slot** | Now downstream of one more consumer. Pinned in both directions by `the_authored_key_down_walk_consumes_ahead_of_the_host_key_slot` |

The Button-family child rejection has a structural side effect of its own:
it removes a shape `sync_visuals`' child-count assertion used to abort on.
That assertion is **kept** — it is still the tripwire for the direct C
path, which stays ungated by design.

#### #3 — Every `SetOffset` / `SetSize` in the runtime, with its pass

Carried from T5 as the standing artifact of any task that could add a
geometry write. `rg 'SetOffset|SetSize'` over `wasamo-runtime/src`,
`wasamo-dll/src`, `bindings`, `examples` returns **six**, all inside
`sync_visuals` (`widget.rs:2710 / 2715 / 2759 / 2765 / 2795 / 2801`) —
unchanged from T5, T6 and T7. This task creates no `Visual` and writes no
Composition geometry.

#### #4 — Parallel-data sync

Two derived pairs, each given one writer.

| Pair | Risk | How it is closed |
|---|---|---|
| **The 22 recognised key names**: the spec table, the checker's admission rule, and the runtime's virtual-key map | A name the checker accepts with no virtual-key mapping is a handler that silently never fires — this phase's signature failure | One owner, `wasamo_ir::RECOGNISED_KEY_NAMES`, in the crate `wasamoc` and `wasamo-runtime` both already depend on. The checker calls `is_recognised_key_name`; there is no second list. The runtime's `key_name_for_vk` is the one place a name still appears twice, and `key_name_for_vk_produces_every_recognised_key_name` sweeps every `u16` and asserts the resulting set covers the shared table in full — mutation-verified (#6, W11) |
| **The handler's storage spelling**: written by the loader at attachment, read by the dispatcher at lookup | Two composers could disagree on the exact string and the handler would never be found | One function, `wasamo_ir::signal_key`. `rg` for `format!("{}(\"` and for `key-down(` over `wasamo-runtime/src` finds it composed nowhere else. `signal_key(name, None) == name`, so `clicked` / `dismiss` storage is byte-identical to pre-T8 |

#### #5 — Branch tests, each fired directly

Each row names the production path the test reaches the branch through
(the T7 retrospective's close-gate corrective — "a fixture exists" is not
"the fixture fires this branch").

| Branch | Test | Production path it reaches the branch through |
|---|---|---|
| `check_grid` no longer rejects a handler | `clicked_handler_on_grid_accepted` | `check_members_inner` → `check_grid` over a Grid's own members |
| the generic `dismiss` rule now owns Grid | `dismiss_handler_on_grid_without_modal_scope_rejected_by_generic_rule` | `check_members_inner`'s `SignalHandler` arm; asserts the message is the generic one and **not** the removed Grid-specific one |
| the ZStack loader gate no longer rejects a handler | `zstack_clicked_handler_validates`, `grid_clicked_handler_validates` | `validate` → `validate_phase6_zstack_node_invariants` |
| the generic `dismiss` rule still owns ZStack | `dismiss_handler_on_zstack_without_modal_scope_prop_rejected`, `dismiss_handler_accepted_on_zstack_carrying_modal_scope` | `validate` → `validate_focus_annotation_invariants` |
| Button-family child, checker | `button_with_widget_child_rejected`, `togglebutton_with_widget_child_rejected` | `check_members_inner`'s `WidgetDecl` arm → `check_button_family_children` |
| …counting conditional / `for` members too | `button_with_conditional_member_rejected`, `button_with_for_member_rejected` | same; measured first that neither `check_if_body` nor `check_for_member` already rejected those positions inside a Button |
| …without rejecting a legitimate Button | `button_with_only_admitted_members_accepted` | same arm, the control leg |
| Button-family child, loader | `validate_rejects_button_with_widget_child`, `validate_rejects_togglebutton_with_widget_child`, `childless_button_is_valid` | `validate` → `validate_phase2_node_invariants` |
| literal `enabled` on `Button` | `literal_enabled_false_on_plain_button_is_respected`, `button_with_no_enabled_prop_constructs_enabled` | `.ui` → `wasamoc` → IR → `construct_widget`'s `"Button"` arm, read back through `__button_enabled_for_test` |
| the `(` member-dispatch route | `key_down_handler_parses_with_string_argument`, `signal_handler_no_arg_still_parses_with_arg_none` | `parse_member` → `parse_signal_handler` |
| the four parser rejects | `signal_handler_arg_empty_parens_rejected`, `_bare_ident_rejected`, `_unclosed_paren_rejected`, `_missing_arrow_rejected` (plus `_interpolated_string_rejected`) | `parse_signal_handler` → `parse_signal_handler_arg` / `expect_rparen` |
| the Grid track-list stop set | `grid_track_list_terminates_before_a_parenthesised_signal_handler` + `grid_track_list_still_absorbs_a_bare_word_track` | `parse_grid_track_list`'s word-continuation lookahead — pinned at the sub-parser's own layer, not only through the checker test that first caught it |
| bare `key-down`, checker | `key_down_without_argument_rejected` | `check_members_inner`'s `SignalHandler` arm |
| unrecognised key, checker | `key_down_unrecognised_key_name_rejected`, `key_down_modifier_combo_rejected_as_unrecognised` | same arm → `wasamo_ir::is_recognised_key_name` |
| argument on a signal that takes none | `argument_on_clicked_rejected`, `argument_on_dismiss_rejected` | same arm's `else if arg.is_some()` |
| the accept side, on three kinds | `key_down_accepted_on_box` / `_on_button` / `_on_grid`, `key_down_recognised_key_names_spot_check_accepted` | same arm, falling through every reject |
| the same three rules, loader | `key_down_without_argument_rejected_at_validate`, `key_down_unrecognised_key_name_rejected_at_validate`, `argument_on_clicked_rejected_at_validate` | `validate` → `validate_key_down_invariants` |
| the IR argument round-trip | `key_down_handler_argument_emitted_in_parenthesised_form`, `no_argument_handler_still_emits_unchanged_form`, `key_down_handler_arg_survives_lowering`, `key_down_handler_surface_emits_parenthesised_argument`, `key_down_handler_parses_with_string_argument` (loader), `clicked_handler_still_parses_with_arg_none` | `lower` → `emit` → the loader's `parse_handler` |
| the vk map's completeness | `key_name_for_vk_produces_every_recognised_key_name`, `every_name_key_name_for_vk_can_produce_is_recognised`, `vk_tab_is_not_authorable_as_a_key_down_name` | `key_name_for_vk` directly |
| the walk fires at all, from the `traversal_root` start | `key_down_fires_on_the_root_box_via_the_traversal_root_start` | real `WM_KEYDOWN` → `key_down_on_key` → `deliver_key_down`; asserts `__focus_path_for_test == None` first, so the start it took is stated |
| the ancestor walk | `key_down_reaches_an_ancestor_containers_handler` | same, with focus established by a real Tab |
| first match consumes | `key_down_first_match_on_the_focused_node_consumes_before_any_ancestor` | same |
| arrows, both legs | `key_down_arrow_two_legs_inside_and_outside_a_focus_group` | `arrow_on_key` consuming ahead of `key_down_on_key`; the second leg asserts the group movement actually happened |
| Escape, both legs | `key_down_escape_two_legs_with_and_without_an_entered_modal_scope` | `dismiss_on_key` consuming ahead; needs `key_and_drain` for the scope's removal and restoration |
| the host key slot, both legs | `the_authored_key_down_walk_consumes_ahead_of_the_host_key_slot` | the `WM_KEYDOWN` arm's ordering itself — **the one branch every other fixture leaves unpinned** (#6, W12) |
| the drain path | `a_key_down_handlers_state_write_re_lays_out_through_the_same_drain_clicks_use` | `key_and_drain` → `emit::flush_layout`; asserts a rectangle only Phase 2 writes |
| the `clicked` widening's occlusion bound | `a_disabled_button_occludes_a_lower_sibling_it_paints_over` | `hit_test_click` → `resolve_topmost`; two legs, so "the counter stayed zero" cannot be satisfied by a Box that never receives clicks |

**No branch was added without a firing test**, and one branch was
*declined* for that reason: `deliver_key_down` has **no `enabled`
suppression arm**. §4.8's disabled contract is written over clicks, and
the case is unreachable from any authored tree — a disabled Button is
excluded from `collect_stops` / `focus_landing` so it can never be the
focused start, and after this task a Button can carry no children so it
can never be a non-start node on a chain. The reasoning is on the
function rather than in a branch no test could fire.

#### #6 — Mutation witnesses

Every mutation was applied by the lead, confirmed present by re-reading
the file, run, then reverted and the revert confirmed by re-reading and
`git diff`.

| Witness | Mutation | Went red | Reading |
|---|---|---|---|
| **W1** | `construct_widget`'s `"Button"` arm calls `WidgetNode::button` again, dropping `initial_enabled` | `literal_enabled_false_on_plain_button_is_respected` **only** | The CF-2 fix is what the test measures; the no-prop control stays green, so the test cannot pass by always-disabling |
| **W2** | the `check_button_family_children` call guarded off | all four Button-child rejects; the control stays green | The checker rule is load-bearing for all four shapes |
| **W3** | the loader's Button-family child rule guarded off | `validate_rejects_button_with_widget_child`, `..._togglebutton_...` | The second gate is independently pinned |
| **W4** | the rule counts `WidgetDecl` only, dropping `Conditional` / `For` | exactly `button_with_conditional_member_rejected` and `button_with_for_member_rejected` | The completeness half is real coverage, not incidental |
| **W5** | `Token::LParen` removed from `parse_grid_track_list`'s stop set | `grid_track_list_terminates_before_a_parenthesised_signal_handler` **and** `key_down_accepted_on_grid` | The shared sub-parser's change is pinned at its own layer as well as through the checker |
| **W6** | `is_recognised_key_name` returns `true` unconditionally | the `wasamo-ir` reject test, both checker unrecognised tests, and the loader's | One shared table, three consumers, all pinned |
| **W7 / W8** | the checker's `key-down` block and its `arg.is_some()` sibling both guarded off | all five checker key-down rejects | Each of the three rules is separately load-bearing |
| **W10** | `validate_key_down_invariants` no longer called from `validate` | all three loader key-down rejects | The second gate is not incidental |
| **W11** | the `VK_F12` arm removed from `key_name_for_vk` | `key_name_for_vk_produces_every_recognised_key_name` | The anti-drift test detects a table/map divergence, which is the whole reason it exists |
| **W12** | the consuming `return LRESULT(0)` after `key_down_on_key` removed | **nothing — the entire suite stayed green** | **This is the witness that changed the task.** Every other fixture reads handler *effects*, so the arm's position relative to the host key slot was unpinned in one direction. `the_authored_key_down_walk_consumes_ahead_of_the_host_key_slot` was written in response, and the mutation re-run against it goes red |
| **W14** | the `traversal_root` fallback dropped, so nothing focused means no dispatch | `key_down_fires_on_the_root_box_via_the_traversal_root_start` | The start-node reading of §4.19 is asserted, not assumed |
| **W15a** | `deliver_key_down` walks only the start node, not `dispatch_chain` | `key_down_reaches_an_ancestor_containers_handler` and the host-slot fixture | The ancestor walk is real |
| **W15b** | `deliver_key_down` keeps walking after a handler runs | `key_down_first_match_on_the_focused_node_consumes_before_any_ancestor` and the host-slot fixture | First-match consumption is real |

W12 is recorded in full because it is a **method failure caught by the
method**: the close gate's own branch table would have listed seven key
fixtures against an arm none of them constrained.

#### #7 — Deterministic-failure disposition

Trap 6 stayed armed and did not fire. No test failed non-deterministically
at any point; the only red runs were the mutation witnesses above, each
deliberate, each reverted.

One **deterministic** failure occurred during implementation and was root-
caused rather than worked around: adding the `(` route to member dispatch
broke `key_down_accepted_on_grid` with ``expected member, found `(` ``. The
cause was `parse_grid_track_list`'s word-continuation lookahead treating
`key-down(` as a trailing track word — a shared sub-parser whose stop set
did not know about the new production. Fixed at the stop set, and pinned
with both legs (W5) rather than only through the checker test that
surfaced it.

#### #8 — Carry-forward

| Constraint | Evidence | Placement | Re-trigger criterion |
|---|---|---|---|
| **CF-T8-1 — Button keyboard activation does not exist, and two normative sentences say it does.** `rg "VK_RETURN\|VK_SPACE"` over `wasamo-runtime/src` returns nothing and `run_clicked_handlers` has exactly one caller, `hit_test_click`. §4.19 ("A Button additionally raises it from keyboard activation") and §4.8 ("cannot be reached or activated from the keyboard", of a disabled one) both describe behaviour the runtime has never had, and plan §T8's "Button keeps … its keyboard activation" presupposes it | Start gate fact 6 | `finding` → **owner**, then T13 | Building it is not free: `Enter` and `Space` would join §4.19's "keys the runtime keeps" table, which does not list them, so an authored `key-down("Enter")` would silently never fire while a Button is focused. That is a surface decision of the same kind CF-1 / CF-2 received, not an implementation detail — so it is an owner disposition. If the answer is "the spec wording narrows", T13 records it |
| **CF-T8-2 — `Text` and `Rectangle` carry the identical childless-layout defect, and are deliberately not rejected.** `build_layout_tree` maps all four of `Rectangle` / `Text` / `Button` / `ToggleButton` to a childless `LayoutNode::rectangle`; `check` accepts a widget child on all four | Start gate fact 4 (probe, exit 0 on all four shapes) | `finding` → **owner**, then whichever task narrows | The owner's 2026-08-07 disposition and the [candidate pool](../../../candidate-pool.md) row both name the **Button family** ("a `Button` / `ToggleButton` holds an authored subtree"), so extending the reject to `Text` / `Rectangle` would narrow an authored surface no decision covers. Re-trigger: the first `.ui` found in the wild carrying such a child, or M5's widget set deciding what a content-holding control is |
| **CF-T8-3 — three normative gaps around the new production.** §3's grammar is still `signal_handler ::= IDENT "=>" block` and its §Disambiguation table has no `IDENT` `(` row; §8.8's IR grammar is still `handler ::= "on" IDENT "{" expr "}"`; §4.5 still reads "The only recognized signal name is `clicked`" (already false after T6's `dismiss`) | Start gate §Three places where the normative text does not answer | `finding` → **T13** | T13 owns the Moment 2 re-verification. The code landed here is what the wording must be checked against; writing normative prose was outside this task's boundary |
| **CF-T8-4 — an unrecognised *signal* name is accepted on every widget kind and silently never fires.** `Box { totally_unknown_signal => { } }` passes `check` and the loader. `Grid` was the one kind that rejected it, and this task removed that rule in the direction of uniformity | Start gate fact 2 (probe, exit 0) | `finding` → **T13 or a later phase** | The gap is pre-existing and is now uniform rather than one-kind-exceptional. Adding a diagnostic narrows a surface every kind accepts today, which no decision in the set asks for. Re-trigger: any phase adding a fourth signal name, or the first report of a handler that never fired |
| **CF-T8-5 — the key walk goes upward only, so a `key-down` handler below the walk's start can never fire.** With nothing focused the start is `traversal_root` (the tree root when no scope is entered), and `dispatch_chain` yields the start and its **prefixes** — never descendants. A handler on a non-root widget therefore needs focus at or below it | `key_down_fires_on_the_root_box_via_the_traversal_root_start`, whose fixture UI comment records the constraint; `hit::dispatch_chain`'s own doc comment | `carry-forward` → this ledger, and `doc-folded` → the fixture's `ROOT_BOX_KEY_DOWN_UI` comment and `deliver_key_down`'s doc | **T10.** The gallery's Left/Right handlers sit on the lightbox's `modal-scope` container, and entry moves focus to the scope's first stop — a descendant — so the walk reaches them. That is a property of entry, not an accident, and T10 must not move those handlers below whatever the scope focuses |
| **CF-T8-6 — the `key-down` host-listener path has no test of its own.** A `wasamo_signal_connect("key-down(\"ArrowLeft\")")` listener would fire the `has_host_listener` branch of `deliver_key_down`'s emptiness check and `run_signal_handlers`'s enqueue step | The branch is the one `clicked`'s host fixture (`a_host_signal_listener_on_a_non_button_widget_consumes_the_walk_until_disconnected`) already fires; only the `key-down` instantiation is untested | `carry-forward` → this ledger | Exactly the position T7 recorded for `dismiss`'s host delivery, and the same reasoning: the branch is fired, the instantiation is not. Re-trigger: M4-Phase 7, the milestone's ABI phase, or any host that connects a key signal |
| **CF-T7-2 is touched and not closed, and its prose is still true.** The key walk is CF-T7-2's stated re-trigger ("the first production reader of `focused_path`"), and the shape that avoids firing it was taken deliberately: `key_down_on_key` projects and rebases like the other three `*_on_key` functions instead of calling `focused_path`, which takes `&WindowFocus` and cannot rebase | `focused_path`'s doc comment re-read at this close gate — its claim that "the exposure is bounded by this function's only caller, `ffi::__focus_path_for_test`" is **still accurate**, verified by `rg focused_path` returning that one caller | no new row; CF-T7-2 stands as T7 recorded it | Unchanged: the first production reader of `focused_path`, or a task giving an ABI-created node a focus annotation |

#### Re-decided at close

The start gate selected traps 1, 2, 3, 4 and 5, armed 6, and declined 7.
**The selection survived**, and each call is confirmed by what was built:
the two uncompiler-enumerable filters were real and are in the audit table
(1); the side-effect enumeration is what placed the arm in the `WM_KEYDOWN`
ordering and identified the drain question fixture 7 asserts (2); two
derived pairs were given one owner each, one of them mutation-verified (3);
every authored arm has a firing test, and one arm was declined *because* no
test could fire it (4); six carry-forwards are recorded with re-triggers
(5); trap 6 stayed armed and did not fire, though one deterministic failure
was root-caused rather than re-rolled (#7); and 7 stayed non-applicable —
nothing this task built paints.

**Two things were built that the gate did not name.** The Grid track-list
stop set (a shared sub-parser the new production collides with, found by a
deterministic failure) and `the_authored_key_down_walk_consumes_ahead_of_the_host_key_slot`
(written after W12 measured the arm's placement to be unpinned). Both are
strengthenings within the selected traps rather than deviations, and
neither changes the review lane.

**The lane stays the corrected one — full independent review** — for the
reasons the start gate recorded, and the trap-#4 branch check composes into
it rather than being replaced by it.

#### Re-audit of the whole task list

Per [plan.md](./plan.md) §Cross-task obligations, the full list was re-read
at this close gate rather than only T8's item.

- **T9** — inherits the `for`-body handler rejection **intact on both
  gates**: `check_members_inner`'s `inside_for_template` arm and
  `validate_node_references_in_scope`'s are untouched. What is new for T9
  is that `Member::SignalHandler` and `IrHandler` now carry `arg`, so the
  loop scope it must thread into handler bodies travels beside an existing
  optional field rather than into a two-field struct — and `signal_key` is
  the function a per-item `key-down` would compose its storage key through,
  if T9's admission reaches that far. CF-T7-1 (anchor address reuse) is
  unchanged and still T9's.
- **T10** — inherits **CF-T8-5**: the walk is upward-only, so the gallery's
  `key-down` handlers must sit at or above whatever the scope focuses.
  Also inherits a smaller surface than the plan predicted in one place and
  a wider one in another: `Grid` and `ZStack` now accept `clicked`, and a
  `Button` may no longer carry a child (nothing in the shipped `.ui` files
  does — verified by building all three example hosts).
- **T11** — unchanged. Touch enters `hit_test_click`; nothing in this task
  touches the pointer path. Whether a touch contact moves focus is still
  T11's explicit decision.
- **T12** — control D (Esc closes the lightbox / an unrelated key does not)
  now has a runtime that can distinguish the two: before this task an
  "unrelated key" had no authored path to fire on at all. Its "unrelated
  key" leg should use a **recognised** key with no handler, which is what
  `the_authored_key_down_walk_consumes_ahead_of_the_host_key_slot` pins at
  the state level.
- **T13** — gains **three** re-verification items beyond the three T7 left:
  CF-T8-3's three normative gaps (§3's grammar production, §8.8's IR
  grammar, §4.5's stale sentence), CF-T8-1 (whether §4.19 / §4.8's Button
  keyboard-activation sentences describe the landed runtime), and CF-T8-4
  (whether an unrecognised signal name should be a diagnostic). §4.16's
  example is **corrected here**, so T13's check of it is a confirmation
  rather than a repair. §4.19's recognised-key table and its "keys the
  runtime keeps" table are both now checkable against named code
  (`RECOGNISED_KEY_NAMES`, and the four-arm `WM_KEYDOWN` order).
- **Cross-task obligation "no new ABI function"** — held, and positively
  evidenced: `wasamo_signal_connect` already admits an arbitrary signal
  name, so a host listener on `key-down("ArrowLeft")` is the vocabulary
  widening [constraints §2](../requirements/constraints.md) permits rather
  than a signature change. No `extern "C"` function was added or altered.
- **Cross-task obligation "every task that measures something re-reads the
  whole task list"** — discharged here.

#### Verification means

Run against the **final branch state**.

`cargo fmt --all -- --check` zero exit, `git diff --check` clean,
`cargo clean` (11,505 files / 3.0 GiB removed), then
`cargo build --release --workspace` 1m26s success,
`cargo build --workspace` 1m24s success, and
`cargo test --workspace --no-fail-fast` **48 binaries/sections, 1,201
passed, 0 failed, 0 ignored**. T7's baseline was 1,145 across 47 sections;
the extra section is `tests/key_down_integration.rs`, and the 56 added
tests reconcile exactly against a per-file `#[test]` count taken between
`723298d` and this branch: `wasamo-ir` +5, `check.rs` +16, `parser.rs` +9,
`lower.rs` +1, `emit.rs` +2, `roundtrip.rs` +1, `ir_loader.rs` +9,
`focus.rs` +3, `togglebutton_runtime_integration.rs` +2,
`event_routing_integration.rs` +1, `key_down_integration.rs` +7.

**The new fixtures ran rather than skipped**, verified by running
`key_down_integration` with `--nocapture` and confirming the shared guard's
`skipping …: runtime compositor unavailable` line does not appear.
`tests/common/mod.rs` was not touched, so the `0x80070005` two-conjunct
check ([constraints §8](../requirements/constraints.md)) is intact and the
standing obligation to verify a newly authored guard does not apply — no
guard was authored.

`cargo build -p counter-rust -p gallery-rust -p bool-demo-rust` succeeds,
which is what would catch a shipped `.ui` tripping either new reject: those
hosts run `wasamoc` over `counter.ui`, `gallery.ui` and `bool-demo.ui` at
build time.

**What this task's evidence cannot show, stated rather than implied.**

- Every fixture runs at 96 DPI and scale 1. Key delivery is
  geometry-independent, and the one click-bearing fixture added here (the
  occlusion leg) derives its coordinates from the scale the runtime
  committed — but nothing here re-exercises the pointer conversion T2's
  non-unit-scale fixture owns, and that fixture is unchanged and green.
- **`key_down_on_key`'s `rebase` call is not pinned by any test in this
  task.** Fixture 7 performs a key-driven structural mutation but sends no
  second key afterwards, so no fixture here observes a retained id being
  re-expressed between two key presses. It is the same call the other three
  `*_on_key` functions make, and T7's fixtures do exercise it through them;
  recorded as a residual rather than claimed.
- The `key-down` **host-listener** delivery has no test (CF-T8-6).
- No frame was captured. This task paints nothing, and a gallery frame
  taken now would be produced identically by the pre- and post-T8 runtime —
  the gallery carries no `key-down` handler until T10.

#### Independent review and its remediation (recorded 2026-08-08)

The review lane
([implementation-gates.md §4](../../../procedures/implementation-gates.md))
was executed as a **full independent review** — the lane this task's start
gate corrected the preamble's prediction to — by a second agent that did
not write the code, against `36829ea` … `e0f52a7`. It built its own
branch-coverage table from the diff rather than from the close gate's,
re-ran the call-site greps, traced the validate recursion on both gates for
the Button-family rule, checked every `peek_next()` site in `wasamoc`'s
parser for a sibling of the Grid track-list collision, searched outside the
diff for claims the change invalidated, and independently reproduced the
suite numbers and the per-file test-count reconciliation.

It confirmed as sound: that all eight then-existing fixtures reach the
branch each is claimed to reach, through the claimed path, with
`send_key` / `key_and_drain` correctly chosen for whether Phase 2 is needed
(the exact gap T7's review found three times, not recurring here); that
the `0u16..=255` sweep is not vacuous, every mapped virtual key being
`≤ 0xFF`; that no second key-name table and no second composer of the
storage key exists anywhere in the workspace, and that
`crate::ir::is_recognised_key_name` in `check.rs` is the `pub use
wasamo_ir as ir` re-export rather than a copy; the `WM_KEYDOWN` borrow
pattern against its three siblings; that `focus_landing` gates the root on
`enabled` where `collect_stops` does not; that neither Button-family gate
over-rejects and both reach a Button generated inside a `for` or an `if`;
that the `has_binding` deferral matches its ToggleButton sibling and
`WidgetNode::button`'s four other callers were correctly left alone; that
`parse_grid_track_list` is the parser's only bare-word-absorption
ambiguity, with no sibling; that nothing else depended on the two removed
per-kind rules; and that `focused_path`'s "only caller" claim is still
accurate.

Three findings, all dispositioned. **F1 is a defect the close gate's own
artifact got wrong**, and it is the artifact-shaped failure this review
exists to catch.

- **F1 — `deliver_key_down`'s declined suppression arm was declined on a
  false premise, and the arm is reachable.** The close gate's #5 recorded
  a branch *not* added, with the reasoning that a disabled Button can
  never be on a `key-down` chain. That covers only the `focus.focused()`
  start. `key_down_on_key`'s **other** start is `traversal_root`, which
  is the tree root, and `collect_stops` never gates the root node —
  it is visited with `is_root = true`, which skips the role and enabled
  checks entirely for that one node. A component whose root widget *is* a
  disabled Button reaches the arm. **Re-measured by the lead** with a
  throwaway probe before acting: a root `Button { enabled: gate_enabled
  key-down("Enter") => { root.gate_enabled = true; } }` with nothing
  focused reports `enabled=Some(false)` before the key and `Some(true)`
  after — the disabled Button's own handler ran.
  **Fixed** (`ecc07ac`): a disabled Button-family node now suppresses its
  own `key-down` dispatch and **does not consume**, the same disposition
  `click_disposition_for` gives a click and the reading §4.8's own focus
  clause supports ("cannot be reached or activated from the keyboard").
  Delivering one authored signal while suppressing the other is an
  asymmetry an author could only find by hitting it.
  `a_disabled_root_button_suppresses_its_own_key_down_without_consuming`
  fires the arm with both legs — disabled and enabled — so "the handler
  did not run" cannot be satisfied by a key that never arrives.
- **The gap behind F1, found while fixing it: `signal_key`'s argument was
  not observable at the behaviour level at all.** Running the W9 mutation
  the close gate had planned and not run (§6's numbering gap, the review's
  F2) reddened **only** `wasamo-ir`'s own unit test — the whole
  1,201-test workspace stayed green. The reason is the single-owner design
  itself: the loader writes the storage key with `signal_key` and the
  dispatcher looks it up with the same function, so an argument dropped on
  *both* sides is invisible to any fixture carrying one `key-down` handler
  per node. `two_key_down_handlers_on_one_node_are_told_apart_by_their_key`
  is the smallest shape where the argument has to survive; the mutation
  now reddens it and `the_authored_key_down_walk_consumes_ahead_of_the_host_key_slot`.
- **F2 — §6's witness numbering skipped W9 and W13 with no explanation.**
  Correct, and both are now accounted for: **W9** (`signal_key` ignoring
  its argument) was planned, not run, and is exactly the witness that
  would have exposed the gap above — it is run and recorded in the table
  below; **W13** (`rebase` removed from `key_down_on_key`) was planned and
  is not run, because no fixture in this task mutates structure between
  two key presses — which the close gate's §Verification means already
  records as a stated residual rather than a claim.
- **F3 — `key_down_on_key`'s unresolvable-path arm returns `false` where
  `dismiss_on_key`'s returns `true`.** Not changed: the asymmetry is the
  two signals' consumption rules rather than an oversight. Escape is
  consumed by an entered scope *existing* — writing no `dismiss` handler
  is how a scope declines to close — so `dismiss_on_key` has decided
  consumption before it resolves a path; a `key-down` is consumed only by
  a handler that *runs*, so a walk that cannot start has consumed nothing.
  Written onto the branch rather than left to be re-derived.

Two further mutation witnesses, both applied by the lead, confirmed
present by re-reading, run, then reverted and the revert confirmed:

| Witness | Mutation | Went red | Reading |
|---|---|---|---|
| **W9 (run at last)** | `signal_key` ignores its argument and returns the bare signal name | before the remediation: **only** `wasamo-ir`'s `signal_key_with_arg_returns_dsl_spelling`, nothing else in 1,201 tests. After: that test **plus** `two_key_down_handlers_on_one_node_are_told_apart_by_their_key` and `the_authored_key_down_walk_consumes_ahead_of_the_host_key_slot` | The single-owner design that prevents drift also hides a symmetric error. A behaviour-level discriminator was missing and now exists |
| **W16** | the `enabled` suppression arm removed from `deliver_key_down` | `a_disabled_root_button_suppresses_its_own_key_down_without_consuming` alone | The new arm is load-bearing for exactly the shape F1 named, and no other fixture depends on it |

**Re-verification after the remediation** (superseding the counts in
§Verification means above, which were taken before it):
`cargo fmt --all -- --check` zero exit, `cargo build --workspace`
successful, `cargo test --workspace --no-fail-fast` **48
binaries/sections, 1,203 passed, 0 failed, 0 ignored** — the two added
tests are `key_down_integration.rs`'s eighth and ninth fixtures. The clean
`cargo clean` rebuild in both profiles recorded in §Verification means was
taken before the remediation and is **not** re-run here; the remediation
touches three files and changes no build input the rebuild was measuring.

**External-agent (codex) review is not performed**, per the owner's
standing disposition for this phase (T1–T7).

#### Owner disposition of CF-T8-1 and CF-T8-2 (2026-08-08)

The two findings the T8 retrospective's item 4 routed to the owner are
dispositioned. Neither changes what T8's own commits landed; one changes
what the branch carries.

**CF-T8-1 — Button keyboard activation.** Not built, and **not assigned
to a milestone**. It goes to the
[candidate pool](../../../candidate-pool.md) with the owner's direction:
**decide it alongside the other keyboard-operable controls** a widget set
brings — a DropDown, CheckBox or Radio each activate from the keyboard
too, and one activation contract for the family beats retrofitting
Button's. The reason it is not a free addition is unchanged from the
finding: it puts `Space` and `Enter` into
[dsl_spec §4.19](../../../../docs/dsl_spec.md)'s keys-the-runtime-keeps
table, which lists neither, so an authored `key-down("Enter")` would stop
firing while a Button is focused. T13 re-verifies §4.19's and §4.8's
keyboard-activation sentences against a runtime that still does not have
it.

**CF-T8-2 — `Text` and `Rectangle` children: reject them too, and do not
obstruct a future re-opening.** The finding stands as measured — the
defect spans four kinds, not two — and the owner settled both halves of
it at once.

The first half is the rule. `Text` and `Rectangle` join `Button` and
`ToggleButton`: a widget child on any of them is a diagnostic at both
gates. Re-measured before implementing, so the release-profile half is
not inherited from the Button case:
`Text { text: "label"  Button { text: "inner" } }` is accepted by
`wasamoc check`, accepted by the loader, and aborts a debug build on
`sync_visuals`'s child-count `debug_assert_eq!` — the same panic message
T3 measured for the Button shape.

**The second half is a constraint on the design, and it is what shapes
the code.** "Do not obstruct a later spec change that lets one of these
kinds hold children" means, concretely, that re-opening a kind must be a
single obvious edit rather than a hunt through two crates — the drift
shape CF-T6-3 recorded for signal admission, arriving in a second place.
So:

- The four kinds are named in **one** place,
  `wasamo_ir::LAYOUT_CHILDLESS_WIDGET_KINDS`, in the crate `wasamoc` and
  `wasamo-runtime` already share. Both gates call
  `layout_treats_as_childless`; neither names a kind.
- The table is named after the **reason** (layout arranges it as a single
  childless rectangle), not after the symptom set. A name tied to Button
  would have been wrong the moment `Text` joined it.
- The const's doc comment carries the **re-opening recipe** as steps:
  give the kind's `build_layout_tree` arm real children, remove its
  entry, and both gates stop rejecting in the same edit.
- `build_layout_tree`'s childless arm carries a **pointer back** to the
  table, because that arm is where a re-opening actually starts.
- The diagnostic is now kind-agnostic. It named Button's `text:` label,
  which is false prose on a `Rectangle`; it now names the offending kind
  and the reason. Its citation moves from §4.8 (the property catalog,
  which says nothing about children) to **§4.4**, the widget registry,
  where the container / leaf distinction is visible.

**The table's membership is deliberately not pinned by a `wasamo-ir`
test.** This is the one place the design differs from
`RECOGNISED_KEY_NAMES`, and the difference is the point:
§4.19 fixes that table's 22 entries normatively, so pinning them is
pinning the spec. This table is a *fact about layout* that a later phase
is expected to change, and an exact-contents assertion beside it would
make re-opening a two-file edit for no gain — the per-kind reject tests
at both gates already pin membership at the level that matters, and those
are the tests a re-opening *should* have to update, because they are what
changes. What stays pinned is the invariant that holds for any
membership: no duplicates.

| Witness | Mutation | Went red | Reading |
|---|---|---|---|
| **W17** | `LAYOUT_CHILDLESS_WIDGET_KINDS` narrowed back to `["Button", "ToggleButton"]` | exactly four tests — `text_with_widget_child_rejected` / `rectangle_with_widget_child_rejected` in `check.rs` and `validate_rejects_text_with_widget_child` / `validate_rejects_rectangle_with_widget_child` in `ir_loader.rs` — and **nothing else**, `wasamo-ir`'s own tests included | The two gates genuinely key off the one table, so "re-opening a kind is one edit" is measured rather than asserted. Run twice: the first run also reddened `wasamo-ir`'s exact-contents assertion, which is what identified that assertion as an obstruction and got it removed |

**A normative gap this opens, recorded for T13.** No spec section says
which widget kinds admit children. §4.9 fixes Box's count and §4.11
ScrollView's; "these four admit none" is now enforced at both gates and
stated nowhere. §4.4's registry is where the distinction is visible and
is what the diagnostic cites. Added to T13's re-verification list.

**Re-verification after both dispositions** (superseding the counts
above): `cargo fmt --all -- --check` zero exit, `cargo build --workspace`
successful, `cargo test --workspace --no-fail-fast` **48
binaries/sections, 1,212 passed, 0 failed, 0 ignored** — the nine added
tests are three in `wasamo-ir`, three in `check.rs` and three in
`ir_loader.rs`. `cargo build -p counter-rust -p gallery-rust -p
bool-demo-rust` succeeds, so no shipped `.ui` trips the widened reject.

---

## T9 — DSL: per-item handlers inside `for`

### Start gate (recorded 2026-08-08, before any source edit)

Read first: [AGENTS.md](../../../../AGENTS.md),
[implementation-gates.md](../../../procedures/implementation-gates.md),
[plan.md](./plan.md) §T9 and §Cross-task obligations,
[preamble.md](./preamble.md),
[DD-M4-P2-005](../decisions/dd-m4-p2-005-dsl-handler-surface.md),
[DD-M4-P2-001](../decisions/dd-m4-p2-001-event-routing-model.md),
[constraints.md](../requirements/constraints.md), the T1 / T3 / T4 / T5 /
T7 / T8 close gates above, and the T8 retrospective.

**The whole plan was grepped for `T9`** (the T8 retrospective's start-gate
corrective — "what has another task's item sent to this one that this
task's own item does not say?"). Seven senders, all of them already in
this task's item or listed below: T1 (a subtree must reach layout before
its rectangle is trusted), T2 (a click-driving fixture must lay its tree
out), T3 (the structural side-effect enumeration is named as T9's too),
T4 (CF-T4-1, the index-based hover record), T5 (CF-T5-1, an in-range but
renamed `FocusId`), T7 (CF-T7-1, anchor address reuse), T8 (`arg` beside
the loop scope, `signal_key`, and the rejection intact on both gates).
**Nothing arrived that the item did not already carry**, which is the
first measurement of that corrective and, this once, does not falsify it.

The other accumulated start-gate lines, answered in order:

- **T1 — new store / unit / coordinate system?** One new retained value:
  the generated subtree's loop scope (`ForItemContext`), in *collection
  index* coordinates — not tree-index, not DIP. It has one writer.
- **T2 — which test pins the property this task deletes?** Four, all
  found and listed under fact 6. Two in `wasamoc::check`, two in
  `ir_loader`; two of the four are *comments asserting unreachability*
  rather than assertions, which is the harder half.
- **T3 — was the evidence a later task needs built once here?** T10
  needs a per-item `clicked` carrying which item, in the gallery. This
  task builds that shape in a fixture, not in `gallery.ui` — landing the
  `.ui` is T10's.
- **T4 — was the negative prediction this task rests on measured once?**
  The prediction is "a retained position never goes stale for a surviving
  row". Measured at fact 4 by reading `plan_tail_range_change`'s two
  arms, and pinned by the click-after-mutation fixture rather than left
  as a reading.
- **T5 — identifiers held across messages: what is their lifetime?**
  Two. (a) The `ForItemContext.position` held on a generated node from
  build to click: it names a *collection index*, and under tail-only
  reconciliation a surviving node's index does not move; an out-of-range
  read (the collection shrank under it) is the failure mode, not a wrong
  node. (b) The focus projection's anchor (a node address) across a
  regeneration — CF-T7-1, whose failure is "focus lands on an unexpected
  widget", never an unsound read.
- **T6 — how many gates does the rule have?** Two (`wasamoc check` and
  the runtime IR loader), and both must be driven from one input by at
  least one test: the integration fixtures go `.ui` → `check` → `lower` →
  `emit` → `parse_ir` → `build_widget_tree`, which is that test.
- **T7 — which closing carry-forward is `doc-folded`, and where?**
  CF-T7-1 is folded into `FocusProjection::id_of_anchor`'s doc comment.
  This task *checks* the residual rather than closing it, so that comment
  is re-read and corrected only if the check changes what it says. One
  other doc comment **does** go stale here and is on the work list:
  `mutate_for_loop_subtree`'s rollback branch says "today's handler-free
  `for`-body children hold none", which this task falsifies (fact 7).

#### Normative statements that already answer this task (DD-V-031)

| Question | Where it is answered | What it fixes |
|---|---|---|
| Is a handler admitted inside a `for` body | [dsl_spec §4.19 §Per-item handlers](../../../../docs/dsl_spec.md), [§4.15 §Handlers inside a `for` body](../../../../docs/dsl_spec.md) | Yes, on any widget the body builds. §4.15's subsection states the M3-era rejection is lifted |
| How the binders are spelled in handler position | [dsl_spec §4.19 §Per-item handlers](../../../../docs/dsl_spec.md), [DD-M4-P2-005 §I1](../decisions/dd-m4-p2-005-dsl-handler-surface.md) | **Exactly as in binding position** — bare `item` / `index`, no qualification, no separate namespace. `for.item` / `loop.index` were considered and rejected |
| When a binder read resolves | [dsl_spec §4.19 §Per-item handlers](../../../../docs/dsl_spec.md) | **When the handler runs**, not when the subtree was generated |
| What a per-item handler belongs to | [dsl_spec §4.19 §Per-item handlers](../../../../docs/dsl_spec.md) with [§4.15 §Identity baseline](../../../../docs/dsl_spec.md) | A **position**. After a collection mutation the handler at position `n` reads whatever item is now at position `n`. That is a joint consequence of positional identity and invocation-time reads; neither is safe to change alone |
| Which subtrees survive a mutation | [dsl_spec §4.15 §Identity baseline](../../../../docs/dsl_spec.md) | Tail-only: a tail append materialises only the new tail, a tail removal disposes only the removed tail, **subtrees at retained positions are retained and not rebuilt**. A same-length whole-value reset makes *no* structural edit |
| When the registration is released | [dsl_spec §4.19 §Per-item handlers](../../../../docs/dsl_spec.md), [DD-M4-P2-005 §Registration lifecycle](../decisions/dd-m4-p2-005-dsl-handler-surface.md) | With the generated subtree, **on the same path that releases that subtree's bindings**. Explicitly *not* separately owned — a second lifecycle would be the parallel-data drift the runtime keeps eliminating |
| What a binder read outside a `for` body is | [dsl_spec §4.15 §Diagnostics](../../../../docs/dsl_spec.md), [DD-M4-P2-005 §I1](../decisions/dd-m4-p2-005-dsl-handler-surface.md) | A diagnostic. Both directions need a test (accept and reject) |
| Whether a new ABI function is allowed | [constraints §2](../requirements/constraints.md), [framing agreement ⑦](../requirements/framing.md) | No. This task adds none |
| Whether nested `for` is in scope | [dsl_spec §4.15 §Out of scope](../../../../docs/dsl_spec.md) | No — still rejected at both gates. So a node in a `for` body has **at most one** loop scope, which is what lets the scope be one field rather than a stack |

#### Where the normative text does **not** answer, recorded rather than resolved here

Per DD-V-031 these are divergences for the phase-close re-verification
(T13), not questions this task settles by editing normative prose.

- **§4.15's Diagnostics table still lists two rows this task makes
  false**: "Handler inside a `for` body … *handlers inside a `for` body
  are not yet supported*" and "Binder read in handler position … *loop-
  local binders are not readable in handlers*". The subsection below the
  table ("Handlers inside a `for` body (admitted in M4-Phase 2)") already
  contradicts them; the Moment 1 sync added the subsection and left the
  table rows. This is a **false statement**, not a gap, and it is listed
  for T13 rather than repaired here for the same reason T8 left §3 /
  §8.8 / §4.5 alone — the phase's normative surface is re-verified in one
  pass, and T13's list already carries §4.15.
- **No section says whether a handler-body assignment is type-checked.**
  It is not (fact 5), for literals as much as for binder reads, so
  admitting `item` in handler position inherits an existing, uniform
  absence rather than opening a new one.

#### Measured facts (probes run before choosing an approach)

Five `.ui` files through `target/release/wasamoc.exe check`, four source
readings. Under ten minutes, and three of them changed the shape of the
task.

**Fact 1 — the rejection is a single arm at each gate, and it is the
*only* thing in the way of the compiler half.** `check.rs`'s
`Member::SignalHandler` arm pushes "handlers inside a `for` body template
are deferred in M3-Phase 7" whenever `inside_for_template`; the loader
has the same rejection twice, in `validate_phase7_iteration_invariants`
(`inside_for_template && !node.handlers.is_empty()`) and in
`validate_node_references_in_scope`'s handler loop. Probes P1 / P2 / P3 /
P5 (a `clicked` with no binder read, one reading `index`, one reading
`item`, and a `key-down("Enter")`) all fail on that one message and on
nothing else.

**Fact 2 — `lower` already carries the loop scope into handler bodies,
so "the phase's only new IR content" is false as written.**
`lower_node_with_loop`'s `Member::SignalHandler` arm is
`expr: lower_block(body, ns, loop_ctx)` — the loop context is threaded
into the handler body today, unconditionally. `HandlerExpr::ItemRead` /
`IndexRead` already exist, `emit` already writes `(item-read x)` /
`(index-read i)`, and the loader's `parse_sexpr` is **shared between
bindings and handlers**, so it already parses both inside an `on` body.
**No IR type, no IR text-grammar production, and no lowering change.**
What is genuinely new is one runtime thing: the *evaluation context* a
handler body runs against. DD-M4-P2-005 §IR and compiler impact names
both halves ("`lower` must carry the loop scope into the handler body,
and the runtime's handler evaluation context must supply it"); only the
second half is outstanding.

**Fact 3 — the review lane stays `full independent review`, for a
corrected reason.** [preamble.md](./preamble.md) predicts it as "the
phase's only new IR content … (schema / IR class)". Fact 2 removes the
schema / IR ground. The lane holds on the *other* trigger: a **runtime
structural change** — a new retained field on every `WidgetNode`, a
change to the one snapshot type all three signal dispatchers (`clicked`,
`dismiss`, `key-down`) share, and a new arm in the handler evaluator.
The same correction shape as T8's, in the opposite direction.

**Fact 4 — a surviving row's position never moves, and that is a
property of the reconciler rather than of the fixtures.**
`mutate_for_loop_subtree` returns early when `old_len == new_len`, and
otherwise takes `plan_tail_range_change`, whose only two arms are
`Insert { start, count }` at the tail and `Remove { tail_first_indices }`
from the tail. Retained rows are neither rebuilt nor re-indexed. So
capturing `position` at generation and reading the *value* at invocation
is exactly §4.15's positional identity — and the discriminating test is a
**same-length whole-value reset** (`labels = ["z", "y"]`), which makes no
structural edit at all: generation-time capture returns the old string,
invocation-time resolution returns the new one.

**Fact 5 — handler-body assignments are not type-checked, at all,
today.** `component P9 … { state n: i32 = 0  state s: string = ""  …
clicked => { root.n = "abc"; } … clicked => { root.s = 5; } }` passes
`wasamoc check` with **exit 0 and no diagnostics** (probe P9); both fail
at *invocation* with a logged `EvalError`, and there is no `set_string`
anywhere in the runtime, so handler-position string assignment does not
exist for any right-hand side. Two consequences: (a) the binder-read rule
this task adds is a **scope** rule, matching DD-M4-P2-005's own wording
("the binders resolve only inside a `for` body, and a reference outside
one is a diagnostic"), and adding a type rule for binder reads alone
would make handler position *stricter* for a binder than for a literal;
(b) `root.n = label` over a `string[]` collection is accepted and logs at
click time — recorded as an inherited limit, not built around.

**Fact 6 — four existing tests stand on the rejection this task
removes, and two of them are comments rather than assertions.**
`check.rs`'s `for_body_rejects_handler_and_nested_for_at_any_depth` and
`dismiss_handler_inside_for_wrapped_container_without_modal_scope_rejected`
(which asserts `errs.len() == 2`); `ir_loader.rs`'s
`for_member_rejects_handler_and_nested_for_inside_template` and
`dismiss_handler_inside_for_wrapped_container_hits_the_pre_existing_handler_gate_first`.
The last one's whole body is an argument that the `dismiss` gate's
`ControlFlow::For` arm is **unreachable through `parse_ir`**, because the
handler rejection short-circuits ahead of it — and it says so in a
comment that would survive this change silently. **This task makes that
arm reachable**, which is T8's learning (b) recurring inside the same
phase: an unreachability claim that was true of one path.

**Fact 7 — one production doc comment becomes false.**
`mutate_for_loop_subtree`'s partial-insert rollback carries "Today's
handler-free `for`-body children hold none [no registry entries], so this
branch's disposal is a *defensive* symmetry … not an active leak fix for
current bodies". After this task a `for`-body child can carry handlers,
and a host listener can be connected to one through
`wasamo_signal_connect`. The branch does not change; its justification
does.

**Fact 8 — the three signal dispatchers already share one snapshot, so
the loop scope has exactly one place to go.** `click_disposition_for`,
`WidgetNode::deliver_dismiss_at` and `WidgetNode::deliver_key_down` all
reach `signal_handlers_for` → `run_signal_handlers`, and
`signal_handlers_for` clones the inline bodies out of the node *before*
any handler runs — that clone is what makes the dispatch sound when a
handler destroys its own node. The loop scope must be cloned in **the
same snapshot**, or it would be read from a node a preceding handler may
already have freed.

**Fact 9 — the item-out-of-range read is reachable, and buildable.**
`read_item_i32_tracked` returns `Ok(None)` when the position is past the
end. A handler that shortens its own collection first —
`{ xs = xs.drop-last(); root.n = item; }` on the last row — reaches it,
because a collection write drains its reactive effects **synchronously,
inside the statement**, and the handler body then keeps evaluating from a
clone. Per [DD-V-030](../../../cross-milestone/decisions/dd-v-030-carry-forward-buildability.md)
that makes it a testable shape rather than a finding.

**Fact 10 — DD-M4-P2-001's stated reason for residual 1 not firing does
not match the runtime, and this task is where it is measured.**
The decision says the near case "stays inside the existing machinery for
the same reason the drain is placed after the walk: **the handler has
already returned when regeneration runs**". It has not:
`register_for_loop_binding` installs an ordinary reactive `EffectHandle`,
so `xs = xs.append(…)` regenerates the subtree inside `Signal::set`,
*during* the handler's own statement — which is precisely the hazard
`run_signal_handlers`'s safety comment is built around. The
**conclusion** (no cycle: regeneration re-invokes no handler) is
unaffected; the **explanation** is wrong. Recorded here as a divergence
to be re-measured against the shipped fixture at this task's close gate,
per plan §T9's own instruction, and dispositioned there rather than by
editing an Accepted decision from a start gate.

#### Trap selection (implementation-gates §1)

```
- [x] #1 semantic migration   - [x] #2 side effects   - [x] #3 parallel data   - [x] #4 branch tests
- [x] #5 carry-forward        - [ ] #6 root cause     - [ ] #7 GUI positive control
```

| # | Applies | Why / why not |
|---|---|---|
| 1 | **yes** | `EvalError` gains a variant for the item-out-of-range read (fact 9), and `WidgetNode` gains a field with ten construction sites. Both are compiler-forcing shapes; the audit must still grep for filtering helpers that could absorb the new case silently — `signal_handlers_for`'s `.filter(\|(sig, _)\| sig == signal)` is exactly that shape, and is where the loop scope has to travel (fact 8) |
| 2 | **yes** | Subtree removal is the named artifact: what a removed generated subtree releases (bindings, registry tokens, inline handlers, focus anchors, the hover record, the Visual). DD-M4-P2-005 says this failure is **silent in one direction** — a handler left registered against a dropped subtree appears in no rendered frame — so the enumeration is the check, not a frame |
| 3 | **yes** | The loop scope is derived data: the same `ForItemContext` already feeds every per-item *binding*. One build-time writer for both is what keeps them from drifting. T8's learning applies directly — a single owner hides *symmetric* errors — so the artifact must name a test that consumes the scope **asymmetrically**: two rows whose handlers must read *different* positions, not one row read twice |
| 4 | **yes** | New branches: the admission arms at both gates, the binder in-scope / wrong-binder / no-scope arms at the loader's handler path, the item-out-of-range arm, and the two newly *reachable* loader arms of fact 6. The out-of-range arm is a **boundary condition**, so [DD-V-029](../../../cross-milestone/decisions/dd-v-029-pure-logic-red-test-obligation.md)'s red-test obligation applies to it |
| 5 | **yes** | CF-T7-1 (anchor address reuse) is checked here rather than assumed, and this task's own residuals — the inherited untyped handler surface (fact 5), the per-item `modal-scope` shape it newly admits — are carried with re-trigger criteria |
| 6 | no | No recurring or vanishing failure is in flight. Armed rather than dismissed: any deterministic failure met during the work gets a minimal repro and a root cause, not a re-run |
| 7 | no | The deliverable is not a rendered frame. DD-M4-P2-005 says so in as many words — the registration-lifecycle failure "is not visible in any rendered frame" — and the identity question is answered by reading handler effects back, not by looking. The gallery's first per-item handler is **T10's**, and T12 owns the frames |

#### Review lane

**Full independent review** (fact 3), by a subagent that wrote none of
the code. The trap-#4 branch/test check composes into it rather than
replacing it.

#### The approach, and the boundary this task does not cross

1. **Both gates admit the handler; the scope rule is the only new
   authored rule.** `check` drops the `inside_for_template` arm and keeps
   the existing out-of-body binder diagnostic — whose wording ("inside
   its `for` body *expression bindings*") stops being true and is
   corrected. The loader drops its two copies and threads the loop scope
   into `validate_expr_references`, which already has in-scope /
   wrong-binder / no-scope arms written for bindings.
2. **The loop scope is one field on `WidgetNode`, written once**, beside
   `set_focus_annotation` in `build_node_with_loop_context` — the same
   site and the same single-writer discipline. Nested `for` is rejected,
   so one field is total (normative table above).
3. **It is snapshotted with the handler bodies**, inside
   `signal_handlers_for`, so all three dispatchers get it without a
   second path and none of them dereferences a node a preceding handler
   may have freed.
4. **The evaluator gains a loop-aware handler context** — writable like
   `HandlerEvalContext`, item-aware like `ForItemEvalContext`, with
   **untracked** reads, because handlers run outside the reactive scope.
5. **`evaluate` gains the two integer-context arms**, `ItemRead` and
   `IndexRead`; `index` is always available, `item` can be out of range
   and gets a named `EvalError`.

Not crossed:

- **No gallery `.ui` change** — T10 lands the first shipped per-item
  handler, and T7's throwaway-probe-then-revert discipline applies to
  anything built against the gallery here.
- **No `set_string` for handler bodies.** String assignment does not
  exist in handler position for *any* right-hand side (fact 5); adding it
  for binder reads would be a surface widening no task owns.
- **No type rule for binder reads in handler position** (fact 5).
- **No keyed identity, no nested `for`, no per-item conditional
  presence** — §4.15's out-of-scope list is untouched.
- **No normative prose.** §4.15's two false diagnostic rows go to T13.
- **No change to the drain boundary** (T3's), the focus seam (T7's), or
  `plan_tail_range_change`.

### Close gate (recorded 2026-08-08)

Six implementation commits plus the start gate and the review correction:

| commit | content |
|---|---|
| `94002c0` | start gate (ten measured facts, the DD-V-031 normative table, trap selection, lane, boundary) |
| `b9e12fd` | `wasamoc check` admits a handler inside a `for` body |
| `ed5117b` | the IR loader gate does too, and threads the loop scope through the handler path |
| `38f4daf` | the runtime evaluates binder reads at invocation time |
| `9bcbd60` | the per-item handler integration fixtures |
| `f8a346e` | the CF-T7-1 anchor-reuse check |
| `bd15ec2` | the two evidence gaps the independent review found |

#### Trap selection re-decided at close (plan's standing instruction)

Unchanged from the start gate: **#1, #2, #3, #4, #5 applied; #6 and #7
did not.** The task built what it predicted it would build, with one
addition inside the same traps rather than a new one — the collection-
append path over three element types (below), which is trap #4 and trap
#1 material, not a new class.

#### #1 — call-site audit

Two semantic migrations, both compiler-forcing.

**`EvalError` gained `ItemOutOfRange { binder }`.**

| Site | `rg` query | Classification | Reason |
|---|---|---|---|
| `handler.rs` `impl Display for EvalError` | `rg "match self" -A 12 handler.rs` | **must-dispatch** — arm added | The only exhaustive match over the enum in the workspace; the compiler forced it |
| ~100 `EvalError::X` occurrences in `handler.rs` / `reactive.rs` / `widget.rs` | `rg "EvalError::"` | ignore-OK | All are variant *constructions* or `Result<_, EvalError>` return types, not matches |
| `matches!(…, Err(EvalError::TypeMismatch { .. }))` and `assert_eq!` sites in tests | same | ignore-OK | Compare against one named variant; a new variant cannot be silently absorbed |
| `wasamo-dll`, `wasamoc`, `bindings/*`, `examples/*` | `rg "EvalError"` outside `wasamo-runtime` | **not reachable** | `EvalContext` / `EvalError` are not re-exported past the crate boundary (`lib.rs` carries no re-export); zero matches |

No wildcard or filtering arm absorbs the new variant.

**`WidgetNode` gained `loop_scope: Option<ForItemContext>`.** Ten
construction sites, all forced by the compiler, all `None`
(`rg -c "loop_scope: None" widget.rs` → 10, matching `focus_annotation`'s
ten one-for-one). Exactly **one writer**: `set_loop_scope`, called once,
from `ir_loader::build_node_with_loop_context`
(`rg "set_loop_scope"` returns one call and one definition; the remaining
hits are doc comments).

**The filtering helper the start gate flagged.**
`signal_handlers_for`'s `.filter(|(sig, _)| sig == signal)` is the shape
trap #1 warns about, and it is *not* where the loop scope travels: the
scope is cloned unconditionally beside the filtered `inline` vector, in
the same read, so a new signal name cannot lose it. Independently
confirmed by the review.

#### #2 — structural side-effect enumeration: what a removed generated subtree releases

The failure DD-M4-P2-005 names is silent in one direction — a handler
left registered against a dropped subtree appears in no rendered frame —
so this enumeration is the check, and F4 is the one leg of it that can be
observed at runtime.

| Derived state | Released by | How this task confirmed it |
|---|---|---|
| Reactive bindings (`EffectHandle`) | `widget_destroy` → `dispose_subtree_bindings`, recursively | Pre-existing; unchanged by this task |
| Host signal registrations (`wasamo_signal_connect` tokens) | `widget_destroy` → `for_each_ptr(remove_for_widget)` | **F4**: a host listener on a generated row fires once while the row is live, and never again after the row is removed; its `destroy_fn` fires exactly once, synchronously with the removal |
| **Inline handler bodies** (this task's addition) | Owned data on the node (`Vec<(String, HandlerExpr)>`); freed with the `Box<WidgetNode>` | No separate lifecycle — which is DD-M4-P2-005's requirement in as many words ("the handler's registration is *not* separately owned"). Nothing to release, so nothing to leak |
| **The loop scope** (this task's addition) | Same — a plain field on the node | Same reason |
| The focus record's anchor | Rebased at the end of the drain (`emit::flush_layout` Phase 2 → `focus::sync_scopes_to_tree`) | **CF-T7-1 fixture**: after a free-and-allocate in one message the record names a live node that paints the indicator |
| The retained hover record | T4's path (index-based, bounds-checked) | Not exercised here: no M4 shape puts a Button-family widget under the pointer inside a `for` body during a reorder. CF-T4-1 stays open, unchanged |
| Layout / the Visual | `remove_structural_child` + `mark_layout_dirty_for` | Pre-existing |

**One justification in production prose became false and was rewritten.**
`mutate_for_loop_subtree`'s partial-insert rollback said "today's
handler-free `for`-body children hold none [no registry entries], so this
branch's disposal is a *defensive* symmetry". After this task a `for`-body
child can carry handlers and a host listener can be connected to one, so
the disposal is load-bearing. The branch's code is unchanged; only its
reason is.

#### #3 — parallel data: one loop scope, two consumers

The same `ForItemContext` feeds the per-item **bindings** (through
`register_for_item_binding` / `register_for_item_bool_binding`) and, from
this task, the per-item **handlers**. Both are written from the one
`loop_context` parameter inside `build_node_with_loop_context`, in the
same function body, so the two cannot drift.

T8's recorded learning is that this design hides **symmetric** errors — a
mutation to the single owner breaks both consumers identically and no
behaviour-level test reddens. The artifact this trap needs is therefore a
test that consumes the scope **asymmetrically**, and there are two:

- unit — `for_item_handler_ctx_two_positions_read_different_values`: two
  contexts over one collection at positions 0 and 2 must read different
  items *and* different indices;
- integration — **F1**: clicking row 0 and row 2 of the same `for` must
  write different values. One row clicked twice would constrain nothing.

Measured, not asserted: mutation **W-E** (the string item read always
takes position 0) reddens the unit test and **F2**.

#### #4 — branch table

Every branch this task added, with the test that fires it directly.

| Branch | Where | Test |
|---|---|---|
| Handler admitted inside a `for` body (checker) | `check.rs` `Member::SignalHandler` | `for_body_accepts_handler_but_still_rejects_nested_for_at_any_depth`, `for_body_handler_reads_index_binder_accepted`, `for_body_handler_reads_item_binder_accepted` |
| Handler admitted inside a `for` body (loader, two sites) | `validate_phase7_iteration_invariants`, `validate_node_references_in_scope` | `for_member_accepts_handler_but_still_rejects_nested_for_inside_template`, `for_body_handler_index_read_validates`, `for_body_handler_item_read_validates` |
| Binder read with no loop scope (loader) | `validate_expr_references` `ItemRead`/`IndexRead` `None` arms | `for_body_handler_item_read_outside_for_body_rejected`, `for_body_handler_index_read_with_no_index_binder_rejected` |
| Binder read naming the wrong binder (loader) | same, `Some(_)` arms | `for_body_handler_item_read_wrong_binder_rejected`, **`for_body_handler_index_read_wrong_binder_rejected`** |
| Binder read outside the `for` body (checker) | `check_expr_type_in_loop_context`, corrected wording | `index_binder_read_in_handler_outside_for_body_rejected`, `handler_reads_a_different_for_loops_binder_rejected` |
| Loop scope threaded into the collection-assignment path | `validate_collection_assignment_rhs` / `validate_collection_element_expr` | `for_body_handler_collection_append_reads_its_own_binders` |
| `evaluate`'s `IndexRead` arm | `handler.rs` | `index_read_in_integer_assignment_evaluates_to_the_position` |
| `evaluate`'s `ItemRead` arm, `Some` | `handler.rs` | `item_read_in_integer_assignment_evaluates_to_the_element_at_the_position` |
| `evaluate`'s `ItemRead` arm, **`None` → `ItemOutOfRange`** | `handler.rs` | `item_read_past_the_end_yields_item_out_of_range` (DD-V-029, witness W-C) **and** F5 end-to-end |
| `evaluate_binding` / `evaluate_binding_part` binder arms (string) | `handler.rs` | `for_item_handler_ctx_collection_append_reads_binders_for_every_element_type`, `for_item_handler_ctx_string_item_read_past_the_end_is_out_of_range` |
| `evaluate_bool_assignment_value` binder arm, `Some` | `handler.rs` | same append test (bool leg) |
| `evaluate_bool_assignment_value` binder arm, **`None`** | `handler.rs` | `for_item_handler_ctx_bool_item_read_past_the_end_is_out_of_range` — **added at the review's finding**, witness W-G |
| `run_signal_handlers`'s loop-scope-vs-plain context selection | `widget.rs` | F1 / F2 / F3 / F6 (witness W-D) |
| `dismiss` admitted inside a `for` body beside `modal-scope: true` | newly reachable at both gates | `dismiss_accepted_inside_for_body_with_modal_scope`, `dismiss_handler_inside_for_body_with_modal_scope_validates` |
| `dismiss` rejected inside a `for` body without it | newly reachable at the loader | `dismiss_handler_inside_for_wrapped_container_rejected_through_parse_ir` |
| Bare `key-down` rejected inside a `for` body | newly reachable at the loader | `key_down_without_argument_inside_for_body_rejected`, `key_down_without_argument_rejected_inside_for_body` |
| CF-T7-1 fixture's collision arm | `focus_identity_integration.rs` | Not fired — see #5; the arm exists so the collision is a named observation rather than a surprise red |

**Branches deliberately not tested, with the path measured** (the T8
close-gate corrective — a negative claim is true of one path until every
path is enumerated):

- `evaluate`'s `IndexRead` `None` fallback. Claim: unreachable. Measured
  by enumerating **all four** `EvalContext` implementations of
  `read_index_tracked` in the crate — the trait default (`handler.rs`),
  the test mock, `ForItemEvalContext`, `ForItemHandlerEvalContext` — every
  one returns `Err(UnknownProperty)` on a binder mismatch and never
  `Ok(None)`. Independently re-enumerated by the review, which reached the
  same conclusion. Kept as a typed fallback rather than an `unwrap()`.
- The CF-T7-1 collision arm — not reachable on demand; see #5.

#### Mutation witnesses

All seven were applied by the lead, read back from the file before
running, and re-read plus `git diff`-confirmed after reverting. **W-F is
not an implementation mutation** — it is a working alternative *design*,
which is what the T5 corrective asks for at least one of.

| Witness | Mutation | Went red | Reading |
|---|---|---|---|
| **W-A** | `evaluate_binding`'s two binder arms removed (the pre-T9 fall-through) | `for_item_handler_ctx_collection_append_reads_binders_for_every_element_type`, `for_item_handler_ctx_string_item_read_past_the_end_is_out_of_range` | The string append path genuinely needed its own arms |
| **W-B** | `evaluate_bool_assignment_value`'s binder arm removed | the append test | Same for `bool[]` |
| **W-C** | `evaluate`'s `ItemRead` `None` → `unwrap_or(0)` | `item_read_past_the_end_yields_item_out_of_range` only | DD-V-029's obligation for the boundary arm, re-measured by the lead rather than taken from the subagent's report |
| **W-D** | `signal_handlers_for` clones `None` instead of the node's loop scope | F1, F2, F3, **F6** | The snapshot is load-bearing. **Run twice**: before F6 existed it left F5 green, which is what made F6 necessary |
| **W-E** | the string item read always takes position 0 | `for_item_handler_ctx_two_positions_read_different_values`, F2 | Position, not merely presence, is constrained |
| **W-F** | **generation-time capture**: substitute each `ItemRead` with a literal of the item's value at subtree build time, in the loader's handler-attachment loop | **F2 alone**, out of 1,244 | The decisive artifact. A working wrong design fails exactly the fixture DD-M4-P2-005 says must exist, with F2's own predicted message. It also measures that F2 is the **single** test constraining invocation-time resolution |
| **W-G** | `evaluate_bool_assignment_value`'s `None` → `unwrap_or(false)` (the review's own probe) | before: nothing; after adding the test: `for_item_handler_ctx_bool_item_read_past_the_end_is_out_of_range` | The review found the gap; the correction is measured rather than asserted |

#### The DD-M4-P2-001 divergence, measured and dispositioned

Start-gate fact 10 predicted it and F5 measured it. DD-M4-P2-001 gives
residual 1 (cycle detection) this reason for not firing: the near case
"stays inside the existing machinery for the same reason the drain is
placed after the walk: **the handler has already returned when
regeneration runs**."

**It has not.** `register_for_loop_binding` installs an ordinary reactive
`EffectHandle`, so a collection write regenerates the subtree inside
`Signal::set`, during the handler's own statement. F5 observes both facts
from one synchronous `send_click`: the clicked row's own subtree is
already destroyed (`destroyed == 1`) *and* the second statement's item
read has already failed. That is regeneration running **before** the
handler's next statement, not after the handler returned.

**Disposition: the conclusion holds, the explanation does not.** No cycle
is created, because regeneration re-invokes no handler — it builds fresh
subtrees with fresh bindings. Under
[workflow.md](../../../procedures/workflow.md) that is the "explanation
narrows" case, which takes a dated annotation rather than a supersede,
and neither is written from a task close gate: it is recorded here and
carried to **T13** beside the other normative re-verification items.

**Owner disposition (2026-08-08): a dated annotation on DD-M4-P2-001,
not a supersede.** T13 writes it and cites F5 as the measurement. The
shipped behaviour is unchanged either way, so nothing in this task's
code moves.

#### #5 — carry-forward

| Item | Evidence | Class | Re-trigger |
|---|---|---|---|
| **CF-T9-1 — CF-T7-1's collision was not reached.** The shape was built and run: focus the last `for`-generated row, then one handler body that runs `xs.drop-last()` and `xs.append(9)`. The run records the freed row at `0x1bd36699970` and the row allocated in the same message at `0x1bd36722640` — **the address was not reused**, so the anchor collision never occurred and focus fell to the domain's first surviving stop | the fixture, which prints the address relation every run | `carry-forward` — CF-T7-1 stays open, **narrowed** | Any change to allocation timing around `mutate_for_loop_subtree`, or the fixture's collision arm firing. Per [DD-V-030](../../../cross-milestone/decisions/dd-v-030-carry-forward-buildability.md) the artifact is the recorded run, and this run does **not** close the residual — it measures that M4-Phase 2's nearest expressible shape does not reproduce it |
| **CF-T9-2 — handler-body assignments are not type-checked, and a scalar `string` state cannot be written from a handler at all.** `root.n = "abc"` on an `i32` state passes `check` with exit 0; there is no `set_string` in the runtime for any right-hand side. This task inherits the absence rather than creating it, and deliberately did not add a type rule for binder reads alone | start-gate fact 5 (probe P9); `rg "fn set_string\b"` returns nothing | `finding` — **owner-dispositioned 2026-08-08, split in two** (below) | see below |
| **CF-T9-3 — §4.15's Diagnostics table carries two rows this task makes false**: "Handler inside a `for` body" and "Binder read in handler position" are still listed as rejected shapes, three paragraphs above the subsection that says they are admitted | the table and the subsection, both in §4.15 | `finding` (owner = **T13**) | T13 already re-verifies §4.15's per-item handler text; these are false statements rather than gaps, so they are the first thing that pass should reach |
| **CF-T9-4 — F2 is the only test in the suite that constrains invocation-time resolution.** W-F reddened it and nothing else | witness W-F | `carry-forward` | Any later task that touches handler attachment in the loader, the loop-scope snapshot, or `ForItemHandlerEvalContext`'s item reads. If F2 is ever deleted or narrowed, the property loses its only pin |
| **CF-T9-5 — a per-item `modal-scope` with a `dismiss` handler is now authorable and is exercised at neither runtime end.** Both gates accept `for x in xs { Box { modal-scope: true  dismiss => { … } } }`; what a scope generated per item does to entry, restore and the scope stack is T7 machinery this task did not drive | the two accept tests, which are compile/load-level only | `finding` (owner = **M4-Phase 9**, the phase that owns scope composition) | The first authored per-item overlay. No M4 app has one — T10's lightbox is a root `ZStack` branch, not a `for` body |

##### Owner disposition of CF-T9-2 (2026-08-08)

The owner reads the absence as a **gap to be filled**, not a limit to be
documented: a handler should be able to write a string state, and it
should land in M4. Two things were separated before placing it, because
they have different sizes and different owners.

**The capability → M4-Phase 5. No plan change was needed to *find* it a
home; one line was added so a reader of the plan can see the
prerequisite.** M4-Phase 5's stated approach is "one-way binding plus a
handler, matching the M3 `ToggleButton.checked` precedent". That
precedent works for `bool` because a handler can write a `bool` state —
the write M3-Phase 1 added for that purpose. **The string twin does not
exist, so the handler half of the precedent is unbuildable for a text
field**, which makes the write a strict sub-problem of the phase's own
ADR obligation rather than an addition to it. The alternatives were
checked against the phases' own scope and rejected: M4-Phase 3 says
"String concatenation stays out" and its consumers need reads (index
read, equality selection), not writes; M4-Phase 4 is scrolling /
`Image` / `fill`; M4-Phase 7 is two phases past the point where the
one-way form is needed. Recorded as
[milestone plan](../../plan.md) revision 1 under
[DD-V-026](../../../cross-milestone/decisions/plan-revision-discipline.md).

**The silence → an M4-Phase 3 pre-doc intake.** Until the capability
lands, `s = "abc"` is accepted by both gates and does nothing but log —
the authorable-accepted-silently-broken class this phase treats as
first-class. Closing it needs **no new decision**: `dsl_spec.md` §8.9
already marks `StrLit`, `Interpolation` and `StrPropRead` **binding-only**,
so a diagnostic *enforces normative text already in force* rather than
narrowing a surface. The rule is narrow — a handler assignment whose
left-hand side is a scalar `string` state; a `string[]` append stays
legitimate. No shipped `.ui` is affected (measured: no example writes a
string state from a handler).

**A sharper reading of the defect than this task's own start gate had.**
Start-gate fact 5 recorded "no section says a handler cannot write a
scalar string state". That is true of §4, and **false of §8.9**, whose
mapping table marks the string forms binding-only and whose `(assign …)`
row enumerates the admitted right-hand sides as `i32` (default), `bool`
(M3-Phase 1) and collections (M3-Phase 7), omitting string. So the
runtime matches the spec and **the compiler does not**: `s = "abc"`
lowers to `(assign s "abc")`, `s = t` to `(assign s (str-prop-read t))`,
and `s = "n is \{n}"` to `(assign s (interp …))` — three binding-only
forms placed in handler position, accepted by the loader, and rejected
only by the evaluator. Measured with a probe `.ui` through
`wasamoc build` and a throwaway evaluator probe (all three return
`TypeMismatch`). CF-T9-3's sibling item on T13's list is corrected
accordingly: the gap is not "unstated", it is "stated in §8.9 and
unenforced at three layers".

**CF-T4-1 and CF-T5-1 are touched but not closed**, and take no new row.
This task adds `for` regeneration, which is their named re-trigger, but
the shapes that would fire them need a Button-family widget under the
pointer during a reorder (CF-T4-1) or a retained id renamed in range
(CF-T5-1, closed by T7's anchor rebase). The CF-T7-1 fixture is the
nearest thing built, and it is recorded above on its own row.

#### Re-audit of the whole task list (cross-task obligation)

Read T10–T13 again at close, not only T9's own item.

- **T10** — gains what it needs and one warning. §4.19's per-item example
  (`root.selected_index = i;`) now works end to end, which is exactly the
  thumbnail-click shape T10's first bullet describes. The warning is
  CF-T9-2: a handler cannot write a scalar `string` state, so "carrying
  which thumbnail" must be carried as an **index**, not as a label. Added
  to T10's item.
- **T11** — unchanged. Touch enters `hit_test_click`; nothing here
  touches the pointer path or the message arms.
- **T12** — control A's state-level equivalent now exists (F1: two rows,
  two different reads), so control A's frame pair has something to be a
  second, independent check *of*, rather than the only evidence.
- **T13** — gains three: CF-T9-3 (§4.15's two false rows), CF-T9-2 (no
  section states that a handler cannot write a `string` state), and the
  DD-M4-P2-001 explanation divergence above. Added to T13's item.
- **Cross-task obligations** — no new ABI function was added (framing
  agreement ⑦ holds); the stretch checkpoint is unaffected.

#### Review lane

**Full independent review**, as the start gate corrected it (runtime
structural change, not the schema/IR class the preamble predicted).
Performed by a subagent that wrote none of the code, over the whole
branch diff plus the start gate, the plan item, DD-M4-P2-005 and the
normative sections. It ran the suite, ran the fixtures unskipped, and
wrote its own throwaway mutation.

Three findings, all dispositioned:

1. **The `bool[]` out-of-range arm had no test** (evidence-gap). Correct,
   and the sharpest form of it: `unwrap_or(false)` leaves the whole
   workspace green, and `false` is a *plausible* stale value rather than
   an obviously wrong one. Fixed in `bd15ec2`, re-measured as W-G.
2. **A per-item handler appending its own binder was pinned end-to-end
   for `string[]` only** (evidence-gap). Correct; `i32[]` and `bool[]`
   were covered below the checker. Fixed in `bd15ec2`.
3. **The close gate was not on the branch when the review ran**
   (process). Correct, and a deviation from
   [implementation-gates.md](../../../procedures/implementation-gates.md)
   §0's order and from T8's precedent. Recorded rather than
   rationalised: the reviewer had to reconstruct the artifacts it should
   have been able to check against, which is real cost, and it is only
   partly offset by the reviewer's read being unbiased by the lead's own
   account. The review is not re-run for this document, because the
   findings it produced are about code it did read; the sequencing itself
   goes to the retrospective.

The trap-#4 branch/test check composed into the review rather than
replacing it — findings 1 and 2 are both that check.

#### Verification

`cargo fmt --all -- --check` zero exit. `cargo build --release
--workspace` successful (18s). `cargo build --workspace` successful.
`cargo test --workspace --no-fail-fast`: **49 binaries/sections, 1,248
passed, 0 failed, 0 ignored** — against T8's 48 / 1,212, so one new test
binary and 36 tests. The new integration binary was confirmed to **run
rather than skip**: `cargo test --test per_item_handler_integration --
--nocapture` shows all six fixtures `ok` with no
`skipping …: runtime compositor unavailable` line, and the expected
`wasamo: handler error in ?.clicked: loop item \`value\` is no longer
live at its position` appears exactly once, from F5's production path.
`cargo build -p counter-rust -p gallery-rust -p bool-demo-rust`
successful — no shipped `.ui` trips the widened surface, and none uses it
yet.

---

## T10 — Gallery slice (consumer A)

### Start gate (recorded 2026-08-08, before any source edit)

Read first: [AGENTS.md](../../../../AGENTS.md),
[implementation-gates.md](../../../procedures/implementation-gates.md),
[plan.md](./plan.md) §T10 and §Cross-task obligations,
[preamble.md](./preamble.md),
[framing.md](../requirements/framing.md) §受入れ基準 / §範囲の縫い目 /
§検証方針, [constraints.md](../requirements/constraints.md),
[DD-M4-P2-002](../decisions/dd-m4-p2-002-hit-testing-and-generic-click.md),
[DD-M4-P2-004](../decisions/dd-m4-p2-004-modal-focus-scope.md),
[DD-M4-P2-005](../decisions/dd-m4-p2-005-dsl-handler-surface.md),
[dsl_spec §4.19](../../../../docs/dsl_spec.md),
[architecture §13](../../../../docs/architecture.md), the T2 / T4 / T5 /
T6 / T7 / T8 / T9 close gates above, and the T9 retrospective.

**The whole plan and log were grepped for `T10`** (the T8 corrective —
"what has another task sent to this one that this task's own item does
not say?"). Nine senders. Seven are already in the item; **two are not**,
and both change what this task builds:

- **T2** — the clip bound is landed and unit-tested; T10 is its first
  *production* consumer (the item says this).
- **T3** — T10 is the gallery's first ancestor handler and first
  non-Button handler. **The item does not say this**: the thumbnail's
  handler sits on a `Box` whose `Text` child is what a centre click
  resolves to, so the shipped app exercises the ancestor walk rather than
  target dispatch alone. **Added to what the fixtures must assert.**
- **T4** — the hover-through-the-scrim defect is fixed, so T10 starts
  from a corrected baseline.
- **T5** — T10 is the first production consumer of Tab traversal in a
  `.ui` the owner sees.
- **T6 / CF-T6-1** — a *present* but *un-entered* `modal-scope` was the
  state DD-M4-P2-004 forbids; T7 closed it, and T10 is the first shipped
  `.ui` to carry the attribute, so CF-T6-1's "bounded meanwhile: no
  shipped `.ui` carries it" ends here.
- **T7** — the throwaway probe is reverted; landing `modal-scope: true`
  and the `dismiss` handler is T10's work, not a diff to recover. T7's
  frame pair already measured that the lightbox's **`<` Button is the
  scope's first stop** on this exact tree.
- **T8 / CF-T8-5** — the key walk is upward-only, so the `key-down`
  handlers must sit at or above whatever entry focuses.
- **T8 (second half)** — `Grid` and `ZStack` now accept `clicked` at both
  gates, and a `Button` may no longer carry a widget child. The shipped
  gallery trips neither.
- **T9 / CF-T9-2** — carry an **index**, not a label.
- **T12** — controls A / C / D are the phase-level versions of shapes
  this task must make reachable, and **control C additionally discharges
  CF-T4-5**. If T10 ships a lightbox whose background click is blocked by
  something other than an authored covering widget, control C's agreement
  leg becomes unbuildable. **Added to what the fixtures must measure**
  (fact 6).

The accumulated per-task start-gate lines, answered in order:

- **T1 — new store / unit / coordinate system?** None. This task writes
  no runtime code.
- **T2 — which test pins the property this task deletes?** This task
  deletes no property. It *adds* the first shipped consumer, and the one
  existing test that reads the shipped file —
  `ir_loader_roundtrip::gallery_ui_emits_and_validates_through_runtime_loader`
  — is re-run against the edited file (fact 1).
- **T3 — was the evidence a later task needs built once here?** T12 needs
  controls A / C / D against this tree. This task builds the *state-level*
  twin of each and records which frame T12 still owes.
- **T4 — was the negative prediction this task rests on measured once?**
  The prediction is "the lightbox blocks background clicks because the
  scrim covers them". Not measured, and **the item's wording may be
  wrong** — the lightbox `Grid` is declared *after* the scrim and is also
  stretch/stretch, so the topmost widget at a background point may be the
  `Grid`. Fact 6 turns this into a fixture measurement rather than a
  reading.
- **T5 — identifiers held across messages: what is their lifetime?** One:
  the scope's captured restore target, from entry to removal. Its failure
  mode is "focus returns to the wrong widget", and the fixture asserts
  both legs (something focused beforehand; nothing focused beforehand).
- **T6 — how many gates does the rule have?** Two, and this task drives
  both from one input: `wasamoc check` (all three hosts run it) and the
  runtime loader (all three hosts hand IR to `wasamo_load_ui`).
- **T7 — which closing carry-forward is `doc-folded`, and where?**
  CF-T8-5 is folded into `deliver_key_down`'s doc comment and the
  `ROOT_BOX_KEY_DOWN_UI` fixture comment. This task consumes it and does
  not change it.
- **T8 — what has another task sent here?** Answered above.
- **T9 — how many paths does this surface split into, counted by type /
  element type / widget kind rather than by call site?** Counted here,
  and carried into the subagent brief:
  - **three host build paths**, each re-embedding IR by a different
    mechanism — Rust `build.rs` (the workspace `wasamoc` crate,
    in-process), C CMake (`add_custom_command` shelling out to
    `wasamoc.exe`), Zig `build.zig` (`addSystemCommand` + `@embedFile`);
  - **two state types written from handlers** — `i32`
    (`selected_index`) and `bool` (`is_lightbox_open`), which are two
    different evaluator paths (T9 measured that scalar/element type is
    what splits handler evaluation);
  - **four key arms ahead of the host key slot** — traversal (`Tab`),
    `arrow_on_key` (no group in this tree, so this arm must *not* fire),
    `dismiss_on_key` (`Escape`), and the authored `key_down_on_key` walk;
  - **two clip boundaries in the shipped tree** — the `ScrollView`
    around the thumbnails, and the `Grid` / root `ZStack`.

#### Ten measured facts (probes run before any source edit)

1. **Both gates accept the whole slice.** A reduced gallery carrying
   `modal-scope: true`, `dismiss`, `key-down("ArrowLeft")` and
   `key-down("ArrowRight")` on the lightbox's outer `ZStack`, a per-item
   `clicked` on the thumbnail `Box` writing
   `root.selected_index = index; root.is_lightbox_open = true;`, `clicked`
   on the `<` / `>` Buttons, and a caption `Text` bound to
   `"Photo #\{root.selected_index}"` passes `wasamoc check` (exit 0) and
   `wasamoc build` (exit 0). Placed at `examples/gallery/gallery.ui`,
   `cargo test -p wasamo-runtime --test ir_loader_roundtrip` is green — 9
   passed, including `gallery_ui_emits_and_validates_through_runtime_loader`,
   which runs the **shipped** file through `parse_ir`'s validate pass.
   Reverted with `git checkout --`; `git status` clean afterwards.
2. **All three hosts build at the branch point**, so a red host build
   after the edit is attributable to the edit: `cargo build --release
   --workspace` 19.17s; CMake configure + Release build →
   `build/t10-baseline-gallery-c/Release/gallery-c.exe`; `zig build`
   exit 0 → `examples/gallery-zig/zig-out/bin/gallery-zig.exe`.
3. **pwsh splits `-Dkey=value` on the `zig build` command line.** The
   first invocation failed with `error: no step named
   '../../target/release/wasamoc.exe'` — the option and its value became
   two argv entries. Quoting each `-D…` argument fixes it. Recorded so
   the close gate's rerun is not mis-read as a T10 regression, and
   because the same shape is a known CI hazard.
4. **No `.uic` is tracked.** `git ls-files examples/**/*.uic` returns
   nothing; the `examples/gallery/gallery.uic` in the working tree is an
   untracked local artifact of an older build. Every host regenerates IR
   from `.ui` at build time. So "host artifacts rebuild in order" is a
   **build-order** obligation, not a commit-content one — there is no
   derived file to keep in step, and none to forget.
5. **The three host READMEs restate the gallery's behaviour instead of
   citing it** — trap #3's documentation analogue — and this task makes
   one restatement false: the C and Zig READMEs both say the app shows a
   "lightbox placeholder surface". `gallery-rust`'s README is *already*
   false, describing "ten uniform 1:1 `Box` thumbnails" and an "M3
   gallery host" while the `.ui` generates 18 through `for`.
6. **Which widget blocks a background click is not known, and the plan
   item asserts an answer.** The lightbox branch is
   `ZStack { Box(scrim, stretch/stretch), Grid(stretch/stretch) }`. Under
   §4.19's reverse-order topmost rule the `Grid` is the later sibling, so
   at a point over neither photo nor button the target may be the `Grid`
   rather than the scrim. Both are inside the scope subtree, so blocking
   holds either way — but the *reason* the item gives ("its scrim is an
   authored covering widget, and it is what blocks background clicks")
   may be wrong, and T12's control C rests on it. The fixture measures
   which node the click resolves to instead of assuming.
7. **The shipped tree fits the integration client**, so the fixtures can
   run against `examples/gallery/gallery.ui` itself rather than a
   gallery-shaped miniature. At the 360x240 / 96 DPI client every
   integration file in this crate uses, the gallery's `rows: 56 1* 28`
   give a 156-DIP viewport; `padding: 12` and `item-cross-size: 88` put
   thumbnail line 1 at y≈68..156 (inside) and line 2 at y≈168..256
   (outside). One "Scroll down" click (`scroll_y += 100`) moves line 1 to
   y≈-32..56 — under the toolbar band — and line 2 into the viewport,
   which is exactly the plan's scrolled hit-testing shape. **Arithmetic,
   not measurement**: the fixture asserts the geometric relationship from
   `__arranged_rect_for_test()` before deriving any click point, per this
   crate's standing convention.
8. **The lightbox's own `Grid` is degenerate at that client** — fixed
   columns `56 + 400 + 56 = 512 > 360`, fixed rows `44 + 300 + 64 = 408 >
   240` — so **no fixture may derive a coordinate inside the lightbox**.
   Everything this task asserts inside the scope is reachable without
   one: focus-path read-back, `WM_KEYDOWN`, and the caption `Text`'s
   content. §4.19's "a focus stop that is scrolled out of view is still a
   stop" is what makes that legitimate rather than a workaround.
9. **There is no test-only getter for an `i32` state on a loaded
   window.** `ir_loader.rs` exposes `__set_*_for_test` only. The
   read-back surface is therefore the **bound caption `Text`**, read
   through `WidgetNode::__text_content_for_test` — which is also the
   phase's standing "read every result back as a rendered value"
   discipline (T7's mechanism fixture reads Button labels for the same
   reason).
10. **T7's frame pair already measured this tree's scope entry**: with
    `modal-scope: true` on the lightbox's outer `ZStack`, opening the
    lightbox moved focus off the accent "Open lightbox" Button and onto
    the `<` Button — 86.04 max-abs per channel against a tolerance of
    3.0, with `>` as the agreement leg at exactly 0. That was a throwaway
    probe on a variant build; this task lands the annotation for real, so
    **the frame T10 owes is a different one** — item identity (which
    thumbnail), not entry.

#### Normative statements that already answer this task (DD-V-031)

| Question | Where it is answered | What it fixes |
|---|---|---|
| What the thumbnail handler writes | [dsl_spec §4.19 §Per-item handlers](../../../../docs/dsl_spec.md) | The example is `root.selected_index = i;` — an index, which is also the only thing a handler *can* write (CF-T9-2: there is no `set_string`) |
| Whether clicking a `Box` thumbnail moves focus | [dsl_spec §4.19 §Focus](../../../../docs/dsl_spec.md), [architecture §13.3](../../../../docs/architecture.md) | No — the click moves focus to the nearest focusable widget **at or above** the target, and a thumbnail has none above it, so focus is left unchanged |
| What the lightbox restores focus to | [dsl_spec §4.19 §`modal-scope`](../../../../docs/dsl_spec.md) | Stated in this exact scenario: "a lightbox opened by clicking a `Box` thumbnail restores to whatever the keyboard was on beforehand — possibly nothing", and restoring to the clicked widget "requires that widget to be focusable, which arrives with the focusability attribute a later milestone adds". **The plan's Phase 5 sentence is the spec's own, not a claim this task invents** |
| Where the `key-down` handlers may sit | [dsl_spec §4.19 §`modal-scope`](../../../../docs/dsl_spec.md) + CF-T8-5 | Entry "moves focus to the scope's first stop, so … the scope's own key handlers are live without the user pressing Tab first". With the upward-only walk, handlers on the scope container are reachable and handlers below the first stop are not |
| Where the `dismiss` handler may sit | [dsl_spec §4.19 §`dismiss`](../../../../docs/dsl_spec.md), [DD-M4-P2-005 §Dismissal](../decisions/dd-m4-p2-005-dsl-handler-surface.md) | Only on a container carrying `modal-scope: true`; rejected anywhere else at both gates |
| What the lightbox `.ui` should look like | [dsl_spec §4.19 §Keyboard input](../../../../docs/dsl_spec.md) | The spec's own example **is** this lightbox: a `modal-scope: true` container carrying `dismiss`, `key-down("ArrowLeft") => { root.selected_index -= 1; }` and `key-down("ArrowRight")`. This task's `.ui` is that example on the shipped tree |
| Whether Escape and the arrows reach the handlers | [dsl_spec §4.19 §Which keys the runtime keeps](../../../../docs/dsl_spec.md), [architecture §13.2](../../../../docs/architecture.md) | `Escape` while a scope is present becomes a dismissal request on the innermost one; arrow keys **outside a `focus-group`** go to the propagation walk. The gallery declares no group, so the arrows reach the authored handlers |
| What blocks a background click | [dsl_spec §4.19 §What a scope does not do](../../../../docs/dsl_spec.md), [architecture §13.4](../../../../docs/architecture.md) | The scope confines the keyboard only; a click behind it "is stopped by a covering widget inside the scope (the occlusion rule above), not by the scope itself". **Which** covering widget is fact 6's measurement |
| What a scrolled-out thumbnail receives | [dsl_spec §4.19 §Click handling](../../../../docs/dsl_spec.md), [architecture §13.2](../../../../docs/architecture.md) | Nothing — a clipping container "bounds its whole subtree to its own rectangle for hit-testing as well as for painting" |
| Whether `selected_index` is clamped at the ends | — **not answered** | §4.19's own example is `root.selected_index -= 1;` with no guard, and M4-Phase 2 has no conditional expression to write one with |

**Two places where the normative text does not answer, and neither is an
escalation** (DD-V-031: the specification is the answer, and an
unanswered *authoring* choice is the author's):

- **Whether the gallery's toolbar should be a `focus-group`.** §4.19's
  own `focus-group` example is literally the gallery's three tab
  `ToggleButton`s, but the plan item does not ask for it, and
  [framing.md](../requirements/framing.md) §含まないもの sends "the
  group's canonical `.ui` spelling" and radio-like widgets to **M5**.
  This task does **not** add it, and records the consequence:
  `focus-group` ships with authored-surface tests and no shipped
  consumer.
- **Whether `selected_index` should be clamped.** Not expressible in this
  phase; carried forward with an owner rather than worked around.

#### Trap selection (implementation-gates §1)

```
- [ ] #1 semantic migration   - [x] #2 side effects   - [x] #3 parallel data   - [x] #4 branch tests
- [x] #5 carry-forward        - [~] #6 root cause     - [x] #7 GUI positive control
```

| # | Applies | Reason |
|---|---|---|
| 1 | **no** | This task writes no production Rust. It adds no enum variant, no IR type, no schema field and no traversal; it authors `.ui` against surfaces T6 / T8 / T9 landed and audited. **Re-decide if** the fixtures turn out to need a new runtime accessor |
| 2 | **yes** | The shipped tree gains a modal scope, so its *presence* now drives focus entry, the restore-target capture, the scope stack and the exit restore on every open and close — plus the ordinary drain → re-layout → rectangle-store chain behind two new state writes. The enumeration has a build-time half too: three host artifacts re-embed the IR by three different mechanisms |
| 3 | **yes**, in its documentation form | Three host READMEs restate the gallery's behaviour instead of citing the `.ui` (fact 5). One restatement becomes false with this change and one is already false. Closed under the #2 enumeration |
| 4 | **yes**, in its authored form | Every handler this task adds to the shipped `.ui` must have a test that **fires it**. An authored handler accepted by both gates that silently never fires is the failure class this phase exists to prevent — trap #4 with `.ui` in place of Rust |
| 5 | **yes** | Consumes CF-T8-5 and ends CF-T6-1's bound; creates at least three of its own (the unclamped index, `focus-group` with no shipped consumer, and the shipped `.ui` now being under fixture test) |
| 6 | **armed** | GUI capture and three host builds are where a "green on retry" would be tempting. Fact 3 is already one instance, handled by root cause rather than retry |
| 7 | **yes** | The deliverable **is** a rendered GUI slice. Screenshot + analysis + a positive control, and the control must discriminate *item identity* — the one claim a single frame of an open lightbox could not support |

#### Review lane

**Full independent review**, as [preamble.md](./preamble.md) predicts —
and the T9 retrospective's corrective applies, so the **ground** is
recorded and not only the lane. The preamble's ground is "GUI-render
evidence, across three hosts"; that half holds. What does **not** hold is
any runtime-structural or schema/IR trigger: this task writes no Rust
outside `tests/`. The operative grounds are therefore (a) GUI-render
evidence and (b) the first shipped consumer of three separately-landed
surfaces — `modal-scope` + `dismiss` (T6 / T7), `key-down` (T8) and the
per-item `clicked` (T9) — composing for the first time. Trap #4's
branch/test check composes in rather than replacing it.

#### Boundaries this task does not cross

1. **No runtime, compiler or IR code.** Rust changes are confined to
   `wasamo-runtime/tests/`.
2. **No `focus-group` in the shipped `.ui`** — M5 owns the group's
   canonical spelling (above).
3. **No caption photo name and no selected-thumbnail highlight** — index
   reads and equality selection are M4-Phase 3
   ([framing.md](../requirements/framing.md) §範囲の縫い目).
4. **No click-outside dismissal** — M4-Phase 9's source
   (DD-M4-P2-005 §Dismissal).
5. **No focusability attribute on the thumbnail** — M4-Phase 5, which is
   also what would make the thumbnail the restore target.
6. **No normative text is written.** §4.19 and §13 are re-verified at
   **T13**; a divergence found here is recorded for that pass.
7. **No GUI control T12 owns is claimed as closed here.** T10 takes the
   frame its own deliverable needs (item identity) and records which of
   T12's four remain.

#### Implementation shape (subagent brief, recorded)

Authored by a Sonnet subagent in two stages, with the lead measuring
every verifiable claim independently:

- **Stage 1** — `examples/gallery/gallery.ui`: `state selected_index`,
  the per-item `clicked`, the lightbox annotation plus `dismiss` and the
  two `key-down` handlers, `clicked` on the `<` / `>` Buttons, the
  caption bound to `selected_index`, and the README corrections fact 5
  names.
- **Stage 2** — `wasamo-runtime/tests/gallery_slice_integration.rs`: the
  fixtures, driven against the **shipped** `examples/gallery/gallery.ui`
  through `wasamoc` → IR → loader → real window messages, reading results
  back as rendered `Text` content and focus paths.

### Close gate (recorded 2026-08-08)

| commit | content |
|---|---|
| `d5716e8` | start gate (ten measured facts, the DD-V-031 normative table, trap selection, lane and its ground, seven boundaries) |
| `f1a6555` | the `.ui` wiring, plus the three README sentences it made false |
| `760a37c` | the first seven fixtures over the shipped `.ui`, plus the one test-only accessor they needed |
| `f030a67` | the GUI positive control: script, six frames, and their reading |
| `42ab9d2` | G8 — the two authored handlers this gate's own branch table found untested |

#### Trap selection re-decided at close (plan's standing instruction)

**Unchanged in what applies — #2, #3, #4, #5 and #7 applied, #6 stayed
armed and fired once — and changed in one row's consequence.** Trap #1
was judged non-applicable on the ground that this task writes no
production Rust. That ground **failed**, in exactly the way the row's own
"re-decide if" clause predicted, and the re-decision is recorded below
rather than left as a silent boundary breach.

**Boundary 1 was withdrawn by the lead, mid-task.** The start gate said
"Rust changes are confined to `wasamo-runtime/tests/`". The fixtures then
needed to answer "which node did this point resolve to", and no accessor
could: `ffi::__hover_target_for_test` reads `WindowState::hover`, which
`update_hover` narrows to an **enabled Button-family** target, so a
resolved `Box`, `Text` or `Grid` is invisible through it (verified in the
source, and independently by `hover_transition_integration.rs`'s own
regression fixture). The subagent reported this and worked around it with
a deduction — correctly, since the boundary was its instruction. The lead
re-decided: `WidgetNode::__resolve_topmost_for_test` forwards to the
**production** `hit::resolve_topmost`, so it is a second *caller* rather
than a second predicate implementation, and it takes DIP.

That re-decision paid for itself immediately: **the deduction it replaced
was about to be recorded as an answer, and the deduction could not have
been trusted.** The reasoning offered was "the scrim and the `Grid` both
cover the point, the `Grid` is the later sibling, therefore the `Grid`
resolves" — sound as far as it goes, but `resolve_topmost` descends into
the `Grid`'s children *before* falling back to the `Grid` itself, and at
this client the `Grid`'s track layout is degenerate, so a `Cell` or the
photo `Box` was equally available as an answer. The measurement agreed
with the deduction this time. It did not have to.

#### #2 — structural side-effect enumeration

What the annotation's *presence* pulls in, now that a shipped `.ui`
carries one, and what each is closed by:

| Derived effect | When it runs | Closed by |
|---|---|---|
| Scope pushed on the per-window modal stack | `emit::flush_layout` → `focus::sync_scopes_to_tree`, on the drain that makes `is_lightbox_open` true | G2 (focus lands on the scope's first stop, which only an entered scope produces) |
| Restore target captured | same step | G3b (a Button focused beforehand is the one focus returns to) |
| Focus moved to the scope's first stop | same step | G2, and the GUI frame — the `<` Button paints the indicator and `>` does not |
| Traversal root narrowed to the scope | while the scope is present | G4's part 2 pair: the arrow reaches the authored handler while open and the host key slot while closed |
| Scope dropped and focus restored | the drain that makes `is_lightbox_open` false | G3a (nothing focused → `None`), G3b (restoration beats structural succession — **only after the review corrected it**; as first written this leg could not tell the two apart, see §Independent review finding 1) |
| Two state writes → drain → re-layout → the arranged-rectangle store | every thumbnail click and every arrow press | G6 (rectangles re-read after the scroll write, never cached across it), G1/G4 (the caption re-renders) |
| The three host IR embeddings | build time, three different mechanisms | Rebuilt in order and verified below |
| The prose that restates the tree | — | #3 |

**One effect was enumerated and deliberately left unclosed**: nothing in
this task exercises what happens if the lightbox's subtree is removed
while the pointer is inside it (CF-T4-1's shape). No M4 gesture produces
it — Escape and the `x` Button both close from a keyboard or a click that
is itself the last event of the message.

#### #3 — parallel data, in its documentation form

The three host READMEs restate the gallery's behaviour instead of citing
the `.ui`. Enumerated at the start gate (fact 5) and closed here:

| Document | Statement | Disposition |
|---|---|---|
| `examples/gallery-c/README.md` | "…and lightbox placeholder surface" | Made false by this change. Rewritten to point at `gallery.ui` and name what the lightbox does |
| `examples/gallery-zig/README.md` | same sentence | same |
| `examples/gallery-rust/README.md` | "M3 gallery host", "ten uniform 1:1 `Box` thumbnails", "seven on the first line and the remaining three onto a second" | **Already false** before this task — the `.ui` generates 18 through `for`. Corrected while the file was open, because shipping a known-false sentence next to an edit is worse than the scope discipline it would preserve |
| `examples/gallery/gallery.ui`, status strip | "18 placeholders - Image and hit-testing are M4" | Made false: hit-testing landed. Reworded |
| `examples/gallery/gallery.ui`, lightbox caption | "IMG 001  2026-04-12" | Replaced by the bound `Photo #\{root.selected_index}`, which is the visible result the slice needs |

**No `.uic` is tracked** (start-gate fact 4), so there is no derived IR
artifact to keep in step — the "host artifacts rebuild in order"
obligation is a build-order one, and it is discharged by the three-host
rebuild below rather than by a committed file.

#### #4 — branch table, in its authored form

This task adds no Rust branch. What it adds is **authored surface in a
shipped `.ui`**, and the trap is the same one with `.ui` in place of
Rust: a handler both gates accept and nothing ever fires is the failure
class the phase exists to catch. Every handler and attribute this task
wrote, with the test that fires it directly:

| Authored in `examples/gallery/gallery.ui` | Fired by |
|---|---|
| per-item `clicked` on the thumbnail `Box`, reading the bare `index` binder | **G1** (rows 0 and 2 → two different captions; the same row twice → the same one), G3a, G3b, G4, G6(b), G8 |
| `modal-scope: true` on the lightbox's outer `ZStack` | **G2** (entry moves focus to the `<` Button), **G3b** (restoration beats structural succession — after the review's correction; see finding 1), **G4 part 2** (the arrow is consumed only while the scope is present), **G5** (a background click is blocked only while it is present) |
| `dismiss => { root.is_lightbox_open = false; }` | **G3a** and **G3b** — Escape closes, and it closes because the *handler* writes the state, which is what "the runtime delivers the request and never acts on it" means |
| `key-down("ArrowRight")` on the scope container | **G4 part 1** (caption `#2` → `#3`); GUI frame `k6` |
| `key-down("ArrowLeft")` on the scope container | **G4 part 1** (`#3` → `#1` over two presses); GUI frames `k5` / `k4` |
| `clicked` on the lightbox's `<` Button | **G8** |
| `clicked` on the lightbox's `>` Button | **G8** |
| the caption `Text` bound to `"Photo #\{root.selected_index}"` | every fixture that reads a caption back, and every GUI frame — it is the read-back surface, so a binding that did not update would redden all of them |

**The table is what found the gap.** The `<` / `>` handlers had **no**
firing test when it was first written, and the fixture that fires them
(G8) exists because of that, not before it. Writing the table from the
diff rather than from memory is the T9 retrospective's own corrective,
applied here for the first time — and it caught something on its first
use.

**Branches deliberately not fired, with the path measured** (the T8
corrective: a negative claim is true of one path until every path is
enumerated):

- **The lightbox's `x` Button's `clicked`.** Not a branch this task
  added — it has been in `gallery.ui` since M3-Phase 6 and is unchanged
  here. It is *reachable* at G8's 560x320 client (column 3, row 1 →
  `x≈487..528`, `y≈4.7..39.3`), and it is unreachable at the 360-wide
  client the other seven fixtures use, for the same column-3 reason `>`
  is. Recorded as CF-T10-6 with T12 as its owner rather than fired here,
  because the closing path this task *did* add — `dismiss` — is what G3
  pins, and the `x` route is M3 surface.
- **`focus-group`.** Not authored anywhere in the shipped tree, by
  decision (CF-T10-2). The attribute's own branches are fired by
  `focus_core`'s unit tests and T7's fixtures; what has no consumer is
  the *authoring*, which is the finding rather than an untested branch.

#### The measurement that a table cannot make

Two claims in this task's own plan item were **predictions stated as
facts**, and both were checked rather than inherited:

- "its scrim … is what blocks background clicks" — **false as written**.
  At a background point `__resolve_topmost_for_test` returns the
  lightbox's own `Grid` (`[1, 1]`, kind `Grid`), because it is declared
  after the scrim and is also stretch/stretch. Both sit inside the scope
  subtree, so §4.19's rule — "stopped by a covering widget inside the
  scope, not by the scope itself" — holds exactly as written, and the
  *specification* needs no change. The plan item and T12's control C
  description do.
- "the thumbnail's handler … the ancestor walk" — **true, and now
  measured**: the click at thumbnail 0's centre resolves to
  `[0, 2, 0, 0, 0, 0, 0]`, kind `Text`, whose parent prefix is the `Box`
  that carries the handler. The shipped gallery is the first production
  exercise of T3's walk rather than of target dispatch alone.

#### #5 — carry-forward

| Item | Evidence | Class | Re-trigger |
|---|---|---|---|
| **CF-T10-1 — the overflowing toolbar swallows every click aimed at a tab button.** At a 360 DIP client both toolbar `HStack`s overflow their `Grid` columns *toward each other* — right at `x=[17.8, 360.0]`, left at `x=[0.0, 245.0]` — and each tab `ToggleButton`'s own centre resolves to a scroll `Button` instead: `All` → `Scroll down`, `Albums` → `Scroll down`, `Favorites` → `Scroll up`. A click at `Albums`' centre leaves all three `checked` values unchanged. This is the **input-side half** of an observation [constraints.md §6](../requirements/constraints.md) records only as visual, and whose *semantics* the owner sent to M4-Phase 4 | **G7**, which prints all of it and asserts the two negative resolutions as a tripwire | `finding` (owner = **M4-Phase 4**, which owns `Row`/`HStack` overflow semantics) | G7 going red — which would mean the overflow was fixed, not that something broke; its assertion messages say so. Also any capture or fixture that clicks a toolbar tab at a narrow client |
| **CF-T10-2 — `focus-group` had no shipped consumer.** **Closed by the owner's disposition (2026-08-08): the gallery's tab strip carries it.** The row's original ground was wrong and is corrected rather than restated: [framing.md](../requirements/framing.md) §含まないもの sends **M5's components** and their canonical spelling to M5, and DD-M4-P2-005 subsequently decided to *ship* the attribute, syncing it into §4.19 at Moment 1 — so authoring it in the gallery pre-empts nothing. The real reason it was omitted is that T10's plan item did not ask for one | The attribute is on the toolbar-left `HStack`; **G10** fires both halves of what it means | `finding` → **closed** | — |
| **CF-T10-3 — `selected_index` is unclamped at both ends, and M4-Phase 3 must answer two questions rather than one.** `key-down("ArrowLeft")` at 0 writes −1 and `ArrowRight` at 17 writes 18; the caption renders it. M4-Phase 2 has no conditional expression to guard with, and §4.19's own example has the same shape, so this is inherited rather than introduced. **Owner-settled 2026-08-08: ship as is.** The two questions for the receiving phase are: (a) **what an out-of-range index read yields** — unavoidable, because [the milestone plan](../../plan.md) makes the lightbox caption an index read, so the phase cannot ship without defining it; and (b) **whether a predicate can appear in handler position at all** — *avoidable*, because the phase's stated scope is collection reads, per-item conditional **rendering** and equality selection, none of which obviously gives a handler body an `if`. If (b) is answered "no", then guarding the write is still not expressible after Phase 3. **The owner's expectation, stated 2026-08-08, is that Phase 3 delivers both: an out-of-range index read is a runtime diagnostic that fails rather than degrading, and a handler can guard the write — so (b) is to be answered "yes", and it is a theme for that phase's pre-doc framing rather than a limit to record.** Carried because it is not derivable from the documents: [the milestone plan](../../plan.md)'s M4-Phase 3 entry lists collection reads, per-item conditional *rendering* and equality selection, none of which implies a predicate in handler position, so a framing written from the plan alone would not reach the owner's expectation. Phase 3 decides whether that is inside its stated scope or a plan revision | The `.ui`, and G4 / the GUI arrow legs which step within range deliberately | `finding` (owner = **M4-Phase 3**, pre-doc framing input) | The phase that lands index reads and equality selection. Equality selection degrades safely on its own (an out-of-range index simply matches no thumbnail); the caption does not. **Precedent for the diagnostic half**: T9 landed `EvalError::ItemOutOfRange` for the analogous handler-position read, where the runtime logs a named handler error and skips the assignment — Phase 3 can follow that shape or diverge from it deliberately |
| **CF-T10-4 — the shipped `.ui` is now under fixture test, and that coupling is deliberate.** `gallery_slice_integration.rs` reads `examples/gallery/gallery.ui` with `include_str!` and finds its nodes by label and rendered text rather than by hard-coded child indices, so a later edit reddens an assertion with a readable message instead of silently measuring a different node | The file's `find_path` / `find_text_path` / `find_button_path` helpers, used by every fixture | `carry-forward` | Any task that edits `examples/gallery/gallery.ui`. Renaming a Button label or a thumbnail's text is enough; the fixtures name the string they look for |
| **CF-T10-6 — the lightbox's `x` Button was the one authored closing route with no test.** It predates this task (M3-Phase 6) and is unchanged by it, so it was never a trap-#4 branch; it was also unreachable at the 360-wide client the other fixtures use and reachable at G8's 560x320 one, so the omission was a choice rather than an impossibility. **Closed by the owner's disposition (2026-08-08): fire it now, while it is cheap.** The judgement that had ruled it out — "pre-existing, therefore out of scope" — was a correct line drawn around a cost too small to be worth the line | **G9**, which resolves `x`'s own centre to `x`'s own path before clicking it | `finding` → **closed** | — |
| **CF-T10-5 — `WidgetNode::__resolve_topmost_for_test` is a second caller of the production resolver, and the first one outside the runtime.** It cannot drift from production behaviour, because it *is* the production function — but it takes DIP, and a caller that hands it physical pixels gets a plausible wrong answer rather than an error | The accessor's doc comment, which states the unit and the reason | `carry-forward` → this ledger, and `doc-folded` → the accessor | Any later fixture using it. The trap is the one M4-Phase 1 T5 and this phase's T2 spent a migration closing, so it is named at the seam rather than left to be rediscovered |

**CF-T8-5 is consumed and stays open as written.** The gallery's
`key-down` handlers sit on the `modal-scope` container, above the `<`
Button that entry focuses, so the upward-only walk reaches them — which
is what T8's row predicted and what G4 now measures on the shipped tree.
The constraint itself does not close: it binds every later `.ui` that
declares a key handler.

**CF-T6-1's bound ends here.** Its "bounded meanwhile: no shipped `.ui`
carries the attribute" is no longer true — this one does — and the state
it bounded (a present but un-entered scope) was already closed by T7.
Recorded so the row is not left implying a bound that no longer exists.

#### #6 — deterministic failure, root-caused rather than re-rolled

One fired, in the small. The first `zig build` of the session failed with
`error: no step named '../../target/release/wasamoc.exe'`. The cause is
not the build: **pwsh splits `-Dkey=value` into two argv entries**, so
`zig build` saw the value as a step name. Quoting each `-D…` argument
fixes it, and every `zig build` in this task's record is the quoted form.
Recorded because the failure looks like a broken host build and is not,
and because the same shape is a known CI hazard.

Nothing else failed non-deterministically. The GUI capture was run twice
— once before the arrow legs existed and once after — and both runs
produced identical numbers on the legs they share (79 / 0 / 0 caption and
photo-box pixels, 525379 / 0 whole-client), which is a stability
observation rather than a re-roll: the second run existed to add legs,
not to turn a red green.

#### #7 — GUI evidence

Script, frames and reading:
[capture-t10-item-identity.ps1](./evidence/capture-t10-item-identity.ps1),
[evidence/t10-frames/](./evidence/t10-frames/). One release build, one
launch, one window geometry throughout; display scale **1.25** (120 DPI),
**client** rectangle 982x703 px = 785.6x562.4 DIP; two frames per set,
within-set jitter **0**; Escape and the arrows sent as **real key
presses**, with foreground activation earned by a click and read back,
and the path used printed into the run output because the frames look
identical either way.

| Leg | Region | Differing px |
|---|---|---|
| **difference** — caption, thumbnail 0 vs 3 | caption | **79** |
| **agreement** — caption, thumbnail 0 twice | caption | 0 |
| **agreement** — `[photo]` box, thumbnail 0 vs 3 | photo | 0 |
| **difference** — caption, `ArrowRight` from #5 | caption | **69** |
| **difference** — caption, `ArrowLeft` x2 from #6 | caption | **97** |
| **agreement** — `[photo]` box across the arrow keys | photo | 0 |
| **difference** — closed vs open | whole client | **525379** |
| **agreement** — closed before vs closed after Escape | whole client | 0 |

**Read as images, not only as numbers**, by the lead: the caption reads
`Photo #0`, `Photo #3`, `Photo #5`, `Photo #6` and `Photo #4` in the
frames the numbers pair up. A single frame of an open lightbox could not
have supported any of this — an implementation that opened it from any
click, or that captured a fixed index when the subtree was generated,
renders the same picture.

**Two things were visible that were not designed into the control**, both
worth more than the leg they came with:

- In every open frame the `<` Button is **amber** and `>` is neutral
  grey. That is the scope entry's focus indicator, painted on a node the
  same drain created — the rendered half of what G2 asserts as state, and
  the same discriminator T7's frame pair measured on a throwaway probe
  build. It is now on a committed tree.
- In the closed frame the toolbar's six controls are **cleanly
  separated**: `All` / `Albums` / `Favorites` at the left, `Scroll down` /
  `Scroll up` / `Open lightbox` at the right, with a gap between the
  groups. At this client (785.6 DIP) the overlap G7 measures at 360 DIP
  does not occur, which is the direct evidence that CF-T10-1 is
  width-driven and not present at the size the owner's smoke uses.

This is the assistant baseline and does **not** replace the owner's
human-visible smoke ([CLAUDE.md §Testing rules](../../../../CLAUDE.md)).

#### Re-audit of the whole task list (cross-task obligation)

Read T11–T13 again at close, not only T10's own item.

- **T11** — gains one usable fact and one warning. The fact: the shipped
  gallery is now driven by fixtures that click through
  `hit_test_click`, so a touch arm that reaches the same seam has a
  ready comparison target — the same click point through two message
  families should reach the same handler. The warning: **whether a touch
  contact moves focus is still T11's explicit decision** (T5's and T7's
  re-audit lines, unchanged), and the gallery now has a modal scope, so
  a touch contact that *did* move focus would have a scope to move it
  inside. Nothing in this task decides that.
- **T12** — three changes, all written into its item. Control A is
  **already taken here**, at scale 1.25, in exactly the shape that row
  describes, so what T12 owes on that row is a decision (cite T10's
  frames, or re-take the set at one sitting) rather than a repeat.
  Control C's blocker is the lightbox `Grid`, not the scrim, so the row's
  description is corrected. And the capture-width rule gains an
  input-side reason: where the toolbar overflows, a control that clicks a
  tab button measures the overlap instead of the property it is aimed at.
- **T13** — gains one re-verification item: §4.19's "every widget with a
  visual is a candidate" is **accurate**, and its consequence — that a
  *layout container* is such a widget, so a non-clipping one takes clicks
  across the part of its rectangle that overflows its parent's cell — is
  unstated. T13 decides only whether the sentence is added; the overflow
  semantics stay M4-Phase 4's. Nothing this task built makes any
  normative statement false, which is why T13 gains an addition rather
  than a repair.
- **Cross-task obligations** — no new ABI function (framing agreement ⑦
  holds: the only new symbol is a `#[doc(hidden)]` Rust accessor, not a C
  entry point, and `bindings/c` is untouched). The stretch checkpoint is
  unaffected.

#### Review lane

**Full independent review**, as the start gate corrected the *ground* for
(GUI-render evidence, plus three separately-landed authored surfaces
composing in a shipped `.ui` for the first time — not the
runtime-structural trigger the preamble's row implies, since this task's
only production Rust is one test-only accessor). Performed by a subagent
that wrote none of the code, over the whole branch diff plus the start
gate, the plan item, DD-M4-P2-002 / 004 / 005 and §4.19.

**The close gate was written before the review**, restoring
[implementation-gates.md](../../../procedures/implementation-gates.md)
§0's order after T9 inverted it — the T9 retrospective's corrective,
applied at the first opportunity.

#### Verification

`cargo fmt --all -- --check` zero exit; `git diff --check` clean.
`cargo build --release --workspace` successful; `cargo build --workspace`
successful. `cargo test --workspace --no-fail-fast`: **50
binaries/sections, 1,256 passed, 0 failed, 0 ignored** (against T9's
49 / 1,248 — one new test binary and eight tests), and the new binary was
confirmed to **run rather than skip**: `--nocapture` shows all eight
fixtures `ok` with no `skipping …: runtime compositor unavailable` line,
and prints the measurement block quoted throughout this gate.

**One "failure" in this gate's own tooling was root-caused rather than
re-rolled.** The lead's aggregation command reported `FAILURES PRESENT`
against a suite whose every section said `0 failed`. Cause: PowerShell's
`-match` is case-insensitive, so the guard `$out -match 'FAILED'` matched
the word `failed` in every `0 failed` line. The suite was never red. A
small instance, and the reason it is written down is that the tempting
move — re-run and see it pass — would have produced the same wrong
signal every time.

**Three hosts, rebuilt in order**, which is what "host artifacts rebuild
in order" means when no `.uic` is tracked:

| Host | Mechanism | Result |
|---|---|---|
| Rust | `build.rs`, workspace `wasamoc` crate, in-process | `cargo build --release --workspace` → `gallery-rust.exe`, the binary the GUI control launched |
| C | CMake `add_custom_command` shelling out to `wasamoc.exe` | configure + Release build → `gallery-c.exe`; the regenerated `gallery.uic` is 11,028 bytes and contains `prop modal-scope = true`, `on key-down("ArrowLeft")`, and `(assign selected_index (index-read index))` |
| Zig | `build.zig` `addSystemCommand` + `@embedFile` | `zig build` exit 0 → `gallery-zig.exe`; the `.zig-cache` `gallery.uic` is the same 11,028 bytes with the same content |

Each was also built at the branch point before any edit (start-gate fact
2), so a red build would have been attributable to this task rather than
inherited.

#### Independent review and its remediation

**Full independent review**, performed by a subagent that wrote none of
the code, over the whole branch diff plus the start gate, the close gate,
the revised plan item, DD-M4-P2-002 / 004 / 005 and §4.19 / §13. It ran
the suite, re-derived the branch table from the `.ui` diff rather than
from the table, viewed the frames, and wrote its own throwaway mutation.
**The close gate was on the branch when it ran** — the T9 retrospective's
corrective, applied at the first opportunity, and the reviewer used it as
the thing to check rather than as something to reconstruct.

Two findings, both dispositioned in `586f12d`:

1. **G3 leg (b) did not pin what this gate claimed it pinned.**
   *(evidence gap, and the gate's own overclaim.)* The leg Tabbed once,
   landing on `All` — the domain's **first** stop — so the captured
   restore target and the answer structural succession would give were the
   **same node**. A runtime that ignored the capture entirely and always
   succeeded to the first stop would have passed. This gate's #2 and #4
   tables both credited G3b with "restoration beats structural
   succession", which it could not show.

   **Measured, not argued**: with `focus::sync_scopes_to_tree` mutated to
   substitute `t.initial_focus(s)` for the captured restore target, G3
   stayed **green** (8 passed) while `modal_scope_integration.rs`'s
   `exit_restores_and_restoration_beats_succession` went **red**. T7's
   fixture had avoided exactly this coincidence *and said so in its own
   doc comment*; T10 reintroduced it on the shipped tree without reading
   that sentence.

   Fixed by Tabbing **twice** and restoring to the second stop, with
   `second_stop_path != first_stop_path` asserted **by name** so a later
   `.ui` edit that made them coincide reddens the leg rather than
   silently costing it its power. Re-measured under the same mutation
   **with leg (a) isolated**, because leg (a) also happens to catch that
   particular mutation and would otherwise have masked the measurement:
   leg (b) alone goes red, expecting `Albums` at `[0, 0, 1]` and getting
   `All` at `[0, 0, 0]`. Mutation and isolation both reverted, tree
   confirmed clean.

2. **`__resolve_topmost_for_test`'s doc comment miscounted.**
   *(documentation inaccuracy.)* It called itself "a second caller" and
   named two production call sites; there are three —
   `hit_test_click`, `update_hover` and `focus::focus_on_click`. The
   substantive claims (a caller of the one resolver rather than a second
   implementation of it; DIP rather than physical pixels) were accurate
   and are unchanged. Corrected here and in the two places in the test
   file that repeated the same count.

**What the review confirmed rather than found**, listed because a review
that only reports findings leaves the rest unmeasured: the branch table
rebuilt from the `gallery.ui` diff matches this gate's, with nothing
missing or misattributed; every measurement quoted in this gate
reproduces exactly (G1's `[0, 2, 0, 0, 0, 0, 0]` / `Text`, G5's
`[1, 1]` / `Grid`, G7's three resolutions, 50 sections / 1,256 passed at the time it ran);
the six retained frames show the captions, the amber `<` and the
unoverlapped toolbar this gate reads out of them; the capture discipline
holds; `tests/common` is untouched; and no `.uic` is tracked.

**The trap-#4 branch check composed into the review** rather than
replacing it — finding 1 is that check applied to a *leg* rather than to
a branch, which is the form it takes when the authored surface is a `.ui`
and the discipline being checked is whether a test can tell right from
wrong.

**One observation from how the review was conducted, kept for the next
one.** The reviewer applied its throwaway mutation to
`wasamo-runtime/src/focus.rs`, measured with it, and reverted it with
`git checkout --`. A revert outside the editing tool is an external file
change as far as the harness is concerned, so a system-reminder followed:
the file had been modified, the change was intentional, and the user need
not be told. **The reviewer did not take that message's word for the
file's state.** It re-read the ground truth instead — `git status
--short`, `git diff --stat` and `git diff -- wasamo-runtime/src/focus.rs`,
all three empty — and reported the result. The lead re-verified
independently, and the later clean rebuild agrees: the file is
byte-identical to `HEAD`, and nothing from the mutation reached a commit.

Two things are worth carrying forward, rather than the episode:

- **A claim about repository state is settled by measuring the
  repository**, not by deciding whether a message asserting it is
  trustworthy. That is both the cheaper move and the conclusive one, and
  it is the one the reviewer took.
- **An instruction not to mention something can reach a role whose work
  *is* to report.** Text arriving inside a tool result describes harness
  state; it carries no authority from the owner and does not outrank the
  duty to report on the work under review. Where the two meet, the duty
  wins.

#### Owner disposition of the four questions this gate raised (2026-08-08)

| Question | Disposition |
|---|---|
| **A — should the gallery's tab strip be one Tab stop?** | **Yes.** `focus-group: true` lands on the toolbar-left `HStack`. CF-T10-2 closes |
| **B — is a caption reading `Photo #-1` acceptable until M4-Phase 3?** | **Yes, ship as is.** CF-T10-3 stays open, sharpened into the two questions the receiving phase must answer |
| **C — should the `x` close Button gain a test now?** | **Yes.** G9 fires it. CF-T10-6 closes |
| **D — does the toolbar overlap change owner?** | **No — and M4-Phase 4 gains it as a named deliverable** ([milestone plan](../../plan.md) Revision 2). The owner reads the routing behaviour as per specification, which is right; what M4-Phase 4 owes is the *layout* rule, with "overlapping is acceptable" among its answers |

**A's ground was wrong in this gate's first draft and is corrected rather
than defended.** It cited [framing.md](../requirements/framing.md)
§含まないもの as sending the group's spelling to M5. That row sends **M5's
components** — RadioButton, ComboBox — and the canonical spelling that
comes with them. The *attribute* was subsequently shipped by
DD-M4-P2-005 and synced into §4.19 at Moment 1, so authoring it in the
gallery pre-empts nothing. The honest reason it was omitted is that T10's
plan item did not ask for one.

**Both additions are authored surface, so both arrive with a test that
fires them** — trap #4 in its authored form, applied to work added after
the branch table was written:

- **G10** — the attribute means two things and the fixture asserts both:
  the strip is **one** Tab stop (one Tab focuses `All`; a **second** Tab
  reaches `Scroll down`, leaving the group rather than stepping to
  `Albums`), and **arrows move inside it** (`ArrowRight` from `All`
  reaches `Albums`, `ArrowLeft` returns). Asserting only the first would
  accept a runtime that made the strip unreachable rather than grouped.
- **G9** — the `x` route, as its own fixture rather than a leg on G8,
  whose claim is "one bound value, two authored routes" and stays single.
  It needs G8's 560x320 client for the same reason G8 does: measured,
  `x` is at `x∈[488.8, 527.2]`, `y∈[4.7, 39.3]`, and its centre resolves
  to its own path `[1, 1, 4]`.

**One fixture's printed measurement changed, and the assertion did not.**
G3b's second Tab stop is now `Scroll down` at `[0, 1, 0]` instead of
`Albums` — Tab leaves the group. G3b asserts only that the second stop
**differs from the first**, so it passed unchanged. That is the
label-not-path discipline the file was written with, paying for itself:
had the fixture named `"Albums"`, an approved one-line `.ui` change would
have reddened it for no reason.

**The GUI evidence was re-taken rather than argued to be still valid.**
§4.19 says neither attribute changes layout, but a `.ui` change means the
committed frames came from a binary built before it — the
host-artifact staleness [AGENTS.md §Build ordering](../../../../AGENTS.md)
warns about, in its `.ui` form. After `cargo build --release --workspace`,
the capture was re-run and every leg reproduced to the pixel (79 / 0 / 0,
69 / 97 / 0, 525379 / 0), and the six retained frames are **byte-identical
by SHA-256** to the recapture. So the committed evidence stands, and the
run doubles as a measurement of §4.19's "neither attribute changes
layout" on the shipped app.

**Three hosts rebuilt again after the `.ui` change**: the C host's
regenerated `gallery.uic` is 11,076 bytes (from 11,028) and carries
`prop focus-group = true`; the Zig host rebuilt from the same source.

**Suite after both additions**: 50 binaries/sections, **1,258 passed, 0
failed, 0 ignored** — exactly +2 for G9 and G10, with no new binary since
both joined the existing file.

#### D — the toolbar overlap, and what is actually undecided

The owner reads the behaviour as per specification. **That is right about
the routing rules and does not settle the question**, because the
question sent to M4-Phase 4 was never about routing:

- **Routing**: nothing to fix. §4.19 says every widget with a visual is a
  candidate and later siblings win; the runtime does exactly that. G7
  measures it and asserts it.
- **Layout**: undecided. "What a `Row` does when its children do not fit"
  has no decision behind it — the current answer, *overlap and let the
  later sibling take the clicks*, is what the implementation happens to
  do. Wrapping, clipping, shrinking and scrolling were all available and
  none was chosen. That is the question
  [framing.md](../requirements/framing.md) agreement ④ sent to
  M4-Phase 4, and **"overlapping is fine" is a legitimate answer to it.**

**The receiving phase has no record of receiving it**, which is the part
worth acting on. The disposition lives in this phase's framing ④ and
[constraints §6](../requirements/constraints.md); the M4 milestone plan's
M4-Phase 4 entry lists scrolling, the scrollbar, `Image` and direct-value
`fill`, and does not mention `Row` overflow; and
[M4-Phase 1's handoff](../../phase-1/implementation/handoff.md) still
names **M4-Phase 2** as the landing place, which framing ④ superseded. A
reader following either document lands somewhere the question is not.

**Owner disposition (2026-08-08): M4-Phase 4 specifies it.** Not "decide
whether to fix it", and not an internal decision either — **a rule
written into `dsl_spec.md` saying what a `Row` / `HStack` does when its
children do not fit**, with "overlapping is acceptable" still an
available content. The obligation is the stating: the runtime already
behaves one way, and the gap is that no document says so.

Landed as [milestone plan](../../plan.md) **Revision 2** under
[DD-V-026](../../../cross-milestone/decisions/plan-revision-discipline.md),
which is the instrument rather than this phase's handoff for the reason
Revision 1 recorded: a handoff carries one hop, and M4-Phase 4 is two
from here — it would have to survive M4-Phase 3 forwarding a finding that
is not M4-Phase 3's. The revision adds the deliverable to M4-Phase 4's
own description, states the input-side half as measured, and records that
[M4-Phase 1's handoff](../../phase-1/implementation/handoff.md) still
names M4-Phase 2 as the landing place — superseded by framing ④ and left
unedited, because a closed phase's handoff is its record of what was
known then.

CF-T10-1 therefore stays a `finding` with M4-Phase 4 as its owner, now
with a home in that phase's own scope rather than only in this phase's
ledger. G7 remains the tripwire either way.

---

## T11 — Touch

### Start gate (recorded 2026-08-08, before any source edit)

Read first: [AGENTS.md](../../../../AGENTS.md),
[implementation-gates.md](../../../procedures/implementation-gates.md),
[plan.md](./plan.md) §T11 and §Cross-task obligations,
[preamble.md](./preamble.md),
[framing.md](../requirements/framing.md) §オーナー合意の記録 (agreement ⑥)
and §検証方針 §タッチ,
[constraints.md](../requirements/constraints.md) §7 / §8 / §10,
[DD-M4-P2-001](../decisions/dd-m4-p2-001-event-routing-model.md) §Touch,
[architecture.md §12.3 / §13.2](../../../../docs/architecture.md),
[dsl_spec.md §4.19](../../../../docs/dsl_spec.md),
[verification-environments.md](../../../../docs/notes/verification-environments.md)
§Verification kinds / Observation 3 / Observation 4, the T2–T10 close
gates above, and the T10 retrospective.

**The whole plan, log, decision set and retrospective set were grepped
for `T11`.** Nine senders, plus one homonym that has to be separated
before the list is usable:

- **The homonym.** [constraints.md §6](../requirements/constraints.md)
  and [framing.md](../requirements/framing.md) agreement ④ both say
  "T11" and mean **M4-Phase 1's T11** (the owner's toolbar-overlap
  observation), not this task. Nothing in either row is addressed here.
- **T2** — two rows. `hit_test_click` is already message-family-agnostic,
  so the walk needs no change; and the carry-forward *"a test that drives
  a click must lay its tree out"* names this task explicitly, so the
  fixture goes through `wasamo_load_ui` and a real client-extent message
  rather than hand-pinning geometry.
- **T4 / CF-T4-4** — hover is wired to the three mouse messages only. A
  pointer arm inherits nothing, and **whether a touch contact should
  paint hover at all is this task's explicit decision** rather than
  something to inherit by omission.
- **T5, restated by T7 and T10** — the same shape for focus: a
  `WM_POINTER*` arm would have to call `focus_on_click` itself. **This
  task's explicit decision.** T10 adds the consequence: the shipped
  gallery now has a modal scope, so a touch contact that *did* move focus
  would have a scope to move it inside.
- **T10** — the shipped gallery is now driven by fixtures that click
  through `hit_test_click`, so a touch arm reaching the same seam has a
  ready comparison target: the same point through two message families
  must reach the same handler.
- **T6 / T8 / T9** — recorded as unaffected, and re-checked: correct.
  Nothing in the authored surface distinguishes input family.
- **T12** — its four controls are mouse- and keyboard-driven; no control
  is a touch control. This task therefore owes T12 nothing and takes
  nothing from it.

The accumulated per-task start-gate lines, answered in order:

- **T1 — new store / unit / coordinate system?** **Yes, and it is the
  finding that reshapes this task.** The `WM_POINTER*` family carries
  **screen** coordinates where the mouse family carries client
  coordinates (fact 3). The pointer arm therefore needs a screen-to-client
  translation ahead of the division `pointer_physical` +
  `DipScale::pair_to_dip` already perform. No new *retained* store.
- **T2 — which test pins the property this task deletes?** This task
  deletes no property; it adds arms. The property it must not break is
  T2's audit conclusion (zero `visual_rect` readers on the input path) —
  the new arms read no Visual.
- **T3 — was the evidence a later task needs built once here?** No later
  task needs touch evidence: T12's controls are mouse / keyboard and T13
  is documentation. What this task owes forward is the **stated limit**
  and a taxonomy row, not a fixture.
- **T4 — was the negative prediction this task rests on measured once?**
  The prediction is DD-M4-P2-001's *"handling the pointer message
  suppresses the mouse messages the system would otherwise synthesize"*.
  Measured before writing any runtime code, in both legs (fact 4).
- **T5 — identifiers held across messages: what is their lifetime?**
  None. The arms hold nothing between messages; the OS pointer id is read
  from nothing and stored nowhere.
- **T6 — how many gates does the rule have?** Zero. This task adds no
  authored surface, so neither `wasamoc check` nor the loader changes.
- **T7 — which closing carry-forward is `doc-folded`, and where?**
  CF-T4-4 and the T5 / T7 / T10 focus line are closed here, and the
  closure is folded into the new arms' doc comments so the next reader of
  `wnd_proc` sees the decision beside the code rather than in a ledger.
- **T8 — what has another task sent here?** Answered above.
- **T9 — how many paths does this surface split into, counted by type /
  element type / message rather than by call site?** Counted:
  **five `WM_POINTER*` members a stationary or moving contact produces**
  (`ENTER`, `DOWN`, `UPDATE`, `UP`, `LEAVE`), **two coordinate spaces**
  (screen on the pointer family, client on the mouse family), and **two
  delivery outcomes per contact** (claimed, so no promotion; unclaimed,
  so promotion), which is what the single-delivery assertion is about.

#### Eight measured facts (probes run before any source edit)

The probe is a standalone C#-in-PowerShell window with an instrumented
`WndProc`, run in two modes that differ **only** in whether the pointer
messages are returned without calling the default window procedure. It
touches no wasamo code, so every fact below is a property of Windows on
this machine rather than of this runtime.

1. **Touch injection is available on this dev box, by both APIs.**
   `InitializeTouchInjection(1, TOUCH_FEEDBACK_NONE)` + `InjectTouchInput`
   and `CreateSyntheticPointerDevice(PT_TOUCH, 1, …)` +
   `InjectSyntheticPointerInput` both succeeded and both delivered
   messages. The plan's *"if the probe finds injection infeasible
   everywhere"* branch therefore does **not** fire, and no swap to the
   weaker posted-frame-only claim is proposed.
2. **A stationary tap delivers `WM_POINTERENTER`, `WM_POINTERDOWN`,
   `WM_POINTERUP`, `WM_POINTERLEAVE`** — in that order, with one pointer
   id — and **no `WM_POINTERUPDATE`**.
3. **`WM_POINTER*` `lParam` carries SCREEN coordinates; the promoted
   mouse messages carry CLIENT coordinates.** Measured on the same
   contact: client centre `(241,176)`, pointer `lParam` `(450,414)`,
   promoted `WM_LBUTTONDOWN` `lParam` `(241,176)`, window at `(200,200)`.
   **This is the fact the plan item does not have.** Its
   *"the shared seam is the DIP conversion"* is true and incomplete: the
   pointer family needs a translation the mouse family does not, and a
   fixture whose window sits at the desktop origin cannot fail on its
   absence.
4. **Claiming the pointer messages suppresses promotion entirely.**
   Handled leg: `ENTER / DOWN / UP / LEAVE` and nothing else. Unclaimed
   leg (same script, same window, only the early `return` removed):
   the same four, then `WM_MOUSEMOVE`, `WM_LBUTTONDOWN`, `WM_LBUTTONUP`,
   `WM_MOUSEMOVE`. DD-M4-P2-001's single-delivery property is therefore
   measured rather than assumed, and the "two deliveries" state it
   excludes is shown to be reachable.
5. **Injection is desktop-scoped, not window-scoped.** The contact goes
   to whatever window is at the screen point — the probe asserts
   `WindowFromPoint(p)` is its own `hwnd` before injecting. An
   injection-driven **cargo test** would therefore depend on its window
   being visible, foreground and unobstructed, which is
   [verification-environments.md](../../../../docs/notes/verification-environments.md)'s
   **GUI / interactive** environment class, not the *headless runtime
   with live Compositor* class the existing integration suite runs in.
   This is the fact that decides where the injection evidence lives
   (below).
6. **Two injection-side traps, recorded because both fail with a bare
   `ERROR_INVALID_PARAMETER` and no diagnostic.** `pressure` is a touch
   range (0 to 1024); the pen-ish `32000` is rejected. And PowerShell
   assignment to a **nested** value-type field mutates a copy, so a
   `POINTER_TOUCH_INFO` built field-by-field from PowerShell is sent to
   the OS still zeroed — the struct has to be built in the C# layer.
7. **No normative text says whether a touch contact moves focus or
   paints hover.** [architecture.md §13.2](../../../../docs/architecture.md)
   fixes the message family, the promotion suppression and the shared DIP
   boundary; §13.3 / §13.4 describe focus without reference to input
   family; [dsl_spec.md §4.19](../../../../docs/dsl_spec.md) never
   mentions touch. Both are this task's decisions, and both are recorded
   for T13 rather than silently taken.
8. **[architecture.md §12.3](../../../../docs/architecture.md) row 2 is
   stale, and no task's re-verification list names it.** *"Where
   hit-testing reads a widget's rectangle back off its Visual (§7.5),
   that readback is converted alongside them"* was falsified by T2's
   migration. A grep of this log for `12.3` returns nothing and T13's
   check list names §4.x and §13.x only. Carried to T13 as a finding,
   together with the question §12.3 newly acquires here — whether its
   four-kind enumeration should mention that the pointer family arrives
   in screen space.

#### Normative statements that already answer this task (DD-V-031)

| Document / section | What it fixes | Consequence here |
|---|---|---|
| [architecture.md §13.2](../../../../docs/architecture.md) "Pointer, mouse and touch" | Touch is consumed as `WM_POINTER*`, not through mouse promotion; **handling the pointer message is what suppresses that promotion — one delivery per contact**; `EnableMouseInPointer` is deliberately not called; both families cross the same DIP boundary | The message family, the suppression mechanism and the refusal are **answers, not questions**. The task builds them; it does not re-decide them |
| [architecture.md §13.2](../../../../docs/architecture.md) target selection | One target, topmost, bounded by ancestor clips; ancestors until a handler runs | The touch arm reuses `hit::` through `hit_test_click`; it adds no resolution rule |
| [architecture.md §12.3](../../../../docs/architecture.md) row 2 | Pointer message coordinates are divided by the scale at the window procedure | Confirms the division belongs in `wnd_proc`. Silent on screen-versus-client space (fact 3) — recorded as a §12.3 question for T13, not resolved here |
| [dsl_spec.md §4.19](../../../../docs/dsl_spec.md) | "A pointer event resolves to exactly one target…"; `clicked` on any widget | A touch tap is a pointer event, so the authored surface is already specified and unchanged |
| [DD-M4-P2-001 §Risks](../decisions/dd-m4-p2-001-event-routing-model.md) | "only the subset the fixture and the two apps exercise is handled, and unhandled members fall through to `DefWindowProc`" | Fixes the shape of the handled set: claim the members a contact produces, let the rest fall through |
| [CLAUDE.md §Testing rules](../../../../CLAUDE.md) | A CI-gated mock-free test fails rather than skips on GitHub Actions; the skip guard is verified where the capability is absent | Applies to whatever is landed as a cargo test — which is why the split below matters |

No divergence between the decision set and the specification was found.
The one gap is fact 7 (touch and focus, touch and hover), which is an
**absence** rather than a disagreement.

#### What T11's responsibility actually is (critical re-reading of the item)

The plan item is right about the path and understates two things.

- **Right:** the path under test is touch to DIP to hit resolution to
  handler; `EnableMouseInPointer` is refused; the limit must be stated;
  the taxonomy question must be answered.
- **Understated 1 — the conversion.** "Touch rides the same seam" is
  true of the *division* and false of the *space*: fact 3 makes
  screen-to-client a new step on the input path, in the class
  [architecture.md §12](../../../../docs/architecture.md) treats as
  enumerable and audited. It is invisible at a window position of
  `(0,0)` and invisible at scale 1 in the other direction, so the
  fixture has to be positioned **and** scaled off the identity.
- **Understated 2 — where the evidence can run.** The item predicts one
  artifact, a cargo test, and asks whether it can run on CI. Fact 5 says
  the two halves of the claim have **different environment
  requirements**:
  - the claim *"the runtime converts, resolves and dispatches a pointer
    message correctly"* is a property of `wnd_proc` and needs only the
    environment the existing integration suite already has;
  - the claim *"a real OS touch contact reaches the shipped app, exactly
    once, because the promotion was suppressed"* needs the interactive
    desktop of fact 5.

  So T11 lands **both**, each in the tier that can run it: a CI-gated
  message-level integration fixture, and a desktop-tier injection
  evidence script whose result is recorded here. This is **not** the
  plan's fallback swap — the fallback was *"if injection is infeasible
  everywhere, post frames instead"*, and injection is feasible (fact 1).
  The posted-frame fixture is added **beside** injection rather than
  instead of it, and the weaker claim is labelled as such.

  **The open question this leaves for the owner is narrow**: whether the
  injection half should *additionally* be a cargo test gated on CI, which
  cannot be decided without probing GitHub Actions, and probing GitHub
  Actions needs a push — a separate owner gate. Raised at the close gate
  with a recommendation rather than blocking delivery, because the split
  above is complete evidence on its own.

#### Decisions this task makes explicitly (fact 7's two halves)

- **A touch contact moves focus, exactly as a click does.** The
  `WM_POINTERUP` arm calls `focus_on_click` before dispatching, in the
  same order and for the same reason `WM_LBUTTONUP` does (a handler's
  synchronous rebuild can invalidate the resolved path). The alternative
  — a tap that activates a widget without focusing it — would make the
  keyboard's next `Tab` depend on which input family opened the lightbox,
  and DD-M4-P2-003's click rule is written about the *click*, not about
  the mouse. Closes the T5 / T7 / T10 line.
- **A touch contact does not write hover or pressed.** The arms call
  neither `update_hover` nor `clear_hover`, and do not touch
  `state.mouse_down`. Three reasons: hover is a cursor concept and a
  contact that has lifted leaves no cursor behind, so a painted hover
  would have no natural clearer; the hover record and the painted state
  are the derived pair T4 made single-writer, and a second producer is
  the shape DD-M4-P1-002 §Row 6 closed; and `update_hover`'s pressed arm
  is driven by `state.mouse_down`, which a contact does not own. Closes
  CF-T4-4 with a decision rather than by omission. The limit — a touch
  user gets no press feedback from wasamo in M4 — is stated in the same
  doc comment.

#### Trap selection (implementation-gates §1)

```
- [x] #1 semantic migration   - [x] #2 side effects   - [x] #3 parallel data   - [x] #4 branch tests
- [x] #5 carry-forward        - [x] #6 root cause     - [ ] #7 GUI positive control
```

- **#1 — applies, in the conversion-seam sense.** No enum or schema gains
  a variant, so the literal trigger is absent. What does apply is its
  artifact: the pointer-coordinate conversion sites are the enumerable
  audited class of
  [architecture.md §12.3](../../../../docs/architecture.md), and this
  task adds callers to it. The close artifact is a call-site table of
  every pointer-coordinate conversion in `wnd_proc` with **its input
  space** (screen or client) classified — the classification fact 3 makes
  load-bearing, and the one a search for `pair_to_dip` alone would not
  produce.
- **#2 — applies.** New message arms change more than they appear to:
  what the window claims from the OS (promotion for every contact,
  including gestures no fixture drives), the focus record, the reactive
  drain reached through a handler's state write, and the four things the
  arms deliberately do **not** write (`hover`, `mouse_down`,
  `tracking_mouse`, the host pointer callback slots). Enumerated at
  close, including the deliberate non-writes, because "did not write it"
  is exactly the side effect an enumeration is for.
- **#3 — applies, folded into #2.** The hover record and painted state
  pair is the parallel structure in reach; the decision is that neither
  arm touches either half.
- **#4 — applies.** Every arm added is a branch, and the arms that
  deliberately do nothing but return are the ones no incidental test
  fires. Each needs a test that fires it directly.
- **#5 — applies.** Three things carry: the stated limit (synthesized
  injection does not establish that a physical digitizer produces these
  messages), the taxonomy row, and the two decisions above, which
  M4-Phase 4's pointer capture and drag surface will be the first to
  re-open.
- **#6 — applies, armed rather than triggered.** Injection is
  timing-dependent (the contact is delivered asynchronously and has to be
  pumped for). A "no messages arrived" result must be root-caused, not
  re-rolled — and fact 6 is already one instance of that discipline
  paying off: the first probe's `ok=False err=0x57` was a wrong
  `pressure` and a PowerShell copy-semantics bug, not a missing
  capability. Reading it as "injection is unavailable here" would have
  swapped the task to its weaker fallback on a false premise.
- **#7 — does not apply as a rendering claim, and its principle applies
  anyway.** Nothing this task builds paints: the decision above is that
  touch writes no presentation state, so there is no frame in which a
  correct implementation differs from a wrong one. The **positive-control
  obligation** is met where the claim actually lives — at the message
  level, with the promotion leg (fact 4) as the leg that shows "two
  deliveries" is reachable. The injection evidence does render (the
  lightbox opens), but the rendering is the *readout*, not the claim.

#### Review lane — corrected to full independent review

[preamble.md](./preamble.md) predicts **branch/test-focused**: *"new
message arms routing into the already-reviewed seam; the single-delivery
assertion is the artifact"*. Corrected here, on the Phase 1 F-12 / T12
precedent for a stale lane, for a reason the prediction could not have:
fact 3 makes this a **new conversion site on the input path**, which is
the class DD-M4-P1-002's audit governs and the class this phase's whole
T1 / T2 migration exists to keep enumerable. A second reason is that the
arms change what the window claims from the OS for *every* contact,
including members no fixture drives. The branch/test-focused check
composes in for the arms themselves.

#### Boundaries this task does not cross (and what would retract each)

1. **No authored surface.** No new signal, attribute or grammar. Retract
   if a fixture cannot express what it must assert without one — it
   cannot; `clicked` already covers a tap.
2. **No new ABI function** (cross-task obligation). The host pointer
   callback slots stay uninstalled, as `key_down_fn` does.
3. **No change to `hit`, `focus` or `emit`.** The arms call existing
   primitives. Retract if the fixture needs a runtime accessor that does
   not exist — T10's boundary 1 was retracted for exactly that, so the
   condition is written down this time rather than discovered.
4. **No `EnableMouseInPointer`** — DD-M4-P2-001, not a judgement call.
5. **No pointer capture, no drag, no gesture** — deferred to M4-Phase 4
   by DD-M4-P2-001.
6. **No pointer-type filtering.** A pen contact routes like a touch
   contact. Filtering would add a branch no test in this phase can fire
   with the widget set available, which is trap #4 in the direction that
   creates untestable code.

### Close gate (recorded 2026-08-09)

What landed: five `WM_POINTER*` arms in `wnd_proc` behind two pure
predicates, one new conversion helper, ten integration fixtures in
`touch_pointer_integration.rs`, two unit tests in `window.rs`, two
evidence scripts, and a taxonomy row plus an observation in
[verification-environments.md](../../../../docs/notes/verification-environments.md).
No authored surface, no ABI, no change to `hit` / `focus` / `emit` /
`widget`. `cargo fmt --all -- --check`, `cargo build --workspace`,
`cargo test --workspace` (0 failing blocks; `wasamo_runtime` lib 609,
`touch_pointer_integration` 10) and `cargo build --release --workspace`
are all green.

#### Three facts measured *after* the start gate

- **Fact 9 — promotion suppression is per contact, gated on the
  button-transition members.** The start gate's fact 4 measured only the
  two ends (all claimed / none claimed), and both the arm's comment and
  the independent review's counter-hypothesis (that promotion is
  per-message) were wrong about the middle. Measured per member, on a
  stationary contact:

  | claimed set | promoted mouse messages |
  |---|---|
  | all five | 0 |
  | none | 4 |
  | all but `DOWN` / all but `UP` / all but `ENTER` | 0 |
  | `DOWN`+`UP` only | 0 |
  | `DOWN` only / `UP` only | 0 |
  | `ENTER` only / `LEAVE` only | 4 |

  and on a **moving** contact (`DOWN`, three `UPDATE` frames, `UP`):

  | claimed set | promoted mouse messages |
  |---|---|
  | all five | 0 |
  | `DOWN`+`UP` only, `UPDATE` unclaimed | 0 |
  | none | 6 |

  So claiming `WM_POINTERDOWN` **or** `WM_POINTERUP` — either alone —
  suppresses the whole contact, including the `WM_MOUSEMOVE` an unclaimed
  `WM_POINTERUPDATE` would otherwise produce; claiming only `ENTER` or
  only `LEAVE` suppresses nothing. **The shipped code is unchanged and its
  reason is not.** All five are claimed not because dropping one would
  re-enable promotion (measured: it would not) but so that no member of a
  contact this runtime has taken responsibility for reaches
  `DefWindowProcW` — which keeps the arm correct independently of a
  promotion rule the OS owns.
- **Fact 10 — an injected contact carries `POINTER_MESSAGE_FLAG_PRIMARY`
  on every member of its sequence** (`flags=0x6017` on `ENTER` / `DOWN`,
  `0x6000` on `UP` / `LEAVE`), through both injection APIs. This is what
  made the review's multi-contact correction safe to take: gating dispatch
  on the primary bit cannot change what a single-contact tap does.
- **Fact 11 — the message-level suite cannot see the suppression at
  all.** Mutation witness W2 made the `WM_POINTERUP` arm fall through to
  `DefWindowProcW` and the whole suite stayed green: a `SendMessageW`-borne
  pointer message carries no real pointer id, so `DefWindowProcW` promotes
  nothing either way. This is the measurement that splits the evidence into
  two tiers rather than a preference.

#### #1 — call-site audit table (pointer-coordinate conversions)

Query: `rg "pair_to_dip|pointer_physical|pointer_message_to_client_dip|ScreenToClient" wasamo-runtime/src`.
Every hit classified; the column that matters is the **input space**,
which a search for `pair_to_dip` alone would not surface.

| # | Site | Message | Input space | Conversion | Classification |
|---|---|---|---|---|---|
| 1 | `window.rs:1163` | `WM_MOUSEMOVE` | client | `pair_to_dip(pointer_physical(lparam))` | must-not-translate — the mouse family is already client-space |
| 2 | `window.rs:1202` | `WM_LBUTTONDOWN` | client | as above | as above |
| 3 | `window.rs:1214` | `WM_LBUTTONUP` | client | as above | as above |
| 4 | `window.rs:1334` | `WM_POINTERUP` | **screen** | `pointer_message_to_client_dip` = `pointer_physical` → `ScreenToClient` → `pair_to_dip` | must-translate — the one new site |
| — | `window.rs:368`, `:784`, `:1027`, `emit.rs:171` | `WM_SIZE` / attach / `WM_DPICHANGED` / drain | client **extent**, not a pointer coordinate | `pair_to_dip` | ignore-OK — an extent has no origin to translate |
| — | `dip_scale.rs:148`, `:309` | — | — | the definition and its unit test | ignore-OK |

`pointer_physical` now has callers in two different coordinate spaces, so
its doc was rewritten to state that it is a raw `lParam` unpacker with no
opinion on space, and each caller names its own. Tests: the translation is
pinned by `a_touch_tap_fires_the_handler_of_the_widget_it_lands_on` and
`a_pointer_message_carrying_an_untranslated_client_point_resolves_to_nothing`,
the division by
`a_touch_tap_at_a_non_unit_scale_resolves_the_widget_whose_dip_rectangle_contains_it`
— each shown red under its own mutation below.

#### #2 / #3 — structural side-effect enumeration, including the deliberate non-writes

| Derived state | What the arms do | Why |
|---|---|---|
| OS mouse promotion for the contact | **Changed for every contact this window receives**, including pen and gesture contacts no fixture drives | Claiming is the mechanism §13.2 names; fact 9 measures what it is keyed on |
| `WindowState::focus` | Written by `WM_POINTERUP`, before dispatch | A touch contact moves focus exactly as a click does; a handler's synchronous rebuild can invalidate the resolved path |
| Reactive drain / re-layout / `arranged_rect` store | Reached through `hit_test_click`'s handler, exactly as the mouse path reaches it | Not new; the arm adds no drain point |
| `WindowState::hover` | **Not written by any arm** | The record and the painted state are T4's single-writer derived pair; a lifted contact leaves no cursor to clear it |
| `WindowState::mouse_down` | **Not written** | It is the mouse's pressed-painting flag; a contact does not own it |
| `WindowState::tracking_mouse` | **Not written** | `TrackMouseEvent` is the mouse-leave mechanism; the pointer family has its own `LEAVE` |
| Host pointer callback slots | **Not installed** | Cross-task obligation: no new ABI surface (framing agreement ⑦) |
| Non-primary contacts | Claimed, but dispatch nothing | Only the primary contact activates a widget; claiming must not become contact-dependent or promotion returns |

The parallel-data trap (#3) reduces to the hover pair, and the answer is
that neither half is touched — recorded here rather than left as an
absence, because "did not write it" is exactly what an enumeration is for.

#### #4 — branch tests, each shown red under its own mutation

| Branch | Test that fires it | Mutation | Result |
|---|---|---|---|
| `pointer_message_to_client_dip`'s translation | `a_touch_tap_fires_the_handler_of_the_widget_it_lands_on`; `a_pointer_message_carrying_an_untranslated_client_point_resolves_to_nothing` | **W1** delete `ScreenToClient` | 7 of 9 fixtures red |
| its division | `a_touch_tap_at_a_non_unit_scale_resolves_the_widget_whose_dip_rectangle_contains_it` | **M1** drop `pair_to_dip`; **M2** apply it twice | 1 red each, same assertion, `left: 0 right: 1` |
| claimed-but-inert arm (`ENTER`/`DOWN`/`UPDATE`/`LEAVE`) | `the_claimed_pointer_messages_that_act_on_nothing_change_no_state` fires each member directly | **M5** drop `WM_POINTERUPDATE` from the set | the unit test red |
| the claimed **set's membership** | `claims_pointer_message_without_acting_is_pinned_by_name` | **M3** rewrite as `matches!(msg, 577..=582 \| 584..=586)` | unit test red naming `0x0241` (`WM_NCPOINTERUPDATE`); all 10 fixtures stayed green |
| `WM_POINTERUP` dispatch | `one_touch_contact_reaches_the_handler_exactly_once`, and `two_deliveries_of_the_same_contact_are_reachable` as the ceiling control | **W2** fall through to `DefWindowProcW` | **nothing red** — fact 11 |
| the primary gate | `a_non_primary_contact_is_claimed_but_dispatches_nothing` | **M4** drop the primary check | 1 red, `left: 1 right: 0` |
| the primary bit itself | `pointer_message_is_primary_reads_the_high_word` | — | pure-logic pin, both directions |

**DD-V-029 applies and is discharged.** `pointer_message_to_client_dip`
is a newly authored coordinate-conversion rule, so its named test had to
be shown going red under a deliberately wrong implementation: W1 for the
translation half, M1 and M2 for the division half. The division half was
**not** discharged at first — the fixture written to be that leg targeted
a full-width, 72-DIP-tall `Box` whose centre survives both mutations, so
it stayed green under each. The independent review found it; the fixture
was retargeted at a small `Button` with a precondition assertion that the
undivided point falls outside its rectangle, and only then did M1 and M2
go red. The start gate's trap #4 paragraph did not name DD-V-029, which is
the catalog item its selection missed.

#### #5 — carry-forward ledger

| Item | Evidence | Class | Re-trigger criterion |
|---|---|---|---|
| **CF-T11-1 — the synthesized-touch limit.** Injection establishes that this message path works, not that a physical digitizer produces the same messages. Same shape as Phase 1's synthesized-`WM_DPICHANGED` limit | The evidence scripts and their recorded output | `stated limit` | Any task claiming touch-hardware behaviour, and M4-Phase 4's drag surface, which is the first to need a *moving* contact |
| **CF-T11-2 — pointer capture, drag and gesture are not built.** `WM_POINTERCAPTURECHANGED` is reachable (the system can take a gesture over mid-contact) and is deliberately unclaimed | The claimed-set unit test's must-not-claim list | `carry-forward` | **M4-Phase 4**, whose scrollbar is DD-M4-P2-001's named trigger for pointer capture. It is also the first task that would need the moving-contact path to do anything |
| **CF-T11-3 — multi-contact behaviour is one decision, not a mechanism.** Only the primary contact activates; a second simultaneous contact is claimed and does nothing | `a_non_primary_contact_is_claimed_but_dispatches_nothing`, and the M4 mutation | `carry-forward` | The first surface that wants two contacts to mean something — pinch, two-finger scroll — which is at the earliest M4-Phase 4 |
| **CF-T11-4 — `pointer_physical` has two callers in two coordinate spaces, with nothing in the type system separating them.** This is the near-miss form of the phantom-typed length newtype (`Dip<T>` / `Px<T>`) the M4-Phase 1 handoff reserved "only if a unit-confusion defect actually recurs" | The #1 audit table's Input-space column | `carry-forward` | A third caller, or the first actual unit-confusion defect. The reserve condition has **not** fired: no defect occurred, and the classification is currently carried by a doc comment and a table rather than by the compiler |
| **CF-T11-6 — the injection half is deliberately outside the CI gate, and the reservation is open.** Owner-settled 2026-08-09: not needed for now, revisited when it is. The two tiers that exist close AC1's touch half; a CI-gated injection test would be additive | The disposition below, and fact 11 (the message-level tier cannot make the suppression claim) | `carry-forward` | A touch surface whose regression would go unnoticed between desktop captures — the first is M4-Phase 4's drag, which needs a *moving* contact — or a CI runner known to support injection. Adding it needs a GitHub Actions capability probe, which needs a push |
| **CF-T11-5 — an evidence script's fixed wait encodes the machine's load at the moment it was written.** `capture-t11-touch-counter.ps1` slept 3 s and looked for the host's window once; on a loaded machine the titled window appears between 3 s and 5 s, and the run failed with "no visible Counter HWND" while the host was alive and healthy. Replaced by a poll to a deadline, with a failure message that lists the windows the process actually owns | The measured appearance times (none at 1 s / 2 s, untitled-only at 3 s, titled at 5 s) | `carry-forward` | Any later capture or smoke script that waits for a host to be ready. [verification-environments.md](../../../../docs/notes/verification-environments.md) Observation 4 already states this for foreground activation — "a single refusal is not an environment verdict" — and the step before it had been left on a fixed sleep |

#### #6 — deterministic failures, root-caused rather than re-rolled

Three, none of them re-rolled to green:

- **`InjectTouchInput` returning `ok=False err=0x57`** at the first probe.
  Read as "injection is unavailable on this machine" it would have swapped
  the task to its weaker fallback on a false premise. Root cause was two
  bugs in the probe, not a missing capability: `pressure` was a pen-scale
  value where the touch range is 0–1024, and PowerShell assignment to a
  **nested** value-type field mutates a copy, so the contact reached the OS
  still zeroed. Fixed by building the structure in the C# layer.
- **The probe reporting `OVERALL: FAIL` on a correct runtime.** The claim
  leg's log contained a `WM_MOUSEMOVE`, and the verdict counted any mouse
  message as promotion. Root cause: genuine physical-cursor traffic —
  the coordinates `(291,335)` are not the contact's client point
  `(241,176)` and it arrived *before* the pointer sequence. Fixed at the
  source (park the cursor off-window, clear the log, then inject) **and**
  in the verdict (count only a mouse message that follows the contact's
  pointer sequence and carries the contact's client point).
- **A stale `GetLastError` printed beside `ok=True`.** Cosmetic in effect
  and not in an artifact: a non-zero error code next to a success reads as
  a defect to whoever re-runs it. The error is now printed only on failure.

#### #7 — not applicable as a rendering claim; where the obligation was met instead

Nothing this task builds paints — the decision is that a touch contact
writes no presentation state — so there is no frame in which a correct
implementation differs from a wrong one, and trap #7's screenshot artifact
has nothing to be about. The **positive-control principle** still applies
and is met where the claim lives: at the message level,
`two_deliveries_of_the_same_contact_are_reachable` is the leg that shows
the counter can reach two, so "exactly once" is a count rather than a
ceiling; and at the OS level, fact 9's unclaimed legs show the promoted
messages the claimed legs are absent of. The counter-app frame set is a
readout of a state change, not a rendering claim.

#### Independent review — six defects taken, two dispositioned

The lane was corrected to full independent review at the start gate, and
the review earned it: it found a defect no mutation in the task's own plan
would have caught.

- **Taken and fixed:** the non-discriminating scale fixture (above); two
  false statements in `pointer_message_to_client_dip`'s doc (`ScreenToClient`
  returns `BOOL`, not a `Result`, and the precedent it cited *reports*
  rather than swallows); the claimed-set unit test omitting the non-client
  variants, where a plausible range-based rewrite passed every assertion
  while breaking touch caption interaction; multi-contact dispatch; two
  comments claiming more than facts 2 and 4 measured; and the unit test's
  own doc claiming it pinned the call site when it pins only the predicate.
- **Dispositioned rather than implemented:** the phantom-typed newtype
  suggestion is CF-T11-4 — the M4-Phase 1 handoff reserved it for an actual
  recurrence, and none occurred. The frame-retention finding is CF-T11-5.

The review's own counter-hypothesis about per-message promotion was itself
wrong, which is why fact 9 was measured rather than argued.

#### Re-decided at close (start-gate selection is a prediction)

Two changes. **#7 stays non-applicable** but its principle is recorded
against the message-level control rather than left as "not applicable
therefore nothing". **#4 widened**: the start gate counted arms, and the
task ended with two pure predicates and a primary gate that arms alone do
not cover, plus the DD-V-029 obligation it did not name. The trap that was
*under*-selected was #4, not over-selected.

#### Re-audit of the whole task list

Read T12 and T13 again at close, not only T11's own item.

- **T12 — unaffected.** Its four controls are mouse- and keyboard-driven
  and none is a touch control; this task neither owes it a frame nor takes
  one from it. One usable fact: `capture-t11-touch-counter.ps1` is a
  working two-input-family comparison harness with cursor parking and a
  fail-loud "the step did not change" guard, which is the shape control A's
  re-take would want if T12 chooses to re-capture rather than cite T10's.
- **T13 — gains four re-verification items**, all recorded here rather
  than fixed:
  - **§13.2's touch paragraph is satisfied and its wording is now
    narrower than the measurement.** "Handling the pointer message is what
    suppresses that promotion — one delivery per contact" is true;
    fact 9 sharpens it to *which* members carry the suppression. T13
    decides whether the sentence gains that precision or stays at the level
    it is written.
  - **§13.2 says nothing about whether a touch contact moves focus or
    paints hover.** Both are decided here and neither is normative. T13
    decides whether §13.2 gains the two sentences.
  - **§12.3's four-kind conversion enumeration does not mention that the
    pointer family arrives in screen space.** Row 2 says pointer
    coordinates are divided at the window procedure and is silent on the
    translation ahead of it.
  - **§12.3 row 2's second sentence is false and is in no task's list.**
    "Where hit-testing reads a widget's rectangle back off its Visual
    (§7.5), that readback is converted alongside them" was falsified by
    T2's migration; a grep of this log for `12.3` returned nothing before
    this task, and T13's check list names §4.x and §13.x only.
- **Cross-task obligations** — "no new ABI function" holds: the arms call
  private runtime functions and install no callback slot.

#### The desktop-tier evidence, taken against the landed runtime

[evidence/t11-frames/](./evidence/t11-frames/) holds two full capture
runs of the shipped `counter-rust.exe` at `e0cb862` — the commit that
carries the primary-contact gate — one driven by real OS touch injection
and one by mouse, differing in nothing else. Three touch contacts render
`Count: 3`, agreeing with three mouse clicks at every step within the
F-33 tolerance (`max_channel` 0 or 1, no pixel over the visible-change
threshold), while step 0 versus step 1 differs by 6,628 px in both
families. A contact delivered twice — the state the suppression exists to
prevent — would have rendered `Count: 2` against the mouse run's
`Count: 1`, so the agreement leg fails on a whole digit rather than on a
rounding difference. Read directly: step 1 also shows the Button in its
focus indicator, which is the "a touch contact moves focus" decision
rendered rather than only read back as state.

**Two defects in the artifact itself were found while taking it**, both
of the class that makes an evidence script lie rather than fail:

- The script slept a fixed 3 s and looked for the host's window **once**.
  On a loaded machine the titled window appears between 3 s and 5 s, so
  the run aborted with "no visible Counter HWND" while the host was alive
  and healthy. Measured (nothing at 1 s / 2 s, an untitled visible window
  at 3 s, the titled one at 5 s) rather than guessed at, then replaced by
  a poll to a deadline whose failure message lists the windows the
  process actually owns. Recorded as CF-T11-5.
- The comparison reported `differing_px=0` and a within-set jitter of
  `0 px` for frames that differ by 1 per channel over 4,638 pixels. The
  verdicts were sound — they check `max_channel` against the tolerance —
  but the *counter* being reported was "pixels over a 60-per-channel-sum
  visible-change threshold" under a label that reads as "pixels that
  differ". F-33's whole point is that the noise floor is a measured
  quantity, and the artifact was reporting a threshold artifact as that
  quantity. Now reports all three numbers under names that say which is
  which, and the measured noise floor is `max_channel` 1 over 4,638 px in
  the mouse set and 0 in the touch set.

#### Owner disposition — the injection half stays out of the CI gate (2026-08-09)

The plan predicted one artifact, a cargo test gated on CI. Fact 5 (start
gate) and fact 11 split it into two tiers instead: the message-level
fixture is CI-gated and green, and the injection half is desktop-tier.
Whether the injection half should *additionally* be a cargo test that
fails rather than skips on GitHub Actions
([CLAUDE.md §Testing rules](../../../../CLAUDE.md)) could not be settled
from here — a runner may or may not have the capability, probing it needs
a push, and push is its own gate.

**Owner-settled 2026-08-09: not needed for now, revisited when it is.**
This is a reservation rather than a closed question — the owner kept the
possibility of change open — so it is recorded with its re-trigger rather
than as a decision the phase is done with (CF-T11-6). Nothing in the
phase is blocked by it: AC1's touch half is closed by the two tiers that
exist, and adding the CI-gated variant later is additive, needing a
GitHub Actions capability probe and nothing else.

---

## T12 — GUI evidence with positive controls

### Start gate (recorded 2026-08-09, before any capture code was written)

#### Eight measured facts (probes run before writing a line of the capture script)

1. **The development desktop is a single 120-DPI monitor, so this sitting
   is at scale 1.25 throughout.** `EnumDisplayMonitors` + `GetDpiForMonitor`
   under a harness that declared Per-Monitor-Aware V2 **and read its posture
   back**: one monitor, `(0,0)-(2452,1291)` physical, work area
   `(0,0)-(2452,1231)`, DPI **120**, primary. The plan item asks for "at
   least one control (A or C) at a display scale ≠ 100%"; here **all four**
   are, which discharges that row without a second sitting. Recorded as a
   measurement rather than an assumption because the same probe shows how
   easily it could be got wrong: `GetDpiForWindow(GetDesktopWindow())`
   returns **96** on this machine. The scale must be read from the window
   under capture, which is what T10's script already does.
2. **The window geometry T10 used still fits, and reusing it is what keeps
   the toolbar out of the overlap.** 1000x750 at (120,120) inside a
   2452x1291 desktop; T10 measured the resulting client at 982x703 px =
   785.6x562.4 DIP, at which its own frames show the toolbar's six controls
   cleanly separated. The item's capture-width rule now has an input-side
   reason as well (T10's G7: where the toolbar overflows, a tab
   `ToggleButton` stops being clickable at all), and control C clicks a tab
   `ToggleButton`, so this is load-bearing here rather than cosmetic.
3. **The runtime has not changed on any path control A exercises since T10
   captured it.** `git diff --stat 99af98c..ecda8d8 -- wasamo-runtime/src
   wasamo-ir/src wasamoc/src examples/` is **one file** — `window.rs`,
   +391/-11 — and all of it is T11's five `WM_POINTER*` arms, two new
   helpers (`pointer_message_to_client_dip`, the claimed-member predicate)
   and a doc rewrite on `pointer_physical`. The `WM_LBUTTONDOWN` /
   `WM_LBUTTONUP` / `WM_MOUSEMOVE` / `WM_KEYDOWN` arms are untouched.
   **Citing T10's control-A frames would therefore have been defensible on
   the staleness ground**; the decision below to re-take rests on a
   different ground, and saying so keeps the decision from resting on a
   reason that is not true.
4. **The gallery has four Tab stops, and the first two are already pinned
   at the state level.** `focus-group: true` on the toolbar-left `HStack`
   makes All / Albums / Favorites **one** stop (dsl_spec §4.19
   `focus-group`), and `gallery_slice_integration.rs`'s G10 asserts by
   label that one Tab focuses `All` and a second focuses `Scroll down`.
   The thumbnails are `Box`es, not Button-family, so they are not stops
   (§4.19 Focus). Declaration order therefore gives:
   **[group] → Scroll down → Scroll up → Open lightbox → wrap**.
   Screen order matches: `Cell { row: 0 column: 0 h-align: start }` puts
   the group at the left of the client and
   `Cell { row: 0 column: 1 h-align: end }` puts the other three at the
   right in declaration order — so the four stops are **monotone
   left-to-right**, which is what makes control B's ordering claim
   measurable from the frames instead of from hand-worked pixel constants.
5. **The lightbox has three stops and entry focuses the first.** `<`, `>`,
   `x` in declaration order inside the lightbox `Grid`; G2 pins the entry
   at the state level and T10's frames show the `<` Button amber. Five Tabs
   inside the scope therefore land on `x` — deliberately more presses than
   the scope has stops, so a leak has several chances to happen.
6. **`Home` is a recognised key name with no handler anywhere in the
   gallery.** `wasamo_ir::RECOGNISED_KEY_NAMES` has 22 entries and `Home`
   is one; `examples/gallery/gallery.ui` authors `key-down("ArrowLeft")`
   and `key-down("ArrowRight")` only; and §4.19's keys-the-runtime-keeps
   table does not list `Home`. That is exactly what control D's re-audited
   row asks for — a **recognised** key with no handler, so the leg
   discriminates rather than being satisfiable by "the compiler never
   heard of it". `Enter` is avoided deliberately: whether Button keyboard
   activation should exist is CF-T8-1, open for T13, and a leg standing on
   an open question is not a leg.
7. **The scrim is `#101820cc` — alpha `cc`, not opaque — so the toolbar is
   visible through an open lightbox.** T4 measured the consequence
   directly (the checked `ToggleButton` entered its hovered colour
   *through* the scrim before T4 fixed it). This matters because control
   C's containment leg is a **no-change** claim about the toolbar band
   while the lightbox is open, and a no-change claim through an opaque
   cover would be vacuous. It is not taken on trust: the run measures the
   band's observability in-flight (fact 8's sensor leg).
8. **Control C's blocker is the lightbox `Grid`, and the geometry says why
   a toolbar-height point reaches it.** The lightbox `Grid` is
   stretch/stretch with `rows: 1* 44 300 64 1*`; at a 562.4 DIP client the
   two flexible rows are 77.2 DIP each, so a point at the toolbar's
   y ≈ 28 DIP falls in the `Grid`'s **empty** first row — no `Cell` and no
   Button contains it, so `resolve_topmost` falls back to the `Grid`
   itself, which carries no `clicked` handler and whose ancestor walk
   reaches only the `modal-scope` `ZStack` (which carries `dismiss` and
   `key-down`, not `clicked`) and the root. T10 measured the same
   conclusion through `__resolve_topmost_for_test` (G5). Neither the
   `Grid` nor the `ZStack` is focusable, so the blocked click must also
   leave focus alone — which is why control C can compare **whole client**
   rectangles rather than only the toolbar band.

#### Normative statements that already answer this task (DD-V-031)

This phase synchronised its normative text at Moment 1, ahead of
implementation, so the start gate lists what already fixes the behaviour
being measured. T12 builds no behaviour; what the table fixes is **what
each control is entitled to assert**, and a control that asserted more
than its section says would be evidence for a rule the project has not
made.

| Document / section | What it already fixes | Which control stands on it |
|---|---|---|
| dsl_spec §4.19 Per-item handlers | a binder read resolves when the handler runs, so clicking thumbnail N gives `Photo #N` | A |
| dsl_spec §4.19 Focus | nothing focused at open; Tab / Shift+Tab in declaration order wrapping both ends; a click never clears focus | B |
| dsl_spec §4.19 `focus-group` | an annotated container is **one** Tab stop; arrows move within it | B (why Tab ×1 is the whole strip, not `All` alone) |
| dsl_spec §4.19 `modal-scope` | presence is the entry; Tab cycles only within the subtree; entry moves focus to the scope's first stop and remembers the previous one | C |
| dsl_spec §4.19 "What a scope does not do" | the scope confines the **keyboard only**; a background click is stopped by a **covering widget inside the scope**, not by the scope | C — and it is why C's click leg is evidence about *occlusion*, with containment carried by C's Tab leg |
| dsl_spec §4.19 `dismiss` / "Which keys the runtime keeps" | `Escape` while a scope is present becomes a dismissal request on the innermost one; a key that reaches the end of the walk with no handler is not consumed | D |
| architecture.md §13.3 | the focus indicator is presentation state, not a `Visual` written at focus-change time | B (what the indicator is, hence that it can be sampled as colour) |

No divergence found at this gate. One near-miss recorded rather than
resolved: §4.19's scope section says restoration returns focus "to the
remembered widget", and control C opens the lightbox by **clicking a
thumbnail**, which §4.19 itself says does not move focus — so the
remembered widget is whatever the keyboard was on beforehand. That is the
section's own worked example, not a divergence, and it is what makes C's
closed-before / closed-after frames comparable.

#### What T12's responsibility actually is (critical re-reading of the item)

The item reads as "take four frame sets". Read against the landed code
and the three tasks that already took frames, it is narrower in one place
and wider in two.

- **Narrower: T12 is not where these behaviours are first checked.** Every
  one of the four has a state-level twin that already passes — G1, G10,
  G2/G3, and the `escapes_two_legs` / key-slot fixtures. What no state
  read-back can show is that the state reaches **pixels**, and that is the
  entire product of this task. So each control is written as "the frame a
  wrong implementation would also produce, and the leg that separates it",
  never as a re-assertion of the fixture.
- **Wider (1): the legs themselves must be shown able to go red.** T11's
  retrospective lesson (c) is the governing one here — a red-test nobody
  has seen go red is not a red-test. A capture script's verdicts are the
  same class of hazard as a fixture's asserts, and they are *worse*,
  because a comparison over the wrong region or with an over-generous
  tolerance passes silently and looks like a measurement. T12 therefore
  owes a **self-check pass**: every verdict fed a deliberately wrong
  pairing from its own committed frames, and shown to fail. This is not in
  the item's text; it is what the item's text would be worth nothing
  without.
- **Wider (2): the sensor for control C has to be measured, not assumed.**
  C's containment leg is a no-change claim about a region seen through a
  semi-transparent scrim. A no-change claim is satisfied for free if the
  region cannot register change at all. The run therefore carries a
  **transmissivity leg**: two lightbox-open frames that differ only in
  which tab is checked underneath must **differ** in the toolbar band. If
  that leg fails, the containment leg is withdrawn rather than reported.

#### Decisions this task makes explicitly

- **Control A is re-taken, not cited** — and the ground is *not*
  staleness, which fact 3 shows does not apply. It is that the item's own
  alternative ("re-take them beside the other three so the set is captured
  at one sitting") buys something the cite does not: one build, one launch,
  one window geometry and one measured scale across all four controls, so
  the four sets are mutually comparable and a single `-Compare` run reads
  the whole artifact. T10's set is **cited beside it** as an independent
  earlier sitting; two sittings agreeing on A is worth more than either.
  The re-take is the item row's minimal shape (thumbnail N vs M, and N
  twice, with the `[photo]` box as the localisation leg) and does **not**
  reproduce T10's arrow-key legs, which are T10's and are cited.
- **The order is B, A, D, C**, and it is forced rather than chosen.
  Control B's baseline needs *nothing focused*, which only a fresh launch
  gives (§4.19: nothing is focused when a window opens, and no authored
  surface in this phase clears focus). Control C ends with a different tab
  checked, which is a permanent state change, so it goes last. A and D sit
  between because both are insensitive to a leftover toolbar focus
  indicator: A samples the lightbox caption cell, and D compares two
  frames that carry the same indicator.
- **Control C opens the lightbox by clicking a thumbnail, not the "Open
  lightbox" Button.** A `Box` thumbnail is not focusable, so the click
  leaves focus alone and the scope's remembered widget is unchanged —
  which is what makes C's closed-before and closed-after frames comparable
  over the **whole client**. Opening with the Button would make the Button
  the restore target and put a focus indicator into the comparison.
- **Control D's unrelated key is `Home`** (fact 6).

#### Trap selection (implementation-gates §1)

```
- [ ] #1 semantic migration   - [x] #2 side effects   - [x] #3 parallel data   - [x] #4 branch tests
- [x] #5 carry-forward        - [x] #6 root cause     - [x] #7 GUI positive control
```

- **#1 semantic migration — not applicable.** No enum, IR or schema type
  is touched; this task adds no production Rust at all. **Retraction
  condition, stated because T10's identical judgement failed:** if any
  leg turns out to need a fact no existing accessor exposes and the
  answer would otherwise be *deduced*, the trap is re-selected and the
  accessor is added as a second **caller** of production logic (T10's
  `__resolve_topmost_for_test` shape), never as a second implementation.
  The capture path here reads pixels rather than widget state, which is
  why the exposure is expected to be nil rather than merely hoped to be.
- **#2 side effects — applies, in its documentation form.** The task
  writes prose that other documents own: the plan item, the evidence
  README, the owner smoke protocol and this log. The derived effect to
  enumerate is *which document each statement belongs to*.
- **#3 parallel data — applies, in the documentation analogue the trap
  names.** The owner smoke protocol restates behaviour whose source of
  truth is dsl_spec §4.19, and the evidence README restates the capture
  discipline whose source of truth is
  [verification-environments.md](../../../../docs/notes/verification-environments.md)
  Observation 4. Both must **cite** rather than re-state, or the phase
  acquires a second, drifting copy of its own rules.
- **#4 branch tests — applies, in the form the §Wider (1) reading gives
  it.** Every verdict branch in the comparison must be shown firing under
  a deliberately wrong input. Recorded as the self-check pass.
- **#5 carry-forward — applies.** T12 is the last task before the close
  gate, so anything it finds has exactly one place to go (T13 or the
  phase handoff) and no later task to absorb it.
- **#6 root cause — armed.** Capture runs fail for environmental reasons
  (foreground refused, a window not yet titled — T11's CF-T11-5). A
  re-run to green without a named cause is the failure mode; each retry
  is recorded with what it was retried *for*.
- **#7 GUI positive control — this is the task.** Every control ships a
  difference leg, an agreement leg, and the measured jitter floor the two
  are judged against.

#### Review lane

**Full independent review**, matching
[preamble.md](./preamble.md)'s prediction for T12 (GUI-render evidence).
Not corrected at this gate: the trigger the preamble names is the trigger
that actually applies. The trap #4 branch-focused check composes in over
the self-check pass rather than replacing the full review
([implementation-gates.md §4](../../../procedures/implementation-gates.md)).

#### Boundaries this task does not cross (and what would retract each)

1. **No production Rust, and no change to `examples/gallery/gallery.ui`.**
   Retracted only under the #1 retraction condition above, by the lead,
   recorded at the close gate.
2. **No re-taking of T7's or T10's committed frames.** They are cited.
   Retracted if a leg here contradicts one of them — which would be a
   finding, not a re-take.
3. **The toolbar overlap is not fixed, measured or worked around.** It is
   M4-Phase 4's ([constraints.md §6](../requirements/constraints.md)); the
   capture width simply keeps it out of frame, and the frames record that
   it is out of frame rather than that it is absent.
4. **No normative text is edited.** Anything found goes to T13's
   re-verification list, in the shape T11 used.
5. **The owner smoke protocol prescribes no verdict the owner must
   reach.** It says what to do and what to look at; what the owner
   concludes is the owner's.

### Close gate (recorded 2026-08-09)

| commit | content |
|---|---|
| `fd5d192` | start gate (eight measured facts, the DD-V-031 normative table, trap selection with trap #1's retraction condition, the review lane, five boundaries) |
| `b06611e` | the capture script, its 48 frames and their README — the four controls in one sitting |
| `18e17fb` | the owner smoke protocol |
| `b7ce010` | this close gate and the T12 item revision |
| `a41d51d` | the independent review's central finding: bands no longer computed from the quantity they judge; self-check coverage enforced by the script; region-scoped wrong pairings; the in-run guards made reachable |
| `0d9f659` | the review record and ten prose corrections, plus the truncated-caption finding |

#### Trap selection re-decided at close (plan's standing instruction)

**Unchanged in what applies, and one row's *content* turned out to be
where the task's real work was.** #2, #3, #4, #5, #6 and #7 applied as
selected; #1 stayed non-applicable and its retraction condition did not
fire — the task wrote no production Rust, touched no `.ui`, and needed no
new accessor, because the whole instrument reads pixels rather than
widget state.

The row that behaved differently from its prediction is **#7**. The start
gate treated it as "this is the task" and expected the work to be
capture mechanics. What it actually cost was **calibration**, and the
scale of that only became clear at the independent review: of the **37**
verdicts `-Compare` registers, **13** could not have discriminated
anything as first written, and not one of the 13 was visible in a
passing run. Two were caught inside the task by checks the start gate had
armed for exactly this (§Wider (1) and (2)); the remaining **11** were
caught by the review, and they are the same failure in a place the start
gate did not think to look — the bands the legs are judged against were
derived from the very quantity the jitter legs assert. Both halves are
recorded below, and the second is the more instructive.

#### #7 — GUI evidence: the four controls

Script, frames and provenance:
[capture-t12-controls.ps1](./evidence/capture-t12-controls.ps1),
[evidence/t12-frames/](./evidence/t12-frames/). **One sitting**: one
`cargo build --release --workspace`, one launch of
`target\release\gallery-rust.exe`, one window at (120,120) 1000x750
throughout, display scale **1.25** (120 DPI), **client** rectangle
982x703 px = 785.6x562.4 DIP, **real key presses** (`keybd_event`) with
foreground activation earned by a click and read back, and the input path
printed into the run because the frames look identical either way. Two
frames per set, 24 sets, 48 frames; the meta file carries the commit, the
derived click points and a SHA-256 per file.

**The measured within-set jitter was 0 at every metric, in every set.**
Every agreement leg below is therefore byte-identical rather than merely
inside a band — `b5` ≡ `b1`, `b3b` ≡ `b3`, `brev` ≡ `b2`, `a0` ≡ `a0b`,
`c-openA-click` ≡ `c-openA`, `d-home` ≡ `d-open`, and six frames of
different sets share one hash. Recorded as a measurement of this host and
this sitting, not as a property to rely on: F-33's 13/channel tolerance
was available and simply not needed here.

| Control | Leg | Region | Result |
|---|---|---|---|
| **A** | DIFFERENCE caption, thumbnail 0 vs 3 | caption | **79** px over threshold, `max_channel` 206 |
| A | AGREEMENT caption, thumbnail 0 twice | caption | 0 |
| A | AGREEMENT `[photo]` box, thumbnail 0 vs 3 | photo | 0 |
| **B** | DIFFERENCE stop 1 painted | toolbar | **2567**, bbox x[10..69] |
| B | DIFFERENCE stop 2 painted | toolbar | **5547**, bbox x[564..693] |
| B | DIFFERENCE stop 3 painted | toolbar | **4636**, bbox x[704..811] |
| B | DIFFERENCE stop 4 painted | toolbar | **6444**, bbox x[821..971] |
| B | monotone left-to-right, and consecutive bboxes disjoint | toolbar | PASS |
| B | AGREEMENT wrap returns to the first stop (`b5` vs `b1`) | toolbar | 0 |
| B | AGREEMENT traversal is deterministic (`b3b` vs `b3`) | toolbar | 0 |
| B | AGREEMENT Shift+Tab from stop 3 returns to stop 2 | toolbar | 0 |
| B | DIFFERENCE Shift+Tab actually moved (`brev` vs `b3`) | toolbar | **10183** |
| **C** | SENSOR the toolbar is observable through the scrim | toolbar, open | **6964** px differing, `max_channel` 31 |
| C | AGREEMENT a click on the covered toolbar does nothing | whole client | 0 |
| C | AGREEMENT and it wrote no state either — checked in the clear | whole client | 0 |
| C | DIFFERENCE the same coordinate fires with the lightbox closed | toolbar | **12373** |
| C | DIFFERENCE the handler ran — the previously checked tab lost its colour | `All`'s bbox | **2583** |
| C | EXCLUDE `All`'s new colour is not the checked+focused blend | `All`'s bbox | Δ 86.7 per channel |
| C | AGREEMENT five Tabs inside the scope never reach the toolbar | toolbar, open | **0 px differing at all** |
| C | DIFFERENCE …but they did move focus inside it | `<`/`>`/`x` columns | **4369** |
| C | DIFFERENCE with the scope gone, one Tab reaches the toolbar | toolbar | **9729** |
| C | AGREEMENT the world returned after open/Tab/close | whole client | 0 |
| **D** | AGREEMENT a recognised key with no handler changes nothing | whole client | 0 |
| D | DIFFERENCE Escape closed the lightbox | whole client | **525698** |
| D | AGREEMENT the client returned to its pre-open state | whole client | 0 |

**Read as images, not only as numbers**, by the lead:

- **B is legible frame by frame.** In `b-n` no control carries an
  indicator and `All` is the plain checked blue. In `b1` `All` alone has
  turned pale grey, with `Albums` and `Favorites` untouched — which
  excludes an implementation that highlights the whole annotated
  container, and **nothing more**: a build with no `focus-group` at all,
  three separate stops, renders `b1` identically. **`b2` is the frame
  that carries the single-stop rule**, and it carries it in a number
  rather than an impression: stop 2's painted bbox is `x[564..693]`,
  which is `Scroll down`, while `Albums`' face sits near `x[80..178]`.
  Tab left the group instead of stepping inside it. In `b4` `All` is blue
  again and `Open lightbox` — the accent Button — is pale green. The four
  painted bboxes land on the four controls the toolbar shows, left to
  right, with the group counted once.
- **C's containment is visible in a single pair.** In `c-openB` the `<`
  Button is amber (the scope entry's indicator, on a node the same drain
  created) and `>` / `x` are neutral. In `c-tab`, after five Tabs, `<` is
  neutral and **`x` is amber** — the third stop, which is where 5 presses
  over 3 stops must land. The toolbar band between those two frames is
  byte-identical.
- **The scrim's transparency is visible too.** `c-openA` shows `All`
  highlighted through the scrim and `c-openB` shows `Albums`; the sensor
  leg is a thing the eye confirms, not only the metric.
- **A reads `Photo #0` and `Photo #3`** in the frames the numbers pair
  up, with the `[photo]` box and every other pixel identical. A single
  open frame could not have supported that — an implementation that
  opened the lightbox from any click, or captured a fixed index at
  generation time, renders the same picture.
- In every closed frame the toolbar's six controls are **cleanly
  separated** at this 785.6 DIP client. The width-driven overlap
  ([constraints.md §6](../requirements/constraints.md)) is out of frame
  here, which is what the item's capture-width rule asks the frames to
  show rather than assert.

This is the assistant baseline and does **not** replace the owner's
human-visible smoke ([CLAUDE.md §Testing rules](../../../../CLAUDE.md)).

#### The two legs that could not have discriminated anything, and what they cost

Both are the same failure class in different places: **a leg that passes
without being able to fail.** Neither was visible in a green run; both
were found because the start gate had armed a check for them.

**1. The scrim divides the background's contrast by five, and the
containment leg was being judged with an instrument that could not see
through it.** Control C's central claim — five Tabs inside the scope
never reach the toolbar — is a *no-change* claim about a region behind a
semi-transparent cover, and a no-change claim is satisfied for free if
the region cannot register change. The sensor leg (§Wider (2)) measured
it and failed:

| | `max_channel` | px differing at all | px over the 60-summed bar |
|---|---:|---:|---:|
| unscrimmed, `All`'s face, checked → unchecked | 157 | 2608 | 2583 |
| **scrimmed**, same face, same change | **31** | **2608** | **0** |

`px_differing_at_all` is *identical*: the scrim hides nothing. It divides
contrast by **5.06**, against the **5.00** that `1/(1-0.8)` predicts from
the alpha in the gallery's own `fill: #101820cc` (`cc` = 204/255) — a
1.2% gap, consistent with quantisation (157 × 0.2 = 31.4 → 31), which is
close enough to identify the mechanism and not close enough to call the
two numbers equal. The 60-summed bar exists to clear F-33's text jitter
on *unscrimmed* frames, and under the scrim it cannot see a real change
at all.

**Disposition: the two lightbox-open toolbar-band legs are judged on
`px_differing_at_all`.** This is a **tightening** — the agreement bar
moves from "under 40 px over a 60-summed threshold" to "under 40 px
differing by *any* amount", and the leg meets it at **0** — and it is
recorded on the plan item so a later reader does not "correct" it back.
Every other leg keeps the original metric untouched.

The proxy is **argued** conservative rather than shown: the
focus-indicator swing covers the **same 2608-pixel button face** at
`max_channel` 97 *unscrimmed*, and the attenuation table above measures
that the scrim leaves `px_differing_at_all` untouched, so an indicator
appearing on a toolbar stop under the scrim would register on this
metric. A *scrimmed* focus-indicator swing is measured nowhere and
cannot be in this design — it is the thing containment denies. The
inference is sound; it is an inference.

**2. "The same coordinate fires" cannot be read off the button that was
clicked.** The leg first identified the fired click by re-finding the
checked ToggleButton's blue on the clicked tab. It found **0 px**.
Measured cause: a click on a Button moves focus to it (§4.19 Focus), so
`Albums` ends up checked *and focused*, whose rendered colour is a third
blend again — and control B had already measured that blend, at
`(143.8, 153.2, 149.5)` on `b1`.

**Disposition: the leg moves to `All`** — the *previously* checked tab,
never clicked and never focused in this sequence, so only the handler's
`tab_all_selected = false` can change its face. Measured: `(52.4, 121.2,
214.4)` → `(66.6, 66.6, 66.6)`. Re-tuning the predicate was rejected:
that would have made the leg depend on a colour blend rather than on a
behaviour. The one look-alike that could otherwise explain a colour
change at `All` without the handler running — focus landing there — is
excluded against `b1`'s own measurement, 86.7 per channel away.

#### #4 — every verdict shown able to go red (the branch-test artifact)

`-SelfCheck` feeds every verdict a deliberately wrong pairing drawn from
the same committed frames and requires each to fail. It calls the same
`Assert-*` functions the real pass calls, so it exercises the shipped
predicate rather than a copy. **`-Compare` registers 43 verdicts;
`-SelfCheck` exercises 45 rows — the 43 plus the two in-run guards
`-Compare` never touches — and it fails the run if any registered verdict
has no row.** Result: 45/45 fired red, coverage complete, exit 0.

**The coverage is enforced by the script rather than asserted here, and
that is the review's doing.** The first version of this pass covered 23
of 37 verdicts while three documents called it "every", and the 14 it
left out were exactly the ones that could not have gone red — the
tautological jitter legs plus three unwritten stop rows. A prose claim
about coverage is the same hazard as a leg nobody has seen fail, one
level up. It is now a check.

**Each DIFFERENCE row's wrong pairing is classified `region-scoped` or
`degenerate`, and the split is printed**: 11 region-scoped, 1 degenerate
of 12. A degenerate row — two byte-identical frames — proves only that
`0 > 40` is false; it cannot catch a mis-specified region, which is
precisely the hazard this pass exists for. So each row is now fed two
frames that differ **elsewhere but not in the sampled region**: `a0` vs
`a3` for the toolbar legs (they differ only in the lightbox caption),
`b1` vs `b2` for the caption leg (they differ only in the toolbar),
`c-openB` vs `c-tab` for the sensor, `b2` vs `b3` for `All`'s bbox,
`c-openA` vs `c-openB` for the lightbox columns. Each pairing's claimed
property was measured before it was wired in. The one degenerate row is
`D Escape closed the lightbox`, and it is degenerate necessarily: its
region is the whole client, so any two frames that differ at all differ
inside it. The output says that rather than leaving it looking like an
oversight.

Two rows are worth naming:

- **C containment** (wrong pairing `c-openA` vs `c-openB`) *falsely
  passed* under the original metric — the symptom of finding 1 showing
  up a second time, resolved by the metric rather than by editing the
  row.
- **The six noise-floor rows**, fed a genuinely differing pair, report
  `max_channel` 191–208 against the limit of 13 and fail. Under the
  bands this task first shipped, the equivalent legs could not have
  failed at all.

This is the T11 retrospective's lesson (c) applied to a capture script:
its verdicts are the same hazard class as a fixture's asserts, and worse,
because a comparison over the wrong region — or against a band derived
from itself — passes silently and *looks* like a measurement.

#### #2 / #3 — side effects and parallel data, in their documentation form

The task writes no state, so both traps apply in the form the catalog
names for documentation work: a second source of truth in derived prose.
Enumerated, with where each statement's owner is:

| Statement | Owner | How this task avoids a second copy |
|---|---|---|
| what a modal scope confines, what `focus-group` means, what a click does to focus | dsl_spec §4.19 | the script header and the frames README **cite** §4.19; the owner protocol names behaviours in plain Japanese without restating the rule text |
| capture mechanics (PMv2 read-back, CopyFromScreen, client rectangle, foreground earned) | [verification-environments.md](../../../../docs/notes/verification-environments.md) Obs 4 | cited by name in the script header and the README; not restated |
| the width-driven toolbar overlap and its owner | [constraints.md §6](../requirements/constraints.md) | the protocol tells the owner not to judge it and links the constraint |
| T10's control-A frames and their numbers | [evidence/t10-frames/](./evidence/t10-frames/) | linked, never re-tabulated here |
| the scrim attenuation and the metric it forces | **this task** | stated once in the plan item, once in the script header, and measured in the `-Compare` output — the output is the source, the prose points at it |

No `.uic`, no schema, no parallel index exists in this task's outputs.

#### The owner smoke protocol, checked performable at the target commit

[evidence/owner-smoke/protocol.md](./evidence/owner-smoke/protocol.md),
in Japanese because the owner runs it (the Phase 1 T11 shape; the record
side stays English). The framing requires it to be "verified against the
target commit before it is used", and the verification is this mapping —
every step is performable at `fd5d192` because something at that commit
already exercises it:

| Step | What makes it performable at this commit |
|---|---|
| 0 nothing focused at open | `b-n`, this task's own frame |
| 1 Tab order, the strip as one stop, wrap, Shift+Tab | `b1`–`b5`, `brev`; G10 part 1 |
| 2 arrows inside the group | G10 part 2 |
| 3 which thumbnail opened it | `a0` / `a3` / `a0b`; G1, G2 |
| 4 covered click does nothing, then fires when closed | `c-openA-click`, `c-blocked`, `c-fired`; G5 |
| 5 Tab contained inside the lightbox | `c-openB` vs `c-tab` |
| 6 ← → and the `<` / `>` Buttons step the caption | T10's `k5` / `k6` / `k4`; G4, G8 |
| 7 `Home` does not close, `Esc` does, `x` does | `d-home`, `d-closed`; G9 |
| 8 focus restores after close | G3 leg (b) |
| 9 free operation | the host launches and survives the whole capture run |

**What the protocol does and does not leave open, stated precisely**
because the start gate's boundary 5 ("prescribes no verdict the owner
must reach") is looser than the artifact. The protocol **does** state
what should be seen at each step — it has to, since an owner cannot
judge a Tab order without knowing which order is expected — and that is
expectation-setting, the standard confirmation-bias hazard of a human
pass. What it leaves open is the **conclusion**: it asks the owner to
record "seen / not seen / seen differently, and how", never to sign off
a pass, and it says in its own closing paragraph that where the
observation and the expectation disagree it does not decide which is
right. Boundary 5 holds in that narrower sense and not in the wider one
it was written in.

#### #6 — deterministic failure, root-caused rather than re-rolled

Two failing legs, neither re-run to green. Each was reproduced, measured
at the pixel level by the lead independently of the agent that wrote the
script, root-caused to a physical fact (the scrim's own alpha; the
click-moves-focus rule), and dispositioned above. **No threshold was
loosened**: one leg's metric changed to a strictly stricter one and one
leg moved to a region where its claim is unambiguous.

The capture run itself needed no retry: the window-polling loop replaced
the fixed three-second wait CF-T11-5 recorded, and every in-run guard
(the whole-client diff after each open and each Escape) passed on the
first attempt.

#### #5 — carry-forward

T12 is the last task before the close gate, so each item has one place to
go and no later task to absorb it.

- **CF-T12-1 — a semi-transparent cover silently disables a
  visible-change threshold.** Any later no-change claim about a region
  behind a scrim, an overlay or a top layer needs its own sensor leg and
  its own metric; the attenuation factor is the cover's alpha complement
  and is derivable from the `.ui` rather than guessed. Evidence: the
  attenuation table above. Placement: `carry-forward`. Re-trigger:
  **M4-Phase 9** (the top layer is a second, probably more opaque cover),
  and any phase whose evidence is "the background did not react".
- **CF-T12-2 — a comparison script breaks as readily at "how the pass
  band was built" as at the comparison itself.** 13 of 37 verdicts here
  could not discriminate as written: 2 because a leg sampled a region the
  instrument could not see, and **11 because the band each was judged
  against was computed from the quantity it asserts** — a shape invisible
  in the leg's own text, which appears only when the band's definition is
  read beside it. A self-check pass finds the first class and **cannot**
  find the second unless it has a row for every verdict.
  **This is a recorded hazard, not a proposed rule.** A carry-forward
  under [implementation-gates.md](../../../procedures/implementation-gates.md)
  §2 #5 is an invariant a later task could trip, recorded with evidence
  and a re-trigger *even when no ADR changes* — and no rule covers this
  today: [DD-V-029](../../../cross-milestone/decisions/dd-v-029-pure-logic-red-test-obligation.md)'s
  red-test obligation is pure-logic only and states that it "does not
  widen the GUI screenshot/positive-control rule", the wider version
  having been **rejected as disproportionate** at the M4-Phase 1
  phase-end owner decision. Widening it would need a successor record,
  which is the owner's call and not this gate's. What this task leaves
  instead is a **working mechanism to copy**: `-Compare` registers its
  verdicts and `-SelfCheck` fails on any gap. Evidence: the `-SelfCheck`
  pass with its coverage assertion, and the three dispositions.
  Placement: `carry-forward`. Re-trigger: the next task whose deliverable
  is a comparison script — first candidate M4-Phase 4's scroll evidence.
- **CF-T12-3 — the frames record a behaviour no normative text
  states.** A click on a Button-family widget moves focus to it, so a
  toolbar tab ends up checked *and* focused, and §4.19 fixes the rule
  ("a click moves focus to the nearest focusable widget at or above")
  without saying that the two presentation states compose. Evidence:
  `b1`'s `(143.8, 153.2, 149.5)` against `c-closed`'s `(52.4, 121.2,
  214.4)` and `c-fired`'s `(66.6, 66.6, 66.6)`. Placement: **T13
  re-verification item** (below), not a finding — the runtime matches
  the rule; what is unstated is the composition.
- **CF-T12-4 — the owner smoke. CLOSED 2026-08-09**, run against the
  target commit with all ten steps seen as described and step 9's
  open-ended check clean. Recorded in full at §Owner human-visible smoke
  below. Not carried forward.
- **CF-T12-5 — whether to *oblige* a task to show its positive control's
  comparisons can fail is a question the next phase opens, and it is
  open.** Owner-settled 2026-08-09, in two parts. **(i) No rule this
  time.** M4-Phase 2 closes with no rule change: no edit to
  [implementation-gates.md](../../../procedures/implementation-gates.md),
  no successor to DD-V-029, no new obligation on any task in this phase.
  Nothing has to be reverted for that to be true — the start gate's
  boundary 4 held, and this task edited no normative or process text.
  **(ii) The question is carried, and decided at the next phase's
  pre-doc.** This carry-forward carries the question, not an intention:
  **the deferral is not an implied yes**, and the next phase may as
  properly answer "no rule" as "rule".
  - **What was settled, and what was not.** Settled: the placement of
    such a rule would not be the plan. This phase measured that
    plan-time predictions about *verification method* fail at about the
    rate implementation predictions do (T6's four layers, T8's lane,
    T9's "only new IR content", T11's question itself, T12's calibration),
    so loading more prediction into a document the process already treats
    as a hypothesis runs against the grain. The forcing point that
    already exists is the start gate and trap #7's close artifact. **Not
    settled: whether any of it becomes an obligation at all.**
  - **The material a future decision needs, recorded now so it is not
    re-derived.** An obligation here is a **structural** change
    ([AGENTS.md §Process rule lifecycle](../../../../AGENTS.md)): it adds
    a review obligation, and it touches
    [DD-V-029](../../../cross-milestone/decisions/dd-v-029-pure-logic-red-test-obligation.md),
    whose text says it "does not widen the GUI screenshot/positive-control
    rule". DD-V-029's own revision rule requires a **successor vision
    decision record** before extending it. Any such successor must also
    be **narrower than the version already rejected** at the M4-Phase 1
    phase-end owner decision ("every green or identical observation must
    be falsified", rejected as disproportionate) — otherwise it reopens a
    settled decision rather than extending one.
  - **Why it is not urgent, stated so the deferral is a judgment rather
    than a delay.** Neither the plan-side nor the gate-side version would
    have caught this task's eleven; those came from the independent
    review and are now bounded by a mechanical coverage check inside the
    script. A rule change here must not be presented as the remedy for
    them. M4-Phase 4's scroll evidence is the next task that writes a
    comparison script, so one more measurement exists by the time the
    question is taken up.
  - Placement: **next-phase pre-doc input**; T13 carries it into
    [handoff.md](./handoff.md). Re-trigger: drafting the next phase's
    framing.

**One finding, not a carry-forward** ([DD-V-030](../../../cross-milestone/decisions/dd-v-030-carry-forward-buildability.md):
a shape with an owner is a finding). **The lightbox caption's second line
is rendered truncated.** `examples/gallery/gallery.ui:167` authors
`"Box 4:3 placeholder; the Image widget and the photo name land later in
M4"`; every lightbox frame in this set renders it ending at `…the photo
name land`. The rendered run spans roughly x 240–737 px = 497 px, which
is the caption cell's own width (`columns: 1* 56 400 56 1*`, 400 DIP =
500 px at 1.25), so the line is laid out to the cell and stops — with no
ellipsis and nothing to tell an author it happened.

**Not a T12 regression, and not a T12 defect**: T10's committed
`t10-item-identity-a0.png` shows the identical truncation, so it predates
this task and went unnoticed there too. It is recorded here because this
task is the one that read the frames closely enough to see it. **Owner:
M4-Phase 4**, which owns what a container does when its children do not
fit ([constraints.md §6](../requirements/constraints.md), framing
agreement ④) — the same question one level down, for a single `Text`
inside a fixed track rather than for a `Row`. The status strip at the
bottom of the same frames is *not* truncated, so the behaviour is
specific to content wider than its track, not general to long text.

#### Re-audit of the whole task list (cross-task obligation)

Read T13 and the cross-task obligations again at close, not only T12's
own item.

- **T13** gains **one** re-verification item, from CF-T12-3: §4.19 fixes
  the click-moves-focus rule and §13.3 fixes what the focus indicator
  is, but **no section says the presentation states compose** — that a
  checked `ToggleButton` which is also focused renders a third
  appearance distinct from both. M4 has only a background change to
  express any of them, and DD-M4-P2-003 requires focus to be visibly
  distinct from selected *and* hovered, which this frame set measures
  and satisfies. T13 decides only whether §13.3 gains the sentence; the
  runtime is not in question.
- **T13** also inherits a **confirmation rather than a repair**: the
  T12 frames are the rendered evidence for §4.19's traversal order,
  `focus-group` single-stop rule, scope containment, restoration and
  `dismiss`, so its Moment 2 re-verification of those clauses has
  something to check against rather than only a fixture name.
- **Cross-task obligations** — no new ABI function (framing agreement ⑦
  holds trivially: this task adds a PowerShell script and Markdown, and
  no Rust at all). The stretch checkpoint is unaffected. The re-audit
  discipline itself is what produced CF-T12-3, which no leg was aimed at
  and which fell out of reading `b1` beside `c-fired`.
- **Nothing in T12 makes any normative statement false.** Every leg
  agreed with §4.19.

#### Review lane

**Full independent review**, as
[preamble.md](./preamble.md) predicted for T12 and the start gate
confirmed (GUI-render evidence). Not corrected at either gate. The trap
#4 branch-focused check composes in over the `-SelfCheck` pass rather
than replacing the full review. **The close gate was written before the
review**, keeping
[implementation-gates.md](../../../procedures/implementation-gates.md)
§0's order.

#### Verification

`cargo fmt --all -- --check` zero exit; `git diff --check` clean. Clean
rebuild in both profiles: `cargo clean` (6,652 files / 2.4 GiB removed)
→ `cargo build --release --workspace` (47.20s, zero exit) →
`cargo build --workspace` (35.46s, zero exit) →
`cargo test --workspace --no-fail-fast` **zero exit**, **1,270 passed,
0 failed, 0 ignored** across **51 sections** (46 test binaries + 5
doc-test sections), with **zero** `test result: FAILED` blocks —
`wasamo_runtime` unit 609, `gallery_slice_integration` 10,
`touch_pointer_integration` 10. Counted in **sections**, the unit every
earlier gate in this phase used, so the figure is comparable to the
series it sits in: T10 recorded 50 / 1,258 and T11 added
`touch_pointer_integration.rs` (10) plus two `window.rs` unit tests.
T12 adds no Rust, so the delta is zero and the reconciliation is exact.

This is a **regression check, not evidence** that the controls are
right ([preamble.md §What "green" is worth](./preamble.md)): no test in
the suite routes an event through a real window and a rendered frame,
which is the whole reason this task exists. The evidence is the frame
set and its two comparison passes.

`-Compare` and `-SelfCheck` were each re-run by the lead independently
of the agent that wrote the script, against the committed frames: exit 0
and exit 0, with every number reproducing.

#### Independent review, and the defect it found that the close gate had missed

Performed by a subagent that wrote none of the code and had not seen the
capture run, over the whole branch diff plus the start gate, the plan
item, DD-M4-P2-001–005, §4.19 and Observation 4. It re-ran `-Compare`
and `-SelfCheck`, read six of the frames as images, and ran one
non-invasive experiment against a scratch copy of the frame set.

**Fourteen findings. Eleven taken, one narrowed, two carried.** The
lead re-measured every one rather than accepting it as reported.

**The one that matters — R1: a band computed from the quantity it
judges.** Each region's pass band was `max(measured within-set noise × 4,
floor)`, and the eleven "two frames with no input agree within the
measured jitter" legs were then compared against it. Since a leg's own
value is bounded by the maximum the band is built from, **no frame set
could ever fail them.** The reviewer did not stop at deriving that: it
copied the 48 frames to a scratch directory, painted 2,000 magenta pixels
into `b-n-1`'s toolbar band, and re-ran — the jitter leg **passed** with
gross jitter present while all four control-B difference legs went red,
because the inflated band had propagated to every leg in that region. A
noisy sitting would therefore have destroyed the real claims and
certified the noise.

This is the same failure as the task's own two findings — a leg that
passes without being able to fail — in the one place the start gate's
§Wider (1) reading did not reach. That reading armed a check on the
**legs**; nothing armed a check on the **bands**. The self-check pass
could not have found it either, because the eleven legs had no
self-check row, and they had no row for the same reason they could not
fail. **Two blind spots that hid each other**, which is why an
independent reader found it and neither gate did.

Remediation, all four defects, verified by the lead re-running both
passes: every band is now a **chosen constant with a stated reason**, and
the measured noise is a **checked quantity** — a six-row pre-flight gate
requiring each region's within-set jitter to sit inside F-33's 13/channel
with no pixel over the visible-change bar, failing the run rather than
absorbing the noise into a band; the eleven jitter legs judge against
those same independent constants; `-SelfCheck` covers all 43 registered
verdicts plus the two extracted in-run guards, with coverage enforced by
the script; and the DIFFERENCE rows use region-scoped wrong pairings.
`-Compare` 43/43 and `-SelfCheck` 45/45, both exit 0, with **every
measured number unchanged** — the frames were not re-captured and did not
need to be, because the defect was in how the numbers were judged, not in
the numbers.

**Ten further findings taken, all in this log or the plan:** the
"fourteen legs" denominator did not reproduce (it is 13 of 37, and the
distinction between the 2 the task found and the 11 the review found is
the interesting part); "the measurement and the `.ui` agree to two
decimal places" was false (5.06 against 5.00, a 1.2% gap consistent with
quantisation); the prose in three documents described a stricter
threshold than the code implements; "the proxy is **shown** conservative"
was an inference stated as a measurement; the suite figure changed
counting unit mid-series (46 binaries against the phase's 51 sections);
`-SelfCheck`'s DIFFERENCE rows were all degenerate; the in-run guards
were unreachable verdict branches; the bands were picked constants
described as measurements; and the link between the capture geometry and
the geometry an owner gets by not resizing the window was real but stated
nowhere. Each is corrected in place above.

**R6 is the one worth naming separately, because it is a defect in the
lead's own reading of the evidence.** The close gate claimed `b1` — `All`
alone turning pale, `Albums` and `Favorites` untouched — showed what
`focus-group: true` means. It does not: a build with **no** grouping at
all, three separate stops, renders `b1` identically. What carries the
single-stop rule is `b2`, whose painted bbox is `x[564..693]` = `Scroll
down` while `Albums`' face sits near `x[80..178]`. The evidence was
present and cited against the wrong frame. Corrected above. A frame that
*looks* like the property is not the frame that *distinguishes* it —
which is the trap #7 principle turned on the analysis rather than on the
capture.

**One narrowed (R9).** The reviewer read boundary 5 ("the owner smoke
protocol prescribes no verdict") as in tension with a protocol whose
every step states what should be seen. Correct, and the boundary's wording
was the loose half: the protocol must state expectations or an owner
cannot judge a Tab order at all. Boundary 5 is restated above in the
sense that is true — the protocol leaves the **conclusion** open, not the
expectation.

**Two carried as observations rather than corrections.** Control A's
caption difference has the thinnest margin in the set (79 against a band
of 40) and is the one text-only region, so it is the leg most exposed to
a future sitting with real F-33 jitter — now bounded by the noise gate,
which fails such a sitting instead of widening the band under it. And the
reviewer noticed the truncated lightbox caption independently of the
lead, which is recorded as a finding with M4-Phase 4 above.

**What the review did not check, in its own words:** it did not run
`-Capture`, so the guards, `Try-Activate` and the capture sequence are
unexercised by it; it read 6 of the 48 frames; it did not verify start-gate
facts 1, 2, 3, 5 or 7 independently, nor T10's frames. Those are the
lead's measurements, taken at the start gate and during the run, and they
stand on that rather than on the review.

#### Owner human-visible smoke — run 2026-08-09, all ten steps as described

The owner ran [evidence/owner-smoke/protocol.md](./evidence/owner-smoke/protocol.md)
against the target commit and reported **every step seen as described**,
including step 9's open-ended one: *違和感なし* — no discomfort in free
operation, which is the only step the protocol leaves without a stated
expectation.

**CF-T12-4 closes.** The assistant baseline and the owner's pass are now
both in hand, and neither substitutes for the other
([CLAUDE.md §Testing rules](../../../../CLAUDE.md)): the frame set shows
that specific state reaches specific pixels, and the smoke shows that a
person operating the app finds it works. T12's deliverable is complete.

**What this pass does and does not establish**, stated because the
protocol's own shape bounds it:

- **It is confirmation against stated expectations, not independent
  discovery.** Every step but the last names what should be seen, because
  an owner cannot judge a Tab order without knowing the expected order.
  That is the confirmation-bias hazard the review's R9 narrowed boundary 5
  around, and it is why the *discriminating* work sits in the frame set —
  which compares measured pixels against legs that were each shown able to
  go red — rather than here. The two are complementary in exactly that
  direction.
- **Step 9 is the part that could have found something unplanned**, and it
  is the part with no expectation to confirm. It came back clean.
- **It says nothing about touch** (framing agreement ⑥: no touch hardware;
  T11's synthesized injection carries that half with its limit stated), and
  nothing about the two things the protocol tells the owner not to judge —
  the width-driven toolbar overlap (M4-Phase 4) and the picture inside the
  lightbox (M4-Phase 3 / 4).
- **It neither corroborates nor contradicts the truncated-caption
  finding** above, for that last reason: the caption's second line is
  inside the lightbox content the protocol excludes. The finding stands on
  the frames and on `gallery.ui:167`, not on this pass.

## T13 — Close gate

### Start gate (recorded 2026-08-09, before any T13 normative edit)

Read before choosing the approach, in the owner-specified order:
[AGENTS.md](../../../../AGENTS.md),
[implementation-gates.md](../../../procedures/implementation-gates.md),
[retrospectives.md](../../../procedures/retrospectives.md),
[workflow.md](../../../procedures/workflow.md), [plan.md](./plan.md) §T13,
this log through T12, retrospective items 10 and 11 for T1–T12, the five
Accepted DDs and their [preamble](../decisions/preamble.md), the framing
acceptance and verification sections, constraints §6 / §9, the M4 plan and
roadmap ACs, the Phase 1 handoff, every normative section named by T13, and
[verification-environments.md](../../../../docs/notes/verification-environments.md)
Observations 4 / 6. The branch is `feat/m4-phase-2-t13`, created from the
clean, unpushed phase tip `0676acb`.

#### Boundary fixed before the pass

T13 owns re-verification and phase-close records, not a runtime repair. A
false normative statement is corrected, an unstated landed rule is either
recorded or explicitly left unspecified within the narrow choice the T13
item assigns, and an Accepted DD whose conclusion still ships receives only
the owner-authorised dated explanatory annotation. A discrepancy outside
those bounds is recorded with an owner; it is not silently fixed. No process
rule changes in this task. In particular, CF-T12-5 is carried as an **open
question**, with no implication that a later phase will make the proposed
obligation a rule.

#### Failure-mode selection

| Trap | Applies? | Start-gate reason and close artifact |
|---|---|---|
| #1 semantic migration | **No** | T13 changes no enum, IR variant, schema field or traversal. Re-decide if the re-verification unexpectedly requires source or schema work; that would exceed the current boundary and be recorded before implementation. |
| #2 missed side effects | **Yes — documentation analogue.** | The close changes several coordinated phase-close surfaces. The close artifact will enumerate every touched normative statement, status / progress marker, ADR annotation, AC closure, residual pointer, retrospective mapping, handoff entry and task checkbox, including why untouching `abi_spec.md`, `VISION.md`, `CHANGELOG.md`, or a named candidate is correct. |
| #3 parallel / derived data drift | **Yes — documentation analogue.** | `plan.md`, `log.md`, `phase-end.md`, `handoff.md`, roadmap / milestone progress and ADR revisions can become competing summaries of one rule. The artifact will identify the owning source for each statement and use pointers rather than restating normative behaviour in a derived ledger. |
| #4 untested authored branch | **No.** | No code, diagnostic, reject arm, size arm or script branch is planned. Re-decide if T13 authors executable logic. |
| #5 carry-forward underweighted | **Yes.** | T1–T12 carry-forward and findings are reclassified at phase end as doc-folded / carry-forward / local-only or retained finding, with evidence, owner and re-trigger. The final handoff is written only after that phase-end classification. |
| #6 symptom at face value / flake-rolling | **No current failure.** | T13 begins from green recorded T12 evidence and no recurring failure. Any deterministic or twice-recurring rebuild, suite, evidence, or doc-check failure activates this trap and receives rerun history, root cause and disposition rather than a retry-to-green. |
| #7 weak GUI evidence | **No new GUI-evidence deliverable.** | T13 does not launch or capture a GUI; T12 already produced and independently reviewed the 48-frame, positive-control set and the owner completed all ten smoke steps. T13 must nevertheless inspect the committed frames when confirming the five named runtime properties and the checked+focused composition; its artifact is a confirmation mapping to T12's already-closed #7 evidence, not a second claim that process survival rendered correctly. Re-decide if a new capture is needed. |

#### Normative answers already in force, and known divergences entering the pass

This phase was normatively synchronised ahead of implementation, so
DD-V-031 requires the already-answering text to be named before work. The
table is an input classification, not a conclusion that each sentence is
still true:

| Owner | Answer already present | Start-gate classification against landed evidence |
|---|---|---|
| `dsl_spec.md` §4.8 / §4.19 | disabled occlusion, focus eligibility and propagation | Occlusion and Tab skipping answer the runtime; the keyboard-activation wording overclaims a capability the runtime does not have and the owner sent to the candidate pool. |
| `dsl_spec.md` §3 / §4.5 / §8.8 | authored signal and textual-IR handler grammar | They still encode the pre-argument / `clicked`-only surface and are known false after `dismiss` and `key-down(\"…\")`. |
| `dsl_spec.md` §4.15 | per-item handler admission, invocation-time binder reads, positional identity and shared subtree lifetime | The admitted subsection answers the landed T9 model; two rows in the Diagnostics table still reject that same model and are known false. |
| `dsl_spec.md` §4.16 | child-carried placement examples | T8 corrected the childless-widget example; T13 confirms rather than repairs it. |
| `dsl_spec.md` §4.19 | target/bubble routing, key admission and fallthrough, focus groups, modal containment/restoration and dismissal | The core answer exists. Named gaps entering this pass are arrow-axis direction, outside-scope click focus, container candidacy consequence and the unrecognised-signal policy; the Button keyboard sentence is false. |
| `dsl_spec.md` §8.9 | string handler expressions are binding-only and assignment admits the documented scalar/collection forms | Runtime rejection matches the text, while checker/lowering/loader accept the string assignment until invocation; owner assigned capability to Phase 5 and diagnostic intake to Phase 3, so T13 records an unenforced normative statement rather than changing behaviour. |
| `architecture.md` §12.3 | enumerable DIP / device conversion seams | The pointer row omits screen-to-client conversion for `WM_POINTER*` and still claims Visual readback after T2 removed it. |
| `architecture.md` §13 | layout-derived hit geometry, routing, touch, focus, scopes and dismissal | The model answers most landed behaviour. Known wording/gaps concern touch precision and focus/hover policy, the focus-indicator write path and composed state, and the pre-/post-mutation split in scope restoration. |
| DD-M4-P2-001 | one drain per dispatch and no residual-1 cycle | The conclusion ships, but F5 falsifies the reason that regeneration waits for handler return. Per the 2026-08-08 owner disposition and `workflow.md`, this is a dated annotation, not a supersede. |

The recognised-key table, `dismiss` admission, child-kind admission, exact
checker/runtime callers, conversion sites and the scope/focus mechanics are
not accepted from prose alone: the close pass audits the landed code and
tests. T12's frame set is the rendered cross-check for traversal order,
single-stop grouping, containment, restoration, dismissal and composed
checked/focused appearance.

#### Review lane

**Normal review, confirmed at start and subject to close-time re-decision.**
The implementation preamble's T13 prediction remains correct for the present
boundary: normative documentation, phase-close ledgers and one dated
explanatory ADR annotation change, with no schema / IR migration, runtime
structural change, newly authored diagnostic / reject / size branch, or new
GUI capture. T12's GUI-render evidence already received full independent
review; this task maps and re-verifies it without replacing or widening its
claim. If T13 crosses any of those boundaries, the applicable full or
branch/test-focused lane composes before merge.

### Moment 2 implementation re-verification

This is the item-by-item comparison required by `plan.md` §T13. The
runtime source and the tests that exercise it were read alongside the
normative sentence; a green suite is recorded separately at the end gate
rather than being substituted for this comparison.

| # | Plan check | Landed evidence and disposition |
|---|---|---|
| 1 | Disabled occlusion versus child traversal | `hit::resolve_topmost` chooses one target before dispatch and `ClickDisposition::Suppress` prevents a disabled Button from firing without exposing a lower sibling. §4.8 / §4.19 already agreed; no child-traversal wording survived. |
| 2 | Disabled traversal skip | `focus_core::collect_stops` omits disabled stops; `a_disabled_stop_is_skipped` pins it. Retained in §4.8 / §4.19. |
| 3 | Disabled Tab wording | The runtime has no Button key activation (`run_clicked_handlers` is reached from pointer click only). Both sections now state the shipped Tab consequence and no longer imply a keyboard-activation feature. CF-T8-1 remains in the candidate pool for the M5 widget-family decision. |
| 4 | §4.19 `for` example under §4.15 | The example has exactly one body-root widget and uses `for photo, i in photos`; the parser's optional index-binder production accepts it. The gallery and the per-item integration fixture exercise the same form. |
| 5 | Per-item handler semantics | `ForItemHandlerEvalContext` resolves item/index at invocation; `per_item_handler_integration` covers click identity, same-length reset, surviving position and removal failure. Handler registration is owned and released by the generated subtree. §4.15 / §4.19 match. |
| 6 | False §4.15 diagnostic rows | Removed the two rows that rejected handlers and handler-position binder reads, and corrected the body bullet that still prohibited handlers anywhere in the template. |
| 7 | §8.9 string-assignment enforcement | Reconfirmed: §8.9 marks `StrLit`, `StrPropRead` and `Interpolation` binding-only; checker, lowering and loader admit an `(assign …)` string RHS, while evaluator invocation rejects it. **Unenforced normative statement retained**, not rewritten as a supported capability: capability owner M4-Phase 5; diagnostic pre-doc intake M4-Phase 3 (milestone-plan revision 1). |
| 8 | DD-M4-P2-001 residual-1 reason | Added the owner-approved 2026-08-09 qualification and preamble revision pointer. T9 F5 shows regeneration inside `Signal::set` during the handler statement; the no-cycle conclusion and decision are unchanged, so no supersede. |
| 9 | Recognised keys and fallthrough | `wasamo_ir::RECOGNISED_KEY_NAMES` contains the exact 22 names printed in §4.19; runtime `key_name_for_vk` is anti-drift tested against it. `WM_KEYDOWN` returns only for traversal/group/dismiss/handled authored key/host slot; otherwise it reaches `DefWindowProcW`. |
| 10 | `dismiss` admission and kept-key table | Checker and loader both require a sibling `modal-scope: true`; `dismiss_on_key` addresses Escape to the innermost scope. Tab, group arrows and scoped Escape are the only built-ins kept ahead of the authored walk. Text unchanged. |
| 11 | Focus-indicator write path | `effective_button_color` and `set_focused` repaint node presentation; no focus path creates or positions a Visual. The six `SetOffset` / `SetSize` call sites remain in `sync_visuals`. §13.3 now states the landed split. |
| 12 | Presence-entry and clip bound | `sync_scopes_to_tree` runs at initial root attachment and structural drain; entry moves focus to the first stop. `resolve_topmost` intersects ancestor clip bounds. §13.1 / §13.4 match. |
| 13 | Restore versus successor sequence | `sync_scopes_to_tree` uses the restore anchor captured at entry, then falls back to the first surviving post-mutation stop. §13.4 now states those two times rather than claiming every successor is computed before mutation. |
| 14 | Focus-group arrow axes | `arrow_direction`: Left / Up → previous; Right / Down → next. Added the exact mapping and both-axis rule to §4.19. |
| 15 | Click outside an entered scope | `focus_landing_outside_an_entered_modal_scope_is_none`: landing is bounded by `traversal_root`, so focus is unchanged. Added the sentence to §4.19's pointer-limit paragraph. |
| 16 | Authored and textual-IR handler grammar | Synced §3 to `IDENT ("(" STRING_LIT ")")? "=>" block`, added the `IDENT` + `(` disambiguation row, and synced §8.8 to the optional `STRING` argument used by `on key-down("…")`. |
| 17 | §4.5 signal inventory | Replaced the false clicked-only statement with the three defined signals and their §4.19 admission pointer. |
| 18 | Button keyboard activation | Owner outcome from T8 retained: do not build it here; revisit one activation contract with the M5 keyboard-operable widget set. The two false normative clauses were narrowed, and the keys-kept table remains unchanged. |
| 19 | Unknown signal diagnostic | Current checker and loader accept arbitrary bare signal names; no DD requires narrowing that surface. §4.5 now states that other names' semantics and diagnostic requirements are **unspecified**. Re-trigger remains a fourth signal or the first silent-handler bug report. |
| 20 | Child admission | `wasamo_ir::LAYOUT_CHILDLESS_WIDGET_KINDS` is the shared four-kind owner read by checker and loader. §4.4 now states that Rectangle / Text / Button / ToggleButton admit no widget children and points container rules to their sections. |
| 21 | Containers as hit candidates | Every `WidgetNode` visual participates in `resolve_topmost`; T10 G7 measured the overflow consequence. §4.19 now says a non-clipping layout container's painted overflow remains reachable and can occlude a sibling; M4-Phase 4 still owns the overflow-layout policy. |
| 22 | §4.16 placement example | Confirmed the example uses direct child-carried `slot.row` / `slot.column` on a Button and does not put a widget child inside a childless kind. No T13 repair. |
| 23 | Touch-promotion precision | `WM_POINTERDOWN` and `WM_POINTERUP` are the promotion-suppression gates; ENTER / UPDATE / LEAVE are claimed but inert. §13.2 now states that measured division. |
| 24 | Touch focus / presentation / primary contact | `touch_pointer_integration`: a tap focuses like click, non-primary dispatches nothing, and the inert-message test pins no hover/pressed mutation. §13.2 now states all three limits, including no touch-down feedback in M4. |
| 25 | Pointer screen-to-client conversion | `pointer_message_to_client_dip` performs `ScreenToClient` before DIP division for `WM_POINTER*`; mouse is already client physical. Added both paths to §12.3's enumerated inbound seam. |
| 26 | Stale Visual readback sentence | Hit rectangles have been retained in DIP since T2. Removed §12.3 row 2's false Visual-readback conversion and pointed to §13.1. |
| 27 | Checked + focused composition | `effective_button_color` composes `checked` before the focused blend; its colour tests and T12 frames show a third appearance. Added the composition sentence to §13.3. |
| 28 | Fixture spelling in the public DSL spec | `rg` found no M4-Phase 2 fixture identifier or test-only signal spelling in `docs/dsl_spec.md`; only public DSL / IR forms appear. |

One additional mechanical drift was found while doing the named pass:
`dsl_spec.md`'s revision table already ended at 1.20 while the header still
said 1.19. The Moment 2 record is 1.21 and brings the header forward; this
does not change a language rule.

### Rendered-evidence confirmation mapping

T13 inspected the committed PNGs, not only their fixture names. No new
capture is needed and trap #7 remains out of scope: this mapping confirms
the already independently reviewed T12 claim without widening it.

| Runtime property | T12 rendered positive control |
|---|---|
| Declaration-order traversal | `b1` → `b2` → `b3` → `b4` paints All → Scroll down → Scroll up → Open lightbox; `b5` wraps and `brev` reverses. The differentiating single-group-stop frame is `b2`, because Albums / Favorites were skipped. |
| `focus-group` is one Tab stop | Same B sequence: after All, the next painted focus is Scroll down, not another tab. |
| Scope containment and covering-widget occlusion | `c-openA` ≡ `c-openA-click` / `c-blocked` ≡ `c-closed` in their agreement regions; five Tabs move among lightbox controls in `c-openB` / `c-tab` without painting toolbar focus, while after close `c-tab-closed` reaches the toolbar. |
| Restoration and dismissal | `d-closed` ≡ `d-pre` shows Escape returned to the pre-open focused state; `d-home` ≡ `d-open` is the recognised-but-unhandled-key agreement leg, excluding a generic redraw or key effect. |
| Checked/focused composition | `b1` shows checked+focused All distinct from checked-only and neither; `c-fired` shows the clicked Albums state while the previously checked All changes, excluding focus colour as the handler result. |

The owner separately completed the ten-step human-visible smoke on
2026-08-09 with every step as described, including no discomfort during
free operation. That evidence remains T12's; T13 does not relabel it as a
new observation.

### Close-time review-lane re-decision before verification

The lane remains **Normal review**. The realised diff contains one dated
qualification on an Accepted DD, normative prose / grammar corrections,
and phase-close records. It contains no runtime, IR/schema migration,
diagnostic/reject/size branch, or new GUI evidence. Thus no full or narrow
independent-review trigger was added by T13; T12's existing full review is
the owner of the rendered-evidence quality claim.

### End-gate attempt 1 — CF-T7-1 re-triggered (2026-08-09)

The required cold sequence reached a real phase residual rather than a
documentation failure:

| Command | Result |
|---|---|
| `cargo clean` | pass; 329 files / 180.0 MiB removed |
| `cargo build --release --workspace` | pass in 61.3 s; only the known `wasamo-sys` import-library ordering warning and linker import-library message |
| `cargo build --workspace` | pass in 48.3 s; same warnings |
| `cargo test --workspace` | **fail** in `focus_identity_integration::a_for_regeneration_that_frees_and_allocates_leaves_a_consistent_focus_record` after the preceding suites passed (including 32 `wasamo-ir` and 609 runtime unit tests) |

The failing run freed the focused row at `0x294cae60990` and allocated
row 9 at the **same address** in the same message. `WindowFocus::rebase`
therefore matched the stale anchor to the new node and retained focus path
`[3]`, but the new node's presentation flag was `Some(false)`. The focus
record and its painted indicator — documented as a derived pair with one
writer — were inconsistent.

Trap #6 is active. Three exact reruns against the unchanged build produced
**pass, pass, fail**, and each outcome matched the allocator observation:

| Exact rerun | Address reuse | Result |
|---|---:|---|
| 1 | false (`0x1819cf4bf90` → `0x1819cf4d090`) | pass |
| 2 | false (`0x2913d6eafb0` → `0x2913d6ecc60`) | pass |
| 3 | true (`0x1995196d240` → same) | **same assertion failure** |

This is not a retry-to-green flake: address reuse is nondeterministic, but
the failure is deterministic whenever reuse occurs (two observed reuse
runs, two identical failures). It fires CF-T7-1's exact re-trigger and
falsifies its former bound that reuse could only choose an unexpected
focus target while keeping the record/presentation pair consistent.

Static diagnosis: `WindowFocus::rebase` remaps ids by pointer, adopts the
new projection and sees focused id 3 survive. Because focus never becomes
`None`, `sync_scopes_to_tree` takes neither restoration nor structural
succession and invokes no `with_focus_write`; the freshly allocated row
therefore never receives the focus repaint. The smallest compatible repair
is to reconcile the final retained focused id through the existing
`with_focus_write` primitive after rebase and the exit / succession /
entry decisions it feeds. That preserves the already-documented
pointer-identity bound
(the new row may become the unexpected focus target) while restoring the
record/presentation invariant. A deterministic regression witness must
force or simulate the lost-presentation state rather than rely only on
allocator reuse.

That repair is **not silently taken**. It crosses T13's recorded no-runtime
boundary, changes the structural focus seam and therefore changes the
review lane to **full independent review**. The remaining local suites and
the push/CI request are paused pending the owner's choice: add a gated
repair task before close, or stop phase close and route the repair to a
separate task. T13 remains unchecked and no completion claim is made.

### T13a start gate — owner-authorized runtime repair (2026-08-09)

The owner authorized the repair inside T13 after reviewing the cold-suite
failure and the distinction between an implementation defect and the
probabilistic reach of its existing test. The implementation boundary is
now widened, visibly and narrowly: preserve pointer-anchor focus identity,
repair only the post-rebase record/presentation inconsistency, and add a
deterministic witness. A stable generation id or a new identity policy is
outside this authorization.

#### Failure-mode re-selection before code

| Trap | Applies? | T13a reason and required close artifact |
|---|---|---|
| #1 semantic migration | **No.** | No enum/schema/IR migration and no focus-identity policy change. The existing `FocusId` / pointer-anchor representation stays. Re-decide if a generation id or new retained token becomes necessary. |
| #2 missed side effects | **Yes.** | Rebase runs after every initial attach / structural drain. Enumerate focus record, group memory, active items, modal stack, restoration/succession, current and stale presentation, Composition failure handling, and every `sync_scopes_to_tree` call site. |
| #3 parallel / derived data drift | **Yes.** | `WindowFocus::core.focused`, its anchor coordinate system and `ButtonData.focused` presentation are one derived triple. The repair must use `with_focus_write`, not create a second presentation writer. The deterministic test must show the previously inconsistent pair converges. |
| #4 untested authored branch | **Yes — test-side forced state plus repair path.** | The existing allocator branch is probabilistic. Add an unconditional repair step and an allocator-independent integration witness that is red with the repair removed; retain the reuse test to cover the natural path when reachable. |
| #5 carry-forward underweighted | **Yes.** | CF-T7-1 / CF-T9-1 moves from carried residual to fired defect. On success, close only the record/presentation inconsistency; retain the pointer-identity semantic bound and its re-trigger. Update retrospective/handoff from close-blocking to the exact residual that remains. |
| #6 symptom / flake rolling | **Yes.** | The four-run evidence is already recorded: reuse false → pass twice, reuse true → identical failure twice. No number of green non-reuse runs dispositions it. Repair evidence begins with a deterministic red witness, then the full suite. |
| #7 weak GUI evidence | **No new GUI-render claim at start.** | T13a's claim is the runtime's retained presentation flag paired with its focus record, observed through a mock-free real-Compositor integration test. It does not claim a captured host frame. T12 remains the evidence that focus presentation generally reaches pixels. Re-decide if implementation/review makes a new rendered claim necessary. |

#### Normative answer and approach boundary

DD-M4-P2-003 and `architecture.md` §13.3 already require one focus
record and one presentation-write discipline; no new semantic answer is
needed. `with_focus_write` is the owning primitive. The stochastic test's
assertion is also retained: whatever node the record names must carry the
focus presentation. T13a repairs implementation to those answers; it does
not weaken either text to accept the inconsistency.

#### Review lane re-decision

**Full independent review.** T13a changes the structural focus seam and a
GUI-visible presentation transition, so the original Normal T13 lane is
superseded for the combined close. The full review includes the narrower
branch/test-focused review of the forced-state witness. It must examine
the realised code and tests after the repair, not only this plan.

### T13a repair evidence and structural audit (2026-08-09)

Commit `428d62a` keeps pointer-anchor identity unchanged and adds one final
presentation reconciliation at `sync_scopes_to_tree` step 6. It runs after
restoration, structural succession and modal entry have selected the final
focus target; placing it earlier would needlessly animate an intermediate
same-address target that one of those operations could immediately replace.
The write closure is empty. The existing `with_focus_write` primitive alone
resolves the final path and calls `set_button_focused_at(..., true)`; no
second production writer was added.

#### Deterministic failure and repair witness

The new mock-free Windows integration fixture uses a real window and real
Compositor, focuses generated `row 3`, then uses one test-only seam to clear
only that node's cached `ButtonData.focused` flag while retaining focus path
`[2]`. Clicking a non-focusable `Box` appends `row 9` and therefore reaches
the real reactive structural drain. Its postcondition is the complete
derived pair: focus remains `[2]` and the node there is again `Some(true)`.
The child-count and `row 9` assertions prove the structural drain actually
ran rather than letting an unrelated focus write make the test green.

| Realised variant | Exact command | Result |
|---|---|---|
| Repair present | `cargo test -p wasamo-runtime --test focus_identity_integration a_structural_rebase_reconciles_a_retained_focus_records_presentation -- --exact --nocapture` | pass |
| Step-6 `with_focus_write` call temporarily removed, no other change | same command | **deterministic fail**: retained path `[2]`, node flag `Some(false)` instead of `Some(true)` |
| Repair restored | same command | pass |
| Realised integration file | `cargo test -p wasamo-runtime --test focus_identity_integration` | pass, 5 / 5 |
| Natural allocator witness, unchanged build | exact CF-T7-1 test repeated six times | pass, 6 / 6; all six happened to be non-reuse runs and are supporting evidence only |

This dispositions trap #6 without retry-to-green: the forced-state fixture
is the gate, while the allocator-address print remains the natural-path
observer. The test seam has exactly one call site, in that fixture; it does
not repaint or alter the focus record, so it cannot itself perform the
repair under test.

#### Complete production call-site audit

`rg "sync_scopes_to_tree\\(" wasamo-runtime/src` finds the definition and
exactly these two production calls:

| Caller | State on entry | Effect of new final reconciliation |
|---|---|---|
| `window::set_root` | The replaced root's focus record was reset to default; initial modal entry may then select a stop. | No-op with no focus; otherwise reasserts the stop already painted by modal entry. No root-swap identity is retained. |
| `emit::flush_layout` Phase 2 | All reactive conditional / iteration insertions and removals have materialised; rebase, exit / succession and entry run here. | Repaints only the final retained target when its cached flag is false, including the fired same-address allocation case. |

#### Structural side effects and derived-state disposition

| State / effect | Disposition |
|---|---|
| `WindowFocus::core.focused` and `anchors` | `rebase` still writes this coordinate pair exactly as before. The empty reconciliation closure changes neither. |
| Group memory, per-widget active items, modal stack | Unchanged by the empty closure. Restoration, succession and entry retain their previous order and algorithms. |
| Current presentation | The final focused Button-family node is set to `focused = true`; the guard makes an already-correct flag a no-op. |
| Previous / stale presentation | `prev_path == next_path` in the empty write, so it clears no surviving node. A removed node no longer exists. Ordinary focus transitions still clear their previous path in the same primitive. |
| Composition animation failure | As for every existing focus write, the cached flag changes before animation and the structural seam ignores the returned error. No new rollback policy is introduced; the deterministic fixture proves the cached record/presentation invariant, not pixel delivery under a Composition failure. |
| Pending signal queue / dirty set | The repair invokes only the focus repaint primitive and enqueues no handler or layout work. It is after the existing modal-entry pending-count assertion and does not change the drain algorithm. |
| Pointer-address ABA | **Not closed.** A fresh same-address node may still become the unexpected retained target. T13a closes only the additional record/presentation divergence once that accepted identity rule selects the target. |

Trap #7 remains non-applicable to the T13a claim: the new assertion is a
runtime derived-state witness, not a claim that a captured frame displays a
particular pixel. It does use the real Windows runtime and Compositor rather
than an OS mock. T12's separately inspected frames and owner-visible smoke
remain the phase evidence that the same focus-presentation path reaches the
screen in ordinary use. This classification is included in the pending full
independent review rather than treated as self-approved.

A warm `cargo test --workspace` after `428d62a` passed the complete
workspace in 55 s, including 32 `wasamo-ir` tests, 609 runtime unit tests,
all mock-free Windows integration binaries, 480 `wasamoc` tests and 8
round-trip tests. This is an early regression check only. T13a still owes
the end gate's new `cargo clean` sequence, external C / Zig hosts and the
independent-review disposition before it can be checked.

### T13a full independent review (2026-08-09)

An independent agent that authored none of `428d62a` read `AGENTS.md`, the
implementation-gate procedure, the realised commit and the T13a gate. It
independently ran the deterministic fixture (1 / 1 pass, no skip), derived
the two production call sites and the side effects, and reported **no
blocking correctness or test-validity finding**.

The reviewer agreed that step 6 belongs after restoration, succession and
modal entry; that the forced cached-flag / retained-path state plus a real
structural drain discriminates the repair; and that §13.3 already owns the
normative one-presentation-primitive rule, so no further normative edit is
required. It also agreed with trap #7's classification only under the exact
claim used here: derived runtime-state reconciliation through the already
pixel-verified primitive. T13a does not claim a new captured frame for this
path.

One low documentation finding was accepted: `ButtonData.focused`'s field
comment still named `move_focus` as `set_button_focused_at`'s sole caller,
while the method's own comment and realised code correctly name
`with_focus_write`. Commit `9a4610b` corrects that stale caller name. It
changes no code or test behaviour. With that remediation, the required full
review is complete.

### T13a end gate — final local verification (2026-08-09)

The implementation-gate procedure was read again at close. The realised
artifacts close traps #2 / #3 with the production call-site and structural
side-effect tables above, #4 with the forced-state integration branch and
its removal mutation, #5 with the narrowed handoff residual, and #6 with
the two reuse failures followed by a deterministic repair gate. Trap #1
remains inapplicable (no enum, schema or IR migration). Trap #7 remains
inapplicable under the independently reviewed derived-state-only claim; no
new path-specific pixel claim is made.

Run against the repaired branch after `428d62a`, review remediation
`9a4610b` and the repair-gate record `2871d49`:

| Command | Result |
|---|---|
| `cargo clean` | pass; 9,537 files / 2.8 GiB removed |
| `cargo build --release --workspace` | pass in 50.57 s; only the known import-library ordering / linker messages |
| `cargo build --workspace` | pass in 43.08 s; same known messages; creates the uplifted debug runtime archive required before cold workspace tests |
| `cargo test --workspace --no-fail-fast` | pass in 80.5 s; 1,271 tests, 0 failed, 0 ignored / skipped (the previous 1,270-test T12 total plus the one T13a fixture) |
| `target\\release\\wasamoc.exe check examples\\gallery\\gallery.ui` | pass |
| Visual Studio CMake configure + `--build build/gallery-c --config Release` | pass; regenerated `gallery.uic` / `gallery_uic.h` and produced `gallery-c.exe` |
| quoted `zig build` ReleaseSafe invocation in `examples/gallery-zig` | pass; `gallery-zig.exe` present |

The first CMake invocation by bare command name failed before configure
because this PowerShell session has no `cmake` on `PATH`. The executable
was found at Visual Studio 18 Community's bundled CMake path and the same
configure/build then passed. This is a tool-resolution fact, not a rolled
build failure. DSL and Zig were rerun individually because the initial
parallel wrapper stopped reporting when the CMake command could not be
resolved.

T13a's authorized repair, deterministic regression gate, end-gate audit,
full independent review and clean local evidence are therefore complete.
Actual GitHub Actions is not claimed: the branch remains unpushed and the
owner's separate push authorization is still required.

### T13 phase-branch CI completion (2026-08-09)

After explicit owner authorization, `feat/m4-phase-2-t13` was pushed and
the `CI` workflow was dispatched against exact phase HEAD
`11f77b689bc234453d2e9ff2f6a1a540c879320a`. GitHub Actions
[run 31298945418](https://github.com/matarillo/wasamo/actions/runs/31298945418)
completed **successfully**; job
[93208499151](https://github.com/matarillo/wasamo/actions/runs/31298945418/job/93208499151)
ran for 4m08s.

Every required step was green: release and debug workspace builds,
workspace tests, MSVC and clang-cl C ABI smoke, CMake smoke, Zig binding
smoke, the C / Rust / Zig counter consumers, the C / Rust / Zig gallery
consumers, and `wasamoc check` for both `counter.ui` and `gallery.ui`.
The run emitted one non-failing annotation: `mlugg/setup-zig@v2` declares
Node.js 20 while GitHub currently forces it onto Node.js 24. The action and
all downstream Zig steps passed, so this is recorded as an upstream action
maintenance observation rather than a phase failure or a reason to change
CI YAML inside T13.

The dispatched SHA includes every code, test, normative-spec, handoff and
retrospective change through the local close record `11f77b6`. Only this
run-id recording and status flips under `process/` follow it. Item 16 is
therefore satisfied for the current code tree. No merge has been performed;
phase-to-`main` remains a separate explicit owner gate.
