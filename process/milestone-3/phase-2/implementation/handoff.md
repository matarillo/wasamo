## Out-of-phase residuals

- **Dedicated `WASAMO_ERR_*` ABI code for layout-time runtime errors
  (T8).** DD-M3-P2-005 specifies that the unbounded-both-axes /
  no-extent conditions surface as runtime diagnostics with the Box's
  IR location. T8 lands the structural surface (`LayoutError` enum
  returned from `layout::run_layout`) and maps both variants to
  `windows::core::Error(E_FAIL)` at `WidgetNode::run_layout` so the
  existing `WM_SIZE` callsites keep their `windows::core::Result<()>`
  shape. The dedicated `wasamo.h` error code, IR-location plumbing on
  `LayoutNode`, and the C ABI translation are out of Phase 2 scope:
  the call sites at `window.rs::WM_SIZE` and `emit.rs::mark_layout
  _dirty_for` already swallow the Result with `let _ = …`, so a richer
  surface would be unused until a phase introduces a `wasamo_run
  _layout` (or layout-error callback) entry point. Tracked here so
  the residual lands in the M3-Phase 3 / Phase 4 pre-doc input scan
  rather than getting lost.
