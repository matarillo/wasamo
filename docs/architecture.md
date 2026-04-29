# Wasamo Architecture

**Document version:** 0.8
**Last updated:** 2026-04-29
**Status:** Phase 0, Phase 1, Phase 2, Phase 3, and Phase 4 complete

---

## 1. Cargo Workspace Layout

```
wasamo/                         ← workspace root
├── Cargo.toml                  ← workspace manifest
├── wasamo/                     ← runtime DLL crate
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── wasamoc/                    ← DSL compiler CLI crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── bindings/
│   └── rust/                   ← Rust bindings (M1 scope)
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
└── examples/
    └── counter/                ← Hello Counter (Phase 8)
```

### Crate responsibilities

| Crate | crate-type | Output | Responsibility |
|---|---|---|---|
| `wasamo` | `cdylib` | `wasamo.dll` + `wasamo.lib` | Runtime. Exposes the C ABI. |
| `wasamoc` | `bin` | `wasamoc.exe` | `.ui` file parser and checker CLI. |
| `bindings/rust` | `lib` | Rust library | Safe Rust wrapper over `wasamo.dll`. |
| `examples/counter` | `bin` | `counter.exe` | Sample app via Rust bindings. |

### Inter-crate dependencies

```
wasamoc
  └── (future) wasamo-ast crate  ← to be split in M2; internal to wasamoc in M1

bindings/rust
  └── wasamo.dll (dynamic link)

examples/counter
  └── bindings/rust
```

`wasamo` (the DLL) does not depend on any other Rust crate in this workspace. The C ABI boundary is the only coupling point.

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

- Every public function carries `WASAMO_EXPORT` (`__declspec(dllexport)`).
- Public types are opaque pointers (`WasamoWindow*`, `WasamoWidget*`) to preserve ABI stability.
- Error handling: M1 uses `int` return values (0 = success, negative = error code).
- Thread safety: M1 requires all calls to originate from the main thread only.

The full ABI specification will be finalized in Phase 6 as `docs/abi_spec.md`. No ABI stability guarantee is made for M1.

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

State transitions use brush replacement (instant, per DD-V-001). `ButtonStyle::Accent` reads the
system accent color via `UISettings::GetColorValue(UIColorType::Accent)` at creation time.

### 7.5 `wnd_proc` ↔ `WindowState` linkage

`window::create()` stores `*mut WindowState` in `GWLP_USERDATA` after the `Box` is allocated.
`wnd_proc` reads it via `GetWindowLongPtrW` and calls the corresponding callback field:

| Message | Callback field | Effect |
|---|---|---|
| `WM_SIZE` | `resize_fn: Option<Box<dyn FnMut(f32, f32)>>` | Re-run layout with new client dimensions |
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
| DD-P4-004: Button visual structure | Root `SpriteVisual` + child text `SpriteVisual`; instant brush swap | [ADR](./decisions/phase-4-widget-implementation.md#dd-p4-004) |
| DD-P4-005: `wnd_proc` linkage | `GWLP_USERDATA` + event callbacks on `WindowState`; unsafe confined to `window.rs` | [ADR](./decisions/phase-4-widget-implementation.md#dd-p4-005) |
| DD-P4-006: Button clicked callback | `Box<dyn Fn()>` internally; C ABI adapter deferred to Phase 6 | [ADR](./decisions/phase-4-widget-implementation.md#dd-p4-006) |

---

## 8. Three-Layer Tree Model

| Layer | Owner | Contents |
|---|---|---|
| **DSL tree** | `wasamoc` | Parsed AST of `.ui` file declarations |
| **View tree** | `wasamo` runtime | Widget hierarchy with resolved properties |
| **Visual tree** | Windows.UI.Composition | `SpriteVisual` hierarchy, the actual render target |

In M1 there is no reconciler. The host language constructs the view tree directly through the C ABI.

---

## 9. wasamoc (DSL Compiler) — M1 Scope

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

## 10. Open Questions (to be resolved in later phases)

The following are intentionally left open at this draft stage.

| Question | Resolution phase | Status |
|---|---|---|
| `DispatcherQueueController` thread model | Phase 2 | Resolved → DD-P2-001 (§5.3) |
| Global state management strategy (singleton vs. handle-based) | Phase 2 | Resolved → DD-P2-002 (§5.4) |
| Mica backdrop support scope for M1 | Phase 2 | Resolved → DD-P2-003 (§5.5) |
| Layout algorithm (custom measure/arrange vs. Taffy) | Phase 3 | Resolved → DD-P3-001 (§6.6) |
| Layout node ownership model (opaque handle vs. direct Rust type exposure) | Phase 3 | Resolved → DD-P3-002 (§6.6) |
| Widget property API details | Phase 4 | Resolved → DD-P4-001 through DD-P4-006 (§7.7) |
| Full C ABI function signatures | Phase 6 | Open |
| DPI scaling localization: whether the layout engine should operate in physical pixels and implications for DirectWrite hinting | M2+ | Open |
| AccessKit / UIA sync: when and how layout results are propagated to the accessibility tree, and the performance impact | M2 | Open |
| Async measure: how to handle widgets whose size is unknown at measure time (e.g. image load pending) | M2+ | Open |
| Cache invalidation granularity: strategy for detecting local property changes and recomputing only affected subtrees | M2+ | Open |
| Custom layout extensibility: approach to layouts beyond built-in primitives — host-language callbacks, data-driven IR injection, or other | M2+ | Open |

---

## Revision history

| Version | Date       | Notes                                                         |
|---------|------------|---------------------------------------------------------------|
| 0.1     | 2026-04-27 | Initial draft (Phase 0, pending owner agreement)              |
| 0.2     | 2026-04-27 | Phase 0 agreed; added §7 wasamoc detail (Phase 1, pending owner agreement) |
| 0.3     | 2026-04-27 | Phase 1 agreed; status updated to reflect completed implementation |
| 0.4     | 2026-04-28 | Phase 2 pre-doc: §5 expanded with thread model, global state, Mica scope, feature decisions (pending owner agreement) |
| 0.5     | 2026-04-28 | Phase 2 post-doc: status updated to complete; initialization sequence corrected (WS_EX_NOREDIRECTIONBITMAP, WM_ERASEBKGND; DwmExtendFrameIntoClientArea removed) |
| 0.6     | 2026-04-28 | Phase 3 post-doc: §6 Layout Engine added; §7–§9 renumbered; Open Questions updated |
| 0.7     | 2026-04-28 | §9 Open Questions: five layout-related items added (DPI scaling, AccessKit sync, async measure, cache invalidation, custom layout extensibility) |
| 0.8     | 2026-04-29 | Phase 4 post-doc: §7 Widget Implementation added; §7–§9 renumbered to §8–§10; §4 windows feature set updated; Open Questions updated |
