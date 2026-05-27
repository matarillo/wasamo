# M2-Phase 1 — cdylib-shim cleanup: Architecture Decisions

**Phase:** M2-Phase 1 (cdylib-shim cleanup)
**Date:** 2026-05-03
**Status:** Accepted (2026-05-03)

## Context

M2 acceptance criterion **A3** (see
[ROADMAP.md M2](../../ROADMAP.md#m2-foundation),
[m2-plan.md](../plans/m2-plan.md#frozen-agreement)):

> `wasamo-runtime` and the `wasamo` safe wrapper no longer share an
> rlib filename through the cdylib-shim split; the post-M1 cleanup
> flagged in [DD-P7-002](./phase-7-language-bindings.md) is discharged.

The post-M1 implementation note in DD-P7-002 records the symptom and
the planned shape of the long-term fix. [`architecture.md §11.4`](../architecture.md)
sketches the same shape:

> A cdylib-shim crate (`wasamo-dll`) that depends on `wasamo-runtime`
> (rlib-only, renamed to `wasamo_runtime`) will restore the separation
> cleanly — `wasamo-dll` emits `wasamo.dll` without an rlib, so no
> collision with the safe wrapper's rlib. The Phase 2-5 dev examples
> can be re-introduced under a `wasamo-poc` workspace once that
> refactor is complete.

That sketch is at the level of *what to do*, not *how to do it*. This
ADR resolves the design questions that come up when actually doing it,
in the order they constrain each other. DD-M2-P1-001 (does the shim
exist at all?) gates the rest; DD-M2-P1-002 (naming) follows; the
remaining three are subordinate but each is a real fork.

The acceptance lens is narrow: A3 is a *structural* criterion. Pre-doc
discipline says the phase is done when the rlib-collision class is
gone, not when every conceivable cleanup has happened. Decisions below
are framed against that lens; speculative scope (e.g. resurrecting
Phase 2-5 examples) is treated as out-of-scope unless A3 demands it.

---

## Out of scope (for M2-Phase 1; recorded explicitly)

- **Resurrecting Phase 2-5 dev examples on main.** Mechanism enabled
  by this phase; experimental branch created after main lands; formal
  resurrection deferred (DD-M2-P1-003).
- **Renaming any public crate (`wasamo`, `wasamoc`, `wasamo-sys`).**
  DD-P7-002's naming is settled; this phase does not re-open it.
- **Changes to `wasamo.h`, ABI symbol names, or DLL filename.** All
  preserved by construction.
- **Adding new ABI symbols.** A4 (M2-Phase 4) territory.
- **Workspace-wide `crates/` reorganisation.** Recorded as an open
  question in [`docs/notes/workspace-layout.md`](../notes/workspace-layout.md).

## Summary of Accepted decisions

| ID | Topic | Decision |
|---|---|---|
| DD-M2-P1-001 | Cdylib shim existence/shape | Option A — two-crate split (`wasamo-runtime` rlib-only + `wasamo-dll` cdylib shim) |
| DD-M2-P1-002 | Naming | Option A — keep `wasamo-runtime` name; shim = `wasamo-dll`; `[lib].name = "wasamo_runtime"` on rlib, `"wasamo"` on cdylib |
| DD-M2-P1-003 | Phase 2-5 examples | Option B (main) — defer; experimental branch `exp/m2-p1-poc-examples` after main lands |
| DD-M2-P1-004 | Shim location | Option A — top-level `wasamo-dll/`; `crates/` question deferred to `docs/notes/workspace-layout.md` |
| DD-M2-P1-005 | ABI symbol propagation | Option A — `+whole-archive` via build.rs; local SSH dev box verification required |
| DD-M2-P1-006 | Build-order edge for cdylib consumers | Option A — add `wasamo-dll` to `[dependencies]` of `bindings/rust-sys/Cargo.toml`; `no linkable target` warning accepted as deferred (see `docs/notes/cdylib-shim-build-graph.md`) |

Implementation task list: see
[`docs/plans/m2-plan.md` — M2-Phase 1 Progress](../plans/m2-plan.md#m2-phase-1--cdylib-shim-cleanup).
