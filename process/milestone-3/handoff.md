---
title: M3 handoff - public-draft residuals and pre-1.0 carry-forward
status: draft
created: 2026-07-05
source-milestone: M3
---

# M3 Handoff

Drafted during M3-Phase 8 T9 for the FD-8-G(4) owner review. This file is
the milestone-level handoff, distinct from
[phase-8/implementation/handoff.md](phase-8/implementation/handoff.md).
It stays `status: draft` until the milestone-close batch finalizes it after
Phase 8 closes.

M3's public draft and integrated Gallery prove the shipped DSL surface. This
handoff records the roadmap / trigger-driven residuals that need milestone
handoff visibility; it is not a parallel spec and it does not index every
per-primitive future surface. The normative surfaces live in
[docs/dsl_spec.md](../../docs/dsl_spec.md) and
[docs/architecture.md](../../docs/architecture.md).

Per-primitive / per-family deferrals remain in their owning spec sections:
for example keyed identity / state retention, nested control flow, handler
forms inside `for`, operator conditions, and Grid future track surfaces are
documented in the relevant Out of scope / Reserved future surface sections of
`docs/dsl_spec.md`. This handoff carries only the residuals whose roadmap
trigger, pre-1.0 timing, or milestone-close ownership would be easy to lose if
left only inside a primitive chapter.

## Close State

- `ToggleButton.checked` shipped as the M3 selected-state surface:
  dedicated `ToggleButton`, controlled one-way `checked`, background-colour
  selected visual, and author-composed exactly-one exclusion. No two-way
  binding, widget-owned state, group widget, equality expression, or generic
  appearance toggle shipped.
- The integrated Gallery shipped across Rust, C, and Zig hosts. Real images,
  thumbnail hit-testing, wheel/drag scrolling, modal focus, dynamic title /
  status, and runtime DPI-awareness remain out of M3.
- The DSL spec reached public-draft readiness in T8. T11 still owns the
  formal public-draft marker flip, promotion change-history entry,
  architecture status sync, ABI no-op confirmation, and CHANGELOG entry.

## Public-Draft Residuals

### PM-2 Grid Wrapper Rule

M3 ships Grid placement in two authoring forms: `Cell { ... }` and direct
`slot.*` placement on the child. This is a deliberately provisional pre-1.0
state, not a canonical-form decision.

Reopen when any of these occurs:

- a new container wants a wrapper form;
- a public code-construction API / builder is designed;
- custom containers or custom slot attributes are designed;
- the first non-layout parent data appears;
- Wasamo approaches 1.0.

The future decision should choose whether Grid converges toward keeping
`Cell`, dropping `Cell`, or a broader wrapper-rule model. Until then, public
prose must keep the two-form state visibly provisional.

### Author-Controllable Sizing (Problem B)

The accepted Vision Decision Record
[author-controllable-sizing-surface.md](../cross-milestone/decisions/author-controllable-sizing-surface.md)
assigns the roadmap responsibility: run a sizing design spike no later than
M5, preferably in M4, and retain an M6 ABI-freeze disposition backstop.

The owning spike must audit grammar / parser / checker impact, IR and runtime
layout impact, C ABI / host-construction impact, interaction with `aspect`,
Fill / Shrink defaults, Grid tracks, ZStack alignment, ScrollView viewport,
WrapPanel item sizing, and diagnostics for under-constrained combinations.

M3 does not reserve syntax, IR shape, or ABI shape for this surface. The
public draft should keep saying that M3 sizing is kind-default and that
explicit author sizing is unresolved pre-1.0 work.

### Default Alignment

Grid and ZStack share the `slot.*` placement namespace, but not default
alignment semantics:

- Grid defaults placed children to stretch through their resolved cell.
- ZStack defaults overlays to center within the union bounds.

T8's external-reader smoke found the asymmetry explainable as
container-owned semantics, so no B-3c revision procedure fired. Reopen if a
future public-draft / stabilization pass finds this to be a real
explicability debt, or if an application needs cross-container default
consistency.

### Placement Spelling And Bindability

M3 affirmatively kept the inherited kebab-case placement spellings
(`slot.h-align`, `slot.v-align`, `row-span`, `column-span`). Reopen during a
pre-1.0 naming / ergonomics pass or a public compatibility-policy pass.

Placement is constant per instance in M3. Binding RHS is rejected. Reopen only
when an app needs reactive placement; design that together with binding-target
machinery and child-slot effect lifecycle, not as a local `slot.*` addition.

## Selected-State Deferred Axes

These axes come from DD-M3-P8-001. M3 keeps them non-foreclosed but does not
reserve syntax.

| Axis | M3 disposition | Reopen trigger |
|---|---|---|
| Equality / single-discriminant selection | Deferred. M3 has no `==`, so tab exclusion is author-composed with per-tab bool states. | A future expression-grammar phase adds equality or a concrete app needs one-state exclusive selection. |
| Group-surface family | Deferred. No `RadioGroup`, `SegmentedControl`, or parent-owned exclusive selection. | A component needs selected value, group role, or exclusion semantics beyond author-written handlers. |
| Two-way binding | Deferred. `checked` is one-way and handler-driven. | A family-consistent two-way binding design opens; not merely because `ToggleButton` exists. |
| Widget-owned state | Deferred. `ToggleButton` does not self-toggle. | A narrow opt-in self-toggle is designed, or a family-level widget-owned state model is raised by Vision DR. |
| Generic Toggle / appearance | Deferred. M3 ships a dedicated `ToggleButton`, not a role / appearance split. | A future appearance, theming, or control-family phase compares role, appearance, and input semantics together. |

## M4 Residual Cluster

The Gallery intentionally uses Box + Text placeholders and Button-driven
controls. The following remain M4+ residuals, not M3 failures:

- real image widget / image loading;
- thumbnail hit-testing;
- wheel / drag scrolling and real scrollbar interaction;
- lightbox modal focus and input trapping;
- dynamic Window title / dynamic collection-count status;
- runtime DPI-awareness and DPI-localized layout evidence.

T10 still owns the owner human-visible smoke for the M3 state set. That smoke
does not convert the M4 residuals into M3 scope.

## Type-System And Host-State Notes

`TypedValue` remains deferred. M3 added the `bool` scalar binding path and the
`i32[]` / `string[]` / `bool[]` collection paths without needing a generic
runtime value union. Reopen when structured item data, record-like collection
items, or richer expression typing actually require it.

Host state write-back remains outside M3. The Gallery's tab exclusion is
authored in DSL handlers; C / Rust / Zig hosts load the same compiled UI and
do not mutate widget state from host code. Reopen host-state ownership when a
public host construction or imperative state API is designed.

## Public-Facing Phase Vocabulary Cleanup

Public docs and some compiler diagnostics still use internal milestone /
phase vocabulary such as `M3-Phase N` and phase-relative support wording.
This does not block M3 close, but it is a pre-1.0 public-facing consistency
residual: before 1.0, run a vocabulary pass that replaces internal phase
wording with draft-status / capability-status wording where appropriate. If
compiler diagnostics contain the same vocabulary, update docs and diagnostics
together so they do not drift.

## Phase-End Candidate Review

Phase 8's implementation log and retrospectives also contain local process
learnings: string-carried widget-kind catalogs, visible-desktop screenshot
capture, coordinate re-derivation after UI changes, and C/Zig Gallery build
ordering. T11 and the phase-end batch must decide which of those belong in
the phase-level `implementation/handoff.md`.

Do not silently promote all local mechanics into this milestone handoff. This
file should stay focused on durable public-surface and roadmap residuals.
