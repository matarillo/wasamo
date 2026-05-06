---
title: Architectural family — current hypothesis status
status: live
created: 2026-05-05
related-adrs:
  - docs/decisions/m2-phase-5-reactive-engine.md
  - docs/decisions/m2-phase-2-wasamoc-output-format.md
  - docs/decisions/m2-phase-3-handler-exec-location.md
  - docs/decisions/phase-6-c-abi.md
related-notes:
  - docs/notes/headless-verification.md
---

# Architectural family — current hypothesis status

## Why this note exists

Several reactive GUI architectural families exist in current practice:
tree-with-bindings (Slint, QML/Qt), view-function with re-execution
(SwiftUI, Jetpack Compose), coarse subtree-rebuild (Flutter
`setState`), and manual property notification (WPF / WinUI
`INotifyPropertyChanged`). The choice shapes the reactive engine, the
IR semantics, the ABI surface, and the binding grammar in the DSL.

Wasamo has not made an explicit family-level decision. The accepted
ADRs (VISION's DSL × C ABI thesis; DD-M2-P2-001 = B textual IR as a
tree description; DD-P6-001..007 handle-based stable core;
DD-M2-P3-001 = A handler on internal `set_property`) cumulatively fit
the tree-with-bindings family best, but the cumulative effect was not
the result of a deliberate family-level vote. This note records that
distinction so the implicit framing does not silently calcify into
commitment.

**Status: live working hypothesis, not a long-term direction.** The
owner retains full latitude to revisit at any phase boundary; this
note is the working description, not a ratification. Public readers
should read it as "current best account of where wasamo sits", not as
"wasamo's architectural commitment".

## The four families

### (1) Tree-with-bindings (Slint, QML/Qt)
- Tree described declaratively in DSL; bindings are annotations on
  tree nodes; the engine evaluates them in place. Tree-shape changes
  (conditional / for-loop) are themselves binding nodes rebuilding
  local subtrees.
- Reactivity primitive: Signal-equivalent cells with read-time
  dependency tracking; Effects bound to tree nodes.
- 30+ years of native-GUI deployment (Qt → QML → Slint).

### (2) View-function with re-execution (SwiftUI, Jetpack Compose)
- UI is a function over state; on state change the function
  re-executes and a new tree representation is diffed against the
  old.
- Reactivity primitive: still Signal-equivalent at the bottom
  (`@Observable`; `mutableStateOf` + snapshot system), but layered
  with a "scope" abstraction (view body / composable function) that
  re-executes as the unit.
- Strong fit when the runtime has access to in-process language
  metadata (Swift reflection; Kotlin compiler plugins). Awkward
  across a C ABI boundary: a "view function" is hard to express as a
  stable handle-based primitive.

### (3) Coarse subtree-rebuild (Flutter `setState`)
- State change marks a subtree dirty; the framework rebuilds the
  subtree on the next frame.
- No fine-grained dependency tracking; rebuild scope is the declared
  `setState` boundary. Equivalent to the rejected DD-M2-P5-001
  Option A in M2.

### (4) Manual property notification (WPF / WinUI)
- Reactivity is opt-in per property; bindings subscribe via
  property-changed events (`INotifyPropertyChanged`).
- Mature on Windows but coarser and more verbose than the others.
  Listed for completeness.

## How wasamo's accepted decisions currently align

The cumulative effect of accepted decisions positions wasamo within
family (1). The table below is descriptive of the current state, not a
commitment:

| Accepted decision | Effect on family alignment |
|---|---|
| VISION DSL × C ABI thesis | Hosts work in widget handles, not view functions; handle-based ABI fits family (1) most directly |
| DD-M2-P2-001 = B (textual IR + runtime interpreter) | IR is a tree description; family (2) would require IR semantics covering view-function execution |
| DD-P6-001..007 (stable-core C ABI) | Handle-based; no scope / view-body primitive |
| DD-M2-P3-001 = A (handler runs runtime-side) | Handler mutates persistent property storage; family (2) handler would update state to trigger view re-execution |
| DD-M2-P5-001..006 (Phase 5, Accepted) | Signal + Effect on a tree with `BindingTarget` enum; family (1)-flavored API surface |

No accepted ADR names family (1) as the long-term selection. This
note does not upgrade the implicit fit to an explicit commitment.

## Family-neutral vs family-coupled parts of the current design

Treating the current design as a hypothesis means asking: which parts
would survive a family-level pivot, and which would not?

**Family-neutral (survive any pivot):**
- Signal + Effect 2-layer with read-time auto-tracking
  (DD-M2-P5-001/002). All four families either use this directly
  (1, 2) or can be expressed as a degenerate case of it (3).
- Outermost-frame deferred dispatch (DD-M2-P5-004). Composes with any
  family's flush model.
- Effect lifetime via owner Drop (DD-M2-P5-003). Lifetime management
  of observable closures is family-orthogonal.

**Family-coupled but internal (survive pivot via internal refactor):**
- `BindingTarget` enum and `register_binding()` API (DD-M2-P5-005).
  Encodes the "binding lives on a tree node" assumption. `pub(crate)`
  and not exposed across the C ABI; M3+ could replace its shape if a
  pivot occurs.
- Phase 5 dependency-graph storage (per-Signal dependent-Effect set).
  Family (2) might prefer scope-keyed rather than effect-keyed;
  refactor surface is internal.

**Family-coupled at the public-contract boundary (pivot forces ABI /
IR / DSL revision):**
- C ABI as handle-based (DD-P6-001..007). Family (2) would need a
  fundamentally different host interface (e.g. an embedded scripting
  runtime), not a refactor.
- Textual IR as tree description (DD-M2-P2-001/002). Family (2) would
  need IR semantics covering view-function execution.
- DSL grammar (M3, not yet drafted). The grammar the M3 DSL spec
  settles on is the most committal family-level choice forthcoming.

The practical content of "long-term policy is not yet committed": the
public-contract layer is currently family (1)-shaped, but each
accepted decision has its own supersede path, and the M3 DSL spec is
the natural place to either confirm or revise the family direction
with the design force visible (grammar in hand).

## Why this matters for Phase 5

Phase 5 is the first phase where architectural family becomes engine-
shape-relevant. The Phase 5 ADR (DD-M2-P5-001..006, Proposed) is
framed inside the current hypothesis: it picks Signal + Effect
2-layer because that primitive is family-neutral, and it picks the
Slint/QML-flavored `BindingTarget` API because the current cumulative
state of accepted decisions points there.

If a future family-level revision shifts wasamo away from family (1),
Phase 5's primitives stay; only the `BindingTarget`-shaped internal
API changes. This bounds the cost of the implicit framing.

## Re-evaluation triggers

Re-read this note (and consider upgrading to a vision ADR or revising
the alignment table) when any of the following occur:

1. **M3 DSL spec drafting begins.** The grammar choices made during
   M3 spec work are the most committal family-level decisions wasamo
   will face. Re-read before starting; if the grammar pushes toward
   family (2) (function-call composition with re-execution
   semantics), the family selection becomes a real decision and may
   warrant a vision ADR.
2. **Hot reload work begins (post-1.0).** Hot reload composes
   differently with each family. Family (2)'s view-function
   re-execution makes hot reload structurally cheap; family (1)'s
   tree mutation requires a "tear down old tree, build new tree"
   path. Re-read when hot reload is taken off the post-1.0 deferral
   list.
3. **A binding feature is proposed that does not fit
   `BindingTarget`.** If a future binding shape (e.g. a derivation
   spanning multiple subtrees) cannot be expressed as a
   `BindingTarget` variant cleanly, the tree-with-bindings framing is
   straining; re-read and consider whether to revise within family
   (1) or pivot.
4. **Owner explicit revisitation.** Any time the owner wants to
   revisit family-level direction, regardless of phase boundary.

When upgrading: a vision ADR (DD-V-NNN scope) records the explicit
family commitment; this note's status changes to `superseded by
docs/decisions/vision-architectural-family.md`. When revising in
place: update the alignment table and re-evaluation triggers; commit
normally.

## References

- Phase 5 ADR — [m2-phase-5-reactive-engine.md](../decisions/m2-phase-5-reactive-engine.md) (Accepted 2026-05-05)
- DSL × C ABI thesis — [VISION.md](../../VISION.md)
- IR shape — [m2-phase-2-wasamoc-output-format.md](../decisions/m2-phase-2-wasamoc-output-format.md)
- Stable-core C ABI — [phase-6-c-abi.md](../decisions/phase-6-c-abi.md)
- Handler runtime location — [m2-phase-3-handler-exec-location.md](../decisions/m2-phase-3-handler-exec-location.md)
