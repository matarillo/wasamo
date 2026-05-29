### DD-P2-001 — `DispatcherQueueController` thread model

**Status:** Accepted

**Context:**
Visual Layer (`Windows.UI.Composition`) requires a `DispatcherQueue` to be
associated with the calling thread before any Compositor API is used.
`CreateDispatcherQueueController` offers two thread placement modes.
The choice determines how the Win32 message loop and Composition dispatch
coexist.

**Options:**

Option A — `DQTYPE_THREAD_CURRENT` (main-thread STA)
- What you gain: Win32 message loop and Visual Layer dispatch run on the same
  thread. No cross-thread calls, no synchronization primitives, no wakeup
  signalling. The entire runtime is single-threaded and trivially debuggable.
- What you give up: Heavy Composition work (e.g. surface uploads) can block
  the message loop and cause input lag. Not a concern at M1 scale.

Option B — `DQTYPE_THREAD_DEDICATED` (dedicated Composition thread)
- What you gain: Visual Layer operations run on a separate thread, so the
  Win32 message loop stays responsive even under rendering load. Gives a
  head start toward an architecture where animation and input are decoupled.
- What you give up: Every interaction between the runtime and Composition
  must be marshalled across threads. Substantial additional complexity for
  no perceptible benefit at M1's widget count.

**Decision:** Option A — single-thread STA is the standard pattern for Win32
desktop apps using `Windows.UI.Composition` and matches M1's complexity budget.

---
