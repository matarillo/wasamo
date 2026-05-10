# Wasamo Architecture

**Status:** M1 complete (Phases 0-8); M2-Phases 1-5 complete; M2-Phase 6 in progress — acceptance A1/A2 discharged via DD-M2-P6-008 (counter examples migration); Phase-end gate pending (DD-M2-P6-010/011/012 pre-doc re-execution, upstream-document bundle, Phase 6 CHANGELOG entry)

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
│   ├── Cargo.toml              ← [lib].name = "wasamo" (DD-M2-P1-002); crate-type = ["cdylib"]
│   ├── build.rs                ← /WHOLEARCHIVE:libwasamo_runtime.rlib (DD-M2-P1-005)
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
| `wasamo-dll` | `cdylib` | `wasamo.dll` + `wasamo.dll.lib` | Cdylib shim (M2-Phase 1). Depends on `wasamo-runtime`; re-exports all C ABI symbols via `/WHOLEARCHIVE` (DD-M2-P1-005). `[lib].name = "wasamo"` — see note below. |
| `wasamoc` | `bin` | `wasamoc.exe` | `.ui` file parser and checker CLI. |
| `wasamo-sys` (at `bindings/rust-sys/`) | `lib` | Raw FFI crate | `extern "C"` declarations matching `wasamo.h`; `build.rs` links `wasamo.dll.lib` via `dylib:+verbatim`. |
| `wasamo` (at `bindings/rust/`) | `lib` | Safe Rust wrapper | Idiomatic Rust over `wasamo-sys`: `Runtime`/`Window`/`Widget`/`Value`/`Error`; `wasamo::experimental` for the M1 experimental layer. **This** is the supported public Rust API. |
| `examples/counter` *(Phase 8)* | `bin` | `counter.exe` | Sample app via the safe `wasamo` wrapper. |
| `bindings/zig/` | Zig package | link-time artifact | Zig binding: hand-written extern block + idiomatic wrappers. `wasamo.experimental` namespace mirrors the M1 experimental layer. |

`wasamo-dll` sets `[lib].name = "wasamo"` (not the cargo-conventional
`wasamo_dll`). This deviation is deliberate: `wasamo.dll` is the public
C ABI artifact name fixed by DD-P6-007; changing it would break all
downstream consumers. The deviation is confined to the shim crate and
documented in `wasamo-dll/Cargo.toml` — see
[DD-M2-P1-002](./decisions/m2-phase-1-cdylib-shim.md#dd-m2-p1-002--naming-of-the-rlib-crate-and-the-shim-crate).

### Inter-crate dependencies

```
wasamoc
  └── (future) wasamo-ast crate  ← to be split in M2; internal to wasamoc in M1

wasamo-dll  (cdylib shim; produces wasamo.dll)
  └── wasamo-runtime  (rlib; all C ABI symbol definitions)

bindings/rust  (safe wrapper, crate name: wasamo)
  └── wasamo-sys (raw FFI)
        ├── wasamo-dll  ← build-order edge (DD-M2-P1-006); no Rust link
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
`cargo build -p wasamoc` must precede the host build. See `CLAUDE.md` §
"Build ordering requirements" for the operational rule.

**This pipeline is provisional.** The current shape — three independent
build systems each invoking `wasamoc` — is acceptable while M2 has a single
DSL example, but does not generalize:

- **Hot reload** (post-1.0; `wasamoc`-output-format ADR /
  [DD-M2-P2-001](./decisions/m2-phase-2-wasamoc-output-format.md))
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

The full ABI specification is `docs/abi_spec.md` (Accepted, 2026-04-30).
No ABI stability guarantee is made for M1; M4 is when stability
commitments begin.

`abi_spec.md` is structured in **two layers**:

- **Stable core** — runtime lifecycle, window + event loop, property
  get/set, property change observers, signal connect/disconnect, and
  **tree mutation** (M2-Phase 4: append / insert / remove / replace /
  child_count / widget_destroy — `abi_spec.md §4.6`).
  Written as a candidate for the M4 ABI freeze.
  The stable core covers **six areas** as of M2-Phase 4 (DD-P6-001
  defined the initial five-area minimum; §4.6 tree mutation is the
  sixth area added by DD-M2-P4-001).
- **M1 experimental** — all-at-once widget constructors
  (`wasamo_text_create`, `wasamo_button_create`, `wasamo_vstack_create`,
  `wasamo_hstack_create`), `wasamo_window_set_root`, and the typed
  `wasamo_button_set_clicked` convenience. Required because M1 `wasamoc`
  is parser-only and the host must construct the widget tree by hand.
  Marked `WASAMO_EXPERIMENTAL` in both header and spec; not subject to
  M4 stability. Constructor promotion to stable core deferred to M3
  (DD-M2-P4-001).

The Phase 6 ADR explicitly **does not decide** (a) where DSL inline handler
bodies (`clicked => { … }`) will execute — host-side vs runtime-side; or
(b) `wasamoc`'s M2 output format — host-language codegen vs IR + runtime
interpretation. The stable core is sized so it survives either resolution.
These remain open in §11.

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

Full decision rationale: [`docs/decisions/phase-2-runtime-foundation.md`](./decisions/phase-2-runtime-foundation.md)

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

### 5.3 Decisions summary

| Decision | Chosen | See |
|---|---|---|
| DD-P2-001: `DispatcherQueueController` thread model | `DQTYPE_THREAD_CURRENT` — main thread; single-threaded, no synchronization needed | [ADR](./decisions/phase-2-runtime-foundation.md#dd-p2-001) |
| DD-P2-001b: COM apartment type | `DQTAT_COM_STA` — standard STA; Win32 desktop convention, matches Windows App SDK direction | [ADR](./decisions/phase-2-runtime-foundation.md#dd-p2-001b) |
| DD-P2-002: Global state management | Two-layer split: process-wide `Runtime` singleton (`Compositor` + `DispatcherQueueController`) + per-window `WindowState` handle (`HWND` + `DesktopWindowTarget` + root `ContainerVisual`) | [ADR](./decisions/phase-2-runtime-foundation.md#dd-p2-002) |
| DD-P2-003: Mica backdrop | `DwmSetWindowAttribute` direct (Win11 21H2+); solid color fallback on Win10; root ContainerVisual is transparent | [ADR](./decisions/phase-2-runtime-foundation.md#dd-p2-003) |

### 5.4 `windows` crate feature additions for Phase 2

```toml
"System",              # Windows::System::DispatcherQueueController
"Win32_Graphics_Dwm",  # DwmSetWindowAttribute, DwmExtendFrameIntoClientArea, DWMWA_* constants
```

`Win32_System_WinRT` (already present) provides `CreateDispatcherQueueController`,
`DispatcherQueueOptions`, `DQTYPE_THREAD_CURRENT`, and `DQTAT_COM_STA`.
(`"System_DispatcherQueue"` does not exist in windows 0.58 — types live directly in the `System` module.)

---

## 6. Layout Engine (Phase 3)

Full decision rationale: [`docs/decisions/phase-3-layout-engine.md`](./decisions/phase-3-layout-engine.md)

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

The `LayoutNode` tree is rebuilt on each layout pass (O(n)).
No persistent layout cache exists in M1.

### 6.6 Decisions summary

| Decision | Chosen | See |
|---|---|---|
| DD-P3-001: Layout algorithm | Custom two-pass measure/arrange; Taffy deferred to M2 | [ADR](./decisions/phase-3-layout-engine.md#dd-p3-001) |
| DD-P3-002: Node ownership | Engine owns; host holds `WasamoWidget*` opaque handles | [ADR](./decisions/phase-3-layout-engine.md#dd-p3-002) |
| DD-P3-003: Size model | `Fixed / Fill / Shrink` (`Fill` returns 0.0 in measure, resolved in arrange) | [ADR](./decisions/phase-3-layout-engine.md#dd-p3-003) |
| DD-P3-004: Cross-axis alignment | `Leading / Center / Trailing / Stretch` (Stretch default) | [ADR](./decisions/phase-3-layout-engine.md#dd-p3-004) |
| DD-P3-005: Error handling | API errors strict (`Result`); degenerate layout clamps to 0.0 | [ADR](./decisions/phase-3-layout-engine.md#dd-p3-005) |

### 6.7 Layout invalidation on property change (Phase 8, DD-P8-002)

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

Window registration lifecycle:
- `window::create` calls `emit::register_window` after `Box<WindowState>` is
  heap-allocated (pointer is stable).
- `wasamo_window_destroy` calls `emit::unregister_window` before the box
  is dropped.

### 6.8 Reactive engine (M2-Phase 5)

Full decision rationale: [`docs/decisions/m2-phase-5-reactive-engine.md`](./decisions/m2-phase-5-reactive-engine.md).
Architectural-family hypothesis (tree-with-bindings, working
hypothesis only): [`docs/notes/architectural-family.md`](./notes/architectural-family.md).

The reactive engine is the M2 thesis-validation surface for
acceptance A2 — `count++` in a host handler updates a bound
`Text` label without any host-side `wasamo_set_property` call. It
sits entirely inside `wasamo-runtime` and is `pub(crate)`; no C
ABI symbol is added (DD-M2-P4-004 = A).

#### 6.8.1 Module placement

| Module | Responsibility |
|---|---|
| `wasamo-runtime/src/reactive.rs` | `Signal<T>` / `EffectHandle` / dependency graph / `with_batched_writes` / dirty-set drain / `BindingEvalContext` / `register_binding` / `BindingTarget` |
| `wasamo-runtime/src/handler.rs` | `HandlerExpr` AST + `EvalContext` trait + `evaluate()` (Phase 3); reused by the binding evaluator in read-only mode |
| `wasamo-runtime/src/widget.rs` | `WidgetNode.bindings: Vec<EffectHandle>`; binding disposal during the Phase 4 `widget_destroy` sweep |
| `wasamo-runtime/src/emit.rs` | `drain_if_outermost` integrates the reactive drain between observer drain and layout drain |

Pure-logic surfaces — Signal storage, dependency tracker,
dirty-set drain, evaluator wiring — are unit-tested with
side-effect-logger Effect closures (DD-M2-P5-006 = A); no
test-only mirror of `WidgetNode` is introduced.

#### 6.8.2 Two-layer primitive (DD-M2-P5-001 = B, DD-M2-P5-002 = B)

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
`untrack` as a public escape hatch (DD-M2-P5-002 Out of scope).

#### 6.8.3 Drain ordering inside `drain_if_outermost`

The reactive dispatch is **deferred**, not synchronous (DD-M2-P5-004
= B). `Signal::set()` writes the new value, marks dependent
Effects in a thread-local dirty-set, and returns. Re-evaluation
runs at the outermost-frame boundary — the same boundary
DD-P6-003 already uses for queued observer notifications.

The drain itself is a three-phase + terminal transaction
([DD-M2-P6-001](./decisions/m2-phase-6-ui-lowering.md#dd-m2-p6-001--drain-transaction-semantics) =
Option D, supersedes DD-M2-P5-004's three-stage `observer → reactive
→ layout` framing). Phase 1 unifies signal-handler firing and
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
  `wasamo_post_event` API (DD-M2-P6-001 Option F, scheduled for
  M3). Observer callbacks never write runtime state directly.

`with_batched_writes(f)` (Phase 4 skeleton, body filled in
Phase 5) increments a thread-local depth counter; the per-call
drain at the end of each `wasamo_*` entry is suppressed while
depth > 0. On outermost-frame exit, a single drain processes the
accumulated dirty-set. The iteration cap is a small constant
(16 in current implementation, named `MUTATION_CAP` per
DD-M2-P6-001) — enough headroom for legitimate multi-pass
cascades, low enough to surface a divergent binding before it
exhausts CPU. Cap exhaustion is fatal: the runtime transitions
to a `Diverged` terminal state, Phase 2 and Phase 3 are skipped
for that frame, and every subsequent ABI call other than
`wasamo_runtime_destroy` returns `WASAMO_ERR_REACTIVE_DIVERGED`
(see DD-M2-P6-001 §"Divergence semantics").

#### 6.8.4 Runtime safety guard placement

Guard placement is a global runtime invariant
([DD-M2-P6-012](./decisions/m2-phase-7-reactive-foundation.md#dd-m2-p6-012--re-entrancy-and-safety-guard-placement-principle)
= Option C). Re-entrancy and lifecycle guards are enforced with
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
reopens DD-M2-P6-012, most likely because typed guard tokens become
worth their API cost.

#### 6.8.5 Signal-dispatch ordering (signal-side runtime contract)

Independent of the reactive drain, when a `WasamoSignal` fires
through `signal_emit`:

```
signal_emit(widget, signal_id, payload)
  │
  ├─ 1. Inline handler         (DD-M2-P3-002 = B; runtime-side
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

#### 6.8.6 Effect lifetime (DD-M2-P5-003 = A)

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

#### 6.8.7 Phase 6 binding registration API (DD-M2-P5-005 = A)

```rust
pub(crate) fn register_binding(
    target: BindingTarget,
    expr: HandlerExpr,
    write_fn: fn(WidgetId, PropertyKey, &str),
    properties: Rc<HashMap<String, Signal<i32>>>,
) -> EffectHandle;

pub(crate) enum BindingTarget {
    WidgetProperty { node: WidgetId, prop: PropertyKey },
    // M3+ adds ConditionalSubtree, ForLoopSubtree, …
}
```

`WidgetId` is type-erased (`*mut ()`) to keep `reactive.rs`
free of a circular dependency on `widget.rs`; the production
caller in `widget.rs` casts `*mut WidgetNode` at the call site.
The `write_fn` function-pointer parameter is the seam that lets
`reactive.rs` perform property writes without importing
`widget.rs` types — production callers pass
`widget::widget_write_property`. An internal
`register_binding_with_writer(Box<dyn FnMut(String)>, …)` is the
testable core; pure-logic tests inject a recording writer.

The `properties` parameter (provisional shape — likely revisited
when Phase 6 wires the IR loader) carries the Signal-backed
property store the binding expression evaluates against. The
broader pattern survives: M3 binding shapes (Computed,
conditional, for-loop) add `BindingTarget` variants and reuse
the same `HandlerExpr` AST without disturbing `register_binding`'s
signature.

#### 6.8.8 Forward-compatibility and out-of-scope

The Phase 5 architecture is shape-compatible with the M3
extensions it defers:

- `Computed<T>` lands as a third layer between Signal and Effect;
  the drain loop gains a pre-Effect topological pass before
  dirty Effects re-run.
- Structural bindings (conditional / for-loop / list-rendered)
  add `BindingTarget` variants; subtree rebuilds Drop old
  Effects through the existing widget teardown path.
- Subtree-grain layout dirty (open question in
  [layout-engine note §3.4](./notes/layout-engine.md)) is
  unaffected by Phase 5; the engine inherits DD-P8-002's
  whole-window dirty path.
- `untrack` / explicit `engine.flush()` / multi-threaded Signal
  access are post-M2 and have no M2 driver.

The post-1.0 hot-reload work fits the same drain shape:
whole-graph teardown disposes every Effect via root drop; the
new graph's Effects re-run on first drain. No engine change is
required.

---

## 7. Widget Implementation (Phase 4)

Full decision rationale: [`docs/decisions/phase-4-widget-implementation.md`](./decisions/phase-4-widget-implementation.md)

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

State transitions animate the background brush color using `ColorKeyFrameAnimation` (Phase 5,
DD-P5-005). The `CompositionColorBrush` is retained on `ButtonData` and animated in place; no
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

### 7.7 Decisions summary

| Decision | Chosen | See |
|---|---|---|
| DD-P4-001: Text rendering pipeline | `ICompositionDrawingSurface` + D2D + DirectWrite | [ADR](./decisions/phase-4-widget-implementation.md#dd-p4-001) |
| DD-P4-002: Font property model | 4-value `TypographyStyle` enum (Caption / Body / Subtitle / Title) | [ADR](./decisions/phase-4-widget-implementation.md#dd-p4-002) |
| DD-P4-003: Text natural size | Measured at creation/update; cached as `Fixed` on `WidgetNode` | [ADR](./decisions/phase-4-widget-implementation.md#dd-p4-003) |
| DD-P4-004: Button visual structure | Root `SpriteVisual` + child text `SpriteVisual`; color animated via `ColorKeyFrameAnimation` (Phase 5) | [ADR](./decisions/phase-4-widget-implementation.md#dd-p4-004) |
| DD-P4-005: `wnd_proc` linkage | `GWLP_USERDATA` + event callbacks on `WindowState`; unsafe confined to `window.rs` | [ADR](./decisions/phase-4-widget-implementation.md#dd-p4-005) |
| DD-P4-006: Button clicked callback | `Box<dyn Fn()>` internally; C ABI adapter deferred to Phase 6 | [ADR](./decisions/phase-4-widget-implementation.md#dd-p4-006) |

---

## 8. Animation (Phase 5)

Full decision rationale: [`docs/decisions/phase-5-compositor-independence-check.md`](./decisions/phase-5-compositor-independence-check.md)

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

### 8.3 Button state-transition animation (permanent — DD-P5-005)

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

### 8.4 Property-change animation (deferred — DD-V-001)

The default behavior when host code changes a widget property is **instant** — no animation
occurs. Opt-in property-change animation is the scope of M5 "Higher-level animation DSL" and
is not designed or implemented in M1.

This is the same convention used by SwiftUI, Jetpack Compose, Material Design, and CSS:
built-in widgets animate their own *state transitions* internally, but property changes
driven by host code are instant unless the host explicitly opts in to animation.

### 8.5 Verification synthetic visual (DD-P5-006)

`examples/phase5_visual_check.rs` contains a 32×32 magenta `SpriteVisual` in the top-right
corner of the window. A looping `Vector3KeyFrameAnimation` (2-second period, `Forever`)
drives its `Offset` property. Pressing `[B]` blocks the app thread for 2 seconds; the
synthetic visual continues moving, confirming compositor-thread independence.

The synthetic visual is attached directly to `WindowState::root` (the public
`ContainerVisual` field) from the example. No new API surface was added to the runtime or
C ABI for this purpose.

### 8.6 Decisions summary

| Decision | Chosen | See |
|---|---|---|
| DD-P5-004: Verification approach | Widget-internal state animation + continuous synthetic visual (Option D) | [ADR](./decisions/phase-5-compositor-independence-check.md#dd-p5-004) |
| DD-P5-005: Button state animation | `ColorKeyFrameAnimation` on retained brush; 83/167 ms durations | [ADR](./decisions/phase-5-compositor-independence-check.md#dd-p5-005) |
| DD-P5-006: Synthetic visual | `SpriteVisual` + `Vector3KeyFrameAnimation` in example only; no new runtime API | [ADR](./decisions/phase-5-compositor-independence-check.md#dd-p5-006) |

---

## 9. Three-Layer Tree Model

| Layer | Owner | Contents |
|---|---|---|
| **DSL tree** | `wasamoc` | Parsed AST of `.ui` file declarations |
| **View tree** | `wasamo` runtime | Widget hierarchy with resolved properties |
| **Visual tree** | Windows.UI.Composition | `SpriteVisual` hierarchy, the actual render target |

In M1 there is no reconciler. The host language constructs the view tree directly through the C ABI.

---

## 10. wasamoc (DSL Compiler) — M1 Scope

M1 covers lexing, parsing, and syntax checking only.
Code generation (conversion to runtime calls, binding generation) is M2 scope.
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

### Relation to the runtime (M1)

In M1, `wasamoc` and the `wasamo` runtime DLL are **decoupled**:
`wasamoc check` only validates syntax; it does not call into the runtime or produce any
output artifact consumed by the DLL.

The host language constructs the widget tree directly through the C ABI at startup.
The DSL file serves as the design source of truth; code generation that bridges the two
is M2 scope.

```
M1 data flow:

developer ──writes──▶ counter.ui ──wasamoc check──▶ OK / errors
                                                          (no artifact)

host app ──calls──▶ wasamo C ABI ──builds──▶ widget tree at runtime
                    (manually, by the developer)
```

---

## 11. Language Bindings (Phase 7)

Full decision rationale: [`docs/decisions/phase-7-language-bindings.md`](./decisions/phase-7-language-bindings.md)

### 11.1 Binding overview

| Binding | Path | Status |
|---|---|---|
| C | `bindings/c/` | Header (`wasamo.h`) + CMake template; **no wrapper needed** — host `#include`s directly |
| Rust (raw FFI) | `bindings/rust-sys/` | `wasamo-sys` crate; `extern "C"` declarations; not for direct host use |
| Rust (safe) | `bindings/rust/` | `wasamo` crate; idiomatic API; **public Rust API** |
| Zig | `bindings/zig/` | Hand-written extern block + idiomatic wrappers; `wasamo.experimental` namespace |

### 11.2 Why Rust uses a sys + safe pair (DD-P7-001)

M1's acceptance criterion is "C ABI verified in three languages". Routing
Rust through the `wasamo-runtime` rlib (which bypasses FFI entirely) would
be a hollow check. `wasamo-sys` crosses the actual C ABI boundary; `wasamo`
(the safe wrapper) builds on top of it.

### 11.3 Why `@cImport` was not used for Zig (DD-P7-005)

`@cImport` parses a C header at compile time. `wasamo.h` uses
`__declspec(dllimport)` / `WASAMO_API` macros that complicate header
parsing on Windows. A hand-written `extern` block is more predictable
and explicit; it mirrors exactly what `wasamo-sys` does in Rust.

### 11.4 cdylib-shim split (M2-Phase 1, DD-M2-P1-001..006)

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
  [`docs/notes/cdylib-shim-build-graph.md`](./notes/cdylib-shim-build-graph.md).

Full rationale: [`docs/decisions/m2-phase-1-cdylib-shim.md`](./decisions/m2-phase-1-cdylib-shim.md).
Phase 2-5 examples can be re-introduced under a `wasamo-poc` workspace
(experimental branch `exp/m2-p1-poc-examples`; not merged to main).

### 11.5 Experimental module convention (DD-P7-003)

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

## 12. Open Questions (to be resolved in later phases)

The following are intentionally left open at this draft stage.

| Question | Resolution phase | Status |
|---|---|---|
| `DispatcherQueueController` thread model | Phase 2 | Resolved → DD-P2-001 (§5.3) |
| Global state management strategy (singleton vs. handle-based) | Phase 2 | Resolved → DD-P2-002 (§5.4) |
| Mica backdrop support scope for M1 | Phase 2 | Resolved → DD-P2-003 (§5.5) |
| Layout algorithm (custom measure/arrange vs. Taffy) | Phase 3 | Resolved → DD-P3-001 (§6.6) |
| Layout node ownership model (opaque handle vs. direct Rust type exposure) | Phase 3 | Resolved → DD-P3-002 (§6.6) |
| Widget property API details | Phase 4 | Resolved → DD-P4-001 through DD-P4-006 (§7.7) |
| Full C ABI function signatures | Phase 6 | Resolved → `docs/abi_spec.md` (Accepted) + DD-P6-001..007 |
| Component-declared signal model: Slint-style (DSL inline body) vs XAML-style (host code-behind only) vs hybrid | Phase 6 pre-doc | Resolved → DD-P6-002 (string-keyed + `WasamoValue` payload) |
| Inline DSL handler execution location: host-side (callback) vs runtime-side (interpreted IR) | M2 | Resolved → DD-M2-P3-001 (runtime-side interpreter) |
| `wasamoc` M2 output format: host-language codegen vs IR + runtime interpretation | M2 | Resolved → DD-M2-P2-001 (textual IR + runtime interpreter) |
| DPI scaling localization: whether the layout engine should operate in physical pixels and implications for DirectWrite hinting | M2+ | Open |
| AccessKit / UIA sync: when and how layout results are propagated to the accessibility tree, and the performance impact | M4 | Open (re-scoped from M2 to M4 alongside the M2-as-foundation redefinition; see [docs/plans/m2-plan.md](./plans/m2-plan.md) Out-of-scope) |
| Async measure: how to handle widgets whose size is unknown at measure time (e.g. image load pending) | M2+ | Open |
| Cache invalidation granularity: strategy for detecting local property changes and recomputing only affected subtrees | M2+ | Open |
| Custom layout extensibility: approach to layouts beyond built-in primitives — host-language callbacks, data-driven IR injection, or other | M2+ | Open |
