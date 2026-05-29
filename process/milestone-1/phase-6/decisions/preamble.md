# Phase 6 — C ABI Header: Architecture Decisions

**Phase:** 6 (C ABI header — `wasamo.h` + `docs/abi_spec.md`)
**Date:** 2026-04-30
**Status:** Accepted and implemented (2026-04-30)

## Context

Phase 6's acceptance criterion (from
[VISION §7 M1](../../VISION.md#7-roadmap--milestones)) is:
**"Minimal C ABI header"** sufficient to validate the core hypothesis
(external DSL × C ABI × Visual Layer) by running "Hello Counter" in
C, Rust, and Zig at Phase 8. The header is *minimal*, not *frozen*:
M4 is when ABI stability commitments begin.

Two pre-pre-doc framing decisions (Accepted 2026-04-29, recorded in
[../../ROADMAP.md §Phase 6](../../ROADMAP.md)) precede this ADR:

1. **Two-layer `abi_spec.md`.** The spec is partitioned into a
   **stable core** (M4 freeze candidate) and an **M1 experimental**
   layer. The experimental layer exists only because M1 `wasamoc` is
   parser-only — host code must imperatively construct widget trees
   until M2 codegen lands. Marking it `WASAMO_EXPERIMENTAL` keeps
   M1 stopgap shapes from leaking into long-term ABI commitments.

2. **Two deferred questions.** Phase 6 explicitly does **not** decide:
   - **(a)** Where DSL inline handler bodies (`clicked => { … }`)
     execute — host-side trampoline vs runtime-side interpreter.
   - **(b)** `wasamoc`'s M2 output format — host-language codegen vs
     IR + runtime interpretation.
   The stable core is sized to survive either resolution of (a) and
   (b). Decisions in this ADR that would presuppose either are
   deliberately scoped down or pushed into the experimental layer.

The current C ABI surface
([wasamo/src/lib.rs:62-114](../../wasamo/src/lib.rs#L62-L114))
is five functions: `wasamo_init`, `wasamo_window_create`,
`wasamo_window_show`, `wasamo_window_destroy`, `wasamo_run`. They
are the seed for the stable core but predate this ADR's framing —
error convention, threading contract, and string-encoding rules
need to be specified, not just inherited.

The six decisions below correspond to the Phase 6 ROADMAP checklist
([../../ROADMAP.md L122-L125](../../ROADMAP.md#L122-L125)):
stable-core scope / signal model / callback contract /
threading and re-entrancy / error convention / header generation
method.

---

## Summary of recommended decisions

| ID | Topic | Recommendation |
|---|---|---|
| DD-P6-001 | Stable-core scope | Option A — five-area minimum (lifecycle / window+loop / property R/W / observer / signal) |
| DD-P6-002 | Signal model | Option A — string-keyed, tagged-value payload |
| DD-P6-003 | Callback contract | Option A — `(fn, user_data, destroy_fn)` + token + queued emission |
| DD-P6-004 | Threading | Option A — strict UI-thread affinity, defer `wasamo_post` |
| DD-P6-005 | Error convention | Option A — `WasamoStatus` enum + out-params + thread-local last-error message |
| DD-P6-006 | Header generation | Option A — hand-written `wasamo.h`, CI-verified |
| DD-P6-007 | DLL boundary | `WASAMO_EXPORT` via `WASAMO_BUILDING_DLL`; `WASAMO_API = __cdecl`; Option A for memory (runtime owns, bounded lifetime) |

Once Accepted, this ADR's status moves to **Accepted** and the next
artifact (`docs/abi_spec.md` initial draft) is written against
these decisions.
