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
path's prefix is discarded rather than validated. Weighing the
interrogation spellings against that accident decides them against a
defect, not on merit. This record removes the accident; DD-001 then
chooses on merit.

Two prior commitments bound the answer.

- **Tree traversal is not the reason a dot exists here.** The standing
  position is that every reference goes through `state`, with loop
  binders as the single recorded exception
  ([dsl-grammar.md](../../../../docs/notes/dsl-grammar.md) Q1). Grid
  placement was solved without widget ids, and conditional rendering was
  solved with structural IR rather than element references. Nothing in
  this record opens a `root.Children[0]`-shaped path, and the widget-id
  question stays where Q1 left it.
- **`slot.` is the one prefix already well-formed.**
  [dsl_spec.md §4.16](../../../../docs/dsl_spec.md) defines `slot` as a
  contextual prefix that is significant only as the head of a dotted
  placement key, with the right-hand side resolved against a closed
  keyword set. Whether it and `root` belong under one rule is
  M4-P3-007-001 rather than an assumption.

### What exists to build on (measured)

Every claim below was produced by running the release `wasamoc` against a
probe `.ui`, or by reading the named source.

- **The prefix is optional.** Replacing every `root.count` in
  `examples/counter/counter.ui` with `count` passes `wasamoc check`. This
  record calls that prefix-less spelling the **unprefixed form**.
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
- **Both spellings ship, and two positions already reject the prefix.**
  Beside those 26, five property bindings read state unprefixed —
  `checked: tab_all_selected` (three times) and `offset-y: scroll_y` in
  `gallery.ui`, `enabled: ready` in `bool-demo.ui` — so `gallery.ui`
  writes both spellings. Two shipped diagnostics go further and
  *require* the unprefixed form: §4.15 rejects `for x in root.xs` with "the
  loop collection must be a local state name", and
  `root.xs = root.xs.append(1)` with "collection mutation requires a
  local state name". The language therefore already has an expressed
  preference in the positions where it has any.

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
no instantiation syntax — and it is what makes M4-P3-007-002 askable at
all: without it, "root" reads as the component and as the top of the
component's tree at the same time.

## Sub-issues

Each sub-issue carries a stable identifier. Records outside this one cite
a sub-issue by that identifier and state its answer in words; the option
labels used below are local to this record and are not cited from outside
it. Every sub-issue has its own subsection under §Options and
§Recommendation, and every sub-issue that is a choice also has one under
§Comparison; the two marked as outcomes below have nothing to compare,
and their subsections say what determines them instead.

| ID | Question | Nature | Live only when |
|---|---|---|---|
| **M4-P3-007-001** | What may stand to the left of a dot | choice | — |
| **M4-P3-007-002** | What a (B) prefix denotes | choice | — |
| **M4-P3-007-003** | Whether `root` is a member of the set | choice | — |
| **M4-P3-007-004** | How a retained member is written and documented | choice | 003 retains |
| **M4-P3-007-005** | Whether `photos.count` is free for DD-001 | outcome of 001 | — |
| **M4-P3-007-006** | How many segments a name position takes | choice | — |
| **M4-P3-007-007** | Whether the rule reaches an assignment target | choice | — |
| **M4-P3-007-008** | What happens to the authored occurrences | outcome of 003 | — |
| **M4-P3-007-009** | Who decides later members of the set | choice | — |
| **M4-P3-007-010** | How a `state` named after a member is handled | choice | 003 retains |

**001 is the spine.** It decides whether a closed set exists at all;
005, 006 and 007 are its reach. **003 hangs off it** — there is nothing
to be a member of until 001 answers — and 003 in turn gates 004 and 010,
which have no content if the member is retired. 009 survives retirement,
because a later member could still be admitted to an empty set.
**002 feeds 003**, because one of 003's options redefines the prefix as a
value, and that is a different design under each denotation. 005 and 008
are stated as outcomes rather than choices: nothing can be decided about
them independently of 001 and 003.

## Options

### M4-P3-007-001 — What may stand to the left of a dot

- **U1 — a value, with one closed set of exceptions spanning both
  sides.** One rule covers the placement-key prefix and the expression
  prefix: the left of a dot is a value unless it is a checked member of a
  closed set.
- **U2 — a value on the expression side, with `slot` left as its own
  rule.** The expression side gets a closed set; §4.16 keeps its
  placement-key prefix as a separate, unrelated construct.
- **U3 — the status quo.** A dotted path's prefix is discarded and the
  last segment resolves against state, so any identifier is admissible
  in prefix position.

### M4-P3-007-002 — What a (B) prefix denotes

- **D1 — the enclosing component instance.** A `self`-shaped reading:
  `root` names the component whose body the expression sits in, and
  `count` is that component's member.
- **D2 — the root of the widget tree.** `root` names the content root
  widget — the top-level `node Window` of §8.3 — so a dotted name reads
  as reaching the top of the tree.
- **D3 — a lookup space, and no object.** `root` labels *where a name is
  looked up* — the enclosing component's state — and denotes no value at
  all, which is this record's (B) definition applied to this member.

### M4-P3-007-003 — Whether `root` is a member of the set

- **K1 — retire it.** State is reached by unprefixed name only; the
  expression-side set is empty.
- **K2 — retain it as a checked member.** `root` is validated in prefix
  position, and anything else to the left of a dot must be a value.
- **K3 — redefine it as a value.** Whatever `root` denotes becomes
  first-class, so `root.count` is (A) like every other dot. Which design
  that is depends on 002, which is why the two are settled in that order.

**K1 is not this phase's to take.** The accepted
[framing](../requirements/framing.md) §含まないもの assigns to
**M4-Phase 7** whether the expression-side set is emptied, so K1 is
listed to show what is being deferred and on what grounds. That exclusion
and this record were proposed in the same pass
([plan Revision 7](../requirements/plan-revision-7-proposal.md)), so it
does not constrain this record independently: if the comparison below
reads as favouring K1, taking it is a framing revision, not a milestone
change.

### M4-P3-007-004 — How a retained member is written and documented

- **W1 — required, the way `slot` is.** Writing the prefix becomes
  obligatory; the unprefixed `count` is a reject. One spelling per read.
- **W2 — optional and normative.** The prefix stays writable or omittable
  and is documented as a settled member alongside `slot`, with neither
  spelling named canonical.
- **W3 — optional, unprefixed form canonical, member provisional.** Validation
  is identical to W2 — same checker behaviour, same diagnostics, same
  reject tests — but the spec names the unprefixed form canonical and records
  the member as provisional pending M4-Phase 7.

### M4-P3-007-005 — Whether `photos.count` is free for DD-001

No options. Under U1 or U2 the spelling is free, because `photos` is not
a member; under U3 it stays a state read. Nothing here is separable from
001.

### M4-P3-007-006 — How many segments a name position takes

- **G1 — at most one prefix.** A name position is an optional member
  prefix followed by a name; `a.b.c.count` is a reject.
- **G2 — an unbounded chain,** as §3's production is written today, with
  §2.4 corrected to match it.

### M4-P3-007-007 — Whether the rule reaches an assignment target

- **L1 — the same rule on both sides.** Membership checking and the
  segment cap apply to an assignment's left-hand side exactly as they
  apply to a read.
- **L2 — reads only.** An assignment target keeps today's
  last-segment-wins resolution.

### M4-P3-007-008 — What happens to the authored occurrences

No options. Under K2 nothing changes; under K1 all 26 occurrences and the
`dsl_spec.md` examples are rewritten. Naming a canonical form under W3 is
a statement about what the spec recommends, not a migration.

### M4-P3-007-009 — Who decides later members of the set

- **Y1 — this record fixes the final set,** including whether a host
  prefix is ever admitted and whether members carry a reserved symbol.
- **Y2 — M4-Phase 7 decides,** held by a
  [pre-1.0 candidate pool](../../../candidate-pool.md) row that a
  planning pass has to dispose of, and by the provisional note W3 places
  in the spec.

The symbol used to spell a member — a bare identifier today, a sigil such
as `^root` / `^slot` / `^host` later — is a spelling of the set rather
than a separate option, and it is triaged in the same pool row.

### M4-P3-007-010 — How a `state` named after a member is handled

- **C1 — reject the declaration.** `state root` becomes a `wasamoc check`
  error, so one identifier never holds two roles.
- **C2 — a contextual prefix.** On §4.16's model for `slot` — "not a
  reserved keyword, significant only as the head of a dotted placement
  key and a valid ordinary identifier everywhere else" — `state root`
  stays legal and the prefix is significant only in head position.

## Comparison

### M4-P3-007-001 — U1, one closed set

U3 is the defect, not a candidate: it is what makes `photos.count` a
state read and what blocks DD-001. It is listed because the record has to
say what it is displacing.

U1 and U2 are checked identically — under both, a non-member in prefix
position is an error and `photos.count` is free. They differ in whether
the spec states one rule or two, and the honest case for U2 is that the
two members are not alike:

| | `slot.` | `root.` |
|---|---|---|
| Space it labels | placement keys — a space distinct from the widget's own properties | component state — **the same space an unprefixed name reaches** |
| Writing it | **required**; an unprefixed `h-align:` is a §4.16 reject | optional; `count` and `root.count` both compile |
| Effect on the right side | decides that it resolves against a closed placement-keyword set, not against state | none |
| Lowering | normalised to a child-slot placement record (§4.12, §4.16) | discarded |

So `slot` qualifies something and `root` does not: it names the space
that is already the default when nothing is written. What the two share
is the syntactic shape — a dotted head that is not a value — and the rule
this record wants is a rule about **that shape**. U1 takes it because the
shape is what a reader meets first and what a checker validates first: an
author who writes `foo.bar` in either position is asking the same
question, and two rules would answer it twice. U2's merit is that it
would not have to explain why one member is required and the other
optional; U1 pays that explanation in prose and gets one admission rule
in exchange.

It is worth being exact about what the set buys under U1: **one member
that does work, and one that marks a read as component state in a
language where every read is component state.** The rule is still real
and DD-001 is still blocked on it, but the second member is not what
carries it.

### M4-P3-007-002 — D3, a lookup space

No document says. The spec's prose spends "root" on the content root
widget and the body root child, and the only recorded intent is a checker
comment reading "the component root alias" — in which "component root" is
itself two readings, the component *as* a root or the root *of* the
component's tree. Today one component owns one tree, so D1, D2 and D3
select the same signals and no authored `.ui` can separate them. They
separate on four questions the language has to answer anyway.

| | D1 component instance | D2 widget-tree root | D3 lookup space |
|---|---|---|---|
| Where `count` lives (§8.10) | a member of the component — matches | a **sibling** of `node Window`, so the name is read past the thing it names | the component's state space — matches |
| `root.x` inside a reusable definition placed three times | that instance's own `x` — a self | the one tree root's `x` — a reach-out to a global | the enclosing definition's `x` — a self, lexically |
| A conditional or multiplexed content root (§4.14: a distinct design, unopened) | referent unaffected | referent changes as the tree does | referent unaffected |
| What K3 would make a value | a component instance | a widget handle — the widget-id question Q1 holds | nothing; K3 becomes a separate addition |

**D2 is out on the first row alone.** §8.10's IR puts `state count` and
`node Window` side by side inside `component Counter`; §8.3 says host
attributes "live beside it … they are not children or properties of the
content root", and §4.1 says they are "never stored as `prop` entries on
the root `node Window`". Under D2 a dotted state read reaches past the
widget the prefix names to that widget's sibling, and `root.title`
failing has no available explanation. The remaining rows compound it: D2
turns a name-resolution prefix into an upward walk, which is the shape
§Context closes off, and it makes the referent depend on a content-root
design the spec has deliberately left unopened.

**D1 and D3 answer every row the same way**, and no checker behaviour or
`.ui` can tell them apart. They differ in what the spec sentence commits
to. D1 names an object the language cannot otherwise mention: no syntax
produces a component instance, none holds one, and this record's own (B)
definition says a prefix's left side is **not** a value. Asserting the
object in prose while declining to design it (003 below) leaves the spec
claiming a thing it does not support, and it turns K3 into "expose what
is already named" rather than "add something". D3 says only what is true
and checked: the prefix labels the space an unprefixed name already reaches,
which is what `check_qualified_name` resolves against.

D3's cost is that `root` is then an **unmotivated name** — nothing about
the enclosing state scope is a root, and fitting the word was the one
thing D1 supplied. That cost is real, and it is one of the reasons 004
carries the member as provisional.

### M4-P3-007-003 — K2, retain (K3 is out; K1 is M4-Phase 7's)

**K3 is out. This phase does not take on component-instance-as-value.**
Under D3 nothing denotes an instance today, so K3 is not the promotion of
an existing notion but the introduction of one — a value that invites the
questions values attract: may it be passed, held in a state, compared,
put in a collection? Nothing in AC9, in M4, or in the candidate pool asks
for any of them, and answering them here would take the custom-component
surface away from the milestone that owns it. A narrower form is
imaginable — a receiver-only value, usable in no other position, which is
what `slot` is on the placement side — but this record has not explored
it and does not claim it is unworkable. It claims only that designing it
is not this phase's work, and that 001's rule frees `photos.count`
without it. Secondarily, the word is taken: the spec's prose spends
"root" on the content root widget, so K3 taken over D1 gives one word two
meanings inside the same document. That is a naming problem, answerable
by choosing another word, and it is a cost rather than the reason K3 is
out.

**K1 versus K2 is the size of the set, not the rule.** Both satisfy 001
and **both free `photos.count`** — under K1 because no prefix exists,
under K2 because `photos` is not a member.

| | K1 retire | K2 retain |
|---|---|---|
| Authored `.ui` | 26 occurrences rewritten | unchanged |
| `dsl_spec.md` examples | rewritten | unchanged |
| `photos.count` | free | free |
| Dot in expressions | (A) only | (A), plus a checked (B) |
| The identifier `root` | stays an ordinary name | claimed under C1 (010) |
| Spellings for one state read | one | two, synonymous |
| A marker for "not ordinary local state" | none | retained |
| A later host prefix | opens (B) at that point | extends the set — but not its lowering rule |

The first two rows are the migration cost and are the ones a reader
reaches first; on their own they read as "K1 costs 26 rewrites and K2
costs nothing". That reading is wrong for two reasons priced elsewhere in
this record — the identifier claim is weighed at 010, and what K2 buys is
weighed at 001 — and for one that belongs here.

**Deferring the retirement is not cost-neutral.** Today `root` is
undefined, undocumented and unvalidated. Retiring it costs 26 mechanical
edits and no reader expectation, because nothing has ever promised it.
After this record's spec sync it is published surface in §2.4, §3, §4.16
and §5, shipping through Phases 4–6 and the public draft. **Phase 7's
option to empty the set is therefore strictly more expensive after this
record than before it**, and the phase most likely to leave it
unexercised — the negative outcome, where Phase 7 marks the host boundary
on declarations and simply never mentions prefixes — is the one where
nobody is holding the question. The split this record proposes (decide
the rule, hand the membership forward) is still right, because the rule
does not depend on Phase 7 and DD-001 is blocked on it. But the deferral
is not neutral, and the record does not present Phase 7 as inheriting the
same choice this phase has.

**The tiebreak between K1 and K2 sits in M4-Phase 7.** That phase designs
the host state boundary. If it spells host state with a prefix, the (B)
set never becomes empty and K1 would have to reopen it; if it marks the
boundary on the declaration instead, the set can be emptied and K1 is the
simpler end state. Neither answer exists yet, which is why K2 is taken
with 004 and 009 shaped to keep K1 cheap.

### M4-P3-007-004 — W3, optional with the unprefixed form canonical and the member provisional

**W1 is out, and the repository shows why.** Requiring the prefix would
give one spelling per read, which is the outcome the other two options
have to argue around. It fails on three counts. It breaks currently legal
`.ui`: five unprefixed property bindings ship today, four of them in
`gallery.ui`, so this would be a third change to what a legal file means
on top of the two this record already carries. It contradicts two shipped
rejects — §4.15 requires the *unprefixed* form for a loop collection and for a
collection mutation target — so a required prefix would need carve-outs
exactly where the language has already expressed the opposite preference.
And it makes the member maximally expensive to retire: a required prefix
is load-bearing syntax, not an optional marker, so Phase 7 would be
withdrawing a form authors are obliged to write. `slot` is required
because it qualifies something (001); `root` under D3 does not.

W2 and W3 validate identically. What separates them is whether the two
synonymous spellings are left **unranked**, and what that costs Phase 7.
W2 documents the member as settled alongside `slot`; it reads cleanly,
and it is the shape that makes the retirement cost above largest. W3
names the unprefixed form canonical and records the member as **provisional**,
with membership of the expression-side set settled at M4-Phase 7.

Naming the unprefixed form canonical is not a new preference invented here: it
is the one the §4.15 rejects already enforce where the language expresses
any preference at all. W2 would leave the spec silent on a ranking its
own diagnostics have taken.

W3 is not an invention of this record's. §4.16 already ships the pattern
for the Grid `Cell` / direct `slot.*` duplication: "That convention is
**provisional** — a future pre-1.0 decision fixes whether a wrapper form
is retained — and is not an acceptance criterion." The same sentence
shape, applied to `root`, keeps Phase 7's choice at close to today's
price and ranks the two spellings without deciding retirement.

W3's cost is that the public draft carries a surface it labels unsettled.
That is a real cost for a language approaching a freeze, and it is the
honest one: the surface **is** unsettled, and the alternative is not
settling it but concealing that it is not settled.

### M4-P3-007-006 — G1, at most one prefix

G2 has no reading to offer. Once a prefix is a checked member of a closed
set, a *chain* of prefixes is meaningless: `a.b.c.count` has no
interpretation in which `a.b` labels a lookup space. G1 also repairs the
§2.4 / §3 disagreement as a by-product — both sections then describe the
same shape, "an optional member prefix, then a name", instead of two
different ones. Keeping G2 would mean keeping a production that admits
forms no rule can explain.

### M4-P3-007-007 — L1, the same rule on both sides

Today an assignment's left-hand side is a `qualified_name` (§3) — the
same production a read uses. L2 would therefore require *adding* a
divergence: two resolutions for one production, differing by position.
The author-visible consequence of L2 is worse than the implementation
one: `root.count += 1` and `photos.count += 1` would be admitted on
different grounds than the reads spelled the same way. Whether any
non-prefix dotted target is ever admitted is a separate question and not
this record's.

One carve-out is pre-existing and L1 does not disturb it. §4.15 rejects a
qualified target and a qualified receiver in the collection positions
outright — "collection mutation requires a local state name" — which is
stricter than membership checking, not an exception to it. L1 makes the
two constraints universal; where §4.15 already admits no prefix at all,
it continues to admit none.

### M4-P3-007-009 — Y2, M4-Phase 7 decides

Y1 would require this record to settle the host-state prefix, which is
M4-Phase 7's by the accepted framing, and to settle the sigil spelling,
whose value is only measurable once a second member exists. Y2's risk is
that a deferred question is never picked up — see §Forward-compat
exposure on the negative outcome — which is why it is held twice: by a
candidate-pool row carrying a per-planning disposition duty, and by the
provisional note W3 places in a surface Phase 7 has to read anyway.

### M4-P3-007-010 — C1, reject the declaration

Validating membership means a component declaring `state root` puts one
identifier in two roles. C1 rejects the declaration; C2 keeps it legal by
reading the prefix contextually.

**C1 claims the identifier, and this record is what claims it.** That is
a claim on the author's identifier namespace which K1 does not make, and
it is not reversible after the M6 freeze. The
[candidate pool](../../../candidate-pool.md) row already carries the
argument — "a bare new prefix has to be claimed out of the identifier
namespace", so a later `host` prefix would break any `.ui` holding a
state named `host`, "while a symbol form cannot collide". The argument is
sound and it applies **now**, to `root`: this is the record that turns an
undefined token into a claimed one, and the sigil question deferred to
M4-Phase 7 is deferred **across** the point where the first collision is
created.

C2 avoids the claim and is not invented here — §4.16's contextual reading
of `slot` is exactly it. C1 is taken anyway, for two reasons. The reject
is the smaller and more legible surface: one diagnostic at the
declaration, rather than a resolution rule that has to be understood at
every use. And a contextual reading has to answer `root.root`, which is a
question with no good answer and no caller asking it. The choice is a
choice, not an absence of one, and its cost is recorded rather than
assumed away.

## Recommendation

- **001 — the left of a dot is a value; the only exceptions are members
  of a closed, validated set.** One rule spans both sides. The set today
  is `slot` on the placement-key side (§4.16) and `root` on the
  expression side; a non-member in prefix position is a `wasamoc check`
  error whose message names the members.
- **002 — a lookup space.** The prefix labels *where a name is looked
  up* — the enclosing component's state — and denotes no value. It is not
  the content root widget, which is what "root" means in the spec's
  prose; the two uses of the word are unrelated, and neither this record
  nor the prose moves for the other.
- **003 — `root` is retained and validated.** Retirement is M4-Phase 7's
  to price, and 004 and 009 are chosen to keep it affordable there.
- **004 — optional, unprefixed form canonical, member provisional.** After
  validation the prefix does nothing an unprefixed name does not: `count` and
  `root.count` are the same read. The spec records the member as
  provisional on the §4.16 model.
- **005 — `photos.count` is not a state read.** With the set closed, the
  spelling is free and DD-001 decides it on its own merits.
- **006 — one prefix at most.** `a.b.c.count` is rejected. §2.4 and §3
  stop disagreeing about how many segments a name position takes, and the
  interpolation placeholder stops claiming a segment count of its own.
- **007 — the rule reaches the assignment target.** Membership checking
  and the segment cap apply there as they do to a read. The collection
  positions keep their stricter §4.15 rule, which admits no prefix at
  all; this record does not loosen it.
- **008 — no authored `.ui` changes,** and no example in `dsl_spec.md`
  changes. The 26 prefixed occurrences and the five unprefixed ones all keep
  compiling.
- **009 — M4-Phase 7 decides later membership,** backed by a
  [candidate pool](../../../candidate-pool.md) row so a planning pass has
  to dispose of it rather than remember it.
- **010 — `state root` is a reject.** A component declaring it puts one
  identifier in two roles. This is the second of this phase's two changes
  to what a currently legal `.ui` means, and it does not expire with the
  provisional label.

Two consequences follow from the set above rather than from any single
sub-issue.

- **The expression-side prefix stays out of the IR.** `root.count` lowers
  to `(prop-read count)` as it does today; no representation, loader rule
  or C ABI surface changes. This is a property of *this* member, not of
  the set — `slot` already lowers to a placement record, and a member
  that means something would have to survive lowering.
- **`docs/dsl_spec.md` moves** — §2.4 (the placeholder stops stating a
  segment count of its own; **which production it takes is DD-001's**,
  and this record constrains only its prefix part), §3 (a name position
  is an optional member prefix plus a name, with the prefix stated as a
  lookup-space label rather than a value), §4.16 (a cross-reference
  naming `slot` as a member of the same set, and the provisional note for
  `root`), §5 (the AST carrier). `docs/abi_spec.md` does not move.

## Forward-compat exposure

- **The set's spelling can change without changing its meaning.** A
  sigil form (`^root`, `^slot`, `^host`) is a rename of a set that is
  already closed and checked, and a rename that the checker can
  diagnose. That is why the pool row can carry it to M4-Phase 7 instead
  of this record settling it. Its value there is concrete: a bare `host`
  prefix has to be claimed out of the identifier namespace, and would
  silently change the meaning of any `.ui` holding a state named `host`,
  while a sigil form cannot collide at all. What the deferral does not
  avoid is that the **first** such claim is made here, on `root` (010) —
  Phase 7 inherits a namespace already claimed once, not an untouched
  one.
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
- **A second scope arrives without an inherited global reading.** The 002
  decision fixes what `root` means once a reusable definition can be
  placed more than once: the enclosing definition's state, not the
  outermost component's. Had the denotation been left open, the reading
  most available to a reader of the published draft — the top of the
  tree — is the one that would have to be withdrawn.
- **Custom components may put pressure on scope qualification.** Today's
  no-shadowing rule holds because one scope exists; a reusable definition
  with parameters could not promise that its parameter names differ from
  every using component's state names. Whether the answer is a prefix, a
  declaration-site marking, or a lexical scoping rule is not something
  this record has compared, and it does not claim the closed set is the
  mechanism that case will use. What it claims is narrower: retaining the
  member leaves a closed, checked prefix set in place, so that surface is
  one of the available answers rather than one that has to be built from
  nothing.
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
  the question. That is why 009 carries it as a pool row with a
  per-planning disposition duty rather than as prose someone has to
  remember. The provisional label at 004 is the second holder of the same
  question: it sits in the surface Phase 7 has to read anyway, and it is
  what keeps the retirement priced near today's cost rather than at the
  cost of withdrawing settled public text.

## Technical risk re-evaluation

- **The reject is the deliverable.** Every claim above is a narrowing:
  one prefix at most, membership checked, non-members are values. A
  narrowing with no firing reject test does not exist. The close artifact
  is a table with a firing case per rejected shape — a non-member prefix,
  a chained prefix, a member prefix in a position that takes none, and a
  `state` declared with a member's name.
- **The denotation has no reject to carry it.** The 002 decision changes
  no checker behaviour and no authored `.ui`; it is what the spec says
  about a member that is otherwise inert. Its only holder is that
  sentence, so the spec sync has to state it outright rather than leave
  it to the reader, and the close artifact cannot evidence it — the
  reject table covers the set, not what a member denotes.
- **This record holds the phase's changes to what a legal `.ui` means,
  and there are two.** `photos.count` compiles today and becomes a
  diagnostic; `state root` compiles today and becomes a reject. No
  shipped example is affected by either, but a file written against the
  public draft can be, so the first message has to name the members
  rather than say "unknown state" — the failure mode to avoid is a
  diagnostic that sends the author looking for a missing declaration.
- **A prefix that is also a state name is the collision to test.** The
  reject decided at 010 has to fire on the declaration rather than at
  each use. Its cost — a claim on the identifier namespace that outlives
  the provisional label — is weighed in §Comparison, not only tested
  here.
- **DD-001 consumes this record's outcome, so the two must not drift.**
  The interrogation spelling is DD-001's; the availability of
  `photos.count` is this record's. Neither should restate the other's
  conclusion. The two records also move the same spec section: §2.4's
  target production is DD-001's to choose, and this record constrains
  only the prefix part of whatever it chooses. That coordination is
  settled at the Accepted flip and the design sync, not by either record
  predicting the other.
