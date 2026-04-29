# Vision Revision — M1 Acceptance Criteria and Animation Defaults

**Date:** 2026-04-29
**Status:** Agreed
**Triggered by:** Pre-implementation review of Phase 5 (see
[phase-5-implicit-animations-dev-api.md](./phase-5-implicit-animations-dev-api.md))

---

## Context

This is not a phase ADR. It records a vision/roadmap-level revision
that took place outside any single phase's pre-implementation review,
and is filed under the `vision-` naming convention defined in
[README.md](./README.md).

The Phase 5 pre-implementation discussion surfaced two issues with
VISION.md as previously written:

1. **§7 (M1 acceptance criteria) and §2.1 (Slint critique) implicitly
   elevated "implicit animation" to differentiation status.** A survey
   of comparable declarative UI frameworks (SwiftUI, Jetpack Compose,
   Flutter, Qt QML, CSS) confirmed that implicit animation as a feature
   is universal across modern declarative UI — it is not a Wasamo
   differentiator. VISION.md §5 (the comparison table) correctly omits
   "implicit animation" as a differentiation axis, so §7 and §2.1 were
   inconsistent with §5.

2. **§4 (Product principles) and the original Phase 5 plan implied that
   Wasamo would animate property changes by default.** This conflicted
   with the conventions of the frameworks Wasamo's audience already
   knows. The same survey showed that all of SwiftUI / Compose /
   Flutter / Qt QML / CSS default to *instant* property change with
   opt-in animation. Apple's CALayer is the historical exception
   (default-on), and that choice is widely regarded as a usability
   mistake — iOS developers routinely call
   `CATransaction.setDisableActions(true)` to disable it. Microsoft's
   own Visual Layer requires explicit attachment of an
   `ImplicitAnimationCollection`, mirroring the opt-in convention.

This revision addresses both issues at the vision/roadmap level. The
downstream Phase 5 implementation decisions are recorded separately in
[phase-5-implicit-animations-dev-api.md](./phase-5-implicit-animations-dev-api.md).

---

### DD-V-001 — Default property-change behavior is instant; animation is opt-in

**Status:** Agreed

**Context:**
Should Wasamo's default property-change behavior be animated (always
on) or instant (opt-in)? See the Context section above for the survey
results that informed this decision.

**Options:**

Option A — Default instant, animation opt-in (consistent with SwiftUI / Compose / Flutter / CSS)
- What you gain: Predictable behavior matching the conventions developers
  arriving from other declarative UI frameworks already know. Fine-grained
  control over which property changes animate. No risk of accidental
  animations causing performance issues or visual noise. Aligns with the
  Visual Layer's own API design (collection attachment is explicit).
- What you give up: Wasamo apps do not animate "for free" — the developer
  must opt in once a public animation API exists.

Option B — Default animated, opt-out per visual
- What you gain: Wasamo apps look polished out of the box without any
  developer effort. The compositor's smoothness is immediately visible.
- What you give up: Surprising behavior for developers used to other
  frameworks. Opt-out APIs tend to be ergonomically awkward (cf. CALayer
  history). Risk of animations firing during programmatic updates that
  the developer expected to be instant. Conflicts with the survey of
  industry conventions.

**Decision:** Option A — Wasamo's default property-change behavior is
**instant**. A public opt-in animation API will be designed in M5
("Higher-level animation DSL" per ROADMAP.md). VISION.md §4 has been
updated to make this explicit.

---

## Documents updated alongside this ADR

| Document / section | Change | Rationale |
|---|---|---|
| VISION.md §2.1 | Slint critique reframed: the parenthetical "(animations that keep running while the app thread is idle, vSync alignment with the OS, integration with system materials)" replaced with "independent compositor-thread rendering, vSync alignment with the OS, and integration with system materials are all properties Slint cannot easily inherit" | The architectural property of the Visual Layer is *independent compositor-thread rendering*. Animations that keep running are one downstream consequence of that property, not the property itself. The new wording puts the structural fact first. |
| VISION.md §4, principle #1 | "smooth animation" removed from the default-behavior list. New sentence makes opt-in explicit and references industry conventions (SwiftUI, Jetpack Compose, Flutter, CSS) | Implements DD-V-001. The default-behavior list now contains only items that genuinely engage by default (Mica/Acrylic, theming, type ramp, accent). |
| VISION.md §7, M1 acceptance | "Rendering through Visual Layer with implicit animations" → "Rendering through the Visual Layer (DWM compositing engaged, visual tree responsive on the compositor thread)" | The hypothesis Wasamo exists to test (per §2.2) is "external DSL × C ABI × Visual Layer". Implicit animations are not part of that hypothesis — they are one possible demonstration of it. The revised wording names the architectural property to be verified and leaves the choice of demonstration to the implementation phase. |
| ROADMAP.md M1 | Synced with VISION.md §7. Phase 5 section rewritten to reflect dev-only sanity-check scope | Same rationale as §7. Phase 5 implementation specifics live in [phase-5-implicit-animations-dev-api.md](./phase-5-implicit-animations-dev-api.md). |

---

## Relation to phase ADRs

[phase-5-implicit-animations-dev-api.md](./phase-5-implicit-animations-dev-api.md)
records the Phase 5 implementation decisions that follow from DD-V-001:

- A dev-only internal API (not exposed via C ABI) for sanity-checking
  Visual Layer integration
- Animation parameters used by that helper
- Removal plan for when M5 ships the public opt-in API

DD-V-001 is the upstream root for those decisions. The Phase 5 ADR
references this ADR rather than restating the default-behavior decision.

**Update (2026-04-29):** The Phase 5 implementation ADR referenced
above was itself superseded later the same day by
[phase-5-compositor-independence-check.md](./phase-5-compositor-independence-check.md)
(DD-P5-004..006), after a second pre-doc review found that its
premise — verifying property-change animation via a dev-only toggle
— contradicted DD-V-001. The redirection adopts widget-internal
state-transition animation for Button (permanent product behavior)
plus a continuous synthetic Visual in the verification example.
DD-V-001 in this vision ADR remains in effect unchanged; only the
downstream phase-level implementation decisions were redirected.
See the new phase ADR for the current Phase 5 plan.
