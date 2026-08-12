# DD-M4-P3-003 — Per-item conditional rendering: loop context ownership and structural lifecycle

**Status:** Proposed
**Phase:** M4-Phase 3
**AC:** AC9 (per-item conditional rendering); AC1 as a regression
boundary only; phase-end criterion 4 (spec synchronization)

## Context

`if` and `for` both exist. Composing them — an `if` inside a `for` body
whose condition reads the loop's own binders — is the surface AC9 names,
and the plan's Revision 3 already corrected the phase's responsibility
for it from compiler-side to cross-layer.

The source audit that motivated Revision 3 found two seams where loop
context is dropped. **The measurement is stronger than that.**

`if <bool state> { Text { text: "\{label}" } }` inside a `for` body
**compiles and loads today**, producing

```
if (bool-prop-read flag) {
    child { node Text { bind text = (interp ((item-read label))) } }
}
```

and on the runtime side:

- `ir_loader::append_static_member` takes a `loop_context` parameter and
  forwards it to the `Widget` and `For` arms — but the `If` arm never
  mentions it;
- `reactive::register_conditional_binding` builds a
  `BindingEvalContext`, which has no item-aware methods;
- `ir_loader::mutate_conditional_subtree` calls `build_node`, the
  no-loop-context wrapper, so the child is also written
  `set_loop_scope(None)`;
- `EffectHandle::new` runs its closure immediately, so the mutation path
  is how the subtree is **first** materialised, not only how it is
  re-materialised.

So loop context is absent at every materialisation, in both bindings and
handlers, and the composition is an authoring form that passes both
gates and then does not work. This record therefore has a repair
obligation, not only a design one.

The subtree involved is not inert. It owns bindings and, since
M4-Phase 2, handlers; inserting and removing it inside a repeated region
touches the recurrence conditions the Phase 2 handoff carried forward.

### What exists to build on (measured)

- **`ForItemContext` and `ForItemEvalContext` already exist** and are
  what the `for` body's own bindings use. `ForItemEvalContext`'s item
  reads go through `Signal::get()` — **tracked** — so a condition
  evaluated in that context re-runs when the collection changes.
- **`build_node_with_loop_context` already exists** and is what the
  `for` insert path calls; `build_node` is the `None` wrapper.
- **`WidgetNode::set_loop_scope` already exists** and is written from
  the same `loop_context` parameter, one kind-independent site, which is
  what per-item handlers read at invocation.
- **The structural seam is single and already reached.**
  `mutate_conditional_subtree` calls `mark_layout_dirty_for` on both the
  insert and the remove path; `emit::flush_layout` is the one production
  site where focus is rebased and modal scopes reconciled
  (`focus::sync_scopes_to_tree`), and the four `ir_loader` mutation call
  sites named in that function's doc comment are conditional insert /
  remove and `for`-range insert / remove.
- **Materialised-child accounting already composes.**
  `DeclaredMemberSlot` / `materialized_offset_for_declared_slot` compute
  a live index from the declared members of **one** parent, and a
  conditional inside a `for` body lives in the body child's own slot
  vector, not the repeating parent's.
- **Removal disposal already exists.** `widget_destroy` on the removed
  child drops its `EffectHandle`s (which unregister from the graph on
  `Drop`) and removes registry entries; the `for` insert rollback path
  documents this explicitly.
- **Nested `for` is rejected at any depth**, in both `wasamoc check` and
  the loader; `if` inside a `for` body widget is not.

## Sub-issues

- **Which binders a condition may read**, and over which element types.
- **Who owns the loop context used to evaluate the condition.**
- **Who owns the loop context used to re-materialise the subtree**, and
  whether it is the same owner.
- **Positional semantics** under collection replacement.
- **Structural lifecycle**: layout, focus, hover, handler registry,
  effect disposal.
- **The already-admitted broken composition** — repair or reject.
- **What stays closed**: nested `for`, bare nested control flow,
  multi-widget bodies, shadowing, keyed identity.
- **Whether the chosen ownership forecloses multi-level `for`**
  (framing agreement ⑬).

## Options

### Loop-context ownership

- **L1 — thread the context through the existing seams.**
  `append_static_member`'s `If` arm captures its `loop_context`;
  `register_conditional_binding` takes it and evaluates the condition in
  a `ForItemEvalContext`; `mutate_conditional_subtree` takes it and
  calls `build_node_with_loop_context`. One owner, captured where the
  enclosing subtree was built.
- **L2 — read it back from the tree.** The conditional effect looks up
  its parent node's `loop_scope` at evaluation time, so the node is the
  owner and nothing is captured in the closure.
- **L3 — an ambient context stack** in the loader, pushed and popped
  around build and evaluation.
- **L4 — a distinct IR form.** Lower a conditional inside a `for` to a
  variant that names its binders, and have the runtime resolve them
  against the enclosing repetition's live state.

### The already-admitted composition

- **H-a — repair.** Thread the context so the existing form starts
  behaving as written.
- **H-b — reject.** Add a diagnostic refusing a binder read inside a
  conditional inside a `for` until the capability lands.
- **H-c — leave.** Ship the new capability and let the old composition
  keep failing where it already fails.

### Structural lifecycle integration

- **W-1 — reuse the existing seam unchanged.** Per-item conditional
  mutation goes through `mark_layout_dirty_for` → `flush_layout`, which
  is where focus rebasing and scope reconciliation already happen.
- **W-2 — a dedicated per-item mutation path** with its own layout and
  focus handling.

## Comparison

### Ownership: L1 is the only option whose cost is a parameter

L4 is the largest and buys the least. A distinct IR form means a second
lowering, a second loader validation branch and a second evaluator path
for a construct whose author-visible meaning is "the `if` you already
know, inside the `for` you already know". It also makes the IR carry the
nesting relationship explicitly, which is the one thing a positional,
un-keyed model does not otherwise need.

L3 is genuinely the most natural fit for **later** nesting — a stack is
what multi-level `for` wants — but an ambient stack in the loader is a
hidden dependency: any code path that builds a node has to be inside the
right push/pop pair, and nothing in the type system says so. The current
code has the opposite property, which is worth preserving: the context
is an explicit parameter, so a call site that fails to pass it is
visible in the source, which is exactly how the present defect was
found. Adopting L3 now would remove that visibility in exchange for a
generality no `.ui` can express.

L2 is attractive because it removes the capture-versus-live question
entirely: if the node owns its scope, the effect cannot hold a stale
one. Two things spoil it. The conditional's effect runs to *create* the
child, so at first materialisation the node whose `loop_scope` it would
read is the parent — which is the enclosing `for` body child, whose
scope is the right one, so far so good — but `WidgetId` is a raw pointer
and the effect would be following it on every evaluation, widening
exposure to the pointer-identity residual the Phase 2 handoff carries
(CF-T7-1) into a path that currently does not have it. And it makes the
condition's meaning depend on tree state at evaluation time, which is a
strictly larger surface than the parameter it replaces.

L1's cost is that it is one-level by construction: a single
`Option<ForItemContext>` cannot represent two enclosing repetitions.
That is the framing ⑬ question and it is answered below rather than
waved at.

### L1 and multi-level `for` (framing agreement ⑬)

The obligation is not to open nesting but to judge whether L1
**structurally excludes** it, and to record whether a later addition
would be additive or would change existing meaning.

L1 does not structurally exclude it. Making the runtime hold a chain
instead of a single context is a change to one type and its construction
sites, and because the project derives no `Default` for these types,
every construction site surfaces as a compile error rather than
silently defaulting — the same construction-site discipline the IR types
already use. Binder resolution is by name and shadowing is rejected, so
a chain has exactly one candidate for any name and no resolution-order
rule has to be invented.

Crucially, **no existing `.ui` can change meaning**, because nested
`for` is rejected at any depth by both gates today. So the later
addition is additive in the sense that matters: it cannot alter a
program that already exists.

What this record does **not** claim is that nesting is therefore cheap.
The chain is the easy part; the parts that are not sized here are the
`for`-within-`for` cardinality accounting, the removal ordering when an
outer repetition shrinks, and how positions compose for the hover and
focus recurrence conditions below. Those are named as unsized, not
estimated.

### The already-admitted composition: H-a

H-c is not available on inspection. Shipping the capability while a
neighbouring composition of the same two constructs stays broken means
the phase ships a form that compiles, loads and silently does nothing —
and it would be the *only* such form, since the whole point of DD-006 is
that a form which passes both gates should work.

H-b is coherent and is what the phase would have to do if the repair
were large. It is not: the repair is the same parameter L1 threads, so
rejecting the form would cost a diagnostic, a test, and a later removal
of both. H-b would also have to reject a composition that AC9 is
delivering in the same phase, which reads as a rule invented to be
deleted.

H-a has one property worth stating: because the broken behaviour exists
**now**, the red test for the repair can be written against the current
tree before the fix, which is the strongest form of evidence available
for a structural change and does not depend on constructing a wrong
implementation after the fact.

### Lifecycle: W-1, and the recurrence conditions decided by call path

W-2 is excluded by [../requirements/constraints.md](../requirements/constraints.md)
§6, which requires re-materialisation to return to the existing layout
entry and forbids new geometry, scale-cache or Composition writers. It
is listed only so the exclusion is a decision rather than an omission.

W-1 is not a free ride, because a per-item conditional's insertions and
removals happen **inside a repeated region**, which is a different
exposure from the existing top-level conditional. The Phase 2
carry-forward conditions are dispositioned by call path:

| Condition | Fires? | Reasoning from the call path |
|---|---|---|
| **CF-T4-1** — hover retains an index path; a structural shift can make an in-range path name a different node | **Yes** | The pointer can be over a thumbnail while a sibling conditional inside the same repeated region appears or disappears. The retained path's tail can then name a different node. Bounds checks do not catch it |
| **CF-6** — registry lookup by raw pointer identity across a synchronous drain | **Yes, conditionally** | Reachable when a handler's state write flips a per-item condition, so the drain allocates or frees a widget before the registry is re-queried. Whether the gallery's own consumers reach it depends on which handlers write the discriminant; the audit has to be per handler, not per phase |
| **CF-T9-4** — invocation-time binder resolution has one discriminating test | **Yes** | This record touches loop-scope propagation directly. The named test must not be deleted or narrowed, and the repair must extend it to a handler **inside** a conditional inside a `for` |
| **CF-T7-1 / CF-T9-1** — focus anchors are node addresses; allocator reuse can retain focus on a fresh same-address node | **Yes** | A per-item conditional that contains, or shifts the position of, a focusable node reaches this. The two existing fixtures (the allocator observer and the deterministic presentation fixture) must not be removed or weakened |
| **Focus presentation after structural update** | **Repaired baseline** | Goes through the single focus writer repaired at M4-Phase 2 T13a. Do not add a second presentation path |

Three of the five fire, which is the reason this record's evidence
obligation is a structural side-effect enumeration and a call-site
audit rather than a screenshot. What the table does **not** do is
re-open Phase 2's identity policy: pointer anchors stay, and a change to
them needs a Phase 2 successor.

### Condition contents and the closed boundaries

The condition is a `bool` expression under DD-001's uniform rule, so
what a per-item condition may contain follows from DD-001 rather than
from a table here: the element binder (typed by the collection's element
type), the index binder (`i32`), component state, and a comparison over
them. A bool-typed element binder is directly usable as a condition; a
non-bool binder is usable through a comparison. No separate per-item
admission rule is created.

The boundaries that stay closed are the M3-Phase 7 ones, unchanged:
single-widget conditional body, no bare nested control flow directly in
an `if` or `for` body, no nested `for` at any depth, no binder
shadowing, no multi-widget or member-range body, and no key identity.

### Positional semantics

`ForItemEvalContext`'s reads are tracked, so a condition reading the
element binder re-runs when the collection signal is dirty, and a
condition reading `index` and a state re-runs when the state is dirty.
A same-length replacement therefore re-evaluates each surviving
position's condition against the **new** value at that position, which
is the existing positional contract and needs no new mechanism —
provided the effect is created in an item-aware context in the first
place, which is what L1 supplies.

## Recommendation

- **L1** — the enclosing `ForItemContext` is threaded through the three
  seams that drop it today: `append_static_member`'s `If` arm,
  `register_conditional_binding` (which evaluates the condition in a
  `ForItemEvalContext` rather than a `BindingEvalContext`), and
  `mutate_conditional_subtree` (which calls
  `build_node_with_loop_context`). One parameter, one owner, both for
  condition evaluation and for re-materialisation.
- **The subtree's `set_loop_scope` is written from the same parameter**,
  so a handler inside a per-item conditional resolves its binders at
  invocation exactly as a handler directly in the `for` body does. Fixing
  the condition without fixing the subtree — or the reverse — is the
  half-repair this record exists to prevent.
- **H-a** — the currently-admitted composition is repaired, not
  rejected. The red test is written against the present tree before the
  repair lands.
- **Condition contents follow DD-001.** Element binder, index binder,
  component state, and comparisons over them; no per-item-specific
  admission table.
- **W-1** — re-materialisation returns to the existing
  `mark_layout_dirty_for` → `flush_layout` →
  (`sync_scopes_to_tree`, layout, `sync_visuals`) path. No new
  structural writer, no new geometry or scale-cache writer, no second
  focus-presentation path.
- **The three firing recurrence conditions are task-start gates**, and
  their dispositions are close artifacts: a hover-path check for
  CF-T4-1, a per-handler call-path judgement for CF-6, an extension of
  the CF-T9-4 discriminating test to a handler inside a per-item
  conditional, and preservation of both CF-T7-1 fixtures.
- **The M3-Phase 7 structural boundaries are unchanged.**
- **`docs/dsl_spec.md` moves**: §4.14 (conditions may read loop binders;
  the condition's evaluation context), §4.15 (per-item conditionals and
  what they mean for positional identity), §8.11 if the loader's
  validation text needs the composition stated.

## Forward-compat exposure

- **Multi-level `for` and nested structural control flow are not
  structurally excluded**, and no existing `.ui` could change meaning if
  they arrived, because nesting is rejected at both gates today. The
  runtime change is a single context becoming a chain, surfaced by
  compile errors at every construction site. The parts **not** sized
  here are nested cardinality accounting, removal ordering when an outer
  repetition shrinks, and how composed positions interact with the hover
  and focus recurrence conditions above. Naming them unsized is the
  point; claiming they are small would be the forward-compatibility
  assertion this set is not entitled to make.
- **Keyed identity remains the reopening**, not an addition. "The
  condition belongs to a position" is a joint consequence of positional
  identity and live re-reading, the same joint consequence M4-Phase 2
  recorded for per-item handlers. A keyed opt-in would have to re-decide
  both together.
- **`else` remains the recorded family extension point.**
  `ControlFlowNode::If { branches }` is already a branch list, so the
  IR shape does not need to change for it; nothing here narrows that.
- **A multi-widget conditional body is additive** and unaffected by this
  record's choices; it is excluded by scope, not by mechanism.
- **A per-item conditional wrapping a modal scope or a focus group is
  expressible** under this record and is untested in this phase. It is
  named because Phase 2's scope-entry semantics are presence-based, and
  a scope whose presence is per-item is a combination no consumer needs
  yet.

## Technical risk re-evaluation

- **The half-repair is the phase's most likely silent failure.** Fixing
  condition evaluation without fixing re-materialisation gives a
  conditional that toggles correctly and renders a subtree whose binder
  reads fail; fixing re-materialisation without fixing evaluation gives
  a subtree that never appears. Both look like "mostly working". The
  evidence must exercise **false → true → false with a binder read
  inside the body**, not a body of static content.
- **The first materialisation is a separate case from the toggle.**
  Because `EffectHandle::new` runs immediately, a conditional that is
  true at load takes the mutation path once at startup. A test that only
  toggles after load never exercises it, and a test that only checks the
  startup state never exercises the toggle.
- **Structural change inside a repeated region is where the Phase 2
  residuals become reachable**, and three of the five carry-forward
  conditions fire. The close artifact is a structural side-effect
  enumeration for the new insert and remove call sites — listing effect
  disposal, handler registration, focus anchors and hover paths — checked
  against ground truth, not asserted.
- **Same-length replacement is the discriminating positive control** for
  live re-reading. A conditional that captured its item at build time
  produces an identical screen to one that re-reads, for every test that
  never replaces the collection in place.
- **The visible evidence needs a negative leg.** A per-item marker that
  is always present looks the same as one that is correctly present for
  the selected item, in any single frame. The control is that changing
  the discriminant moves the marker and leaves exactly one.
- **The gallery may not be the right host for the visible proof.** If
  the marker cannot be added to the shipped gallery without a contrived
  UI, the framing already permits a named mechanism fixture on the same
  `.ui` → IR → runtime path; DD-004 decides which, since the marker's
  shape is its subject.
