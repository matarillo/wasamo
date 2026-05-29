### DD-M2-P3-003 — Handler error / panic policy

**Status:** Accepted

**Context:**
Runtime evaluation of handler bodies (DD-M2-P3-001 = A) can raise
errors. Concrete sources at M2 scope:

1. Type errors that escaped `wasamoc` (should be impossible if
   DD-M2-P2-003 = B holds, but the runtime cannot assume the IR
   was produced by a correct compiler — for hot reload and for
   robustness, the runtime treats malformed IR as a runtime error).
2. Arithmetic overflow on `+=` / `-=` / `*=` (Rust default panics
   in debug, wraps in release).
3. Future: division by zero, array index out of bounds, etc. (not
   reachable from `counter.ui` but architectural).
4. Internal runtime panics inside the evaluator (bug in the
   interpreter itself).

Whatever happens in the handler must not unwind through the C ABI
boundary. Panics across `extern "C"` are undefined behaviour in
Rust and would corrupt every binding's stack discipline.

**Options:**

Option A — Catch-and-log; continue event loop (recommended)
- The interpreter wraps each handler invocation in
  `std::panic::catch_unwind` (or equivalent error-channel for
  expected errors). On error, the runtime logs to a configurable
  sink (default: stderr for M2; pluggable hook for M3+) and returns
  control to the event loop. The signal dispatch continues to
  registered host listeners (DD-M2-P3-002 = B keeps inline and host
  paths separate, so a handler error does not poison the listener
  iteration).
- For arithmetic overflow specifically, the interpreter follows the
  Rust release default (wrapping) for M2 — overflow is not
  classified as an error. Documented in the IR semantics note.

- What you gain: No UB at the C ABI boundary. The application stays
  responsive after a handler bug; the user sees a logged error
  rather than a process crash. Logging output is sufficient for M2
  scale (Hello Counter; one developer reading their own terminal).
- What you give up: Silent recovery can mask bugs in development if
  the logger output is not surfaced. Mitigated by stderr being the
  default sink; the developer sees it during `cargo run`.
- **Technical risk: Low.** `catch_unwind` semantics are
  well-understood; the only subtlety is ensuring no resource
  acquired by the handler (e.g. an interior `RefCell` borrow on the
  property storage) outlives the unwind. Addressable with discipline
  (release borrows before invoking user code, idiom already used in
  observer dispatch).

Option B — Catch + propagate to a host error callback
- Same as A, but instead of (or in addition to) logging, the runtime
  invokes a host-registered error callback (`wasamo_set_error_handler`
  or similar).

- What you gain: Hosts can route errors into their own logging /
  telemetry / crash-report stack.
- What you give up: New ABI surface (the error callback registration)
  not justified by M2 acceptance criteria. Adds a re-entrancy edge
  during error recovery (host callback invoked while the runtime is
  cleaning up from a handler error). Premature for M2.
- **Technical risk: Low–medium.** The mechanism is small but the
  re-entrancy contract during error recovery has to be thought
  through, which is work for no current acceptance benefit.

Option C — Crash the process (no catch)
- Treat handler errors as bugs that should fail loud. Let the panic
  propagate; rely on the host's panic handler to clean up.

- What you give up: UB at the C ABI boundary (unwind across
  `extern "C"` is UB unless every export uses
  `extern "C-unwind"`; that is not currently the case in
  `wasamo-runtime`'s ABI surface). A bug in one handler in one
  widget kills the entire host process. Hostile to embedded use
  cases (a UI library should not bring down the host on a malformed
  click).
- **Technical risk: High.** UB is the high-risk shape. Even if the
  whole ABI surface were converted to `extern "C-unwind"`, every
  binding language would need to handle Rust unwinds, which is an
  open research area in places (Zig especially). Rejected.

**Recommendation:** **Option A.**

A is the minimum mechanism that prevents UB and keeps the runtime
usable as a library. The pluggable error-sink (B) is a non-breaking
extension; a future ADR can add it when a concrete host requirement
appears (M3+ or external user). C is rejected on UB grounds.

**Logging contract for M2:** Errors are written to stderr in a
single line of the form
`wasamo: handler error in <component>.<widget-path>.<signal>: <message>`.
The exact format is implementation detail, not a contract; the
contract is "errors are visible by default and don't crash the
host". Format may be revisited when M3 introduces structured
diagnostics.

**Technical-risk re-evaluation:** Option A is the lowest-risk
choice that meets the constraint (no UB at the ABI boundary).
Option B's risk-vs-benefit is unfavourable at M2 (mechanism cost
without acceptance demand). Option C's risk is disqualifying.

---
