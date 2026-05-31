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

The axis here is **not** "where do we stash the condition" but **what
is the structural shape of control flow in the IR** — because the IR
schema is what `else` / `switch` / `for` / a future host-language DSL
will all have to fit. The options span from "change the IR schema so
control flow is first-class" (O1/O2) to "reuse the existing widget-node
slot" (O3) to "stash it on a generic node" (O4/O5). The `Eq` derive on
`KindPayload` is treated as a **real cost to weigh**, not a blocker
that auto-disqualifies a schema change.

- **O1 — member-level structural IR (recommended).** Change `IrNode`'s
  `children` from `Vec<IrNode>` to `Vec<IrMember>`, where

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

  Control flow is a **first-class member-level construct**, not a
  widget. `else` is an additional `Branch` (an `else` branch is a
  `Branch` with a trivially-true / sentinel condition); `switch` is a
  `ControlFlowNode` variant with arms; `for` is a variant with a body.
  The condition rides `HandlerExpr` (no `IrProp`/`IrLiteral` change).
  This is a **genuine IR-schema change** — every `children` construction
  / traversal site moves to `IrMember` (surfaced at compile time by the
  no-`Default` discipline) — and the textual IR grammar (§8.5) gains a
  control-flow member production. Phase 6 ships only the single-`Branch`
  `If` variant.

- **O2 — distinct control-flow node carried in `children`, with a
  branch-list payload.** Keep `children: Vec<IrNode>`, but represent
  control flow as a distinguished IR node carrying a real `branches:
  Vec<Branch>` structure (in a dedicated field or payload). Control
  flow is still *shaped like* a node in the children vector, but it is
  not a widget kind and it carries first-class branches. Cost: the
  branch list holds `HandlerExpr`, so the carrying type cannot derive
  `Eq` — dropping `Eq` from that type (or from `KindPayload` if reused)
  is the real, bounded cost.

- **O3 — `widget_type: "If"` reusing the generic `IrNode` (the easy
  seam).** An IR-only node kind (like `Cell`): condition on a reserved
  `IrBinding`, the one branch in `children`, `kind_payload: None`.
  Minimal change, no schema edit. But a single `children: Vec<IrNode>`
  has **no place for a second (`else`) branch or `switch` arms** — they
  would need a hack (two children groups distinguished by convention),
  so the family does not fit cleanly and a migration to O1/O2 is likely
  when the family grows.

- **O4 — `KindPayload::Conditional { condition }`.** Stash the
  condition in a new `KindPayload` variant on a generic node. The `Eq`
  derive on `KindPayload` is a cost (drop `Eq`, touching Grid), not a
  hard blocker — but this option carries only the *condition*, not a
  branch structure, so it is O3 with the condition relocated and shares
  O3's family-fit problem.

- **O5 — presence binding on the gated widget.** No new node; a
  reserved `IrBinding` ("present") on the single gated widget,
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

This axis decides **author-visible semantics that go into the spec**:
whether re-appearing a subtree restores prior state. It is not a pure
implementation detail — once "absent = dispose" is normative,
introducing retention later changes observable behaviour unless it is
shaped as an opt-in (see Recommendation / forward-compat).

- **ID-1 — full rebuild, no identity preservation (recommended for
  Phase 6).** Absent = the entity subtree is destroyed (Effects
  disposed, `WidgetNode`s + `Visual`s dropped). Present-again =
  rebuilt fresh from the **declared tree** (the control-flow member's
  body, stable across the toggle). No state retention, no keys.
  Author-visible semantics: re-appearing is a fresh subtree.
- **ID-1.5 — runtime identity anchor now, no state retention yet.**
  Keep a persistent runtime handle for the conditional subtree across
  absent→present (an "anchor" the future reconciler attaches to), but
  still destroy/rebuild the entity subtree and retain no state in Phase
  6. The intent is to pre-install the identity seam so Phase 7 `for`
  keys / state retention bolt on without touching the present/absent
  path.
- **ID-2 — Element-level reconciliation now.** Keep a persistent
  per-subtree identity ("Element") across absent→present, preserving
  in-progress state / focus / effect identity. Full Flutter-style
  Widget/Element/RenderObject reconciler.

## Comparison

### IR encoding

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

## Recommendation

**O1 + R-1 + ID-1.** This is the consequential fork in the Phase 6
ADR set: O1 is a genuine IR-schema change, so it needs explicit owner
blessing. If the owner prefers to avoid re-typing `children` this
phase, **O2 is the fallback** — it keeps the branch-list family-fit
(so it does not fall into O3's migration trap) at a smaller change.
O3/O4/O5 are recorded as rejected: they model control flow as a widget
or a property and force a later re-shape.

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

### IR encoding (O1)

- `IrNode.children` becomes `Vec<IrMember>` with `IrMember = Widget(IrNode)
  | ControlFlow(ControlFlowNode)`; `ControlFlowNode::If { branches:
  Vec<Branch> }`; `Branch { condition: HandlerExpr, body: Vec<IrMember> }`.
- **Phase 6 ships exactly one `Branch` and no `else`** — `branches` has
  length 1, `condition` is the lowered `if` condition
  (`HandlerExpr::BoolLit` or `HandlerExpr::BoolPropRead`, per
  DD-M3-P6-003's E1 vocabulary), `body` is the block's members in
  document order. The multi-`Branch` shape exists in the type but is
  rejected at lowering / loader until `else` lands (forward-compat).
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
  `else`), or that appears where a member is not admitted, surfaces
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

### Runtime mechanism (R-1)

- Fill `BindingTarget::ConditionalSubtree { parent: WidgetId, slot:
  ChildSlot }` (exact field set finalised in implementation; `slot`
  records the conditional block's stable position among the parent's
  members).
- The loader, when it encounters a control-flow member, **captures the
  branch body as a builder** — the declared `IrMember` body plus a
  factory closure, **with no entity or Effect instantiated up front** (a
  builder is not a built subtree; the body's props/bindings are *not*
  resolved until the body is materialised on a present evaluation) — and
  registers a **bool Effect** on the branch condition. On each
  evaluation:
  - **false → true:** build a fresh entity subtree from the declared
    children and `insert_child` it at the recorded slot;
  - **true → false:** `remove_child` the subtree (dropping it, which
    disposes its Effects via the structural teardown — DD-M3-P6-005);
  - the first evaluation establishes the initial presence (a `false`
    initial condition inserts nothing).
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
- **Quiescent child-order invariant (normative; effect-/drain-order
  independent).** Because each conditional's slot is derived from its
  position in the **declared** member order, the parent's child order at
  quiescence — **both** the `children` Vec **and** (with the Visual
  sibling-order revision above) the parent's Visual sibling order — is a
  function of declared member order **alone**; it does **not** depend on
  the order in which the condition Effects fire or drain. Concretely: with multiple sibling / nested conditionals toggled
  by the same or different signals, whichever ones are present at
  quiescence appear among the static siblings in **declared document
  order**, regardless of effect-evaluation order. This is the guarantee
  that lets DD-M3-P6-005 adopt **SM-1** (no structural-ordering contract
  on the drain) without leaving final child order implementation-defined:
  drain *order* is unspecified, but quiescent *layout order* is fixed by
  the declared tree. The integration test (verification closure item 4,
  case (c) two sibling conditionals toggled independently) asserts this
  invariant directly.

### Identity model (ID-1) — and its normative-semantics contract

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

## Forward-compat exposure

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
