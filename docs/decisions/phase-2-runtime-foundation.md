# Phase 2 — Runtime Foundation: Architecture Decisions

**Phase:** 2 (Runtime Foundation)
**Date:** 2026-04-28
**Status:** Agreed

---

### DD-P2-001 — `DispatcherQueueController` thread model

**Status:** Agreed

**Context:**
Visual Layer (`Windows.UI.Composition`) requires a `DispatcherQueue` to be
associated with the calling thread before any Compositor API is used.
`CreateDispatcherQueueController` offers two thread placement modes.
The choice determines how the Win32 message loop and Composition dispatch
coexist.

**Options:**

Option A — `DQTYPE_THREAD_CURRENT` (main-thread STA)
- What you gain: Win32 message loop and Visual Layer dispatch run on the same
  thread. No cross-thread calls, no synchronization primitives, no wakeup
  signalling. The entire runtime is single-threaded and trivially debuggable.
- What you give up: Heavy Composition work (e.g. surface uploads) can block
  the message loop and cause input lag. Not a concern at M1 scale.

Option B — `DQTYPE_THREAD_DEDICATED` (dedicated Composition thread)
- What you gain: Visual Layer operations run on a separate thread, so the
  Win32 message loop stays responsive even under rendering load. Gives a
  head start toward an architecture where animation and input are decoupled.
- What you give up: Every interaction between the runtime and Composition
  must be marshalled across threads. Substantial additional complexity for
  no perceptible benefit at M1's widget count.

**Decision:** Option A — single-thread STA is the standard pattern for Win32
desktop apps using `Windows.UI.Composition` and matches M1's complexity budget.

---

### DD-P2-001b — `apartmentType`: `DQTAT_COM_STA` vs `DQTAT_COM_ASTA`

**Status:** Agreed

**Context:**
`CreateDispatcherQueueController` takes a `DispatcherQueueOptions.apartmentType`
that controls how the calling thread's COM apartment is initialized.
Two meaningful values exist for desktop apps:

| Constant | Value | COM apartment |
|---|---|---|
| `DQTAT_COM_ASTA` | 1 | Application STA — WinRT / UWP apartment model |
| `DQTAT_COM_STA` | 2 | Standard STA — classic Win32 COM apartment model |

Microsoft's official `HelloComposition` C++ sample uses `DQTAT_COM_ASTA`
because it is written with C++/WinRT, whose default execution model is ASTA.
However, C++/WinRT is not the execution model of Wasamo.

**Options:**

Option A — `DQTAT_COM_ASTA` (Application STA)
- What you gain: Reentrancy protection — the ASTA model blocks nested
  message-pump reentrancy that can cause subtle bugs and deadlocks. Matches
  the threading model WinRT objects were originally designed for (UWP).
- What you give up: ASTA semantics are UWP-specific. Windows App SDK (WinUI 3)
  explicitly migrated *away* from ASTA to standard STA for desktop apps,
  accepting the reentrancy trade-off in exchange for conventional Win32
  behavior. Wasamo would diverge from the direction of the broader ecosystem.

Option B — `DQTAT_COM_STA` (standard STA)
- What you gain: Follows the Win32 desktop app convention. Aligns with Windows
  App SDK / WinUI 3 desktop, which chose standard STA when moving off UWP.
  No UWP-specific apartment constraints on the message loop.
- What you give up: No automatic reentrancy protection. Nested message-pump
  reentrancy (e.g., from a modal dialog or a blocking call that pumps messages)
  must be handled by the application, not the apartment model.

**Decision:** Option B (`DQTAT_COM_STA`) — Wasamo is a Win32 desktop app,
not UWP. Standard STA is what Windows App SDK uses when targeting desktop,
and it is the natural fit for a framework built on Win32 HWND hosting.

**Note — windows 0.58 feature name:**
`"System_DispatcherQueue"` does not exist in windows 0.58.
`DispatcherQueueController` is defined directly in `Windows::System` (no
sub-feature). The correct addition to `wasamo/Cargo.toml` is `"System"`.
`CreateDispatcherQueueController` and its supporting types
(`DispatcherQueueOptions`, `DQTYPE_THREAD_CURRENT`, `DQTAT_COM_STA`) are
already available via the existing `Win32_System_WinRT` feature.

---

### DD-P2-002 — Global state management strategy

**Status:** Agreed

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

### DD-P2-003 — Mica backdrop implementation approach

**Status:** Agreed

**Context:**
The DSL already declares `backdrop: mica` in the reference example
(`counter.ui`). The question is when and how to implement it.

Two approaches exist for applying Mica to a Win32 HWND:
- **DWM direct** (`DwmSetWindowAttribute`): a Win32 API call of ~20 lines.
  Used by Tauri (`window-vibrancy`) and Flutter (`flutter_acrylic`).
- **`SystemBackdropController`** (WinUI 3 high-level API): significantly more
  complex; designed for WinUI 3, not raw Win32.

**Options:**

Option A — Solid color background, Mica deferred
- What you gain: Phase 2 implementation is minimal. Works on Windows 10 1809+
  without any OS-version guard.
- What you give up: `backdrop: mica` in the DSL produces no visible effect
  during M1. The gap between DSL declaration and actual output persists through
  all of M1 development. Requires a subsequent phase to close it.

Option B — Mica via `DwmSetWindowAttribute` + OS-version guard + solid color fallback
- What you gain: `backdrop: mica` in the DSL works end-to-end from Phase 2.
  Implementation is ~20 lines (2–3 DWM calls + version check) — the same
  pattern used by Tauri and Flutter in production. Root `ContainerVisual`
  is transparent, letting the DWM-rendered Mica show through. On Windows 10
  the path degrades gracefully to a solid-color background.
- What you give up: Mica is visible only on Windows 11 21H2+ (Build 22000+).
  Root `ContainerVisual` must not carry a background brush (this is a
  constraint on Phase 3+ rendering, not a Phase 2 complexity).

**Decision:** Option B — DWM direct approach.

OS version tier:

```
Build 22523+ (Win11 22H2): DwmSetWindowAttribute(DWMWA_SYSTEMBACKDROP_TYPE, DWMSBT_MAINWINDOW)
Build 22000–22522 (Win11 21H2): DwmSetWindowAttribute(DWMWA_MICA_EFFECT, 1)
Pre-Win11: solid-color fallback (no DWM call)
```

Additional `windows` feature required: `Win32_Graphics_Dwm`.

**Implementation notes (post-implementation):**

Two additional requirements emerged during implementation:

- `WS_EX_NOREDIRECTIONBITMAP` must be set on the HWND. Without it, DWM creates a GDI
  redirection buffer that paints an opaque white surface over the Mica backdrop.
- `WM_ERASEBKGND` must return 1. Without it, GDI paints the default background colour
  over the DWM backdrop even when a redirection buffer is not present.
- `DwmExtendFrameIntoClientArea` with `{-1,-1,-1,-1}` (Aero Glass "sheet of glass") must
  **not** be called alongside `DWMSBT_MAINWINDOW`. When called with no GDI surface, DWM
  renders the DWM frame colour (dark in dark mode) across the entire client area, covering
  the Mica material. `DWMSBT_MAINWINDOW` covers the full window backdrop automatically.
- `DWMWA_USE_IMMERSIVE_DARK_MODE` is not set unconditionally. The Mica material follows the
  system colour theme; forcing it overrides appearance without a corresponding system preference.
