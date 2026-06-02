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
      [dsl_spec §4.10](../../../../docs/dsl_spec.md#410-wrappanel-layout-primitive-m3-phase-3)
      that traverse the generic parser unmodified.

### T2 — `wasamoc check`: aspect-only-Box warning

Discharges the DD-M3-P3-004 Recommendation companion judgement
(Checkpoint 2 ship-warning pick).

- [x] When a WrapPanel directly contains one or more
      `Box { aspect: <ratio>; … }` children and `item-cross-size`
      is **not** set on the WrapPanel, `wasamoc check` emits a
      **warning** (not error) suggesting the attribute. The warning
      text cross-references
      [dsl_spec §4.10 Common pitfalls](../../../../docs/dsl_spec.md#410-wrappanel-layout-primitive-m3-phase-3).
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
      [DD-M3-P3-005](../decisions/preamble.md#dd-m3-p3-005--measure-arrange-algorithm-novel-normative-spec).
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
      [CLAUDE.md §Testing rules](../../../../CLAUDE.md)) when
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
      [verification closure evidence item 4](../decisions/preamble.md#phase-3-verification-closure-what-counts-as-a3-evidence):
      the wrap-path fixture (primary positive control) and the
      oversized-child fixture (visible-overflow regulation).
- [x] Skip-guard matches Phase 1 T6 / T13 / Phase 2 T11: fail (not
      skip) on CI when Compositor unavailable; locally skip on
      `0x80070005` from `wasamo_init`.
- [x] Skip-guard verified on an SSH dev box (or equivalent
      environment per
      [verification-environments.md](../../../../docs/notes/verification-environments.md))
      before landing — local "passed without skip" does not prove
      the guard works.

### T9 — `examples/gallery/` + `examples/gallery-rust/` additive growth

Discharges ADR verification closure **evidence item 5** (visible
smoke) and the
[m3-plan §Phase-end criteria item 5](../../plan.md#phase-end-criteria)
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

- [x] `cargo fmt --all -- --check` green.
- [x] `cargo build --release --workspace` and `cargo test
      --workspace` green locally **and** on CI
      (`workflow_dispatch`). Local half: green at T10 (release +
      debug build + workspace tests). CI half: green on
      `feat/m3-phase-3` `78652c1` —
      <https://github.com/matarillo/wasamo/actions/runs/26256127948>.
- [x] Windows-only integration test (T8) green on CI (fail, not
      skip, if Compositor missing). T8 skip-guard verified locally
      on SSH dev box at T8 landing; CI green rode the same
      phase-branch `workflow_dispatch` run as the previous bullet
      (<https://github.com/matarillo/wasamo/actions/runs/26256127948>).
- [x] **Moment 2 spec re-sync.** Flipped
      [dsl_spec.md §4.10](../../../../docs/dsl_spec.md#410-wrappanel-layout-primitive-m3-phase-3)
      Phase status marker to
      `**Phase status:** M3-Phase 3 closed; implementation-synced`.
      Folded the T1 Decisions-log lexer-surface item per owner
      approval (§2.2 `Ident` pattern admits kebab-case
      continuations; §2.2 `IntLit` pattern admits an optional
      leading `-`; one-line note that the negative-sign surface is
      `IntLit`-only). Doc version 0.9 → 1.0 with a revision-history
      row describing the close.
- [x] **Moment 2 architecture re-sync.** Flipped
[docs/architecture.md](../../../../docs/architecture.md) top-level
      Status to `M3-Phase 1, M3-Phase 2, and M3-Phase 3 complete`.
      §6.8 WrapPanel paragraph already described the realised
      implementation (no wording reconciliation needed). §6.5
      `WidgetNode` / Visual-Layer sync diagram gained a one-line
      clarification of the absolute (`LayoutNode`) vs.
      parent-relative (`Visual.Offset`) offset convention, folded
      as R3-A from T9 visible-smoke (a separate commit per review-
      concern).
- [x] **Out-of-phase residuals filed** per
      [m3-plan §Phase-end criteria item 6](../../plan.md#phase-end-criteria):
      R1 (`.gitignore` `*.uic` pattern) and R2 (`sync_visuals` ↔
      pure-layout boundary test gap) recorded in §Out-of-phase
      residuals below and cross-referenced from
      [the ADR's Phase 3 implementation residuals subsection](../decisions/preamble.md).
      R3 (architecture §6.5 offset convention) was folded in T10
      as R3-A and is not a residual.
- [x] Forward-distillation note for M3-Phase 4 authored within
      this phase's close (per
      [retrospectives.md forward-carry rule](../../../procedures/retrospectives.md)):
      [`docs/notes/m3-phase-4/pre-doc-inputs.md`](../../phase-4/requirements/constraints.md).
- [x] Phase-end retrospective entry recorded per the
      [docs/notes/retrospectives.md](../../../procedures/retrospectives.md)
      procedure, with the durable entry at
      [`docs/notes/m3-phase-3/phase-end-retrospective.md`](../retrospectives/phase-end.md).
- [x] Progress file lifecycle: `status: active` → `status: closing`
      at the end of this T10 commit set, then `closing` →
      `status: retired` on the phase branch before the phase-end
      main-merge gate (matching the Phase 2 close pattern in
      [m3-phase-2-progress.md](../../phase-2/retrospectives/phase-end.md); per
      [plans/README.md §Phase progress file lifecycle](../README.md#phase-progress-file-lifecycle)).
