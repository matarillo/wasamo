# DD-M3-P6-008 — Component-root window-attribute / widget-attribute boundary

**Status:** Proposed
**Phase:** M3-Phase 6
**Surfaced by:** T7 (gallery lightbox slice) — the first example with a
**ZStack root**, which made the latent boundary fail deterministically at
`wasamo_load_ui`.

## Context

A component declares window-level attributes at the component body level:

```
component Gallery inherits Window {
    title: "Gallery"
    backdrop: mica
    theme: system
    ZStack { … }      // root widget
}
```

`wasamoc` lowering splices these component-level prop-binds onto the **root
widget's** `props`, and component-level dynamic binds onto its `bindings`
([`wasamoc/src/lower.rs`](../../../../wasamoc/src/lower.rs#L59) —
`root.props.splice(0..0, comp_props)` / `root.bindings.splice(…)`). T6
read the static title back from `component.root.props`
(`resolve_static_window_title`), cementing this "window attributes live on
the root node" model.

### Root cause: the base type is never modeled

The splice is a *symptom*. The root cause is that `title` / `backdrop` /
`theme` are *Window* attributes, but the system never models `Window` as an
entity that owns attributes. `inherits Window` is reduced to an inert string
at the front of the pipeline (dsl_spec §4.1: "`<Base>` is stored as a
string; no base-type validation"), and `IrComponent.base`
([`wasamo-ir/src/lib.rs`](../../../../wasamo-ir/src/lib.rs#L186)) is
round-tripped through emit / parse but **never semantically consumed** — no
code branches on it. `Window` is not in the widget registry, has no prop
catalog, and is not a modeled type; it is a label.

So `IrComponent { name, base: String, states, root: IrNode }` conflates the
two distinct things a `component … inherits Window` denotes: **what the
component *is*** (a Window, with Window's attribute surface — title /
backdrop / theme) and **what it *contains*** (the content subtree, the
`root` widget). It models the *contains* half as a structured node (`root`)
and degrades the *is* half to a string (`base`). Window-ness is a name, not
a structure — so Window attributes have no structural home.

Lowering is **not** missing the context to place them correctly: when it
collects component-body prop-binds it knows both that they are
component-level and that `base == "Window"`
([`wasamoc/src/lower.rs`](../../../../wasamoc/src/lower.rs#L36)). It
*discards* that context and splices onto `root` precisely because there is
no typed destination for a Window attribute — the only structured node in
the IR is the content root, so the attributes are squatted there. T6 read
the static title back off `component.root.props`, building on the squat
rather than questioning it. This stayed invisible because the M2 corpus
declared no window attributes (the counter had no `title:`), so the model
was never exercised until T6 added one and T7 made the content root a
ZStack with a strict validator.

**Why the splice looked natural at the time.** At M2-Phase 6 the IR was
deliberately minimal — a component was *states + a single root widget*,
which was all the counter example required, and `window::create` already
consumed the component as the window-equivalent host. With no window
attribute anywhere in the corpus, "component = states + root" was
sufficient, and a component-level prop-bind had exactly one structured place
to land. The splice was not a misjudgment against a known requirement; it
was a minimal IR meeting a requirement (window attributes) that did not yet
exist. T6 is the growth point where that requirement arrived — and the
model was carried rather than revisited because `root.props` was a working
drop-site. This is a minimal IR reaching its growth point, not a past
implementation that was simply wrong.

This missing owner makes the **two validation gates asymmetric**:

- **`wasamoc check`** sees these as *component-level* prop-binds in the AST,
  **before** the splice. It has no component-prop catalog, so it passes
  *any* component-level name through
  ([`bind_component_level_no_type_check`](../../../../wasamoc/src/check.rs#L1883)).
- The **runtime loader** sees the **post-splice IR**, where the window
  attributes are now ordinary widget props/bindings on the root node.

Most widget validators do no strict unknown-prop rejection, so a VStack /
Grid / Box root silently absorbed the spliced window attributes and the
asymmetry never showed. **Phase 6's ZStack validator
(`validate_phase6_zstack_node_invariants`) is the first — and currently
only — widget validator that strictly rejects unknown props and *all*
bindings.** When T7 made the root a ZStack, the spliced `title` hit
`ZStack accepts no Phase-6 attributes; found title`.

T7's fix (a root-only allowlist of `title | backdrop | theme`) unblocks the
gallery but does **not** close the boundary: it is narrower than what the
compiler accepts, and it only covers props.

## Sub-issue — the general problem under the Window instance

Stated narrowly, *window* attributes have no owner. Stated generally, **the
component-level member namespace is undifferentiated.** A bare `name: value`
at the component body lowers identically whether it is a Window attribute
(`title`), a future base-type attribute under some other `inherits`
(`Dialog` / `Page` / `Scene` / `Popover`), component metadata, an exported
component property, or a typo. The DSL gives them all one production
(`property_bind`) and the IR gives them all one destination (the root node);
no layer assigns them a *semantic namespace*. Window is simply the first
instance to demand an owner — and the consequence of closing it Window-only
is that the next base type re-opens the identical boundary. The "dual-gate
divergence" and the missing Window-attribute owner are both symptoms of this
undifferentiated namespace.

Where this lands eventually: the component-body namespace will likely split
into distinct roles —

```
component body namespace
  - state members
  - base/host attributes          (title, backdrop, theme, …)
  - content root                  (the single root widget)
  - component metadata
  - exported / input component props
```

(A) below separates only **base/host attributes** from the **content root** —
the two that actually collide today. Component metadata and exported/input
props are *not* solved here; they stay folded into the same undifferentiated
surface until a later phase gives them owners. Naming the eventual split
keeps (A)'s scope honest: it resolves the live collision, not the whole
namespace.

Two facets, both currently divergent and both only pinned as interim:

1. **Props.** `wasamoc check` accepts an arbitrary component-level prop; the
   runtime ZStack root accepts only `{title, backdrop, theme}`. A fourth
   future window prop (or a typo such as `titlee:`) is accepted by the
   compiler but rejected by a ZStack root — and silently accepted by any
   other root.
2. **Bindings.** A component-level dynamic bind (e.g. `bind title = …`,
   FD-D, deferred) is spliced onto `root.bindings`; the ZStack root rejects
   *all* bindings unconditionally, while the compiler passes the
   component-level bind. (Dynamic title is unimplemented, so this is latent
   today.)

## Design space

The options form a spectrum from "reconcile the two gates" (least) to "model
base types as a system" (most). They are framed against the end-state (B) so
the in-phase choices are positioned as steps toward it, not as isolated
patches.

### (B) First-class base/host type system — the end-state

Model `Window` (and future `Dialog` / `Page` / `Scene`) as a real host/base
type: a registry entry, an attribute catalog, `inherits` validation, and
lowering that distributes component-level attributes onto the base's surface.
This is the complete form the other options approximate.

- Gain: the namespace is fully owned; the typo / diagnostic / binding facets
  all close together; a new base type is an additive registry entry, not a
  re-opened boundary.
- Give up: far beyond Phase 6 — a type-system feature touching DSL, check,
  IR, runtime, and spec. Listed to **anchor the end-state; not proposed for
  Phase 6.**

**Where (B) is essentially owned — M4, not a vague "later."** Three M4
acceptance criteria ([`_roadmap.md` §M4](../../../_roadmap.md#m4-interaction-stack))
converge to force base-type modeling, so M4 is its real home, not an
open-ended deferral:

- **`backdrop` / `theme` are M4 deliverables.** Today only static `title` is
  wired (T6); `backdrop: mica` (Mica/Acrylic) and `theme: system` (accent
  follow-through) are M4 criteria. The moment M4 wires them from the DSL, it
  cannot leave them squatted on the content root — it forces window
  attributes a real home.
- **Multi-window makes `Window` a real type.** `IrComponent.base` can stay an
  inert string only while there is one implicit singleton window that the
  runtime treats the component *as*. M4 multi-window instantiates windows as
  addressable entities with per-window state and focus — the structural point
  where "model the base type" becomes load-bearing.
- **The multi-window ABI is pre-freeze.** The roadmap pulls multi-window
  pre-1.0 precisely because its ABI surface cannot be appended post-freeze
  (M6); a window-attribute representation that crosses the ABI must settle in
  M4, where that ABI is designed.

The full **diagnostic** surface (rich editor diagnostics for the typo hole)
is **M5** (VS Code LSP), and additional host types (`Dialog` / `Popover` /
`Page`) stretch M4→M5 — but the **core Window base-type modeling is M4's
responsibility.** This pins the consequence for (A) below: an in-phase (A)
must be **forward-compatible with M4's window-entity model**, or M3 buys a
surface M4 has to migrate again.

### (A) IR structural separation — the minimal step toward (B)

Stop splicing onto the root; give the component a dedicated attribute surface
so the content root is content again and base/host attributes have a
structural home. Two sub-choices on **abstraction level**:

- **(A1) Window-specific** — `window_props` / `window_bindings` on
  `IrComponent`. Simplest; names exactly the concrete need. Risk: closes
  Window-only, so a later `inherits Dialog` adds `dialog_props`, … — surface
  proliferation, and the next base type re-opens this DD.
- **(A2) Base/host-general** — an attribute surface *not* tied to the literal
  `Window` name, so future base types reuse it without schema churn. Two
  distinct **depths** hide under this label and must be chosen between, not
  blurred:
  - **(A2a) Host surface** — flat `host_props` / `host_bindings` on
    `IrComponent`; `base: String` kept as-is. This only *separates*
    host-owned attributes from the content root; it adds no `inherits`
    semantics. `host_props` matches the "a Window / Dialog / Page *hosts* a
    content subtree" reading — the meaning actually needed now. It is an
    **internal IR improvement, not an ABI-facing window descriptor**: the
    host-facing handle / descriptor a multi-window ABI needs is M4's to
    design, and `host_props` deliberately does not fix it.
  - **(A2b) Structured base object** — fold `base: String` into `base:
    IrBase { name, props, bindings }`. This puts "the base *owns* attributes"
    into the IR and is materially closer to (B); it begins carrying
    `inherits` semantics.

  A2a is the lighter step (an IR separation); A2b is a down-payment on the
  base-type system. Picking "A2" without picking the depth is the blur to
  avoid.

**Why A2b is not taken in Phase 6 — a two-sided read, not a dismissal.** Now
that (B) is established as an M4 responsibility, the natural counter-argument
is "if B is unavoidable at M4, why not pay down A2b now?" A2b is genuinely
weaker *both* ways it can be defined:

- **If `IrBase` is merely `base: String` plus `props` / `bindings` grouped
  into a struct**, its difference from A2a is small — it stops the same root
  contamination — while it *adds* a schema migration. More churn for the same
  separation.
- **If `IrBase` is more than grouping** — if it actually means "the base
  *owns* attributes" — it begins deciding base-type semantics: known-base
  validation, unknown-base handling, an attribute catalog keyed by base, the
  relationship between a component's `base` and M4's window *instances*, where
  attributes live under multi-window, and the mapping to an ABI-facing window
  descriptor. Those are exactly M4's questions (multi-window identity, window
  entity, ABI shape — see the (B) milestone note). Deciding them as a side
  effect of a Phase-6 ZStack boundary fix would pre-empt M4's design with no
  multi-window example to test it against.

So A2b is either too light to be worth the migration or too heavy to settle
outside M4. A2a isolates the collision that is actually broken now — host
attributes vs content root — **without choosing the carrier for the future
base-type system.**

A **representation** sub-choice is orthogonal to A1/A2: the surface may be
**generic lists** (`Vec<IrProp>` / `Vec<IrBinding>`, like widget props) or
**typed fields** (`WindowAttrs { title, backdrop, theme }`). Typed fields are
typo-proof but force an IR change per new attribute; generic lists are
flexible but push validity to a catalog. Generic lists are consistent with
the rest of the IR and are the assumed default unless the owner wants the
typed form.

- Gain (A overall): removes the root cause — the content root is pure content
  again, base/window attributes have a structural home, provenance is
  preserved, and future strict-root widgets need no special-casing.
- Give up: an **IR schema + textual-IR format change** (schema/IR-migration
  high-risk — full independent review) across `wasamo-ir` + `wasamoc`
  emit/lower + the runtime parser + the test corpus, plus a
  `docs/dsl_spec.md` / `docs/architecture.md` Moment-2 sync. Precedented this
  phase by the T4 `Vec<IrMember>` migration, but larger than a localized fix.

### (D) Compiler-owned catalog, runtime mirrors it — no schema change

Give `wasamoc check` a Window-attribute catalog with diagnostics; make the
runtime root allowlist the **mirror** of that catalog (the established mirror
pattern, cf. `STAR_WEIGHT_MAX`).

- Gain: closes the divergence with a *principled* allowlist; closes the typo
  hole at compile time; no IR schema change; fits the R1 / T6 window theme.
  Spec sync is a small dsl_spec addition (the Window-attribute set), not a
  format change.
- Give up: it teaches the gates to *recognize* window attributes but leaves
  them stored on the wrong node — the root cause (the namespace is
  undifferentiated; attributes still ride `root.props`) persists, and two
  mirrored lists must stay in sync. **It also removes the pain:** once both
  gates agree, the splice no longer fails, so the pressure to migrate to
  (A)/(B) drops and the mis-model gets pinned into still more tests. D lowers
  short-term risk at the cost of *institutionalizing* the wrong shape. The
  binding facet also stays interim until dynamic title (FD-D).

### (C) Runtime accepts anything on the root — rejected (the floor)

Make the ZStack validator treat any prop on the root as a non-widget
attribute and not reject it.

- Gain: minimal change; the divergence disappears.
- Give up: **wrong direction** — window attributes stay unvalidated
  *everywhere* (the `titlee:` typo passes every gate), and the root ZStack
  loses its junk-attribute guard. Symmetry bought by deleting a check.
  Retained only as the rejected floor of the spectrum.

### Set aside (considered, not carried)

- **(E) DSL syntax change.** Require the source itself to separate attributes
  from content — an explicit `window { … }` attribute block, or a literal
  `Window { … }` node wrapping the content root. This is the only option that
  questions the *DSL*, not just the IR: today the component body mixes
  attributes and a content root in one scope, and that co-habitation is part
  of the ambiguity. Set aside for a reason more essential than its breaking
  cost: window attributes sitting directly in the component body read
  *naturally* (`component Gallery inherits Window { title: … ZStack { … } }`
  is declarative and clear); the ambiguity is **not** in the syntax but in
  the compiler / IR never assigning that syntax a semantic namespace.
  (A2)/(B) add the semantic owner while keeping the natural syntax, whereas
  (E) changes the syntax to encode an owner the compiler could instead infer.
  Recorded because a future base-type design (B) may still revisit the
  syntax — but a breaking change is unwarranted when the missing piece is
  semantic, not syntactic.
- **(D+) Provenance metadata.** Keep the splice but tag each spliced prop
  with its origin (`prop.origin = ComponentLevel(Window)`). No cleaner than D
  if it needs a textual-IR change, and strictly worse than A as a root fix; a
  half-measure, set aside.

## Comparison criteria

Evaluate the carried options against these axes, in roughly priority order:

1. **Semantic ownership** — does it give window/base attributes a real owner
   (B, A) or only reconcile the gates (D, C)?
2. **Namespace generality** — does it close Window-only (A1, D) or open to
   future base types (A2, B)?
3. **Phase-locality** — sized for Phase 6 (A, D), or owned by **M4** (B — see
   its milestone-ownership note) with the diagnostic surface in M5 (E
   likewise M4→M5)?
4. **Migration reversibility** — how costly is the *next* step after this
   one? D→A re-pins tests and user expectations; A1→A2 is another schema
   move; A2→B is mostly additive. This favors not stopping at A1 or D.
5. **Diagnostic quality / typo hole** — (A) does **not** close the `titlee:`
   typo *by itself*; only the **catalog** does. Whichever option ships, the
   catalog is the piece that buys compile-time diagnostics, so "A closes the
   typo hole" holds only if A ships *with* the catalog.
6. **Runtime trust boundary** — is the runtime a compiler *mirror* or a
   defensive reader of untrusted textual IR? This is currently **undefined**
   and changes D's and A's runtime-validation weight: if the runtime must be
   independently robust, a mirror list is *necessary*, not a smell; if it
   only ever consumes `wasamoc` output, runtime re-validation can shrink.
7. **Content-root invariant cleanliness** — (A) returns the root to pure
   content, so strict root validators need no window exemption; (D)/(C) keep
   a root-only exemption that every future strict-root widget must carry.

## If (A) is chosen — acceptance criteria (direction here, details in impl)

This DD selects the *direction*; the following must be settled when (A) is
implemented (they need not all be resolved at acceptance):

- **Depth: A1 vs A2a vs A2b** (Window-specific `window_props` / host-general
  `host_props` keeping `base: String` / structured `IrBase`) and the surface
  name.
- **M4 forward-compatibility** — the chosen surface must promote cleanly into
  M4's window-entity model (multi-window makes the component's host surface
  *one window instance among many*; see the (B) milestone-ownership note).
  This is the concrete reason `host_props` (A2a) is preferred over
  `window_props` (A1): when M4 instantiates `window` as a real entity, an A1
  `window_props` surface is *more likely to need reinterpretation or renaming*
  as the model distinguishes component-level host defaults from concrete
  window instances (not that it necessarily collides — a `window_props`
  "component-level defaults" reading is conceivable), whereas `host_props`
  stays at the weaker "this host's attributes" level. The surface should not
  bake in assumptions a multi-window M4 would have to undo.
- **Generic-list vs typed-field** representation.
- **Catalog owner abstraction** — Window-specific, host-general, or a
  base-type registry entry. Keep it aligned with the IR depth: a host-general
  IR surface (A2a) with a Window-only catalog drifts back toward a latent
  re-divergence (IR general, validation specific), so prefer a host-attribute
  catalog with a Window entry today.
- **Catalog lookup key** — decided at implementation: the `base` string, a
  future host-kind enum, or just the `"Window"` literal while Window is the
  only entry. The DD does not fix this; it only requires the key not bake in
  Window-exclusivity (so a second host entry is additive).
- **Mirror sync mechanism** — if the runtime mirrors the compiler catalog
  (per the trust-boundary stance), how the two are held in lockstep: a shared
  constant, a mirrored unit test, or a golden test (cf. the `STAR_WEIGHT_MAX`
  precedent). Without a sync mechanism the mirror is a fresh drift source.
- Non-Window base handling — keep two checks separate. *Base-name validation*
  (is `inherits Dialog` a known base?) is **carried to B / M4**; A2a keeps
  `base: String` and does not validate the name. What Phase 6 gates is the
  *host attributes*: the Recommendation proposes the **catalog as the gate**
  (`host_props` accepts only catalogued attributes), so an uncatalogued
  attribute is rejected regardless of base, while a non-Window base with an
  *empty* `host_props` (e.g. `inherits Dialog { ZStack { … } }`) is **not**
  rejected by this gate. Confirm at acceptance.
- Whether the **catalog / diagnostic lands with (A)** or as a follow-up
  (without it, the typo hole stays open even after the structural fix).
- **Dynamic-binding policy** — which attributes may be bound (intersects FD-D
  dynamic title). Provisional stance: since dynamic title is deferred (FD-D),
  Phase 6 likely ships `host_bindings` as a *structural* surface that the
  catalog admits **none** of yet — host bindings are parked / rejected and
  only static `host_props` are handled — rather than opening any bindable
  host attribute this phase. Confirm at acceptance.
- Does `resolve_static_window_title` read the **new surface only**, or keep a
  `root.props` fallback during transition?
- **Old root-squatted shape (strengthened):** new emit **must never** splice
  to the root; the tests make the **new shape canonical**; any `root.props`
  compatibility fallback is **explicitly transitional** with a stated removal
  trigger (it is the exact debt this DD is paying down — do not re-incur it
  open-endedly). The runtime parser either rejects or deprecation-warns a
  stray root-squatted attribute; pick one.
- **Provenance — two layers, kept separate.** *IR-level* provenance is
  preserved structurally (an attribute in `host_props` is structurally
  distinct from one in `root.props`). *Diagnostic* provenance (a source span
  saying "this Window attribute came from the component body") lives in the
  AST / `wasamoc check` layer — `IrProp` carries no span — so error-message
  quality depends on the check layer, not the IR surface. Do not assume the
  IR move alone improves diagnostics.

## Recommendation

The root problem is not "window attributes have no home" but that **the
component-level member namespace is undifferentiated** — Window is only its
first instance. Phase 6 should not build the full base type system (B), but
it should stop conflating the content root with base/host-owned attributes in
the IR. That points to **(A) over (D)**: (D) makes the gates agree but
institutionalizes the mis-model and removes the very pressure that would fix
it, while (A) gives the namespace a structural owner and returns the content
root to pure content. The T4 `Vec<IrMember>` migration precedent shows Phase
6 can absorb the schema change.

The genuinely open call for the owner is the **abstraction depth inside
(A)**:

- **A1 (`window_props`)** — Window-only, simplest, but re-opens this DD at
  the next base type.
- **A2a (`host_props` / `host_bindings`, `base: String` kept)** — host-owned
  surface separated from the content root, open to Dialog / Page / Scene; no
  `inherits` semantics added.
- **A2b (`IrBase { name, props, bindings }`)** — the base *owns* attributes
  in the IR; a down-payment on (B); heavier.

I recommend **A2a, with the generic-list representation and a host-attribute
catalog (a Window entry today) shipped alongside**:

- it removes the root-content contamination (unlike D);
- it is open to future host base types (unlike A1);
- it does **not** pull in the base-type system (unlike A2b/B) — `base` stays
  a string, no `inherits` semantics yet;
- the **catalog** is what actually closes the `titlee:` typo hole (the IR
  move alone does not), and choosing it **host-general** keeps IR generality
  and validation generality aligned — an IR that is host-general with a
  Window-only catalog is a latent re-divergence. A full base-type *registry*
  catalog is (B), deferred.

For Phase 6 the catalog is **host-general in shape but holds only the Window
entry.** An attribute on the host surface with no catalog entry is rejected;
since only Window is catalogued this phase, a non-Window base's *host
attributes* have no entries yet and are rejected — but the base *name* itself
is not validated (that is carried to M4 / B), so an empty host surface on a
non-Window base is not rejected. The provisional rule is "`host_props`
accepts only attributes the host catalog knows" rather than "`host_props`
accepts only when `base` is `Window`" — the catalog, not the base name, is
the gate.

(B) is the end-state to grow toward, **owned by M4** (multi-window +
backdrop/theme force base-type modeling there — see the (B) milestone note).
A2a is chosen **not to avoid (B)** but to leave M4 the room to design (B)
correctly while Phase 6 fixes *only* the contamination it actually exposed:
it stops the splice and gives host attributes a home, but it does not choose
the base-type carrier, the ABI-facing window descriptor, or the multi-window
attribute-ownership model — all M4's to settle. A2a is therefore a
**stepping stone, not a final shape**: M4 may promote or re-place it as the
window-entity model takes form, and that is expected, not a failure of A2a.
The **canonical invariant A2a preserves for M4 is not the field name
`host_props`** but the **separation between host-owned attributes and the
content root**; M4 may replace the carrier, but it should preserve that
separation. A2b is the step beyond Phase 6. (D) is a
fallback if an IR migration genuinely cannot land — but the Phase-6 boundary
that would force that is an *assumption*, not a hard constraint (see
Time-box): if (A) is the right design and does not fit the remaining budget,
**revising the phase scope is as legitimate a response as falling back to
(D)**, and it avoids institutionalizing the mis-model. Choosing (D) should be
a deliberate call that (D)'s shape is acceptable, not a default forced by an
assumed deadline.

**Runtime trust boundary — provisional Phase-6 stance (criterion 6).**
Because the runtime still parses textual IR directly, treat the runtime
loader as a **defensive reader of untrusted IR**, not a pure mirror of
`wasamoc` output: under (A) the runtime still validates the new host surface
and still rejects the old root-squatted shape. The catalog's **source of
truth is the compiler** (`wasamoc check`); the runtime carries a **mirror**
for defense-in-depth. This is a provisional default, not a closed decision —
if the project later declares the runtime a trusted-input mirror, the
runtime-side re-validation can shrink — but A2a's acceptance criteria can be
written against this default now rather than blocking on the boundary
question.

**Time-box (a working assumption, not a hard constraint).** The premise that
this is a *Phase-6 responsibility* — Phase 6 introduced both the strict
ZStack validator and window-attribute-on-root (T6), so shipping them mutually
inconsistent or leaking the divergence to M4 is a Phase-6 gap — is itself a
hypothesis, not a fixed boundary. It argues for resolving before Phase 6
closes (T8 fix-container or a dedicated task — see plan.md T7b), and against
a T6 reopen (T6 had no ZStack-root example to exercise the boundary). But if
the right design (e.g. A2a) does not fit the remaining Phase-6 budget, the
correct response is to **revise the assumption by proper means** — re-scope
the phase, carry the structural fix forward with the interim explicitly held,
or split (A) — *not* to downgrade the design to (D) solely to honour an
assumed deadline. The interim is already pinned on both gates precisely so
the divergence can be carried safely if the schedule is revised. (Per the
project's revise-don't-work-around discipline: the plan is a hypothesis;
when it does not fit, revise it rather than bend the design to it.)

The three sections below assume the recommended **A2a** is accepted; they do
not pre-empt the owner's choice. Their roles are distinct from
[If (A) is chosen](#if-a-is-chosen--acceptance-criteria-direction-here-details-in-impl):
that section lists *what to decide at acceptance*; these list the *residual
risk*, the *forward-compat exposure*, and the *implementation order* that
follow from the decision.

## Technical risk re-evaluation (if A2a accepted)

Accepting A2a raises the risk class from a localized runtime / compiler
allowlist fix to an **IR-schema / textual-IR migration**. The risk is
justified — it removes the root contamination rather than institutionalizing
it (the (D) failure mode) — but the migration must be treated as high-risk
and reviewed independently (the schema/IR-migration review tier). The main
risk points:

- `wasamo-ir` schema and the textual-IR parser / emitter must change
  **together** (a half-migrated textual IR fails to round-trip).
- `wasamoc` lowering must **stop** splicing component-level host attributes
  onto `root.props` / `root.bindings` — the splice site that started this DD.
- runtime loading must read the new host surface **and** reject or explicitly
  warn on the old root-squatted shape (no silent dual acceptance).
- the compiler and runtime host-attribute catalogs must stay synchronized
  (the mirror-drift risk — see the mirror-sync acceptance item).
- `resolve_static_window_title` must move to the new surface; any `root.props`
  fallback must be transitional and removal-triggered.
- host bindings need an explicit Phase-6 policy, or `host_bindings` becomes a
  structural promise with no validation semantics.

**Residual risk (intentional):** A2a does *not* complete base-type modeling.
It deliberately leaves `base: String` inert and carries full base-name
validation / window-entity semantics to M4. The only invariant accepted here
is the **separation of host-owned attributes from the content root**.

## Forward-compat exposure (if A2a accepted)

A2a is forward-compatible with one specific invariant — host-owned attributes
are separated from the content root — and does **not** commit M4 to keep the
exact `host_props` / `host_bindings` carrier.

**Opened (kept forward-compatible):**

- the content root no longer receives host attributes, so future strict-root
  widgets need no root-only exemption;
- the surface is host-general, not Window-specific, so `Dialog` / `Page` /
  `Scene` *can* reuse the same conceptual slot once they are catalogued /
  modeled later (the slot is opened, not the host types themselves);
- M4 may promote the surface into a real window/base-entity model without
  preserving the field name.

**Exposed (not yet closed):**

- `base: String` stays semantically inert until M4 / (B);
- base-name validation is not solved this phase;
- non-Window host types stay uncatalogued unless added explicitly;
- host bindings exist structurally only to the extent the accepted
  dynamic-binding policy permits;
- ABI-facing window descriptors / handles remain M4-owned and must **not** be
  inferred from `host_props`.

**Forward-compat rule:** M4 may replace the carrier, but should preserve the
host-attribute / content-root separation.

## Implementation handoff (if A2a accepted)

Implement as a **structural migration**, not a runtime-only ZStack exemption.
Expected order:

1. Add `host_props` / `host_bindings` to `IrComponent` (same generic-list
   representation as widget props / bindings).
2. Update textual-IR emit / parse to round-trip the new surface.
3. Update `wasamoc` lowering so component-level host attributes emit to
   `host_props` / `host_bindings`, **never** spliced onto `root`.
4. Add a host-attribute catalog — host-general in shape, Window-only entry for
   Phase 6 (subject to the acceptance decisions above: catalog lookup key,
   catalog-with-(A)-or-follow-up, dynamic-binding policy).
5. Update `wasamoc check` to validate component-level host attributes through
   the catalog and reject unknown ones (`titlee:`).
6. Update runtime loading to consume the new surface, validate it via the
   runtime mirror of the catalog, and reject / warn on the old root-squatted
   shape.
7. Move `resolve_static_window_title` to the new surface; any `root.props`
   fallback transitional and removal-triggered.
8. Make the new shape **canonical** in tests: compiler rejects unknown host
   attrs; runtime rejects (or deprecation-warns) old root-squatted attrs; the
   ZStack root needs no window-attribute exemption; catalog mirror-drift is
   detected.
9. Sync `docs/dsl_spec.md`, `docs/architecture.md`, and the preamble /
   decision index **after** acceptance (Moment 2).

**Do not** implement this as: "root ZStack accepts more attributes"; a
Window-only `window_props` surface (unless the owner rejects A2a); or
`IrBase` / base-type validation (unless scope is deliberately expanded toward
(B)).

## Interim (currently shipped, pinned by tests)

The divergence is pinned on **both gates** so a future alignment visibly
flips exactly one side, not silently both.

**Compiler (accept) side — `wasamoc`:**

- `zstack_root_component_window_attrs_accepted` — an arbitrary component prop
  (`foo: bar`) and a dynamic `title: <state>` bind pass `wasamoc check` on a
  ZStack root (no component-prop catalog).
- `bind_component_level_no_type_check` (pre-existing) — static `title:` /
  `backdrop:` pass through.

**Runtime (reject) side — `wasamo-runtime`:** the loader rejects outside the
narrow allowlist:

- `nested_zstack_rejects_component_window_prop` — window-prop exemption is
  root-only.
- `root_zstack_rejects_non_window_component_prop` — arbitrary component prop
  on a ZStack root is rejected (the compiler accepts it → the divergence).
- `root_zstack_rejects_placement_prop` — a placement prop on a root ZStack
  is rejected.
- `root_zstack_accepts_component_window_props` /
  `root_zstack_still_rejects_widget_attribute` (T7) — the three-name
  allowlist and the widget-attr rejection it sits beside.
- `root_zstack_rejects_spliced_component_window_binding` — the binding facet
  with the **exact** IR `wasamoc` emits for a dynamic `title:`
  (`bind title = (str-prop-read s)`), verified against `wasamoc build`
  output; `zstack_binding_rejected_at_validate` is the proxy widget-binding
  variant of the same gate.

## Preamble integration

Not indexed in [preamble.md](./preamble.md) §Decisions while `Proposed`
(the preamble records accepted decisions only). On acceptance, add to the
§Decisions index and a Revisions entry recording the mid-phase addition
surfaced by the T7 review, and reconcile plan.md T7b to the chosen option.

## Revision history

- **Proposed (initial draft, 2026-06-07)** — surfaced by the T7 review:
  recorded the component-root window/widget attribute boundary as a dual-gate
  divergence (props + bindings), options A/C/D, and the interim pinned on
  both gates.
- **Proposed (revision, 2026-06-08)** — root-cause deepening: reframed from
  the dual-gate symptom to the **unmodeled base type** (`inherits Window` is
  an inert string), so the component-level namespace is undifferentiated.
- **Proposed (revision, 2026-06-08)** — design-space widening: mapped the
  option spectrum (end-state B; A1 / A2a / A2b; D / C / E / D+), softened the
  Phase-6 time-box to a working assumption, and sharpened the recommendation
  to A2a + a host-attribute catalog. Still `Proposed` — no option selected.
- **Proposed (revision, 2026-06-08)** — M4 ownership + A2b rejection: pinned
  (B) as an **M4** responsibility and hardened A2a over A2b (too light → thin
  gain; too heavy → pre-empts M4) and over A1. Still `Proposed`.
- **Proposed (revision, 2026-06-08)** — added (if-A2a-accepted) Technical risk
  re-evaluation, Forward-compat exposure, and Implementation handoff sections,
  role-separated from the acceptance-criteria list. Still `Proposed`.
