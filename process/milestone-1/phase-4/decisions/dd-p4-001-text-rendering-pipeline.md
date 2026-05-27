### DD-P4-001 — Text rendering pipeline

**Status:** Accepted

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
