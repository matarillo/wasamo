### DD-P7-002 — `wasamo` rlib status and crate naming

**Status:** Accepted

**Context:**
If DD-P7-001 = B, two derived questions arise:

- The cdylib is named `wasamo` and emits `wasamo.dll`. The safe
  wrapper would naturally also want to be called `wasamo` on
  crates.io (it is the user-facing crate). Two crates cannot share
  a name in one workspace, and even if we publish one and not the
  other, the workspace today already has a `wasamo` rlib path that
  Phase 4/5 examples consume.
- The rlib's Rust-native API has no experimental marker. If we are
  about to publish a safe wrapper as the *real* Rust face of wasamo,
  the rlib's standing relative to it must be stated.

**Options:**

Option A — Rename the runtime crate to `wasamo-runtime`; safe wrapper takes `wasamo` (recommended)
- `wasamo` runtime crate → `wasamo-runtime` (cdylib + rlib;
  cdylib still emits `wasamo.dll` via `[lib].name = "wasamo"`).
- New `wasamo-sys` crate (raw FFI).
- New `wasamo` crate (safe wrapper) at `bindings/rust/`.
- Phase 4/5 examples that use the rlib update their dependency
  from `wasamo` to `wasamo-runtime` and are explicitly documented
  as "internal dev examples; not part of the public Rust surface."
- The rlib's Rust-native API is treated as **internal/experimental**
  (`#![doc(hidden)]` on the public re-exports it currently has, or
  a `WASAMO_INTERNAL` cargo feature gate). It is **not** retired —
  removing it would gut Phase 4/5 dev-loop infrastructure — but it
  is documented in `architecture.md` as not the supported Rust API.

- What you gain: Public Rust API ships under the obvious name
  (`wasamo`). The two Rust paths (rlib for dev, sys+safe for hosts)
  are clearly distinguished by crate name. M1's "experimental"
  qualifier applies to both the C experimental layer and the rlib
  path uniformly.
- What you give up: One crate rename, touching `Cargo.toml`,
  Phase 4/5 examples' deps, and `architecture.md` §1.

Option B — Safe wrapper takes a different name (e.g. `wasamo-rs`); cdylib keeps `wasamo`
No runtime rename. Safe wrapper crate is `wasamo-rs` or similar.

- What you gain: No rename of the runtime crate.
- What you give up: The user-facing Rust crate ships under a
  non-obvious name. Reads as "the Rust binding of the Rust
  framework," which is awkward when the framework is *primarily*
  the Rust crate. A future `wasamo-py` / `wasamo-go` naming pattern
  for sister bindings would make sense, but `wasamo-rs` for the
  Rust binding to a Rust framework reads as a workaround.

Option C — Retire the rlib entirely; runtime crate becomes cdylib-only
The Phase 4/5 examples are rewritten to use `wasamo-sys`+safe.

- What you gain: Single Rust API. No name collision (the runtime
  crate has no public Rust surface, just the DLL).
- What you give up: Rewriting the visual-check examples is
  pure churn — they exist to verify Win32/WinRT integration, not
  to demo the public API. They predate Phase 6's C ABI. Forcing
  them through the C ABI for no acceptance-criterion reason is
  exactly the kind of "implement the task list literally" the
  pre-doc discipline warns against.

**Recommendation:** **Option A.** Rename runtime crate to
`wasamo-runtime`; let `wasamo` be the safe wrapper's name. Reposition
the rlib's Rust-native API as internal/experimental in
`architecture.md`. Phase 4/5 examples remain on the rlib path.

The `[lib].name = "wasamo"` setting in `wasamo-runtime/Cargo.toml`
preserves `wasamo.dll` / `wasamo.dll.lib` filenames, so the C ABI
artifact is unaffected.

**Post-M1 implementation note (2026-05-01):** Option A was shipped as
Accepted. However, the cargo#6313 filename collision (`libwasamo.rlib`
produced by both `wasamo-runtime` and the `wasamo` safe wrapper)
escalated from a warning to an actual compile error — cargo resolved
`counter-rust`'s `wasamo` dep to `wasamo-runtime`'s rlib instead of
the safe wrapper. This made Option C (retire the rlib) effectively
necessary. The Phase 2-5 visual-check examples were deleted (their
internal Rust APIs are not accessible through the C ABI, so rewriting
them would be pure churn as this ADR's Option C analysis notes). The
`rlib` crate-type was removed from `wasamo-runtime`. The DLL filename
`wasamo.dll` and all C ABI symbols are unaffected. The long-term proper
fix — a cdylib-shim crate (`wasamo-dll`) that separates DLL output from
rlib so `wasamo-runtime` can be renamed cleanly — is planned for M2
(see `docs/architecture.md` §11.4).

---
