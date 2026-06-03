## Decisions log

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
