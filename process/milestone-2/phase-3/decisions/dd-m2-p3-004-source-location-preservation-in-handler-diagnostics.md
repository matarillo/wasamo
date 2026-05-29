### DD-M2-P3-004 — Source location preservation in handler diagnostics

**Status:** Accepted

**Context:**
DD-M2-P3-003 commits the runtime to logging handler errors. The
quality of those logs depends on whether IR carries source
positions back to `.ui` line:column.

M1 `wasamoc` ([wasamoc/src/ast.rs](../../wasamoc/src/ast.rs))
already tracks `Span` on every AST node. The IR (DD-M2-P2-002 = B,
textual) can carry an optional `(span L:C)` annotation per
expression at modest cost. The question is whether M2 *uses* those
spans in runtime diagnostics, and whether the IR grammar *requires*
spans (forces them on every expression) or *permits* them
(annotation is optional).

The DSL spec public draft is M3 work; LSP / editor diagnostics
(VS Code) is M5 work. Both will eventually need source-mapped
handler diagnostics.

**Options:**

Option A — Required at M2: every IR expression carries a span;
runtime diagnostics include `counter.ui:19:30`-style positions
- `wasamoc` always emits `(span L:C)` on every IR node.
- Runtime diagnostics surface it.

- What you gain: Best diagnostic quality immediately. M3 LSP work
  has a stable "spans are present and reliable" foundation already
  exercised.
- What you give up: Larger IR files (modest at counter scale,
  unmeasured at any larger scale). Spans become part of the IR
  grammar contract; later relaxation (omit spans from generated
  IR) would be a breaking change. Implementation surface in
  `wasamoc` IR printer + runtime IR loader + runtime diagnostic
  formatter — three places that all have to stay in sync from day
  one.
- **Technical risk: Low–medium.** Mechanically straightforward
  (the AST already tracks spans; threading them through the IR is
  bookkeeping). Risk is "now it's a contract" — relaxing the
  requirement later costs an IR-version break.

Option B — Deferred to M3: IR grammar permits an optional `(span L:C)`;
M2 wasamoc may emit it or not; M2 runtime ignores it (recommended)
- `wasamoc` emits spans only where doing so is trivially aligned
  with existing AST traversal (free); other paths emit no span.
- M2 runtime diagnostics use a coarse identifier:
  `<component>.<widget-path>.<signal>` (e.g.
  `Counter.button[1].clicked`) — derived from IR structure without
  needing source positions.
- M3 (DSL surface) revisits and decides whether to require spans.

- What you gain: M2 ships without committing to a span-emission
  contract before the DSL spec is drafted; M3's DSL spec work
  decides span policy alongside grammar formalization, in one
  coherent ADR. The IR's `(span ...)` slot is reserved (grammar
  permits but does not require), so M3 can tighten the rule from
  "permitted" to "required" without an IR-format break. Coarse
  identifiers are good enough for Hello-Counter scale debugging
  (one component, two widgets).
- What you give up: M2 error logs say `Counter.button[1].clicked`
  not `counter.ui:19:30`. Acceptable for M2 acceptance scope (one
  developer debugging their own one-component DSL); inadequate for
  a real LSP, which is M5 work.
- **Technical risk: Low.** Reserving an optional grammar slot is
  cheaper than wiring full span propagation. The risk of
  *deferring* is that M3's eventual decision turns out to require
  an IR-format break — mitigated by reserving the slot now so the
  worst-case break is "spans become required" (additive on consumers
  that already accept them as optional, breaking only on the
  producer side, which is one tool: `wasamoc`).

Option C — Required at M2 **and** the runtime exposes a structured
diagnostic API to hosts
- A on top of B's host-callback shape from DD-M2-P3-003 Option B.

- What you gain: Future-ready.
- What you give up: Bundles two M3+ commitments (full spans + host
  diagnostic API) into M2 with no acceptance criterion driving
  either. Premature.
- **Technical risk: Medium** (combines A's contract risk with the
  added ABI surface and re-entrancy of B).

**Recommendation:** **Option B.**

The IR grammar reserves an optional `(span L:C)` annotation; M2
emits it where convenient and ignores it on the runtime side.
Diagnostics use coarse component-and-widget-path identifiers. M3,
which is where the DSL spec public draft and the bulk of editor /
LSP groundwork lives, decides whether to tighten the optional
slot into a requirement. This avoids a contract commitment ahead
of the spec work that will refine it, while not foreclosing any
future tightening.

The M2-Phase 6 implementation in `wasamoc` is encouraged to thread
spans through the IR opportunistically (cost is bookkeeping, not
design), so that M3 has a working baseline rather than a clean
slate. But **the Phase 6 task list does not require it**, and the
runtime treats spans as optional.

**Technical-risk re-evaluation:** Option B is the lowest-risk
choice that does not foreclose the eventual M3 outcome. Option A
is also low-risk but converts a future decision into a present
commitment with no current beneficiary. Option C is overcommitment.

**Forward-compat exposure:** Options differ; this axis is the
dominant one for this DD. The relevant out-of-scope item is
"Tightening DD-M2-P3-004 to require spans" — M3's DSL spec work
decides whether spans become required, and what shape that
requirement takes (positions only, or richer provenance such as
macro-expansion stacks and generated-code origin).

- Option A commits M2 to `(span L:C)` as the IR span shape. If M3
  ultimately requires a different shape, M2 producers and consumers
  are both wrong, and an IR-format break is needed.
- Option B reserves the slot as optional: M2 producers may emit
  nothing, M2 consumers ignore the slot. M3 can replace the slot's
  payload (`(span L:C)` → `(origin …)`) without breaking either
  side — the deferral is itself the mitigation.
- Option C inherits Option A's shape commitment.

Implementation risk is similar across Options A and B; this axis is
what tilts the recommendation to B.

---
