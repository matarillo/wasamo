### DD-M2-P6-010 — `dirty_effects` topological sort fidelity

**Status:** Accepted (2026-05-09) in
[m2-phase-7-reactive-foundation.md](./m2-phase-7-reactive-foundation.md#dd-m2-p6-010--dirty_effects-topological-sort-fidelity)
(housing migrated 2026-05-08; resolved Option A — true topological
walk in M2). The Phase 6 ADR's "Forward-compat carry-forward" entry
for DD-010 (line above) refers to the resolved DD; the mandatory
pre-condition for M3 multi-binding is **discharged at acceptance**
(the walk lands in M2; M3 inherits the verified primitive). M3
residuals (cycle detection, ordering ties, fan-out × `MUTATION_CAP`)
are recorded in
[docs/notes/m2-to-m3-handover.md](../notes/m2-to-m3-handover.md).

This DD was opened in the Phase 6 ADR's draft slate and deferred to
M2-Phase 7 per the 2026-05-08 acceptance-criteria revision recorded in
[m2-plan.md](../plans/m2-plan.md)'s Progress section. The full
Context / Options / Recommendation now lives in the Phase 7 ADR; this
stub preserves the section anchor and the Phase 6 chronological record
that the issue surfaced during the DD-M2-P6-001 implementation
retrospective on 2026-05-07. The DD number remains `DD-M2-P6-010` (a
historical surface-time identifier, not a housing-location reference).

---
