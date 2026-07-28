# M4-Phase 1 — Implementation log

Append-only mixed log: decisions log (mid-implementation judgments) +
CI / verification log (evidence pointers, run ids). Per-task
implementation-gate selections (start) and close artifacts land here per
[preamble.md §Implementation gates](./preamble.md#implementation-gates).

Three entry kinds this phase carries by obligation, so they are named
here rather than discovered:

- **The T5 / T6 call-site audit table** — DD-002's 13 rows, each with
  its classification, its source location as landed, and the
  verification that closed it.
- **The T7 structural side-effect enumeration** — DD-003's 13 rows, each
  stated as updated or verified-unchanged.
- **Stated limits**, recorded with their reason rather than elided: the
  synthesised-`WM_DPICHANGED` limit (T8) and the trap-#4 disposition for
  the tolerated-declaration-failure branch (T9).

---

## T1 — Pre-implementation spike

### Start gate (recorded 2026-07-28, before any edit)

Read before selecting: [plan.md](./plan.md) §T1, the ADR set (preamble +
DD-001 … DD-004), and
[implementation-gates.md](../../../procedures/implementation-gates.md).

**Trap selection.**

| # | Trap | Applies | Reason |
|---|---|---|---|
| 1 | Semantic-migration miss | **yes** | T1's second obligation *is* a call-site audit: DD-002's 13-row table must be verified against the source, and any coordinate-carrying path the table does not name is a finding. T1 adds no variant itself; it establishes the baseline T5 closes against. |
| 2 | Missed side effects | no | No state or structure change lands on the T1 commit. T1 *scopes* T7's enumeration but does not perform it; performing it here would fabricate an artifact for a change that has not been written. |
| 3 | Parallel/derived data drift | **yes**, documentation analogue | T1's landing artifacts are prose in this log plus revisions to [plan.md](./plan.md), both of which sit alongside the ADR set. The failure mode is restating ADR content instead of citing it, creating a second source of truth for the audit table and the risk register. |
| 4 | Untested authored branch | no | No branch, diagnostic, or reject arm lands. T9's diagnostic branch is explicitly re-decided at T9 per [preamble.md §Implementation gates](./preamble.md#implementation-gates); T1 does not discharge it. |
| 5 | Carry-forward underweighted | **yes** | The carrier shape and the walk site are invariants every task from T2 to T8 must preserve. They are recorded here with re-trigger criteria, not left as tacit context. |
| 6 | Symptom taken at face value | **yes**, low expectation | T1 builds the workspace repeatedly with throwaway edits. A build or test failure that is *not* the expected signature breakage must be root-caused, not reverted past. |
| 7 | Weak GUI evidence | no | T1 renders nothing and launches no host. |

**Review lane.** Normal. T1 lands no production code, so none of the
high-risk classes in
[gates §4](../../../procedures/implementation-gates.md) applies to the
T1 commit itself. The decisions recorded here feed T5 / T6 / T7, which
carry the full independent review.

**Planned proof obligations** (each closed below before T1 ends):

1. Per-file touch-point record for every landing file, from an
   end-to-end read.
2. DD-002's 13 rows verified against the source, with discrepancies and
   unnamed coordinate-carrying paths recorded as findings.
3. The `DipScale` carrier and threading shape, decided once, with the
   breakage set enumerated **by the compiler** — throwaway edit, build,
   record, revert.
4. Where the re-rasterization walk lives and what it re-creates.
5. The sequencing thesis confirmed or revised, task by task from T2 to
   T8.
6. The awareness-declaration site confirmed against `runtime::init()`'s
   one-shot guard.
7. Risk sharpening against source line numbers, plus the T5 and T6 gate
   selections.

**Exit criterion** (spike-specific, per
[spike discipline](../../../procedures/implementation-gates.md) and
[plan.md](./plan.md) §T1): every open point is assigned to a downstream
task **and its scope is seen** — not "no surprises expected".

**Baseline for the revert.** Throwaway edits are made on top of
`8f9e4e3` (T0 closure). `git status` must be clean of `wasamo-*` changes
at the T1 commit.

