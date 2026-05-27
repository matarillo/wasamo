### DD-P7-005 — Zig binding strategy

**Status:** Accepted (with CI-driven fallback clause — see note below)

**Context:**
Zig has two natural strategies for consuming a C ABI:
`@cImport("wasamo.h")` (which translates the header at build time
via `zig translate-c`), or hand-written `extern` declarations.

**Options:**

Option A — `@cImport` over `wasamo.h` (recommended)
- `bindings/zig/wasamo.zig` does
  `const c = @cImport({ @cInclude("wasamo.h"); });` and re-exports
  Zig-flavored wrappers (slices for strings, tagged unions for
  `WasamoValue`, error sets for `WasamoStatus`).
- The build system points Zig at `bindings/c/` for the header and
  links `wasamo.dll.lib`.

- What you gain: No duplication. Header changes propagate
  automatically. Zig's `translate-c` is well-suited to a small,
  clean header like `wasamo.h`.
- What you give up: `@cImport` builds depend on Zig's bundled
  Clang. CI needs Zig + Windows SDK. The translated names live
  in a `c` namespace; the wrapper has to re-export with idiomatic
  shapes (acceptable, this is the wrapper's job).

Option B — Hand-written `extern` block
Mirror `wasamo.h` as Zig `extern fn` and `extern struct`
declarations.

- What you gain: No `@cImport` toolchain dependency. Drift can be
  CI-checked the same way Phase 6 already checks the Rust side.
- What you give up: A second source of truth for the same ABI.
  Phase 6 made `wasamo.h` the canonical artifact precisely to
  avoid two-source-of-truth setups (DD-P6-006 rejected `cbindgen`
  for the same reason). Hand-writing the Zig `extern` block here
  re-introduces the same problem on the Zig side.

**Recommendation:** **Option A.** `@cImport` is the Zig idiom, and
`wasamo.h` is exactly the kind of small, idiomatic-C header it
handles cleanly. CI grows by one Zig install. Drift is impossible
by construction.

**Agreement note (2026-04-30):** Adopted Option A on the
understanding that GitHub-hosted CI is the first place Zig
`@cImport` against `wasamo.h` is exercised end-to-end (the local
SSH dev box does not currently have a Zig toolchain installed; it
can be added later if needed for local iteration). If CI surfaces
a `translate-c` failure or a Windows-SDK header-resolution issue
that cannot be cleanly resolved, fall back to Option B
(hand-written `extern` block) is acceptable. The choice will be
re-evaluated on concrete CI evidence rather than speculatively.

**Implementation note (2026-05-01):** Option B (hand-written `extern`
block) was chosen during implementation. Zig 0.16.0 was found to be
installed locally (`winget`), enabling local verification before CI.
`wasamo.h` uses `__declspec(dllimport)` / `WASAMO_EXPORT` macros whose
Windows-specific expansion complicates `@cImport` on MSVC targets;
the hand-written extern block is more predictable and mirrors the
established pattern in `wasamo-sys` (Rust). The fallback clause in
the agreement was exercised. Rationale recorded in `docs/architecture.md`
§11.3 (DD-P7-005).

---
