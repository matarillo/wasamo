# DD-M3-P7-002 — Collection value surface, mutation statements, and `TypedValue` pressure

**Status:** Proposed
**Phase:** M3-Phase 7
**AC:** A8 (collection binding); plan-level obligation: explicit
`TypedValue` pressure judgment

## Context

A8 requires a collection binding, but the shipped state surface has no
collection anywhere: `state_type` is `i32 | string | bool`
(`IrType::{I32, Str, Bool}`), `IrState.default` is a scalar
`IrLiteral`, the runtime `SignalRegistry` holds per-scalar-type
signals, and the handler statement grammar knows only scalar
assignment (`=`, `+=`, `-=`, `*=`, `/=`).

This DD fixes, end to end: the author-facing collection type and
literal syntax, the IR carrier, the runtime value representation, the
authored mutation path, and the two explicit judgments the plan and
framing demand — **`TypedValue` adopt-or-defer** and the
**statement-vs-expression line** against the Q5 operator-uniformity
rule (owner-intent answers §3 note 2).

Boundary fixed by FD-C / host-state-boundary.md: the collection is
**runtime-owned state**. Host-supplied initial values, host set /
replace, and write-back belong to the general host state boundary —
a separate thesis, deferred with triggers — but this DD must record
the representation constraints that keep a future host replace
unblocked (gates trap #5).

## Decision dependency summary

Primary for the **collection value representation** bundle: the
whole-value-signal choice here determines DD-M3-P7-005's reactive
positional item reads, DD-M3-P7-003's binder lowering, and
DD-M3-P7-007's mutation-statement reject matrix.

## Sub-issues

- **Type & literal surface** — type names, initial-value syntax,
  element scalar set.
- **IR carrier** — how state typing and list literals enter the IR.
- **Runtime value representation** — signal granularity, copy /
  ownership, element identity.
- **Mutation surface** — the authored append / remove path and its
  grammatical category.
- **`TypedValue` pressure** — adopt or trigger-backed defer.

## Type & literal surface

### Options

- **T1 — postfix array types: `i32[]` / `string[]` / `bool[]`**
  - ```
    state thumbs: i32[] = [101, 102, 103]
    state captions: string[] = []
    ```
  - What you gain: reads identically across the C / Rust / Zig / Swift
    / Go host audience; visibly *derived from* the existing scalar
    names, so the type table stays one table; the empty literal types
    itself from the declaration.
  - What you give up: nothing observable this phase; nesting (`i32[][]`)
    is syntactically conceivable and must be explicitly rejected.

- **T2 — generic form: `list<i32>`**
  - What you gain: a named constructor future container kinds could
    share (`map<…>`).
  - What you give up: introduces angle-bracket generic syntax into a
    DSL that has none, for one container kind with no second kind in
    sight; heavier than the thesis needs and not more expressive.

- **T3 — context-free `[i32]`**
  - What you give up: collides visually with the list *literal* and
    with future index syntax; no compensating merit over T1.

### Recommendation

**T1.** Element scalar set this phase: **`i32`, `string`, `bool`** —
homogeneous, scalar-only. `f64[]` is **deferred with the framing
trigger** (a concrete `f64`-element case: coordinates, ratios,
metrics); it is an additive fourth element later, not a shape change.
Nested collection types are rejected with a named diagnostic. The
literal is `[` scalar literals `]`, comma-separated, homogeneous,
possibly empty; literal element types must match the declared element
type at `wasamoc check`.

```
state_type         ::= "i32" | "string" | "bool"
                    |  "i32[]" | "string[]" | "bool[]"   ; M3-Phase 7
collection_literal ::= "[" (collection_scalar_literal
                       ("," collection_scalar_literal)*)? "]"
collection_scalar_literal ::= INT_LIT | STRING_LIT | BOOL_LIT
                       ; each literal must match the declared element
                       ; type; no idents/operators this phase
```

## IR carrier

### Options

- **I1 — widen `IrType` itself** (`IrType::I32List` × 3, or
  `IrType::List(Box<IrType>)`)
  - What you give up: every existing `IrType` consumer (prop typing,
    handler lowering, evaluator scalar paths) silently admits
    collections where only scalars are meaningful — the
    silent-absorption shape trap #1 warns about; `List(Box<IrType>)`
    additionally admits `List(List(_))` structurally and pushes the
    reject to validators.

- **I2 — split state typing: `IrStateType::Scalar(IrType) |
  Collection(IrType)`**
  - `IrState.ty` becomes `IrStateType`; `IrType` stays the scalar enum
    everywhere else; `IrLiteral` gains `List(Vec<IrLiteral>)` for the
    initial value (scalar-homogeneous, enforced at check / loader).
  - What you gain: changing the field type **breaks every `IrState`
    construction and match site at compile time** — the
    compile-error-forcing migration the implementation gates prefer
    over silent-absorb helpers; scalar positions (props, conditions)
    cannot type-admit a collection by construction; `Collection(IrType)`
    cannot nest.
  - What you give up: an IR-schema / textual-IR migration (full review
    lane), and `state_type` now has two layers in the IR where the
    surface shows one.

### Recommendation

**I2.** This is an **IR-schema change** (schema/IR-migration full
review lane, AGENTS.md §Implementation task gates). Textual IR: the
state production gains the three collection type tokens and a list
literal form; the loader rejects element-type mismatches, nested
lists, and a list default on a scalar state (and vice versa) as
`WASAMO_ERR_IR_MALFORMED`.

`HandlerExpr` (single unified enum — settled premise) gains the
collection forms needed by DD-003 / DD-005 / the mutation path:
a typed collection read (`ListPropRead { path, elem: IrType }` — one
variant carrying the element tag rather than three parallel variants;
the per-scalar `*PropRead` triple is a 1-element-type-per-variant
convention that would double the enum every time a container arrives,
and the element tag is data the evaluator needs anyway), the loop-local
reads (DD-003), and the mutation statements (below). Exact variant
spelling may be adjusted at implementation without reopening this DD,
provided the enum stays single.

## Runtime value representation

### Options

- **R1 — whole-value collection signals** — one signal per collection
  state holding `Vec<i32>` / `Vec<String>` / `Vec<bool>` (per-element-
  type seam, mirroring the existing per-type writer seam); any change
  marks the signal dirty; the `for` effect reads the whole value and
  diffs cardinality.
  - What you gain: one reactive identity per collection (the thing the
    `for` effect depends on); append / pop are read-modify-write on one
    cell; a **future host replace is a whole-value set through the same
    signal path** — exactly the operation R1 already implements, so the
    host boundary stays unblocked by construction; value semantics
    (copy-in / copy-out) with no element aliasing to design.
  - What you give up: per-element granularity — an in-place element
    write would dirty the whole collection. No such write path exists
    this phase, and the positional read contract (DD-005) keeps
    per-item bindings correct anyway.

- **R2 — per-element signals** (a signal per index)
  - What you gain: element-granular invalidation.
  - What you give up: signal lifecycle now tracks cardinality (create /
    dispose signals on append / pop), ordering identity gets entangled
    with reactive identity, and a future host replace must reconcile a
    signal *set* — machinery proportionate to keyed identity and
    large-N performance, both explicitly deferred theses. Premature on
    merit, not just cost.

### Recommendation

**R1 — whole-value, per-element-type, runtime-owned.** Recorded
future-compat constraints (trap #5, host-state-boundary.md): the
collection value is **value-semantic** (no element identity beyond
position; copies cross the boundary), **element identity is
positional** (consistent with DD-005's un-keyed baseline), and the
only whole-collection write operation is a full-value set — the
shape a host replace API would call. Batching and drain interaction:
a collection write is one signal write riding the existing
`BATCH_DEPTH` / drain machinery, nothing bespoke. The per-element-type
registry seam is accepted deliberately because the registry is already
per scalar type; replacing it with a type-erased cell is a registry
redesign with no Phase 7 driver, and the `TypedValue` trigger below is
the point that would reopen that asymmetry.

## Mutation surface

The gallery proof needs an authored append and an authored remove
(FD-B). Owner-intent answers §3 note 2 obliges this DD to adjudicate
the form against the Q5 rule: **operators grow uniformly across all
expression positions or not at all** — no collection-only operator
pocket.

### Options

- **M1 — overload `+=` / `-=`** (`thumbs += 104;`)
  - What you gain: no new statement grammar.
  - What you give up: `+=` is documented as arithmetic compound
    assignment; a collection overload makes one *assign_op* mean two
    unrelated things depending on LHS type, and `-=` has no coherent
    remove reading (remove value? remove last?). It is exactly the
    operator-pocket Q5 forbids, smuggled through the statement layer.

- **M2 — method-style collection statements**
  - ```
    add_thumb => { thumbs.append(next_id); }
    remove_thumb => { thumbs.pop(); }
    ```
    `collection_stmt ::= IDENT "." "append" "(" expr ")" | IDENT "."
    "pop" "(" ")"` — a new `statement` alternative beside
    `assign_stmt`, valid only on a collection-typed state.
  - What you gain: the line the owner asked to be drawn is drawn **in
    the grammar itself**: these are *handler statements* (effects on
    state), the same grammatical category as assignment — not
    expressions, so the expression grammar stays operator-free and
    uniform; `append` / `pop` name their effect; the form extends to
    future statements (`insert`, `remove-at`, `clear`) without touching
    `expr`.
  - What you give up: `.`-method syntax is new in statement position
    (the lexer already handles `.` in qualified reads); two
    *contextual* method names.

- **M4 — reserve `append` / `pop` as ordinary keywords**
  - What you gain: the reserved-keyword table stays the single
    vocabulary category; future collection methods would be easy to
    explain as globally reserved words.
  - What you give up: two common identifier spellings are removed from
    the author namespace even though parsing does not require it. Unlike
    `in`, which separates binder slots from the collection reference in
    the loop header, `append` / `pop` occur after `IDENT "." IDENT "("`
    and can be recognized without a global reservation. Reserving them
    would make the public vocabulary simpler by making the DSL less
    hospitable to author names.

- **M3 — whole-collection assignment** (`thumbs = [1, 2, 3];`)
  - What you give up as the *only* path: the proof mutation (append
    one) would have to restate the entire collection — unwritable
    without expression-level list construction from current state,
    which doesn't exist. As an *additional* path it is the future host
    replace / Q5-era surface; this phase it is rejected with a
    diagnostic naming the deferral.

### Comparison

M1 is rejected on the uniformity principle (it is the operator pocket
in disguise). M3 cannot serve the proof and is deferred as a surface.
M4 is rejected on namespace merit: parse disambiguation does not need a
global keyword, so the cost would be paid only by authors. M2 makes the
statement-vs-expression boundary *visible in the
grammar* — which is precisely what note 2 requires to be recorded —
and stays inside the family of effectful handler statements the DSL
already has.

### Recommendation

**M2.** `append(expr)` — element-type-checked against the collection;
`pop()` — removes the last element, no-op on an empty collection. The
empty-`pop` no-op is an author-facing product contract: a boundary
Remove action can be idempotent, while diagnostics stay reserved for
authoring errors rather than normal runtime boundary states. It also
avoids adding an undriven failure path to the proof, but that is a
secondary verification benefit, not the primary reason. Both lower to
new `HandlerExpr` statement variants evaluated by the runtime as
read-modify-write on the whole-value signal. `append` / `pop` are
**contextual** names (valid only in this production), not reserved
keywords — they remain usable as state / widget identifiers because
parse disambiguation does not require a global reservation. Statements
on a scalar LHS, `append` arity / type mismatches, `pop(expr)`, and
whole-collection assignment are all `wasamoc check` rejects
(DD-M3-P7-007 matrix).

Reference-shape rule: the statement LHS is intentionally a **bare
state name** (`thumbs.append(...)`), not a qualified name. This matches
the Phase 7 loop-header collection reference (DD-M3-P7-001) and keeps
new collection mutation scoped to local component state. Qualified
forms such as `root.thumbs.append(...)` are rejected with a diagnostic
rather than being parsed as a three-segment qualified assignment; the
uniform expression/reference expansion is the trigger that would
reopen this boundary.

## `TypedValue` pressure (explicit judgment)

The plan names Phase 7 as the most likely point where the M2-deferred
`TypedValue` generic value union becomes unavoidable (per-item context
/ collection element type).

**Judgment: not adopted — pressure did not materialise.** Grounds:

- The element set is the existing three scalars; R1's per-element-type
  signal seam and I2's `Collection(IrType)` tag never need a value that
  is "one of several types at runtime" — every value position stays
  monomorphic at lowering time, the same property that let M2 / M3
  defer `TypedValue` for scalars.
- The unified `HandlerExpr` carries element typing as tags
  (`ListPropRead.elem`), not as a runtime union.
- The genuine `TypedValue` driver is **structured item fields**
  (`item.filename` — record-like values), which FD-C keeps out of the
  Phase 7 surface on thesis-sequencing grounds.

**Trigger-backed defer** (framing 正本 table): structured item fields /
record-like state / a concrete app case where scalar items cannot
express the data ⇒ reopen as an M4 showcase-spec or M5
widget/data-surface DD, with the acceptance-revision path named (a
`TypedValue` adoption revises M3 acceptance — it cannot be smuggled).

## Spec content seed

dsl_spec: §state-decl gains the three collection types + list-literal
default; §3 gains `collection_literal` and `collection_stmt`; the
handler-statement section documents `append` / `pop` semantics
(including empty-`pop` no-op) and states the category explicitly:
*collection mutations are handler statements, not expressions; the
expression grammar is unchanged*. The keyword / handler-statement text
also states that `append` / `pop` are contextual method names, not
reserved keywords, so `append` remains a valid state / widget
identifier outside `collection_stmt`. Textual IR: collection state-type
tokens, `(list …)` literal, `(list-prop-read …)`, `(list-append …)` /
`(list-pop …)` statement forms, loader validation policy. Invalid
examples per DD-007.

## Forward-compat exposure

- **`f64[]`** — fourth element scalar; additive to T1 / I2 / R1.
- **Structured fields / `TypedValue`** — the recorded trigger above.
- **Host-supplied initial value / host replace / write-back** — R1's
  whole-value set is the exact operation a host replace performs; the
  deferred host-boundary DD designs the ABI, not the value model.
- **Further mutation statements** (`insert(i, x)`, `remove-at(i)`,
  `clear()`) — additive `collection_stmt` alternatives; `remove-at` /
  reorder interact with the positional identity contract and belong to
  the collection-UX wave.
- **Collection expressions** (literals in handler RHS, slices) — Q5
  uniform extension territory.
- **Loop-external collection reads** (`length`, empty checks, element
  index reads) — deferred to the Q5 uniform expression/reference
  extension; the trigger is the first concrete gallery or host-state
  case that needs to read a collection outside the `for` header or its
  loop-local binders.

## Strategic review disposition

- **Review F1 folded.** Added the reserved-keyword vs contextual-name
  option comparison and restated the reservation rule as
  parse-necessity plus author-namespace merit.
- **Review F2 folded.** Recorded the bare-state collection statement
  LHS and the qualified-form diagnostic boundary.
- **Review F3 folded.** Added a trigger-backed deferral for
  loop-external collection reads.
- **Review F4 folded.** Reframed empty-`pop` no-op around author-facing
  idempotence, leaving verification cost as secondary.
- **Review F5 folded.** Recorded why the runtime registry keeps its
  per-type seam until the `TypedValue` trigger.

## Recommendation-choice review disposition

- **Finding 3 folded.** The collection-literal grammar seed now
  constrains elements by nonterminal rather than comment only; DD-M3-P7-007
  owns the matching non-literal element reject row.

## Revision history

- Strategic owner-alignment review fold: clarified contextual method
  names, collection reference shape, loop-external reads, empty-`pop`
  merit, and registry seam asymmetry; status remains Proposed.
- Recommendation-choice review fold: changed the collection-literal
  seed from `expr` elements to scalar-literal elements and aligned the
  reject matrix; status remains Proposed.

## Technical risk re-evaluation

- **I2 is the consequential migration** of this DD: every `IrState`
  construction / match site across `wasamoc`, the textual-IR
  emitter/loader, and the runtime registry breaks at compile time —
  intended (forcing), but it makes this DD's landing a full-review-lane
  schema change with a call-site audit artifact (trap #1).
- **Two contextual method names** (`append` / `pop`) introduce the
  DSL's first method-call statement syntax; the parse is unambiguous
  (`IDENT "." IDENT "("` cannot prefix any existing statement) but the
  "contextual, not reserved" rule needs a positive test (a state named
  `append` still parses).
- **Empty-`pop` no-op** is an authored-behaviour contract; it gets a
  direct runtime test, not just a spec sentence (trap #4).
- **No ABI surface** is touched; divergence from this would trip the
  preamble's abi_spec no-touch judgment and requires owner sign-off at
  Moment 2.
