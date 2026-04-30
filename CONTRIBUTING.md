# Contributing to Wasamo

## How to add a language binding

Wasamo exposes a stable C ABI (`bindings/c/wasamo.h`). A binding for a
new language wraps that ABI. This document describes the conventions
established in M1 so that future bindings stay consistent.

### 1. Crate / package layout

Place the new binding under `bindings/<language>/`. The M1 bindings are:

| Path | Language | Notes |
|---|---|---|
| `bindings/c/` | C | Header + CMake template; no wrapper code needed |
| `bindings/rust-sys/` | Rust (raw FFI) | Raw `extern "C"` declarations; not for direct host use |
| `bindings/rust/` | Rust (safe wrapper) | Idiomatic API over `rust-sys`; **this** is the public Rust API |
| `bindings/zig/` | Zig | Hand-written extern block + idiomatic wrappers |

For languages where raw-FFI and safe-wrapper are naturally separate
(e.g. Rust with its `*-sys` convention), use a two-crate/two-package
sys + safe pair. For languages where the two layers are naturally
merged (e.g. Zig, Go, Swift), a single package is fine.

### 2. Scope per phase (DD-P7-004)

Each binding ships Hello-Counter-minimal coverage first, not full ABI
coverage. The minimum set for a new binding is the symbols required to
run the Hello Counter example:

**Stable core (required)**

- `wasamo_init` / `wasamo_shutdown` / `wasamo_last_error_message`
- `wasamo_window_create` / `wasamo_window_show` / `wasamo_window_destroy`
- `wasamo_run` / `wasamo_quit`
- `wasamo_get_property` / `wasamo_set_property`

**M1 experimental (required for Hello Counter)**

- `wasamo_text_create` / `wasamo_button_create`
- `wasamo_vstack_create` / `wasamo_hstack_create`
- `wasamo_window_set_root`
- `wasamo_button_set_clicked`

Observers (`wasamo_observe_property` / `wasamo_unobserve_property`) and
generic signal connect/disconnect may be omitted until a phase that
introduces a concrete consumer.

### 3. Experimental module convention (DD-P7-003)

Every binding must expose the `WASAMO_EXPERIMENTAL`-marked symbols in
a clearly-separated namespace:

| Language | Convention |
|---|---|
| Rust | `wasamo::experimental` submodule |
| Zig | `wasamo.experimental` namespace (pub const struct) |
| C | Use `wasamo.h` directly; `WASAMO_EXPERIMENTAL` macro marks each symbol |
| Other | A similarly-named sub-namespace or module |

This makes it visually obvious at the call site which symbols are subject
to breakage in M2+.

### 4. Linking against wasamo.dll.lib

`wasamo.dll.lib` (the Windows import library) is produced by
`cargo build --release --workspace` at `target/release/wasamo.dll.lib`.
It is **not** checked into the repository; consumers build it locally or
obtain it from a release artifact.

Each binding's build script must locate and pass the import library to
the linker. The established patterns are:

| Language | Mechanism |
|---|---|
| Rust (`wasamo-sys`) | `build.rs` with `cargo:rustc-link-lib=dylib:+verbatim=wasamo.dll.lib` |
| Zig | `build.zig` with `module.addObjectFile(.{ .cwd_relative = wasamo_lib_path })` |
| C / CMake | `add_library(wasamo SHARED IMPORTED)` + `IMPORTED_IMPLIB` |

For CI, add a step after the `cargo build --release --workspace` step
that invokes the binding's own build and smoke test. See
`.github/workflows/ci.yml` for the existing C, Rust, and Zig steps.

### 5. Smoke test requirement

Every binding must include a **link-resolution smoke test**: a small
program that takes the address of (or otherwise forces the linker to
resolve) every declared ABI symbol, without calling into the runtime.
Calling the runtime requires `wasamo_init` on a UI thread with a
message loop, which is not available in a normal test harness.

See `bindings/rust-sys/src/lib.rs` (`link_smoke::symbols_resolve`),
`bindings/c/smoke.c`, and `bindings/zig/smoke_test.zig` for examples.

### 6. Calling convention

All functions in `wasamo.h` are declared `__cdecl` (`WASAMO_API`). On
x64 Windows `__cdecl` and the System V / Microsoft x64 calling convention
are equivalent; no special annotation is needed in most languages. Where
the language requires an explicit annotation, use:

| Language | Annotation |
|---|---|
| Rust | `extern "C"` |
| Zig | `callconv(.c)` (Zig 0.16+) |
| C | default (no annotation needed; `__cdecl` is the default on x64 MSVC) |

### 7. `!Send` / thread-safety markers

The runtime is strictly UI-thread-affine. Binding types that wrap
`WasamoWindow*` or `WasamoWidget*` handles must not be `Send`. In Rust
this is enforced with `PhantomData<*const ()>`; in Zig and C it is
documented convention only.
