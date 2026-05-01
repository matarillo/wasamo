# Wasamo C ABI Specification

**Version:** M1 (2026-04-30)
**Status:** Agreed (2026-04-30) — finalised against the implemented `wasamo.h`
**Authoritative decisions:** [decisions/phase-6-c-abi.md](decisions/phase-6-c-abi.md) (DD-P6-001..007)

This document specifies the C ABI exposed by `wasamo.dll` via the
`wasamo.h` header. It is the normative reference for binding
authors and for the runtime implementation. The header is the
artifact reviewers reason about; this document is its prose
counterpart.

The ABI is **two-layer**:

- **Stable core** — candidate for the M4 ABI freeze. Functions
  and types in this layer are designed to survive both deferred
  Phase 6 questions: (a) where DSL inline handler bodies execute,
  and (b) `wasamoc`'s M2 output format. They are not yet frozen;
  M1-M3 may revise them. M4 commits to backward compatibility
  going forward.
- **M1 experimental** — exists because M1 `wasamoc` is parser-only
  and host code must construct widget trees imperatively. Every
  symbol in this layer is marked `WASAMO_EXPERIMENTAL` in the
  header and is **not** subject to M4 stability. Code that uses
  these symbols must expect breakage.

## 1. Conventions

UTF-8 is the only string encoding accepted or returned by the ABI.
Strings the host passes are `(const char* ptr, size_t len)`
**without** a NUL requirement. Strings the runtime returns are
NUL-terminated `const char*` with a documented bounded lifetime
(see §2.3).

Integer types use the fixed-width forms from `<stdint.h>`
(`int32_t`, `uint32_t`, `uint64_t`). The ABI never uses `int`,
`long`, or other implementation-defined widths.

All functions that can fail return `WasamoStatus` (§3.1).
Functions that produce a handle take an out-parameter:

```c
WasamoStatus wasamo_window_create(/* … */, WasamoWindow** out);
```

A non-`WASAMO_OK` return means `*out` is unchanged (typically
left as the host-initialised `NULL`).

## 2. DLL boundary

### 2.1 Symbol export

`wasamo.h` declares the following macros:

```c
#if defined(WASAMO_BUILDING_DLL)
#  define WASAMO_EXPORT __declspec(dllexport)
#else
#  define WASAMO_EXPORT __declspec(dllimport)
#endif
```

The wasamo build defines `WASAMO_BUILDING_DLL`. Hosts including
`wasamo.h` do **not** define it. Static linking is unsupported in
M1.

### 2.2 Calling convention

```c
#define WASAMO_API __cdecl
```

Every public function and every host-supplied callback typedef in
`wasamo.h` carries `WASAMO_API`. On x64 Windows this is the only
calling convention and the macro is a no-op; the explicit
declaration keeps the header correct for x86 and ARM64EC.

The full prefix for any public function is:

```c
WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_foo(/* … */);
```

### 2.3 Memory ownership

The ABI is structured so that allocations never cross the CRT
boundary. Three rules govern any pointer that crosses the ABI:

1. **Host-passed pointers are borrowed for the duration of the
   call only.** The runtime copies internally if it needs to
   retain them. This applies to all `(const char* ptr, size_t len)`
   string arguments and to `const WasamoValue*` value arguments.
   Concretely: a string passed as `title_utf8` / `content_utf8` /
   `label_utf8` / a `WasamoValue.v_string` payload does not need
   to remain valid after the `wasamo_*` call returns. The caller
   is free to free, overwrite, or reuse that buffer immediately.
2. **Runtime-returned pointers are owned by the runtime** and have
   a documented bounded lifetime tied to a specific runtime state.
   Hosts must not call `free` on them. Hosts that need to retain
   the data copy it before the lifetime expires.
3. **Host `user_data` pointers** registered with callbacks remain
   host-owned. The runtime calls the host-provided `WasamoDestroyFn`
   exactly once when the registration is severed (explicit
   disconnect, owning widget destroyed, or runtime shutdown).
   The runtime never calls `free` on `user_data`.

The stable core defines no `wasamo_*_free` functions; if a future
phase needs unbounded-lifetime returns, they will be added as
targeted exceptions (and will not affect existing signatures).

## 3. Types

### 3.1 `WasamoStatus`

```c
typedef int32_t WasamoStatus;

#define WASAMO_OK                  0
#define WASAMO_ERR_INVALID_ARG    -1
#define WASAMO_ERR_RUNTIME        -2
#define WASAMO_ERR_NOT_INITIALIZED -3
#define WASAMO_ERR_WRONG_THREAD   -4
```

The status space is closed at M4. New codes added before M4 are
non-breaking; codes are never reassigned. Negative values denote
errors; zero is success. Hosts should treat any unknown negative
code as a generic failure rather than asserting.

After any ABI call returning a non-OK status, `wasamo_last_error_message`
(§4.1) returns a thread-local human-readable description of that
specific failure. The description is for diagnostics only; the
numeric `WasamoStatus` is the contract.

### 3.2 Opaque handles

```c
typedef struct WasamoWindow WasamoWindow;
typedef struct WasamoWidget WasamoWidget;
```

The runtime never reveals the layout of these structs. Hosts pass
and store handles by pointer only.

### 3.3 `WasamoValue`

`WasamoValue` is a tagged union over the M1 property-and-signal
type set. It carries values in both directions across the ABI
(property R/W, signal payloads).

```c
typedef enum {
    WASAMO_VALUE_NONE   = 0,
    WASAMO_VALUE_I32    = 1,
    WASAMO_VALUE_F64    = 2,
    WASAMO_VALUE_BOOL   = 3,
    WASAMO_VALUE_STRING = 4,
    WASAMO_VALUE_WIDGET = 5,
} WasamoValueTag;

typedef struct {
    WasamoValueTag tag;
    union {
        int32_t        v_i32;
        double         v_f64;
        int32_t        v_bool;     /* 0 = false, non-zero = true */
        struct {
            const char* ptr;       /* UTF-8, not necessarily NUL-terminated */
            size_t      len;
        } v_string;
        WasamoWidget*  v_widget;
    } as;
} WasamoValue;
```

When the runtime fills a `WasamoValue*` (e.g. `wasamo_get_property`,
or signal-handler arguments), the lifetime of any contained string
or widget pointer follows §2.3 rule 2 — the value is valid until
the next ABI call on the same thread for property reads, or for
the duration of the callback for signal arguments. Hosts copy
when they need retention.

The tag set is closed at M4. New tags added before M4 are
non-breaking provided existing tags keep their numeric values.

### 3.4 Callback typedefs

```c
typedef void (WASAMO_API *WasamoDestroyFn)(void* user_data);

typedef void (WASAMO_API *WasamoSignalHandlerFn)(
    WasamoWidget*       sender,
    const WasamoValue*  args,        /* may be NULL when arg_count == 0 */
    size_t              arg_count,
    void*               user_data);

typedef void (WASAMO_API *WasamoPropertyObserverFn)(
    WasamoWidget*       widget,
    uint32_t            property_id,
    const WasamoValue*  new_value,
    void*               user_data);
```

All callback invocations occur on the UI thread (§6) and are
queued such that no callback fires while the host is inside a
`wasamo_*` call on the same thread.

## 4. Stable core API

### 4.1 Runtime lifecycle

```c
WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_init(void);
WASAMO_EXPORT void         WASAMO_API wasamo_shutdown(void);
WASAMO_EXPORT const char*  WASAMO_API wasamo_last_error_message(void);
```

`wasamo_init` must be called once from the thread that will own
the UI before any other `wasamo_*` function. That thread is the
**UI thread** for the lifetime of the runtime.

`wasamo_shutdown` releases all runtime state. After it returns,
all handles previously issued are invalid; calling any other
`wasamo_*` function (other than another `wasamo_init`) is
undefined behavior.

`wasamo_last_error_message` returns a thread-local NUL-terminated
UTF-8 string describing the most recent non-OK status produced on
the calling thread. The pointer is valid until the next ABI call
on that thread. If no error has been produced, the function may
return an empty string or `NULL`; hosts must tolerate both.

### 4.2 Window and event loop

```c
WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_window_create(
    const char*    title_utf8,
    size_t         title_len,
    int32_t        width,
    int32_t        height,
    WasamoWindow** out);

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_window_show(WasamoWindow*);
WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_window_destroy(WasamoWindow*);

WASAMO_EXPORT void WASAMO_API wasamo_run(void);
WASAMO_EXPORT void WASAMO_API wasamo_quit(void);
```

`wasamo_run` blocks until `WM_QUIT` is received and pumps the
Win32 message loop. `wasamo_quit` posts a quit message; it is
safe to call from a UI-thread callback. Calling `wasamo_quit` on
another thread is unsupported in M1 (use `PostMessage` to the
window's HWND instead — see §6).

`wasamo_window_destroy` is idempotent on a `NULL` argument and
invalidates the handle on success.

### 4.3 Property get/set

```c
WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_get_property(
    WasamoWidget*  widget,
    uint32_t       property_id,
    WasamoValue*   out_value);

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_set_property(
    WasamoWidget*       widget,
    uint32_t            property_id,
    const WasamoValue*  value);
```

Property IDs are `uint32_t` keys. The ID space is partitioned per
widget type. The mechanism is stable; the concrete ID values for
M1 widgets are defined in the M1 experimental layer (§5) and may
change before M4. The contract here:

- An ID unknown to the given widget returns `WASAMO_ERR_INVALID_ARG`.
- A type-mismatched value (e.g. setting a string property to an
  i32) returns `WASAMO_ERR_INVALID_ARG`.
- On a successful `wasamo_set_property`, any registered observers
  for this `(widget, property_id)` pair are scheduled to fire
  after the call returns (§6).
- **String lifetime for `wasamo_set_property`:** the
  `WasamoValue.v_string` payload passed to `wasamo_set_property`
  follows §2.3 rule 1 — it is borrowed for the duration of the
  call only. The runtime copies the UTF-8 bytes internally; the
  host may free or reuse the buffer as soon as the call returns.
  (The same rule applies to `wasamo_window_create`'s `title_utf8`
  and to all widget-constructor `content_utf8` / `label_utf8`
  arguments.)

### 4.4 Property-change observers

```c
WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_observe_property(
    WasamoWidget*             widget,
    uint32_t                  property_id,
    WasamoPropertyObserverFn  callback,
    void*                     user_data,
    WasamoDestroyFn           destroy_fn,    /* may be NULL */
    uint64_t*                 out_token);

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_unobserve_property(uint64_t token);
```

`out_token` receives a stable opaque identifier for the
registration. Observers are severed in three situations: explicit
`wasamo_unobserve_property`, destruction of the owning widget, or
`wasamo_shutdown`. In all three, `destroy_fn(user_data)` is
called exactly once (if non-NULL).

Disconnecting from inside the callback is permitted; the
disconnect takes effect after the current emission completes.

### 4.5 Component-declared signal register

```c
WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_signal_connect(
    WasamoWidget*          widget,
    const char*            signal_name_utf8,
    size_t                 name_len,
    WasamoSignalHandlerFn  callback,
    void*                  user_data,
    WasamoDestroyFn        destroy_fn,    /* may be NULL */
    uint64_t*              out_token);

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_signal_disconnect(uint64_t token);
```

Signal names are UTF-8. The same `(widget, signal_name)` pair may
have multiple connections; each gets a distinct token. Lifetime
and disconnect-during-emission semantics match property observers
(§4.4).

This is the path that handles **both** built-in widget signals
(e.g. `Button.clicked`) and DSL `signal foo(...)` declarations.
The M1 experimental `wasamo_button_set_clicked` (§5) is a
convenience for the former; it is not part of the stable core.

## 5. M1 experimental layer

Every symbol in this section is declared with `WASAMO_EXPERIMENTAL`:

```c
#define WASAMO_EXPERIMENTAL  /* documentation marker; binds to no behavior */
```

The marker is a documentation contract: code that includes any
`WASAMO_EXPERIMENTAL`-annotated symbol must expect breakage in any
M2+ release. The runtime build does not gate experimental symbols
behind a compile-time flag in M1; binding generators are expected
to honor the marker by tagging generated wrappers as experimental.

The set finalised for Phase 8 "Hello Counter":

```c
WASAMO_EXPERIMENTAL
WasamoStatus wasamo_text_create(
    const char* content_utf8, size_t content_len,
    WasamoWidget** out);

WASAMO_EXPERIMENTAL
WasamoStatus wasamo_button_create(
    const char* label_utf8, size_t label_len,
    WasamoWidget** out);

WASAMO_EXPERIMENTAL
WasamoStatus wasamo_vstack_create(
    WasamoWidget** children, size_t count,
    WasamoWidget** out);

WASAMO_EXPERIMENTAL
WasamoStatus wasamo_hstack_create(
    WasamoWidget** children, size_t count,
    WasamoWidget** out);

WASAMO_EXPERIMENTAL
WasamoStatus wasamo_window_set_root(
    WasamoWindow* window, WasamoWidget* root);

WASAMO_EXPERIMENTAL
WasamoStatus wasamo_button_set_clicked(
    WasamoWidget* button,
    WasamoSignalHandlerFn callback,
    void* user_data,
    WasamoDestroyFn destroy_fn,
    uint64_t* out_token);
```

Property-ID constants for M1 widgets — used with the stable §4.3
mechanism — are also experimental:

```c
#define WASAMO_BUTTON_LABEL  1u   /* String  */
#define WASAMO_BUTTON_STYLE  2u   /* I32     */
#define WASAMO_TEXT_CONTENT  3u   /* String  */
#define WASAMO_TEXT_STYLE    4u   /* I32     */
```

`BUTTON_STYLE` values: `0 = Default`, `1 = Accent`. `TEXT_STYLE`
values: `0 = Caption`, `1 = Body`, `2 = Subtitle`, `3 = Title`. The
numeric assignments are M1 stopgaps and may change before M4.

**Construction defaults.** Container constructors do not take
spacing / padding / alignment arguments in M1; the runtime applies
fixed defaults (`spacing = 8.0`, `padding = 8.0`, alignment
`Center`). `wasamo_button_create` defaults to `BUTTON_STYLE = Default`;
`wasamo_text_create` defaults to `TEXT_STYLE = Body`. Hosts override
post-construction via `wasamo_set_property` (§4.3).

**Ownership semantics for container constructors.** Children
pointers passed in the `children` array are MOVED into the new
container. On `WASAMO_OK` return, the host's child pointers are
stale and must not be reused. On any non-OK return, no children
are consumed (constructors validate the array before taking
ownership of any element).

**Ownership transfer to a window.** `wasamo_window_set_root` moves
the root widget into the window. After that call the widget tree
is owned by the window: it is dropped when the window is destroyed
or when `wasamo_shutdown` is called, whichever comes first. There
is no separate widget-destroy ABI in M1 — destruction is always
keyed off the owning window.

**Auto-routing on installed roots.** When a window has a root
installed via `wasamo_window_set_root`, the runtime forwards
`WM_SIZE` to a re-layout pass, `WM_MOUSEMOVE` /
`WM_LBUTTON{DOWN,UP}` to per-widget hover and click hit-testing,
and emits `"clicked"` signals through the registry on Button hits.
Hosts do not need to wire window-message callbacks for these.

These symbols are removed (or migrated into the stable core) when
`wasamoc` codegen lands in M2.

### 5.1 What M1 experimental verifies, and what it does not

The shape above is deliberately the smallest experimental layer
that lets Phase 8 "Hello Counter" run, while keeping the eventual
M2 direction (SwiftUI/Compose-style codegen vs Slint-style
IR/runtime interpretation — deferred question (b) in
[../decisions/phase-6-c-abi.md](decisions/phase-6-c-abi.md)) open.

**M1 experimental verifies:**

- **Stable-core property R/W as the post-construction update
  channel.** Hello Counter's `+/-` mutates `Text.content` via
  `wasamo_set_property`; this is the one runtime path both
  candidate M2 directions need.
- **Signal registry token lifecycle.** `wasamo_button_set_clicked`
  exercises both the direct-callback experimental shape and the
  underlying stable `wasamo_signal_connect` path.
- **Queued emission re-entrancy contract** on the UI thread (§6).
- **Bottom-up immutable tree construction** as one viable
  building primitive. This matches DSL semantics (`.ui` declares
  a tree, not a mutation sequence).

**M1 experimental does NOT verify, and intentionally does not
attempt to:**

- **Tree mutation primitives** (incremental `append_child`,
  widget destroy of unattached subtrees, reparenting). Whether
  these belong in any future ABI surface depends on the
  resolution of deferred question (b); investigated in M2 pre-doc.
- **Codegen vs IR design alternatives.** This is the core M2
  question and belongs to M2 pre-doc, not M1 implementation.
  Prototyping multiple candidates is M2's job.
- **Reactive primitives** (conditional rendering, list
  rendering, fine-grained reactivity). These are M2+ scope; M1
  validates only static tree construction with property-level
  updates.
- **`.ui` DSL → ABI lowering.** M1 wasamoc is parser-only by
  design; host code constructs the equivalent tree directly
  through the experimental layer. The lowering itself is M2
  scope.

This division is recorded so M1 implementation work is not
inflated by speculative future-proofing, and so M2 pre-doc starts
from a clean slate rather than from M1 stopgap shapes that may
read as commitments.

## 6. Threading and re-entrancy

Wasamo follows strict UI-thread affinity:

- The thread that calls `wasamo_init` is the **UI thread**.
- All other `wasamo_*` functions, except as noted, must be called
  on the UI thread. Calling from another thread is undefined
  behavior; the runtime may detect it via debug assertions but is
  not required to in release builds. When detected, the function
  returns `WASAMO_ERR_WRONG_THREAD`.
- All callbacks (signal handlers, property observers, destroy
  functions) are invoked on the UI thread.
- **Re-entrancy:** while the host is inside a `wasamo_*` call, the
  runtime does not invoke any callback on that thread. Emissions
  triggered by the call are queued and drained after the call
  returns. Callbacks may freely call back into the ABI.

Cross-thread UI updates in M1 use the standard Win32 escape hatch:
the host obtains the window's HWND through a future
(`WASAMO_EXPERIMENTAL`) accessor, posts a custom message via
`PostMessage`, and performs the `wasamo_*` work in the UI-thread
message handler. A built-in `wasamo_post` is deferred to the phase
that needs it; adding it later is purely additive.

## 7. Header generation, distribution, and CI

`wasamo.h` is **hand-written**. It is the canonical artifact this
document mirrors. The Rust source (`extern "C"` block) and the
header are kept in sync by CI, not by code generation:

- A C compilation smoke test in CI builds a minimal TU that
  `#include`s `wasamo.h` and exercises every public function
  signature; linker errors against `wasamo.lib` catch ABI drift.
- A function-name parity check (optional, may land later) parses
  both `wasamo.h` and the Rust ABI block and asserts the function
  sets agree.

The two-layer split is expressed by section ordering and the
`WASAMO_EXPERIMENTAL` marker, not by `#ifdef` gates — hosts get
both layers from the same header.

`wasamo.h` and the import library `wasamo.lib` are placed under
`bindings/c/` in Phase 7.

---

## Appendix A. Summary of cross-references to ADR

| Spec section | ADR decision |
|---|---|
| §1 conventions, §3.1 status, §4.1 last-error | DD-P6-005 |
| §2.1 export, §2.2 calling convention, §2.3 ownership | DD-P6-007 |
| §3.3 `WasamoValue`, §4.5 signals | DD-P6-002 |
| §3.4 callbacks, §4.4 observers, §4.5 signals (lifetime) | DD-P6-003 |
| §4 stable core scope | DD-P6-001 |
| §5 experimental layer | DD-P6-001 (experimental layer), framing |
| §6 threading | DD-P6-004 |
| §7 header generation | DD-P6-006 |
