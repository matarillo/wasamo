# DD-M4-P3-006 — Completeness of handler-assignment admission and type checking

**Status:** Proposed
**Phase:** M4-Phase 3
**AC:** AC9 ("every handler assignment is checked before execution for
expression-position admission and LHS / RHS type compatibility");
phase-end criterion 4 (spec synchronization)

## Context

Plan Revision 5 (`1499241`) made this a completeness contract rather
than a defect list, on the owner's explicit requirement: the expectation
is "not to plug known cases ad hoc, but to establish a mechanism that
covers every assignment without omission".

Phase 2 measured three escapes. This drafting measured five, which is
the point — the set was never the specification.

| Probe | `wasamoc check` today |
|---|---|
| `root.count = "many"` (`i32` ← string literal) | **accepted** |
| `root.caption = "New"` (`string` ← string literal) | **accepted** |
| `root.caption = 5` (`string` ← int literal) | **accepted** |
| `root.count = true` (`i32` ← bool literal) | **accepted** |
| `root.caption += "x"` (compound assign on a `string` state) | **accepted** |

All five pass with exit `0`. Two of them are outside the framing's
example list, and one — compound assignment on a `string` — is a
different axis altogether: `dsl_spec.md` §4.6 / §8.9 state that compound
assignment is not defined over `bool` and not defined over collections,
and say nothing about `string`, while `check_block_statement` rejects
compound assignment only on a **collection** left-hand side.

The cause is structural and visible in one line of the checker. For a
scalar left-hand side, `check_block_statement` calls
`check_expr_type_in_loop_context` — a function that validates the
right-hand side **in isolation** and is never told what type the
destination is. There is no path by which the left-hand side's declared
type reaches the right-hand side's judgement, so no amount of
per-variant rejection can produce the invariant AC9 asks for.

Two judgements have to stay separate, and the framing (agreement ④)
requires it: **may this right-hand side appear in handler position at
all** (capability), and **does its type match the destination**
(compatibility). They are independent — `caption = "New"` is
type-correct and capability-blocked; `count = "many"` is
capability-blocked and type-wrong; `count = true` is capability-fine and
type-wrong.

This phase does **not** add the scalar `string` write. That capability
is M4-Phase 5's, and plan Revision 5 restated Phase 5's paragraph so it
consumes this invariant rather than reopening it.

### What exists to build on (measured)

- **The property-binding path already does what the handler path does
  not.** `check_property_bind_target_in_context` looks up the target
  property's declared type and compares it against
  `expr_static_type_in_context`, producing "type mismatch in binding
  `Text.text`: target is `string`, source is `i32`". The machinery for
  an expected-type comparison exists; it is simply not wired into
  assignment.
- **The collection-assignment path is already complete-ish.**
  `check_collection_assignment_rhs` takes the declared element type,
  rejects a foreign receiver, checks `append`'s element type, and
  rejects bare collection copies. This is the shape the scalar path
  lacks.
- **The evaluator rejects at invocation.** `evaluate` returns
  `TypeMismatch` for a string-typed form in integer context and
  `UnknownProperty` from `set_bool` against an `i32` signal, so the
  runtime is safe today — the failure is that the author is told at
  click time instead of at check time.
- **The loader already resolves assignment targets.**
  [dsl_spec.md §8.11](../../../../docs/dsl_spec.md) records that every
  `assign` / `compound-assign` name is validated against a declared
  `state` at load, and simultaneously records that "binding expression
  result type matches target property type" is **not** enforced and is
  trusted from `wasamoc`.
- **`string[]` mutation with a loop binder is legitimate and must keep
  working**: `root.archive = root.archive.append(label)` inside a `for`
  body is an admitted form, and a rule of the shape "reject a string
  anywhere in the right-hand side" would break it.

## Sub-issues

- **Where the invariant is defined**, such that no assignment path can
  avoid it.
- **How capability and compatibility stay separable** without being
  checked twice or in two places.
- **Which gates enforce it**: `wasamoc check`, lowering, the runtime
  loader, the evaluator.
- **Diagnostic priority** when both judgements fail.
- **Diagnostic wording** that does not depend on an internal schedule.
- **Non-regression** for the forms that are legitimate today.
- **The extension point** for M4-Phase 5's scalar `string` write and for
  later value-producing expressions (framing agreement ⑬).
- **The auditable artifact** that makes "complete" checkable rather than
  claimed.

## Options

### Mechanism

- **M1 — two total functions plus one gate call site per assignment
  form.** A capability judgement and a result-type judgement, each an
  exhaustive `match` over the expression AST; every assignment form
  routes through a single gate that takes the destination type as a
  parameter. A new expression variant is a compile error in both
  functions.
- **M2 — a capability table.** Variant → row describing admitted
  positions and result type, consulted by one gate. Data-driven and
  readable, but a missing row is a lookup miss at run time rather than a
  compile error, so completeness depends on discipline.
- **M3 — per-variant rejects added to the existing per-position
  checks.** The status quo, extended. Rejected by AC9 in its own words.
- **M4 — a typing pass producing a typed AST**, with assignment
  checking a by-product. The most thorough and the largest; it would
  also absorb DD-001's typing rules into a new pass.

### Enforcement gates

- **E-check — `wasamoc check` only**, with the loader continuing to
  trust types per §8.11's existing policy.
- **E-dual — `wasamoc check` plus loader dual-gating** of both
  judgements, matching how every spec invariant added since M3-Phase 2
  is treated.
- **E-eval — rely on the evaluator**, the status quo. Listed to be
  rejected: it is what AC9 exists to end.

### Diagnostic priority when both fail

- **P-type — report the type mismatch.**
- **P-cap — report the capability violation.**
- **P-both — report both.**

### Compound assignment applicability

- **A-i32 — compound assignment on `i32` only.** `bool` and collections
  are already rejected; `string` joins them.
- **A-status — leave `string` compound assignment accepted at check**
  and let the evaluator refuse it.

## Comparison

### Mechanism: M1

M4 is the right answer for a language with a real type system and the
wrong answer for this phase. It would rewrite how every expression
position is checked, in the same phase that is adding six operators,
three collection reads, an element access, a per-item structural
capability and a handler statement form. Its benefit over M1 —
propagated types available everywhere — is a benefit DD-001's positions
do not currently need, since each position knows the type it expects.

M2 reads well and fails on the one property AC9 asks for. A table whose
lookup returns `Option` cannot distinguish "this variant is deliberately
not admitted" from "nobody added a row", and the second is exactly the
failure mode the phase is closing. A table implemented as a total
`match` is M1 with a different surface.

M1's completeness argument is mechanical rather than procedural, which
is what makes it auditable: the two functions are exhaustive matches, so
adding an AST variant fails to compile until both are updated; and the
gate takes the destination type as a **parameter**, so an assignment
path that does not have a destination type cannot call it. The audit
then reduces to a claim a reviewer can check by reading: *there is
exactly one gate call site per assignment form, and the list of
assignment forms is complete*.

The second half of that claim is where the real work is, and it has
grown during this phase: the assignment forms are scalar `=`, scalar
compound `=`, collection `=`, and — new in DD-005 — any of those
**nested inside a guard body**, which the gate must descend into exactly
as it descends into a block. A completeness contract that stops at the
top level of a handler body would be defeated by the construct this
phase adds.

### The separation, and why it is not two checks in two places

Capability answers "may this expression form be the right-hand side of a
handler assignment at all", and it is a property of the **form**, not of
the types involved: `StrLit`, `StrPropRead` and `Interpolation` are
binding-only by §8.9, and remain so until Phase 5 admits them for a
`string` destination. Compatibility answers "does the form's result type
equal the destination's declared type".

Both are consulted at the same gate, in a fixed order, with a single
diagnostic emitted. This is what keeps them separable **as judgements**
without being separate **passes**: DD-001 owns what an expression's
result type is, this record owns what a destination accepts, and the two
meet once.

### Gates: E-dual

E-check has the weight of the existing §8.11 sentence behind it —
"binding expression result type matches target property type: **No**
(trusted from `wasamoc`)". That row is about **bindings**, and it is the
one row of its kind; every spec invariant added from M3-Phase 2 onward
(Box child count, ratio sign, WrapPanel ranges, ScrollView children,
Grid tracks and placement, ZStack payloads, control-flow branch/body/
condition, collection state and `for` shapes, collection assignment) is
dual-gated, and §8.11 gives the reason: `wasamo_load_ui`'s memory-IR
entry point never passes through `wasamoc`.

AC9's own words settle it. "Every handler assignment is checked
**before execution**" is a claim about executions, not about
compilations, and a host that hands the runtime an in-memory IR is an
execution path with no `wasamoc` in it. Under E-check the claim would be
true of `.ui` authors and false of that host.

E-dual's cost is real and is named rather than waved at: the loader must
know each expression's result type, and one form makes that non-trivial.
`ItemRead`'s type comes from the enclosing `for`'s element tag, not from
the expression itself, so the loader's validation walk has to carry the
element type down into `for` bodies — which it is already positioned to
do, since it already tracks whether it is inside a `for` template and
already reads the `ListPropRead { elem }` tag. `IndexRead` is always
`i32`, and every other form carries its type in its own variant. This is
an implementation-plan item with a known shape, not an open question,
and it is the only part of E-dual that is more than a re-application of
the existing rule.

### Diagnostic priority: P-type

P-both produces two messages for one mistake and forces the author to
decide which matters — which is the compiler's job.

Between P-type and P-cap, consider the case that actually arises.
`root.count = "many"` is both a type mismatch and a binding-only form in
handler position. P-cap would tell the author "string expressions cannot
be written from a handler", which is true and **misleading**: the
author's bug is almost certainly that they meant a different destination
or a different value, and the message points them at a capability that
would not help them if it existed. P-type says the destination is `i32`
and the value is a string, which is the correction.

The reverse case — type-correct and capability-blocked, like
`root.caption = "New"` — has no competition: only the capability
judgement fails, so only it is reported.

So P-type is not "types are more important"; it is "when both fire, the
type is the more fundamental correction, and the capability message is
the one that misleads".

### Diagnostic wording

The framing requires wording that does not depend on an internal
schedule and that describes what the author can do now. A capability
message therefore says that writing a `string` state from a handler is
not available and points at the binding form that is, without naming a
phase. A phase number in a compiler diagnostic is a promise the compiler
cannot keep.

### Compound assignment: A-i32

A-status is the status quo and it leaves a hole of the same kind the
record exists to close — a form that passes check and fails at
invocation. `string += "x"` has no meaning to give it: string
concatenation is outside M4 by AC9, so the operator has no defined
behaviour on a `string` destination.

A-i32 is a **narrowing of currently-accepted input**, which is the one
place this record can break something. The check is cheap and was done:
the three shipped examples use compound assignment only on `i32`
(`root.count += 1` in the counter, `root.scroll_y += 100` and
`root.selected_index -= 1` in the gallery). No shipped `.ui` regresses.

### Non-regression

The forms that must keep working are not an afterthought; a
completeness rule that over-rejects is worse than the gap it replaces,
because it breaks working programs. The named ones:
`root.archive = root.archive.append(label)` inside a `for` body (a
collection whole-value assignment whose right-hand side contains a
string binder), `root.xs = root.xs.drop-last()`, a static list literal
assignment, bool assignment from a literal or a bool state, `i32`
assignment and compound assignment, and — new this phase — any of those
inside a guard body. Each is a positive control, not merely a
non-regression note.

## Recommendation

- **M1** — two total functions over the expression AST:
  - a **capability** judgement, exhaustive over expression variants,
    answering whether a form may be a handler-assignment right-hand
    side;
  - a **result-type** judgement, exhaustive over expression variants,
    answering what type a form produces (DD-001 owns the rules; this
    function is where they are read from).

  Every assignment form routes through **one gate** that takes the
  destination's declared type as a parameter. There is no path to a
  handler assignment that does not pass it.
- **The assignment forms are enumerated and the enumeration is part of
  the contract**: scalar `=`, scalar compound `=`, collection `=`, and
  each of those nested at any depth inside a guard body (DD-005) or a
  block.
- **Capability and compatibility are consulted at that one gate**, in a
  fixed order, emitting a single diagnostic.
- **E-dual** — `wasamoc check` is the author-facing gate and the runtime
  loader re-checks both judgements, consistent with every spec invariant
  since M3-Phase 2 and with AC9's "before execution". The loader's
  element-type threading for `ItemRead` is named as the one
  implementation subtlety.
- **The evaluator's existing rejections stay** as the last line of
  defence. They stop being author-facing, and they are not removed:
  removing them would make the two upstream gates load-bearing for
  memory safety rather than for diagnostics.
- **P-type** — when both judgements fail, the type mismatch is reported.
  When only capability fails, the capability message is reported, and it
  describes what is available rather than when something will be.
- **A-i32** — compound assignment is admitted on `i32` destinations
  only. `bool` and collections keep their existing rejections; `string`
  joins them. No shipped example regresses.
- **This phase adds no write capability.** A type-correct scalar
  `string` assignment is refused as a missing capability, and the
  refusal is the thing M4-Phase 5 removes.
- **The close artifacts are two tables**: an **expression variant ×
  assignment form** matrix with admit / type-mismatch /
  capability-mismatch outcomes and a firing test per row, and a
  **call-site audit** showing one gate call per assignment form with the
  form enumeration checked against the AST and IR definitions rather
  than against memory.
- **`docs/dsl_spec.md` moves**: §4.5 / §4.6 (what a handler assignment
  admits and the two judgements), §8.9 (the binding-only rows stated as
  a capability rule), §8.11 (the loader rows this record dual-gates).

## Forward-compat exposure

- **M4-Phase 5's scalar `string` write is a registration, not a new
  branch.** It adds `string` to the writable destination set and moves
  three forms from binding-only to handler-admitted in the capability
  function. It does not add a call site, and it does not touch the
  result-type function.
- **A later value-producing expression** gets a result type from the
  same function and is admitted by the same comparison. What this
  record can honestly claim is bounded, and the bound is worth stating
  precisely, because framing agreement ⑬ asks for exactly this judgement:
  **adding an expression variant still requires editing both total
  functions.** The property is not that no edit is needed; it is that
  the edit is *forced by the compiler* and lands in *two known places*,
  rather than being a branch that must be remembered at each of several
  call sites. That is registration in one place, and it is what makes
  the completeness claim survive the next addition instead of being a
  snapshot at landing time.
- **Two-way binding (M4-Phase 7)** introduces a destination that is not
  a component state. That is a new *destination* kind, not a new
  right-hand side, so it extends the gate's parameter rather than its
  matches — and this record does not claim that extension is free, only
  that it is on a different axis from the ones fixed here.
- **A structured item type would change what a result type is.**
  `TypedValue` is M4-Phase 7's decision; if it lands, the result-type
  function's return becomes richer and every row is revisited. Nothing
  here forecloses it and nothing here makes it cheap.
- **The loader's dual-gate is a policy precedent.** After this record,
  §8.11's "trusted from `wasamoc`" applies to strictly less than it does
  today, and the remaining trusted rows are the per-node emitter
  invariants. A later phase that wants to add a trusted-only invariant
  has to argue against a narrower default.

## Technical risk re-evaluation

- **Over-rejection is the failure mode with a shipped consumer.** The
  gallery's `string[]` append with a loop binder, and the three
  examples' compound assignments, are the programs that break if the
  capability rule is written as "no string anywhere in the right-hand
  side" or if the compound-assign narrowing catches `i32`. These are
  positive controls that must be in the matrix as **admitted** rows, not
  assumed.
- **The completeness claim is only as good as the form enumeration.**
  Exhaustive matches guarantee every *expression variant* is judged;
  they guarantee nothing about every *assignment form* reaching the
  gate. The recursive forms are where this is lost: a block, and now a
  guard body. A gate that descends into `Block` but not into a guard
  would leave the phase's own new construct unchecked, which would be
  the same defect one layer down.
- **Narrowing without a firing test is not a narrowing.** Each of the
  five measured escapes needs a reject test that is red before the
  change, and `i32 = true` and `string += "x"` are the two most likely
  to be omitted because they are not in the framing's example list.
- **The loader's element-type threading is the one non-mechanical part
  of E-dual.** If it is deferred, the honest outcome is that the loader
  dual-gates every form except `ItemRead` — which should be recorded as
  a named exception with its reason, not quietly shipped as
  "complete".
- **Diagnostic priority is testable and usually untested.** The case
  where both judgements fail needs a test asserting *which* message
  appears, or the ordering is a comment rather than a behaviour.
- **Nothing here is ABI-bearing**, and the loader change is inside the
  existing `validate` walk with no new entry point, value carrier or
  callback.
