# Wasamo Architecture

**Document version:** 0.1 (Draft — Phase 0, pending owner agreement)
**Last updated:** 2026-04-27
**Status:** Draft

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

## 5. Visual Layer Integration Overview

### HWND host model

A Win32 `HWND` created by `CreateWindowExW` hosts the Visual Layer via `DesktopWindowTarget`.

```
HWND
  └── DesktopWindowTarget
        └── ContainerVisual (root)
              └── (widget SpriteVisual tree)
```

Detailed decisions — thread model for `DispatcherQueueController`, global state management, Mica support scope — are deferred to the Phase 2 pre-implementation document.

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

M1 does not include code generation. Only parsing and syntax checking.

```
.ui file
  ↓ lexer
token stream
  ↓ parser
AST (Rust enum/struct)
  ↓ wasamoc check command
result: OK  or  error message + line number
```

Code generation (conversion to runtime calls, binding generation) is M2 scope.

---

## 8. Open Questions (to be resolved in later phases)

The following are intentionally left open at this draft stage.

| Question | Resolution phase |
|---|---|
| `DispatcherQueueController` thread model | Phase 2 |
| Global state management strategy (singleton vs. handle-based) | Phase 2 |
| Mica backdrop support scope for M1 | Phase 2 |
| Layout algorithm (custom measure/arrange vs. Taffy) | Phase 3 |
| Widget property API details | Phase 4 |
| Full C ABI function signatures | Phase 6 |

---

## Revision history

| Version | Date | Notes |
|---|---|---|
| 0.1 | 2026-04-27 | Initial draft (Phase 0, pending owner agreement) |
