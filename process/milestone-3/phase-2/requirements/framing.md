# M3-Phase 2 pre-doc framing

**Status:** framing aligned with owner (2026-05-20); input artefact for ADR drafting
**Date:** 2026-05-20
**Targets phase:** M3-Phase 2 (Box layout primitive)

Per the project's doc-driven workflow established at
[M2-Phase 6 pre-doc framing](../../../milestone-2/phase-6/requirements/framing.md),
individual DDs are not negotiated one-by-one in chat — instead, framing
is aligned first, then the full ADR is drafted in one pass as
`Status: Proposed`, reviewed, and flipped to `Status: Accepted`. This
note records the framing agreement reached with the owner before ADR
drafting begins; it remains as an input artefact and is not promoted
into the ADR.

---

## Phase 2 acceptance criteria (restated)

- **A6** (see [process/_roadmap.md M3](../../../_roadmap.md#m3-dsl-surface),
  [m3-plan.md §Acceptance criteria](../../plan.md#acceptance-criteria)):

  > Box layout primitive (0+ child container; `aspect: <ratio>`
  > attribute subsumes a standalone AspectRatio; minimal
  > `fill: <color>` attribute for scrim use). Image-widget deferral
  > is carried by Box + Text-child placeholders.

- **A11 (operational obligation).** `.ui`, `wasamo-ir`, `wasamoc`,
  `wasamo-runtime`, `docs/dsl_spec.md`, and a sub-screen of
  `examples/gallery/` all advance within Phase 2. No side is left
  ahead of the others at phase close. Per
  [m3-plan.md §Phase-end criteria item 5](../../plan.md#phase-end-criteria),
  the foundational-phase exception is scoped to Phase 1 only —
  Phase 2 is the first phase that seeds `examples/gallery/`, and
  every subsequent phase grows it. Framing decision F operationalises
  this; the sibling-example substitute is not available to Phase 2.

- **Downstream commitments grounded in Phase 2.** The Box + Text
  placeholder pattern (DD-006) is the structural carrier of the M3
  Image-widget deferral; Phase 3 (WrapPanel) and Phase 6 (ZStack) reuse
  Boxes as their child shape. Phase 2 must therefore settle Box's
  attribute surface in a form Phase 3+ can build on, not only in a form
  that makes Phase 2's own example run.

---

## Agreed DD slate (6 entries proposed)

The Phase 2 ADR (working title
`process/milestone-3/phase-2/decisions/preamble.md`) will carry the following six
DDs.

**Layering note for DD-001 and DD-005.** The two DDs that govern
Box's size and child layout are layered, not co-equal. DD-005
settles **Box's own outer / resolved bounds** (the rectangle Box
occupies in its parent's coordinate space). DD-001 then settles
**what happens inside those bounds** (child measure, alignment,
overflow). The dependency direction is fixed by framing:

- DD-005 resolves Box's outer bounds **without** considering child
  intrinsic size **when `aspect` is set**. Aspect-derived bounds
  win; children do not get to grow the aspect-fixed Box. Child
  intrinsic size may participate in DD-005 only when `aspect` is
  absent, or as the explicitly chosen fallback for the both-axis
  unbounded edge case.
- DD-001 receives Box's resolved outer bounds as input and decides
  child measure / alignment / overflow inside them. The framing's
  working assumption is **child clipped or aligned inside the
  aspect-fixed bounds**, never extending them.

Concrete consequence for the ADR's Options tables: the following
DD-001 × DD-005 combinations are **invalid** and should not appear
as recommended cells —

- DD-005 = "aspect set; child intrinsic size grows the Box" with any
  DD-001 alignment / clip option (contradicts the layering — Phase 2
  does not admit a stretch-Box-to-fit-child variant).
- DD-001 = "child measure overrides Box outer bounds" with any
  DD-005 option (same contradiction from the other side).

The ADR's Option tables for both DDs should cite the layering note
in their Recommendation prose so the reviewer can verify each Option
respects the dependency direction.

### DD-M3-P2-001 — Box IR node form and 0+ child layout semantics

Box is a new layout primitive in `wasamo-ir` and `wasamo-runtime`.
Phase 2 must commit to the IR shape and the full child-layout
contract. The latter has more degrees of freedom than the framing's
first draft acknowledged; each one has visible consequences for
Phase 3+ that cannot be deferred without leaving Box's contract
under-specified.

Sub-issues:

- **IR node shape.** Per-kind tag parallel to `HStack` / `VStack` /
  `Rectangle`, vs a structural variant in `IrLayout`.
- **0-child shape.** A Box with no children but with `aspect` and/or
  `fill` must still produce a visual rectangle. This is the
  placeholder-shape minimum and the structural support for DD-006.
- **Child measure pass.** Given Box's resolved outer bounds (from
  DD-005), how does a Box child receive its measure constraint —
  Box bounds passed through unchanged, the child's intrinsic size
  capped at Box bounds, or a percentage / fill mode? Per the
  layering note, the child does **not** influence Box's outer
  bounds when `aspect` is set; this sub-issue is strictly about
  how the inner constraint is shaped.
- **Child alignment within Box bounds.** When a child is smaller than
  Box (e.g. a Text placeholder shorter than the aspect-derived
  rectangle), where does the child land — top-left, center, or
  configurable per-child? The default determines what `Box { aspect:
  1:1; Text { ... } }` looks like in Phase 3's WrapPanel of
  thumbnails and Phase 6's lightbox; ad-hoc per-phase choices would
  drift the visual contract.
- **Overflow / clip.** When a child measures larger than Box bounds
  (e.g. a long Text in a small aspect-fixed Box), does Box clip
  the child or overflow visibly? "Grow Box to fit child" is **not
  an admissible option** per the layering note — it would void the
  aspect guarantee. The framing's working assumption is **clip**
  (consistent with M4 ScrollView's separate clip surface). Visible
  overflow remains a candidate for the DD's Options table.
- **Multi-child semantics and orthogonality to ZStack.** This is
  the load-bearing sub-issue. Box's N-child layout must **not** be a
  back-door ZStack — overlay is A4 / Phase 6's responsibility.
  Candidate shapes include (a) single-child-only with multi-child
  rejected at IR-load time, (b) all-children share full Box bounds
  but with no z-order semantics declared (Phase 6 ZStack adds those
  semantics on top), (c) document-order top-left stacking with each
  child consuming the next available space. The choice has follow-on
  consequences for Phase 6's ZStack definition: if Box already
  stacks visually, ZStack's primitive contribution narrows to z-order
  declaration, not the visual stacking itself.

**Inputs consumed.** [m2-to-m3-handover.md §1](../../../milestone-2/handoff.md)
(new IR form = grammar production + `wasamo-ir` variant + loader
wiring triple);
[m3-target-app-predoc.md "必要 surface" Box row](../../requirements/spec.md#layout-primitive)
and the Grid / ZStack 責務境界 paragraph (Box's orthogonality
extension of the same principle).

### DD-M3-P2-002 — `aspect: <ratio>` value-type representation

Box's `aspect` attribute requires a new value type threaded through
`wasamoc` / `wasamo-ir` / `wasamo-runtime`. Sub-issues:

- **Numeric form.** float (`aspect: 1.7777`), rational pair
  (`aspect: 16:9`), compile-time-parsed string (`aspect: "16:9"`), or
  twin i32 attributes (`aspect-width: 16`, `aspect-height: 9`).
- **`f32`/`f64` re-evaluation.**
  [predoc-inputs.md §7](constraints.md#7-f32--f64-を-irtype-に入れるかの再評価)
  applies directly. If float is chosen, `IrType::F32` / `IrLiteral::F32` /
  `HandlerExpr::F32Lit` extend the Phase 1 type-suffix pattern.
- **ABI value-conversion arms.**
  [predoc-inputs.md §1](constraints.md#1-box-が新規-propertyvalue-variant-を入れるなら-abi-value-conversion-arm-は同じ-step-に-fold-する)
  mandates folding any new `PropertyValue` variant's exhaustive-match
  ABI arms into the same step.
- **`HandlerExpr` extension.** Per
  [m2-to-m3-handover.md §2](../../../milestone-2/handoff.md), the new literal
  goes into the single unified enum, type-suffixed (`F32Lit` /
  `RatioLit` per chosen option).
- **IR text grammar wording** in `docs/dsl_spec.md`.

### DD-M3-P2-003 — `fill: <color>` value-type representation

The `fill` attribute requires a color representation. The central
question is **alpha**, surfaced ahead of the variant-strategy
question because alpha drives the variant choice rather than being
downstream of it.

- **Alpha availability (central question).** A6 explicitly names
  "scrim use" as the motivating use case for `fill`. A scrim is
  semantically a semi-transparent overlay — without alpha, the
  `fill` value cannot express what A6 calls for, and the
  m3-target-app-predoc Out-of-scope wording ("scrim の alpha 値
  styling は M3 では扱わない" but "不透明 fill で代替する") becomes
  internally inconsistent (an opaque scrim is not a scrim). Phase 2
  must therefore decide whether the value type carries alpha (and
  the styling layer separately decides whether to expose alpha
  control), or whether `fill` is intentionally alpha-less and the
  M3 scrim is opaque-by-spec. The former is the framing's working
  recommendation; the DD owns the decision.
- **Variant strategy (derived from alpha decision).** If alpha is
  in the value type, candidates are new `PropertyValue::Color` /
  `IrType::Color` / `IrLiteral::Color` carrying four channels, or
  reuse `PropertyValue::Str` with `#RRGGBBAA` parsing at the loader
  (soft path). If alpha is excluded, the same options apply with
  three channels.
- **Surface form.** `#RRGGBB`, `#RRGGBBAA`, named colors, or a
  combination. The chosen forms determine the lexer / parser
  extension in `wasamoc`.
- **Forward compatibility** with the future theming surface
  (M4/M5 — m3-plan §Out of scope). A `PropertyValue::Color` with
  alpha is forward-compatible with theme bindings producing
  semi-transparent values; a Str-with-parse path forces theming to
  redesign value plumbing later.
- ABI value-conversion arms per
  [predoc-inputs.md §1](constraints.md#1-box-が新規-propertyvalue-variant-を入れるなら-abi-value-conversion-arm-は同じ-step-に-fold-する),
  symmetrically with DD-002.

### DD-M3-P2-004 — Bindable surface for `aspect` and `fill`

Whether `aspect` and `fill` can each be driven by a reactive Signal at
runtime, or are constant at load time. The two attributes decide
independently.

Sub-issues:

- **Per-type writer seam.** Per
  [predoc-inputs.md §2](constraints.md#2-新しい-bindable-property-は-per-type-writer-seam-を-ir_loader-call-site-で選ぶ)
  and the Phase 1 DD-M3-P1-007 precedent, any bindable new type adds
  an `evaluate_<T>_binding` + `widget_write_property_<T>` +
  `register_<T>_binding` triple selected at the
  `ir_loader::build_node` call site. The reactive engine itself
  remains type-agnostic.
- **F5 (`TypedValue` deferral) preservation.** Per
  [m2-to-m3-handover.md §4](../../../milestone-2/handoff.md) and
  [m3-start-framing.md §F5](../../requirements/framing.md), Phase 2 must
  not be the phase that pressures `TypedValue` adoption. The seam
  pattern structurally protects this.
- **Gallery use case driver.** Does any wireframe surface in
  [docs/references/m3-gallery-wireframe.html](../../requirements/gallery-wireframe.html)
  require Box's aspect or fill to vary reactively (e.g. lightbox
  theme, animated thumbnail size)? The answer shapes the
  cost/benefit of bindable-vs-constant.

### DD-M3-P2-005 — Aspect constraint measure-arrange algorithm

When Box carries `aspect`, the measure-arrange pass must compute a
resolved width × height from parent bounds. The algorithm has more
edge cases than the "pure primitive" framing initially suggested;
leaving any of them under-specified means WrapPanel (Phase 3) and
ScrollView (Phase 4) inherit Box's ambiguity at the worst possible
time. Sub-issues:

- **Bounded parent (happy path).** Fit aspect-constrained Box inside
  parent bounds: inscribed (fit the smaller), circumscribed (overflow
  the larger), or major-axis driven.
- **Unbounded parent on one axis.** When the parent provides no
  constraint on one axis (an intrinsic-sizing context, e.g. inside a
  WrapPanel or ScrollView later in M3), the Box derives the
  unbounded axis from the bounded axis × aspect. Spec must state
  the bounded-axis-wins direction explicitly.
- **Unbounded parent on both axes.** When both axes are unbounded
  (e.g. a top-level Box with no host-provided window size, or
  pathological nesting), Box has no anchor for aspect resolution.
  Options: (a) fall back to a spec-defined intrinsic size (e.g. 0×0
  or a sentinel), (b) load-time error, (c) take the child's
  intrinsic size as the bounded axis if children exist. Phase 2
  must pick one; silently producing zero or NaN is rejected.
- **Conflict with explicit width/height.** If Box carries explicit
  dimensions alongside `aspect`, how do they interact — explicit
  overrides aspect, aspect overrides explicit, error, or treated as
  a soft preference? The framing's working assumption is **explicit
  dimensions win and aspect becomes informational**; the DD owns it.
- **Child intrinsic size participation (gated by the layering note).**
  When `aspect` is **set**, child intrinsic size does not participate
  in Box's outer-bounds resolution — the aspect-derived bounds win and
  oversized children are handled at the DD-001 inner layer (clip or
  visible overflow per its overflow / clip decision). When `aspect`
  is **absent**, child intrinsic size may participate; Phase 2 must
  pick: Box shrink-to-fit the child's union intrinsic size, expand
  to parent bounds, or a spec-defined default. The third use of
  child intrinsic size — as the explicitly chosen fallback for the
  both-axis-unbounded edge case above — is also gated here.
- **Aspect value validity at runtime.** What happens when aspect
  resolves to ≤ 0, NaN, infinity, or (in the rational-pair form) a
  zero denominator? Options: compile-time rejection in `wasamoc check`
  (preferred where feasible), load-time rejection in `ir_loader`,
  or runtime fallback to a spec-defined default. The decision
  interacts with DD-002's value-type choice — a float form makes
  NaN / infinity reachable in a way a rational pair does not.
- **Parse failure handling.** If aspect's surface form
  (e.g. `aspect: "16:9"` string-parse path per DD-002 Option C)
  fails to parse, this is a `wasamoc check` diagnostic, not a
  runtime fallback. Symmetric with Phase 1's
  `bool_state_in_string_interp_rejected` discipline (T14): bad
  surface forms fail at the source-level diagnostic gate.
- **Spec wording** in `docs/dsl_spec.md`.
  [m3-plan.md §Phase breakdown](../../plan.md#phase-breakdown)
  describes Phase 2 as "pure primitive — no novel measure-arrange
  algorithm". This DD nuances that claim: "no novel" refers to the
  absence of a new measure-arrange paradigm (vs WrapPanel's two-stage
  reflow in Phase 3), not the absence of any algorithmic spec content
  — the edge-case enumeration above is a non-trivial spec contribution.

### DD-M3-P2-006 — Placeholder pattern (Box + Text child) canonicalization

[m3-target-app-predoc.md 保留 2 closure](../../requirements/spec.md#保留-2-closure-image-widget-surface-の-m3-開封可否--不開封-m4-へ-defer)
establishes that Box + Text-child substitutes for Image during M3.
Phase 2 settles how this pattern is canonicalized.

Sub-issues:

- **Spec wording.** Normative convention (e.g.
  `Box { aspect: 1:1; fill: #ccc; Text { text: "Photo 12" } }`),
  informal pattern noted but not normative, or a helper widget alias.
- **Example placement.** How `examples/box-demo-rust/` (per framing
  decision F) demonstrates the placeholder.
- **Downstream phase usage.** Phase 3 (WrapPanel of thumbnails) and
  Phase 6 (ZStack lightbox photo) consume this pattern. The Phase 2
  spec should be sufficient for Phase 3 to point at it rather than
  redefine it.

---

### Out of scope (to be carried in the ADR's Out-of-scope section)

- Image widget surface, asset pipeline, icon font, image decoder
  (M4 or later;
  [m3-plan.md §Out of scope](../../plan.md#out-of-scope-deferred-to-later-milestones),
  [m3-target-app-predoc.md 保留 2 closure](../../requirements/spec.md#保留-2-closure-image-widget-surface-の-m3-開封可否--不開封-m4-へ-defer)).
- Button content other than text (M4 or later).
- ZStack overlay primitive (Phase 6).
- `TypedValue` generic value union
  ([m3-start-framing.md §F5](../../requirements/framing.md) maintained;
  [m2-to-m3-handover.md §4](../../../milestone-2/handoff.md)).
- `bool` string-interpolation surface (Phase 6+ formatting work —
  [predoc-inputs.md §8](constraints.md#8-bool-の-display-conversion-は明示-surface-ができるまで禁止)).
- Synchronous non-batched drain proof contract — cross-phase reactive
  premise carried in
  [m2-to-m3-handover.md §3 item 4](../../../milestone-2/handoff.md). Box does
  not introduce batching, so Phase 2 does not alter this contract;
  [predoc-inputs.md §9](constraints.md#9-bool-live-proof-は現行の同期-non-batched-drain-に依存している)
  is a back-pointer.
- Cycle detection / ordering ties / `MUTATION_CAP` × fan-out residuals
  ([m2-to-m3-handover.md §3 items 1–3](../../../milestone-2/handoff.md)) —
  Phase 6/7 work.
- Scrim alpha styling, theme system, multi-color named-palette resolution
  ([m3-target-app-predoc.md Out-of-scope §Visual / styling](../../requirements/spec.md#visual--styling)).

---

## Owner-agreed framing decisions

### A. DD slate completeness

The 6 DDs above are proposed as the cut.
[predoc-inputs.md](constraints.md) §1 / §2 / §6 / §7 are absorbed
as sub-issues within these DDs (see "Inputs absorbed" mapping below).
§3 is lifted to framing decision E; §4 to F; §5 to G. §8 and §9 are
Out of scope.

### B. Pre-doc-discipline check

Per [process/README.md §Pre-doc discipline](../../../README.md),
the framing must verify that the proposed DD slate serves A6, not
merely execute the m3-plan task description literally. Check:

- A6 enumerates Box-as-0+-child-container, `aspect: <ratio>`,
  `fill: <color>` (minimal), and Box + Text placeholder. The 6 DDs
  map directly to (i) container shape and child semantics (DD-001),
  (ii) aspect representation (DD-002), (iii) fill representation
  (DD-003), (iv) bindable surface (DD-004), (v) measure-arrange spec
  (DD-005), (vi) placeholder canonicalization (DD-006).
- The slate neither drops nor adds material relative to A6. No surface
  is smuggled in.
- The m3-plan §Phase breakdown line "pure primitive — no novel
  measure-arrange algorithm" is acknowledged and nuanced (DD-005),
  not silently overridden.

### C. Verification strategy

Per [m3-plan.md §Verification strategy](../../plan.md#verification-strategy),
Phase 2 chooses from the menu:

- **Pure-logic unit tests** for the aspect measure-arrange resolver
  (DD-005) and IR-loader handling of new value types (DD-002, DD-003).
  Both are decoupled from Compositor and exercise as free functions.
- **Mock-free Windows-only integration test** (CI-gated, fails rather
  than skips per [CLAUDE.md §Testing rules](../../../../CLAUDE.md)) for
  live `.ui → IR → runtime` propagation through Box on a live
  `WidgetNode` — analogous in shape to the Phase 1 T13 test, scoped
  to whichever Box attribute DD-004 makes bindable (if any).
- **Visible smoke** via the Box sub-screen seeded in
  `examples/gallery/` + `examples/gallery-rust/` (framing decision F)
  for owner-manual GUI smoke (framing decision G).

### D. Upstream-document revision timing (two sync moments)

Phase 2 has **two distinct spec-sync moments**, both required by A11
but serving different roles. The M2-Phase 6 precedent of bundling
all upstream edits into the ADR-Accepted commit was appropriate for
Phase 6's structural constraints (VISION P2 supplement inseparable
from DD-001); Phase 2's situation differs because Box's spec
content is implementation-shaped rather than ADR-shaped, and
implementation can legitimately reveal corrections to the design
draft. Splitting the two moments preserves A11's "no side left
ahead" while admitting that implementation findings can refine the
spec text.

**Marker convention.** Both moments use a **section-level marker**
matching the existing `docs/dsl_spec.md` style. The current spec
file uses plain markdown header metadata
(`**Document version:** 0.6` / `**Last updated:**` / `**Status:**`
at the top of the file, not YAML frontmatter), and the Phase 2 spec
section adopts the same form. **Phase 2 does not introduce YAML
frontmatter to `dsl_spec.md`** — a frontmatter convention is a
heavier change than the marker requirement justifies, and if the
project later wants whole-document frontmatter (e.g. for the Phase 8
public draft promotion), that decision belongs to Phase 8, not Phase
2. The Phase 2 marker takes the form:

```
**Phase status:** M3-Phase 2 ADR-accepted design draft; pending
implementation re-sync
```

placed as the first line under the Box chapter heading. The same
line flips at phase close to:

```
**Phase status:** M3-Phase 2 closed; implementation-synced
```

`dsl_spec.md`'s existing document-level `**Status:**` remains the
whole-document state and continues to be edited per existing
discipline; the Phase 2 marker is scoped to the Box chapter only.
The document's existing Revision history (or equivalent) records
Moment 1 and Moment 2 as separate entries.

**Moment 1 — ADR Accepted commit (design-spec draft).**

Bundles the design-level spec content the ADR's accepted DDs
commit to:

- `docs/dsl_spec.md` — Box surface chapter as a **design-spec draft**.
  Describes the surface shape committed by the accepted DDs:
  attribute names, value-type representation (per DD-002 / DD-003),
  bindable surface (per DD-004), measure-arrange algorithm
  (per DD-005), placeholder pattern (per DD-006). The Box-chapter
  section marker (above) is set to "ADR-accepted design draft;
  pending implementation re-sync".
- `docs/architecture.md` §6 — Box entry under the M2-revised IR
  section if structural placement warrants.
- `docs/plans/m3-plan.md` Progress section — Phase 2 row populated.
- `docs/notes/retrospectives.md` step-checklist amendment per
  framing decision E (a).

Implementation begins only after this commit lands, so structural
constraints are review-ready as code-review rulers.

**Moment 2 — Phase close commit (impl re-sync).**

Re-syncs spec text to the implementation's actual surface:

- `docs/dsl_spec.md` — any corrections required because the design
  draft and implementation diverged (per
  [predoc-inputs.md §6](constraints.md#6-retroactive-spec-gap-fold-は最小範囲で同じ-phase-に折り込む)
  retroactive fold discipline). The Box-chapter section marker
  flips to "M3-Phase 2 closed; implementation-synced".
  Earlier-phase (Phase 1 / M2) spec gaps surfaced during this
  re-sync may fold into the same commit with explicit owner
  confirmation, continuing the T10 discipline.
- `docs/plans/progress/m3-phase-2-progress.md` — phase-close
  retrospective, CI evidence pointer, impl summary.

A11's "no side left ahead" is satisfied at Moment 2; Moment 1
documents the intent A11 will hold the impl to, but is not itself
the A11 close gate.

**Postmortem (added 2026-05-20): "Moment = 1 commit" is the wrong unit.**

Moment 1 above was first executed as a single commit on a private
`wip/m3-phase-2-moment1-draft` branch, followed by 4 fixup commits as
owner review surfaced issues across the bundled documents. The final
tree was then re-decomposed into 6 separate commits on
`docs/m3-phase-2-predoc` (Box ADR, m3-plan tracking, architecture
boundary, dsl_spec §4.9 draft, progress file open, retrospectives §3
amendment).

The structural failure was bundling documents with different review
profiles into one commit:

- Owner-pre-approved content (ADR Status flip).
- Normative spec draft requiring multi-round review (dsl_spec §4.9).
- Project-wide process change with broad blast radius (retrospectives §3).
- Mechanical tracking (m3-plan Progress row).
- New 13-task execution list (progress file).

Bundling forced owner review to be all-or-nothing across the bundle.
The natural response — iterating in private fixup commits on a
discardable wip branch — kept review history off the shared pre-doc
branch, defeating the point of the pre-doc branch as a reviewable
ledger.

**Rule for future phases:** Moment (or any analogous bundle construct)
is a milestone label, not a commit unit. Constituent documents land
as separate commits on the pre-doc branch; the commit shape follows
review-concern boundaries, not the Moment boundary. The Moment is
"achieved" when all constituent commits have landed. The general
rule lives in [CLAUDE.md §Commit rules](../../../../CLAUDE.md#commit-rules).

The *list* of which documents belong to Moment 1 / Moment 2 above is
not retracted by this Postmortem — only the commit shape used to
land that list. Moment 2 will be authored under the new rule
(per-review-concern commits at phase close).

### E. `cargo fmt` enforcement strategy (predoc-inputs §3)

The M3-Phase 1 phase-end retrospective surfaced that step-end
"`cargo fmt` — green" notes did not catch drift across step commits.
Two candidate remediations are proposed in
[predoc-inputs.md §3](constraints.md#3-cargo-fmt-process-gap--step-checklist-改訂--ci-強制-のどちらを選ぶか):

- **(a)** Amend the step retrospective checklist in
  [docs/notes/retrospectives.md](../../../procedures/retrospectives.md) item 3 (clean
  rebuild) to require `cargo fmt --all -- --check` against the
  post-commit state explicitly, with "green" interpreted as the
  `--check` form, not just `cargo fmt`'s exit code.
- **(b)** Add `cargo fmt --all -- --check` to
  [.github/workflows/ci.yml](../../../../.github/workflows/ci.yml).

**Recommended treatment:** **(a) only**, deferring (b). Reason: Phase 2
does not introduce a new language or build system, so per
[CLAUDE.md §CI rules](../../../../CLAUDE.md) the CI YAML is off-limits
absent explicit owner agreement; (a) gives immediate prevention while
(b) can be revisited as a standalone process change with its own
agreement. The (a) amendment lands in the same spec-sync commit as
framing decision D, since it is a documentation edit to
`docs/notes/`.

(Open for owner to revise to (a)+(b) if enforcement-level guarantee is
preferred over discipline-level prevention.)

### F. Phase 2 visible proof — seed `examples/gallery/`

The Box-surface visible proof lives at **`examples/gallery/`** (shared
`.ui`) and **`examples/gallery-rust/`** (Rust host), created in
Phase 2. Phase 2 is the first phase that seeds the gallery directory;
every subsequent M3 phase grows it sub-screen by sub-screen until
Phase 8 assembles the full A1 proof.

This commitment honors three converging constraints:

- [m3-plan.md §Phase-end criteria item 5](../../plan.md#phase-end-criteria)
  requires the relevant slice of `examples/gallery/` for every phase
  from Phase 2 onward. The foundational-phase exception was scoped
  to Phase 1 only and is not available to Phase 2. Earlier framing
  drafts of this document attempted an `examples/box-demo-rust/`
  sibling-example substitute; that path would constitute an unrecorded
  re-extension of the Phase 1 exception, not a framing-level decision,
  and is rejected.
- [predoc-inputs.md §4](constraints.md#4-可視-proof-は既存-canonical-example-を太らせず-sibling-example-を立てる)
  forbids extending existing canonical examples (`counter-*`,
  `bool-demo-*`). Creating fresh `examples/gallery/` + `examples/gallery-rust/`
  directories satisfies this rule by construction — the gallery
  directory did not exist prior to Phase 2, so nothing is fattened.
- The Box sub-screen seeded in Phase 2's `gallery.ui` becomes
  WrapPanel-of-Boxes in Phase 3, ScrollView-wrapped in Phase 4, etc.
  The growth path is additive, not a per-phase scrap-and-rebuild.

**C/Zig host parity for Box is not required in Phase 2.**
[m3-plan.md §Phase-end criteria item 5](../../plan.md#phase-end-criteria)
calls for at least one host per phase; Phase 8 broadens the full
gallery to all three. Phase 2 seeds `examples/gallery-rust/` only.

**Phase 2's `examples/gallery/` is a partial gallery, not the A1
proof.** A1 acceptance lives in Phase 8 per the
[acceptance ↔ phase mapping](../../plan.md#acceptance--phase-mapping).
Phase 2's slice is necessarily incomplete: it cannot exercise
WrapPanel, ScrollView, Grid, ZStack, conditional rendering, iteration,
or selected state, because those primitives do not yet exist. The
Phase 2 sub-screen demonstrates Box + Text placeholder against a
trivial frame.

### G. GUI smoke responsibility separation (predoc-inputs §5)

Visual correctness of `aspect: <ratio>` (the rendered rectangle has
the right ratio) and `fill: <color>` (the rendered rectangle is the
right color, including alpha if DD-003 admits it) is **owner-manual
GUI smoke** territory. The assistant records `Start-Process` launch
command success and any captured headless integration output, but
does not assert on visual rendering.

The Phase 2 ADR's verification strategy section distinguishes
headless integration test gates (measure/arrange numeric assertions,
property-propagation propagation) from owner GUI smoke gates (pixel-
or eyeball-level correctness).

---

## Inputs absorbed

This section is the bibliographic mapping. Each row records which
Phase 2 framing artifact (DD or framing decision) consumed which
input section. Rows marked **Out of scope** carry through to the
ADR's Out-of-scope section, not into any DD.

### From [m2-to-m3-handover.md](../../../milestone-2/handoff.md)

| Section | Disposition | Consumed at |
|---|---|---|
| §1 `wasamo-ir` is the shared IR crate | Premise / sub-issue input | DD-001 (IR shape: new IR form = grammar + variant + loader wiring triple), DD-002 / DD-003 (new value types extend the shared crate, not the compiler-only or runtime-only halves) |
| §2 `HandlerExpr` unified across handler bodies and binding expressions | Premise / sub-issue input | DD-002 / DD-003 (new value-type literals add type-suffixed variants — `F32Lit` / `RatioLit` / `ColorLit` per chosen options — to the single unified enum, following Phase 1's `BoolLit` precedent) |
| §3 item 1 — cycle detection policy | Out of scope | Out-of-scope section (Phase 6/7 work) |
| §3 item 2 — ordering ties | Out of scope | Out-of-scope section (Phase 6/7 work) |
| §3 item 3 — fan-out × `MUTATION_CAP` interaction | Out of scope | Out-of-scope section (Phase 6/7 work) |
| §3 item 4 — synchronous non-batched drain proof contract | Out of scope (carries) | Out-of-scope section. Box introduces no batching; the contract is unaffected by Phase 2. predoc-inputs §9 is the back-pointer. |
| §4 `TypedValue` evaluator unification — open question | Discipline reminder | DD-004 sub-issue. F5 deferral preserved by routing bindable types through the per-type writer seam at `ir_loader::build_node`, not by extending the reactive engine with type dispatch |

### From [predoc-inputs.md](constraints.md)

| Section | Disposition | Consumed at |
|---|---|---|
| §1 New `PropertyValue` variant ⇒ ABI value-conversion arm fold in same step | Discipline reminder | DD-002, DD-003 sub-issues. Any option in either DD that adds a new variant carries this fold obligation; only ABI-public-function-adding changes warrant a standalone step |
| §2 Per-type writer seam at `ir_loader` call site for new bindable property | Premise / sub-issue input | DD-004 sub-issue. Seam location is the structural support for F5 deferral and dictates the shape of the new triple (`evaluate_<T>_binding` + `widget_write_property_<T>` + `register_<T>_binding`) if either attribute is admitted bindable |
| §3 `cargo fmt` process gap — checklist vs CI | Lifted to framing decision | Framing decision E. Recommendation: (a) checklist amendment only; (b) CI YAML deferred |
| §4 Visible proof via sibling example, not canonical example extension | Lifted to framing decision, with substantive deviation | Framing decision F. The "do not extend canonical examples" rule is honored by **creating** `examples/gallery/` + `examples/gallery-rust/` as new directories. predoc-inputs §4's stated default of `examples/box-demo-rust/` would have re-extended the Phase 1 foundational-phase exception, conflicting with m3-plan §Phase-end criteria item 5; framing decision F overrides that default in favor of seeding the gallery directly |
| §5 GUI smoke = owner manual; assistant records launch command success | Lifted to framing decision | Framing decision G. The Phase 2 ADR's verification strategy distinguishes headless test gates from owner GUI smoke gates |
| §6 Retroactive spec-gap fold permitted within same phase, minimum scope | Discipline reminder | Framing decision D Moment 2 (phase close). Earlier-phase spec gaps surfaced during phase-close impl re-sync may fold into the re-sync commit with explicit owner confirmation |
| §7 `f32` / `f64` in `IrType` re-evaluation | Sub-issue input | DD-002 sub-issue. Float-option in DD-002 triggers the type-suffix extension (`IrType::F32`, `IrLiteral::F32`, `HandlerExpr::F32Lit`) per the Phase 1 pattern. Default position remains "do not add" unless DD-002 selects the float option |
| §8 `bool` display conversion forbidden until explicit surface | Out of scope | Out-of-scope section. Phase 2 introduces no formatting surface; the rule continues without Phase 2 action. Phase 6+ inherits |
| §9 Bool live proof depends on synchronous non-batched drain | Out of scope | Out-of-scope section, as a back-pointer to m2-to-m3-handover §3 item 4. Phase 2 does not touch event/input batching, layout scheduling, or proof boundaries |

### From [m3-target-app-predoc.md](../../requirements/spec.md) (carryover context)

| Section | Disposition | Consumed at |
|---|---|---|
| 保留 1 closure (AspectRatio → Box attribute) | Premise | DD-002 (aspect is a Box attribute, not a standalone primitive — already settled at target-app pre-doc; Phase 2 does not re-open) |
| 保留 2 closure (Image deferral via Box + Text placeholder) | Premise | DD-006 (placeholder canonicalization). The closure direction is fixed; Phase 2 settles only the canonicalization form |
| 保留 3 closure (Button selected state + bool scalar) | Out of scope | Phase 8 work; mentioned only as context for the F5 maintenance argument under DD-004 |
| 必要 surface §Layout primitive — Box row | Premise | DD-001, DD-002, DD-003, DD-006 (the Box row enumerates exactly these attributes and the placeholder role) |
| Out-of-scope §Value / type (Image, Button content, TypedValue) | Premise | Out-of-scope section of this framing |
| Out-of-scope §Visual / styling (scrim alpha) | Premise | Out-of-scope section; affects DD-003's alpha sub-issue (the value-type layer admits or excludes alpha; styling is M3-out-of-scope either way) |

### From [m3-plan.md](../../plan.md)

| Section | Disposition | Consumed at |
|---|---|---|
| §Acceptance criteria — A6 | Constraint | Framing decision B (pre-doc-discipline check). The DD slate is verified against A6 |
| §Phase breakdown — Phase 2 description ("pure primitive — no novel measure-arrange algorithm") | Constraint with nuance | DD-005 (the claim is acknowledged and nuanced; "no novel" = no new paradigm vs WrapPanel, not "no spec content") |
| §Phase dependencies — Phase 2 → 3 → 4 chain | Constraint | DD-001 / DD-006 (Box's attribute surface and placeholder pattern must be sufficient for Phase 3 WrapPanel and Phase 6 ZStack to consume without redefinition) |
| §Verification strategy | Menu | Framing decision C (verification strategy chosen from the menu) |
| §Phase-end criteria item 5 (foundational-phase exception scoped to Phase 1 only) | Hard constraint | Framing decision F. The exception is **not** extended to Phase 2; Phase 2 seeds `examples/gallery/` + `examples/gallery-rust/` directly. An earlier framing draft attempted an `examples/box-demo-rust/` substitute and was rejected because that would constitute an unrecorded plan revision, not a framing decision |
| §Risks — WrapPanel / Grid measure-arrange spec complexity | Adjacent risk | DD-005 (Phase 2's measure-arrange spec is the rehearsal that lowers the WrapPanel / Grid spec risks in later phases) |
| §Risks — Reactive-drain residuals | Out of scope | Out-of-scope section (Phase 2 does not touch the drain) |

---

## Next session — handoff

Inputs are complete. The next session begins ADR drafting:

1. Create `process/milestone-3/phase-2/decisions/preamble.md` (working title) as
   `Status: Proposed`, carrying the 6 DDs above with full Option
   tables, Recommendation prose, and the two-axis risk/exposure
   evaluation per DD (per
   [process/README.md §Risk evaluation](../../../README.md)).
2. Owner review pass.
3. On `Status: Accepted` flip, the upstream document edits enumerated
   under **framing decision D Moment 1** bundle into the same commit:
   `docs/dsl_spec.md` Box chapter as ADR-accepted design draft,
   `docs/architecture.md` §6 entry, `docs/plans/m3-plan.md` Progress
   row populated, `docs/notes/retrospectives.md` amendment per
   framing decision E.
4. Phase progress file
   `docs/plans/progress/m3-phase-2-progress.md` opens with `Status:
   active` after the Accepted flip; the m3-plan.md Progress row's
   Status flips from `not started` to `in progress`.
5. Implementation phase proceeds. At phase close, **framing decision
   D Moment 2** lands: `docs/dsl_spec.md` re-synced to impl with
   frontmatter flipped from "ADR-accepted design draft" to
   "Phase 2 closed; impl-synced", any earlier-phase spec gaps folded
   per [predoc-inputs.md §6](constraints.md#6-retroactive-spec-gap-fold-は最小範囲で同じ-phase-に折り込む),
   phase progress file retired per the standard `active → closing →
   retired → archived` lifecycle.
