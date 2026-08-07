# Verification Environments

**Status:** Live note — collects observations on which environments are
suitable for which kinds of verification (build, link, GUI, …).

## Background

Different ADRs require different kinds of verification — some are
purely static (does the linker produce the expected exports?), some
require the program to run headlessly (does `cargo build` succeed?),
and some require interactive observation (does a button hover-animate
when the mouse enters it?). The right environment depends on the
verification kind, and conflating them has already caused at least one
mid-phase confusion (see *Observations* below).

## Verification kinds and matching environments

| Kind | What's checked | Environment requirement |
|---|---|---|
| Build | `cargo build` / `cargo check` succeeds | Any Rust toolchain (local, SSH dev box, CI runner). |
| Link / static export | DLL exports the expected ABI symbols (e.g. `dumpbin /exports`) | MSVC toolchain. SSH dev box or local — both equivalent. |
| Headless runtime with live Compositor | Runtime can initialize `wasamo_init`, DispatcherQueue, Compositor, TextRenderer / DirectWrite, build live `WidgetNode`s, and expose runtime property state without showing a window | Windows session with the required runtime compositor capability. GitHub Actions `windows-latest` has run this successfully; an SSH dev box may return `0x80070005` and should be classified as runtime-compositor-unavailable rather than GUI-capable. |
| Assistant-visible capture (screenshot) | Assistant launches the host, captures the rendered window to an image, and analyses the pixels (did the screen render non-blank? is the intended sub-screen present?) — a pre-owner automated baseline, no human input | **Visible Windows desktop session required** (same as GUI/interactive). Capture must be **per-monitor-DPI-aware** and use `Graphics.CopyFromScreen` over the window's `GetWindowRect`; `PrintWindow` reads back blank for the DirectComposition client area. Does not replace owner human-visible smoke. |
| GUI / interactive | Window opens; hover, click, key input, animation behave correctly | **Visible Windows desktop session required.** Local physical machine, or RDP/VNC into a dev box. Plain SSH is **not sufficient** because it provides no interactive desktop session for the spawned window. |

## Observations

### Observation 1 — GUI verification needs an interactive desktop session

ADR DD-M2-P1-003's verification target (Phase 2-5 example animation,
hover, click, [B]-key Compositor-independence test) cannot be fulfilled
by a plain SSH session into a Windows host. `cargo run` will start the
process, but the window either doesn't appear on any visible desktop
or appears in a session no human is watching. All four examples need a
human looking at pixels and operating the mouse/keyboard.

### Observation 2 — "SSH dev box" in DD-M2-P1-005 means static-link verification, not GUI

ADR DD-M2-P1-005 says "local SSH dev box verification required" for
the cdylib `+whole-archive` link path. That verification is a static
check (does `wasamo.dll` export the 20 `wasamo_*` ABI symbols?) and
is satisfiable over SSH. **It is not a license to verify other ADRs
the same way.** When DD-M2-P1-003's resurrection experiment was
prepared, this distinction was nearly missed — it would have led to
declaring the experiment "verified" without actually observing any
button hover.

### Observation 3 — Headless runtime is distinct from GUI / interactive

DD-M2-P6-011 introduced a Windows-only headless integration test
(`wasamo-runtime/tests/live_widgetnode_headless.rs`) that lowers `.ui`
source, parses the emitted IR, builds live `WidgetNode`s with the real
Compositor / TextRenderer, and reads the resulting property state through
`wasamo_get_property`. This needs no visible window and no human input,
so it is not GUI / interactive verification.

It is still not plain build or link verification. The test depends on a
Windows session that can initialize the runtime compositor stack. A plain
SSH dev box has been observed to fail `wasamo_init` with `0x80070005`;
that environment is runtime-compositor-unavailable for this verification
kind. GitHub Actions `windows-latest` has run the test binary under
`cargo test --workspace` successfully. For CI-gated evidence, tests in
this category should fail rather than silently skip when the required
runtime capability is unavailable on GitHub Actions.

### Observation 4 — Assistant-visible capture sits between headless and human

M3-Phase 5 (Grid) surfaced a fourth kind during the T5 gallery slice
evidence. `Start-Process` survival was originally offered as the
assistant's automated GUI evidence, but it cannot show that the screen
rendered non-blank or that the intended sub-screen is in view (Codex
review #1). The assistant evidence was therefore strengthened to
**launch + screenshot capture + assistant analysis** — a non-interactive
but *visible* check that is stronger than headless-runtime verification
(it observes pixels) yet weaker than human-visible smoke (no human
judgment of correctness). The capture and comparison procedure, corrected by
M4-Phase 1 evidence, is:

- Use `Graphics.CopyFromScreen` over the window's `GetWindowRect`, not
  `PrintWindow`: the DirectComposition / Visual-Layer client area reads
  back **blank** under `PrintWindow`. Bring the window foreground +
  topmost before capture. When comparing processes with different awareness
  postures, capture the **client** rectangle instead: use `GetClientRect` and
  map both corners with `ClientToScreen`. Equal outer rectangles do not imply
  equal client rectangles because the non-client frame follows DPI-indexed
  system metrics.
- The capture tooling must declare **Per-Monitor-Aware V2 and verify that it
  has that posture**. Discard the result of
  `SetProcessDpiAwarenessContext`, read `GetThreadDpiAwarenessContext` back,
  and abort on a mismatch. Declaring is not evidence that the declaration
  took effect: process awareness may already have been fixed before the tool
  ran. An unaware observer asking for an aware window's rectangle receives
  virtualized coordinates divided by the system scale, even though
  `GetDpiForWindow` and `GetWindowDpiAwarenessContext` themselves are not
  virtualized. Plausible DPI and awareness reads therefore do not make an
  unaware rectangle physical.
- **Anything that synthesizes keyboard input must acquire foreground
  activation first, verify it, and retry.** Mouse input is routed by cursor
  position and needs no activation; keyboard input is routed to the focused
  window of the *foreground* thread, so a synthesized key press without
  foreground goes to another window — or nowhere, when the session has no
  foreground window at all and `GetForegroundWindow` returns `0`.
  `SetForegroundWindow` alone is refused unless the caller is already
  foreground, so activation is **earned** with a real click inside the
  target's client area (choose a point that changes nothing being measured)
  and then **read back**, never requested and assumed. A single refusal is
  not an environment verdict: a window just created, shown and repositioned
  takes a moment to become activatable, so retry before concluding
  anything. This is a property of how Windows routes input, not of a
  particular machine — a tool that clicks-verifies-retries works on any
  interactive desktop session, and one that does not is unreliable
  everywhere.
- **A capture that delivers keys by posting `WM_KEYDOWN` instead supports a
  weaker claim, and says so.** A posted message still travels the window's
  real message loop and window procedure, so it evidences "this message
  makes the runtime do X"; it skips the OS input stack that decides *which
  window* a key reaches, so it evidences nothing about real keyboard
  delivery. Every capture records which of the two paths it used, because
  the resulting numbers look identical and a silent fallback would be
  invisible in the artifact.

The old premise for that second rule was false after M4-Phase 1: the Wasamo
runtime now declares Per-Monitor-Aware V2 itself; DWM is not bitmap-stretching
the normal host. On the T10 development desktop at 125% (120 DPI), a PMv2
harness that read its own posture back measured the following values for the
same executable and an 800 × 600 DIP request:

| | Declared PMv2 | `__COMPAT_LAYER=DPIUNAWARE` |
|---|---:|---:|
| Outer physical rectangle | 1000 × 750 | 1000 × 750 |
| Client physical rectangle | 982 × 703 | 980 × 701 |
| Non-client left / top / right / bottom | 9 / 38 / 9 / 9 | 10 / 39 / 10 / 10 |
| Extent supplied to layout | 785.6 × 562.4 DIP | 784 × 560.8 logical |

These are the exact values in the
[T10 coordinate artifact](../../process/milestone-4/phase-1/implementation/evidence/t10-capture-coordinates.md),
not a new measurement made by this note.

The existing `compare-frames.ps1` defaults (`InsetX 12`, `InsetTop 44`,
`InsetBottom 12`) still cleared every measured frame, but their margin shrank:
the 96-DPI basis had top / side insets 31 / 8 (margins 13 / 4), the declared
120-DPI window 38 / 9 (margins 6 / 3), and the unaware window on the 120-DPI
desktop 39 / 10 (margins 5 / 2). These too are T10's values, not universal
frame metrics.

The identical outer number has a different mechanism from the old note: the
runtime itself realizes `800 × 600 DIP` as `1000 × 750` physical before show.
The exact client and frame figures are measurements of that machine's theme
metrics, not universal constants. Re-derive them above 125%, when non-client
treatment changes (including a custom title bar), or for a host not created by
the runtime's normal window path. The invariant to retain is that non-client
metrics are DPI-indexed and independent of the runtime's scale multiplication.

Frame comparison has two additional provenance requirements:

- **A committed frame is not a baseline, and one capture is not a baseline.**
  Re-capture both sides in the same comparison session and require at least
  two agreeing captures per side before comparing across the change. Any
  difference is non-zero by default; a measured intensity band may classify a
  result only when its observed shape matches that band, and is never a general
  clean-pass rule.
- **Different source trees use different cargo target directories.** A source
  mutation run ends with a package clean and an accepted-source rebuild.
  Artifact timestamps, a cargo `Fresh` result, and the hash in a test
  executable's filename do not establish source identity when two trees have
  shared an artifact directory. Byte-identical restoration frames establish
  restoration only when the mutation was known to change that rendered frame;
  a render-neutral mutation can produce equality without proving which source
  was built.

This assistant baseline is a pre-owner check; it does not substitute for
the owner's human-visible smoke
([human-visible-smoke.md](human-visible-smoke.md)). CLAUDE.md
`Testing rules` lifts the evidence standard into a project-wide rule.

Visible verification (assistant capture or owner smoke) must also carry a
**positive control**: a single static frame that a coincidental
look-alike could also produce is not evidence. The M3-Phase 5 T6 smoke pinned the concrete cases — a star
track's flexibility is only proven by resizing (a fixed width can match
the ratio at one size), and an outer-bounds clip is only proven by
checking against the source *what is missing* (clipped content is
invisible), with resize as the positive control. The same discipline
generalises (e.g. conditional rendering is proven by toggling the state,
not by the initial frame). CLAUDE.md `Testing rules` carries the
project-wide statement.

### Observation 5 — `scroll_view_layout_integration` access violation when a second test reuses the process-global Compositor (root-caused; originally filed as the "teardown AV")

A `cargo test` run of the `scroll_view_layout_integration` suite has been
observed to exit with a `STATUS_ACCESS_VIOLATION` (`0xC0000005`). It was
originally filed here as a **process-exit teardown** fault on the
assumption that, because the first test's `... ok` line had printed, the
crash must be in COM / Compositor teardown. A 2026-06-05 minidump capture
**disproved that framing**: the fault is **not** at teardown or process
exit — it is in the **setup of the *next* test** (`build_widget_tree` →
`Compositor::CreateSpriteVisual`), calling through a COM vtable that lives
in an **already-unloaded `dcomp.dll`**.

This first surfaced as a recurring, diff-independent crash:

- Phase 5 T1 (diff was `wasamoc` check tests only — no widget / insertion
  path touched), recorded in
  [process/milestone-3/phase-5/retrospectives/t1.md](../../process/milestone-3/phase-5/retrospectives/t1.md).
- Phase 6 T5 (after the `append_child` → `insert_child_inner` refactor,
  which is behaviour-identical for ScrollView), recorded in
  [process/milestone-3/phase-6/implementation/log.md](../../process/milestone-3/phase-6/implementation/log.md).

Diff-independence was the first clue it was not a regression in the task
under review. The 2026-06-05 investigation (dedicated branch
`investigate/obs5-scrollview-teardown-av`) settled it by evidence.

**Reproduction (100% deterministic).** The recurrence was never random —
it is a function of libtest's thread scheduling:

| Run form | Result |
|---|---|
| `--test-threads=1` (sequential; libtest spawns a fresh thread per test) | 5/5 access violation |
| default (multi-threaded) | green |
| each test in isolation (`--exact`) | green |

A single test in isolation mirrors the production lifecycle (init once →
build → use → drop → thread exit → process exit) and is **green** — the
crash strictly requires a *second* test.

**Root cause (confirmed by minidump).** Symbolicated faulting stack:

```
windows::UI::Composition::Compositor::CreateSpriteVisual   ← faults reading the vtable slot
wasamo_runtime::widget::WidgetNode::scroll_view
wasamo_runtime::ir_loader::{construct_widget, build_node, build_widget_tree}
scroll_view_layout_integration::scroll_path_fixture_r2_three_level_visual_nesting...   ← the SECOND test
```

The faulting read targets a vtable pointer inside the address range
`dcomp.dll` occupied before it was unloaded (`dcomp.dll` appears in the
debugger's *unloaded* module list). Mechanism:

1. libtest spawns a **dedicated thread per test even under
   `--test-threads=1`** (panic isolation).
2. The first test to run creates the process-global Compositor via
   `wasamo_init` (`static OnceLock<Runtime>` in `runtime.rs`). That
   Compositor has **STA-apartment affinity to the creating thread**.
3. When that test ends, its thread — and therefore its STA apartment —
   is torn down, and the in-proc COM server `dcomp.dll` is **unloaded**
   from that apartment.
4. The next test, on a different thread/apartment, fetches the **stale
   cached Compositor** via `get_compositor()` and calls `CreateSpriteVisual`
   → the object's vtable now points into the unloaded `dcomp.dll` →
   access violation. ScrollView is incidental: any widget built by the
   first test to touch the stale Compositor would fault; the intermediate
   Visual / `InsetClip` (DD-M3-P4-004) are **not** involved.

**Disposition — hypothesis (A) confirmed, (B) excluded.** This is a
**test-harness artifact and is production-safe**:

- Production hosts call `wasamo_init` once on the main thread, which owns
  the apartment for the whole process; `dcomp.dll` stays loaded while the
  Compositor lives, and there is never a second apartment. The
  `static RUNTIME` is leaked (never dropped) at exit, so no teardown code
  runs against the Compositor.
- The ABI thread-affinity guard (DD-M2-P6-005: `OWNING_THREAD` /
  `is_owning_thread()`) forbids cross-thread ABI use, protecting
  production hosts. The integration tests reach **past** that guard into
  the internal Rust API (`get_compositor()` / `build_widget_tree`), which
  is where the foreign-thread reuse slips in.
- The earlier (B) "production could fault on shutdown" risk is therefore
  excluded; the single-test-in-isolation green result is the empirical
  confirmation.

The prior "next occurrence: capture the faulting stack" standing rule is
**discharged** (the dump was captured). The earlier fix dichotomy
("if `dcomp.dll` → never-dropped `static`; if our `layout.rs`/`widget.rs`
→ teardown-contract defect") does **not** apply: the Compositor is
already in a never-dropped `static` with no explicit `RoUninitialize`, and
the fault is not at teardown. The real defect is **cross-apartment reuse
of the process-global Compositor across libtest's per-test threads**; the
remediation belongs in **test infrastructure**, not the runtime teardown
path.

**Remediation status.** Two steps, by owner direction — **both now DONE**.

- **Step 2 — keep-alive apartment — DONE (committed).** A shared
  `wasamo-runtime/tests/common/mod.rs` first initialized the runtime on a
  dedicated thread that *parked* for the process lifetime, keeping the
  Compositor's apartment and `dcomp.dll` resident for the whole test binary.
  The five integration binaries with two or more Compositor tests
  (`scroll_view`, `conditional_toggle`, `zstack`, `wrap_panel`, `grid`) route
  through it. This made the full `wasamo-runtime` suite (333 unit + all
  integration tests) green under `--test-threads=1`, where `scroll_view` /
  `wrap_panel` / `grid` previously crashed deterministically — the safe point
  to merge CI green. Step 1 below then superseded the "merely parked" form.
- **Step 1 — marshal Compositor work onto the owning thread — DONE
  (committed).** Step 2 still left the test bodies calling the Compositor from
  their own libtest threads — cross-apartment access to non-agile Composition
  objects, safe only while `dcomp.dll` stayed resident and **not guaranteed by
  the COM apartment contract** (UB-adjacent, though **test-harness-only**;
  production unaffected — see the disposition above). Step 1 turns the parked
  thread into a **work-queue executor**: `run_on_owning_runtime_thread_or_skip`
  (replacing `init_runtime_or_skip`) ships each Compositor test body to that
  one owning thread, where the Compositor is created *and used*, catching the
  body's panic and re-raising it on the libtest thread so `#[test]` still
  reports failures. The cross-apartment access is **eliminated, not merely
  tolerated**, matching production's single-UI-thread model. Verified: full
  suite green under `--test-threads=1`; a thread-identity probe confirmed test
  bodies run on `wasamo-test-runtime-owner`, not the libtest thread; clean
  rebuild green; and GitHub Actions CI (run 27014203528, commit `4d2cb3e`,
  `cargo test --workspace`) green on the windows-latest runner.

The keep-alive/executor helper can be **deleted entirely** once the harness
stops creating the precondition — a process-per-test runner (e.g.
`cargo nextest`, each `#[test]` in its own process) or libtest no longer
spawning a thread per test; either makes per-test inline init safe again. (The
earlier list of step-1 revisit triggers is discharged: step 1 is done.) Two
forward notes remain: a new test binary with two or more Compositor tests
should adopt the same helper, and M4+ interactive GUI tests (hover / click /
animation) will need this owning thread plus a message pump.

**Regenerating the evidence (preferred over storing the binary dump).**
The crash is 100% reproducible, so the dump is not retained in git (a
57 MB full-memory dump would also leak process memory); the textual proof
above plus this recipe is the durable record:

```
# capture (Sysinternals procdump + the prebuilt test exe)
procdump -accepteula -e -ma -x <out-dir> \
  target/debug/deps/scroll_view_layout_integration-*.exe --test-threads=1
# analyse (Debugging Tools for Windows)
cdb -z <dump.dmp> -c ".reload /f; .ecxr; kn; lm; q"
#   _NT_SYMBOL_PATH must include target/debug/deps for Rust frame symbols.
```

The captured dump and the full analysis note live in `private/`
(git-ignored), consistent with how binary verification artifacts
(screenshots) are kept out of the repo.

### Implication for future ADRs

When a future ADR (M2-Phase 4/5/6 or later) prescribes a verification
path, name the environment kind explicitly:

- "build verification on CI runner" — fully covered by GitHub Actions
- "link/export verification on SSH dev box" — same as DD-M2-P1-005
- "headless runtime verification on a Windows runner with live
  Compositor capability" — no visible window, but stronger than build/link
- "assistant-visible capture on a visible desktop" — launch + screenshot
  + assistant analysis as a pre-owner baseline; no human input, but
  observes rendered pixels (Observation 4)
- "GUI/interactive verification on local or RDP-attached desktop" —
  required for any animation, hover, focus, IME, or DPI behaviour

Avoid the bare phrase "verify on SSH dev box" if the verification
includes any visual or input-driven observation.

### Implication for future mock-free Windows integration tests

When introducing a new mock-free Windows-only integration test that
follows the `runtime_compositor_unavailable` skip-guard pattern
(origin DD-M2-P6-011 in
[wasamo-runtime/tests/live_widgetnode_headless.rs](../../wasamo-runtime/tests/live_widgetnode_headless.rs);
also used in
[wasamo-runtime/tests/button_enabled.rs](../../wasamo-runtime/tests/button_enabled.rs)),
verify the skip path is actually triggered on an SSH-dev-box-class
environment (where `wasamo_init` returns `0x80070005`) before landing.
A successful local run that does **not** hit the skip path is
necessary but **not sufficient** evidence — it only proves the guard's
happy path doesn't break the test, not that the guard correctly
classifies the compositor-unavailable failure on the environments the
guard exists to protect against.

In particular, the substring match against `0x80070005` is fragile to
different HRESULTs; a guard that has never been observed to fire is a
guard that may not fire when needed. Reproduce the skip path
explicitly on a Compositor-unavailable environment and record the
observation in the relevant step-end retrospective.

CLAUDE.md `Testing rules` lifts this implication into a project-wide
rule so it is in scope for every Windows test author.

## Origin

These observations crystallised during the Phase 2-5 example
resurrection experiment on branch `exp/m2-p1-poc-examples`
(tip `d86d81c`, 2026-05-03). All four examples were verified on a
local Windows 11 machine; an SSH-only approach would not have
produced any of the observations the ADR's verification target
requires.

## Open questions

- **Is `docs/notes/` the right home for an enforceable SSOT?**
  Observation 4's capture mechanics currently act as the SSOT for *how to
  capture GUI evidence*: the normative core lives in
  [AGENTS.md](../../AGENTS.md) §Testing rules, the mechanics here (the
  M3-Phase 5 phase-end split), and
  [process/procedures/implementation-gates.md](../../process/procedures/implementation-gates.md)
  close-gate #7 links here. But `docs/notes/` is defined as *exploratory,
  owner-authored notes*, not a process SSOT — so an operational,
  enforceable rule living in an exploratory note is a category mismatch.
  If these mechanics are genuinely a process SSOT, the cleaner placement
  is to carve the mechanics out into `process/procedures/` (a proper
  procedure home). That move would supersede the M3-Phase 5 placement
  decision, so it is a **structural change to settle with a vision
  decision record**, not an in-place edit. Recorded as an open question,
  not yet decided.
