# DD-M3-P6-004 — Conditional IR representation + runtime present/absent mechanism

**Status:** Proposed
**Phase:** M3-Phase 6
**AC:** A7 (conditional rendering grammar — binding drives the present /
absent state of a subtree)

## Context

DD-M3-P6-003 fixes the `.ui` surface (`if <bool-expr> { <member>* }`).
This DD fixes **(1)** how that construct is encoded in `wasamo-ir` and
**(2)** how `wasamo-runtime` makes the subtree present/absent when the
bound `bool` changes. It is the runtime backbone of A7 and the DD that
carries the framing's two thesis axes (FD-CR, originating in
[../../../../docs/notes/dsl-grammar.md Q6](../../../../docs/notes/dsl-grammar.md)):

- **(i) control-flow family extensibility** — the IR / runtime must
  extend to `else` / `switch` and to Phase 7 `for` without a
  one-off "`ConditionalSubtree`" dead end.
- **(ii) runtime identity / declared-tree vs entity-tree separation** —
  a lightweight **declared tree** (may be regenerated on state change)
  vs a lifetime-bearing **entity tree** (state / effect / layout
  object / focus / in-progress input held by identity), Flutter's
  Widget / Element / RenderObject split as reference point.

The implementation is the minimal `if <bool>`; the **design** is
evaluated against (i) and (ii).

Relevant end-state shapes (preamble §Context):

- `IrNode { widget_type, props, bindings: Vec<IrBinding>, handlers,
  children: Vec<IrNode>, kind_payload: Option<KindPayload> }`.
  `IrBinding { prop_name: String, expr: HandlerExpr }`. `KindPayload`
  derives `Eq`; `IrBinding`/`HandlerExpr` are `PartialEq` only.
- Grid's `Cell` is an **IR-only node kind** (`widget_type: "Cell"`):
  parsed and loaded, consumed by Grid's lowering to extract metadata,
  but **never registered as a runtime widget kind** — it materialises
  no `WidgetNode` and no `Visual` (DD-M3-P5-001). This is the
  established pattern for an IR construct that the runtime *interprets*
  rather than *renders*.
- Runtime: `WidgetNode { … children: Vec<Box<WidgetNode>>, bindings:
  Vec<EffectHandle> }` with `append_child` / `insert_child(index,
  child)` / `remove_child(index) -> Box<WidgetNode>` /
  `replace_child` — each moving both the `WidgetNode` and its `Visual`
  (and maintaining `attached`). Effects are owned by the hosting
  widget; subtree teardown disposes Effects structurally
  ([architecture.md §6.8.6](../../../../docs/architecture.md#686-effect-lifetime-dd-m2-p5-003--a)).
- The reactive seam already anticipates this work:
  `BindingTarget { WidgetProperty { node, prop }, /* M3+ adds
  ConditionalSubtree, ForLoopSubtree, … */ }`
  ([§6.8.7](../../../../docs/architecture.md#687-binding-registration-api-after-m2-dd-m2-p5-005-dd-m2-p6-007-dd-m2-p6-011-dd-m3-p1-007)),
  and §6.8.8 records "structural bindings (conditional / for-loop /
  list-rendered) add `BindingTarget` variants; subtree rebuilds Drop
  old Effects through the existing widget teardown path."
- The Three-Layer Tree Model (§9): **DSL tree** (`wasamoc` AST) /
  **View tree** (runtime `WidgetNode`, resolved properties) / **Visual
  tree** (Composition). "In M1 there is no reconciler."

## Options

### IR encoding of the conditional construct

- **IR-1 — structural control-flow IR node kind `If` (recommended).**
  A `widget_type: "If"` IR node, **IR-only** (like `Cell`): consumed
  by the loader, never a runtime widget. Its **condition** rides a
  single reserved `IrBinding` (`prop_name: "condition"`, `expr:
  HandlerExpr`); its **conditional children** are the node's
  `children`. `kind_payload: None`. Future `for` is a sibling IR-only
  node kind (`widget_type: "For"`); future `else` is an additional
  branch carried on the `If` node.
- **IR-2 — `KindPayload::Conditional { condition }`.** Carry the
  condition in a new `KindPayload` variant on a generic node.
  **Blocked by a type constraint:** `KindPayload` derives `Eq`, but
  `HandlerExpr` is `PartialEq` only, so a condition expression cannot
  live in `KindPayload` without removing `Eq` from the enum
  (a churn that touches Grid).
- **IR-3 — presence binding on the gated widget.** No new node;
  attach a reserved `IrBinding` ("present") to the single gated widget,
  approach-1-flavoured. Cannot express a multi-widget conditional
  subtree without a wrapper, and reads as a property (against FD-CR).

### Runtime present/absent mechanism

- **R-1 — `BindingTarget::ConditionalSubtree` + insert/remove
  (recommended).** Fill the reserved `BindingTarget` variant. The
  loader builds the conditional children **once into a detached
  template / builder**, registers a bool-typed Effect on the condition
  Signal whose body, on each evaluation, **inserts** the subtree into
  the parent at the recorded slot when true and **removes** it when
  false, using `insert_child` / `remove_child`.
- **R-2 — always build, toggle Visual visibility.** Build the subtree
  always; on false, set the Visuals invisible / detached but keep the
  `WidgetNode`s. **Rejected by FD-CR** (this is approach 1: structural
  absence is faked by a visibility property; effects keep running).

### Identity model on absent→present (the runtime-identity axis)

- **ID-1 — full rebuild, no identity preservation (recommended for
  Phase 6).** Absent = the entity subtree is destroyed (Effects
  disposed, `WidgetNode`s + `Visual`s dropped). Present-again =
  rebuilt fresh from the **declared tree** (the IR `If` node's
  children, which is stable across the toggle). No state retention, no
  keys.
- **ID-2 — Element-level reconciliation now.** Keep a persistent
  per-subtree identity ("Element") across absent→present, preserving
  in-progress state / focus / effect identity. Full Flutter-style
  Widget/Element/RenderObject reconciler.

## Comparison

### IR encoding

IR-2 is **blocked**: putting `HandlerExpr` in `Eq`-deriving
`KindPayload` would force dropping `Eq` from the enum, a gratuitous
churn on Grid for no benefit. IR-3 cannot represent a multi-widget
conditional subtree without a wrapper and frames presence as a widget
property (against FD-CR's "structural, not property" stance). IR-1
reuses the **exact** pattern Phase 5 validated for `Cell` — an IR-only
node kind the runtime interprets, not renders — keeps `IrProp.value`
strictly `IrLiteral`, carries the condition on the existing
`IrBinding`/`HandlerExpr` machinery, and is the only option where the
**family** (thesis axis i) reads cleanly: `If` today, `For` as a
sibling node kind (Phase 7), `else` as an extra branch on `If`. The
"control-flow node family" is literally a set of IR-only node kinds in
the same shape — exactly what the thesis asks for.

### Runtime mechanism

R-2 is the approach-1 anti-pattern the thesis rejects: it fakes
structural absence with visibility and leaves inner Effects running
(violating FD-CR axis ii and DD-M3-P6-005's "absent = disposed"
policy). R-1 uses the **already-reserved** `BindingTarget` slot and the
**already-existing** `insert_child` / `remove_child` + structural
Effect teardown — i.e. the architecture was pre-shaped for exactly this
(§6.8.7/§6.8.8). It is the minimal real structural mechanism.

### Identity model (thesis axis ii)

ID-2 (a full reconciler) is **out of proportion** to Phase 6: the
lightbox needs no state retention across close→open (the photo
placeholder is stateless; reopening fresh is correct), and a
reconciler is a large subsystem (keys, diffing, Element lifetimes) that
M3 has no driver for. But the thesis is explicit that the *design* must
not foreclose it. ID-1 satisfies this precisely **because the IR `If`
node is the stable declared tree**: the declaration persists across the
toggle (it is in the IR, never destroyed), and only the **entity
subtree** (WidgetNodes/Visuals/Effects) is destroyed and rebuilt. That
is the declared-tree / entity-tree separation in its base, un-keyed
form — the Widget (declared) persists, the Element/RenderObject
(entity) is recreated. A future reconciler adds an identity layer
**between** the stable declared `If` node and the entity subtree
(keys, state carry-over) **without changing the IR** — the exact
forward path the thesis wants. So ID-1 is not a shortcut that blocks
ID-2; it is ID-2's un-reconciled base case.

## Recommendation

**IR-1 + R-1 + ID-1.**

### IR encoding (IR-1)

- The `if` construct lowers to an **IR-only node kind** `widget_type:
  "If"`:
  - `bindings`: exactly one reserved `IrBinding` with
    `prop_name: "condition"` and `expr` = the lowered condition
    (`HandlerExpr::BoolLit` or `HandlerExpr::BoolPropRead`, per
    DD-M3-P6-003's E1 vocabulary);
  - `children`: the conditional subtree (the members inside the
    block), in document order;
  - `props`: empty; `handlers`: empty; `kind_payload: None`.
- The `If` node is **not** registered as a runtime widget kind — like
  `Cell`, it materialises no `WidgetNode` and no `Visual`. The loader
  consumes it to build the conditional binding (R-1).
- The textual IR grammar (`dsl_spec.md` §8.5) gains an `If` node form
  carrying its condition binding and children; emit/load roundtrips
  preserve both (verification closure item 3).
- Loader defense-in-depth (dual-gate with `wasamoc check`,
  DD-M3-P6-003): an `If` node whose condition binding is missing,
  non-bool, or unresolved, or that appears in a position where a
  conditional child is not admitted, surfaces `WASAMO_ERR_IR_MALFORMED`.

### Runtime mechanism (R-1)

- Fill `BindingTarget::ConditionalSubtree { parent: WidgetId, slot:
  ChildSlot }` (exact field set finalised in implementation; `slot`
  records the conditional block's stable position among the parent's
  members).
- The loader, when it encounters an `If` node, **builds the
  conditional children once** (resolving their own props/bindings into
  a detached subtree builder keyed to the declared children) and
  registers a **bool Effect** on the condition. On each evaluation:
  - **false → true:** build a fresh entity subtree from the declared
    children and `insert_child` it at the recorded slot;
  - **true → false:** `remove_child` the subtree (dropping it, which
    disposes its Effects via the structural teardown — DD-M3-P6-005);
  - the first evaluation establishes the initial presence (a `false`
    initial condition inserts nothing).
- **Slot bookkeeping:** the conditional subtree occupies a recorded
  position among the parent's children. Phase 6's minimal rule: the
  slot is computed from the count of preceding *materialised* siblings
  (static widgets + any preceding present conditional blocks) so
  re-insertion lands at the correct index. For the lightbox the
  conditional is the top (last) child of the root container, so the
  slot is the simplest case; the rule is written to be correct for a
  conditional with static siblings on both sides, and the
  multi-conditional-sibling interaction is pinned by an integration
  test (verification closure item 4).

### Identity model (ID-1)

- **Absent = destroyed, present = rebuilt.** No state retention across
  absent→present; no `key:` attribute; no Element-level identity layer
  in Phase 6.
- The **declared tree is stable**: the IR `If` node and its children
  description persist across every toggle; only the entity subtree is
  recreated. This is documented in `architecture.md` §9 as the
  declared-tree / entity-tree separation in its base un-keyed form, so
  a future reconciler (keys, state carry-over, Phase 7 `for` item
  identity) is an additive identity layer with **no IR change**
  (forward-compat below).

## Forward-compat exposure

- **`else` / `switch`.** `else` is an additional branch carried on the
  same `If` node (an additional `children` group + branch condition);
  `switch` is a sibling IR-only node kind. Both reuse R-1's
  insert/remove machinery — the present/absent of a branch is the same
  operation as the present/absent of the whole `If`.
- **`for` (Phase 7 iteration).** A sibling IR-only node kind
  (`widget_type: "For"`) filling `BindingTarget::ForLoopSubtree`
  (the other reserved slot). It needs **keyed identity** (item
  reorder / state retention), which is the first real driver for the
  Element-level identity layer ID-2 defers. Because the declared tree
  (the `For` node + item template) is stable and ID-1 already
  separates declared from entity, the identity layer lands between
  them without an IR change.
- **Approach 3 (host-language constructs).** A future language-internal
  DSL lowers its own `if`/`switch`/loop into the **same** IR-only
  control-flow node kinds and the **same** `BindingTarget` structural
  seam — the thesis requirement that approach 3 stay reachable. The
  runtime mechanism is surface-agnostic by construction (it consumes
  IR `If`/`For` nodes, not `.ui` syntax).
- **Subtree-grain layout dirty.** Phase 6 inserts/removes whole
  subtrees, which dirties layout; this rides the existing
  whole-window dirty path (DD-P8-002), with subtree-grain invalidation
  remaining the open question in
  [layout-engine note §3.4](../../../../docs/notes/layout-engine.md)
  — unchanged by Phase 6.

## Technical risk re-evaluation

- **No new IR scalar/literal type**; the `If` node reuses the existing
  `IrBinding`/`HandlerExpr` machinery and the `Cell`-style IR-only
  node-kind pattern, so the `IrProp.value = IrLiteral` and
  `KindPayload: Eq` invariants are untouched (IR-2's `Eq` problem is
  avoided).
- **The reactive seam was pre-shaped** (`BindingTarget` reserved
  variant, structural Effect teardown §6.8.6/§6.8.8), so R-1 fills a
  documented slot rather than inventing a mechanism — the lowest-risk
  path, and the M2/Phase-5 architecture notes anticipated it.
- **Slot bookkeeping is the real risk surface.** A conditional block
  with static siblings on both sides, or two adjacent conditionals,
  must reinsert at the correct index. Mitigation: the minimal
  preceding-materialised-siblings rule, pinned by an integration test
  (verification closure item 4) covering (a) conditional as last
  child, (b) conditional with static siblings on both sides, (c) two
  sibling conditionals toggled independently.
- **Drain interaction** — inserting a subtree creates fresh Effects
  that must run to initialise their properties before quiescence; this
  is the DD-M3-P6-005 drain contract, verified there. The risk that a
  freshly-inserted Effect is *not* drained in the same outermost drain
  is the load-bearing concern and is pinned by the item-4 drain test.
- **ID-1 vs ID-2 scope** — shipping the un-keyed base case avoids a
  large reconciler subsystem with no M3 driver, while the stable
  declared tree keeps ID-2 reachable; the risk of "painting into a
  corner" is mitigated by the IR `If` node being approach- and
  identity-neutral (it describes structure, not lifetime).
