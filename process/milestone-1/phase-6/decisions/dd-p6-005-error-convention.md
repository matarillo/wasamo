### DD-P6-005 — Error convention

**Status:** Accepted

**Context:**
The current ABI returns `i32` (`0` = ok, `-1` = err) for `wasamo_init`
and null pointers for `wasamo_window_create`. There is no way for
host code to learn what went wrong. M1 does not need rich error
reporting, but the convention chosen now will be hard to retrofit
across every function once bindings exist.

**Options:**

Option A — Numeric `WasamoStatus` enum + `wasamo_last_error_message` (recommended)
- Every fallible function returns `WasamoStatus` (a `int32_t` enum:
  `WASAMO_OK = 0`, `WASAMO_ERR_INVALID_ARG = -1`, `WASAMO_ERR_RUNTIME = -2`, …).
- Functions that return a handle (e.g. window create) take an
  out-parameter and return `WasamoStatus`:
  `WasamoStatus wasamo_window_create(..., WasamoWindow** out)`.
- A thread-local last-error string is queryable via
  `const char* wasamo_last_error_message(void)` for diagnostics
  only — the numeric code is the contract.

- What you gain: Numeric codes are stable; bindings translate them
  into native error types. Last-error-message gives humans something
  to read without bloating every signature. Out-parameter form for
  handle-returning functions removes the null/error ambiguity in
  the current `*mut WindowState` return.
- What you give up: Slightly more verbose call sites in C. Mitigated
  by typed wrappers in higher-level bindings.

Option B — `errno`-style: handle-or-null + global last error
Keep the current "return null on failure" shape; pair with
`wasamo_last_error_message` and `wasamo_last_error_code`.

- What you gain: Minimal change to existing signatures.
- What you give up: Conflates "operation succeeded but produced no
  result" with "operation failed" for any future API that could
  legitimately return null. Globals pretending to be thread-local
  cause bugs across binding boundaries.

Option C — Rich `WasamoError*` object returned by reference
Every fallible function returns `WasamoError*` (null on success);
caller frees with `wasamo_error_free`.

- What you gain: Richest information per error.
- What you give up: Allocation per error path. Adds an entire object
  type to the stable core. Overkill for M1.

**Recommendation:** **Option A.** Numeric `WasamoStatus` enum +
out-parameters + thread-local last-error message. This is the
shape every modern C library settled on (SQLite, libgit2,
LLVM-C). The current `wasamo_*` signatures will be revised to fit
this shape (handle-returning functions become out-parameter; status
return). Keep the enum small at M1 (4-6 codes) and grow it as
needed — adding new codes is non-breaking.

---
