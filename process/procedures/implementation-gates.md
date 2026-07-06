# Implementation task gates (start gate + close gate)

> [AGENTS.md](../../AGENTS.md) is the enforceable-rules SSOT; **this file
> is the triggered implementation-gate surface** it points to — it owns the
> failure-mode catalog and the close-gate artifacts as operative gate
> content. An agent (Codex / Claude) uses it at **both task start and task
> close**. Design and origin:
> [rule-enforcement discipline](../cross-milestone/decisions/agents-md-and-rule-enforcement.md)
> (DD-V-025).
>
> Core principle: **build + existing tests green is necessary but not
> sufficient for "done".** Most failures come not from a missing rule but
> from a rule that *did not fire at the moment the implementation approach
> was chosen*. So the traps are read **before** committing to an approach,
> not as a checklist at completion.

---

## 0. How to run this template

1. **Start gate** (§1): before writing code, read the failure-mode
   catalog, select the traps **relevant to this task**, and **record that
   selection** (§1 start-gate artifact). Fold the selected traps into the
   design — if the intended approach trips one, reconsider the approach.
2. **Implement**: keep the selected traps in view. The moment you find you
   *cannot decide the approach on your own*, escalate (§3) — escalate
   before committing to a structure, not only after finishing.
3. **Close gate** (§2): for each trap that applied, produce an
   **auditable artifact**. An abstract "checked" is not acceptable; make
   it something a reviewer / owner / CI can verify against ground truth.
4. **Review lane** (§4): take the full review or the branch/test-focused
   review as classified.

---

## 1. Start gate — failure-mode catalog (select what applies, before coding)

For each trap, decide whether it applies; close every applying trap with
its close-gate artifact. **When a new trap is found, add it to this
catalog (§5).**

| # | Trap (failure mode to avoid) | Applies when |
|---|---|---|
| 1 | **Semantic-migration miss** — when an enum / schema gains a variant or field, audit **every traversal call-site** and classify it. A filter helper that silently drops the new variant slips past validation and counting alike. | enum / IR / schema type change; new variant |
| 2 | **Missed side effects** — on a state / structure change, **enumerate every derived effect** (layout invalidation, Visual sibling order, parent-owned metadata, …); do not implement only the visible mutation. | tree-structure change, state change, insert/remove |
| 3 | **Parallel/derived data drift** — a parallel vector / map / index must be updated **atomically inside the same primitive** that mutates its source. For **documentation tasks**, the analogue is a second source of truth in derived prose (a CHANGELOG entry, candidate ledger, or handoff restating spec / handoff content instead of citing the owning document); enumerate and close it under the trap-#2 side-effect enumeration (M3-Phase 8 T9 origin). | parallel vectors, derived indices, caches; doc tasks that summarize other documents |
| 4 | **Untested authored branch** — a newly written reject / diagnostic / size branch must ship with a test that **fires it**. "Covered incidentally by another test" is not enough. | adding a branch / diagnostic / size arm |
| 5 | **Carry-forward underweighted** — an implementation invariant a later task could trip must be recorded as carry-forward **with evidence and a re-trigger criterion**, even when no ADR changes. | a change where later tasks must preserve an ordering / identity / validation rule |
| 6 | **Symptom taken at face value / flake-rolling** — never re-roll a deterministic (or ≥2× recurring) failure to green and call it a flake. Minimal repro → root cause (dump if needed) → disposition. | recurring failures, failures that vanish on retry, AV / crashes |
| 7 | **Weak GUI evidence** — for a task whose evidence is GUI rendering, launch / process-survival is not enough. Need **screenshot + analysis + a positive control** (a single static frame a wrong implementation could also produce is not evidence). Does not replace the owner smoke. | tasks whose deliverable is GUI-host rendering |

Selected traps (example checklist):

```
- [ ] #1 semantic migration   - [ ] #2 side effects   - [ ] #3 parallel data   - [ ] #4 branch tests
- [ ] #5 carry-forward        - [ ] #6 root cause     - [ ] #7 GUI positive control
```

**Start-gate artifact (required).** Record — in a durable, reviewable
place: the task log, the first implementation commit message, or the PR
body — the traps selected, a one-line reason for each trap judged **not**
applicable, and the review lane (§4). This makes the
*selection itself* auditable — a reviewer can catch a trap wrongly marked
non-applicable, which is exactly the Phase 6 failure mode (a missed
"which trap applies"). "None applies" is allowed only with an explicit
reason.

---

## 2. Close gate — auditable artifacts (close each trap that applied)

For each applying trap, put a **concrete artifact** in the completion
report.

- **#1 call-site audit table** — the `rg` query used, the files covered,
  each call-site's classification (must-dispatch / ignore-OK) and **its
  reason**, and the tests added or deliberately not added. Prefer
  compile-error-forcing shapes for semantic migrations where Rust can
  enumerate the breakage (for example, adding a non-exhaustively matched
  enum variant or changing a field type), but still grep for wildcard /
  filtering helpers that can silently absorb the new case.
- **#2 structural side-effect enumeration** — the derived state touched
  (layout dirty / Visual order / parent metadata / …) and how each was
  updated.
- **#3 parallel-data sync** — which parallel structure was updated, in
  which primitive, atomically (may be folded into the #2 artifact).
- **#4 branch tests** — the test name per added branch (one that fires it
  directly).
- **#5 carry-forward** — where it is recorded (plan / log / handoff) and
  its re-trigger criterion.
- **#6 deterministic-failure rerun / disposition** — the failure's rerun
  history and disposition; a bare "green on retry" is not acceptable. Link
  the root cause or the tracked known-issue.
- **#7 GUI evidence** — screenshot, analysis, and a **positive control**
  (resize and watch a ratio hold / check the source for what is missing /
  toggle the state — an action that distinguishes the intended behavior
  from a look-alike). Capture mechanics (CopyFromScreen not PrintWindow,
  per-monitor DPI, visible desktop) are in
  [docs/notes/verification-environments.md](../../docs/notes/verification-environments.md)
  §Observation 4.

---

## 3. Escalation (when you reach a judgment limit)

If you find the approach trips a trap but **cannot decide it on your
own**, consult the owner or the other agent without waiting to finish.
**Consultation with the owner is in Japanese** (per the project language
rule — chat with the owner is Japanese); the implementation artifacts and
this template stay in English. Novel, un-cataloged traps (a first-time
failure class) cannot be caught by self-check, so the §4 review and the
§5 learning loop are the backstop.

---

## 4. Review lane (before merge)

- **High-risk classes** (schema / IR migration, runtime structural
  change, GUI-render evidence) — require a **full independent review**
  (transitional; revisited after ≥2 phases of evidence).
- **Diagnostic / reject / size branch additions only** — no full review
  needed, but the **branch/test-focused review** of trap #4 is still
  required (until replaced by a concrete CI check). "No full review" is
  not "no review".

If a high-risk change *also* adds diagnostic / reject / size branches, the
full review **must include** the trap #4 branch/test-focused check — the
lanes compose, they are not exclusive.

---

## 5. Learning loop (grow the catalog)

When a review or retrospective surfaces a new failure class, **add it to
the §1 catalog** as a trap (recurrence prevention). This shrinks, over
time, the novel residual that self-check cannot catch.

Lifecycle boundary: adding a concrete example to an existing trap is a
**minor edit** (edit here directly). A genuinely new gate, a new review
obligation, or a tier change is a **structural change** — it goes through
the Process rule lifecycle (a vision decision record) per
[AGENTS.md](../../AGENTS.md#process-rule-lifecycle), not a silent catalog
edit.
