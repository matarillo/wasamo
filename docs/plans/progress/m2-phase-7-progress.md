---
milestone: M2
phase: M2-Phase 7
status: active
plan: docs/plans/m2-plan.md
adr: docs/decisions/m2-phase-7-reactive-foundation.md
created: 2026-05-09
---

# M2-Phase 7 Progress - Reactive Foundation Hardening & Contract Finalization

## Scope

Discharge the three DDs deferred from Phase 6 closing. Phase 6 established
the `.ui -> runtime` pipeline (A1/A2); Phase 7 establishes the foundation
guarantees that distinguish "it runs" from "it is a Foundation" (A5/A6).

The DDs retain their `DD-M2-P6-NNN` numbering as historical surface-time
identifiers. The Phase 6 ADR retains stub anchors that forward to the
Phase 7 ADR.

## ADR

[docs/decisions/m2-phase-7-reactive-foundation.md](../../decisions/m2-phase-7-reactive-foundation.md)
houses DD-M2-P6-010 / 011 / 012.

Current status:

- DD-M2-P6-010: Accepted and implemented on 2026-05-09.
- DD-M2-P6-011: Accepted on 2026-05-10; implementation pending.
- DD-M2-P6-012: Accepted and implemented on 2026-05-10.

## Order of Work

1. **DD-M2-P6-010 - dirty_effects topological sort fidelity**
   - [x] Re-run pre-doc with full Phase 6 implementation evidence.
   - [x] Flip agreement to Accepted.
   - Implementation notes: [docs/notes/m2-phase-7/dd-010-implementation-notes.md](../../notes/m2-phase-7/dd-010-implementation-notes.md).
   - [x] Replace EffectId-numeric-order approximation with true graph walk.
   - [x] Add pure-logic unit tests on synthetic dependency graphs.

2. **DD-M2-P6-012 - re-entrancy / safety-guard placement principle**
   - [x] Re-run pre-doc with full Phase 6 implementation evidence.
   - [x] Flip agreement to Accepted.
   - [x] Record the principle in `docs/architecture.md` as a global
     runtime invariant.
   - [x] Reflect the principle in implementation.

3. **DD-M2-P6-011 - String-typed property binding**
   - [x] Re-run pre-doc.
   - Pre-doc framing: [docs/notes/m2-phase-7/dd-011-pre-doc-framing.md](../../notes/m2-phase-7/dd-011-pre-doc-framing.md).
   - [x] Flip agreement to Accepted.
   - [ ] Add the String read surface to `EvalContext` and route
     `BindingEvalContext` tracked String reads through
     `SignalRegistry.strings`.
   - [ ] Add the accepted String property-read representation
     (`HandlerExpr::StrPropRead`) and dispatch it to the tracked String
     read path.
   - [ ] Wire a real `.ui` / emitted-IR path into the String read form based
     on declared state type; hand-written `StrPropRead` tests alone do not
     discharge A6.
   - [ ] Demonstrate `.ui` String binding through runtime widget property
     state without adding a new Visual Layer CI fixture.
   - [ ] Regression-protect existing integer `PropRead`, integer
     interpolation, and counter-style handler mutation behavior.
   - [ ] Record any implementation deviations in a DD-011 retrospective and
     update the appropriate higher-level document if the accepted design
     assumptions change.

## Acceptance Discharged Here

- **A5 - Reactive Foundation Hardening.** DD-M2-P6-010 and DD-M2-P6-012
  must be Accepted with implementation landed; the guard-placement
  principle must be recorded in `docs/architecture.md`.
- **A6 - Type-Agnostic Reactive Binding.** DD-M2-P6-011 must be Accepted
  with end-to-end `Signal<String>` binding demonstrated.

## Verification

- Unit tests for topological sort fidelity.
- Unit tests for guard-placement enforcement.
- Automated runtime-state tests for String binding propagation and integer
  binding regression.
- Existing Phase 6 GUI counter regression check. No new GUI fixture is
  mandated unless a DD demands it.

## Closing Items

- [ ] `cargo build --release --workspace` green.
- [ ] `cargo test --workspace` green.
- [ ] `CHANGELOG.md` Phase 7 entry added.
- [ ] `ROADMAP.md` M2 marked shipped.
- [ ] `docs/plans/m2-plan.md` status changed to `completed`.
- [ ] This phase progress file distilled and deleted or archived.

## Owner-Facing Notes

DD-M2-P6-010 and DD-M2-P6-012 are Accepted and implemented. DD-M2-P6-011
is Accepted with Option B (`StrPropRead`) after pre-doc framing; implementation
is the remaining A6 work. DD-M2-P6-012 implementation alignment added focused
guard-placement tests for both the ABI diagnostic boundary and the internal
`drain_if_outermost` invariant boundary.
