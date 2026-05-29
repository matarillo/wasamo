### DD-P7-001 — Rust binding architecture

**Status:** Accepted

**Context:**
Phase 8's Rust "Hello Counter" needs *some* Rust API to drive the
runtime. There are three natural shapes, and the choice determines
whether M1's "C ABI verified in three languages" claim is real or
hollow.

**Options:**

Option A — Rlib path only (no FFI; what Phase 4/5 examples already do)
The Rust example links the `wasamo` rlib statically and calls Rust
types directly. No `wasamo-sys`, no safe wrapper.

- What you gain: Zero new crates. Phase 4/5 examples already
  demonstrate this works. Smallest delta from current state.
- What you give up: M1's acceptance criterion is "C ABI verified in
  three languages." Rust never crosses the C ABI in this option, so
  the C-Rust-Zig triplet collapses to "C and Zig exercise the C ABI;
  Rust separately exercises a Rust-native rlib." That is a weaker
  validation than the milestone claims. Also leaves the rlib's
  unmarked stability story unresolved.

Option B — `wasamo-sys` (raw FFI) + safe wrapper (recommended)
Two new crates:
- `wasamo-sys` — raw `extern "C"` declarations matching `wasamo.h`,
  links dynamically to `wasamo.dll` via `wasamo.dll.lib`.
- A safe wrapper crate (name TBD — see DD-P7-002) — translates the
  C ABI into idiomatic Rust (`Result<_, WasamoError>`, RAII handles,
  closure-capable callbacks).

Phase 8's Rust counter consumes the safe wrapper, which consumes
`wasamo-sys`, which calls into `wasamo.dll`. The rlib Rust-native
API is **not** removed but is repositioned (see DD-P7-002).

- What you gain: Rust genuinely traverses the C ABI on the same
  path C and Zig do. M1's three-language claim is real. The safe
  wrapper is also the artifact that proves the C ABI is *usable*
  from a memory-safe language — a non-trivial check on DD-P6-003
  (callback contract) and DD-P6-007 (memory ownership).
- What you give up: Two new crates, plus the crate-name collision
  problem (DD-P7-002). More moving parts in CI.

Option C — Hybrid: keep the rlib path **and** add sys+safe
Both A and B coexist. Phase 4/5 visual-check examples continue to
use the rlib for development convenience; Phase 8 Hello Counter
goes through sys+safe.

- What you gain: No regression of existing examples. Validates the
  C ABI without disturbing the dev-loop ergonomics that the rlib
  path gives internal contributors.
- What you give up: Two parallel Rust APIs to the same runtime,
  with overlapping type names and divergent stability stories.
  The rlib's "is this experimental or not?" question doesn't go
  away, it gets harder.

**Recommendation:** **Option B.** M1's acceptance criterion only
holds water if Rust crosses the C ABI like the other two languages.
Option A produces a hollow check; Option C carries Option B's costs
without retiring Option A's confusion. The price of B is bounded
(two crates, one CI step, one naming decision) and the wins
are durable: every sys/safe binding pair we ship later (Swift, Go,
.NET) reuses the contract this Rust pair will pin down.

**Implication for Phase 4/5 visual-check examples:** they continue
to compile against the rlib. They are dev-internal and not part of
the M1 acceptance surface. DD-P7-002 documents this explicitly.

---
