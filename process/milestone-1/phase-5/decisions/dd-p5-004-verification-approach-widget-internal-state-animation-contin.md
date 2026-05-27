### DD-P5-004 — Verification approach: widget-internal state animation + continuous synthetic visual

**Status:** Accepted
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
