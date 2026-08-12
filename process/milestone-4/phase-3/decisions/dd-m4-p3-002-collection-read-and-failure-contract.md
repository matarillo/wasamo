# DD-M4-P3-002 — Reading a collection from outside the repetition, and what an out-of-range index means

**Status:** Proposed
**Phase:** M4-Phase 3
**AC:** AC9 (collection count, emptiness, index access), and
phase-end criterion 4 (spec synchronization)

## Context

Until now a collection could only be read by iterating it. Every other
read — a bare name, a navigated member, a bracket index — is rejected
with the same deferral message, "collection reads outside iteration not
yet supported". AC9 lifts that for three questions: how many, whether
any, and what is at position *i*.

The first two are **total**: they have an answer for every collection
state at every moment. The third is **partial**, and this is the record
where that asymmetry has to be resolved into a contract an author can
predict. The gallery makes it concrete rather than hypothetical:
`selected_index` is written by four handlers that today increment and
decrement it without any bound, so `-1` and `count` are both reachable
values, and a caption bound to `photos[selected_index]` will meet them.

The framing (agreement ⑤) deliberately returned this to the ADR rather
than settling it: the Phase 2 handoff's expectation of a runtime
diagnostic is a strong input but not a conclusion, and DD-005's write
guard is a **different** responsibility — preventing the bad state —
that does not substitute for deciding what a read of an already-bad
state does.

### What exists to build on (measured)

- **A length computation over a collection signal already exists.**
  `reactive::collection_len_tracked` reads the element-typed list signal
  through `Signal::get()` — a **tracked** read — and is what the `for`
  member's cardinality effect uses.
- **Dependency tracking is per-signal, per-effect, and accumulates
  within one effect run.** `Signal::get()` registers an edge from the
  signal to the currently running effect; an effect reading two signals
  gets two edges. Nothing about reading a collection and a scalar in the
  same effect is new.
- **Collections are whole-value signals with change comparison.**
  `set_if_changed` propagates only when the new value differs, so a
  replacement with different contents at the same length **does** mark
  the signal dirty and re-run its readers. Positional re-reading is a
  property of the existing signal, not something this record has to
  build.
- **An out-of-range read already has a classification and a
  behaviour.** `EvalError::ItemOutOfRange` exists for the loop-internal
  case (a handler shortened its own collection mid-statement). In
  binding position, `evaluate_bool_binding_optional` returns `Ok(None)`
  for that case and the writer is **skipped**; on the integer and string
  paths the error propagates and the effect's closure prints it to
  stderr and writes nothing.
- **The failure boundary is the effect.** Each registered binding's
  closure matches on its own `Result` and logs; there is no
  drain-level abort and no transactional grouping.
- **There is no author-visible diagnostic channel at runtime.** Errors
  reach stderr. Any structured channel is ABI-adjacent and outside this
  phase.

## Sub-issues

- **Which reads exist** and what each returns.
- **Result typing** for element access across the three element types.
- **Reactive dependency**: what a count read, an emptiness read and an
  index read each depend on, and whether a same-length replacement
  re-reads.
- **The out-of-range contract**: what the author is promised when the
  index is negative, equal to the count, or any position in an empty
  collection.
- **Failure containment**: what happens to the target property, to the
  failing effect, and to other effects in the same drain.
- **Observability**: where the author sees that it happened.
- **The relationship to DD-005**, so the two are not solving the same
  problem twice.

## Options

### Out-of-range result

- **R1 — no write, previous value retained.** The binding evaluates,
  finds nothing at the position, and does not write. The target keeps
  whatever it last held. No error is raised.
- **R2 — error, no write, previous value retained, diagnostic
  emitted.** Same pixels as R1, plus the read is classified as a failure
  and reported through the runtime's existing channel.
- **R3 — typed fallback value.** The read yields the element type's
  zero value — `""`, `0`, `false` — and that value is written.
- **R4 — clamp.** The index is clamped into `[0, count-1]` and the
  clamped element is read. Requires a separate rule for the empty
  collection, where there is no position to clamp to.
- **R5 — reject at check time.** Listed for completeness: the index is a
  runtime value, so this is only available for a literal index and
  cannot be the general contract.

### Failure containment

- **C-a — per-effect.** The failing binding writes nothing; every other
  effect in the same drain runs normally.
- **C-b — per-drain.** An out-of-range read aborts the drain, so the
  frame shows a consistent pre-write state.

### Observability

- **O-1 — the existing stderr channel**, as every other evaluation
  error uses today.
- **O-2 — a structured runtime diagnostic surface** the host can read.
- **O-3 — silent.**

### Count and emptiness dependency

- **D-1 — the whole-value collection signal.** Any change to the
  collection re-runs any effect that read its count.
- **D-2 — a derived length signal**, so a replacement that preserves
  length does not re-run count readers.

## Comparison

### The out-of-range result: R3 and R4 lose on the same ground

R3 and R4 share a defect that outweighs everything else about them:
**they render the author's mistake as a plausible screen**. A caption
showing `""` because the index is `-1` is indistinguishable from a
caption showing `""` because the photo has no name; a caption showing
photo 17 because the index was clamped from 18 is indistinguishable from
a correct selection of the last photo. Both convert a fault into a value
and hand it to the layout, and the author's first evidence is a bug
report about the wrong photo, not a message.

R4 has a second, independent problem. Clamping has no defensible answer
for the empty collection, so it needs a sub-rule — which means the
"simple" option is actually two rules, one of which is R1 or R3 wearing
a different name. And it silently disagrees with the rest of the
runtime: [dsl_spec.md §8.11](../../../../docs/dsl_spec.md) already
records that Grid placement is "reject-at-validate, not
clamp-at-arrange" because "a silently-clamped placement would displace
legitimately-placed siblings and produce order-dependent layout". A
clamped index is the same shape of decision on a different surface, and
this set should not make it the other way without a reason specific to
indices. There is none.

R3's remaining argument is that it is total — every read yields a value,
so no downstream code has to handle absence. That argument is real for a
language where a `Text` must always show something. It is answered by
the fact that under R1/R2 the `Text` **does** always show something: the
value it last held, or its authored static content if it never held one.
Totality of the property is preserved without totality of the read.

### R1 versus R2: same pixels, so the difference is entirely observability

R1 and R2 produce identical rendered output. R1 is R2 with the
diagnostic removed. Framed that way there is no case for R1: the
question is only whether the failure is *reported*, and reporting a
failure the runtime has already detected costs one existing code path.

R2 also keeps **one rule for one situation**. The loop-internal
out-of-range case already exists and is already classified
(`ItemOutOfRange`); if the loop-external index read used a different
contract, the language would have two answers to "the position isn't
there" depending on whether the position came from a `for` header or an
author's index expression. The author never asked for two.

The honest cost of R2 is that the diagnostic lands on stderr, which a
GUI author may never see. That is a real limitation of the current
runtime and it is not this phase's to fix — O-2 is a host-visible
surface and therefore ABI-adjacent. What R2 buys even without a good
channel is a *classification*: the failure exists in the type system of
the runtime, so a later phase that adds a channel does not also have to
re-decide what an out-of-range read is.

### The boundary R2 has to state explicitly

"The target keeps the value it last held" has an edge that must be
written down rather than discovered: **if the index is out of range at
the first evaluation, the target never held a bound value**, so it keeps
its authored static content — or, for a property with no static content,
its constructed default. A gallery whose `selected_index` starts at `0`
over an empty collection shows the caption's authored placeholder, not
an empty string chosen by the runtime. This is the same outcome R1 would
give and it is a consequence of "no write", not a separate rule; but a
reader who is told only "the previous value is retained" will ask what
the previous value is on the first frame, and the spec has to answer.

### Containment: C-a

C-b's appeal is frame consistency: if one binding in a drain fails, the
frame shows a mixture of updated and stale properties. That is true
under C-a and it is worth stating rather than hiding.

C-b is rejected on three grounds. It is a new mechanism — the engine has
no transaction, no rollback and no drain-level result — so adopting it
means designing one in a phase whose scope is expressions. It has no
defined rollback for the writes that already landed earlier in the same
drain, which is the same problem clamping had: the "consistent" option
needs a second rule to be consistent. And it converts a **local**
authoring mistake into a **global** frame failure, which is a worse
failure mode for a GUI than a stale caption.

DD-005's guard is what keeps the gallery out of this state altogether,
so C-a's mixed-frame exposure is not the shipped app's normal path — it
is the fixture's path.

### Count and emptiness dependency: D-1

D-2 would avoid re-running a count binding when a same-length
replacement happens. That is a real optimisation and it is the wrong
trade here: it means a second signal per collection whose invalidation
has to stay in lockstep with the collection's own, which is precisely
the parallel-data hazard the project's implementation gates name. The
collection is one whole-value signal by
[dsl_spec.md §4.7](../../../../docs/dsl_spec.md) — "there is no
per-element signal" — and a derived length signal is a per-collection
shadow of exactly that kind. D-1 re-runs a count binding on a
same-length replacement and writes the same value; `set_if_changed` on
the *target* absorbs the redundancy at the property level.

D-1 also gives the index read what it needs for free: an index binding
reads **both** the collection signal and the index state, so a
same-length replacement re-reads position *i* — which is exactly the
positional semantics
[../requirements/constraints.md](../requirements/constraints.md) §4
requires this record to preserve.

### What this record does not decide

The gallery's four writers are DD-005's subject. This record decides
what happens when a read meets a bad index; it does not decide how a bad
index is prevented, and it explicitly does not define a rollback or
transaction contract that DD-005 could consume — DD-005 consumes this
one's per-effect containment as given.

## Recommendation

- **Three total reads and one partial one.** `xs.count()` → `i32`;
  `xs.is-empty()` → `bool`; `xs.last-index()` → `i32`, which is
  `count - 1` and **`-1` for an empty collection**; `xs[i]` → the
  collection's element type, partial.
- **`last-index()`'s `-1` is normative**, not incidental: DD-005's
  empty-collection row depends on it, and it is specified here so it is
  specified once.
- **R2 — an out-of-range index is an error, not a value.** The binding
  writes nothing; the target keeps the value it last held, and on a
  first evaluation that is its authored static content; the failure is
  classified as the existing out-of-range condition rather than as an
  unknown property; and it is reported through the runtime's existing
  channel.
- **Out of range means** a negative index, an index `>= count`, and any
  index into an empty collection. There is no separate empty-collection
  rule.
- **C-a — containment is the effect.** Other effects in the same
  synchronous drain are unaffected, and the frame may show a mixture of
  updated and stale properties. No drain-level abort, no rollback, no
  transaction.
- **O-1 — the existing runtime channel.** A host-readable diagnostic
  surface is not created here; it is named as ABI-adjacent and left to
  the phase that owns the host boundary.
- **D-1 — dependencies are the whole-value collection signal plus any
  state the index expression reads.** A same-length replacement with
  different contents re-runs the read, because the collection signal is
  dirty; a replacement with equal contents does not, because
  `set_if_changed` did not propagate.
- **No fallback value and no clamping**, anywhere, for any element
  type.
- **`docs/dsl_spec.md` moves**: §4.15 (the loop-external read family and
  the out-of-range contract), §4.6 (the new forms in the expression
  table), §8.9 (the IR rows).

## Forward-compat exposure

- **A structured diagnostic channel is additive and is not foreclosed.**
  What this record fixes is the *classification* of the failure; where
  it is delivered is a separate axis. A later phase that gives the host
  a diagnostic surface inherits the classification rather than
  re-deciding it.
- **A total element read is additive and would be a different form.**
  If a later phase wants "the element or a default", the honest shape is
  a second, explicitly total form — not a change to `xs[i]`'s contract.
  Changing `xs[i]` later would silently alter every existing `.ui`'s
  behaviour, which is why the partial form is specified now rather than
  left implicit.
- **Element identity remains positional.** Nothing here introduces a key
  or a stable element identity, so `xs[i]` after a replacement means
  "whatever is at position *i* now". A later keyed-identity opt-in
  ([dsl_spec.md §4.15](../../../../docs/dsl_spec.md) records it as
  possible) would have to say what an index read means under it; this
  record does not claim that question is easy.
- **Slicing, searching and aggregation are unaddressed and unreserved.**
  `xs.count()` is a length, not the start of an aggregate family. No
  spelling is reserved for one.
- **`TypedValue` is untouched.** Element access carries an element-type
  tag exactly as `ListPropRead` does, so the structured-item decision
  stays with M4-Phase 7.

## Technical risk re-evaluation

- **The out-of-range branch is a pure-logic boundary condition, so
  [DD-V-029](../../../cross-milestone/decisions/dd-v-029-pure-logic-red-test-obligation.md)
  applies directly.** The close artifact must name the test and record
  the wrong implementation that actually turned it red — and the wrong
  implementations to try are the rejected options themselves: a
  fallback-value read and a clamping read both produce a green screen, so
  a test that only asserts "the app does not crash" or "the caption is
  non-empty" would pass all three contracts. The discriminating
  assertion is that the target holds its **prior** value, which requires
  the test to establish a prior value first.
- **`-1` from `last-index()` is a sentinel with a downstream consumer.**
  DD-005's empty and one-item rows depend on it. A test that pins
  `last-index()` on a non-empty collection and not on an empty one
  leaves the load-bearing case unpinned.
- **The first-evaluation edge is the easiest thing to get wrong and the
  hardest to see.** "Retains its previous value" is untestable on the
  first frame unless the fixture deliberately starts out of range, and a
  gallery guarded by DD-005 will never start out of range. This is the
  case that has to go to a named mechanism fixture rather than to the
  shipped app.
- **The mixed-frame consequence of C-a is a documented outcome, not a
  defect**, and the phase should not later "fix" it by adding a drain
  abort without superseding this record.
- **Count and index reading the same collection is one effect with two
  edges**, and the risk is a re-run that reads a stale pair — for
  example a collection shrinking and the index updating in the same
  drain, evaluated between the two writes. The synchronous drain makes
  this reachable, and the resulting read is out of range, which is
  exactly the case R2 defines. This is a reason the contract must be
  defined rather than a defect to remove.
