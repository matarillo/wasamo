# Wasamo Architecture

**Document version:** 0.4
**Last updated:** 2026-04-28
**Status:** Phase 0, Phase 1, and Phase 2 pre-implementation agreed

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
  "Win32_Foundation",
  "Win32_UI_WindowsAndMessaging",
  "Win32_System_LibraryLoader",
  "Win32_Graphics_DirectWrite",
  "Win32_Graphics_Direct2D",
  "Win32_Graphics_Direct2D_Common",
  "Win32_Graphics_Dxgi_Common",
  "Win32_System_WinRT_Composition",
  "UI_Composition",
  "UI_Composition_Desktop",
  # DispatcherQueue feature TBD in Phase 2
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
3. CreateWindowExW(…)                         — create HWND
4. apply_mica(hwnd)                           — DwmSetWindowAttribute (Win11); no-op on Win10
5. DwmExtendFrameIntoClientArea(hwnd, -1)     — extend DWM frame to cover client area
6. DesktopWindowTarget::CreateForWindow(hwnd) — attach Visual Layer to HWND
7. ContainerVisual::new() → set as root       — root visual (no background brush; Mica shows through)
8. GetMessage / TranslateMessage / DispatchMessage loop
```

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

## 6. Three-Layer Tree Model

| Layer | Owner | Contents |
|---|---|---|
| **DSL tree** | `wasamoc` | Parsed AST of `.ui` file declarations |
| **View tree** | `wasamo` runtime | Widget hierarchy with resolved properties |
| **Visual tree** | Windows.UI.Composition | `SpriteVisual` hierarchy, the actual render target |

In M1 there is no reconciler. The host language constructs the view tree directly through the C ABI.

---

## 7. wasamoc (DSL Compiler) — M1 Scope

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

## 8. Open Questions (to be resolved in later phases)

The following are intentionally left open at this draft stage.

| Question | Resolution phase | Status |
|---|---|---|
| `DispatcherQueueController` thread model | Phase 2 | Resolved → DD-P2-001 (§5.3) |
| Global state management strategy (singleton vs. handle-based) | Phase 2 | Resolved → DD-P2-002 (§5.4) |
| Mica backdrop support scope for M1 | Phase 2 | Resolved → DD-P2-003 (§5.5) |
| Layout algorithm (custom measure/arrange vs. Taffy) | Phase 3 | Open |
| Widget property API details | Phase 4 | Open |
| Full C ABI function signatures | Phase 6 | Open |

---

## Revision history

| Version | Date       | Notes                                                         |
|---------|------------|---------------------------------------------------------------|
| 0.1     | 2026-04-27 | Initial draft (Phase 0, pending owner agreement)              |
| 0.2     | 2026-04-27 | Phase 0 agreed; added §7 wasamoc detail (Phase 1, pending owner agreement) |
| 0.3     | 2026-04-27 | Phase 1 agreed; status updated to reflect completed implementation |
| 0.4     | 2026-04-28 | Phase 2 pre-doc: §5 expanded with thread model, global state, Mica scope, feature decisions (pending owner agreement) |
