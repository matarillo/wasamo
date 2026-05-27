### DD-P3-005 — Error handling strategy

**Status:** Accepted

**Context:**
The layout engine can encounter two categories of failure:

1. **API errors**: the host calls an API incorrectly (null handle, invalid
   parent/child relationship, Rectangle with no explicit dimension).
2. **Layout errors**: a size constraint is degenerate (e.g., `Fill` child
   inside a zero-size `Shrink` parent), producing a zero or negative extent.

**Options:**

Option A — All errors are fatal (return error code; abort layout on any failure)
- What you gain: Deterministic — no silent fallbacks.
- What you give up: A single bad widget crashes the entire tree's layout.
  Unacceptable for a UI runtime.

Option B — Split strategy: API errors strict; layout errors resilient
- API errors: return an error code immediately. The host is responsible.
- Layout errors (degenerate constraints, zero-size fill): clamp to 0.0,
  no error returned. The affected subtree renders at zero size; the rest
  of the tree is unaffected.
- What you gain: Matches how WPF, UWP, and SwiftUI handle bad constraints —
  graceful degradation rather than a process crash.
- What you give up: Degenerate layouts are silent in M1 (no runtime warning).

**Decision:** Option B — split strategy.
API error codes reuse the `int` return convention from Phase 2.
Degenerate layout dimensions clamp to 0.0 without surfacing an error.
