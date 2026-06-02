## Out-of-phase residuals

- **R1 — Gallery host Window title wiring (2026-05-25).** Phase 4
  smoke recorded `MainWindowTitle = "Wasamo"` (framework default)
  while `examples/gallery/gallery.ui` declares `title: "Gallery"`.
  Current `.ui` lowering preserves the component-level `title:`
  surface, but the runtime/ABI host path creates the Window with the
  framework default title.
  - **Owner intent (T7 Q2 disposition, 2026-05-25):** `.ui` `title:`
    must drive the actual native Window title. This is an **M3
    residual, not an M4 theming/chrome handoff**.
  - **Resolution condition:** "the runtime/ABI host path applies the
    component-level `title` to the native window", **not** "title
    attribute declared unsupported".
  - **Gate structure:**
    - **M3-Phase 5 pre-doc input distillation** assigns the owning
      M3 phase (1-2 candidates narrowed; full assignment must
      complete before Phase 5 ADR is Accepted). Pre-doc input note
      filed at
      [docs/notes/m3-phase-5/predoc-inputs.md §4](../../phase-5/requirements/constraints.md#4-r1-window-title-wiring-の-owning-phase-割当--phase-5-pre-doc-内で必須完了).
    - **Implementation deadline:** no later than **M3-Phase 8
      Gallery E2E close**.
    - **Natural candidate phase:** Phase 6 (ZStack + conditional
      rendering) — small host/window-metadata wiring task that
      pairs naturally with lightbox UX. Phase 5 (Grid layout
      primitive) is **not** a recommended owning phase because the
      task is unrelated to Grid thesis.
