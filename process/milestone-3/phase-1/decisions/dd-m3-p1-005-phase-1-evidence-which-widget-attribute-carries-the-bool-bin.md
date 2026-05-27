### DD-M3-P1-005 — Phase 1 evidence: which widget attribute carries the `bool` binding?

**Status:** Accepted

**Context:**
The m3-plan ([§Phase 1](../plans/m3-plan.md#phase-breakdown)) requires
"live `WidgetNode` propagation of a `bool`-bound attribute on a trivial
widget that already exists — no new layout primitive is required for
the phase to close." This DD picks the attribute. The chosen attribute
must:

1. Live on an existing M2 widget (`Rectangle | VStack | HStack | Text |
   Button`) — no new widget kind in Phase 1.
2. Be naturally `bool`-typed (not an enum encoded as i32, not a numeric
   threshold).
3. Not conflict with surfaces that later M3 phases own (conditional
   rendering A7 owns subtree presence; selected state A10 owns Button
   selected styling).

**Options:**

Option A — `Button.enabled: bool` with a deliberately narrow Phase 1
contract (recommended)
- A new attribute on the existing `Button` widget. The Phase 1
  contract is intentionally small:

  - **In scope (Phase 1):** declared `bool`-typed property; default
    `true`; when `false`, the button suppresses click-handler dispatch
    and renders in a minimal disabled visual state (greyed colours,
    no animation); layout slot is preserved (the button still
    measures and arranges as if enabled — no `display: none`
    semantics); property type is strictly `bool`, no coercion from
    `i32` / string.
  - **Out of scope (Phase 1, deferred to later milestones):**
    keyboard focusability and tab-order semantics when disabled;
    AccessKit / accessibility tree state (`aria-disabled` equivalent);
    hover / focus visual variations; key activation suppression. The
    M4 input/focus and M5 accessibility milestones own the full
    interaction-state contract for disabled controls.

  Bind `state ready: bool` to `Button.enabled`. The live-propagation
  proof is driven by a `.ui`-side handler writing to `ready` (e.g.
  `Button { text: "disable"; on click { ready = false } }`), made
  possible by DD-M3-P1-004 Option B / DD-M3-P1-008 Option A admitting
  handler-side bool writes.

  - What you gain: Naturally `bool`-typed (no degrees-of-disabled
    semantics to argue about). Orthogonal to A10 (selected) and A7
    (subtree present/absent) — neither phase touches `enabled`.
    Matches the M2 String-binding evidence pattern (a real visible
    widget property drives off a binding). The narrow contract above
    keeps the surface re-openable by M4/M5 without breaking Phase 1's
    proof.
  - What you give up: One new property ID (`PROP_BUTTON_ENABLED`), one
    new `PropertyValue::Bool` enum variant, and a small amount of
    visual styling work for the disabled state. A future ADR (M4
    input or M5 a11y) will widen the contract; Phase 1's narrow
    contract is structured to be additive under that widening, not
    superseded by it.
  - **Technical risk:** Low *given the contract narrowing above*.
    Without the narrowing, the "disabled control" surface drags in
    focus / a11y / keyboard concerns that M4–M5 haven't started yet;
    the contract scope above is the load-bearing reason the risk
    stays Low.

Option B — `Text.visible: bool` (or `Rectangle.visible`, etc.)
- A boolean visibility attribute hidden on layout (visible/hidden).

  - What you gain: Bool-typed, applies to any widget uniformly.
  - What you give up: Opens a layout-semantics question Phase 1 should
    not own — does `visible: false` reserve the layout slot
    (`visibility: hidden`) or release it (`display: none`)? Either
    choice pre-empts a design conversation that belongs in M3-Phase 6
    when conditional rendering ships. Doing visibility in Phase 1 and
    then conditional rendering in Phase 6 risks two overlapping
    surfaces with subtly different semantics.

Option C — Bind `bool` to an existing attribute by coercing through
`i32` (e.g. `Button.style` toggled by a bool→i32 cast)
- No new attribute; reuse `Button.style` which already exists as i32.

  - What you give up: Requires coercion semantics (`true → 1`,
    `false → 0`) which is exactly what DD-M3-P1-001 Option B was
    rejected for. Defeats the type-tagging discipline. Also produces
    weak evidence — the proof would show coercion working, not bool
    propagation.

Option D — No widget attribute; prove propagation by reading bool
state from a handler that prints to stdout
- Skip the WidgetNode portion entirely.

  - What you give up: The plan explicitly requires WidgetNode
    propagation as Phase 1's evidence shape. Going below that bar
    leaves A9 understaffed and forces the next phase to relitigate
    "how does bool actually reach a widget."

Option E — Internal `Button.bool_probe: bool` (or similar) not in the
public DSL spec
- Add a property ID and `PropertyValue::Bool` plumbing for a bool
  attribute that exists in the runtime widget catalog but is **not**
  exposed in `docs/dsl_spec.md` and is not parseable from `.ui`. The
  Phase 1 evidence test wires it through `wasamoc` lowering paths
  reachable via internal test helpers only (or via the IR text
  directly, bypassing the surface parser).

  - What you gain: Phase 1 ships the full bool plumbing
    (IrType / IrLiteral / HandlerExpr / EvalContext / `PropertyValue`
    / writer / `PROP_*` id) without committing the public widget
    spec to any specific bool attribute. Defers `Button.enabled` (or
    any other public bool attribute) to the phase that needs it
    (Phase 6 / Phase 8 / a future input ADR), giving that phase
    full latitude on contract scope (focus, a11y, etc.). The "weight
    of disabled-control semantics" objection vanishes from Phase 1.
  - What you give up: A12 (DSL public draft) sees no bool widget
    attribute from Phase 1's work. The propagation pipeline is
    proven in code but not in user-visible spec; the spec growth for
    A9 is limited to the scalar type, literals, and grammar — not
    a widget attribute. Phase 1's "live proof" is harder to
    demonstrate to an external reader of the spec (it requires
    reading test code).

**Recommendation:** Option A with the narrowed contract described
above. `Button.enabled` is the cleanest public surface for the
evidence and is a real attribute Phase 8 / M4 / M5 will need anyway;
the narrowing keeps interaction-state weight out of Phase 1.

The owner's review surfaced concern that click suppression itself
already drags Phase 1 toward interaction-state territory. The
counter-argument: click suppression is a one-line dispatch guard
inside `Button`'s click handler invocation; it does **not** require
focus tree integration, a11y plumbing, or keyboard handling, and
those are deferred explicitly in the contract scope above. If even
that lightweight semantic is too much for Phase 1, Option E is the
fallback: ship the bool plumbing without committing the public spec
to any specific attribute. The decision between Option A (narrow
public surface) and Option E (no public surface for the attribute)
is the load-bearing trade-off for owner agreement.

**Forward-compat exposure:** Option A is additive to the public
widget surface; the narrow contract is structured to be additive
under M4/M5 widening (focus, a11y, keyboard), not superseded by it.
Option E preserves maximum public-surface flexibility but offers no
spec evidence for A9 beyond the scalar/literal/grammar additions.

---
