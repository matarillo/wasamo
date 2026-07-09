# Pre-1.0 candidate pool

Triaged items the owner wants before 1.0 that are not (yet) assigned
to a milestone. Entries carry **no acceptance criteria** and are not
commitments; they exist so milestone planning (workflow §1.1) and the
M6 freeze decision see them. Governing rules (entry criterion, tags,
lifecycle, per-planning disposition duty) are in
[DD-V-028](./cross-milestone/decisions/pre-1.0-candidate-pool.md);
`process/_roadmap.md` carries a stub section between M5 and M6
pointing here.

Undifferentiated wishes are **not** listed here — they stay in
[docs/notes/owner-intake.md](../docs/notes/owner-intake.md) until
triaged. One table row per item is a hard bound: an item that needs
more than a row needs its own note, linked from the row. Items leave
the pool only by milestone adoption, explicit move to Post-1.0, or
rejection — never silently; every `take` / `retire` disposition line
links its landing (destination-link rule).

## Items

Preliminary seed from a full `docs/notes/` sweep (2026-07-07,
agent-triaged; to be confirmed at M4 planning §1.1). Entries are
capability desires or freeze-relevant dispositions, not open design
questions (those stay in their notes):

| Item | ABI-bearing | Leans | Origin |
|---|---|---|---|
| Literal `fill` on layout containers beyond `Box` | no | small additive | [owner-intake](../docs/notes/owner-intake.md) |
| Background color on themed widgets (`Button`, `ToggleButton`, `Text`) | unknown | M5 theming | [owner-intake](../docs/notes/owner-intake.md) |
| Reactive `fill` (color as a bound value) | unknown | `TypedValue` design space (row below) | [owner-intake](../docs/notes/owner-intake.md) |
| Host state boundary: host-supplied initial state, host write, in-out write-back | **yes** | M4 (TextField write-back); M6 backstop | [host-state-boundary](../docs/notes/host-state-boundary.md) |
| Expression predicates + scalar calc (M-expr1/2a: `if count > 0`, `labels[i]`) | no | early M4 | [expression-language-roadmap](../docs/notes/expression-language-roadmap.md) |
| `TypedValue` + structured data (M-expr2b/3: computed color / dimension, `item.filename`) | unknown — `abi_spec` check pending | dedicated pre-1.0 slot (M4–M5) | [expression-language-roadmap](../docs/notes/expression-language-roadmap.md), [typed-value-evaluator](../docs/notes/typed-value-evaluator.md) |
| Top-layer overlays (popover / dropdown / tooltip / menu) | unknown | M4 input-focus adjacency | [top-layer-overlays](../docs/notes/top-layer-overlays.md) |
| Window config props: dynamic title, initial window size, `WindowConfig` | unknown | M4 multi-window | [dsl-grammar Q2](../docs/notes/dsl-grammar.md) |
| `Image` widget (asset / decoder surface) | unknown | M5 widget set | [layout-engine §3.3](../docs/notes/layout-engine.md) |
| Developer debug support (first step: `wasamoc` lint for silent 0-collapse) | no (lint); unknown (runtime diag channel) | M5 tooling | [developer-debugging](../docs/notes/developer-debugging.md) |
| Release distribution (artifacts, channels, discovery, versioning) | no | M4 showcase outreach; M6 1.0 | [release-distribution](../docs/notes/release-distribution.md) |
| Component extension model — freeze check only (registration ABI append-only-safe?) | unknown | post-1.0 + M6 disposition | [component-extension-model](../docs/notes/component-extension-model.md) |
| Anchored popover — declarative widget-anchored placement on top of the M4 top-layer surface (anchor reference, coordinate conversion, placement rule; gated on the [dsl-grammar Q1](../docs/notes/dsl-grammar.md) widget-id question) | unknown | M5 widget set (Menu / ComboBox are the canonical buyers) | [M4 framing](./milestone-4/requirements/framing.md) granularity decision (2026-07-09); [top-layer-overlays](../docs/notes/top-layer-overlays.md) candidate C |

## Disposition log

One dated line per milestone planning (§1.1), recording per-item
`take (milestone N)` / `hold` / `retire` with destination links, plus
the item count (the Option C growth falsifier per DD-V-028):

- _none yet — first pass due at M4 planning (§1.1)._
