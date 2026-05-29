### DD-M2-P5-003 — Effect lifetime and disposal

**Status:** Accepted

**Context:**
DD-M2-P5-001 = B introduces Effects (re-runnable closures) registered
with the engine. Each Effect holds a reference into the dependency
graph (Signals point at it; on Signal write, dirty marks propagate to
it). When the bound widget is removed from the tree (Phase 4
`remove_child` / `replace_child` / `widget_destroy`), the Effect must
be disposed so:

1. Signals stop pushing dirty marks at a defunct Effect.
2. The Effect's closure (which captures references into widget
   property storage) is dropped before the widgets it captures.
3. A re-attach of the same widget to a different parent does not
   resurrect a stale Effect.

This DD decides whose responsibility disposal is and how it is
threaded through the existing widget lifecycle.

**Options:**

Option A — Effects are owned by the widget that hosts the binding (recommended)
- Each `WidgetNode` gains a `bindings: Vec<EffectHandle>` field
  ([wasamo-runtime/src/widget.rs](../../wasamo-runtime/src/widget.rs)).
  Phase 6's IR loader, when it lowers `Text { content: "..." }`,
  creates an `Effect` and pushes its handle onto the widget's
  `bindings`.
- Disposal is automatic on widget drop: `Drop for WidgetNode`
  iterates `bindings` and calls `engine.dispose_effect(handle)`,
  which removes the Effect from every Signal's dependent set and
  drops the closure.
- `wasamo_widget_destroy` (Phase 4) and `wasamo_window_destroy`
  drop subtrees through Box ownership, so binding disposal piggy-
  backs on the existing teardown sweep with no new ABI surface.
- The Phase 4 `attached: bool` flag is unrelated to effect
  registration: an Effect is registered the moment the IR loader
  creates it (regardless of whether the widget is yet attached to a
  window). The dependency graph holds the effect live until the
  widget Drop-fires.

- What you gain: Disposal is structural, not bookkeeping —
  ownership of the Effect mirrors ownership of the widget, and
  every existing teardown path (window destroy, widget destroy,
  remove_child + drop) handles bindings without new code paths.
  No "leaked Effect whose target widget is gone" failure mode.
- What you give up: Each `WidgetNode` carries one extra `Vec`
  field (often empty in M2 — only Text widgets with bound content
  get an entry). Trivial.
- **Technical risk: Low.** Existing `Drop` paths and the Phase 4
  subtree-teardown sweep are the integration surface; both already
  exist. The new code is a single iterator in `Drop for WidgetNode`
  plus the `dispose_effect` engine method.

Option B — Effects are owned by the engine; widgets reference by handle
- The engine maintains the authoritative `HashMap<EffectId, Effect>`.
  Widgets store an `EffectId` (an opaque integer); on widget drop,
  some external mechanism is responsible for telling the engine to
  free the corresponding entry.
- The "external mechanism" is either: (a) a Drop impl that calls
  `engine.dispose_effect(id)` (functionally identical to Option A),
  or (b) a sweep at outermost-frame boundaries that walks live
  widgets and reaps orphaned Effects.

- What you gain: Centralised registry shape — useful if Effects
  ever need to be enumerated by the engine (e.g. for a "force
  flush all" debug command).
- What you give up: Sub-option (a) is Option A in disguise; sub-
  option (b) requires the engine to walk the widget tree, which is
  the kind of registry-with-no-clear-owner pattern M2 has been
  avoiding (cf. DD-M2-P4-003 = A's rejection of the limbo registry).
  Adds an integer-handle layer with no benefit Option A doesn't
  also have.
- **Technical risk: Low–medium.** Sub-option (b) introduces a
  reaper sweep that has to run at the right moment and can leak if
  the trigger is missed. Sub-option (a) is just Option A with
  extra indirection.

Option C — Manual disposal via explicit `unbind` calls
- Phase 6's IR loader returns `EffectHandle` to the host (or to a
  binding-tracking layer); explicit cleanup is required at widget
  removal time.

- What you gain: Maximum control.
- What you give up: Phase 6 has to generate disposal calls for
  every `remove_child` / `replace_child` it emits, doubling the
  per-mutation work and risking leaks. Re-attach (M3 conditional
  bindings will rebuild subtrees) becomes "destroy old effects,
  rebuild new effects" with no help from the structural mechanism.
- **Technical risk: Medium.** Manual disposal scales poorly with
  M3 structural bindings. Rejected.

**Recommendation:** **Option A.**

Effect ownership mirrors widget ownership: an Effect bound to a
widget's property is owned by that widget and disposed when the
widget drops. This makes binding lifecycle structural rather than
bookkept — every existing teardown path (Phase 4
`wasamo_widget_destroy` subtree sweep, `wasamo_window_destroy`
whole-tree drop, plain `remove_child` + `drop`) handles binding
disposal correctly with no new ABI and no engine-level reaper.

The `WidgetNode.bindings` field is `pub(crate)` (no C ABI
exposure); Phase 6's IR loader populates it during construction.
The `Drop for WidgetNode` impl forwards each handle to
`reactive::dispose_effect`, which removes the effect from every
Signal's dependent set and drops the closure.

Re-attach (M3 conditional binding rebuilds a subtree at a different
position) just creates fresh Effects on the new widgets; old widgets
go through normal Drop, which disposes their old Effects. No
explicit hook needed.

**Forward-compat exposure:** Options differ. The relevant out-of-
scope items are M3 structural bindings (conditional / for-loop, which
rebuild subtrees), M3 Computed (which has its own lifetime), and
post-1.0 hot reload (which destroys whole graphs at once).

- Option A's structural ownership extends naturally: Computed nodes
  are owned by whoever creates them (an enclosing Effect, or the
  engine if they outlive the cycle); structural-binding subtree
  rebuilds dispose old Effects via Drop and create new ones; hot
  reload's whole-tree teardown disposes everything via root drop.
- Option B (sub-option b reaper) accumulates risk per future shape:
  Computed adds another registry, structural bindings add another
  trigger, hot reload adds another sweep moment.
- Option C does not scale to structural bindings without additional
  scaffolding.

This axis reinforces Option A: ownership-first design composes with
foreseeable growth without engine-side bookkeeping.

**Technical-risk re-evaluation:** Option A's risk is the smallest;
the integration surface is existing Drop paths plus one engine
method. Option B's reaper sub-option introduces correctness risk
without acceptance benefit. Option C's manual disposal is high-cost
and scales poorly. Risk reinforces Option A.

---
