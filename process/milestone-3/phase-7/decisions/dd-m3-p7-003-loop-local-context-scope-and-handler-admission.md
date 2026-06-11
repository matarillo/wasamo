# DD-M3-P7-003 — Loop-local context, scope, and handler admission

**Status:** Proposed
**Phase:** M3-Phase 7
**AC:** A8; A12 (the scope rules are external-reader-bar spec content)

## Context

Iteration introduces the DSL's first **template-local names**: inside a
`for` body, the element (and optionally its index) must be readable so
generated subtrees can differ per item (`label: thumb`). Until
now every dynamic reference in a binding resolved to a component
`state`; FD-D codifies loop locals as the **first explicit exception**
to that rule — a *loop-local read-only binding* — and confines the
exception to **expression (binding) positions**. Whether handler
positions may read loop locals, and whether `for` bodies may carry
handlers at all, is an admission judgment this DD must make explicitly
(owner answers §4: spec silence here would breach the A12 bar).

These names are **not** widget ids and **not** item keys
(dsl-grammar.md Q1 discipline): a binder names a *value in scope during
template instantiation*; it confers no identity on the generated
subtree (identity is DD-M3-P7-005's positional contract) and no
addressable handle on any widget.

## Decision dependency summary

Consumes DD-M3-P7-001's header slots (binder syntax) and DD-M3-P7-002's
value representation (whole-value signal ⇒ binder reads lower to
positional collection reads, DD-M3-P7-005). Owns: naming, scope,
admitted read positions, handler admission. The reject branches land in
DD-M3-P7-007's matrix.

## Sub-issues

- **Binder naming** — author-named vs fixed implicit names.
- **Index exposure** — form and type.
- **Read positions** — where a binder may appear.
- **Scope & collision rules** — flatness, collisions, nesting.
- **Handler admission** — handlers inside `for` bodies; `item` in
  handler position.

## Binder naming

### Options

- **N1 — author-named binders** (`for thumb, i in thumbs`)
  - Every name in scope is written in the source.
  - What you gain: no invisible vocabulary — an external reader of a
    body sees where every identifier comes from (the A12 axis); the
    collision story is ordinary shadow-free naming (you chose the name,
    choose another); a future nested `for` has no forced-shadowing
    problem (inner loops pick fresh names; same-name nesting can simply
    stay an error); the binder reads as what it is — a binding, not a
    keyword.
  - What you give up: marginally longer headers than magic names.

- **N2 — fixed implicit `item` / `index`**
  - The body may use `item` / `index` without declaring them.
  - What you gain: shortest possible body prose; matches the framing's
    placeholder vocabulary.
  - What you give up: the DSL's first magic identifiers — names that
    resolve to something declared nowhere; `item` / `index` become
    de-facto reserved (a state named `item` either breaks every `for`
    body or is shadowed silently — both bad; collision-as-error would
    make a *state declaration* fail because of a *loop* elsewhere,
    action at a distance); nested loops would force a shadowing /
    qualification design **now**, exactly what FD-C defers.
  - Rejected on merit: it front-loads the deferred scope design and
    weakens the read-where-declared property the external-reader bar
    leans on. (The framing's `item` / `index` wording is read as
    placeholder vocabulary for "the element / index binders", not as a
    surface commitment — this DD records that interpretation.)

### Recommendation

**N1 — author-named.** `item` and `index` remain perfectly good — and
likely conventional — *choices* of binder name; they are not keywords
and not implicit.

## Index exposure

**Optional second binder** (`for thumb, i in thumbs`), type **`i32`**,
read-only, zero-based. Grounds: the index is occasionally needed
(numbering, alternation later) but a mandatory binder is dead weight in
the common case (DD-001 H3). No third slot exists. The index is a
*value* (the position at instantiation time under the positional
contract of DD-005), not an identity handle — the Q1 discipline
restated.

Admitting the index binder does not rely on an in-phase numbering /
alternation driver. FD-D / framing thesis 4 already aligns `item` and
`index` as loop-local bindings; the index is read-only, expression-only,
and complete as part of the A12 external-reader surface. The "no driver
this phase" bar used below applies to surfaces that entangle deferred
identity / lifecycle theses, not to this aligned local value binding.

## Read positions

### Options

- **P1 — binding expressions inside the body template only.** A binder
  is readable wherever a binding expression (property binding
  `name: expr`, interpolation parts) is evaluated within the `for`
  body's widget subtree. Not in handler position, not in an `if`
  condition, not in property *literal* position (which is static by
  definition), not outside the body. The `if` condition remains the Phase 6 narrow
  bool-expr whose identifier resolves to a `bool`-typed `state`;
  admitting loop-local bool binders there would widen condition-name
  resolution and make per-item conditional presence a separate
  structural surface.
- **P1a — also in descendant `if` conditions.** A bool element binder
  could drive per-item conditional presence (`for t in flags { Box {
  if t { … } } }`).
  - What you gain: the most natural use of `bool[]` items in structure.
  - What you give up: the condition resolver stops being state-only;
    `ConditionalSubtree` presence would depend on a loop instantiation
    context and compose into the DD-M3-P7-004 expansion seam. That is a
    real per-item presence surface, adjacent to the HA1 per-item
    interaction deferral and the nested-scope family trigger.
- **P2 — also in handler position** — folded into the handler-admission
  judgment below; cannot be decided independently of whether body
  handlers exist at all.

### Recommendation

**P1**, per FD-D's codified exception boundary. Binder reads are
limited to property-binding / interpolation expression positions in the
body widget subtree; descendant `if` members remain admitted only when
their conditions read `bool` state, not loop-local binders. Per-item
conditional presence is a recorded deferral in the framing FD-F
scope table: it reopens on the first concrete UI case needing per-item
display / state branching from `bool` elements, naturally at M4 input
per-item interaction or the next structural control-flow extension,
whichever comes first. Lowering: a binder read
becomes a typed loop-local read in the unified `HandlerExpr`
(`ItemRead { binder }` / `IndexRead { binder }`; one enum, no per-item
side enum — settled premise). `wasamoc` types the read from the
collection's element type (resp. `i32`) and validates that the binder
is in scope; the runtime evaluator resolves it against the
instantiation context — which, per DD-M3-P7-005, is a **reactive
positional read of the collection signal** (element `i` of the
whole-value signal), so prefix items stay live-bound rather than
frozen copies. A binder read outside any `for` body, or naming an
undeclared binder, is a `wasamoc check` error.

## Scope & collision rules

- **Flat scope.** Binders are visible from the `{` to the matching `}`
  of their `for` body, within binding-expression positions only.
- **Collision = error** (`wasamoc check`):
  - binder name = any declared `state` name (collection or scalar);
  - element binder = index binder in the same header;
  - binder name = a reserved keyword (free with the lexer).
- **No shadowing anywhere** — nothing can nest this phase (DD-001
  rejects `for`-in-`for` at any template depth), so no shadowing rule
  is shipped, and that absence is *stated* in the spec rather than
  left implicit (the A12 bar: an external reader must know nesting is
  rejected-by-design, not undefined).
- **Nested template scope / shadowing** — deferred to the phase that
  opens the next structural control-flow extension (framing 正本
  trigger), where `else` / `switch` / bare-nesting and scope rules are
  designed as one piece.

## Handler admission (explicit judgment)

May a widget inside a `for` body template carry `signal_handler`
members — and if so, may handler expressions read the binders?

### Options

- **HA1 — reject handlers inside `for` bodies this phase.**
  - Any `signal_handler` member within a `for` body template is a
    `wasamoc check` error naming the deferral.
  - What you gain: no half-surface — admitting handlers *without*
    binder reads (HA2) ships per-item widgets whose handlers can only
    mutate global state, an asymmetry the spec would have to explain
    away ("you can bind `thumb` to a label but not react to it") and
    authors would hit immediately; per-item interaction has **no
    driver this phase** (FD-B fixes the proof mutation outside the
    body precisely so the proof doesn't depend on it) and a real
    driver arriving with M4 input (select-this-item, delete-this-item)
    — at which point handler-position binder reads, per-item handler
    registration lifecycles, and possibly keyed identity are designed
    together against an actual use case.
  - What you give up: a `for`-generated Button is unusable this phase
    even for global mutations.

- **HA2 — admit handlers, binders unreadable in them.**
  - What you gain: N copies of globally-acting widgets.
  - What you give up: the asymmetry above, registered N times with no
    way to tell instances apart — a surface that *invites* the
    question this phase explicitly does not answer, and whose
    semantics keyed-identity work may want to revisit. Shipping it
    normatively (A12) would freeze a half-contract.

- **HA3 — admit handlers with binder reads.**
  - What you gain: the future per-item interaction surface is admitted
    now; M4 select-this-item / delete-this-item work can consume an
    already-normative handler-position loop-read contract instead of
    reopening handler admission.
  - What you give up: this *is* the per-item interaction surface —
    handler-position loop reads widen the property-binding /
    interpolation-only surface at the open question FD-D delegated here, and the
    value-vs-live-position question (does the handler see the index at
    creation or at click time?) is exactly the identity question
    deferred with keyed retention. HA3 is within this DD's delegated
    admission space, but its real driver arrives with M4 input; deciding
    it there lets the handler-read contract, registration lifecycle, and
    identity posture be designed against the use case that actually
    needs them.

### Recommendation

**HA1 — reject, with the recorded trigger** (framing 正本 row
"per-item handler / handler 内 `item` 参照"): per-item interaction UI
(select / delete-this-item), arriving naturally with M4 input ⇒ the
admission DD designs handler-position reads, registration lifecycle,
and their identity interaction together. This is a merit deferral inside
the owner-delegated admission judgment, not a claim that HA3 is outside
the framing boundary. The reject diagnostic names the deferral
explicitly (a *recorded* deferral, not a silent gap — same posture as
the Phase 6 operator-condition reject).

## Spec content seed

The iteration chapter's scope section states normatively: binders are
author-named, read-only, property-binding / interpolation-position-only;
their types (element type / `i32`); the flat-scope visibility window;
the collision errors;
the no-nesting and no-handler rules **as designed rejections with named
triggers**; the `if` condition boundary (`cond_expr` identifiers
resolve to state only, not loop-local binders); and the Q1 boundary
sentence — *a binder is not a widget id and not an item key; generated
subtrees have positional identity (§identity baseline)*. Invalid
examples: state-collision, undeclared binder, binder in `if` condition,
binder in handler, handler in body, nested `for`.

## Forward-compat exposure

- **Per-item handlers + handler-position reads** — the HA1 trigger;
  lands as an admission widening, no grammar reshaping (handlers are
  already `member`s; the reject simply lifts).
- **Nested scope / shadowing** — with the family-extension phase; N1
  means the design space is open (no magic-name shadowing debt).
- **Author-facing `key:`** — explicitly *not* a binder concern; lands
  with keyed identity (DD-005 forward-compat) as a widget-id-distinct
  surface per Q1.

## Revision history

- Strategic owner-alignment review fold: fairly stated HA3 and moved
  its reject ground from framing-boundary exclusion to M4/identity
  sequencing; clarified index-binder admission despite no in-phase
  numbering driver; status remains Proposed.
- Recommendation-choice review fold: clarified that loop-local binders
  are not readable in `if` conditions and recorded per-item conditional
  presence as a framing FD-F 正本 deferral; tightened read-position
  wording; status remains Proposed.

## Technical risk re-evaluation

- **`HandlerExpr` widening** (`ItemRead` / `IndexRead`) touches the
  shared enum consumed by `wasamoc` lowering, the textual-IR
  emitter/loader, and the runtime evaluator — a trap-#1 call-site
  audit accompanies it (every `match` on `HandlerExpr` is enumerated;
  compile-error-forcing because the enum is non-exhaustive nowhere).
- **Scope checking is pure logic** — binder resolution and the
  collision matrix are `wasamoc check` unit-test material, no OS
  dependency.
- **The evaluator's instantiation context** is new runtime state
  (which loop instance an effect belongs to); its design lands in
  DD-005's positional-read contract, and the risk (stale index after
  tail mutation) is owned there.
- Every reject branch added here gets a direct failure-path test
  (trap #4).
