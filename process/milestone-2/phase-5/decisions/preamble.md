# M2-Phase 5 — Reactive engine: Architecture Decisions

**Phase:** M2-Phase 5 (reactive state propagation engine)
**Date:** 2026-05-05
**Status:** Accepted

## Context

M2 acceptance criterion **A2** ([m2-plan.md](../../plan.md#acceptance-criteria),
mirrored from [process/_roadmap.md M2](../../../_roadmap.md#m2-foundation)):

> Reactive state propagation works without host-side property-set
> plumbing: `count++` in the host updates the visible label through
> the M2 reactive path, not through a manual `wasamo_set_property`
> call written by the application.

A2 is the M2 thesis-validation point: every other phase contributes
structure (A3 cdylib-shim split), ABI surface (A4 tree-mutation
primitives), or integration (A1 `.ui`-driven counter), but A2 is where
"property write → bound widget update without host wiring" is exercised
end-to-end for the first time. The other M2 phases either feed into
this engine or consume its output; the engine itself is the foundation
hypothesis the milestone rests on.

Phase 5's job is to design and implement the property-change → binding
re-evaluation → widget property write → invalidate path on top of the
machinery prior phases established, with shape choices that survive
M3 binding-feature growth (Grid cell bindings, List per-item context)
without front-loading fine-grained tracking before there is any DSL
grammar to align against.

### Constraints carried in from prior decisions

- **DD-M2-P3-001 = Option A** (runtime-side handler interpreter).
  Handler bodies mutate property storage through the internal
  `set_property` path
  ([wasamo-runtime/src/widget.rs:334](../../../../wasamo-runtime/src/widget.rs#L334)).
  The reactive engine observes those internal writes directly; no
  C ABI round-trip is involved. This is the load-bearing argument
  for runtime-side reactivity — see DD-M2-P3-001's reactive-integration
  paragraph.
- **DD-M2-P3-002 = Option B** (separate inline-handler slot vs host
  listener list). The handler evaluator core
  ([wasamo-runtime/src/handler.rs](../../../../wasamo-runtime/src/handler.rs))
  is already factored as `HandlerExpr` + `EvalContext` trait +
  `evaluate()`. Phase 5 reuses this evaluator for binding-expression
  evaluation, with a read-only context variant — the binding evaluator
  is **not** a parallel implementation.
- **DD-M2-P4-004 = Option A** (no host-visible batching ABI). The
  internal `with_batched_writes` helper
  ([wasamo-runtime/src/reactive.rs:18](../../../../wasamo-runtime/src/reactive.rs#L18))
  is the runtime-internal coalescing primitive. Phase 5 implements it
  (Phase 4 shipped the skeleton).
- **DD-P6-003 = Option A** (queued emission). The runtime guarantees
  no callback fires while the host is inside a `wasamo_*` call;
  `emit::drain_if_outermost` ([wasamo-runtime/src/abi.rs:369](../../../../wasamo-runtime/src/abi.rs#L369))
  runs at outermost-frame boundaries. Phase 5's reactive dispatch must
  compose with this rule, not bypass it.
- **DD-P8-002** (size-affecting property writes invalidate layout).
  The runtime already auto-marks layout-dirty on writes to
  `TEXT_CONTENT` / `TEXT_STYLE` / `BUTTON_LABEL`; the layout drain
  runs after the emission drain. Phase 5's binding writes go through
  the same `set_property` path and inherit this behaviour. The
  whole-window dirty granularity is preserved; subtree-grain dirty is
  out of scope (open question in
  [layout-engine note §3.4](../../../../docs/notes/layout-engine.md)).
- **Pre-aligned design axes.** [docs/notes/m2-phase-5-design-axes.md](../requirements/framing-draft.md)
  records owner direction (2026-05-05) on two axes before pre-doc:
  (a) intermediate dependency-tracker depth (Signal + Effect 2-layer,
  read-time auto-collection; Computed deferred to M3); (b) Option A
  verification (pure-logic tests with fake Effect closure; no new
  mirrors; no headless backend). These axes are recorded as DDs in
  this ADR with full options for the record, but the recommendations
  align with the pre-aligned direction.

### What "reactive" means concretely at M2

The smallest set of behaviours that satisfies A2:

1. **Reactive primitive.** A storage cell whose reads are observable
   and whose writes notify dependents.
2. **Binding.** A side-effecting closure that re-runs when any
   primitive it read on its previous run changes. In M2 the only
   binding shape is `Text { content: "Count: \{root.count}" }` and
   similar property-bound expressions; M3 introduces structural
   bindings (conditional / `for` over collections).
3. **Re-evaluation cycle.** Property write → mark dirty bindings →
   (at the appropriate flush point) re-run dirty bindings → new values
   are written into widget property storage → existing layout/render
   invalidation kicks in.
4. **Coalescing.** A burst of property writes from one logical update
   (`count += 1` typically writes once, but reactive writes from a
   binding may cascade) results in each binding re-evaluating at most
   once per cycle, not once per intermediate write.

Computed values (derived primitives that other bindings can read) and
fine-grained dirty propagation through derived nodes are **out of
scope** for M2 — they belong with the M3 DSL spec work that defines
the DSL surface for derivations. The Phase 5 architecture leaves room
for a Computed layer to be added without disturbing Signal or Effect.

### Architectural framing assumption

Reactive GUI architectures span several families: tree-with-bindings
(Slint, QML/Qt), view-function with re-execution (SwiftUI, Jetpack
Compose), coarse subtree-rebuild (Flutter `setState`), and manual
property notification (WPF / WinUI). The cumulative effect of accepted
decisions (DSL × C ABI thesis; DD-M2-P2-001 = B textual IR as a tree
description; DD-P6-001..007 handle-based stable core; DD-M2-P3-001 = A
handler on internal `set_property`) currently fits the
tree-with-bindings family best, but no accepted ADR names that
selection as a long-term commitment.

[docs/notes/architectural-family.md](../../../../docs/notes/architectural-family.md)
tracks the current hypothesis status, the family-neutral vs
family-coupled split of the design, and the re-evaluation triggers
(M3 DSL spec drafting; hot-reload work; binding shapes that don't fit
`BindingTarget`; owner revisitation). This Phase 5 ADR's
recommendations apply within that current hypothesis frame.

The Phase 5 primitive selection is deliberately family-neutral:
Signal + Effect 2-layer with read-time auto-tracking is shared by
families (1) and (2) and degenerates cleanly to (3); only the
`BindingTarget` shape (DD-M2-P5-005) is family-coupled, and it is
`pub(crate)`, revisable through internal refactor without C ABI
churn. The cost of the implicit framing is bounded to that surface.

---

## Summary of proposed decisions

| ID | Topic | Recommendation | Impl risk | Forward-compat exposure |
|---|---|---|---|---|
| DD-M2-P5-001 | Reactive primitive layering | **Option B** — Signal + Effect 2-layer; Computed deferred to M3 | Low–medium | Low |
| DD-M2-P5-002 | Dependency collection mechanism | **Option B** — read-time auto-track via thread-local effect stack | Low–medium | Low |
| DD-M2-P5-003 | Effect lifetime and disposal | **Option A** — owned by host widget; disposed via `Drop for WidgetNode` | Low | Low |
| DD-M2-P5-004 | Reactive dispatch timing | **Option B** — deferred to outermost-frame drain alongside observer + layout drains | Low–medium | Low |
| DD-M2-P5-005 | Phase 6 binding registration API | **Option A** — one generic `register_binding(target, HandlerExpr)`; `BindingTarget` enum | Low | Low |
| DD-M2-P5-006 | Verification strategy | **Option A** — pure-logic tests with side-effect-logger Effect closures; no new mirrors; GUI manual checkpoint | Low | Low |

**Aggregate impl-risk picture.** The non-trivial impl-risk axes are
DD-M2-P5-001's 2-layer engine bookkeeping, DD-M2-P5-002's thread-
local effect stack invariants, and DD-M2-P5-004's drain-loop
quiescence. All three are bounded: the engine is `pub(crate)` and
freely revisable; the thread-local stack is single-threaded by
construction (M2 is single-threaded throughout); the drain loop has
an iteration cap as a divergence trap. No DD introduces a mechanism
the runtime hasn't already exercised in shape (DD-P6-003's
queued-emission drain is the model for DD-M2-P5-004's reactive drain;
DD-M2-P3-001's `EvalContext` is the model for DD-M2-P5-002's
`BindingEvalContext`; Phase 4's subtree teardown is the model for
DD-M2-P5-003's binding disposal).

**Aggregate forward-compat exposure.** All six DDs recommend the
M3-additive option. Phase 5's runtime delta is intentionally
internal: no new C ABI symbols (DD-M2-P4-004 = A precludes that),
one new public-to-Phase-6 internal Rust function
(`register_binding`), one new `pub(crate)` module
(`reactive` already exists from Phase 4). M3's Computed and
structural-binding work lands as additions to `BindingTarget`,
to the drain loop's pre-Effect topological pass, and to the
reactive module — not as rewrites.

**Pre-doc validation spike.** Not required. The handler evaluator
([wasamo-runtime/src/handler.rs](../../../../wasamo-runtime/src/handler.rs))
is the pre-existing reference for the evaluator-with-context shape
DD-M2-P5-005 = A reuses; the queued-emission drain
([wasamo-runtime/src/abi.rs:369](../../../../wasamo-runtime/src/abi.rs#L369))
is the pre-existing reference for the deferred-dispatch shape
DD-M2-P5-004 = B extends; the `with_batched_writes` skeleton
([wasamo-runtime/src/reactive.rs:18](../../../../wasamo-runtime/src/reactive.rs#L18))
is the integration point already in place. The Solid.js / Vue ref
prior art is broadly understood and exercised at scale; no spike is
needed to validate the 2-layer auto-tracking premise.

## Out of scope

- **`Computed<T>` derived primitives.** Deferred to M3 alongside the
  DSL spec public draft, which decides derivation grammar. The
  Phase 5 architecture (DD-M2-P5-001 = B) is shape-compatible with
  a future Computed layer added between Signal and Effect; the
  drain loop (DD-M2-P5-004 = B) extends with a pre-Effect
  topological re-eval pass.
- **Structural bindings (conditional / for-loop / list-rendered).**
  Deferred to M3. M2 binds property values only; subtree shape is
  static. M3 adds new `BindingTarget` variants; DD-M2-P5-005's
  registration API is unchanged.
- **Subtree-grain layout dirty.** Phase 5 inherits DD-P8-002's
  whole-window dirty path; finer granularity remains the open
  question in [layout-engine note §3.4](../../../../docs/notes/layout-engine.md)
  and is revisited only if M2 acceptance demands it (it does not).
- **Host-visible reactive API.** No C ABI symbol is added (per
  DD-M2-P4-004 = A). Hosts in M2 do not interact with the reactive
  engine directly; they observe its effects through the existing
  property-set + observer machinery.
- **`untrack` / read-without-subscribe escape hatch.** Deferred to
  M3 when Computed adds use cases. DD-M2-P5-002 = B's auto-track
  is shape-compatible with an additive `untrack` helper.
- **Explicit `engine.flush()` primitive.** Not added; the drain
  trigger remains the outermost-frame boundary (DD-M2-P5-004 = B).
  M3+ may revisit if a host-driven flush case appears.
- **Headless / "no-Compositor" runtime mode.** Per DD-M2-P5-006 = A
  and [headless-verification.md](../../../../docs/notes/headless-verification.md);
  not built in M2. Re-evaluation triggers (post-1.0 hot reload CI;
  binding-conformance tests) remain as recorded in that note.
- **Multi-threaded Signal access.** M2 is single-threaded; the
  thread-local effect stack assumes single-threaded use. Multi-
  thread support is post-1.0 and would need its own ADR.
- **Updating A4's "required by the reactive engine" wording.**
  Already deferred per the M2-Phase 4 ADR's Out of scope item.
  Phase 5's implementation does not change the situation: the
  reactive engine remains internal Rust and does not call across
  the C ABI. If a vision decision record rewriting A4 is desired, it remains
  a one-line documentation change; Phase 5's work product is
  unchanged either way.
