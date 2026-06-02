---
phase: M3-Phase 1
title: bool scalar binding
status: retired
adr: process/milestone-3/phase-1/2.designs/_index.md
plan: process/milestone-3/_plan.md
opened: 2026-05-19
---

# M3-Phase 1 — `bool` scalar binding: Progress

This is the live task list and execution log for M3-Phase 1. The
design decisions are frozen in
[m3-phase-1-bool-scalar.md](../decisions/preamble.md);
this file is mutable per
[plans/README.md §Phase progress file lifecycle](../README.md#phase-progress-file-lifecycle).

Task ordering follows the dependency direction
`wasamo-ir → wasamoc → wasamo-runtime → tests → host/spec`, so each
commit builds on a green workspace per
[CLAUDE.md §Commit rules](../../../CLAUDE.md). Items may be split,
reordered, or merged when implementation reveals a tighter ordering
— this list is the record of what actually happens, not a frozen
prediction.
