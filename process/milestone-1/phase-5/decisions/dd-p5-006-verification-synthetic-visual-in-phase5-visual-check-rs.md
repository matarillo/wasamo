### DD-P5-006 — Verification synthetic visual in `phase5_visual_check.rs`

**Status:** Accepted
**Supersedes:** DD-P5-002, DD-P5-003 ([phase-5-implicit-animations-dev-api.md](./superseded/dd-p5-001-003-implicit-animations-dev-api.md))

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
