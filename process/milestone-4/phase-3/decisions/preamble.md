# M4-Phase 3 — Predicate expressions: Architecture Decisions

**Phase:** M4-Phase 3 (predicate expressions)
**Date:** 2026-08-12
**Status:** Proposed (DD-001 … DD-007 all Proposed; no owner accept yet)

## Context

M4 acceptance criterion **AC9** (see
[../../../_roadmap.md M4](../../../_roadmap.md#m4-interaction-stack),
[../../plan.md §Acceptance criteria](../../plan.md)):

> **AC9** — Expression predicates: reading a collection from outside the
> repetition (count, emptiness, index access), per-item conditional
> rendering, equality-based selection, and a small reusable
> handler-control-flow surface sufficient to guard a state write at
> collection boundaries. Every handler assignment is checked before
> execution for expression-position admission and LHS / RHS type
> compatibility. Scalar `string` write capability remains with
> M4-Phase 5. String concatenation and general arithmetic stay outside
> M4.

The phase's binding inputs are the accepted
[../requirements/framing.md](../requirements/framing.md) (owner-agreed
2026-08-12, §2.1–§2.4 complete) and
[../requirements/constraints.md](../requirements/constraints.md), on top of
the M4 plan as revised by Revisions 3–5 (`7763555`, `4afa204`, `1499241`).

Three properties of this phase shape the whole set.

- **It is the first phase to grow the expression grammar.** Every
  `expr` position in the language today admits literals, name
  references, string interpolation, and — inside a `for` body — loop
  binders. **No operator is admissible anywhere**, and
  [dsl_spec.md §4.6](../../../../docs/dsl_spec.md) states that the
  deferred extension "grows across all `expr` positions at once, not
  condition-only". That standing sentence is a constraint on DD-001's
  option space, not a background remark.
- **It is not a compiler-only phase.** Per-item conditional rendering
  creates and destroys widgets, effects and handlers inside a
  repetition, so it crosses the focus, hover, handler-registry and
  layout lifecycles Phases 1 and 2 established (plan Revision 3).
- **It has to close a completeness contract, not add cases.** AC9's
  handler-assignment clause is satisfied by a mechanism whose coverage
  can be audited, not by rejecting the three right-hand sides Phase 2
  happened to measure (plan Revision 5).

### The starting state (measured against the workspace at drafting time)

Every claim below was produced by running the current `wasamoc` against a
probe `.ui`, or by reading the named call path — not inferred from the
documents.

- **No comparison operator exists at any level.** The lexer has no
  `==` / `!=` / `<` / `>` token; `if index == sel { … }` fails in the
  parser with `expected \`{\`, found \`=\``. `!cond` parses to
  `Expr::UnsupportedOperator` and is rejected in check.
  `check_if_condition` admits a bool literal or a bool-typed state
  identifier and nothing else, and explicitly rejects a loop binder.
- **Two spellings this phase needs are already reserved by named
  diagnostics.** `xs[i]` is recognised by the parser and rejected with
  "indexed reads (`xs[i]`) are deferred in M3-Phase 7"; a bare or
  navigated collection read outside iteration is rejected with
  "collection reads outside iteration not yet supported". Neither is a
  grammar gap; both are placeholders with a message.
- **A method-call expression form already exists** (`Expr::CollectionCall`,
  `xs.append(v)` / `xs.drop-last()`), parsed, checked and lowered.
- **A dotted prefix is discarded, not validated.** `check_qualified_name`
  resolves the **last** segment as the state name and drops the rest, so
  `photos.count` and `a.b.c.count` both compile as a read of the state
  `count` while `root.nope` fails with "undefined state `nope`". No
  document defines the identifier `root`: it appears only inside code
  examples, and in the spec's prose the word denotes the **content root
  widget** instead. Lowering discards the prefix — `root.count` emits
  `(prop-read count)` — so no representation or ABI surface carries it.
- **The runtime already stringifies an integer binding into a string
  property** (`handler::evaluate_binding`'s fall-through arm), but
  `wasamoc check` rejects it: `Text { text: root.count }` produces
  "type mismatch in binding `Text.text`: target is `string`, source is
  `i32`". The blocker for a bound count display is the checker's type
  rule, not the writer.
- **A per-item conditional compiles today and its runtime behaviour is
  wrong.** `if <bool state> { Text { text: "\{label}" } }` inside a
  `for` body passes `wasamoc check` and the loader's validation, and
  emits `if (bool-prop-read flag) { child { node Text { bind text =
  (interp ((item-read label))) } } }`. On the runtime side
  `append_static_member`'s `If` arm does not forward its
  `loop_context` parameter, `register_conditional_binding` builds a
  `BindingEvalContext`, and `EffectHandle::new` runs the effect
  immediately — so the subtree is built by `build_node` with no loop
  context at **every** materialisation, including the first, and the
  child's `set_loop_scope` is written `None`. Binder reads inside a
  per-item conditional therefore cannot resolve, in bindings or in
  handlers.
- **Handler assignments are checked without an expected type.**
  `check_block_statement`'s scalar branch validates the right-hand side
  in isolation through `check_expr_type_in_loop_context`, which never
  receives the left-hand side's declared type. Measured: `i32 = "many"`,
  `string = "New"`, `string = 5`, `i32 = true` and `string += "x"` all
  pass `wasamoc check` with exit 0.
- **The structural-mutation seam already exists and is single.** The
  conditional and `for` mutation paths mark layout dirty through
  `mark_layout_dirty_for`; `emit::flush_layout` is the one place focus
  is rebased and modal scopes reconciled against the new tree
  (`focus::sync_scopes_to_tree`), and `sync_visuals` remains the single
  Composition geometry pass.
- **The per-item bool binding path already exists.**
  `register_for_item_bool_binding` / `evaluate_bool_binding_optional`
  were added at M4-Phase 2 for `checked` inside a `for` body; what is
  missing for `checked: index == selected_index` is the expression, not
  the writer.
- **The gallery's thumbnails are `Box`, not `ToggleButton`.**
  `examples/gallery/gallery.ui` builds each thumbnail as
  `Box { aspect: 1:1 fill: #4f6272 … }`. `Box.fill` is constant-only
  (DD-M3-P2-004) and `checked` is admitted on `ToggleButton` only
  ([dsl_spec.md §4.17](../../../../docs/dsl_spec.md)); `ToggleButton`
  resolves to `FocusRole::Stop`, so changing the thumbnail's widget kind
  would also change the window's Tab traversal.

### What the phase inherits as settled

Not re-litigated here
([../requirements/constraints.md](../requirements/constraints.md)):
iteration is positional and un-keyed; the element binder carries the
collection's element type and the index binder is a zero-based `i32`;
every Composition geometry write happens in `sync_visuals` alone; the
per-node scale cache and the layout-derived `arranged_rect` have one
writer each; focus presentation after a structural update goes through
the single focus writer repaired at M4-Phase 2 T13a; and no new C ABI
entry point, value carrier or host callback is created.

## Summary of decisions

Numbers are identities, not a reading order. **DD-007 is upstream of the
rest** — it settles what a dot means, which is what frees the spelling
DD-001 chooses — so it is listed first and the remaining six follow in
the order they consume one another.

| DD | Question | Recommendation |
|---|---|---|
| [DD-007](dd-m4-p3-007-dot-meaning-and-prefix-set.md) | What may stand to the left of a dot, and does the `root.` prefix survive? | **The left of a dot is a value; the exceptions are a closed, validated set of prefixes** — `slot` on the placement-key side and `root` on the expression side (**N2**), with at most one prefix per name. A non-member in prefix position becomes a diagnostic naming the members, which frees `photos.count` for DD-001 to decide on merit. `root` is **retained and validated**; whether the expression-side set should be emptied (**N1**) is handed to M4-Phase 7, which decides whether the set gains a host member. Making a component instance a value (**N3**) is rejected: it collides with the spec's existing use of "root" for the content root widget and promises more than anything needs. No authored `.ui` changes; the prefix stays out of the IR |
| [DD-001](dd-m4-p3-001-predicate-surface-and-typing.md) | Which predicate forms exist, where may they be written, and what are their types? | **One comparison level added to the shared `expr` grammar**, admissible in every position that already takes a scalar expression (option **P1**); collection interrogation spelled as **method calls** on the existing `CollectionCall` form (**S3**) and element access as **`xs[i]`** (**X1**); comparison operands must be the **same scalar type**, ordering restricted to `i32`, result `bool`; **no implicit integer-to-string display** in a bound string property (**T1**); carried as **new `HandlerExpr` variants** (**C1**), no second expression tree and no `TypedValue` |
| [DD-002](dd-m4-p3-002-collection-read-and-failure-contract.md) | How is a collection read from outside the repetition, and what happens when the index is out of range? | `xs.count()`, `xs.is-empty()`, `xs.last-index()` are **total**; `xs[i]` is **partial**. An out-of-range index is an **error, not a value** (**R2**): the binding writes nothing, the target keeps the value it last held, the failure is **contained to the failing effect** (**C-a**), and the diagnostic goes to the existing runtime channel. Fallback values and clamping are rejected because both render an author's mistake as a plausible screen |
| [DD-003](dd-m4-p3-003-per-item-conditional-and-lifecycle.md) | How does a condition inside a `for` body read its binders, and what owns the subtree's lifecycle? | **Thread the enclosing `ForItemContext` through the three seams that drop it today** (**L1**) — `append_static_member`'s `If` arm, `register_conditional_binding`, and `mutate_conditional_subtree` — so condition evaluation and re-materialisation share one owner, and the subtree's `set_loop_scope` is written from the same parameter. **Repair the already-admitted composition rather than reject it** (**H-a**). Reuse the existing `mark_layout_dirty_for` → `flush_layout` seam unchanged; add no structural writer |
| [DD-004](dd-m4-p3-004-equality-selection.md) | How does one discriminant produce exactly one selected item? | At the language level, **nothing beyond DD-001**: a `bool`-valued comparison is admissible wherever a `bool` expression is, so both `checked:` and a conditional marker subtree are legal projections (**V3**). For the **shipped gallery consumer**, project through **DD-003's conditional marker** (**V2**) and leave the thumbnail a `Box`, because turning 18 thumbnails into `ToggleButton`s would add 18 focus stops and assert a toggle semantics the thumbnail does not have. An invalid discriminant means **zero items selected**, which is not a diagnostic and is not DD-002's contract |
| [DD-005](dd-m4-p3-005-handler-control-flow.md) | What is the smallest reusable way to make a handler's write conditional? | **`if <bool-expr> { <statements> }` as a handler-body statement, single branch, no `else`** (**F-1**), plus the **relational operators and `last-index()`** that the four-producer boundary table actually requires (**G-J**). A false guard writes nothing and the handler still **consumes** the event; each guard is evaluated when its statement is reached, against live state. Arithmetic — the conventional way to write the upper bound — is excluded by AC9, and `last-index()` is named as the price of that exclusion |
| [DD-006](dd-m4-p3-006-handler-assignment-validation.md) | How is every handler assignment made to pass admission and type checking? | **Two total functions over the AST** — a position-capability judgement and a result-type judgement — with **exactly one gate call site per assignment form**, so a new expression variant is a compile error rather than a silent hole (**M1**). Capability and compatibility stay separable, so a type-correct scalar `string` write is refused as a **missing capability** while `i32 = "abc"` is refused as a **type mismatch**, and the type message wins when both apply. `wasamoc check` is the author-facing gate; the loader **dual-gates** the same two invariants, because the memory-IR entry point never passes through `wasamoc` |

## Out of scope

Per [../requirements/framing.md](../requirements/framing.md) §含まないもの,
sent onward rather than absorbed: string concatenation and general
arithmetic (outside M4); logical operators, `else`, early return, loops,
general functions and arbitrary commands in a handler body; `TypedValue`,
structured item data and field access; keyed identity, nested `for`,
binder shadowing and multi-widget / member-range `for` bodies; new
selection widgets, widget-owned selection state, generic toggle
appearance and `Button` Space / Enter activation; two-way binding
(M4-Phase 7) and the scalar `string` write capability (M4-Phase 5); a
bindable `Box.fill` ([dsl_spec.md §8.12](../../../../docs/dsl_spec.md)
defers it to the phase that first needs reactive fill); a structured
runtime diagnostic channel, which is ABI-adjacent; any C ABI entry point,
value carrier or host callback; and any redesign of Phase 2's routing,
focus, modal-scope, hit-test or identity policy.

DD-007 sends two further questions onward rather than answering them,
both to **M4-Phase 7**: the host-state prefix spelling, and the prefix
set's final form — whether the expression-side set is emptied and whether
its members carry a reserved symbol. The second is backed by a
[pre-1.0 candidate pool](../../../candidate-pool.md) row, because its
trigger depends on a negative outcome that Phase 7 would leave
undocumented.

Two exclusions are **downstream of what this set chooses**, and framing
agreement ⑬ makes not-foreclosing them a judgement requirement rather
than a scope extension: multi-level `for` and nested structural control
flow depend on DD-003's loop-context ownership, and the Phase 5 scalar
`string` write and later value-producing expressions depend on DD-006's
admission framework. Each record states what a later addition would cost
and whether it would change the meaning of an existing `.ui`.

## Pre-ADR spike assessment

Framing agreement ⑥ admits a narrow spike only where one of five firing
conditions is met by evidence rather than by suspicion. Each was tested
against the measurements in §Context; **none fired**, so this set
proposes no pre-ADR spike.

| Firing condition | Disposition |
|---|---|
| 1 — type-representation wall | Not fired. Count, emptiness, last index and index are `i32` / `bool` / element-typed; equality is `bool`. `collection_len_tracked` already computes a length per element type, and every result lands in the existing three scalars. No `TypedValue` and no second expression tree is needed to compare the candidates |
| 2 — dependency-tracking wall | Not fired. `Signal::get()` registers one edge per signal read inside the running effect, and an effect that reads several signals already exists (a two-reference interpolation binding). Whether a binding reading a collection **and** an index re-runs on either change is answerable from `reactive.rs` and its existing unit tests |
| 3 — structural-update wall | Not fired, and its premise is now stronger evidence rather than weaker. The missing piece is not an unknown mechanism but a parameter dropped at three named call sites, and the seam a per-item conditional would use (`mark_layout_dirty_for` → `flush_layout` → `sync_scopes_to_tree`) is the one the existing conditional already uses. The Phase 2 recurrence conditions are decided by call-path audit in DD-003, which is ADR investigation, not a spike |
| 4 — guard expressiveness wall | Not fired. The four-producer × cardinality table in DD-005 is decidable on paper, and it does decide: it eliminates equality-only, and it separates the two candidates that close it. No IR or evaluator question is left carrying the comparison |
| 5 — contradiction with prior measurement | Not fired. The binding-only string right-hand side behaves as the Phase 2 handoff recorded, and the legitimate `string[]` append with a loop binder is admitted as documented. The new measurements (`i32 = true`, `string += "x"`) widen the known gap; they do not contradict it |

## Revisions

*(none — initial draft)*
