---
phase: M3-Phase 6
title: ZStack + conditional rendering
status: closing
adr: process/milestone-3/phase-6/decisions/preamble.md
plan: process/milestone-3/plan.md
opened: 2026-06-02
---

# M3-Phase 6 — ZStack + conditional rendering: Implementation

This is the live task list and execution framing for M3-Phase 6. The
design decisions are frozen in the ADR set under
[../decisions/](../decisions/preamble.md) (preamble + DD-M3-P6-001
through DD-M3-P6-006, Status: Accepted on 2026-06-02). This file and
its sibling [plan.md](./plan.md) are mutable during the phase per
[../../../README.md §SSOT distribution](../../../README.md#ssot-distribution);
the in-flight decisions log and CI evidence land in
[log.md](./log.md) as the phase progresses. Cross-phase residuals
land in [handoff.md](./handoff.md) at phase close.

## Phase 6 scope

Phase 6 ships **two surfaces as one unit** (FD-F) — ZStack overlay
layering + conditional rendering open/close, exercised by the gallery
lightbox — satisfying **A4** (ZStack) and **A7** (conditional
rendering), discharging carry-forward residual **R1** (Window-title
host-wiring), and advancing **A11** / **A12** per phase. The decisions
and their rationale are frozen in the
[ADR §Decisions table](../decisions/preamble.md#decisions); this list
is only *what each task builds* (DD pointers, not a re-derivation):

- **ZStack** — IR node (per-kind tag, direct children, no `KindPayload`,
  no new `IrType`/`IrLiteral`) + author surface (DD-M3-P6-001);
  union-sizing / `Fill/Fill`-default measure-arrange + document-order
  z-order + outer-bounds clip (DD-M3-P6-002).
- **Conditional rendering** — `if <bool-expr> { <widget-child> }`
  grammar + diagnostics (DD-M3-P6-003); the **member-level `IrMember` /
  `ControlFlowNode` IR-schema change** + runtime present/absent via
  `BindingTarget::ConditionalSubtree` (DD-M3-P6-004); absent-subtree
  effect disposal + the **preserved** synchronous-drain contract
  (DD-M3-P6-005).
- **R1 Window-title** — the loader routes the static component-level
  `title:` to `window::create`; dynamic (`String`-binding) title
  evaluated-and-deferred (DD-M3-P6-006, FD-D).

Out of scope — iteration, `else` / `switch`, nested `if` directly in a
body, `key:` retention, condition operators, explicit `z-index`,
per-child ZStack clip, the dynamic-title *implementation*, the Image
widget, runtime per-monitor DPI, and `TypedValue`:
[../decisions/preamble.md §Out of scope](../decisions/preamble.md#out-of-scope)
and §Forward-compat exposure carry the deferrals and their landing
points.

## Verification closure

The ADR's
[Phase 6 verification closure](../decisions/preamble.md#phase-6-verification-closure-what-counts-as-a4--a7-evidence)
fixes the seven evidence lines and their content; per FD-C they do not
collapse even where they share helper infrastructure. This plan adds
only the *where* (task mapping); the per-item sub-assertions
(declared-sibling-order landing, removal-index shift, re-eval
idempotency, toggle-then-observe drain) live in the ADR item and are
referenced — not re-stated — by the owning task:

| ADR evidence item | Task(s) |
|---|---|
| (1) `wasamoc check` compile-time | ZStack T1; conditional T4; gallery positive control T7 |
| (2) pure-logic layout + presence reducer | ZStack measure-arrange T2; conditional presence reducer T4 |
| (3) lowering / roundtrip / loader-invariant | ZStack T1 (emit) + T3 (loader); conditional T4 (emit + loader) |
| (4) Windows-runtime integration (CI-gated, fail-not-skip) | ZStack z-order + clip T3; conditional reactive toggle + drain T5; R1 title T6 |
| (5) assistant-visible GUI (before/after toggle pair) | T7 |
| (6) E2E visible smoke | T7 (assistant build/launch) + T8 (owner GUI smoke) |
| (7) A12 spec-closure gate | T9 |

The Windows-runtime fixtures (item 4) fail — not skip — on a runner
that cannot create the Compositor; the skip-guard inherits the Phase 2
T11 / Phase 3 / 4 / 5 pattern (`0x80070005` from `wasamo_init`).

**Positive-control discipline** ([constraints.md §3](../requirements/constraints.md)
— load-bearing this phase): state-driven evidence must **toggle the
state** (a single static frame is not evidence). Phase applications:
conditional proofs are **before/after toggle pairs** (`is_lightbox_open`
present → absent → present); ZStack z-order is proven by **occlusion**
(photo/caption/nav over the scrim; scrim dims the thumbnails); `Fill`
behaviour by **resize**. T7 (assistant) and T8 (owner) carry this as
their minimum evidence.

## Step-end / phase-end retrospective split (constraints §6 / FD-I)

Per
[../requirements/constraints.md §6](../requirements/constraints.md)
and FD-I (inherited from Phase 4 T7 / Phase 5), the final-step
retrospective is split from the start in this plan, not after the
fact:

- The **final-task (T9) step-end retrospective**
  ([retrospectives.md](../../../procedures/retrospectives.md) checklist
  items 1–11; step → phase merge gate) is **owned by T9** and is a T9
  deliverable. T9's checkbox flips to `[x]` when this retro is
  recorded.
- The **phase-end retrospective** (checklist items 12–18; phase → main
  merge gate) is **NOT owned by T9**. It is recorded on the phase
  branch after T9 merges in, by a separate retro commit, and is the
  precondition for the phase → main merge gate. The corresponding T9
  plan bullet **stays `[ ]` at T9 close** and is checked by the
  phase-end retro commit on the phase branch.

This split exists so the reviewer's mental model of "T9 closes the
phase" matches the operational reality of "T9 closes the step, the
phase-end retro closes the phase". Before the final task closes, the
T0-frozen task list is cross-checked against any mid-phase owner
decisions and the mutable phase plan is revised where they diverge
(constraints §6 — revise, do not work around).

## Lifecycle transition

This implementation file opens at `status: active` and transitions to
`status: closing` at the **phase-end batch commit** — the phase-branch
commit that lands the CI-verified gates (local `cargo fmt`, plus CI
`build` / `test` / Windows integration) and the spec / architecture /
plan status flips + log.md — **not** at T9 step-close. The on-CI gates
are phase-end-owned and verified only after the phase branch runs
`workflow_dispatch` CI (Phase 5 actual-operation correction), so
**T9 step-close itself leaves `status: active`**. The handoff may land as
a separate phase-end review concern before this batch. The phase-end
retrospective is a **separate commit in the same phase-end cluster**
(not folded into the closing-flip commit). The file remains mutable
during the `closing` window only for the phase-end retrospective
evidence pointer and any final post-merge distillation; no further task
checkboxes are added once the phase-end batch lands. The `closing` →
`retired` transition belongs to the phase → main merge commit /
post-merge distillation, not this batch. The phase-end retrospective itself is
recorded under
`process/milestone-3/phase-6/retrospectives/phase-end.md` (with sibling
`tN.md` per-step retros under the same directory) and is a separate
commit on the phase branch — it is not a T9 deliverable per the split
above.

Per
[../../../procedures/retrospectives.md](../../../procedures/retrospectives.md),
implementation start is gated on Moment 1 commit-set completion
(ADR Accepted; dsl_spec ZStack + conditional chapters; architecture
§6.9 ZStack + conditional IR/runtime; m3-plan.md Phase 6 row; this
implementation/preamble.md + plan.md). At T0 land time, the Moment 1
commit set closes and T1 may open.

## Technical risks (planning-time recon)

Spot-checked against the current source on 2026-06-02 to surface risks
where ADR-level assumptions sit further from the existing abstractions
than the ADR text implies. Recorded here so the [plan.md](./plan.md)
tasks that mitigate them are explicit pre-implementation spikes, not
implicit discoveries; revisit and close each risk in [log.md](./log.md)
as it lands or evolves.

| ID | Risk | Severity | Source pin | Mitigation |
|---|---|---|---|---|
| R-A | **`IrNode.children: Vec<IrNode>` → `Vec<IrMember>` is a breaking schema change** (DD-M3-P6-004 O1). Unlike Phase 5's additive `Option<KindPayload>` field, switching the child collection's element type breaks **every** site that reads or builds `node.children` as a `Vec<IrNode>`: `wasamoc` emit / lower, the runtime loader's `build_node` child loop and all `validate_phaseN_*` walkers, and the cross-crate test corpus. The workspace will not build until all sites migrate together. | **High** | [`wasamo-ir/src/lib.rs:146`](../../../../wasamo-ir/src/lib.rs#L146) (`children: Vec<IrNode>`); [`wasamo-runtime/src/ir_loader.rs:1421`](../../../../wasamo-runtime/src/ir_loader.rs#L1421) (generic child loop) | [plan.md T4 pre-implementation spike](./plan.md#t4--conditional-ir-schema-grammar-and-static-loader). Settle the `IrMember` / `ControlFlowNode` shape and the construction-site migration discipline (helper constructor vs broad edit) before opening the bullets; land the schema change bundled with emit + loader + validators in one buildable commit per [CLAUDE.md §Commit rules](../../../../CLAUDE.md#commit-rules). Record the chosen shape in [log.md](./log.md). |
| R-B | **`build_node` must dispatch member kind, not just widget kind.** The Grid path special-cases `node.widget_type == "Grid"` and flattens `Cell` content children, but the control-flow member is **not** a `widget_type` — it is a sibling member kind alongside `Widget`. So `build_node` must iterate `Vec<IrMember>` and dispatch `Widget(_)` vs `ControlFlow(_)`: a present-at-load `if` materialises its body, an absent one materialises nothing and registers a `ConditionalSubtree` binding instead. The insertion index for a later toggle is **recomputed from declared order + live presence**, not stored. This is a structural change to the child loop beyond the Phase 5 Grid branch. | **High** | [`wasamo-runtime/src/ir_loader.rs:1344`](../../../../wasamo-runtime/src/ir_loader.rs#L1344) (`build_node`), [`widget.rs:1260`](../../../../wasamo-runtime/src/widget.rs#L1260) (`insert_child`) / [`:1289`](../../../../wasamo-runtime/src/widget.rs#L1289) (`remove_child`) | [plan.md T4](./plan.md#t4--conditional-ir-schema-grammar-and-static-loader) (member dispatch in build_node) + [T5](./plan.md#t5--conditional-reactive-toggle-and-windows-runtime-evidence) (index recomputation / mutation). |
| R-C | **`BindingTarget` is a single-variant enum destructured by irrefutable `let`.** DD-M3-P6-004 adds `ConditionalSubtree { parent, declared_member_index }`. Both `register_binding` and `register_bool_binding` do `let BindingTarget::WidgetProperty { node, prop } = target;` — irrefutable only because there is one variant; adding a variant makes those `let`s refutable (compile error). The structural-mutation binding needs a **new registration path** (it inserts/removes a subtree, not a property write). Confirmed seam: `EffectHandle::new<F: FnMut()>` ([reactive.rs:253](../../../../wasamo-runtime/src/reactive.rs#L253)) already accepts an arbitrary `FnMut()` closure (the property writers wrap it at `:618`), so a conditional binding rides `EffectHandle::new` with an insert/remove closure rather than a writer — no engine change, a new call-site seam. | Medium | [`reactive.rs:585`](../../../../wasamo-runtime/src/reactive.rs#L585) (enum), [`:602`](../../../../wasamo-runtime/src/reactive.rs#L602) / [`:640`](../../../../wasamo-runtime/src/reactive.rs#L640) (`let`-destructures), [`:253`](../../../../wasamo-runtime/src/reactive.rs#L253) (`EffectHandle::new`) | [plan.md T5](./plan.md#t5--conditional-reactive-toggle-and-windows-runtime-evidence): convert the destructures to `match`, add the conditional-subtree registration path; record the seam in [log.md](./log.md). |
| R-D | **The static title is dropped at the loader → window seam — extraction point now confirmed.** Lowering splices component-level props onto the root node ([lower.rs:58](../../../../wasamoc/src/lower.rs#L58) — *"Component-level props/bindings (e.g. title, backdrop) belong on the root node"*), so a static `title: "Gallery"` survives as an `IrProp` (name `"title"`, `IrLiteral::Str`) in `component.root.props` — but the root widget has no such property, so `build_node` silently skips it ([ir_loader.rs:1356](../../../../wasamo-runtime/src/ir_loader.rs#L1356)) and `wasamo_load_ui` calls `window::create(DEFAULT_WINDOW_TITLE, …)`. DD-M3-P6-006: read the static `title:` `IrProp` from `component.root.props` and thread it to `window::create` (which already takes `&str`). A `bind title = …` lands in `root.bindings` instead → dynamic title stays deferred (FD-D); the static/dynamic split falls out of props-vs-bindings, so no extra discrimination logic is needed. | **Low** (extraction point verified; mechanical) | [`abi.rs:1220`](../../../../wasamo-runtime/src/abi.rs#L1220) (`window::create(DEFAULT_WINDOW_TITLE, …)`), [`window.rs:57`](../../../../wasamo-runtime/src/window.rs#L57), [`lower.rs:58`](../../../../wasamoc/src/lower.rs#L58) | [plan.md T6](./plan.md#t6--r1-window-title-host-wiring): read `component.root.props` `"title"` literal at the `wasamo_load_ui` seam. |
| R-E | **Effect + widget-pointer-registry disposal on remove.** DD-M3-P6-005 requires an absent subtree's Effects **and** its `WidgetId`-keyed registry entries to be disposed via the existing structural teardown (`widget_destroy`), with no dangling `WidgetId` pointers feeding stale bindings. The Phase 4 mutation API (`remove_child`) returns the `Box<WidgetNode>`; the conditional path must route it through `widget_destroy` (not merely drop it) so the §6.7.6 dispose-ahead-of-teardown invariant holds. | Medium | [`widget.rs:1679`](../../../../wasamo-runtime/src/widget.rs#L1679) (`widget_destroy`), [`:1289`](../../../../wasamo-runtime/src/widget.rs#L1289) (`remove_child`) | [plan.md T5](./plan.md#t5--conditional-reactive-toggle-and-windows-runtime-evidence): wire `remove_child` → `widget_destroy`; assert Effect + registry disposal in the toggle integration fixture. |
| R-F | **`insert_child` ignores `index` for Visual z-order — it always `InsertAtTop`.** `insert_child` inserts into `self.children` at `index` but places the Visual via `parent_container.Children().InsertAtTop(&child_visual)` unconditionally ([widget.rs:1282](../../../../wasamo-runtime/src/widget.rs#L1282); `append_child` is the same at `:1248`). Sequential appends therefore give correct document-order z-order for a ZStack's *initial* build, but a conditional subtree **re-inserted between static siblings lands on top of later siblings**, not in declared sibling order — directly contradicting DD-M3-P6-004's *"the re-inserted subtree's Visual landing in declared sibling order (not merely on top) when static siblings flank it"*. Composition's `VisualCollection` has no index insert; the fix is `InsertAbove` / `InsertBelow` relative to the sibling Visual at the recomputed index. | **High** | [`widget.rs:1260`-`1287`](../../../../wasamo-runtime/src/widget.rs#L1260) (`insert_child`, `InsertAtTop`) | [plan.md T5](./plan.md#t5--conditional-reactive-toggle-and-windows-runtime-evidence): give `insert_child` a positional Visual insert; the toggle fixture's declared-sibling-order assertion is the regression gate. Pre-implementation spike. |

Low-severity items confirmed during recon (no mitigation needed,
listed here for traceability):

- `WidgetKind::ZStack` add + `measure_zstack` / `arrange_zstack` is a
  direct extension of the Phase 2–5 per-kind layout pattern
  ([`layout.rs:6`](../../../../wasamo-runtime/src/layout.rs#L6)); ZStack
  introduces **no new `LayoutError`** (DD-M3-P6-002).
- The ZStack outer-bounds `InsetClip` install + per-child clip-absence
  guard follows the Grid / ScrollView / WrapPanel precedents already in
  the loader / widget layer.
- `WidgetNode::run_layout_as_window_root` is WidgetKind-agnostic, so a
  ZStack-rooted fixture and a `VStack { ZStack { … } }` production-root
  fixture ride the same entry point used since Phase 4 T6.

## Cross-references

- ADR set: [../decisions/preamble.md](../decisions/preamble.md) +
  DD-M3-P6-001 through DD-M3-P6-006.
- Phase 6 framing decisions: [../requirements/framing.md](../requirements/framing.md)
  (FD-CR / FD-A … FD-I).
- Phase 6 carry-forward inputs:
  [../requirements/constraints.md](../requirements/constraints.md)
  (§1 R1 owning-phase; §2 assistant-visible evidence; §3 positive
  control — load-bearing this phase; §4 DPI-aware capture; §5 runtime
  DPI not adopted → M4; §6 step-end / phase-end split — applied in this
  plan from the start; §7 reactive-drain fix-or-carry obligation).
- Specification chapters (design-spec draft):
  [`docs/dsl_spec.md`](../../../../docs/dsl_spec.md) ZStack + conditional
  rendering chapters (Phase status: `M3-Phase 6 design accepted;
  implementation pending` until the T9 Moment 2 flip).
- Architecture entries:
  [`docs/architecture.md`](../../../../docs/architecture.md) §6.9 ZStack
  layout primitive + conditional IR/runtime (member-level `IrMember` /
  `ControlFlowNode`, `BindingTarget::ConditionalSubtree`, effect
  lifecycle, declared-tree / entity-tree note under §9); top Status
  flips to `M3-Phase 6 complete` at T9 Moment 2.
- ABI: [`docs/abi_spec.md`](../../../../docs/abi_spec.md) — **no touch**
  in Phase 6 per ADR (static title rides the existing
  `wasamo_load_ui` → `window::create` internal path; the `If` construct
  adds no host-facing ABI surface; no new `PropertyValue` tag).
