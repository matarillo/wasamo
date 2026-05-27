### DD-M2-P6-009 — IR loader malformed-input validation policy

**Status:** Accepted

**Context:**
The IR loader (DD-M2-P6-006 = A, in-runtime) reads textual IR
produced by `wasamoc`. M2 co-builds `wasamoc` and
`wasamo-runtime` from the same workspace; in M2 the loader can
trust its input *for correctness*. Post-M2 scenarios (hot
reload, ahead-of-time-built IR shipped with bindings) introduce
the possibility of stale or malformed IR; the validation policy
written now sets the post-M2 defensiveness baseline. Cross-refs
DD-M2-P6-005 (error reporting) for how detected errors surface
to the host.

**Options:**

Option A — Strict
- Every node validated for structure, type, and reference
  resolution; any irregularity fails the load.
- What you gain: maximum safety; clear diagnostics.
- What you give up: validation cost on every load (small for
  M2 IR sizes); two parses (one to validate, one to build) or
  validation interleaved with construction. M2 doesn't need
  this defensiveness against its own emitter.

Option B — Lenient
- Build whatever parses; warn on unknown tags; keep going.
- What you gain: forward compatibility (newer `wasamoc` writes
  tags an older runtime ignores).
- What you give up: malformed IR may produce a partially
  constructed tree; failure modes are silent; post-M2 hot
  reload inherits a debugging hazard. Forward compatibility
  is also a non-goal in M2 (single-workspace co-build).

Option C — Defense-in-depth (recommended)
- `wasamoc` output is *trusted* in the sense that the loader
  performs lightweight checks rather than re-validating
  every invariant the emitter is responsible for. The loader
  strictly verifies:
  - Magic + version line (DD-M2-P6-002 header).
  - Reference resolution (every name referenced by a binding
    or handler resolves to a declared signal/widget).
  - Top-level structure (the IR has the expected document
    shape).
- Anything else the parser would accept structurally is
  trusted; type-level invariants the emitter establishes
  (e.g. binding expression result type matches target
  property type) are *not* re-checked at load.
- The emitter's type-checking pass (DD-M2-P6-004 = B's "check"
  activity, restricted to `i32` and string for M2) is the sole
  guard on binding-expression / property-type integrity. The
  runtime is permitted to assume that every binding
  expression's evaluated `PropertyValue` matches its target
  property's declared type; mismatches indicate a `wasamoc`
  bug, not a recoverable load-time error, and surface as
  whatever evaluation behaviour the type mismatch produces (no
  guaranteed diagnostic). This trust placement is what makes
  DD-M2-P6-007's monomorphic per-type registry sound at load
  time.
- What you gain: cheap; aligned with the M2 trust model
  (single-workspace co-build); correct defensiveness for
  the post-M2 stale-IR scenario (header + reference
  resolution catch the realistic failure modes); diagnostics
  via DD-M2-P6-005's last-error API.
- What you give up: a deliberately unverified surface (the
  emitter's per-node invariants); acceptable because that
  surface is `wasamoc`'s test responsibility, not the
  loader's.

**Recommendation:** **Option C.**

The trust gradient maps the realistic failure modes:
header/version mismatch (post-M2 scenarios), reference
resolution failure (any time), and structural malformation
(parser-level) are all detectable cheaply. Re-validating
emitter-side invariants doubles the spec without catching
failures the test suite for `wasamoc` already addresses.

Detected errors surface through DD-M2-P6-005's last-error
mechanism with a status code distinct from
`WASAMO_ERR_OBSERVER_MUTATION` (suggested:
`WASAMO_ERR_IR_MALFORMED`).

**Forward-compat exposure:**

- Out-of-scope items engaged: post-1.0 hot reload (the
  realistic stale-IR scenario); M5 LSP/diagnostics.
- C is additive: hot reload exercises the existing
  validation paths; LSP attaches to the same diagnostic
  channel.
- A's per-node validation is rebuilt against M3 grammar
  expansion; B's lenient mode is incompatible with hot reload
  defensiveness goals.

**Technical-risk re-evaluation:** C's risk is the smallest
that meets M2 needs without foreclosing post-M2 use. Risk
reinforces C.

---
