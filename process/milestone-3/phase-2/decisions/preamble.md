# M3-Phase 2 — Box layout primitive: Architecture Decisions

**Phase:** M3-Phase 2 (Box layout primitive)
**Date:** 2026-05-20
**Status:** Accepted

## Context

M3 acceptance criterion **A6** (see
[process/_roadmap.md M3](../../../_roadmap.md#m3-dsl-surface),
[m3-plan.md §Acceptance criteria](../../plan.md#acceptance-criteria)):

> Box layout primitive (0+ child container; `aspect: <ratio>` attribute
> subsumes a standalone AspectRatio; minimal `fill: <color>` attribute
> for scrim use). Image-widget deferral is carried by Box + Text-child
> placeholders.

The pre-doc framing for this phase was aligned with the owner on
2026-05-20 and is recorded in
[docs/notes/m3-phase-2/m3-phase-2-pre-doc-framing.md](../requirements/framing.md).
That framing fixed the 6-DD slate carried below, the visible-proof
location (framing decision F — seed `examples/gallery/` +
`examples/gallery-rust/`), the verification-strategy menu picks
(framing decision C), the `cargo fmt` discipline amendment (framing
decision E), and the two upstream-document-revision moments (framing
decision D — Moment 1 design-spec draft at ADR-Accepted commit;
Moment 2 implementation re-sync at phase close).

The M2/M3-Phase 1 end-state shape that this phase extends without
breaking:

- `wasamo-ir` ([wasamo-ir/src/lib.rs](../../../../wasamo-ir/src/lib.rs)):
  `IrType` is `I32 | Str | Bool`; `IrLiteral` is `Int | Str | Ident |
  Bool`. `HandlerExpr` uses the unified-but-type-suffixed pattern
  (`IntLit` / `StrLit` / `BoolLit` / `PropRead` / `StrPropRead` /
  `BoolPropRead`). Adding new primitive types follows the same
  type-suffix discipline (DD-M2-P6-003 / DD-M3-P1-003).
- `wasamo-runtime` widget catalog
  ([wasamo-runtime/src/widget.rs](../../../../wasamo-runtime/src/widget.rs)):
  `Rectangle | VStack | HStack | Text | Button`; `PropertyValue` enum
  is `I32(i32) | String(String) | Bool(bool)`. Per-widget per-attribute
  `PROP_*` u32 IDs; `resolve_prop_key` returns `(PropertyKey, IrType)`
  (DD-M3-P1-009).
- Binding pipeline
  ([wasamo-runtime/src/handler.rs](../../../../wasamo-runtime/src/handler.rs),
  [wasamo-runtime/src/ir_loader.rs](../../../../wasamo-runtime/src/ir_loader.rs)):
  per-type binding evaluator + per-type widget writer, dispatched at
  `ir_loader::build_node` by the property's `IrType` (DD-M3-P1-007).
  The reactive engine itself remains type-agnostic. F5
  (`TypedValue` deferral) is held in force by this seam pattern.
- `wasamoc` ([wasamoc/src/check.rs](../../../../wasamoc/src/check.rs)):
  state-name → declared-type table; identifier resolution lowers to
  typed `*PropRead` variants; `bind` LHS / RHS type pairings are
  diagnosed at compile time (DD-M3-P1-010). `TypeName::Float` already
  exists in the AST but has no IR / runtime mirror.

This ADR is framed against A6 and the M2/M3-Phase 1 type-suffix
pattern. It does **not** re-open F5 (`TypedValue` deferral) — adding
new constant-only literal forms and (where admitted) new per-type
bindable seams is the additive path proven in Phase 1.

The acceptance lens for this phase: A6 is satisfied when (i) `.ui`
declares `Box { aspect: <ratio>; fill: <color>; <child> }` and the
shared crates lower → load → render it with the right rectangle, (ii)
the placeholder pattern is canonicalized in `docs/dsl_spec.md` so
Phase 3 (WrapPanel) and Phase 6 (ZStack) cite rather than redefine
it, and (iii) `examples/gallery/` + `examples/gallery-rust/` are
seeded with the Box sub-screen as the visible proof. Per A11, all
sides advance together by phase close.

### Layering note (DD-001 ⇄ DD-005)

The two DDs that govern Box's size and child layout are **layered**,
not co-equal. DD-005 settles Box's **outer / resolved bounds** (the
rectangle Box occupies in its parent's coordinate space). DD-001
then settles **what happens inside those bounds** (child measure,
alignment, overflow). The dependency direction is fixed:

- DD-005 resolves Box's outer bounds **without** considering child
  intrinsic size **when `aspect` is set**. Aspect-derived bounds win;
  children do not get to grow the aspect-fixed Box. Child intrinsic
  size participates in DD-005 only when `aspect` is absent, or as
  the explicitly chosen fallback for the both-axis unbounded edge
  case.
- DD-001 receives Box's resolved outer bounds as input and decides
  child measure / alignment / overflow inside them. The phase
  contract is **child clipped or aligned inside the aspect-fixed
  bounds, never extending them**.

Concrete consequence: the following DD-001 × DD-005 combinations are
**invalid** and do not appear as recommended options —

- DD-005 = "aspect set; child intrinsic size grows the Box" with any
  DD-001 alignment / clip option (would contradict the layering;
  Phase 2 does not admit a stretch-Box-to-fit-child variant).
- DD-001 = "child measure overrides Box outer bounds" with any
  DD-005 option (same contradiction from the inside).

The Option tables below cite this layering in each DD's
Recommendation prose so reviewers can verify Option respect for the
dependency direction.

---

## Out of scope (for M3-Phase 2; recorded explicitly)

- **Image widget surface, asset pipeline, icon font, image decoder.**
  M4 or later
  ([m3-plan.md §Out of scope](../../plan.md#out-of-scope-deferred-to-later-milestones),
  [m3-target-app-predoc.md — 保留 2 closure](../../requirements/spec.md#保留-2-closure-image-widget-surface-の-m3-開封可否--不開封-m4-へ-defer)).
  Phase 2 ships the structural bridge (DD-006); the Image widget
  itself ships when M4+ commits to it.
- **Button content other than text** (e.g. Image inside Button).
  M4 or later (tied to the Image-widget deferral).
- **ZStack overlay primitive and multi-child overlap semantics.**
  Phase 6. DD-001 Option A's single-child-only Box is the
  structural defence against pre-empting ZStack's contract.
- **`TypedValue` generic value union.** F5 deferral maintained
  ([m3-start-framing.md §F5](../../requirements/framing.md);
  [m2-to-m3-handover.md §4](../../../milestone-2/handoff.md)).
  DD-004's both-attributes-constant-only stance preserves the
  deferral structurally; the per-type writer seam pattern remains
  available for the phase that first opens a new bindable type.
- **`bool` string-interpolation surface** and any generic
  display-conversion surface. Phase 6+ formatting work
  ([predoc-inputs.md §8](../requirements/constraints.md#8-bool-の-display-conversion-は明示-surface-ができるまで禁止)).
  Phase 2 introduces no formatting surface; the rule from
  Phase 1's T14 (no implicit `bool` → string) continues without
  Phase 2 action.
- **Synchronous non-batched drain proof contract.** Cross-phase
  reactive premise carried in
  [m2-to-m3-handover.md §3 item 4](../../../milestone-2/handoff.md).
  Box introduces no event / input batching, no layout scheduling,
  and no headless proof boundary changes; Phase 2 does not alter
  this contract
  ([predoc-inputs.md §9](../requirements/constraints.md#9-bool-live-proof-は現行の同期-non-batched-drain-に依存している)
  is a back-pointer).
- **Cycle detection / ordering ties / `MUTATION_CAP` × fan-out
  residuals.**
  [m2-to-m3-handover.md §3 items 1–3](../../../milestone-2/handoff.md).
  Phase 6/7 work — Phase 2 does not exercise the reactive engine
  beyond the constant-load path.
- **Scrim alpha styling, theme system, multi-color named palette
  resolution.** M4+ (per
  [m3-target-app-predoc.md Out-of-scope §Visual / styling](../../requirements/spec.md)).
  DD-003's alpha-yes decision is at the *value-type* layer; the
  *styling* layer (theme palette, dynamic alpha control) remains
  M4+ work.
- **Bindable surface for `aspect` and `fill`.** Constant-only in
  Phase 2 per DD-004 Option A. The first phase that exercises
  bindable aspect or fill opens the per-type writer seam triple
  for that attribute.
- **Per-child `align: ...` attribute under Box** and any other
  child-positioning attribute beyond "centred". DD-001 Option A
  (alignment) commits to centred-by-default with no override;
  later phases that need other alignments open their own DD.
- **`f32` / `f64` numeric scalar in `IrType`.** Deferred per
  DD-002 Option A (rational-pair aspect) closing the float surface
  for Phase 2.
  [predoc-inputs.md §7](../requirements/constraints.md#7-f32--f64-を-irtype-に入れるかの再評価)'s
  default of "do not add" stands.
- **C / Zig host parity for the Box sub-screen.**
  [m3-plan.md §Phase-end criteria item 5](../../plan.md#phase-end-criteria)
  calls for at least one host per phase; Phase 8 broadens the full
  gallery to all three. Phase 2 seeds `examples/gallery-rust/`
  only.

## Owner-agreement checkpoints

Two of the DDs above are load-bearing value judgements that warrant
explicit yes/no from the owner before this ADR moves to Accepted.
All other DDs follow mechanically from these two.

### Checkpoint 1 — DD-M3-P2-001 multi-child semantics

**Question:** Is Box single-child-only in Phase 2 (Option A,
recommended), or does Phase 2 admit N children with shared bounds
and no z-order declared (Option B)?

**Default answer:** Option A — single-child-only; 2+ children
rejected at both `wasamoc check` and `ir_loader::build_node`
(defense in depth).

**Framing for owner:** The recommendation narrows A6's "0+ child
container" surface wording at the spec level — readers see "Box
admits 0 or 1 child in M3 Phase 2; multi-child overlap belongs in
ZStack (Phase 6)." This is a public surface narrowing recorded in
`docs/dsl_spec.md` and visible on the Phase 2 spec marker.

The trade-off:

- Option A keeps Phase 2's contract narrow and gives Phase 6 ZStack
  full latitude to define z-order and multi-child overlap without
  inheriting an implicit Box contract. The two Phase 2 use cases
  (0-child scrim and 1-child placeholder) both fit. The diagnostic
  message points users at ZStack / VStack / HStack for multi-child
  needs.
- Option B honours A6's "0+" literally, at the cost of "implementation
  defined" overlap semantics that Phase 6 either ratifies (silently
  set by Phase 2) or contradicts (breaks Phase 2's proof). The
  framing's load-bearing sub-issue is this risk.

If the owner prefers A6's literal wording preserved, Option B is
acceptable but requires Phase 6's ADR to commit affirmatively on
ZStack-vs-Box layering; the Phase 2 marker would record the
"implementation defined" overlap as a known fold-forward to Phase 6.

### Checkpoint 2 — DD-M3-P2-003 alpha decision

**Question:** Does the `fill` value type carry alpha in Phase 2
(Option A, recommended), or is the M3 scrim opaque-by-spec
(Option B)?

**Default answer:** Option A — alpha-yes; new `IrLiteral::Color` plus
a Box-internal `Color(u32)` domain type (stored on `WidgetData::Box`)
carry four 8-bit channels; surface admits `#RRGGBB` (alpha 0xFF
implied) and `#RRGGBBAA`. `PropertyValue::Color` and
`WASAMO_VALUE_COLOR` are **not** added in Phase 2 — see DD-003
variant-strategy Option A for the boundary.

**Framing for owner:** A6 explicitly names "scrim use" as the
motivating use case for `fill`. A scrim is semantically a
semi-transparent overlay; an opaque "scrim" is not a scrim. The
m3-target-app-predoc wording "scrim の alpha 値 styling は M3
では扱わない" but "不透明 fill で代替する" is internally
inconsistent without Option A.

Option A's positioning: *the value type carries alpha; the M3
styling layer does not gain alpha-styling controls beyond the
literal hex form*. Theming, palette, and dynamic alpha adjustment
all remain M4+ work. M3 authors can write `fill: #00000080` for
a half-opaque black scrim today; what they *cannot* do is bind
that alpha to a state variable (DD-004 says `fill` is
constant-only in Phase 2) or pull the color from a theme palette
(M4+ work).

Option B's positioning: the value layer matches the
styling-layer constraint — both exclude alpha. M3 scrims are
opaque, and the target-app pre-doc wording is internally consistent
only if M3's "scrim" is read as "an opaque background panel". M4+
alpha-styling adoption then forces a value-layer revision (the
`Color` domain type widened from 3 to 4 channels), mechanically
additive but a revision Option A avoids.

The decision is design-quality dominated: Option A makes the M3
spec self-consistent at the cost of one extra channel; Option B
keeps the value type tighter at the cost of an internally-tense
"opaque scrim" wording in the spec.

---

## Summary of decisions

| ID | Topic | Recommendation |
|---|---|---|
| DD-M3-P2-001 | Box IR node form + child-layout contract | Option A across sub-issues — per-kind tag `WidgetKind::Box`; 0-child valid; **single-child-only with 2+ rejected at both `wasamoc check` and `ir_loader::build_node` (defense in depth)**; child measure passes Box bounds through unchanged; child alignment centred (no per-child override); overflow clips child to Box bounds |
| DD-M3-P2-002 | `aspect: <ratio>` value type | Option A — rational pair `aspect: <num>:<den>`; new `IrLiteral::Ratio { num, den }` + Box-internal `Ratio` domain type on `WidgetData::Box`; **no** `PropertyValue::Ratio`, **no** `WASAMO_VALUE_RATIO` tag, **no** `abi.rs` arms in Phase 2 (constant-only per DD-004); no new `IrType` |
| DD-M3-P2-003 | `fill: <color>` value type | Option A — alpha-yes; new `IrLiteral::Color(u32)` + Box-internal `Color(u32)` domain type on `WidgetData::Box`; **no** `PropertyValue::Color`, **no** `WASAMO_VALUE_COLOR` tag, **no** `abi.rs` arms in Phase 2 (constant-only per DD-004); surface forms `#RRGGBB` and `#RRGGBBAA` only |
| DD-M3-P2-004 | Bindable surface for `aspect` / `fill` | Option A — both attributes **constant-only** in Phase 2; no new per-type writer seam built; F5 deferral structurally preserved |
| DD-M3-P2-005 | Aspect measure-arrange algorithm | Option A across sub-issues — inscribed fit (bounded parent) with **integer branch selection + `f32` derived axis, no pixel-snapping in Phase 2**; bounded-axis-wins (unbounded on one axis); layout-time runtime error (unbounded on both axes) **applied symmetrically to the no-aspect empty-Box case**; explicit width/height wins + warning (conflict) — **forward-looking: `width` / `height` are not in the M3-Phase 2 DSL surface, so this rule lands as spec text only in Phase 2**; child intrinsic shrink-to-fit + parent-bounds fallback (no aspect); `wasamoc check` rejects non-positive sides |
| DD-M3-P2-006 | Placeholder pattern (Box + Text) | Option A — normative spec convention in `docs/dsl_spec.md` Box chapter; Phase 3 / Phase 6 cite it; M4 Image-widget ADR supersedes it cleanly |

Implementation task list: belongs in the Phase 2 progress file
`docs/plans/progress/m3-phase-2-progress.md` (created when this ADR
is Accepted and Phase 2 starts execution); not in this ADR and not
in `m3-plan.md` itself. See
[plans/README.md §Scope rule (plan vs ADR)](../../../README.md#scope-rule-plan-vs-adr)
and [plans/README.md §Phase progress file lifecycle](../../../README.md#phase-progress-file-lifecycle)
for the authoritative location and the `active → closing → retired
→ archived` lifecycle the file follows. The Progress table in
[m3-plan.md](../../plan.md) carries only a one-row index entry
pointing at this progress file.

## Spec impact preview (for owner agreement)

When this ADR is Accepted, the following docs change in the
**Moment 1** commit set (framing decision D — ADR-Accepted /
design-spec draft):

- [docs/dsl_spec.md](../../../../docs/dsl_spec.md) — extensions in three regions:
  - **DSL surface** — new Box chapter under the widget catalog
    documenting the IR node, attributes (`aspect`, `fill`), child-
    layout contract (single-child, centred, clipped), and the
    image-placeholder pattern subsection (DD-006). Section marker
    `**Phase status:** M3-Phase 2 ADR-accepted design draft; pending
    implementation re-sync` at the chapter top.
  - **DSL surface lexer / literal grammar** — `aspect: <num>:<den>`
    ratio literal; `fill: #RRGGBB` and `fill: #RRGGBBAA` color
    literals.
  - **IR text grammar** (§8) — `IrLiteral::Ratio` and
    `IrLiteral::Color` productions.
- [docs/architecture.md](../../../../docs/architecture.md) §6 — Box entry under
  the M2-revised IR section if structural placement warrants;
  short paragraph noting the per-type binding seam is *not*
  extended by Phase 2 (`aspect` / `fill` constant-only) so the F5
  deferral is unpressured, **and** noting that `Ratio` / `Color`
  enter the runtime as Box-internal domain types only — not as
  `PropertyValue` variants — so the ABI surface remains unchanged
  through Phase 2.
- [docs/abi_spec.md](../../../../docs/abi_spec.md) — **no changes in Phase 2**.
  No new ABI public function, no new `WASAMO_VALUE_*` tag, no new
  arms in `abi.rs` (`read_property_value` / `write_property_value` /
  `property_value_to_owned`). The new `Ratio` and `Color` types are
  Box-internal domain types stored on `WidgetData::Box`; they do
  **not** become `PropertyValue` variants in Phase 2 (per DD-002 IR /
  runtime plumbing block and DD-003 variant-strategy Option A), so
  the C ABI boundary is untouched. This is consistent with DD-004's
  constant-only stance: with no bindable surface and no
  `get_property` / `set_property` / observer payload exposure, the
  values never reach the C ABI. When a later phase opens bindable
  `fill` or `aspect`, the ABI extensions
  (`PropertyValue::Color(Color)` / `PropertyValue::Ratio(Ratio)`
  variants, `WASAMO_VALUE_COLOR` / `WASAMO_VALUE_RATIO` tags, and the
  corresponding `abi.rs` arms) land together in that phase per
  [predoc-inputs.md §1](../requirements/constraints.md#1-box-が新規-propertyvalue-variant-を入れるなら-abi-value-conversion-arm-は同じ-step-に-fold-する).
- [docs/plans/m3-plan.md](../../plan.md) — Progress section's
  Phase 2 row populated (Status: `in progress`; ADR link; progress
  file link).
- [docs/notes/retrospectives.md](../../../procedures/retrospectives.md) —
  per framing decision E (a), the step-retrospective checklist's
  item 3 (clean rebuild) is amended to require `cargo fmt --all --
  --check` against the post-commit state explicitly, with "green"
  interpreted as the `--check` form. CI YAML (framing decision E
  (b)) is **not** updated in Phase 2 — deferred per CLAUDE.md §CI
  rules.

The **Moment 2** commit set (framing decision D — Phase close /
implementation re-sync) lands at phase end; the Box-chapter spec
marker flips to
`**Phase status:** M3-Phase 2 closed; implementation-synced`, any
divergence between the design-spec draft and the implementation is
corrected in the same commit, and earlier-phase spec gaps surfaced
during re-sync may fold per
[predoc-inputs.md §6](../requirements/constraints.md#6-retroactive-spec-gap-fold-は最小範囲で同じ-phase-に折り込む)
with owner confirmation. The Phase 2 progress file is retired in
the same commit per the standard `active → closing → retired →
archived` lifecycle.

No ROADMAP revision is anticipated — A6 is already explicit, this
ADR operationalises it.

## Phase 2 verification closure (what counts as A6 evidence)

This section is not a DD — it records the agreed shape of the
proof that closes Phase 2 per framing decision C, so the
implementation plan inherits a concrete target rather than
re-litigating "what does Box's verification mean here?".

A6 (Box layout primitive + image-placeholder pattern) is considered
satisfied when **all four** of the following are observed:

1. **Unit-test evidence (host-independent).** Pure-logic tests in
   `wasamoc` (parse + check + lower) and in `wasamo-runtime`
   non-Windows-bound modules (aspect measure-arrange resolver,
   IR-loader handling of `IrLiteral::Ratio` / `IrLiteral::Color`,
   `Ratio` / `Color` Box-internal domain-type plumbing) cover:
   ratio literal parsing; color literal parsing (both `#RRGGBB` and
   `#RRGGBBAA`); DD-005 measure-arrange edge cases (bounded
   parent inscribed fit; unbounded-on-one-axis bounded-axis-wins;
   unbounded-on-both-axes layout-time runtime error; explicit width/height
   conflict; child intrinsic shrink-to-fit when aspect absent;
   non-positive ratio sides rejected); `wasamoc check` diagnostics
   for `bind aspect: ...` and `bind fill: ...` (rejected per DD-004)
   and for 2+ children under Box at compile time **and** at IR-load
   time in `ir_loader::build_node` (both rejected per DD-001). These
   run on any CI runner.

2. **IR text round-trip evidence.** For the fixture
   `Box { aspect: 16:9; fill: #00000080; Text { text: "Photo 12" } }`,
   two separate checks gate the IR ↔ runtime boundary:

   - **Emitted IR (`wasamoc` side).** An in-process test inspects the
     `wasamoc`-emitted IR text and asserts the Box node carries
     `IrLiteral::Ratio { num: 16, den: 9 }` and
     `IrLiteral::Color(<packed>)`.
   - **Loaded runtime state (`wasamo-runtime` side).** `wasamo-runtime`
     loads that IR through `ir_loader::build_node`, which **materialises**
     the literals into the Box-internal domain types — the resulting
     `WidgetData::Box { aspect: Some(Ratio { num: 16, den: 9 }),
     fill: Some(Color(<packed>)), .. }` is asserted directly. The
     `IrLiteral::*` variants do **not** survive into runtime state;
     they are the IR-text-side encoding only.

   Tests the DD-001 / 002 / 003 / 006 surfaces together and makes the
   "IR-side `IrLiteral::*`, runtime-side Box-internal domain type"
   boundary (DD-002 / DD-003 variant-strategy Option A) executable.

3. **Windows-runtime layout evidence (CI-gated).** A mock-free
   integration test (per CLAUDE.md "Testing rules") on the Windows
   CI runner: a `.ui` fixture declares an aspect-fixed Box with a
   Text child inside a parent of known size. The test loads the
   IR, runs the layout pass, and asserts the Box's resolved
   rectangle matches the inscribed-fit calculation and that the
   Text child is centred. The Box's `fill` color is verified
   either through an in-process / test-only accessor that exposes
   the Box-internal `Color` value, or via the layout / render model
   (`SpriteVisual` brush color) the layout pass emits — **not**
   through `wasamo_get_property` or any `PropertyValue`-mediated
   path, because `fill` is not a `PropertyValue` variant in Phase 2
   (per DD-003 variant-strategy Option A). Fails (not skips) on a
   runner that cannot create the Compositor — the test gates A6
   evidence in CI, not local convenience.

4. **End-to-end host evidence (visible smoke).**
   `examples/gallery/gallery.ui` is seeded with the Phase 2
   sub-screen (a Box + Text placeholder against a trivial frame),
   and `examples/gallery-rust/` (newly created) is the
   workspace-member host that builds and runs it. `Start-Process`
   launch is recorded as successful by the assistant; visual
   correctness of `aspect: <ratio>` (rendered rectangle has the
   right ratio) and `fill: <color>` (rendered rectangle is the
   right color, including alpha) is **owner-manual GUI smoke** per
   framing decision G — the assistant does not assert on pixel- or
   eyeball-level correctness.

Items (1)–(3) are required for A6 acceptance; item (4) ties the
evidence back to the m3-plan target-app trajectory and seeds the
gallery directory that every subsequent M3 phase grows. C and
Zig hosts for the Box sub-screen are explicitly **not** required
in Phase 2 (per framing decision F and the Out of scope list);
Phase 8 broadens the full gallery to all three.

The acceptance / non-acceptance of test items (1)–(4) is the
operational form of "Phase 2 done"; the corresponding
implementation checklist (which crate / which test file / which
fixture) belongs in the Phase 2 progress file, not here.
