---
title: M3-Phase 5 Grid surface options
status: draft
target-phase: M3-Phase 5
role: supplemental requirement index
---

# Grid surface options

This directory contains supplemental owner-alignment notes for
[../framing.md](../framing.md). The framing document remains the phase
requirements SSOT; these files expand the `.ui` writing style,
ecosystem contrast, and future-extension implications for each candidate
surface.

No file in this directory is an ADR recommendation. The ADR should
compare the five surfaces in the canonical order below:

1. [Surface A — track-list + direct child placement](./surface-a-direct-placement.md)
2. [Surface A2 — track-list + placed `Cell` wrapper](./surface-a2-placed-cell.md)
3. [Surface B — pure structural `Row` / `Cell`](./surface-b-structural-row-cell.md)
4. [Surface D — Grid columns + structural rows](./surface-d-grid-columns-structural-rows.md)
5. [Surface C — definition nodes + structural rows](./surface-c-definition-nodes.md)

## Comparison Axes

The same five axes used in `framing.md` should guide review:

- `.ui` author taste
- spanning
- shared track sizing
- future iteration
- component-extension-model impact
