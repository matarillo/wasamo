---
phase: M3-Phase 2
title: Box layout primitive
status: retired
adr: process/milestone-3/phase-2/2.designs/_index.md
plan: process/milestone-3/_plan.md
opened: 2026-05-20
---

# M3-Phase 2 — Box layout primitive: Progress

This is the live task list and execution log for M3-Phase 2. The
design decisions are frozen in
[m3-phase-2-box-layout.md](../decisions/preamble.md);
this file is mutable per
[plans/README.md §Phase progress file lifecycle](../README.md#phase-progress-file-lifecycle).

Task ordering follows the dependency direction
`wasamo-ir → wasamoc → wasamo-runtime → tests → host/spec`, so each
commit builds on a green workspace per
[CLAUDE.md §Commit rules](../../../../CLAUDE.md). Items may be split,
reordered, or merged when implementation reveals a tighter ordering
— this list is the record of what actually happens, not a frozen
prediction.

The four pieces of A6 evidence the phase closes against are
enumerated in
[m3-phase-2-box-layout.md §Phase 2 verification closure](../decisions/preamble.md#phase-2-verification-closure-what-counts-as-a6-evidence).
Each T below cites the evidence item it discharges.
