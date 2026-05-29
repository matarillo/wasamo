### DD-P6-007 — DLL boundary contract (export, calling convention, memory ownership)

**Status:** Accepted

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
