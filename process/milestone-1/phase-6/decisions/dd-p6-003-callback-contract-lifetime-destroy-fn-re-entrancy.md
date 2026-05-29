### DD-P6-003 — Callback contract (lifetime, destroy_fn, re-entrancy)

**Status:** Accepted

**Context:**
Every callback registered through the ABI (signal connect, property
observe) needs a defined contract on three axes: who frees
`user_data`, when it's safe to disconnect, and what re-entrancy
guarantees the runtime gives. Get this wrong once and every binding
language inherits a footgun.

**Options:**

Option A — `(callback, user_data, destroy_fn)` triple + token-based disconnect (recommended)
- Connection signature:
  `int32_t wasamo_signal_connect(widget, name, fn, user_data, destroy_fn, &out_token)`
  where `destroy_fn: void (*)(void*)` is invoked exactly once when
  the connection is severed (explicit disconnect, widget destruction,
  or runtime shutdown).
- Disconnect by **opaque token** (`uint64_t`), not by `(widget, fn)`
  pair — tokens are stable, `(widget, fn)` is not (same `fn` may be
  registered twice with different `user_data`).
- **Re-entrancy:** the runtime guarantees a callback is **never**
  invoked while the host is inside a wasamo call on the same thread.
  Signal emissions during a `wasamo_set_property` are queued and
  drained when the call returns. (This is the SwiftUI / GTK model.)
- **Disconnect during emission:** disconnecting from inside a
  callback is allowed; the disconnect takes effect after the current
  emission completes. `destroy_fn` runs after that.

- What you gain: `destroy_fn` lets bindings own arbitrary state
  (Rust `Box`, Swift retained reference) without leaks. Token-based
  disconnect is unambiguous. The "no callbacks during a wasamo call"
  rule eliminates an entire class of binding bugs.
- What you give up: Slightly more API surface than the bare-pointer
  form. Hosts that don't need cleanup pass `NULL` for `destroy_fn`.

Option B — `(callback, user_data)` pair, no destroy_fn, host owns lifetime
Host code is responsible for keeping `user_data` alive until
disconnect.

- What you gain: Smallest signature.
- What you give up: Every binding must build its own
  destroy-on-disconnect machinery. Rust bindings in particular
  cannot safely register a `Box<dyn FnMut>` without leaking it,
  because there is no hook to run `Drop`. This is a well-known
  GLib-era footgun and the reason GObject added `GClosureNotify`.

Option C — Synchronous re-entrant emission (no queueing)
Like A on lifetime, but signal emissions during `wasamo_set_property`
fire **immediately**, before the set returns.

- What you gain: Simpler runtime (no queue). Lower latency.
- What you give up: Host code can observe a widget mid-mutation
  (`wasamo_set_property("text", "new")` fires the property-changed
  observer, which calls back into `wasamo_get_property`, which sees
  what state?). Synchronizing this across multi-property updates
  is nasty. Rules out batched `set` operations later.

**Recommendation:** **Option A.** The
`(callback, user_data, destroy_fn)` + token + queued-emission shape
is the lowest-footgun configuration and matches modern frameworks'
expectations. The cost is one extra parameter per connect call,
which is cheap insurance against the entire category of binding
lifetime bugs.

---
