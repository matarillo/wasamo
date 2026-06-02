# Phase 8 — Hello Counter Sample × 3 Languages: Architecture Decisions

**Phase:** 8 (Hello Counter sample × C / Rust / Zig — final M1 deliverable)
**Date:** 2026-05-01
**Status:** Accepted (2026-05-01)

## Context

Phase 8's acceptance criterion comes directly from
[VISION §7 M1](../../VISION.md#7-roadmap--milestones) and
[process/_roadmap.md M1](../../../_roadmap.md#m1-proof-of-concept):
**"Hello Counter example runs in three languages: C, Rust, and Zig."**

The runtime ([`wasamo-runtime`](../../wasamo/)), the C ABI
([`bindings/c/wasamo.h`](../../bindings/c/wasamo.h),
[`docs/abi_spec.md`](../../../../docs/abi_spec.md)), and the three bindings
([`bindings/c/`](../../bindings/c/),
[`bindings/rust/`](../../bindings/rust/),
[`bindings/zig/`](../../bindings/zig/)) all landed in Phases 6–7.
Phase 8 consumes them: each binding gets one host-language
"counter" program that reproduces [`examples/counter/counter.ui`](../../examples/counter/counter.ui).

The roadmap Phase 8 task list ([process/_roadmap.md M1](../../../_roadmap.md#m1-proof-of-concept))
has eight items. Per
[Pre-doc discipline](../../../README.md) those are
working hypotheses; this ADR revisits them against the acceptance
criterion. Of the questions surfaced, two warrant ADR-level
record:

1. **DD-P8-001** — How `examples/counter/counter.ui` relates to the
   three host programs. The other M1 framing documents
([abi_spec §5.1](../../../../docs/abi_spec.md#51-what-m1-experimental-verifies-and-what-it-does-not),
   [VISION §7 M1](../../VISION.md#7-roadmap--milestones)) already
   carve out the M1/M2 split; Phase 8 is where that split first
   becomes visible to end users, so the application warrants one
   explicit decision and a few small upstream wording adjustments.
2. **DD-P8-002** — A runtime change Phase 8 forces. Property
   updates that change a widget's intrinsic size do not currently
   trigger re-layout; Hello Counter's `Count: N` text becomes
   visually stale after `N` grows past one digit. This is a
   permanent runtime addition, not a Phase 8 workaround.

The remaining items from the Phase 8 exploration — `window_create`
signature kept as-is (status quo); string-lifetime clarification
(documentation fix to abi_spec §4.3, not a decision); Quick Start
language (C, on grounds of the project's "C ABI first" framing);
CI builds all three counter examples to release-build success
(operational); release tagged `v0.1.0` (sole semver-clean option
that keeps the M2/M3/M4 = 0.2/0.3/1.0 mapping legible) — are not
ADR-shaped. They are recorded in the Phase 8 ROADMAP entry, in
PR descriptions, and in the affected docs (READMEs, `abi_spec.md`,
CI workflow). See
[Option enumeration discipline](../../../README.md) for why we keep
ADRs scoped to substantive choices rather than every Phase 8
sub-task.

---

## Out of scope for this ADR

- M2 wasamoc codegen format
- M2 reactive engine design
- Tree-mutation primitives at the ABI surface
- Any new ABI surface beyond what Phase 6/7 already shipped
  (DD-P8-002 is a runtime-internal change; no header changes)
- Multi-window scenarios (M5)

---

## Revision history

| Version | Date       | Notes                             |
|---------|------------|-----------------------------------|
| 0.1     | 2026-05-01 | Initial draft, Accepted same session |
