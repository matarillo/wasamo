# process/

Development process artifacts organized by milestone and phase.

## Structure

```
process/
  _roadmap.md                    # Overall milestone roadmap (SSOT for acceptance criteria)
  procedures/                    # Operating procedures (retrospective, future: release, PR review, etc.)
    retrospectives.md            # Retrospective procedure SSOT (when/how to run task/phase retros)
  cross-milestone/
    decisions/                   # Vision decision records (doc system, RFC policy, DSL policy, process rules)
      exploration/               # Pre-ADR exploratory notes (resolved); paired with the ADR they fed into
  milestone-N/
    plan.md                      # Milestone execution plan
    handoff.md                   # Cross-phase design prerequisites and residuals (written at milestone close)
    requirements/                # Milestone-level requirements (framing, target-app docs)
    phase-M/
      README.md                  # Phase title, status, and folder guide
      requirements/              # Phase scope agreement and constraints (framing.md, constraints.md)
      decisions/                 # Architectural Decision Records
        preamble.md              # Context, scope, summary, revision history
        dd-NNN-<slug>.md         # One file per decision
        exploration/             # Pre-ADR spikes and exploratory notes for this phase's ADR set
        superseded/              # Superseded ADRs preserved as historical record
      implementation/            # Task plan, execution log, residuals
        preamble.md              # Phase intro and task ordering rationale
        plan.md                  # Task checklist and progress
        log.md                   # Decisions log + CI/verification log
        handoff.md               # Out-of-scope items to carry into the next phase
      retrospectives/            # Phase-end and per-task retrospectives
        phase-end.md
        tN.md                    # Per-task (or dd-NNN.md when DDs are the task unit)
```

## Folder conventions

| Folder | Role | Mutability |
|---|---|---|
| `cross-milestone/decisions/` | Vision decision records | Immutable once accepted |
| `requirements/` | Scope, constraints, specs, wireframes (phase or milestone level) | Frozen at ADR drafting |
| `decisions/` | ADRs | Immutable (supersede rule) |
| `implementation/` | Task plan, execution record, handoff | Mutable during the phase |
| `retrospectives/` | Post-implementation reflection | Append-only after phase close |
| `handoff.md` | Items carried to the next phase or milestone | Written at close |

> `milestone-N/plan.md`'s `Frozen agreement` is **not** read-only under
> `in-progress`; it is revisable under the plan-revision discipline
> ([workflow.md](./procedures/workflow.md), DD-V-026).

## SSOT distribution

Process knowledge is split across five homes by category. Each home owns its category; other files link rather than duplicate.

| SSOT | Owns |
|---|---|
| [`AGENTS.md`](../AGENTS.md) (with `CLAUDE.md` as its Claude-Code `@AGENTS.md` import shim) | Enforceable rules: language, testing, commit, CI, build order, rule-enforcement tiers |
| `process/README.md` (this file) | Structural conventions: folder roles and the mutability of doc categories |
| [`process/procedures/workflow.md`](./procedures/workflow.md) | Development workflow (milestone/phase stages, document status lifecycle including the plan-revision discipline, glossary) |
| [`process/procedures/retrospectives.md`](procedures/retrospectives.md) | Retrospective procedure (task/phase retro, merge gate) |
| `process/cross-milestone/decisions/` | Vision decision records (the *why* behind the conventions) |

## What lives elsewhere

- `docs/architecture.md`, `docs/abi_spec.md`, `docs/dsl_spec.md` — normative technical specs
- `docs/notes/` — owner-authored exploratory notes and open questions (Japanese allowed)
