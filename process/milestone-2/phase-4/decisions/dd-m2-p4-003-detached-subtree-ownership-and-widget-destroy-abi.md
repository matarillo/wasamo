### DD-M2-P4-003 — Detached subtree ownership and widget destroy ABI

**Status:** Accepted

**Context:**
DD-M2-P4-002 = A's `remove_child` and `replace_child` produce
detached widget handles via `WasamoWidget** out_removed` /
`out_old`. Today no `wasamo_*` function returns a free-standing widget
handle: every widget reaches the host as part of a subtree owned by
its eventual `wasamo_window_set_root` target ([abi_spec.md §5
"Ownership transfer"](../../../../docs/abi_spec.md#5-m1-experimental-layer)).
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
