# M4-Phase 1 — Per-monitor DPI awareness and the coordinate-space boundary: Architecture Decisions

**Phase:** M4-Phase 1 (per-monitor DPI awareness and the coordinate-space
boundary — **the first M4 phase**)
**Date:** 2026-07-28
**Status:** Accepted (owner approval 2026-07-28; all four DDs Accepted)

## Context

M4 acceptance criterion **AC7** (see
[../../../_roadmap.md M4](../../../_roadmap.md#m4-interaction-stack),
[../../plan.md §Acceptance criteria](../../plan.md)):

> **AC7** — Per-monitor DPI awareness: declare process / window DPI
> awareness, render crisply on high-DPI displays without DWM bitmap
> scaling, and handle DPI changes across monitors.

AC7 states three parallel requirements, and the DD slate maps onto them
one-to-one: **declare** = DD-001, **render crisply** = DD-002, **handle
the change** = DD-003. DD-004 is not named by AC7's wording; it is the
obligation that becomes answerable only once the scale factor stops
being 1 — what `width: 800` *means* — and phase-end criterion 4 (spec
synchronization) requires it before
[docs/abi_spec.md](../../../../docs/abi_spec.md) freezes at M6.

The phase's thesis, fixed by the owner-aligned framing
([../requirements/framing.md](../requirements/framing.md), accepted
2026-07-28), is that this is **not a feature phase**. It is the phase
that **defines Wasamo's coordinate space**. From M1 until now there has
been exactly one space: `GetClientRect`'s numbers feed layout, layout's
numbers feed Visual offsets, and DirectWrite's measurements mix in as
lengths. That held together only because the scale factor was always 1 —
or, more precisely, because on a scaled monitor DWM stretched the whole
window as a bitmap and Wasamo never saw anything but 1.

This phase splits that one space into a **DIP space** and a **physical-
pixel space** and confines the conversion to a countable set of sites.

### The starting state (verified against the workspace at drafting time)

- **No awareness is declared.**
  [`wasamo-runtime/src/window.rs`](../../../../wasamo-runtime/src/window.rs)
  `create_hwnd` calls neither `SetProcessDpiAwarenessContext` nor ships
  an application manifest. `wasamo-runtime/Cargo.toml` does not enable
  the `Win32_UI_HiDpi` feature of the `windows` crate, so none of the
  per-monitor API surface is currently in scope of the build.
- **Layout consumes client pixels one-to-one.** `set_root`'s
  `GetClientRect` and `wnd_proc`'s `WM_SIZE` both hand physical client
  extents straight to `run_layout_as_window_root`. No scale is ever
  applied.
- **`WM_DPICHANGED` is not handled.**
- **Text is rasterized at 96 DPI onto a logically-sized surface.**
  [`text.rs`](../../../../wasamo-runtime/src/text.rs) `draw_text` calls
  `CreateDrawingSurface(Size { width, height })` — the `windows` crate
  names that parameter `sizepixels` — and draws through a D2D device
  context left at its default 96 DPI. The surface is then applied as a
  `CompositionSurfaceBrush`. **This, not the coordinate arithmetic, is
  where crispness is won or lost.**
- **Hit-testing reads geometry back off the live Visual.**
  `hit_test_click_inner` / `update_hover_inner` in
  [`widget.rs`](../../../../wasamo-runtime/src/widget.rs) call
  `visual_rect`, which reads `Visual.Offset` / `Visual.Size`, and
  compare those against the raw `lparam` pointer coordinates. Both sides
  are physical today, which is why the existing path is arithmetically
  correct and will stay correct through any consistent choice below —
  a property DD-002 examines rather than assumes.
- **One ABI function carries coordinates**: `wasamo_window_create`'s
  `width` / `height`. There is no ABI entry point that sets or reads
  pointer or resize geometry.
- **The stated OS floor is Windows 10 1809+**
  ([docs/architecture.md](../../../../docs/architecture.md) §deployment
  diagram). `SetProcessDpiAwarenessContext` (1703+) and
  `GetDpiForWindow` (1607+) are both below that floor, which removes a
  question DD-001 would otherwise have to answer.

### The owner prior governing comparisons

M4-Phase 1 DD comparisons take **product merit as the primary axis**;
implementation and revision cost is a tie-breaker, never on its own a
ground for rejecting an option. This is not maximalism — over-design
justified by future extensibility remains a named failure mode
(framing §再検討しない前提). Each DD below rejects its non-chosen
options on merit and says so.

### Framing agreements this ADR consumes

Owner-aligned 2026-07-28, recorded in
[../requirements/framing.md](../requirements/framing.md) §オーナー合意の記録:

- **① Subject boundary.** The phase is confined to **conversion at the
  boundary**. The layout calculation method (two-pass, `f32`, no integer
  snapping) does not change. If a decision would cross that line, a
  stage-2 plan revision is proposed *before* proceeding, per
  [DD-V-026](../../../cross-milestone/decisions/plan-revision-discipline.md).
- **② Who declares awareness.** The **runtime (DLL) side** is the
  working direction, because per-host application manifests would push
  three host executables past the declarative-host boundary
  ([../requirements/constraints.md](../requirements/constraints.md) §6).
- **③ How far the ABI moves.** The phase **states the unit semantics of
  existing arguments** and **adds no new ABI function**. A new function
  would collide with the plan's "M4-Phase 7 is the milestone's only
  ABI-bearing phase" and would require a stage-2 plan revision first.
- **④ DD slate.** Four decisions.
- **⑤ Verification split.** Positive controls A / B / C are captured by
  the assistant on the development machine (single monitor, measured
  2452×1291 at DPI 120 = 125%, Console session); the literal
  cross-monitor case is the owner's human-visible smoke on a laptop plus
  external display.

**② and ③ are directions for framing the decisions, not pre-empted
conclusions.** Both were tested inside the DDs below:

- **② holds.** DD-001 adopts runtime-side declaration on merit (the
  comparison is in DD-001 §Comparison), not by inheritance.
- **③ holds, and no plan revision is required.** DD-004 examined
  whether a host-facing "query the scale factor" function is needed in
  M4 and concluded it is not, with a recorded trigger that lands in
  M4-Phase 7 / 8 if it fires.
- **① holds, and no plan revision is required.** DD-002's recommended
  option leaves the layout engine's calculation untouched: layout stays
  in DIP, `f32`, unsnapped, and the pure-logic layout tests are
  unaffected. What DD-002 *does* move is where Composition writes
  happen (two construction-time Visual writes move into the scale-aware
  sync pass) — a Visual-write-site change, not a layout-calculation
  change.

## Decisions

| DD | Title | Decision summary (Accepted) |
|---|---|---|
| [DD-M4-P1-001](./dd-m4-p1-001-dpi-awareness-declaration.md) | DPI-awareness declaration: level, site, actor, failure handling | **Per-Monitor-Aware V2**, declared by the **runtime DLL** as the first act of `runtime::init()` (i.e. inside `wasamo_init`, before the DispatcherQueue and Compositor are created), via `SetProcessDpiAwarenessContext`. Hosts are unchanged — no manifest asset, no build-system change in any of the three hosts. V2's automatic non-client-area scaling is relied on in full (Wasamo paints no non-client area). A failed declaration is **tolerated, not fatal**: the common cause is a legitimate host that already declared awareness itself, and the conversion machinery of DD-002 is unconditional — it reads the *effective* per-window DPI, so an unaware process simply runs at scale 1 through the same code path. The outcome is recorded as a diagnostic and asserted in an integration test through `GetWindowDpiAwarenessContext` + `AreDpiAwarenessContextsEqual`. No legacy-OS fallback is written: both APIs predate the stated Windows 10 1809 floor. |
| [DD-M4-P1-002](./dd-m4-p1-002-coordinate-space-and-conversion-boundary.md) | Coordinate-space definition and the conversion boundary | **Layout coordinates are DIP; the Composition visual tree and the pointer message stream are physical pixels; conversion happens only at the seams.** The root-Visual scale-transform alternative is rejected on merit: it does not avoid any of the text-rasterization work (the hard part is identical in both), it makes crispness depend on unspecified compositor resampling, and it puts the visual tree's coordinate space out of step with the physical space that hit-testing and the Phase 5 / 6 / 9 screen-rectangle consumers live in. **Crispness is bought explicitly**: the drawing surface is allocated at `ceil(dip × scale)` pixels and the D2D context's DPI is set to `96 × scale`, so DirectWrite lays out in DIP and rasterizes at device resolution. Surfaces are built at scale 1 at construction time and brought to the window's scale by a **re-rasterization walk** shared with DD-003 — which keeps the widget tree window-independent at construction and so does not block M4-Phase 8. A `DipScale` conversion type carries the arithmetic and the rounding contract as pure, unit-testable logic. Two construction-time Visual writes (the Button label's offset and size) **move into the sync pass**. The pointer is converted to DIP at the window entry; the `visual_rect` readback is converted alongside it. |
| [DD-M4-P1-003](./dd-m4-p1-003-dpi-change-propagation.md) | Initial scale acquisition and `WM_DPICHANGED` propagation | Scale is held **per window**, as a field on `WindowState`, seeded from `GetDpiForWindow` immediately after `CreateWindowExW` returns and before any layout runs. This is the shape M4-Phase 8 needs, so per-window scale arrives without a structural rebuild. `wasamo_window_create`'s DIP size is realised by creating the HWND and then applying `size × scale` through `SetWindowPos` before the window is shown — flash-free precisely because creation and `wasamo_window_show` are separate ABI calls. On `WM_DPICHANGED` the order is **fixed and load-bearing**: update the cached scale **first**, then apply the OS-suggested rectangle via `SetWindowPos`, whose synchronous nested `WM_SIZE` performs the re-layout; then re-rasterize text surfaces through DD-002's shared walk. Because layout results in DIP are **invariant under a scale change**, the change invalidates only the physical projection and the rasterization — which is what makes that order safe and what the integration test asserts. Whole-window invalidation is confirmed, not reworked. Failures are logged and survived, matching the existing resilient posture. |
| [DD-M4-P1-004](./dd-m4-p1-004-unit-contract-and-spec-wording.md) | The outward unit contract and its wording in the three specs | **DIP is the unit of every outward-facing length**: `wasamo_window_create`'s `width` / `height` (of the **outer window rectangle**, which is what they have always meant), every DSL dimension literal, and the typography ramp (12 / 14 / 20 / 28). At 100% every existing host is bit-identical, so stating the semantics breaks no compatibility. **No new ABI function**: the question "must a host be able to query the scale factor" was examined and answered *not in M4*, with a recorded trigger landing in M4-Phase 7 / 8 — so agreement ③ holds and no plan revision is proposed. `docs/architecture.md` gains a normative coordinate-space section and its open-questions DPI row is resolved; `docs/dsl_spec.md`'s "pixel extents in the layout coordinate system" is replaced with a DIP definition; `docs/abi_spec.md` §4.2 states the argument unit. [layout-engine.md](../../../../docs/notes/layout-engine.md) §3.1 is answered while the note stays `live` for §3.2, and [verification-environments.md](../../../../docs/notes/verification-environments.md) Observation 4 is revised, because this phase **falsifies its stated premise** that the host is DPI-unaware. |

## Cross-DD decision dependencies

Three couplings span the slate. The primary DD owns the choice; the
dependents carry the consequence (index only — the arguments live in the
owning DDs).

| Coupling | Primary DD | Dependent DDs | Consequence |
|---|---|---|---|
| **Where the scale is applied** | DD-002 (conversion at the seams, layout stays DIP) | DD-003 (what a scale change invalidates), DD-004 (what the outward unit can be) | Layout results are scale-invariant ⇒ a DPI change re-projects and re-rasterizes but does not re-decide layout ⇒ the outward unit can be DIP without the engine ever seeing a scale factor. |
| **Effective awareness vs. declared awareness** | DD-001 (declare, tolerate failure) | DD-002 (conversion is unconditional), DD-003 (`GetDpiForWindow` is the single source) | The runtime never branches on "did the declaration succeed" — it reads the effective per-window DPI, which is 96 when the process is unaware. The scaled path is therefore the only path, exercised even at 100%. |
| **The re-rasterization walk** | DD-002 (surfaces built at scale 1, brought to window scale by a walk) | DD-003 (the same walk is the `WM_DPICHANGED` step) | One mechanism with two callers — attach and scale change — so the change path is exercised without a DPI change and the widget tree stays window-independent at construction (M4-Phase 8 unblocked). |

## Phase 1 verification closure (what counts as AC7 evidence)

Per the framing verification strategy and the positive-control
constraint — **a single frame on a 100% monitor is not evidence,
because a correct and an incorrect implementation produce the same
picture there** — the phase closes only when all six are observed.

1. **Pure-logic unit tests (DD-002).** The `DipScale` conversion type:
   DIP ↔ physical at 125% / 150% / 200%; rectangle conversion
   (position and extent converted separately stay consistent);
   round-trip error and rounding direction; the `ceil` surface-
   allocation contract; the convert-the-difference-once rule.
2. **Windows integration evidence (mock-free, CI-gated,
   fail-not-skip).** The declared awareness level is Per-Monitor-Aware
   V2 (`GetWindowDpiAwarenessContext` + `AreDpiAwarenessContextsEqual`);
   a created window's cached scale matches `GetDpiForWindow`; and — the
   integration-side positive control — **after a synthesised scale
   change the layout's DIP results are unchanged** while the Visual
   offsets and sizes have moved by the scale ratio. The **stated limit**
   is recorded with the test: a synthesised `WM_DPICHANGED` proves the
   handling path, never that a real monitor crossing delivers the same
   message. That half is item 5's.
3. **Positive control A — crispness, before and after.** The same text
   at the same monitor scale, captured before and after the change, and
   compared at magnification. **The pair is the control; the "after"
   frame alone proves nothing.** If a pre-change frame is reused rather
   than re-captured, the commit it was captured at is checked against
   the current surface first
   ([constraints §9](../requirements/constraints.md)).
4. **Positive control B — logical layout invariance.** The same `.ui` at
   the same logical window size, captured at 100% and at 125%, with
   wrap positions and element order compared. **Invariance is the
   evidence.** Note that "the window's physical size scales with the
   scale factor" is *not* a control — DWM bitmap stretching satisfies it
   too.
5. **Positive control C — following a scale change.** Two frames across
   a change. The assistant captures the **path** on the development
   machine by changing the display scale while the window is up; the
   **literal cross-monitor form** is the owner's human-visible smoke on
   a laptop plus external display. Neither alone discharges AC7's third
   requirement, and the ADR does not claim otherwise.
6. **Spec-closure gate (non-test).** The three normative specs state the
   unit at the external-reader bar, `layout-engine.md` §3.1 is answered,
   `verification-environments.md` Observation 4 is revised, and the
   Moment 1 → Moment 2 markers are flipped.

## Implementation gates armed at drafting time

Selected under
[implementation-gates.md](../../../procedures/implementation-gates.md)
before an approach was chosen, so the selection is itself auditable.

- **Call-site audit (armed).** The seven coordinate-carrying paths in
  [constraints §4](../requirements/constraints.md) are the audit table.
  A path missed keeps its old unit and produces a discrepancy visible
  **only** at scale ≠ 1 — the single most likely way this phase ships
  broken. DD-002 §The conversion sites is written as the audit table so
  the implementation checks against it rather than from memory.
- **Structural side-effect enumeration (armed).** What a scale change
  drags along. DD-003 §Structural side-effect enumeration is the close
  artifact.
- **Visible positive controls (armed).** Items 3–5 above.
- **Deterministic-failure rerun (armed, low expectation).** Nothing in
  the slate is expected to be flaky; the gate is carried because
  Composition surface recreation is WinRT-fallible.
- **Judged non-applicable, with reasons.** *New reject/diagnostic
  branches* — this phase adds no author-facing surface and no new
  validation branch, so there is nothing to pin with a failure-path
  test. *Shared-lexer / IR schema migration* — neither `wasamoc` nor
  `wasamo-ir` is touched: the unit contract is stated in prose, not
  encoded in the IR. *Parallel-data drift* — no parallel vectors are
  added; the scale is a single scalar per window.

## Out of scope

The deferred-items authority (with activation triggers and where the
responsibility lands) is the framing scope table
([../requirements/framing.md](../requirements/framing.md) §含まないもの);
this ADR does not duplicate it. Out of AC7 scope this phase, by
decision: new hit-testing surface (generic click, per-item handlers,
ZStack sibling occlusion — M4-Phase 2); per-window differing scale
factors (M4-Phase 8, whose shape DD-003 is responsible for not
blocking); caret and composition-string screen rectangles (M4-Phase 5 /
6); top-layer placement coordinates (M4-Phase 9); resolution-dependent
image asset selection (M4-Phase 4); integer pixel snapping (deferred,
trigger recorded); text rendering-quality tuning — rendering mode,
gamma, explicit hinting (deferred to the M5 theming wave); cache
invalidation granularity (deferred); touch coordinates (M4-Phase 2).

## Upstream document revisions (Moment 1 / Moment 2)

The per-review-concern commit rule applies
([AGENTS.md §Commit rules](../../../../AGENTS.md)). Touch / no-touch
judgments are explicit.

**Moment 1 — ADR Accepted commit set (design sync):**

- This directory — `Status: Proposed` → `Accepted` flip.
- [`docs/architecture.md`](../../../../docs/architecture.md) —
  **touch.** A normative coordinate-space section (DIP layout space,
  physical visual/pointer space, the conversion seams, the text-surface
  resolution contract, the scale-change propagation order); §12
  open-questions DPI row moves from `Open` to resolved with a link to
  this ADR. Provenance is the hyperlink only — no DD labels or
  decision-summary vocabulary in spec prose.
- [`docs/dsl_spec.md`](../../../../docs/dsl_spec.md) — **touch.** The
  "pixel extents in the layout coordinate system" wording is replaced
  by a DIP definition, stated once normatively and referenced from the
  dimension-bearing sections rather than repeated. The typography ramp
  is stated in DIP.
- [`docs/abi_spec.md`](../../../../docs/abi_spec.md) — **touch.** §4.2
  states that `wasamo_window_create`'s `width` / `height` are DIP of the
  outer window rectangle, and that the runtime declares process DPI
  awareness during `wasamo_init` (which hosts must know, because it
  interacts with a host that declares awareness itself).
- [`docs/notes/layout-engine.md`](../../../../docs/notes/layout-engine.md)
  — **touch, revise in place.** §3.1 becomes answered with a pointer
  here. The note stays `live`: §3.2 (AccessKit sync) belongs to
  M4-Phase 11.
- [`docs/notes/verification-environments.md`](../../../../docs/notes/verification-environments.md)
  — **touch at Moment 2, not Moment 1.** Observation 4's premise ("the
  host is DPI-unaware, so DWM stretches logical 800×600 to physical
  1000×750") stops being true when the *implementation* lands, not when
  the ADR is accepted, and the corrected capture coordinates can only
  be derived against the running surface. Revising it at Moment 1 would
  put an untested claim into the document that later phases rely on for
  their capture procedure.
- [`../../plan.md`](../../plan.md) — Phase 1 row populated.
- All three specs' revision-history tables are **appended to**, never
  edited in place ([constraints §8](../requirements/constraints.md)).

**Moment 2 — phase-close commit set (implementation sync):** spec
markers flip to implementation-synced with divergence corrections;
`verification-environments.md` Observation 4 is revised and the capture
coordinates for later phases are re-derived against the new coordinate
space; the plan row flips complete; the phase-end retrospective and the
CI run id land per the ownership split (local rebuild belongs to the
step that changed code, the CI run id to phase end).

## Inputs absorbed

| Source | Disposition | Consumed at |
|---|---|---|
| Framing ① — subject boundary is conversion at the seams | Constraint | DD-002; §Context (tested, holds) |
| Framing ② — runtime-side declaration as the working direction | Direction | DD-001 §Comparison (tested on merit, holds) |
| Framing ③ — state unit semantics, add no ABI function | Direction | DD-004 §Does the host need the scale (tested, holds) |
| Framing ④ — four-DD slate | Structure | §Decisions |
| Framing ⑤ — verification environments and evidence split | Constraint | §Verification closure items 3–5 |
| Framing — product-merit comparison prior | Discipline | every DD §Comparison |
| [constraints §1](../requirements/constraints.md) — the three concrete facts of DPI-unawareness | Starting point | §Context; DD-001 / 002 / 003 one-to-one |
| [constraints §2](../requirements/constraints.md) — §3.1's real subject is DirectWrite hinting precision | Decision input | DD-002 §The rasterization surface |
| [constraints §3](../requirements/constraints.md) — one of six M3 residual lines | Scope | §Out of scope; framing scope table |
| [constraints §4](../requirements/constraints.md) — the seven coordinate-carrying paths | Audit target | DD-002 §The conversion sites (the gate's audit table) |
| [constraints §5](../requirements/constraints.md) — no spec defines a unit; tension with the plan | Decision input | DD-004 |
| [constraints §6](../requirements/constraints.md) — the declarative host boundary | Constraint | DD-001 §Where the declaration lives |
| [constraints §7](../requirements/constraints.md) — capture procedure is coupled to DPI, in both directions | Verification premise | §Verification closure; §Upstream revisions |
| [constraints §8 / §9 / §10](../requirements/constraints.md) — append-only revision history, evidence–surface coupling, process discipline | Process premise | §Upstream revisions; §Verification closure item 3 |
| [DD-V-022](../../../cross-milestone/decisions/dpi-awareness-m4-deferral.md) | Settled premise | §Context (AC7 wording; M4 ownership not re-litigated) |
| [layout-engine.md §3.1](../../../../docs/notes/layout-engine.md) | Open question being closed | DD-002; DD-004 §Notes |
| [DD-V-026](../../../cross-milestone/decisions/plan-revision-discipline.md) | Procedure | §Context (① / ③ tested without a revision being needed) |
| Workspace source at drafting time (`window.rs`, `text.rs`, `widget.rs`, `runtime.rs`, `abi.rs`, `Cargo.toml`) | Ground truth | §Context; DD-002 §The conversion sites |

## Revision history

| Date | Change |
|---|---|
| 2026-07-28 | Initial draft (Status: Proposed). All four DDs at Proposed pending owner review. Framing-level owner alignment confirmed 2026-07-28 ([../requirements/framing.md](../requirements/framing.md) §オーナー合意の記録). Framing agreements ①, ②, ③ each tested inside the slate; none required a stage-2 plan revision. |
| 2026-07-28 | Accepted flip. Owner approved the slate as drafted, with no change requested — including the three points raised for review: DD-002's recommendation (layout in DIP, conversion at the seams, the rasterization surface at device resolution), DD-004's outer-window-rectangle reading of `width` / `height`, and DD-003's fixed `WM_DPICHANGED` ordering. Preamble and DD-001 through DD-004 flipped `Status: Proposed` → `Accepted`. Moment 1 design sync follows. |
