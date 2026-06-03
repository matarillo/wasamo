## Decisions log

- **2026-06-03 / T4 IrMember schema migration:** T4 landed the accepted
  DD-M3-P6-004 O1 shape directly: `IrNode.children` is now
  `Vec<IrMember>`, with `IrMember::Widget(IrNode)` and
  `IrMember::ControlFlow(ControlFlowNode::If { branches })`.
  Construction-site migration used a narrow helper discipline rather than
  a broad abstraction: production walkers use `IrNode::widget_children()`
  when an invariant is widget-child-only, and explicit
  `IrMember` dispatch where control flow is semantically relevant
  (`wasamoc` lower / emit; runtime parse / validate / static member
  append). The schema change, `wasamoc` emit/lower, textual IR parser,
  validators, and static load-time presence reducer were bundled in one
  buildable implementation commit per the T4 R-A/R-B risk note.
- **2026-06-03 / T4 review follow-up traversal audit:** owner review
  found two places where the initial `widget_children()` split silently
  changed semantics: phase-specific runtime validators did not descend
  into `ControlFlow` bodies, and ZStack placement vectors were built from
  widget-only declared children while materialised children came from
  static `IrMember` expansion. The follow-up fixes both by dispatching
  phase validators through `IrMember`, building ZStack placements through
  `collect_static_zstack_placements` with the same load-time condition
  evaluation as `append_static_member`, and rejecting direct runtime
  `ControlFlow` under `Grid` / `Cell` because those IR-only wrappers are
  flattened by a Grid-specific build path. This amends the T4
  retrospective's original "no new constraint" statement: declared-member
  traversal that affects validation or positional metadata must dispatch
  `IrMember`; widget-only helpers are valid only when dropping
  `ControlFlow` is explicitly part of the invariant.
- **2026-06-03 / T3 skip-guard disposition:** ZStack live Visual
  integration introduces no new runtime capability path beyond the
  existing `wasamo_init` → Compositor creation surface. The
  `init_runtime_or_skip` guard in
  `wasamo-runtime/tests/zstack_layout_integration.rs` therefore reuses
  the Phase 5 Grid pattern byte-for-byte in behavior: local
  `0x80070005` returns `None` (developer-laptop skip), while GitHub
  Actions fails rather than silently skipping. This records the
  inheritance disposition requested by T3 instead of re-proving the
  already inherited missing-Compositor path.
- **2026-06-03 / T3 VisualCollection evidence seam:** The ZStack live
  Visual-order fixture needs to enumerate `VisualCollection`; the
  runtime crate's existing `windows` dependency now enables the
  `Foundation_Collections` feature so the test can read the live child
  collection directly. This is an API-feature enablement for the
  existing dependency, not a new build system / CI surface.

---

## CI / verification log

- **2026-06-03 / T4 local scoped:** `cargo fmt --all -- --check`
  — green; `cargo test -p wasamo-ir` — green (17 tests);
  `cargo test -p wasamoc --lib` — green (308 tests);
  `cargo test -p wasamo-runtime --lib` — green (322 tests).
  Covered `IrMember` schema encoding, control-flow keyword / parser /
  check / lower / emit diagnostics, runtime textual IR parsing /
  roundtrip, and static conditional presence / validator rejection
  evidence.
- **2026-06-03 / T4 local pre-commit:** `cargo build --release
  --workspace` — green; `cargo build --workspace` — green;
  `cargo test --workspace` — green; `cargo test -p wasamo-runtime`
  — green (runtime lib 322 plus integration tests). Existing Cargo
  warnings about the `wasamo` linkable target were observed.
- **2026-06-03 / T4 task-end clean rebuild (post-commit
  `774b567`):** `cargo fmt --all -- --check` — green; `cargo clean`
  completed (`4862 files, 1.3GiB` removed); `cargo build --release
  --workspace` — green; `cargo build --workspace` — green;
  `cargo test --workspace` — green; `cargo test -p wasamo-runtime`
  — green (runtime lib 322 plus integration tests). Existing Cargo
  warnings about the `wasamo` linkable target / `wasamo-sys`
  import-library ordering were observed.
- **2026-06-03 / T4 review follow-up local:** `cargo test -p
  wasamo-runtime --lib ir_loader::tests` — green (122 tests);
  `cargo test -p wasamoc --lib check::tests::conditional` — green
  (11 tests); `cargo test -p wasamoc --lib` — green (310 tests);
  `cargo test -p wasamo-runtime --lib` — green (327 tests). Covered
  Grid direct conditional diagnostics, literal-condition diagnostics,
  runtime bool-read/non-bool condition rejection, ControlFlow-body
  validator descent, Grid/Cell direct-ControlFlow runtime rejection, and
  ZStack static placement order matching materialised member order.
  Final follow-up verification: `cargo fmt --all -- --check` — green;
  `cargo build --workspace` — green; `cargo test --workspace` —
  green; `cargo test -p wasamo-runtime` — green (runtime lib 327 plus
  integration tests). Existing Cargo warnings about the `wasamo`
  linkable target were observed.
- **2026-06-03 / T4 Cell conditional check follow-up local:**
  added the source-level dual gate for `Cell { <widget> if ... }`,
  matching the runtime `validate_rejects_direct_conditional_cell_member`
  defense-in-depth rejection. `cargo test -p wasamoc --lib
  check::tests::conditional_cell_sibling_rejected` — green; `cargo test
  -p wasamoc --lib check::tests::conditional` — green (12 tests);
  `cargo test -p wasamoc --lib` — green (311 tests); `cargo fmt --all
  -- --check` — green.
- **2026-06-02 / T1 local:** `cargo fmt --all -- --check` — green.
- **2026-06-02 / T1 local:** `cargo test -p wasamoc` — green;
  covered the ZStack check / lower / emit evidence with tests including
  `zstack_known_widget_no_warning`,
  `zstack_direct_child_alignment_accepted`,
  `zstack_unknown_attribute_rejected`,
  `zstack_reserved_layering_attribute_rejected`,
  `zstack_grid_track_attribute_rejected`,
  `zstack_child_bad_alignment_value_rejected`,
  `placement_attr_outside_zstack_child_or_cell_rejected`,
  `placement_attr_on_zstack_itself_rejected_with_container_position`,
  `zstack_lowers_as_direct_children_without_kind_payload`, and
  `zstack_emitted_as_node_with_direct_children_in_order`.
- **2026-06-02 / T1 local:** `cargo clippy -p wasamoc` — green.
- **2026-06-02 / T1 task-end clean rebuild:** `cargo clean`
  completed (`2993 files, 1012.3MiB` removed);
  `cargo build --release --workspace` green; `cargo build --workspace`
  green; `cargo test --workspace` green. Existing Cargo warnings about
  the `wasamo` linkable target / `wasamo-sys` import-library ordering
  were observed.
- **2026-06-02 / T2 local:** `cargo fmt --all -- --check` — green.
- **2026-06-02 / T2 local:** `cargo test -p wasamoc` — green.
- **2026-06-02 / T2 local:** `cargo test -p wasamo-runtime zstack` —
  green; added pure-logic ZStack layout tests
  `zstack_defaults_to_fill_fill_and_centers_children`,
  `zstack_shrink_measure_uses_child_union_with_fill_child_zero`,
  `zstack_arrange_alignment_overrides`, and
  `zstack_arrange_preserves_document_order_substrate`.
- **2026-06-02 / T2 local:** `cargo build --release --workspace` —
  green. Existing Cargo warnings about the `wasamo` linkable target /
  `wasamo-sys` import-library ordering were observed.
- **2026-06-02 / T2 task-end clean rebuild:** `cargo clean` completed
  (`4163 files, 1.1GiB` removed); `cargo build --release --workspace`
  green; `cargo build --workspace` green; `cargo test --workspace`
  green. Existing Cargo warnings about the `wasamo` linkable target /
  `wasamo-sys` import-library ordering were observed.
- **2026-06-02 / T2 review follow-up local:** tightened the
  `zstack_arrange_preserves_document_order_substrate` evidence so the two
  children have distinguishable overlapping geometry, corrected the T2
  retrospective's limited helper-rename classification, and renamed
  `align_in_rect` parameters from cell-specific to rect-specific names.
  `cargo fmt --all -- --check` green; `cargo test -p wasamo-runtime
  zstack` green (4 passed); `cargo build` green. Clean follow-up
  verification: `cargo clean` completed (`3707 files, 1.1GiB` removed);
  `cargo build --release --workspace` green; `cargo build --workspace`
  green; `cargo test --workspace` green. Existing Cargo warnings about
  the `wasamo` linkable target / `wasamo-sys` import-library ordering
  were observed.
- **2026-06-03 / T3 local scoped:** `cargo fmt --all -- --check` —
  green after formatting; `cargo test -p wasamo-runtime zstack` —
  green. Covered runtime validate tests
  `zstack_positive_control_validates_direct_children`,
  `zstack_attribute_rejected_at_validate`,
  `zstack_binding_rejected_at_validate`,
  `zstack_child_unknown_alignment_rejected_at_validate`,
  `placement_prop_outside_zstack_child_or_grid_cell_rejected_at_validate`,
  and `validate_rejects_zstack_with_kind_payload`; roundtrip test
  `zstack_emit_then_parse_preserves_direct_children_and_order`; live
  Visual fixtures
  `zstack_rooted_fixture_preserves_live_visual_order_and_clip` and
  `zstack_vstack_root_fixture_pins_production_root_shape`.
- **2026-06-03 / T3 local pre-commit:** `cargo test -p wasamo-runtime`
  — green (included the new ZStack live Visual fixtures, plus existing
  Grid / ScrollView / WrapPanel integration coverage); `cargo build
  --release --workspace` — green; `cargo build --workspace` — green;
  `cargo test --workspace` — green. Existing Cargo warnings about the
  `wasamo` linkable target / `wasamo-sys` import-library ordering were
  observed.
- **2026-06-03 / T3 task-end clean rebuild (post-commit
  `63d6262`):** `cargo fmt --all -- --check` — green; `cargo clean`
  completed (`7195 files, 2.2GiB` removed); `cargo build --release
  --workspace` — green; `cargo build --workspace` — green; `cargo test
  --workspace` — green. Existing Cargo warnings about the `wasamo`
  linkable target and `wasamo-sys` import-library ordering were observed.
- **2026-06-03 / T3 review follow-up local:** pinned empty ZStack as a
  valid runtime shape (`zstack_zero_children_validates`) and strengthened
  the live Visual fixture so the aligned child's `Visual.Offset` proves
  `h-align: end` / `v-align: start` through the runtime
  `WidgetData::ZStack` → `LayoutNode::zstack` boundary. `cargo fmt
  --all -- --check` passed after formatting; `cargo test -p
  wasamo-runtime zstack` — green.
- **2026-06-03 / T3 review follow-up clean rebuild (post-commit
  `395da0f`):** `cargo fmt --all -- --check` — green; `cargo clean`
  completed (`3755 files, 1.2GiB` removed); `cargo build --release
  --workspace` — green; `cargo build --workspace` — green; `cargo test
  --workspace` — green. Existing Cargo warnings about the `wasamo`
  linkable target and `wasamo-sys` import-library ordering were observed.
- **2026-06-03 / T1+T2 cross-task review follow-up clean rebuild
  (post-commit `4616e48`):** a T1/T2 re-review on the test-breadth and
  cross-phase-constraint lenses pinned three deliberate diagnostic/size
  branches that had no test —
  `zstack_child_non_keyword_alignment_value_rejected` (T1 `wasamoc`
  `check_zstack_child_align` non-identifier arm),
  `zstack_handler_rejected_at_validate` (T3 runtime `validate` ZStack
  handler arm), and
  `zstack_fixed_size_measure_reports_declared_extent_not_child_union`
  (T2 `measure_zstack` `Fixed` size arm) — and corrected t1.md item 5 /
  10 and t2.md item 10 to record the placement/alignment constraint as a
  single `carry-forward` with three implementation sites. `cargo fmt
  --all -- --check` — green; `cargo clean` completed (`4935 files,
  1.3GiB` removed); `cargo build --release --workspace` — green
  (39.27s); `cargo build --workspace` — green (35.80s); `cargo test
  --workspace` — green (`wasamoc` 293, `wasamo-runtime` lib 314).
  Existing Cargo warnings about the `wasamo` linkable target and
  `wasamo-sys` import-library ordering were observed. This is the single
  SSOT record for the follow-up verification; the t1 / t2 / t3 retro
  item-3 sections stay scoped to their own original commits.
