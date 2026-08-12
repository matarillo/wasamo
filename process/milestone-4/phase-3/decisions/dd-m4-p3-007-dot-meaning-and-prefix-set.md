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
  well-formed. What generalises is the **shape** — a dotted head that is
  not a value; where the two prefixes differ, and how much of the set's
  value rests on `slot` alone, is recorded in §Comparison.

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
type decides what the right side may be.**

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

### Terms used in this record

`dsl_spec.md` calls `Text` and `VStack` **widgets** (widget registry,
widget declaration); [component-extension-model.md](../../../../docs/notes/component-extension-model.md)
calls the same things **built-in layout components**. Neither term is
defined anywhere. Reconciling the two across the spec is **not
attempted here** — that vocabulary belongs with the surface that owns
custom components, and this record does not move it. Three layers are
enough to compare the options below, and this record uses them locally:

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
- If it is retained, whether writing it is optional or required, and
  whether the language declares one of the two spellings canonical.
- Whether `photos.count` is free for DD-001 to use.
- How the §2.4 / §3 segment-count disagreement is repaired.
- Whether the rule reaches an assignment's left-hand side the same way
  it reaches a read.
- What happens to the 26 authored occurrences and the spec examples.
- Who decides later members of the prefix set.

## Options

- **N1 — retire the prefix.** State is reached by bare name only. Every
  dot in an expression is (A); the (B) set is empty on the expression
  side.
- **N2 — a closed, validated prefix set.** `root` becomes a checked
  member alongside `slot`, and anything else to the left of a dot must be
  a value. Two dispositions of the retained member are weighed
  separately below:
  - **N2a — a normative member**, documented in the spec the way `slot`
    is.
  - **N2b — validated but recorded as provisional**, with the bare name
    named canonical and the prefix carried pending M4-Phase 7.
- **N3 — make `root` a value.** A component instance becomes a
  first-class value, so `root.count` is (A) like every other dot.

**N1 is not this phase's to take.** The accepted
[framing](../requirements/framing.md) §含まないもの assigns to
**M4-Phase 7** whether the expression-side set is emptied, so N1 is
listed to show what is being deferred and on what grounds — not as an
option this record picks between. That exclusion and this record were
proposed in the same pass
([plan Revision 7](../requirements/plan-revision-7-proposal.md)), so it
does not constrain this record independently: if the comparison below
reads as favouring N1, taking it is a framing revision, not a milestone
change.

The symbol used to spell a (B) prefix — a bare identifier today, a
sigil such as `^root` / `^slot` / `^host` later — is a spelling of N2's
set rather than a fourth option. It is triaged in
[the pre-1.0 candidate pool](../../../candidate-pool.md) with M4-Phase 7
as its decision point. What that deferral costs is priced below rather
than assumed to be nothing.

## Comparison

### N3 is out

**This phase does not take on component-instance-as-value.** A
component instance that is a value invites the questions values attract
— may it be passed, held in a state, compared, put in a collection?
Nothing in AC9, in M4, or in the candidate pool asks for any of them,
and answering them here would take the custom-component surface away
from the milestone that owns it. A narrower form is imaginable — a
receiver-only value, usable in no other position, which is what `slot`
is on the placement side — but this record has not explored it and does
not claim it is unworkable. It claims only that designing it is not
this phase's work, and that N2's rule frees `photos.count` without it.

**Secondarily, the word is taken.** The spec's prose spends "root" on
the content root widget, so making `root` denote a component instance
gives one word two meanings inside the same document. This is a naming
problem, answerable by choosing another word, and it is recorded as a
cost rather than as the reason N3 is out.

### N1 versus N2 is the size of the set, not the rule

Both satisfy the rule this record wants, and **both free
`photos.count`** — under N1 because no prefix exists, under N2 because
`photos` is not a member. What the choice turns on is below:

| | N1 retire | N2 closed set |
|---|---|---|
| Authored `.ui` | 26 occurrences rewritten | unchanged |
| `dsl_spec.md` examples | rewritten | unchanged |
| `photos.count` | free | free |
| Dot in expressions | (A) only | (A), plus a checked (B) |
| The identifier `root` | stays an ordinary name | claimed — `state root` becomes a `wasamoc check` error |
| Spellings for one state read | one | two, synonymous (`count` and `root.count`) |
| A marker for "not ordinary local state" | none | retained |
| A later host prefix | opens (B) at that point | extends the set — but not its lowering rule (below) |

The first two rows are the migration cost and are the ones a reader
reaches first; on their own they read as "N1 costs 26 rewrites and N2
costs nothing". The rows after them are why that reading is wrong, and
three of them need saying out loud.

#### The two members of the set are not alike

Under this record's own definition, a (B) prefix **labels a lookup
space**. Applied to the two members:

| | `slot.` | `root.` |
|---|---|---|
| Space it labels | placement keys — a space distinct from the widget's own properties | component state — **the same space a bare name reaches** |
| Writing it | **required**; bare `h-align:` is a §4.16 reject | optional; `count` and `root.count` both compile |
| Effect on the right side | decides that it resolves against a closed placement-keyword set, not against state | none |
| Lowering | normalised to a child-slot placement record (§4.12, §4.16) | discarded |

So `slot` qualifies something and `root` does not: it names the space
that is already the default when nothing is written. What the two share
is the syntactic shape — a dotted head that is not a value — and the
rule this record wants is a rule about that shape. That is a real rule
and it is what DD-001 is blocked on. But it is worth being exact about
what N2 buys: **one member that does work, and one that marks a read as
component state in a language where every read is component state.**

The "two spellings" row follows from the same fact. Retaining an
optional prefix that changes nothing leaves the language with two ways
to write one read; whether it also leaves them **unranked** is what
N2a and N2b differ on below. The placement surface did not accept that
state for itself — it made its prefix required.

#### The identifier is claimed, and this record is what claims it

Validating membership means a component cannot declare `state root`
without putting one identifier in two roles, so the declaration becomes
a reject (see §Technical risk). That is a claim on the author's
identifier namespace which N1 does not make, and it is not reversible
after the M6 freeze.

The [candidate pool](../../../candidate-pool.md) row already carries
this argument — "a bare new prefix has to be claimed out of the
identifier namespace", so a later `host` prefix would break any `.ui`
holding a state named `host`, "while a symbol form cannot collide". The
argument is sound and it applies **now**, to `root`, in this record,
rather than only to a future member: this is the record that turns an
undefined token into a claimed one.

Two things follow. First, the migration comparison is not
cost-free-versus-26-rewrites; it is a mechanical, diagnosable one-time
edit against a permanent, freeze-bearing namespace claim. Second, the
symbol question deferred to M4-Phase 7 is deferred **across** the point
where the collision is created. An alternative that avoids the claim
exists and is not invented here: §4.16 already treats `slot` as "a
contextual prefix, not a reserved keyword — significant only as the
head of a dotted placement key and a valid ordinary identifier
everywhere else". Reading `root` the same way would leave `state root`
legal. This record does not take that reading — the reject is the
smaller and more legible surface, and a contextual reading would have
to answer `root.root` — but the choice is a choice, not an absence of
one.

#### Deferring the retirement is not cost-neutral

Today `root` is undefined, undocumented and unvalidated. Retiring it
costs 26 mechanical edits and no reader expectation, because nothing
has ever promised it. After this record's spec sync it is published
surface in §2.4, §3, §4.16 and §5, shipping through Phases 4–6 and the
public draft. **Phase 7's option to empty the set is therefore strictly
more expensive after this record than before it**, and the phase most
likely to leave it unexercised — the negative outcome, where Phase 7
marks the host boundary on declarations and simply never mentions
prefixes — is the one where nobody is holding the question.

The split this record proposes ("decide the rule, hand the membership
forward") is still right: the rule does not depend on Phase 7 and
DD-001 is blocked on it. But the deferral is not neutral, and the
record should not present it as though Phase 7 inherits the same choice
this phase has.

#### N2a versus N2b

**The tiebreak between N1 and N2 sits in M4-Phase 7, not here.** That
phase designs the host state boundary. If it spells host state with a
prefix, the (B) set never becomes empty and N1 would have to reopen it;
if it marks the boundary on the declaration instead, the set can be
emptied and N1 is the simpler end state. Neither answer exists yet.

Given that, the disposition of the retained member is the part this
phase can choose deliberately. **N2a** documents `root` as a normative
member alongside `slot`; it reads cleanly, and it is the shape that
makes the retirement cost above largest. **N2b** validates membership
identically — the checker behaviour, the diagnostics and the reject
tests are the same — but records the member as **provisional**, names
the bare form canonical, and says in the spec that the expression-side
set's membership is settled at M4-Phase 7.

N2b is not an invention of this record's. §4.16 already ships the
pattern for the Grid `Cell` / direct `slot.*` duplication: "That
convention is **provisional** — a future pre-1.0 decision fixes whether
a wrapper form is retained — and is not an acceptance criterion." The
same sentence shape, applied to `root`, keeps Phase 7's choice at close
to today's price and ranks the two spellings without deciding
retirement.

N2b's cost is that the public draft carries a surface it labels
unsettled. That is a real cost for a language approaching a freeze, and
it is the honest one: the surface **is** unsettled, and the alternative
is not settling it but concealing that it is not settled.

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
- **One prefix at most.** `a.b.c.count` is rejected. §2.4 and §3 stop
  disagreeing about how many segments a name position takes, and the
  interpolation placeholder stops claiming a segment count of its own.
- **The rule reaches the assignment target.** Today an assignment's
  left-hand side is a `qualified_name` (§3), so the same two
  constraints — membership checked, at most one prefix — apply there as
  they do to a read. Whether any non-prefix dotted target is ever
  admitted is not this record's question.
- **`root` is retained and validated (N2b).** After validation it does
  nothing a bare name does not: `count` and `root.count` are the same
  read, and the **bare form is canonical**. The prefix is therefore
  recorded in the spec as **provisional**, on the §4.16 model, with
  membership of the expression-side set settled at **M4-Phase 7** —
  which decides whether the set gains a host member — and backed by a
  [candidate pool](../../../candidate-pool.md) row so a planning pass
  has to dispose of it rather than remember it.
- **The identifier is claimed.** A component declaring `state root`
  puts one identifier in two roles; the declaration is a reject. This is
  the second of this phase's two changes to what a currently legal `.ui`
  means, and it does not expire with the provisional label.
- **The expression-side prefix stays out of the IR.** `root.count`
  lowers to `(prop-read count)` as it does today; no representation,
  loader rule or C ABI surface changes. This is a property of *this*
  member, not of the set — `slot` already lowers to a placement record,
  and a member that means something would have to survive lowering.
- **No authored `.ui` changes**, and no example in `dsl_spec.md`
  changes. The 26 existing occurrences keep compiling; naming the bare
  form canonical is a statement about what the spec recommends, not a
  migration. Rewriting them is what retirement would cost, and pricing
  that is Phase 7's.
- **`docs/dsl_spec.md` moves** — §2.4 (the placeholder stops stating a
  segment count of its own; **which production it takes is DD-001's**,
  and this record constrains only its prefix part), §3 (a name position
  is an optional member prefix plus a name), §4.16 (a cross-reference
  naming `slot` as a member of the same set, and the provisional note
  for `root`), §5 (the AST carrier). `docs/abi_spec.md` does not move.

## Forward-compat exposure

- **The set's spelling can change without changing its meaning.** A
  sigil form (`^root`, `^slot`, `^host`) is a rename of a set that is
  already closed and checked, and a rename that the checker can
  diagnose. That is why the pool row can carry it to M4-Phase 7 instead
  of this record settling it. Its value there is concrete: a bare `host`
  prefix has to be claimed out of the identifier namespace, and would
  silently change the meaning of any `.ui` holding a state named `host`,
  while a sigil form cannot collide at all. What the deferral does not
  avoid is that the **first** such claim is made here, on `root` (see
  §Comparison) — Phase 7 inherits a namespace already claimed once, not
  an untouched one.
- **A later member need not be erasable at lowering.** The one
  expression-side member today is semantically inert, so discarding it
  costs nothing. `slot` already contradicts the generalisation on the
  placement side, and a host prefix could not be erased at all: where a
  name is read from is exactly what the runtime would need to keep. The
  closed set as specified here is therefore a rule about **admission**,
  and whether it is also a rule about **resolution** is open the moment
  it gains a member that means something. Phase 7 is where that is
  discovered, and it should not read the "prefixes are discarded"
  sentence as a constraint it inherits.
- **Structured data makes the (A) rule load-bearing rather than
  decorative.** When element fields arrive, `photo.filename` resolves
  through the receiver's type — which is coherent only if everything to
  the left of a non-prefix dot is a value. This record is what makes that
  true in advance.
- **Custom components may put pressure on scope qualification.** Today's
  no-shadowing rule holds because one scope exists; a reusable definition
  with parameters could not promise that its parameter names differ from
  every using component's state names. Whether the answer is a prefix, a
  declaration-site marking, or a lexical scoping rule is not something
  this record has compared, and it does not claim the closed set is the
  mechanism that case will use. What it claims is narrower: N2 leaves a
  closed, checked prefix set in place, so that surface is one of the
  available answers rather than one that has to be built from nothing.
- **Tree traversal stays closed.** No form here names an element, and
  the widget-id question remains
  [dsl-grammar.md](../../../../docs/notes/dsl-grammar.md) Q1's. If a
  named-element reference is ever admitted it is a single hop, not a
  path; an index-shaped walk **into the widget tree** is what this
  record's rule makes hardest to add, deliberately. Indexing a
  collection is a separate question and is DD-001's.
- **The retirement question turns on a negative outcome.** If Phase 7
  marks the host boundary on declarations rather than with a prefix,
  `root` is left with no work to do — and a phase does not document the
  prefix set it declined to use, so nothing in its deliverables surfaces
  the question. That is why it is carried as a pool row with a
  per-planning disposition duty rather than as prose someone has to
  remember. The provisional label recommended above is the second
  holder of the same question: it sits in the surface Phase 7 has to
  read anyway, and it is what keeps the retirement priced near today's
  cost rather than at the cost of withdrawing settled public text.

## Technical risk re-evaluation

- **The reject is the deliverable.** Every claim above is a narrowing:
  one prefix at most, membership checked, non-members are values. A
  narrowing with no firing reject test does not exist. The close artifact
  is a table with a firing case per rejected shape — a non-member prefix,
  a chained prefix, a member prefix in a position that takes none, and a
  `state` declared with a member's name.
- **This record holds the phase's changes to what a legal `.ui` means,
  and there are two.** `photos.count` compiles today and becomes a
  diagnostic; `state root` compiles today and becomes a reject. No
  shipped example is affected by either, but a file written against the
  public draft can be, so the first message has to name the members
  rather than say "unknown state" — the failure mode to avoid is a
  diagnostic that sends the author looking for a missing declaration.
- **A prefix that is also a state name is the collision to test.** With
  `root` a member, a component declaring `state root` puts the same
  identifier in both roles. Rejecting the declaration is the smaller
  surface, and the reject has to fire on the declaration rather than at
  each use. The cost of that reject — a claim on the identifier
  namespace that outlives the provisional label — is weighed in
  §Comparison, not only tested here.
- **DD-001 consumes this record's outcome, so the two must not drift.**
  The interrogation spelling is DD-001's; the availability of
  `photos.count` is this record's. Neither should restate the other's
  conclusion. The two records also move the same spec section: §2.4's
  target production is DD-001's to choose, and this record constrains
  only the prefix part of whatever it chooses. That coordination is
  settled at the Accepted flip and the design sync, not by either record
  predicting the other.
