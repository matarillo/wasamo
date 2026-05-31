# M3-Phase 6 — ZStack + conditional rendering: Architecture Decisions

**Phase:** M3-Phase 6 (ZStack layout primitive + conditional rendering grammar)
**Date:** 2026-05-31
**Status:** Proposed

## Context

M3 acceptance criteria **A4** and **A7** (see
[../../../_roadmap.md M3](../../../_roadmap.md#m3-dsl-surface),
[../../plan.md §Acceptance criteria](../../plan.md#acceptance-criteria)):

> **A4** — ZStack layout primitive: sibling z-order by document order.
>
> **A7** — conditional rendering grammar: binding drives the present /
> absent state of a subtree.

Phase 6 ships these **two surfaces as one unit** (framing decision
FD-F): the gallery lightbox uses ZStack for overlay layering and
conditional rendering for open/close, so a single visible slice
exercises both. The two AC are load-bearing in different ways:

- **A4 (ZStack)** is the overlay-dedicated layout primitive whose
  boundary Phase 5 drew explicitly — "same-cell overlap is **not**
  provided by Grid; overlay is ZStack's responsibility"
  ([../../phase-5/decisions/preamble.md](../../phase-5/decisions/preamble.md)).
  The two load-bearing parts are **(i)** sibling overlap (each child
  occupies the same overlap region) and **(ii)** document order =
  paint order = z-order.
- **A7 (conditional rendering)** is the **first grammar surface of M3
  where a `binding` drives widget-tree *structure* (the present /
  absent state of a subtree) rather than a property value**. Every
  M2 / M3-Phase 1..5 binding drove a property value; A7 is the step
  from property-driven to structure-driven. The `bool` scalar this
  depends on landed in M3-Phase 1 (DD-M3-P1-002); the hard
  prerequisite is satisfied.

Three further milestone obligations apply to Phase 6:

- **A11 (operational obligation).** `.ui`, `wasamo-ir`, `wasamoc`,
  `wasamo-runtime`, `docs/dsl_spec.md`, and the `examples/gallery/`
  sub-screen all advance within Phase 6. The visible gallery proof is
  the **lightbox** (framing decision FD-B): a `bool`-toggled overlay
  shown over the thumbnail gallery slice.
- **A12 (DSL specification public-draft obligation).** Phase 6 adds a
  ZStack chapter and a **conditional-rendering grammar chapter** to
  [`docs/dsl_spec.md`](../../../../docs/dsl_spec.md) at Moment 1, held
  to the external-reader bar at close. The conditional-rendering
  chapter is **not** a single-construct write-up: per the framing
  design thesis it is written as the **first chapter of Wasamo's
  structural rendering model**, such that an external reader sees
  `if` as the first member of a structural control-flow grammar family
  whose later members (`else` / `switch` / `for`) arrive in the same
  family.
- **R1 (carry-forward).** Gallery host **Window-title wiring** — Phase
  4 residual, assigned to Phase 6 as owning phase (Phase 5 FD-E,
  [../requirements/constraints.md §1](../requirements/constraints.md)).
  Static `title:` reaching the native window is a **required**
  Phase 6 completion condition (DD-M3-P6-006).

The pre-doc framing for this phase was aligned with the owner on
2026-05-31 and is recorded in
[../requirements/framing.md](../requirements/framing.md) ("Owner
alignment outcome" section). That alignment settled the owner-facing
framing decisions (FD-CR / FD-B / FD-G / FD-D / FD-E / FD-F); the
remaining sub-decisions are recorded below as ADR `Recommendation`
directions and approved at the `Status: Proposed` → `Accepted` review
pass.

### The structural-rendering thesis (FD-CR) as the cross-cutting lens

The framing's most load-bearing input is **FD-CR** (the owner's
conditional-rendering philosophy, originating in
[../../../../docs/notes/dsl-grammar.md Q6](../../../../docs/notes/dsl-grammar.md)).
It fixes Wasamo's stance across DD-003 / DD-004 / DD-005 / A12:

1. **v1 surface = approach 2** (template + dedicated structural
   syntax/attribute). Not approach 1 (property toggling of an
   always-built tree) and not approach 3 (host-language `if`/`switch`
   embedding).
2. **approach 3 must not be foreclosed** — the IR / runtime must be
   able to grow toward language-construct-level freedom later.
3. The construct is the **first member of a structural control-flow
   grammar family** (`if` now; `else` / `switch` / `for` later in the
   same family), not a one-off feature.
4. **Effect / binding lifetimes inside a conditional subtree must not
   be left ambiguous** (DD-005).

FD-CR also carries a **runtime-identity** dimension (Q6 "implications
for runtime design"): separating a lightweight **declared tree** (may
be regenerated on state change) from a lifetime-bearing **entity tree**
(state / effect / layout object / focus / in-progress input handled by
identity), with Flutter's Widget / Element / RenderObject split as the
reference point. This separation is an **evaluation axis for DD-004**
— not an owner-facing choice, but a design-judgment criterion. Phase
6's **implementation** is the minimal `if <bool>` (no `else` / `switch`
/ `for`), but its **design** is evaluated against family-extensibility
and the declared-tree / entity-tree separation. We widen the design
area without widening the implementation.

### End-state shape this phase extends without breaking

The M2 / M3-Phase-1..5 shapes Phase 6 builds on:

- **`wasamo-ir`**: `IrType` is `I32 | Str | Bool`; `IrLiteral` is `Int
  | Str | Ident | Bool | Ratio | Color`. Phase 6 introduces **no new
  `IrType` and no new `IrLiteral` variant**. ZStack is a pure layout
  container that needs **no `KindPayload`** (unlike Grid's track
  lists; DD-M3-P6-001). The conditional construct is encoded by
  **making control flow a first-class member-level IR construct**
  (DD-M3-P6-004 recommends `IrNode.children: Vec<IrMember>` with
  `IrMember = Widget | ControlFlow`, a genuine IR-schema change carrying
  a branch list so `else` / `switch` / `for` are same-family variants;
  O2 is the lighter branch-list fallback). The control-flow member is
  IR-only — it materialises no runtime widget, the same *interpret-not-
  render* posture as Grid's `Cell`. Its condition rides the existing
  `IrBinding` / `HandlerExpr` machinery; `IrProp.value` stays strictly
  `IrLiteral`. **This member-level IR change is the consequential
  owner-decision fork of the phase** (see DD-M3-P6-004).
- **`wasamo-runtime` widget catalog**: `Rectangle | VStack | HStack |
  Text | Button | Box | WrapPanel | ScrollView | Grid`. Phase 6 adds
  **`ZStack`** as a per-kind tag (a pure overlap layout container,
  DD-M3-P6-001). The control-flow member is **not** a runtime widget
  kind — like `Cell` it materialises no `WidgetNode` and no `Visual`;
  the runtime consumes it to build a conditional binding (DD-M3-P6-004).
- **Layout engine**: pure-data `LayoutNode` / `measure` / `arrange`
  boundary. ZStack defaults to **`Fill/Fill`** (overlay-first; like
  Grid / ScrollView), and its desired size on a **Shrink/unbounded**
  axis is the **union (per-axis max) of its children**; each child is
  arranged within the ZStack content rect by per-child alignment
  (DD-M3-P6-002). A `Fill` child contributes `0.0` to the union and
  fills its rect in *arrange* — the lightbox's full-viewport scrim
  comes from the ZStack's own `Fill` default, not from a child driving
  the union. ZStack introduces **no new `LayoutError`** — children's
  own Fill / Shrink rules and the existing unbounded-axis conventions
  apply.
- **Reactive engine**: Signal / Effect two-layer primitive,
  per-widget Effect ownership with structural disposal
  ([../../../../docs/architecture.md §6.8.6](../../../../docs/architecture.md#686-effect-lifetime-dd-m2-p5-003--a)),
  `BATCH_DEPTH` / `MUTATION_CAP` drain
  ([§6.8.3](../../../../docs/architecture.md#683-drain-ordering-inside-drain_if_outermost)),
  and the `BindingTarget` enum whose `// M3+ adds ConditionalSubtree,
  ForLoopSubtree, …` slot
  ([§6.8.7](../../../../docs/architecture.md#687-binding-registration-api-after-m2-dd-m2-p5-005-dd-m2-p6-007-dd-m2-p6-011-dd-m3-p1-007))
  Phase 6 now fills (DD-M3-P6-004). The M3-Phase 1 **synchronous
  non-batched drain proof contract** (item 4 of
  [../../../milestone-2/handoff.md §3](../../../milestone-2/handoff.md))
  is preserved, not revised (DD-M3-P6-005).
- **Expression grammar**: the `.ui` `expr` rule admits `STRING_LIT |
  number_with_unit | BOOL_LIT | RATIO_LIT | COLOR_LIT | IDENT` — no
  operators. Phase 6 conditional conditions admit the **same narrow
  bool-expr already accepted by `Button.enabled`** (a `bool` state
  identifier or a `BOOL_LIT`); `!ready` / comparison / logical
  operators stay deferred to a future expression-grammar extension
  ([Q5](../../../../docs/notes/dsl-grammar.md), DD-M3-P6-003).
- **`wasamoc`**: state-name → declared-type table; identifier
  resolution to typed `*PropRead` variants. Phase 6 adds no new value
  type; `wasamoc check` extends to ZStack and to the `if` condition
  (bool-typed, resolvable) and structural placement (DD-M3-P6-003).
- **Window title host path**: `wasamo_load_ui` builds the widget tree
  then calls `window::create(DEFAULT_WINDOW_TITLE, …)`
  ([abi.rs:1220](../../../../wasamo-runtime/src/abi.rs)); the
  component-level `title:` is lowered to root props (per
  [Q2](../../../../docs/notes/dsl-grammar.md)) but dropped by the
  loader. DD-M3-P6-006 routes the **static** title to `window::create`.

This ADR does **not** re-open F5 (`TypedValue` deferral) — no new
scalar type. The lightbox photo is a Box(aspect 4:3) + Text
placeholder (Image widget deferred to M4 per Phase 2 DD-M3-P2-006).

### Acceptance lens for this phase

- **A4** is satisfied when `.ui` declares `ZStack { <child> <child> …
  }`, the shared crates lower → load → render it with correct overlap
  measure/arrange, document-order z-order (later child painted on
  top), and the ZStack outer-bounds clip (DD-M3-P6-001 / DD-M3-P6-002).
- **A7** is satisfied when `.ui` declares `if <bool-expr> { <child>… }`,
  the shared crates lower → load → render it, and toggling the bound
  `bool` **inserts / removes the subtree structurally** (not merely
  hiding a built subtree), with the effect-lifecycle policy and drain
  contract of DD-M3-P6-004 / DD-M3-P6-005 holding.
- **A11** is satisfied when the lightbox slice advances every side and
  grows `examples/gallery/gallery.ui` additively (FD-B).
- **A12** is satisfied when the ZStack chapter and the conditional-
  rendering (structural-rendering-model) chapter land in
  `docs/dsl_spec.md` at the external-reader bar by phase close.
- **R1** is satisfied when the static component-level `title:` reaches
  the native window title bar (DD-M3-P6-006).

## Decisions

The Phase 6 ADR carries six DDs (framing DD slate → ADR numbering 1:1):

| DD | Title | Recommendation summary |
|---|---|---|
| [DD-M3-P6-001](./dd-m3-p6-001-zstack-ir-node-form-and-surface.md) | ZStack IR node form and author surface | Per-kind tag `ZStack`; **direct children** (no `Cell`-style wrapper); **no `KindPayload`, no new `IrType`/`IrLiteral`**; author surface `ZStack { <child>… }`, document order = bottom-to-top z-order; runtime **default size constraint `Fill/Fill`** (overlay-first; no Phase-6 override surface — DD-M3-P6-002) |
| [DD-M3-P6-002](./dd-m3-p6-002-zstack-measure-arrange-zorder-clip.md) | ZStack measure / arrange + z-order + clip | **ZStack default constraint `Fill/Fill` (overlay-first)** — a `Fill` child contributes `0.0` to sizing, so the full-viewport scrim comes from the ZStack's own Fill default, not from the child; **union (per-axis max) sizing** is the Shrink/unbounded-axis desired-size policy. Owner-visible trade-off: **intrinsic (bounded) ZStack is not author-expressible until a future size-constraint surface**. Each child arranged in the content rect by per-child alignment (**default `center`**, `h-align`/`v-align` overrides); **document-order z-order** (no explicit `z-index`); **ZStack outer-bounds clip** on, per-child clip out; no new `LayoutError` |
| [DD-M3-P6-003](./dd-m3-p6-003-conditional-rendering-grammar-surface.md) | Conditional rendering author-facing grammar surface | **Approach 2**; **`if <bool-expr> { <member>… }` block form** (family-extensible to `else`/`switch`/`for`, unlike a `when:` attribute); condition = **E1**, the narrow bool-expr already accepted by `Button.enabled`. Intermediates **E1.5 (`!`-only)** / **E1.75 (bool-only `&&`/`||`/`!`)** weighed and declined **for grammar uniformity** (operators should grow once across all `expr` positions per Q5, not in a condition-only pocket) — not for effort; non-bool / mis-placed `if` rejected at `wasamoc check` |
| [DD-M3-P6-004](./dd-m3-p6-004-conditional-ir-and-runtime-present-absent.md) | Conditional IR representation + runtime present/absent | **Member-level structural IR (O1, recommended; O2 lighter fallback)** — `children: Vec<IrMember = Widget \| ControlFlow>` with a branch list so `else`/`switch`/`for` are same-family variants; IR-only (no runtime widget); Phase 6 ships the single-branch `ControlFlowNode::If`. Runtime fills **`BindingTarget::ConditionalSubtree`**; present/absent via `insert_child`/`remove_child`. **Phase 6 = full destroy+rebuild (ID-1)**; absent=fresh-on-return is **normative author-visible semantics**, future retention is **opt-in (keyed)** so the default never breaks. **Consequential owner-decision fork** (IR-schema change) |
| [DD-M3-P6-005](./dd-m3-p6-005-conditional-effect-lifecycle-and-drain-contract.md) | Conditional effect lifecycle + reactive-drain proof contract | **(a)** absent subtree's Effects are **disposed** via the existing structural teardown; re-present **recreates** fresh widgets + Effects (no paused-effect state). **(b)** the M3-Phase 1 synchronous non-batched drain contract (item 4) is **preserved** — toggle-then-observe holds; newly-created subtree Effects run before quiescence **within the existing `MUTATION_CAP`**, and a cap-overflowing insertion uses the existing divergence path (documented backstop, not silent staleness). **(c)** structural-mutation ordering = **SM-1** (status quo) — multiple sibling/nested conditionals are kept observable by the **quiescent child-order invariant** (present conditionals settle into declared document order, drain-order-independent); SM-2/3/4 (normatised ordering / two-phase drain / separate insertion budget) weighed and declined — **items 1–3** carried forward with owner-impact reasoning (safe + no regression; model frozen only when the `for`/multi-conditional family reveals requirements) |
| [DD-M3-P6-006](./dd-m3-p6-006-window-title-host-wiring.md) | Window-title host-wiring (R1) surface | **Static title required**: loader passes the component-level `title:` literal to `window::create` in place of `DEFAULT_WINDOW_TITLE` — **no new ABI export** (`abi_spec.md` no-touch). **Dynamic (`String`-binding) title evaluated and explicitly deferred** (FD-D): it needs a window-property binding seam overlapping M4 backdrop/theme wiring; the question is recorded, not closed |

## Phase 6 verification closure (what counts as A4 / A7 evidence)

This section is not a DD — per framing decision FD-C it records the
agreed shape of the proof that closes Phase 6, so the implementation
plan inherits a concrete target. Per the framing constraint that
**state-driven evidence must toggle the state** (positive control,
[../requirements/constraints.md §3](../requirements/constraints.md)),
every conditional and z-order proof is a **before/after pair**, never a
single frame.

Phase 6 closes only when **all seven** of the following are observed.
Items 1–6 are test / GUI evidence; item 7 is a non-test documentary
closure gate (A12). Per FD-C the evidence lines do not collapse even
where they share helper infrastructure — `wasamoc check` diagnostics,
pure-logic measure-arrange / presence-reducer tests, lowering /
IR-roundtrip tests, and Windows-runtime integration tests each carry
distinct evidence meaning.

1. **`wasamoc check` compile-time evidence (host-independent).**
   Pure-logic tests in the check / lower path cover:
   - **ZStack surface lowering positive controls** — the gallery
     lightbox `.ui` (item 6) and representative `ZStack { … }`
     fixtures compile cleanly and lower to a ZStack IR node with
     direct children in document order (DD-M3-P6-001).
   - **ZStack attribute rejection** — attributes outside the
     documented ZStack surface (e.g. `z-index`, `spacing`,
     `columns`) are rejected on ZStack (DD-M3-P6-001 / DD-M3-P6-002).
   - **Conditional grammar positive controls** — `if <bool-state> {
     … }` and `if true { … }` compile cleanly and lower to a
     **member-level control-flow construct** (`ControlFlowNode::If`)
     carrying the branch condition and body (DD-M3-P6-003 /
     DD-M3-P6-004).
   - **Conditional condition rejection** — a non-bool condition
     (`if count { … }` where `count: i32`; `if "x" { … }`), a
     condition referencing an undeclared name, and (per the deferred
     expression grammar) an operator condition (`if !ready { … }`)
     each surface a `wasamoc check` diagnostic naming the offending
     shape (DD-M3-P6-003).
   - **Conditional placement rejection** — an `if` block in a
     position where members are not admitted (per the grammar of
     DD-M3-P6-003) surfaces a diagnostic.

2. **Pure-logic layout + presence-reducer evidence (host-independent).**
   - **ZStack measure-arrange** — union (per-axis max) sizing across
     1, 2, and 3 children; each child arranged within the ZStack
     content rect; per-child alignment defaults (center) and
     `h-align` / `v-align` overrides anchor correctly; a Fill child
     contributes `0.0` to the union desired-size and fills the ZStack
     content rect under a bounded allocation, while the ZStack's
     `Fill/Fill` default takes the parent allocation (DD-M3-P6-002).
   - **ZStack document-order z-order** — overlapping painted children
     resolve later-child-on-top at the layout/paint-order layer
     (the real-Visual confirmation is item 4).
   - **Conditional presence reducer** — the pure `bool →
     present/absent` decision (true ⇒ subtree present, false ⇒ absent)
     is exercised as a free function independent of the Compositor
     (DD-M3-P6-004).

3. **Lowering / IR-roundtrip / loader-invariant evidence (host-independent).**
   - **ZStack roundtrip** — emit → load of a ZStack subtree preserves
     child count and document order.
   - **Control-flow member roundtrip** — emit → load of an `if`
     construct preserves the branch condition and body; the control-flow
     member materialises no runtime widget (DD-M3-P6-004).
   - **Loader rejection** — a control-flow member with a non-bool /
     unresolved condition or more than one branch (until `else`), and a
     ZStack with a malformed shape, surface `WASAMO_ERR_IR_MALFORMED`
     (DD-M3-P6-003 / DD-M3-P6-004 dual gate).

4. **Windows-runtime integration evidence (mock-free, CI-gated,
   fail-not-skip).** Per
   [../../../../CLAUDE.md §Testing rules](../../../../CLAUDE.md#testing-rules):
   - **ZStack real-Visual z-order** — a `.ui` with overlapping ZStack
     children asserts the child Visual order / paint order matches
     document order under the live Visual tree (z-order is **not**
     dischargeable by pure logic alone — the real Visual child order
     must be confirmed).
   - **ZStack outer-bounds clip** — the ZStack Visual has a non-null
     `Visual.Clip` (InsetClip); each child Visual has
     `Visual.Clip = null` (clip-absence regression guard, symmetric
     with the Grid / ScrollView / WrapPanel precedents)
     (DD-M3-P6-002).
   - **Conditional toggle insert/remove** — a live `WidgetNode` tree
     with an `if`-bound subtree: writing the `bool` true→false→true
     **inserts / removes** the subtree (and its Visuals) from the
     parent (DD-M3-P6-004), and the absent subtree's Effects are
     disposed / recreated (DD-M3-P6-005).
   - **Drain proof contract (item 4)** — with `BATCH_DEPTH == 0`, a
     write that toggles the condition drains before control returns
     (toggle-then-observe): immediately after the toggling call the
     subtree presence is observable, and freshly-inserted subtree
     Effects have run (DD-M3-P6-005). This pins the M3-Phase 1
     synchronous-drain contract under structural mutation.
   - **R1 static title** — a `.ui` whose component declares
     `title: "Gallery"` produces a native window whose title bar
     reads `"Gallery"`, not `"Wasamo"` (DD-M3-P6-006).

   All fixtures fail (not skip) on a runner that cannot create the
   Compositor; the skip-guard inherits the Phase 2 T11 / Phase 3 / 4 /
   5 pattern (fires on `0x80070005` from `wasamo_init`).

5. **Assistant-visible GUI evidence.** Launch +
   `Graphics.CopyFromScreen` screenshot + assistant analysis,
   per-monitor-DPI-aware capture; positive control = the
   **before/after toggle pair** (lightbox closed vs open), per
   [../requirements/constraints.md §2/§3/§4](../requirements/constraints.md).
   The z-order proof is read off the open frame: the photo / caption /
   nav are painted **over** the scrim, and the scrim dims (does not
   replace) the thumbnails behind it. `Start-Process` survival is a
   supporting "no early crash" signal only.

6. **End-to-end host evidence (visible smoke).**
   `examples/gallery/gallery.ui` is grown **additively** with the
   lightbox slice (FD-B): a thumbnail-gallery background (WrapPanel /
   ScrollView slice, Phase 3/4) with a `bool`-toggled (`is_lightbox_open`)
   ZStack overlay = scrim (`Box { fill: #RRGGBBAA }`, FD-G) + centered
   photo (Box aspect 4:3 + Text placeholder) + caption (VStack) + nav
   (`<` `>` `x` text Buttons). The toggle is driven by a **plain text
   Button click handler** (`Open lightbox` opens, `x` closes), so the
   proof traverses **event handler → `bool` state → conditional
   subtree** (FD-C; thumbnail-click-to-open is out of scope — Box
   hit-testing / image Button is M4). `examples/gallery-rust/` builds
   and runs the grown sub-screen. **Visual correctness** (overlay
   appears on open and is gone on close; z-order; scrim dim) is
   **owner-manual GUI smoke** per FD-I, distinct from the assistant
   pre-check in item 5.

7. **A12 spec-closure gate (non-test, external-reader).** Phase 6 does
   not close until [`docs/dsl_spec.md`](../../../../docs/dsl_spec.md)
   carries, at the external-reader bar (§Upstream document revisions
   enumerates the exact section set; this item is the *close condition*
   on it):
   - the **ZStack chapter** (§4.13: union sizing, document-order
     z-order, outer-bounds clip, per-child alignment) plus its §4.4
     widget-registry row (DD-M3-P6-001 / DD-M3-P6-002);
   - the **conditional-rendering chapter** (§4.14 + the §3 grammar
     addition for the `if`-block member) written as the **first chapter
     of Wasamo's structural rendering model**, such that an external
     reader sees `if` as the first member of a structural control-flow
     family (`else` / `switch` / `for` arriving in the same family),
     including the **absent=fresh-on-return / opt-in-retention**
     normative semantics (DD-M3-P6-004);
   - **invalid examples / diagnostics** — the reader-facing rejected
     shapes (non-bool condition, undeclared-name condition, operator
     condition, misplaced `if`, disallowed ZStack attributes) match the
     diagnostics exercised in items 1 / 3 (DD-M3-P6-003);
   - the **Moment 1 → Moment 2 re-sync** completed — the
     `M3-Phase 6 design accepted; implementation pending` markers
     flipped to `closed; implementation-synced`, with any design-draft
     ↔ implementation divergence corrected (§Upstream document
     revisions, Moment 2).
   This gate is documentary, not an automated test: it is the
   external-reader-reproducibility check that lets the owner judge at
   close that A12 is satisfied independently of the test evidence in
   items 1–6.

Items (1)–(4) are the automated A4 / A7 evidence set. Items (5)–(6)
are required for Phase 6 close under A11; item (7) is the non-test
A12 spec-closure gate. The corresponding implementation checklist
(which crate / which test file / which fixture) belongs in the Phase 6
implementation plan, not here.

## Forward-compat exposure (anticipated future surfaces)

Documented here so later family-extension / input / iteration work has
a named landing point. None of these requires modifying Phase 6's IR
shape, runtime widget catalog, or measure-arrange algorithm — all are
additive.

1. **`else` / `else if` / `switch`.** The member-level control-flow IR
   (`ControlFlowNode` with a branch list) and the `if`-block grammar are
   designed so these slot in as same-family variants (DD-M3-P6-003 /
   DD-M3-P6-004). `else` lifts the single-branch restriction (an extra
   `Branch`); `switch` is a new `ControlFlowNode` variant with the same
   present/absent runtime machinery — **no `IrMember` shape change**.
2. **Iteration (`for item in items { … }`, Phase 7).** A new
   `ControlFlowNode` variant reusing the
   `BindingTarget::ConditionalSubtree` → `ForLoopSubtree` runtime
   seam. Phase 7 adds keyed identity / state retention (the Element-
   level identity DD-M3-P6-004 defers); Phase 6's full-rebuild policy
   is the un-keyed base case.
3. **Expression-grammar extension** (`!ready`, comparison, logical
   operators) — DD-M3-P6-003 / Q5. Intentionally a **uniform** growth
   across all `expr` positions (condition and every property RHS at
   once), not a condition-only pocket (E1.5/E1.75 declined for that
   reason); the control-flow member and runtime seam are unaffected.
4. **Declared-tree / entity-tree identity model** (state retention
   across absent→present, `key:` attributes, Element-level
   reconciliation) — DD-M3-P6-004 forward-compat. Phase 6 ships the
   un-reconciled base case as **normative author-visible semantics**:
   a subtree that goes absent and returns is **fresh** (state resets).
   Future retention is **opt-in (keyed)** so this default never changes
   silently; the declared tree (the control-flow member) is the stable
   identity anchor, so a future reconciler is added **without an IR
   change**.
5. **Dynamic (`String`-binding) Window title** — DD-M3-P6-006.
   Evaluated and deferred; lands when a window-property binding seam
   is introduced (alongside or after M4 backdrop / theme wiring, Q2).
6. **Explicit `z-index` / author-facing layering** — out of scope;
   paint order is fixed to document order (DD-M3-P6-002).
7. **Scrim alpha *styling controls*** (theming / named palette /
   dynamic alpha) — M3 out of scope; the half-transparent scrim
   itself is expressible today via the existing `fill: #RRGGBBAA`
   literal (FD-G, dsl_spec §4.9).

## Out of scope

Each is explicitly out of A4 / A7 scope or deferred — not deferred by
oversight (consolidated from
[../requirements/framing.md §Phase 6 scope](../requirements/framing.md#phase-6-スコープ)):

- **Iteration / repeated-child generation grammar** — Phase 7. The
  lightbox needs only a single-subtree toggle. Phase 6's conditional
  grammar is nonetheless designed as the first member of the same
  structural family (forward-compat item 2).
- **Property-toggling conditional (approach 1)** — explicitly not the
  Phase 6 model (FD-CR). Phase 6 proves structural present/absent, not
  `visible` / `enabled` toggling of an always-built tree.
- **`else` / `else if` / `switch`** — reserved family members, not
  implemented (forward-compat item 1).
- **Identity preservation / state retention across absent→present,
  `key:` attributes** — DD-M3-P6-004 ships the full-rebuild base case
  (forward-compat item 4).
- **Expression operators in the condition** (`!`, comparison,
  logical) — deferred to a future expression-grammar extension
  (DD-M3-P6-003 / Q5).
- **Explicit `z-index` / author-facing layering attribute** —
  DD-M3-P6-002 fixes paint order to document order.
- **Per-child clip on ZStack children and an author-facing `clip:`
  surface** — DD-M3-P6-002 ships only the ZStack outer-bounds clip.
- **Dynamic (`String`-binding) Window title implementation** —
  evaluated in DD-M3-P6-006 but not committed this phase (FD-D);
  static title is required.
- **Scrim alpha *styling controls*** (theming / named palette /
  dynamic alpha) — M3 out of scope; the literal `#RRGGBBAA` scrim is
  in scope (FD-G).
- **Lightbox swipe / pinch / keyboard gestures, hit-testing / focus
  capture / modal focus trap** — M4 input. close/nav are text-Button
  click handlers; nav photo content is M3 placeholder.
- **Button selected-state surface** — Phase 8 (A10); tab-strip /
  selected-thumbnail styling not opened here.
- **Image widget surface** — M4. The lightbox photo is Box(aspect 4:3)
  + Text placeholder.
- **per-monitor DPI awareness (runtime)** — M4 (DD-V-022/023,
  [../requirements/constraints.md §5](../requirements/constraints.md));
  Phase 6 verifies logical-pixel correctness only and notes DPI blur
  as a known M4 residual during evidence analysis.
- **reactive-drain items 1–3** (cycle detection, ordering ties,
  fan-out × `MUTATION_CAP`) — DD-M3-P6-005 weighs a structural-mutation
  ordering/transaction model (SM-1..SM-4), adopts **SM-1** (status quo),
  and declines SM-2/SM-3/SM-4: structural mutation introduces **no
  safety regression** (the §6.8.6 dispose-ahead-of-teardown invariant)
  and **no observability regression** (inter-Effect ties were already
  implementation-defined), so the items are carried forward to be
  re-evaluated when the family (`for`, multiple / nested conditionals,
  large subtrees) reveals the real transaction requirements — not
  frozen on the single-`if` case.
- **`TypedValue` generic value union** (F5 maintained — no new scalar
  type).

## Upstream document revisions (Moment 1 / Moment 2)

Phase 6 follows the two-moment structure inherited from Phase 2/3/4/5
(framing decision FD-H). Doc set and commit shape follow the
per-review-concern rule
([../../../../CLAUDE.md §Commit rules](../../../../CLAUDE.md#commit-rules),
[../../../procedures/retrospectives.md](../../../procedures/retrospectives.md)).
Per FD-H the ADR makes an **explicit touch / no-touch judgment** for
`architecture.md` and `abi_spec.md` (no "may touch" left ambiguous).

The dsl_spec section markers mirror the Phase 2/3/4/5 form:

```
**Phase status:** M3-Phase 6 design accepted; implementation pending
```

flipping at phase close to:

```
**Phase status:** M3-Phase 6 closed; implementation-synced
```

placed as the first line under each new chapter heading.

**Moment 1 — ADR Accepted commit set (design-spec draft).** Each lands
as its own commit on the pre-doc branch per the per-review-concern
rule; the draft-side doc set is enumerated below (the Moment 2
phase-sync set is a related but distinct rule):

- `process/milestone-3/phase-6/decisions/preamble.md` and
  `dd-m3-p6-*.md` (this directory) — ADR `Status: Accepted` flip.
- [`docs/dsl_spec.md`](../../../../docs/dsl_spec.md) — **two new
  chapters** as design-spec drafts: a **ZStack chapter** (new §4.13,
  alongside §4.9 Box … §4.12 Grid) with the DD-M3-P6-001/002 sub-issues
  as outline plus a §4.4 widget-registry row for `ZStack`; and a
  **conditional-rendering / structural-rendering-model chapter** (new
  §4.14 + a Grammar §3 addition for the `if`-block member and a §4.6
  condition-expression note) written as the **first chapter of the
  structural control-flow family** per A12 (DD-M3-P6-003/004/005
  sub-issues as outline; the `if` construct defined as a structural
  control-flow construct, **not** a §4.4 registry widget — a pointer
  from §4.4 names it, mirroring the `Cell` treatment; the
  absent=fresh-on-return / opt-in-retention normative semantics stated
  in §4.14). Section markers: `M3-Phase 6 design accepted;
  implementation pending`.
- [`docs/architecture.md`](../../../../docs/architecture.md) —
  **touch (judged required).** ZStack entry under the layout-engine
  section (union sizing + document-order z-order + outer-bounds clip,
  no intermediate Visual); the conditional construct under the IR
  section (**member-level `IrMember`/`ControlFlowNode` structural IR**,
  the consequential schema change of DD-M3-P6-004) and the reactive
  section (the `BindingTarget::ConditionalSubtree` variant now filled,
  §6.8.7/§6.8.8; the present/absent insert/remove path; the effect-
  lifecycle policy of DD-M3-P6-005; the SM-1 structural-ordering
  disposition); and a note under §9 Three-Layer Tree Model on the
  declared-tree / entity-tree separation that the conditional construct
  introduces in nascent form (DD-M3-P6-004).
- [`docs/abi_spec.md`](../../../../docs/abi_spec.md) — **no touch
  (judged).** DD-M3-P6-006 routes the static title through the
  existing `wasamo_load_ui` → `window::create` internal path with **no
  new ABI export and no `PropertyValue` tag**; LayoutError stays
  internal; the `If` construct adds no host-facing ABI surface. (If
  implementation re-sync surfaces an unavoidable ABI need, it is
  recorded at Moment 2 with owner confirmation.)
- [`../../plan.md`](../../plan.md) — Phase 6 row populated (Status: in
  progress; implementation-plan link; ADR link).
- `process/milestone-3/phase-6/implementation/preamble.md` /
  `plan.md` — implementation planning opened after ADR acceptance,
  with the final-task retrospective split (FD-I) represented from the
  start ([../requirements/constraints.md §6](../requirements/constraints.md)).

Implementation begins only after these commits land.

**Moment 2 — Phase close commit set (impl re-sync).**

- [`docs/dsl_spec.md`](../../../../docs/dsl_spec.md) §4.13 / §4.14 —
  section markers flip to "closed; implementation-synced", plus
  corrections if design draft and implementation diverged (marker flip
  required regardless; corrections conditional). Earlier-phase spec
  gaps surfaced during re-sync may fold in with explicit owner
  confirmation (retroactive spec-gap minimum-fold pattern).
- [`docs/architecture.md`](../../../../docs/architecture.md) — top
  Status flips to `M3-Phase 6 complete`; impl-divergent paragraphs
  re-synced.
- `process/milestone-3/phase-6/implementation/log.md` and
  `retrospectives/phase-end.md` — phase-close retrospective link, CI
  evidence pointer, impl summary; the implementation plan enters
  `in-progress → completed`.
- [`../../plan.md`](../../plan.md) Phase 6 row — Status flips to
  complete.
- This ADR — touch only if one of the three retrospectives.md
  §phase-sync ADR-touch cases applies.
- Step retro `phase-sync` items must all close into `doc-folded` /
  `carry-forward` / `local-only` at Moment 2.

No ROADMAP revision is anticipated — A4 / A7 / R1 are already explicit;
this ADR operationalises them.

## Inputs absorbed

Mapping from [../requirements/framing.md](../requirements/framing.md)
framing decisions and aligned outcomes to DDs and ADR sections:

| Source | Disposition | Consumed at |
|---|---|---|
| FD-CR — structural-rendering thesis (v1=approach 2 / future=approach 3 / approach 1 non-central / control-flow family / runtime identity) | Cross-cutting design lens | §Context (thesis lens); DD-M3-P6-003 / 004 / 005 evaluation axes |
| FD-A — DD slate completeness | Discipline | DD slate (6 DDs); §Out of scope |
| FD-B — visible proof = lightbox | Constraint | §Verification closure items 5–6 |
| FD-C — verification strategy + positive control | Constraint | §Verification closure items 1–6 |
| FD-D — Window title (static required / dynamic evaluated, defer ok) | Settled framing | DD-M3-P6-006 |
| FD-E — conditional binding lifetime + toggle-observe contract | Settled framing | DD-M3-P6-005 (a)/(b) |
| FD-F — ZStack + conditional shipped as a unit | Settled framing | §Context; §Verification closure item 6 (single lightbox slice) |
| FD-G — scrim via existing `#RRGGBBAA` | Settled framing (confirm) | §Verification closure item 6; §Out of scope (alpha styling controls) |
| FD-H — two-moment sync + explicit architecture/abi touch judgment | Constraint | §Upstream document revisions |
| FD-I — final-task retro split | Discipline | §Upstream document revisions (impl-plan opening) |

Cross-phase / source inputs:

| Source | Disposition | Consumed at |
|---|---|---|
| M3-Phase 5 DD-M3-P5-005 (document-order z-order; outer-bounds clip in / per-child clip out; no intermediate Visual; same-cell overlap → ZStack) | Pattern reuse | DD-M3-P6-002 |
| M3-Phase 5 DD-M3-P5-001 (`Cell` is an IR-only node kind consumed by lowering, not a runtime widget) | Pattern reuse (interpret-not-render posture) + deliberate contrast | DD-M3-P6-004 (control flow is a member-level construct, *not* an `IrNode`/widget kind like `Cell` — same IR-only posture, different category) |
| M3-Phase 1 DD-M3-P1-002 (`bool` scalar; `true`/`false` keywords) | Prerequisite | DD-M3-P6-003 (condition type) |
| M3-Phase 1 DD-M3-P1-007 (per-type writer seam) | Pattern reuse | DD-M3-P6-004 (conditional binding registration) |
| architecture.md §6.8.6 (Effect lifetime: structural disposal; re-attach creates fresh Effects) | Direct input | DD-M3-P6-005 (a) |
| architecture.md §6.8.7/§6.8.8 (`BindingTarget` ConditionalSubtree slot; structural bindings drop old Effects via teardown) | Direct input | DD-M3-P6-004 |
| M2 handoff §3 item 4 (synchronous non-batched drain proof contract) | Preserve, not revise | DD-M3-P6-005 (b) |
| M2 handoff §3 items 1–3 (cycle / ties / fan-out) | Explicit carry-forward | DD-M3-P6-005; §Out of scope |
| dsl-grammar.md Q5 (condition expression grammar extension point) | Deferral input | DD-M3-P6-003 |
| dsl-grammar.md Q6 (conditional-rendering philosophy + runtime identity) | Originating thesis | FD-CR / §Context; DD-M3-P6-003 / 004 / 005 |
| dsl-grammar.md Q2 (Window-derived prop runtime wiring) | Direct input | DD-M3-P6-006 |
| dsl_spec.md §4.9 (`fill: #RRGGBBAA` admits scrim use case) | Confirmation | FD-G; §Verification closure item 6 |
| target-app pre-doc / spec.md (conditional normative in M3, not M4-reserved; ZStack = overlay / document-order z-order) | Direct input | §Context (A12); DD-M3-P6-002 |
| constraints.md §1–§7 (R1 owning; assistant-visible evidence; positive control; DPI not adopted; reactive-drain obligation; final-task retro split) | Constraint set | DD-M3-P6-006; §Verification closure; DD-M3-P6-005; §Out of scope |

## Revision history

| Date | Change |
|---|---|
| 2026-05-31 | Review revisions folded (preamble + DD-001/002/003/004/005), still Status: Proposed. Reflects the strategic-design / owner-alignment review and the recommendation-choice review findings. |
| 2026-05-31 | Initial draft (Status: Proposed). All 6 DDs at Proposed pending owner review pass. Framing-level owner alignment confirmed 2026-05-31 ([../requirements/framing.md §Owner alignment outcome](../requirements/framing.md#オーナー合意の記録owner-alignment-outcome)) settles FD-CR / FD-B / FD-D / FD-E / FD-F / FD-G; the remaining ZStack and conditional-grammar sub-decisions are ADR-review approvals. |
