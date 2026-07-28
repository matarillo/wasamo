# DD-M4-P1-001 — DPI-awareness declaration: level, site, actor, failure handling

**Status:** Accepted
**Phase:** M4-Phase 1
**AC:** AC7, first requirement ("declare process / window DPI
awareness")

## Context

Windows decides how to treat a process's window geometry from the
process's (or thread's, or window's) declared **DPI awareness**. A
process that declares nothing is *unaware*: the OS reports 96 DPI to it
unconditionally, lets it lay out as though every monitor were at 100%,
and then has DWM stretch the finished window as a bitmap to the
monitor's real scale. Every element blurs uniformly, and the blur is
invisible on a 100% monitor — which is exactly the situation Wasamo is
in today
([constraints §1](../requirements/constraints.md)):
`create_hwnd` calls no awareness API and no application manifest ships
with any host.

Declaring awareness is therefore the precondition for everything else in
this phase. It is also the decision with the sharpest boundary
consequence: **the choice of *where* the declaration lives decides
whether the three example hosts stay declarative.** The C and Zig hosts
today embed a compiled `.uic` and call `wasamo_load_ui`; they own no
window code and no per-platform build assets
([constraints §6](../requirements/constraints.md)). An application
manifest would change that for all three.

Two facts narrow the space before the options are enumerated:

- **The stated OS floor is Windows 10 1809+**
  ([docs/architecture.md](../../../../docs/architecture.md)).
  `SetProcessDpiAwarenessContext` arrived in 1703 and `GetDpiForWindow`
  in 1607, both below the floor. There is no version at which Wasamo is
  supported and these APIs are absent, so **no legacy fallback needs to
  be designed**. (The `windows` crate binds these as static `user32`
  imports; on a hypothetical pre-1703 system the DLL would fail to
  *load*, not fail at the call site. That is a consequence of the stated
  floor, not a new one introduced here.)
- **The `Win32_UI_HiDpi` feature of the `windows` crate is not enabled**
  in `wasamo-runtime/Cargo.toml`. Enabling it is a prerequisite of any
  option here except the manifest one.

## Sub-issues

- **Level** — which awareness level to declare.
- **Site and actor** — who declares it and at which point.
- **Failure handling** — what happens when the declaration does not take
  effect.
- **Reliance on the level's automatic behaviour** — how much of V2's
  automatic non-client-area scaling to depend on.

## Level

### Options

- **L1 — Per-Monitor-Aware V2**
  (`DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2`)
  - The window's DPI follows the monitor it sits on; the process is told
    about changes through `WM_DPICHANGED` with a suggested window
    rectangle.
  - What you gain over V1: the **non-client area scales automatically**
    (caption, borders, system menu, scroll bars) without the process
    calling `EnableNonClientDpiScaling` in `WM_NCCREATE`; dialogs and
    common controls scale; `WM_GETDPISCALEDSIZE` is delivered before the
    change so a process that wants to negotiate its own size can. The
    suggested rectangle is reliable, which is what makes the
    drag-between-monitors case behave.
  - What you give up: nothing this phase is in a position to want. V2 is
    a superset of V1's contract.

- **L2 — Per-Monitor-Aware V1**
  - Same per-monitor model, but the non-client area does **not** scale
    automatically. The process must call `EnableNonClientDpiScaling`
    during `WM_NCCREATE`, and the recommended-rectangle behaviour is
    weaker.
  - Rejected on merit: it asks Wasamo to hand-manage a surface it does
    not paint and has no opinion about, in exchange for nothing. The
    only reason to prefer V1 is compatibility with pre-1703 Windows, and
    the stated floor removes that reason.

- **L3 — System-DPI-aware** (one scale factor decided at process start)
  - The process is told the primary monitor's scale once and is
    bitmap-stretched on any other monitor.
  - Rejected on merit: AC7 says *per-monitor* in its title and *across
    monitors* in its third requirement. L3 cannot satisfy the third
    requirement at all, and it would put positive control C — following
    a scale change — permanently out of reach. Choosing it would be
    re-litigating the acceptance criterion, not implementing it.

- **L4 — Unaware, with GDI scaling**
  - Status quo plus an OS compatibility shim. Rejected: it *is* the
    failure this phase exists to remove.

### Comparison

L3 and L4 fail AC7 by construction, so the real comparison is L1 against
L2, and it is one-sided: V2 supplies automatic non-client scaling and a
trustworthy suggested rectangle, and it costs nothing that this phase
values. The tie-breaker (V1 would need `EnableNonClientDpiScaling`
wiring in a `WM_NCCREATE` branch that `wnd_proc` does not currently
have) is not needed to decide it.

The forward-looking argument reinforces rather than carries the choice:
M4-Phase 8 puts a second window on a possibly-different monitor and
M4-Phase 9 may introduce top-layer host windows. V2's per-window model
is what those consume; V1 would have to be migrated first.

### Recommendation

**L1 — Per-Monitor-Aware V2.**

## Reliance on V2's automatic non-client scaling

**Rely on it in full.** Wasamo paints no non-client area: `create_hwnd`
uses `WS_OVERLAPPEDWINDOW` with the standard frame, and the visual
content lives entirely in the Composition tree over the client area
(`WS_EX_NOREDIRECTIONBITMAP` + the DWM Mica backdrop). There is no
custom caption, no owner-drawn frame, and no menu. Nothing in the phase
needs to intervene between the OS and the frame.

**Trigger for re-examination:** if M5's theming wave introduces a custom
title bar or extends the client area into the frame, this reliance must
be re-decided at that point — the automatic behaviour would then be
scaling a surface Wasamo also paints.

## Where the declaration lives

### Options

- **P1 — Application manifest, one per host executable**
  - The classic Windows answer: `<dpiAwareness>PerMonitorV2</dpiAwareness>`
    in each executable's embedded manifest.
  - What you gain: the awareness is established by the loader before a
    single line of user code runs, so no ordering question exists at
    all, and it cannot be defeated by a call-order mistake.
  - What you give up: **the declarative-host boundary**
    ([constraints §6](../requirements/constraints.md)). Each of the three
    hosts would need a manifest resource and the build-system wiring to
    embed it — an `.rc` plus a resource-compiler step for the C/CMake
    host, a build-script or linker argument for the Rust host, and the
    equivalent for `zig build`. The M3 shape — "the host embeds a
    `.uic` and calls in" — becomes "the host embeds a `.uic`, ships a
    platform manifest asset, and calls in". Worse, the burden lands on
    *every future host binding* Wasamo acquires, which is precisely the
    property the declarative boundary exists to avoid: a runtime
    behaviour that only works if each binding author remembers a
    platform ritual. Rejected on merit — this is a product-boundary
    argument, not a cost argument.

- **P2 — Runtime DLL, inside `wasamo_init`**
  - `runtime::init()` calls `SetProcessDpiAwarenessContext` before it
    does anything else.
  - What you gain: hosts are untouched — no manifest, no build change,
    in any of the three. `wasamo_init` is already the contractual
    "first call, on the owning thread" seam (abi_spec §6; `init()`
    already captures the owning thread there), so ordering has a
    natural, documented home. One call site, greppable, testable.
  - What you give up: the ordering is a runtime obligation rather than a
    loader guarantee — the declaration must be made before anything
    that would lock the process's awareness in, and before the first
    window. Bounded and enforceable (see §The ordering obligation), but
    real.

- **P3 — Runtime DLL, per-thread at window creation**
  (`SetThreadDpiAwarenessContext` in `window::create`)
  - What you gain: it scopes the change to Wasamo's UI thread rather
    than to the whole process, which is the polite thing to do inside a
    host that has its own unrelated windows.
  - What you give up: it is the right tool for a *component embedded in
    a foreign UI*, which Wasamo is not — Wasamo owns the window and the
    message loop (`wasamo_run` pumps it). More importantly, thread
    context is a stack the caller can push and pop, so the awareness in
    effect during any given callback becomes a function of who called
    whom. And a host that legitimately wants a different process-wide
    posture is better served by P2's tolerant failure path, which
    already defers to it. Rejected on merit.

- **P4 — `DllMain`**
  - Rejected: `DllMain` runs under the loader lock, and silently
    changing a process's DPI posture as a side effect of *loading a
    library* is hostile to any host that had its own intent. The
    ordering it would buy is not worth it.

### Comparison

P1 is the only option with a loader-level ordering guarantee, and that
guarantee is genuinely worth something. It loses anyway, on the boundary
argument: it converts a runtime property into a per-host, per-binding
build obligation, and the shape it damages ("hosts embed and call") is
one M3 spent three phases establishing across three languages. Under
the product-merit prior, "the runtime owns its own platform posture" is
the better product, and the ordering P1 would have given is recoverable
by construction inside P2.

P3 is the right answer to a question Wasamo is not asking; P4 trades
away host trust for the same ordering P2 can obtain honestly.

Framing agreement ② named P2's direction; this comparison reaches it on
merit rather than inheriting it, and records P1's real advantage rather
than dismissing it.

### Recommendation

**P2 — the runtime DLL declares, inside `wasamo_init`.** No host gains a
manifest asset or a build-system change; the confirmation that all three
hosts still build unchanged is part of this DD's verification.

## The ordering obligation

The declaration is the **first act of `runtime::init()`** — before
`CreateDispatcherQueueController`, before `Compositor::new`, before
`TextRenderer::new`, and (necessarily) before any window exists. Two
reasons:

- Process DPI awareness can only be set while it is still unset;
  anything that reads it first can lock it. Putting the call first
  removes the need to reason about which of the WinRT initialisations
  might do so.
- `TextRenderer::new` builds the D2D/D3D device stack. Device creation
  happening under a known awareness posture is one less variable when
  crispness is investigated later.

`init()`'s existing early return (`if RUNTIME.get().is_some()`) means a
second `wasamo_init` must not re-declare. The declaration is made once,
guarded by the same one-shot the runtime already uses.

## Failure handling

`SetProcessDpiAwarenessContext` fails with `ERROR_ACCESS_DENIED` when
the process's awareness has **already been set** — by a manifest, or by
an earlier call. That is not a pathological case: it is exactly what
happens when Wasamo is loaded into a host that is already a DPI-aware
Windows application and declared its own posture. Failing `wasamo_init`
there would break a legitimate host for doing the right thing.

### Options

- **F1 — Hard error.** `wasamo_init` returns `WASAMO_ERR_RUNTIME`.
  Rejected on merit: it converts "the host already handled this" into
  "the runtime refuses to start."
- **F2 — Tolerate, then run against the *effective* awareness.** The
  declaration is attempted; whatever the outcome, the runtime proceeds
  and derives every scale factor from `GetDpiForWindow` on the actual
  window. The outcome of the attempt is recorded as a diagnostic.
- **F3 — Tolerate silently and assume scale 1.** Rejected: it is the
  one option that can be *wrong*. If the process is per-monitor aware
  (via the host's manifest) and the runtime assumes 1, every conversion
  is wrong on a scaled monitor — the worst failure mode in the phase,
  and invisible at 100%.

### Recommendation

**F2.** The property that makes it safe is structural, and it belongs to
DD-M4-P1-002's design rather than to a defensive branch here: **the
conversion machinery is unconditional.** The runtime never asks "did my
declaration succeed"; it asks the OS what this window's DPI is. For an
unaware process `GetDpiForWindow` returns 96, the scale is 1.0, and the
scaled code path runs with a scale of exactly one. There is no
second code path to keep correct, and the scaled path is exercised on
every machine including 100% ones.

What the runtime must **not** do is claim more than it delivers: if the
effective awareness is below Per-Monitor-Aware V2, AC7's crispness
guarantee does not hold, and that is a fact a developer needs to be able
to see. The recorded diagnostic (through the existing last-error
thread-local mechanism, as a diagnostic string rather than a returned
error status) is the disclosure, and the integration test below is the
assertion.

**No new ABI surface is added for this.** A host-visible "what awareness
is in effect" query was considered and is not needed in M4: the only
in-phase consumer is the test, which can call
`GetWindowDpiAwarenessContext` directly. This is part of what keeps
framing agreement ③ intact (see
[DD-M4-P1-004](./dd-m4-p1-004-unit-contract-and-spec-wording.md)
§Does the host need the scale factor).

## Verification

Per the framing's DD ↔ verification mapping:

- **Windows integration test (mock-free, CI-gated, fail-not-skip).**
  After `wasamo_init` and `wasamo_window_create`,
  `GetWindowDpiAwarenessContext(hwnd)` compared against
  `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2` with
  `AreDpiAwarenessContextsEqual`. This asserts the *level actually in
  effect*, which is the thing that matters — not that a particular
  function was called.
  **Skip-guard discipline:** if the test needs a guard for an
  environment without a usable window station, that guard must be shown
  to fire on an environment that actually lacks the capability before
  the test lands
  ([verification-environments.md](../../../../docs/notes/verification-environments.md)).
  A guard verified only on the happy path is not verified.
- **All three hosts build and run unchanged.** The C, Rust, and Zig
  hosts are rebuilt with no manifest asset and no build-system edit.
  This is the auditable artifact for the declarative-host boundary
  claim — the point of choosing P2 over P1 is falsifiable exactly here.

## Forward-compat exposure

All additive; none reshapes what this DD ships.

1. **Per-window awareness contexts** — V2 is declared process-wide; if
   M4-Phase 8 or M4-Phase 9 ever needs a window at a different context
   (an unusual but legal shape), it arrives as a per-window override on
   top of the process default, not as a change to it.
2. **Custom non-client area** — the reliance trigger recorded above;
   lands with M5 theming if it fires.
3. **Host-visible awareness / scale query** — deferred with its trigger
   in DD-M4-P1-004; would land in the M4-Phase 7 ABI wave.
4. **Mixed-mode hosting** (Wasamo content inside a foreign window) —
   would reopen P3, and would be a change to the window-ownership model
   well beyond a DPI decision.

## Technical risk re-evaluation

- **Enabling `Win32_UI_HiDpi` grows the build surface.** It is a
  feature-flag addition to an already-large feature list in
  `wasamo-runtime/Cargo.toml`; no new crate dependency. Low.
- **The declaration silently not taking effect** is the failure this
  DD's integration test exists to catch, and it is the reason the test
  asserts the *effective* context rather than the call's return value.
- **CI environments may not present a real desktop.** The assertion is
  about the process/window awareness context, which does not require a
  visible desktop — but it does require a window. If the CI runner
  cannot create one, the test must **fail rather than skip**, per
  [AGENTS.md §Testing rules](../../../../AGENTS.md).
- **Order-dependence inside `init()`** is a discipline point, mitigated
  structurally by putting the call first rather than by a comment. If a
  future change inserts OS work ahead of it, the integration test is
  what notices.
- **Host builds are the falsifier for the boundary claim**, so they must
  actually be run — not assumed from "we did not edit them."

## Revision history

- 2026-07-28: Initial draft (Status: Proposed).
- 2026-07-28: Accepted flip following owner approval of the phase slate; no
  change requested to the recommendations or their comparisons.
