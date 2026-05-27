### DD-P2-003 — Mica backdrop implementation approach

**Status:** Accepted

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
- `DWMWA_USE_IMMERSIVE_DARK_MODE` must be set based on the system theme, not omitted and not
  hardcoded. The correct pattern: read `UISettings::GetColorValue(UIColorType::Background)`;
  if `R < 128` (near-black) the system is in dark mode. Pass `TRUE` or `FALSE` accordingly.
  Omitting the call causes DWM to default to light-mode Mica even on dark-mode systems.
  Hardcoding `TRUE` overrides the system theme for users on light mode (original finding still
  holds — do not force; do read and mirror the system preference).
