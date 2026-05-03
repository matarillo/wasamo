# M2-Phase 3 — Handler execution location: Architecture Decisions

**Phase:** M2-Phase 3 (DSL inline handler execution location)
**Date:** 2026-05-04
**Status:** Proposed

## Context

[Phase 6 ADR](./phase-6-c-abi.md) explicitly deferred two questions to
M2 to keep the stable C ABI core neutral:

> **(a)** Where DSL inline handler bodies (`clicked => { … }`) execute
> — host-side trampoline vs runtime-side interpreter.

This ADR resolves question (a). [M2-Phase 2 ADR](./m2-phase-2-wasamoc-output-format.md)
resolved (b) by Accepting Option B (compile to IR + runtime interpreter
inside `wasamo-runtime`), with feasibility verified by the
`exp/m2-p2-ir-loader-spike` branch.

DD-M2-P2-004 recorded the relationship between Phase 2 and Phase 3 as
one-directional: Phase 2's outcome reduces Phase 3's option space, but
not vice versa. With DD-M2-P2-001 = B Accepted, Phase 3 is "a real
decision" (per the table in DD-M2-P2-004), not a foregone conclusion;
both runtime-side interpretation and host-side trampoline remain
viable shapes, and this ADR picks between them.

### What is "an inline handler"?

In `examples/counter/counter.ui` the only inline handler is

```
clicked => { root.count += 1; }
```

attached to the Increment button. In the M1 AST
([wasamoc/src/ast.rs:98](../../wasamoc/src/ast.rs#L98)) this is a
`Member::SignalHandler { signal, body, span }` — a `Block` of
statements that runs when the named signal fires on the enclosing
widget. M2 acceptance criterion **A1** requires this construct to
execute end-to-end driven by `counter.ui` rather than reproduced by
hand in each host language. **Where the handler body runs, and how it
interacts with `wasamo_signal_connect` host listeners, is the question
this ADR answers.**

### Constraints carried in from prior decisions

- **DD-M2-P2-001 = Option B.** Handler bodies are stored in the IR as
  typed expressions (DD-M2-P2-003 activity 7). The runtime interpreter
  exists; the question is whether *handler-body evaluation* is one
  more job for that interpreter or a separate path that round-trips
  through the host.
- **DD-P6-002 (signal model).** `wasamo_signal_connect(widget, name,
  cb, user_data, &out_token)` with string-keyed, tagged-value payload
  is the stable-core mechanism for hosts to observe widget signals.
  This ADR does **not** modify that mechanism; it decides how DSL
  inline handlers coexist with it (DD-M2-P3-002).
- **Acceptance A2 (reactive propagation).** State assignments inside a
  handler (`root.count += 1`) must drive the reactive engine without
  the host writing `wasamo_set_property` by hand. The handler's
  execution location determines whether the reactive engine sees the
  write directly (runtime-side) or only after a C-ABI round trip
  (host-side).
- **Acceptance A4 (tree-mutation ABI).** Out of scope for this ADR;
  decided by M2-Phase 4. This ADR constrains Phase 4 only insofar as
  any handler-side state mutation (property assign, child append) must
  reach the runtime through *some* path — direct calls if runtime-side,
  C ABI if host-side.
- **Binding workload scaling** (recurring constraint from
  [VISION §11](../../VISION.md), reused from DD-M2-P2-001 framing).
  Anything that requires a per-binding-language code path increases
  the cost of adding a new binding; anything that lives once in
  `wasamo-runtime` is paid for once.
- **Pre-existing experimental setter.** The M1 experimental layer's
  `wasamo_button_set_clicked` ([phase-6-c-abi.md DD-P6-002](./phase-6-c-abi.md#dd-p6-002--signal-model))
  is a per-widget typed setter, not an inline-handler mechanism. It
  is preserved as-is for M1 hosts and is not in scope here.

---

### DD-M2-P3-001 — Where DSL inline handler bodies execute

**Status:** Proposed

**Context:**
With DD-M2-P2-001 = B, the IR carries handler bodies as typed
expressions. Two real points in the design space:

1. The runtime interpreter evaluates the handler body directly when
   the signal fires.
2. The runtime synthesizes a signal emission; the binding-side
   trampoline (generated or hand-written) invokes a host-language
   transliteration of the handler body.

**Options:**

Option A — Runtime-side interpreter (recommended)
- The runtime stores each inline handler's IR expression alongside
  the widget. When the underlying signal fires (e.g. button click),
  the runtime walks the expression tree against an evaluation context
  that resolves `root.count` to the live property storage and
  evaluates the assignment in place.
- `+=`, `-=`, property reads/writes, and the small set of expression
  forms used in handler bodies are evaluated by the same interpreter
  that evaluates property bindings (per DD-M2-P2-003 activity 6).
- Hosts do not need to register a callback for inline-handler
  bodies; they appear from the host's perspective as state changes
  observable through the existing property-observer mechanism
  ([DD-P6-001 area 4](./phase-6-c-abi.md#dd-p6-001--stable-core-scope-at-function-granularity)).

- What you gain: One evaluator (in `wasamo-runtime`) serves all
  bindings — adding a new binding language is "wire up the C ABI",
  matching the DD-M2-P2-001 layering. The reactive engine
  (M2-Phase 5) sees property writes in-process and can invalidate
  immediately, with no C-ABI round trip per write. Handler logic is
  decoupled from host-language toolchains: no per-language
  expression emitter, no per-language operator-overload reproduction.
  Hot reload (post-1.0) stays feasible because handler bodies live
  in the swappable IR file, not in compiled host code.
- What you give up: A handler-body evaluator is a new component to
  design and maintain — a strict subset of a small expression
  language (assignment, compound assignment, property read,
  arithmetic, comparison, possibly `if`). Handler bodies cannot call
  arbitrary host functions; calls into host-defined functions
  require an explicit binding mechanism (out of scope for M2 — only
  property assignments and arithmetic are needed for Hello Counter).
  Stack traces for handler errors live in IR-position terms, not
  host-language terms (see DD-M2-P3-004 for source-location
  treatment).
- **Technical risk: Medium.** A handler-body evaluator is a small
  superset of the expression evaluator that DD-M2-P2-003 activity 6
  already requires for property bindings. The Phase 2 spike
  exercised the IR-walker → internal-builder path; the additional
  surface for handler bodies is the assignment/compound-assignment
  case and side-effecting evaluation order, both well-understood.
  The risk is **largely shared with M2-Phase 5** (the reactive
  engine needs to evaluate property-binding expressions on
  invalidation regardless of where handlers run); choosing A locates
  evaluation in one place rather than two.

Option B — Host-side trampoline (synthetic signal)
- For each inline handler in the DSL, `wasamoc` emits IR that
  describes the handler in terms the binding can re-emit as host
  code, **and** the runtime treats the inline handler as if the host
  had called `wasamo_signal_connect` with a runtime-generated
  callback id. On click, the runtime emits the signal via the
  standard DD-P6-002 path; the binding-provided trampoline matches
  the callback id, invokes a host-language function it generated for
  the body, and that function calls `wasamo_set_property` for each
  assignment.
- Equivalent to DD-M2-P2-001 Option A's handler shape, retrofitted
  on top of the IR.

- What you gain: Host code can debug handler bodies in the host
  language (Rust `panic!` in a handler is a Rust panic with a Rust
  stack frame). Host-defined functions can be called from handler
  bodies "for free" once the binding lays down a calling convention.
- What you give up: Reintroduces the per-binding-language emitter
  Option B of DD-M2-P2-001 was chosen specifically to avoid — the
  trampoline must transliterate handler-body IR into Rust, C, and
  Zig expression syntax, including `+=` semantics, integer
  promotion, and property-access desugaring, in three different
  emitters. The reactive engine (M2-Phase 5) sees state writes only
  through C-ABI calls, which is correct but slower per click and
  introduces a re-entrancy edge during signal dispatch (write fires
  observers; observers may schedule layout; layout completes; then
  the trampoline returns). Per-binding emitters compound the
  workload axis the project has explicitly chosen to bound.
- **Technical risk: Medium–high.** Layering risk dominates. The
  emitter risk is the same toil-not-unknowns shape Option A of
  DD-M2-P2-001 carried, paid here at the handler-body granularity
  rather than the whole-tree granularity. Re-entrancy of
  `set_property` during a signal dispatch is a known soft spot
  ([DD-P6-003 / DD-P6-004](./phase-6-c-abi.md#dd-p6-003--callback-contract-lifetime-destroy_fn-re-entrancy));
  inline handlers as host callbacks would be the highest-frequency
  re-entrant path in normal use, where the runtime today only has to
  worry about programmer-written observer callbacks.

Option C — Hybrid (runtime evaluates "pure" forms, host handles the rest)
- The runtime evaluates handlers whose body is restricted to property
  reads/writes and arithmetic over scalar types. Handlers that fall
  outside that subset (host-function calls, future imports) are
  routed through the Option B trampoline path.

- What you gain: Best-of-both for sufficiently constrained handlers.
- What you give up: Two evaluation paths to specify, document, test,
  and explain. Whether a handler is "pure enough for runtime
  evaluation" becomes a user-visible classification — and one that
  is not stable across DSL evolution (adding a host-function call
  silently flips a handler's execution location). M2 has zero
  handlers in the "non-pure" category (Counter is the only DSL
  example); no acceptance criterion demands the host-side path
  exist. Premature mechanism for a problem M2 doesn't have.
- **Technical risk: High.** Two pathways means two correctness
  arguments and two test surfaces; the classification rule itself
  is a piece of the language semantics that has to be designed,
  documented, and stably maintained. The risk multiplier is
  significant for a benefit M2 does not require.

**Recommendation:** **Option A.**

Three reasons, parallel to the DD-M2-P2-001 case:

1. **Layering consistency.** DD-M2-P2-001 = B placed the
   `.ui`-evaluation responsibility inside `wasamo-runtime`. Option A
   here keeps handler-body evaluation in the same place; Option B
   carves a hole back into the host languages. Two consecutive
   decisions arguing the same layering go in the same direction.

2. **Binding workload.** Option B requires three handler-body
   transliterators by M2-Phase 6 (Rust / C / Zig), one per future
   binding forever. Option A requires zero per-binding work beyond
   the C ABI surface each binding already needs. This is the same
   compounding-cost argument that decided DD-M2-P2-001 and applies
   identically here.

3. **Reactive integration.** The reactive engine in M2-Phase 5 needs
   to observe state writes and invalidate. Under Option A those
   writes are direct in-process calls into property storage; under
   Option B they are observable only after a C-ABI emit/dispatch
   round trip. The Option A path is the one M2-Phase 5's design will
   target naturally; choosing Option B forces M2-Phase 5 to be
   correct on a path that exists for layering reasons it doesn't
   benefit from.

Option C is rejected on simplicity grounds: it earns no acceptance
criterion in M2 and creates a user-visible classification of
handlers ("does this handler run in-runtime or in-host?") that is
worse than either pure choice.

**Technical-risk re-evaluation:** Risk reinforces the
recommendation.

- Option A's evaluator is a small extension of work already required
  by DD-M2-P2-003 (property-binding evaluation). The marginal risk
  for handlers — assignment and compound-assignment semantics over
  the same scalar set — is incurred once, in one place.
- Option B's emitter risk is paid 3× now and N× in the long run, on
  the handler-body grain rather than whole-tree grain (which makes
  the per-emitter cost smaller per call but the maintenance surface
  larger).
- Option C carries Option A's risk **plus** Option B's risk **plus**
  the classification rule, for no acceptance benefit at M2.

---

### DD-M2-P3-002 — Coexistence with `wasamo_signal_connect`

**Status:** Proposed

**Context:**
Inline handlers in the DSL and host-registered listeners through
`wasamo_signal_connect` (DD-P6-002) are two ways to react to the
same widget signal. A button has `clicked => { root.count += 1 }`
in `counter.ui`; an instrumentation host might also call
`wasamo_signal_connect(button, "clicked", on_click_metric, …)` to
log every click. The runtime must define what happens on click.

The question is conceptual, not pick-an-option-from-three: are
inline handlers the *same path* as host listeners (consume one
shared listener list), or are they a *separate path* that fires
alongside the listener list?

**Options:**

Option A — Single list, inline handler enqueued first
- The runtime treats the inline handler as a synthetic
  `wasamo_signal_connect` registration done at widget construction.
  Click fires every entry in the listener list in registration
  order; the inline handler entry is always first because it is
  registered first.

- What you gain: One mechanism. Host can introspect/disconnect the
  inline handler if `wasamo_signal_connect` returns a token.
  Conceptually clean if you accept "inline handler is just a
  built-in subscriber".
- What you give up: Surfaces the inline handler as a token the host
  can disconnect — a footgun ("why does my counter stop
  incrementing?"). Mixes two distinct artifacts (DSL-author intent
  vs host-author observation) into one orderable list. Forces a
  decision on whether the inline handler appears in the host's
  listener enumeration.
- **Technical risk: Low.** Pure mechanism choice; ordering rules
  are stable. The risk is design-quality, not technical.

Option B — Separate paths; inline runs first, listeners run after (recommended)
- The runtime stores the inline handler IR on the widget directly.
  On signal fire, the runtime first evaluates the inline handler in
  the interpreter (DD-M2-P3-001 = A), then iterates the host
  listener list registered via `wasamo_signal_connect` and dispatches
  each.
- The inline handler is **not** a token returned to the host and is
  **not** disconnectable from the host side. It is part of the DSL
  contract for the component.

- What you gain: Each path expresses what it actually is — inline =
  DSL-author intent, listener list = host-side observation. Hosts
  see a coherent "inline first, then me" ordering: any state change
  the handler causes is visible to the host listener that fires next
  on the same click. Aligns with the natural DSL reading: `clicked
  => { root.count += 1; }` is the component's own response to its
  own button being clicked, not a subscription a stranger should be
  able to revoke.
- What you give up: Two pieces of state on the widget rather than
  one (inline-handler slot + listener list). Slight asymmetry
  between built-in widget signals (always potentially have an inline
  handler) and component-declared signals (in M2 scope, never have
  an inline handler at the declaration site, only at instantiation
  sites — but `counter.ui` does not instantiate sub-components, so
  this gap is invisible at M2).
- **Technical risk: Low.** Two clearly-separated lookups on emit;
  documented ordering ("inline before host"). Each path is small.
  Less risk than Option A because the host cannot accidentally
  disconnect DSL-author code.

Option C — Separate paths; listeners run first, inline after
- Same as B but reversed order: host listeners observe pre-handler
  state, then the handler runs.

- What you gain: Hosts can implement "observe what happened, then
  let DSL react" patterns.
- What you give up: Counterintuitive: the DSL author wrote the
  handler as the component's response, and observation typically
  watches the *result* of a response. Reversing the order forces
  every host listener to remember it sees the pre-state. No M2
  acceptance criterion benefits from this order; Counter's host
  doesn't even use `wasamo_signal_connect`.
- **Technical risk: Low** (same as B); design-quality risk is
  higher because the order is the surprising one.

**Recommendation:** **Option B.**

Inline handlers and host listeners are different artifacts at
different layers; treating them as a single list (Option A) compresses
two roles into one and creates the disconnect-the-handler footgun.
Between B and C, B's order ("DSL response first, then host
observation") matches the natural reading of the DSL and is the
order any host listener would assume by default if not told
otherwise.

The order rule is **documented in `architecture.md` §6 (or the M2
revision thereof) as a runtime contract**, so the M2-Phase 5
reactive engine and any future host-listener author can rely on it.

**Out of scope:** What happens when a future DSL feature lets the
inline handler `return false` or otherwise short-circuit further
emission. M2 has no such mechanism in `counter.ui`. If/when added,
the contract here becomes "inline handler runs; if it requests
short-circuit, host listeners are skipped" — a non-breaking
addition.

**Technical-risk re-evaluation:** All three options are
mechanically straightforward; the risk axis is design-quality
rather than implementability. Option B is the lowest design risk
because it isolates two unrelated concerns and uses the natural
order. Risk reinforces the recommendation.

---

### DD-M2-P3-003 — Handler error / panic policy

**Status:** Proposed

**Context:**
Runtime evaluation of handler bodies (DD-M2-P3-001 = A) can raise
errors. Concrete sources at M2 scope:

1. Type errors that escaped `wasamoc` (should be impossible if
   DD-M2-P2-003 = B holds, but the runtime cannot assume the IR
   was produced by a correct compiler — for hot reload and for
   robustness, the runtime treats malformed IR as a runtime error).
2. Arithmetic overflow on `+=` / `-=` / `*=` (Rust default panics
   in debug, wraps in release).
3. Future: division by zero, array index out of bounds, etc. (not
   reachable from `counter.ui` but architectural).
4. Internal runtime panics inside the evaluator (bug in the
   interpreter itself).

Whatever happens in the handler must not unwind through the C ABI
boundary. Panics across `extern "C"` are undefined behaviour in
Rust and would corrupt every binding's stack discipline.

**Options:**

Option A — Catch-and-log; continue event loop (recommended)
- The interpreter wraps each handler invocation in
  `std::panic::catch_unwind` (or equivalent error-channel for
  expected errors). On error, the runtime logs to a configurable
  sink (default: stderr for M2; pluggable hook for M3+) and returns
  control to the event loop. The signal dispatch continues to
  registered host listeners (DD-M2-P3-002 = B keeps inline and host
  paths separate, so a handler error does not poison the listener
  iteration).
- For arithmetic overflow specifically, the interpreter follows the
  Rust release default (wrapping) for M2 — overflow is not
  classified as an error. Documented in the IR semantics note.

- What you gain: No UB at the C ABI boundary. The application stays
  responsive after a handler bug; the user sees a logged error
  rather than a process crash. Logging output is sufficient for M2
  scale (Hello Counter; one developer reading their own terminal).
- What you give up: Silent recovery can mask bugs in development if
  the logger output is not surfaced. Mitigated by stderr being the
  default sink; the developer sees it during `cargo run`.
- **Technical risk: Low.** `catch_unwind` semantics are
  well-understood; the only subtlety is ensuring no resource
  acquired by the handler (e.g. an interior `RefCell` borrow on the
  property storage) outlives the unwind. Addressable with discipline
  (release borrows before invoking user code, idiom already used in
  observer dispatch).

Option B — Catch + propagate to a host error callback
- Same as A, but instead of (or in addition to) logging, the runtime
  invokes a host-registered error callback (`wasamo_set_error_handler`
  or similar).

- What you gain: Hosts can route errors into their own logging /
  telemetry / crash-report stack.
- What you give up: New ABI surface (the error callback registration)
  not justified by M2 acceptance criteria. Adds a re-entrancy edge
  during error recovery (host callback invoked while the runtime is
  cleaning up from a handler error). Premature for M2.
- **Technical risk: Low–medium.** The mechanism is small but the
  re-entrancy contract during error recovery has to be thought
  through, which is work for no current acceptance benefit.

Option C — Crash the process (no catch)
- Treat handler errors as bugs that should fail loud. Let the panic
  propagate; rely on the host's panic handler to clean up.

- What you give up: UB at the C ABI boundary (unwind across
  `extern "C"` is UB unless every export uses
  `extern "C-unwind"`; that is not currently the case in
  `wasamo-runtime`'s ABI surface). A bug in one handler in one
  widget kills the entire host process. Hostile to embedded use
  cases (a UI library should not bring down the host on a malformed
  click).
- **Technical risk: High.** UB is the high-risk shape. Even if the
  whole ABI surface were converted to `extern "C-unwind"`, every
  binding language would need to handle Rust unwinds, which is an
  open research area in places (Zig especially). Rejected.

**Recommendation:** **Option A.**

A is the minimum mechanism that prevents UB and keeps the runtime
usable as a library. The pluggable error-sink (B) is a non-breaking
extension; a future ADR can add it when a concrete host requirement
appears (M3+ or external user). C is rejected on UB grounds.

**Logging contract for M2:** Errors are written to stderr in a
single line of the form
`wasamo: handler error in <component>.<widget-path>.<signal>: <message>`.
The exact format is implementation detail, not a contract; the
contract is "errors are visible by default and don't crash the
host". Format may be revisited when M3 introduces structured
diagnostics.

**Technical-risk re-evaluation:** Option A is the lowest-risk
choice that meets the constraint (no UB at the ABI boundary).
Option B's risk-vs-benefit is unfavourable at M2 (mechanism cost
without acceptance demand). Option C's risk is disqualifying.

---

### DD-M2-P3-004 — Source location preservation in handler diagnostics

**Status:** Proposed

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

---

## Summary of proposed decisions

| ID | Topic | Recommendation | Risk of recommended |
|---|---|---|---|
| DD-M2-P3-001 | Handler-body execution location | **Option A** — runtime-side interpreter (consistent with DD-M2-P2-001 = B layering) | Medium (handler-body evaluator; largely shared with M2-Phase 5) |
| DD-M2-P3-002 | Coexistence with `wasamo_signal_connect` | **Option B** — separate paths; inline runs first, host listeners after | Low |
| DD-M2-P3-003 | Handler error / panic policy | **Option A** — `catch_unwind` + stderr log; continue event loop; pluggable sink deferred | Low |
| DD-M2-P3-004 | Source-location preservation | **Option B** — IR reserves optional span slot; M2 uses coarse identifiers; M3 tightens | Low |

**Aggregate risk picture.** The only non-trivial risk in the
recommended package is DD-M2-P3-001's handler-body evaluator, and
that work is largely shared with M2-Phase 5 (which evaluates
property-binding expressions on invalidation regardless of handler
location). No option in the recommended package introduces a
mechanism that has no prior art in similar projects (Slint /
SwiftUI / Vue all run UI-state mutations through an in-process
evaluator); the M2-Phase 2 spike already exercised the
IR-walker → internal-builder shape that Option A extends.

**Pre-doc validation spike.** Not required. The M2-Phase 2 spike
([`exp/m2-p2-ir-loader-spike`](https://github.com/matarillo/wasamo/tree/exp/m2-p2-ir-loader-spike),
commit `b7ab4dc`) drove `set_clicked` from IR-walker code, which
is the structural shape DD-M2-P3-001 = A relies on. The remaining
work (small expression evaluator for `+=` over int, error catching
at the handler boundary) is implementation detail rather than
architectural feasibility, and the failure mode is "small
implementation rework", not "two of four DDs collapse" — the
asymmetric-cost argument that gated M2-Phase 2 agreement does not
apply here.

## Out of scope

- **Calling host-defined functions from handler bodies.** No M2
  acceptance criterion requires it; `counter.ui` doesn't use it.
  Decided when (if) imports / FFI in handlers becomes a feature
  request post-M2.
- **Component-declared signals firing inline handlers at the
  declaration site.** `counter.ui` does not declare signals. Phase 4
  / Phase 6 of M2 do not require this. Decided in M3 component
  surface work.
- **Async handler bodies (`async clicked => { … }`).** Not in DSL
  surface; M3+ if at all.
- **Tightening DD-M2-P3-004 to require spans.** Reserved for M3.
