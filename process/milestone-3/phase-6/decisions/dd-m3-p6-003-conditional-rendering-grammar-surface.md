# DD-M3-P6-003 — Conditional rendering author-facing grammar surface

**Status:** Accepted
**Phase:** M3-Phase 6
**AC:** A7 (conditional rendering grammar — binding drives the present /
absent state of a subtree)

## Context

A7 is **the first M3 grammar surface where a binding drives widget-tree
structure**, not a property value. This DD fixes the `.ui` surface:
*how an author writes "this subtree is present when this `bool` is
true."* The IR encoding and runtime present/absent mechanism are
DD-M3-P6-004; the effect lifecycle is DD-M3-P6-005. This DD is bound by
the framing's structural-rendering thesis (FD-CR, originating in
[../../../../docs/notes/dsl-grammar.md Q6](../../../../docs/notes/dsl-grammar.md)),
which the framing made the cross-cutting lens.

The thesis fixes three things this DD must honour:

1. **Approach 2** (template + dedicated structural syntax) is the v1
   surface — **not** approach 1 (property toggling of an always-built
   tree) and **not** approach 3 (host-language `if`/`switch`).
2. The construct is the **first member of a structural control-flow
   grammar family**; its shape must let `else` / `switch` / `for`
   (Phase 7 iteration) arrive as same-family members without a grammar
   redesign.
3. **approach 3 must not be foreclosed** — the surface chosen here
   should not paint the IR / runtime into a corner that a future
   language-construct-level surface could not also lower into.

Relevant grammar end-state. The `.ui` `member` rule is:

```
member ::= property_decl | property_bind | widget_decl
         | signal_handler | state_decl | grid_track_list_member
```

and the `expr` rule (RHS / condition position vocabulary) is:

```
expr ::= STRING_LIT | number_with_unit | BOOL_LIT | RATIO_LIT
       | COLOR_LIT | IDENT
```

— **no operators**. `Button.enabled: <bool-expr>` (M3-Phase 1) already
admits exactly a `BOOL_LIT` (`true`/`false`) or an `IDENT` resolving to
a `bool`-typed `state`; `!ready` / comparison / logical operators were
deliberately deferred to a future expression-grammar extension (Q5).

## Decision dependency summary

Most of this DD's sub-issues are local to the author surface (Approach,
Surface form, Condition vocabulary). One is a **cross-DD bundle this DD
owns** — **Conditional body shape** — because the author-surface choice
about what an `if` body admits propagates into the runtime and lifecycle
DDs (full phase map: preamble §Cross-DD decision dependencies):

- **Primary (here):** `if` body cardinality + admitted member kinds —
  **B1** single widget child (nested `if` deferred) / **B2** sibling
  range / **B3** arbitrary `member*`.
- **Runtime consequence — DD-M3-P6-004:** insertion granularity
  (**IG-1** single-slot vs **IG-2** child-range); B1 ⇒ IG-1, B2 ⇒ IG-2.
- **Lifecycle consequence — DD-M3-P6-005:** disposal grain (one subtree
  destroy/rebuild vs a range of subtrees).
- **Evidence consequence — preamble §verification closure:**
  single-child cases vs multi-child order/disposal cases.

Recommended bundle: **B1 + IG-1 + subtree-grain lifecycle + strict-body
diagnostics**. B3 is rejected as a *bundle* — not just locally — because
it forces DD-003/004/005 to define conditional property/state/handler
semantics with no Phase-6 driver. The other sub-issues (Approach,
Surface form, Condition vocabulary) do not couple out of this DD.

## Sub-issues

- **Approach** (the thesis-level choice): property control vs dedicated
  structural syntax vs host-language constructs — **settled by FD-CR**,
  recorded here because A12 requires the spec chapter to present it.
- **Surface form within approach 2**: the concrete `.ui` syntax for the
  structural directive.
- **Conditional body shape**: what an `if` body admits — its cardinality
  (one child vs many) and which member kinds are allowed inside it.
- **Condition expression vocabulary**: what expression the condition
  position admits, and whether operators grow here.

## Approach

The thesis-level choice of *how* structural conditionality is
expressed. This DD records the three-approach comparison because A12
requires the spec chapter to present it (so an external reader
understands *why* Wasamo uses a structural directive rather than a
`visible` property), but the choice itself is **settled framing**
(FD-CR), not re-litigated here.

### Options

- **Approach 1 — property control** (`visible:`/`enabled:` on an
  always-built subtree)
  - The subtree is always built; a property toggles whether it shows.
  - What you gain: reuses the existing property-binding machinery; no
    new structural grammar.
  - What you give up: proves *property toggling*, not structural
    present/absent — it is exactly the model A7 exists to move past
    (**rejected by FD-CR**).

- **Approach 2 — template + dedicated structural syntax**
  - A dedicated structural directive makes a subtree genuinely present
    or absent. The sub-options below are *within* this approach.
  - What you gain: genuine structural present/absent; a surface that can
    host the `if` → `else` / `switch` / `for` family.
  - What you give up: a new grammar surface to design and specify
    (**adopted by FD-CR**).

- **Approach 3 — host-language constructs** (`if`/`switch` embedded in a
  host language)
  - Conditionality is expressed in a host programming language that
    lowers into the runtime.
  - What you gain: maximal expressiveness (full language control flow).
  - What you give up: far larger surface than M3 needs; not the v1
    choice — but it **must stay reachable** later (thesis requirement
    3), which constrains the IR/runtime (DD-M3-P6-004), not this DD's
    surface.

### Recommendation

**Approach 2** (settled by FD-CR). The v1 surface is a template plus a
dedicated structural directive. Approach 1 is rejected (it proves the
wrong thing); approach 3 is not v1 but is kept reachable by the
approach-neutral member-level IR (DD-M3-P6-004). The three-approach
record is retained for the A12 spec chapter.

## Surface form within approach 2

### Options

- **G1 — `if`-block member**
  - ```
    if <bool-expr> {
        <member>*
    }
    ```
    An `if` block is a new `member` alternative; the members inside it
    become conditional children of the enclosing container.
  - What you gain: `else { }` chains the block, and `switch` / `for` are
    sibling block keywords — exactly the control-flow family the thesis
    names; **high approach-3 reachability** (an `if` block is the shape
    a host-language `if` lowers to); it reads as **structure**; the A12
    spec story ("`if` is the first control-flow construct") reads
    cleanly as a family chapter.
  - What you give up: wrapping a single widget costs one extra block
    (minor ergonomic vs an inline attribute).

- **G2 — `when:` structural attribute**
  - ```
    Box { when: is_open; … }      // the Box is present iff is_open
    ```
    A reserved attribute on a widget that gates that widget's presence.
  - What you gain: terse for the single-widget case (inline, no wrapper).
  - What you give up: **no extensibility path to `else` / `switch`** —
    you cannot write `else` for an attribute, and `for` cannot be an
    attribute; it reads as a **property** and nudges authors back toward
    approach-1 thinking; a multi-widget subtree needs a wrapper widget
    to group; it undercuts the structural-family thesis (A12).

- **G3 — structural directive token** (e.g. `@if(is_open) { … }` /
  `#if is_open { … }`)
  - A sigil-prefixed directive distinct from both ordinary members and
    attributes.
  - What you gain: the same structural feel as G1.
  - What you give up: introduces a sigil token class that then has to
    justify `@else` / `@for` siblings (reads as ad-hoc); it is G1 with
    extra lexical ceremony.

### Comparison

| Axis | G1 (`if` block) | G2 (`when:` attr) | G3 (directive token) |
|---|---|---|---|
| Family extensibility (`else`/`switch`/`for`) | **natural** — `else { }` chains the block; `switch`/`for` are sibling block keywords; this is exactly the control-flow family the thesis names | poor — an attribute has no place to hang `else`; `switch` over attributes is unnatural; `for` cannot be an attribute | possible but the sigil family (`@else`?) reads as ad-hoc |
| Approach-3 reachability | high — an `if` block is the same shape a host-language `if` lowers to; the member-level control-flow IR (DD-004) is approach-neutral | low — `when:` is a property-flavoured surface, closer to approach 1 in feel | medium |
| Structural vs property feel | reads as **structure** (a block that exists or not) | reads as a **property** of a widget (drifts toward approach 1 framing) | reads as structure |
| Grammar cost | one new `member` alternative + a block | a reserved attribute name + per-widget gating semantics | a new token class / sigil + directive grammar |
| Single-widget ergonomics | wrap one widget in `if { }` (one extra block) | terse (`when:` inline) | sigil inline |
| Multi-widget subtree | natural (block holds many members) | needs a wrapper widget to group | natural |
| Spec story (A12) | "`if` is the first control-flow construct" reads cleanly as a family chapter | "a presence attribute" undercuts the structural-family thesis | sigils need their own justification |

G2's terseness for the single-widget case is real, but it is exactly
the case the thesis warns against: `when:` *looks like* a property and
nudges authors back toward approach-1 thinking ("set this widget's
presence flag"), and it has **no extensibility path to `else` /
`switch`** — you cannot write `else` for an attribute. G3 buys the same
structural feel as G1 but introduces a sigil token class that then has
to justify `@else` / `@for` siblings; it is G1 with extra lexical
ceremony. G1 is the only form where the family the thesis demands
(`if` → `else` → `switch` → `for`) reads as one coherent grammar, and
where the surface matches what a future approach-3 host `if` would
lower into (so approach 3 stays reachable — thesis requirement 3).

### Recommendation

**G1 (`if`-block member).** Concrete decisions:

- **`if` is a `member` alternative, admitted inside a widget body's
  `member*` only.** The members inside the block are the **conditional
  subtree**: when the condition is true they are present as children of
  the enclosing widget at the block's document position; when false
  they are absent (DD-M3-P6-004 for the structural mechanism). An `if`
  block in a non-member position is a `wasamoc check` / parse error.
  - **Component-level `if` (conditional / multiple root) is out of
    scope for Phase 6** and rejected at `wasamoc check`. A component
    body resolves to the single root (`IrComponent.root: IrNode`); an
    `if` there would gate or multiply the root, but DD-M3-P6-004's
    runtime mechanism (`BindingTarget::ConditionalSubtree { parent:
    WidgetId, slot }`) requires a **parent** to insert/remove the
    subtree into, and the IR holds exactly one root. Conditional-root
    replacement (a nullable / multiplexed root) is a distinct design
    this DD group does not settle, so Phase 6 admits `if` only where a
    parent widget exists. The lightbox `if` sits inside the root
    container, not at component level — the in-scope shape.
- **The `if` body admits a single widget child (recommended B1) —
  see the Conditional body shape sub-issue below.** The body's
  cardinality and admitted member kinds are decided there; the runtime
  insertion granularity that pairs with it is DD-M3-P6-004's
  insertion-granularity sub-issue.
- **The structural control-flow family keywords are reserved now, not
  just `if`.** Phase 6 adds **`if`, `else`, `switch`, `for`** to the
  keyword set (dsl_spec §2.1), all four reserved by the lexer so they
  may not appear as `state` / `property` / widget-type / qualified-name
  identifiers — mirroring the `true`/`false` reservation (DD-M3-P1-002)
  and the Q5 reserved-word rule. Using any of them in identifier
  position is a `wasamoc check` / parse error.
  - **Why reserve the whole family now (owner decision).** FD-CR makes
    the structural control-flow family (`if` → `else` / `switch` → `for`)
    a Phase-6 *design premise*, not a maybe. Reserving only `if` and
    adding `else` / `switch` / `for` later would **source-break** any
    `.ui` that meanwhile used them as identifiers; reserving the
    committed family up front pays the (one-time, today-zero) cost now
    so the family lands additively without a future break. This is the
    keyword counterpart of the member-level IR being shaped for the
    family in DD-M3-P6-004.
  - **`else if` is not a keyword** — it is `else` followed by `if`
    (two reserved keywords), no separate token.
  - **Source-compat note:** no shipped `.ui` uses `if` / `else` /
    `switch` / `for` as an identifier (greppable), so the reservation
    breaks nothing today; it is recorded as a forward-compat note in
    dsl_spec §2.1.
  - **Scope of the reservation: family *block* keywords only.**
    Contextual sub-tokens of not-yet-designed productions — `in`
    (`for item in items`), `case` / `default` (`switch` arms) — are
    **not** reserved this phase, because their grammar role is not yet
    fixed (the `for` iteration syntax is Phase 7; `switch` arm syntax is
    undrawn), and reserving a contextual token ahead of its production
    would freeze syntax we have not designed. They are reserved when
    their production is specified.
- **`if true { … }` / `if false { … }`** are well-typed but
  degenerate (always present / always absent); permitted, not
  special-cased.
- **No `else` this phase.** `else` / `else if` / `switch` / `for` are
  reserved keywords (above) but have **no production yet**; a bare
  `else` / `switch` / `for` block is a parse error with a "reserved /
  not yet supported" diagnostic that names the construct (distinct from
  the identifier-position rejection, which fires when one is used as a
  name).

A7 visible proof uses the single-`bool` form
`if is_lightbox_open { ZStack { … } }`, driven by a text-Button click
handler (FD-B / FD-C).

## Conditional body shape

Having chosen the `if`-block surface (Surface form), a separable
question is **what the block body admits** — its **cardinality** (one
child or many) and its **member kinds**. The grammar rule as first
sketched (`if cond { member* }`) inherits the full `member`
alternation, which also includes `property_decl` / `property_bind` /
`signal_handler` / `state_decl` / `grid_track_list_member`; left
unrestricted, `if open { fill: red }` or `if open { state x: bool =
true }` would be grammatically admissible. The body shape is therefore
a real author-surface decision, and it pairs with the runtime insertion
granularity (DD-M3-P6-004 §Conditional insertion granularity).

### Options

- **B1 — exactly one widget child**
  - The `if` body admits **one** `widget_decl` only. No property / bind
    / handler / `state` / track-list directly in the body; no second
    child; and a **nested `conditional_member`** (a bare `if` directly
    in the body) is **deferred** this phase — nested control flow lands
    with the family extension (`else` / `for`), and Phase 6 reaches a
    nested conditional by wrapping the inner `if` in a widget (`if a {
    VStack { if b { … } } }`). A multi-widget conditional likewise wraps
    in a container (`if open { VStack { … } }`).
  - What you gain: the body **always materialises exactly one
    `WidgetNode` / `Visual`**, so the runtime present/absent is exactly
    **one** `insert_child` / `remove_child` (no child-range API, no
    slot-range bookkeeping, and no 0/1-materialised-child case a nested
    `if` would introduce — DD-M3-P6-004 IG-1 / DD-M3-P6-005 LA-1 stay an
    exact single-subtree grain); the smallest Phase-6 surface; the
    lightbox (a single `ZStack` body) is fully served; the multi-child
    range form is deferred cleanly to the Phase 7 `for`, which is its
    real driver.
  - What you give up: a two-widget conditional, **and a nested
    conditional**, must introduce a wrapper container — minor authoring
    ceremony, and the wrapper is usually wanted anyway (the overlay is a
    `ZStack`, the panel a `VStack`).

- **B2 — multiple widget children (`widget_decl+`)**
  - The `if` body admits **one or more** `widget_decl` members, still
    excluding property / bind / handler / `state` / track-list (and, as
    in B1, a nested `conditional_member` directly in the body).
  - What you gain: an author can place several widgets directly under
    `if` without a wrapper.
  - What you give up: the runtime now needs a **child-range** insert /
    remove (a declared range → materialised range), range slot
    bookkeeping, range Visual sibling-order, and range effect teardown
    — a materially larger Phase-6 runtime surface (DD-M3-P6-004 IG-2)
    with **no Phase-6 driver** (the lightbox needs one child), and it
    pre-empts the range machinery the Phase 7 `for` will build and
    generalise anyway.

- **B3 — arbitrary `member*`**
  - The body admits the full `member` alternation, so property / bind /
    handler / `state` / track-list members are allowed directly inside
    the `if`.
  - What you gain: nothing structural beyond B1/B2 — it is simply the
    unrestricted grammar rule.
  - What you give up: it opens **property / state / handler
    conditionality** — conditional property application to the parent,
    branch-local `state` lifetime, conditional handlers — an entirely
    different, unscoped design well beyond A7's *structural
    present/absent*. This is the surface the impl-readiness review
    flagged as a place an implementer would have to invent semantics.

### Comparison

B3 is **rejected**: A7 is about a binding driving subtree *structure*
(present/absent), not about conditionally applying properties, scoping
branch-local state, or gating handlers. Admitting non-structural
members in the body would force the implementer to decide property
conditionality / state lifetime / handler gating semantics that no DD
settles — exactly the "fills a hole with an on-the-spot design"
hazard. So the body is restricted to **structural** members regardless
of cardinality.

The live choice is **B1 vs B2** — single-child simplicity vs
wrapper-free multi-widget bodies. B1 keeps the runtime to a single
`insert_child` / `remove_child`; B2 needs a child-range mechanism. The
deciding fact is **driver and timing**: the Phase-6 driver (the
lightbox) needs only a single widget child, and the multi-child
range form is precisely what the Phase 7 `for` introduces and
generalises. Building B2's range machinery now would pay for Phase 7's
surface a phase early with no Phase-6 use, while B1's wrapper cost is
small and usually a container the author wants anyway. B2 is the
reasonable alternative **only if** the owner wants wrapper-free
multi-widget conditionals this phase and accepts the range runtime.

### Recommendation

**B1 — exactly one widget child.** The `if` body admits a single
`widget_decl`; non-structural members (property / bind / handler /
`state` / track-list), a **nested `conditional_member`** (a bare `if`
directly in the body), and a second child are rejected at `wasamoc
check` and re-checked at the loader (`WASAMO_ERR_IR_MALFORMED`,
DD-M3-P6-004). A multi-widget **or nested-conditional** body wraps in a
container; nested control flow itself lands additively with the family
extension (`else` / `for`).

**What B1 defers, and what it does *not* (the owner-facing boundary).**
B1 narrows the *immediate branch body* only; it does **not** defer
nested-conditional runtime / lifecycle semantics as a whole. Three
cases, fixed here so the acceptance target is unambiguous:

- **Phase 6 branch body admission:** `conditional_body ::= widget_decl`
  only — exactly one widget child.
- **Deferred:** bare nested control-flow *as the immediate branch body*,
  e.g. `if a { if b { … } }` (a `ControlFlow(_)` directly in the body);
  it lands with the family extension (`else` / `for`).
- **In scope (Phase 6):** **sibling** conditionals (`if a { … } if b { …
  }` under the same parent) **and** **descendant** conditionals nested
  inside the admitted widget subtree, e.g. `if a { VStack { if b { … }
  } }`. These are ordinary `if` members at a deeper `member*` position,
  reached by the wrapper B1 already requires; their runtime present/
  absent, the quiescent child-order invariant (DD-M3-P6-004), and the
  effect-lifecycle / SM-1 ordering (DD-M3-P6-005) **do** cover them.

So the answer to "what does an `if` body admit in Phase 6" is
`widget_decl` only — while **wrapped descendant** and **sibling**
conditionals are inside Phase 6 runtime / lifecycle semantics.

**Recommended bundle (so the acceptance is one decision, not three).**
Choosing **B1** here selects, as a single bundle across the coupled DDs:

- **DD-M3-P6-004 IG-1** — single-child `insert_child` / `remove_child`
  (no child-range machinery);
- **DD-M3-P6-005** — subtree-grain destroy/rebuild lifecycle (one
  subtree, not a range of subtrees);
- **diagnostics** — a non-structural or multi-child body is rejected at
  `wasamoc check` and re-checked at the loader
  (`WASAMO_ERR_IR_MALFORMED`).

If the owner instead selects **B2**, the bundle shifts together:
DD-M3-P6-004 reads as **IG-2** (child-range insert/remove), DD-M3-P6-005
disposes a *range* of subtrees, the verification closure gains range
slot / range Visual-order / range teardown cases, and the grammar
`conditional_body` becomes `widget_decl+`. **B3 is rejected as a
bundle** — it would force conditional property/state/handler semantics
into DD-003/004/005 — and is not carried. This is the same paired-fork
treatment as DD-M3-P6-004's O1/O2 (preamble §Cross-DD decision
dependencies indexes both).

## Condition expression vocabulary

### Options

- **E1 — narrow bool-expr**
  - The condition admits exactly what `Button.enabled` admits today — a
    `BOOL_LIT` or an `IDENT` resolving to a `bool`-typed `state`.
    `!ready`, comparison, logical operators deferred (Q5).
  - What you gain: structural novelty only — the condition is exactly
    the shipped `Button.enabled` bool-expr, so no operator surface
    enters this phase and the expression grammar stays **one coherent
    grammar**; sufficient for the lightbox (`if is_lightbox_open { … }`).
  - What you give up: no `if !x` ergonomics now — an author inverts by
    introducing a complementary `state`, or waits for the uniform Q5
    extension.

- **E1.5 — bool-only negation**
  - E1 plus a single prefix `!` on a bool operand (`cond_expr ::=
    BOOL_LIT | IDENT | "!" (BOOL_LIT | IDENT)`).
  - What you gain: the single operator authors reach for most
    (`if !is_open`), without comparison or binary logical operators and
    without touching numeric RHS positions.
  - What you give up: `!` works in a condition but not in `enabled:`
    unless `!` is *also* added there — an asymmetry that opens "why not
    `&&`?" immediately.

- **E1.75 — a bool-only `BoolExpr` sub-grammar**
  - A self-contained boolean expression AST over **bool operands only**
    — `!`, `&&`, `||`, parenthesisation — but no comparison operators
    and no coercion, admissible symmetrically in `bool` property RHS.
  - What you gain: `!`/`&&`/`||` over bool this phase.
  - What you give up: stands up a **second, parallel boolean grammar**
    that the eventual full `expr` extension (Q5) would then have to
    subsume or reconcile — two grammars for one position.

- **E2 — open the full expression grammar now**
  - Admit `!ready`, comparison (`count > 0`), logical operators, and
    `root.x` qualified reads in the condition position this phase.
  - What you gain: full expressiveness immediately.
  - What you give up: drags operator precedence, short-circuit,
    relational ops, qualified reads, and coercion into a
    *structural-rendering* phase as a multi-DD surface.

### Comparison

The real axis is **not** "minimal vs everything" but **where the
expression grammar should grow and whether it should grow uniformly.**
The deciding principle is a Wasamo-design one, not an effort one: the
condition position is just one `expr` position among many (every
property RHS is an `expr`), and Q5 already frames operators as a
**uniform expression-grammar extension** across all `expr` positions.

| Option | What it buys | The asymmetry / cost it creates |
|---|---|---|
| E1 | structural novelty only; condition is exactly the shipped `Button.enabled` bool-expr | none — but no `if !x` ergonomics |
| E1.5 | the single most-wanted operator (`!`) | `!` works in a condition but not in `enabled:` unless `!` is *also* added there; opens "why not `&&`?" immediately |
| E1.75 | bool-only `&&`/`||`/`!` | a second, parallel boolean grammar that the eventual full `expr` extension (Q5) would then have to subsume or reconcile — two grammars for one position |
| E2 | full expressiveness now | drags operator precedence, short-circuit, relational ops, qualified reads, coercion into a *structural-rendering* phase as a multi-DD surface |

E1.5 and E1.75 are the tempting middles, and they are tempting for a
**real** reason — `if !is_open` is genuinely ergonomic and authors will
want it. The reason to still hold the line at E1 is **uniformity, not
size**: admitting `!` (or a bool-only sub-grammar) *only* in the
condition fragments the expression grammar — it makes the condition
position more expressive than `enabled:` for no principled reason, and
E1.75 in particular stands up a parallel boolean grammar that the
later, uniform Q5 extension would have to reconcile or replace. The
Wasamo-faithful path is to grow operators **once, across all `expr`
positions together** (so `if !x` and `enabled: !x` arrive in the same
breath), which is its own focused surface (precedence, short-circuit,
coercion) rather than a corner of the conditional DD. E1 keeps the
condition exactly as expressive as the shipped bool-expr; it is
sufficient for the lightbox (`if is_lightbox_open { … }`) and for the
family seed, and it does not pre-commit a fragment that the uniform
extension would have to undo.

This is the honest trade-off for the owner: **E1 costs the `!`
ergonomic now** (authors invert by introducing a complementary `state`,
or wait for the uniform extension), in exchange for **one coherent
expression grammar** instead of a condition-only operator pocket. If
the owner weights the `!` ergonomic above grammar uniformity, E1.5 is
the smallest principled concession — but it should then be paired with
admitting `!` in `bool` property RHS too, to avoid the asymmetry.

### Recommendation

**E1.** `cond_expr` is a `BOOL_LIT` or an `IDENT` resolving to a
`bool`-typed `state`. `wasamoc check` rejects:

- a non-bool condition (`if count { … }` with `count: i32`;
  `if "x" { … }`) — type error;
- an undeclared / unresolved condition identifier — name-resolution
  error;
- an operator condition (`if !ready { … }`, `if a && b { … }`,
  `if count > 0 { … }`) — "operators in conditions are not yet
  supported" diagnostic pointing at the deferred expression-grammar
  extension (Q5), so the rejection is a *recorded deferral*, not a
  silent gap;
- a non-structural body member (a property / bind / handler / `state` /
  track-list directly inside the `if` body), a **nested
  `conditional_member`** (a bare `if` directly in the body), or **more
  than one** child in the body — "an `if` body admits a single widget
  child" type / shape diagnostic (Conditional body shape sub-issue, B1;
  the loader re-checks this as `WASAMO_ERR_IR_MALFORMED`,
  DD-M3-P6-004).

## Spec content seed

Grammar addition (for `dsl_spec.md` §3 and a new §4.14):

```
member            ::= property_decl | property_bind | widget_decl
                   |  signal_handler | state_decl
                   |  grid_track_list_member
                   |  conditional_member            ; M3-Phase 6

conditional_member ::= "if" cond_expr "{" conditional_body "}"

conditional_body  ::= widget_decl                   ; M3-Phase 6: exactly one
                                                    ; widget child — no property/
                                                    ; bind/handler/state/track-list,
                                                    ; no nested conditional_member,
                                                    ; no multiple children

cond_expr         ::= BOOL_LIT | IDENT              ; M3-Phase 6: same
                                                    ; bool-expr as Button.enabled
```

**Placement note (Phase 6).** The grammar admits `conditional_member`
wherever `member` appears, but Phase 6 restricts it semantically (at
`wasamoc check`) to **inside a widget body** — a component-level `if`
that would gate or multiply the single root is out of scope this phase
(no parent slot for a conditional root; see Surface form Recommendation
and DD-M3-P6-004). The spec chapter states this restriction explicitly.

**Body note (Phase 6).** The `if` body is a **single widget child**
(`widget_decl`), not the general `member*`: properties / binds /
handlers / `state` / track-list members directly in the body, a nested
`conditional_member` directly in the body, and more than one child are
rejected at `wasamoc check` and the loader. A multi-child or
nested-conditional body is authored by wrapping (`if open { VStack { …
} }`); the multi-child + child-range runtime form is the Phase 7 `for`
driver, and nested control flow lands with the family extension
(Conditional body shape Recommendation B1; DD-M3-P6-004 IG-1
single-child insert/remove).

**Why this is written as a *family*, not a feature (A12).** The spec
chapter (§4.14) is the **first chapter of Wasamo's structural rendering
model**. It states explicitly that `if` is one member of a structural
control-flow grammar family, that `else` / `switch` (more branches) and
`for` (Phase 7 iteration) are future members of the **same** family
with the **same** present/absent runtime machinery (DD-M3-P6-004), and
that the condition position will gain operators through a uniform
expression-grammar extension (Q5) — so an external reader can predict
the family's growth from the `if` chapter alone (the A12 external-reader
bar).

## Forward-compat exposure

- **Nested control flow directly in an `if` body** (`if a { if b { … }
  }`, no intervening widget). Deferred this phase (B1 admits a single
  `widget_decl`); reached meanwhile by wrapping the inner `if` in a
  widget. It lands additively with the family extension — the
  `conditional_body` widens to admit a `conditional_member`, the runtime
  body grain becomes 0/1-or-more materialised children, and the
  insertion/lifecycle grain (IG-1 / LA-1) is re-stated for that case in
  the same DD that ships `else` / `for`. No `if`-grammar or
  control-flow-IR (DD-004) shape change is needed — only the body
  admission widens.
- **`else` / `else if` / `switch`.** G1's block form chains naturally:
  `if c { … } else { … }`; `switch` is a sibling block keyword over a
  non-bool discriminant. The lowering target — DD-M3-P6-004's
  **member-level control-flow IR** (`ControlFlowNode` with a branch
  list) — is designed to carry additional branches, so `else` lifts the
  single-branch restriction and `switch` is a new `ControlFlowNode`
  variant, neither needing an `IrMember` shape change.
- **`for item in items { … }` (Phase 7).** A sibling
  `conditional_member`-like `iteration_member` block, the same
  family shape, reusing the structural-subtree runtime seam
  (DD-M3-P6-004). G1 makes this a parallel keyword, not a retrofit.
- **Operator conditions** (`!ready`, comparison, logical) — a uniform
  expression-grammar extension (Q5) applied to the `cond_expr` (and
  every other `expr`) position. The `if` grammar and the control-flow
  member (DD-004) are unaffected; only `cond_expr` widens.
- **Approach 3 (host-language constructs).** Because the surface is a
  structural `if` block lowering to the approach-neutral **member-level
  control-flow IR** (DD-M3-P6-004), a future language-internal DSL can
  lower its own `if`/`switch`/loop into the **same** IR / runtime seam —
  the thesis requirement that approach 3 stay reachable.

## Technical risk re-evaluation

- **One new `member` alternative + four reserved family keywords** —
  these are two separable changes with different blast radii. The
  *grammar / parser* change is small: the `member` dispatch gains a
  single `if`-first-token arm (no 2-token lookahead ambiguity — `if`
  is a keyword, not an `IDENT`, so it does not collide with
  `property_bind` / `widget_decl` / `signal_handler`), and `else` /
  `switch` / `for` get only "reserved, no production yet" parse-error
  arms — no new productions this phase. This is lower-risk than Phase
  5's Grid `grid_track_list_member` routing, which needed enclosing-
  type context; `if` needs none. The *lexer reservation* is the wider
  surface: **four** identifiers (`if` / `else` / `switch` / `for`)
  become reserved words, not one — the F4 family-pre-reservation
  decision, whose source-compat cost (below) covers all four.
- **No expression-grammar change** (E1) ⇒ no operator-precedence /
  short-circuit / coercion surface enters this phase; the Q5 deferral
  stays intact and is enforced by an explicit reject test, so the gap
  is recorded, not silent.
- **Reserved-word addition** (`if` / `else` / `switch` / `for`) is a
  source-compatibility note: an existing `.ui` using any of the four as
  an identifier would break, but no shipped `.ui` does (greppable; none
  was a prior identifier). Reserving the committed family now is cheaper
  than a per-keyword break as each construct lands; the not-yet-fixed
  contextual tokens (`in` / `case` / `default`) are deliberately left
  unreserved until their production exists.
- **Diagnostics carry the A12 weight** — the non-bool / operator /
  misplacement rejections are what make the public draft reproducible;
  they are pure-logic `wasamoc check` tests (verification closure
  item 1), independent of any host.
