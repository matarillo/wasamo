# Wasamo Roadmap

Milestones are defined by **acceptance criteria**, not dates. This
file is the SSOT for those criteria
([DD-V-010](./docs/decisions/vision-doc-system.md#dd-v-010--acceptance-criteria-ssot)).
For thesis-level framing see [VISION.md §7](./VISION.md#7-roadmap).
For shipped milestones see [CHANGELOG.md](./CHANGELOG.md). For the
current state of work see the **Status** section of
[README.md](./README.md).

Phase structure for the active milestone lives in its plan under
[docs/plans/](./docs/plans/), not here
([DD-V-016](./docs/decisions/vision-doc-system.md#dd-v-016--plan--roadmap-commit-flow-redefinition)).
Per-phase design decisions are recorded as ADRs in
[docs/decisions/](./docs/decisions/); the pre-implementation
discipline is described in
[docs/decisions/README.md](./docs/decisions/README.md#pre-doc-discipline).

---

## M1: Proof of Concept ✅ shipped 2026-05-01 (v0.1.0)

See [CHANGELOG entry](./CHANGELOG.md#v010--2026-05-01--m1-proof-of-concept)
and [docs/decisions/](./docs/decisions/) for the per-phase ADRs.

## M2: Foundation

**Goal:** close the loop on the DSL side — make `.ui` files actually
drive the runtime, with reactive state propagation, so Hello Counter
in each language is written against the DSL rather than reproduced
by hand through the experimental C ABI.

**Acceptance criteria**

- `examples/counter/counter.ui` drives the running Hello Counter
  in C, Rust, and Zig — the M1 host-imperative trees are replaced
  by hosts that load the DSL through the agreed wasamoc pipeline
- Reactive state propagation works without host-side property-set
  plumbing: `count++` in the host updates the visible label
  through the M2 reactive path, not a manual `wasamo_set_property`
- The cdylib / rlib filename collision flagged in
  [DD-P7-002](./docs/decisions/phase-7-language-bindings.md) is
  discharged
- The C ABI gains the tree-mutation primitives required by the
  reactive engine; the experimental layer's all-at-once
  constructors remain available but are no longer the only way
  to construct UI

Phase breakdown, dependencies, and progress live in
[docs/plans/m2-plan.md](./docs/plans/m2-plan.md).

## M3: DSL surface

**Thesis:** the DSL is expressive enough to write real layouts, and
is published as a stable public draft.

**Acceptance criteria**

- Grid layout primitive
- ScrollView primitive
- List primitive
- DSL specification first public draft (covers M2 + M3 surface;
  reserves syntax for material — see M4 — without committing to its
  rendering semantics)

## M4: Interaction stack

**Thesis:** input, multi-window, text input, and accessibility share
a focus model; they ship together so the focus model is settled
once. Wasamo's identity feature (Mica/Acrylic) becomes demonstrable
from this milestone, and the first contributor-facing showcase ships
here.

**Acceptance criteria**

- Input handling: keyboard, mouse, touch; focus model and event
  routing
- Multi-window support (per-window state, cross-window focus).
  Included pre-1.0 because its ABI implications are cross-cutting
  and an append-only post-freeze surface cannot accommodate them
- TextField widget (minimum editable text widget; required by IME
  verification)
- IME via TSF (Japanese / CJK input)
- AccessKit / UIA integration
- Mica / Acrylic root-window backdrop; system accent color
  follow-through (initial — full theming surface is M5)
- First showcase application — sufficient to demonstrate Wasamo
  identity for contributor outreach, even if rough around
  polish-level details

## M5: Identity & tooling

**Thesis:** Wasamo looks like Wasamo by default, and authoring `.ui`
is a first-class editor experience.

**Acceptance criteria**

- Full theming surface (light / dark, accent propagation through
  widgets, type ramp coverage)
- Official widget set (CheckBox, ComboBox, Menu, and the rest
  beyond TextField)
- VS Code extension (LSP, syntax highlighting, diagnostics). The
  VS Code work may begin in parallel any time after M3's DSL spec
  public draft is agreed; M5 is its acceptance gate, not its
  earliest start

## M6: 1.0 — C ABI stabilization

**Thesis:** the ABI is settled, performance targets are met, a
polished showcase ships, and SemVer applies.

**Acceptance criteria**

- C ABI freeze; SemVer applies from this point
- Public backward-compatibility commitment
- Performance targets: <100 ms cold start, <30 MB memory,
  single-digit-MB binaries
- Polished showcase application (production-grade, distinct from
  M4's contributor-outreach showcase)
- C / Rust / Zig bindings mature. Swift and Go bindings are out of
  scope for 1.0; they are welcomed as community-prototyped
  bindings post-1.0 (see [VISION §11](./VISION.md#11-how-to-contribute))

## Post-1.0

- Hot reload (interpreter mode during development) — feasibility
  depends on the wasamoc output format chosen in M2-Phase 2
- Higher-level animation DSL (the public property-change animation
  API deferred from Phase 5; see
  [DD-V-001](./docs/decisions/vision-m1-acceptance-criteria.md))
- Advanced layout (LazyList, CollectionView)
- System tray and notification integration
- MSIX packaging integration
- Swift / Go bindings (community-maintained)
