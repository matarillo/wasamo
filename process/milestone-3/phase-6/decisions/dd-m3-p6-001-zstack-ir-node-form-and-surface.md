# DD-M3-P6-001 — ZStack IR node form and author surface

**Status:** Proposed
**Phase:** M3-Phase 6
**AC:** A4 (ZStack layout primitive — sibling z-order by document order)

## Context

A4 ships an **overlay-dedicated** layout primitive. Phase 5 drew the
boundary explicitly: Grid is "1 cell 1 child", same-cell overlap is
**not** Grid's job — "overlay is ZStack's responsibility"
([../../phase-5/decisions/preamble.md](../../phase-5/decisions/preamble.md)).
ZStack is that primitive: a container whose children **occupy the same
overlap region** and paint in **document order** (later child on top).

This DD fixes **(i)** the IR node form (how ZStack is encoded in
`wasamo-ir` and how the runtime registers it) and **(ii)** the
author-facing `.ui` surface. The measure/arrange, z-order, and clip
*contract* is DD-M3-P6-002; this DD settles the carrier and surface so
that DD-002 has a shape to compute against.

The relevant end-state shapes:

- `IrNode { widget_type, props, bindings, handlers, children,
  kind_payload: Option<KindPayload> }`. `KindPayload` currently has
  one variant, `Grid { columns, rows }` (DD-M3-P5-001 carrier **c1**),
  used because Grid carries track-list domain data that does not fit
  `IrProp.value` (which stays strictly `IrLiteral`).
- The runtime widget catalog is a per-kind tag set: `Rectangle |
  VStack | HStack | Text | Button | Box | WrapPanel | ScrollView |
  Grid`. The pure containers (VStack / HStack / Box / WrapPanel) take
  their children **directly** as `children: Vec<Box<WidgetNode>>` with
  no wrapper node; Grid is the exception, wrapping each child in a
  `Cell` IR node kind that carries placement metadata.

ZStack is, like VStack / HStack / WrapPanel, a **pure overlap layout
container**: a child needs no per-child structured placement data to
participate in the overlap (each child simply occupies the overlap
region). The question is whether ZStack nonetheless needs a payload,
a wrapper, or any new IR vocabulary — or whether it rides the existing
generic `IrNode` machinery unchanged.

## Options

### Option A — Per-kind tag, direct children, no payload (recommended)

ZStack is a new per-kind tag in the runtime widget catalog and a
recognised `widget_type: "ZStack"` in the IR. Its children are the
node's ordinary `children: Vec<IrNode>` in document order, exactly
like VStack / HStack / WrapPanel. **No `KindPayload` variant, no new
`IrType`, no new `IrLiteral`, no wrapper node.** Author surface:

```
ZStack {
    Box { fill: #00000080 }        // scrim (bottom)
    Box { aspect: 4:3 Text { … } } // photo (above scrim)
    VStack { … }                   // caption / nav (top)
}
```

Document order = bottom-to-top z-order. Per-child alignment (DD-002)
is expressed as **optional attributes directly on each ZStack child**
(`h-align` / `v-align`), reusing the existing `IrProp` machinery
(ident literals), not via a wrapper.

### Option B — Per-kind tag with a `Cell`-style wrapper (`Layer`)

Mirror Grid: wrap each ZStack child in an IR-only `Layer` node kind
carrying per-child alignment / future z metadata. Author surface:

```
ZStack {
    Layer { h-align: stretch; Box { fill: #00000080 } }
    Layer { h-align: center; v-align: center; Box { … } }
}
```

### Option C — Reuse `Box` with an overlap mode / new `KindPayload`

No new widget kind; add an `overlap`/`stacking` mode to an existing
container (e.g. `Box { stacking: z; … }`) or carry stacking config in
a `KindPayload::ZStack { … }`.

## Comparison

| Axis | A (direct children) | B (`Layer` wrapper) | C (mode/payload) |
|---|---|---|---|
| New IR vocabulary | none (generic `IrNode` only) | new IR node kind `Layer` | new mode attr or `KindPayload` variant |
| Author ergonomics | minimal — children are direct, like VStack | verbose — one wrapper per child | overloads an existing widget's meaning |
| Consistency with M3 catalog | matches VStack/HStack/WrapPanel (pure containers take direct children) | matches Grid, but Grid wraps **because it needs placement data**; ZStack does not | breaks the "one widget kind = one layout contract" pattern |
| Per-child alignment | direct `h-align`/`v-align` attrs (existing `IrProp`) | wrapper attrs | mode-specific |
| Forward-compat (`z-index`, per-child clip) | additive child attrs later | wrapper is the natural home | muddied |
| Validation surface | ZStack attr allow-list + child attr allow-list | extra `Layer`-outside-`ZStack` rejection rule | mode-value validation |

Option B's wrapper earns its cost only when the wrapper carries data
the child cannot — that was true for Grid (`row`/`column`/`span`) but
is **false** for ZStack: overlap needs no per-child structured
placement. A `Layer` wrapper would be ceremony with no payload to
justify it, and would add a whole IR node kind plus its
"outside-parent rejection" rule for nothing. Option C overloads an
existing widget's contract (the exact anti-pattern the per-kind-tag
catalog avoids) and either introduces a stacking mode value to
validate or a `KindPayload` variant that, unlike Grid's, carries no
real data.

Option A keeps ZStack in the same shape as the other pure containers,
introduces **zero** new IR vocabulary, and leaves the alignment /
future-layering surface as additive child attributes — the lightest
shape that satisfies A4.

## Recommendation

**Option A.** ZStack is a per-kind tag `ZStack` in the runtime widget
catalog and `widget_type: "ZStack"` in the IR, taking its children
directly in document order. No `KindPayload` variant, no new `IrType`
or `IrLiteral`, no wrapper node.

Concrete decisions:

- **IR node:** `widget_type: "ZStack"`, `kind_payload: None`,
  `children` = the overlap layers in document order. ZStack carries no
  ZStack-level structured payload (no track lists, no stacking config).
- **Author surface:** `ZStack { <child>* }`; each `<child>` is any
  widget. Document order is bottom-to-top z-order (DD-M3-P6-002 fixes
  this normatively). Per-child alignment is the optional `h-align` /
  `v-align` attributes **on the child** (DD-M3-P6-002), lowered through
  the existing `IrProp` ident-literal machinery — no new value type.
- **ZStack attribute allow-list:** ZStack admits no Phase-6
  ZStack-level attributes (no `spacing`, no `padding`, no `z-index`,
  no `columns`/`rows`). Unknown ZStack attributes are rejected at
  `wasamoc check` and runtime `validate()` (DD-M3-P6-002 verification).
  Whether ZStack should later admit a background `fill` is **out of
  scope** — the scrim is a child `Box { fill: #RRGGBBAA }` (FD-G), not
  a ZStack attribute.
- **Runtime catalog:** `ZStack` registers as a runtime widget kind
  (a pure layout container, no intermediate Visual — DD-M3-P6-002),
  parallel to WrapPanel. It is **not** a `KindPayload` consumer.

`Cell` has a dual nature in Grid (IR node kind + lowering consumer);
ZStack has none of that — its children are real widgets that each
materialise a `WidgetNode` and a `Visual`, with the **1 WidgetNode =
1 Visual** convention intact.

## Forward-compat exposure

- **`z-index` / explicit layering.** Out of scope (paint order =
  document order, DD-M3-P6-002). If ever admitted, it lands as an
  additive child attribute (`Box { z-index: 2; … }`) on the existing
  `IrProp` machinery — no ZStack IR-shape change, no wrapper
  retrofit. Option A's direct-child surface keeps this open.
- **Per-child clip (`clip:` on a ZStack child).** Out of scope
  (DD-M3-P6-002 ships only the ZStack outer-bounds clip). Additive
  child attribute later; no shape change.
- **ZStack background `fill`.** Not introduced; the scrim is a child.
  A future ZStack-level `fill` would be an additive ZStack attribute
  (the allow-list grows), not a structural change.
- **Bindable children count.** ZStack children are static in Phase 6;
  iteration-generated children are Phase 7 (a `for` block as a ZStack
  member), reusing the structural control-flow family (DD-M3-P6-004
  forward-compat) — no ZStack-specific change.

## Technical risk re-evaluation

- **No new IR vocabulary** ⇒ no risk to the `IrProp.value` =
  `IrLiteral` invariant, no `KindPayload` `Eq`/`HashMap` churn, no new
  construction sites beyond the catalog tag. The R-C construction-site
  discipline (`kind_payload` explicit at every site) is unaffected —
  ZStack sites set `kind_payload: None`.
- **Catalog growth** is the same low-risk additive step as WrapPanel
  (Phase 3) and ScrollView (Phase 4): a new tag, a constructor, a
  lowering arm, a `validate()` arm.
- **Lightbox dependency:** A is sufficient for the lightbox overlay
  (scrim + photo + caption + nav as direct children), so A4's visible
  proof does not need B or C. Confirmed against FD-B.
