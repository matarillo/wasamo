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
| Background color on themed widgets (`Button`, `ToggleButton`, `Text`) | unknown | M5 theming | [owner-intake](../docs/notes/owner-intake.md) |
| Reactive `fill` (color as a bound value) | unknown | `TypedValue` design space (row below) | [owner-intake](../docs/notes/owner-intake.md) |
| `TypedValue` + structured data (M-expr2b/3: computed color / dimension, `item.filename`) | unknown — `abi_spec` check due during M4 | dedicated pre-1.0 slot (M4–M5) | [expression-language-roadmap](../docs/notes/expression-language-roadmap.md), [typed-value-evaluator](../docs/notes/typed-value-evaluator.md) |
| Developer debug support (first step: `wasamoc` lint for silent 0-collapse) | no (lint); unknown (runtime diag channel) | M5 tooling | [developer-debugging](../docs/notes/developer-debugging.md) |
| Release distribution (artifacts, channels, discovery, versioning) | no | M4 showcase outreach; M6 1.0 | [release-distribution](../docs/notes/release-distribution.md) |
| Component extension model — freeze check only (registration ABI append-only-safe?) | unknown | post-1.0 + M6 disposition | [component-extension-model](../docs/notes/component-extension-model.md) |
| Anchored popover — declarative widget-anchored placement on top of the M4 top-layer surface (anchor reference, coordinate conversion, placement rule; gated on the [dsl-grammar Q1](../docs/notes/dsl-grammar.md) widget-id question) | unknown | M5 widget set (Menu / ComboBox are the canonical buyers) | [M4 framing](./milestone-4/requirements/framing.md) granularity decision (2026-07-09); [top-layer-overlays](../docs/notes/top-layer-overlays.md) candidate C |

## Disposition log

One dated line per milestone planning (§1.1), recording per-item
`take (milestone N)` / `hold` / `retire` with destination links, plus
the item count (the Option C growth falsifier per DD-V-028):

- **2026-07-28 — M4 planning (§1.1).** Item count entering this pass:
  **13** (the 12-item 2026-07-07 seed, plus anchored popover added
  2026-07-09 by the M4 framing granularity decision). Six taken, seven
  held, none retired; **7 items remain** — below the ~25-item growth
  falsifier, and the count fell rather than grew across this pass.
  The dispositions were agreed in the M4 §1.1 handoff review
  (2026-07-07 owner chat) and their landings became linkable when the
  [M4 target app spec](./milestone-4/requirements/spec.md) and the
  revised M4 criteria landed:

  | Item | Disposition | Landing |
  |---|---|---|
  | Host state boundary | `take (M4)` — core | [M4 AC](./_roadmap.md#m4-interaction-stack) "Host state boundary"; split across both apps in [spec](./milestone-4/requirements/spec.md) §2 本の役割分担 (supply/replace = gallery, write-back = inbox) |
  | Expression predicates (M-expr1/2a) | `take (M4)` — core | [M4 AC](./_roadmap.md#m4-interaction-stack) "Expression predicates"; proven by the gallery ([spec](./milestone-4/requirements/spec.md) §アプリ仕様 A) |
  | Top-layer overlays | `take (M4)` — core | [M4 AC](./_roadmap.md#m4-interaction-stack) "Top-layer overlays"; proven by the inbox item menu ([spec](./milestone-4/requirements/spec.md) §アプリ仕様 B4). Anchored placement stays in the pool as its own row |
  | Window config props | `take (M4)` — core | [M4 AC](./_roadmap.md#m4-interaction-stack) "Window config properties"; proven by the inbox dynamic title ([spec](./milestone-4/requirements/spec.md) §2 本の役割分担) |
  | `Image` widget | `take (M4)` — stretch, no AC | [spec](./milestone-4/requirements/spec.md) §M4 が開く機能面 (tier: 予備) — real thumbnails in the gallery. Falling out costs one line back into this pool |
  | Literal `fill` beyond `Box` | `take (M4)` — stretch, no AC | [spec](./milestone-4/requirements/spec.md) §M4 が開く機能面 (tier: 予備) — gallery only |
  | Background color on themed widgets | `hold` | M5 theming surface |
  | Reactive `fill` | `hold` | `TypedValue` design space |
  | `TypedValue` + structured data | `hold` | Kept out of M4 deliberately: the adopted second app caps items at a list of strings ([spec](./milestone-4/requirements/spec.md) §B4 の 2 つの上限). Its `ABI-bearing: unknown` is not deferred — the `abi_spec` cross-check is homework **during M4** ([framing](./milestone-4/requirements/framing.md) §検討ノートのケース分類と M4 期間中の宿題) |
  | Developer debug support | `hold` | M5 tooling |
  | Release distribution | `hold` | Only the minimal answer is consumed by M4 (contributors clone + build); the note itself stays closed |
  | Component extension model | `hold` | Post-1.0 + M6 freeze check |
  | Anchored popover | `hold` | Explicitly out of M4 ([framing](./milestone-4/requirements/framing.md) §M4 に入れないもの); canonical buyer is the M5 widget set |
