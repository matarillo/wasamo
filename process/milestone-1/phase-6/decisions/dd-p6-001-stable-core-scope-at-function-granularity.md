### DD-P6-001 — Stable-core scope at function granularity

**Status:** Accepted

**Context:**
The stable core must be the smallest set of functions that lets a
host language drive a wasamo runtime end-to-end **without** depending
on the M1 experimental imperative builder. It is sized assuming M2
codegen (or IR) will produce the widget tree, so the core does not
need any tree-construction primitive — only the surfaces a generated
binding would call at runtime: lifecycle, window + event loop,
property get/set, property-change observers, and signal registration
for component-declared signals.

**Options:**

Option A — Five-area minimum (recommended)
The core covers exactly the five areas listed in
[ROADMAP Phase 6](../../ROADMAP.md#L128-L131):
1. **Runtime lifecycle:** `wasamo_init`, `wasamo_shutdown`.
2. **Window + event loop:** `wasamo_window_create`, `wasamo_window_show`, `wasamo_window_destroy`, `wasamo_run`, `wasamo_quit`.
3. **Property get/set:** `wasamo_get_property`, `wasamo_set_property` keyed by `(widget, property_id)`.
4. **Property-change observer:** `wasamo_observe_property` / `wasamo_unobserve_property`.
5. **Component-declared signal register:** `wasamo_signal_connect` / `wasamo_signal_disconnect`.

- What you gain: One-to-one match with the agreed framing. Each area
  is independently justifiable as a survivor of both deferred
  questions: lifecycle, event loop, property R/W, observers, and
  signals are needed regardless of whether handlers run host-side or
  runtime-side, and regardless of codegen vs IR.
- What you give up: Some functions a "real" ABI wants — text
  measurement, image loading, HWND escape hatch — are outside the
  core. They will surface during Phase 7-8 work and either get added
  to the experimental layer or trigger a follow-up ADR.

Option B — Five-area minimum **plus** an HWND escape hatch
Same as A, plus `wasamo_window_get_hwnd` for host code that needs to
interop with native Win32 (custom drag regions, system menu, etc.).

- What you gain: Reduces the chance of host code being forced into
  the experimental layer for genuinely missing primitives.
- What you give up: HWND in the stable core leaks an implementation
  detail that may not survive future windowing changes (e.g. if
  wasamo ever supports `AppWindow`-only modes). Cleaner to add later
  with full deliberation than to retract.

Option C — Five-area minimum **plus** a focus / IME hook
Same as A, plus `wasamo_window_set_focus` and a minimal IME query.

- What you gain: Phase 8 "Hello Counter" likely wants keyboard focus
  navigation. Including focus in the core avoids a same-phase
  follow-on.
- What you give up: M1 has no widget set that needs IME yet (Phase 4
  delivered Button + Text only). Premature.

**Recommendation:** **Option A.** It matches the agreed framing
exactly, keeps the M4 freeze surface auditable, and defers escape
hatches (HWND, focus, IME) until concrete phase work demands them.
HWND access during M1 stays in the experimental layer if needed.

---
