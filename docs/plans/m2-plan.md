---
milestone: M2
status: in-progress
roadmap-anchor: ROADMAP.md#m2-foundation
adrs:
  - docs/decisions/vision-post-m2-roadmap.md
  - docs/decisions/m2-phase-1-cdylib-shim.md
  - docs/decisions/m2-phase-2-wasamoc-output-format.md
  - docs/decisions/m2-phase-3-handler-exec-location.md
  - docs/decisions/m2-phase-4-tree-mutation-abi.md
  - docs/decisions/m2-phase-5-reactive-engine.md
  - docs/decisions/m2-phase-6-ui-lowering.md
  - docs/decisions/m2-phase-7-reactive-foundation.md
created: 2026-05-02
---

# M2 Plan — Foundation Milestone

## Frozen agreement

### Purpose

M1 proved the core hypothesis (external DSL × C ABI × Visual Layer) by
exercising the runtime side end-to-end, with hosts constructing widget
trees imperatively through the experimental C ABI layer. M2's purpose
is to **close the loop on the DSL side**: make `.ui` files actually
drive the runtime, with reactive state propagation, so that Hello
Counter in each language is written against the DSL rather than
reproducing the DSL by hand.

This redefines M2 from the original "Alpha" feature wishlist (Grid /
ScrollView / List / input / IME / AccessKit / VS Code / DSL spec
public draft) into a **foundation milestone** whose acceptance is
structural, not feature breadth. The Alpha-style feature work has
been redistributed across M3–M6 (see
[ROADMAP.md](../../ROADMAP.md) and
[docs/decisions/vision-post-m2-roadmap.md](../decisions/vision-post-m2-roadmap.md)).

### Phase numbering

Phase numbers in this plan are **local to M2** (M2-Phase 1, 2, …).
M1's global Phase 1–8 numbering is not continued. ADR identifiers
from M2 onward use the scope `M<N>-P<n>` (e.g. `DD-M2-P2-001`); see
[docs/decisions/README.md](../decisions/README.md#file-naming).
M1 phase ADRs (`DD-P3-001` etc.) remain as historical records and
are not renumbered.

### Acceptance criteria

ROADMAP is the SSOT; mirrored here for ergonomics:

- **A1.** `examples/counter/counter.ui` drives the running Hello
  Counter in C, Rust, and Zig — the M1 host-imperative trees in
  `examples/counter-{c,rust,zig}/` are replaced by hosts that load
  the DSL through the agreed wasamoc pipeline.
- **A2.** Reactive state propagation works without host-side
  property-set plumbing: `count++` in the host updates the visible
  label through the M2 reactive path, not through a manual
  `wasamo_set_property` call written by the application.
- **A3.** `wasamo-runtime` and the `wasamo` safe wrapper no longer
  share an rlib filename through the cdylib-shim split; the post-M1
  cleanup flagged in
  [DD-P7-002](../decisions/phase-7-language-bindings.md) is
  discharged.
- **A4.** The C ABI gains the tree-mutation primitives required by
  the reactive engine; the experimental layer's all-at-once
  constructors remain available but are no longer the only way to
  construct UI.

- **A5.** Reactive Foundation Hardening. The reactive engine's
  execution-order guarantees and the runtime's re-entrancy/guard
  placement principle are settled at design level (Accepted ADRs)
  and reflected in implementation. Specifically:
  - DD-M2-P6-010 (topological sort of the dirty Effect drain) is
    Accepted and the implementation no longer relies on the counter
    case happening to converge.
  - DD-M2-P6-012 (re-entrancy / safety-guard placement principle)
    is Accepted and the principle is recorded in
    `docs/architecture.md` as a global runtime invariant that future
    M3+ entry paths must observe.

- **A6.** Type-Agnostic Reactive Binding. The reactive binding path
  is demonstrated end-to-end with a non-`i32` property type
  (`String`), proving the `EvalContext` / `HandlerExpr` / IR design
  is not silently `i32`-specialized.
  - DD-M2-P6-011 is Accepted; `.ui` String property bound to
    `Signal<String>` propagates to the visible widget.

### Phase breakdown

The phases below are working hypotheses; each one's design questions
become a phase ADR at pre-doc time, per
[the decisions README](../decisions/README.md).

- **M2-Phase 1 — cdylib-shim cleanup.** Split DLL output from the
  rlib so `wasamo-runtime` can be renamed cleanly without the
  cargo#6313 filename collision. Pure infra; independent of the DSL
  track. Origin:
  [DD-P7-002 post-M1 implementation note](../decisions/phase-7-language-bindings.md).

- **M2-Phase 2 — wasamoc output format decision.** Resolve the
  question Phase 6 pre-doc explicitly deferred to M2: host-language
  codegen vs IR + runtime interpretation. Includes implications for
  binding-author workload and the feasibility of post-M2 hot-reload
  (the latter is out of M2 scope but is constrained by this
  decision).

- **M2-Phase 3 — Handler execution location.** Resolve the second
  Phase 6-deferred question: where DSL inline handler bodies
  (`clicked => { ... }`) execute. The decision interacts with
  M2-Phase 2 and with M2-Phase 4's ABI surface.

- **M2-Phase 4 — Tree-mutation primitives at the ABI surface.**
  Promote the operations the reactive engine needs (insert / remove
  / replace child; property batching) from runtime-internal to the
  stable-core C ABI. M1 deliberately deferred this — see
  [DD-P8 "Out of scope"](../decisions/phase-8-hello-counter.md).

- **M2-Phase 5 — Reactive engine.** State change → invalidate →
  relayout → render path, building on the queued-emission machinery
  from Phase 6 and the layout invalidation hooks from
  [DD-P8-002](../decisions/phase-8-hello-counter.md).
  Subtree-vs-root dirty granularity is in scope only insofar as M2
  acceptance demands; large-tree optimization stays an open question
  in [layout-engine note §3.4](../notes/layout-engine.md).

- **M2-Phase 6 — `.ui → runtime` lowering.** The end-to-end pipeline
  that consumes M2-Phases 2 / 3 / 5 and produces a running Hello
  Counter from `counter.ui`. Replaces the imperative tree
  construction in `examples/counter-{c,rust,zig}/`.

- **M2-Phase 7 — Reactive Foundation Hardening & Contract
  Finalization.** Discharge the three DDs deferred from Phase 6
  closing (DD-M2-P6-010 / 011 / 012). Phase 6 establishes the
  pipeline (counter `.ui` → runtime, A1/A2); Phase 7 establishes the
  foundation guarantees that distinguish "it runs" from "it is a
  Foundation" (A5/A6). Order of work: 010 (topo sort) → 012 (guard
  placement principle, including `architecture.md` update) → 011
  (String binding end-to-end). The phase closes when all three DDs
  are Accepted and their implementation lands.

### Phase dependencies

```
M2-Phase 1   ── independent infra; lands any time

M2-Phase 2 ─┐
M2-Phase 3 ─┤
            ├─ M2-Phase 4 ─ M2-Phase 5 ─ M2-Phase 6 ─ M2-Phase 7
```

M2-Phases 2 and 3 are decision phases and can run in parallel; both
gate M2-Phase 4. M2-Phase 5 depends on 4. M2-Phase 6 depends on the
decisions (2, 3) and on 5. M2-Phase 7 depends on M2-Phase 6.

### Acceptance ↔ phase mapping

| Acceptance | Phase(s) |
|---|---|
| A1 (`counter.ui` drives all three hosts) | M2-Phase 6 |
| A2 (reactive propagation, no host wiring) | M2-Phase 5, M2-Phase 6 |
| A3 (cdylib-shim cleanup) | M2-Phase 1 |
| A4 (tree-mutation ABI primitives) | M2-Phase 4 |
| A5 (Reactive Foundation Hardening) | M2-Phase 7 |
| A6 (Type-Agnostic Reactive Binding) | M2-Phase 7 |

M2-Phases 2 and 3 are **decision phases** without a direct acceptance
hook; their outputs are ADR-shaped and feed M2-Phases 4 / 6.

### Out of scope (deferred to later milestones)

Items that originally appeared in the M2 Alpha paragraph but are
**not** in M2-as-foundation. Allocation to post-M2 milestones is
recorded in [ROADMAP.md](../../ROADMAP.md):

- Grid / ScrollView / List layout primitives → M3
- DSL spec public draft → M3
- Input handling (kbd / mouse / touch + focus model) → M4
- Multi-window support → M4 (pre-1.0 because of cross-cutting ABI)
- TextField widget → M4 (required by IME verification)
- IME via TSF (Japanese / CJK input) → M4
- AccessKit / UIA accessibility integration → M4
- Mica / Acrylic root-window backdrop, system accent → M4
- VS Code extension (LSP / highlighting / diagnostics) → M5 (parallel
  track may begin once M3 spec draft is agreed)
- Full theming surface, official widget set beyond TextField → M5
- Performance target verification (<100 ms startup, <30 MB memory) → M6
- Polished showcase + ABI freeze + C/Rust/Zig bindings mature → M6
- Hot reload (interpreter mode) → post-1.0; feasibility depends on
  M2-Phase 2's wasamoc output format decision
- Higher-level animation DSL → post-1.0
- Swift / Go bindings → post-1.0 community track

### Risks

- **Decision phases (M2-Phase 2, M2-Phase 3) blocking the DSL
  track.** If 2 / 3 do not converge in pre-doc, M2-Phase 6 cannot
  start. Mitigation: each decision phase is timeboxed to a single
  ADR review cycle; if the question does not converge, escalate to a
  VISION-level ADR (analogous to DD-V-001) rather than re-opening
  pre-doc indefinitely.

- **Reactive engine coupling with layout invalidation.**
  [DD-P8-002](../decisions/phase-8-hello-counter.md) installed a
  coarse "whole-window dirty" path. If M2-Phase 5 demands finer
  granularity for correctness (not performance), the layout-engine
  changes ripple beyond M2-Phase 5's nominal scope.

### Resolved deferrals

The post-M2 questions raised alongside the M2 redefinition were
resolved on 2026-05-02 and are now recorded in
[ROADMAP.md](../../ROADMAP.md), [VISION.md §7](../../VISION.md#7-roadmap),
and [docs/decisions/vision-post-m2-roadmap.md](../decisions/vision-post-m2-roadmap.md)
(DD-V-005..009). Summary:

- Grid / DSL spec public draft → M3
- Post-M2 structure: thesis-driven milestones M3 (DSL surface) /
  M4 (Interaction stack) / M5 (Identity & tooling) / M6 (1.0);
  Alpha / Beta labels dropped
- Multi-window → M4 (pre-1.0, ABI cross-cutting)
- Mica / Acrylic + first showcase → M4 (identity feature
  demonstrable from M4)
- VS Code LSP → M5 acceptance, parallel track from M3 spec draft
- Hot reload → post-1.0
- 1.0 binding list → C / Rust / Zig; Swift / Go → post-1.0 community
- ADR identifier scope `M<N>-P<n>` from M2 onward (see Phase
  numbering above)

### Revision log

- **2026-05-08** — Acceptance criteria revision under the
  README.md "Acceptance criteria revision" exception.
  - Motivation: A1–A4 cover pipeline wiring and structural cleanup
    but do not cover the runtime guarantees (execution order,
    re-entrancy/guard placement, type-agnostic binding) required to
    call M2 a Foundation milestone in a non-trivial sense.
  - Added: A5 (Reactive Foundation Hardening), A6 (Type-Agnostic
    Reactive Binding).
  - Added: M2-Phase 7 (Reactive Foundation Hardening & Contract
    Finalization), depending on M2-Phase 6.
  - DD-M2-P6-010 / 011 / 012 status remains Proposed; their
    discharge is now scoped to Phase 7.

- **2026-05-09** — Phase 7 progress (no acceptance-criteria change).
  - DD-M2-P6-010 (`dirty_effects` topological sort fidelity)
    Accepted in [m2-phase-7-reactive-foundation.md](../decisions/m2-phase-7-reactive-foundation.md#dd-m2-p6-010--dirty_effects-topological-sort-fidelity)
    — Option A (true topological walk in M2; pure-logic unit tests
    on synthetic dependency graphs; single drain code path). M3
    residuals (cycle detection, ordering ties, fan-out × MUTATION_CAP)
    recorded in [m2-to-m3-handover.md](../notes/m2-to-m3-handover.md)
    §3. Implementation step pending; A5 first clause (DD-010 Accepted)
    discharged at design level; second clause (implementation
    reflects it) opens the next step.

- **2026-05-10** — Phase 7 progress (no acceptance-criteria change).
  - DD-M2-P6-010 implementation landed; the production dirty-Effect
    drain no longer relies on `EffectId` numeric order.
  - DD-M2-P6-012 (`re-entrancy / safety-guard placement principle`)
    Accepted in [m2-phase-7-reactive-foundation.md](../decisions/m2-phase-7-reactive-foundation.md#dd-m2-p6-012--re-entrancy-and-safety-guard-placement-principle)
    — Option C (role-specified defense in depth). The principle is
    recorded in [architecture.md](../architecture.md#684-runtime-safety-guard-placement)
    as a global runtime invariant. A5 now has both DDs accepted at
    design level; remaining A5 work is DD-012 implementation alignment
    and focused guard-placement tests.
  - DD-M2-P6-011 (`String`-typed property binding)
    Accepted in [m2-phase-7-reactive-foundation.md](../decisions/m2-phase-7-reactive-foundation.md#dd-m2-p6-011--string-typed-property-binding)
    — Option B (`StrPropRead`). A6 is accepted at design level;
    remaining A6 work is implementation: `.ui` String binding must
    propagate through runtime widget property state while preserving
    existing integer binding behavior.

## Progress

The Progress section is a compact milestone index. Detailed live task
tracking belongs in phase progress files under `docs/plans/progress/`;
completed phase logs are distilled into ADRs, CHANGELOG, notes, and git
history, then deleted by default.

| Phase | Status | Progress file | ADR | Notes |
|---|---|---|---|---|
| M2-Phase 1 - cdylib-shim cleanup | completed | retired | [m2-phase-1-cdylib-shim.md](../decisions/m2-phase-1-cdylib-shim.md) | CHANGELOG entry added; residual notes in `docs/notes/workspace-layout.md` and `docs/notes/cdylib-shim-build-graph.md`. |
| M2-Phase 2 - wasamoc output format decision | completed | retired | [m2-phase-2-wasamoc-output-format.md](../decisions/m2-phase-2-wasamoc-output-format.md) | Option B adopted after IR loader spike. |
| M2-Phase 3 - Handler execution location | completed | retired | [m2-phase-3-handler-exec-location.md](../decisions/m2-phase-3-handler-exec-location.md) | Runtime-side interpreter path accepted; headless verification note filed. |
| M2-Phase 4 - Tree-mutation ABI primitives | completed | retired | [m2-phase-4-tree-mutation-abi.md](../decisions/m2-phase-4-tree-mutation-abi.md) | Stable-core tree mutation ABI landed; CHANGELOG entry added. |
| M2-Phase 5 - Reactive engine | completed | retired | [m2-phase-5-reactive-engine.md](../decisions/m2-phase-5-reactive-engine.md) | Reactive primitives and binding path landed; later drain refinements folded into Phase 6/7 records. |
| M2-Phase 6 - `.ui -> runtime` lowering | completed | retired | [m2-phase-6-ui-lowering.md](../decisions/m2-phase-6-ui-lowering.md) | A1/A2 discharged by the C/Rust/Zig counter migration; CHANGELOG entry added. |
| M2-Phase 7 - Reactive Foundation Hardening & Contract Finalization | in-progress | [progress/m2-phase-7-progress.md](progress/m2-phase-7-progress.md) | [m2-phase-7-reactive-foundation.md](../decisions/m2-phase-7-reactive-foundation.md) | Active phase for A5/A6. DD-M2-P6-010 and DD-M2-P6-012 are implemented; DD-M2-P6-011 is Accepted with implementation pending. |

### Owner-facing resume note

Continue in [progress/m2-phase-7-progress.md](progress/m2-phase-7-progress.md).
DD-M2-P6-011 is Accepted as Option B (`StrPropRead`). The next implementation
step is to land String binding propagation through runtime widget property
state and protect the existing integer binding path.
