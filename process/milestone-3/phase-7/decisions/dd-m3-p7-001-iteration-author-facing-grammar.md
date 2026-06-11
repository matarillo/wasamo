# DD-M3-P7-001 — Iteration author-facing grammar surface

**Status:** Proposed
**Phase:** M3-Phase 7
**AC:** A8 (iteration grammar — collection binding drives widget-tree
generation)

## Context

This DD fixes the `.ui` surface: *how an author writes "generate one
subtree per element of this collection."* The IR encoding is
DD-M3-P7-004; the loop-local binder semantics are DD-M3-P7-003; the
collection value surface is DD-M3-P7-002.

The decisive precedent is Phase 6's DD-M3-P6-003: conditional rendering
chose the **`if`-block member** (G1) precisely because `for` was named
as a sibling block keyword of the same structural control-flow family —
`for` is already a reserved keyword (dsl_spec §2.1), reserved in Phase 6
*for this phase*. The contextual token `in` was deliberately **not**
reserved then, because its production did not exist; this DD specifies
the production, so the reservation lands now.

The thesis constraint (FD-A): the surface must read as **structure**
(a member that expands to 0..N children), not as a property of a
widget, and must be the shape a future host-language loop (approach 3)
could lower into.

## Decision dependency summary

This DD owns the **iteration body shape** bundle (preamble §Cross-DD
decision dependencies): choosing one-widget-child-per-iteration selects
DD-M3-P7-004's length-1 body template, DD-M3-P7-005's per-item subtree
grain, and DD-M3-P7-007's body-shape rejects. The binder *syntax slot*
is fixed here; binder *semantics* (naming, scope, read positions) are
DD-M3-P7-003.

## Sub-issues

- **Surface form** — block member vs attribute vs directive.
- **Header shape** — binder slots, `in` reservation, collection
  expression position.
- **Body cardinality** — one widget child per iteration vs member range.
- **Per-container direct-`for` admission sweep** — where a `for` member
  is admitted.
- **Nested control flow** — what may appear inside the body template.

## Surface form

### Options

- **F1 — `for`-block member**
  - ```
    for thumb in thumbs {
        Box { … }
    }
    ```
    A `for` block is a new `member` alternative; per element of the
    collection, the body instantiates as a child of the enclosing
    container at the block's document position.
  - What you gain: the exact family shape DD-M3-P6-003 reserved `for`
    for — `if` and `for` read as two members of one structural grammar;
    the A12 chapter continues the structural-rendering-model story
    rather than opening a new one; high approach-3 reachability (a host
    `for` lowers into the same block shape).
  - What you give up: nothing relative to the family premise; the block
    costs one wrapper for trivial cases, same as `if`.

- **F2 — repetition attribute on a widget** (`Box { repeat: thumbs; … }`)
  - A reserved attribute multiplying the widget it sits on.
  - What you gain: terse single-widget form.
  - What you give up: reads as a **property** and revives exactly the
    approach-1 drift G2 was rejected for in Phase 6; no place to hang
    the binder names (`thumb`, index) without inventing further
    attribute micro-syntax; breaks family symmetry with the shipped
    `if` block — the spec would have to explain why one structural
    construct is a block and the other an attribute.
  - Rejected on merit: it contradicts the structural-family thesis the
    owner fixed in Phase 6 and re-confirmed in FD-A.

- **F3 — container-driven generation** (`WrapPanel { items: thumbs;
  template: Box { … } }`)
  - The *container* owns iteration via `items:`/`template:` properties.
  - What you gain: resembles XAML `ItemsControl`; no new member kind.
  - What you give up: iteration becomes a per-container feature instead
    of a grammar construct — every container needs its own
    items/template surface, mixing static children with generated ones
    needs ad-hoc rules, and `if`/`for` stop being siblings. It answers
    a widget-catalog question, not the A8 grammar question.
  - Rejected on merit: A8 demands an iteration *grammar*; binding
    iteration to containers forfeits the family and the uniform
    mixed-members model (`Text … for … Text …`) that document-order
    expansion gives for free.

### Recommendation

**F1 — `for`-block member.** Same family, same lowering posture, same
spec story as `if`. F2 / F3 are rejected on thesis merit, not cost.

## Header shape

### Options

- **H1 — `for <binder> in <collection> { … }`, optional index via
  comma:** `for thumb, i in thumbs { … }`
  - One mandatory author-named element binder; an optional second
    author-named index binder; the collection position holds an
    identifier resolving to a collection-typed `state`.
  - What you gain: explicit names (no invisible vocabulary entering
    scope); the index is opt-in so the common case stays minimal; the
    comma form has precedent in mainstream languages (Go, Swift
    `enumerated()` analogues) and needs no new tokens beyond `in`.
  - What you give up: two binder slots to specify and validate.

- **H2 — implicit fixed names** (`for in thumbs { … }` or
  `for thumbs { … }` with `item` / `index` magic names)
  - What you gain: shortest header.
  - What you give up: injects names into scope that appear nowhere in
    the source — the first magic identifiers in the DSL; collision
    rules become spooky (a state named `item` breaks bodies at a
    distance); nested loops (future) would force shadowing semantics
    immediately. DD-M3-P7-003 carries the full comparison; the header
    consequence is recorded here.

- **H3 — index always bound** (`for thumb, i in thumbs` mandatory)
  - What you give up: forces a dead binder in the majority case;
    invites `_`-style conventions the DSL doesn't have.

### Recommendation

**H1.** Header grammar:

```
iteration_member ::= "for" IDENT ("," IDENT)? "in" IDENT
                     "{" iteration_body "}"
```

- The first `IDENT` is the **element binder**, the optional second the
  **index binder** — both author-named, semantics in DD-M3-P7-003.
- The post-`in` `IDENT` must resolve to a **collection-typed `state`**
  (DD-M3-P7-002). General collection *expressions* (literals, slices,
  computed collections) are not admitted this phase — the expression
  grammar has no operators, and a collection-literal-in-place has no
  driver; the position widens with the uniform Q5 expression extension.
- The post-`in` state reference is intentionally **bare state name
  only**, not a qualified name. New Phase 7 collection-reference
  positions use local component state by name; cross-component or
  `root.`-qualified collection references are deferred to the same
  uniform expression/reference expansion that would also govern
  collection reads outside loop headers (DD-M3-P7-002). This records
  the reference-shape boundary rather than letting the grammar spelling
  decide it implicitly.
- **`in` becomes a reserved keyword** (dsl_spec §2.1). Phase 6
  explicitly deferred this reservation until the production existed;
  it now does because the header needs a non-ambiguous separator token
  between binder slots and the collection reference. Source-compat: no
  shipped `.ui` uses `in` as an
  identifier (greppable); the existing `in-out` property token is a
  distinct hyphenated lexeme and is unaffected.

## Body cardinality

### Options

- **B1 — exactly one widget child per iteration**
  - `iteration_body ::= widget_decl`. Each element materialises exactly
    one subtree; N elements ⇒ N children.
  - What you gain: the materialised-children count **equals** the
    collection length — the cardinality contract A8 is proven on is
    directly observable; the runtime range math stays per-item
    single-subtree (DD-004 / DD-005); exact symmetry with the shipped
    `if` body rule (B1 of DD-M3-P6-003), so the spec states one body
    discipline for the whole family; multi-widget items wrap in a
    container, which a thumbnail cell wants anyway (Box + caption ⇒
    VStack).
  - What you give up: a per-item wrapper for multi-widget items.

- **B2 — member range per iteration** (`iteration_body ::= member*` of
  structural members)
  - What you gain: wrapper-free multi-widget items.
  - What you give up: cardinality becomes N × k with per-item ranges —
    slot math, disposal, and Visual-order bookkeeping all generalise a
    second time in the same phase that first generalises 0/1 → 0..N;
    and the family becomes asymmetric (an `if` body admits one child, a
    `for` body a range) unless `if` is widened simultaneously.
  - Not chosen: it solves a problem the thesis doesn't pose this phase;
    it lands later as the body generalisation on the canonized
    expansion seam (DD-004 forward-compat) for `if` and `for`
    together. Deferred, not rejected.

### Recommendation

**B1 — one `widget_decl` per iteration.** Non-structural members
(property / bind / handler / `state` / track-list directly in the
body), multiple children, and a bare nested control-flow member as the
immediate body are rejected at `wasamoc check` and re-checked at the
loader — the same strict-body discipline as the `if` body, extended
with the iteration-specific rules below.

## Per-container direct-`for` admission sweep

A `for` member is grammatically a `member`, but containers with
cardinality contracts cannot absorb a dynamically-0..N member. The
sweep (the Phase 6 DD-M3-P6-007 question, asked once per container up
front instead of surfacing mid-phase):

| Container | Direct `for` child | Reason |
|---|---|---|
| VStack / HStack / WrapPanel | **admitted** | arbitrary-children contract; no per-child parent metadata |
| ZStack | **admitted** | arbitrary-children contract; per-child placement handled by DD-M3-P7-006 (child-carried) |
| ScrollView | **rejected** | exactly-one-content-child contract (DD-M3-P4-001); symmetric with the direct-conditional reject (DD-M3-P6-007) — the `for` wraps inside the single content widget (`ScrollView { WrapPanel { for … } }`), which is also the gallery shape |
| Box | **rejected** | at-most-one-child contract; a `for` can produce > 1 |
| Grid | **rejected** | children are `Cell`-mediated; a `for` of `Cell`s couples iteration to per-cell placement metadata — deferred with the DD-006 Grid trigger |
| component level | **rejected** | no parent slot for a 0..N root; same ground as the component-level `if` reject |

Each reject is a named `wasamoc check` diagnostic + loader re-check
(DD-M3-P7-007); each is a *recorded deferral or contract statement*,
not a silent gap.

## Nested control flow

- **Admitted inside the body's widget subtree:** descendant `if`
  members (`for t in thumbs { Box { if flag { … } } }`) — `if` adds no
  scope, and its presence math composes through the canonized expansion
  seam (DD-004). This admission covers the existing Phase 6 condition
  surface: the condition identifier resolves to `bool` state. A
  loop-local binder in that condition is not admitted by this grammar
  decision; DD-M3-P7-003 records it as a read-position deferral.
- **Rejected this phase:** a `for` member **anywhere inside a `for`
  body template** — not only as the immediate body but at any depth
  (`for a in xs { VStack { for b in ys { … } } }` is rejected). This is
  deliberately stricter than the `if` precedent (which admitted
  wrapped descendants): a descendant `for` introduces **nested
  template scope** (outer binders visible inside the inner template),
  and FD-C defers scope nesting / shadowing to the phase that opens
  the next structural control-flow extension. A `for` nested inside an
  `if` body's widget subtree is admitted when that `if` is not itself
  inside a `for` template (no scope nesting arises).
- **Bare nested control flow as the immediate body** (`for t in xs {
  if c { … } }`): rejected, same wrap rule as Phase 6.

## Spec content seed

Grammar addition (dsl_spec §3 + the iteration chapter):

```
member            ::= property_decl | property_bind | widget_decl
                   |  signal_handler | state_decl
                   |  grid_track_list_member
                   |  conditional_member             ; M3-Phase 6
                   |  iteration_member               ; M3-Phase 7

iteration_member  ::= "for" IDENT ("," IDENT)? "in" IDENT
                      "{" iteration_body "}"
                      ; binder (, index-binder)? in collection-state

iteration_body    ::= widget_decl                    ; exactly one widget
                                                     ; child per iteration
```

§2.1: `in` joins the reserved keywords (with the note that `for` —
reserved since Phase 6 — now has its production). The chapter is
written as the **second chapter of the structural rendering model**:
`if` drives presence (0/1), `for` drives cardinality (0..N), same
member-level family, same expansion model; `else` / `switch` and the
body-range generalisation are named as future same-family members.

## Forward-compat exposure

- **Member-range body** — widens `iteration_body` (and the `if` body)
  on the canonized expansion seam; no `for`-header or IR-member shape
  change (DD-004).
- **Nested `for` / template scope** — lands with the family extension
  phase together with the scope rules (DD-003 forward-compat).
- **Collection expressions after `in`** — the position widens with the
  uniform Q5 expression-grammar extension; the header shape is
  unaffected.
- **`for` of `Cell` under Grid** — opens with the DD-006 Grid
  placement-migration trigger.
- **Approach 3** — a host-language loop lowers into the same
  member-level `For` IR; the block surface keeps that path open.

## Strategic review disposition

- **Review F2 folded.** The header recommendation now records the
  bare-state reference boundary and its relationship to the existing
  qualified-name surfaces; no recommendation change.

## Revision history

- Strategic owner-alignment review fold: clarified the post-`in`
  reference-shape boundary and the reservation rationale; status remains
  Proposed.
- Recommendation-choice review fold: clarified that descendant `if`
  admission under a `for` template does not admit loop-local binder
  reads in the `if` condition; status remains Proposed.

## Technical risk re-evaluation

- **Parser:** one new `member` arm keyed on the already-reserved `for`
  token — no lookahead ambiguity (mirror of the Phase 6 `if` arm). The
  comma-optional second binder is LL(1) after the first `IDENT`.
- **`in` reservation** is the widest lexical change: `in` becomes
  unusable as a state / property / widget / binder name. Greppable
  zero usage today; `in-out` is lexed as its own hyphenated token and
  is unaffected — a regression test pins this.
- **Admission sweep** adds five reject branches; each is a pure-logic
  `wasamoc check` test plus a loader re-check test (trap #4).
- The grammar carries **no expression-grammar change** — the Q5
  deferral stays intact; the collection position is an `IDENT` only.
