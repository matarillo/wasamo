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

### Implication for future ADRs

When a future ADR (M2-Phase 4/5/6 or later) prescribes a verification
path, name the environment kind explicitly:

- "build verification on CI runner" — fully covered by GitHub Actions
- "link/export verification on SSH dev box" — same as DD-M2-P1-005
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
