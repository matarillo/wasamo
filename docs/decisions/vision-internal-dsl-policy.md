# Vision Revision — Canonical Status of `.ui` and the Role of Internal DSLs

**Date:** 2026-04-30
**Status:** Accepted
**Triggered by:** Pre-Phase-7 review of how host-language bindings relate to the `.ui` DSL

---

## Context

VISION.md §2.2 names the project's core hypothesis as
**"external DSL × C ABI × Visual Layer"**. The phrasing leaves a
question unresolved: what status, if any, do *internal* DSLs built on
top of host-language bindings have?

Three observations make this question worth settling now rather than
later:

1. **The C ABI already permits bottom-up tree construction.** Phase 6
   shipped an experimental layer (`wasamo_text_create`,
   `wasamo_vstack_create`, `wasamo_button_set_clicked`, …) that lets a
   host build the widget tree directly. Any sufficiently expressive
   binding can wrap this in language-idiomatic sugar — Rust macros,
   Swift result builders, Zig `comptime`, Go builders — and the result
   is, in effect, an internal DSL. This is not hypothetical; it is the
   default trajectory of a successful binding.

2. **Language-idiomatic expression has real DX value.** The audience
   VISION.md §3.1 names (Rust / Swift / Zig / Go developers) arrives
   with established expectations about what feels native in their
   language. Forcing all UI declaration through `.ui` files when an
   internal DSL would feel more natural in that language sacrifices
   developer experience without serving any architectural goal.

3. **Equal canonical status would dissolve the hypothesis.** §2.2's
   reasoning for choosing an external DSL is that "every language gets
   the same declarative experience". If internal DSLs stand on equal
   canonical footing with `.ui`, language ecosystems will diverge —
   each evolving its own dialect, its own conventions, its own
   tooling. That is precisely the failure mode an external DSL was
   chosen to avoid.

The tension is real but resolvable. `.ui` can remain the canonical,
language-neutral form while internal DSLs are welcomed as derivative
shapes that serve language-specific DX without claiming canonical
status. The substantive question is what gates the evolution of `.ui`
itself once internal DSL experiments begin to surface gaps.

---

### DD-V-002 — `.ui` is canonical; internal DSLs are welcome derivatives

**Status:** Accepted

**Context:**
What is the relationship between `.ui` and host-language internal DSLs
that wrap the C ABI? The three observations above bound the answer:
language-idiomatic expression should be welcomed; canonical fragmentation
must be avoided.

**Options:**

Option A — `.ui` is canonical; internal DSLs are derivative and welcome
- What you gain: Preserves the §2.2 property that every language sees
  the same declarative form when interoperating around `.ui` (LSP,
  tooling, design-system components, hot reload, third-party
  implementations of the spec). Captures the DX benefit of
  language-idiomatic expression where the host community wants it,
  without obligating every binding to provide one.
- What you give up: The project takes on an explicit asymmetry —
  internal DSLs may legitimately be richer in some directions than
  `.ui`, and that is *acceptable rather than a problem to be fixed*.
  Some users will read this as `.ui` being "behind" their language's
  internal DSL.

Option B — Equal canonical status for `.ui` and any sufficiently
complete internal DSL
- What you gain: Maximum freedom for language ecosystems to optimize
  for their own idioms. No friction between binding authors and the
  canonical form.
- What you give up: The cross-language uniformity that motivated the
  external DSL choice. Tooling fragments along language lines (each
  ecosystem evolves its own LSP, its own preview, its own hot reload).
  Design-system components written in one ecosystem's internal DSL
  cannot be consumed from another. Third-party `.ui` implementations
  lose their meaning as alternative frontends because there is no
  single canonical form to implement.

Option C — `.ui` is the only sanctioned form; internal DSLs are
discouraged or unsupported
- What you gain: Strongest possible guarantee of cross-language
  uniformity. No risk of ecosystem-specific drift.
- What you give up: Active hostility toward the natural shape of every
  successful host-language binding. The C ABI already permits the
  construction patterns that make internal DSLs possible, so a "no
  internal DSLs" stance would be unenforceable in practice and would
  alienate binding authors who are core ecosystem contributors
  (VISION.md §3.2).

**Decision:** Option A — `.ui` is the canonical form. Internal DSLs
built on top of host-language bindings are welcomed as derivative
shapes serving language-specific developer experience. They do not
claim canonical status, and the project does not attempt to keep `.ui`
at feature parity with the most expressive internal DSL.

---

### DD-V-003 — Conditions under which `.ui` and the C ABI evolve in response to internal DSL experience

**Status:** Accepted

**Context:**
DD-V-002 establishes that `.ui` need not absorb everything an internal
DSL can express. But internal DSL experiments will, over time, reveal
gaps in `.ui`, and binding authors will request changes. Without an
explicit gate, two failure modes are likely: (a) `.ui` accretes
features driven by whichever language ecosystem moves fastest,
biasing the canonical form toward that ecosystem's idioms; (b) C ABI
additions accumulate to support specific bindings, undermining the
M4 stability commitment (VISION.md §7).

**Options:**

Option A — Gate `.ui` extension on product motivation and
cross-binding expressibility
- What you gain: Two filters that together preserve the project's
  load-bearing properties. **Product motivation** keeps `.ui`
  evolution tied to capability that end users of Wasamo apps actually
  need, rather than to internal-layer ergonomics. **Cross-binding
  expressibility** preserves the property that any feature in `.ui`
  can be consumed by every officially supported binding — without
  this, `.ui` quietly becomes a language-specific document.
- What you give up: Some changes that would be locally convenient for
  one binding will not pass the gate. Binding authors must accept
  that not every refinement they want from `.ui` is in scope.

Option B — Gate `.ui` extension on internal-DSL feature pressure alone
- What you gain: Clear, observable signal — extend `.ui` when N
  bindings have implemented feature X.
- What you give up: The signal is the wrong one. Convergence among
  internal DSLs reflects what is *idiomatically expressible* in those
  languages, not what end-user products need. Following this gate
  reliably produces feature accretion without product justification,
  and biases `.ui` toward features that several languages happen to
  share rather than features that serve users.

Option C — Treat `.ui` as frozen at the M4 specification baseline
- What you gain: Maximum stability of the canonical form.
- What you give up: Premature freeze. M1–M3 have not yet generated
  enough operational evidence about real product needs for the
  spec to be stable; freezing at M4 already requires confidence that
  the surface is correct.

**Decision:** Option A — `.ui` is extended only when the proposed
feature is **(i)** motivated by an end-user product capability that
cannot be provided through bindings or design-system components, and
**(ii)** expressible across all officially supported bindings.

C ABI changes follow the same review process as any other ABI
proposal under the M4 stability commitment; no special review path
exists for internal-DSL-driven additions, and pre-M4 additions are
weighed against the same permanence cost as post-M4 additions.

When multiple bindings independently converge on the same internal
pattern, that observation may call for one of three responses, judged
case by case: (a) a binding-author convention document that records
the pattern without runtime or `.ui` change; (b) a `.ui` extension
proposal that meets the gate above; (c) leaving the pattern as
parallel community evolution. The default response is (a) or (c); (b)
is reserved for cases that meet both gate conditions on their own
merits.

---

### DD-V-004 — Treat M1–M3 as an observation period for `.ui` evolution

**Status:** Accepted

**Context:**
Internal DSL experiments have not yet happened; product usage of
Wasamo at any scale has not yet happened. Decisions to extend `.ui`
made during this period would be made on speculation rather than
operational evidence.

**Options:**

Option A — Defer `.ui` extension proposals until M3 unless blocking a
milestone acceptance criterion
- What you gain: Decisions about canonical-form evolution wait for
  evidence from real bindings, real internal DSL experiments, and
  real applications. Reduces the risk of locking in shapes that look
  correct on paper but reveal problems under load.
- What you give up: Some genuine improvements wait. Binding authors
  who hit `.ui` limits during M1–M2 must work around them in their
  binding rather than push the limit upstream.

Option B — Evaluate `.ui` extension proposals continuously from M1
onward
- What you gain: No artificial delay on improvements.
- What you give up: The shape of `.ui` is pinned by decisions made
  before the project has the operational evidence to make them well.

**Decision:** Option A — During M1–M3, the default disposition for
`.ui` extension proposals is to record them as observations and defer
substantive design until M3 review. Exceptions are proposals that
unblock a milestone acceptance criterion (VISION.md §7), which are
handled through the normal phase ADR process.

---

## Documents to be updated alongside this ADR

| Document / section | Change | Rationale |
|---|---|---|
| VISION.md §2.2 | Add a clarifying sentence after the three bullet points stating that internal DSLs built on bindings are welcomed as derivative shapes, with `.ui` remaining canonical | Make explicit what DD-V-002 settles, so the hypothesis statement is not misread as forbidding internal DSLs |
| VISION.md §9.4 | Expand "Independence of the core specification" with a paragraph naming the canonical/derivative split and the conditions under which `.ui` evolves (DD-V-002, DD-V-003) | §9.4 is already where the project's commitment to spec independence lives; canonical-form policy is the same topic |
| VISION.md §3.2 (Secondary users) | No change required; "Authors of alternative DSL implementations" already covers internal DSL authors implicitly | — |
| ROADMAP.md | No change required for M1–M5. M3 review milestone may pick up `.ui` extension proposals deferred under DD-V-004 | DD-V-004 affects timing of `.ui` evolution but not any milestone's acceptance criteria |

---

## Relation to phase ADRs

This ADR does not supersede any prior decision. It clarifies a vision-
level question that was previously implicit, before Phase 7 begins
work that may surface concrete `.ui` extension proposals. Phase ADRs
that touch `.ui` evolution from this point forward should reference
DD-V-003 rather than restating the gate conditions.
