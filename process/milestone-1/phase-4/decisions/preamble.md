# Phase 4 — Widget Implementation: Architecture Decisions

**Phase:** 4 (Text + Button widgets)
**Date:** 2026-04-29
**Status:** Accepted and implemented

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
