# DD-M4-P3-005 — A small reusable way to make a handler's write conditional

**Status:** Proposed
**Phase:** M4-Phase 3
**AC:** AC9 (a small reusable handler-control-flow surface sufficient to
guard a state write at collection boundaries); phase-end criterion 4
(spec synchronization)

## Context

This record exists because of an owner requirement recorded on
2026-08-11 and landed as plan Revision 4 (`4afa204`): by the end of
Phase 3, the gallery's four selection producers — the `ArrowLeft` and
`ArrowRight` key handlers on the lightbox and the `<` and `>` buttons —
must all stop at both ends of the collection, for empty, one-item and
multi-item collections; and the capability that achieves it must be a
**general but small** surface, not a gallery-specific instruction.

The plan revision's critical check is the boundary this record works
inside: the guard "need not admit general functions, loops, an `else`
family, string concatenation or general arithmetic". AC9 repeats the
arithmetic exclusion in its own text.

Two things are deliberately *not* this record's job. DD-002 decides what
a read of an already-invalid index does; this record decides how an
invalid index is never written. And DD-002's per-effect containment is
consumed as given — no rollback, transaction or drain-level contract is
defined here.

### What exists to build on (measured)

- **The four producers write unconditionally.** In
  `examples/gallery/gallery.ui`:
  `key-down("ArrowLeft") => { root.selected_index -= 1; }`,
  `key-down("ArrowRight") => { root.selected_index += 1; }`, and the two
  navigation `Button`s with the same two statements.
- **A handler body is a statement list today.** `Block` /
  `BlockStatement` exist in the AST, `HandlerExpr::Block` in the IR;
  every statement is an assignment or an expression statement. There is
  no branching form of any kind.
- **`if` exists only as a structural member** of a widget body — a
  `Member::Conditional` in the AST and a `ControlFlowNode::If` in the
  IR. It is not a statement and cannot appear in a handler block.
- **A state write drains synchronously inside the statement.** M4-Phase
  2 T9 measured that collection regeneration runs during the handler
  statement rather than after handler return, and the reactive engine
  has no batching around a handler body.
- **Consume-on-handle is fixed.** DD-M4-P2-001 settled that a handler
  which runs consumes the event, and that an unconsumed key falls
  through to the default window procedure.

## Sub-issues

- **The control-flow shape** — what an author writes.
- **The minimum predicate vocabulary** that actually closes the
  four-producer boundary table.
- **What "false" means** — for the write, and for event consumption.
- **When the condition is evaluated**, given synchronous drains and
  multiple statements.
- **Whether guards may nest**, and what that implies about the excluded
  logical connectives.
- **IR carrier and evaluation.**
- **What stays out.**

## The boundary table

The table is the instrument, not an illustration: it is what eliminates
candidates. `i` is `selected_index`; the decrement producers are
`ArrowLeft` and `<`, the increment producers are `ArrowRight` and `>`.
A cell is correct when the write happens exactly when the resulting `i`
stays in `[0, count-1]`.

| count | `i` | decrement must | increment must |
|---|---|---|---|
| 0 | 0 | not write | **not write** |
| 1 | 0 | not write | not write |
| 3 | 0 | not write | write → 1 |
| 3 | 1 | write → 0 | write → 2 |
| 3 | 2 | write → 1 | not write |

The empty-collection increment cell in bold is the one that separates
the candidates.

## Options

### Control-flow shape

- **F-1 — a guarded block statement**: `if <bool-expr> { <statements> }`
  inside a handler body, single branch, no `else`.
- **F-2 — a guarded assignment**: a single statement carrying its own
  condition, e.g. `<target> -= 1 when <cond>;`.
- **F-3 — an early exit**: `guard <cond>;` / `return when <cond>;`,
  which ends the handler rather than scoping a block.
- **F-4 — `if` / `else`**, excluded by AC9 and the plan revision's
  critical check; listed so the exclusion is visible.

### Predicate vocabulary that closes the table

- **G-C — two cursor predicates**: `xs.has-previous(i)` and
  `xs.has-next(i)`, each `bool`. No operator is needed for the guard at
  all.
- **G-J — relational operators plus a last-index read**: `<`, `>`,
  `<=`, `>=` at DD-001's single comparison level, plus
  `xs.last-index()`. The guards are `i > 0` and `i < xs.last-index()`.
- **G-E — equality only, with nesting.** Equality closes nine of the ten
  cells; the empty-collection increment needs a second, nested guard on
  emptiness.
- **G-K — relational operators plus integer arithmetic**:
  `i < xs.count() - 1`. The conventional formulation, **excluded by
  AC9**.

### Event consumption when the guard is false

- **U-1 — the handler consumed the event**, because it ran.
- **U-2 — a handler whose guard was false did not handle the event**, so
  the event continues.

### Guard nesting

- **B-1 — a guard body is a statement list, and a guard is a
  statement**, so guards nest.
- **B-2 — a guard body admits assignments only**, so guards do not
  nest.

## Comparison

### Shape: F-1

F-2 is the smallest possible addition and the one that most obviously
cannot become a general imperative language. It fails on the
requirement's own words: "a small reusable **surface**". A per-assignment
condition cannot guard two statements together, and the gallery's own
future — a handler that both moves the index and closes a lightbox —
would need the condition repeated on each statement, with no guarantee
they stay in step. It also invents a keyword (`when`) for a concept the
language already spells `if`.

F-3 changes what a handler is. Today a handler body runs to completion;
an early exit introduces a control point, which is a larger semantic
change than a scoped block, and it composes badly with the synchronous
drain (an exit after a partially-drained write leaves a state the author
did not write and cannot see in the source order).

F-1 reuses the keyword, the block syntax and the single-branch shape the
language already has for structural `if`, and the IR's branch list is
already the recorded extension point for `else`. Its one cost is that
`if` now means two different things in two different positions — a
structural member in a widget body, a statement in a handler body. The
positions are syntactically disjoint, so there is no ambiguity to
resolve, but the **spec** has to say it plainly rather than let a reader
infer that a handler `if` can contain a widget. That is a documentation
obligation, not a grammar problem.

### Vocabulary: the table eliminates G-E, AC9 eliminates G-K, and the
### remaining choice is generality against size

**G-K first, because it is what most languages would do.** `i <
xs.count() - 1` is the formulation an author arriving from any other
language writes. AC9 excludes general arithmetic and the framing repeats
the exclusion, so it is unavailable. Everything below is a consequence
of that exclusion, and it is worth saying so directly: **the extra
vocabulary in G-C and G-J is the price of not having `- 1`.**

**G-E closes nine cells and loses the tenth.** With `i != 0` and
`i != xs.last-index()`, every row is correct except the empty
collection's increment: `last-index()` is `-1` there, `0 != -1` is true,
and the guard writes `1` into an empty collection. It is recoverable —
`if xs.is-empty() == false { if i != xs.last-index() { … } }` — and that
recovery is what disqualifies it. The authored form for "step right" is
then two nested guards and an `== false`, in the app that is the
milestone's outward-facing banner. The requirement asks for the gallery
edges to be expressible naturally; this is expressible, not natural.

**G-C is the smallest and reads the best at the point of use.**
`if photos.has-next(selected_index) { root.selected_index += 1; }` is
one line, needs no operator, and is correct in every row including the
empty one. Two methods, zero grammar.

**G-J is larger and leaves the language able to say more.** Four
operators at the comparison level DD-001 is already creating for `==`,
plus one read. `if selected_index < photos.last-index() { … }`.

The deciding question is what "general but small" means for the
**language**, not for the gallery. Both close the table, so neither is
gallery-specific in the narrow sense. But G-C's generality stops at the
cursor pattern: after adopting it, the language still cannot express any
ordering comparison at all. The next predicate an author reaches for —
"show a badge when there are more than three items", "disable this when
the count is below the minimum" — is unavailable, and there is no
combination of `has-next` and `has-previous` that supplies it. G-J's
four operators cover the whole family in one addition, and they are the
family [dsl_spec.md §4.6](../../../../docs/dsl_spec.md) already told
readers to expect ("it grows across all `expr` positions at once").

G-C also carries a retirement problem G-J does not. `has-next(i)` is a
predicate over a collection **and** an index — a fused concept whose
parts the language will eventually have separately. Once ordering
comparison exists, `has-next` is a redundant spelling of
`i < xs.last-index()`, and a redundant spelling in a frozen 1.0 surface
is a permanent apology. `last-index()` has a milder version of the same
exposure — it becomes `count() - 1` sugar if arithmetic arrives — but it
is a plain collection read that stands on its own, in the same family as
`count()` and `is-empty()`, and a language can reasonably keep all three
forever.

The honest cost of G-J: `last-index()` returns `-1` for an empty
collection. That sentinel is load-bearing for the empty row, and a
sentinel in a frozen surface is a thing to be uncomfortable about. It is
specified once, in DD-002, and pinned by a test.

### False means no write, and the handler still consumed: U-1

U-2 is superficially attractive — "nothing happened, so let it through"
— and it is wrong for a reason worth stating: it makes event routing
depend on **data**. Pressing `ArrowRight` on the last photo would fall
through to the default window procedure while pressing it on the first
photo would not, so the window's behaviour at the boundary would differ
from its behaviour in the middle for reasons invisible in the source.
It would also contradict DD-M4-P2-001, where consumption is a property
of a handler having been found and run.

U-1 keeps consumption a structural property: the handler matched, the
handler ran, the event is consumed, and what the handler chose to write
is its own business.

### Nesting: B-1, with the consequence stated

B-1 costs nothing — a guard body is the statement list a handler body
already is — and it makes the language uniform in the way F-1 was chosen
for.

Its consequence must be written down rather than discovered: **nested
guards are conjunction**. The language excludes `&&`, and nesting
supplies exactly that connective with a different shape. Disjunction has
no such shape and remains unavailable. This is not a loophole to close
— forbidding nesting to protect the absence of `&&` would be
protecting a boundary that was drawn to keep the *grammar* small, not to
keep authors from expressing conjunction. But a spec that excludes `&&`
and silently admits its nested equivalent is a spec that misleads, so
§4.6 or §4.15 says so.

### Evaluation timing

A guard's condition is evaluated **when its statement is reached**,
against live state, in the handler's own evaluation context — untracked,
like every other handler read, because handlers do not register reactive
dependencies.

The consequence of the synchronous drain has to be stated rather than
designed around: if an earlier statement in the same body writes state,
that write drains before the next statement is reached, so a later
guard sees the post-drain world. An author who writes two guarded
statements over the same discriminant gets sequential semantics, not a
snapshot. This is the existing engine's behaviour surfacing in a new
construct; inventing a snapshot here would be a second execution model
for handler bodies.

## Recommendation

- **F-1** — `if <bool-expr> { <statements> }` as a handler-body
  statement. Single branch, **no `else`**. The condition is any `bool`
  expression under DD-001's uniform rule.
- **G-J** — the four relational operators join DD-001's single
  comparison level (ordering on `i32` only), and `xs.last-index()` joins
  DD-002's collection reads. The gallery's guards are
  `selected_index > 0` and `selected_index < photos.last-index()`, and
  the same two guards serve all four producers unchanged.
- **The `- 1` that AC9 excludes is what `last-index()` pays for**, and
  the record says so rather than presenting the read as independently
  motivated.
- **U-1** — a false guard writes nothing and the handler still consumes
  the event. Routing does not depend on data.
- **B-1** — guards nest, which supplies conjunction. The spec states
  that nesting is conjunction and that disjunction remains
  unavailable.
- **The condition is evaluated when its statement is reached**, against
  live state, untracked; a preceding statement's synchronous drain is
  visible to a following guard.
- **IR**: one new guarded-block form beside `Block`, carrying a
  condition and a statement list. `docs/dsl_spec.md` §8.9 gains one row;
  §4.5 / §4.15 gain the handler-statement grammar and the
  `if`-means-two-things note.
- **Out**: `else`, `else if`, loops, general functions, early return,
  arbitrary commands, logical connectives, arithmetic, and any
  gallery-specific instruction.
- **DD-002's contract is consumed, not re-decided.** No rollback, no
  transaction, no drain-level guarantee is defined here.

## Forward-compat exposure

- **`else` is the recorded family extension point** and the IR already
  models branches as a list, so adding it later does not reshape the
  form. Nothing here reserves the keyword's semantics beyond
  single-branch.
- **Arithmetic is additive and would make `last-index()` redundant
  sugar.** That is a retirement *question* for a later editorial pass,
  not a commitment either way; what this record fixes is that
  `last-index()` is a plain total read rather than a guard-specific
  construct, so keeping it costs nothing if arithmetic arrives.
- **Logical connectives are additive** and would make nesting the sugar
  rather than the mechanism. Their unmeasured interaction with
  dependency tracking is named in DD-001 and is not re-argued here.
- **A catch-all key signal (M4-Phase 2's K4) needed "a handler body that
  can branch"**, which this record supplies. That does **not** make K4
  available: DD-M4-P2-005 recorded that K4 and a structured key value
  reopen *together*, and the structured value still has no typed-constant
  kind in the value grammar. This record removes one of two blockers and
  claims nothing about the other.
- **Early return remains a different design**, not an extension. It
  changes what a handler body is, and adopting it later would have to
  say what happens to a partially-drained write.
- **A guard around a collection mutation is expressible** and is
  untested in this phase; the gallery's guards are over a scalar. The
  combination is named because collection writes drain synchronously and
  a guard evaluated before one sees the pre-write collection.

## Technical risk re-evaluation

- **The boundary table is the evidence, and it needs the negative legs
  more than the positive one.** A guard that is always true produces the
  same screen as a correct guard for every step that is not at an edge.
  The discriminating legs are: at each end, one further input in the
  same direction changes nothing; and — the leg that catches a guard
  that is always *false* — a step inward from the end does work. Both
  legs, at both ends, for all four producers.
- **Empty and one-item are where a plausible implementation fails.** A
  guard written as `i < xs.count()` passes every multi-item row and
  fails the last-item row; a guard written without the `last-index()`
  sentinel passes every non-empty row and fails the empty one. Neither
  is visible in a gallery with eighteen photos, so the cardinality
  matrix belongs in a mock-free integration test with a mutable
  collection, not in screenshots.
- **Four producers, one rule.** The failure mode the requirement names
  is a surface where the key handlers are guarded and the buttons are
  not, or one end is guarded and the other is not. The matrix is
  producer × cardinality × end, and a partially-applied guard passes any
  test that only exercises one producer.
- **`if` in two positions is a checker hazard, not a parser one.** The
  positions are disjoint, so the risk is that the two admission paths
  drift: a widget member accepted in a handler block, or a statement
  accepted as a widget member. Both directions need a firing reject
  test.
- **Guard nesting is a recursion the completeness contract has to
  reach.** DD-006's admission gate must descend into a guard body the
  way it descends into a block; a guarded assignment that skips the gate
  would reintroduce exactly the hole DD-006 exists to close, in the
  construct this record adds.
- **Nothing here is ABI-bearing.** The new form is a handler-body
  statement lowered into the existing expression enum and evaluated by
  the existing handler evaluator.
