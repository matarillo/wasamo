# DD-M3-P6-004 — Conditional IR representation + runtime present/absent mechanism

**Status:** Proposed
**Phase:** M3-Phase 6
**AC:** A7 (conditional rendering grammar — binding drives the present /
absent state of a subtree)

## Context

DD-M3-P6-003 fixes the `.ui` surface (`if <bool-expr> { <widget-child> }`
— a single widget child in Phase 6). This DD fixes **(1)**
how that construct is encoded in `wasamo-ir` and
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

## Decision dependency summary

This DD sits in two cross-DD bundles (full phase map: preamble
§Cross-DD decision dependencies):

- **Owns — Control-flow IR shape.** The IR-encoding choice (**O1**
  member-level `children: Vec<IrMember>`, recommended; **O2**
  branch-node fallback) is the schema DD-M3-P6-003's surface lowers
  into, DD-M3-P6-005's effect teardown rides, and dsl_spec §8.5 /
  architecture §6.8/§9 document. This is the consequential
  owner-decision fork of the phase.
- **Consequence-of — Conditional body shape (owned by DD-M3-P6-003).**
  The **Conditional insertion granularity** sub-issue here is the
  runtime arm of that bundle: **IG-1** (single-slot insert/remove)
  pairs with DD-M3-P6-003 **B1**, **IG-2** (child-range) with **B2**.
  The body cardinality DD-M3-P6-003 decides determines whether this DD
  moves one child or a range; the Identity model (ID-1) is unaffected by
  that choice.

Recommended: **O1 + IG-1** (IG-1 pairing with DD-M3-P6-003 B1).

## Sub-issues

- **IR encoding of the conditional construct**: what the structural
  shape of control flow is in the IR — the schema `else` / `switch` /
  `for` / a future host-language DSL must all fit.
- **Runtime present/absent mechanism**: how the runtime makes the
  subtree appear and disappear when the bound `bool` changes.
- **Conditional insertion granularity**: whether the runtime
  inserts/removes a **single child** or a **child range**, pairing with
  DD-M3-P6-003's conditional-body-shape choice.
- **Identity model on absent→present**: whether re-appearing a subtree
  restores prior state — author-visible semantics that go into the spec.

## IR encoding of the conditional construct

The axis here is **not** "where do we stash the condition" but **what
is the structural shape of control flow in the IR** — because the IR
schema is what `else` / `switch` / `for` / a future host-language DSL
will all have to fit. The options span from "change the IR schema so
control flow is first-class" (O1/O2) to "reuse the existing widget-node
slot" (O3) to "stash it on a generic node" (O4/O5). The `Eq` derive on
`KindPayload` is treated as a **real cost to weigh**, not a blocker
that auto-disqualifies a schema change.

### Options

- **O1 — member-level structural IR.** Change `IrNode`'s `children`
  from `Vec<IrNode>` to `Vec<IrMember>`, where

  ```
  enum IrMember {
      Widget(IrNode),
      ControlFlow(ControlFlowNode),
  }
  enum ControlFlowNode {
      If { branches: Vec<Branch> },   // Phase 6: exactly one Branch, no else
      // future: Switch { subject, arms }, For { binding, body }, …
  }
  struct Branch { condition: HandlerExpr, body: Vec<IrMember> }
  ```
  - Control flow is a **first-class member-level construct**, not a
    widget. `else` is an additional `Branch`; `switch` is a
    `ControlFlowNode` variant with arms; `for` is a variant with a body.
    The condition rides `HandlerExpr` (no `IrProp`/`IrLiteral` change).
    Phase 6 ships only the single-`Branch` `If` variant.
  - What you gain: control flow is first-class; `else` / `switch` /
    `for` fit natively (add a `Branch` / a variant); high approach-3
    reachability; the declared/entity separation (axis ii) is explicit
    at the schema level; **no `Eq` impact** (sum type, derive as
    needed); **zero migration risk** when the family grows.
  - What you give up: the **largest single change** in the phase — every
    `children` construction / traversal site moves to `IrMember`
    (surfaced at compile time by the no-`Default` discipline), and the
    textual IR grammar (§8.5) gains a control-flow member production.
    Paid **once, now**, while `if` is the only variant.

- **O2 — distinct control-flow node carried in `children`, with a
  branch-list payload.** Keep `children: Vec<IrNode>`, but represent
  control flow as a distinguished IR node carrying a real `branches:
  Vec<Branch>` structure (in a dedicated field or payload).
  - Control flow is still *shaped like* a node in the children vector,
    but it is not a widget kind and it carries first-class branches.
  - What you gain: the branch-list family-fit (`else` = branch, `switch`
    / `for` = variants) **without** the full `children` re-type; a
    medium-sized change.
  - What you give up: the branch list holds `HandlerExpr`, so the
    carrying type cannot derive `Eq` — dropping `Eq` from that type (or
    from `KindPayload` if reused) is the real, bounded cost; control
    flow stays shaped like a node in the children vector (the category
    is less clean than O1's sum type).

- **O3 — `widget_type: "If"` reusing the generic `IrNode` (the easy
  seam).** An IR-only node kind (like `Cell`): condition on a reserved
  `IrBinding`, the one branch in `children`, `kind_payload: None`.
  - What you gain: the **smallest change**, no schema edit; the seam the
    existing architecture invites (the `Cell` IR-only-node pattern, the
    reserved `BindingTarget` slot).
  - What you give up: a single `children: Vec<IrNode>` has **no place
    for a second (`else`) branch or `switch` arms** — they would need a
    per-convention hack, so the family does not fit cleanly and a
    migration to O1/O2 is **likely** when the family grows (high
    migration risk).

- **O4 — `KindPayload::Conditional { condition }`.** Stash the
  condition in a new `KindPayload` variant on a generic node.
  - What you gain: a small change.
  - What you give up: the `Eq` derive on `KindPayload` is a cost (drop
    `Eq`, touching Grid); it carries only the *condition*, not a branch
    structure, so it is O3 with the condition relocated and shares O3's
    family-fit problem.

- **O5 — presence binding on the gated widget.** No new node; a
  reserved `IrBinding` ("present") on the single gated widget,
  approach-1-flavoured.
  - What you gain: no new node.
  - What you give up: cannot express a multi-widget conditional subtree
    without a wrapper, and reads as a property (against FD-CR).

### Comparison

| Axis | O1 member-level | O2 branch-node in children | O3 widget-kind `If` | O4 KindPayload | O5 presence binding |
|---|---|---|---|---|---|
| Control flow is… | a first-class member-level operator | a distinguished node (not a widget) with branches | a widget-like node kind | a generic node + payload | a property on a widget |
| `else` / `switch` fit | native (add a `Branch` / variant) | native (branch list) | poor (single `children`, needs a hack) | poor (no branch structure) | n/a |
| `for` (next phase) fit | native (`ControlFlowNode` variant) | native (variant) | sibling node kind, but same single-`children` limit | poor | n/a |
| approach-3 reachability | high (surface-agnostic member IR) | high | medium | low | low |
| declared/entity clarity | explicit in the IR shape | explicit | implicit (control flow looks like a widget) | implicit | absent |
| `IrProp.value=IrLiteral` invariant | preserved | preserved | preserved | preserved | preserved |
| `Eq` impact | none (sum type, derive as needed) | drop `Eq` on the carrying type (bounded) | none | drop `Eq` on `KindPayload` (touches Grid) | none |
| Change size | **large** (children → `Vec<IrMember>`, all sites) | medium | small | small | small |
| Migration risk when family grows | none | low | **high** (re-shape to a branch model) | high | high |

The honest trade-off is **change size now vs migration risk later**.
O3 is the smallest change and the one the existing architecture invites
(the `Cell` IR-only-node pattern, the reserved `BindingTarget` slot) —
but it models control flow *as a widget kind* and gives a single
`children` vector no room for a second branch, so `else` / `switch`
force either a per-convention hack or exactly the O1/O2 migration we
would be deferring. O4/O5 are O3's problems with the condition
relocated or the construct demoted to a property (O5 also fails FD-CR's
"structural, not property" stance).

O1 and O2 are the two options that make the **control-flow family
first-class** (thesis axis i): both carry a real branch list, so
`else` is a branch and `switch`/`for` are variants, not bolt-ons. O1
goes further and removes the "control flow is a widget" category error
entirely — children are a sum of `Widget | ControlFlow`, which states
in the IR shape itself that control flow is a structural operator over
members, and makes the declared/entity separation (axis ii) explicit
at the schema level. O1's cost is real (it re-types `children` and
touches every construction/traversal site), but it is paid **once**,
**now**, while `if` is the only variant — exactly the
"widen design area, hold implementation to `if`" posture the framing
asks for — rather than as a migration when `for` (the very next phase)
and `else`/`switch` arrive. O2 is the lighter structural fallback: it
keeps `children: Vec<IrNode>` and accepts a bounded `Eq` drop, getting
branch-list family-fit without the full re-type, at the price of
leaving control flow shaped like a node in the children vector.

### Recommendation

**O1** (with **O2** as the explicit fallback). This is the
consequential fork in the Phase 6 ADR set: O1 is a genuine IR-schema
change, so it needs explicit owner blessing. If the owner prefers to
avoid re-typing `children` this phase, **O2** keeps the branch-list
family-fit (so it does not fall into O3's migration trap) at a smaller
change. O3/O4/O5 are recorded as rejected: they model control flow as a
widget or a property and force a later re-shape.

**What `Accepted` selects (so the acceptance is unambiguous to later
readers).** Flipping this DD to `Accepted` means the owner accepts
**O1 as the design baseline**, and the O1-specific vocabulary
elsewhere in the ADR — the preamble §Context member-level IR language,
verification-closure item 3's "member-level control-flow construct",
and the §Upstream `Vec<IrMember>` / `IrMember` schema wording — is read
in its O1 form. The O1/O2 fork is **not** left open into the
implementation plan by default. **If the owner instead selects O2**,
the acceptance note says so explicitly, and that same O1-specific
vocabulary is read in its O2 form: a distinguished branch-list
control-flow node carried in `children: Vec<IrNode>` (not a re-typed
`children`), accepting the bounded `Eq` drop on the carrying type.
**R-1 and ID-1 are unaffected by the O1/O2 choice** — both options
expose the same branch-list family-fit, the same
`BindingTarget::ConditionalSubtree` runtime seam, and the same
absent=destroy / opt-in-retention normative semantics.

IR encoding decisions (O1):

- `IrNode.children` becomes `Vec<IrMember>` with `IrMember = Widget(IrNode)
  | ControlFlow(ControlFlowNode)`; `ControlFlowNode::If { branches:
  Vec<Branch> }`; `Branch { condition: HandlerExpr, body: Vec<IrMember> }`.
- **Phase 6 ships exactly one `Branch` and no `else`** — `branches` has
  length 1, `condition` is the lowered `if` condition
  (`HandlerExpr::BoolLit` or `HandlerExpr::BoolPropRead`, per
  DD-M3-P6-003's E1 vocabulary), and `body` holds **exactly one
  `Widget(_)` `IrMember`**, per DD-M3-P6-003's single-widget-child body
  restriction. A nested `ControlFlow(_)` directly in the body (a bare
  nested `if`) is **deferred** this phase along with the surface (it
  lands with the family extension `else` / `for`). The `Vec<IrMember>`
  `body` and the `Vec<Branch>` shapes both exist in the type for
  forward-compat but are constrained at lowering / loader to **length 1
  and `Widget(_)`-only** in Phase 6 (a multi-element body, a
  non-structural body member, or a `ControlFlow(_)` body member is
  `WASAMO_ERR_IR_MALFORMED`; a multi-`Branch` waits for `else`). The
  single-`Widget` body always materialises **exactly one `WidgetNode`**,
  which is what lets the runtime present/absent be **one**
  `insert_child` / `remove_child` rather than a child-range API — and
  removes the 0/1-materialised-child case a nested `if` body would
  introduce (§Conditional insertion granularity, IG-1; the range form is
  deferred to the Phase 7 `for`).
- Control flow materialises **no `WidgetNode` and no `Visual`** (like
  `Cell`, the runtime *interprets* it). The loader walks `IrMember`s,
  emitting widgets for `Widget(_)` and building a conditional binding
  for `ControlFlow(_)` (R-1).
- `Cell` stays an `IrNode`-children construct (a Grid-specific layout
  *child wrapper*); control flow is a *structural operator over
  members* — the two are deliberately different categories, which the
  `IrMember` sum makes explicit.
- The textual IR grammar (`dsl_spec.md` §8.5) gains a control-flow
  member production carrying the branch condition and body; emit/load
  roundtrips preserve both (verification closure item 3).
- Loader defense-in-depth (dual-gate with `wasamoc check`,
  DD-M3-P6-003): a control-flow member whose condition is missing,
  non-bool, or unresolved, that carries more than one `Branch` (until
  `else`), **whose `body` is empty, holds more than one member, holds
  a non-structural member, or holds a `ControlFlow(_)` member** (a bare
  nested `if`; Phase 6 single-widget-child rule), or
  that appears where a member is not admitted, surfaces
  `WASAMO_ERR_IR_MALFORMED`. **Impl-readiness:** enforcing the
  **non-bool** rejection at the loader needs type information the current
  `validate` lacks — it resolves declared **names** through a
  `HashSet<&str>` ([ir_loader.rs:153](../../../../wasamo-runtime/src/ir_loader.rs)),
  with `validate_expr_references` checking name presence only, never the
  referent's `IrType`. Phase 6 extends the resolver to a
  `declared: HashMap<&str, IrType>` so a control-flow condition is
  admitted only as `HandlerExpr::BoolLit` or a **bool**-typed
  `BoolPropRead`; a `PropRead` / `StrPropRead` / any non-bool target is a
  `validate` error. This is the loader-rejection evidence in
  [preamble §verification closure item 3](./preamble.md), making the
  loader a real second gate rather than a name-only check.

## Runtime present/absent mechanism

### Options

- **R-1 — `BindingTarget::ConditionalSubtree` + insert/remove**
  - Fill the reserved `BindingTarget` variant. The loader builds the
    conditional children **once into a detached template / builder**,
    registers a bool-typed Effect on the condition Signal whose body, on
    each evaluation, **inserts** the subtree into the parent at the
    recorded slot when true and **removes** it when false, using
    `insert_child` / `remove_child`.
  - What you gain: uses the **already-reserved** `BindingTarget` slot
    and the **already-existing** `insert_child` / `remove_child` +
    structural Effect teardown — the architecture was pre-shaped for
    exactly this (§6.8.7/§6.8.8); the minimal real structural mechanism.
  - What you give up: nothing material — it is the intended mechanism.

- **R-2 — always build, toggle Visual visibility**
  - Build the subtree always; on false, set the Visuals invisible /
    detached but keep the `WidgetNode`s.
  - What you gain: the simplest mechanism (no insert/remove).
  - What you give up: fakes structural absence with a visibility
    property and leaves inner Effects running — this is approach 1,
    **rejected by FD-CR** (and by DD-M3-P6-005's "absent = disposed").

### Comparison

R-2 is the approach-1 anti-pattern the thesis rejects: it fakes
structural absence with visibility and leaves inner Effects running
(violating FD-CR axis ii and DD-M3-P6-005's "absent = disposed"
policy). R-1 uses the **already-reserved** `BindingTarget` slot and the
**already-existing** `insert_child` / `remove_child` + structural
Effect teardown — i.e. the architecture was pre-shaped for exactly this
(§6.8.7/§6.8.8). It is the minimal real structural mechanism.

### Recommendation

**R-1.**

- Fill `BindingTarget::ConditionalSubtree { parent: WidgetId,
  declared_member_index: usize }` (exact field set finalised in
  implementation). **`declared_member_index` is the conditional block's
  stable position in the parent's *declared* member order — not a cached
  materialised child index.** The materialised insertion index is
  **recomputed at each mutation** (see Slot bookkeeping below), because a
  preceding conditional toggling present/absent shifts every following
  sibling's live index. The field name is chosen to make this the
  obvious reading: store the declared index, derive the live index.
- The loader, when it encounters a control-flow member, **captures the
  branch body as a builder** — the declared `IrMember` body plus a
  factory closure, **with no entity or Effect instantiated up front** (a
  builder is not a built subtree; the body's widgets are not constructed
  and its bindings register no Effects until the body is materialised on
  a present evaluation) — and registers a **bool Effect** on the branch
  condition. On each evaluation:
  - **false → true:** build a fresh entity subtree from the declared
    children and `insert_child` it at the recorded slot;
  - **true → false:** **detach and destroy** the subtree —
    `let removed = parent.remove_child(index)?; widget_destroy(removed);`
    (or a dedicated `detach_and_destroy_child(parent, index)` helper
    wrapping the pair), **not** bare `remove_child` + drop.
    `remove_child` ([widget.rs:1289](../../../../wasamo-runtime/src/widget.rs))
    only detaches the child Visual and **returns** the
    `Box<WidgetNode>`; dropping that box disposes the reactive Effects
    (via `EffectHandle::Drop`) but the **widget-pointer registry** sever
    is `widget_destroy`'s additional step
    ([widget.rs:1679](../../../../wasamo-runtime/src/widget.rs):
    `dispose_subtree_bindings` *then* `registry::remove_for_widget`). If
    the implementer discards the `remove_child` return value, every
    hit-test target inside the absent subtree (the lightbox `< > x`
    Buttons) retains a stale registry pointer, breaking DD-M3-P6-005's
    *absent subtree has no live effects / registry* and the toggle-then-
    observe teardown evidence. The teardown path is therefore
    `widget_destroy(removed)`;
  - the first evaluation establishes the initial presence (a `false`
    initial condition inserts nothing).
- **Validation is *not* deferred (load-time, recursive).** Deferring
  *materialisation* must not defer *validation*. The loader recurses
  into the declared branch body **at load time** and runs the full
  validate / name-resolution / type check on it — the same static
  checks a present subtree gets (DD-M3-P6-003 condition rules plus the
  loader defense-in-depth below) — **even when the initial condition is
  `false` and the body is never materialised this run**. So an invalid
  binding / prop / unknown-widget shape inside an absent-initial
  conditional is rejected at load with `WASAMO_ERR_IR_MALFORMED`, not
  surfaced as a deferred runtime error on the first toggle to present.
  Only entity construction and Effect registration are deferred to
  materialisation; static well-formedness of the whole declared tree
  (including every absent-initial branch body) is proven up front.
- **Builder vs build entry (impl-readiness).** Today's `build_node`
  constructs the widget, registers its bindings (with each Effect's
  initial run), and recurses its children in a single pass
  ([ir_loader.rs:1344](../../../../wasamo-runtime/src/ir_loader.rs)), so
  materialising a `false`-initial subtree through it would register
  **live Effects on an absent subtree** — violating DD-M3-P6-005's
  *absent subtree has no live effects*. The builder therefore holds the
  declared body only; a dedicated build entry point (split out of
  `build_node`, e.g. `build_members` / `build_widget_node`) is invoked
  from the condition Effect's true-branch, so entities and Effects come
  into existence **exactly when the subtree becomes present**. Splitting
  `build_node` is the first implementation task this DD implies.
- **Visual sibling order honours the slot (impl-readiness).**
  `WidgetNode::insert_child(index, …)` must keep the parent's
  `VisualCollection` sibling order consistent with `index`, not only the
  `children` Vec. The current primitive inserts the `children` Vec at
  `index` but always `InsertAtTop`s the Visual
  ([widget.rs:1280](../../../../wasamo-runtime/src/widget.rs)) — correct
  only for a top-slot insertion (the lightbox case), and **mis-ordering a
  conditional re-inserted between static siblings**, which would break
  both the quiescent child-order invariant below and, for a conditional
  child of a ZStack, the document-order z-order (DD-M3-P6-002). Phase 6
  therefore updates `insert_child` (and `replace_child`) so the child
  Visual lands at the position matching `index`: reference the adjacent
  already-attached sibling Visual and insert above/below it
  (`InsertAbove` / `InsertBelow`), falling back to top/bottom at the
  ends. The exact `VisualCollection` API selection is an
  implementation-task detail; the **contract** is that `children` Vec
  order and Visual sibling order agree after every structural mutation.
  This is a runtime primitive that `else` / `switch` / `for` and
  multiple sibling conditionals will all reuse, so it is paid in Phase 6.
- **Subtree teardown helper (impl-readiness).** Symmetrically, the
  true→false path's `widget_destroy(remove_child(index))` pairing (R-1
  above) is fixed as a runtime primitive — `detach_and_destroy_child`
  or the explicit pair — **before** the conditional builder/Effect is
  wired, alongside the `insert_child` Visual-order fix. Both are
  structural-mutation primitives the conditional Effect calls and that
  `else` / `switch` / `for` reuse, so a sequencing that touches the
  conditional runtime before these primitives are correct would leak
  registry state (teardown) or mis-order Visuals (insertion). Fixing
  them first keeps the workspace green at the point the conditional
  Effect lands.
- **Slot bookkeeping (recompute, do not cache) — both directions.** The
  conditional holds its stable `declared_member_index`; the
  **materialised index** is **recomputed at the moment of *each*
  mutation**, not cached at load, for **both** transitions:
  - **`false → true` (insertion index):** the count of currently-*live*
    preceding members — static widgets (always live) plus any preceding
    conditional blocks **that are present at that moment** (their
    live/absent state read from the presence view below).
  - **`true → false` (removal index):** resolved at the moment of removal
    the same way — either recomputed by the **same** preceding-live-member
    count, or located by finding the `live_child` handle's current
    position in the parent's `children` (both yield the same index since
    `live_child` *is* the materialised child). It is **not** the index
    captured at the prior insertion: a preceding conditional that went
    absent in between shifts this conditional's live position, so a cached
    index would `remove_child` the **wrong sibling**. Concretely, with two
    sibling conditionals A (earlier) and B both present, A→absent shifts
    B's live index down by one; a later B→absent must remove at B's
    *current* index, not its original one.

  A cached materialised index would mis-place the re-inserted subtree (on
  insertion) or remove the wrong child (on removal), breaking the
  quiescent child-order invariant below and (for a conditional child of a
  ZStack) document-order z-order (DD-M3-P6-002). For the lightbox the
  conditional is the last child of the root container — the simplest case
  — but the recompute rule is written to be correct for a conditional
  with static siblings on both sides and for two adjacent conditionals;
  the interaction is pinned by an integration test (verification closure
  item 4, cases (b) static siblings on both sides and (c) two sibling
  conditionals toggled independently, **including a preceding-conditional
  removal while both are present** so the removal-index shift is exercised).
- **Minimal runtime state + presence ownership (impl-readiness).** The
  per-conditional state the condition Effect's closure owns is minimal —
  `ConditionalRuntimeState { parent, declared_member_index, live_child:
  Option<live handle> }` — where `live_child` is `Some` exactly when the
  subtree is currently materialised (`None` when absent). The
  "presence map" the recompute reads is the set of preceding conditionals
  whose `live_child` is `Some`; whether that is a dedicated parent-owned
  table keyed by `declared_member_index` or derived by querying the
  preceding sibling conditionals' `live_child` is an implementation
  choice (the exact field set is finalised in implementation), but these
  invariants are **fixed** so sibling/re-evaluation cases are
  deterministic:
  - **single owner.** Each conditional's presence is owned by its own
    `ConditionalRuntimeState.live_child`; no second copy of the
    present/absent bit is kept. **If a parent-owned table is used, it
    stores/points to the `ConditionalRuntimeState` handles, not duplicate
    `present` bits** — presence is always *derived* from
    `state.live_child.is_some()`, so there is no second bit that can drift
    out of sync with the live tree.
  - **update after the mutation succeeds.** `live_child` is set to
    `Some(handle)` only after `insert_child` returns `Ok`, and to `None`
    only after `widget_destroy` of the removed subtree; a failed mutation
    leaves the prior state, so presence always reflects the actual tree.
  - **idempotent re-evaluation.** The condition Effect compares the
    newly-evaluated `bool` against current presence (`live_child.is_some()`)
    and mutates **only on a transition**: true→true and false→false are
    **no-ops** (no duplicate `insert_child`, no spurious `remove_child`).
    This is what makes a condition Effect that re-fires for an unrelated
    dependency change safe. The integration test (verification closure
    item 4) adds a re-evaluation-to-same-state case asserting the no-op.
- **Quiescent child-order invariant (normative; effect-/drain-order
  independent).** Because each conditional's slot is derived from its
  position in the **declared** member order, the parent's child order at
  quiescence — **both** the `children` Vec **and** (with the Visual
  sibling-order revision above) the parent's Visual sibling order — is a
  function of declared member order **alone**; it does **not** depend on
  the order in which the condition Effects fire or drain. Concretely:
  with multiple **sibling** conditionals (and **descendant** conditionals
  reached via a wrapper widget — both Phase-6 in scope; a bare nested
  `if` directly in a body is deferred, DD-M3-P6-003 B1) toggled
  by the same or different signals, whichever ones are present at
  quiescence appear among the static siblings in **declared document
  order**, regardless of effect-evaluation order. This is the guarantee
  that lets DD-M3-P6-005 adopt **SM-1** (no structural-ordering contract
  on the drain) without leaving final child order implementation-defined:
  drain *order* is unspecified, but quiescent *layout order* is fixed by
  the declared tree. The integration test (verification closure item 4,
  case (c) two sibling conditionals toggled independently) asserts this
  invariant directly.

## Conditional insertion granularity

R-1 fixes *that* the runtime inserts/removes via `BindingTarget::
ConditionalSubtree`; this sub-issue fixes the **granularity** of that
mutation, which is the runtime face of DD-M3-P6-003's conditional-body
shape (B1 single child / B2 multiple children). The two must agree: the
body's cardinality determines whether one child or a range is moved.

### Options

- **IG-1 — single-child insert/remove (pairs with DD-M3-P6-003 B1)**
  - The conditional body is one widget child; present/absent is a
    single `insert_child(index, child)` / `widget_destroy(remove_child(
    index))` at the recomputed materialised index (R-1). `ConditionalSubtree`
    carries `{ parent, declared_member_index }` and a single optional live
    child handle.
  - What you gain: reuses the **existing single-child primitives**
    (`insert_child` / `remove_child`) with only the Visual-sibling-order
    fix; slot bookkeeping is a single index; effect teardown is the
    existing single-subtree **detach + `widget_destroy`** path (R-1 /
    DD-M3-P6-005); no new range concepts. Smallest Phase-6 runtime
    surface, sufficient for the lightbox.
  - What you give up: a multi-widget conditional body is not directly
    representable — it requires an author-side wrapper (DD-M3-P6-003 B1).

- **IG-2 — child-range insert/remove (pairs with DD-M3-P6-003 B2)**
  - The conditional body is `structural_member*`; present/absent moves a
    **contiguous range** of children. `ConditionalSubtree` carries
    `{ parent, declared_member_index, live_len }` (or a live-id range);
    present builds N children and inserts them as a range, absent removes
    the range.
  - What you gain: a multi-widget body needs no wrapper.
  - What you give up: a new **range** machinery — range insertion
    (N children at consecutive indices), range slot bookkeeping (the
    base index plus length, recomputed against preceding live ranges),
    range Visual sibling-order, and range effect teardown (dispose N
    subtrees) — all with **no Phase-6 driver**, and overlapping the
    range work the Phase 7 `for` will build and generalise.

### Comparison

The granularity is **determined by the body-shape choice**, so this is
not an independent decision but the runtime half of it: B1 ⇒ IG-1, B2 ⇒
IG-2. The case for IG-1 is the case for B1 (DD-M3-P6-003 Comparison) —
the lightbox needs a single child, IG-1 reuses the existing single-child
primitives with only the Visual-order fix, and the range form is the
Phase 7 `for` driver, so IG-2 would pay for that surface a phase early
with no Phase-6 use. IG-2 is correct **only if** B2 is selected (the
owner wants wrapper-free multi-widget bodies now).

### Recommendation

**IG-1 — single-child insert/remove** (pairing with DD-M3-P6-003's
recommended B1). The Runtime present/absent mechanism (R-1) above is
written for IG-1: single `insert_child` / `widget_destroy(remove_child)`,
a single `declared_member_index` recomputed to a materialised index on
each mutation (both directions), single-subtree detach + `widget_destroy`
teardown. **If the owner selects DD-M3-P6-003 B2**, this
sub-issue reads as **IG-2** and R-1's single-child operations,
`ConditionalSubtree` field set, slot bookkeeping, and the verification
closure (item 4) generalise to ranges (range insert/remove, range
Visual-order, range teardown). The IR `body: Vec<IrMember>` already
holds the multi-element shape, so IG-2 is an additive runtime change,
not an IR re-type.

## Identity model on absent→present

This axis decides **author-visible semantics that go into the spec**:
whether re-appearing a subtree restores prior state. It is not a pure
implementation detail — once "absent = dispose" is normative,
introducing retention later changes observable behaviour unless it is
shaped as an opt-in (see Recommendation / forward-compat).

### Options

- **ID-1 — full rebuild, no identity preservation**
  - Absent = the entity subtree is destroyed (Effects disposed,
    `WidgetNode`s + `Visual`s dropped). Present-again = rebuilt fresh
    from the **declared tree** (the control-flow member's body, stable
    across the toggle). No state retention, no keys.
  - What you gain: the minimal mechanism; matches architecture.md
    §6.8.6's documented re-attach behaviour; the **declared tree is the
    stable anchor**, so ID-2 stays reachable later **without an IR
    change**; the deferral is made safe by shaping retention as opt-in.
  - What you give up: no state retention across absent→present —
    re-appearing is a fresh subtree (correct for the stateless lightbox
    photo); this is stated as **normative author-visible semantics**.

- **ID-1.5 — runtime identity anchor now, no state retention yet**
  - Keep a persistent runtime handle for the conditional subtree across
    absent→present (an "anchor" the future reconciler attaches to), but
    still destroy/rebuild the entity subtree and retain no state in
    Phase 6.
  - What you gain: the intent is to pre-install the identity seam so
    Phase 7 `for` keys / state retention bolt on without touching the
    present/absent path.
  - What you give up: it earns nothing in Phase 6 — the stable identity
    anchor **already exists** (the declared tree); a future reconciler
    keys off the *declared* construct, not a separate runtime handle, so
    a Phase-6 anchor is dead weight with a real chance of guessing the
    seam wrong.

- **ID-2 — Element-level reconciliation now**
  - Keep a persistent per-subtree identity ("Element") across
    absent→present, preserving in-progress state / focus / effect
    identity. Full Flutter-style Widget/Element/RenderObject reconciler.
  - What you gain: full state/focus/effect retention across
    absent→present.
  - What you give up: out of proportion to Phase 6 — a large subsystem
    (keys, diffing, Element lifetimes) with **no M3 driver** (the
    lightbox needs no retention; reopening fresh is correct).

### Comparison

ID-2 (a full reconciler) is **out of proportion** to Phase 6: the
lightbox needs no state retention across close→open (the photo
placeholder is stateless; reopening fresh is correct), and a
reconciler is a large subsystem (keys, diffing, Element lifetimes) that
M3 has no driver for. But the thesis is explicit that the *design* must
not foreclose it, and finding-4 of the review is right that the choice
is **author-visible normative semantics**, not an internal detail.

ID-1.5 (install a runtime anchor now) is the tempting "pre-wire the
seam" middle. The reason it earns nothing in Phase 6: **the stable
identity anchor already exists — it is the declared tree.** Under O1/O2
the control-flow member and its body persist in the IR across every
toggle; only the entity subtree (WidgetNodes/Visuals/Effects) is
destroyed and rebuilt. A future reconciler keys off the *declared*
construct (and an author-supplied `key:` for `for` items), not off a
separately-maintained runtime handle, so a Phase-6 runtime anchor would
be dead weight that the eventual reconciler may not even use in the
shape we guessed. ID-1.5 adds lifetime bookkeeping with no Phase-6
observable benefit and a real chance of guessing the seam wrong.

ID-1 satisfies the no-foreclosure requirement precisely **because the
declared tree is the anchor**: declaration persists, entity is
recreated — the declared/entity separation in its base, un-keyed form
(the Widget persists, the Element/RenderObject is recreated). A future
reconciler adds the identity layer **between** the stable declared
construct and the entity subtree **without an IR change**. So ID-1 is
not a shortcut that blocks ID-2; it is ID-2's un-reconciled base case.
What makes the deferral safe is **shaping retention as opt-in** so the
ID-1 default never silently changes (see Recommendation): destroy/
recreate is the spec's baseline, and retention arrives as keyed /
explicit opt-in semantics, not as a behavioural change to existing
`if` blocks.

### Recommendation

**ID-1 (full rebuild)** — and its normative-semantics contract:

- **Absent = destroyed, present = rebuilt.** No state retention across
  absent→present; no `key:` attribute; no Element-level identity layer
  in Phase 6.
- This is **author-visible normative semantics**, stated as such in
  `dsl_spec.md` §4.14: *a conditional subtree that goes absent and
  returns is a **fresh** subtree; any state inside it resets.* Authors
  who need persistence across toggles keep that state in a
  component-level `state` (outside the conditional), which is the
  established Wasamo pattern.
- **Compatibility shape for future retention (so the deferral is
  safe).** Destroy/recreate is the **baseline** that does not change.
  Future state-retention arrives as **opt-in** semantics — a `key:` /
  retention marker on the construct — so existing `if` blocks keep
  destroy/recreate behaviour and retention never silently alters
  observable behaviour. This makes ID-1 a forward-compatible default,
  not a semantic we will have to break.
- The **declared tree is the identity anchor**: the control-flow member
  and its body persist in the IR across every toggle; only the entity
  subtree is recreated. Documented in `architecture.md` §9 as the
  declared/entity separation in its base un-keyed form, so the future
  reconciler (keys, state carry-over, Phase 7 `for` item identity) is
  an additive layer with **no IR change** (forward-compat below).

## Spec content seed

Textual IR form for the control-flow member (`dsl_spec.md` §8.5
`node_body`), pinned here so `wasamoc` emit, the runtime loader,
roundtrip tests, and the spec share one shape rather than inventing it
per crate. The control-flow member is a new `node_body` alternative
**alongside** `widget_node` (it is not a `node`):

```
node_body           ::= ( … | widget_node | control_flow_member )*

control_flow_member ::= "if" cond "{" widget_node "}"   ; Phase 6: exactly one
                                                        ; widget node — no else,
                                                        ; no nested control flow
cond                ::= BOOL | IDENT   ; BOOL → HandlerExpr::BoolLit
                                       ; IDENT → bool-typed HandlerExpr::BoolPropRead
```

Worked example — `.ui` → textual IR → loaded IR for the lightbox slice
(`if is_lightbox_open { ZStack { … } }`):

`.ui`:

```
component Gallery inherits Window {
    state is_lightbox_open: bool = false
    title: "Gallery"
    WrapPanel { /* thumbnails */ }
    if is_lightbox_open {
        ZStack {
            Box { fill: #00000080 }
            Box { aspect: 4:3  Text { text: "photo" } }
        }
    }
}
```

textual IR (`wasamoc` emit):

```
node Window {
    prop title = "Gallery"
    node WrapPanel { /* … */ }
    if is_lightbox_open {
        node ZStack {
            node Box { prop fill = #00000080 }
            node Box { prop aspect = 4:3  node Text { prop text = "photo" } }
        }
    }
}
```

loaded IR (O1):

```
IrNode { widget_type: "Window", children: [
    Widget(IrNode { widget_type: "WrapPanel", … }),
    ControlFlow(ControlFlowNode::If { branches: [
        Branch {
            condition: HandlerExpr::BoolPropRead("is_lightbox_open"),
            body: [ Widget(IrNode { widget_type: "ZStack", … }) ],   // exactly one
        },
    ] }),
] }
```

**Rejection shapes (loader `WASAMO_ERR_IR_MALFORMED`, mirroring the
`wasamoc check` diagnostics of DD-M3-P6-003):**

- **multi-branch** — a `control_flow_member` parsing to `branches.len()
  > 1` (an `else` arm) until `else` is specified;
- **multi-child / non-structural / nested-control-flow body** — more
  than one `node` inside the `if { … }`, a `prop` / `binding` /
  `handler` / `tracks` line directly in the body, or a nested `if`
  directly in the body (Phase 6 single-widget-child rule);
- **non-bool / unresolved `cond`** — handled by the `declared:
  HashMap<&str, IrType>` resolver extension (IR encoding Recommendation).

The emit/load roundtrip preserves the branch condition and the
single-child body (verification closure item 3).

## Forward-compat exposure

- **Nested control flow directly in a branch `body`** (a `ControlFlow(_)`
  `IrMember` in `body`, not just a `Widget(_)`). Deferred this phase
  (Phase 6 constrains `body` to a single `Widget(_)`); the `body:
  Vec<IrMember>` type **already admits** `ControlFlow(_)`, so lifting the
  restriction is a loader/lowering relaxation plus the insertion-grain
  re-statement for the 0/1-materialised-child case — **no IR shape
  change** — landing with the family extension (`else` / `for`).
- **`else` / `switch`.** `else` is an additional `Branch` on the same
  `ControlFlowNode::If` (the length-1 `branches` restriction lifts);
  `switch` is a new `ControlFlowNode` variant (`Switch { subject, arms
  }`). Both reuse R-1's insert/remove machinery — the present/absent of
  a branch is the same operation as the present/absent of the whole
  construct. No `IrMember` shape change.
- **`for` (Phase 7 iteration).** A new `ControlFlowNode` variant
  (`For { binding, body }`) filling `BindingTarget::ForLoopSubtree`
  (the other reserved slot). It needs **keyed identity** (item
  reorder / state retention), which is the first real driver for the
  Element-level identity layer ID-2 defers. Because the declared tree
  (the `For` member + item template) is stable and ID-1 already
  separates declared from entity, the identity layer lands between
  them without an IR-shape change — only a new `ControlFlowNode`
  variant and the additive `key:` / retention opt-in.
- **Approach 3 (host-language constructs).** A future language-internal
  DSL lowers its own `if`/`switch`/loop into the **same**
  `ControlFlowNode` variants and the **same** `BindingTarget`
  structural seam — the thesis requirement that approach 3 stay
  reachable. The runtime mechanism is surface-agnostic by construction
  (it consumes `IrMember` control-flow members, not `.ui` syntax). This
  is the payoff of O1 over O3/O4: a member-level structural IR is the
  surface-neutral target an embedded DSL can also reach.
- **Subtree-grain layout dirty.** Phase 6 inserts/removes whole
  subtrees, which dirties layout; this rides the existing
  whole-window dirty path (DD-P8-002), with subtree-grain invalidation
  remaining the open question in
  [layout-engine note §3.4](../../../../docs/notes/layout-engine.md)
  — unchanged by Phase 6.

## Technical risk re-evaluation

- **O1 is the largest single change in the phase** — re-typing
  `IrNode.children` to `Vec<IrMember>` touches every construction and
  traversal site (`wasamoc` lowering, the IR loader's `build_widget_tree`
  / `validate`, textual IR emit/parse, and tests). The mitigation is
  the no-`Default` construction-site discipline: every site is surfaced
  at compile time, so the change is mechanical and exhaustive rather
  than a silent-omission hazard. The owner-impact of choosing O1 is
  this up-front cost; the owner-impact of *not* choosing it (O3/O4) is a
  re-shape migration when `for`/`else` arrive — and `for` is the very
  next phase. O2 is the de-risking fallback if the up-front cost is
  judged too high this phase. **No new IR scalar/literal type**; the
  condition rides existing `HandlerExpr`, so `IrProp.value = IrLiteral`
  is untouched and the `Eq` question only arises (and is a bounded,
  deliberate cost) under O2.
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
  corner" is mitigated by the control-flow member being approach- and
  identity-neutral (it describes structure, not lifetime) and by the
  opt-in retention compatibility shape (the ID-1 default never has to
  break). The residual risk is mis-guessing the future `key:` opt-in
  surface — bounded, because retention attaches to the declared
  construct (which is stable) rather than to a Phase-6-frozen runtime
  handle.
