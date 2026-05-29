### DD-P6-006 — Header generation method

**Status:** Accepted

**Context:**
`wasamo.h` can be hand-written, generated from Rust by a tool
(`cbindgen`), or both. The chosen method affects how the spec
(`abi_spec.md`) and the header stay in sync.

**Options:**

Option A — Hand-written `wasamo.h`, CI-verified against Rust signatures (recommended)
- `wasamo.h` is hand-authored. It is the canonical artifact.
- A CI check builds a small C/C++ TU that `#include`s `wasamo.h`
  and links against the Rust-built `wasamo.lib`. Linker errors
  catch signature drift.
- A second CI check (optional, can land later) parses both
  `wasamo.h` and the `extern "C"` block of `lib.rs` and asserts
  function-name parity.

- What you gain: The spec, the header, and the docs evolve as one
  intentional artifact — important when the header is the M4 freeze
  surface. Comments in `wasamo.h` can be normative spec text, not
  generator output. The two-layer split (stable / experimental) is
  trivially expressed with `#ifdef WASAMO_EXPERIMENTAL` regions.
- What you give up: Drift is possible if CI checks are weak. Manual
  toil when adding/removing functions.

Option B — `cbindgen`-generated, header committed
`cbindgen` runs at build time and writes `wasamo.h`; the result is
checked into git so consumers don't need cbindgen.

- What you gain: Zero drift by construction. Single source of truth
  (the Rust signatures).
- What you give up: cbindgen output is mechanical — comments,
  ordering, and section structure are constrained by the tool.
  Annotating M1-experimental regions requires per-function
  attributes or post-processing. The header reads as machine
  output, not specification. The header ceases to be the artifact
  reviewers reason about; the Rust source becomes that, which is
  a less stable commitment for a soon-to-be-frozen ABI.

Option C — `cbindgen` for stable core, hand-written for experimental
Hybrid: stable core comes from cbindgen, experimental layer is
hand-written and `#include`-d.

- What you gain: Mechanical correctness on the long-term-stable
  surface, expressive freedom on the M1 throwaway surface.
- What you give up: Two source-of-truth systems for one header.
  Toolchain complexity (Rust build → cbindgen → concatenate →
  emit). The motivation evaporates if Option A's CI checks already
  catch drift.

**Recommendation:** **Option A.** A frozen-in-M4 ABI deserves a
hand-written, prose-rich header that doubles as readable
specification. `cbindgen` optimizes for a different problem
(generating bindings for fast-moving Rust libraries). Drift is a
solvable CI problem; loss of authorial control over the freeze
artifact is not. Revisit if drift is observed in practice during
Phase 7-8.

---
