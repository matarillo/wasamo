# M2-Phase 4 — Tree-mutation ABI primitives: Architecture Decisions

**Phase:** M2-Phase 4 (tree-mutation primitives at the stable C ABI)
**Date:** 2026-05-04
**Status:** Accepted (2026-05-05)

## Context

M2 acceptance criterion **A4** ([m2-plan.md](../plans/m2-plan.md#acceptance-criteria),
mirrored from [ROADMAP.md M2](../../ROADMAP.md#m2-foundation)):

> The C ABI gains the tree-mutation primitives required by the reactive
> engine; the experimental layer's all-at-once constructors remain
> available but are no longer the only way to construct UI.

[DD-P6-001](./phase-6-c-abi.md#dd-p6-001--stable-core-scope-at-function-granularity)
deliberately excluded tree construction from the M1 stable core: the
five-area minimum was sized assuming M2 codegen (or IR) would build the
widget tree, leaving no need for host-callable construction primitives.
[DD-P8 "Out of scope"](./phase-8-hello-counter.md) recorded the
matching exclusion at the runtime level — incremental
`append_child`, widget destroy of unattached subtrees, and reparenting
were all out of M1 scope. M1 in fact ships only one mutation primitive
on `WidgetNode`: `append_child`, used at construction time only.

Phase 4's job is to discharge A4: extend the runtime's mutation surface
(insert / remove / replace child; widget destroy; property write
batching as needed) and decide which subset is promoted to the
stable-core C ABI versus left as internal Rust API.

### Reading A4 under DD-M2-P3-001 = A

A4's literal phrasing — "primitives **required by the reactive
engine**" — was written before
[M2-Phase 3 Accepted DD-M2-P3-001 = Option A](./m2-phase-3-handler-exec-location.md#dd-m2-p3-001--where-dsl-inline-handler-bodies-execute)
(runtime-side handler interpreter). Under Option A, handler bodies
mutate property storage via the **internal** `set_property` path, not
across the C ABI boundary (this is the load-bearing argument behind
Option A — see DD-M2-P3-001's reactive-integration paragraph). Phase 5
will compose its dependency tracker on top of that internal write
path; the reactive engine itself does not cross the C ABI.

This means A4's "required by the reactive engine" is descriptive of
the *kind* of primitive (insert / remove / replace; bulk write), not a
literal claim that the reactive engine calls these primitives through
`extern "C"`. The A4 work splits cleanly:

- **Internal Rust mutation primitives** are required by Phase 5
  (reactive engine reordering children, replacing sub-trees on
  conditional branches in M3+) and by Phase 6 (IR loader builds trees
  by walking the IR; needs more than `append_child` once the IR
  expresses incremental updates). These are needed regardless of A4.
- **C ABI promotion** of those primitives is what A4 explicitly asks
  for. In M2 itself, no internal consumer needs the C ABI form — the
  reactive engine and IR loader are both inside `wasamo-runtime` and
  use the internal Rust API directly. The C ABI form serves
  **post-M2 hosts** that want to mutate the tree imperatively without
  going through `.ui` (e.g. dynamically-generated UI from a database
  query, or a future Rust safe-wrapper API that exposes idiomatic tree
  mutation).

The pre-doc proceeds on this reading. The alternative — file a
vision decision record revising A4's wording to drop "required by the reactive
engine" and replace it with "for host use" — is a documentation-only
change and is treated as out of scope for this ADR. If the owner wants
that revision, it is a one-line vision decision record, not a redirect of Phase 4
itself; Phase 4's work product is the same either way.

### Constraints carried in from prior decisions

- **DD-M2-P2-001 = Option B** (IR + runtime interpreter). The IR loader
  inside `wasamo-runtime` builds the widget tree by calling internal
  Rust constructors and mutators. It does **not** call the new C ABI
  primitives. C ABI promotion is for hosts, not for the runtime's own
  IR consumer.
- **DD-M2-P3-001 = Option A** (runtime-side handler interpreter).
  Handler bodies mutate property storage through the internal
  `set_property` path. C ABI primitives are not on the handler hot
  path. Phase 4 does not need to optimise for handler-frequency
  invocation rates.
- **DD-M2-P3-002 = Option B** (separate inline-handler slot vs host
  listener list). Phase 4 mutation primitives must not corrupt either
  slot when removing or replacing widgets — both per-widget storage
  artifacts must travel with the widget through detach / reattach.
- **DD-P6-001** (stable-core scope, five-area minimum). Phase 4 grows
  the stable core with a sixth area: *tree mutation*. The growth is
  permitted because A4 is the M2 acceptance hook DD-P6-001 was sized to
  defer to. The growth must obey the same neutrality test the original
  five areas met: each new function must survive plausible post-M2 DSL
  and binding-author evolution without revision.
- **DD-P6-007** (DLL boundary; Option A — runtime owns runtime-allocated
  memory; bounded lifetimes). Detached subtrees (output of
  `remove_child`) are runtime-owned `*mut WasamoWidget` handles. No
  `wasamo_*_free` exists in the stable core today. Phase 4's detach /
  destroy story has to either keep that property or justify the
  exception explicitly.
- **DD-P6-003** (callback contract; queued emission). The runtime
  guarantees no callback fires while the host is inside a `wasamo_*`
  call. Property batching at the C ABI is therefore already in effect
  *for observer dispatch*: a host loop calling `wasamo_set_property`
  10× in succession sees observers fire only after the 10th call
  returns to the outermost frame. The batching question Phase 4 must
  answer is whether anything *beyond* this existing queueing semantics
  needs a host-visible API.
- **Existing internal builder.** `wasamo_runtime::widget::WidgetNode`
  exposes one mutation method today: `append_child(Box<WidgetNode>)`
  ([wasamo-runtime/src/widget.rs:627](../../wasamo-runtime/src/widget.rs#L627)).
  Insert-at-index, remove, replace, and widget destroy do not exist
  yet — neither at the Rust nor the ABI level. Phase 4 must design and
  implement them before any C ABI promotion is meaningful.

### What "tree mutation" means concretely at M2

The smallest set of operations that satisfies A4's spirit, expressed
in implementation-neutral terms:

1. **append child** — current `append_child`; already present.
2. **insert child at position** — index- or sibling-anchored.
3. **remove child** — detach a child; recover a handle (or destroy in
   place).
4. **replace child** — detach the child at position N and attach a
   different widget there; recover the old handle or destroy in place.
5. **destroy a detached widget** — release a widget that is not
   attached to any parent or window.
6. **bulk property writes on one widget** — for the case where a host
   wants to set N properties as a single observable transaction. (This
   may or may not need a host-visible API; see DD-M2-P4-004.)

Reparenting (move a child from one parent to another) and
heterogeneous bulk operations (set property on widget A and append
child to widget B in one transaction) are intentionally outside this
list. Reparenting can be expressed as remove + insert; transactional
heterogeneous bulk operations have no M2 acceptance hook and would
introduce a transaction object that would itself need a stability
commitment. Both are listed in **Out of scope**.

---

## Summary of proposed decisions

| ID | Topic | Recommendation | Impl risk | Forward-compat exposure |
|---|---|---|---|---|
| DD-M2-P4-001 | Stable-core mutation primitive scope | **Option A** — mutation only (insert / remove / replace / append + destroy); constructors stay experimental | Low | Low |
| DD-M2-P4-002 | Mutation primitive identifier scheme | **Option A** — index-based with `child_count`; no parent pointers | Low | Low |
| DD-M2-P4-003 | Detached subtree ownership / widget destroy | **Option A** — host-owned after detach; `wasamo_widget_destroy` added to stable core | Low–medium | Low |
| DD-M2-P4-004 | Property batching API shape | **Option A** — no new ABI; document existing queue-and-drain as the M2 batching contract | Low | Low |

**Aggregate impl-risk picture.** The only non-trivial impl-risk axis
is DD-M2-P4-003's attached-state tracking, and that risk is bounded
to a per-widget bool and a careful `wasamo_widget_destroy`
precondition. No DD in the recommended package introduces a new
mechanism the runtime hasn't already exercised in shape (DD-P8's
window-destroy sweep is the model for DD-M2-P4-003's subtree
teardown).

**Aggregate forward-compat exposure.** All four DDs recommend the
lowest-exposure option. Phase 4's stable-core delta is intentionally
narrow — five new mutator/destroy functions plus one count query,
no new types, no new error codes, no new threading rules — to keep
the M4 freeze surface auditable. The deferred decisions
(constructor promotion in DD-M2-P4-001, host-visible batching in
DD-M2-P4-004) land more naturally in M3 alongside the DSL spec
draft, where the design forces are visible.

**Pre-doc validation spike.** Not required. The existing
`append_child` exercises the same `Vec<Box<WidgetNode>>` mutation
shape that the four new mutators extend; the
`wasamo_window_destroy` path
([wasamo-runtime/src/abi.rs:210](../../wasamo-runtime/src/abi.rs#L210))
exercises the subtree-teardown shape that
`wasamo_widget_destroy` reuses. The remaining Phase 4 implementation
is bookkeeping (attached-state flag, index bounds checks, registry
sweeps on detach) over already-exercised primitives.

## Out of scope

- **Promoting widget constructors to the stable core.** Deferred to
  M3 alongside the DSL spec public draft, where the constructor
  parameter axes (spacing / padding / alignment / typography style)
  align with DSL grammar work. Until then `wasamo_*_create` symbols
  remain `WASAMO_EXPERIMENTAL`.
- **Reparenting as a primitive.** Expressible as `remove_child` +
  `append_child` (or `insert_child`) using the new ABI; no separate
  `wasamo_widget_reparent` is added. Revisit if profiling shows the
  detach-reattach round trip is materially more expensive than a
  hypothetical fused operation.
- **Heterogeneous bulk operations.** Cross-widget transactions
  (e.g. set property on A and append child to B atomically) are
  out of M2. Decided in M3+ if a concrete host need appears.
- **Host-visible property batching API.** Deferred per
  DD-M2-P4-004; the queue-and-drain semantics suffice for M2.
  Vector form (Option B) and begin/commit (Option C) remain
  candidate shapes for M3+.
- **Sibling-anchored mutation API.** Deferred per DD-M2-P4-002; the
  parent-pointer maintenance cost is paid only when a host need
  appears that index-based mutation cannot serve.
- **Widget destroy events / observability.** No
  `wasamo_widget_observe_destroy` is added. Future addition is
  purely additive over DD-M2-P4-003 = A.
- **Updating A4's wording to reflect DD-M2-P3-001 = A.** A4's
  literal phrasing ("primitives required by the reactive engine")
  is reinterpreted in this ADR's Context section rather than
  rewritten in ROADMAP / VISION. If the owner prefers an explicit
  vision decision record revising A4, that is a one-line change handled
  separately and does not affect Phase 4's work product.
