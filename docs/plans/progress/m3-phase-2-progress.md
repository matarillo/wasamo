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

- [ ] Bounded inscribed-fit branch selection per the DD-M3-P2-005
      numeric / rounding contract.
- [ ] Unbounded-on-one-axis: bounded-axis-wins.
- [ ] Unbounded-on-both-axes (aspect set, or no-aspect Box with
      no bounded extent): layout-time runtime error.
- [ ] No-aspect bounded Box: matches parent bounds when empty;
      shrink-to-fit child intrinsic measure when child present.
- [ ] Single child: measured against Box bounds, centred, clipped
      on overflow (per DD-M3-P2-001 child measure / alignment /
      overflow).
- [ ] Zero-child Box still produces a sized rectangle (filled with
      `fill`, or transparent when absent).

### T9 — Pure-logic unit tests (ADR §Phase 2 verification closure item 1)

- [ ] Ratio literal: accept shapes; zero side rejected at check.
- [ ] Color literal: `#RRGGBB` / `#RRGGBBAA` accept; malformed
      forms rejected at lex / parse.
- [ ] Aspect measure-arrange resolver: each DD-M3-P2-005 case
      enumerated in T8.
- [ ] `wasamoc check` diagnostics: `bind aspect:`, `bind fill:`,
      2+ children rejected (per DD-M3-P2-001 / DD-M3-P2-004).

### T10 — IR text round-trip evidence (ADR §Phase 2 verification closure item 2)

- [ ] Round-trip fixture:
      `Box { aspect: 16:9; fill: #00000080; Text { text: "Photo 12" } }`.
- [ ] Emit side: Box node carries
      `IrLiteral::Ratio { num: 16, den: 9 }` and
      `IrLiteral::Color(<packed>)`.
- [ ] Load side: after `ir_loader::build_node`, runtime state is
      `WidgetData::Box { aspect: Some(Ratio { 16, 9 }),
      fill: Some(Color(<packed>)), .. }` — `IrLiteral::*` do not
      survive into runtime state (per DD-M3-P2-002 / DD-M3-P2-003).
- [ ] `ir_loader` rejection of 2+ children also exercised here.

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

(empty — record here anything discovered during execution that is
out of Phase 2 scope, and file a `docs/notes/m3/` entry pointing
back to it per the m3-plan §Phase-end criteria.)
