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

### Implication for future ADRs

When a future ADR (M2-Phase 4/5/6 or later) prescribes a verification
path, name the environment kind explicitly:

- "build verification on CI runner" — fully covered by GitHub Actions
- "link/export verification on SSH dev box" — same as DD-M2-P1-005
- "headless runtime verification on a Windows runner with live
  Compositor capability" — no visible window, but stronger than build/link
- "GUI/interactive verification on local or RDP-attached desktop" —
  required for any animation, hover, focus, IME, or DPI behaviour

Avoid the bare phrase "verify on SSH dev box" if the verification
includes any visual or input-driven observation.

## Origin

These observations crystallised during the Phase 2-5 example
resurrection experiment on branch `exp/m2-p1-poc-examples`
(tip `d86d81c`, 2026-05-03). All four examples were verified on a
local Windows 11 machine; an SSH-only approach would not have
produced any of the observations the ADR's verification target
requires.
