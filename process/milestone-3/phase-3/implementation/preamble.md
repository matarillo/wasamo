---
phase: M3-Phase 3
title: WrapPanel layout primitive
status: retired
adr: process/milestone-3/phase-3/2.designs/_index.md
plan: process/milestone-3/_plan.md
opened: 2026-05-21
---

# M3-Phase 3 — WrapPanel layout primitive: Progress

This is the live task list and execution log for M3-Phase 3. The
design decisions are frozen in
[m3-phase-3-wrap-panel.md](../../decisions/m3-phase-3-wrap-panel.md);
this file is mutable per
[plans/README.md §Phase progress file lifecycle](../README.md#phase-progress-file-lifecycle).

Task ordering follows the dependency direction
`wasamoc → wasamo-runtime → tests → host/spec`, so each commit
builds on a green workspace per
[CLAUDE.md §Commit rules](../../../CLAUDE.md). Phase 3 introduces
**no new parser grammar**: `wasamoc`'s parser already accepts the
generic `IDENT "{" ... "}"` widget-declaration shape and the
generic `IDENT ":" expr` property-bind shape, so WrapPanel and its
three attributes traverse the existing surface unchanged. Phase 3
likewise introduces **no new `IrType`, no new `IrLiteral` variant,
no new `PropertyValue` variant, no new `LayoutError` variant**
(DD-M3-P3-001 / DD-M3-P3-003 / DD-M3-P3-004 / DD-M3-P3-005). The
WrapPanel-shaped IR surfaces as a new `widget_type` value plus
three new `IrProp` names on the generic `IrNode`. Items may be
split, reordered, or merged when implementation reveals a tighter
ordering — this list is the record of what actually happens, not
a frozen prediction.

The five pieces of A3 evidence the phase closes against are
enumerated in
[m3-phase-3-wrap-panel.md §Phase 3 verification closure](../../decisions/m3-phase-3-wrap-panel.md#phase-3-verification-closure-what-counts-as-a3-evidence).
Each T below cites the evidence item it advances or discharges.
