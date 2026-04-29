# Phase 6 — C ABI Header: Architecture Decisions

**Phase:** 6 (C ABI header — `wasamo.h` + `docs/abi_spec.md`)
**Date:** 2026-04-30
**Status:** Agreed and implemented (2026-04-30)

## Context

Phase 6's acceptance criterion (from
[VISION §7 M1](../../VISION.md#7-roadmap--milestones)) is:
**"Minimal C ABI header"** sufficient to validate the core hypothesis
(external DSL × C ABI × Visual Layer) by running "Hello Counter" in
C, Rust, and Zig at Phase 8. The header is *minimal*, not *frozen*:
M4 is when ABI stability commitments begin.

Two pre-pre-doc framing decisions (agreed 2026-04-29, recorded in
[../../ROADMAP.md §Phase 6](../../ROADMAP.md)) precede this ADR:

1. **Two-layer `abi_spec.md`.** The spec is partitioned into a
   **stable core** (M4 freeze candidate) and an **M1 experimental**
   layer. The experimental layer exists only because M1 `wasamoc` is
   parser-only — host code must imperatively construct widget trees
   until M2 codegen lands. Marking it `WASAMO_EXPERIMENTAL` keeps
   M1 stopgap shapes from leaking into long-term ABI commitments.

2. **Two deferred questions.** Phase 6 explicitly does **not** decide:
   - **(a)** Where DSL inline handler bodies (`clicked => { … }`)
     execute — host-side trampoline vs runtime-side interpreter.
   - **(b)** `wasamoc`'s M2 output format — host-language codegen vs
     IR + runtime interpretation.
   The stable core is sized to survive either resolution of (a) and
   (b). Decisions in this ADR that would presuppose either are
   deliberately scoped down or pushed into the experimental layer.

The current C ABI surface
([wasamo/src/lib.rs:62-114](../../wasamo/src/lib.rs#L62-L114))
is five functions: `wasamo_init`, `wasamo_window_create`,
`wasamo_window_show`, `wasamo_window_destroy`, `wasamo_run`. They
are the seed for the stable core but predate this ADR's framing —
error convention, threading contract, and string-encoding rules
need to be specified, not just inherited.

The six decisions below correspond to the Phase 6 ROADMAP checklist
([../../ROADMAP.md L122-L125](../../ROADMAP.md#L122-L125)):
stable-core scope / signal model / callback contract /
threading and re-entrancy / error convention / header generation
method.

---

### DD-P6-001 — Stable-core scope at function granularity

**Status:** Agreed

**Context:**
The stable core must be the smallest set of functions that lets a
host language drive a wasamo runtime end-to-end **without** depending
on the M1 experimental imperative builder. It is sized assuming M2
codegen (or IR) will produce the widget tree, so the core does not
need any tree-construction primitive — only the surfaces a generated
binding would call at runtime: lifecycle, window + event loop,
property get/set, property-change observers, and signal registration
for component-declared signals.

**Options:**

Option A — Five-area minimum (recommended)
The core covers exactly the five areas listed in
[ROADMAP Phase 6](../../ROADMAP.md#L128-L131):
1. **Runtime lifecycle:** `wasamo_init`, `wasamo_shutdown`.
2. **Window + event loop:** `wasamo_window_create`, `wasamo_window_show`, `wasamo_window_destroy`, `wasamo_run`, `wasamo_quit`.
3. **Property get/set:** `wasamo_get_property`, `wasamo_set_property` keyed by `(widget, property_id)`.
4. **Property-change observer:** `wasamo_observe_property` / `wasamo_unobserve_property`.
5. **Component-declared signal register:** `wasamo_signal_connect` / `wasamo_signal_disconnect`.

- What you gain: One-to-one match with the agreed framing. Each area
  is independently justifiable as a survivor of both deferred
  questions: lifecycle, event loop, property R/W, observers, and
  signals are needed regardless of whether handlers run host-side or
  runtime-side, and regardless of codegen vs IR.
- What you give up: Some functions a "real" ABI wants — text
  measurement, image loading, HWND escape hatch — are outside the
  core. They will surface during Phase 7-8 work and either get added
  to the experimental layer or trigger a follow-up ADR.

Option B — Five-area minimum **plus** an HWND escape hatch
Same as A, plus `wasamo_window_get_hwnd` for host code that needs to
interop with native Win32 (custom drag regions, system menu, etc.).

- What you gain: Reduces the chance of host code being forced into
  the experimental layer for genuinely missing primitives.
- What you give up: HWND in the stable core leaks an implementation
  detail that may not survive future windowing changes (e.g. if
  wasamo ever supports `AppWindow`-only modes). Cleaner to add later
  with full deliberation than to retract.

Option C — Five-area minimum **plus** a focus / IME hook
Same as A, plus `wasamo_window_set_focus` and a minimal IME query.

- What you gain: Phase 8 "Hello Counter" likely wants keyboard focus
  navigation. Including focus in the core avoids a same-phase
  follow-on.
- What you give up: M1 has no widget set that needs IME yet (Phase 4
  delivered Button + Text only). Premature.

**Recommendation:** **Option A.** It matches the agreed framing
exactly, keeps the M4 freeze surface auditable, and defers escape
hatches (HWND, focus, IME) until concrete phase work demands them.
HWND access during M1 stays in the experimental layer if needed.

---

### DD-P6-002 — Signal model

**Status:** Agreed

**Context:**
A "signal" in wasamo is a named, typed event a component can declare
and emit (e.g. `Button.clicked`). The C ABI must let a host
language register a callback to receive emissions. The model must
work for both component-declared signals (DSL `signal foo(i32)`)
and built-in widget signals (`Button.clicked`), and must survive
deferred question (a): whether DSL inline handlers run host-side or
runtime-side. (Inline handlers are a separate path — they are bodies
of code, not host callbacks. Signals are the host-callback path.)

**Options:**

Option A — String-keyed, untyped payload (recommended)
`wasamo_signal_connect(widget, "clicked", callback, user_data, &out_token)`
where `callback` has signature
`void (*)(WasamoWidget*, const WasamoValue* args, size_t arg_count, void* user_data)`.
`WasamoValue` is a tagged union over the M1 property-type set
(i32, f64, bool, string-view, widget-handle).

- What you gain: One signature handles every signal regardless of
  arity or types. Codegen (M2) can produce typed wrappers on top
  without ABI changes. Untyped payload + string key is the
  GTK / GLib idiom — well-understood by C-ABI binding authors.
- What you give up: Runtime type-check cost (small) and the ergonomic
  loss of compile-time mismatch detection at the C boundary. Both
  are recovered by generated bindings in Rust/Zig/Swift.

Option B — Per-signal typed C function pointers
`wasamo_button_set_clicked(button, void (*)(WasamoButton*, void*), user_data)`,
one per widget×signal pair.

- What you gain: Compile-time type safety at the C boundary. Cheaper
  per-emission (no value packing).
- What you give up: Does **not** scale to component-declared signals
  — a `signal foo(i32, string)` defined in `.ui` cannot have a
  hand-written `wasamo_*_set_foo` because the runtime has no static
  knowledge of it. Forces all DSL-declared signals into a separate
  mechanism, fragmenting the model. This is exactly the M1
  experimental shape (`wasamo_button_set_clicked` already exists)
  that the framing wants to keep **out** of the stable core.

Option C — Single global dispatch
`wasamo_set_signal_dispatcher(callback)` — one host-side router for
all signals from all widgets, with `(widget, signal_name, args)` in
the payload.

- What you gain: Smallest ABI surface (one register call).
- What you give up: Per-widget connection lifetime is awkward —
  disconnecting one connection means the dispatcher must keep its
  own table. Hostile to bindings that want per-callback ownership
  (Rust closures, Swift `@escaping`). Doesn't compose with multiple
  bindings in the same process.

**Recommendation:** **Option A.** The string-key + tagged-value
model is the only one that uniformly handles built-in and
component-declared signals, survives deferred (a)/(b), and is the
established C-ABI idiom for this shape of problem. Option B's
`wasamo_button_set_clicked` form is preserved in the **M1
experimental** layer for Phase 8 simplicity but is explicitly not
part of the stable core. Option C is rejected for ownership
reasons.

---

### DD-P6-003 — Callback contract (lifetime, destroy_fn, re-entrancy)

**Status:** Agreed

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

### DD-P6-004 — Threading and re-entrancy

**Status:** Agreed

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

### DD-P6-005 — Error convention

**Status:** Agreed

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

### DD-P6-006 — Header generation method

**Status:** Agreed

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

### DD-P6-007 — DLL boundary contract (export, calling convention, memory ownership)

**Status:** Agreed

**Context:**
`wasamo.dll` is consumed across a dynamic link boundary by hosts
that may use a different C runtime (CRT), a different language
toolchain, or both. Three sub-questions must be answered up-front
because they propagate through every signature in `wasamo.h` and
become hard to revise after M4 freeze:

- **(a) Symbol export.** How does `wasamo.h` switch between
  `__declspec(dllexport)` (when the runtime is being built) and
  `__declspec(dllimport)` (when a host includes it)?
- **(b) Calling convention.** Which calling convention do public
  functions and host-provided callbacks use?
- **(c) Memory ownership across the boundary.** Mixing
  `malloc`/`free` across CRTs corrupts the heap. What is the rule
  for any pointer that crosses the boundary?

**(a) and (b) are not really option-shaped** — there is one Windows
idiom for each, and we just need to commit to it. **(c) is the
real decision.**

**Symbol export (sub-decision, no options):**

```c
#if defined(WASAMO_BUILDING_DLL)
#  define WASAMO_EXPORT __declspec(dllexport)
#else
#  define WASAMO_EXPORT __declspec(dllimport)
#endif
```

The wasamo build sets `WASAMO_BUILDING_DLL`; hosts do not. Static
linking is **not** a supported configuration in M1; if a future
phase wants it, a `WASAMO_STATIC` branch can be added without
breaking either side.

**Calling convention (sub-decision, no options):**

```c
#define WASAMO_API __cdecl
```

All public functions are declared `WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_foo(...)`.
All host-provided callback function-pointer typedefs in `wasamo.h`
also carry `WASAMO_API`. On x64 Windows this is the only calling
convention and the macro is a no-op, but stating it explicitly
makes the header correct for x86 and future ARM64EC targets without
revision.

**Memory ownership across the boundary (the decision):**

Three coherent options for how any pointer that crosses the DLL
boundary is owned and freed:

Option A — Runtime owns all runtime-allocated memory; bounded lifetime (recommended)
- Any `const char*` / pointer the runtime returns is **owned by the
  runtime**. The host must not call `free` on it.
- Each such pointer has a documented lifetime tied to a runtime
  state (e.g. `wasamo_last_error_message` is valid until the next
  ABI call on the same thread; a property-string returned by
  `wasamo_get_property` is valid until the next `wasamo_set_property`
  on that widget). Hosts that need to retain the value copy it.
- Strings the **host** passes in (e.g. `wasamo_set_property` string
  values) are passed as `(const char* ptr, size_t len)` UTF-8
  **borrowed** for the duration of the call only. The runtime
  copies internally if it needs to retain them.
- `user_data` pointers attached via callbacks are owned by the host;
  `destroy_fn` (DD-P6-003) is the runtime's hook to release them.
- No `wasamo_*_free` functions exist in the stable core.

- What you gain: Zero allocator crossings — the host's CRT and the
  runtime's CRT never touch each other's heaps. Every signature
  reads as "borrow in, borrow out" with explicit lifetimes. Removes
  a whole category of binding bugs.
- What you give up: Hosts that want to retain runtime-returned data
  must copy. Trivial cost; the cost is paid where it is most
  obvious (at the call site that wants persistence), not hidden in
  a future debugging session.

Option B — Paired allocator/deallocator per type (`wasamo_string_free`, etc.)
The runtime allocates, hosts free via paired `wasamo_X_free`
functions that route through the runtime's CRT.

- What you gain: Returned values have unbounded lifetime — hosts
  can hold them as long as they like.
- What you give up: Adds one `_free` function per allocated type to
  the stable core. Hosts must remember which pointers came from
  wasamo (free with `wasamo_X_free`) vs. their own code (`free`).
  Forgetting in either direction silently corrupts the heap. The
  unbounded-lifetime convenience is rarely needed in practice
  (last-error is read-once; property values are usually compared
  or copied immediately).

Option C — Caller-provided buffers (`wasamo_get_X(buf, buf_len, &out_len)`)
Hosts allocate the buffer; the runtime fills it. If the buffer is
too small the runtime returns the required length.

- What you gain: No allocations cross the boundary at all. Same
  convention C programmers expect from Win32 (`GetWindowTextW`).
- What you give up: Two-call idiom (size query, then real call) for
  every variable-length getter. Higher-level bindings have to
  paper over this every time. Inconvenient for `wasamo_get_property`
  which is called frequently.

**Recommendation:** **Option A** for memory ownership, combined
with the (forced) choices on export macro and calling convention
above. Option A keeps the boundary clean by construction — no
allocator ever crosses it — and matches the "minimal stable core"
spirit by adding zero `_free` functions. Bounded-lifetime contracts
are documented per-function in `abi_spec.md`. Option B and Option C
remain available as targeted exceptions in future phases if a
specific API genuinely needs unbounded ownership or zero-allocation
queries.

This decision interacts with prior DDs:
- **DD-P6-002 (signal model):** `WasamoValue` string payloads are
  borrowed for the duration of the callback only — host copies if
  retention is needed.
- **DD-P6-003 (callback contract):** `destroy_fn` is the
  host-owned-memory release hook; the runtime never frees host
  `user_data`.
- **DD-P6-005 (error convention):** `wasamo_last_error_message`
  returns a runtime-owned, thread-local pointer valid until the
  next ABI call on that thread.

---

## Summary of recommended decisions

| ID | Topic | Recommendation |
|---|---|---|
| DD-P6-001 | Stable-core scope | Option A — five-area minimum (lifecycle / window+loop / property R/W / observer / signal) |
| DD-P6-002 | Signal model | Option A — string-keyed, tagged-value payload |
| DD-P6-003 | Callback contract | Option A — `(fn, user_data, destroy_fn)` + token + queued emission |
| DD-P6-004 | Threading | Option A — strict UI-thread affinity, defer `wasamo_post` |
| DD-P6-005 | Error convention | Option A — `WasamoStatus` enum + out-params + thread-local last-error message |
| DD-P6-006 | Header generation | Option A — hand-written `wasamo.h`, CI-verified |
| DD-P6-007 | DLL boundary | `WASAMO_EXPORT` via `WASAMO_BUILDING_DLL`; `WASAMO_API = __cdecl`; Option A for memory (runtime owns, bounded lifetime) |

Once agreed, this ADR's status moves to **Agreed** and the next
artifact (`docs/abi_spec.md` initial draft) is written against
these decisions.
