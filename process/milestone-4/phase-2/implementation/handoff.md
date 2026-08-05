---
title: M4-Phase 2 handoff
status: draft
source-phase: M4-Phase 2
---

# M4-Phase 2 — Handoff

> **Status: draft.** Seeded at the ADR-acceptance boundary, before T1,
> from consequences the decision set exposes rather than from
> implementation experience. Each row is re-confirmed or revised at phase
> close, when the T1–T13 carry-forward ledgers in [log.md](./log.md) are
> distilled into it, per
> [workflow.md](../../../procedures/workflow.md) and
> [retrospectives.md](../../../procedures/retrospectives.md).

## Carry-forward to later phases

| Item | Lands at | Re-trigger criterion |
|---|---|---|
| **A disabled Button does not end propagation.** It is a hit target and occludes what is beneath it, but having dispatched nothing it runs no handler — and "a handler that runs consumes the event" is the only terminator ([DD-M4-P2-001](../decisions/dd-m4-p2-001-event-routing-model.md), [DD-M4-P2-002](../decisions/dd-m4-p2-002-hit-testing-and-generic-click.md)). A click on a disabled Button inside a clickable container therefore reaches the container. This is a consequence of two accepted rules rather than a third rule, and it is normative in [dsl_spec.md §4.8 / §4.19](../../../../docs/dsl_spec.md); what is *not* decided is whether it is the behaviour wanted, because M4 has no case that exercises it | M5's official widget set, or M4-Phase 9 if its dialog composition puts a disabled control inside a clickable region | The first authored or widget-internal composition placing a disabled control inside a container that carries `clicked`. The alternative reading — a disabled control swallowing the event entirely, which is what HTML does — is a change to the consumption rule, not a local exception, so it arrives as a successor decision. Deciding it earlier would be deciding it without a consumer |

## Residuals and local residue

*(Filled at phase close.)*

## Verification closure

*(Filled at phase close.)*
