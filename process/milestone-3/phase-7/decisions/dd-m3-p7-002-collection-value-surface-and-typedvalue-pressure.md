# DD-M3-P7-002 — Collection value surface, mutation surface, and `TypedValue` pressure

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
framing demand — **`TypedValue` adopt-or-defer** and the **Q5
adjudication of the authored mutation form** (owner-intent answers §3
note 2; the form itself is owner-directed as of 2026-06-12 — the
VISION §4.1 declarative model rules out a method-statement surface,
see §Mutation surface).

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
DD-M3-P7-007's collection-assignment reject matrix.

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
reads (DD-003), and the collection-assignment forms (below). Exact variant
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
    `for` effect depends on); a tail-edit assignment is read-modify-write
    on one cell; a **future host replace is a whole-value set through the same
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
pocket. An owner correction (2026-06-12) directs the choice within
that rule: the VISION §4.1 declarative unidirectional model
(`view = f(state)`) wants a handler statement to read as **assigning
the state its next value**, not as an imperative in-place operation on
a data structure — among the options below, M3 is preferred. A second
owner review (same date) widened the space further: the
method-vocabulary axis is separated from the form axis (§Method
vocabulary), the static-literal RHS is added as **M3b** (§RHS extent),
and a clarifying note records where `;` is and is not required
(§Statement terminator).

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
  - What you gain: a statement-vs-expression line drawn in the grammar;
    `append` / `pop` name their effect; `expr` stays untouched.
  - What you give up: it introduces the DSL's **first non-assignment
    statement**. Until now every handler statement assigns a state its
    next value — the statement-level mirror of `view = f(state)`. A
    method-call statement reads as a destructive in-place operation on
    a data structure, and shipping it at the DSL's first collection
    surface would fix that imperative idiom into the public grammar
    (A12 makes it normative immediately).
  - Rejected on owner thesis (2026-06-12 correction, VISION §4.1): the
    declarative model outweighs the convenience of leaving `expr`
    untouched; the cost is paid instead as a small typed expression
    surface (M3).

- **M3 — whole-value assignment over pure collection expressions**
  - ```
    add_thumb    => { thumbs = thumbs.append(next_id); }
    remove_thumb => { thumbs = thumbs.pop(); }
    ```
    `assign_stmt` extends to a collection LHS with `=` only; the RHS
    admits the pure tail-edit expressions on the assigned state itself
    and (M3b) a static collection literal:
    ```
    assign_stmt     ::= IDENT assign_op expr          ; scalar (unchanged)
                     |  IDENT "=" collection_expr     ; M3-Phase 7
    collection_expr ::= IDENT "." "append" "(" expr ")"
                     |  IDENT "." "pop" "(" ")"
                     |  collection_literal            ; M3b — reset / clear
                     ; method receiver IDENT = the assigned state
    ```
  - What you gain: **assignment stays the only statement form** — the
    invariant "a handler statement assigns a state its next value"
    survives its first collection surface; the authored operation *is*
    the whole-value set R1 implements and a future host replace will
    perform, so surface, runtime model, and ABI future all say the
    same thing; the method expressions are pure (`pop` on an empty
    collection is the identity — a boundary Remove is idempotent by
    *function semantics*, not by a statement special case); compound
    ops never gain a collection meaning.
  - What you give up: the expression grammar gains its first
    collection-valued forms. The admission is **type-driven, not
    positional**: a collection-valued expression is admitted only
    where a collection value is expected, and the only such
    author-reachable position this phase is the collection-assignment
    RHS (state defaults stay literal-only; the `for` header stays a
    bare state name, DD-001). Operators remain absent from every
    `expr` position, so the Q5 uniformity rule is untouched — there is
    no operator pocket to fence, which satisfies answers §3 note 2
    more directly than the statement line it anticipated.
  - The blocker previously recorded against M3 ("append one" cannot be
    written without list construction from the current value) is
    resolved by exactly the two tail-edit expressions plus the static
    literal, and nothing more.

- **M4 — reserve the method names as ordinary keywords**
  - What you gain: the reserved-keyword table stays the single
    vocabulary category.
  - What you give up: common identifier spellings are removed from the
    author namespace even though parsing does not require it. Unlike
    `in`, which separates binder slots from the collection reference in
    the loop header, the method names occur after `IDENT "." IDENT "("`
    and can be recognized without a global reservation. Reserving them
    would make the public vocabulary simpler by making the DSL less
    hospitable to author names.

### Comparison

M1 is rejected on the uniformity principle (the operator pocket in
disguise). M2 is rejected on the owner's declarative-model correction:
it would introduce the DSL's first non-assignment statement and fix an
imperative in-place idiom into the public grammar at its first
collection surface. M4 is rejected on namespace merit. M3 keeps the
statement grammar pure assignment and moves the novelty into typed,
pure expressions whose admission is type-driven; the authored extent
(self-receiver tail edits + static literals) is fixed in §RHS extent.

### Method vocabulary (sub-axis, owner-directed)

The first fold of this DD chose `with` / `without-last` to make purity
visible in the name. The owner's second review separates the axes:
**purity is carried by the assignment form, not by the spelling** —
once the only admitted use is `xs = xs.append(e)`, the expression is
structurally pure whatever it is called — and the invented names carry
a real demerit of their own.

- **V1 — `with` / `without-last`:** descriptive-pure; nobody misreads
  them as in-place ops. Cost: vocabulary no host-language audience
  recognises, a hyphenated method name, and redundancy once the form
  itself is pure.
- **V2 — `append` / `pop` (owner-directed):** the vocabulary every
  C / Rust / Zig / Swift / Go reader knows, and the wording the
  framing / answers / Phase 6 handoff already use. `xs = xs.append(e)`
  has precedent in exactly this shape (Go's value-returning
  `xs = append(xs, e)`), and a *pure* `pop` returning the remaining
  collection has persistent-collection precedent (Clojure). Recorded
  risk — the cross-language false friend: in most languages `pop`
  returns the removed *element*. Mitigations: the misread that wants
  the element (`x = xs.pop()`) is already a type error (collection RHS
  on a scalar LHS); the spec states normatively that `pop()` evaluates
  to the collection minus its last element; and the bare-statement
  reject (`xs.append(a);`) points imperative-habit authors at the
  assignment form.

**Chosen: V2 — `append` / `pop`.** Contextual names (valid only inside
`collection_expr`), not reserved keywords — a state named `append`
still parses (positive test).

### RHS extent (sub-axis): M3b — static collection literals

Owner-directed addition: the assignment RHS also admits a **static
collection literal**, giving authored clear / reset:

```
clear_thumbs => { thumbs = []; }
reset_thumbs => { thumbs = [101, 102, 103]; }
```

- The cost is near zero by construction: a literal assignment is a
  whole-value set on the R1 signal; under positional identity (W2) any
  replace's *structural* delta is exactly its length delta (the C1
  cardinality math), and value changes at retained positions ride the
  V2 positional reads — the machinery DD-005 already builds for the
  host-replace future. Element typing reuses the state-default literal
  rules: homogeneous, element-type-checked against the LHS, `[]` typed
  from the LHS, no nesting, no identifiers inside.
- **FD-C boundary note (recorded).** M3b extends the authored
  capability beyond FD-C's append/truncate-only baseline to include
  the static whole-value set — an owner-directed boundary extension.
  The theses FD-C protects are untouched: no keyed diff (identity
  stays positional), no ordering contract (a replace has no reorder
  semantics under positional identity), no dynamic collection
  expressions (the literal is compile-time static).
- The equal-value no-dirty rule covers the new forms for free: a reset
  to the identical current value, like a `pop` on empty, writes an
  equal value and produces no dirty effects.

### Statement terminator (`;`) — clarifying note

Where `;` appears is unchanged by this DD and follows the shipped
grammar's single rule: **`;` terminates handler-block statements
only** (`statement ::= assign_stmt ";"`, dsl_spec §3) — including the
new collection assignment, which is an `assign_stmt` alternative. No
other position carries one: `state` declarations
(`state thumbs: i32[] = [101, 102, 103]`), widget property settings
(`label: thumb`), and property binds are members delimited by the
member grammar itself. Phase 7 adds no new `;` position and removes
none.

### Recommendation

**M3 + M3b, with `append` / `pop`.** `xs = xs.append(expr)` — a new
collection with one element appended, element-type-checked;
`xs = xs.pop()` — the collection minus its last element, the
**identity on an empty collection** (a boundary Remove action is
idempotent by function semantics; diagnostics stay reserved for
authoring errors rather than normal runtime boundary states);
`xs = <static collection literal>` — whole-value reset / clear.

Restrictions, each a `wasamoc check` reject (DD-M3-P7-007 matrix):

- **RHS extent.** The RHS must be a single tail-edit application whose
  receiver is the assigned state itself, or a static collection
  literal: not `xs = ys.append(a)`, not `xs = xs.append(a).append(b)`,
  not a bare state copy `xs = ys`, not identifiers inside a literal.
  Everything wider is rejected with a diagnostic naming the Q5 /
  host-replace deferral.
- **`=` only on a collection LHS** — compound assign ops have no
  collection meaning.
- Collection RHS on a scalar LHS and vice versa; `append` arity /
  element-type mismatch; `pop(expr)`; literal-RHS typing violations
  (the state-default rules applied in assignment position); a
  `collection_expr` outside collection-assignment RHS; a bare
  collection expression as a statement (`xs.append(a);` — the
  diagnostic points at the assignment form); qualified LHS or receiver
  (`root.xs = …`) — the DD-001 reference-shape boundary, reopened by
  the uniform expression/reference expansion.

The forms lower to `HandlerExpr` expression variants evaluated by the
runtime as read-modify-write (or, for the literal, a direct
whole-value set) on the whole-value signal — R1, drain, and the
DD-007 cap accounting are unchanged. **A collection assignment whose
new value equals the current value performs no dirty propagation**
(value-equality check on the signal set; `Vec` equality is O(N), a
non-axis at gallery N). This generalises the former empty-`pop`
special case: `xs = xs.pop()` on an empty collection writes an equal
value and produces no dirty effects — pinned by a direct runtime test.

Reference-shape rule: the assignment LHS and the method receiver are
intentionally **bare state names**, matching the Phase 7 loop-header
collection reference (DD-M3-P7-001) and keeping new collection
mutation scoped to local component state.

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
default; §3 gains `collection_literal` and the collection-assignment
extension of `assign_stmt` (`collection_expr`, including the literal
alternative); the handler-statement section documents the collection
assignment — the pure `append` / `pop` expressions, **stating
normatively that `pop()` evaluates to the remaining collection, not
the removed element** (the cross-language false friend), the
static-literal reset / clear, the `pop`-on-empty identity, and the
equal-value no-dirty rule — and states the category explicitly:
*collection mutation is whole-value assignment; the method forms are
pure expressions; assignment remains the only statement; the
expression grammar gains no operators*. The keyword / handler-statement
text also states that `append` / `pop` are contextual method names,
not reserved keywords, so they remain valid state / widget identifiers
outside `collection_expr`. The handler-statement section states the
terminator scope plainly: `;` terminates handler-block statements
only; member positions (state declarations, property settings) carry
none. Textual IR: collection state-type tokens, `(list …)` literal,
`(list-prop-read …)`, `(list-append …)` / `(list-pop …)` expression
forms inside the assignment, loader validation policy. Invalid
examples per DD-007.

## Forward-compat exposure

- **`f64[]`** — fourth element scalar; additive to T1 / I2 / R1.
- **Structured fields / `TypedValue`** — the recorded trigger above.
- **Host-supplied initial value / host replace / write-back** — R1's
  whole-value set is the exact operation both the authored assignment
  and a future host replace perform; the deferred host-boundary DD
  designs the ABI, not the value model.
- **Further edit expressions** (`insert(i, x)`, `remove-at(i)`) —
  additive `collection_expr` alternatives; positional removal /
  reorder interact with the positional identity contract and belong to
  the collection-UX wave.
- **Collection expressions** (literals in handler RHS, slices) — Q5
  uniform extension territory.
- **Loop-external collection reads** (`length`, empty checks, element
  index reads) — deferred to the Q5 uniform expression/reference
  extension; the trigger is the first concrete gallery or host-state
  case that needs to read a collection outside the `for` header or its
  loop-local binders.

## Revision history

- Strategic owner-alignment review fold: clarified contextual method
  names, collection reference shape, loop-external reads, empty-`pop`
  merit, and registry seam asymmetry; status remains Proposed.
- Recommendation-choice review fold: changed the collection-literal
  seed from `expr` elements to scalar-literal elements and aligned the
  reject matrix; status remains Proposed.
- Implementation-readiness review fold: specified empty-`pop` as no
  signal write / no dirty effects; status remains Proposed.
- Owner-direction fold (2026-06-12, declarative mutation surface):
  recommendation moved from M2 method statements to M3 whole-value
  assignment over self-receiver pure tail-edit expressions (`with` /
  `without-last`), grounded in the VISION §4.1 declarative
  unidirectional model; the empty-remove no-dirty contract is preserved
  as an equal-value write-suppression rule; status remains Proposed.
- Owner-review fold (2026-06-12, second pass — options-space widening):
  vocabulary axis split from the form axis and moved to `append` /
  `pop` as pure expression names (false-friend risk recorded);
  static-literal RHS admitted as M3b (FD-C boundary extension
  recorded); clarifying note added on the statement-terminator scope
  (`;` terminates handler-block statements only); status remains
  Proposed.

## Technical risk re-evaluation

- **I2 is the consequential migration** of this DD: every `IrState`
  construction / match site across `wasamoc`, the textual-IR
  emitter/loader, and the runtime registry breaks at compile time —
  intended (forcing), but it makes this DD's landing a full-review-lane
  schema change with a call-site audit artifact (trap #1).
- **Two contextual method names** (`append` / `pop`) introduce
  the DSL's first method-call *expression* syntax, admitted only in
  collection-assignment RHS; the parse is unambiguous (the RHS shape
  `IDENT "." IDENT "("` follows `=` on a collection-typed LHS) but the
  "contextual, not reserved" rule needs a positive test (a state named
  `append` still parses). The `pop`-returns-collection semantics is a
  documented false friend; the spec sentence and the scalar-LHS type
  reject are the guardrails.
- **The equal-value no-dirty rule** (a remove on an empty collection
  writes an equal value) is an authored-behaviour contract; it gets a
  direct runtime test that also proves no dependent effect re-runs, not
  just a spec sentence (trap #4). The collection signal set gains a
  value-equality check (O(N) — a non-axis at gallery N).
- **No ABI surface** is touched; divergence from this would trip the
  preamble's abi_spec no-touch judgment and requires owner sign-off at
  Moment 2.
