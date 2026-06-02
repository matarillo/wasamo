## Decisions log

(empty — record here mid-phase decisions that deviate from the ADR,
refine task slicing, or close implementation-planning risks.)

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
