# DD-M3-P6-003 — Conditional rendering author-facing grammar surface

**Status:** Proposed
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

## Options

### Approach (the thesis-level choice — settled by FD-CR)

- **Approach 1 — property control** (`visible:`/`enabled:` on an
  always-built subtree). **Rejected by FD-CR**: proves property
  toggling, not structural present/absent.
- **Approach 2 — template + dedicated structural syntax.** **Adopted
  by FD-CR.** The sub-options below are *within* approach 2.
- **Approach 3 — host-language constructs.** Not v1; must stay
  reachable later.

This DD records the three-approach comparison because A12 requires the
spec chapter to present it (so an external reader understands *why*
Wasamo uses a structural directive rather than a `visible` property),
but the choice itself is settled framing, not re-litigated here.

### Surface form within approach 2

- **G1 — `if`-block member (recommended):**

  ```
  if <bool-expr> {
      <member>*
  }
  ```

  An `if` block is a new `member` alternative; the members inside it
  become conditional children of the enclosing container.

- **G2 — `when:` structural attribute:**

  ```
  Box { when: is_open; … }      // the Box is present iff is_open
  ```

  A reserved attribute on a widget that gates that widget's presence.

- **G3 — structural directive token** (e.g. `@if(is_open) { … }` /
  `#if is_open { … }`): a sigil-prefixed directive distinct from both
  ordinary members and attributes.

### Condition expression vocabulary

- **E1 — narrow bool-expr (recommended):** the condition admits
  exactly what `Button.enabled` admits today — a `BOOL_LIT` or an
  `IDENT` resolving to a `bool`-typed `state`. `!ready`, comparison,
  logical operators deferred (Q5).
- **E1.5 — bool-only negation:** E1 plus a single prefix `!` on a bool
  operand (`cond_expr ::= BOOL_LIT | IDENT | "!" (BOOL_LIT | IDENT)`).
  Adds the one operator authors reach for most (`if !is_open`) without
  comparison or binary logical operators, and without touching numeric
  RHS positions.
- **E1.75 — a bool-only `BoolExpr` sub-grammar:** a self-contained
  boolean expression AST over **bool operands only** — `!`, `&&`, `||`,
  parenthesisation — but no comparison operators (`>`, `==`) and no
  coercion, so it stays inside the `bool` type without opening the
  numeric/relational expression surface. Could be admitted in the
  condition (and symmetrically in `bool` property RHS) this phase.
- **E2 — open the full expression grammar now:** admit `!ready`,
  comparison (`count > 0`), logical operators, and `root.x` qualified
  reads in the condition position this phase.

## Comparison

### Surface form

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

### Condition vocabulary

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

## Recommendation

**Approach 2, form G1 (`if`-block member), condition vocabulary E1.**

Grammar addition (for `dsl_spec.md` §3 and a new §4.14):

```
member            ::= property_decl | property_bind | widget_decl
                   |  signal_handler | state_decl
                   |  grid_track_list_member
                   |  conditional_member            ; M3-Phase 6

conditional_member ::= "if" cond_expr "{" member* "}"

cond_expr         ::= BOOL_LIT | IDENT              ; M3-Phase 6: same
                                                    ; bool-expr as Button.enabled
```

Concrete decisions:

- **`if` is a `member` alternative**, admitted wherever `member*` is
  admitted (a component body and any widget body). The members inside
  the block are the **conditional subtree**: when the condition is
  true they are present as children of the enclosing container at the
  block's document position; when false they are absent (DD-M3-P6-004
  for the structural mechanism). An `if` block in a non-member
  position is a `wasamoc check` / parse error.
- **`if` is a reserved keyword** (added to the keyword set,
  dsl_spec §2.1), so `state if: …` and `IDENT == "if"` widget/prop
  names are rejected — mirroring the `true`/`false` reservation
  (DD-M3-P1-002) and the Q5 reserved-word rule.
- **Condition (E1):** `cond_expr` is a `BOOL_LIT` or an `IDENT`
  resolving to a `bool`-typed `state`. `wasamoc check` rejects:
  - a non-bool condition (`if count { … }` with `count: i32`;
    `if "x" { … }`) — type error;
  - an undeclared / unresolved condition identifier — name-resolution
    error;
  - an operator condition (`if !ready { … }`, `if a && b { … }`,
    `if count > 0 { … }`) — "operators in conditions are not yet
    supported" diagnostic pointing at the deferred expression-grammar
    extension (Q5), so the rejection is a *recorded deferral*, not a
    silent gap.
- **`if true { … }` / `if false { … }`** are well-typed but
  degenerate (always present / always absent); permitted, not
  special-cased.
- **No `else` this phase.** `else` / `else if` / `switch` are reserved
  as future family members (forward-compat below); a bare `else` is a
  parse error with a "not yet supported" diagnostic.

A7 visible proof uses the single-`bool` form
`if is_lightbox_open { ZStack { … } }`, driven by a text-Button click
handler (FD-B / FD-C).

### Why this is written as a *family*, not a feature (A12)

The spec chapter (§4.14) is the **first chapter of Wasamo's structural
rendering model**. It states explicitly that `if` is one member of a
structural control-flow grammar family, that `else` / `switch` (more
branches) and `for` (Phase 7 iteration) are future members of the
**same** family with the **same** present/absent runtime machinery
(DD-M3-P6-004), and that the condition position will gain operators
through a uniform expression-grammar extension (Q5) — so an external
reader can predict the family's growth from the `if` chapter alone
(the A12 external-reader bar).

## Forward-compat exposure

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

- **One new `member` alternative + one keyword** is a contained
  grammar change; the parser's `member` dispatch gains an `if`-first-
  token arm (no 2-token lookahead ambiguity — `if` is a keyword, not
  an `IDENT`, so it does not collide with `property_bind` /
  `widget_decl` / `signal_handler`). This is lower-risk than Phase 5's
  Grid `grid_track_list_member` routing, which needed enclosing-type
  context; `if` needs none.
- **No expression-grammar change** (E1) ⇒ no operator-precedence /
  short-circuit / coercion surface enters this phase; the Q5 deferral
  stays intact and is enforced by an explicit reject test, so the gap
  is recorded, not silent.
- **Reserved-word addition** (`if`) is a source-compatibility note: an
  existing `.ui` using `if` as an identifier would break, but no
  shipped `.ui` does (greppable; `if` was not a prior identifier).
- **Diagnostics carry the A12 weight** — the non-bool / operator /
  misplacement rejections are what make the public draft reproducible;
  they are pure-logic `wasamoc check` tests (verification closure
  item 1), independent of any host.
