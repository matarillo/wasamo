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
- **No name in the language is ambiguous without the prefix.** A binder
  can never share a name with a state — §4.15 rejects
  `state thumb: i32 = 0 … for thumb in xs { … }` as a name collision —
  and a placement value does not shadow a state of the same name
  (§4.16). There is no position in the language where writing the prefix
  changes which name is found.

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
§Comparison; the one marked as an outcome below has nothing to compare,
and its subsection says what determines it instead.

| ID | Question | Nature | Live only when |
|---|---|---|---|
| **M4-P3-007-001** | What may stand to the left of a dot | choice | — |
| **M4-P3-007-002** | What a (B) prefix denotes | choice | — |
| **M4-P3-007-003** | Whether `root` is a member of the set | choice | — |
| **M4-P3-007-004** | How a retained member is written and documented | choice | 003 retains |
| **M4-P3-007-005** | Whether `photos.count` is free for DD-001 | outcome of 001 | — |
| **M4-P3-007-006** | How many segments a name position takes | choice | — |
| **M4-P3-007-007** | Whether the rule reaches an assignment target | choice | — |
| **M4-P3-007-008** | What the authored and documented occurrences are made to say | choice | — |
| **M4-P3-007-009** | Who decides later members of the set | choice | — |
| **M4-P3-007-010** | How a `state` named after a member is handled | choice | 003 retains |

**001 is the spine.** It decides whether a closed set exists at all;
005, 006 and 007 are its reach. **003 hangs off it** — there is nothing
to be a member of until 001 answers — and 003 in turn gates 004 and 010,
which have no content if the member is retired. 009 survives retirement,
because a later member could still be admitted to an empty set.
**002 feeds 003 twice**: one of 003's options keeps a member only where
it qualifies something, which is a claim about what the prefix denotes;
and another redefines the prefix as a value, which is a different design
under each denotation. 005 is stated as an outcome rather than a choice,
because nothing can be decided about it independently of 001. **008 is a
choice under either answer at 003** — what the occurrences that teach
and the occurrences that record are made to say does not follow from the
membership decision alone.

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

- **K1 — no expression-side member.** State is reached by name; the
  expression-side membership is empty. The set, its admission rule and
  its placement-key member remain, and a later member can be admitted.
- **K2 — retain `root` as a checked member.** `root` is validated in
  prefix position and anything else to the left of a dot must be a
  value. The member qualifies nothing; it is kept as a marker, and as a
  place already held against whatever the language later wants a prefix
  for.
- **K3 — retain a member that qualifies something.** The prefix names
  the enclosing definition's *own* space — the `self` family, D1 at
  002 — and is kept because a second scope is coming: parameters on a
  reusable definition, or a host boundary. Under this option the member
  decides a resolution, and its name says what it decides; `root` is not
  that name.
- **K4 — redefine the prefix as a value.** Whatever it denotes becomes
  first-class, so `root.count` is (A) like every other dot. Which design
  that is depends on 002, which is why the two are settled in that order.

**The framing assigns emptying the set to another phase.** The accepted
[framing](../requirements/framing.md) §含まないもの gives **M4-Phase 7**
whether the expression-side set is emptied. That is a constraint on what
*accepting* this record means rather than on which option the comparison
reaches: if it decides for K1, the accept carries a framing revision
with it, and §What accepting this record means states what that revision
touches.

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

### M4-P3-007-008 — What the authored and documented occurrences are made to say

- **A1 — leave every occurrence as it is.** Available only where the
  spelling still compiles.
- **A2 — move what teaches, leave what records.** `examples/*.ui` and
  the illustrative examples in `dsl_spec.md` are written in the spelling
  the record recommends; the closed phase's evidence artifact is left
  alone, because it records what was run rather than what the language
  now recommends.
- **A3 — move everything that mentions the prefix,** the evidence
  artifact included.

Four of the prefix's appearances in `dsl_spec.md` are not examples of
good spelling but **examples of a reject**: §4.15 illustrates "the loop
collection must be a local state name" with `for x in root.xs` and
"collection mutation requires a local state name" with
`root.xs = root.xs.append(1)`. What those rows are made to say is part
of this sub-issue and is not a migration question.

### M4-P3-007-009 — Who decides later members of the set

- **Y1 — this record fixes the final set,** including whether a host
  prefix is ever admitted and whether members carry a reserved symbol.
- **Y2 — M4-Phase 7 decides,** held by a
  [pre-1.0 candidate pool](../../../candidate-pool.md) row that a
  planning pass has to dispose of.

The symbol used to spell a member — a bare identifier today, a sigil such
as `^slot` / `^host` later — is a spelling of the set rather than a
separate option, and it is triaged in the same pool row.

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

Membership under U1 is **per position**, not one list admitted
everywhere: a placement key admits `slot`, an expression admits whatever
the expression-side membership holds, and a member written in the other
position is a reject. U1 unifies the shape rule and the admission
mechanism, not the tables.

What the expression-side membership contains is 003's, and U1 stands
under either answer. If it is empty, the rule the spec states is still
one rule and still does the work DD-001 needs — a non-member in prefix
position is an error, so `photos.count` is not a state read — and it is
still where a later member is admitted. U2's merit rises in that case,
because an expression side with no members could be stated as "the left
of a dot is always a value" and nothing more. What U2 gives up is the
place a later member is admitted, which would then have to be invented
at the moment it is first needed, in the phase least able to afford it.

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
| What K4 would make a value | a component instance | a widget handle — the widget-id question Q1 holds | nothing; K4 becomes a separate addition |

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
produces a component instance and none holds one, so asserting the object
in prose while declining to design it (003) leaves the spec claiming a
thing it does not support, and it turns K4 into "expose what is already
named" rather than "add something". D3 says only what is true and
checked — the prefix labels the space a name is looked up in, which is
what `check_qualified_name` resolves against — and it is the reading
`slot` needs anyway: a placement key's head labels a space too, and it is
not a value either.

D1 is not idle, though. It is the reading a **motivated** member would
need: a `self`-shaped qualifier means the enclosing definition's own
space, and that is a self, not a lookup label. D1 is therefore the
denotation K3 selects at 003, and taking D3 here does not close that
option. It says that the member the language has **today** is a lookup
label, because nothing today needs a self.

D3's cost is that `root` is then an **unmotivated name** — nothing about
the enclosing state scope is a root, and fitting the word was the one
thing D1 supplied. That cost is real, and it is one of 003's inputs.

D3 also reaches 003 by making a test available. If a prefix labels a
space, then whether a particular prefix earns its place is answerable:
it earns it when it changes what the right-hand side resolves against.
That test is applied at 003; the reading is what makes it applicable.

### M4-P3-007-003 — K1, no expression-side member

**K4 is out. This phase does not take on component-instance-as-value.**
Under D3 nothing denotes an instance today, so K4 is not the promotion of
an existing notion but the introduction of one — a value that invites the
questions values attract: may it be passed, held in a state, compared,
put in a collection? Nothing in AC9, in M4, or in the candidate pool asks
for any of them, and answering them here would take the custom-component
surface away from the milestone that owns it. A narrower form is
imaginable — a receiver-only value, usable in no other position, which is
what `slot` is on the placement side — but this record has not explored
it and does not claim it is unworkable. It claims only that designing it
is not this phase's work, and that 001's rule frees `photos.count`
without it.

**The language already has a test for whether a prefix belongs, and
`slot` is where it comes from.** A prefix exists where it **changes what
the right-hand side resolves against**. `slot` meets it exactly:
`slot.h-align` resolves its right side against a closed
placement-keyword set instead of against state, and a placement value
does not shadow a state of the same name (§4.16). It is *required* for
the same reason — where the prefix decides the space, omitting it would
leave the space undecided.

`root` fails that test, and not narrowly. It selects the space a name
selects anyway; a binder and a state can never collide, so no name in
the language is ambiguous without it; and it reaches no representation.
There is no `.ui`, written or constructible under today's grammar, in
which writing it changes which name is found.

**K3 is the only coherent case for keeping something, and it is not yet
takeable.** Its premise is real: a reusable definition with parameters
creates a second scope, and a host boundary may create a third, and at
that point a qualifier decides a resolution and earns its place under
the same test `slot` passes. But the qualifier that case wants is a
`self` (D1), and it should be named for what it qualifies. Taking K3 now
would specify a self against a scope the language does not have, in a
milestone that does not own custom components, under a word the spec's
prose already spends on the content root widget. And K3 need not be
taken now to stay available: 001's rule is what admits a later member,
009 keeps that question live, and a member that qualifies something
passes the test whenever it arrives. **Retaining `root` is not a cheap
way of holding K3 open — it holds the place in the wrong shape, under
the wrong name.**

**That leaves K1 against K2, and the test decides them.** Both satisfy
001 and **both free `photos.count`** — under K1 because no
expression-side member exists, under K2 because `photos` is not one.

| | K1 no member | K2 retain `root` |
|---|---|---|
| Does the prefix decide a resolution | none to ask about | no, in any position the language has |
| What the spec says about the member | nothing on the expression side | that it labels the space a name already reaches — a sentence stating that it does no work |
| The word "root" | keeps its one meaning, the content root widget (§4.14, §4.16) | carries two unrelated meanings in one document |
| Spellings for one state read | one | two, synonymous, to the freeze |
| Collection positions (§4.15) | admit no dotted head, by the general rule | keep a carve-out saying the member is not admitted here either |
| The identifier `root` | stays an ordinary name | claimed, or made contextual (010) |
| A later member that qualifies something | admitted to an empty set by 001's rule | admitted to a set of one, by the same rule |

The fifth row is the one that is easy to miss, and it is where retention
is paid for permanently. §4.15 rejects a qualified loop collection and a
qualified mutation target outright, which is **stricter** than
membership checking. Under K2 that stays a carve-out the language states
and a reader learns: the member is admissible in expressions, and not
there. Under K1 those positions need no rule of their own — no dotted
head reaches them, because the general rule already refuses one.
Retention buys an inert marker and pays for it with an exception in the
two positions where the language has already said what it prefers.

**What K1 gives up.** The marker that says "this read is component
state" disappears — in a language where every read is component state,
which is why it marks nothing today. If the owner's judgement is that
the second scope is close and certain enough to want a qualifier ahead
of it, the position that follows is K3, not K2, and it is a framing
revision either way.

**The author-facing cost is real and belongs to the public draft.**
`dsl_spec.md` is a public draft that teaches the prefix in its examples,
so an external `.ui` written against it stops compiling. Nothing has
ever *defined* the identifier, and the draft carries no compatibility
commitment before 1.0 — but examples are what readers copy, and this is
the honest price of the recommendation. It is also strictly rising: the
same break costs more in every later phase than it does here.

**Migration is a tie-breaker, and it points the same way.** K1 rewrites
24 occurrences in `examples/` and nine example lines in `dsl_spec.md`;
the rewrite is mechanical, diagnosable, and moves no meaning, and 008
settles the four reject-illustrating places and the two occurrences in a
closed phase's evidence artifact. That cost is not what decides this
sub-issue. It is worth recording only because it is the smallest it will
ever be: after this record's spec sync the spelling is published surface
in §2.4, §3, §4.16 and §5, carried through Phases 4–6.

### M4-P3-007-004 — not live under the recommendation

004 has content only if a member is retained. With the expression-side
membership empty there is no member to require, to rank against a
prefix-less spelling, or to label provisional: the spec states the
spelling once instead of twice, and W1's stated benefit — one spelling
per read — arrives without obliging anyone to write a form that decides
nothing.

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

§4.15's collection positions are unaffected in substance and simplified
in statement. They reject a qualified target and a qualified receiver
outright — "collection mutation requires a local state name" — which
under a retained member would be a carve-out *stricter* than membership
checking. With the expression-side membership empty there is nothing to
carve out: a dotted head in those positions is refused by the general
rule, because no member exists to admit and the left of a dot must be a
value.

### M4-P3-007-008 — A2, move what teaches and leave what records

Under K1 the occurrences that must keep compiling move: 24 in
`examples/` and nine example lines in `dsl_spec.md`. The choice is about
the rest.

The two occurrences in
[t3-label-update.ui](../../phase-1/implementation/evidence/t3-label-update.ui)
are inputs to a closed phase's evidence capture, not a fixture any build
compiles. A3 would edit them so the file still compiles under the new
rule, which makes the artifact stop describing what was actually run —
the one thing an evidence artifact is for. A2 leaves it, and a phase
that ever re-runs that capture migrates a copy at that point.

The four places where `dsl_spec.md` spells a **reject** with the prefix
are a correctness question rather than a migration one. Under K1
`for x in root.xs` no longer reaches "the loop collection must be a
local state name"; it is refused earlier, as a dotted head that is not a
value. Left alone, those rows would document a diagnostic the compiler
no longer produces for the example printed beside them. Either the rows
are respelled to a shape that still reaches the rule they illustrate, or
the rule is recorded as subsumed by the general one — which of the two
is a spec-sync judgement made against the diagnostics that actually
fire, and this record does not predict it.

### M4-P3-007-009 — Y2, M4-Phase 7 decides

Y1 would require this record to settle the host-state prefix, which is
M4-Phase 7's by the accepted framing, and to settle the sigil spelling,
whose value is only measurable once a second member exists. Y2's standing
risk is that a deferred question is never picked up, which is why it is
held by a candidate-pool row carrying a per-planning disposition duty.
Settling membership at 003 removes the sharper half of that risk: the
question Phase 7 now inherits is what its *own* boundary needs, which
its deliverables cannot avoid answering, rather than whether a member it
never used should be withdrawn.

### M4-P3-007-010 — not live under the recommendation

With no expression-side member there is no identifier to claim: `state
root` stays legal, and `root` stays an ordinary name. **This record
claims nothing out of the author's namespace**, which is what the
[candidate pool](../../../candidate-pool.md) row asks of a bare-identifier
prefix — "a bare new prefix has to be claimed out of the identifier
namespace … while a symbol form cannot collide". M4-Phase 7 therefore
reaches the sigil question over a namespace no prefix has touched.

## Recommendation

- **001 — the left of a dot is a value; the only exceptions are members
  of a closed, validated set.** One rule spans both sides, and
  **membership is per position**: a placement key admits `slot` (§4.16),
  an expression admits the expression-side membership, and a member
  written in the other position is a reject. A non-member in prefix
  position is a `wasamoc check` error whose message names what is
  admissible there.
- **002 — a lookup space.** A member labels *where a name is looked up*
  and denotes no value. It is not the content root widget, which is what
  "root" means in the spec's prose; the two uses of the word are
  unrelated, and neither this record nor the prose moves for the other.
- **003 — the expression-side membership is empty; `root` is retired.**
  A prefix belongs where it changes what the right-hand side resolves
  against — the test `slot` passes and `root` fails in every position
  the language has. What survives is the set, its admission rule and its
  placement-key member; what goes is a member that decides nothing,
  under a word the spec spends on something else. **This is a change to
  the accepted framing** — see §What accepting this record means.
- **004 — not live.** With no expression-side member there is nothing to
  require, to rank, or to record as provisional.
- **005 — `photos.count` is not a state read.** With the set closed, the
  spelling is free and DD-001 decides it on its own merits.
- **006 — one prefix at most.** `a.b.c.count` is rejected. §2.4 and §3
  stop disagreeing about how many segments a name position takes, and the
  interpolation placeholder stops claiming a segment count of its own.
- **007 — the rule reaches the assignment target.** The segment cap and
  the membership check apply there as they do to a read. §4.15's
  collection positions need no rule of their own under this
  recommendation: no dotted head reaches them.
- **008 — what teaches moves; what records does not.** The 24
  occurrences in `examples/` and the nine example lines in
  `dsl_spec.md` are rewritten without the prefix. The two occurrences in
  M4-Phase 1's evidence artifact stay as the record of what was run. The
  four places that spell a §4.15 reject with the prefix are re-examined
  at spec sync against the diagnostics that actually fire.
- **009 — M4-Phase 7 decides later membership,** backed by a
  [candidate pool](../../../candidate-pool.md) row so a planning pass has
  to dispose of it rather than remember it. Retirement does not close
  that question: a member that qualifies something is admitted to an
  empty set by 001's rule, and the sigil question is unchanged.
- **010 — not live.** `state root` stays legal and `root` stays an
  ordinary identifier. This record claims nothing out of the author's
  namespace.

Two consequences follow from the set above rather than from any single
sub-issue.

- **The expression side gains no IR surface and loses none.** A name in
  expression position lowers exactly as it does today — `count` and
  today's `root.count` both emit `(prop-read count)` — so no
  representation, loader rule or C ABI surface changes. That is a
  property of removing an inert member, not a general property of the
  set: `slot` lowers to a placement record, and a member that decides a
  resolution would have to survive lowering.
- **`docs/dsl_spec.md` moves** — §2.4 (the placeholder stops stating a
  segment count of its own; **which production it takes is DD-001's**,
  and this record constrains only its prefix part), §3 (a name position
  is an optional member prefix plus a name, with membership stated per
  position and the prefix stated as a lookup-space label rather than a
  value), §4.15 (the two reject rows spelled with the prefix), §4.16 (a
  cross-reference naming `slot` as the set's one member), §5 (the AST
  carrier), and the examples that teach the spelling. `docs/abi_spec.md`
  does not move.

### What accepting this record means

003 is outside what the accepted framing gives this phase.
§含まないもの assigns to **M4-Phase 7** whether the expression-side set
is emptied, and §DD と検証手段の対応 names the 26 surviving `root.`
occurrences as this record's positive control. Accepting the
recommendation therefore carries a **framing revision** with it — tier 2
refining, on the model of
[Revision 7](../requirements/plan-revision-7-proposal.md) — touching:

| Target | Edit |
|---|---|
| framing §含まないもの | Membership of the expression-side set is settled here. What stays M4-Phase 7's is the host-state prefix spelling and whether members carry a reserved symbol |
| framing §DD と検証手段の対応, DD-007 row | The positive control becomes the migrated `examples/` compiling without the prefix, not the 26 occurrences surviving |
| [candidate pool](../../../candidate-pool.md), prefix-set row | Part (1), membership, is discharged; part (2), spelling, stays open |
| `plan.md` §Revision log | One entry |

**There is no fallback inside this record.** Without the revision, 003
keeps a member, 004 and 010 become live, and 008 has a different answer —
a materially different record, proposed as one rather than read out of
this one.

## Forward-compat exposure

- **The set's spelling can change without changing its meaning.** A
  sigil form (`^slot`, `^host`) is a rename of a set that is already
  closed and checked, and a rename the checker can diagnose. That is why
  the pool row can carry it to M4-Phase 7 instead of this record
  settling it. Its value there is concrete: a bare `host` prefix has to
  be claimed out of the identifier namespace, and would silently change
  the meaning of any `.ui` holding a state named `host`, while a sigil
  form cannot collide at all. Phase 7 inherits that question over an
  **untouched** namespace, because this record claims no identifier.
- **A later member need not be erasable at lowering.** With the
  expression side empty, nothing there survives lowering — but that is
  the absence of a member, not a property of the set. `slot` already
  lowers to a placement record, and a host prefix could not be erased at
  all: where a name is read from is exactly what the runtime would need
  to keep. The closed set as specified here is a rule about
  **admission**, and whether it is also a rule about **resolution**
  opens the moment it gains a member that decides one. Phase 7 is where
  that is discovered, and it should not read a "prefixes are discarded"
  sentence as a constraint it inherits.
- **Structured data makes the (A) rule load-bearing rather than
  decorative.** When element fields arrive, `photo.filename` resolves
  through the receiver's type — which is coherent only if everything to
  the left of a non-prefix dot is a value. This record is what makes that
  true in advance.
- **A second scope arrives without an inherited global reading.** The
  002 decision fixes what a member denotes before a reusable definition
  can be placed more than once: a lookup space, resolved where the
  expression sits, never the top of the tree. Had the denotation been
  left open, the reading most available to a reader of the published
  draft — the top of the tree — is the one that would later have to be
  withdrawn.
- **Custom components may put pressure on scope qualification, and that
  is where a motivated member would arrive.** Today's no-shadowing rule
  holds because one scope exists; a reusable definition with parameters
  could not promise that its parameter names differ from every using
  component's state names. Whether the answer is a prefix, a
  declaration-site marking, or a lexical scoping rule is not something
  this record has compared. What retirement costs that case is bounded
  and stated: the admission rule, the reject and the diagnostic all
  remain, so a `self`-shaped member (K3) is admitted by machinery that
  already exists rather than built from nothing — and it arrives named
  for what it qualifies instead of inheriting a word the spec spends on
  the content root widget.
- **Tree traversal stays closed.** No form here names an element, and
  the widget-id question remains
  [dsl-grammar.md](../../../../docs/notes/dsl-grammar.md) Q1's. If a
  named-element reference is ever admitted it is a single hop, not a
  path; an index-shaped walk **into the widget tree** is what this
  record's rule makes hardest to add, deliberately. Indexing a
  collection is a separate question and is DD-001's.
- **Phase 7 inherits a question with no negative outcome attached.**
  Under a retained member, the question "does `root` still have work to
  do" fires only if Phase 7 declines to use a prefix — an outcome that
  produces no deliverable, so nobody would be holding it. Settling
  membership here removes that trap: Phase 7 decides whether its own
  boundary wants a prefix, and if it does not, there is nothing left
  unexamined. The pool row still carries the spelling question, which
  fires either way.

## Technical risk re-evaluation

- **The reject is the deliverable.** Every claim above is a narrowing:
  one prefix at most, membership checked, non-members are values. A
  narrowing with no firing reject test does not exist. The close artifact
  is a table with a firing case per rejected shape — a prefix in
  expression position at all (`root.count` included), a chained prefix,
  and `slot` written where no placement key is admitted. The positive
  control moves with the recommendation: it is the migrated `examples/`
  compiling, not the prefixed occurrences surviving.
- **The denotation has no reject to carry it.** The 002 decision changes
  no checker behaviour; it is what the spec says a member's head is. Its
  only holder is that sentence, so the spec sync has to state it outright
  rather than leave it to the reader, and the close artifact cannot
  evidence it — the reject table covers the set, not what a member
  denotes.
- **This record holds the phase's change to what a legal `.ui` means,
  and it is wider than a diagnostic on one spelling.** Every prefixed
  read and write stops compiling, the repository's own 24 included until
  they are migrated (008). The first message therefore has to say that a
  prefix is not admitted in expression position — the failure mode to
  avoid is "undefined state `count`", which sends the author looking for
  a missing declaration. The exposed party is a reader of the public
  draft who copied an example, which is why 008 moves the examples in the
  same pass as the rule.
- **DD-001 consumes this record's outcome, so the two must not drift.**
  The interrogation spelling is DD-001's; the availability of
  `photos.count` is this record's. Neither should restate the other's
  conclusion. The two records also move the same spec section: §2.4's
  target production is DD-001's to choose, and this record constrains
  only the prefix part of whatever it chooses. That coordination is
  settled at the Accepted flip and the design sync, not by either record
  predicting the other.
- **DD-006 owns the assignment gate; this record narrows what reaches
  it.** 007 puts the segment cap and the membership check on an
  assignment's left-hand side, while DD-006's completeness claim is that
  every assignment form passes exactly one capability gate and one type
  gate. These are different judgements over different parts of the
  statement — the left-hand side's *shape* is settled here, what may be
  written to it is DD-006's — and neither record should state the
  other's rule. Where the shape check runs relative to DD-006's gate is
  an ordering question, answered once in the phase's implementation plan
  rather than twice in the ADRs.
