### DD-M2-P2-004 — Sequencing relative to M2-Phase 3

**Status:** Accepted

**Context:**
[m2-plan phase dependencies](../../plan.md#phase-dependencies)
says M2-Phase 2 and M2-Phase 3 are parallelizable decision phases,
both gating M2-Phase 4. In practice the two interact:

- DD-M2-P2-001 = Option A (codegen) **forecloses** Phase 3 to
  host-side execution: the handler body lives in host-language
  source, so the host is the only place that can execute it.
- DD-M2-P2-001 = Option B (IR) leaves Phase 3 open: the IR carries
  handler bodies as typed expressions; either the runtime
  interpreter evaluates them or the runtime emits a synthetic
  signal that a host-side trampoline (also generated, or written
  by the binding) handles.
- DD-M2-P2-001 = Option C (runtime parse) is similar to B w.r.t.
  Phase 3.

The reverse direction (Phase 3 outcome constraining Phase 2) is
weaker. A Phase 3 strong preference for runtime-side execution
would argue against Phase 2's Option A; a Phase 3 preference for
host-side leaves all three Phase 2 options viable.

**Options:**

Option A — Sequential: this ADR (Phase 2) lands first; Phase 3 ADR
follows once Phase 2 is Accepted (recommended)

- What you gain: Phase 2's outcome reduces Phase 3's option space
  before Phase 3 review begins. Owner reviews one ADR at a time.
  Lower cognitive load; faster total review time. Matches the
  user's stated preference.
- What you give up: A Phase 3 surprise could in principle force
  Phase 2 reopen. Mitigated by this ADR explicitly enumerating the
  Phase 3 implication of each Phase 2 option (table below) so the
  Phase 2 decision is not made blind.
- **Technical risk: None** (process choice).

Option B — Parallel: both ADRs filed and reviewed together
- What you gain: A coherent joint shape can be reviewed in one
  pass; no risk of "Phase 2 Accepted then Phase 3 reopens it".
- What you give up: Two ADRs in flight at once; doubled review
  surface; risk that Phase 3 disagreements re-litigate Phase 2
  mid-discussion.
- **Technical risk: None** (process choice).

Option C — Joint: one ADR covers both Phase 2 and Phase 3
- What you gain: No artificial split where the questions interact.
- What you give up: Conflates two phases the milestone plan
  separated for a reason. Larger ADR. Future readers lose the
  one-decision-per-ADR property.
- **Technical risk: None** (process choice).

**Recommendation:** **Option A (sequential).**

The interaction is one-directional in practice: Phase 2 → Phase 3.
Sequential capitalizes on this; parallel does not. Joint is
overkill given the questions are conceptually separable.

To insulate the sequential path against a Phase 3 surprise, the
following table records each Phase 2 option's downstream Phase 3
implication. If a future Phase 3 outcome is incompatible with the
Phase 2 option Accepted here, this ADR is reopened (per
[process/README.md supersede policy](../../../README.md)) rather
than Phase 3 silently working around it.

| DD-M2-P2-001 outcome | DD-M2-P3 (handler exec) implication |
|---|---|
| Option A (codegen) | Forecloses to host-side execution. Phase 3 becomes a sequencing/contract refinement, not a real fork. |
| Option B (IR + interpreter) | Both host-side trampoline and runtime-side interpreter remain viable. Phase 3 is a real decision. |
| Option C (runtime parse) | Same as B — both viable. |

---
