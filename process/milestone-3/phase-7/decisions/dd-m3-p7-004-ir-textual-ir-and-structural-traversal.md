# DD-M3-P7-004 — IR / textual IR representation and structural traversal model

**Status:** Proposed
**Phase:** M3-Phase 7
**AC:** A8; A11 (IR / loader / roundtrip advance in lockstep)

## Context

Phase 6 made control flow a first-class member-level IR construct:
`IrMember = Widget(IrNode) | ControlFlow(ControlFlowNode)`, with
`ControlFlowNode::If { branches }` designed explicitly so `else` /
`switch` / `for` arrive as **same-family variants**. Phase 7 is the
first test of that family extension point.

Two Phase 6 learnings are load-bearing here (constraints §1 / §8):

- **Traversal dispatch is where this family breaks.** The
  widget-only-filter failure mode (`widget_children()` silently
  dropping `ControlFlow` members) actually occurred in Phase 6; adding
  a second control-flow variant multiplies the sites where a `match`
  written for `If` quietly mishandles `For`. The semantic-migration
  audit gate (DD-V-025, implementation-gates trap #1) applies in full.
- **The Phase 6 DD-007 reservation** — whether to canonize "member
  emission" as a content model — was deliberately left open. With one
  control-flow form, presence math could live inline; this DD must
  decide whether a second form tips it into a shared model.

Runtime seam (constraints §2): `BindingTarget::ConditionalSubtree {
parent, declared_member_index }` recomputes the materialised insertion
index from declared order + live presence at each mutation. Iteration
generalises "live presence ∈ {0,1}" to "live cardinality ∈ 0..N".

## Decision dependency summary

Primary for the **member-expansion canonization** bundle: the shared
seam chosen here is consumed by DD-M3-P7-005 (insertion index math)
and DD-M3-P7-006 (splice-primitive call shape). Consumes DD-M3-P7-001
(body shape), DD-M3-P7-002 (`HandlerExpr` collection forms),
DD-M3-P7-003 (binder reads).

## Sub-issues

- **IR node shape** — the `For` variant's form.
- **Textual IR syntax** — emission and loader validation.
- **Member-expansion model** — canonize or keep per-site (the Phase 6
  DD-007 reservation).
- **Runtime binding target** — `ForLoopSubtree` shape.
- **Call-site audit** — the trap-#1 obligation.

## IR node shape

### Options

- **S1 — sibling variant, parallel structure to `If`:**
  ```rust
  pub enum ControlFlowNode {
      If { branches: Vec<ControlFlowBranch> },
      For {
          binder: String,
          index_binder: Option<String>,
          collection: HandlerExpr,   // ListPropRead{path, elem} (DD-002)
          body: Vec<IrMember>,       // length-1, Widget-only (enforced)
      },
  }
  ```
  - `body` keeps the `Vec<IrMember>` type of `ControlFlowBranch.body`
    (length-1 / `Widget`-only enforced at lowering + loader, exactly
    the Phase 6 mechanism) so the future member-range generalisation
    widens an admission rule, not a type.
  - What you gain: the family stays one enum with one dispatch point;
    every `match ControlFlowNode` is forced by the compiler to confront
    `For` (the compile-error-forcing property the gates prefer);
    textual IR and traversal extend by one arm each.
  - What you give up: nothing structural; `For` carries loop-specific
    fields `If` doesn't, which is what enum variants are for.

- **S2 — a separate `IrMember::Iteration(IterationNode)`**
  - What you give up: a third member kind splits the control-flow
    family the spec sells as one family; every member dispatch gains an
    arm that is semantically a control-flow arm but syntactically
    distinct — pure surface area, no compensating merit. Rejected.

- **S3 — desugar `For` into repeated `If`-like branches at lowering**
  - What you give up: cardinality is runtime data — there is nothing to
    desugar to statically; any encoding that pretends otherwise
    reintroduces static expansion, which FD-A rejects as the thesis
    failure. Rejected on thesis grounds.

### Recommendation

**S1.** The `collection` field is the typed collection read
(`HandlerExpr` unified — no side enum); binders are plain `String`s
(scope checking is `wasamoc`'s job, DD-003; the IR records names, not
scopes).

## Textual IR syntax

Member production gains the `for` form beside the Phase 6 control-flow
member, mirroring the in-memory shape one-to-one:

```
member ::= widget | control_flow_member
control_flow_member ::= "(" "if" branch+ ")"
                     |  "(" "for" IDENT index_binder?
                            handler_expr   ; collection read
                            member ")"     ; body: one widget
index_binder ::= "(" "index" IDENT ")"
```

(Exact token spelling finalised with the spec draft; the normative
properties are:) binders, collection read, and body **roundtrip
losslessly**; the loader enforces — collection expr is a collection
read of a declared collection state; body is exactly one `Widget`
member; binder well-formedness (non-empty, distinct) — each violation
`WASAMO_ERR_IR_MALFORMED` (the dual gate: `wasamoc check` rejects
authored sources, the loader re-rejects hand-written IR text).
Loader-side static structure: a `for` member's **declared slot** is
present at load time with its initial cardinality materialised from the
collection's initial value (0..N children at load — the empty-initial
case materialises zero and must not be conflated with "member absent").

## Member-expansion model (the canonization judgment)

The question Phase 6 reserved: is "a declared member list expands to a
materialised child list" a **canonized content model** with one
implementation, or per-site logic?

### Options

- **C1 — canonize: one shared expansion seam.**
  - One function family owns the math: per declared member a **live
    cardinality** (Widget = 1; `If` = 0/1; `For` = current collection
    length) and the **materialised insertion offset** of declared slot
    `k` = Σ cardinality(declared members < k) — a prefix sum. Static
    load materialises by walking it; every reactive structural mutation
    (conditional toggle, range insert / remove) computes its
    insertion / removal indices through the *same* functions.
  - What you gain: the Phase 6 recompute-from-declared-order rule and
    the Phase 7 range math cannot drift apart — interleaved `if` /
    `for` / static siblings are correct by shared construction (the
    declared-order invariant tests of Phase 6 generalise rather than
    duplicate); the seam is pure logic (unit-testable without WinRT);
    the future member-range body widens cardinality from "collection
    length" to "length × per-item arity" in one place.
  - What you give up: a refactor of the Phase 6 conditional path onto
    the seam (it becomes the 0/1 special case) — touching shipped
    working code is the cost; the audit covers it.

- **C2 — keep per-site: `For` gets its own index logic beside `If`'s.**
  - What you gain: no touch of the shipped conditional path.
  - What you give up: two implementations of one invariant (quiescent
    children = declared order with live cardinalities). Phase 6's
    drift evidence was *within one phase, one form*; two forms × two
    sites is the parallel-data-drift shape (trap #3) at the logic
    level. Every future family member (`else` arms changing branch
    arity, `switch`, ranges) re-pays the duplication.
  - Rejected on merit: it preserves code at the price of the invariant.

### Recommendation

**C1 — canonize.** This resolves the Phase 6 DD-007 reservation in the
canonize direction, scoped to **structural member expansion** (not a
general content-model framework — no admission rules, no widget
semantics move into the seam; containers keep their own child-shape
contracts, DD-001 sweep). The conditional path's migration onto the
seam is an explicit implementation task with its own regression run of
the Phase 6 declared-order fixtures.

## Runtime binding target

```rust
BindingTarget::ForLoopSubtree {
    parent: WidgetId,
    declared_member_index: usize,
}
```

— the shape `ConditionalSubtree` reserved for it (constraints §2): the
declared slot is the stable identity; the materialised range
`[offset, offset + cardinality)` is **recomputed via the C1 seam at
each mutation, never cached** (the Phase 6 rule, generalised). The
`for` effect reads the collection signal, obtains old/new cardinality,
and hands DD-M3-P7-005 a tail insert / remove plan addressed through
the seam. Per-item binding effects created during materialisation
belong to their generated subtree (ownership / disposal: DD-005).

## Call-site audit (trap #1 obligation)

Adding `ControlFlowNode::For` and `BindingTarget::ForLoopSubtree` is a
semantic migration. The close-gate artifact is the standard table —
`rg`-enumerated `match` / consumer sites over `IrMember`,
`ControlFlowNode`, `BindingTarget`, `HandlerExpr` (DD-002/003 variants)
across `wasamoc`, textual-IR emit/load, validator, runtime loader,
reactive engine — each classified (extended / correctly unaffected /
deliberately rejects) with per-class reasons and tests-added-or-not.
Known hotspot pinned in advance: `IrNode::widget_children()` and every
widget-only filter — each use is classified as *correct filtering*
(layout-time over materialised children) or *a bug under `For`*
(traversal over declared members), the exact Phase 6 failure mode.
Mechanism preference (constraints §8): compile-error-forcing
constructions over silent-absorb helpers; S1 + I2 (DD-002) provide it.

## Spec content seed

dsl_spec textual-IR chapter: the `for` member production beside the
`if` member (the control-flow members subsection Phase 6 established),
a loaded-IR example mirroring the gallery shape (`.ui → textual IR →
loaded IR`, one `for` with an `if` sibling so declared-slot offsets are
exemplified), loader validation policy rows. architecture.md: the
member-expansion seam description (declared cardinality / prefix-sum
offsets) and the `ForLoopSubtree` entry in §6.7.7/§6.7.8 — stated as
the accepted contract, no option labels.

## Forward-compat exposure

- **`else` / `switch`** — additional branches / a new variant on the
  same enum; their cardinality plugs into the C1 seam unchanged (still
  0/1 per member).
- **Member-range bodies** — body admission widens; the seam's
  per-member cardinality becomes `length × arity`; `ForLoopSubtree`
  shape unchanged.
- **Keyed identity** — the declared slot stays the anchor; a keyed
  reconciler changes *which* materialised child maps to *which*
  element, not the slot/offset model — no IR **reshaping** of the
  expansion model. A future opt-in `key:`-like surface may add a field
  to the `For` variant to carry the key selector; that additive field
  is the expected keyed-retention cost and does not disturb the
  declared-slot / prefix-sum contract.
- **Approach 3** — a host-language loop lowers to the same `For`
  member; the IR stays approach-neutral.

## Strategic review disposition

- **Review F7 folded.** The keyed forward-compat note now distinguishes
  no expansion-model reshaping from a possible additive key-selector IR
  field; no recommendation change.

## Revision history

- Strategic owner-alignment review fold: narrowed keyed
  forward-compat from "no IR change" to "no IR reshaping, additive key
  field possible"; status remains Proposed.

## Technical risk re-evaluation

- **The C1 refactor is the riskiest piece** — it touches the shipped
  conditional mutation path. Mitigations: the seam is pure logic with
  its own unit suite (interleavings, zero-cardinality, boundary
  slots); the Phase 6 Windows-runtime declared-order fixtures run
  unchanged as regressions; the refactor is its own task/commit,
  separate from `For` runtime work.
- **`Vec<IrMember>` body with length-1 enforcement** repeats a Phase 6
  pattern that worked, but now two variants enforce it — the
  enforcement helper is shared, not duplicated (small trap-#3
  instance).
- **Textual-IR additions** are mechanical; the loader's
  reject-malformed branches each get direct tests (trap #4).
- **Static load with non-empty initial collection** is a new load-time
  path (Phase 6 conditionals evaluated to at most one child) — the
  load-time materialisation count gets an explicit roundtrip + load
  test including the empty-initial (zero children, member present)
  case.
