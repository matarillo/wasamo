# Vision Decision Record — AGENTS.md as the cross-agent rules SSOT, and a rule-enforcement discipline

**Status:** Accepted 2026-06-06

**Scope:** `CLAUDE.md` → `AGENTS.md` (the enforceable-rules SSOT named by
DD-V-019), the root `CLAUDE.md` import shim, `process/README.md` §SSOT
distribution; and a rule-enforcement discipline that extends the
rule-change lifecycle of DD-V-020. Surfaced by M3-Phase 6, the first
phase in which an agent other than Claude Code carried implementation
work.

**Background.** Wasamo is developed with two agents: Claude Code and
Codex. Through M3-Phase 5 the division of labour was stable — Claude
implemented, Codex reviewed — and the conventions in `CLAUDE.md` (most of
all the testing rules) held, because the agent writing the code was the
one the file was named for and auto-loaded into. M3-Phase 6
**experimentally moved implementation tasks to Codex**, and the
conventions that had held under that arrangement stopped holding: several
of the testing and completion rules were not followed, producing the
defects analysed in this phase's `implementation/log.md` and task
retrospectives (T1–T5).

Read critically, that outcome is two separable failures, not one:

1. **The rules may never have been in the implementing agent's context.**
   `CLAUDE.md` is named for Claude Code and auto-loaded only by it; Codex
   does not read it natively (it follows the cross-tool `AGENTS.md`
   convention). So "Codex broke the testing rules" partly reduces to
   "Codex was never reliably handed the testing rules." That is an SSOT
   *placement* problem, addressed by DD-V-024.
2. **Even rules that are read are context, not a gate.** The rules that
   did reach the implementing agent were standing prose, not an enforced
   check, and every M3-Phase 6 defect passed build and the existing tests
   — so "green == done" lapses slipped through to later review rather than
   being caught by the author. *Why a written rule fails to bind an
   agent, and what to do about it,* is a question DD-V-020 does not
   currently answer: DD-V-020 governs *when* a rule change must be
   recorded, not *how strongly the rule is enforced* once written. That
   gap is addressed by DD-V-025.

This VDR settles both: a single cross-agent SSOT both agents are routed
through (DD-V-024 — with an in-session read confirmation, since routing is
not the same as reading), and a discipline that makes "how is this rule
enforced" an
explicit, recorded part of adding any rule, biasing the defense toward
mechanism and required artifacts rather than after-the-fact review
(DD-V-025).

**Honest limits, recorded up front so the decisions are not over-sold.**
Neither change is a guarantee. Only hard-tier rules (a hook or CI check)
truly block a bad action; everything softer raises adherence without
ensuring it (CLAUDE.md/AGENTS.md is context, not machinery —
[Claude Code docs — memory](https://code.claude.com/docs/en/memory)). And
moving rules up the enforcement ladder trades some implementation
velocity for fewer review-cycle escapes — M3-Phase 6 is the evidence that
review-only defense lands that cost too late, so the trade is taken
deliberately rather than by default.

Both changes are structural per
[DD-V-020](./process-rule-ssot.md): DD-V-024 touches another SSOT (the
DD-V-019 distribution table) and DD-V-025 adds a new rule-lifecycle
category, so they are recorded here before the SSOTs are edited.

## DD-V-024 — AGENTS.md is the cross-agent enforceable-rules SSOT

**Status:** Accepted

**Context:**
[DD-V-019](./process-rule-ssot.md) named `CLAUDE.md` as the SSOT for
enforceable rules (language, testing, commit, CI, build order,
retrospective, process-rule lifecycle). That was correct while Claude
Code was the only agent writing code and Codex only reviewed. Once
M3-Phase 6 moved implementation to Codex, the naming became load-bearing
in a way it had not been before: Codex — like Cursor, Amp, and others —
standardises on `AGENTS.md`, not `CLAUDE.md`, while Claude Code
conversely only auto-loads `CLAUDE.md` (and `CLAUDE.local.md` /
`.claude/rules/`) and does **not** natively read `AGENTS.md`
([Claude Code docs — memory §AGENTS.md](https://code.claude.com/docs/en/memory)).
The conventions therefore lived in a file the implementing agent did not
read. Splitting them into two files to fix that would re-create the exact
drift DD-V-019 was written to prevent.

**Decision:** The enforceable-rules SSOT moves to `AGENTS.md`. The root
`CLAUDE.md` becomes a thin shim that imports it:

```markdown
@AGENTS.md
```

with any Claude-Code-specific instructions appended below the import (none
at adoption time). This keeps Claude Code reading the full conventions
(it loads `CLAUDE.md`, which expands `@AGENTS.md` at session start) while
Codex and other agents are routed to `AGENTS.md` directly (expected to
read it, subject to the in-session confirmation below). One file is the
SSOT; no convention text is duplicated.

On Windows a symlink requires Administrator or Developer Mode, so the
`@AGENTS.md` import — not `ln -s AGENTS.md CLAUDE.md` — is the adopted
mechanism (the primary dev environment is Windows).

Placement is necessary but not sufficient: Claude Code auto-loads its
shim, but Codex's in-session loading depends on the prompt / IDE / tool
surface, so `AGENTS.md` existing does not guarantee Codex read it this
session. When implementation is delegated to Codex, the request must
therefore **confirm at task start that `AGENTS.md` and the relevant
triggered checklist were read** (the same checklist is closed out at task
end, per DD-V-025) — otherwise the "never in the agent's context" failure
(Background §1) simply recurs at a new filename.

This **amends the SSOT distribution table** of
[DD-V-019](./process-rule-ssot.md): the "enforceable rules" home is now
`AGENTS.md`, with `CLAUDE.md` as its Claude-Code import surface. DD-V-019
is immutable; this VDR supersedes that single row. The living table in
`process/README.md` §SSOT distribution is updated in the same commit
batch that flips this VDR to `Accepted`.

**Rationale:** A single cross-agent SSOT removes the two-file drift risk
while preserving Claude Code's load path through the documented import
mechanism. Naming follows the cross-tool convention the other agents
already use, rather than asking every other tool to learn a
Claude-specific filename.

## DD-V-025 — Rule-enforcement discipline (when adding a rule, decide how it is enforced)

**Status:** Accepted

**Context:**
[DD-V-020](./process-rule-ssot.md) defined a two-tier lifecycle for
*changing* process rules — minor edits in place, structural changes via a
vision decision record. It governs *whether a change is recorded*, but it
is silent on *whether the resulting rule will actually be followed*.
M3-Phase 6 is the counter-example that exposes the gap: the rules
existed, were arguably well written, and were still not followed once the
implementing agent changed. Reviewing the failures, three things were
true of every rule involved — they were **soft** (standing prose the
agent is asked to read), **un-triggered** (nothing required the check at
the moment of completion), and **self-attested** (no independent or
mechanical confirmation). Those three properties, not the wording, are
why "green == done" lapses reached review instead of the author.

**Decision:** Extend the DD-V-020 rule-change lifecycle so that adding or
changing a process rule also requires deciding and recording **how the
rule is enforced**, using the discipline below. This **amends DD-V-020**;
DD-V-020 is immutable, so this VDR supersedes that addition, and the
living lifecycle text (`CLAUDE.md` / `AGENTS.md` §Process rule lifecycle
and `process/README.md`) is updated in the Accept commit batch.

*Enforcement tiers — assign the strongest feasible to each rule:*

- **Hard** — a hook or CI check that blocks regardless of what the agent
  decides. The only true enforcement.
- **Forcing** — an **auditable artifact** a reviewer, owner, or CI can
  check against ground truth: e.g. a call-site table that cites the `rg`
  query used, the files covered, the reason for each classification, and
  the tests added or deliberately not added. An abstract "checked: yes"
  is *not* Forcing — an LLM can emit a plausible-looking field without
  doing the work, so a Forcing rule must produce something falsifiable by
  someone other than the author.
- **Soft** — standing prose the agent is asked to read and follow.
  Necessary for irreducible judgment, but the weakest rung and the
  M3-Phase 6 default.

*Caveat for LLM agents:* a Forcing artifact only binds if it is actually
audited — an unaudited artifact decays into self-attestation. So in
practice the ladder for an LLM is **Hard / (auditable artifact + an audit
step) / Soft** — roughly two-and-a-half rungs, not three, and the middle
rung presupposes a reviewer or CI, not just a template field.

*Discipline when adding or changing a rule:*

1. **Diagnose the failure the rule targets** before choosing a tier — was
   the gap "not in the agent's context", "in context but didn't fire at
   the decision moment", or "a judgment the agent lacked"? The remedy
   differs; a rule aimed at the wrong cause adds ceremony without
   adherence.
2. **Prefer the strongest feasible tier.** Route mechanically-checkable
   rules to Hard (CI/hook); route checkable-by-artifact rules to Forcing
   (a required report field); leave only irreducible judgment as Soft.
3. **Arm the self-check; reserve the backstop for the residual.** A
   judgment rule works as a self-check only when the failure mode it
   targets is named in the agent's context: a *named* trap is a cued
   check, not a blind spot, so a capable agent can recognise "my approach
   trips this" and reconsider on its own. The self-attestation ceiling
   therefore bites only on the *un-cataloged, novel* failure class that no
   checklist yet contains. For that residual, name a backstop — an
   independent review pass, or escalation to the other agent / owner — and
   feed every newly-found failure back into the catalog (a learning loop)
   so the residual shrinks. The backstop is for genuine judgment limits
   and novelty, not a blanket pass substituting for the implementing
   agent's own judgment. Crucially, the arming must reach the agent
   *before* the design decision, not only at completion: a checklist read
   only at task close becomes post-hoc justification of a structure
   already built, so a self-check rule fires at task start (select the
   relevant traps) as well as task close (produce the artifacts).
4. **Respect the always-loaded context budget.** Soft prose competes for
   attention with every other always-loaded rule and can *lower* overall
   adherence as the file grows
   ([Claude Code docs — memory](https://code.claude.com/docs/en/memory)).
   Large soft material belongs on a triggered surface (a skill, or a
   required completion / retrospective step) that loads only when
   relevant, not inline in `AGENTS.md`.

**Rationale:** DD-V-020 made rule *changes* leave a rationale trail; this
makes rule *enforcement* a first-class, recorded decision, so the project
stops producing well-written rules that nothing makes anyone follow. The
discipline is reusable for every future rule, not specific to the
M3-Phase 6 lessons that surfaced it, and it deliberately biases toward
mechanism over prose because M3-Phase 6 showed prose alone failed the
moment the implementing agent changed. The division of labour it encodes
is *armed agent self-judges, then escalates* — not *agent produces,
reviewer catches*: a capable implementing agent given the failure modes
is expected to recognise a poor approach and reconsider, asking another
agent or the owner only on reaching a genuine judgment limit. Review and
the owner are the backstop for that limit and for novel failure classes,
not a substitute for the agent's capability.

**First application — the M3-Phase 6 coding lessons (contingent, not
frozen).** The M3-Phase 6 failures (recorded in this phase's
`implementation/log.md` and retrospectives T1–T5) yield seven recurring
lessons. Discipline rule 1 requires diagnosing the failure before tiering,
so the diagnosis comes first.

*Step-1 diagnosis.* Codex implemented T1–T5; the defects were Codex's, and
most were caught before merge by Claude's independent review (the
Box/ScrollView count-basis under-count in T4; the untested ZStack
diagnostic branches across T1–T3) or by self-review (the T5
layout-invalidation miss). Cause attribution is inferential — the
implementing agent's context is not observable — but the pattern is
legible:

- The core implementation misses (T4 traversal/count, T5 layout-dirty)
  were **not unwritable judgment**. Each is expressible as a concrete
  failure mode to avoid (*audit every call-site when an enum gains a
  variant*; *a structural mutation invalidates layout*). They were missed
  because that failure mode was **not shared with the implementing agent
  in a form that fires at decision time** — a context/triggering gap, not
  a capability gap. A capable agent handed the trap could have recognised
  "this approach trips it" and reconsidered on its own.
- The lighter misses (T1–T3 untested branches; the T4 item-10
  carry-forward narrow reading) were **existing soft rules that
  under-fired** — present but vague and unforced at completion.
- The novel residual is real but small: the *first* occurrence of a
  failure class (T4's traversal-audit lesson did not exist before T4) is
  in no catalog and cannot be self-caught. That is what the backstop and
  the learning loop exist for.

The bet, therefore, is that the dominant cause is **articulable failure
modes that were not in-context as a firing check** — not unwritable
judgment. The remedy is to arm the implementing agent and let it
self-judge and escalate (discipline rule 3). But this is a **one-phase
sample**, and the project's own ≥2-sample discipline forbids ruling a
rule on a single sample — so it would be inconsistent to *also* remove,
on this evidence, the independent review that demonstrably caught these
defects. The armed self-check is therefore shipped as the intended
direction, while review is retained as a transitional safety net, not
declared a novel-only backstop yet.

Concretely, and kept deliberately lightweight (the elaborate
"earn-the-trust" machinery — a waiver process, a counterfactual
catch-rate metric, a staged-removal criterion — is **rejected** as
premature on one sample and prone to gate-hollowing):

- **For now**, high-risk Codex implementation classes — **schema /
  IR migration, runtime structural change, and GUI-render evidence** —
  keep a required *full* independent review before merge. A
  diagnostic-only addition is **not** high-risk and needs no full review,
  but it still requires the narrower **branch/test-focused review** of
  lesson 4 — "no full review" is not "no review".
- **No bespoke waiver mechanism**: the owner already gates every merge,
  so case-by-case discretion exists without inventing one.
- **Revisit through existing machinery**: after ≥2 further phases of
  Codex implementation, a future VDR may narrow or lift the gate using
  the accumulated review record and retrospectives — no special metric is
  built to measure it.

This keeps autonomy as the target without removing a proven safety net
before its replacement is shown to work.

*Tiering, from that diagnosis:*

| # | Lesson | Tier | How it fires |
|---|---|---|---|
| 1 | Semantic-migration traversal audit (classify every call-site) | Forcing artifact | a call-site classification table (citing the `rg` query, files, per-class reason, tests added/not); self-check produces it, auditability makes it binding |
| 2 | Enumerate the side effects of a state/structure change | Forcing artifact | a listed enumeration in the report, armed by the catalog |
| 3 | Keep parallel/derived data synced atomically | Forcing artifact | part of the #2 structural-change artifact: list the derived structures touched and how each was updated; T5's real drift makes Soft too weak |
| 4 | Test every newly-authored reject/diagnostic/size branch | Forcing artifact + review | coverage alone is **insufficient** (false pos/neg on diagnostic / negative / OS-integration branches); a branch/test-focused review stays required until a concrete CI check exists |
| 5 | Carry-forward by impact, not "is it an ADR change" | Forcing artifact (existing) | already the `retrospectives.md` item-10 check; tighten its wording, do not duplicate |
| 6 | Root cause over symptom; never re-roll a deterministic failure to green | Forcing artifact | the report must give the failure's rerun history and disposition, not a bare "green on retry"; directly closes the Obs5 re-roll pattern |
| 7 | GUI positive control | Forcing artifact when GUI is the task evidence | the screenshot + analysis (with a positive control) is the auditable artifact, per the existing `CLAUDE.md`/`AGENTS.md` rule; Soft only when GUI is not the evidence |

The first line of defense for the judgment lessons (1, 2, 3, 6) is the
**armed implementing agent**: a concise, in-context failure-mode catalog
plus a Forcing completion self-check that converts "done" into "have I
cleared these named traps, and if unsure, escalate". For the **high-risk
classes** above, that self-check is *backed by*, not replaced by, the
transitional required review; for everything else, review and the owner
are the backstop for escalations and novel classes. Every new failure
class found — by review or in a retrospective — is added to the catalog,
the learning loop that shrinks the novel residual over time.

**Surface (decided, not deferred).** Discipline rule 4 (context budget)
rules out bolting the catalog + self-check inline into the always-loaded
`AGENTS.md`. The adopted primary surface is a **checked-in Codex
implementation template under `process/procedures/`**, used at **both task
start and task close** — at start the agent reads the failure-mode catalog
and selects which gates apply *before* committing to a structure, so the
design-time traps (T4/T5) are seen before the code is built, not
rationalised after; at close the agent produces the per-lesson auditable
artifacts. (A close-only checklist would invite exactly the post-hoc
justification discipline rule 3 warns against.) This template is newly
authored (distilled from the M3-Phase 6 lessons), not a copy of any
temporary scratch file. *Some* always-loaded pointer to the template is
**required**, not deferred — discoverability is part of enforcement;
without a pointer the template relies entirely on the request prompt
injecting it. What is cosmetic is only the pointer's location and name (a
new `AGENTS.md` section vs an existing one; whether to coin a label). Each
lesson still lands in exactly one enforcement surface (CI, the template,
or the relevant existing SSOT section), with no duplicated prose copy.

**Adoption boundary.** Accepting this VDR adopts DD-V-024 in full and the
DD-V-025 *discipline* (tiers, the four discipline points, the
arming-before-decision rule) in full. Of the First application, the
following are **binding**:

- **(i)** creation of the checked-in start+close template under
  `process/procedures/`, meeting at least these **minimum requirements** —
  start-time selection of the applicable gates, and close-time auditable
  artifacts including a schema/IR-migration call-site table, a structural
  side-effect enumeration, a deterministic-failure rerun/disposition
  entry, and a GUI-evidence field;
- **(ii)** the transitional *full* independent review gate for the
  high-risk classes (schema/IR migration, runtime structural change,
  GUI-render evidence);
- **(iii)** the narrower **branch/test-focused review** for diagnostic /
  reject / size branch additions, required until replaced by a concrete CI
  check;
- **(iv)** the existence of *an* always-loaded pointer to the template.

**Not** frozen by acceptance: the per-lesson tier *wording* in the table
(finalised when the template is authored), the pointer's location and
label, and the Step-1 diagnosis (a one-sample bet, revisited per the
≥2-sample path above).

## Revision history

| Version | Date | Notes |
|---------|------|-------|
| 0.1 | 2026-06-06 | Initial draft, Proposed. |
| 1.0 | 2026-06-06 | Accepted (DD-V-024 + DD-V-025) after Codex review passes. |
