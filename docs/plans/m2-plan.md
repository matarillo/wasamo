---
milestone: M2
status: drafting
roadmap-anchor: ROADMAP.md#m2-alpha
adrs: []
created: 2026-05-02
---

# M2 Plan — Foundation Milestone

## Purpose

M1 proved the core hypothesis (external DSL × C ABI × Visual Layer) by
exercising the runtime side end-to-end, with hosts constructing widget
trees imperatively through the experimental C ABI layer. M2's purpose
is to **close the loop on the DSL side**: make `.ui` files actually
drive the runtime, with reactive state propagation, so that Hello
Counter in each language is written against the DSL rather than
reproducing the DSL by hand.

This redefines M2 from the original "Alpha" feature wishlist (Grid /
ScrollView / List / input / IME / AccessKit / VS Code / DSL spec
public draft) into a **foundation milestone** whose acceptance is
structural, not feature breadth. The Alpha-style feature work — now
including Grid layout and the DSL spec public draft — is deferred to
a later milestone whose label and scope will be settled in a
follow-up revision (see "Deferred to the next plan revision" below).

## Phase numbering

Phase numbers in this plan are **local to M2** (M2-Phase 1, 2, …).
M1's global Phase 1–8 numbering is not continued. ADR identifier
discipline under the new scheme is one of the questions deferred to
the next plan revision; pre-doc for each phase will settle the ADR ID
form at the time it is written.

## Acceptance criteria

- **A1.** `examples/counter/counter.ui` drives the running Hello
  Counter in C, Rust, and Zig — the M1 host-imperative trees in
  `examples/counter-{c,rust,zig}/` are replaced by hosts that load
  the DSL through the agreed wasamoc pipeline.
- **A2.** Reactive state propagation works without host-side
  property-set plumbing: `count++` in the host updates the visible
  label through the M2 reactive path, not through a manual
  `wasamo_set_property` call written by the application.
- **A3.** `wasamo-runtime` and the `wasamo` safe wrapper no longer
  share an rlib filename through the cdylib-shim split; the post-M1
  cleanup flagged in
  [DD-P7-002](../decisions/phase-7-language-bindings.md) is
  discharged.
- **A4.** The C ABI gains the tree-mutation primitives required by
  the reactive engine; the experimental layer's all-at-once
  constructors remain available but are no longer the only way to
  construct UI.

## Phase breakdown

The phases below are working hypotheses; each one's design questions
become a phase ADR at pre-doc time, per
[the decisions README](../decisions/README.md).

- **M2-Phase 1 — cdylib-shim cleanup.** Split DLL output from the
  rlib so `wasamo-runtime` can be renamed cleanly without the
  cargo#6313 filename collision. Pure infra; independent of the DSL
  track. Origin:
  [DD-P7-002 post-M1 implementation note](../decisions/phase-7-language-bindings.md).

- **M2-Phase 2 — wasamoc output format decision.** Resolve the
  question Phase 6 pre-doc explicitly deferred to M2: host-language
  codegen vs IR + runtime interpretation. Includes implications for
  binding-author workload and the feasibility of post-M2 hot-reload
  (the latter is out of M2 scope but is constrained by this
  decision).

- **M2-Phase 3 — Handler execution location.** Resolve the second
  Phase 6-deferred question: where DSL inline handler bodies
  (`clicked => { ... }`) execute. The decision interacts with
  M2-Phase 2 and with M2-Phase 4's ABI surface.

- **M2-Phase 4 — Tree-mutation primitives at the ABI surface.**
  Promote the operations the reactive engine needs (insert / remove
  / replace child; property batching) from runtime-internal to the
  stable-core C ABI. M1 deliberately deferred this — see
  [DD-P8 "Out of scope"](../decisions/phase-8-hello-counter.md).

- **M2-Phase 5 — Reactive engine.** State change → invalidate →
  relayout → render path, building on the queued-emission machinery
  from Phase 6 and the layout invalidation hooks from
  [DD-P8-002](../decisions/phase-8-hello-counter.md).
  Subtree-vs-root dirty granularity is in scope only insofar as M2
  acceptance demands; large-tree optimization stays an open question
  in [layout-engine note §3.4](../notes/layout-engine.md).

- **M2-Phase 6 — `.ui → runtime` lowering.** The end-to-end pipeline
  that consumes M2-Phases 2 / 3 / 5 and produces a running Hello
  Counter from `counter.ui`. Replaces the imperative tree
  construction in `examples/counter-{c,rust,zig}/`.

## Phase dependencies

```
M2-Phase 1   ── independent infra; lands any time

M2-Phase 2 ─┐
M2-Phase 3 ─┤
            ├─ M2-Phase 4 ─ M2-Phase 5 ─ M2-Phase 6
```

M2-Phases 2 and 3 are decision phases and can run in parallel; both
gate M2-Phase 4. M2-Phase 5 depends on 4. M2-Phase 6 depends on the
decisions (2, 3) and on 5.

## Acceptance ↔ phase mapping

| Acceptance | Phase(s) |
|---|---|
| A1 (`counter.ui` drives all three hosts) | M2-Phase 6 |
| A2 (reactive propagation, no host wiring) | M2-Phase 5, M2-Phase 6 |
| A3 (cdylib-shim cleanup) | M2-Phase 1 |
| A4 (tree-mutation ABI primitives) | M2-Phase 4 |

M2-Phases 2 and 3 are **decision phases** without a direct acceptance
hook; their outputs are ADR-shaped and feed M2-Phases 4 / 6.

## Out of scope (deferred to later milestones)

Items that appear in the current ROADMAP M2 paragraph but are **not**
in M2-as-foundation:

- Grid layout primitive (originally an M2 acceptance criterion;
  moved to the next milestone alongside the other layout work)
- DSL spec public draft (originally an M2 acceptance criterion;
  moved to the next milestone so it can reflect Grid and the rest
  of the post-foundation surface)
- ScrollView, List, and other layout primitives beyond Grid
- Input handling beyond M1's mouse-button hit-testing (full mouse
  semantics, keyboard, touch)
- IME via TSF (Japanese / CJK input)
- AccessKit / UIA accessibility integration
- VS Code extension (LSP, syntax highlighting, diagnostics)
- Theming, Mica / Acrylic, accent color follow-through
- Hot reload (interpreter mode)
- Performance target verification (<100 ms startup, <30 MB memory)
- Multi-window support
- Binding coverage beyond Hello-Counter level for Rust / Zig

These will be reallocated across post-M2 milestones in a follow-up
revision. This plan does not commit to that reallocation.

## Risks

- **Decision phases (M2-Phase 2, M2-Phase 3) blocking the DSL
  track.** If 2 / 3 do not converge in pre-doc, M2-Phase 6 cannot
  start. Mitigation: each decision phase is timeboxed to a single
  ADR review cycle; if the question does not converge, escalate to a
  VISION-level ADR (analogous to DD-V-001) rather than re-opening
  pre-doc indefinitely.

- **Reactive engine coupling with layout invalidation.**
  [DD-P8-002](../decisions/phase-8-hello-counter.md) installed a
  coarse "whole-window dirty" path. If M2-Phase 5 demands finer
  granularity for correctness (not performance), the layout-engine
  changes ripple beyond M2-Phase 5's nominal scope.

## Deferred to the next plan revision

The following questions were raised in the same conversation that
agreed the M2 redefinition, but the owner decided to settle M2 first
and rebalance the rest afterward. They are recorded here so the next
revision picks them up rather than re-discovering them:

- Placement of Grid layout and the DSL spec public draft (moved out
  of M2 in this revision; the receiving milestone is open)
- Renumbering / relabeling of post-M2 milestones (current
  ROADMAP M2 → new M3 Alpha, etc.)
- Goal rebalance across post-M2 milestones (IME, AccessKit, and
  VS Code LSP each plausibly warranting their own milestone;
  Mica / Acrylic positioning given VISION's identity-feature
  framing)
- C ABI freeze positioning (currently ROADMAP M4 = 1.0; question
  whether the freeze should move given M2 redefinition)
- Bindings list for 1.0 (currently lists Rust / Swift / Zig / Go;
  M1 verified only C / Rust / Zig — Swift / Go positioning open)
- Showcase-app placement (VISION §10.2 commits to "shipping
  showcase apps early"; not currently anchored in any milestone)
- ADR identifier discipline under M2-local phase numbering (e.g.
  whether IDs become `DD-M2-P<N>-<seq>` or another form), and the
  mapping note for prior references such as DD-V-001's "M5 scope"
  if milestone numbers shift
