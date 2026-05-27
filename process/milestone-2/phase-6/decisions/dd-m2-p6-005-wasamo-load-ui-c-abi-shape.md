### DD-M2-P6-005 — `wasamo_load_ui` C ABI shape

**Status:** Accepted

**Context:**
The host calls into the runtime to turn a `.ui` (or its
compiled IR) into a running widget tree. The ABI shape is
exposed to all binding languages and is the surface that
hosts write against; small choices here propagate to every
host-language wrapper.

DD-M2-P6-004 = B settles Signal ownership in `.ui`, removing
element-identity from this DD's scope. DD-M2-P6-001 = D
introduces `WASAMO_ERR_OBSERVER_MUTATION`; this DD chooses how
load-time errors and other runtime-loaded errors surface.

Resource-resolution sub-issue: whether the loader takes a
filesystem path, a memory blob, or a build-time-embedded
string. The framing note narrows this to three live
candidates (A/B/C below). Search-path / bundle features are
out of scope for M2 per the framing note.

**Options:**

Option α — Single-function load (recommended)
- One function:
  ```
  WasamoStatus wasamo_load_ui(
      const char* resource,         // path or in-memory pointer per DD-005-r
      WasamoWindowHandle* out_root  // root widget handle on success
  );
  ```
- Returns `WasamoStatus` (existing error-code enum extended
  with the load-related codes). Subsequent host calls use
  `*out_root` and any Signal handles bound to its IR (per
  DD-M2-P6-004 = B, Signal handles are referenced by name
  through a separate accessor — design lives in DD-M2-P6-007).
- What you gain: smallest ABI surface; one round-trip.
- What you give up: a future split into "compile" + "instantiate"
  (e.g. for hot reload pre-loading) requires a new function;
  acceptable because the M2 contract does not require that
  split.

Option β — Split loader / instantiate
- `wasamo_compile_ui(resource) → WasamoIRHandle` and
  `wasamo_instantiate(ir_handle) → WasamoWindowHandle`.
- What you gain: the IR handle can be reused across instances
  (relevant for M3 list rendering and post-1.0 hot reload).
- What you give up: two-phase ABI for an M2 case that doesn't
  exercise reuse (counter loads once); doubles the error-path
  surface; encourages premature lifetime concerns in host
  code.

**Recommendation:** **Option α** for M2.

Multi-instantiation and IR reuse are post-M2 needs; the
single-function form is the smallest shape that satisfies A1.
Split-on-demand is additive: introducing
`wasamo_compile_ui` + `wasamo_instantiate` later does not
break α-shape callers.

**Resource-resolution form (sub-decision):**

- (A) **Absolute path only** — host computes the absolute path
  and passes it. Simplest contract.
- (B) **Path relative to host executable** — runtime resolves
  using the executable directory. Adds platform-specific
  resolution code; useful when the host distributes a `.ui`
  alongside the binary.
- (C) **Compile-time embedded string** — host embeds the
  `.ui` content at compile time and passes a memory blob.
  No filesystem access at runtime.

**Recommended sub-decision: support (A) and (C); defer (B).**
The ABI accepts a path or a `(pointer, length)` blob distinguished
by a small flag. (A) is the lowest-friction shape for a
counter example; (C) is increasingly the right shape for
production deployments and for binding languages where build
systems can embed at compile time. (B) is a small convenience
that adds platform code (Windows: `GetModuleFileNameW` +
path manipulation) for limited M2 benefit; deferral does not
foreclose it.

**Error reporting (sub-decision).** Three live candidates:

- (i) Last-error-string API:
  `const char* wasamo_last_error_message(void);`
- (ii) Continue DD-M2-P3-003's stderr-only convention.
- (iii) Logger callback registration:
  `wasamo_set_error_callback(fn(const char*))`.

**Recommended: (i) for M2; document (iii) as the planned M3 path.**
A last-error string is universally writeable from every binding
language. Stderr-only (ii) is hostile to GUI deployments where
stderr is not visible. (iii) is the right long-term shape but
requires the host to set the callback before the first call;
M2 hosts (counter examples) are simple enough that the
last-error pattern suffices, and (i) does not block (iii) being
added later as a precedence-overriding mechanism.

`WASAMO_ERR_OBSERVER_MUTATION` (DD-M2-P6-001) is consolidated
in this error mechanism: the error code is returned from the
violating ABI call, and the message string identifies the
observer callback in flight (file/line where available). The
other M2-introduced error codes
(`WASAMO_ERR_REACTIVE_DIVERGED`, `WASAMO_ERR_REENTRANT_LOAD`,
`WASAMO_ERR_IR_MALFORMED`, `WASAMO_ERR_WRONG_THREAD`) use the
same channel.

**Lifetime and threading model (sub-decision).**

The single-function load shape leaves four contract points
unspecified that every binding language must agree on. Each
is fixed here:

- **Handle ownership.** `WasamoWindowHandle` is owned by the
  runtime. The host receives an opaque pointer; passing it to
  any `wasamo_*` ABI is the only legal use. The runtime is the
  sole party that mutates or frees the underlying window
  structure.
- **Handle lifetime.** A handle is valid from successful return
  of `wasamo_load_ui` until runtime shutdown. M2 does not
  expose a per-window destroy ABI; the M2 counter's window
  lifetime spans the process. M3 multi-instance scenarios will
  introduce `wasamo_destroy_window` (or equivalent); the M2
  contract is the constant-lifetime base case of that future
  shape, so M2-era hosts do not require revision when explicit
  destruction lands.
- **Last-error message lifetime.** The string returned by
  `wasamo_last_error_message` is owned by the runtime, valid
  until the next `wasamo_*` ABI call from the same thread. The
  storage is thread-local; concurrent calls from different
  threads do not overwrite each other's last error (modulo the
  thread-affinity rule below, which makes "different threads"
  itself a contract violation in M2). The host must copy the
  string if it needs to retain it across ABI calls.
- **Thread affinity (UI-thread confinement).** All `wasamo_*`
  ABI calls must originate from a single thread per runtime
  instance — the thread that called `wasamo_load_ui`. Calls
  from any other thread return `WASAMO_ERR_WRONG_THREAD`
  without performing the requested action and without
  modifying runtime state. This matches the discipline of
  every major retained-mode UI framework (Win32 message
  thread, AppKit main thread, GTK main thread, Slint event
  loop) and is the only model under which the lock-free
  queue / TLS-flag machinery in DD-M2-P6-001 is sound (the
  TLS used by DD-P6-001's IN_OBSERVER_CALLBACK and DD-P6-003's
  IN_DRAIN flags is the same TLS the thread-affinity check
  relies on).

Cross-thread "post a callable to the UI thread" patterns are
the host's responsibility for M2; if a binding-author audience
need surfaces in M3, a `wasamo_post_to_ui_thread` helper can
be added additively. The M2 contract does not foreclose it.

**Forward-compat exposure:**

- Out-of-scope items engaged: M3 list rendering (multiple
  instantiations); post-1.0 hot reload (re-load same
  resource); M3 logger callback (iii).
- α + (A)/(C) supports M3 list rendering by introducing
  `wasamo_instantiate` additively; supports hot reload by
  the same path. Logger callback (iii) layers over (i) without
  breaking it.
- β front-loads design for an M2-uncommitted use; (B) and (ii)
  carry costs without M2 benefit.

**Technical-risk re-evaluation:** α + (A) + (C) + (i) is the
smallest ABI satisfying A1; risk concentrates in (C)'s
embedding ergonomics, which is a binding-side concern, not a
runtime concern. Risk reinforces the recommendation.

---
