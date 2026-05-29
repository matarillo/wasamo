### DD-M2-P1-005 — How the shim re-exports the C ABI symbols

**Status:** Accepted

**Context:**
The `#[no_mangle] pub extern "C"` symbols defined in
`wasamo-runtime` need to end up exported from the cdylib produced by
`wasamo-dll`. Three mechanisms exist.

**Options:**

Option A — Whole-archive link of the rlib via build.rs (recommended)
- `wasamo-dll/build.rs` emits a MSVC-specific whole-archive link
  argument to force all rlib symbols into the cdylib output.
- `wasamo-dll/src/lib.rs` is minimal (a crate-level `extern crate`
  or empty body as implementation determines).

  - What you gain: Zero per-symbol maintenance. New ABI symbols in
    `wasamo-runtime` automatically appear in `wasamo.dll`. Standard
    Rust cdylib-shim pattern.
  - What you give up: One MSVC-specific link arg in build.rs.
    Verified on the local SSH dev box before pushing to CI
    (`dumpbin /exports wasamo.dll` shows all current ABI symbols).

Option B — Per-symbol re-export from the shim's `lib.rs`
  - What you give up: Per-symbol maintenance burden proportional to
    ABI growth (M2-Phase 4 adds tree-mutation primitives — exactly
    the wrong scaling). Requires stripping `#[no_mangle]` from the
    rlib side, defeating its usefulness as a direct dev dependency.

Option C — `#[used]` annotations on each symbol in `wasamo-runtime`
  - What you give up: Non-idiomatic for functions in stable Rust;
    documented whole-archive is the standard cdylib-shim mechanism.

**Decision:** Option A — Accepted (2026-05-03). SSH dev box verification
(cargo build + `dumpbin /exports`) is required before pushing to CI.

---
