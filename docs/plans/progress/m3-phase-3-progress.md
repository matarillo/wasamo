---
phase: M3-Phase 3
title: WrapPanel layout primitive
status: active
adr: docs/decisions/m3-phase-3-wrap-panel.md
plan: docs/plans/m3-plan.md
opened: 2026-05-21
---

# M3-Phase 3 — WrapPanel layout primitive: Progress

This is the live task list and execution log for M3-Phase 3. The
design decisions are frozen in
[m3-phase-3-wrap-panel.md](../../decisions/m3-phase-3-wrap-panel.md);
this file is mutable per
[plans/README.md §Phase progress file lifecycle](../README.md#phase-progress-file-lifecycle).

Task ordering follows the dependency direction
`wasamoc → wasamo-runtime → tests → host/spec`, so each commit
builds on a green workspace per
[CLAUDE.md §Commit rules](../../../CLAUDE.md). Phase 3 introduces
**no new parser grammar**: `wasamoc`'s parser already accepts the
generic `IDENT "{" ... "}"` widget-declaration shape and the
generic `IDENT ":" expr` property-bind shape, so WrapPanel and its
three attributes traverse the existing surface unchanged. Phase 3
likewise introduces **no new `IrType`, no new `IrLiteral` variant,
no new `PropertyValue` variant, no new `LayoutError` variant**
(DD-M3-P3-001 / DD-M3-P3-003 / DD-M3-P3-004 / DD-M3-P3-005). The
WrapPanel-shaped IR surfaces as a new `widget_type` value plus
three new `IrProp` names on the generic `IrNode`. Items may be
split, reordered, or merged when implementation reveals a tighter
ordering — this list is the record of what actually happens, not
a frozen prediction.

The five pieces of A3 evidence the phase closes against are
enumerated in
[m3-phase-3-wrap-panel.md §Phase 3 verification closure](../../decisions/m3-phase-3-wrap-panel.md#phase-3-verification-closure-what-counts-as-a3-evidence).
Each T below cites the evidence item it advances or discharges.

## Task list

### T1 — `wasamoc check`: WrapPanel validity and reject set

Discharges DD-M3-P3-001 (child count + surface registration),
DD-M3-P3-006 (compile-time half of two-gate defense for non-
negative integers), and the constant-only halves of
DD-M3-P3-003 / DD-M3-P3-004.

- [x] **Lexer prerequisite — kebab-case ident generalization +
      negative IntLit.** dsl_spec §4.10's kebab attribute names
      (`item-cross-size`, `item-spacing`, `line-spacing`) require
      the lexer to emit a single `Token::Ident` for each, and the
      DD-M3-P3-006 compile-time diagnostic naming the attribute on
      `: -1` requires `-1` to lex as `IntLit(-1)`. Both are lexer
      changes the progress doc's "no new parser grammar" prose did
      not surface; see Decisions log. The parser's IDENT-keyed
      PropertyBind shape is unchanged.
- [x] `WrapPanel` added to the checker's known-widget registry
      (`KNOWN_WIDGET_TYPES` in `wasamoc/src/check.rs` or its
      equivalent) so that the generic parser's
      `widget_type: "WrapPanel"` node is recognised as a valid
      widget rather than warning as unknown.
- [x] Accept 0-child / 1-child / multi-child WrapPanel (DD-M3-P3-001
      0+ children, no upper bound).
- [x] Reject negative literal on `item-cross-size`, `item-spacing`,
      `line-spacing` (DD-M3-P3-006 compile-time gate); diagnostic
      names the rejected attribute. Zero is a *valid* setting on
      all three (DD-M3-P3-006 zero-handling); the rejection
      threshold is `< 0`, not `<= 0`.
- [x] Reject `bind item-cross-size:`, `bind item-spacing:`,
      `bind line-spacing:` (constant-only per DD-M3-P3-003 /
      DD-M3-P3-004); diagnostic names the rejected attribute.
- [x] Reject non-`IntLit` RHS shapes on the three attributes
      (e.g. `Ident`, `RATIO_LIT`, `STRING_LIT`, `BOOL_LIT`,
      `COLOR_LIT`, `number_with_unit`) — they are constant-only
      `i32` literals per §4.10. Diagnostic names the rejected
      attribute.
- [x] Reject the three attributes outside WrapPanel (attribute-
      position rejection); diagnostic names the offending position.
- [x] Widget property catalog extended for WrapPanel
      (`item-cross-size: i32`, `item-spacing: i32`,
      `line-spacing: i32`) so the checker can name the attribute
      types in diagnostics. The catalog reuses the existing `i32`
      `TypeName` entry (no new `TypeName` variant).
- [x] Unit tests cover each row of the reject set + each accept
      shape from the ADR, including the accept-shape fixtures
      from
      [dsl_spec §4.10](../../dsl_spec.md#410-wrappanel-layout-primitive-m3-phase-3)
      that traverse the generic parser unmodified.

### T2 — `wasamoc check`: aspect-only-Box warning

Discharges the DD-M3-P3-004 Recommendation companion judgement
(Checkpoint 2 ship-warning pick).

- [x] When a WrapPanel directly contains one or more
      `Box { aspect: <ratio>; … }` children and `item-cross-size`
      is **not** set on the WrapPanel, `wasamoc check` emits a
      **warning** (not error) suggesting the attribute. The warning
      text cross-references
      [dsl_spec §4.10 Common pitfalls](../../dsl_spec.md#410-wrappanel-layout-primitive-m3-phase-3).
- [x] Warning fires on direct-child Boxes only (does not scan into
      nested containers); narrow guard scope per DD-M3-P3-004
      "the warning does not classify all possible child shapes,
      only the known aspect-only-Box footgun".
- [x] Unit tests cover the firing shape, the non-firing shape
      (`item-cross-size` set explicitly; the Phase 3 gallery
      sub-screen's positive control), and the non-direct-child
      shape (an aspect-only Box nested inside another container
      inside the WrapPanel — no warning).

### T3 — `wasamoc` lowering: AST → IR

- [x] `Expr::IntLit` on WrapPanel attribute RHSes lowers via the
      existing `IrLiteral::Int` path. Non-`IntLit` RHS shapes
      accepted by the grammar are rejected by T1 before reaching
      lowering — no new lowering branches added.
- [x] `WrapPanel { ... }` widget declaration lowers to an
      `IrNode { widget_type: "WrapPanel", props: [...], children: [...] }`
      with the three attributes recorded as `IrProp` entries when
      set; absent attributes are omitted from the IR (defaults are
      applied at the runtime layer in T5, not at the IR layer).
- [x] Unit tests assert end-to-end parse → lower for
      representative `WrapPanel { ... }` forms (0-child, single
      Box child, multi-Box-child with all three attributes set,
      multi-child with only `item-cross-size` set, multi-child
      with no attributes). These tests also serve as regression
      coverage that the generic parser handles WrapPanel-shaped
      declarations without modification.

### T4 — `wasamoc` IR text emit

- [x] Emitter writes the WrapPanel widget node and the three
      attribute properties in the existing `prop` literal position
      using the standard `i32` literal form (decimal integer; no
      new emit grammar).
- [x] Attributes absent on the IR side are also omitted from the
      IR text (round-trip fidelity: parse → IR → emit → parse
      produces the same IR shape).
- [x] Unit tests cover the emit forms for each combination of
      attribute presence / absence; one round-trip test asserts
      stability across an emit / re-parse cycle.

### T5 — `wasamo-runtime` widget catalog

- [x] `WidgetData::WrapPanel { item_cross_size: Option<i32>,
      item_spacing: i32, line_spacing: i32 }` variant added in
      `wasamo-runtime/src/widget.rs` (children live on
      `WidgetNode.children` per the existing per-widget
      convention, mirroring Phase 2's `Box` shape). Defaults when
      an attribute is unset: `item_cross_size: None`
      (parent-cross passthrough per DD-M3-P3-004 Option (a));
      `item_spacing: 0` / `line_spacing: 0` (touching items /
      lines per DD-M3-P3-003).
- [x] `WidgetKind::WrapPanel` arm added; all existing exhaustive
      matches on `WidgetKind` / `WidgetData` gain a `WrapPanel`
      arm.
- [x] Layout dispatch wires `WidgetKind::WrapPanel` into the
      Phase 3 layout function added in T7 (placeholder dispatch
      arm at this commit; T7 fills it).

### T6 — `wasamo-runtime` IR loader + `validate()` defense-in-depth

Discharges DD-M3-P3-006 runtime gate; ADR verification closure
**evidence item 3**.

- [x] IR loader recognises the WrapPanel widget node and
      materialises `WidgetData::WrapPanel` directly (no
      `PropertyValue` involvement; the three attributes stay
      Box-internal-pattern fields).
- [x] `validate()` rejects memory-IR with negative
      `item_cross_size` / `item_spacing` / `line_spacing` values
      (last-line-of-defence for the spec invariant since
      `wasamo_load_ui`'s memory-IR path bypasses `wasamoc`).
      Error surface: `WASAMO_ERR_IR_MALFORMED` (DD-M3-P3-006
      error class).
- [x] Pure-logic unit tests covering: 0-child WrapPanel valid;
      1-child WrapPanel valid; multi-child WrapPanel valid (no
      upper bound); each of the three negative-value rejection
      paths fires under memory-IR; the zero-value path is *not*
      rejected (zero is valid per DD-M3-P3-006). Symmetric with
      Phase 2 T7's `validate()` discipline.

### T7 — Layout engine: WrapPanel line-breaker and arrange

Discharges DD-M3-P3-005 (novel normative measure-arrange); ADR
verification closure **evidence item 2**.

- [x] Implement the pure-data WrapPanel measure/arrange path in
      `wasamo-runtime/src/layout.rs` per
      [DD-M3-P3-005](../../decisions/m3-phase-3-wrap-panel.md#dd-m3-p3-005--measure-arrange-algorithm-novel-normative-spec).
      The layout boundary remains Win32/WinRT-free; the algorithm
      operates on `LayoutNode` / measure / arrange inputs only.
- [x] Add the pure-logic test coverage enumerated under the
      DD-M3-P3-005 Recommendation (bounded / unbounded main-axis
      cases, oversized-first-child unconditional placement and
      its visible-overflow arrange evidence, cross-axis sizing
      with / without `item-cross-size`, spacing-aware overflow
      inequality, zero-attribute degenerate cases, and the
      unbounded-cross-axis-with-aspect-child propagation to
      `LayoutError::BoxAspectUnboundedBoth`).
- [x] Prefer free-function extraction before the test-only
      mirror pattern (per
      [CLAUDE.md §Testing rules](../../../CLAUDE.md)) when
      pure logic entangles a Win32/WinRT-bound type.
- [x] Rounding contract inherits Phase 2 DD-M3-P2-005's
      discipline (no pixel-snapping in Phase 3).

### T8 — Windows-runtime integration test (ADR §Phase 3 verification closure evidence item 4, CI-gated)

Lands the Windows-runtime integration test required by ADR
verification closure **evidence item 4** (CI-gated Compositor
pipeline). CI green confirmation of the test itself remains T10's
"Windows-only integration test (T8) green on CI" checkbox — T8
closes the test landing and the skip-guard verification, not the
CI-execution evidence.

- [x] Mock-free integration test on the Windows CI runner that
      exercises the two fixtures specified in the ADR's
      [verification closure evidence item 4](../../decisions/m3-phase-3-wrap-panel.md#phase-3-verification-closure-what-counts-as-a3-evidence):
      the wrap-path fixture (primary positive control) and the
      oversized-child fixture (visible-overflow regulation).
- [x] Skip-guard matches Phase 1 T6 / T13 / Phase 2 T11: fail (not
      skip) on CI when Compositor unavailable; locally skip on
      `0x80070005` from `wasamo_init`.
- [x] Skip-guard verified on an SSH dev box (or equivalent
      environment per
      [verification-environments.md](../../notes/verification-environments.md))
      before landing — local "passed without skip" does not prove
      the guard works.

### T9 — `examples/gallery/` + `examples/gallery-rust/` additive growth

Discharges ADR verification closure **evidence item 5** (visible
smoke) and the
[m3-plan §Phase-end criteria item 5](../m3-plan.md#phase-end-criteria)
"gallery sub-screen per phase" obligation.

- [x] `examples/gallery/gallery.ui` grows additively from Phase 2's
      single-Box sub-screen into a WrapPanel of uniform 1:1 Box
      thumbnails (5–10 items, hand-written; no iteration, no
      ScrollView). Per framing decision E.
- [x] `examples/gallery-rust/` (already a workspace member from
      Phase 2) builds and runs the grown sub-screen.
- [x] `Start-Process` launch recorded as successful by the
      assistant; visual correctness is **owner-manual GUI smoke**
      per framing decision G — the assistant does not assert on
      pixel- or eyeball-level correctness.
- [x] C / Zig hosts not required in Phase 3 (per framing decision E
      and the ADR Out-of-scope list); Phase 8 broadens the full
      gallery to all three.

### T10 — Phase-end gates

Discharges the m3-plan §Phase-end criteria checklist for Phase 3.

- [ ] `cargo fmt --all -- --check` green.
- [ ] `cargo build --release --workspace` and `cargo test
      --workspace` green locally and on CI (`workflow_dispatch`).
- [ ] Windows-only integration test (T8) green on CI (fail, not
      skip, if Compositor missing).
- [ ] **Moment 2 spec re-sync.** Flip
      [dsl_spec.md §4.10](../../dsl_spec.md#410-wrappanel-layout-primitive-m3-phase-3)
      Phase status marker to
      `**Phase status:** M3-Phase 3 closed; implementation-synced`,
      correcting any draft / impl divergence in the same commit.
      Earlier-phase spec gaps may fold per
      [m3-phase-2 predoc-inputs §6 retroactive spec-gap fold](../../notes/m3-phase-2/predoc-inputs.md#6-retroactive-spec-gap-fold-は最小範囲で同じ-phase-に折り込む)
      (rule established in M3-P1 T10 and applied in Phase 2 T13)
      with explicit owner confirmation. Update doc version 0.9 →
      next (per Phase 2's 0.7 → 0.8 close pattern) and add the
      revision-history row describing the close.
- [ ] **Moment 2 architecture re-sync.** Flip
      [docs/architecture.md](../../architecture.md) top-level
      Status from `M3-Phase 3 ADR-accepted design draft (pending
      implementation re-sync)` to `M3-Phase 3 complete`, and
      reconcile any §6.8 WrapPanel paragraph block wording against
      the realised implementation in the same commit.
- [ ] **Out-of-phase residuals filed** per
      [m3-plan §Phase-end criteria item 6](../m3-plan.md#phase-end-criteria):
      anything surfaced during Phase 3 that is real but out of
      scope is recorded in §Out-of-phase residuals below and
      cross-referenced from the ADR's residual / handover section.
- [ ] Forward-distillation note for M3-Phase 4 authored within
      this phase's close (per
      [retrospectives.md forward-carry rule](../../notes/retrospectives.md)):
      `docs/notes/m3-phase-4/predoc-inputs.md` (or phase-named
      pre-doc candidate file).
- [ ] Phase-end retrospective entry recorded per the
      [docs/notes/retrospectives.md](../../notes/retrospectives.md)
      procedure, with the durable entry at
      `docs/notes/m3-phase-3/phase-end-retrospective.md`
      (following Phase 2 practice; m3-plan criterion 7's wording
      alignment is tracked in a separate plan-side revision).
- [ ] Progress file lifecycle: `status: active` → `status: closing`
      → retired (per
      [plans/README.md §Phase progress file lifecycle](../README.md#phase-progress-file-lifecycle)).

## Decisions log

### 2026-05-21 — Lexer prerequisite for kebab-case attribute names and negative integer literals (T1)

dsl_spec §4.10's three WrapPanel attribute names are kebab-case
(`item-cross-size`, `item-spacing`, `line-spacing`). The
`wasamoc` lexer prior to T1 only recognised the `in-out` keyword
through a hardcoded special case in `scan_ident` and rejected `-`
in identifier position elsewhere; the top-level `-` handler
rejected `-` not followed by `=`. Both surfaces blocked T1:

- Without kebab-aware `scan_ident`, `item-cross-size: 88`
  tokenized as `item` Ident, then `-` "unexpected `-`" error.
  The PropertyBind never reached the checker, so the attribute
  diagnostics had no surface to fire on.
- Without negative-IntLit support, `item-spacing: -1` errored
  at the lexer. The DD-M3-P3-006 compile-time gate ("diagnostic
  names the rejected attribute") was unreachable.

The owner confirmed the lexer-extension approach on 2026-05-21
(over the alternative of revising the spec to snake_case
attribute names):

- `scan_ident` now loops over alphanumeric segments joined by `-`
  when the next character after `-` is alphabetic. `count -= 1`
  still tokenizes correctly because the `=` after `-` breaks the
  kebab continuation rule.
- The `in-out` keyword's hardcoded entry in `scan_ident` is
  removed; `in-out` is matched via the post-scan keyword table
  on parity with `component` / `inherits` / etc.
- The top-level `-` arm emits a negative `IntLit` when followed
  by an ASCII digit. Bare `-` remains an error; binary
  subtraction is not grammatical in the DSL, so the leading-sign
  reading is unambiguous in expression position.
- **The negative entry path is integer-only.** `scan_number`'s
  fractional (`-1.5`) and `px`-unit (`-12px`) branches are
  rejected at lex time when entered with `negative == true`,
  yielding diagnostics that point to integer literals as the
  only legal negative form. `FloatLit` and `Measurement` remain
  unsigned per dsl_spec §5 (AST table) and §2 (measurement
  surface). This bound was introduced after rev 1 of the T1
  retrospective surfaced a scope-widening issue in owner review;
  see commit `bf7aee0` and `t1-step-end-retrospective.md` rev 2.
- Ratio literals (`<num>:<den>`) remain unsigned by construction
  (dsl_spec §4.9 surface). The negative-entry path in
  `scan_number` skips the ratio fold.

**Behavioural deltas from the pre-T1 lexer:**

- `in-outx` previously errored at lex time (`scan_ident` saw
  `in`, hit `-` not followed by `out`-end, returned `in`, then
  the `-` arm errored). It now lexes as `Token::Ident("in-outx")`.
  No existing program shape relied on the previous rejection; the
  parser would still reject `in-outx` in any meaningful context.
- The lexer test `in_out_followed_by_alphanumeric_is_error` was
  replaced with `in_outx_lexes_as_kebab_ident` plus six new
  kebab / negative-literal tests:
  - `kebab_case_ident`
  - `kebab_ident_breaks_on_non_alpha_after_hyphen`
  - `negative_int_literal`
  - `negative_int_in_property_bind_position`
  - `negative_float_literal_rejected` (rev 2; `-1.5` reject)
  - `negative_measurement_literal_rejected` (rev 2; `-12px` reject)

The progress doc's "Phase 3 introduces no new parser grammar"
prose continues to hold — the parser's IDENT-keyed PropertyBind
shape is unchanged. The lexer's `Token::Ident` surface widens
(kebab segments admitted) and the integer-literal sign domain
widens (negative IntLit admitted); `FloatLit` / `Measurement` /
`RatioLit` surfaces remain unsigned. This is recorded so the
Phase 3 closing Moment 2 spec re-sync can decide whether the
lexer change deserves its own note in dsl_spec §2 (lexical
surface) — and specifically whether the "IntLit signed,
FloatLit / Measurement unsigned" split needs a one-line
confirmation in §2 / §5 — or is sufficiently absorbed by
§4.10's normative attribute-name list and the existing AST
table.

### 2026-05-22 — Gallery sub-screen numerics restored to ADR canonical `88 / 12 / 12` (T9 rev 2)

Initial T9 landing (commit `d1e5ba6`) used
`item-cross-size: 120; item-spacing: 16; line-spacing: 16` with
8 thumbnails because, on the default 800×600 window
(≈ 784 px client width), the ADR-canonical `88 / 12 / 12` with
8 thumbs produces a `7 + 1` wrap which is visually unbalanced. The
deviation was un-documented at landing time; owner review
([t9-step-end-retrospective.md rev 2](../../notes/m3-phase-3/t9-step-end-retrospective.md))
flagged that the ADR
[§Phase 3 verification closure item 1 (sub-screen positive control)](../../decisions/m3-phase-3-wrap-panel.md#phase-3-verification-closure-what-counts-as-a3-evidence)
and item 4 (CI integration fixture) both reference `88` as the
canonical example, so the implementation should match unless the
deviation is recorded.

Resolution (commit set landed after `e89a423`): restore
`88 / 12 / 12` and grow the thumbnail count from 8 to 10 (still
within framing decision E's 5–10 ceiling). On the same default
window the layout becomes `7 + 3` — visible wrap is preserved and
the numerics match the ADR-canonical example. The choice "match
ADR canonical example, adjust count for visible balance" was
selected over the alternatives "keep `120 / 16 / 16` with a
deviation note" and "keep 8 thumbs with `7 + 1` wrap" after the
review. No ADR or spec change is required by this revert — the
ADR's normative content does not pin the gallery to specific
dimensions; it pins the integration fixture (T8, unaffected) and
references the sub-screen example as `item-cross-size: 88` in the
positive-control description.

## CI / verification log

(empty — populated as T1–T10 land; see Phase 2 progress file for
the shape.)

## Out-of-phase residuals

- **(R1) `.gitignore` `*.uic` pattern.** During T9, an ad-hoc debug
  invocation `wasamoc build examples\gallery\gallery.ui
  examples\gallery\gallery.uic` produced an in-tree `.uic` artefact
  (removed manually). The production build paths route `.uic`
  through `OUT_DIR` via `build.rs` (`examples/*/build.rs`), so the
  in-tree artefact is never produced by a normal workspace build —
  but the temptation to write `.uic` in-tree for debugging recurs.
  A `.gitignore` rule for `*.uic` would prevent accidental commits.
  Phase 3 scope did not include build-hygiene changes, so this is
  not folded here; tracked for any future cross-cutting hygiene
  pass. Surfaced in
  [t9-step-end-retrospective.md](../../notes/m3-phase-3/t9-step-end-retrospective.md)
  Follow-Up R1.

- **(R2) `sync_visuals` ↔ pure-layout boundary test gap.** The
  Phase 2 test suite pins `LayoutNode.offset` to the absolute
  (root-relative) convention but does not exercise the conversion
  to parent-relative `Visual.Offset` performed by `sync_visuals()`.
  The T9 visible-smoke bug whose fix landed at commit `570d08a` was
  detected only by owner-manual GUI smoke (framing decision G); a
  regression of the same class would again rely on visible-smoke
  detection. A pure-or-Compositor-backed test that asserts the
  relative-offset computation for a nested non-zero-offset visual
  tree would close the detection gap independently of visible
  smoke. Belongs to whichever later phase first revisits the
  `WidgetNode` / Visual-Layer sync seam (likely Phase 4 ScrollView
  or a focused test-coverage pass). Surfaced in
  [t9-step-end-retrospective.md](../../notes/m3-phase-3/t9-step-end-retrospective.md)
  Follow-Up R2. Architecture-level offset convention is now stated
  in [docs/architecture.md §6.5](../../architecture.md) (folded in
  T10 as R3-A); this residual is the test-coverage half that is
  not folded.
