### DD-M2-P1-006 — Build-order edge between cdylib shim and final binaries

**Status:** Accepted (2026-05-03)

**Context:**
After implementing DD-M2-P1-001..005 in a working tree,
`cargo clean && cargo build --release --workspace` reproducibly failed
with `LNK1181: cannot open input file 'wasamo.dll.lib'`. Diagnosis: the
cargo dependency graph had no edge between `wasamo-dll` (cdylib
producer of `wasamo.dll.lib`) and the final binaries (`counter-rust`
etc.) that consume it via `bindings/rust-sys`'s `#[link]`. Cargo
parallelised them, and the linker for `counter-rust` ran before the
cdylib finished. The `#[link]` attribute alone does not create a build
order edge — cargo only orders crates that appear in some `dependencies`
table.

**Options:**

Option A — Add `wasamo-dll` to `[dependencies]` of `bindings/rust-sys/Cargo.toml` (recommended)
- One edge covers every binary that links the C ABI (all Rust hosts
  go through `rust-sys`).
- Verified locally: `cargo clean && cargo build --release --workspace`
  succeeds; `dumpbin /exports target/release/wasamo.dll` shows all 19
  ABI symbols; `cargo run -p counter-rust --release` works
  end-to-end.
- What you give up: cargo emits `warning: the package wasamo
  provides no linkable target` (rust-lang/cargo#6313) for every
  build, because a cdylib has no Rust-linkable surface and `rust-sys`
  is a normal Rust crate. Accepted as a deferred / open issue —
  recorded in [`docs/notes/cdylib-shim-build-graph.md`](dd-m2-p1-006-build-order-edge-between-cdylib-shim-and-final-binaries.md)
  with explicit re-evaluation triggers.

Option B — `[build-dependencies] wasamo-dll` or `artifact = "cdylib"`
  - What you give up: `[build-dependencies]` triggers host-target
    double build → filename collision on `wasamo.dll`. The
    `artifact`/`-Z bindeps` mechanism is unstable on stable Rust and
    has had similar collision behaviour in tested forms. Not
    actionable today.

Option C — Add `wasamo-dll` to `[dependencies]` of each Rust binary individually
  - What you give up: Fragile — every new Rust binary added to the
    workspace would silently regress to LNK1181 if the maintainer
    forgot the extra line. Centralising the edge in `rust-sys` (which
    every Rust host already depends on) is strictly safer.

**Decision:** Option A — Accepted (2026-05-03). The `no linkable target`
warning is accepted as a known wart, not a settled end-state; the
note records re-evaluation triggers (cargo making the warning a hard
error; a second cdylib-only build-order dependency appearing; a real
need to consume `wasamo-dll`'s Rust surface). If any trigger fires,
revisit this DD.

---
