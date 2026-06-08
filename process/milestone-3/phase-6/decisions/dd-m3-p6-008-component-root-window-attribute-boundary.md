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

## Sub-issue

There is no owner for **window-level attributes** because the base type
(`Window`) is not a modeled entity. They are neither given a structural home
in the IR (no `IrComponent` attribute surface — they ride the content
`root`) nor validated against a Window-attribute catalog (no gate has one).
The "dual-gate divergence" is the visible symptom of that missing owner.

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

## Options

- **(A) IR-schema separation — give the window a structural attribute
  surface.** Stop splicing component-level attributes onto the root node;
  carry them on a dedicated `IrComponent` surface (e.g. `window_props` /
  `window_bindings`). This addresses the root cause directly: `IrComponent`
  is already the IR's window node (it is what `wasamo_load_ui` →
  `window::create` consumes), but today it models only the window's
  *content* (`root`); (A) gives it the window's *attribute* surface too, so
  Window-ness stops being a name-only `base` string and becomes structural
  for attributes. Widget validators never see window attributes (the content
  root is just content again); `resolve_static_window_title` reads the
  dedicated surface.
  - Gain: the root cause (no structural home for window attributes) is
    removed at the source; **both** props and bindings are handled
    uniformly; provenance is preserved; a Window-attribute catalog gets a
    natural home (closes the typo hole); future strict-root widgets need no
    special-casing.
  - Give up: an **IR schema + textual-IR format change** (the
    schema/IR-migration high-risk category — full independent review) across
    `wasamo-ir` + `wasamoc` emit/lower + the runtime parser + the test
    corpus, plus a `docs/dsl_spec.md` / `docs/architecture.md` Moment-2
    sync. Precedented this phase by the T4 `Vec<IrMember>` migration, but
    larger than a localized fix.

- **(C) Align the runtime to the compiler (root accepts anything).** Make
  the ZStack validator treat *any* prop on the **root** as a non-widget
  (window) attribute and not reject it.
  - Gain: minimal change; the divergence disappears.
  - Give up: **wrong direction** — window attributes stay unvalidated
    *everywhere* (the `titlee:` typo is accepted at every gate), and the root
    ZStack loses its junk-attribute guard. Symmetry bought by deleting a
    check rather than by giving window attributes an owner. Not recommended.

- **(D) Compiler-owned catalog, runtime mirrors it.** Give `wasamoc check`
  a Window-attribute catalog with diagnostics; make the runtime root
  allowlist the **mirror** of that catalog (the established mirror pattern,
  cf. `STAR_WEIGHT_MAX`).
  - Gain: closes the divergence with a *principled* allowlist; closes the
    typo hole at compile time; no IR schema change; fits the R1 / T6 window
    theme. Spec sync is a small dsl_spec addition (the Window-attribute set),
    not a format change.
  - Give up: it teaches the gates to *recognize* window attributes but
    leaves them stored on the wrong node — the root cause (Window is not a
    modeled owner; attributes still ride `root.props`) persists. Consistently
    handled, but two mirrored lists must stay in sync, and the binding facet
    still needs explicit handling (the catalog must cover bind-able window
    attributes, or the binding rejection stays interim until dynamic title
    lands).

## Comparison

(C) is the cheapest but the wrong direction: it achieves symmetry by
abandoning validation, leaving window attributes unowned. (A) is the only
option that removes the root cause (it gives Window attributes a structural
home) and handles props **and** bindings uniformly, at the cost of an IR
migration + Moment-2 spec sync. (D) is the proportionate middle: it closes
the *divergence* with a principled, compile-time-checked allowlist and no
schema change, but it does **not** close the *root cause* — window
attributes still ride the content root, Window is still unmodeled — and it
defers the binding facet with dynamic title.

The choice is genuinely a design call (does Phase 6 model the window's
attribute surface now, or close the divergence proportionately and carry the
unmodeled base?), which is why this is a DD and not a silent T7 patch.

## Recommendation

Owner decision required (A vs D; C rejected). I lean **(A)** because the
splice is a symptom of an unmodeled base type, not merely a divergence to
reconcile: window attributes belong on the window, but Window has no
structural home for them — `base` is an inert string and `IrComponent`
models only the content. (A) is the only option that gives them that home,
closing both facets (props **and** bindings) at the source. (D) makes the
two gates *consistent* but keeps the attributes on the wrong node and leaves
Window unmodeled, so the root cause persists and two mirrored lists must
stay in sync. The T4 `Vec<IrMember>` migration precedent shows Phase 6 can
absorb the IR-schema change. Only if the remaining Phase 6 budget (and the
open teardown-AV residual) argue decisively against an IR migration is
**(D)** the proportionate in-phase close, with the binding facet carried
until dynamic title (FD-D).

Scope note for (A): Phase 6's (A) is the minimal structural fix — a
`window_props` / `window_bindings` surface on `IrComponent`. Fully modeling
`Window` as a validated type (base-type validation, a first-class
Window-attribute catalog, `inherits` semantics) is a larger design that this
DD does not require and that realistically lands later (M4+); (A) is the
first step that makes window-ness structural rather than decorative.

**Time-box:** resolve **before Phase 6 closes** (T8 fix-container or a
dedicated conditional task — see plan.md T7b). Phase 6 introduces *both* the
strict ZStack validator and window-attribute-on-root (T6); shipping them
mutually inconsistent, or letting the divergence leak to M4, is a Phase-6
responsibility gap. The realistic home is T7-surfaced / Phase-6-close — not
a T6 reopen, because T6 had no ZStack-root example to exercise the boundary.

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

- **Proposed** — surfaced by the T7 review (2026-06-07): the ZStack-root
  gallery exposed the component-root window-attribute / widget-attribute
  boundary as a dual-gate divergence (props + bindings); options A/C/D
  recorded; interim behavior pinned by runtime reject tests.
- **Proposed (root-cause deepened)** — T7b deliberation (2026-06-08):
  traced the splice to its origin (introduced incidentally in the M2-Phase 6
  lowering pass, `08daf6a`, with no DD deciding it) and reframed the root
  cause as the **unmodeled base type** — `inherits Window` is an inert
  string (`IrComponent.base` is never semantically consumed), so the IR
  conflates what the component *is* (a Window with an attribute surface) with
  what it *contains* (the `root` widget) and gives window attributes no
  structural home. Lowering has the positional context to distinguish them
  but discards it for lack of a typed destination. Options (A)/(D)
  re-characterized against this root cause; (A) scope-noted as the minimal
  structural step toward modeling the window. Still `Proposed` — no option
  selected.
