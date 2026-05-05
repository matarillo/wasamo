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
vision ADR revising A4's wording to drop "required by the reactive
engine" and replace it with "for host use" — is a documentation-only
change and is treated as out of scope for this ADR. If the owner wants
that revision, it is a one-line vision ADR, not a redirect of Phase 4
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

### DD-M2-P4-001 — Stable-core mutation primitive scope

**Status:** Accepted

**Context:**
A4 explicitly puts tree-mutation primitives in the C ABI. The question
is which of the six operations enumerated above are exposed at the C
ABI in M2, and which stay internal-Rust. Internal Rust API is required
for all six regardless (Phase 5 / Phase 6 consumers). The decision is
about which subset crosses the boundary.

**Options:**

Option A — Mutation only; constructors stay experimental (recommended)
- Stable-core promotion: `wasamo_widget_append_child`,
  `wasamo_widget_insert_child`, `wasamo_widget_remove_child`,
  `wasamo_widget_replace_child`, `wasamo_widget_destroy`.
- Property batching: see DD-M2-P4-004; this DD does not commit either
  way.
- Construction primitives (`wasamo_text_create`, `wasamo_button_create`,
  `wasamo_vstack_create`, `wasamo_hstack_create`,
  `wasamo_window_set_root`) stay in the **M1 experimental** layer
  unchanged. Hosts that mutate trees use experimental constructors to
  obtain widgets and stable mutators to compose them.

- What you gain: Cleanly satisfies A4's "no longer the only way to
  construct UI" clause via the DSL path (Phase 6 makes
  `wasamoc`-emitted IR the primary construction route; the M1
  experimental layer is the secondary route, and the new mutation
  primitives are the tertiary route layered on top of either).
  Construction is the design-loaded surface (spacing / padding /
  alignment / typography style — every constructor is a parameter
  set that is going to grow); deferring its stable-core promotion to
  the phase that actually has DSL-level vocabulary (M3 DSL spec
  draft) avoids freezing parameter shapes at M2 for a surface no M2
  acceptance criterion exercises.
- What you give up: Hosts that want to construct trees imperatively
  in M2 still depend on `WASAMO_EXPERIMENTAL` symbols. Acceptable —
  no acceptance criterion demands a stable construction surface in
  M2; A1 routes construction through the DSL.
- **Technical risk: Low.** All five mutator functions are mechanical
  wrappers over Rust API that Phase 4 implements anyway. The
  promotion adds no new failure modes beyond the boundary checks
  every C ABI function does (null pointer, valid widget handle,
  index in range). Header generation method (DD-P6-006 = A,
  hand-written) absorbs the additions as edits to `wasamo.h` plus a
  CI smoke-test extension.

Option B — Mutation + stable constructors (deprecate experimental layer)
- Stable-core promotion: all of Option A's mutators **plus** stable
  versions of `wasamo_text_create`, `wasamo_button_create`,
  `wasamo_vstack_create`, `wasamo_hstack_create`,
  `wasamo_window_set_root`. Experimental constructors are marked
  superseded; the `WASAMO_EXPERIMENTAL` block shrinks toward empty.

- What you gain: A4's "no longer the only way to construct UI"
  reads more strongly — host code can construct UI from the stable
  core alone, with no experimental dependency.
- What you give up: Constructor parameter shape becomes a stable
  commitment in M2. `wasamo_vstack_create` today takes only
  children (spacing / padding / alignment defaulted at the runtime
  side per `abi_spec.md §5`). Promoting it to the stable core means
  either (a) freezing the "no parameters beyond children" shape
  until M4 (and adding setters for each axis post-construction —
  fine but pre-commits the parameter axes) or (b) expanding the
  constructor signatures now (and freezing those expansions at M4).
  Both paths overrun M2-Phase 4's scope: parameter design belongs
  with the DSL spec work in M3.
- **Technical risk: Medium.** Mechanically the same as Option A,
  but the design surface is bigger. The risk is "shapes locked at
  M2 turn out wrong by M3 DSL spec time" — a forward-compat shape
  more than an implementation shape. M3's DSL spec draft is the
  natural place to settle constructor parameter axes (it has to
  enumerate them anyway for grammar reasons); committing them in
  M2-Phase 4 with no DSL grammar to align against is premature.

Option C — Mutation + detach/destroy only; no append/insert/replace promotion
- Stable-core promotion: `wasamo_widget_destroy` (detach optional —
  see DD-M2-P4-003). All `append`/`insert`/`replace` stay internal,
  reachable only through the experimental construction path
  (`wasamo_vstack_create` etc.).

- What you gain: Smallest stable-core growth. Avoids committing to
  an attach API shape until a host requirement appears.
- What you give up: A4 unsatisfied. The experimental layer remains
  the *only* way to compose a tree, just with the additional ability
  to dispose of one. The M2 acceptance criterion requires that
  experimental construction is no longer the only way to construct
  UI; Option C is at best a partial answer.
- **Technical risk: Low** (smallest surface), but the acceptance
  argument is weak — Option C would need the owner to either accept
  a narrowed-A4 reading or open a vision ADR redrafting A4. Not
  recommended without that prior step.

**Recommendation:** **Option A.**

The four-mutator + destroy stable-core surface is the smallest set
that satisfies A4 without pre-committing to the constructor design
work that belongs with the DSL spec in M3. Hosts in M2 obtain widgets
through experimental constructors (acknowledged as transient by the
`WASAMO_EXPERIMENTAL` marker) and compose them through the new stable
mutators; the DSL-driven path (Phase 6) is the primary route for
production code and does not touch the stable-core mutators at all.
Option B's constructor promotion is rejected on premature-freeze
grounds (parameter shape decisions land cleaner alongside DSL grammar
in M3). Option C is rejected on A4-coverage grounds.

The split between Option A's stable-core mutators and the experimental
constructor layer is intentionally asymmetric: mutation primitives are
a small, narrow ABI surface (parent + child + index/anchor + handle
out-param), while constructors carry every per-widget design decision.
Asymmetric promotion lets us freeze the structural-but-design-light
half now without freezing the design-heavy half.

**Forward-compat exposure:** Options differ. The relevant out-of-scope
items are M3 DSL spec finalisation (constructor parameter axes) and
post-M2 hosts that want imperative tree construction without the DSL.

- Option A leaves M3 free to add stable constructors with whatever
  parameter axes the DSL spec settles on, with no prior commitment to
  unwind. The new mutators survive trivially because their signatures
  are about widget *handles*, not widget *types* or *parameters*.
- Option B commits constructor parameter shapes at M2. If M3 DSL spec
  grammar needs different axes, the M2 stable constructors either
  get parallel "v2" siblings (ABI bloat) or get superseded by them
  (ABI churn, defeats the M4 freeze story).
- Option C delays the question one phase at the cost of leaving A4
  unsatisfied; it doesn't reduce forward-compat exposure compared to
  Option A.

This axis reinforces the Option A recommendation: minimum forward-
compat exposure for the M2 phase that has no DSL grammar to align
against yet.

**Technical-risk re-evaluation:** Option A is the lowest-impl-risk
option that meets A4. Option B's risk is design-quality (forward-
compat exposure on constructor shape), not implementability. Option
C is acceptance-coverage-deficient.

---

### DD-M2-P4-002 — Mutation primitive identifier scheme

**Status:** Accepted

**Context:**
DD-M2-P4-001 = A commits to four mutator functions plus destroy. Each
mutator that operates on an existing child must identify *which* child
it acts on. Two coherent schemes are in common use:

1. **Index-based.** `insert_child(parent, index, child)`,
   `remove_child(parent, index, &out_handle)`,
   `replace_child(parent, index, new, &out_old)`. Child position is a
   `size_t`.
2. **Sibling-anchored.** `insert_before(anchor, new)`,
   `insert_after(anchor, new)`, `remove(child, &out_handle)`,
   `replace(old, new, &out_old)`. The child's handle is the
   identifier; the parent is implicit.

Both schemes can express every operation; they differ in API
ergonomics, in how they tolerate sibling-list reshuffling, and in
which categories of bug they invite.

**Options:**

Option A — Index-based, with append as `(index = count)` shorthand (recommended)
- Functions:
  ```c
  WasamoStatus wasamo_widget_append_child(
      WasamoWidget* parent, WasamoWidget* child);
  WasamoStatus wasamo_widget_insert_child(
      WasamoWidget* parent, size_t index, WasamoWidget* child);
  WasamoStatus wasamo_widget_remove_child(
      WasamoWidget* parent, size_t index, WasamoWidget** out_removed);
  WasamoStatus wasamo_widget_replace_child(
      WasamoWidget* parent, size_t index,
      WasamoWidget* new_child, WasamoWidget** out_old);
  WasamoStatus wasamo_widget_child_count(
      WasamoWidget* parent, size_t* out_count);
  ```
- `wasamo_widget_append_child` is a separate entry point rather than
  `insert_child(parent, count, child)` because most call sites are
  appends; making it a distinct symbol both reads better at the call
  site and avoids a count-query round-trip in the most common path.

- What you gain: Direct match to the underlying `Vec<Box<WidgetNode>>`
  storage. Out-of-bounds index is the only failure mode beyond the
  trivial null/handle checks; easy to specify and test. Reads
  naturally for IR-driven construction patterns where the IR walker
  knows positions ahead of time. Iteration patterns (apply N
  insertions during a reactive update) compose without sibling-handle
  bookkeeping.
- What you give up: Index drift under concurrent host-side code is
  possible — if a host reads `child_count`, then inserts at index 3,
  but a callback fires between (DD-P6-003 = A makes this *not*
  happen on the same thread, but the API still allows the host to
  re-enter from another widget's hit-test on a later message), the
  index may not be the one originally meant. In practice this is the
  same "indices are ephemeral" caveat every index-based API has, and
  the queued-emission rule (no callbacks during a host call) makes
  it tolerable.
- **Technical risk: Low.** The internal Rust API maps onto
  `children: Vec<Box<WidgetNode>>` with no transformation. Bounds
  checking is a one-liner per function. Out-param widget handles
  follow DD-P6-007 = A: removed/replaced widgets become detached
  runtime-owned handles (DD-M2-P4-003 elaborates).

Option B — Sibling-anchored
- Functions:
  ```c
  WasamoStatus wasamo_widget_insert_before(
      WasamoWidget* anchor_sibling, WasamoWidget* new_child);
  WasamoStatus wasamo_widget_insert_after(
      WasamoWidget* anchor_sibling, WasamoWidget* new_child);
  WasamoStatus wasamo_widget_remove(
      WasamoWidget* child, WasamoWidget** out_removed);
  WasamoStatus wasamo_widget_replace(
      WasamoWidget* old_child, WasamoWidget* new_child,
      WasamoWidget** out_old);
  WasamoStatus wasamo_widget_append_child(
      WasamoWidget* parent, WasamoWidget* child);  /* keep for the no-anchor case */
  ```
- The runtime needs a parent pointer on every widget to support
  sibling-anchored operations (today widgets only know their children;
  parent pointers are absent). Phase 4 would add a `parent: Option<*mut WidgetNode>`
  field maintained by the mutator implementations.

- What you gain: Hosts identify children by handle, not by index;
  drift is impossible. Reads as "put widget X next to widget Y",
  which matches DOM-style mental models.
- What you give up: Adds a parent-pointer field on every
  `WidgetNode` and a maintenance burden on every mutation
  (insert / remove / replace / append must update parent pointers
  on the moved children, possibly recursively if widgets re-attach
  to a different tree). Aliasing risk: today the runtime relies on
  unique ownership of `Box<WidgetNode>` via the parent's children
  vector; introducing a back-pointer means there are now two paths
  to reach a widget, and any future tree-walking code must be
  careful not to cycle. The full API has no `child_count` because
  hosts navigate by handle; for IR-driven loops that need to apply
  positional changes, this requires extra book-keeping on the
  caller side.
- **Technical risk: Medium.** Parent-pointer maintenance is the
  source of the risk: the M1 widget tree was designed without back-
  pointers; introducing them touches every existing mutation path
  (currently only `append_child`, but Phase 4 expands that surface
  before exposing it). The aliasing-discipline cost is paid once but
  paid in every code path that mutates children.

Option C — Both schemes
- Provide Option A and Option B simultaneously.

- What you gain: Caller picks per-call-site.
- What you give up: Two ABI surfaces for one operation, doubling
  the test matrix and the documentation surface. Parent-pointer
  maintenance is paid for sibling-anchored support whether the
  index-based form is also exposed or not. No M2 acceptance criterion
  benefits from offering both. Premature optionality.
- **Technical risk: Medium–high.** Combines Option B's
  parent-pointer cost with the documentation/maintenance cost of
  two parallel surfaces. Rejected.

**Recommendation:** **Option A.**

Index-based mutation is the natural match for the existing internal
data structure (`Vec<Box<WidgetNode>>`), introduces no parent-pointer
maintenance, and reads naturally for the IR-driven construction
pattern Phase 6 will exercise. The principal Option B advantage —
handle-based identification — is not load-bearing in M2: the only
consumers of these primitives in M2 are hosts the project does not
yet have, and the index-vs-handle question is one the M3 DSL spec
work can revisit (with grammar to align against) if a binding-level
ergonomic argument materialises.

The `_count` query is included for completeness (hosts need it to
write loop bounds, and the runtime trivially knows it). No `_get`
indexer is included; iterating children from the host is not on any
M2 acceptance hook and adds an aliasing question (the returned widget
handle's lifetime is tied to its position in the vector, which moves
under any subsequent `insert`/`remove`).

**Technical-risk re-evaluation:** Option A's risk is the lowest of
the three. Option B's parent-pointer cost is significant relative to
the marginal benefit; Option C compounds Option B's cost without
acceptance gain. Risk reinforces the recommendation.

---

### DD-M2-P4-003 — Detached subtree ownership and widget destroy ABI

**Status:** Accepted

**Context:**
DD-M2-P4-002 = A's `remove_child` and `replace_child` produce
detached widget handles via `WasamoWidget** out_removed` /
`out_old`. Today no `wasamo_*` function returns a free-standing widget
handle: every widget reaches the host as part of a subtree owned by
its eventual `wasamo_window_set_root` target ([abi_spec.md §5
"Ownership transfer"](../abi_spec.md#5-m1-experimental-layer)).
After detach, the widget is owned by neither a parent nor a window —
ownership state new to the ABI. This DD decides what the host can do
with it and how it ultimately gets freed.

**Options:**

Option A — Host-owned after detach; explicit `wasamo_widget_destroy` ABI (recommended)
- After a successful `remove_child` / `replace_child`, the
  `out_removed` / `out_old` handle is **owned by the host** in a
  detached state. The host's options:
  1. Re-attach it to some parent via `append_child` /
     `insert_child` / `replace_child` (ownership transfers to that
     parent).
  2. Destroy it via `wasamo_widget_destroy(WasamoWidget*)` — the
     runtime drops the underlying `Box<WidgetNode>` and any visual
     resources it holds; the handle is invalid after this call.
- `wasamo_widget_destroy` is added to the stable core. Calling it on
  a widget that is currently attached (still referenced by some
  parent) returns `WASAMO_ERR_INVALID_ARG`; only detached widgets
  may be destroyed directly. Attached widgets are released via
  `wasamo_window_destroy` (whole-tree drop) as today.
- This is the **only** stable-core function that frees a runtime-
  allocated object directly via the host's request. It complies with
  DD-P6-007 = A only by virtue of the runtime doing the free
  internally; no allocator crosses the CRT boundary (the host passes
  a handle, the runtime calls `Box::from_raw` and drops on its own
  CRT).

- What you gain: Symmetry with the underlying object lifetime —
  detached subtrees have no parent to free them, so host-driven
  destruction is the only viable scheme. Re-attach is a natural
  follow-on. Token-equivalent ABI cleanliness: the host always
  knows which lifecycle stage a widget is in (attached → owned by
  parent; detached → owned by host; destroyed → invalid).
- What you give up: Adds a destructor function to the stable core,
  which DD-P6-007 = A's "no `_free` in the stable core" spirit
  reads as a precedent break. Mitigated by framing: this is a
  *widget destructor*, not a *memory free* (the runtime handles
  the actual deallocation with its own CRT); the host is requesting
  release of a runtime-owned object, the same shape as
  `wasamo_window_destroy`. The "no `_free`" rule was about strings
  and other runtime-allocated *data buffers* crossing the boundary,
  not about whole-object lifecycle.
- **Technical risk: Low–medium.** Mechanically straightforward
  (`Box::from_raw` + drop). The risk is attached/detached state
  tracking: the runtime needs a way to tell whether a widget is
  currently a child of some parent. The natural mechanism is a
  per-widget `attached: bool` flag maintained by the mutator
  functions, complemented by a registry sweep on
  `wasamo_window_destroy` that severs callbacks for the entire
  subtree (DD-P8 already does this for windows; the mutators
  extend it to detached subtrees).

Option B — Limbo storage: runtime keeps the handle live; host destroys explicitly
- Same external API as Option A (`wasamo_widget_destroy`), but
  internally the runtime maintains an explicit "detached widgets"
  registry. The motivation is to centralise the bookkeeping that
  makes attached/detached state queryable and to give a clear
  shutdown sweep target ("on `wasamo_shutdown`, free everything in
  the detached registry").

- What you gain: Centralised bookkeeping of detached widgets;
  explicit "where do orphaned widgets live" answer. Cleaner story
  for the future hot-reload deferral (post-1.0): a hot-reload
  rebuild can be modelled as "detach old root, install new root,
  then destroy old root" with the limbo registry as the explicit
  staging area.
- What you give up: A new runtime-side data structure (the
  registry) without an M2 consumer. Adds one indirection and one
  HashMap-class structure for what Option A handles with a per-
  widget bool. The hot-reload case Option B optimises for is
  out-of-scope for M2 (post-1.0 deferral); building the
  infrastructure now is speculative.
- **Technical risk: Low–medium.** Same fundamental risk profile as
  Option A but with extra moving parts. Risk shape is "premature
  abstraction" rather than "missed correctness".

Option C — No detach: `remove_child` always destroys; reattach not supported
- `wasamo_widget_remove_child` returns `WasamoStatus` only (no
  out-param); the removed widget is destroyed in the same call.
  `replace_child` similarly drops the old child synchronously.
- Reparenting is not expressible at the C ABI in M2.

- What you gain: Simplest API; no detached-state lifecycle to
  document or enforce. No `wasamo_widget_destroy` symbol needed.
- What you give up: Reparenting is an obvious post-M2 host need
  (drag-and-drop UI, list virtualisation) — disallowing it at the
  M2 ABI shape forces an M3+ ABI extension that adds the
  out-parameter form alongside the destroying form, doubling the
  surface. Future-compat penalty for an M2 simplicity gain that no
  acceptance criterion demands.
- **Technical risk: Low** for M2 itself, but the forward-compat
  penalty is a known cost. Rejected.

**Recommendation:** **Option A.**

A widget destructor in the stable core is the natural counterpart to
the new mutators; without it the ABI cannot express "the host took
this child out and is done with it." Option A frames this as a
*lifecycle*-shaped ABI, not a *memory*-shaped one — the runtime
performs the free internally on its own CRT, and the host signals
release through a handle, not by calling the OS allocator across the
boundary. This is consistent with `wasamo_window_destroy`'s shape
(host requests release; runtime performs it).

Option B's limbo registry is rejected on premature-abstraction
grounds: the hot-reload use case it would serve is post-1.0 and the
M2 reactive engine has no detached-widget workflow at all. Option C
is rejected on forward-compat grounds.

The stable-core API addition:

```c
WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_widget_destroy(WasamoWidget*);
```

Behaviour:
- `NULL` argument: idempotent return `WASAMO_OK` (matches
  `wasamo_window_destroy` shape).
- Attached widget: returns `WASAMO_ERR_INVALID_ARG` with last-error
  message "widget is currently attached; remove it from its parent
  first or destroy the owning window".
- Detached widget: drops the widget and any owned subtree; severs
  registry entries (signal handlers, observers) for the whole
  subtree; handle is invalid after the call.

`wasamo_window_destroy`'s existing semantics (drop the whole
attached tree) are unchanged. Subtree teardown logic factors into
shared helpers used by both paths.

**Forward-compat exposure:** Options differ. The relevant out-of-
scope items are reparenting, hot reload (post-1.0), and the
M3+ DSL/binding question of whether widgets get a richer lifecycle
(e.g. observable destroy events).

- Option A leaves room for reparenting (already supported via
  remove + insert), for hot reload (host scripts the
  detach-install-destroy sequence), and for richer destroy
  semantics (a future `wasamo_widget_observe_destroy` or similar
  is purely additive). The destroyed-widget terminal state is
  unambiguous; future signals-on-destroy don't break it.
- Option B's limbo registry is forward-compatible with the same
  range of futures, but the registry itself becomes a stable-core
  contract with no M2 consumer to validate it; future revisions
  may find the registry shape wrong.
- Option C precludes reparenting at the M2 ABI shape, forcing an
  ABI extension to add it.

The exposure axis reinforces Option A: future evolution is
accommodated additively without committing to internal
infrastructure (Option B's registry) that the M2 acceptance set
does not exercise.

**Technical-risk re-evaluation:** Option A's risk is bounded
(attached-state tracking is small and local). Option B introduces
a registry without a consumer (medium). Option C avoids both but
trades for forward-compat penalty (out-of-scope-but-foreseeable).
Risk reinforces the recommendation.

---

### DD-M2-P4-004 — Property batching API shape

**Status:** Accepted

**Context:**
The Phase 4 plan task list calls for "複数 property write のバッチ化
(Phase 5 invalidation cascade の amortize 用)". The framing carried
in: a host (or the reactive engine) writes N properties in succession,
and observers fire only after all N writes complete, so no observer
sees a partially-applied state.

Two pieces of existing machinery need to be considered before adding
new ABI surface:

1. **`emit::drain_if_outermost`** ([wasamo-runtime/src/abi.rs:369](../../wasamo-runtime/src/abi.rs#L369)).
   `wasamo_set_property` enqueues observer notifications and drains
   them only at the outermost call frame. A host loop calling
   `wasamo_set_property` 10× already gets observer batching for free
   if the loop runs from the *outermost* host frame — which for the
   common case (host calls `wasamo_set_property` from a button click
   handler) it does, because the outer frame is `wasamo_run`'s
   message-loop dispatcher, not the host's loop.
2. **DD-P6-003 = A** (queued emission). Callbacks never fire while
   the host is inside a `wasamo_*` call. By definition, observer
   coalescing for a sequence of host calls is already in effect;
   what's *not* in effect is coalescing of internal mutations the
   host can't sequence (e.g. the reactive engine writing to several
   bound properties as a single conceptual transaction).

The latter — internal coalescing — is the motivation for the plan's
"batching primitive". Under DD-M2-P3-001 = A, the reactive engine is
internal Rust; it can coalesce internally without a host-visible API.
Under DD-M2-P3-001 = B (host-side handler — *not* what was decided),
the reactive engine would have crossed the C ABI and a host-visible
batching API would have been load-bearing. DD-M2-P3-001 = A vacates
that need.

The remaining question is whether host code itself benefits from a
batching API. Two cases:

- **Sequential `wasamo_set_property` calls from a host frame.** Already
  batched by `drain_if_outermost`. No new API needed.
- **Re-entrant writes during an observer callback.** The observer is
  itself running inside `drain_if_outermost`'s loop; subsequent writes
  are added to the same emission queue and dispatched in the same
  drain. Effectively already batched, with a documented FIFO order.

Neither host case has a coalescing gap that a new API would close.

**Options:**

Option A — No host-visible batching API; rely on existing queueing (recommended)
- Phase 4 adds **no** new ABI for batching. The existing
  queue-and-drain semantics (DD-P6-003 = A; `drain_if_outermost`)
  are documented as the M2 batching contract in `abi_spec.md`.
- Internal Rust API gains a private `with_batched_writes` helper
  used by the Phase 5 reactive engine to suppress per-write
  invalidation cascades and re-evaluate dirty bindings once at the
  end of a logical transaction. The helper is private to
  `wasamo-runtime`; no C ABI symbol is added.
- M3+ revisits if a concrete host need appears.

- What you gain: Smallest stable-core growth — Phase 4's actual ABI
  delta is the four mutators + destroy from DD-M2-P4-001/003, no
  more. The reactive engine's internal coalescing is implemented
  where it's used; no premature stability commitment on a batching
  shape that has no M2 host consumer.
- What you give up: If a future host genuinely wants explicit
  begin/commit transactional semantics — observers see the entire
  batch as one notification rather than as a queued sequence —
  Option A doesn't provide it. Adding such a shape later is purely
  additive (new functions, no signature change). Acceptable trade.
- **Technical risk: Low.** No new ABI to risk on. The internal
  Rust helper is private and can evolve freely with Phase 5.

Option B — Vector form: `wasamo_set_properties(widget, prop_array, count)`
- New stable-core function:
  ```c
  WasamoStatus wasamo_set_properties(
      WasamoWidget* widget,
      const uint32_t* property_ids,
      const WasamoValue* values,
      size_t count);
  ```
- All N writes are applied; observers fire only after all N are
  applied (single drain at function exit).

- What you gain: Single ABI call expresses "set these N properties
  on this widget in one transaction". Hosts that build a UI patch
  from a snapshot diff get a natural call shape.
- What you give up: New ABI surface with no M2 consumer. The
  property-id + value parallel-array form is awkward (no built-in
  size validation between the two arrays; tagged-value packing must
  be done call-site). Equivalent to a host-side loop over
  `wasamo_set_property` in observable behaviour, modulo the
  bounded-size validation up-front. M3+ may want a richer batching
  primitive (heterogeneous: set property on widget A and append
  child to widget B in one transaction); Option B's per-widget
  shape is the wrong shape for that future.
- **Technical risk: Low** mechanically; the design risk is "we
  picked the per-widget-N-property shape and the future wants a
  cross-widget shape." Forward-compat penalty without M2 driver.

Option C — Begin/commit scope tokens
- New stable-core functions:
  ```c
  WasamoStatus wasamo_property_batch_begin(uint64_t* out_token);
  WasamoStatus wasamo_property_batch_commit(uint64_t token);
  ```
- Between `begin` and `commit`, all `wasamo_set_property` calls on
  any widget are queued; observers fire on `commit`. Nested
  begin/commits are reference-counted (innermost commit drains
  nothing; outermost drains all).

- What you gain: Most expressive batching shape; supports cross-
  widget transactions.
- What you give up: Two new symbols, a new tokenised lifecycle to
  document, an interaction with the existing `drain_if_outermost`
  semantics that has to be specified carefully (does an explicit
  `begin` suppress drains during inner `wasamo_*` calls?). Premature:
  no M2 acceptance criterion benefits.
- **Technical risk: Medium.** The interaction with the existing
  outermost-drain semantics is the technical risk: today the
  drain rule is a free function at the bottom of every set_property
  call; layering an explicit begin/commit on top means the drain
  decision becomes "outermost-frame AND not inside an explicit
  batch", and every ABI surface that schedules emissions has to
  honour the second clause. Quick to write, careful to verify.

**Recommendation:** **Option A.**

The plan's "batching primitive" framing was written under the
assumption that the reactive engine would cross the C ABI. With
DD-M2-P3-001 = A, that assumption is voided: reactive batching is
internal Rust and needs no ABI commitment. The existing queue-and-
drain semantics already cover the host-loop case for free. Adding
a host-visible batching API now would be a new stable-core symbol
without an M2 consumer; deferring is the lower-cost choice.

`abi_spec.md §6` is amended in Phase 4 to call out the queue-and-
drain semantics as the **batching contract** explicitly (today the
section talks about callback re-entrancy but not about batching
qua batching). Documentation, not code.

If a concrete host requirement appears in M3+, Option B and Option
C are both purely additive — they can land then with a real driver.
The deferral does not foreclose either.

**Forward-compat exposure:** Options differ. The relevant out-of-
scope items are M3+ host-driven UI-patching APIs and the post-1.0
hot-reload work.

- Option A's deferral leaves both Option B's and Option C's shapes
  fully available later. The "M2 batching contract is already in
  the queue-and-drain semantics" framing is non-breaking with
  either future addition.
- Option B locks the per-widget vector shape at M2. If M3 wants
  the cross-widget shape (Option C), Option B becomes a redundant
  parallel surface that has to be maintained alongside.
- Option C's tokenised batch is a strong-shape commitment with no
  M2 driver to validate it. Lock-in penalty if M3+ DSL semantics
  reveal a different cross-cutting transaction shape.

This axis reinforces Option A: deferring the API decision until
there is a real consumer is the lowest-exposure path.

**Technical-risk re-evaluation:** Option A is the lowest-risk
option (no new ABI to risk on). Option B is mechanically low-risk
but design-risk medium. Option C carries a documented integration
risk with the existing drain semantics. Risk reinforces the
recommendation.

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
  vision ADR revising A4, that is a one-line change handled
  separately and does not affect Phase 4's work product.
