# Phase 5 — Compositor Independence Check: Architecture Decisions

**Phase:** 5 (Visual Layer integration sanity check)
**Date:** 2026-04-29
**Status:** Agreed (pending implementation)
**Supersedes:** [phase-5-implicit-animations-dev-api.md](./phase-5-implicit-animations-dev-api.md) (DD-P5-001..003)

The original Phase 5 ADR
([phase-5-implicit-animations-dev-api.md](./phase-5-implicit-animations-dev-api.md))
treated the ROADMAP task list — "ImplicitAnimationCollection animates
Offset/Size/Opacity property changes" — as a fixed premise and
deliberated only on how to expose a dev-only API for that behavior.
Pre-doc review (the kind described in
[README.md "Pre-doc discipline"](./README.md#pre-doc-discipline))
surfaced three problems with that premise:

1. Property-change animation is exactly the behavior
   [DD-V-001 in vision-m1-acceptance-criteria.md](./vision-m1-acceptance-criteria.md#dd-v-001--default-property-change-behavior-is-instant-animation-is-opt-in)
   defines as **opt-in, not default**. Verifying it as if it were
   default would embed a contradicting expectation into M1 itself,
   even when gated behind a dev API.
2. An industry survey (CSS, SwiftUI, Jetpack Compose, Material,
   WinUI) shows that built-in widgets animate their **own state
   transitions** internally — distinct from property-change
   animation. This is the convention M1 should follow.
3. The original verification approach (toggling property-change
   animation) provides a weak signal for compositor independence:
   state-driven transitions are transient and require precisely-timed
   app-thread blocks to observe. A continuous ambient animation
   gives a structurally stronger signal.

This ADR records the redirection. Phase 5 splits into a permanent
product decision (Button widget-internal state-transition animation)
and a verification artifact decision (continuous synthetic visual in
the Phase 5 example). Neither requires a `wasamo::dev` toggle for
property-change animation.

---

### DD-P5-004 — Verification approach: widget-internal state animation + continuous synthetic visual

**Status:** Agreed
**Supersedes:** DD-P5-001 ([phase-5-implicit-animations-dev-api.md](./phase-5-implicit-animations-dev-api.md))

**Context:**
The acceptance criterion Phase 5 must verify is "the Visual Layer is
correctly engaged on the DWM compositor thread (DWM compositing
engaged, visual tree responsive on the compositor thread)." The
question is what observable behavior best demonstrates this property
without committing to a public animation API in M1.

The verification target splits into two independently-decidable
parts:

- **(a) Permanent product behavior:** what animation, if any, should
  ship as part of M1 widget behavior?
- **(b) Verification artifact:** what additional animation should the
  Phase 5 example exhibit to make compositor-thread independence
  observable?

**Options:**

Option A — Dev-only Rust API toggling property-change animation (original DD-P5-001)
- What you gain: A single mechanism exercises Offset / Size / Opacity
  animation primitives at once.
- What you give up: Verifies behavior that contradicts DD-V-001 even
  while disabled by default — the toggle's existence implies
  property-change animation is the intended demonstration. Transient
  signal — requires the user to time hover and app-thread blocking
  to observe. Adds a removable internal API surface (`wasamo::dev`
  module) that must be tracked for removal in M5.

Option B — Widget-internal state animation only (Button hover/press)
- What you gain: Aligns with industry convention — built-in widgets
  animating their own state transitions is universal across
  CSS / SwiftUI / Compose / Material / WinUI. Does not contradict
  DD-V-001. No removal plan needed.
- What you give up: Transient signal — observing compositor-thread
  independence requires the user to time hover and app-thread
  blocking precisely. Single primitive exercised (Color animation).

Option C — Continuous synthetic visual only (no widget-internal change)
- What you gain: Strongest signal for compositor independence —
  ambient continuous animation makes the "press B to block, watch
  the visual keep moving" demo unambiguous regardless of timing.
- What you give up: M1 widgets (Button) remain unanimated, diverging
  from industry convention on a visible product surface. The
  verification example does not exercise any production widget
  behavior beyond what Phase 4 already shipped.

Option D — Combination: widget-internal state animation + continuous synthetic visual
- What you gain: All of B's industry-convention alignment plus all
  of C's strong-signal property. The two parts are independent —
  B is decided as a permanent product behavior on its own merits,
  C is decided as a verification artifact on its own merits — and
  they coexist without coupling.
- What you give up: Slightly more code than B or C alone. The
  primitive coverage is two animation primitives (Color for Button,
  Vector3/scalar for synthetic), still narrower than Option A.

Option E — Pure passive observation (no animation)
- What you gain: Zero new code. The compositor-thread property is
  already verifiable: block the app thread for 2 s and confirm that
  Mica continues to redraw and the mouse cursor continues to render
  — both are OS-driven, but they prove that the window is not
  app-thread-gated.
- What you give up: Does not actually exercise the runtime's wiring
  of `Compositor` / animation primitives — only proves the OS-side
  rendering loop. The cheapest *visible* proof of "our Visual Layer
  configuration works" is lost.

Option F — Lifecycle-only animation (one-shot fade-in on window appear)
- What you gain: Minimal — a single `ScalarKeyFrameAnimation` on
  the root visual's Opacity at startup. No ongoing implication for
  property-change semantics.
- What you give up: One-shot animations cannot be observed under
  app-thread blocking — by the time the user presses 'B', the
  animation has already completed. Effectively useless for the
  compositor-independence demo.

**Decision:** Option D. The narrower primitive coverage compared to
Option A is acceptable because Phase 5 is a sanity check, not a
coverage audit; the M5 public animation API will exercise the full
primitive set when designed. Options B and C are recorded as the
component decisions of D and are individually weaker than their
combination — neither alone delivers both industry-aligned product
behavior and a strong-signal verification artifact. Option E is
preserved as a complementary passive check the verification example
may also exhibit, but is insufficient on its own. Option F was
rejected outright.

---

### DD-P5-005 — Button widget-internal state-transition animation (permanent)

**Status:** Agreed

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
Button's brush color over **150 ms** (cubic ease-out for hover-in
and press-down; cubic ease-in for hover-out and press-up). These
values are internal Button implementation and are not exposed to
host code. Option C was rejected for diverging from Windows
convention; if Microsoft's own design system later adopts press
depression, this decision is revisable in a future ADR without ABI
impact.

---

### DD-P5-006 — Verification synthetic visual in `phase5_visual_check.rs`

**Status:** Agreed
**Supersedes:** DD-P5-002, DD-P5-003 ([phase-5-implicit-animations-dev-api.md](./phase-5-implicit-animations-dev-api.md))

**Context:**
To provide a strong, continuous signal of compositor-thread
independence, the Phase 5 verification example exhibits one
ambient-animated visual in addition to the production widgets. The
animation must run independently of any app-thread activity.

**Options:**

Option A — Introduce a minimal `ProgressIndicator` / spinner widget in `wasamo`
- What you gain: A "real widget" demonstrating compositor
  independence; the spinner is a recognizable UI affordance.
- What you give up: Premature widget design. `ProgressIndicator`'s
  public API (size, color, speed, accessibility role) belongs to M2
  or M3 when the widget set is intentionally expanded. Introducing
  it in Phase 5 to satisfy a verification need contaminates a product
  decision.

Option B — Synthetic `SpriteVisual` added directly by the verification example
- What you gain: Zero design surface — it is a colored rectangle that
  rotates, nothing more. Exists only inside
  `examples/phase5_visual_check.rs`. Honest about what is being
  verified (compositor wiring, not a UI feature). When the
  verification's value fades, deletion is a few lines in the example.
- What you give up: Less visually polished than a real widget. The
  verification example does not double as a product demo.

Option C — No additional visual; rely on Button hover animation alone (DD-P5-005)
- What you gain: Smallest verification surface. No synthetic code at
  all. The permanent product behavior (Button hover animation) is
  the entire demonstration.
- What you give up: Reverts to Option B's weakness in DD-P5-004 —
  transient signal requires precisely-timed app-thread blocking.
  The reader of the verification artifact cannot easily distinguish
  "compositor independence works" from "I happened to release the
  hover before pressing B."

**Decision:** Option B. The synthetic visual is a small `SpriteVisual`
in a corner of the window, with a continuous rotation or translation
driven by a looping `Vector3KeyFrameAnimation` (period ~2 seconds).
The example exposes a 'B' key to block the app thread for ~2 seconds;
the synthetic visual must continue animating during the block,
demonstrating compositor-thread independence.

The runtime exposes the minimum surface the example needs to attach
a `Visual` to the root container — a `pub(crate)` accessor or a
small `wasamo::dev` helper restricted to root-Visual access. This
is **not** the property-change animation toggle the superseded ADR
proposed; it is a narrow scaffolding hook for the verification
example only. No C ABI surface is added in this phase.
