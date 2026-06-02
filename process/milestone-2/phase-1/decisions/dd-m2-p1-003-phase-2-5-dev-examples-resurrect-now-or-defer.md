### DD-M2-P1-003 — Phase 2-5 dev examples: resurrect now, or defer?

**Status:** Accepted

**Context:**
The M1 resolution to the rlib collision deleted
`phase2_window_check`, `phase3_layout_check`, `phase4_visual_check`,
`phase5_visual_check`. They depended on the runtime's internal Rust
API (`Runtime`, `WindowState`, `WidgetNode`, `TextRenderer`, …),
which was reachable only through the rlib. Once DD-M2-P1-001 Option A
lands, that reachability is restored.

A3's literal text says nothing about these examples. The architecture.md
§11.4 "long-term fix" paragraph says they "*can* be re-introduced
under a `wasamo-poc` workspace once that refactor is complete."

**Options:**

Option A — Resurrect under a new workspace dir (e.g. `wasamo-poc/`) in this phase
Option B — Defer; ship the structural split alone (recommended)
Option C — Drop them permanently

**Decision:** Option B on the main branch — Accepted (2026-05-03).

After the main-branch portion of M2-Phase 1 lands, an experimental
branch (`exp/m2-p1-poc-examples`) will be created to attempt
resurrecting the Phase 2-5 examples under `wasamo-poc/`. The branch
serves as a validation that the shim structure actually enables
in-workspace rlib consumers and as a reference for any future formal
resurrection. It is not merged to main unless a concrete acceptance
criterion demands it.

**Verified (2026-05-03).** Branch `exp/m2-p1-poc-examples` (tip
`d86d81c`, pushed to origin) restored all four examples from
`3a6da11^` into `wasamo-poc/` (workspace-excluded). The only edit
needed was renaming `wasamo::` to `wasamo_runtime::`; no widget or
runtime API change was required, confirming the cdylib-shim split
left the internal Rust API surface untouched. Build and interactive
verification (window display, hover/press, click → stdout, resize
re-layout, KeyFrame animation continuing while [B]-key blocks the
app thread) all pass on a local Windows 11 desktop. A side
observation about which environments are appropriate for which kinds
of verification (and why "SSH dev box" in DD-M2-P1-005 is not
interchangeable with GUI verification) is recorded in
[`docs/notes/verification-environments.md`](../../../../docs/notes/verification-environments.md).

---
