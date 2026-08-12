# DD-M4-P3-007 — What a dot means, and the closed set of prefixes

**Status:** Proposed
**Phase:** M4-Phase 3
**AC:** AC9 (upstream of the spellings DD-001 chooses), and phase-end
criterion 4 (spec synchronization)

## Context

This phase puts two new things to the right of a value —
`photos.count()` and `photos[i]`. Before their spelling can be chosen,
the language has to say what a **dot** means. It currently does not say,
and what the implementation does is not what any document describes.

DD-001 cannot settle its collection-interrogation spelling until this is
answered. The member-read candidate (`photos.count`) is not merely
unattractive today — it is **legal and already taken**, because a dotted
path's prefix is discarded rather than validated. Deciding S2 versus S3
against that accident decides it against a defect, not on merit. This
record removes the accident; DD-001 then chooses on merit.

Two prior commitments bound the answer.

- **Tree traversal is not the reason a dot exists here.** The standing
  position is that every reference goes through `state`, with loop
  binders as the single recorded exception
  ([dsl-grammar.md](../../../../docs/notes/dsl-grammar.md) Q1). Grid
  placement was solved without widget ids, and conditional rendering was
  solved with structural IR rather than element references. Nothing in
  this record opens a `root.Children[0]`-shaped path, and the widget-id
  question stays where Q1 left it.
- **`slot.` is already the shape this record generalises.**
  [dsl_spec.md §4.16](../../../../docs/dsl_spec.md) defines `slot` as a
  contextual prefix that is significant only as the head of a dotted
  placement key, with the right-hand side resolved against a closed
  keyword set. It is the one prefix in the language that is already
  well-formed.

### What exists to build on (measured)

Every claim below was produced by running the release `wasamoc` against a
probe `.ui`, or by reading the named source.

- **The prefix is optional.** Replacing every `root.count` in
  `examples/counter/counter.ui` with `count` passes `wasamoc check`.
- **The prefix is arbitrary.** Replacing it with `photos.count` passes;
  with `a.b.c.count` passes; with `root.nope` fails with "undefined
  state `nope`". `check_qualified_name` resolves the **last** segment and
  discards the rest, and the string `"root"` appears nowhere in the
  compiler outside test assertions.
- **The prefix is absent from the IR.** `root.count` lowers to
  `(prop-read count)` ([dsl_spec.md §8.9](../../../../docs/dsl_spec.md)),
  so no representation, loader path or C ABI surface carries it.
- **No document defines it.** `dsl_spec.md` never defines the identifier
  `root`; it appears only inside code examples. In the spec's prose the
  word denotes a different thing entirely — the **content root widget**
  and the body root child (§4.14, §4.16). The only recorded intent is a
  checker comment calling it "the component root alias".
- **The two grammar statements disagree.** §3 gives
  `qualified_name ::= IDENT ("." IDENT)*` — unbounded — while §2.4 says
  an interpolation placeholder takes "one or two `IDENT` segments". The
  implementation follows §3: `"\{a.b.c.count}"` compiles.
- **The surface in the wild is small.** A `root.` prefix appears 26
  times across four `.ui` files (`gallery.ui` 21, `counter.ui` 2,
  `bool-demo.ui` 1, one M4-Phase 1 evidence file 2), plus the
  `dsl_spec.md` examples.

### The two things a dot can spell

Dotted syntax in this language expresses two unrelated ideas.

**(A) Navigation into a value.** The left side is a value and the right
side belongs to it — a field, a query, an operation. **The left side's
type decides what the right side may be**, and the form composes in
principle.

**(B) Qualification of where a name is looked up.** The left side is not
a value; it labels a lookup space. No type is consulted, and the form
does not compose — one prefix, never a chain.

| Form | Kind | State |
|---|---|---|
| `xs.append(v)`, `xs.drop-last()` | (A) | shipped |
| `photos.count()` / `photos.count` | (A) | DD-001's choice |
| `photo.filename` (structured elements) | (A) | not in this milestone |
| `slot.row`, `slot.h-align` | (B) | shipped, closed (§4.16) |
| `root.count` | (B) | shipped, **unbounded** |

Stated in one line, the defect is: **the (B) prefix set is not closed.**
`slot` is closed, so `foo.row` cannot be mistaken for a placement key.
`root` is not, so any identifier at all works as a prefix — which is
precisely why `photos.count` resolves to the state `count` and why the
spelling DD-001 wants is unavailable.

### Terminology this record fixes

`dsl_spec.md` calls `Text` and `VStack` **widgets** (widget registry,
widget declaration); [component-extension-model.md](../../../../docs/notes/component-extension-model.md)
calls the same things **built-in layout components**. Neither term is
defined anywhere. Three layers are enough to compare the options below,
and this record uses them:

- **component** — a reusable *definition*. Today: one per file, top level
  only (§4.1).
- **widget** — an *instance* placed in the tree. Today: registry entries
  only; §4.4 already reserves the position for user-defined components
  by warning rather than rejecting unknown type names.
- **built-in / custom** — a component's provenance, not a separate
  category.

Under these terms, a component definition is not a widget; a component's
**instance** is. The distinction is invisible today only because there is
no instantiation syntax.

## Sub-issues

- What may stand to the left of a dot.
- Whether the `root.` prefix is retained, retired, or redefined.
- Whether `photos.count` is free for DD-001 to use.
- How the §2.4 / §3 segment-count disagreement is repaired.
- What happens to the 26 authored occurrences and the spec examples.
- Who decides later members of the prefix set.

## Options

- **N1 — retire the prefix.** State is reached by bare name only. Every
  dot in an expression is (A); the (B) set is empty on the expression
  side.
- **N2 — a closed, validated prefix set.** `root` becomes a checked
  member alongside `slot`, and anything else to the left of a dot must be
  a value.
- **N3 — make `root` a value.** A component instance becomes a
  first-class value, so `root.count` is (A) like every other dot.

The symbol used to spell a (B) prefix — a bare identifier today, a
sigil such as `^root` / `^slot` / `^host` later — is a spelling of N2's
set rather than a fourth option. It is triaged in
[the pre-1.0 candidate pool](../../../candidate-pool.md) with M4-Phase 7
as its decision point.

## Comparison

### N3 is out

**It collides with the term already in use.** The spec's prose spends
the word "root" on the content root widget. Making `root` denote a
component instance gives one word two meanings inside the same document,
in the middle of an effort to remove exactly that kind of overlap.

**Its promise is larger than anything needs.** A component instance that
is a value invites the questions values attract — may it be passed, held
in a state, compared, put in a collection? Nothing in AC9, in M4, or in
the candidate pool asks for any of them, and answering them here would
take the custom-component surface away from the milestone that owns it.

### N1 versus N2 is the size of the set, not the rule

Both satisfy the rule this record wants, and **both free
`photos.count`** — under N1 because no prefix exists, under N2 because
`photos` is not a member. The difference is narrow:

| | N1 retire | N2 closed set |
|---|---|---|
| Authored `.ui` | 26 occurrences rewritten | unchanged |
| `dsl_spec.md` examples | rewritten | unchanged |
| `photos.count` | free | free |
| Dot in expressions | (A) only | (A), plus a checked (B) |
| A marker for "not ordinary local state" | none | retained |
| A later host prefix | opens (B) at that point | extends a set that exists |

**The tiebreak sits in M4-Phase 7, not here.** That phase designs the
host state boundary. If it spells host state with a prefix, the (B) set
never becomes empty and N1 would have to reopen it; if it marks the
boundary on the declaration instead, the set can be emptied and N1 is the
simpler end state. Neither answer exists yet.

What does **not** depend on Phase 7 is the rule itself, and the rule is
what DD-001 is blocked on. Separating the two lets this phase decide what
it can decide and hand forward only what it cannot.

### The segment count follows from the rule

Once a prefix is a checked member of a closed set, a *chain* of prefixes
is meaningless: `a.b.c.count` has no reading in which `a.b` labels a
lookup space. The production becomes "an optional member prefix, then a
name", which also repairs the §2.4 / §3 disagreement — both sections then
describe the same shape instead of two different ones.

## Recommendation

- **The rule.** The left of a dot is a **value**. The only exceptions are
  members of a **closed, validated set of prefixes**.
- **The set today** is `slot` on the placement-key side (§4.16) and
  `root` on the expression side. Membership is checked; a non-member in
  prefix position is a `wasamoc check` error whose message names the
  members.
- **`photos.count` is not a state read.** With the set closed, the
  spelling is free and DD-001 decides it on its own merits.
- **One prefix at most.** `a.b.c.count` is rejected. §2.4 and §3 state
  the same production, and the interpolation placeholder stops claiming a
  segment count of its own.
- **`root` is retained and validated.** Whether the expression-side set
  should be emptied is handed to **M4-Phase 7**, which decides whether
  the set gains a host member, and is backed by a
  [candidate pool](../../../candidate-pool.md) row so a planning pass has
  to dispose of it rather than remember it.
- **The prefix stays out of the IR.** Lowering discards it, as it does
  today; no representation, loader rule or C ABI surface changes.
- **No authored `.ui` changes**, and no example in `dsl_spec.md` changes.
- **`docs/dsl_spec.md` moves** — §2.4 (the placeholder takes the same
  production as every other name position), §3 (the `qualified_name`
  production becomes an optional member prefix plus a name), §4.16 (a
  cross-reference naming `slot` as a member of the same set), §5 (the AST
  carrier). `docs/abi_spec.md` does not move.

## Forward-compat exposure

- **The set's spelling can change without changing its meaning.** A
  sigil form (`^root`, `^slot`, `^host`) is a rename of a set that is
  already closed and checked, and a rename that the checker can
  diagnose. That is why the pool row can carry it to M4-Phase 7 instead
  of this record settling it. Its value there is concrete: a bare `host`
  prefix has to be claimed out of the identifier namespace, and would
  silently change the meaning of any `.ui` holding a state named `host`,
  while a sigil form cannot collide at all.
- **Structured data makes the (A) rule load-bearing rather than
  decorative.** When element fields arrive, `photo.filename` resolves
  through the receiver's type — which is coherent only if everything to
  the left of a non-prefix dot is a value. This record is what makes that
  true in advance.
- **Custom components will need scope qualification.** Today's
  no-shadowing rule holds because one scope exists; a reusable definition
  with parameters cannot promise its parameter names differ from every
  using component's state names. The closed set is the mechanism that
  case will extend, and N2 leaves it in place rather than requiring its
  reinvention.
- **Tree traversal stays closed.** No form here names an element, and
  the widget-id question remains
  [dsl-grammar.md](../../../../docs/notes/dsl-grammar.md) Q1's. If a
  named-element reference is ever admitted it is a single hop, not a
  path; an index-shaped traversal is what this record's rule makes
  hardest to add, deliberately.
- **The retirement question turns on a negative outcome.** If Phase 7
  marks the host boundary on declarations rather than with a prefix,
  `root` is left with no work to do — and a phase does not document the
  prefix set it declined to use, so nothing in its deliverables surfaces
  the question. That is why it is carried as a pool row with a
  per-planning disposition duty rather than as prose someone has to
  remember.

## Technical risk re-evaluation

- **The reject is the deliverable.** Every claim above is a narrowing:
  one prefix at most, membership checked, non-members are values. A
  narrowing with no firing reject test does not exist. The close artifact
  is a table with a firing case per rejected shape — a non-member prefix,
  a chained prefix, a member prefix in a position that takes none.
- **This is the phase's only change to what a legal `.ui` means.**
  `photos.count` compiles today and becomes a diagnostic. No shipped
  example is affected, but a file written against the public draft can
  be, so the message has to name the members rather than say "unknown
  state" — the failure mode to avoid is a diagnostic that sends the
  author looking for a missing declaration.
- **A prefix that is also a state name is the collision to test.** With
  `root` a member, a component declaring `state root` puts the same
  identifier in both roles. Rejecting the declaration is the smaller
  surface, and the reject has to fire on the declaration rather than at
  each use.
- **DD-001 consumes this record's outcome, so the two must not drift.**
  The interrogation spelling is DD-001's; the availability of
  `photos.count` is this record's. Neither should restate the other's
  conclusion.
