# Phase 5 — Implicit Animations (Dev API): Architecture Decisions

**Phase:** 5 (Visual Layer integration sanity check)
**Date:** 2026-04-29
**Status:** Superseded by [phase-5-compositor-independence-check.md](./phase-5-compositor-independence-check.md)

> **Note (added 2026-04-29):** This ADR's premise — that Phase 5
> verifies property-change animation via a dev-only toggle — was
> found in pre-implementation review to contradict DD-V-001 and to
> provide a weaker signal of compositor independence than a
> continuous ambient animation. The redirection is recorded in
> [phase-5-compositor-independence-check.md](./phase-5-compositor-independence-check.md).
> This file is preserved per the
> [Revision rule](./README.md#revision-rule) as historical record
> and as a worked example of the
> [Pre-doc discipline](./README.md#pre-doc-discipline) introduced
> alongside the supersession.

The vision-level decision driving this phase — that Wasamo's default
property-change behavior is **instant**, with animation deferred to a
public opt-in API in M5 — is recorded separately as
[DD-V-001 in vision-m1-acceptance-criteria.md](./vision-m1-acceptance-criteria.md#dd-v-001--default-property-change-behavior-is-instant-animation-is-opt-in).
This ADR captures only the Phase 5 implementation decisions that
follow from DD-V-001.

---

### DD-P5-001 — Phase 5 implements a dev-only internal API, not a public one

**Status:** Superseded by DD-P5-004 ([phase-5-compositor-independence-check.md](./phase-5-compositor-independence-check.md))

**Context:**
Given DD-V-001, Phase 5 cannot ship a public animation API — that work
belongs in M5. But Phase 5 still has value as a **sanity check** that
the Visual Layer is correctly engaged on the DWM compositor thread:

- `ImplicitAnimationCollection` actually animates property changes
- The compositor thread keeps animating while the app thread is blocked
- DWM compositing is engaged for the visual tree

This sanity check is not the differentiator of Wasamo — implicit
animation is table stakes across modern declarative UI frameworks (see
[DD-V-001](./vision-m1-acceptance-criteria.md#dd-v-001--default-property-change-behavior-is-instant-animation-is-opt-in)).
It is the cheapest visible proof that the substrate is working.

The question is how to expose the animation behavior so the
verification can run without committing to a public API.

**Options:**

Option α — Phase 5 only: temporarily make 150 ms interpolation the runtime default
- What you gain: Simple to implement. The visual_check binary needs no
  special hook.
- What you give up: Contradicts DD-V-001 within M1 itself. The decision
  to revert to "default instant" before M2 is easily forgotten, risking
  the wrong default leaking into the M1 release.

Option β — Public-but-deprecated C ABI dev functions
- What you gain: Verification works from any binding language. Removal
  is signaled by `__wasamo_dev_*` naming or a deprecation attribute.
- What you give up: Pollutes the C ABI surface that Phase 6 is trying
  to lock down. Any function exposed in `wasamo.h` is at risk of being
  depended on by early users despite the deprecation marking. M4 ABI
  freeze becomes harder.

Option γ — Sneak the public DSL syntax in early
- What you gain: No throwaway code; Phase 5 work feeds directly into M5.
- What you give up: Forces a DSL design decision in M1 that should
  happen in M5 with full context. Phase 1 (parser) would need to be
  reopened. Scope creep.

Option δ — Per-widget animation setters in C ABI
- What you gain: Verification works from any binding language. Granular.
- What you give up: Same ABI pollution problem as Option β, with more
  surface area. Phase 6 inflates.

Option ε — Internal Rust-only dev helper, not exposed via C ABI
- What you gain: Zero pollution of the C ABI or DSL surface. Removal is
  a local refactor, not a breaking change. Verification runs from a
  Rust binary (`examples/phase5_visual_check.rs`), which is sufficient
  because Phase 5 is a sanity check, not a user-facing demo. The "3
  languages" M1 acceptance criterion is unaffected (Counter does not
  need animations).
- What you give up: C and Zig consumers cannot exercise the dev API
  during M1 — they can still observe smooth resize, Mica, etc., but
  not implicit animations specifically.

**Decision:** Option ε — Phase 5 implements an internal `wasamo::dev`
module with a Rust-only `set_dev_implicit_animations(bool)` function.
The C ABI (`wasamo.h`) does not gain any animation-related functions
in M1. Phase 6's ABI surface review must explicitly exclude this
helper.

---

### DD-P5-002 — Animation parameters and trigger properties

**Status:** Superseded by DD-P5-006 ([phase-5-compositor-independence-check.md](./phase-5-compositor-independence-check.md))

**Context:**
When the dev helper is enabled, which `Visual` properties animate, and
with what timing? These values do not constitute a public API
commitment (see DD-P5-001), but they should be defensible defaults
that exercise the compositor realistically.

**Options:**

Option A — Match the original Phase 5 plan (Offset 150 ms cubic-ease, Size 150 ms, Opacity 100 ms)
- What you gain: Aligns with Microsoft Fluent motion guidance (150–
  300 ms range). Visible enough to verify smoothness, short enough to
  not feel sluggish during interactive verification.
- What you give up: None significant — the values are revisitable in M5.

Option B — Longer durations (e.g. 300 ms) for clearer visual demonstration
- What you gain: Easier to perceive the interpolation curve.
- What you give up: Verification becomes less representative of typical
  Fluent motion. Visible jank would be harder to spot at this duration.

**Decision:** Option A. These durations apply only to the dev helper
and carry no commitment for the public API designed in M5.

---

### DD-P5-003 — Removal plan for the dev helper

**Status:** Superseded by DD-P5-006 ([phase-5-compositor-independence-check.md](./phase-5-compositor-independence-check.md))

**Context:**
The dev helper is intentionally throwaway code. Without an explicit
removal trigger, it risks living indefinitely.

**Options:**

Option A — Remove when M5 public animation API ships
- What you gain: Removal is paired with the replacement, ensuring no
  capability gap.
- What you give up: M5 may be far in the future; helper persists for
  several minor versions.

Option B — Remove at start of M2
- What you gain: Short, predictable lifetime.
- What you give up: M2 work has no need to remove this; the removal
  becomes a procedural chore unrelated to M2's goals.

Option C — Remove when its first internal user disappears (when Phase 5 visual_check is no longer maintained)
- What you gain: Removal is driven by actual disuse.
- What you give up: visual_check binaries may be kept as regression
  tests indefinitely.

**Decision:** Option A — the helper is removed when the M5 public
animation API ships. Until then, it remains in `wasamo::dev` with a
module-level comment naming this ADR. The verification binary
`examples/phase5_visual_check.rs` and any tests built on the helper
are removed in the same change.
