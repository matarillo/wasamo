### DD-M3-P1-008 — Mutation source for the Phase 1 live-propagation evidence

**Status:** Accepted

**Context:**
DD-M3-P1-005 picks `Button.enabled: bool` as the property carrying
the evidence. The remaining question: *what changes the bound
`state ready: bool` value at runtime* so that live propagation
(not just initial value) can be observed?

The previous draft of this DD assumed "host-side via existing C ABI
write path" using `wasamo_set_property`. That assumption is false:
[`wasamo_set_property` (abi.rs L711)](../../../../wasamo-runtime/src/abi.rs#L711)
writes a **widget property** by `(widget*, property_id)`. State
signals live in
[`SignalRegistry` (reactive.rs L389)](../../../../wasamo-runtime/src/reactive.rs#L389)
keyed by state name; the C ABI surface has no `(state_name) →
WasamoValue` entry point. So "host-side mutation of `ready`" is not
something the M2 ABI grants for free — it has to be built.

Four real options exist. The choice constrains DD-M3-P1-004's
`EvalContext` trait shape and the size of the Phase 1 ABI delta.

**Options:**

Option A — Admit handler-side bool writes in Phase 1 (recommended;
flips DD-M3-P1-004 to its Option B)
- Add `set_bool` to `EvalContext`. Extend `evaluate()` so
  `Assign { lhs, rhs: BoolLit }` and `Assign { lhs, rhs:
  BoolPropRead }` are well-typed in handler context. The `.ui` proof
  becomes self-contained: e.g. `Button { on click { ready = false } }`
  (one-way) or two buttons setting `true` / `false` respectively
  (no `!` operator needed — `!` is out of scope per the Phase 1 OOS
  list).

  - What you gain: `.ui`-only proof, no new ABI surface. Symmetric
    with `set_i32` (which exists because the M2 counter handler
    writes it). Phase 8 (selected state) is likely to need handler-
    side bool writes — the concrete construct for A10
    (`selected: bool` attribute vs `ToggleButton` primitive vs
    theming binding) is the open Phase 8 question, but the natural
    toggle constructs all want bool handler-write — so pre-shipping
    `set_bool` in Phase 1 probably removes a Phase 8 prerequisite.
    The strength of this argument depends on Phase 8 deciding in
    favour of a handler-toggle construct; it is supporting evidence,
    not the decisive factor.
  - What you give up: Flips DD-M3-P1-004's recommendation to its
    Option B (`get_bool` + `read_bool_tracked` + `set_bool` —
    full trait surface for bool). One extra trait method and one
    additional `evaluate()` arm. Bounded.
  - **Technical risk:** Low. The shape mirrors the M2 i32 write
    path verbatim.

Option B — Introduce a new C ABI entry point for state writes
(e.g. `wasamo_set_state(component*, name, *WasamoValue)`)
- A permanent, public ABI primitive: "host sets a named state
  signal." The host fixture would call this with `WASAMO_VALUE_BOOL`
  to drive `ready`. The bool tag (`WASAMO_VALUE_BOOL = 3` and
  `v_bool` in [abi.rs L74-90](../../../../wasamo-runtime/src/abi.rs#L74-L90))
  is already in the ABI; only the dispatch arm in
  `wasamo_set_state` would be new.

  - What you gain: A genuinely useful ABI primitive that M3+ hosts
    will want for asynchronous patterns (timer-driven state, I/O
    completion writes, host-side animation parameters). Preserves
    DD-M3-P1-004 read-only stance. Phase 1 ships purely "read on the
    `.ui` side; write on the host side."
  - What you give up: A new public ABI function — a permanent
    addition to the C surface that M6 will freeze. The shape
    deserves its own design pass (component identity model, error
    cases, observer firing semantics, thread-affinity guard). That
    pass arguably belongs in its own ADR, not this phase. Doing it
    here widens Phase 1 from "add a scalar" to "add a scalar **and**
    open a state-write ABI surface".
  - **Technical risk:** Medium. Not the implementation — the design
    surface. State-name resolution across components, observer
    interactions with `Signal::set`, and the relationship with
    handler-side writes (do they collide on the same signal?) are
    not closed.

Option C — Test-only / fixture-only internal hook for state mutation
- Add a non-public Rust function (e.g. `runtime::testing::set_state_bool`)
  used only by Phase 1's evidence test, gated by `#[cfg(test)]` or
  an internal feature flag.

  - What you gain: No public surface change at all. Smallest delta.
  - What you give up: The Phase 1 proof becomes test-only — no
    C / Rust / Zig host can demonstrate the live-propagation
    pipeline outside the runtime crate's own test harness. A12
    (DSL public draft) gains nothing user-visible from Phase 1's
    proof. The pattern doesn't scale to Phase 8 (which needs a real
    user-visible mutation surface for selected-state toggling).

Option D — Initial-value only; no live mutation in Phase 1
- The proof shows `Button.enabled` reflects the *initial* value of
  `state ready: bool = false`. No dynamic write.

  - What you give up: m3-plan Phase 1 explicitly asks for "live
    `WidgetNode` propagation" — initial render does not satisfy that.
    The whole reactive pipeline (`Signal::set` → effect re-run →
    widget writer) goes unexercised on the bool path. Phase 6
    (conditional rendering) would inherit unverified reactive
    plumbing for its bool gate.

**Forward-compat exposure:**
Option A bakes `set_bool` into the trait. If the trait ever gains
generic value handling (post-F5), `set_bool` becomes one of several
type-suffixed methods superseded by a typed `set`; the deprecation
is the same as for `set_i32`, so exposure is symmetric with the
existing M2 surface. Option B adds a permanent public ABI function
whose design space isn't closed; if its details turn out wrong, an
M6-frozen mistake is harder to walk back than a Rust trait method.
Option C builds nothing user-facing — zero forward-compat exposure,
but zero forward-compat *value* either. Option D defers everything,
preserving optionality at the cost of the phase's evidence.

**Recommendation:** Option A. The previous draft's reasoning leaned
on a non-existent ABI route (the M2 ABI does not write state
signals); once that's revealed as soft, the cleanest path is to
admit `set_bool` and let DD-M3-P1-004 flip. Three factors converge
on Option A:

1. The bool handler-write surface is likely needed elsewhere in M3
   anyway — Phase 8 (selected state A10) leaves its concrete
   construct open, but the natural toggle shapes for selected state
   all want bool handler-write. Pre-shipping it in Phase 1
   probably removes a Phase 8 prerequisite at low cost; even if
   Phase 8 ends up not needing it (e.g. A10 ships as a pure theming
   binding), the cost is one trait method that mirrors `set_i32`.
2. The state-write ABI primitive (Option B) is a legitimate but
   *separate* design conversation that belongs in its own DD, not
   piggy-backed onto a scalar-introduction phase. Deferring it
   keeps Phase 1's scope honest.
3. Option C's test-only hook leaves the public surface unconnected
   to the evidence — A12 (spec public draft) sees nothing of the
   pipeline that Phase 1 proves works. Bad spec-evidence ratio.

The previous draft's M2-precedent argument ("M2 added `String` read-
only, so M3 bool should too") doesn't hold on closer reading: M2's
`set_string` is absent because no M2 handler wrote to a string, not
because there is a principle "read-first when introducing a scalar."
The bool case is different — Phase 1's evidence shape *requires*
live mutation, and the cheapest, in-spec route to live mutation is
handler-side `set_bool`.

If Option A is rejected in favour of Option B, this DD spawns a
sibling ADR (`m3-state-write-abi.md` or similar) before Phase 1
proceeds; the m3-plan Phase 1 entry is widened to reflect the
additional ABI scope.

---
