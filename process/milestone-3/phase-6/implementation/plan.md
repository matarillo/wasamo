## Task list

Phase 6 ships two surfaces (ZStack + conditional rendering) plus the
R1 Window-title host-wiring, so the task list is wider than a
single-primitive phase. ZStack (T1–T3) and conditional rendering
(T4–T5) are decomposed by crate layer following the Phase 2–5 pattern;
R1 (T6) is its own review concern (DD-M3-P6-006); the gallery slice,
owner smoke, and phase-close gates (T7–T9) follow the Phase 5 T5–T7
shape with the FD-I retrospective split represented from the start.

Default to **one commit per task-list item** per
[CLAUDE.md §Commit rules](../../../../CLAUDE.md#commit-rules). The
exception this phase is the **`IrMember` schema change** (R-A): it
breaks `wasamoc` and the runtime loader simultaneously, so it bundles
with emit + loader + validators in one buildable commit (recorded in
T4). If implementation reveals an item should split or reorder, revise
this list so it stays an accurate record rather than a frozen
prediction.

### T0 — Moment 1 document sync

Opens execution after ADR acceptance and records the design draft in
the upstream documents named by the ADR's
[Moment 1 commit set](../decisions/preamble.md#upstream-document-revisions-moment-1--moment-2).
T0 closes when this implementation plan lands and all preceding
Moment 1 commits are on the pre-doc branch. Implementation (T1) begins
only after T0 closes.

- [x] `process/milestone-3/phase-6/decisions/preamble.md` and
      `dd-m3-p6-001` through `dd-m3-p6-006` flipped to
      `Status: Accepted`; preamble §Decisions restructured to record
      the accepted decisions only (commit `1af778c`, 2026-06-02).
- [x] `docs/dsl_spec.md` — ZStack chapter + conditional-rendering /
      structural-rendering-model chapter added as design-spec drafts
      (the `§4` title generalized; the `if` construct defined as a
      structural control-flow construct, not a `§4.4` registry widget,
      with a pointer from `§4.4`; absent=fresh-on-return /
      opt-in-retention normative semantics; Grammar `§3` `if`-block
      addition; keyword reservation for `if` / `else` / `switch` /
      `for`; control-flow-member production in the textual IR chapter).
      Phase status markers `M3-Phase 6 design accepted; implementation
      pending`; living-spec vocabulary cleanup folded (commit
      `1756963`, v1.5).
- [x] `docs/architecture.md` — **touch (judged required), with IA
      cleanup.** The M3 layout-primitive runtime-shape material split
      out of `6.8.7 Binding registration API after M2` into the new
      `§6.9 M3 layout primitives and runtime shape` sibling; ZStack
      added there (union sizing + `Fill/Fill` default + parent-owned
      child alignment + document-order z-order + outer-bounds clip + no
      intermediate Visual + no new `LayoutError`); the conditional
      construct added under the IR / runtime sections (member-level
      `IrMember` / `ControlFlowNode`; `BindingTarget::ConditionalSubtree`
      present/absent path; DD-M3-P6-005 effect lifecycle + drain
      ordering) and a declared-tree / entity-tree separation note under
      §9; top Status `design accepted; implementation pending`;
      living-spec vocabulary cleanup folded (commit `3a9aea0`;
      section renumber + link repair `603fc02`).
- [x] `process/milestone-3/plan.md` Phase 6 row populated (Status
      `in progress`; Progress-file link → this directory; ADR link →
      the ADR set; R1 owning-phase + ADR-Accepted note in the Notes
      column) — **this review batch (commit pending owner review).**
- [x] `process/milestone-3/phase-6/implementation/preamble.md` and
      this `plan.md` opened with `status: active` and the FD-C / FD-I /
      constraints §3 / §6 step-end / phase-end retrospective split
      represented in T9 from the start — **this review batch (commit
      pending owner review).**
- [x] `docs/abi_spec.md` deliberately untouched at Moment 1 per
      DD-M3-P6-006 (judged no-touch): the static title rides the
      existing `wasamo_load_ui` → `window::create` internal path with
      no new ABI export and no `PropertyValue` tag; the `If` construct
      adds no host-facing ABI surface; LayoutError stays internal. If
      implementation re-sync surfaces an unavoidable ABI need, it is
      recorded at Moment 2 with owner confirmation.

### T1 — ZStack: `wasamoc check` surface, IR emit, and diagnostics

Discharges the **ZStack portion** of ADR
[verification closure evidence item (1)](../decisions/preamble.md#phase-6-verification-closure-what-counts-as-a4--a7-evidence)
(ZStack surface-lowering positive controls + attribute rejection) and
the **`wasamoc` emit half** of item (3) (ZStack roundtrip). The
gallery-slice positive control half of item (1) closes at T7; runtime
roundtrip / loader rejection is T3.

- [x] Register `ZStack` in `wasamoc`'s `KNOWN_WIDGET_TYPES` and check
      surface as a per-kind tag with **direct children** (no `Cell`-style
      wrapper, no `KindPayload`, no new `IrType` / `IrLiteral`) per
      DD-M3-P6-001. Implemented in `wasamoc/src/check.rs`
      (`KNOWN_WIDGET_TYPES`), with
      `zstack_known_widget_no_warning` and lower/emit tests pinning the
      direct-child/no-payload shape.
- [x] Implement ZStack check-side diagnostics per DD-M3-P6-001 /
      DD-M3-P6-002: admit `h-align` / `v-align` as child placement props
      consumed by the ZStack context (and rejected elsewhere, mirroring
      the Grid `Cell` placement-prop rule); **reject** attributes outside
      the documented ZStack surface (`z-index`, `spacing`, `columns`, …).
      Implemented in `wasamoc/src/check.rs`
      (`check_zstack_unknown_attr`, `check_zstack_child_align`,
      `check_child_placement_outside_parent`, and parent-context
      traversal in `check_members_inner`); tests cover valid direct-child
      placement, bad alignment, misplaced placement, and ZStack-level
      disallowed attributes.
- [x] `wasamoc` emits the ZStack IR node (per-kind tag, direct children,
      document order preserved) to textual IR; `IrProp.value` stays
      strictly `IrLiteral`. Implemented via the existing generic
      `lower_node` / `emit_node` path, with
      `zstack_lowers_as_direct_children_without_kind_payload` and
      `zstack_emitted_as_node_with_direct_children_in_order`.
- [x] Add `wasamoc` positive / negative tests covering the ZStack half
      of ADR evidence item (1) (surface-lowering positive controls +
      disallowed-attribute / mis-placed-placement-prop rejection). Added
      check tests `zstack_known_widget_no_warning`,
      `zstack_direct_child_alignment_accepted`,
      `zstack_unknown_attribute_rejected`,
      `zstack_reserved_layering_attribute_rejected`,
      `zstack_grid_track_attribute_rejected`,
      `zstack_child_bad_alignment_value_rejected`, and
      `placement_attr_outside_zstack_child_or_cell_rejected`, plus the
      lower/emit roundtrip-shape tests named above.

### T2 — ZStack: layout engine measure / arrange / z-order / clip

Discharges the **ZStack portion** of ADR
[verification closure evidence item (2)](../decisions/preamble.md#phase-6-verification-closure-what-counts-as-a4--a7-evidence).
Per [CLAUDE.md §Testing rules](../../../../CLAUDE.md#testing-rules) the
layout engine is pure logic; tests are pure-logic unit tests on the
algorithm's `(input → output)` shape.

- [x] Add `WidgetKind::ZStack` and implement `measure_zstack` /
      `arrange_zstack` in `wasamo-runtime/src/layout.rs` per
      DD-M3-P6-002 (`Fill/Fill` default, union per-axis-max sizing,
      per-child alignment with `center` default + `h-align` / `v-align`
      overrides, no new `LayoutError`). Implemented in
      `wasamo-runtime/src/layout.rs` with `WidgetKind::ZStack`,
      `ZStackPlacement`, `LayoutNode::zstack`, `measure_zstack`, and
      `arrange_zstack`.
- [x] Add pure-logic tests covering the ZStack half of ADR evidence
      item (2). Scope note: this covers the **layout-side** document-order
      substrate only; the **visible paint-precedence** half of z-order
      (later-child-on-top under overlap = real Visual insertion order) is
      T3's, not asserted in pure logic. Added
      `zstack_defaults_to_fill_fill_and_centers_children`,
      `zstack_shrink_measure_uses_child_union_with_fill_child_zero`,
      `zstack_arrange_alignment_overrides`, and
      `zstack_arrange_preserves_document_order_substrate`.

### T3 — ZStack: runtime loader + Windows-runtime Visual evidence

Discharges the **ZStack portion** of ADR
[verification closure evidence items (3) and (4)](../decisions/preamble.md#phase-6-verification-closure-what-counts-as-a4--a7-evidence).
Mock-free Windows-runtime integration tests per
[CLAUDE.md §Testing rules](../../../../CLAUDE.md#testing-rules);
skip-guard inherits the Phase 2 T11 / Phase 3 / 4 / 5 pattern (fires on
`0x80070005` from `wasamo_init`) and **fails** rather than silently
skips on a runner that cannot create the Compositor.

- [x] Materialise `ZStack` as a runtime widget kind: loader parses the
      textual ZStack node, builds the `WidgetNode`, installs the
      **outer-bounds `InsetClip`** on the ZStack's own Visual, and wires
      the `WidgetData` → `LayoutNode` build boundary (per-child alignment
      vector parallel to `children`). Per-child clip stays absent
      (DD-M3-P6-002). No `docs/abi_spec.md` change. Implemented in
      `wasamo-runtime/src/widget.rs` (`WidgetData::ZStack`,
      `WidgetNode::zstack`, `build_layout_tree`) and
      `wasamo-runtime/src/ir_loader.rs` (`construct_widget`
      `"ZStack"` arm + `extract_zstack_placement`).
- [x] Runtime `validate()` defense-in-depth for ZStack malformed shapes
      surfaces `WASAMO_ERR_IR_MALFORMED` (dual gate with T1 `wasamoc
      check`); ZStack emit → load roundtrip preserves child count and
      document order (evidence item 3 ZStack half). Add pure-logic
      loader tests. Implemented in `validate_phase6_zstack_node_invariants`
      with tests `zstack_positive_control_validates_direct_children`,
      `zstack_attribute_rejected_at_validate`,
      `zstack_binding_rejected_at_validate`,
      `zstack_child_unknown_alignment_rejected_at_validate`,
      `placement_prop_outside_zstack_child_or_grid_cell_rejected_at_validate`,
      `validate_rejects_zstack_with_kind_payload`, and
      `zstack_zero_children_validates`; roundtrip test
      `zstack_emit_then_parse_preserves_direct_children_and_order`.
- [x] **ZStack real-Visual z-order fixture** — a `.ui` with overlapping
      ZStack children asserts the child Visual order matches document
      order under the live Visual tree (z-order is **not** dischargeable
      by pure logic alone). Include both a ZStack-rooted fixture and a
      `VStack { ZStack { … } }` production-root fixture (Phase 4 T6
      carry-forward). **Scope note:** this confirms Visual *child order*;
      the **real-pixel paint precedence** (overlapping content actually
      occluding what is behind it) is observed by the visible smoke (T7
      assistant + T8 owner) on the by-construction-overlapping lightbox —
      it is corroborated by, not substituted by, this fixture. This is
      the gap the Phase 5 Grid slice left as an acceptance judgment (its
      only overflow was clipped away downward, so no two cells
      overlapped); the lightbox closes it because the scrim sits under the
      photo / caption / nav and over the thumbnails by construction.
      Implemented in `wasamo-runtime/tests/zstack_layout_integration.rs`
      with `zstack_rooted_fixture_preserves_live_visual_order_and_clip`
      and `zstack_vstack_root_fixture_pins_production_root_shape`; the
      fixture enables `windows` `Foundation_Collections` so
      `VisualCollection` can be enumerated; review follow-up also reads
      the aligned child `Visual.Offset` so `h-align: end` /
      `v-align: start` is proven through the live runtime boundary.
- [x] **ZStack outer-bounds clip fixture** — the ZStack Visual has a
      non-null `Visual.Clip` (InsetClip); each child Visual has
      `Visual.Clip = null` (clip-absence regression guard, symmetric with
      the Grid / ScrollView / WrapPanel precedents) (DD-M3-P6-002,
      evidence item 4 ZStack half). Covered by
      `assert_zstack_visual_contract` in
      `wasamo-runtime/tests/zstack_layout_integration.rs`.
- [x] Confirm the skip-guard fires (test **fails**, not skips) on an
      environment where `wasamo_init` returns `0x80070005` before
      landing T3, or record the inheritance disposition (no new runtime
      capability path → `init_runtime_or_skip` reused byte-identically)
      in [log.md](./log.md) per the Phase 4 / 5 pattern. Recorded in
      `implementation/log.md`: T3 reuses the Phase 5
      `init_runtime_or_skip` disposition; no new runtime capability path
      was introduced.

### T4 — Conditional: IR schema, grammar, and static loader

Discharges the **host-independent** conditional evidence: item (1)
(grammar positive controls + condition / body / placement rejection),
the presence-reducer half of item (2), and item (3) (control-flow
emit → load roundtrip + loader rejection). This task lands the
**`IrMember` schema change** (R-A) — the consequential owner-decision
fork — so it spans `wasamo-ir` + `wasamoc` + the runtime loader; the
schema commit bundles those sites to stay buildable. The **reactive**
toggle (binding, mutation, Visual ordering) is **T5**, not here: T4's
loader builds a conditional's *initial* presence from the condition's
load-time value and does not yet register the toggle binding.

- [x] **Pre-implementation spike** for [R-A](./preamble.md#technical-risks-planning-time-recon)
      / [R-B](./preamble.md#technical-risks-planning-time-recon): settle the
      `IrMember = Widget(IrNode) | ControlFlow(ControlFlowNode)` shape
      (DD-M3-P6-004 O1, branch-list-ready), the `ControlFlowNode::If`
      single-`Widget`-child body, and the construction-site migration
      discipline (helper constructor vs broad edit) across `wasamo-ir`,
      `wasamoc` emit / lower, the loader, and the test corpus. Fix the
      commit-bundling boundary so the workspace builds at the
      schema-change commit. Record in [log.md](./log.md) before opening
      the bullets below. Settled as direct DD-M3-P6-004 O1 with
      `IrNode::widget_children()` as the widget-only traversal helper and
      explicit `IrMember` dispatch in lowering / emit / parse / validate /
      static append; recorded in `implementation/log.md` (2026-06-03
      T4 IrMember schema migration).
- [x] `wasamo-ir`: `IrNode.children` → `Vec<IrMember>`, add
      `ControlFlowNode` (DD-M3-P6-004); migrate construction sites; IR-type
      unit tests cover the member encoding. Implemented in
      `wasamo-ir/src/lib.rs` with `IrMember`, `ControlFlowNode`,
      `ControlFlowBranch`, `IrNode::widget_children()`, and
      `ir_member_encodes_widget_and_control_flow`.
- [x] `wasamoc` lexer: reserve the **whole control-flow family** —
      `if` / `else` / `switch` / `for` — as keywords now, not just `if`
      (DD-M3-P6-003 reserves the family this phase; the current `Keyword`
      enum + `scan_ident` table stop at `true` / `false` and have none of
      them, [lexer.rs:5](../../../../wasamoc/src/lexer.rs#L5) /
      [:361](../../../../wasamoc/src/lexer.rs#L361)). Add the four to the
      `Keyword` enum + `scan_ident` table and reject them at identifier
      positions. Phase 6 *implements* only `if`; `else` / `switch` / `for`
      surface a **reserved / not-yet-supported** `wasamoc check`
      diagnostic (parse + check tests), so the family is locked at the
      lexer without opening the grammar (mirrors the dsl_spec §2.1
      keyword-reservation update landed at Moment 1). Implemented in
      `wasamoc/src/lexer.rs` (`Keyword::{If,Else,Switch,For}` +
      `scan_ident`) with tests `control_flow_family_keywords_reserved`
      and `reserved_control_flow_keywords_without_production_rejected`.
- [x] `wasamoc` parser: the `if <bool-expr> { <widget-child> }` block
      (DD-M3-P6-003). Implemented in `wasamoc/src/ast.rs` /
      `wasamoc/src/parser.rs` as `Member::Conditional`,
      `parse_conditional_member`, and `parse_condition_expr`; covered by
      `conditional_member_parses_inside_widget_body`.
- [x] `wasamoc check` diagnostics (DD-M3-P6-003) — **reject** non-bool /
      undeclared-name / operator condition, a non-structural body member,
      a nested `if` directly in the body, a multi-child body, and a
      **mis-placed `if`** including a **component-level `if`** (admitted
      only inside a widget body; required test). Implemented in
      `wasamoc/src/check.rs` (`check_if_condition`, `check_if_body`, and
      `Member::Conditional` traversal) with tests
      `conditional_bool_state_accepted`,
      `conditional_bool_literal_accepted`,
      `conditional_non_bool_condition_rejected`,
      `conditional_literal_condition_rejected`,
      `conditional_undeclared_condition_rejected`,
      `conditional_operator_condition_rejected`,
      `conditional_non_structural_body_rejected`,
      `conditional_direct_nested_if_body_rejected`,
      `conditional_multi_child_body_rejected`, and
      `conditional_component_level_rejected`; review follow-up added
      `conditional_direct_grid_child_rejected` to pin the Grid-placement
      diagnostic branch, and `conditional_cell_sibling_rejected` to keep
      the `Cell { <widget> if ... }` source diagnostic symmetric with the
      runtime Grid/Cell direct-ControlFlow rejection.
- [x] `wasamoc` lower → `ControlFlowNode::If` + textual-IR emit (the
      §Spec content seed shape, DD-M3-P6-004); the member materialises no
      runtime widget. Implemented in `wasamoc/src/lower.rs`
      (`lower_condition_expr`, `lower_widget_body_member`) and
      `wasamoc/src/emit.rs` (`emit_member`); covered by
      `conditional_lowers_to_control_flow_member`,
      `conditional_bool_literal_lowers_to_bool_lit_condition`, and
      `conditional_emitted_as_control_flow_member`.
- [x] **Loader (static):** `build_node` iterates `Vec<IrMember>` and
      dispatches `Widget(_)` vs `ControlFlow(_)` (R-B); a `ControlFlow`
      builds its body present/absent from the **load-time** condition
      value (no toggle binding yet). `validate()` dual-gates (with
      `wasamoc check`) a non-bool / unresolved condition, >1 branch (until
      `else`), or an empty / multi-child / non-structural / nested-control-
      flow body → `WASAMO_ERR_IR_MALFORMED`. Control-flow roundtrip
      preserves condition + single-child body. Implemented in
      `wasamo-runtime/src/ir_loader.rs` (`parse_if_member`,
      `validate_phase6_control_flow_invariants`,
      `validate_condition_expr`, `append_static_member`,
      `evaluate_static_condition`, and
      `collect_static_zstack_placements`) with tests
      `control_flow_if_parses_as_member_with_single_widget_body`,
      `control_flow_roundtrip_preserves_condition_and_body`,
      `static_condition_reducer_maps_bool_to_presence`,
      `zstack_static_placements_follow_materialized_member_order`,
      `validate_rejects_if_with_non_bool_condition`,
      `validate_rejects_if_with_bool_read_resolving_to_non_bool_state`,
      `validate_rejects_if_with_unresolved_condition`,
      `validate_rejects_if_with_empty_body`,
      `validate_rejects_if_with_multi_child_body`,
      `validate_rejects_if_with_nested_control_flow_body`,
      `validate_rejects_invalid_subtree_inside_if_body`,
      `validate_rejects_direct_conditional_grid_member`, and
      `validate_rejects_direct_conditional_cell_member`.
- [x] Tests: `wasamoc` positive controls (`if <bool-state> { … }` /
      `if true { … }`) + a reject case per diagnostic (item 1); the
      pure-function presence reducer `bool → present/absent` (item 2);
      loader roundtrip + rejection (item 3). Added the test sets named
      in the bullets above; scoped verification green:
      `cargo test -p wasamo-ir`, `cargo test -p wasamoc --lib`, and
      `cargo test -p wasamo-runtime --lib`.
- [x] **Review follow-up** (`fix/m3-phase-6-t4-review-followup`):
      semantic-migration audit of the `Vec<IrMember>` traversal contracts
      (recorded in [log.md](./log.md)). Closed two under-count defects the
      migration left: the **Box** at-most-one (`Box { Content  if c { … } }`
      → reject) and **ScrollView** exactly-one (`ScrollView { Content
      if c { … } }` → reject) single-child gates counted widget children
      only and so missed a conditional sibling. Fixed at `wasamoc check` +
      runtime `validate()` with tests `box_widget_and_conditional_sibling_rejected`,
      `box_conditional_only_child_accepted`,
      `box_multiple_conditional_siblings_rejected`,
      `scrollview_conditional_member_rejected`,
      `scrollview_conditional_only_member_rejected`,
      `validate_rejects_box_with_widget_and_conditional_sibling`,
      `validate_accepts_box_with_conditional_only_child`,
      `validate_rejects_box_with_multiple_conditional_siblings`,
      `validate_rejects_scrollview_with_conditional_member`, and
      `validate_rejects_scrollview_with_conditional_only_member`. The
      conditional-only ScrollView case (`ScrollView { if c { … } }`) stays
      rejected as a **conservative interim** pinned by the
      `*_conditional_only_member_rejected` tests, pending **T4b /
      DD-M3-P6-007** (the `if c`-alone case is exactly the value a (b)
      relaxation would flip — Codex review-flagged provenance).

### T4b — ScrollView conditional-content policy (DD-M3-P6-007)

Owns the ScrollView × conditional cardinality decision surfaced by the T4
review semantic-migration audit — a Phase 6 responsibility (defining the
new `if` construct's interaction with each container's cardinality
invariant). Inserted with a non-integer label (no renumber): the task is
**conditional** — its weight depends on the deliberation outcome. A
non-integer label is used rather than a renumber so that an (a) outcome
(near-no-op) leaves no churn in the T5–T9 references; a (b) outcome may
promote it to a full numbered task at that point. Deliberation should land
**before T5 closes** (a (b) outcome's reactive-empty evidence folds into
T5).

- [x] Deliberate [DD-M3-P6-007](../decisions/dd-m3-p6-007-scrollview-conditional-content-policy.md)
      ((a) reject conditional-only content vs (b) allow conditionally-empty)
      and flip it to `Accepted` with owner comparison. **Outcome: (a)**
      (owner 2026-06-04, after a multi-pass design-decision review).
- [x] **(a) selected**: T4-follow-up interim confirmed as the final rule —
      the `ScrollView { if c { … } }` rejection carries an intent-revealing
      diagnostic. No prior-DD touch; preamble §Decisions + Revisions updated;
      `docs/dsl_spec.md` §4.11 (any direct conditional member rejected — wrap
      inside the content widget) + §4.14 diagnostics-list entry synced. No
      code/test change (the interim already ships the dual-gate evidence —
      see the DD §Implementation handoff if (a)).
- [ ] ~~If **(b)**~~ — **not selected**; conditionally-empty ScrollView is
      deferred (DD-007 §Deferred design space). The work below was the (b)
      branch and is not taken this phase: relax the ScrollView gate to
      at-most-one-materialised;
      reopen DD-M3-P4-001's exact-one invariant (loader gate DD-M3-P4-006)
      + dependent content-size/clamp in DD-M3-P4-003/005 + dsl_spec /
      architecture (not DD-M3-P4-003's offset-y / binding-direction surface);
      add reactive toggle-to-empty Windows-runtime evidence (coordinate with
      T5). Promote this task to a numbered slot if it grows to full
      implementation.

### T5 — Conditional: reactive toggle and Windows-runtime evidence

Discharges the **conditional portion** of ADR
[verification closure evidence item (4)](../decisions/preamble.md#phase-6-verification-closure-what-counts-as-a4--a7-evidence)
and the **DD-M3-P6-005 drain proof contract** (M2 handoff item 4).
Mock-free Windows-runtime integration tests; skip-guard per T3. This
is the novel-runtime task — it adds the `BindingTarget` variant, fixes
the Visual-ordering primitive (R-F), and wires structural disposal
(R-E).

- [ ] **`insert_child` positional Visual insert** (risk
      [R-F](./preamble.md#technical-risks-planning-time-recon),
      pre-implementation spike): `insert_child` / `append_child` currently
      `InsertAtTop` unconditionally, so a subtree re-inserted between
      static siblings lands on top rather than in declared sibling order.
      Give `insert_child` an `InsertAbove` / `InsertBelow`-relative
      positional Visual insert keyed to the recomputed index. The T5
      declared-sibling-order fixture is its regression gate.
- [ ] **`BindingTarget::ConditionalSubtree`** (risk
      [R-C](./preamble.md#technical-risks-planning-time-recon)): add the
      variant `{ parent, declared_member_index }`; convert the
      `register_binding` / `register_bool_binding` irrefutable `let`
      destructures to `match`; register the conditional binding via the
      `EffectHandle::new` seam (an insert/remove closure, not a property
      writer). The materialised insertion index is **recomputed from
      declared order + live presence** at each mutation, not cached.
- [ ] **Present/absent mutation + Effect disposal** (risk
      [R-E](./preamble.md#technical-risks-planning-time-recon)): toggle
      true ⇒ `insert_child` the freshly-built subtree at the recomputed
      index; false ⇒ `remove_child` then `widget_destroy` so the subtree's
      Effects **and** `WidgetId`-keyed registry entries are disposed
      (DD-M3-P6-005 (a)). Re-present recreates fresh widgets + Effects.
- [ ] **Toggle integration fixture** (item 4) — `bool` true → false → true
      inserts / removes the subtree + its Visuals; assert **declared
      sibling order** for siblings-on-both-sides and two-sibling-
      conditional (the latter including a **preceding-conditional removal
      while both present** so the removal-index shift is exercised); a
      **re-evaluation-to-same-state** case (true→true / false→false)
      asserts a **no-op** (no duplicate insertion, no spurious removal);
      Effects + registry entries disposed on absence.
- [ ] **Drain proof fixture** (item 4 / DD-M3-P6-005 (b)) — with
      `BATCH_DEPTH == 0`, a toggling write drains before control returns
      (toggle-then-observe): presence is observable and freshly-inserted
      Effects have run, within the existing `MUTATION_CAP`. Pins the
      M3-Phase 1 synchronous-drain contract under structural mutation.
      Record the reactive-drain items 1–3 fix-or-carry disposition
      (carried forward per DD-M3-P6-005 / constraints §7) in
      [log.md](./log.md).

### T6 — R1 Window-title host-wiring

Discharges ADR
[verification closure evidence item (4) R1 line](../decisions/preamble.md#phase-6-verification-closure-what-counts-as-a4--a7-evidence)
and the carry-forward residual **R1** (DD-M3-P6-006,
[../requirements/constraints.md §1](../requirements/constraints.md)).

- [ ] **Pre-implementation spike** for risk
      [R-D](./preamble.md#technical-risks-planning-time-recon): settle the
      static-title extraction point (component root props per Q2) and
      confirm `build_widget_tree` / the loader exposes the static literal
      separately from a dropped binding. Record in [log.md](./log.md).
- [ ] Route the **static** component-level `title:` literal to
      `window::create` in place of `DEFAULT_WINDOW_TITLE` in
      `wasamo_load_ui` ([`abi.rs:1220`](../../../../wasamo-runtime/src/abi.rs#L1220)),
      reading the `"title"` `IrProp` from `component.root.props`. Per
      DD-M3-P6-006: a **`Str` literal** is applied; **absent / empty**
      falls back to `DEFAULT_WINDOW_TITLE`; a **non-`Str` literal**
      (`title: 3`, `title: #fff`, …) is rejected as
      `WASAMO_ERR_IR_MALFORMED` at the loader. **No new ABI export**,
      `docs/abi_spec.md` untouched. The dynamic (`String`-binding) title
      (a `bind title = …` landing in `root.bindings`) stays
      evaluated-and-deferred (FD-D) — no implementation.
- [ ] **Loader-level title gate test** (DD-M3-P6-006) — a hand-written
      IR with a non-`Str` `title` prop surfaces `WASAMO_ERR_IR_MALFORMED`;
      absent / empty `title` falls back to the default. Host-independent
      defense-in-depth, distinct from the GUI fixture below.
- [ ] **R1 static title integration fixture** — a `.ui` whose component
      declares `title: "Gallery"` produces a native window whose title bar
      reads `"Gallery"`, not `"Wasamo"` (evidence item 4 R1 line).

### T7 — End-to-end gallery lightbox slice + assistant-side build / launch

Discharges the **assistant-automated portion** of ADR
[verification closure evidence items (5) and (6)](../decisions/preamble.md#phase-6-verification-closure-what-counts-as-a4--a7-evidence)
and the **gallery positive-control portion** of item (1) (the slice
`.ui` compiles cleanly through `wasamoc check`). The visible-correctness
portion of item (6) (owner-manual GUI smoke) is owned by T8 per FD-I.
T7's assistant-automated evidence is **build + launch + screenshot
capture + assistant analysis**; `Start-Process` survival is the
supporting "no early crash" signal only and the assistant analysis is a
pre-T8 baseline, not a substitute for the owner's visible-correctness
judgment.

- [ ] Grow `examples/gallery/gallery.ui` **additively** with the
      lightbox slice (FD-B): a thumbnail-gallery background (WrapPanel /
      ScrollView slice, Phase 3/4) with a `bool`-toggled
      (`is_lightbox_open`) ZStack overlay = scrim (`Box { fill:
      #RRGGBBAA }`, FD-G) + centered photo (Box aspect 4:3 + Text
      placeholder) + caption (VStack) + nav (`<` `>` `x` text Buttons).
      The toggle is driven by a **plain text Button click handler**
      (`Open lightbox` opens, `x` closes), so the proof traverses
      **event handler → `bool` state → conditional subtree** (FD-C).
      Existing gallery slices stay byte-identical. (Thumbnail-click-to-open
      is out of scope — Box hit-testing / image Button is M4.)
- [ ] Lightbox photo uses `Box { aspect: 4:3 }` + `Text` per the Phase 2
      DD-M3-P2-006 placeholder pattern. No Image widget (M4).
- [ ] Build and run `examples/gallery-rust/`. Record assistant-automated
      visual evidence as a **before/after toggle pair** (lightbox closed
      vs open) — launch + `Graphics.CopyFromScreen` screenshot
      (per-monitor-DPI-aware) + assistant analysis confirming: the
      overlay appears on open and is gone on close (positive control =
      state toggle, not a single frame); the photo / caption / nav are
      painted **over** the scrim and the scrim **dims** (does not replace)
      the thumbnails behind it (z-order read off the open frame); the
      window title bar reads `"Gallery"` (T6 corroboration). C / Zig
      gallery hosts remain out of Phase 6 scope. Screenshots land under
      [evidence/](./evidence/). **This is the real-pixel paint-precedence
      observation** the Phase 5 gallery slice could not provide — the
      lightbox overlaps by construction (scrim under photo/nav, over
      thumbnails), so the occlusion is genuinely exercised rather than
      left as a Visual-insertion-order assumption.

### T8 — Owner-manual GUI smoke and any visible-correctness fix

Discharges the **owner-visible smoke** for ADR
[verification closure evidence item (6)](../decisions/preamble.md#phase-6-verification-closure-what-counts-as-a4--a7-evidence)
and the A11 gallery-proof owner-acceptance half. This step exists so
visible smoke is verified — and fixed if it fails — **before** any
phase-close mechanical work (spec / plan status flips) lands in T9,
matching the Phase 4 T5 / T6 and Phase 5 T5 / T6 split rationale.

- [ ] Owner runs `examples/gallery-rust/` and observes, with the
      **positive control = `is_lightbox_open` toggled** (constraints §3):
      - **closed → open toggle:** the lightbox overlay appears on open and
        is gone on close (proves structural present/absent, not a hidden
        always-built subtree);
      - **z-order:** photo / caption / nav painted over the scrim; the
        half-transparent scrim **dims** the thumbnails behind it rather
        than replacing them (proves document-order = paint-order overlay,
        not a flat opaque panel);
      - **scrim fill:** the `#RRGGBBAA` scrim covers the full viewport
        (resize positive control — the scrim holds `Fill/Fill` as the
        window grows, not a fixed rect);
      - **Window title bar reads `"Gallery"`** (R1 / DD-M3-P6-006), not
        `"Wasamo"`.
      The DPI blur on a high-DPI box is a **known M4 residual**
      (constraints §5), noted during analysis, not a smoke pass/fail
      criterion.
- [ ] Owner explicitly accepts the smoke result, or records a fail
      observation note. **If smoke fails:** the implementation fix lands
      additively on the T8 branch (new commits); the smoke checklist is
      re-run to green before T8 closes. Fix scope stays inside the Phase 6
      ADR / `docs/dsl_spec.md` / `docs/architecture.md`; any fix requiring
      a normative spec change escalates to T9 Moment 2 (or a mid-ADR
      addendum if unsuitable for Moment 2).
- [ ] T8 step-end retrospective recorded at
      `process/milestone-3/phase-6/retrospectives/t8.md`
      (retrospectives.md checklist items 1–11).

### T9 — Phase-end gates and Moment 2 re-sync

Discharges the m3-plan phase-end criteria for Phase 6, the Moment 2
doc-set in the
[ADR preamble Moment 2 section](../decisions/preamble.md#upstream-document-revisions-moment-1--moment-2),
and the **A12 spec-closure gate** (ADR verification closure item 7).
Per [../requirements/constraints.md §6](../requirements/constraints.md)
(FD-I), the step-end retrospective is **owned by T9**; the phase-end
retrospective is **NOT owned by T9** and stays unchecked at T9 close —
see the two retrospective bullets at the end of this list.

Before closing, cross-check this T0-frozen task list against any
mid-phase owner decisions and revise the mutable plan where they diverge
(constraints §6 — revise, do not work around).

- [ ] `cargo fmt --all -- --check` green locally and on CI.
- [ ] `cargo build --release --workspace` green locally and on CI.
- [ ] `cargo test --workspace` green locally and on CI.
- [ ] Windows-only integration evidence green on CI (skip-guard verified
      per T3): ZStack z-order + clip (T3), conditional toggle insert/remove
      + drain proof (T5), R1 static title (T6).
- [ ] `docs/dsl_spec.md` ZStack + conditional chapters' Phase status
      markers flip to `M3-Phase 6 closed; implementation-synced`;
      document-level Status header updated; revision-history entry
      recording the Moment 2 close, any impl/spec divergence corrections,
      and any earlier-phase retroactive spec-gap fold surfaced during
      re-sync (owner-confirmation required per the retroactive spec-gap
      minimum-fold pattern). The textual-IR `§8` control-flow-member
      production is folded to match the landed `ControlFlowNode` shape if
      it pinned at implementation time.
- [ ] **A12 spec-closure gate (evidence item 7)** — confirm the ZStack
      chapter, the conditional-rendering (structural-rendering-model)
      chapter (`if` as the first member of a structural control-flow
      family; absent=fresh-on-return / opt-in-retention normative
      semantics), and the reader-facing invalid examples / diagnostics
      match the diagnostics exercised in T1 / T4 / T5, at the
      external-reader-reproducibility bar.
- [ ] `docs/architecture.md` top Status flips to include `M3-Phase 6
      complete`; any implementation-divergent paragraphs in §6.9 /
      §6.7 / §9 re-synced to the actual landed shape.
- [ ] `process/milestone-3/plan.md` Phase 6 row Status flips to
      `complete`.
- [ ] `docs/abi_spec.md` re-confirmed untouched (static title rode the
      existing `wasamo_load_ui` → `window::create` internal path; no new
      ABI export). Touch only if a Moment 2 surprise forced an ABI surface
      change, in which case escalate per the ADR preamble's three
      retrospectives.md §phase-sync ADR-touch cases.
- [ ] `process/milestone-3/phase-6/decisions/preamble.md` /
      `dd-m3-p6-*.md` touched **only** if one of the three
      retrospectives.md §phase-sync ADR-touch cases applies (AC
      discharged-vs-impl divergence; out-of-phase residual cross-reference;
      thesis-level finding). Otherwise the ADR set stays at its Moment 1
      Accepted state.
- [ ] [log.md](./log.md) records the phase-close evidence pointer, CI run
      id, implementation summary distilled from T1–T8, and any final
      post-merge distillation.
- [ ] Carry-forward inputs to Phase 7's pre-doc recorded under
      [handoff.md](./handoff.md) (at minimum: the `IrMember` /
      `ControlFlowNode` family-extension landing point for `else` /
      `switch` / iteration; the `BindingTarget::ConditionalSubtree` →
      `ForLoopSubtree` seam; the declared-tree / entity-tree identity /
      `key:` retention deferral; the dynamic Window-title deferral; the
      reactive-drain items 1–3 carry-forward; any residual surfaced during
      T2–T8). **NOT owned by T9** (phase-end retro item 15 per
      [retrospectives.md §6.3/§15](../../../procedures/retrospectives.md));
      stays `[ ]` at T9 close.
- [ ] Front-matter `status` (on the sibling
      [implementation/preamble.md](./preamble.md)) flips `active` →
      `closing` at the **phase-end batch commit** — the phase-branch
      commit that lands the CI-verified gates (fmt / build / test /
      Windows integration) + the spec / architecture / plan status flips
      + log.md + handoff — **not at T9 step-close**. Per the Phase 5
      actual-operation correction the on-CI gates are phase-end-owned
      (verified only after the phase branch runs `workflow_dispatch` CI),
      so **T9 step-close itself leaves `status: active`**; the
      [preamble Lifecycle](./preamble.md#lifecycle-transition) is the SSOT
      for this timing. **NOT owned by T9** — like the handoff and
      phase-end-retro bullets, this **stays `[ ]` at T9 close** and is
      checked by the phase-end batch commit on the phase branch. No
      further task checkboxes are added after the phase-end batch.
- [ ] **T9 step-end retrospective recorded** at
      `process/milestone-3/phase-6/retrospectives/t9.md`
      (retrospectives.md checklist items 1–11; step → phase merge gate;
      **owned by T9**, this is a T9 deliverable).
- [ ] **Phase-end retrospective recorded** at
      `process/milestone-3/phase-6/retrospectives/phase-end.md`
      (retrospectives.md checklist items 12–18; phase → main merge gate;
      **NOT owned by T9**, performed on the phase branch after T9 merges
      in by a separate retro commit per
      [../requirements/constraints.md §6](../requirements/constraints.md)).
      **This bullet stays `[ ]` at T9 close**; the phase-end retro commit
      on the phase branch flips it. Step retro `phase-sync` items from
      T1–T9 close into `doc-folded` / `carry-forward` / `local-only` at
      the phase-end retro — **no open `phase-sync` items survive past
      phase close.**
