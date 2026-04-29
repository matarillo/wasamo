# Phase 4 — Widget Implementation: Architecture Decisions

**Phase:** 4 (Text + Button widgets)
**Date:** 2026-04-29
**Status:** Agreed and implemented

---

### DD-P4-001 — Text rendering pipeline

**Status:** Agreed

**Context:**
`Text` must render Unicode glyphs onto a `SpriteVisual` using the Windows
rendering stack. Two approaches exist that are compatible with the M1
minimum target of Windows 10 1809+.

**Options:**

Option A — `ICompositionDrawingSurface` + Direct2D + DirectWrite
- What you gain: Works on Windows 10 1809+; well-documented interop path;
  same API surface as WinUI 2 and Windows App SDK on lower OS versions.
  `Win32_Graphics_Direct2D` and `Win32_Graphics_DirectWrite` are already
  declared in `wasamo/Cargo.toml`. The pipeline is:
  `IDWriteFactory` → `IDWriteTextLayout` → measure/draw;
  `ICompositorInterop::CreateGraphicsDevice(ID2D1Device)` →
  `CompositionDrawingSurface` → `BeginDraw()` → `ID2D1DeviceContext` →
  `DrawTextLayout()` → `EndDraw()` → `CompositionSurfaceBrush`.
- What you give up: Requires a `ID3D11Device` + `IDXGIDevice` +
  `ID2D1Device` setup; 3–4 new `windows` crate features.

Option B — `CompositionColorGlyphRunParameters` / Composition text APIs
- What you gain: Tighter Visual Layer integration, no D2D device setup.
- What you give up: Available only on Windows 11 22H2+, which would raise
  the minimum OS version and contradict the M1 acceptance criteria.

**Decision:** Option A — `ICompositionDrawingSurface` + Direct2D + DirectWrite.
Option B is ineligible for M1 due to the OS version constraint.

**Migration note:** If M2+ formally decides to raise the minimum OS version
to Windows 11 22H2+ or later, the text rendering backend may be migrated to
Option B at that time. That decision belongs in the M2 pre-document.

**New `windows` crate features required:**

```toml
"Win32_Graphics_Direct3D",           # D3D_DRIVER_TYPE, D3D_FEATURE_LEVEL
"Win32_Graphics_Direct3D11",         # D3D11CreateDevice, ID3D11Device
"Win32_Graphics_Dxgi",               # IDXGIDevice
"Win32_System_WinRT",                # ICompositionDrawingSurfaceInterop
```

(`Win32_Graphics_Direct2D`, `Win32_Graphics_DirectWrite`, and
`Win32_System_WinRT_Composition` are already present.)

---

### DD-P4-002 — Font property model

**Status:** Agreed

**Context:**
`Text` needs a `font` property. The DSL spec (`docs/dsl_spec.md`) already
uses `font: title` in the Counter example. The question is how far to
formalise the font API for M1.

**Options:**

Option A — Semantic enum (`TypographyStyle`) mapping to Windows type ramp
- Define `TypographyStyle` as an enum with four values for M1:
  `Caption` (12 sp, regular), `Body` (14 sp, regular),
  `Subtitle` (20 sp, semi-bold), `Title` (28 sp, semi-bold).
  Each variant maps to Segoe UI Variable with the corresponding size and
  weight, matching the WinUI 2 / WinApp SDK typography tokens and the DSL
  example syntax.
  The name `TypographyStyle` is preferred over `FontStyle` because
  `FontStyle` conventionally denotes the posture axis (Normal / Italic /
  Oblique), not the semantic size-and-weight scale.
- What you gain: DSL `font: title` maps directly to `TypographyStyle::Title`.
  DPI-aware sizing is managed by the type-ramp constants. Consistent with
  the platform visual language.
- What you give up: Custom font families and arbitrary point sizes are not
  expressible in M1. A larger font vocabulary must wait for M2.

Option B — Explicit font descriptor (`family: String, size: f32, weight: u16`)
- What you gain: Flexible; any system font is available.
- What you give up: More verbose; requires richer DSL syntax
  (`font: { family: "Segoe UI", size: 14 }`); DPI scaling becomes the
  caller's problem.

**Decision:** Option A — four-value `TypographyStyle` enum for M1.
Custom descriptors deferred to M2. This matches the DSL example and keeps
the API surface small.

---

### DD-P4-003 — Text natural size measurement

**Status:** Agreed

**Context:**
The layout engine (`layout.rs`) is pure Rust with no Win32/WinRT
dependencies (requirement from Phase 3). `Text` needs to report a natural
size (width × height) to the layout engine's `measure()` pass. The natural
size depends on font, text content, and the point size set by
`TypographyStyle`. Measuring requires calling `IDWriteTextLayout::GetMetrics()`.

**Options:**

Option A — Measure once at widget creation; cache as `(natural_w, natural_h)`
- When a `Text` `WidgetNode` is created (or `set_text()` / `set_font()`
  is called), call `IDWriteTextLayout` measurement immediately and store
  the result as `(natural_w, natural_h)` on the `WidgetNode`.
  `build_layout_tree()` uses `Fixed(natural_w)` × `Fixed(natural_h)` for
  the `LayoutNode`, keeping `layout.rs` dependency-free.
- What you gain: `layout.rs` stays pure Rust; measurement cost is paid once
  per text change, not every layout pass. Clean separation.
- What you give up: Natural size becomes stale if DPI changes. DPI-aware
  re-measurement is a M2+ concern (tracked in `architecture.md §9`).

Option B — Measure on every `build_layout_tree()` call
- What you gain: Always fresh.
- What you give up: Adds Win32 calls into `build_layout_tree()`, which
  forces `layout.rs` to accept Win32 context or complicates the call site.
  Measurably slower on deep widget trees.

Option C — Introduce a measurement callback in `LayoutNode`
  (`measure_fn: Option<Box<dyn Fn(f32, f32) -> (f32, f32)>>`)
- What you gain: `layout.rs` stays dependency-free while supporting lazy
  measurement.
- What you give up: Adds heap allocation and `dyn Fn` to `LayoutNode` for
  every widget; overcomplicated for M1's single-threaded, startup-time
  layout model.

**Decision:** Option A — measure at creation/update, cache on `WidgetNode`.
DPI re-measurement deferred to M2 (tracked in `architecture.md §9`).

---

### DD-P4-004 — Button visual structure

**Status:** Agreed

**Context:**
`Button` needs a background layer (fill color that changes on hover/press)
and a text label. Both must be `SpriteVisual` objects parented into the
Visual Layer tree.

**Options:**

Option A — `SpriteVisual` container (background brush) + child text `SpriteVisual`
- The button's root is a `SpriteVisual` with a `CompositionColorBrush`
  as background. A child `SpriteVisual` (created the same way as a `Text`
  widget) is added as an overlay for the label.
  `SpriteVisual` already supports `Children()` (it inherits from
  `ContainerVisual`), so the `append_child` pattern from Phase 3 applies.
- What you gain: Background and label are independent visuals; changing
  hover/press state only requires swapping the background brush on the root
  visual. Consistent with the existing `WidgetNode.visual: SpriteVisual`
  type contract.
- What you give up: Two SpriteVisuals per button.

Option B — Single `ICompositionDrawingSurface` with background + text co-drawn
- Background and label are drawn into one surface via D2D.
- What you gain: One GPU object per button.
- What you give up: Every state change (hover, press) requires redrawing
  the entire surface, including text layout. More complex and slower.

**Decision:** Option A — layered `SpriteVisual` structure.
Background brush swap on the root visual covers all state transitions cheaply.

**Button states and colors (M1):**

| State   | Default style (background)          | Accent style (background)          |
|---------|-------------------------------------|------------------------------------|
| Normal  | `#20FFFFFF` (20% white glass)       | System accent color (`UISettings`) |
| Hover   | `#33FFFFFF` (33% white)             | Accent color lightened by 10%      |
| Pressed | `#10FFFFFF` (10% white)             | Accent color darkened by 10%       |

Text color: `#FFFFFFFF` (always white) for both styles in M1.

**Known limitation (deferred to M2):** The color table above was designed for dark mode.
In light mode, `#20FFFFFF` on a light Mica surface provides insufficient contrast —
the Default button is nearly invisible and white text is unreadable.
The correct fix is theme-aware color sets (dark semi-transparent background + dark text
in light mode, matching WinUI 3 conventions). Deferred to M2 as part of broader
theme-aware widget styling work.

**Animation scope:** Button state transitions in M1 are instant brush swaps,
consistent with DD-V-001 (default behavior is instant; animation is opt-in).
Phase 5's dev-only implicit animation helper covers `Offset`, `Size`, and
`Opacity`; it does not animate `CompositionColorBrush` color changes.
Animated hover/press feedback requires `ColorKeyFrameAnimation` and is
deferred to M5 (public animation API).

---

### DD-P4-005 — `wnd_proc` ↔ `WindowState` linkage; WM_SIZE and mouse input

**Status:** Agreed

**Context:**
`wnd_proc` is a `unsafe extern "system" fn` registered at window class
creation. It has no inherent access to Rust state. Two event types require
reaching into Rust state from `wnd_proc`:

1. **WM_SIZE** — trigger re-layout with the new window dimensions.
   (Deferred from Phase 3; `private/CLAUDE.md` lists this as Phase 4 scope.)
2. **WM_LBUTTONDOWN / WM_LBUTTONUP / WM_MOUSEMOVE / WM_MOUSELEAVE** —
   button hit-test and hover state.

**Options:**

Option A — `GWLP_USERDATA` stores `*mut WindowState`; `WindowState` holds event callbacks
- After `WindowState` is constructed, call
  `SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize)`.
  In `wnd_proc`, retrieve it with `GetWindowLongPtrW(hwnd, GWLP_USERDATA)`.
  Add optional callbacks to `WindowState`:
  ```rust
  pub resize_fn:     Option<Box<dyn FnMut(f32, f32)>>,
  pub mouse_down_fn: Option<Box<dyn FnMut(i32, i32)>>,
  pub mouse_move_fn: Option<Box<dyn FnMut(i32, i32)>>,
  pub mouse_leave_fn: Option<Box<dyn FnMut()>>,
  ```
  The host sets these before calling `wasamo_run()`.
  `wnd_proc` dereferences the raw pointer and calls the relevant closure.
- What you gain: `window.rs` stays decoupled from `widget.rs` (callbacks
  are type-erased); standard Win32 idiom; works for both resize and mouse.
- **`unsafe` scope:** The only unsafe operations are
  `SetWindowLongPtrW` (one line in `window::create()`) and
  `GetWindowLongPtrW` + pointer dereference (2–3 lines in `wnd_proc`).
  `wnd_proc` is already `unsafe extern "system"`, and `window.rs` already
  contains extensive Win32 unsafe calls. All callback fields on
  `WindowState` (`Box<dyn FnMut>`) are safe Rust types. The public API
  (`wasamo::window_create`, `wasamo::run`, etc.) gains no new `unsafe`
  annotations; the unsafe surface does not grow beyond `window.rs`.
- What you give up: Raw pointer dereference in `wnd_proc` is `unsafe`;
  callers must ensure `WindowState` outlives the HWND (already required
  by the existing ownership model).

Option B — Thread-local static holds a reference to the widget tree
- `wnd_proc` calls into a thread-local `WIDGET_ROOT` to trigger layout
  or hit-test directly.
- What you gain: No pointer manipulation in `wnd_proc`.
- What you give up: Couples `window.rs` to `widget.rs` through a global;
  harder to extend to multi-window; non-idiomatic for Win32.

Option C — Poll mouse state in the message loop instead of `wnd_proc`
- What you give up: Misses out-of-focus events; not idiomatic; hover
  detection requires `TrackMouseEvent` anyway.

**Decision:** Option A — `GWLP_USERDATA` + callbacks on `WindowState`.
`SetWindowLongPtrW` is called at the end of `window::create()`, after the
`WindowState` `Box` is constructed (pointer is stable from that point).
The unsafe surface is contained entirely within `window.rs`.

**Hover tracking:** `WM_MOUSELEAVE` requires a prior `TrackMouseEvent`
call. `mouse_move_fn` calls `TrackMouseEvent` on first invocation (one-shot
per enter/leave cycle).

**WM_SIZE detail:** The `(width, height)` passed to `resize_fn` are the
new client area dimensions from `LOWORD(lparam)` / `HIWORD(lparam)`,
converted to `f32`. The host closure calls `root.run_layout(w, h)`.

---

### DD-P4-006 — Button clicked callback model

**Status:** Agreed

**Context:**
When a button is clicked, the host code must be notified. The callback
must work for both the Rust-native API (examples, `bindings/rust`) and
the C ABI (Phase 6 `wasamo.h`). The design should not introduce a
second, incompatible mechanism.

**Options:**

Option A — Store `Box<dyn Fn()>` on the Rust side; C ABI adds a separate setter
- `ButtonNode` (the internal type tracking button state) stores a
  `clicked_fn: Option<Box<dyn Fn()>>`.
- Rust-native callers pass a closure: `button.set_clicked(|| { ... })`.
- The C ABI Phase 6 function will be:
  `wasamo_button_set_clicked(widget, cb: unsafe extern "C" fn(*mut c_void), userdata: *mut c_void)`
  which wraps the C function pointer + userdata into a `Box<dyn Fn()>`.
- What you gain: Rust API is ergonomic; the C ABI wrapper (added in Phase 6)
  is a thin adapter; single internal dispatch path.
- What you give up: Phase 4 ships only the Rust-native setter; the C ABI
  wrapper is added in Phase 6 when all other ABI functions are finalized.

Option B — Only expose C ABI function pointer / userdata from the start
- What you gain: Forces early ABI thinking.
- What you give up: Rust-native callers must write `unsafe extern "C"` blocks
  for simple closures; awkward; misaligns with the Rust binding layer planned
  in Phase 7.

**Decision:** Option A — `Box<dyn Fn()>` internally; C ABI adapter in Phase 6.

---

## Decisions summary

| ID | Question | Decision |
|----|----------|----------|
| DD-P4-001 | Text rendering pipeline | `ICompositionDrawingSurface` + D2D + DirectWrite; migration to Option B if M2+ raises min OS to Win11 22H2+ |
| DD-P4-002 | Font property model | Semantic 4-value `TypographyStyle` enum (Caption / Body / Subtitle / Title) |
| DD-P4-003 | Text natural size measurement | Measure at create/update; cache `(natural_w, natural_h)` on `WidgetNode` |
| DD-P4-004 | Button visual structure | Root `SpriteVisual` (background brush) + child text `SpriteVisual`; state changes are instant brush swaps (animation is M5+ scope) |
| DD-P4-005 | `wnd_proc` ↔ window state | `GWLP_USERDATA` + event callbacks on `WindowState`; unsafe confined to `window.rs` |
| DD-P4-006 | Button clicked callback | `Box<dyn Fn()>` internally; C ABI adapter deferred to Phase 6 |
