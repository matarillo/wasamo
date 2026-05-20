---
phase: M3-Phase 2
title: Box layout primitive
status: active
adr: docs/decisions/m3-phase-2-box-layout.md
plan: docs/plans/m3-plan.md
opened: 2026-05-20
---

# M3-Phase 2 — Box layout primitive: Progress

This is the live task list and execution log for M3-Phase 2. The
design decisions are frozen in
[m3-phase-2-box-layout.md](../../decisions/m3-phase-2-box-layout.md);
this file is mutable per
[plans/README.md §Phase progress file lifecycle](../README.md#phase-progress-file-lifecycle).

Task ordering follows the dependency direction
`wasamo-ir → wasamoc → wasamo-runtime → tests → host/spec`, so each
commit builds on a green workspace per
[CLAUDE.md §Commit rules](../../../CLAUDE.md). Items may be split,
reordered, or merged when implementation reveals a tighter ordering
— this list is the record of what actually happens, not a frozen
prediction.

The four pieces of A6 evidence the phase closes against are
enumerated in
[m3-phase-2-box-layout.md §Phase 2 verification closure](../../decisions/m3-phase-2-box-layout.md#phase-2-verification-closure-what-counts-as-a6-evidence).
Each T below cites the evidence item it discharges.

## Task list

### T1 — `wasamo-ir`: add `Ratio` and `Color` literal variants

Discharges the IR-layer halves of DD-M3-P2-002 and DD-M3-P2-003.

- [x] `IrLiteral::Ratio { num: i32, den: i32 }` variant added;
      every existing `match` on `IrLiteral` gains a `Ratio` arm.
- [x] `IrLiteral::Color(u32)` variant added; arm coverage as above.
- [x] **No** `IrType::Ratio` / `IrType::Color` added; **no** new
      `HandlerExpr` variant (per DD-M3-P2-004).
- [x] Pure-logic unit tests covering construction and equality of
      both variants.

Closed by commit `3708bb2 feat(ir): add Ratio / Color literals to
IrLiteral (M3-Phase 2 T1)`. Step-end retrospective recorded in
[../../notes/m3-phase-2/t1-step-end-retrospective.md](../../notes/m3-phase-2/t1-step-end-retrospective.md).

### T2 — `wasamoc` lexer / parser: `RatioLit` and `ColorLit` tokens, AST variants

Discharges the surface-syntax halves of DD-M3-P2-002 and DD-M3-P2-003.

- [x] Lexer recognises `<num>:<den>` as a `RatioLit` token (surface
      form per DD-M3-P2-002 Option A), with lookahead / contextual
      disambiguation against a stray integer-then-colon appropriate
      to the existing lexer structure.
- [x] Lexer recognises `#` followed by 6 or 8 hex digits as a
      `ColorLit` token (per DD-M3-P2-003 Option A).
- [x] `Expr::RatioLit { num: i32, den: i32 }` and
      `Expr::ColorLit { value: u32 }` AST variants added in
      `wasamoc/src/ast.rs`.
- [x] Parser produces these AST nodes in `property_bind` RHS
      position.
- [x] Unit tests covering the accept shapes from
      [dsl_spec §4.9](../../dsl_spec.md#49-box-layout-primitive-m3-phase-2).

Closed by commit `735a337 feat(wasamoc): lex Ratio / Color literals,
add Expr variants (M3-Phase 2 T2)`. Step-end retrospective recorded
in [../../notes/m3-phase-2/t2-step-end-retrospective.md](../../notes/m3-phase-2/t2-step-end-retrospective.md).

### T3 — `wasamoc check`: validity and reject set

Discharges DD-M3-P2-001 (multi-child reject), DD-M3-P2-004 (bind reject
for `aspect` / `fill`), and the value-validity portion of DD-M3-P2-005.

- [x] Reject zero on either side of ratio (per DD-M3-P2-005 aspect
      value validity); diagnostic names the rejected side.
- [x] Reject `bind aspect:` and `bind fill:` (per DD-M3-P2-004);
      diagnostic names the rejected attribute.
- [x] Reject 2+ children on Box (per DD-M3-P2-001 multi-child);
      diagnostic recommends `VStack` / `HStack` / `ZStack` (Phase 6
      forward-pointer).
- [x] Widget property catalog extended for Box (`aspect: Ratio`,
      `fill: Color` — Box-internal types, not new `IrType` entries)
      so the checker can name the attribute types in diagnostics.
      Box-internal value types are deliberately **not** entered in
      `widget_prop_type`'s `TypeName` table (they have no `TypeName`
      enum entry per DD-M3-P2-002 / DD-M3-P2-003 Option A); the
      catalog row is named in a code comment in that function and
      validity is enforced by a dedicated `check_box_const_only_bind`
      helper.
- [x] Unit tests cover each row of the reject set + each accept
      shape from the ADR.

Closed by commit `f70424d feat(wasamoc): Box widget validity and
reject set (M3-Phase 2 T3)`. Step-end retrospective recorded in
[../../notes/m3-phase-2/t3-step-end-retrospective.md](../../notes/m3-phase-2/t3-step-end-retrospective.md).

### T4 — `wasamoc` lowering: AST → IR

- [x] `Expr::RatioLit` → `IrLiteral::Ratio { num, den }`.
- [x] `Expr::ColorLit` → `IrLiteral::Color(u32)`; packed `u32`
      layout per
      [dsl_spec §8.2](../../dsl_spec.md#82-notation) `COLOR` token.
- [x] Unit tests assert lowering of representative `Box { ... }`
      forms.

Closed by commit `5be7df6 feat(wasamoc): lower Ratio / Color literals
to IR (M3-Phase 2 T4)`. Step-end retrospective recorded in
[../../notes/m3-phase-2/t4-step-end-retrospective.md](../../notes/m3-phase-2/t4-step-end-retrospective.md).

### T5 — `wasamoc` IR text emit

Discharges the IR-text-spelling halves of DD-M3-P2-002 and DD-M3-P2-003.

- [x] Emitter writes ratio and color literals in the surface forms
      (`<num>:<den>`, `#RRGGBB` / `#RRGGBBAA`) in `prop` literal
      position. Canonical emit policy: alpha = `0xFF` normalises to
      short `#RRGGBB`; otherwise the full `#RRGGBBAA` form is
      written. Policy documented on `emit_color_lit` and pinned by
      `color_emit_normalises_alpha_ff_input_to_short_form`.
- [x] IR text emit covers the Box widget node shape:
      `node Box { prop aspect = 16:9; prop fill = #cccccc; node Text { ... } }`.
- [x] In-process roundtrip-shaped test in `wasamoc::emit` (ADR
      §Phase 2 verification closure item 2). The
      `Box { aspect: 16:9 fill: #00000080 Text { text: "Photo 12" } }`
      fixture is asserted at both the `IrLiteral::Ratio` /
      `IrLiteral::Color` variant level and the emitted IR text level
      by `box_phase2_ir_text_emit_fixture`; the load-side half is
      T7 / T10.

Closed by commit `935b5d0 feat(wasamoc): IR text emit for Ratio /
Color literals (M3-Phase 2 T5)`. Step-end retrospective recorded in
[../../notes/m3-phase-2/t5-step-end-retrospective.md](../../notes/m3-phase-2/t5-step-end-retrospective.md).

### T6 — `wasamo-runtime` widget catalog: Box

Discharges DD-M3-P2-001 (IR node shape / per-kind tag).

- [x] `WidgetKind::Box` variant added; every existing `match` on
      `WidgetKind` gains a `Box` arm.
- [x] `WidgetData::Box { aspect: Option<Ratio>, fill: Option<Color>,
      child: Option<Box<WidgetNode>> }` (or layout-equivalent shape).
      Layout-equivalent shape taken: child lives on
      `WidgetNode.children: Vec<Box<WidgetNode>>` per the existing
      per-widget convention; single-child invariant is enforced at
      `wasamoc check` (T3) and `ir_loader::build_node` (T7) gates
      rather than this data shape.
- [x] `WidgetNode::box_` (or equivalent constructor) with default
      `aspect: None, fill: None, child: None`.
- [x] Internal `Ratio` and `Color` domain types declared inside
      `wasamo-runtime`; visibility minimal (not `pub` beyond what
      tests require). Declared `pub(crate)` in a private
      `wasamo-runtime/src/box_values.rs` module to avoid the
      `windows::UI::Color` name collision inside `widget.rs`.

Closed by commit `b4dff5d feat(wasamo-runtime): add Box widget
catalog (M3-Phase 2 T6)`. Step-end retrospective recorded in
[../../notes/m3-phase-2/t6-step-end-retrospective.md](../../notes/m3-phase-2/t6-step-end-retrospective.md).

### T7 — `wasamo-runtime` IR loader: parse new literal terminals and Box widget

- [x] IR text loader (`wasamo-runtime/src/ir_loader.rs`) accepts
      `RATIO` and `COLOR` terminals in `literal` position. New
      `Token::Ratio { num, den }` / `Token::Color(u32)` tokens with
      lex-time disambiguation mirroring `wasamoc::lexer`
      (`<digits>:<digits>` only triggers Ratio when the colon
      immediately follows the integer and a digit immediately
      follows the colon; `#` followed by 6 or 8 hex digits, packed
      `0xAARRGGBB` with implicit alpha `0xFF` for the short form
      per dsl_spec §8.2).
- [x] `ir_loader::build_node` materialises ratio / color literals
      for Box `aspect` / `fill` directly into Box-internal `Ratio`
      / `Color` (not via `PropertyValue`), per DD-M3-P2-002 /
      DD-M3-P2-003 variant strategy Option A.
      `WidgetNode::box_` constructor signature extended to take
      `Option<box_values::Ratio>` / `Option<box_values::Color>`
      (only call site is `ir_loader::construct_widget`'s "Box"
      arm; visibility narrowed to `pub(crate)` to match the
      `box_values` visibility surface).
- [x] `ir_loader::build_node` rejects ratio / color literals
      appearing outside Box `aspect` / Box `fill` with
      `WASAMO_ERR_IR_MALFORMED` (defense-in-depth for the
      "not via `PropertyValue`" boundary). Check lives in
      `validate()` (pure logic, exercised without a live
      `Compositor`); error class `IrLoadError::Validate` maps to
      `WASAMO_ERR_IR_MALFORMED` at the C ABI boundary
      (DD-M2-P6-005 / DD-M2-P6-009).
- [x] `ir_loader::build_node` rejects a Box IR node with
      `len(children) > 1` with `WASAMO_ERR_IR_MALFORMED` (defense-
      in-depth for DD-M3-P2-001 against IR not produced by
      `wasamoc`, e.g. via `wasamo_load_ui`). Same `validate()`
      pass as the literal-placement gate.
- [x] **No** new `PropertyValue` variant, **no** new
      `WASAMO_VALUE_*` tag, **no** new `abi.rs` arms (per
      DD-M3-P2-002 / DD-M3-P2-003 / DD-M3-P2-004).

Closed by commit `5169c99 feat(wasamo-runtime): IR loader for Ratio /
Color literals and Box widget (M3-Phase 2 T7)`. Step-end retrospective
recorded in
[../../notes/m3-phase-2/t7-step-end-retrospective.md](../../notes/m3-phase-2/t7-step-end-retrospective.md).

### T8 — `wasamo-runtime` layout: aspect measure-arrange

Discharges DD-M3-P2-005 and the child-layout portion of DD-M3-P2-001.

- [x] Bounded inscribed-fit branch selection per the DD-M3-P2-005
      numeric / rounding contract. Branch selection uses `f64`
      cross-multiplication (`W*den` vs `H*num`) to keep the choice
      independent of `f32` round-off; the derived axis is computed in
      `f32`. Equality lands on the width branch (`<=`), matching the
      `Box(16:9)` in `1600×900` happy path.
- [x] Unbounded-on-one-axis: bounded-axis-wins. Implemented in
      `measure_box` / `arrange_box` by inspecting `avail.is_finite()`
      on each axis.
- [x] Unbounded-on-both-axes (aspect set, or no-aspect Box with
      no bounded extent): layout-time runtime error. New
      `LayoutError::{BoxAspectUnboundedBoth, BoxNoExtent}` enum,
      returned by `measure` / `arrange` / `run_layout` (signature
      changed to `Result<…, LayoutError>`); `WidgetNode::run_layout`
      maps both variants to `windows::core::Error(E_FAIL)` so the
      existing `WM_SIZE → run_layout` call sites keep their
      `windows::core::Result<()>` shape. The deferred dedicated
      `WASAMO_ERR_*` code is captured under Out-of-phase residuals.
- [x] No-aspect bounded Box: matches parent bounds when empty;
      shrink-to-fit child intrinsic measure when child present.
      Empty Box with one axis unbounded collapses to `0.0` on that
      axis (the "scrim-only Box paints a zero-thickness strip"
      reading); only fully-unbounded empties trip `BoxNoExtent`.
- [x] Single child: measured against Box bounds, centred, clipped
      on overflow (per DD-M3-P2-001 child measure / alignment /
      overflow). `arrange_box` measures the child against the
      resolved Box rectangle, then clamps each axis via
      `cw.min(rw) / ch.min(rh)` (Fill children take the full Box
      extent on the Fill axis), and centres the result inside Box
      bounds.
- [x] Zero-child Box still produces a sized rectangle (filled with
      `fill`, or transparent when absent). `WidgetNode::box_` now
      paints `fill` as a `CompositionColorBrush` on the SpriteVisual
      at construction (`#RRGGBBAA` packed `0xAARRGGBB` unpacked into
      WinRT `Color { A, R, G, B }`); an absent `fill` leaves the
      visual brushless (transparent). Box's default size constraints
      flip to `Shrink/Shrink` so parent containers honour the
      measure-arrange output.

Closed by commit `5021936 feat(wasamo-runtime): Box aspect
measure-arrange (M3-Phase 2 T8)`. Step-end retrospective recorded in
[../../notes/m3-phase-2/t8-step-end-retrospective.md](../../notes/m3-phase-2/t8-step-end-retrospective.md).

### T9 — Pure-logic unit tests (ADR §Phase 2 verification closure item 1)

T9 is a coverage-inventory step: every checklist item below is
already discharged by tests landed during T1–T5 / T7 / T8. The
T9 commit updates this checklist to cross-link the existing tests
so the ADR §Phase 2 verification closure item 1 has an auditable
mapping. No new test files are added in this step. The
"explicit width/height conflict" sub-item from the ADR's item 1
enumeration is **out of Phase 2 scope** (per the ADR
DD-M3-P2-005 §"Phase 2 scope note": `width` / `height` are not in
the M3-Phase 2 DSL surface) and is therefore not exercised here.

- [x] Ratio literal: accept shapes; zero side rejected at check.
      - IR variant (T1): `wasamo-ir/src/lib.rs::tests`
        `ir_literal_ratio_round_trip_values`,
        `ir_literal_ratio_distinct_by_components`,
        `ir_literal_ratio_and_color_distinct_from_other_variants`.
      - Lex accept (T2): `wasamoc/src/lexer.rs::tests`
        `ratio_literal_basic`, `ratio_literal_one_to_one`,
        `ratio_literal_in_property_bind_position`,
        `ratio_zero_sides_lex_ok_check_rejects_later`.
      - Lex disambiguation (T2): `wasamoc/src/lexer.rs::tests`
        `integer_followed_by_colon_then_non_digit_is_not_ratio`,
        `integer_with_whitespace_before_colon_is_not_ratio`,
        `float_followed_by_colon_is_not_ratio`,
        `measurement_not_disturbed_by_ratio_lookahead`.
      - Parse accept (T2): `wasamoc/src/parser.rs::tests`
        `property_bind_ratio_literal`,
        `box_image_placeholder_shape`.
      - Check accept (T3): `wasamoc/src/check.rs::tests`
        `box_aspect_only_accepted`, `box_placeholder_shape_accepted`.
      - Check zero-side reject (T3): `wasamoc/src/check.rs::tests`
        `box_aspect_zero_numerator_rejected`,
        `box_aspect_zero_denominator_rejected`,
        `box_aspect_zero_both_sides_rejected`.
      - Positional reject (T3): `wasamoc/src/check.rs::tests`
        `ratio_literal_in_state_default_rejected`,
        `ratio_literal_in_handler_rejected`,
        `ratio_literal_on_non_box_widget_rejected`.
      - Lower (T4): `wasamoc/src/lower.rs::tests`
        `box_aspect_only_lowered_to_ir_ratio`,
        `box_aspect_and_fill_lowered_together`,
        `box_with_text_child_placeholder_shape_lowered`.
      - Emit (T5): `wasamoc/src/emit.rs::tests`
        `box_aspect_ratio_emitted_in_surface_form`,
        `box_phase2_placeholder_widget_node_shape_emitted`,
        `box_phase2_ir_text_emit_fixture`.
- [x] Color literal: `#RRGGBB` / `#RRGGBBAA` accept; malformed
      forms rejected at lex / parse.
      - IR variant (T1): `wasamo-ir/src/lib.rs::tests`
        `ir_literal_color_round_trip_value`,
        `ir_literal_color_distinct_by_packed_value`,
        `ir_literal_ratio_and_color_distinct_from_other_variants`.
      - Lex accept (T2): `wasamoc/src/lexer.rs::tests`
        `color_literal_six_digit_packs_with_full_alpha`,
        `color_literal_eight_digit_explicit_alpha`,
        `color_literal_eight_digit_mixed_channels`,
        `color_literal_uppercase_hex_accepted`.
      - Lex reject (T2): `wasamoc/src/lexer.rs::tests`
        `color_literal_three_hex_rejected`,
        `color_literal_seven_hex_rejected`,
        `color_literal_no_hex_rejected`.
      - Parse accept (T2): `wasamoc/src/parser.rs::tests`
        `property_bind_color_literal_six_hex`,
        `property_bind_color_literal_eight_hex`,
        `box_image_placeholder_shape`.
      - Check accept (T3): `wasamoc/src/check.rs::tests`
        `box_fill_only_accepted`, `box_scrim_alpha_accepted`,
        `box_placeholder_shape_accepted`.
      - Positional reject (T3): `wasamoc/src/check.rs::tests`
        `color_literal_in_state_default_rejected`,
        `color_literal_in_non_box_prop_rejected`.
      - Lower (T4): `wasamoc/src/lower.rs::tests`
        `box_fill_only_opaque_lowered_to_ir_color`,
        `box_fill_with_alpha_lowered_to_ir_color`,
        `box_aspect_and_fill_lowered_together`,
        `box_with_text_child_placeholder_shape_lowered`.
      - Emit (T5): `wasamoc/src/emit.rs::tests`
        `box_fill_opaque_color_emitted_in_short_form`,
        `box_fill_color_with_alpha_emitted_in_full_form`,
        `color_emit_normalises_alpha_ff_input_to_short_form`,
        `box_phase2_placeholder_widget_node_shape_emitted`,
        `box_phase2_ir_text_emit_fixture`.
      - IR-loader lex (T7): `wasamo-runtime/src/ir_loader.rs::tests`
        `color_literal_short_form_packs_implicit_alpha_ff`,
        `color_literal_long_form_carries_explicit_alpha`,
        `color_literal_long_form_with_full_rgba`,
        `color_must_be_six_or_eight_hex_digits`.
- [x] Aspect measure-arrange resolver: each DD-M3-P2-005 case
      enumerated in T8.
      - Already landed in T8 as 13 tests in
        `wasamo-runtime/src/layout.rs::tests`:
        - Inscribed-fit numeric contract:
          `box_aspect_inscribed_width_constrained`,
          `box_aspect_inscribed_height_constrained`,
          `box_aspect_equal_touch_takes_width_branch`.
        - One-axis bounded / both-axes runtime error:
          `box_aspect_unbounded_height_uses_bounded_axis_wins`,
          `box_aspect_unbounded_width_uses_bounded_axis_wins`,
          `box_aspect_unbounded_both_axes_is_runtime_error`.
        - No-aspect cases:
          `box_no_aspect_empty_matches_parent_bounds`,
          `box_no_aspect_empty_unbounded_both_is_runtime_error`,
          `box_no_aspect_empty_one_axis_unbounded_collapses_to_zero`,
          `box_no_aspect_shrinks_to_fit_child`.
        - Single child centred + clipped:
          `box_aspect_child_measured_centred_and_intrinsic_kept`,
          `box_aspect_oversize_child_clipped_to_box_bounds`.
        - Container integration / zero-child rectangle:
          `box_aspect_in_vstack_uses_inscribed_via_bounded_axis_wins`,
          `box_zero_child_still_has_size`.
- [x] `wasamoc check` diagnostics: `bind aspect:`, `bind fill:`,
      2+ children rejected (per DD-M3-P2-001 / DD-M3-P2-004).
      - Bind-aspect reject (T3): `wasamoc/src/check.rs::tests`
        `box_aspect_state_ident_rejected`,
        `box_aspect_int_literal_rejected`,
        `box_aspect_color_literal_rejected`.
      - Bind-fill reject (T3): `wasamoc/src/check.rs::tests`
        `box_fill_state_ident_rejected`,
        `box_fill_string_literal_rejected`,
        `box_fill_ratio_literal_rejected`.
      - 2+ children at compile time (T3):
        `wasamoc/src/check.rs::tests`
        `box_two_children_rejected`,
        `box_three_children_rejected`,
        `box_attrs_do_not_count_as_children`,
        `box_one_child_accepted`.
      - 2+ children at IR-load time (T7,
        `ir_loader::build_node` defense-in-depth path):
        `wasamo-runtime/src/ir_loader.rs::tests`
        `malformed_box_with_two_children`,
        `box_with_single_child_is_valid`,
        `box_with_zero_children_is_valid`.

Closed by commit `1e42d85 docs(m3-phase-2): T9 pure-logic test
inventory and checklist close (M3-Phase 2 T9)`. Step-end
retrospective recorded in
[../../notes/m3-phase-2/t9-step-end-retrospective.md](../../notes/m3-phase-2/t9-step-end-retrospective.md).

### T10 — IR text round-trip evidence (ADR §Phase 2 verification closure item 2)

- [x] Round-trip fixture:
      `Box { aspect: 16:9; fill: #00000080; Text { text: "Photo 12" } }`.
      Lives as the cross-crate driver `PHASE2_FIXTURE` in
      `wasamo-runtime/tests/box_round_trip.rs`. The same fixture
      string is also asserted in-crate by
      `wasamoc::emit::tests::box_phase2_ir_text_emit_fixture` (T5)
      and `wasamo-runtime::ir_loader::tests::box_phase2_load_side
      _fixture` (T7); T10 joins their reference strings through the
      actual `wasamoc::emit::emit` output.
- [x] Emit side: Box node carries
      `IrLiteral::Ratio { num: 16, den: 9 }` and
      `IrLiteral::Color(<packed>)`. Exercised by the pure-logic
      cross-crate test
      `box_phase2_emit_parses_back_to_ir_literal_variants` —
      `wasamoc::emit::emit` output is fed into
      `wasamo_runtime::ir_loader::parse_ir`, and the resulting
      `IrComponent` is asserted to carry `IrLiteral::Ratio { num: 16,
      den: 9 }` and `IrLiteral::Color(0x80_00_00_00)`. Runs on any
      CI runner (no Compositor needed).
- [x] Load side: after `ir_loader::build_node`, runtime state is
      `WidgetData::Box { aspect: Some(Ratio { 16, 9 }),
      fill: Some(Color(<packed>)), .. }` — `IrLiteral::*` do not
      survive into runtime state (per DD-M3-P2-002 / DD-M3-P2-003).
      Exercised by the Windows-only
      `box_phase2_build_node_materialises_box_internal_state`, which
      drives the full `lower → emit → parse_ir → build_widget_tree`
      chain against a live Compositor and reads the resulting
      `WidgetData::Box` through the new `WidgetNode::__box_state
      _for_test` accessor (a `#[doc(hidden)] pub fn` returning
      Box-internal `aspect` / `fill` as primitives so the
      `box_values::Ratio` / `Color` `pub(crate)` surface stays
      narrow). Skip-guard mirrors Phase 1 T6 / T13: fail (not skip)
      on GitHub Actions if `wasamo_init` returns `0x80070005`. The
      `WidgetData::Box.fill` `#[allow(dead_code)]` carried from T8 /
      T9 is dropped in this step because the accessor now reads the
      field unconditionally — T11 reuses the accessor for its
      `fill` brush peek with no further surface change.
- [x] `ir_loader` rejection of 2+ children also exercised here.
      `box_phase2_two_children_rejected_at_parse_ir` re-states the
      `wasamo-runtime::ir_loader::tests::malformed_box_with_two
      _children` (T7) gate from inside the cross-crate file so
      T10's checklist owns an observable defense-in-depth test on
      the integration-test surface.

Closed by commit `8d12f66 feat(wasamo-runtime): IR text round-trip
evidence for Box (M3-Phase 2 T10)`. Step-end retrospective recorded
in
[../../notes/m3-phase-2/t10-step-end-retrospective.md](../../notes/m3-phase-2/t10-step-end-retrospective.md).

### T11 — Windows-runtime layout integration test (ADR §Phase 2 verification closure item 3, CI-gated)

- [ ] Mock-free Windows-only integration test under
      `wasamo-runtime/tests/`: aspect-fixed Box with Text child
      inside a parent of known size; asserts inscribed-fit
      resolved rectangle and child centred.
- [ ] `fill` verified via a Box-internal / test-only accessor or
      via the render model (`SpriteVisual` brush), not via
      `wasamo_get_property` (per DD-M3-P2-003 variant strategy).
- [ ] Skip-guard matches Phase 1 T6 / T13: fail (not skip) on CI
      when Compositor unavailable, per
      [CLAUDE.md §Testing rules](../../../CLAUDE.md).
      Reuse T10's `0x80070005` skip pattern and explicitly observe
      the skip path on an SSH dev box or equivalent non-Compositor
      environment; T10 only observed the `WASAMO_OK` path locally.

### T12 — Seed `examples/gallery/` + `examples/gallery-rust/` (ADR §Phase 2 verification closure item 4)

- [ ] `examples/gallery/` with a Phase 2 sub-screen (Box + Text
      placeholder against a trivial frame). Later M3 phases grow
      this directory sub-screen by sub-screen.
- [ ] `examples/gallery-rust/` workspace-member host (mirrors the
      `examples/bool-demo-rust/` build pipeline from Phase 1).
- [ ] `Start-Process` launch recorded as successful; visual
      correctness is owner-manual GUI smoke (per pre-doc framing
      decision G).
- [ ] C / Zig hosts not required in Phase 2 (per framing decision F
      and the ADR Out-of-scope list).

### T13 — Phase-end gates

Discharges the m3-plan §Phase-end criteria checklist for Phase 2.

- [ ] `cargo fmt --all -- --check` green (per
      [retrospectives.md item 3 amendment](../../notes/retrospectives.md)
      landed in Moment 1).
- [ ] `cargo build --release --workspace` and `cargo test
      --workspace` green locally and on CI (`workflow_dispatch`).
- [ ] Windows-only integration test (T11) green on CI (fail, not
      skip, if Compositor missing).
- [ ] Moment 2 spec re-sync: flip
      [dsl_spec.md §4.9](../../dsl_spec.md#49-box-layout-primitive-m3-phase-2)
      Phase status marker to
      `**Phase status:** M3-Phase 2 closed; implementation-synced`,
      correcting any draft / impl divergence in the same commit.
      Earlier-phase spec gaps may fold per
      [predoc-inputs.md §6](../../notes/m3-phase-2/predoc-inputs.md#6-retroactive-spec-gap-fold-は最小範囲で同じ-phase-に折り込む)
      with explicit owner confirmation.
- [ ] Forward-distillation note for M3-Phase 3 authored within
      this phase's close (per
      [retrospectives.md forward-carry rule](../../notes/retrospectives.md)):
      `docs/notes/m3-phase-3/predoc-inputs.md` (or phase-named
      pre-doc candidate file).
- [ ] Phase-end retrospective entry in
      `docs/notes/m3-phase-2/phase-end-retrospective.md`.
- [ ] Progress file lifecycle: `status: active` → `status: closing`
      → retired (per
      [plans/README.md §Phase progress file lifecycle](../README.md#phase-progress-file-lifecycle)).

## Decisions log

(empty — record here mid-phase decisions that deviate from the ADR
or refine its task slicing; see Phase 1's progress file for the
shape.)

## CI / verification log

(empty — record `cargo` runs, `workflow_dispatch` CI runs, and
GUI-smoke launches here as they happen.)

## Out-of-phase residuals

- **Dedicated `WASAMO_ERR_*` ABI code for layout-time runtime errors
  (T8).** DD-M3-P2-005 specifies that the unbounded-both-axes /
  no-extent conditions surface as runtime diagnostics with the Box's
  IR location. T8 lands the structural surface (`LayoutError` enum
  returned from `layout::run_layout`) and maps both variants to
  `windows::core::Error(E_FAIL)` at `WidgetNode::run_layout` so the
  existing `WM_SIZE` callsites keep their `windows::core::Result<()>`
  shape. The dedicated `wasamo.h` error code, IR-location plumbing on
  `LayoutNode`, and the C ABI translation are out of Phase 2 scope:
  the call sites at `window.rs::WM_SIZE` and `emit.rs::mark_layout
  _dirty_for` already swallow the Result with `let _ = …`, so a richer
  surface would be unused until a phase introduces a `wasamo_run
  _layout` (or layout-error callback) entry point. Tracked here so
  the residual lands in the M3-Phase 3 / Phase 4 pre-doc input scan
  rather than getting lost.
