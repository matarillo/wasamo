# Wasamo Architecture

**Status:** M1 complete (Phases 0-8); M2 complete (Phases 1-7) — Foundation acceptance A1-A6 discharged; M3-Phase 1, M3-Phase 2, M3-Phase 3, M3-Phase 4, and M3-Phase 5 complete. M3-Phase 6 closed (implementation-synced): ZStack, conditional rendering, and the component host surface are documented to match the landed implementation.

---

## 1. Cargo Workspace Layout

```
wasamo/                         ← workspace root
├── Cargo.toml                  ← workspace manifest
├── wasamo-runtime/             ← runtime rlib crate (rlib only; no DLL emitted)
│   ├── Cargo.toml              ← package = wasamo-runtime; [lib].name = "wasamo_runtime"
│   └── src/
│       └── lib.rs
├── wasamo-dll/                 ← cdylib shim crate (M2-Phase 1)
│   ├── Cargo.toml              ← [lib].name = "wasamo"; crate-type = ["cdylib"]
│   ├── build.rs                ← /WHOLEARCHIVE:libwasamo_runtime.rlib
│   └── src/
│       └── lib.rs
├── wasamoc/                    ← DSL compiler CLI crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── bindings/
│   ├── c/                      ← wasamo.h, smoke.c, CMakeLists.txt (Phase 6-7)
│   ├── rust-sys/               ← Rust raw FFI crate wasamo-sys (Phase 7)
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   └── src/
│   │       └── lib.rs
│   ├── rust/                   ← Rust safe wrapper crate wasamo (Phase 7)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   └── zig/                    ← Zig binding (Phase 7)
│       ├── build.zig
│       ├── build.zig.zon
│       ├── wasamo.zig
│       └── smoke_test.zig
└── examples/
    └── counter/                ← Hello Counter (Phase 8)
```

### Crate responsibilities

| Crate | crate-type | Output | Responsibility |
|---|---|---|---|
| `wasamo-runtime` | `rlib` | `libwasamo_runtime.rlib` | Runtime logic. Houses all `#[no_mangle] pub extern "C"` ABI symbol definitions. No DLL emitted — see §11.4. |
| `wasamo-dll` | `cdylib` | `wasamo.dll` + `wasamo.dll.lib` | Cdylib shim (M2-Phase 1). Depends on `wasamo-runtime`; re-exports all C ABI symbols via `/WHOLEARCHIVE`. `[lib].name = "wasamo"` — see note below. |
| `wasamoc` | `bin` | `wasamoc.exe` | `.ui` file parser and checker CLI. |
| `wasamo-sys` (at `bindings/rust-sys/`) | `lib` | Raw FFI crate | `extern "C"` declarations matching `wasamo.h`; `build.rs` links `wasamo.dll.lib` via `dylib:+verbatim`. |
| `wasamo` (at `bindings/rust/`) | `lib` | Safe Rust wrapper | Idiomatic Rust over `wasamo-sys`: `Runtime`/`Window`/`Widget`/`Value`/`Error`; `wasamo::experimental` for the M1 experimental layer. **This** is the supported public Rust API. |
| `examples/counter` *(Phase 8)* | `bin` | `counter.exe` | Sample app via the safe `wasamo` wrapper. |
| `bindings/zig/` | Zig package | link-time artifact | Zig binding: hand-written extern block + idiomatic wrappers. `wasamo.experimental` namespace mirrors the M1 experimental layer. |

`wasamo-dll` sets `[lib].name = "wasamo"` (not the cargo-conventional
`wasamo_dll`). This deviation is deliberate: `wasamo.dll` is the public
C ABI artifact name; changing it would break all
downstream consumers. The deviation is confined to the shim crate and
documented in `wasamo-dll/Cargo.toml` — see
[the cdylib-shim crate-naming decision](../process/milestone-2/phase-1/decisions/dd-m2-p1-002-naming-of-the-rlib-crate-and-the-shim-crate.md#dd-m2-p1-002-naming-of-the-rlib-crate-and-the-shim-crate).

### Inter-crate dependencies

```
wasamoc
  └── (future) wasamo-ast crate  ← to be split in M2; internal to wasamoc in M1

wasamo-dll  (cdylib shim; produces wasamo.dll)
  └── wasamo-runtime  (rlib; all C ABI symbol definitions)

bindings/rust  (safe wrapper, crate name: wasamo)
  └── wasamo-sys (raw FFI)
        ├── wasamo-dll  ← build-order edge; no Rust link
        └── wasamo.dll  (dynamic link via wasamo.dll.lib)

examples/counter
  └── bindings/rust

```

`wasamo-runtime` does not depend on any other Rust crate in this
workspace. `wasamo-dll` depends on `wasamo-runtime` (rlib) only.
The C ABI boundary is the only coupling point between the runtime
pair (`wasamo-runtime` + `wasamo-dll`) and the Rust binding pair.

### DSL build pipeline (M2-Phase 6 onward)

The `examples/counter-{c,rust,zig}/` hosts consume `examples/counter/counter.ui`
through `wasamoc`-emitted IR. The pipeline at build time:

```
counter.ui  ──[wasamoc build]──▶  counter.uic  ──[host build]──▶  counter-{c,rust,zig}.exe
   (DSL)                            (IR text;                       (host calls
                                     wasamoc-internal               wasamo_load_ui at
                                     emit format)                   runtime with the
                                                                    .uic content)
```

Each host's build system invokes `wasamoc` directly:

- **counter-rust**: `build.rs` runs `wasamoc build` and writes the IR to `OUT_DIR`;
  the binary loads it via absolute path through `wasamo_load_ui` (`WASAMO_LOAD_PATH`).
- **counter-c**: CMake's `add_custom_command` runs `wasamoc build` and a generated
  C header (`xxd -i`-equivalent) embeds the IR bytes; the binary passes the blob
  through `wasamo_load_ui` (`WASAMO_LOAD_MEMORY`).
- **counter-zig**: a `build.zig` step runs `wasamoc build` and writes the IR
  alongside the build artifacts; `@embedFile` inlines it; the binary passes the
  blob through `wasamo_load_ui` (`WASAMO_LOAD_MEMORY`).

This means **every host build depends on `wasamoc` having been built first**.
For Rust, `cargo build -p counter-rust` resolves this via the workspace and
`build.rs`'s `cargo:rerun-if-changed` directive; for the C and Zig hosts,
`cargo build -p wasamoc` must precede the host build. See `AGENTS.md` §
"Build ordering requirements" for the operational rule.

**This pipeline is provisional.** The current shape — three independent
build systems each invoking `wasamoc` — is acceptable while M2 has a single
DSL example, but does not generalize:

- **Hot reload** (post-1.0;
  [`wasamoc`-output-format ADR](../process/milestone-2/phase-2/decisions/preamble.md))
  presumes IR can be loaded at runtime without re-linking the host. The
  build-time embed model in counter-{c,zig} is incompatible with that goal
  and would need to migrate to `WASAMO_LOAD_PATH` with a runtime-discovered
  IR file.
- **Multiple `.ui` files per host** (likely from M3 onward) makes per-host
  hand-written wasamoc invocation untenable; a higher-level mechanism (e.g.
  a `cargo wasamoc` subcommand or a build-system-agnostic `wasamoc` driver
  emitting per-host integration shims) is the natural successor.

**Re-evaluation triggers:**

- M3 DSL spec drafting — when the public DSL surface stabilizes, decide
  whether `wasamoc` should expose a build-time integration API beyond
  `wasamoc build <file.ui>`.
- M3+ — the first host with more than one `.ui` file.
- Hot reload feasibility work (post-1.0) — if pursued, the embed-at-build
  model is out and runtime IR loading becomes the only path.

The current per-host invocation should be treated as an **expedient for the
M2 acceptance gate**, not as the long-term architecture.

---

## 2. Layer Diagram

```
┌───────────────────────────────────────────────────┐
│  App Code  (C / Rust / Zig / …)                   │
│  Business logic, state, callbacks                 │
├───────────────────────────────────────────────────┤
│  Language Bindings                                │
│  Thin per-language wrappers                       │
│  (C uses wasamo.h directly; Rust uses bindings/)  │
├───────────────────────────────────────────────────┤
│  C ABI boundary  ←  wasamo.h / wasamo.dll         │
├───────────────────────────────────────────────────┤
│  Wasamo Runtime  (wasamo crate, written in Rust)  │
│  Widget tree / Layout / Property management       │
│  Input / Animation                                │
├───────────────────────────────────────────────────┤
│  Render Backend                                   │
│  Windows.UI.Composition (Visual Layer)            │
│  + DirectWrite + Direct2D                         │
├───────────────────────────────────────────────────┤
│  OS: Windows 10 1809+  (Win32 HWND host)          │
└───────────────────────────────────────────────────┘
```

---

## 3. C ABI Boundary and wasamo.dll

`wasamo.dll` is the single deployable artifact that exposes the C ABI. Any language that can `#include` a C header — C, C++, Rust FFI, Zig `@cImport`, Go `cgo` — can call it directly.

- Every public function carries `WASAMO_EXPORT` (`__declspec(dllexport)`)
  and the `__cdecl` calling convention (`WASAMO_API`).
- Public types are opaque pointers (`WasamoWindow*`, `WasamoWidget*`); the
  runtime never reveals their layout.
- Error handling uses `WasamoStatus` (negative = error) plus an
  out-parameter for produced handles. A thread-local
  `wasamo_last_error_message` carries a human-readable description of
  the most recent non-OK status on the calling thread.
- Strict UI-thread affinity: the thread that calls `wasamo_init` owns
  the runtime; all other functions and callbacks fire on it.
- Re-entrancy: callbacks are queued and drained at safe boundaries —
  no callback fires while the host is inside a `wasamo_*` call.

The full ABI specification is `docs/abi_spec.md` (accepted for M2).
No ABI stability guarantee is made before 1.0; M6 is when stability
commitments begin.

`abi_spec.md` is structured in **two layers**:

- **Stable core** — runtime lifecycle, window + event loop, property
  get/set, property change observers, signal connect/disconnect, and
  **tree mutation** (M2-Phase 4: append / insert / remove / replace /
  child_count / widget_destroy — `abi_spec.md §4.6`).
  Written as a candidate surface for the M6 ABI freeze.
  The stable core covers **six areas** as of M2-Phase 4 (the initial
  five-area minimum was defined in M1; §4.6 tree mutation is the
  sixth area added in M2).
- **M1 experimental** — all-at-once widget constructors
  (`wasamo_text_create`, `wasamo_button_create`, `wasamo_vstack_create`,
  `wasamo_hstack_create`), `wasamo_window_set_root`, and the typed
  `wasamo_button_set_clicked` convenience. Required because M1 `wasamoc`
  is parser-only and the host must construct the widget tree by hand.
  Marked `WASAMO_EXPERIMENTAL` in both header and spec; not subject to
  M6 stability. Constructor promotion to stable core is deferred to a
  later DSL/widget-surface milestone.

M2 resolved the two Phase 6-deferred questions that shaped the stable core:
DSL inline handler bodies execute in the runtime interpreter, and
`wasamoc` emits IR consumed by `wasamo_load_ui`. The stable core
remains sized to survive
those decisions and later M3+ surface growth.

---

## 4. External Crate Policy

### Principle: minimize dependencies; use `windows` for all Windows APIs

For M1, only the following crate is adopted:

| Crate | Purpose | Rationale |
|---|---|---|
| `windows` | Rust bindings for Win32 and WinRT APIs | Official Microsoft crate (a.k.a. windows-rs). Provides the same type safety as C++/WinRT. |

Adding `clap` or similar to `wasamoc` (CLI) is acceptable. Adding any dependency to the `wasamo` runtime DLL requires explicit case-by-case approval.

### OSS adoption criteria (for future phases)

For non-trivial algorithms introduced in later phases (layout, accessibility, etc.), a proven OSS library is preferred over a custom implementation when all of the following hold:

- **Rust-native**: no C FFI required (avoids build system complexity and unsafe surface area)
- **Production-proven**: the library has real-world deployment history at meaningful scale
- **Low integration cost**: the library's output maps naturally onto Visual Layer primitives without a large bridging layer
- **Acceptable dependency risk**: upstream bugs or API churn would not block the project

Specific adoption decisions are made in the pre-implementation document for the relevant phase and require owner agreement before implementation begins.

### `windows` crate feature set

```toml
[dependencies.windows]
version = "0.58"
features = [
  # Core Win32
  "Win32_Foundation",
  "Win32_UI_WindowsAndMessaging",
  "Win32_UI_Input_KeyboardAndMouse",  # TrackMouseEvent, WM_MOUSELEAVE (Phase 4)
  "Win32_System_LibraryLoader",
  # Graphics — Phase 2
  "Win32_Graphics_Dwm",
  "Win32_Graphics_Gdi",
  # Graphics — Phase 4 (text rendering pipeline)
  "Win32_Graphics_Direct2D",
  "Win32_Graphics_Direct2D_Common",
  "Win32_Graphics_Direct3D",
  "Win32_Graphics_Direct3D11",
  "Win32_Graphics_DirectWrite",
  "Win32_Graphics_Dxgi",
  "Win32_Graphics_Dxgi_Common",
  # WinRT interop
  "Win32_System_WinRT",               # ICompositionDrawingSurfaceInterop (Phase 4)
  "Win32_System_WinRT_Composition",
  "Win32_UI_Controls",
  # WinRT / Composition
  "Foundation",                       # Windows.Foundation.Size (Phase 4)
  "Graphics_DirectX",                 # DirectXPixelFormat/AlphaMode (Phase 4)
  "System",
  "UI",
  "UI_Composition",
  "UI_Composition_Desktop",
  "UI_ViewManagement",                # UISettings accent color (Phase 4)
]
```

---

## 5. Visual Layer Integration (Phase 2)

Full decision rationale: [`process/milestone-1/phase-2/decisions/preamble.md`](../process/milestone-1/phase-2/decisions/preamble.md)

### 5.1 HWND host model

A Win32 `HWND` created by `CreateWindowExW` hosts the Visual Layer via `DesktopWindowTarget`.

```
HWND
  └── DesktopWindowTarget
        └── ContainerVisual (root)
              └── (widget SpriteVisual tree)
```

### 5.2 Initialization sequence

```
1. CreateDispatcherQueueController(           — init current thread as STA + attach DQ
       DQTYPE_THREAD_CURRENT, DQTAT_COM_STA)
2. Compositor::new()                          — create WinRT Compositor
3. CreateWindowExW(WS_EX_NOREDIRECTIONBITMAP) — create HWND; flag prevents GDI redirection
                                                buffer that would paint over DWM backdrop
4. apply_mica(hwnd)                           — DwmSetWindowAttribute (Win11); no-op on Win10
5. DesktopWindowTarget::CreateForWindow(hwnd) — attach Visual Layer to HWND
6. ContainerVisual::new() → set as root       — root visual (no background brush; Mica shows through)
7. GetMessage / TranslateMessage / DispatchMessage loop
```

`WM_ERASEBKGND` returns 1 to prevent GDI from painting an opaque background over the DWM
backdrop. `DwmExtendFrameIntoClientArea` is **not** called: with `DWMSBT_MAINWINDOW` the Mica
material covers the entire window automatically; calling it with `{-1,-1,-1,-1}` margins causes
DWM to render the dark frame colour across the client area, covering Mica.

### 5.3 `windows` crate feature additions for Phase 2

```toml
"System",              # Windows::System::DispatcherQueueController
"Win32_Graphics_Dwm",  # DwmSetWindowAttribute, DwmExtendFrameIntoClientArea, DWMWA_* constants
```

`Win32_System_WinRT` (already present) provides `CreateDispatcherQueueController`,
`DispatcherQueueOptions`, `DQTYPE_THREAD_CURRENT`, and `DQTAT_COM_STA`.
(`"System_DispatcherQueue"` does not exist in windows 0.58 — types live directly in the `System` module.)

---

## 6. Layout Engine (Phase 3)

Full decision rationale: [`process/milestone-1/phase-3/decisions/preamble.md`](../process/milestone-1/phase-3/decisions/preamble.md)

### 6.1 Module structure

| Module | Win32/WinRT dependency | Responsibility |
|---|---|---|
| `wasamo/src/layout.rs` | None (pure Rust) | `LayoutNode` data type; `measure()`, `arrange()`, `run_layout()` |
| `wasamo/src/widget.rs` | `windows` crate | `WidgetNode` — `SpriteVisual` + layout configuration + child tree |

The split keeps all layout calculation free of Win32/WinRT so it is unit-testable without
OS initialisation.

### 6.2 Algorithm: two-pass measure/arrange

```
run_layout(root, window_w, window_h)
  │
  ├─ measure(root, window_w, window_h)       — returns desired (w, h); recurses into children
  │
  ├─ resolve root size against SizeConstraint
  │    Fixed(v) → v  |  Fill → available  |  Shrink → desired
  │
  └─ arrange(root, 0, 0, final_w, final_h)   — writes offset/size; recurses into children
```

### 6.3 Size model

```rust
pub enum SizeConstraint { Fixed(f32), Fill, Shrink }
```

| Value | `measure()` returns | Final size in `arrange()` |
|---|---|---|
| `Fixed(v)` | `v` | `v` |
| `Fill` | `0.0` — signals "take what parent allocates" | Remaining space after Fixed+Shrink siblings |
| `Shrink` | Content size | Content size |

Default constraints by widget type:

| Widget | Width default | Height default |
|---|---|---|
| `VStack` | `Fill` | `Shrink` |
| `HStack` | `Shrink` | `Fill` |
| `Rectangle` | Caller-specified | Caller-specified |

**Window-root sizing (runtime boundary, M3-Phase 4).** The
window-root `WidgetNode` is sized to the client rect regardless of
its declared width/height constraints. This is enacted by
`WidgetNode::run_layout_as_window_root`, which overrides the root
`LayoutNode`'s `width` / `height` to `SizeConstraint::Fill` before
delegating to `layout::run_layout`. The plain `WidgetNode::run_layout`
retains the declared-constraint semantics (used by mock-free
integration tests that drive `WidgetNode`s as non-window roots).
`window.rs`'s `WM_SIZE` handler and `set_root` initial layout call
the window-root variant; the pure-logic layout engine and the
declared-constraint conventions (including `degenerate_fill_in_
shrink_parent_clamps_to_zero`) are untouched.

### 6.4 Cross-axis alignment

```rust
pub enum Alignment { Leading, Center, Trailing, Stretch }
```

`alignment` on a stack governs child placement on the cross axis (VStack cross = horizontal;
HStack cross = vertical). `Stretch` is the default. A child with `Fill` on the cross axis
always expands to the full inner extent regardless of the stack's `alignment`.

### 6.5 WidgetNode and Visual Layer sync

```
WidgetNode tree  (owns SpriteVisuals + child WidgetNodes)
  │
  ├── build_layout_tree()  →  LayoutNode tree (pure, temporary)
  │
  ├── layout::run_layout()  →  fills offset/size on each LayoutNode
  │
  └── sync_visuals()  →  Visual.SetOffset / Visual.SetSize on each SpriteVisual
```

`LayoutNode` offsets are absolute (root-relative); `sync_visuals()`
converts each child offset to parent-relative `Visual.Offset` before
writing the Composition visual tree.

M3-Phase 4 ScrollView locally extends the prior "1 WidgetNode = 1
Visual" convention. A ScrollView owns an outer widget Visual plus a
ScrollView-owned intermediate content Visual between that outer Visual
and its single content child's widget Visual. The outer Visual carries
the viewport clip (`Visual.Clip = InsetClip { 0, 0, 0, 0 }`). The
ScrollView-owned intermediate content Visual carries the scroll
translation:

```
Visual.Offset = (0, -offset_y, 0)
```

where `offset_y` is the clamped applied offset from the ScrollView
layout pass. The single content child's widget Visual remains governed
by the normal `sync_visuals()` conversion from absolute `LayoutNode`
offsets to parent-relative `Visual.Offset`. The intermediate content
Visual and the child widget Visual do not carry the viewport clip.

Because the intermediate content Visual carries the scroll
translation `(0, -offset_y, 0)`, `sync_visuals()` shifts each
content-subtree descendant's `parent_abs_offset` by the same
`(0, -offset_y)` when descending under that intermediate Visual, so
the root-relative `LayoutNode` offset of every descendant converts
to the correct parent-relative `Visual.Offset` underneath the
translated parent. This shift is local to the ScrollView subtree and
does not affect siblings or ancestors of the ScrollView. Future
widgets that introduce their own intermediate Visuals follow the
same rule: any Visual that translates its own subtree must shift
`parent_abs_offset` by the inverse translation for its descendants.

ScrollView is the only M3 primitive that adds an intermediate Visual.
Grid (§6.8.4) and ZStack (§6.8.5) keep the `1 WidgetNode = 1 Visual`
convention: each carries its outer-bounds clip on its **own** Visual
(`Visual.Clip = InsetClip{0,0,0,0}`) with child Visuals attached
through the normal document-order `sync_visuals()` path and
`Visual.Clip = null`. M3-Phase 6 additionally tightens `sync_visuals()`
/ `insert_child` so that a structurally inserted or removed child (a
conditional subtree, §6.7.9) lands at the Visual sibling position
matching its `children` Vec index, rather than always at the top —
keeping Visual sibling order equal to child order after every
structural mutation, which a ZStack conditional child relies on for
correct document-order z-order.

The `LayoutNode` tree is rebuilt on each layout pass (O(n)).
No persistent layout cache exists in M1.

### 6.6 Layout invalidation on property and structural change (M1-Phase 8; M3-Phase 6)

Before Phase 8, the only path that triggered a layout pass was `WM_SIZE`.
`wasamo_set_property` for size-affecting properties (`BUTTON_LABEL`,
`TEXT_CONTENT`, `TEXT_STYLE`) updated the widget's intrinsic
`width`/`height` but left the surrounding tree visually stale.

**Implementation (Phase 8):**

- `WidgetNode::set_property` detects size-affecting property updates and
  calls `emit::mark_layout_dirty_for(widget_ptr)`.
- `emit::mark_layout_dirty_for` walks the live-window registry
  (`WINDOWS` thread-local in `emit.rs`) to find the window whose
  `root_widget` subtree contains the widget, then adds that window to
  a `DIRTY` set.
- After each `drain_if_outermost` cycle empties the callback queue,
  `flush_layout` runs one `run_layout` pass on every window in `DIRTY`
  and clears the set. Multiple property changes within one drain cycle
  coalesce into a single pass per window.
- Widgets not yet attached to a window (pre-`set_root`) defer; layout
  runs when they enter a window via `wasamo_window_set_root`.
- `BUTTON_STYLE` does not affect intrinsic size in M1 (Default and
  Accent share the same metrics); it remains a pure visual refresh.

M3-Phase 6 extends the same dirty-window path to structural conditional
mutation. A successful conditional subtree insert or remove calls
`mark_layout_dirty_for` through the parent widget after the tree mutation
lands. This is required even when no size-affecting property write
occurs: a newly-present or newly-absent subtree can change parent
measurement, placement, clip contents, and ZStack document-order visual
composition. The phase keeps the existing whole-window dirty granularity;
subtree-grain invalidation remains a later optimization question.

Window registration lifecycle:
- `window::create` calls `emit::register_window` after `Box<WindowState>` is
  heap-allocated (pointer is stable).
- `wasamo_window_destroy` calls `emit::unregister_window` before the box
  is dropped.

### 6.7 Reactive engine (M2-Phase 5)

Full decision rationale: [`process/milestone-2/phase-5/decisions/preamble.md`](../process/milestone-2/phase-5/decisions/preamble.md).
Architectural-family hypothesis (tree-with-bindings, working
hypothesis only): [`docs/notes/architectural-family.md`](notes/architectural-family.md).

The reactive engine is the M2 thesis-validation surface for
acceptance A2 — `count++` in a host handler updates a bound
`Text` label without any host-side `wasamo_set_property` call. It
sits entirely inside `wasamo-runtime` and is `pub(crate)`; no C
ABI symbol is added.

#### 6.7.1 Module placement

| Module | Responsibility |
|---|---|
| `wasamo-runtime/src/reactive.rs` | `Signal<T>` / `EffectHandle` / dependency graph / `with_batched_writes` / dirty-set drain / `BindingEvalContext` / `register_binding` / `BindingTarget` |
| `wasamo-runtime/src/handler.rs` | `HandlerExpr` AST + `EvalContext` trait + `evaluate()` (Phase 3); reused by the binding evaluator in read-only mode |
| `wasamo-runtime/src/widget.rs` | `WidgetNode.bindings: Vec<EffectHandle>`; binding disposal during the Phase 4 `widget_destroy` sweep |
| `wasamo-runtime/src/emit.rs` | `drain_if_outermost` integrates the reactive drain between observer drain and layout drain |

Pure-logic surfaces — Signal storage, dependency tracker,
dirty-set drain, evaluator wiring — are unit-tested with
side-effect-logger Effect closures; no
test-only mirror of `WidgetNode` is introduced.

#### 6.7.2 Two-layer primitive

```
Signal<T>   — observable storage cell
  │  get()   → records dependency on current Effect (if any)
  │  set(v)  → marks dependent Effects dirty (no inline re-run)
  ▼
Effect      — re-runnable closure with auto-tracked dependencies
   run()    → push self onto thread-local effect stack;
              evaluate body; pop; reconcile dependency edges
```

Dependency collection is automatic: `Signal::get()` peeks the
thread-local "current effect" stack and, if present, records the
edge in both directions (forward `SignalId → {EffectId}` for
dirty propagation; back `EffectId → {SignalId}` for disposal).
Re-running an Effect first clears its prior back-edges so a
binding may pick up different Signals each pass — this is what
makes future M3 conditional bindings (`if cond { a } else { b }`)
work without per-binding scaffolding.

`Signal::get_untracked()` is the escape hatch for the rare reads
outside dependency collection (e.g. diagnostics). The general
"reads outside any Effect" case (host code, handler bodies that
incidentally read a Signal) is already untracked: the
thread-local stack is empty, so `get()` short-circuits without
recording an edge.

`Computed<T>` (a third layer between Signal and Effect) is **not**
introduced in M2; it is shape-additive and lands with the M3 DSL
spec when derivation grammar exists to align against. Same for
`untrack` as a public escape hatch (out of scope).

#### 6.7.3 Drain ordering inside `drain_if_outermost`

The reactive dispatch is **deferred**, not synchronous.
`Signal::set()` writes the new value, marks dependent
Effects in a thread-local dirty-set, and returns. Re-evaluation
runs at the outermost-frame boundary — the same boundary
already used for queued observer notifications.

The drain itself is a three-phase + terminal transaction.
Phase 1 unifies signal-handler firing and
reactive Effect re-runs into a single mutation-convergence loop;
Phase 2 runs layout against a frozen state; Phase 3 fires
post-commit observers under a TLS flag that blocks state mutation:

```
drain_if_outermost()
  │
  ├─ Phase 1: Mutation convergence  (loop until fixed point)
  │     while signal_queue ≠ ∅ OR dirty_effects ≠ ∅:
  │         if signal_queue ≠ ∅:
  │             pop signal handler, fire host callback
  │             (callback may freely mutate state via ABI)
  │         else if dirty_effects ≠ ∅:
  │             take one dirty Effect, re-run in topological order
  │             (effect body calls internal set_property)
  │         iter += 1
  │         if iter > MUTATION_CAP: enter Diverged terminal state
  │
  ├─ Phase 2: Layout  (1 pass, terminal; read-only over runtime state)
  │     for each layout-dirty window: run_layout
  │
  └─ Phase 3: Post-commit observers  (1 pass, terminal)
        IN_OBSERVER_CALLBACK := true
        drain observer queue;
          state-mutating ABI returns WASAMO_ERR_OBSERVER_MUTATION
          (panic in debug)
        IN_OBSERVER_CALLBACK := false
```

Return-time invariant: `signal_queue` empty, `dirty_effects`
empty, layout-dirty empty, `observer_queue` empty.

Phase 1 ordering rules — required for structural determinism:
(1) `signal_queue` is FIFO in emission order; (2) `dirty_effects`
drains in topological order over the Signal dependency graph,
ties broken by `EffectHandle` registration order; (3) same-cycle
write-after-write to a Signal is last-wins, with the Phase 3
observer queue computed from the diff between each Signal's
value at Phase 1 entry and exit (intermediate transitions do not
produce observer entries). The signal/Effect alternation in the
spec block is one canonical interleaving; an implementation may
batch (e.g. drain all signals, then all dirty Effects in
topological order) provided the per-Signal value sequence is
identical.

Phase 2 is strictly read-only: layout reads property values to
compute geometry but does not subscribe to Signals (no
dependency edge originates in Phase 2) and does not write
properties or emit signals. A layout pass that wrote properties
would create a Phase 1↔Phase 2 cycle outside the fixpoint
convergence; the layout API surface (internal to the runtime)
takes a read-only view of property values and returns geometry,
nothing more.

Mutation boundary — what observer callbacks (Phase 3) may and
may not do:

- **Forbidden** (TLS-flag detected, returns
  `WASAMO_ERR_OBSERVER_MUTATION`, panics in debug):
  runtime state writes (`wasamo_set_property`,
  `wasamo_emit_signal`, `wasamo_signal_set`), runtime structure
  changes (window/element/binding create/destroy,
  parent/child reparenting), reactive graph intervention
  (Effect register/dispose, Signal subscribe/unsubscribe),
  re-entrant `wasamo_*` calls.
- **Allowed**: external I/O (file, network, IPC, log,
  telemetry), host-side (runtime-external) state mutation, pure
  reads of runtime state (`wasamo_get_property`, Signal value
  reads), task submission to other threads (the receiving
  thread is bound by the existing UI-thread affinity rule, not
  by the observer mutation rule).
- **Path back into runtime state** when an observer needs to
  trigger one: route through a host-side queue and post a
  signal on the next ABI entry, or use the future
  `wasamo_post_event` API (scheduled for
  M3). Observer callbacks never write runtime state directly.

`with_batched_writes(f)` (Phase 4 skeleton, body filled in
Phase 5) increments a thread-local depth counter; the per-call
drain at the end of each `wasamo_*` entry is suppressed while
depth > 0. On outermost-frame exit, a single drain processes the
accumulated dirty-set. The iteration cap is a small constant
(16 in current implementation, named `MUTATION_CAP`) — enough headroom
for legitimate multi-pass
cascades, low enough to surface a divergent binding before it
exhausts CPU. Cap exhaustion is fatal: the runtime transitions
to a `Diverged` terminal state, Phase 2 and Phase 3 are skipped
for that frame, and every subsequent ABI call other than
`wasamo_runtime_destroy` returns `WASAMO_ERR_REACTIVE_DIVERGED`.

#### 6.7.4 Runtime safety guard placement

Guard placement is a global runtime invariant.
Re-entrancy and lifecycle guards are enforced with
role-specified defense in depth:

- **ABI boundary = diagnostic boundary.** Exported `wasamo_*`
  functions check the relevant runtime state before touching state.
  This layer owns caller-facing `WasamoStatus` values and
  `wasamo_last_error_message` text because it has the public function
  name, argument context, and lifecycle exception in hand.
- **Internal runtime boundary = invariant boundary.** Runtime-owned
  entry points that can be reached without crossing an exported ABI
  function must guard the invariant before executing. In M2,
  `emit::drain_if_outermost()` is that boundary for the drain
  transaction: re-entry while `IN_DRAIN` is set is a no-op, and
  `RuntimeHealth::Diverged` suppresses all drain phases.
- **Non-ABI entries are first-class runtime entries.** The Win32
  message-loop drain in `lib.rs::run()` and future M3 timer,
  async-I/O, or additional window-procedure callbacks must enter
  runtime state through an internal invariant boundary. They do not get
  to rely on ABI-only guards because they do not cross the ABI.
- **Cleanup exceptions are explicit.** Any operation allowed after
  `Diverged` (for example destroy/cleanup paths) must be named at its
  entry boundary. Such an exception does not grant general permission
  to mutate or drain runtime state after divergence.

The current guard set is therefore interpreted by role:
UI-thread-affinity and public error reporting live at ABI entry;
`IN_DRAIN` and `IN_OBSERVER_CALLBACK` remain observable ABI refusals
for structure-changing and state-mutating calls; `drain_if_outermost`
also guards the internal transaction boundary used by non-ABI message
loop entry. M3+ entry paths inherit this split unless a later ADR
reopens the guard-placement principle, most likely because typed guard
tokens become worth their API cost.

#### 6.7.5 Signal-dispatch ordering (signal-side runtime contract)

Independent of the reactive drain, when a `WasamoSignal` fires
through `signal_emit`:

```
signal_emit(widget, signal_id, payload)
  │
  ├─ 1. Inline handler         (runtime-side
  │     HandlerExpr evaluator with EvalContext)
  │
  └─ 2. Host listener iteration (existing C ABI observer list)
```

Inline handlers run before host listeners and write through
`set_property`, which in turn enqueues observer notifications and
marks reactive-bound Signals dirty. The reactive drain (above)
then propagates those changes within the same outermost-frame
boundary. This is the path that makes "click → handler runs
`count.set(...)` → bound Text re-renders" fire end-to-end before
control returns to the host's message loop.

The handler evaluator and the binding evaluator share the same
`HandlerExpr` AST and `EvalContext` trait. The binding side uses
a `BindingEvalContext` variant: reads route through `Signal::get()`
(dependency-tracking), writes are rejected at evaluation time
(binding bodies are pure-read; mutation only happens through the
binding's bound write target).

#### 6.7.6 Effect lifetime

Effects are owned by the widget that hosts the binding.

```
WidgetNode
  │
  └── bindings: Vec<EffectHandle>     ← pub(crate); empty for unbound widgets
        │
        └── on dispose:
              walk back-edge map → remove EffectId from every
              Signal's dependent set → drop the closure
```

Disposal is structural: every existing teardown path —
`wasamo_widget_destroy` subtree sweep, `wasamo_window_destroy`
whole-tree drop, `remove_child` + drop, `replace_child` of an
attached subtree — already walks the subtree and drops each
`Box<WidgetNode>`; binding disposal piggy-backs on that walk by
running ahead of the existing signal-handler / observer
unregistration so an Effect that captured widget references
cannot fire against a half-torn-down widget.

Re-attach (M3 conditional binding rebuilds at a different
position) just creates fresh Effects on the new widgets; old
widgets' Effects dispose through the same path. No explicit hook.

#### 6.7.7 Binding registration API after M2

```rust
// String-baked path (M2): used for i32 / String targets, which the
// per-widget setter parses on the value-side of the call.
pub(crate) fn register_binding(
    target: BindingTarget,
    expr: HandlerExpr,
    registry: Rc<SignalRegistry>,
    write_fn: fn(WidgetId, PropertyKey, &str),
) -> EffectHandle;

// Bool-typed path (M3-Phase 1): selected by the IR
// loader when the target property's declared `IrType` is `Bool`.
pub(crate) fn register_bool_binding(
    target: BindingTarget,
    expr: HandlerExpr,
    registry: Rc<SignalRegistry>,
    write_fn: fn(WidgetId, PropertyKey, bool),
) -> EffectHandle;

pub(crate) struct SignalRegistry {
    i32s:    HashMap<String, Signal<i32>>,
    strings: HashMap<String, Signal<String>>,
    bools:   HashMap<String, Signal<bool>>,   // M3-Phase 1
}

pub(crate) enum BindingTarget {
    WidgetProperty { node: WidgetId, prop: PropertyKey },
    // M3-Phase 6 fills ConditionalSubtree (§6.7.9):
    ConditionalSubtree { parent: WidgetId, declared_member_index: usize },
    // M3+ adds ForLoopSubtree, …
}
```

`WidgetId` is type-erased (`*mut ()`) to keep `reactive.rs`
free of a circular dependency on `widget.rs`; the production
caller in `widget.rs` casts `*mut WidgetNode` at the call site.
The `write_fn` function-pointer parameter is the seam that lets
`reactive.rs` perform property writes without importing
`widget.rs` types — production callers pass
`widget::widget_write_property` (string-baked) or
`widget::widget_write_property_bool` (bool). An internal
`register_binding_with_writer(Box<dyn FnMut(String)>, …)` and its
bool sibling `register_bool_binding_with_writer(Box<dyn FnMut(bool)>, …)`
are the testable cores; pure-logic tests inject a recording writer.

**Per-type seam.** The `write_fn` parameter is typed
per scalar — string for M2 paths, `bool` for the M3-Phase 1 bool
path — and the **IR loader picks which evaluator/writer pair to
instantiate** based on the target property's declared `IrType`
returned by `resolve_prop_key` (which widens the catalog row
to `(PropertyKey, IrType)`). The reactive engine itself stays
type-agnostic: it receives a monomorphic writer closure with the
value type baked in, never branches on a runtime value tag. This is
the structural form of the F5 (`TypedValue`) deferral — see
[dsl_spec.md §8.12 F5 deferral](./dsl_spec.md#812-scope-out-post-m2)
and [M3-Phase 1 bool-scalar decisions](../process/milestone-3/phase-1/decisions/preamble.md).
When a typed-i32 binding writer becomes warranted (no current
catalog row needs it; `Button.style` / `Text.font` are enum i32s the
setter parses from a lowered ident), it lands as a third pair with
the same shape — additive, not by widening `write_fn` into a value
union.

`SignalRegistry` is the per-component Signal store installed by the
IR loader. M2 supports `i32` and `String` Signals; M3-Phase 1 adds
`bool`. Integer reads use `HandlerExpr::PropRead`, String reads use
`HandlerExpr::StrPropRead` (evaluating through
`BindingEvalContext::read_string_tracked`), and bool reads use
`HandlerExpr::BoolPropRead` (evaluating through
`BindingEvalContext::read_bool_tracked`). The broader pattern
survives: M3 binding shapes (Computed, conditional, for-loop) add
`BindingTarget` variants and may expand `SignalRegistry` further
without disturbing the per-type widget-write seam.

**M3 layout primitives and the writer seam.** None of the M3 layout
primitives (Box, WrapPanel, ScrollView, Grid, ZStack) extends this
per-type writer seam: each introduces no new bindable scalar type, so
the seam is neither widened into a value union nor given a new
evaluator/writer pair. ScrollView reuses the existing string-baked
`i32` reader path for `offset-y`; the rest are constant-only. The
per-kind runtime shape of each — IR node form, measure/arrange, Visual
sync, ABI impact — now lives in
[§6.8](#68-m3-layout-primitives-and-runtime-shape); this subsection
covers only the registration / per-type writer seam.

#### 6.7.8 Forward-compatibility and out-of-scope

The reactive architecture is shape-compatible with the M3 extensions it
defers (M3-Phase 6 fills the conditional seam — see §6.7.9):

- `Computed<T>` lands as a third layer between Signal and Effect;
  it inherits the M2 topological dirty-Effect walk. M3 still decides
  cycle policy, ordering ties, and fan-out interaction with
  `MUTATION_CAP`.
- Structural bindings add `BindingTarget` variants; subtree rebuilds
  Drop old Effects through the existing widget teardown path. M3-Phase
  6 realizes the conditional variant
  (`BindingTarget::ConditionalSubtree`, §6.7.9); for-loop /
  list-rendered remain deferred.
- Subtree-grain layout dirty (open question in
  [layout-engine note §3.4](notes/layout-engine.md)) is
  unaffected; the engine inherits the Phase 8 whole-window dirty path.
- `untrack` / explicit `engine.flush()` / multi-threaded Signal
  access are post-M2 and have no M2 driver.

The post-1.0 hot-reload work fits the same drain shape:
whole-graph teardown disposes every Effect via root drop; the
new graph's Effects re-run on first drain. No engine change is
required.

#### 6.7.9 Conditional rendering (M3-Phase 6)

M3-Phase 6 adds the first structural-rendering construct:
`if <bool> { <widget> }`, where a bound `bool` drives whether a subtree
is **present or absent** in the live tree — not merely shown or hidden.
It is the first member of a structural control-flow family (`else` /
`switch` / `for` follow in later phases), and is verified visually by
the lightbox overlay slice.

**Member-level structural IR.** Control flow is encoded as a
first-class **member**, not a widget. `IrNode.children` is
`Vec<IrMember>` rather than `Vec<IrNode>`:

```rust
enum IrMember {
    Widget(IrNode),
    ControlFlow(ControlFlowNode),
}
enum ControlFlowNode {
    If { branches: Vec<Branch> },   // Phase 6: exactly one Branch, no else
    // future: Switch { subject, arms }, For { binding, body }, …
}
struct Branch { condition: HandlerExpr, body: Vec<IrMember> }
```

The condition rides the existing `HandlerExpr` (a `BoolLit` or a
bool-typed `BoolPropRead`); `IrProp.value` stays strictly `IrLiteral`,
so no new scalar / literal type is added. Phase 6 constrains `branches`
to length 1 and `body` to exactly one `Widget(_)` member at lowering
and loader time — anything other than exactly one `Widget(_)` body
member (an empty body, more than one member, or a non-widget member
such as a textual `prop` / `binding` / `handler` line or a nested
`ControlFlow(_)`), or a second `Branch`, is `WASAMO_ERR_IR_MALFORMED` —
while the wider `Vec` shapes already exist
in the type for forward-compat (`else` adds a `Branch`; `switch` /
`for` add `ControlFlowNode` variants) with no future schema migration.
Re-typing `children` is the largest single change of the phase; the
no-`Default` construction discipline surfaces every
construction/traversal site at compile time, so the change is
mechanical and exhaustive rather than a silent-omission hazard. Like
Grid's `Cell`, a control-flow member materialises **no `WidgetNode` and
no `Visual`** — the loader *interprets* it, emitting widgets for
`Widget(_)` members and a conditional binding for `ControlFlow(_)`.
Validation is **not** deferred with materialisation: the loader
recurses into the declared branch body at load time and runs the full
validate / name-resolution / bool-typed-condition check even when the
initial condition is `false` and the body is never built this run, so
an invalid absent-initial body is rejected at load rather than on the
first toggle to present. (Textual IR form in
[dsl_spec.md §8.5](./dsl_spec.md#85-widget-nodes-and-control-flow-members).)

**Present/absent mechanism (`BindingTarget::ConditionalSubtree`).** The
reserved `BindingTarget` variant (§6.7.7) is now filled. When the
loader encounters a control-flow member it captures the branch body as
a **builder** — the declared body plus a factory, with no entity or
Effect instantiated up front — and registers a bool Effect on the
condition Signal. On each evaluation the Effect mutates the parent
**only on a transition** (true→true and false→false are no-ops, so a
condition Effect re-firing for an unrelated dependency is safe):

- **false → true:** build a fresh entity subtree from the declared body
  and `insert_child` it at the materialised slot;
- **true → false:** `widget_destroy(remove_child(index))` — detach
  **and** destroy. `remove_child` alone only detaches the child Visual
  and returns the box; dropping that box severs the reactive graph
  (`EffectHandle::Drop`) but **not** the widget-pointer registry, so the
  explicit `widget_destroy` is required to avoid stale hit-test registry
  pointers (e.g. the lightbox `< > x` Buttons) lingering in the absent
  subtree.

Each successful transition marks the containing window layout-dirty via
the parent widget (§6.6). Structural presence can affect measurement,
placement, and ZStack visual order even when the transition did not
write a size-affecting property.

The conditional stores its stable **declared member index**; the
materialised insert/remove index is **recomputed at each mutation**
from the count of currently-live preceding members. A preceding
conditional going absent shifts every following sibling's live index,
so a cached index would mutate the wrong child. `insert_child` /
`replace_child` are also revised so the child Visual sibling order
matches the `children` Vec order after every structural mutation (the
prior primitive always inserted the Visual at the top — correct only
for a top-slot insertion, and mis-ordering a conditional re-inserted
between static siblings).

**Effect lifecycle: absent = destroyed, present = rebuilt fresh.** An
absent conditional subtree has **no live effects**; a present subtree's
effects are freshly created and run. There is no paused/reconnected
effect state and no state retention across absent→present in Phase 6 —
a subtree that goes absent and returns is a **fresh** subtree, and any
state inside it resets (correct for the stateless lightbox; an author
needing persistence keeps that state in a component-level `state`
outside the conditional). This is the re-attach behaviour §6.7.6
already describes, made normative. (Author-visible
semantics in
[dsl_spec.md §4.14](./dsl_spec.md#414-conditional-rendering-and-the-structural-rendering-model-m3-phase-6).)

**Drain contract: synchronous, same-drain initialisation.** The
M3-Phase 1 synchronous non-batched drain contract is preserved: a
condition write at `BATCH_DEPTH == 0` (e.g. inside a Button click
handler) drains before control returns, so the present/absent change
**and** the initial run of any freshly-inserted subtree Effects are
complete and observable when the toggling call returns. This rests on
one guarantee: registering the inserted subtree's bindings enqueues
their initial run into the current `drain_dirty_effects` loop (which
re-scans `DIRTY_EFFECTS` each iteration, §6.7.3), so bound properties
initialise before quiescence — no one-frame-stale window. The guarantee
holds up to the existing `MUTATION_CAP` (16); a subtree large enough to
exhaust the cap before quiescence trips the existing divergence guard
rather than rendering silently stale.

**Structural-mutation ordering: status-quo drain, quiescent layout
fixed by declared order.** Structural Effects ride the same topological
drain as property Effects, with no special structural-ordering
contract. Safety against use-after-free is the §6.7.6 disposal
invariant (binding disposal unregisters from every Signal's dependent
set ahead of teardown), so a captured-reference Effect cannot fire
against a half-torn-down widget regardless of order. The transient
inter-Effect drain order stays implementation-defined — as it already
was for property Effects — but the **quiescent child order is a
function of declared member order alone**: whichever sibling or
wrapper-descendant conditionals are present at quiescence appear among
the static siblings in declared document order, independent of
effect-evaluation order (and, for a conditional child of a ZStack, this
is what fixes its document-order z-order, §6.8.5). The M2-handoff
structural-mutation residuals (cycle detection, ordering ties, fan-out
× `MUTATION_CAP`) are carried forward — Phase 6 declines to freeze a
structural-transaction model before `for` / multiple conditionals
reveal the real requirements — rather than silently deferred.

### 6.8 M3 layout primitives and runtime shape

M3 adds five layout primitives beyond the M1 Rectangle / VStack /
HStack / Text / Button set: Box (Phase 2), WrapPanel (Phase 3),
ScrollView (Phase 4), Grid (Phase 5), and ZStack (Phase 6). Each is a
per-kind tag in the `wasamo-runtime` widget catalog with a pure-data
`measure` / `arrange` path in
[`wasamo-runtime/src/layout.rs`](../wasamo-runtime/src/layout.rs), so
the layout engine stays Win32/WinRT-free and is exercised by pure-logic
unit tests plus mock-free Windows integration tests through the live
Compositor ([AGENTS.md §Testing rules](../AGENTS.md#testing-rules)).
Three conventions are shared across the family: the **`1 WidgetNode = 1
Visual`** mapping holds for every primitive except ScrollView (which
adds one intermediate content Visual for the scroll translation, §6.5);
each container clips to its outer bounds on its **own** Visual via
`Visual.Clip = InsetClip{0,0,0,0}`; and none of them extends the
per-type writer seam (§6.7.7). The subsections below record each
primitive's IR node form, sizing/arrange contract, Visual sync, and ABI
impact.

#### 6.8.1 Box (M3-Phase 2)

**Box does not extend the per-type writer seam.**
The Box widget's two literal attributes — `aspect: <num>:<den>` and
`fill: #RRGGBB[AA]` — are **constant-only** in Phase 2.
The IR loader materialises `IrLiteral::Ratio` and
`IrLiteral::Color` into Box-internal domain types (`Ratio` / `Color`
on `WidgetData::Box`) directly, **not** as new `PropertyValue`
variants. No `evaluate_ratio_binding` / `evaluate_color_binding`
writer triples are added; no `register_ratio_binding` /
`register_color_binding` sibling is added; no `IrType::Ratio` /
`IrType::Color` is added to the type-suffix chain. F5 (`TypedValue`)
deferral is doubly protected: no new bindable type means no new
pressure on the per-type seam, and no widening of `write_fn` into a
value union. The first phase that needs reactive aspect or fill
opens the seam triple for that attribute at that point — Phase 2's
literal plumbing is forward-compatible and is extended, not revised.
See [M3-Phase 2 Box layout decisions](../process/milestone-3/phase-2/decisions/preamble.md)
and the
[dsl_spec.md §4.9 Box chapter](./dsl_spec.md#49-box-layout-primitive-m3-phase-2).

The same Phase 2 boundary applies to the C ABI: because `Ratio` and
`Color` do not enter `PropertyValue`, the exhaustive-match arms in
`wasamo-runtime/src/abi.rs` (`read_property_value` /
`write_property_value` / `property_value_to_owned`) are untouched in
Phase 2, and no `WASAMO_VALUE_RATIO` / `WASAMO_VALUE_COLOR` tag is
added to the C ABI's value union. See
[abi_spec.md](./abi_spec.md) — Phase 2 ships **no** ABI changes.

#### 6.8.2 WrapPanel (M3-Phase 3)

**WrapPanel does not extend the per-type writer seam either.** WrapPanel
is the first M3 layout primitive whose outer
cross-axis size depends on its children — a **two-stage measure-
arrange** in which the line breaker decides per-child line membership
from main-axis intrinsic measure and a cross-axis bound resolved
from `item-cross-size` (when set) or the parent's cross-axis
constraint (when unset). All three WrapPanel attributes —
`item-cross-size`, `item-spacing`, and `line-spacing` — are
**constant-only `i32`** in Phase 3.
The IR loader materialises them as fields on `WidgetData::WrapPanel`
directly; no new `PropertyValue` variant, no new `IrType`, no new
`IrLiteral` variant, no new evaluator/writer pair. Phase 3 reuses
the existing `i32` literal plumbing; a future bindable WrapPanel
attribute can reuse the M2 string-baked path that `IrType::I32`
properties currently dispatch to (`register_binding` +
`widget_write_property`), or open a typed-`i32` evaluator/writer
pair if that phase warrants it (the third per-type pair anticipated
in the *Per-type seam* paragraph above). F5 (`TypedValue`) deferral
is structurally unpressured by Phase 3.
See [M3-Phase 3 WrapPanel decisions](../process/milestone-3/phase-3/decisions/preamble.md)
and the
[dsl_spec.md §4.10 WrapPanel chapter](./dsl_spec.md#410-wrappanel-layout-primitive-m3-phase-3).

The same Phase 3 boundary applies to the C ABI: because the three
WrapPanel attributes do not enter `PropertyValue`, the exhaustive-
match arms in `wasamo-runtime/src/abi.rs` are untouched in Phase 3,
and no `WASAMO_VALUE_*` tag is added. Phase 3 also adds **no
new `LayoutError` variant** — the unbounded-main-axis branch is
one-line-flow (visible, not an error), and the unbounded-cross-axis-
with-aspect-child case fires Phase 2's existing
`LayoutError::BoxAspectUnboundedBoth` — so no
`WASAMO_LAYOUT_ERROR_*` extension lands in Phase 3 either. See
[abi_spec.md](./abi_spec.md) — Phase 3 ships **no** ABI changes.

The Phase 3 layout engine boundary remains Win32/WinRT-free: the
WrapPanel line breaker and arrange pass operate on pure data
(`wasamo-runtime/src/layout.rs`), composing structurally with the
Phase 2 Box measure-arrange. Mock-free Windows integration tests
exercise the full pipeline through the live Compositor (per
[AGENTS.md §Testing rules](../AGENTS.md#testing-rules)); the
algorithm-correctness evidence lives in pure-logic unit tests.

#### 6.8.3 ScrollView (M3-Phase 4)

**ScrollView reuses the generic IR node form and the existing i32
binding reader path, but deliberately avoids opening the general
typed-`i32` writer pair.** ScrollView appears as another
`widget_type: "ScrollView"` value on `IrNode`, with exactly one child
and a single `offset-y` property. No new `IrType`, `IrLiteral`, or
`PropertyValue` variant is introduced. Literal `offset-y` values use
the existing integer literal path; bound `offset-y: scroll_y` (bare
state identifier RHS per
[dsl_spec.md §4.3](./dsl_spec.md#43-property-binding) property-binding
semantics, with `state scroll_y: i32 = 0` declared at component scope
per
[dsl_spec.md §4.7](./dsl_spec.md#47-state-declarations-m2-surface-bool-added-in-m3-phase-1))
uses `HandlerExpr::PropRead` over `Signal<i32>` and the existing
string-baked `register_binding` / `widget_write_property` effect path.

The Phase 4-specific bridge is intentionally narrow: the binding effect
still writes a string-form value, and ScrollView's own per-widget
`set_property` arm parses that string into the `i32` `offset-y` field.
That parse / write bridge exists only for ScrollView's `offset-y`
surface. The general typed-`i32` evaluator / writer pair anticipated in
the *Per-type seam* paragraph remains deferred to M4 or later input,
scrollbar, writer-seam, or animation work.

The runtime materializes ScrollView as a per-kind widget data shape
with parent-supplied viewport sizing, vertical-only scroll semantics,
and one content child. The layout engine measures the child with the
viewport width and an unbounded vertical constraint, returns the
ScrollView outer size as the viewport size, clamps `offset-y` into
`[0, max(0, content_height - viewport_height)]`, and reports
`LayoutError::ScrollViewUnboundedAxis` when the scroll axis is
unbounded. That `LayoutError` variant is internal in Phase 4: no new C
ABI value tag or host-visible layout-error tag is added. The
algorithm remains a pure-data layout path in
`wasamo-runtime/src/layout.rs`; Windows-runtime evidence is reserved
for the Visual tree clip / offset integration and the Phase 3 R2
relative-offset closure.

#### 6.8.4 Grid (M3-Phase 5)

**Grid does not extend the per-type writer seam, but it extends the IR
node shape with a Grid-specific kind payload alongside the existing
`IrProp` machinery.** Grid appears as another `widget_type: "Grid"` value on `IrNode`, with
zero or more `Cell` children carrying placement / span / alignment
metadata. The two load-bearing structural choices are:

- **`TrackSize` domain type lives outside `IrProp`.** Grid's
  `columns:` and `rows:` track lists carry sequences of a
  Grid-specific `TrackSize` enum (`Fixed(i32)` plus
  `Star(u32)` with weight in `[1, 1024]`; an `Auto` variant is
  reserved for a future phase). The generic `IrProp.value` carrier
  is `IrLiteral` (`Int | Str | Ident | Bool | Ratio | Color`),
  which cannot hold a `Vec<TrackSize>` and would widen for every
  consumer if extended with a Grid-specific carrier. Phase 5
  resolves this by adding an optional **kind-specific payload
  field** on `IrNode` (`KindPayload::Grid { columns: Vec<TrackSize>,
  rows: Vec<TrackSize> }` for `widget_type: "Grid"`; `None` for
  every other kind). Grid's `IrNode.props` does not carry
  `columns:` / `rows:` entries; the narrow track-list parser path
  populates the kind payload directly at parse / lower time. This
  preserves the invariant that `IrProp.value` is strictly
  `IrLiteral` and confines Grid-specific carrier logic to one
  field; future Grid extensions (`auto`, `minmax`, named lines,
  bindable tracks) localise to `KindPayload::Grid` without
  pressuring `IrProp`.
- **`Cell` is an IR-only wrapper, not a runtime widget kind.**
  `Cell` appears in `wasamo-ir` as `widget_type: "Cell"` so the
  parser and IR loader recognise it, but it is **not** registered
  in the `wasamo-runtime` widget catalog. Grid's lowering reads
  its IR Cell subtrees directly to extract `(row, column,
  row-span, column-span, h-align, v-align)` per Cell, and arranges
  each Cell's single content child as Grid's effective layout
  child. The WidgetNode / Visual tree therefore contains one node
  for Grid plus one node per Cell's content widget; `Cell` itself
  does not materialise as a WidgetNode or Visual. `Cell` outside a
  `Grid` parent is rejected at `wasamoc check` and at runtime
  `validate()` (defense-in-depth per Phase 1 / Phase 2 T7 /
  Phase 3 T6 / Phase 4). Cell's placement / span /
  alignment attributes live in standard `IrProp` entries using
  existing `i32` and `Ident` literals — no new `IrLiteral` variant
  is added.

All Phase 5 Grid attributes are **constant-only**; no Grid or Cell
attribute is bindable in Phase 5. No new `IrType`, `IrLiteral`, or
`PropertyValue` variant is introduced, so the per-type writer seam
discussed above is not pressured. The C ABI surface is unchanged:
`Cell` placement and span are internal IR shape that does not
appear in `PropertyValue`, and the new `LayoutError` variant below
is host-internal. F5 (`TypedValue`) deferral is preserved.

The runtime materialises Grid as a per-kind widget data shape
(`WidgetData::Grid { columns, rows, cell_placements }`) holding the
declared track lists (`Vec<TrackSize>` per axis) and the per-Cell
placement metadata (`cell_placements`, parallel to the content
children). It does **not** cache resolved per-Cell rectangles:
`arrange_grid` re-derives each axis's track resolution and writes the
resolved offset / size directly onto each content child's
`LayoutNode` every layout pass (no arrange-result cache, unlike Phase
4 ScrollView's `applied_offset_y`). The layout engine resolves each
axis independently with
a fixed-first / weighted-star distribution over `f32` prefix
boundaries (no integer pixel snap), reports
`LayoutError::GridUnboundedStarAxis` when star tracks meet an
unbounded parent axis (consistent with Phase 4's
`LayoutError::ScrollViewUnboundedAxis` precedent), and reserves a
no-op demand-distribution slot before star distribution for a
future `auto` admission. Negative remaining space (fixed-track sum
exceeds parent bound) is **not** a fault — star tracks resolve to
`0` and the rightmost cells overflow, contained by the Grid
outer-bounds clip below. Placement / span / conflict invariants are
**reject-at-validate**, not clamp-at-arrange; the only layout-time
gate is the unbounded-star error. The
`LayoutError::GridUnboundedStarAxis` variant is internal in Phase
5: no new C ABI value tag or host-visible layout-error tag is
added. See [abi_spec.md](./abi_spec.md) — Phase 5 ships **no** ABI
changes.

Phase 5 does **not** extend the §6.5 WidgetNode / Visual layer
sync convention (unlike Phase 4 ScrollView, which introduced an
intermediate content Visual for the scroll-offset translation).
Grid uses the existing **1 WidgetNode = 1 Visual** convention:
Grid's own Visual carries the outer-bounds clip
(`Visual.Clip = InsetClip { 0, 0, 0, 0 }`); each Cell's content
widget Visual is a direct child of Grid's Visual through the normal
`sync_visuals()` path. Grid has no translation analog to
ScrollView's scroll offset, so the intermediate-Visual pattern is
not needed; admitting per-cell clipping later would be the right
place to revisit Cell-owned Visuals, not Phase 5.

The Phase 5 layout engine boundary remains Win32/WinRT-free: the
Grid track-resolution algorithm and arrange pass operate on pure
data (`wasamo-runtime/src/layout.rs`), composing structurally with
Phase 2 Box, Phase 3 WrapPanel, and Phase 4 ScrollView measure-
arrange. Mock-free Windows integration tests cover both the
Grid-rooted parent shape and the `VStack { Grid { ... } }`
production-root parent shape (Phase 4 T6 runtime-boundary
carry-forward); algorithm-correctness evidence lives in pure-logic
unit tests. See
[dsl_spec.md §4.12 Grid chapter](./dsl_spec.md#412-grid-layout-primitive-m3-phase-5).

#### 6.8.5 ZStack (M3-Phase 6)

**ZStack is a pure overlap container that rides the generic `IrNode`
machinery unchanged — no new IR vocabulary.** ZStack appears as another
`widget_type: "ZStack"` value on `IrNode` with `kind_payload: None`,
taking its children **directly** in document order like VStack / HStack
/ WrapPanel (no `Cell`-style wrapper; unlike Grid, overlap needs no
per-child structured placement carrier). No new `IrType`, `IrLiteral`,
or `PropertyValue` variant is introduced, so neither the per-type
writer seam nor the C ABI value union is touched. The runtime registers
`ZStack` as a layout-container widget kind parallel to WrapPanel; each
child is a real widget that materialises a `WidgetNode` and a `Visual`,
with the `1 WidgetNode = 1 Visual` convention intact.

**Sizing.** ZStack's default size constraint is **`Fill/Fill`** (like
Grid / ScrollView): on a bounded parent axis it takes the full parent
allocation; on a Shrink / unbounded axis its desired size is the
per-axis **max** of its children's measured desired sizes (the union).
A `Fill` child contributes **`0.0`** to that union
([layout.rs:440](../wasamo-runtime/src/layout.rs)) and fills its
allocated rect in *arrange* — it does not inflate the ZStack's measured
size. Consequently the lightbox's full-viewport scrim comes from the
**ZStack's own `Fill` default** taking the parent allocation (then the
scrim child filling that content rect), **not** from a `Fill` child
driving the union the way SwiftUI's flexible children do. ZStack
introduces **no new `LayoutError`** and no ZStack-specific `Fill`
special case. Owner-visible trade-off: with no author-facing
`width:`/`height:` surface in Phase 6, ZStack is overlay-first — an
intrinsic ("size to the largest child" on a bounded axis) ZStack is not
expressible until a future size-constraint surface.

**Per-child alignment.** Each child is measured against the ZStack
content rect and anchored within it; the default `h-align` / `v-align`
is **`center`** (a `Stretch` alignment or a `Fill` constraint expands
the child to the full content rect via the existing cross-axis rule).
All children share the same content rect — the defining property of the
overlap. The alignment is authored as ordinary child `IrProp`
ident-literals and carried at the layout layer as
**parent-owned metadata parallel to `children`**, mirroring Grid's
`cell_placements` ([layout.rs:224](../wasamo-runtime/src/layout.rs)) —
a lean per-child placement (h/v `Alignment` only, not Grid's
row/column/span), extracted by `construct_widget` so the arrange loop
zips `children[i]` with `placements[i]`. `h-align` / `v-align` are
admitted only on a **ZStack direct child** (and a Grid `Cell`); the
parent context consumes them as placement annotations before the
child's own unknown-prop check and excludes them from the child's prop
set, and they are rejected on any other parent — closing both the
"valid placement wrongly rejected" and "stray placement wrongly
accepted" failure modes.

**z-order and clip.** Paint order is document order — first child at the
bottom, last on top, no `z-index`. Static children ride the normal
document-order `sync_visuals()` path; structural insertion is fixed in
Phase 6 to honour the sibling index (§6.5 / §6.7.9), so a conditional
ZStack child re-inserted between static siblings keeps its document-order
slot rather than jumping to the top. ZStack does **not**
add an intermediate Visual (the negative ScrollView precedent is
deliberately not repeated): the outer-bounds clip lands on ZStack's own
Visual via `Visual.Clip = InsetClip{0,0,0,0}`, and each child Visual has
`Visual.Clip = null` (per-child clip out of scope, symmetric with
WrapPanel / ScrollView / Grid).

The C ABI surface is unchanged: ZStack adds no `PropertyValue` variant
and no new `LayoutError`, so [abi_spec.md](./abi_spec.md) ships **no**
ABI changes for Phase 6. See
[dsl_spec.md §4.13 ZStack chapter](./dsl_spec.md#413-zstack-layout-primitive-m3-phase-6).

---

## 7. Widget Implementation (Phase 4)

Full decision rationale: [`process/milestone-1/phase-4/decisions/preamble.md`](../process/milestone-1/phase-4/decisions/preamble.md)

### 7.1 New widget types

| Widget | Module | Description |
|---|---|---|
| `Text` | `wasamo/src/widget.rs` + `text.rs` | Unicode text label rendered via DirectWrite onto a `CompositionDrawingSurface` |
| `Button` | `wasamo/src/widget.rs` | Clickable control with background `SpriteVisual` + child text `SpriteVisual`; hover/press state via brush swap |

### 7.2 Text rendering pipeline

```
TextRenderer (created once per process)
  │
  ├── ID3D11Device (BGRA support)  →  IDXGIDevice  →  ID2D1Device
  │
  ├── ICompositorInterop::CreateGraphicsDevice(d2d_device)
  │     └── CompositionGraphicsDevice
  │
  └── IDWriteFactory (shared)

Text::new(text, style)
  │
  ├── IDWriteFactory::CreateTextLayout  →  measure natural (w, h)
  │     stored as Fixed(w) × Fixed(h) on WidgetNode
  │
  └── CompositionGraphicsDevice::CreateDrawingSurface(Size{w,h}, BGRA, Premultiplied)
        └── ICompositionDrawingSurfaceInterop::BeginDraw
              └── ID2D1DeviceContext::DrawTextLayout  →  EndDraw
                    └── CompositionSurfaceBrush → SpriteVisual
```

### 7.3 TypographyStyle type ramp

```rust
pub enum TypographyStyle { Caption, Body, Subtitle, Title }
```

| Value | Size | Weight | Font |
|---|---|---|---|
| `Caption` | 12 sp | Regular | Segoe UI Variable |
| `Body` | 14 sp | Regular | Segoe UI Variable |
| `Subtitle` | 20 sp | Semi-bold | Segoe UI Variable |
| `Title` | 28 sp | Semi-bold | Segoe UI Variable |

Maps to the WinUI 2 / WinApp SDK typography token set. Custom font descriptors deferred to M2.

### 7.4 Button structure

```
Button root: SpriteVisual (background brush)
  └── child: SpriteVisual (text label, offset by PAD_H/PAD_V)
```

State transitions animate the background brush color using `ColorKeyFrameAnimation` (Phase 5).
The `CompositionColorBrush` is retained on `ButtonData` and animated in place; no
new brush is created per transition. Duration values: 83 ms for entering a more-active state
(hover-in, press-down); 167 ms for returning to a less-active state (hover-out, press-up).
See §8 for details. `ButtonStyle::Accent` reads the system accent color via
`UISettings::GetColorValue(UIColorType::Accent)` at creation time.

### 7.5 `wnd_proc` ↔ `WindowState` linkage

`window::create()` stores `*mut WindowState` in `GWLP_USERDATA` after the `Box` is allocated.
`wnd_proc` reads it via `GetWindowLongPtrW` and calls the corresponding callback field:

| Message | Callback field | Effect |
|---|---|---|
| `WM_SIZE` | `resize_fn: Option<Box<dyn FnMut(f32, f32)>>` | Re-run layout with new client dimensions |
| `WM_KEYDOWN` | `key_down_fn: Option<Box<dyn FnMut(u16)>>` | Deliver virtual key code to host (Phase 5) |
| `WM_MOUSEMOVE` | `mouse_move_fn` | Update button hover state; arm `TrackMouseEvent` for leave |
| `WM_MOUSELEAVE` | `mouse_leave_fn` | Clear all button hover states |
| `WM_LBUTTONDOWN` | `mouse_down_fn` | Hit-test button tree; fire `clicked_fn` if hit |
| `WM_LBUTTONUP` | `mouse_up_fn` | Available for future press-release distinction |

All `unsafe` operations are confined to `window.rs` (`window::create()` + `wnd_proc`). The
callback fields themselves are safe Rust types.

### 7.6 Module additions

| File | Responsibility |
|---|---|
| `wasamo/src/text.rs` | `TextRenderer` + `TypographyStyle`; D3D11/D2D/DWrite device setup; surface draw |
| `wasamo/src/widget.rs` | Extended with `Text`, `Button`, `ButtonStyle`; hit-test and hover methods |
| `wasamo/src/window.rs` | `WindowState` extended with `GWLP_USERDATA`, event callback fields, mouse tracking |

---

## 8. Animation (Phase 5)

Full decision rationale: [`process/milestone-1/phase-5/decisions/preamble.md`](../process/milestone-1/phase-5/decisions/preamble.md)

### 8.1 Compositor-thread independence

The Windows Composition runtime (`Compositor`) drives all `KeyFrameAnimation` instances on
the **DWM compositor thread**, which is independent of the application's Win32 message loop.
This means:

- Animations continue to run while the app thread is blocked (e.g., during a long callback).
- Mica material continues to be composited by DWM regardless of app-thread state.
- The `DispatcherQueueController` created on the main thread (§5.2) initialises the
  `Compositor` and the animation subsystem, but the compositor executes on its own internal
  thread once `StartAnimation` is called.

### 8.2 Animation primitives used in M1

| Primitive | Used for | Loop behavior |
|---|---|---|
| `ColorKeyFrameAnimation` | Button hover/press state-transition color | One-shot (`IterationCount = 1`) |
| `Vector3KeyFrameAnimation` | Synthetic SpriteVisual offset (verification artifact) | Forever |

### 8.3 Button state-transition animation (permanent)

Button hover and press state transitions animate the background brush color in place using
`ColorKeyFrameAnimation`. The `CompositionColorBrush` is retained on `ButtonData` and
animated via `CompositionObject::StartAnimation("Color", ...)` on each state change; no new
brush is created per transition.

**Duration values (measured against WinUI Button on the same OS build):**

| Transition | Duration | Rationale |
|---|---|---|
| Normal → Hovered (hover-in) | 83 ms | Fluent "ControlFast" token; matches WinUI hover-in |
| Hovered → Normal (hover-out) | 167 ms | Fluent "ControlNormal" token; settles rather than snapping |
| Any → Pressed (press-down) | 83 ms | Fast response for direct user input |
| Pressed → Any (press-up) | 167 ms | Slower release gives tactile "settle" feel |

Easing: linear (default; no `CompositionEasingFunction` attached). WinUI Button uses a
near-linear ease-out; the visual difference is imperceptible at these durations. A
cubic-bezier easing can be substituted in a future revision without any API or ABI impact.

These values are **internal Button implementation details**. They are not exposed via the C
ABI or any public Rust surface and can be tuned without a version bump.

### 8.4 Property-change animation (deferred)

The default behavior when host code changes a widget property is **instant** — no animation
occurs. Opt-in property-change animation is the scope of M5 "Higher-level animation DSL" and
is not designed or implemented in M1.

This is the same convention used by SwiftUI, Jetpack Compose, Material Design, and CSS:
built-in widgets animate their own *state transitions* internally, but property changes
driven by host code are instant unless the host explicitly opts in to animation.

### 8.5 Verification synthetic visual

`examples/phase5_visual_check.rs` contains a 32×32 magenta `SpriteVisual` in the top-right
corner of the window. A looping `Vector3KeyFrameAnimation` (2-second period, `Forever`)
drives its `Offset` property. Pressing `[B]` blocks the app thread for 2 seconds; the
synthetic visual continues moving, confirming compositor-thread independence.

The synthetic visual is attached directly to `WindowState::root` (the public
`ContainerVisual` field) from the example. No new API surface was added to the runtime or
C ABI for this purpose.

---

## 9. Three-Layer Tree Model

| Layer | Owner | Contents |
|---|---|---|
| **DSL tree** | `wasamoc` | Parsed AST of `.ui` file declarations |
| **View tree** | `wasamo` runtime | Widget hierarchy with resolved properties |
| **Visual tree** | Windows.UI.Composition | `SpriteVisual` hierarchy, the actual render target |

In M1 the host language constructed the view tree directly through the
C ABI. The M2-onward DSL path instead materialises the view tree from
the loaded IR (`wasamoc` → textual IR → runtime loader), but there is
still no reconciler: the view tree is mutated in place — including the
structural insert/remove of conditional subtrees (§6.7.9) — not diffed
against a freshly computed tree.

**Component host surface (M3-Phase 6).** Loaded IR is rooted at an
`IrComponent`, not directly at the content widget. The component owns
three categories of data:

- `states`: per-component Signals;
- `host_props` / `host_bindings`: host-owned attributes for the
  containing Window surface;
- `root`: the single content-root widget.

The host surface is deliberately separate from the content root. Window
attributes such as `title`, `backdrop`, and `theme` are represented as
component `host_props`, never as `root.props`; dynamic host bindings are
represented by `host_bindings` in the textual IR surface but rejected in
M3-Phase 6. The runtime mirrors the compiler's Window host catalog,
resolves the static window title from `host_props`, rejects host
bindings, and rejects legacy IR that squats host attributes or bindings
on the content root. Future host/base modelling may replace this carrier
with a richer window descriptor, but it must preserve the invariant that
host-owned attributes do not become properties of the rendered content
root. Because this is an internal compiler/textual-IR/runtime-loader
shape, it adds no C ABI export; the host still calls `wasamo_load_ui`
with the emitted IR payload.

**Declared-tree / entity-tree separation (M3-Phase 6, nascent).** The
conditional construct (§6.7.9) introduces, in nascent form, a
distinction the three-layer table does not yet name: a **declared
tree** — the IR control-flow member and its body, stable across every
present/absent toggle — versus an **entity tree** — the runtime
`WidgetNode` / `Visual` / Effect subtree the body materialises, which
is destroyed when the condition is absent and rebuilt fresh when it
returns. Today the declared tree *is* the loaded IR and
the entity tree *is* the view tree, so the separation is observable
only at a conditional boundary; there is still no reconciler. It is
recorded here because it is the stable anchor a future identity layer
attaches to: keyed item identity and state retention (the Phase 7 `for`
driver, Flutter's Widget / Element / RenderObject split as reference)
land **between** the stable declared construct and the recreated entity
subtree, as an additive layer with no IR-shape change. The Phase 6
default — absent destroys, present rebuilds fresh — is the un-keyed
base case of that separation, and retention will arrive as an opt-in
(`key:` / retention marker) so the default never silently changes.

---

## 10. wasamoc (DSL Compiler) — Current Scope

M1 covered lexing, parsing, and syntax checking only. M2 added checked
lowering, textual IR emission, runtime IR loading, inline handler
evaluation, and reactive property bindings for the Foundation counter
surface. M3-Phase 1 extends that surface with `bool` state, bool
property bindings, and `Button.enabled`.
The full DSL grammar and AST type definitions are specified in [`docs/dsl_spec.md`](./dsl_spec.md).

### Processing pipeline

```
.ui source file
  │
  ▼  wasamoc/src/lexer.rs
token stream  (Keyword, Ident, IntLit, StringLit, …)
  │
  ▼  wasamoc/src/parser.rs
AST  (ComponentDef → Vec<Member> → …)
  │
  ▼  wasamoc/src/check.rs
diagnostics  (errors + warnings with file:line:col)
  │
  ▼  wasamoc check exit code
0 = success  |  1 = error
```

### Module layout (`wasamoc/src/`)

| File          | Responsibility                                              |
|---------------|-------------------------------------------------------------|
| `main.rs`     | CLI entry point; parses `wasamoc check <file>` arguments   |
| `lexer.rs`    | Converts `.ui` source text into a flat token stream        |
| `parser.rs`   | Recursive-descent parser; builds the AST from tokens       |
| `ast.rs`      | AST type definitions (`ComponentDef`, `Member`, `Expr`, …) |
| `check.rs`    | Post-parse validation: widget type registry, warnings      |
| `diagnostic.rs` | Error/warning formatting and span-based reporting        |

### Relation to the runtime

In M1, `wasamoc` and the `wasamo` runtime DLL were **decoupled**:
`wasamoc check` only validated syntax; it did not call into the
runtime or produce any output artifact consumed by the DLL.

The host language constructs the widget tree directly through the C ABI at startup.
The DSL file serves as the design source of truth; code generation that bridges the two
landed in M2 as textual IR plus runtime interpretation.

```
M1 data flow:

developer ──writes──▶ counter.ui ──wasamoc check──▶ OK / errors
                                                          (no artifact)

host app ──calls──▶ wasamo C ABI ──builds──▶ widget tree at runtime
                    (manually, by the developer)
```

M2 and later data flow is the IR pipeline described in §1: host builds
invoke `wasamoc build`, then call `wasamo_load_ui` with the emitted
`;wasamo-ir v0` payload at runtime.

---

## 11. Language Bindings (Phase 7)

Full decision rationale: [`process/milestone-1/phase-7/decisions/preamble.md`](../process/milestone-1/phase-7/decisions/preamble.md)

### 11.1 Binding overview

| Binding | Path | Status |
|---|---|---|
| C | `bindings/c/` | Header (`wasamo.h`) + CMake template; **no wrapper needed** — host `#include`s directly |
| Rust (raw FFI) | `bindings/rust-sys/` | `wasamo-sys` crate; `extern "C"` declarations; not for direct host use |
| Rust (safe) | `bindings/rust/` | `wasamo` crate; idiomatic API; **public Rust API** |
| Zig | `bindings/zig/` | Hand-written extern block + idiomatic wrappers; `wasamo.experimental` namespace |

### 11.2 Why Rust uses a sys + safe pair

M1's acceptance criterion is "C ABI verified in three languages". Routing
Rust through the `wasamo-runtime` rlib (which bypasses FFI entirely) would
be a hollow check. `wasamo-sys` crosses the actual C ABI boundary; `wasamo`
(the safe wrapper) builds on top of it.

### 11.3 Why `@cImport` was not used for Zig

`@cImport` parses a C header at compile time. `wasamo.h` uses
`__declspec(dllimport)` / `WASAMO_API` macros that complicate header
parsing on Windows. A hand-written `extern` block is more predictable
and explicit; it mirrors exactly what `wasamo-sys` does in Rust.

### 11.4 cdylib-shim split (M2-Phase 1)

**History.** `wasamo-runtime` originally used
`crate-type = ["cdylib", "rlib"]`. Both it (`[lib].name = "wasamo"`)
and the `wasamo` safe wrapper produced `libwasamo.rlib`. cargo#6313
surfaced as a compile error: cargo resolved `counter-rust`'s `wasamo`
dep to the runtime rlib instead of the safe wrapper. The M1 workaround
was to remove the `rlib` crate-type and delete the Phase 2-5
visual-check examples (which needed internal Rust API reachable only
through the rlib). Source is preserved in git history.

**M2-Phase 1 resolution (structural).** The collision class is
eliminated by construction:

- `wasamo-runtime` is now **rlib-only** (`[lib].name = "wasamo_runtime"`
  → `libwasamo_runtime.rlib`). No filename overlap with the safe
  wrapper's `libwasamo.rlib`.
- `wasamo-dll` is a new **cdylib-only** shim crate
  (`[lib].name = "wasamo"` → `wasamo.dll` + `wasamo.dll.lib`).
  `build.rs` uses MSVC `/WHOLEARCHIVE` to force all
  `#[no_mangle] pub extern "C"` symbols from `wasamo-runtime` into the
  cdylib output. New ABI symbols in `wasamo-runtime` appear in
  `wasamo.dll` automatically, with no per-symbol maintenance.
- `bindings/rust-sys/Cargo.toml` carries a `[dependencies]` entry on
  `wasamo-dll` to create a cargo build-order edge. Without it, cargo
  could parallelise `counter-rust`'s link step ahead of the cdylib
  build, reproducing `LNK1181`. The `warning: no linkable target`
  (cargo#6313) that this edge causes is accepted as a known wart; see
  [`docs/notes/workspace-layout.md`](notes/workspace-layout.md) and
  [`DD-M2-P1-006`](../process/milestone-2/phase-1/decisions/dd-m2-p1-006-build-order-edge-between-cdylib-shim-and-final-binaries.md).

Full rationale: [`process/milestone-2/phase-1/decisions/preamble.md`](../process/milestone-2/phase-1/decisions/preamble.md).
Phase 2-5 examples can be re-introduced under a `wasamo-poc` workspace
(experimental branch `exp/m2-p1-poc-examples`; not merged to main).

### 11.5 Experimental module convention

Every binding exposes `WASAMO_EXPERIMENTAL`-marked symbols in a clearly
separated namespace:

| Language | Namespace |
|---|---|
| Rust | `wasamo::experimental` submodule |
| Zig | `wasamo.experimental` (pub const struct) |
| C | `WASAMO_EXPERIMENTAL` macro annotates each symbol inline |

### 11.6 Smoke test pattern

Each binding includes a link-resolution smoke test that forces the linker
to resolve every declared ABI symbol without calling into the runtime.
See `CONTRIBUTING.md` §5 for the pattern and the three reference
implementations.

---

<a id="open-questions-1"></a>

## 12. Open Questions (to be resolved in later phases)

The following are intentionally left open at this draft stage.

| Question | Resolution phase | Status |
|---|---|---|
| DPI scaling localization: whether the layout engine should operate in physical pixels and implications for DirectWrite hinting | M2+ | Open |
| AccessKit / UIA sync: when and how layout results are propagated to the accessibility tree, and the performance impact | M4 | Open (re-scoped from M2 to M4 alongside the M2-as-foundation redefinition; see [process/milestone-2/plan.md](../process/milestone-2/plan.md) Out-of-scope) |
| Async measure: how to handle widgets whose size is unknown at measure time (e.g. image load pending) | M2+ | Open |
| Cache invalidation granularity: strategy for detecting local property changes and recomputing only affected subtrees | M2+ | Open |
| Custom layout extensibility: approach to layouts beyond built-in primitives — host-language callbacks, data-driven IR injection, or other | M2+ | Open |
