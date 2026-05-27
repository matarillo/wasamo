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
