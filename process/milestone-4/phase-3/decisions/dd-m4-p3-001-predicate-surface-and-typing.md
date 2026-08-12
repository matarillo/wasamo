# DD-M4-P3-001 — The shared predicate surface: spelling, positions, and typing

**Status:** Proposed
**Phase:** M4-Phase 3
**AC:** AC9 (the expression surface DD-002 … DD-006 consume), and
phase-end criterion 4 (spec synchronization)

## Context

Wasamo's expression grammar today admits literals, name references,
string interpolation, and — inside a `for` body — loop binders. It admits
**no operator in any position**. This record is where that stops being
true. With [DD-007](dd-m4-p3-007-dot-meaning-and-prefix-set.md) it is one
of the two records in the set that decide how the language grows rather
than what a feature does: DD-007 settles what a dot means, and this
record settles what may be written and what it types to.

Three commitments constrain it before any option is weighed.

- [dsl_spec.md §4.6](../../../../docs/dsl_spec.md) states, of the
  deferred operator extension, that "it grows across all `expr`
  positions at once, not condition-only". This is shipped normative
  text. An option that admits comparison in one position and not
  another is not merely narrower — it contradicts a published sentence,
  and adopting it would require saying so and revising §4.6, not
  quietly diverging from it.
- [dsl_spec.md §4.17](../../../../docs/dsl_spec.md) already names the
  form DD-004 wants — "a discriminant state with `checked: tab ==
  value`, once an equality operator enters the expression grammar" —
  and records it as a future direction rather than reserved syntax. The
  spelling is anticipated; the typing rules are not.
- **DD-007 is upstream.** It closes the prefix set, which is what frees
  the member-read spelling weighed below. Without it, that spelling is
  legal-and-taken and the choice would be made against an accident
  rather than on merit.

The phase's other five records all consume this one. DD-002 needs
collection interrogation and element access; DD-003 needs a condition
that reads loop binders; DD-004 needs equality producing `bool` in a
binding position; DD-005 needs whatever closes its boundary table;
DD-006 needs a result type for every expression that can be assigned.
Deciding those five separately would produce a language where the same
comparison is spelled or typed differently depending on where the author
wrote it — which is the failure the framing's §論点一覧 names when it
gives DD-001 the common line.

### What exists to build on (measured)

- **The lexer has no comparison token.** `if index == sel { … }` fails
  in the parser with `expected \`{\`, found \`=\``. `Token::Bang` exists
  but `!x` lowers to `Expr::UnsupportedOperator` and is rejected.
- **`xs[i]` already parses** and is rejected by a named diagnostic
  ("indexed reads (`xs[i]`) are deferred in M3-Phase 7"). The bracket
  spelling is reserved, not free.
- **A method-call expression already exists.** `Expr::CollectionCall`
  carries `receiver.method(args)` and is used by `xs.append(v)` /
  `xs.drop-last()`; parser, checker and lowering paths all exist. The
  one position that admits it — the collection-assignment right-hand
  side — requires the receiver to be a single-segment local state name.
- **A dotted prefix is discarded rather than validated.** `wasamoc check`
  resolves the **last** segment as the state name and drops everything
  before it, so `photos.count` and `a.b.c.count` both compile as a read
  of the state `count`. What a dot means, and which prefixes are
  admissible, is settled by
  [DD-007](dd-m4-p3-007-dot-meaning-and-prefix-set.md); this record
  consumes that rule and adds no prefix of its own.
- **`check_if_condition` is a separate, narrower checker** from the
  property-binding path, which is itself separate from the handler
  right-hand-side path. Three positions, three functions, three
  admission rules.
- **String interpolation holds a qualified name, not an expression.**
  The surface production is `\{` *qualified_name* `}`
  ([dsl_spec.md §2.4](../../../../docs/dsl_spec.md)) and the AST is
  `StringPart::Interp(QualifiedName)`, so `"\{photos.count()}"` does
  not parse. The IR is already expression-shaped —
  `interp_part ::= STRING | "(" expr ")"`.
- **The runtime already stringifies an integer binding written into a
  string property**, but `wasamoc check` rejects the pairing:
  `Text { text: root.count }` produces "type mismatch in binding
  `Text.text`: target is `string`, source is `i32`".

## Sub-issues

- **Which forms** Phase 3 admits at all.
- **How collection interrogation is spelled** (count, emptiness, last
  index).
- **How element access is spelled.**
- **What the comparison family looks like** — which operators, one
  precedence level or several, associativity, parentheses.
- **Typing rules** — operand types, result types, and whether an
  integer-valued expression may be displayed in a string property.
- **Which expression positions admit which forms.**
- **Name resolution inside this record's forms** — loop binders, and
  what happens when a state is named `count`. What a dot means and
  which prefixes exist is DD-007's.
- **AST / IR carrier** — extend the existing expression enum or add a
  second one.
- **Which gate checks what**, and how a wrong type or wrong position is
  reported.

## Options

### Collection interrogation spelling

- **S1 — free-function calls**: `count(photos)`, `empty(photos)`.
- **S2 — member reads**: `photos.count`, `photos.is-empty`.
- **S3 — method calls**: `photos.count()`, `photos.is-empty()`,
  `photos.last-index()`.
- **S4 — no interrogation form**; derive everything from element access
  and comparison.

### Element access spelling

- **X1 — bracket**: `photos[selected_index]`.
- **X2 — method**: `photos.at(selected_index)`.
- **X3 — none in Phase 3.** Listed because it is what "count and
  emptiness only" would mean; AC9 names index access, so this is
  excluded by scope rather than by comparison.

### The comparison family

- **E1 — equality only** (`==`, `!=`).
- **E2 — equality plus ordering** (`==`, `!=`, `<`, `>`, `<=`, `>=`),
  one non-associative precedence level.
- **E3 — a predicate call form instead of operators**
  (`a.equals(b)`), avoiding operator grammar entirely.
- **E4 — a full operator grammar** with logical connectives and
  precedence levels.

### Positions

- **P1 — uniform**: every form admitted here is admissible in every
  expression position that already takes a scalar expression — property
  binding, string interpolation, `if` condition, handler assignment
  right-hand side (subject to DD-006), and DD-005's guard.
- **P2 — positional table**: each form is admitted per position, so
  (for example) comparison is admitted in conditions and guards but not
  in property bindings.

### Integer display in a string property

- **T1 — strict.** An `i32`-valued expression is not admissible where a
  `string` is expected. A count is displayed by writing
  `"\{photos.count()}"`, which is the existing interpolation path.
- **T2 — display conversion.** Admit `i32` in a string property
  position with a defined decimal conversion, matching what the runtime
  writer already does.
- **T3 — a string-producing interrogation form.**

### AST / IR carrier

- **C1 — new variants on the existing `HandlerExpr` enum**, following
  the "single unified enum, no side enum" precedent
  [dsl_spec.md §8.9](../../../../docs/dsl_spec.md) records for
  M3-Phase 7.
- **C2 — a second expression tree** for predicates, kept beside
  `HandlerExpr`.
- **C3 — a generic `TypedValue` union**, excluded by the phase
  constraints and listed so the exclusion is visible rather than
  assumed.

## Comparison

### Collection interrogation: S3

S1's cost is a namespace. `count(photos)` introduces a global function
namespace into a language that has none, and it collides immediately:
`count` is the name of the counter example's state, so `count(x)` and
`count` would resolve in different namespaces by syntactic position.
Every later built-in would widen a reserved-word surface that M6 has to
freeze.

S2 reads best, and it is what the existing deferral diagnostic implies
("member navigation such as `xs.length`" is named in
`collection_external_read_segment`'s comment as the deferred shape).
DD-007 leaves the spelling free, so this is a genuine choice rather than
a forced one. Its cost is two rules where S3 has one.

The first is inside the collection surface: under S2 an author asks
`photos.count` but calls `photos.append(v)`, so the surface splits into
queries that are member reads and operations that are calls, and each
new name has to be learned on the right side of that split. The second
arrives with structured element fields (`item.filename`, anticipated in
[dsl-grammar.md Q8](../../../../docs/notes/dsl-grammar.md)): under S2 a
**stored field** and a **computed query** are spelled identically, and
only the receiver's type — which the reader has to know — separates
`photo.filename` from `photos.count`.

S3 costs two characters and buys three things. It reuses
`Expr::CollectionCall`, which parses, checks and lowers today. It puts
interrogation in the **same family** as `append` / `drop-last`, so the
author sees one rule: things you ask of a collection are calls. And the
parenthesised empty argument list keeps the stored/computed distinction
visible at the point of use — when element fields arrive, `photo.filename`
is a field and `photos.count()` is a question, without the reader
consulting a type.

S4 fails on its own: emptiness derived as `photos.count() == 0` is
expressible, but a `for`-external count is required by AC9 anyway, so S4
saves nothing and forces every emptiness test through a comparison.

### Element access: X1

X1 is already the reserved spelling — the parser recognises it and
names it in a diagnostic that says "deferred", which is a promise a
reader can reasonably read as "this is the spelling when it arrives".
X2 would be more consistent with S3's method family, and that is a real
argument, but it would strand the reserved bracket form: either the
diagnostic keeps pointing at a spelling that never lands, or the
diagnostic changes and the parser keeps a rejection branch for a syntax
with no future. X1 also matches every language an author is likely to
arrive from.

The cost of X1 is that bracket access is where **partiality** enters the
language — every other form here is total. That cost is real and is
DD-002's subject, not an argument against the spelling.

### The comparison family: E2, with the operand rules doing the narrowing

E3 avoids operator grammar but does not avoid the work: `a.equals(b)`
still needs operand typing, a `bool` result and admission rules in every
position, and it produces a form no author expects, that §4.17 has
already told readers will be `==`, and that a later real `==` would
retire. It buys nothing except a smaller lexer diff.

E4 is out — logical connectives are excluded by the framing, and
multiple precedence levels are what "no general expression language"
means in practice.

The real choice is E1 versus E2, and **DD-005's boundary table decides
it**, not this record's taste. With equality alone, the gallery's
decrement guard is expressible (`selected_index != 0`) and the increment
guard is not: bounding the top requires either arithmetic (`selected_index
< photos.count() - 1`), which AC9 excludes, or an ordering comparison
against a last-index read. Since DD-005 is an owner-required deliverable
and its table does not close under E1, the operator set has to be E2 and
this record's job is to make E2 as small as it can honestly be:

- **one precedence level**, non-associative — `a == b == c` is a syntax
  error, not a parse tree, so no precedence table is created and none
  has to be frozen at M6;
- **both operands the same scalar type**, no implicit conversion in
  either direction;
- **ordering (`<`, `>`, `<=`, `>=`) on `i32` only** — string collation
  is a locale question this milestone must not answer, and bool ordering
  is meaningless;
- **equality (`==`, `!=`) on `i32`, `string` and `bool`**;
- **no comparison over collections** — `xs == ys` is rejected, so
  whole-value collection comparison stays undesigned;
- **result is always `bool`**;
- **parentheses admitted for grouping** so a later precedence extension
  cannot change the meaning of an existing parenthesised expression.

Rejecting associativity is what keeps this from being "an operator
grammar". The whole addition is: one level, six operators, six typing
rows.

### Positions: P1

P2 is what the codebase does today — three positions, three functions,
three admission rules — and it is exactly the shape that produced the
DD-006 gap: a rule that lives in one position's checker is a rule the
other positions do not have. It also contradicts §4.6's published
sentence.

P1's cost is that it admits combinations no consumer needs, such as a
comparison bound to `Button.enabled` or interpolated into a caption.
That cost is worth naming plainly: those are **not** cases anyone asked
for, and each is a surface M6 freezes. The counter-argument is stronger:
the author-visible rule under P1 is one sentence ("a `bool` expression
is admissible wherever a `bool` is expected"), and under P2 it is a
table the author has to consult. A language whose rule is a table is the
"predictably surprising" outcome the framing's DD-001 rationale names.

P1 does **not** mean every form is admissible everywhere. It means
admission is decided by **type and capability**, not by position
identity — which is the same principle DD-006 applies to assignment, and
the same principle §4.6 already used for collection-valued expressions
("admitted type-driven, not positional").

### Integer display: T1

T2 is tempting because the runtime writer already does it: the only
thing standing between `Text { text: photos.count() }` and a rendered
number is `types_compatible`. But adopting T2 means adopting an implicit
`i32` → `string` conversion **as a language rule**, in a language that
otherwise has no implicit conversion at all, and the conversion is not
neutral: it fixes a decimal formatting with no locale, no width, no
sign policy, at the moment M6 freezes the surface. It also opens the
question T2 cannot avoid — if `i32` converts, why not `bool`? — which
the language has already answered "no" to twice, in
`check_string_interpolation_type` and in the bool-binder interpolation
reject.

T1 costs two things. The gallery consumer's count is authored as
`Text { text: "\{photos.count()}" }` beside a static label rather than
`Text { text: photos.count() }` — one interpolation the author writes.
And the interpolation placeholder has to widen from a qualified name to
an expression, since `"\{photos.count()}"` does not parse today. That
widening is the same P1 rule applied to one more position, and the IR
side already carries it (`interp_part` takes an expression), so it is a
front-end change with no representation change behind it. Both together
are a smaller price than a permanent implicit conversion.

T3 is worse than both: a `count-text()` form would be a display concern
wearing a data form's clothes.

### Carrier: C1

C2's argument is isolation — predicates cannot accidentally be evaluated
by handler paths. But the phase's whole point is that they **are** the
same expressions in different positions: DD-005's guard evaluates a
comparison in handler context and DD-003's condition evaluates the same
comparison in binding context. Two trees would mean two lowerings, two
evaluators, and two places for DD-006's completeness contract to have a
hole. §8.9 already recorded the "no side enum" outcome when M3-Phase 7
faced the same choice for collection expressions.

C1's new variants are: a collection interrogation carrying its receiver
and which interrogation it is, an element access carrying receiver,
element type and index expression, and a comparison carrying operator,
operand type and both operands. The element-type tag on element access
follows the `ListPropRead { path, elem }` precedent, so the evaluator
picks a typed path without a runtime union — which is the structural form
of the `TypedValue` deferral §8.12 records, applied again.

C3 is excluded by [../requirements/constraints.md](../requirements/constraints.md)
§Phase 3 の現行基準線 and would additionally take the `TypedValue`
decision away from M4-Phase 7, which owns it.

### Name resolution and the gates

Loop binders resolve in a `for` body and nowhere else — unchanged from
M3-Phase 7, and unchanged by this record except that the set of
positions a binder may appear in now includes the `if` condition
(DD-003) and the guard (DD-005). Shadowing stays rejected, so a name
resolves in exactly one scope and no resolution order has to be
specified. Everything to the left of a dot is a value under DD-007's
rule, so a receiver is resolved the same way a bare name is and this
record introduces no second resolution path.

`wasamoc check` is the author-facing gate for every rule above: result
type, operand types, position admission, and receiver-must-be-a-
collection. The framing is explicit that "the execution path rejects
the type" is not an acceptable substitute for a diagnostic, and the
starting state shows why — the current string-write failure is exactly
that substitute. The loader re-checks the invariants DD-006 settles;
this record does not add a separate loader policy.

## Recommendation

- **S3** — collection interrogation is a method call on the collection:
  `photos.count()` (`i32`), `photos.is-empty()` (`bool`),
  `photos.last-index()` (`i32`). One family with the existing
  `append` / `drop-last`, no function namespace.
- **X1** — element access is `photos[i]`, the spelling the parser
  already reserves. Its result type is the collection's element type.
- **E2** — one non-associative comparison level with six operators.
  Equality over `i32` / `string` / `bool`; ordering over `i32` only;
  both operands the same type with **no implicit conversion**; result
  `bool`; collections not comparable; parentheses admitted for grouping.
  `a == b == c` is a syntax error.
- **P1** — admission is by type and capability, not by position
  identity. A form admitted here is admissible in every expression
  position that already takes a scalar expression of its type; where a
  position has an additional capability rule, that rule is DD-006's, not
  a second admission table.
- **T1** — no implicit `i32` → `string` conversion. A count is displayed
  through interpolation, whose placeholder widens from a qualified name
  to an expression.
- **C1** — new `HandlerExpr` variants; no second expression tree, no
  `TypedValue`. Element access carries its element-type tag the way
  `ListPropRead` does.
- **No unary `!`, no logical connectives, no arithmetic** in any
  position. `!x` keeps its current diagnostic; a negated predicate is
  written with `!=` or by choosing the opposite comparison.
- **`wasamoc check` reports every violation of the rules above**, with
  the offending span, and never defers a typing decision to invocation.
- **`docs/dsl_spec.md` moves** — §2.4 (the interpolation placeholder
  takes an expression), §4.6 (the operator sentence and the new forms),
  §4.14 (condition expressions), §5 (the AST enums), §8.9 (three IR
  rows), §8.11 (whatever DD-006 settles). `docs/abi_spec.md` does not
  move.

## Forward-compat exposure

- **Arithmetic is additive to the grammar and is not additive to the
  design.** Adding `+ - * /` later means adding precedence levels below
  the comparison level, which is why comparison is specified as
  non-associative and parenthesisable now: an existing `(a) == (b)`
  keeps its meaning under any later precedence table, and an existing
  `a == b == c` cannot, which is why it is a syntax error rather than a
  parse.
- **Logical connectives are additive and would sit above the comparison
  level.** Nothing here forecloses them. What this record does not
  claim is that they will be free: `&&` / `||` bring short-circuit
  evaluation, which interacts with dependency tracking (an unread
  operand registers no dependency), and that interaction is unmeasured.
  It is named, not sized.
- **Ordering over `string` is a reopening, not an addition.** Admitting
  `<` on strings later requires choosing a collation, and any choice
  changes nothing that exists today because no `.ui` can express it.
- **`is-empty()` is redundant once comparison exists**
  (`xs.count() == 0`). It is kept because emptiness is a question
  authors ask directly and because a dedicated form reads at the point
  of use; the cost is one method that a later editorial pass could
  argue to retire. That is recorded as a known redundancy, not as a
  commitment either way.
- **A future value-producing expression** (a computed string, a
  formatted number) lands in the same expression enum and gets its
  result type from the same typing rules; DD-006's admission framework
  is what decides where it may be written. This record does not assert
  that such a form will be cheap — only that it does not need a second
  grammar.
- **The method-call spelling constrains later collection operations.**
  `xs.map(…)` / `xs.filter(…)` would need argument expressions that are
  themselves functions, which this grammar has no form for. S3 does not
  create that gap — it is created by having no lambda — but it does make
  the family look like it should extend that way. Naming it here is the
  point.
- **No prefix is introduced here.** Every form this record adds puts a
  value to the left of the dot, so the prefix set stays exactly as
  DD-007 leaves it and nothing here constrains M4-Phase 7's host-state
  boundary spelling.

## Technical risk re-evaluation

- **The lexer change is the smallest part and the position change is the
  largest.** Adding six tokens is mechanical; making one admission rule
  serve five positions means the three existing per-position checkers
  (`check_if_condition`, `check_property_bind_target_in_context`,
  `check_block_statement`) have to converge on shared judgements. If
  they do not fully converge, the phase ships P2 while claiming P1, and
  the failure is silent — an author finds it, not a test. The close
  artifact that catches this is a **position × form admission matrix**
  with a firing test per cell, including the reject cells.
- **Reject tests are the load-bearing half.** Every rule above is a
  narrowing: same-type operands, ordering on `i32` only, no
  associativity, no collection comparison, no implicit display. A
  narrowing with no firing reject test is a narrowing that does not
  exist. This is the shared-lexer-helper hazard in a new place: one
  admission helper feeding several positions means a widening in the
  helper silently widens all of them.
- **`is-empty()` and `last-index()` must be total, including on an empty
  collection**, and `last-index()`'s value there (`-1`) is a sentinel.
  A sentinel that no test pins is a sentinel that drifts; DD-005 depends
  on that exact value for its empty-collection row, so the two records
  must not specify it twice.
- **T1 makes the count consumer depend on interpolation**, which already
  rejects `bool` operands and today admits only a qualified name.
  Widening the placeholder to an expression has to carry that `bool`
  reject forward rather than lose it in the wider position. A count
  interpolation is `i32`, so it is admitted, but the interaction is
  worth a test rather than an assumption.
- **Nothing here is ABI-bearing**, and nothing here changes an existing
  `.ui`'s meaning: every construct this record adds is currently a parse
  error or a named diagnostic, so no shipped example can change
  behaviour. The changes to what a legal `.ui` means in this phase are
  DD-007's, not this record's, and DD-007 also migrates the examples'
  prefix spelling. The property is worth re-checking at implementation
  close — the counter, bool-demo and gallery examples compiling with no
  edit *this record* requires is the cheapest form of that check.
