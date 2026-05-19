# Wasamo C ABI Specification

**Version:** M2 Foundation ABI surface (2026-05-11)
**Status:** Accepted for M2 — finalised against the implemented `wasamo.h`; M3-Phase 1 in progress with no new ABI surface
**Authoritative decisions:** [decisions/phase-6-c-abi.md](decisions/phase-6-c-abi.md) (DD-P6-001..007), M2 phase ADRs under [decisions/](decisions/)

This document specifies the C ABI exposed by `wasamo.dll` via the
`wasamo.h` header. It is the normative reference for binding
authors and for the runtime implementation. The header is the
artifact reviewers reason about; this document is its prose
counterpart.

The ABI is **two-layer**:

- **Stable core** — candidate surface for the M6 ABI freeze. Functions
  and types in this layer are designed to survive both deferred
  Phase 6 questions: (a) where DSL inline handler bodies execute,
  and (b) `wasamoc`'s M2 output format. They are not yet frozen;
  M1-M5 may revise them. M6 commits to backward compatibility
  going forward.
- **M1 experimental** — exists because M1 `wasamoc` is parser-only
  and host code must construct widget trees imperatively. Every
  symbol in this layer is marked `WASAMO_EXPERIMENTAL` in the
  header and is **not** subject to M6 stability. Code that uses
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
`wasamo.h` do **not** define it. Static linking is unsupported.

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

#define WASAMO_OK                       0
#define WASAMO_ERR_INVALID_ARG         -1
#define WASAMO_ERR_RUNTIME             -2
#define WASAMO_ERR_NOT_INITIALIZED     -3
#define WASAMO_ERR_WRONG_THREAD        -4
#define WASAMO_ERR_REENTRANT_LOAD      -5  /* DD-M2-P6-001 */
#define WASAMO_ERR_REACTIVE_DIVERGED   -6  /* DD-M2-P6-001 / DD-M2-P6-006 */
#define WASAMO_ERR_OBSERVER_MUTATION   -7  /* DD-M2-P6-001 */
#define WASAMO_ERR_IR_MALFORMED        -8  /* DD-M2-P6-005 / DD-M2-P6-009 */
```

The status space is closed at M6. New codes added before M6 are
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

The tag set is closed at M6. New tags added before M6 are
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
change before M6. The contract here:

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

### 4.6 Tree mutation

Added in M2-Phase 4 (DD-M2-P4-001/002/003 = Option A). Grows the stable
core with a sixth area: index-based widget-tree mutation. Constructors
(`wasamo_*_create`) remain in the M1 experimental layer (§5) until a later
DSL/widget-surface milestone settles their parameter shapes.

```c
WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_widget_append_child(
    WasamoWidget*  parent,
    WasamoWidget*  child);

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_widget_insert_child(
    WasamoWidget*  parent,
    size_t         index,
    WasamoWidget*  child);

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_widget_remove_child(
    WasamoWidget*   parent,
    size_t          index,
    WasamoWidget**  out_removed);

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_widget_replace_child(
    WasamoWidget*   parent,
    size_t          index,
    WasamoWidget*   new_child,
    WasamoWidget**  out_old);

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_widget_child_count(
    WasamoWidget*  parent,
    size_t*        out_count);

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_widget_destroy(WasamoWidget* widget);
```

**Identifier scheme.** Children are addressed by zero-based index into the
parent's ordered child list (DD-M2-P4-002 = Option A). `child_count` is
provided so hosts can construct loop bounds; no random-access `_get` is
exposed (the returned handle's lifetime would be tied to the list position,
which shifts under any subsequent `insert` or `remove`).

**Widget lifecycle and attached state.** Each widget is in one of three
states:

- **Detached / host-owned** — produced by a constructor
  (`wasamo_*_create`) or by a successful `remove_child` /
  `replace_child` call. The host holds the only reference.
- **Attached** — successfully passed to `append_child`,
  `insert_child`, or `replace_child` as the `new_child`. Ownership
  transfers to the parent node.
- **Window-root** — attached via `wasamo_window_set_root`; freed when
  the window is destroyed.

A widget in the **attached** or **window-root** state may not be passed to
`wasamo_widget_destroy`; remove it from its parent first.

**`wasamo_widget_append_child`** appends `child` as the last child of
`parent`. Equivalent to `insert_child(parent, child_count(parent), child)`.
A separate entry point is provided because appending is the most common
operation and avoids a round-trip `child_count` query. Returns
`WASAMO_ERR_INVALID_ARG` if `child` is already attached.

**`wasamo_widget_insert_child`** inserts `child` at `index`; existing
children at `index` and beyond shift right. `index` must satisfy
`0 ≤ index ≤ child_count`. Returns `WASAMO_ERR_INVALID_ARG` if
`index > child_count` or if `child` is already attached.

**`wasamo_widget_remove_child`** detaches the child at `index` and writes
a detached handle to `*out_removed`. The host owns the handle after the
call (re-attach or destroy it). Returns `WASAMO_ERR_INVALID_ARG` if
`index ≥ child_count`.

**`wasamo_widget_replace_child`** atomically detaches the child at `index`
and attaches `new_child` in its place. Writes the detached old handle to
`*out_old`. Returns `WASAMO_ERR_INVALID_ARG` if `index ≥ child_count` or
if `new_child` is already attached.

**`wasamo_widget_child_count`** writes the number of direct children of
`parent` to `*out_count`.

**`wasamo_widget_destroy`** releases a detached widget and its entire
subtree. All registry entries (signal handlers, property observers) for
every node in the subtree are severed and their `destroy_fn` callbacks
invoked exactly once. The handle is invalid after the call. Behaviour on
a `NULL` argument: idempotent `WASAMO_OK`, matching
`wasamo_window_destroy`. Behaviour on an attached widget:
`WASAMO_ERR_INVALID_ARG` with last-error message
`"widget is currently attached; remove it from its parent first or destroy
the owning window"`.

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
numeric assignments are M1 stopgaps and may change before M6.

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
or when `wasamo_shutdown` is called, whichever comes first. The
stable-core `wasamo_widget_destroy` (§4.6) handles destruction of
detached widgets that have been removed from their parent.

### 5.2 `wasamo_load_ui` (DD-M2-P6-005)

```c
typedef int32_t WasamoLoadType;
#define WASAMO_LOAD_PATH    0
#define WASAMO_LOAD_MEMORY  1

WASAMO_EXPERIMENTAL
WasamoStatus wasamo_load_ui(
    WasamoLoadType  type,
    const void*     data,
    size_t          data_len,
    WasamoWindow**  out_root);
```

Single-function loader (Option α). The runtime parses the
`.ui`-derived IR (DD-M2-P6-002), constructs the widget tree, opens a
default-sized window, installs the tree as the window's root, and
returns the window handle through `*out_root`.

**`type` discriminant.**

- `WASAMO_LOAD_PATH` — `data` is a UTF-8 filesystem path of
  exactly `data_len` bytes. NUL termination is **not** expected;
  the runtime treats the slice as the full path. This matches the
  `(const char* utf8, size_t len)` convention used by every other
  string-bearing `wasamo_*` ABI.
- `WASAMO_LOAD_MEMORY` — `data` is a `data_len`-byte in-memory IR
  blob. M2 accepts only the IR text grammar (DD-M2-P6-002) and
  rejects non-UTF-8 bytes with `WASAMO_ERR_IR_MALFORMED`. The byte
  layout is the canonical shape so a future binary IR can be added
  by sniffing a header magic without changing the function
  signature.

`data_len == 0` is rejected as `WASAMO_ERR_INVALID_ARG` for both
modes; an unknown `type` is also `WASAMO_ERR_INVALID_ARG`.

**Errors.**

| Status | Cause |
|---|---|
| `WASAMO_OK` | Window opened, root installed, `*out_root` populated. |
| `WASAMO_ERR_INVALID_ARG` | Null `data` / null `out_root`, `data_len == 0`, unknown `type`, or non-UTF-8 path bytes. |
| `WASAMO_ERR_NOT_INITIALIZED` | `wasamo_init` has not been called. |
| `WASAMO_ERR_WRONG_THREAD` | Caller is not the runtime's owning thread (§6). |
| `WASAMO_ERR_IR_MALFORMED` | Header magic / version mismatch, parse error, unknown widget type, defense-in-depth validation failure (DD-M2-P6-009), or non-UTF-8 in-memory blob. |
| `WASAMO_ERR_RUNTIME` | I/O failure while reading the path, or window/Compositor construction failed. |
| `WASAMO_ERR_REENTRANT_LOAD` / `WASAMO_ERR_OBSERVER_MUTATION` / `WASAMO_ERR_REACTIVE_DIVERGED` | Standard structure-changing-ABI guards (DD-M2-P6-001 / DD-M2-P6-006). |

On any non-OK return, `*out_root` is left as `NULL` (the function
zeroes it on entry) and `wasamo_last_error_message` carries a
human-readable description.

**Handle ownership and lifetime.** The returned `WasamoWindow*` is
runtime-owned and remains valid until the host calls
`wasamo_window_destroy` or shuts the runtime down. The loader transfers
ownership of the constructed root widget to the returned window.

**Threading.** Like every other `wasamo_*` ABI, `wasamo_load_ui` must
be called on the runtime's owning UI thread (§6). Because the M2
contract fixes thread affinity at `wasamo_init` time, the loader
inherits the same thread; cross-thread calls return
`WASAMO_ERR_WRONG_THREAD` with no side effect.

**Auto-routing on installed roots.** When a window has a root
installed via `wasamo_window_set_root`, the runtime forwards
`WM_SIZE` to a re-layout pass, `WM_MOUSEMOVE` /
`WM_LBUTTON{DOWN,UP}` to per-widget hover and click hit-testing,
and emits `"clicked"` signals through the registry on Button hits.
Hosts do not need to wire window-message callbacks for these.

These symbols remain experimental after M2. Promotion, removal, or
replacement is deferred to the later DSL/widget-surface milestone that
settles constructor shapes for the stable core.

### 5.1 What M1 experimental verifies, and what it does not

At M1, the shape above was deliberately the smallest experimental
layer that let Phase 8 "Hello Counter" run while keeping the eventual
M2 direction open. M2 later chose textual IR plus runtime
interpretation (DD-M2-P2-001); this section remains as historical
rationale for why the constructor conveniences are still experimental.

**M1 experimental verifies:**

- **Stable-core property R/W as the post-construction update
  channel.** Hello Counter's `+/-` mutates `Text.content` via
  `wasamo_set_property`; this is the runtime path both M2 candidate
  directions needed.
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
  *M2-Phase 4 discharged this item: §4.6 stable-core mutation
  primitives now cover insert / remove / replace / destroy.*
- **Codegen vs IR design alternatives.** This was the core M2
  question and belonged to M2 pre-doc, not M1 implementation.
  M2 resolved it as textual IR plus runtime interpretation.
- **Reactive primitives** (conditional rendering, list
  rendering, fine-grained reactivity). These are M3+ scope; M1
  validates only static tree construction with property-level
  updates.
- **`.ui` DSL → ABI lowering.** M1 wasamoc was parser-only by
  design; host code constructed the equivalent tree directly
  through the experimental layer. The lowering itself landed in M2.

This division is recorded so M1 implementation work is not
inflated by speculative future-proofing, and so M2 pre-doc starts
from a clean slate rather than from M1 stopgap shapes that may
read as commitments.

## 6. Threading and re-entrancy

Wasamo follows strict UI-thread affinity:

- The thread that calls `wasamo_init` is the **UI thread** for the
  runtime's lifetime. The `ThreadId` is captured during `wasamo_init`.
- Every `wasamo_*` function except `wasamo_init` and
  `wasamo_last_error_message` checks the calling thread on entry and
  returns `WASAMO_ERR_WRONG_THREAD` without performing the requested
  action and without modifying runtime state. Functions with a `void`
  return (`wasamo_shutdown`, `wasamo_run`, `wasamo_quit`) silently
  no-op on a wrong-thread call but still record the violation in the
  thread-local last-error string. (DD-M2-P6-005.)
- Calls before `wasamo_init` return `WASAMO_ERR_NOT_INITIALIZED`. The
  same `void`-return rule applies to lifecycle entry points.
- `wasamo_last_error_message` is exempt from the thread check so a
  caller can read the violation message after a wrong-thread call.
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

### M2 batching contract (DD-M2-P4-004 = Option A)

The M2 batching contract is the existing queue-and-drain semantics
described above; no host-visible batching API was added in M2-Phase 4.

**Observable behaviour:** within a single host call frame, consecutive
`wasamo_set_property` calls on any widget (including calls made from
inside a signal-handler callback) are queued and their observer
notifications fired together in a single drain pass after control returns
to the outermost host frame. A host loop calling `wasamo_set_property`
N times in succession therefore causes observers to fire once per
observed `(widget, property_id)` pair, not N times.

*This is the M2 batching contract.* A host-visible begin/commit
transaction API — for cases where heterogeneous cross-widget operations
must be batched as a single observable event — is deferred to M3+
(DD-M2-P4-004 Out of scope). Adding it later is purely additive and does
not break the existing queue-and-drain contract.

## 7. Header generation, distribution, and CI

`wasamo.h` is **hand-written**. It is the canonical artifact this
document mirrors. The Rust source (`extern "C"` block) and the
header are kept in sync by CI, not by code generation:

- A C compilation smoke test in CI builds a minimal TU that
  `#include`s `wasamo.h` and exercises every public function
  signature; linker errors against `wasamo.dll.lib` catch ABI drift.
- A function-name parity check (optional, may land later) parses
  both `wasamo.h` and the Rust ABI block and asserts the function
  sets agree.

The two-layer split is expressed by section ordering and the
`WASAMO_EXPERIMENTAL` marker, not by `#ifdef` gates — hosts get
both layers from the same header.

`wasamo.h` lives under `bindings/c/`; the MSVC import library is emitted
by the Rust build as `target/<profile>/wasamo.dll.lib`.

---

## Appendix A. Summary of cross-references to ADR

| Spec section | ADR decision |
|---|---|
| §1 conventions, §3.1 status, §4.1 last-error | DD-P6-005 |
| §2.1 export, §2.2 calling convention, §2.3 ownership | DD-P6-007 |
| §3.1 status codes (M2 additions) | DD-M2-P6-001, DD-M2-P6-005, DD-M2-P6-006 |
| §3.3 `WasamoValue`, §4.5 signals | DD-P6-002 |
| §3.4 callbacks, §4.4 observers, §4.5 signals (lifetime) | DD-P6-003 |
| §4 stable core scope | DD-P6-001 |
| §5 experimental layer | DD-P6-001 (experimental layer), framing |
| §5.2 `wasamo_load_ui` | DD-M2-P6-005 |
| §6 threading | DD-P6-004, DD-M2-P6-005 (init-time fix, error returns) |
| §7 header generation | DD-P6-006 |
