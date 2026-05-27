### DD-P4-003 — Text natural size measurement

**Status:** Accepted

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
