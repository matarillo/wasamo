# cdylib-shim Build Graph — `no linkable target` Warning

**Status:** Live note — accepted with deferrals
**Origin:** DD-M2-P1-006 (build-order edge between cdylib shim and final binaries, 2026-05-03)

## Background

M2-Phase 1 split the runtime into `wasamo-runtime` (rlib) and
`wasamo-dll` (cdylib shim). Final binaries (e.g. `counter-rust`) link
against `wasamo.dll.lib` via the C ABI through `bindings/rust-sys`'s
`#[link]`. The `#[link]` attribute does not create a cargo
build-order edge, so `cargo build --release --workspace` raced and
failed with `LNK1181: cannot open input file 'wasamo.dll.lib'` when
the linker for the binary ran before the cdylib finished.

DD-M2-P1-006 resolved this by adding
`wasamo-dll = { path = "../../wasamo-dll" }` to the `[dependencies]`
of `bindings/rust-sys/Cargo.toml`. That single edge propagates through
the dependency graph to every Rust binary, ordering the cdylib build
before any consumer link step.

## Status: accepted with deferrals

Cargo emits the following warning on every build:

```
warning: the package `wasamo` provides no linkable target. The
        compiler might raise an error while compiling ...
```

The cause is rust-lang/cargo#6313: a cdylib has no Rust-linkable
surface, and `bindings/rust-sys` is a normal Rust crate listing
`wasamo-dll` purely to enforce build order. Cargo cannot tell that
the dependency is intentional and warns.

The warning is **accepted as a known wart**, not a settled end-state.
The build is correct; only the diagnostic is noisy. Alternative
mechanisms examined and rejected at the time of DD-M2-P1-006:

- `[build-dependencies] wasamo-dll` — host-target double build,
  filename collision on `wasamo.dll`.
- `artifact = "cdylib"` (`-Z bindeps`) — unstable on stable Rust;
  observed collisions in tested forms.
- Per-binary `[dependencies]` — fragile: every new Rust binary added
  to the workspace silently regresses to LNK1181 if the maintainer
  forgets the line.

## Re-evaluation triggers

Re-open DD-M2-P1-006 if any of the following occurs:

- **T1 — cargo upgrades the warning to a hard error.** The current
  acceptance becomes a build failure; we must move to a different
  mechanism (or drop the build-order edge if cargo grows a first-class
  way to declare it).
- **T2 — A second cdylib-only build-order dependency appears.**
  Repeating the same `[dependencies]` workaround across multiple
  cdylibs is a smell; at that point, look for a uniform mechanism
  (build script, workspace-level ordering, stabilised `artifact`).
- **T3 — A real consumer of `wasamo-dll`'s Rust surface emerges.**
  If anything actually wants to link `wasamo-dll` as a Rust
  dependency (not just enforce build order), the dependency stops
  being a workaround and the warning's premise no longer applies.

If none of these fire, the current arrangement stands. Re-evaluation
is not on a calendar — it is event-driven by the triggers above.
