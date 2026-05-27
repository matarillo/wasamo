### DD-P2-002 — Global state management strategy

**Status:** Accepted

**Context:**
The runtime DLL needs to store live objects across C ABI call boundaries.
The objects fall into two categories with different lifetimes and cardinalities:

- **Process-wide objects** (`Compositor`, `DispatcherQueueController`): one
  per process by design; never need more than one.
- **Window-level objects** (`HWND`, `DesktopWindowTarget`, root
  `ContainerVisual`): one per window; Phase 6 already defines
  `wasamo_window_create() → WasamoWindow*`, implying windows are handles.

**Options:**

Option A — Full singleton (everything in one static)
- What you gain: Phase 2 implementation is minimal; no heap allocation for
  window state.
- What you give up: Phase 6 requires refactoring window state out of the
  singleton into `WasamoWindow*` handles. Multiple windows are impossible
  without a full rewrite. Diverges from every mature UI framework (WinUI 3,
  Qt, SDL, GTK) which all use handles for window-level state.

Option B — Runtime singleton + window handle (two-layer split)
- What you gain: `WasamoWindow*` in Phase 6 maps directly onto the
  heap-allocated `WindowState` struct — no refactoring needed. Multiple
  windows are naturally supported. Matches how all prior implementations
  structure their state (process-wide init once; per-window objects are
  handles). Phase 2 complexity increase is one heap allocation.
- What you give up: Slightly more code in Phase 2 compared to a flat
  singleton.

**Decision:** Option B — two-layer split.

```
// Process-wide (singleton)
static RUNTIME: OnceLock<Runtime> = OnceLock::new();
struct Runtime { compositor: Compositor, dq_controller: DispatcherQueueController }

// Per-window (heap-allocated, returned as *mut WindowState by wasamo_window_create)
struct WindowState { hwnd: HWND, target: DesktopWindowTarget, root: ContainerVisual }
```

WinRT types are not `Send + Sync`; both structs use `unsafe impl Send + Sync`
justified by the single-thread contract in §3 of `architecture.md`.

---
