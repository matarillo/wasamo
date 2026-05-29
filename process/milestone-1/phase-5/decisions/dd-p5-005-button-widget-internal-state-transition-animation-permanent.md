### DD-P5-005 — Button widget-internal state-transition animation (permanent)

**Status:** Accepted

**Context:**
Phase 4 shipped Button with hover and press states implemented as
instant brush swaps. This decision concerns whether and how to
animate those transitions as part of Button's permanent
implementation, separate from any public property-change animation
API (which remains deferred to M5 per DD-V-001).

**Options:**

Option A — Keep instant brush swap
- What you gain: Simplest implementation. No `CompositionAnimation`
  attachment needed.
- What you give up: Diverges from the convention of every comparable
  framework: SwiftUI `borderedProminent`, Material `Button`, WinUI
  Button template, and CSS design systems all animate state
  transitions internally. M1's most visible widget feels unpolished
  by Windows standards.

Option B — Animate hover/press color transition with `ColorKeyFrameAnimation`
- What you gain: Matches industry convention. Provides visible polish
  that is not a public API commitment — the duration and easing are
  internal Button details, revisable without ABI impact. Exercises
  the Compositor's color animation primitive on the compositor
  thread.
- What you give up: Slightly larger Button implementation; per-Button
  bookkeeping for the brush animation target.

Option C — Animate color **and** scale (press depression)
- What you gain: Richer feedback — combines color transition with a
  subtle Scale animation on press, similar to iOS/macOS button
  press visuals. Exercises two primitives (Color + Vector3 Scale)
  inside one widget.
- What you give up: Scale-on-press is not native Windows convention
  (WinUI Button templates do not depress on press; they swap colors
  only). Adopting it imports an Apple-platform feel that does not
  match Wasamo's product principle of "Native Windows feel".

**Decision:** Option B. Hover and press transitions animate the
Button's brush color using `ColorKeyFrameAnimation`. The
`CompositionColorBrush` is retained on `ButtonData` and animated in
place via `CompositionObject::StartAnimation("Color", ...)` — no new
brush is created per transition.

Option C was rejected for diverging from Windows convention; if
Microsoft's own design system later adopts press depression, this
decision is revisable in a future ADR without ABI impact.

**Post-implementation update (2026-04-29):**
Duration and easing values determined by side-by-side visual
comparison with a WinUI Button on the same OS build:

| Transition | Duration | Notes |
|---|---|---|
| Normal → Hovered (hover-in) | 83 ms | Fluent "ControlFast" token |
| Hovered → Normal (hover-out) | 167 ms | Fluent "ControlNormal" token |
| Any → Pressed (press-down) | 83 ms | Fast response for direct input |
| Pressed → Any (press-up) | 167 ms | Slower "settle" on release |

Easing: linear (default; no `CompositionEasingFunction` attached).
WinUI Button uses a near-linear ease-out; the visual difference is
imperceptible at these durations. A cubic-bezier easing can be
applied in a future revision without any API or ABI impact.

These values are internal Button implementation details. They are not
part of the C ABI or any public Rust surface.

---
