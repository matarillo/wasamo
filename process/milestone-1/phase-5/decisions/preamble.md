# Phase 5 — Compositor Independence Check: Architecture Decisions

**Phase:** 5 (Visual Layer integration sanity check)
**Date:** 2026-04-29
**Status:** Implemented
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
