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
