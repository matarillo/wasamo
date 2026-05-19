# M3-Phase 1 — `bool` scalar binding: Architecture Decisions

**Phase:** M3-Phase 1 (`bool` scalar binding)
**Date:** 2026-05-19
**Status:** Drafting

## Context

M3 acceptance criterion **A9** (see
[ROADMAP.md M3](../../ROADMAP.md#m3-dsl-surface),
[m3-plan.md](../plans/m3-plan.md#acceptance-criteria)):

> `bool` admitted as the third scalar binding type alongside `i32` and
> `String`. The `TypedValue` generic value union remains deferred.

The M3 plan ([m3-plan.md §Phase breakdown](../plans/m3-plan.md#phase-breakdown))
places this as Phase 1 because it is the **hard prerequisite** for
M3-Phase 6 (conditional rendering grammar A7 rides on a `bool` binding)
and M3-Phase 8 (Button `selected` state A10 rides on a `bool` binding).
Phase 1 closes when `bool` threads through the same `wasamo-ir` ↔
`wasamoc` ↔ `wasamo-runtime` path that `i32` and `String` already
travel, with one live `WidgetNode` attribute proving propagation; the
grammar surfaces that consume `bool` are out of this phase.

The M2 end-state shape that this phase must extend without breaking:

- `wasamo-ir` ([wasamo-ir/src/lib.rs](../../wasamo-ir/src/lib.rs)):
  `IrType` has two variants `I32 | Str`; `IrLiteral` has
  `Int | Str | Ident`; `HandlerExpr` uses **type-suffixed variants**
  (`IntLit` / `StrLit` / `PropRead` / `StrPropRead`) rather than a
  unified typed value.
- `EvalContext`
  ([wasamo-runtime/src/handler.rs](../../wasamo-runtime/src/handler.rs)):
  type-suffixed methods (`get_i32` / `get_string` /
  `read_i32_tracked` / `read_string_tracked` / `set_i32`). `set_string`
  is **absent** — strings are read-only in M2 because no handler writes
  to them. `evaluate()` returns `Result<i32, EvalError>`; binding-side
  evaluation has a separate string-typed path.
- Widget catalog
  ([wasamo-runtime/src/widget.rs](../../wasamo-runtime/src/widget.rs)):
  `Rectangle | VStack | HStack | Text | Button`; `PropertyValue` enum is
  `I32(i32) | String(String)`; per-widget per-attribute `PROP_*` u32
  IDs in [ir_loader.rs](../../wasamo-runtime/src/ir_loader.rs) lines
  799–802.

This ADR is framed against A9 and the M2 type-suffix pattern. It does
**not** re-open F5 (`TypedValue` deferral) — adding `bool` as a third
scalar is a different question, as recorded in
[m3-target-app-predoc.md — Tabs / Button selected-state surface closure (Reservation 3)](../notes/m3/m3-target-app-predoc.md#保留-3-closure-tabs--button-選択状態-surface--採用-bool-を-3-つ目の-scalar-として導入).

The acceptance lens for this phase is narrow: A9 is satisfied when
`bool` reads through the live `.ui → IR → runtime` path on one widget
attribute. Consumers of `bool` (conditional rendering, Button selected)
are explicitly out of scope here.

---

### DD-M3-P1-001 — `IrType` extension

**Status:** Drafting

**Context:**
`IrType` is a two-variant enum (`I32 | Str`) that tags `state`
declarations and disambiguates the type-suffixed `HandlerExpr` variants.
A9 requires that `bool` becomes a first-class declaration type, so
`state foo: bool = false` parses and the resulting `IrState` carries
something distinct from `I32` and `Str`.

**Options:**

Option A — Add `IrType::Bool` variant (recommended)
- `IrType` becomes `I32 | Str | Bool`. Additive.

  - What you gain: One-to-one with the surface-level type vocabulary.
    Pattern-matching exhaustiveness in `wasamoc` / `wasamo-runtime`
    forces every site that branches on type to handle `Bool` —
    compiler-enforced completeness.
  - What you give up: Every existing `match` on `IrType` in the
    workspace needs a `Bool` arm. The set is small and discoverable;
    no abstraction debt.
  - **Technical risk:** Low. Pure enum extension; no FFI / wire format
    changes outside this phase's own work.

Option B — Encode `bool` as `IrType::I32` with a flag/refinement
- Reuse `I32`; treat `0`/`1` as falsy/truthy throughout.

  - What you gain: Zero new variant.
  - What you give up: Loses the type tag at the IR boundary, which is
    where M2 deliberately placed it (DD-M2-P6-002 chose tagged
    representation for exactly this reason). Re-opens the typing
    discipline of a settled DD.
  - **Technical risk:** Low to implement; high to live with — every
    later phase that touches `bool` would need to re-derive "is this
    bool-typed or i32-typed?" from context.

**Forward-compat exposure:**
Option A's exposure under foreseeable future events (see Out of scope):
when `TypedValue` is reconsidered after M3, an `IrType::Bool` variant
naturally maps to a `TypedValue::Bool` arm — strictly additive. Option
B would require *splitting* `I32` into `I32` + `Bool` retroactively,
which is exposure to the same future event but reversed.

**Recommendation:** Option A. The type-suffix pattern is the M2
discipline; extending it for `bool` is the additive path. Design
quality dominates here: a refinement-flag scheme would be a footgun
the rest of M3 has to navigate around.

---

### DD-M3-P1-002 — `IrLiteral` extension and surface syntax

**Status:** Drafting

**Context:**
Once Option A of DD-M3-P1-001 is taken, `IrLiteral` needs to carry a
bool constant for `state foo: bool = <default>` and (eventually) for
literal-driven bindings. Two sub-questions: which variant shape, and
which surface syntax does `wasamoc`'s lexer/parser accept.

**Options for the IR literal variant:**

Option A — Add `IrLiteral::Bool(bool)` (recommended)
- Parallel to `IrLiteral::Int(i32)` and `IrLiteral::Str(String)`.

  - What you gain: Pattern symmetry with the rest of `IrLiteral`.
  - What you give up: Nothing structural.
  - **Technical risk:** Low.

Option B — Encode bool literals as `IrLiteral::Int(0|1)`
- Reuse the integer literal variant; type-context distinguishes.

  - What you give up: Same objection as DD-M3-P1-001 Option B: erases
    type at the IR boundary. Inconsistent with `Str` being its own
    variant.

**Options for the surface syntax:**

Option A — `true` / `false` keywords (recommended)
- Add two reserved identifiers to the lexer; parser produces
  `IrLiteral::Bool(true|false)`.

  - What you gain: Universal across DSL ancestry (XAML, SwiftUI, Slint,
    QML, CSS, every C-family language). Zero ambiguity with
    `IrLiteral::Ident` because the keywords are recognised at the
    lexer.
  - What you give up: Two reserved words. `true` and `false` are not
    plausible identifier choices in `.ui` (they're keywords or builtin
    constants in nearly every reasonable target language a `.ui` author
    is also literate in), so the reservation is essentially free.
  - **Technical risk:** Low. Lexer additions are localised in
    [wasamoc/src/lexer.rs](../../wasamoc/src/lexer.rs).

Option B — Reuse integer literals (`0` / `1`) with type coercion
- No new lexer tokens; `state foo: bool = 1` parses.

  - What you give up: Forces every reader of `.ui` (human and tooling)
    to remember a bool-context convention. Diverges from every prior
    DSL the project is influenced by.

Option C — Defer surface syntax entirely; expose `bool` only through
the host (e.g. component default set from C)
- `IrType::Bool` and `IrLiteral::Bool` exist; `.ui` cannot write a
  bool literal yet.

  - What you give up: Phase 1's E2E proof becomes awkward (the `.ui`
    side cannot declare `state foo: bool = false`). M3-Phase 6
    inherits the surface-syntax decision under deadline pressure when
    conditional rendering needs `if foo` (and `if true` for spec
    examples). No real saving — defers irreducible work.

**Recommendation:**
- IR variant: Option A (`IrLiteral::Bool(bool)`).
- Surface syntax: Option A (`true` / `false` keywords).

**Forward-compat exposure:** Adding `true` / `false` keywords is
irreversible only in the sense that removing them later would be a
breaking change; doing so is not on any plausible roadmap, so exposure
is nil.

---

### DD-M3-P1-003 — `HandlerExpr` variants for `bool`

**Status:** Drafting

**Context:**
`HandlerExpr` is the shared IR ([wasamo-ir/src/lib.rs L28–L49](../../wasamo-ir/src/lib.rs#L28-L49))
between `wasamoc::lower`/`emit` and the runtime evaluator. M2 chose the
type-suffix pattern (DD-M2-P6-003 = Option A): `IntLit` / `StrLit` for
literals, `PropRead` (i32) / `StrPropRead` (string) for property reads.
The Phase 1 question is how `bool` joins this enum.

Per DD-M3-P1-004 Option B and DD-M3-P1-008 Option A, Phase 1 admits
handler-side bool writes through the existing `HandlerExpr::Assign`
variant (no new write variant introduced — `Assign` already exists
from M2 and only its `rhs` set widens to include `BoolLit` and
`BoolPropRead`). The evaluator-side widening is recorded in DD-004 /
DD-008; this DD records only the new literal / property-read
variants. `CompoundAssign` over bool remains out of scope (no
naturally bool-typed `CompoundOp` exists; see Out of scope below).

**Options:**

Option A — Add `HandlerExpr::BoolLit(bool)` and
`HandlerExpr::BoolPropRead { path }`, mirroring the `Str*` pattern
(recommended)
- Additive variants; no rename of existing variants. The implicit
  `PropRead` / `IntLit` retain their (i32) typing by convention,
  matching the M2 status quo.

  - What you gain: Continues the DD-M2-P6-003 discipline without
    modification. Every site that already handles `Str*` learns the
    same pattern for `Bool*`. Pattern-match exhaustiveness compiler-
    enforces completeness.
  - What you give up: Mild asymmetry remains — `PropRead`/`IntLit`
    *implicitly* mean i32 while `Str*` and `Bool*` carry explicit
    suffixes. This is a pre-existing M2 wart; this DD does not fix it
    (see Option C).
  - **Technical risk:** Low.

Option B — Unify all literals and property reads into a single typed
form: `HandlerExpr::Lit { value: TypedLiteral }` and
`HandlerExpr::PropRead { path: String, ty: IrType }`
- Replaces the type-suffix pattern with type-on-variant.

  - What you gain: Cleaner shape; the type is a first-class field.
  - What you give up: Re-opens DD-M2-P6-003. Touches every existing
    `HandlerExpr` match site in `wasamoc` lowering, the IR text
    emitter, the runtime evaluator, and the IR loader. Also drifts
    toward `TypedValue` (F5 deferral) — the whole point of the
    type-suffix pattern was to defer that union. Out of phase scope.
  - **Technical risk:** Medium — large refactor surface across two
    crates and the IR text spec.

Option C — Option A plus renaming `PropRead` → `IntPropRead` and
`IntLit` left as-is (or also renamed) for symmetry
- Cosmetic clarity: every variant's type becomes explicit in the name.

  - What you gain: Symmetry. A reader of `HandlerExpr` no longer needs
    to know "`PropRead` happens to mean i32 because i32 was first."
  - What you give up: Rename churn across `wasamoc` and runtime
    evaluator for a payoff that is purely readability. Touches the IR
    text grammar (`PropRead` vs `IntPropRead` token spelling) and so
    bumps the spec normatively without functional change.
  - **Technical risk:** Low; cost is reviewer attention, not
    correctness.

**Recommendation:** Option A. The rename in Option C is tempting and
the right end-state, but the phase pays its scope dividend by *not*
re-opening M2 IR grammar text. If a future phase touches the IR text
grammar substantively, fold the rename in there.

**Forward-compat exposure:** Option A leaves the M2 asymmetry in place.
If `TypedValue` is later admitted (which would supersede the
type-suffix pattern in its entirety), the rename in Option C would be
discarded anyway, so its absence does not increase exposure.

---

### DD-M3-P1-004 — `EvalContext` method shape for `bool`

**Status:** Drafting

**Context:**
`EvalContext` is the trait through which `HandlerExpr` evaluation
reaches the host's reactive store. M2 added string reads as
`get_string` + `read_string_tracked` (both with default impls that
error or forward) but did **not** add `set_string`, because no handler
writes to a string in M2.

Phase 1's evidence shape (per DD-M3-P1-008 Option A) requires live
mutation of bound `bool` state from a `.ui`-side handler, which means
the trait has to admit a bool-typed write. `CompoundAssign` over
`bool` is not exercised — there is no naturally bool-typed
`CompoundOp` — but plain `Assign { rhs: BoolLit | BoolPropRead }` is.

**Options:**

Option A — Add `get_bool` + `read_bool_tracked` only; no `set_bool`
- Mirrors the M2 String shape exactly. Sufficient only if some
  *other* path drives `Signal<bool>::set` (host-side ABI, test hook,
  or no live mutation at all — see DD-M3-P1-008).

  - What you gain: Minimal trait surface.
  - What you give up: Phase 1's evidence cannot be a `.ui`-only
    proof; the mutation source has to come from elsewhere. Once
    DD-M3-P1-008 lands as anything other than its own Option B
    (separate state-write ABI) or C (test hook), this trait shape
    becomes insufficient.
  - **Technical risk:** Low to implement; load-bearing on
    DD-M3-P1-008 picking a non-handler route.

Option B — Add the full `get_bool` + `read_bool_tracked` + `set_bool`
(recommended)
- Eager symmetry with `i32`. Pair with one bool-typed arm in
  `evaluate()` for `Assign { rhs: BoolLit | BoolPropRead }`.

  - What you gain: `.ui`-only live-propagation proof (e.g. a button
    with `on click { ready = false }`). Mirrors `set_i32` which
    already exists for the M2 counter handler. Phase 8 (selected
    state, A10) will require handler-side bool writes for any
    toggle construct — pre-shipping `set_bool` removes a Phase 8
    hard prerequisite at low marginal cost.
  - What you give up: One trait method and one new `evaluate()` arm
    beyond what a pure read-only path would need.
  - **Technical risk:** Low. Shape mirrors `set_i32` verbatim.

**Recommendation:** Option B. This recommendation is load-bearing
paired with DD-M3-P1-008 Option A — together they form the
"handler-side bool write" path that makes Phase 1's evidence
self-contained in `.ui`. The earlier draft of this DD recommended
Option A on a misread M2 precedent (treating "M2 has no `set_string`"
as a "read-first" principle, when in fact M2 simply had no
handler writing strings). The bool case is materially different:
Phase 1 explicitly needs live propagation, which means *something*
must call `Signal<bool>::set`; the cheapest, in-spec route is the
handler-side trait extension. If DD-M3-P1-008 lands as Option B
(separate state-write ABI), this DD flips back to its Option A.

---

### DD-M3-P1-005 — Phase 1 evidence: which widget attribute carries the `bool` binding?

**Status:** Drafting

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

### DD-M3-P1-006 — IR text grammar surface for `bool`

**Status:** Drafting

**Context:**
The IR text format (DD-M2-P6-002) is the on-disk form `wasamoc` emits
and `wasamo-runtime` parses. Its normative grammar lives in
[docs/dsl_spec.md §8 "Wasamo IR — Normative Specification (M2)"](../dsl_spec.md#8-wasamo-ir--normative-specification-m2).
Adding `bool` requires updates to the IR §§ on types, literals, and
handler expressions (per DD-M3-P1-003 Option A: `BoolLit` and
`BoolPropRead` productions).

This DD is mostly a sub-decision of DD-M3-P1-001..003 made explicit so
the IR text spec update is not treated as an afterthought.

**Options:**

Option A — Spell IR text bool literals as `true` / `false` and add
`BoolLit` / `BoolPropRead` productions verbatim (recommended)
- Matches DD-M3-P1-002 surface syntax; matches DD-M3-P1-003 expression
  shape.

  - **Technical risk:** Low.

Option B — Spell IR text bool literals as `#t` / `#f` or `0` / `1` for
brevity
- Diverges from the surface `.ui` syntax. No real gain.

**Recommendation:** Option A. The IR text and `.ui` surface should
agree on bool spelling; divergence would pay only in characters saved.

This DD is kept independent rather than folded into DD-M3-P1-001..003
because the IR text grammar is the **public spec surface** that A12
(DSL public draft) commits to (see
[m3-plan.md A12](../plans/m3-plan.md#acceptance-criteria) and
[docs/dsl_spec.md §8](../dsl_spec.md#8-wasamo-ir--normative-specification-m2)).
Even when an IR text change is mechanically derived from an in-memory
IR change, the spec-surface decision deserves an explicit DD so that
the per-phase spec update (A11) has a single citable record.

---

### DD-M3-P1-007 — Binding evaluation result shape for `bool`

**Status:** Drafting

**Context:**
M2 ended with `evaluate_binding()` returning `Result<String, EvalError>`
([wasamo-runtime/src/handler.rs L220-L223](../../wasamo-runtime/src/handler.rs#L220-L223)),
and `widget_write_property(id, prop: u32, value: &str)`
([wasamo-runtime/src/widget.rs L937](../../wasamo-runtime/src/widget.rs#L937))
building `PropertyValue::String(value.to_string())` unconditionally
before dispatching to the per-widget setter. The reactive seam
declares its writer as `write_fn: fn(WidgetId, PropertyKey, &str)`
([architecture.md L714](../architecture.md#L714)). The entire binding
write pipeline is string-baked.

DD-M3-P1-003 / DD-M3-P1-005 add `BoolPropRead` and `Button.enabled:
bool`, but do not by themselves answer how the bool value reaches the
widget's property setter without going through that string seam. This
DD resolves it.

**Options:**

Option A — Per-type binding evaluator + per-type writer (recommended)
- Keep `evaluate_binding() -> Result<String, EvalError>` unchanged
  for string-typed bindings.
- Add `evaluate_bool_binding(expr, ctx) -> Result<bool, EvalError>`
  in `handler.rs`. Accepts `BoolLit`, `BoolPropRead`, and rejects all
  other variants with `EvalError::TypeMismatch` (mirroring the way
  `evaluate()` rejects string-typed forms).
- Add `widget_write_property_bool(id, prop, value: bool)` in
  `widget.rs`, constructing `PropertyValue::Bool(bool)` and
  dispatching to a per-widget setter (Phase 1 ships exactly one:
  `Button.enabled`).
- Extend the `register_binding` write-seam:
  `write_fn` becomes per-type at the call site
  ([architecture.md L714](../architecture.md#L714)) — the loader
  picks the bool writer when the target property is bool-typed, the
  string writer otherwise. The reactive engine itself stays
  type-agnostic.

  - What you gain: F5 (`TypedValue` deferral) is preserved by
    construction — there is no single union type that all binding
    types funnel through; each scalar has its own evaluator + writer
    pair. The choice between string / bool happens at the loader
    against the target property's type, not at runtime against a
    value tag. Mirrors the M2 read trait's per-type method shape
    (`get_string` / `get_i32`).
  - What you give up: A second evaluator function and a second
    writer function; the loader's dispatch table grows by one row.
    All mechanical, all bounded.
  - **Technical risk:** Low. The pattern is already established for
    reads; this extends it to writes. No new abstraction.

Option B — Unify `evaluate_binding()` to return `PropertyValue`
- Change the return type to `Result<PropertyValue, EvalError>`. The
  writer becomes `widget_write_property(id, prop, value:
  PropertyValue)`, dispatching on the value tag.

  - What you gain: One evaluator, one writer signature.
  - What you give up: `PropertyValue` becomes the binding result
    type, which is one short step from being the runtime value
    union that F5 defers. Phase 6 and Phase 8 would naturally widen
    it (collection bindings, then handler-side bool writes), and at
    that point `PropertyValue` is `TypedValue` in all but name.
    Option A's per-type seam is the structural fence that keeps F5
    deferred; Option B removes it.
  - **Technical risk:** Low to implement; **high forward-compat
    exposure** to the `TypedValue` deferral.

Option C — Stringify bool as `"true"` / `"false"` through the existing
string pipeline; parse at the widget setter
- Reuse the M2 string-baked seam verbatim. `evaluate_bool_binding`
  produces `"true"` or `"false"`; the `Button.enabled` setter parses
  it.

  - What you give up: Hidden string ↔ bool coercion at the
    per-widget setter. Exactly the kind of context-sensitive type
    interpretation DD-M3-P1-001 Option B was rejected for. Fragile
    against future bool properties on other widgets — each setter
    re-implements the parse. Misclassifies as parse failure any
    legitimate string-typed property whose value happens to be
    "true". Easy footgun.

**Forward-compat exposure:**
Option A's exposure under foreseeable future events (Out of scope):
when `TypedValue` is reconsidered after M3, the per-type seams
collapse into the union naturally — but they also remain perfectly
serviceable if F5 stays in force. The shape is dual-survivable.
Option B's exposure: it has already partially built the union; once
M4+ adds more types, the union *is* `TypedValue` with no remaining
fence. The exposure is asymmetric — Option A survives F5 staying or
reversing; Option B implicitly commits to reversing F5.

**Recommendation:** Option A. The owner's pre-doc-review note
([m3-target-app-predoc — Tabs / Button selected-state surface closure (Reservation 3)](../notes/m3/m3-target-app-predoc.md#保留-3-closure-tabs--button-選択状態-surface--採用-bool-を-3-つ目の-scalar-として導入))
explicitly maintained F5 deferral as the condition for admitting
`bool`; the per-type seam is what makes that condition mechanically
enforceable.

---

### DD-M3-P1-008 — Mutation source for the Phase 1 live-propagation evidence

**Status:** Drafting

**Context:**
DD-M3-P1-005 picks `Button.enabled: bool` as the property carrying
the evidence. The remaining question: *what changes the bound
`state ready: bool` value at runtime* so that live propagation
(not just initial value) can be observed?

The previous draft of this DD assumed "host-side via existing C ABI
write path" using `wasamo_set_property`. That assumption is false:
[`wasamo_set_property` (abi.rs L711)](../../wasamo-runtime/src/abi.rs#L711)
writes a **widget property** by `(widget*, property_id)`. State
signals live in
[`SignalRegistry` (reactive.rs L389)](../../wasamo-runtime/src/reactive.rs#L389)
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
  `v_bool` in [abi.rs L74-90](../../wasamo-runtime/src/abi.rs#L74-L90))
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

### DD-M3-P1-009 — Property type metadata and writer dispatch

**Status:** Drafting

**Context:**
DD-M3-P1-007 chooses per-type binding writers
(`widget_write_property_bool` alongside the existing string-typed
writer). For the loader to pick the right writer when a binding
target is `Button.enabled: bool`, the `(widget_type, prop_name) →
PropertyKey` lookup needs to also carry the property's type.

Today,
[`resolve_prop_key` (ir_loader.rs L797)](../../wasamo-runtime/src/ir_loader.rs#L797)
returns `Option<PropertyKey>` (= `Option<u32>`); the property's type
is implicit in the per-widget setter's `match` on the `PROP_*` id
([widget.rs L375 onwards](../../wasamo-runtime/src/widget.rs#L375)).
That works for M2 (i32 and String dispatched by the setter), but the
*binding loader* doesn't see the type — it just hands the string-baked
writer to `register_binding`. To select a typed writer at binding
registration time, the type needs to be exposed at the lookup
boundary.

**Options:**

Option A — Widen `resolve_prop_key` to return
`Option<(PropertyKey, IrType)>` (recommended)
- The widget catalog grows from `(widget, prop) -> u32` to
  `(widget, prop) -> (u32, IrType)`. The binding loader matches on
  the returned `IrType` to pick `widget_write_property` (string),
  `widget_write_property_i32` (if/when added), or
  `widget_write_property_bool`.

  - What you gain: Single source of truth for property type lives
    in the widget catalog. The mapping is co-located with the
    setter-side `match` it has to agree with — they can be reviewed
    together. Adding a new bool property to a future widget is one
    new row, type included.
  - What you give up: One enum field in the catalog table. Touches
    every existing row (M2: 4 rows — `Text.text` String, `Text.font`
    String, `Button.text` String, `Button.style` I32) with their
    explicit `IrType`.
  - **Technical risk:** Low. Pure refactor of an internal lookup
    function; no public API change.

Option B — Add a parallel `prop_type_for(prop_key) -> IrType` lookup
- Keep `resolve_prop_key` as-is; introduce a second lookup keyed by
  `PropertyKey`.

  - What you gain: `resolve_prop_key`'s signature stays compatible.
  - What you give up: Two lookup tables to keep in sync (M2 already
    has one source of truth for the `PROP_*` id; the type would now
    live in a second). Drift risk. Two callers (the binding
    registration site and the setter) must agree about which one is
    authoritative.

Option C — Encode type in `PROP_*` u32 bit-layout (e.g. high byte =
type tag)
- Magic encoding: `PROP_BUTTON_ENABLED = (BOOL_TAG << 24) | 0x03`.

  - What you give up: Opaque encoding for a problem better solved
    by a struct field. The ABI exposes `property_id: u32`
    ([abi_spec §3.3 + abi.rs L711](../../wasamo-runtime/src/abi.rs#L711));
    leaking type bits into ABI identifiers is a long-term
    maintenance liability.

**Recommendation:** Option A. The widget catalog is already the
right place for property metadata; extending the row by one field
is the lowest-overhead route and is invisible across the ABI
boundary (the `PROP_*` u32 values stay unchanged).

This DD is what makes DD-M3-P1-007's per-type seam operational. The
binding loader queries `(widget, prop) → (key, ty)`, dispatches to
the bool writer when `ty == IrType::Bool`, and the reactive engine's
`write_fn` parameter becomes typed at the call site rather than
globally string-baked.

**Out-of-Phase-1 question (recorded, not decided here):** the
inverse direction — `wasamoc` validating that a binding's *expression
type* matches the *target property's type* — is DD-M3-P1-010's
territory.

---

### DD-M3-P1-010 — `wasamoc` type-checker scope for `bool`

**Status:** Drafting

**Context:**
Phase 1 introduces a new scalar type. The `wasamoc` checker
([wasamoc/src/check.rs](../../wasamoc/src/check.rs))
has to decide which type combinations to accept and reject. The
question is well-bounded: which of the following do we want to be
*compile-time errors* (caught by `wasamoc check`), and which are
*runtime errors* (caught by `EvalError::TypeMismatch` at IR load /
evaluation)?

Examples, paired with the desired classification:

| Pattern | Static? | Reason |
|---|---|---|
| `state ready: bool = false` | ✓ valid | declaration matches default literal type |
| `state ready: bool = 0` | error | i32 literal in bool default |
| `state ready: bool = "false"` | error | string literal in bool default |
| `bind enabled: ready` (both `bool`) | ✓ valid | source/target types agree |
| `bind enabled: 1` (target `bool`) | error | i32 RHS into bool target |
| `bind enabled: true` (target `bool`) | ✓ valid | bool literal into bool target |
| `bind label: true` (target `String`) | error | bool RHS into string target |
| `bind label: ready` (target `String`, source `bool`) | error | bool source into string target |
| `state x: i32 = 5; bind enabled: x` | error | i32 source into bool target |
| identifier `ready` resolves to which `*PropRead`? | depends | resolve via the declared `state` type table |

The shape of "static vs runtime" matters because it affects how much
the M3 `wasamoc` checker grows in Phase 1 (and how much Phase 6
inherits as foundation).

**Options:**

Option A — Phase 1 checker rejects all of the above at
`wasamoc check`, using the state-name → declared-type table built
from parsed `state` decls (recommended)
- `wasamoc` builds a `HashMap<String, IrType>` from `state`
  declarations at parse time. The identifier-lowering pass resolves
  `ready` to `BoolPropRead` (if the table says `bool`), `PropRead`
  (if i32), or `StrPropRead` (if String). Binding LHS types come
  from the widget-property catalog (DD-M3-P1-009). Mismatch is a
  diagnostic with line/column.

  - What you gain: Mismatch errors land where they belong — at
    compile time, on the line they originate from, before any IR
    loader / runtime evaluator ever sees them. The same table
    Phase 6 needs to type-check conditional rendering's `if <expr>`
    becomes available now. Identifier resolution becomes
    deterministic at lowering time rather than at runtime
    evaluator dispatch.
  - What you give up: A typed lowering pass in `wasamoc` —
    bounded, but not zero. The current checker does not thread
    parent-widget context or target-property type down through
    member-checking; making `bind` LHS type-aware requires a
    signature change in `check_members` (or its equivalent) so the
    binding pass knows the widget catalog entry for the enclosing
    widget. This is a real (small) structural change to the
    checker, not just an additional rule.
  - **Technical risk:** Low–medium. `wasamoc` already has a checker
    framework (DD-M2-P6-001 onwards), so the rule additions are
    straightforward; the medium part is the `check_members`
    signature widening to carry widget context, which touches every
    existing member-check call site. Not large, not zero.

Option B — Phase 1 checker validates `state` default-vs-declared-type
only; defer binding type-checking to the IR loader / runtime
- `state ready: bool = 0` is rejected by `wasamoc`. But
  `bind enabled: 1` slips through `wasamoc` and dies at IR-load /
  evaluator with `EvalError::TypeMismatch`.

  - What you give up: Tooling story (M5 LSP) inherits a partly-typed
    `wasamoc` — type errors are not consistently surfaced. Phase 6
    will need the binding type-checker anyway (it needs to know
    that `if <expr>` is bool-typed); deferring forces Phase 6 to
    write the checker that Phase 1 could have written first.

Option C — No static type checking for bool in Phase 1; everything
becomes a runtime error
- All the rejections in the table above land at runtime as
  `EvalError::TypeMismatch`.

  - What you give up: Worst error ergonomics; same Phase 6 debt as
    Option B but larger.

**Recommendation:** Option A. The cost of building the state-type
table in `wasamoc` is the same as Phase 6 would pay anyway for
conditional rendering's expression typing; doing it now keeps
diagnostics consistent and removes a Phase 6 prerequisite. M3 spec
public draft (A12) is also better served by a checker that rejects
malformed type combinations at the source line.

Identifier resolution is the load-bearing detail: in Option A,
`ready` becomes `BoolPropRead` at *lowering* time by consulting the
state-type table, not at runtime by trial-and-error on `EvalContext`
methods. This locks in DD-M3-P1-003's expression shape (one
identifier → one typed PropRead variant).

---

## Out of scope (for M3-Phase 1; recorded explicitly)

- **Comparison and logical operators on `bool`** (`==`, `!=`, `&&`,
  `||`, `!`). Phase 1 establishes literal and property-read bool
  propagation only. Conditional rendering (M3-Phase 6) is where bool
  expressions get exercised; if it needs operators, the Phase 6 ADR
  introduces them with the surface they support.
- **`i32` ↔ `bool` coercion / truthy semantics.** Type-tagged
  representation means a bool-context expression must already be
  bool-typed; an i32 value is not implicitly `0 → false`. If a use
  case demands coercion, the Phase that needs it opens a DD.
- **`CompoundAssign` over bool** (i.e. `ready += true`, `ready *= x`).
  DD-M3-P1-004 Option B and DD-M3-P1-008 Option A admit *only* the
  `Assign { rhs: BoolLit | BoolPropRead }` shape in `evaluate()`.
  Compound assignment for bool has no agreed semantics (no
  `CompoundOp` is naturally bool-typed) and is not introduced by
  Phase 1.
- **State-write C ABI primitive** (e.g. `wasamo_set_state(name,
  WasamoValue)`). DD-M3-P1-008 Option B; deferred to its own ADR
  pending demand from a phase whose evidence shape requires host-
  side state mutation (Phase 1's evidence is admitted via handler-
  side `set_bool` per DD-M3-P1-008 Option A, sidestepping this
  question for Phase 1).
- **Full `Button.enabled` interaction-state contract.** Phase 1
  narrows to: click suppression, layout slot preserved, minimal
  visual. Deferred to M4 (input/focus) / M5 (a11y): keyboard
  focusability and tab-order when disabled, AccessKit /
  `aria-disabled` semantics, hover and focus visual states, key
  activation suppression. See DD-M3-P1-005's "Out of scope (Phase 1)"
  sub-list.
- **`PropertyValue` / binding writer becoming a generic value union.**
  DD-M3-P1-007 keeps per-type evaluators and per-type writers so the
  binding pipeline never funnels through one runtime value tag. The
  decision is structural F5 enforcement, not just current scope.
- **`Button.selected` and any other widget attribute beyond
  `Button.enabled`.** A10 lives in M3-Phase 8 ADR; this phase opens
  one attribute, no more.
- **Visibility / conditional rendering / subtree presence.** A7 lives
  in M3-Phase 6 ADR; DD-M3-P1-005 rejects Option B to keep this
  decoupled.
- **`TypedValue` generic value union.** F5 deferral is preserved
  ([m3-start-framing.md §F5](../notes/m3/m3-start-framing.md#l335)).
  Adding `bool` as a third tagged scalar is not the same decision as
  introducing a typed value union.
- **Per-symbol IR text grammar rename** (`PropRead` →
  `IntPropRead`). DD-M3-P1-003 Option C; deferred to a future phase
  that has substantive IR grammar reason to touch the names.

## Owner-agreement checkpoints

Two of the DDs above are load-bearing value judgements that warrant
explicit yes/no from the owner before this ADR moves to Accepted.
All other DDs follow mechanically from these two.

### Checkpoint 1 — DD-M3-P1-004 ⇄ DD-M3-P1-008 pair flip

**Question:** Does Phase 1 admit handler-side bool writes
(`set_bool` on `EvalContext` + bool-typed `Assign` arm in
`evaluate()`)?

**Default answer:** Yes (DD-004 Option B + DD-008 Option A —
"handler-side bool write" path).

**Framing for owner:** The pair flip is not scope creep — it is the
**minimum** addition that lets the Phase 1 live-propagation evidence
ride the `.ui` surface. The alternatives are concretely worse:

- Read-only `EvalContext` + new `wasamo_set_state` ABI primitive
  (DD-008 Option B): introduces a permanent public ABI surface whose
  design (component identity, observer firing, thread-affinity)
  isn't closed; that belongs in its own ADR, not piggy-backed on a
  scalar introduction.
- Read-only `EvalContext` + test-only state mutation hook
  (DD-008 Option C): cheapest delta, but Phase 1's evidence is no
  longer connected to the public surface that A12 (DSL public draft)
  is supposed to record.
- Read-only `EvalContext` + initial-value only (DD-008 Option D):
  the entire reactive pipeline for bool goes unexercised — m3-plan
  Phase 1 explicitly calls for "live `WidgetNode` propagation."

The cost of saying yes is **one trait method** (`set_bool`,
mirroring the existing `set_i32`) and **one new `evaluate()` arm**
(bool-typed `Assign`). The cost of saying no is one of the three
alternatives above. In that sense the flip keeps Phase 1 small,
not large.

### Checkpoint 2 — DD-M3-P1-005 Option A vs Option E

**Question:** Does Phase 1 surface `Button.enabled: bool` as a
public DSL attribute (Option A), or ship the bool plumbing without
committing the public widget spec to any specific bool attribute
(Option E)?

**Default answer:** Option A (public `Button.enabled`, intentionally
narrow contract).

**Phase 1 `Button.enabled` contract if Option A is taken:** The
public DSL spec entry guarantees only:

- a bool-typed `enabled` attribute on `Button`;
- default `true`;
- when `false`, the layout slot is preserved (no `display: none`
  behaviour);
- when `false`, click-handler dispatch is suppressed;
- a minimal disabled visual (greyed colours, no animation).

The contract **explicitly defers** to M4 (input/focus) and M5
(accessibility): keyboard focusability and tab-order when disabled,
AccessKit / `aria-disabled` semantics, hover and focus visual
variations, key activation suppression.

**Trade-off framing:** Option A puts the live-propagation proof on
a public widget attribute, which keeps the evidence aligned with
A11/A12 (per-phase spec sync; DSL public draft). The risk is that
"disabled control" semantics can creep — the narrow contract above
is the fence.

Option E falls back to a runtime-internal bool probe property that
exists in the widget catalog but is not in `docs/dsl_spec.md`. It
sidesteps the disabled-control framing entirely, at the cost of
A9's public-spec evidence shrinking to scalar / literal /
type-checking — there is no bool-bound widget attribute in the
M3 public DSL draft from Phase 1's work in that scenario.

The default favours Option A on the grounds that A12 (DSL public
draft) is better served by evidence that reaches a user-visible
attribute, but the disabled-control objection is real; if click
suppression alone is judged too heavy for Phase 1, Option E is the
clean fallback.

---

## Summary of decisions

| ID | Topic | Recommendation |
|---|---|---|
| DD-M3-P1-001 | `IrType` extension | Option A — add `IrType::Bool` |
| DD-M3-P1-002 | `IrLiteral` + surface syntax | Option A / Option A — `IrLiteral::Bool(bool)` and `true` / `false` keywords |
| DD-M3-P1-003 | `HandlerExpr` variants for bool | Option A — add `BoolLit` + `BoolPropRead`, no rename of existing variants |
| DD-M3-P1-004 | `EvalContext` method shape | Option B — full `get_bool` + `read_bool_tracked` + `set_bool` (paired with DD-M3-P1-008 Option A) |
| DD-M3-P1-005 | Phase 1 evidence widget attribute | Option A — `Button.enabled: bool` with narrowed Phase 1 contract (click suppression + minimal visual; focus / a11y deferred to M4–M5) |
| DD-M3-P1-006 | IR text grammar surface | Option A — `true` / `false` spelling, parallel `Bool*` productions |
| DD-M3-P1-007 | Binding eval result shape + writer signature | Option A — per-type binding evaluator (`evaluate_bool_binding`) + per-type writer (`widget_write_property_bool`); `PropertyValue::Bool(bool)` added but not unified into a value union |
| DD-M3-P1-008 | Mutation source for Phase 1 evidence | Option A — admit handler-side bool writes in Phase 1 (`set_bool` + `evaluate()` bool-typed `Assign` arm); flips DD-M3-P1-004 to its Option B |
| DD-M3-P1-009 | Property type metadata + writer dispatch | Option A — `resolve_prop_key` returns `(PropertyKey, IrType)`; widget catalog row carries the type |
| DD-M3-P1-010 | `wasamoc` type-checker scope for bool | Option A — full state/binding/identifier type-checking at `wasamoc check`; mismatches are compile-time diagnostics |

Implementation task list: belongs in
[`docs/plans/m3-plan.md` — M3-Phase 1 Progress](../plans/m3-plan.md)
once this ADR is accepted; not in this ADR per
[decisions/README.md §Task lists](./README.md#task-lists).

## Spec impact preview (for owner agreement)

When this ADR is accepted, the following docs change in the same Phase
1 commit set (per A11 same-phase synchronisation):

- [docs/dsl_spec.md](../dsl_spec.md) — extensions in two regions:
  - **DSL surface** (§§ 2–4): `true` / `false` keyword reservation in
    §2.1; bool literal token in §2; `bool` type in §4.2 (`in-out
    property`) / state declarations; bool in §4.3 (property binding)
    and §4.6 (expressions: `BoolLit`, `BoolPropRead`, and `Assign`
    with bool-typed RHS; `CompoundAssign` over bool excluded).
  - **IR text grammar** (§8): `IrType` production adds `bool`;
    literals add `BoolLit`; handler expressions add `BoolPropRead`.
  - `Button.enabled` attribute documented in the widget catalog
    section (minimal disabled styling permitted in M3; no animation
    contract).
- [docs/architecture.md](../architecture.md) — §6 SignalRegistry
  snippet around [L717-L744](../architecture.md#L717-L744) updated:
  add `bools: HashMap<String, Signal<bool>>` alongside `i32s` and
  `strings`; the surrounding prose extends "M2 supports `i32` and
  `String` Signals" to include `bool` and notes that
  `HandlerExpr::BoolPropRead` evaluates through
  `BindingEvalContext::read_bool_tracked`. F5 deferral cross-reference
  is preserved. The binding write-seam description around
  [L714](../architecture.md#L714) is also updated to reflect
  DD-M3-P1-007: `write_fn` is per-type at the call site rather than a
  single string-baked function pointer.
- [docs/abi_spec.md](../abi_spec.md) — **no new ABI surface added.**
  `WASAMO_VALUE_BOOL = 3` and `v_bool` already exist
  ([abi_spec.md §3.3](../abi_spec.md), [abi.rs L74-L90](../../wasamo-runtime/src/abi.rs#L74-L90))
  from M2; Phase 1 only connects this existing tag through the
  property-write path that previously dropped it. Specifically,
  `read_property_value` / `write_property_value` and
  `property_value_to_owned` ([abi.rs L745-L749](../../wasamo-runtime/src/abi.rs#L745-L749))
  gain bool arms; `PropertyValue` ([widget.rs L77-L80](../../wasamo-runtime/src/widget.rs#L77-L80))
  gains `Bool(bool)`; the property observer payload conversion
  carries bool through to `WasamoValue::v_bool`. Existing ABI
  function signatures and value-tag numeric assignments are
  untouched (M6 freeze scope unchanged; this phase is pre-freeze).
- [wasamoc/src/check.rs](../../wasamoc/src/check.rs) and adjacent
  lowering — per DD-M3-P1-010: state-name → declared-type table
  built at parse, used to lower identifiers to typed `*PropRead`
  variants and to type-check `bind` LHS / RHS pairings.

No ROADMAP revision is anticipated — A9 is already explicit, this ADR
operationalises it.

## Phase 1 verification closure (what counts as A9 evidence)

This section is not a DD — it records the agreed shape of the proof
that closes Phase 1, so the implementation plan in
[m3-plan.md Progress](../plans/m3-plan.md) inherits a concrete target
rather than re-litigating "what does live propagation mean here?".

A9 (`bool` admitted as third scalar) is considered satisfied when
**all four** of the following are observed, in this order:

1. **Unit-test evidence (host-independent).** Pure-logic tests in
   `wasamoc` (parse + check + lower) and in `wasamo-runtime`
   non-Windows-bound modules (handler evaluator, binding evaluator,
   `SignalRegistry`) cover: bool literal parsing; type-checker
   accept/reject pairs from DD-M3-P1-010's table; `evaluate()`
   bool `Assign` arm; `evaluate_bool_binding` accept set;
   `Signal<bool>::set` triggering effect re-run. These run on any
   CI runner.

2. **IR text round-trip evidence.** `wasamoc` emits, `wasamo-runtime`
   loads, and an in-process test reads back: `state ready: bool =
   false` as `IrState { ty: Bool, default: Bool(false) }`;
   `bind enabled: ready` as `HandlerExpr::BoolPropRead`. Tests the
   DD-M3-P1-001/002/003/006 surfaces together.

3. **Windows-runtime live-propagation evidence (CI-gated).** A
   mock-free integration test (per CLAUDE.md "Testing rules") on the
   Windows CI runner: an `.ui` fixture declares
   `state ready: bool = true; Button { enabled: ready; on click {
   ready = false } }`. The test loads the IR, invokes the button's
   click signal, observes `Signal<bool>::get_untracked()` flips to
   `false`, and observes that the `Button`'s widget-side
   `PROP_BUTTON_ENABLED` reflects the new value. Fails (not skips)
   on a runner that cannot create the Compositor — the test gates
   A9 evidence in CI, not local convenience.

4. **End-to-end host evidence (one host suffices).** One of
   `examples/counter-rust`, `examples/counter-c`, or
   `examples/counter-zig` is extended (or a new minimal `bool-demo`
   example is added under `examples/`) to drive the Phase 1 fixture
   through to a visible window. The choice of host is recorded in
   the m3-plan Phase 1 Progress section once implementation starts;
   the Rust host is the working default because its build path
   doesn't require the `wasamoc.exe` ordering dance (CLAUDE.md
   "Build ordering requirements"). C and Zig hosts are not required
   for Phase 1 — the C-ABI bool plumbing is exercised by the
   `PropertyValue::Bool` ↔ `WasamoValue::v_bool` conversion arms
   reached via the property-observer payload, which the
   Windows-runtime test in (3) already covers.

Items (1)–(3) are required; item (4) is the visible proof that ties
Phase 1's evidence back to the m3-plan target-app trajectory. Items
(1) and (3) together close DD-M3-P1-007's per-type writer seam (the
unit test covers the dispatch logic; the integration test covers the
end-to-end live propagation). DD-M3-P1-008's choice of handler-side
mutation is what makes items (3) and (4) achievable without new ABI
surface.

The acceptance/non-acceptance of test items (1)–(4) is the
operational form of "Phase 1 done"; the corresponding implementation
checklist (which crate / which test file / which `.ui` fixture)
belongs in m3-plan Progress, not here.
