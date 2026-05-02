# Changelog

All notable shipped milestones for Wasamo. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) at
milestone granularity (see
[DD-V-013](./docs/decisions/vision-doc-system.md#dd-v-013--changelog-granularity-and-length-control)).
Per-phase decisions live in
[docs/decisions/](./docs/decisions/); per-release notes live in
[GitHub Releases](https://github.com/matarillo/wasamo/releases).

This file records what has shipped. For what is planned, see
[ROADMAP.md](./ROADMAP.md). For the current state of work, see
the **Status** section of [README.md](./README.md).

## [v0.1.0] — 2026-05-01 — M1: Proof of Concept

Validated the core hypothesis: external DSL × C ABI × Visual
Layer. VStack / HStack / Text / Button / Rectangle render through
the Visual Layer with DWM compositor independence verified, the
minimal C ABI (`wasamo.h`) is shaped as a stable core plus an M1
experimental layer, and Hello Counter runs end-to-end in C, Rust,
and Zig (host-imperative; the `.ui → runtime` lowering is M2).

Decisions: Phase 0–8 ADRs in
[docs/decisions/](./docs/decisions/) (`DD-P2-*` … `DD-P8-*`,
`DD-V-001` … `DD-V-004`).
Release: [v0.1.0](https://github.com/matarillo/wasamo/releases/tag/v0.1.0).

## Document system

This project's document conventions changed on 2026-05-02 alongside
M1 shipping. Acceptance criteria live in
[ROADMAP.md](./ROADMAP.md), thesis-level framing in
[VISION.md §7](./VISION.md#7-roadmap), shipped milestones here, and
in-flight work in the active plan under
[docs/plans/](./docs/plans/). Rationale:
[DD-V-010..016](./docs/decisions/vision-doc-system.md).
