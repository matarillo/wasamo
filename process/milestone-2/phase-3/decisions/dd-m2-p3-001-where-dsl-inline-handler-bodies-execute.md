### DD-M2-P3-001 — Where DSL inline handler bodies execute

**Status:** Accepted

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

**Forward-compat exposure:** Options differ on this axis. The "Out of
scope" items at the bottom of this ADR — host-defined function calls
from handler bodies, component-declared signals firing inline
handlers at instantiation sites, async handler bodies — are the
foreseeable directions DSL evolution can take post-M2.

- Option A absorbs each direction inside `wasamo-runtime`: the
  interpreter grows, or a separate host-call path is added behind the
  same C ABI surface. Per-binding work scales with C ABI growth, not
  with DSL growth.
- Option B requires every binding-language emitter (Rust + C + Zig at
  M2-Phase 6, more post-M2) to transliterate each new handler-body
  syntax form. DSL growth multiplies binding workload directly. Async
  handler bodies in particular are not cleanly unifiable across the
  three host languages by a syntactic emitter.
- Option C inherits Option B's emitter coupling for the "non-pure"
  branch and adds an additional moving piece: the "pure subset"
  classification rule, which is renegotiated each time the DSL
  surface grows.

This axis reinforces the Option A recommendation independently of the
implementation-risk argument: implementation cost is paid once;
future DSL evolution does not re-open this DD.

---
