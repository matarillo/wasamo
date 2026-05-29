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
