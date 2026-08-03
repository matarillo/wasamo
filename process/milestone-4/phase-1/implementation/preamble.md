---
phase: M4-Phase 1
title: Per-monitor DPI awareness and the coordinate-space boundary
status: active
adr: process/milestone-4/phase-1/decisions/preamble.md
plan: process/milestone-4/plan.md
opened: 2026-07-28
---

# M4-Phase 1 — Per-monitor DPI awareness and the coordinate-space boundary: Implementation

This is the execution framing for **the first M4 phase**. The design
decisions are frozen in the ADR set under
[../decisions/](../decisions/preamble.md) (preamble + DD-M4-P1-001
through DD-M4-P1-004, all Accepted 2026-07-28). This file and its
sibling [plan.md](./plan.md) are mutable during the phase; in-flight
decisions and CI evidence land in [log.md](./log.md); phase residuals
land in [handoff.md](./handoff.md) at phase close. The front-matter
`status` flips `draft` → `active` when the owner approves the T0 review,
and `active` → `closing` at the phase-end batch commit.

## Phase scope

This is **not a feature phase**. It ships no author-facing surface and
no new ABI function. It defines Wasamo's coordinate space and confines
the DIP ↔ physical-pixel conversion to a countable set of sites
([ADR §Context](../decisions/preamble.md#context)).

Four deliverables:

- **Declare** — the runtime declares Per-Monitor-Aware V2 as the first
  act of `runtime::init()`. No host gains a manifest asset or a
  build-system change
  ([DD-M4-P1-001](../decisions/dd-m4-p1-001-dpi-awareness-declaration.md)).
- **Convert** — layout stays in DIP; the Composition visual tree and the
  pointer message stream stay in physical pixels; conversion happens
  only at the seams; crispness is bought at the rasterization surface,
  not inferred from compositor behaviour
  ([DD-M4-P1-002](../decisions/dd-m4-p1-002-coordinate-space-and-conversion-boundary.md)).
- **Follow the change** — scale is held per window and refreshed on
  `WM_DPICHANGED` in a fixed order
  ([DD-M4-P1-003](../decisions/dd-m4-p1-003-dpi-change-propagation.md)).
- **State the unit** — DIP outward-facing, in all three normative specs
  ([DD-M4-P1-004](../decisions/dd-m4-p1-004-unit-contract-and-spec-wording.md)).
  **Landed at Moment 1**, ahead of implementation; re-verified against
  the landed runtime at phase close.

## Acceptance relation

AC7 ([_roadmap.md §M4](../../../_roadmap.md#m4-interaction-stack)) states
three parallel requirements and the DDs map onto them one-to-one:
declare = DD-001, render crisply = DD-002, handle the change = DD-003.
DD-004 is not named by AC7; it is carried by the plan's phase-end
spec-synchronization criterion and by the fact that
[abi_spec.md](../../../../docs/abi_spec.md) freezes at M6.

The M3 residual *"runtime DPI-awareness and DPI-localized layout
evidence"* ([M3 handoff](../../../milestone-3/handoff.md)) is discharged
here.

## The sequencing thesis: build the machinery, then declare

The one sequencing decision that is not inherited from the ADR, and the
reason the task order below looks inverted relative to AC7's wording:

**The awareness declaration lands last (T9), not first.** The property
that permits this is DD-001's, and it is structural rather than
incidental: *the conversion machinery is unconditional*. The runtime
never asks whether its declaration succeeded — it asks the OS for the
window's effective DPI, which is 96 in an unaware process. So every
task from T2 to T8 lands into a process that is still unaware, where
every scale factor is exactly 1 and every conversion is the identity.

Two consequences, both worth the ordering:

- **Every intermediate commit is correct and visually green**, including
  on the 125% development machine. The inverse order — declare first,
  convert later — would leave the window measurably wrong on a scaled
  monitor for the length of the phase, and would make bisecting a
  rendering regression against a moving baseline.
- **T9 becomes a single small flip** with the whole machinery already
  under test behind it.

The cost is that `s ≠ 1` is not exercised by the *real* OS path until
T9. That is why **T8 (the synthesised scale change) is deliberately
placed before T9**: it drives the machinery at 125% / 150% / 200%
without needing an awareness declaration, so the risk the ordering
creates is closed inside the ordering.

**The cost is larger for ordering decisions than for arithmetic ones**
(T4 finding F-31). An identity conversion is still *executed*, so a
missed multiplication is at least present in the code path and becomes
wrong the moment the factor changes. An ordering decision can be worse
than untested: measured at T4, a size-preserving `SetWindowPos`
dispatches **no `WM_SIZE` at all**, so the question "what does the nested
message find" has no answer to get wrong before T9 — the failure mode is
not merely invisible, it is unreachable. Tasks that place work relative
to a message dispatch (T4's correction, T7's step ordering) therefore
cannot lean on "nothing went wrong" in any pre-T9 build, and must argue
structurally instead. This is F-4's lesson one level out: F-4 says a
green suite proves nothing about a conversion; F-31 says a green *run*
proves nothing about an ordering.

**Not all three of those factors are equally load-bearing** (T2 finding
F-13). At a power-of-two factor the scale multiplication is exact, so the
convert-once-on-the-difference rule has no observable effect and a DIP
round trip is exactly the identity — measured, by brute-force search over
`f32`. 200% checks magnitude; 125% and 150% are what check the rules.

T1 confirms this hypothesis against the source before T2 opens. If recon
shows an intermediate state that cannot stay buildable or visually
green, T1 revises the task split rather than proceeding.

## Verification closure (ADR items → tasks)

The ADR's
[§Phase 1 verification closure](../decisions/preamble.md#phase-1-verification-closure-what-counts-as-ac7-evidence)
fixes six evidence lines; this plan adds only the task mapping.

| ADR evidence item | Task(s) |
|---|---|
| (1) `DipScale` pure-logic unit tests — conversion at 125 / 150 / 200%, position-and-extent consistency, round-trip error and rounding direction, the `ceil` allocation rule, convert-once-on-the-difference | T2 |
| (2) Windows integration evidence — declared level is Per-Monitor-Aware V2; cached scale equals `GetDpiForWindow`; after a synthesised scale change the DIP layout results are unchanged while Visual offsets and sizes moved by the ratio. **The unchanged-results half is exact only while the *client* DIP extent is preserved** (T4 finding F-28; the T4 independent review found this correction had not reached here or [plan.md](./plan.md) §T8). T8 synthesises the message and so chooses the rectangle — **necessary and not sufficient, corrected at T8** (mutation M5, T8 independent review): choosing buys equality only if the *physical* client chosen gives an **integer** physical target at every DPI under test (a multiple of 24, since `96 = 2^5 × 3` — which is a claim about rational arithmetic and **not** the same as the DIP extent being recoverable bit-for-bit, which holds at three of T8's four DPIs and not at 100), if the realised value is asserted rather than assumed, and if the quantity asserted is **sensitive to** that extent at the precision the claim needs — T8's per-tile geometry is not below about a DIP, and its root Visual is. The OS's own suggested rectangle preserves the **outer** rectangle instead, and the client one then moves by a DIP or two | T8 (scale cache + invariance control) + T9 (effective-context assertion) |
| (3) Positive control A — crispness before / after, same text, same monitor scale, compared at magnification | T10 |
| (4) Positive control B — logical layout invariance, same `.ui` at the same logical size, 100% vs 125%, wrap positions compared | T10 |
| (5) Positive control C — following a scale change; assistant captures the **path** on the development machine, owner captures the **literal cross-monitor form** | T10 (path) + T11 (literal) |
| (6) Spec-closure gate — three specs at the external-reader bar, `layout-engine.md` §3.1 answered, `verification-environments.md` Observation 4 revised, Moment 1 → Moment 2 markers flipped | **Closed at T12.** Moment 1 landed in `f15eef0`, `7beac4e`, `1769200`, and `80c3fa4`; Moment 2 spec sync landed in `aadca0f` and the Observation 4 correction in `6cb0641` |

**Positive-control discipline.** A single frame on a 100% monitor is not
evidence, because a correct and an incorrect implementation produce the
same picture there. Nor is "the window's physical size scales with the
scale factor" a control — DWM bitmap stretching satisfies it too. The
controls are the *pairs*: before/after crispness at magnification, and
100%-vs-125% layout invariance.

**The invariance in that last pair is not bit-exact** (T4 finding F-28).
The scale factor is realised on the **outer** window rectangle, and the
non-client frame scales by its own DPI-indexed metrics rather than by
`s`, so the same DIP outer size yields a client area of 784 × 561 DIP at
96 DPI and 785.6 × 562.4 DIP at 120 DPI on the development machine.
Layout receives the client extent. A control that demands identical wrap
positions can therefore redden a correct build; [plan.md](./plan.md) §T10
carries the tolerance and the alternative, and §T8 carries the
synthesised-path version of the same qualification.

Windows integration fixtures **fail rather than skip** on a runner
without Compositor capability, following the established `0x80070005`
guard pattern. Any new guard must be shown to fire on an environment
that actually lacks the capability before the test lands
([verification-environments.md](../../../../docs/notes/verification-environments.md)).

## Obligations carried from the ADR / framing

1. **The first implementation task is a spike (T1).** Reads every
   landing file end-to-end — not grep-sample — and compiler-verifies the
   signature changes (throwaway edit → build → enumerate breakage →
   revert; no production code on the T1 commit). Exit criterion: every
   open point is **assigned to a downstream task and its scope is
   seen**, not "no surprises expected"
   ([implementation-gates.md](../../../procedures/implementation-gates.md)).
2. **The Button label Visual write moves in its own commit (T3), ahead
   of the scale work.** DD-002 requires this so a regression in shipped
   rendering code is bisectable independently of the DPI change.
3. **The rasterization surface is specified before the arithmetic is
   trusted.** The ADR is organised around the surface rather than the
   coordinates precisely because an implementation that gets every
   coordinate right and leaves the surface alone produces exactly the
   blur it set out to remove, passes every integration test, and looks
   perfect at 100%. T6 is not an optimisation pass appended to T5.
4. **The audit table is the ADR's, not the implementation's.**
   [DD-002 §The conversion sites](../decisions/dd-m4-p1-002-coordinate-space-and-conversion-boundary.md)
   is written as the audit table so the implementation checks against it
   rather than from memory. T5 audits **all** 13 rows and rows marked
   *unchanged* are assertions to be verified, not omissions — but T5 does
   not *close* all thirteen: **row 13 is closed at T4 and row 7 at T6**,
   and T5's table cites them rather than re-deriving them. Corrected at T5
   (proposition Q7): F-26 fixed this in [plan.md](./plan.md) §T5's end
   gate at T4 and did not reach the obligation that summarises it, which
   is the "correct the carriers, not the summaries" failure a fourth time.
   The obligation that matters is the one T5 discharged: **the audit query
   is assembled from an enumeration of the coordinate-carrying API
   surface, written before the diff exists**, because a query derived from
   what was written cannot falsify what was forgotten (T4 independent
   review finding R-8).
5. **The synthesised-message limit is stated, not elided.** A synthesised
   `WM_DPICHANGED` proves the handling path; it never proves that
   crossing a real monitor boundary delivers the same message with a
   usable suggested rectangle. T8 records the limit with the test; T11
   discharges the other half. Neither is claimed to close AC7's third
   requirement alone.
6. **Host builds are the falsifier for the declarative-host boundary
   claim** (DD-001), so T9 must actually rebuild and run all three hosts
   — not infer the claim from "we did not edit them".
7. **The runnable set is front-loaded to the owner's laptop** so the only
   thing waiting on the owner at phase end is one observation
   (ADR risk R3): host executable + `wasamo.dll` + compiled `.uic`
   delivered at T10, observed at T11.
8. **Final-task / phase-end ownership split.** T12 owns local gates, the
   Moment 2 doc sync, the M4 plan row flip, and its own step retro. The
   **phase-end batch** owns the CI run id, `handoff.md` finalization,
   the phase retrospective, and this file's status flip.

## Implementation gates

Every task runs
[implementation-gates.md](../../../procedures/implementation-gates.md)
at start and close, with the selection — and a one-line reason for each
trap judged non-applicable — recorded in [log.md](./log.md) **before an
approach is chosen**. The phase-wide load, from the
[ADR §Implementation gates armed at drafting time](../decisions/preamble.md#implementation-gates-armed-at-drafting-time):

- **Trap #1 (call-site audit)** — armed, and the single most likely way
  this phase ships broken: a missed conversion path keeps its old unit
  and produces a discrepancy visible **only** at scale ≠ 1, which is
  precisely where CI is not. DD-002's 13-row table is the audit
  artifact; T5 closes it with each row classified and verified, and T6
  closes row 7.
  **The table is the audit's starting point and not its boundary**
  (T5 independent review finding R-1, sharpening T1's F-1). Its row 2
  names `set_root`'s `GetClientRect`, while the runtime has a second
  production `GetClientRect → layout` path in `emit::flush_layout` — so
  the closure was reached against a **14-site** enumeration, and the
  contract's own "no coordinate enters or leaves outside these rows"
  claim is true of the extended list rather than of the thirteen. The
  implementation is complete; the enumeration was not. **Decided
  2026-07-29 (owner): a dated annotation on DD-002**, body unchanged and
  `Status` still `Accepted`, covering row 2 and also row 12, which names
  `Box` — a widget that installs no clip — while omitting `ZStack`, which
  does. Not a successor: no option is re-chosen. The same omission in
  [architecture.md §12.3](../../../../docs/architecture.md#coordinate-spaces)
  is a Moment 2 item in [plan.md](./plan.md) §T12.
- **Trap #2 (structural side effects)** — armed. DD-003's 13-row
  enumeration of what a scale change drags along is T7's close artifact.
  Rows 9–13 (`SetRelativeSizeAdjustment`, clip insets, signal registry /
  effect graph / binding state, drain accounting, hover and press state)
  are *unchanged* assertions and must be verified as such.
- **Trap #5 (carry-forward)** — armed. Known carriers at planning time:
  layout-derived hit rectangles → M4-Phase 2; the host-visible scale or
  work-area query trigger → M4-Phase 7 / 8; the non-zero clip inset
  re-check; the custom-title-bar re-examination of V2's automatic
  non-client scaling → M5; and the fact that a scale-dependent `measure`
  would turn T7's step ordering from a free choice into a correctness
  constraint.
- **Trap #6 (deterministic-failure root cause)** — armed with low
  expectation. Nothing in the slate is expected to be flaky; the gate is
  carried because Composition surface recreation is WinRT-fallible.
- **Trap #7 (GUI positive control)** — armed. T10, with the pair
  discipline above.
- **Trap #3 (parallel data)** — the ADR judged this non-applicable for
  the phase, "no parallel vectors or derived indices are added; the scale
  is a single scalar per window". **This plan narrows that judgment
  rather than inheriting it** (T3 finding F-22). The judgment is right
  *about the scale* — one scalar per window, one authoritative owner —
  but T3 landed `ButtonData.label_size`, a cached derivative of the
  node's label text and style that sits beside two existing derivatives
  of the same measurement (`self.width` / `self.height` as
  `SizeConstraint::Fixed`). Its close artifact is the single-writer
  discipline: the field is written inside the same statement group as
  `label_text` in **both** primitives that produce a measurement, so no
  primitive mutates the source without updating the cache, and the
  re-trigger criterion — a third writer of a Button-family label — is
  recorded in [handoff.md](./handoff.md). This is the third phase-wide
  non-applicability narrowed by what actually landed, after trap #4 at T2
  (F-12) and the review lane at T3 (F-17).
  **T5 is the second site, and it was armed as non-applicable one level
  further down** (T5 finding F-32). T1's §T5 gate selection marked trap #3
  "no" with the reason "No parallel vector, index, or **cache** is added"
  — in the same sentence that names the node-side scale cache T5 adds. The
  cache is a derived copy of `WindowState::scale`, whose source the
  runtime does mutate (T7's handler), so the trap applies and its artifact
  is an enumeration of that source's mutators and of every path that
  attaches a node. Running it surfaced two shipped path classes the plan's
  walk bullet did not name — incremental tree mutation and the IR loader's
  conditional / `for` sites — which trap #5's re-trigger sentence would
  not have produced. Close artifact in [log.md](./log.md) §T5.
- **Trap #4 (untested authored branch)** — the ADR judged this
  non-applicable on the grounds that the phase adds no author-facing
  surface and no new validation branch. **This plan narrows that
  judgment rather than inheriting it**, and the narrowing has **two**
  sites, not one (T2 finding F-12 — this paragraph originally named only
  T9, while [plan.md](./plan.md) §T2 already named T2, so the two
  documents disagreed from the moment they were written):
  - **T2** ships two authored arithmetic branches — the zero-DPI
    fallback to the identity, and the one-pixel surface floor. Both
    landed with a test that fires them, and each test was in turn shown
    to fail against a deliberately wrong implementation; the mutation
    table in [log.md](./log.md) is the artifact.
  - **T9** adds a diagnostic branch (the tolerated-declaration-failure
    path) and must re-run the trap-#4 decision explicitly. If that branch
    cannot be fired by a test because process DPI awareness is a one-shot
    per process, that is recorded as a **stated limit with its reason**
    in [log.md](./log.md) — not silently skipped, and not left as an
    inherited "non-applicable".
  - **T4 is a third site where the judgment was live, and it survives
    only because the approach was chosen against the ADR's own
    phrasing.** [DD-003 §Initial scale acquisition](../decisions/dd-m4-p1-003-dpi-change-propagation.md)
    words option I1 as "**if the scale is not 1** apply `size × s`". A
    literal implementation would have shipped a branch reachable only
    after T9, on the path every host takes, and directly against
    [DD-001 §Failure handling](../decisions/dd-m4-p1-001-dpi-awareness-declaration.md)'s
    structural argument that tolerating a failed declaration is safe
    *because the conversion machinery has no second code path*. T4's
    correction is unconditional and the absence of the branch is its
    artifact. Recorded because "trap #4 did not apply" and "trap #4 was
    avoided by a design choice" are different facts, and only the second
    one is true here.

**Review lanes** ([gates §4](../../../procedures/implementation-gates.md)):

| Task | Lane | Why |
|---|---|---|
| T5 | Full independent review | Runtime structural change across every coordinate-carrying path |
| T6 | Full independent review | Rendering path; the phase's hard part and its highest-consequence silent failure |
| T7 | Full independent review | Runtime structural change with re-entrancy through the message loop |
| T9 | Full independent review | Process-wide platform posture + the diagnostic branch (trap #4 folded in) |
| T10 | Full independent review | GUI-render evidence |
| T2 | Branch/test-focused review | **Corrected at T2 (finding F-12); this row read "Normal review / pure logic".** Pure logic is why T2 is not in a full-review class, but it adds two authored branches, and [gates §4](../../../procedures/implementation-gates.md) assigns exactly that case the branch/test-focused lane. "No full review" is not "no review" |
| T3 | Full independent review | **Corrected at T3 (finding F-17); this row read "Normal review", grouped with T4 and T8, on the ground that a behaviour-identical refactor "carries an explicit regression check against shipped rendering".** That ground was the existing fixtures, and T1's finding F-4 removed it: the fixtures do not react to a geometry-write relocation, so [plan.md](./plan.md) §T3 now makes the **rendered frame** the gate. T3's evidence class is therefore GUI-render evidence, and it relocates a write between passes in shipped rendering code — two of the three high-risk classes in [gates §4](../../../procedures/implementation-gates.md) |
| T4 | Full independent review | **Corrected at T4 (finding F-25); this row read "Normal review", grouped with T8, on the ground "additive per-window state".** The `DipScale` field is additive, but the task also inserts a `SetWindowPos` that **re-enters `wnd_proc` synchronously in the middle of `window::create`** — on the single path both public window-create entries and all three example hosts take, at a point where the object being re-entered is half-constructed. [gates §4](../../../procedures/implementation-gates.md) names *runtime structural change* as a high-risk class, and this preamble justifies T7's full lane as "runtime structural change with **re-entrancy through the message loop**" — the same property, four tasks earlier |
| T8 | Normal review | Test-only |
| T12 | Normal review | Assigned at T12 start because the row was absent. T12 changes normative and procedural documentation and runs existing gates, but adds no schema / IR migration, runtime structural change, GUI-render evidence, diagnostic branch, reject branch, or size branch; none of the full or branch/test-focused triggers in [gates §4](../../../procedures/implementation-gates.md) applies |

## Technical risks (planning-time recon; T1 sharpens)

| ID | Risk | Mitigation |
|---|---|---|
| R-1 | **Coordinates right, crispness not bought** (ADR risk R2). The phase's defining failure: every integration test passes, 100% looks perfect, and the blur the phase existed to remove is still there. Integration tests cannot discharge it — a stretched bitmap reports the same numbers. | T6 is specified and reviewed as the phase's hard part, not appended to T5. Positive control A (T10) is the only evidence that closes it. **T1 de-risked the approach**: a throwaway `ceil(dip × s)` surface + `SetDpi(96 × s)` + origin ÷ s produced visibly crisper glyphs than the DWM-stretched baseline in a magnified before/after pair on the 125% machine. That is spike evidence on one machine, not T10's artifact — but the approach is no longer unproven going in. |
| R-2 | **A missed conversion site** is wrong only at scale ≠ 1. **Sharpened at T1 (finding F-4): the 125% development machine does not catch it either.** Every layout integration test drives `WidgetNode`s directly and never through a window, so no existing test routes a coordinate through a window's scale. Measured: with the full conversion machinery *and* the V2 declaration in place at 125%, all 32 test binaries passed — identical to baseline. | DD-002's audit table closed at T5/T6 with each row verified — now the **primary** defence, not one of three. T8's synthesised change is the **only** automated defence and its weight rises accordingly. Positive control B (T10) fails visibly if a size path is missed and a wrap position moves; T1 saw that signal fire on a deliberately incomplete build. |
| R-1b | **The build command does less than it looks like it does**, in two measured ways, both pre-existing and unrelated to DPI. (i) **A cold-directory workspace build does not link** (T1 finding F-5): `wasamo-dll/build.rs` whole-archives the *uplifted* `<profile>/libwasamo_runtime.rlib`, which cargo only produces once `wasamo-runtime` is built as a primary package. A cold `cargo test --workspace` fails `LNK1356`; a stale uplifted rlib fails as `LNK2019` on `core` / `std` symbols. `cargo check` never links, so it stays green through both and gives false comfort. (ii) **A host-package build relinks the DLL around stale object code** (T3 finding F-21, mechanism corrected at the independent review): the same whole-archive path takes the **uplifted** rlib, which cargo refreshes only on a primary-package build — so `cargo build -p gallery-rust` *does* recompile `wasamo-runtime` and *does* relink `wasamo.dll`, and the DLL still carries the previous runtime. Unlike (i) this fails **silently and green**, with a fresh DLL timestamp, which makes it a false-negative generator for every GUI evidence gate and defeats any freshness check. | (i) Build `-p wasamo-runtime` first — verified green. (ii) Precede every release capture with `cargo build --release -p wasamo-runtime` and then `cargo build --release --workspace` — measured at T3, where a mutation built the other way produced a frame identical to the unmutated build. Both are **one root cause with two symptoms**, recorded in [handoff.md](./handoff.md) with re-trigger criteria and folded into T6 / T9 / T10 and T12's clean-rebuild and current-instruction correction. Neither is fixed by this phase. |
| R-3 | **The atlas-offset trap.** `BeginDraw` returns the offset in pixels; once the context DPI is `96 × s` it must be divided by `s`. The offset was described as frequently `(0, 0)`, so omitting the conversion would work most of the time and displace text within its own surface intermittently. **Measured at T5 (finding F-33) and the qualification is generous**: instrumenting `draw_text` over the gallery gives offsets `(1,2)`, `(19,2)`, `(68,2)`, `(125,2)`, `(199,2)`, `(255,2)`, `(345,2)`, `(348,2)` … — they march across the atlas and essentially **none** is `(0, 0)`. On any UI with more than a couple of text nodes the omission is wrong nearly everywhere rather than intermittently. | Named in DD-002 so T6 wrote it deliberately rather than discovering it; T6's full review verified the landed conversion. **Spec divergence closed at T12:** [architecture.md §12.4](../../../../docs/architecture.md#coordinate-spaces) now keeps the general pixel-origin rule without the falsified frequency claim. The T5 run also recorded that atlas packing is deterministic across launches, disqualifying it as the cause of F-33's frame drift. |
| R-4 | **`s ≠ 1` was unexercised by the real OS path until T9** — the cost of the sequencing thesis. | **Closed at T9.** T8 first pinned the synthesised propagation path; T9 then exercised the landed declaration and effective per-window DPI through the real OS path on all three hosts |
| R-5 | **Closed at T1.** The planning-time estimate ("two production sites and at least four integration tests") undercounted: `run_layout_as_window_root` has 2 production + **13** test call sites in 6 files, and the plain `run_layout` — omitted from the estimate — has 1 production (`emit::flush_layout`) + **8** test sites in 3 files. Naive parameter threading was compiler-measured at **28 broken test call sites across 12 files**. | Resolved by caching the scale on the node instead of threading it: **7** broken sites in 4 files, all of them the `hit_test_click` literals that the `i32`→`f32` pointer-unit change costs under any carrier. The layout entry points keep their signatures and no test learns about scale. Decided and compiler-verified at T1; details in [log.md](./log.md) §T1. |
| R-6 | **Owner-visible verification needed a second machine** (ADR risk R3). | **Closed at T11.** T10 delivered the runnable set; the owner then executed the literal cross-monitor path on a laptop plus external display, including the aware/unaware positive-control pair |
| R-7 | **Moment 2's `verification-environments.md` revision depended on running evidence.** Observation 4's old premise (the host is DPI-unaware, so DWM stretches logical 800×600 to physical 1000×750) was falsified by this phase's implementation, and the corrected capture coordinates could not be derived at ADR time. | **Closed at T10/T12.** T10 produced the posture-read-back client/outer coordinate artifact; T12 revised Observation 4 with those exact condition-scoped values and the PMv2 readback/client-capture procedure |
| R-8 | **The declarative-host boundary claim is assumed rather than tested.** Choosing runtime-side declaration over per-host manifests is falsifiable exactly at "all three hosts still build and run unchanged". | T9 rebuilds and runs C, Rust, and Zig hosts with no manifest asset and no build-system edit, as a recorded artifact (obligation 6). **Closed at T9, and the artifact is stronger than the risk asked for.** All three were rebuilt from a cleared build directory and run, and each was asked what awareness level is in force in its own process: **Per-Monitor-Aware V2** in all three, with `GetDpiForWindow` reporting 120 and the window realised at 1000 × 750 physical. "They still build and run" would have been satisfied by three processes that declared nothing, which is F-9's trap one level out; the level readback is what makes the run a falsifier rather than a smoke test. |
| R-9 | **`SetWindowPos` correction is inert until T9**, so T4's create-then-correct path ships untested at `s ≠ 1`. **Sharpened at T4**: "inert" understates it for the *placement* half — at `s = 1` the correction dispatches no `WM_SIZE` at all, so the ordering question the placement answers cannot even be posed before T9 (finding F-31). | The arithmetic is covered by `window_size_to_physical`'s tests, each shown to fail against a wrong implementation. **The real-window form is no longer deferred to T10**: T4 measured it directly with a throwaway declaration, recording three window states — unaware (1000 × 750, 7 tiles), aware without the correction (800 × 600, 7 tiles), aware with it (1000 × 750, 9 tiles, the 9 being the signature of T5's still-absent inbound seam). The residual risk is that the measurement was taken against a throwaway declaration rather than the landed one, which T9 closes. **Closed at T9**: the three hosts, run against the landed declaration, realise 1000 × 750 physical from an 800 × 600 DIP request with a client of 982 × 703 — the same numbers T4's throwaway probe recorded, so the two measurements agree and the residual is discharged rather than merely re-asserted. |

## Lifecycle transition

Implementation opened once T0 closed: the ADR set is Accepted (done,
`09ff0d4`), the Moment 1 spec commit set is complete (done — see the
verification-closure table above), and this `preamble.md` + `plan.md` +
skeleton `log.md` / `handoff.md` were owner-reviewed and landed. T12 completed
the Moment 2 sync; the phase-end batch follows per obligation 8.

## Cross-references

- ADR set: [../decisions/preamble.md](../decisions/preamble.md) +
  [DD-M4-P1-001](../decisions/dd-m4-p1-001-dpi-awareness-declaration.md) +
  [DD-M4-P1-002](../decisions/dd-m4-p1-002-coordinate-space-and-conversion-boundary.md) +
  [DD-M4-P1-003](../decisions/dd-m4-p1-003-dpi-change-propagation.md) +
  [DD-M4-P1-004](../decisions/dd-m4-p1-004-unit-contract-and-spec-wording.md).
- Phase requirements:
  [../requirements/framing.md](../requirements/framing.md) +
  [../requirements/constraints.md](../requirements/constraints.md).
- Milestone plan: [../../plan.md](../../plan.md) §Phase breakdown,
  §Progress.
- Normative specs synced at Moment 1 and re-synced at T12 Moment 2:
  [architecture.md §12](../../../../docs/architecture.md#coordinate-spaces),
  [dsl_spec.md §1 units](../../../../docs/dsl_spec.md),
  [abi_spec.md §4.1 / §4.2](../../../../docs/abi_spec.md).
- Notes:
  [layout-engine.md §3.1](../../../../docs/notes/layout-engine.md) (answered),
  [verification-environments.md](../../../../docs/notes/verification-environments.md)
  (Observation 4 — corrected at T12 Moment 2).
- Procedures:
  [implementation-gates.md](../../../procedures/implementation-gates.md),
  [retrospectives.md](../../../procedures/retrospectives.md),
  [workflow.md](../../../procedures/workflow.md).
