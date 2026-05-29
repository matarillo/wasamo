### DD-P6-004 — Threading and re-entrancy

**Status:** Accepted

**Context:**
The runtime is built on Win32 + Visual Layer, both of which have
strong thread-affinity requirements. The ABI must state plainly
which functions are callable from which threads, what happens on
violation, and whether any cross-thread post mechanism exists.

**Options:**

Option A — Strict UI-thread affinity, no cross-thread post in M1 (recommended)
- All `wasamo_*` functions other than (TBD: a future
  `wasamo_post_to_ui_thread`) must be called from the thread that
  called `wasamo_init`. Calling from another thread is **undefined
  behavior** at the ABI level; the runtime may assert in debug
  builds and is not required to detect it in release.
- Callbacks (signals, observers) are always invoked on the UI thread.
- **Cross-thread posting is deferred** — host code that needs to
  update UI from a worker thread must use OS primitives
  (`PostMessage` to the wasamo HWND, then call `wasamo_*` from the
  message handler) until a future ADR adds `wasamo_post`.

- What you gain: Simplest possible threading contract. Matches
  Win32 / Visual Layer reality. No locking needed in the runtime.
  Survives deferred (a)/(b) trivially — neither host-side nor
  runtime-side handler execution changes the affinity story.
- What you give up: Worker-thread → UI updates require host
  boilerplate in M1. Acceptable given the M1 demos (Hello Counter)
  are single-threaded.

Option B — Strict affinity **plus** a built-in `wasamo_post`
Same as A, plus `wasamo_post(callback, user_data, destroy_fn)`
schedules a closure on the UI thread.

- What you gain: One built-in primitive removes the boilerplate.
- What you give up: Adds one function to the stable core that is
  not strictly required for Hello Counter. Smells right but earns
  its place better when a real worker-thread sample exists.
  Adding it later is purely additive.

Option C — Free-threaded with internal synchronization
The runtime serializes calls from any thread onto the UI thread
internally.

- What you gain: Hosts call from anywhere.
- What you give up: Every call pays a synchronization cost; the
  runtime grows a queue and a dispatcher; deadlocks become possible
  if a UI-thread callback waits on a worker-thread call. Big
  invariant for a minimal ABI.

**Recommendation:** **Option A.** State strict UI-thread affinity
in the spec. Defer `wasamo_post` to the phase that actually needs
it; adding it is non-breaking. Document the
`PostMessage`-to-wasamo-HWND escape hatch in `abi_spec.md` so hosts
have a path forward in M1.

---
