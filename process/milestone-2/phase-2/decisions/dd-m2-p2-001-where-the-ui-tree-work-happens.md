### DD-M2-P2-001 — Where the .ui→tree work happens

**Status:** Accepted

**Context:**
Three real points in the design space, distinguished by *when* the
`.ui` source is parsed/checked and *what* gets shipped to the host
binary:

1. Compile-time, output is host-language source code (codegen).
2. Compile-time, output is a portable intermediate representation
   (IR), consumed by an interpreter inside the runtime.
3. Run-time, no compile step — the runtime parses `.ui` directly.

These are not the only conceivable points (e.g. compile to native
code via LLVM), but they are the only ones that make sense at M2's
scope and at the size of UI definitions the framework targets.

**Options:**

Option A — Host-language codegen
- `wasamoc build counter.ui --target rust` emits `counter.ui.rs`,
  `--target c` emits `counter.ui.c` + `counter.ui.h`, `--target zig`
  emits `counter.ui.zig`. Each emitted file calls the existing C ABI
  to construct the tree, registers signal handlers, and wires
  property bindings into reactive expressions.
- The host build system compiles the emitted file alongside the
  application source.
- Handler bodies (`clicked => { root.count += 1 }`) are translated
  directly into host-language statements that call C ABI setters.

- What you gain: Zero runtime overhead (the tree is built by direct
  function calls). Errors in the generated code surface at host build
  time. Generated code is debuggable in the host language. No new
  runtime component to ship.
- What you give up: One generator per binding language. M2 needs
  three (Rust / C / Zig); post-1.0 community track adds Swift / Go;
  every future binding adds another. Each generator is a non-trivial
  artifact that must handle handler-body lowering into the target
  language's syntax — three different concrete-syntax expression
  emitters. **Hot reload is foreclosed**: re-running the generator
  produces source that must be recompiled and relinked, which is not
  what "hot reload" means in any UI framework. **Phase 3 is
  effectively pre-decided** to host-side execution: the handler body
  is host-language code, so the only place it can run is the host.
- **Technical risk: Low–medium.** Codegen for C-friendly languages is
  well-trodden (Slint, FlatBuffers, protobuf). The risk is toil
  rather than unknowns: per-target syntax emitters for handler bodies
  in 3 languages, plus integrating generated files into 3 host build
  systems (cargo build script / CMake custom command / build.zig).
  Bugs are local; no novel mechanism. Predictable, linearly-scaling
  risk.

Option B — Compile-to-IR + runtime interpreter (recommended)
- `wasamoc build counter.ui` emits `counter.uic` (a portable IR file
  capturing the typed, checked tree shape, property bindings, and
  handler bodies in an IR-expression form).
- The runtime gains an interpreter that loads `counter.uic` at host
  startup and constructs the widget tree via internal builders.
  Hosts call something like `wasamo_load_ui("counter.uic", &out_root)`
  — one new ABI call, neutral in shape vs Phase 4's tree-mutation
  primitives.
- Handler bodies are stored in the IR as small typed expressions; the
  runtime evaluates them, with calls back to the host for any
  external work (Phase 3 question).

- What you gain: One interpreter (in `wasamo-runtime`) serves all
  bindings. Adding a new binding language is "wire up the C ABI",
  which is exactly the cost the C ABI was designed to bound. Hot
  reload becomes natural: swap the IR file and rebuild the tree —
  the runtime is the only thing that needs to know how. Errors
  caught by `wasamoc check` (already present) still surface at host
  build time; only runtime-tied errors (e.g. asset-not-found) move
  to runtime. **Phase 3 stays open**: handler execution can be
  host-side trampoline (signal emission, host runs the body) or
  runtime-side (interpreter evaluates the body), without changing the
  IR.
- What you give up: A new component (`wasamo-runtime` interpreter)
  to design, implement, and maintain. IR design is itself a problem
  (versioning, extensibility — see DD-M2-P2-002). One-time startup
  cost to deserialize and walk the IR; for Hello-Counter scale this
  is microseconds, but at large-app scale it would need measurement.
- **Technical risk: Medium.** The IR walker that calls existing
  internal builders (`widget_create`, `widget_set_property`) is
  mechanical. The novel-to-this-codebase piece is the **reactive
  dependency tracker** the interpreter needs to evaluate property
  bindings — a well-understood pattern (Solid / Vue / Slint signals,
  on the order of a few hundred lines of Rust) but with no prior art
  in this repo. Crucially, **this risk is paid by M2-Phase 5
  regardless of the Phase 2 choice** (acceptance A2 demands a reactive
  engine); choosing Option B does not introduce the risk, it locates
  it inside the runtime where Phase 5 can build directly on it.

Option C — Runtime parses `.ui` directly (no compile step)
- `wasamo-runtime` ships the lexer, parser, and checker.
  Hosts ship their `.ui` files alongside the application binary;
  `wasamo_load_ui("counter.ui", &out_root)` parses and instantiates.
- `wasamoc` becomes optional — only needed for ahead-of-time
  validation (`wasamoc check counter.ui` in CI).

- What you gain: Hot reload is the default mode of operation. One
  fewer build step. No artifact format to design. The simplest
  possible mental model.
- What you give up: Every shipping application carries the parser +
  checker code. Errors in `.ui` syntax surface at application
  startup, not at host build time, unless the host wires up
  `wasamoc check` in CI (becomes mandatory rather than optional).
  Source files are exposed in shipped binaries — minor IP / tamper
  surface. The runtime grows by the size of the parser
  (~the entire current `wasamoc` codebase moves into runtime).
- **Technical risk: Medium.** Inherits Option B's interpreter and
  reactive-tracker risks unchanged. Removes B's serializer/printer
  effort. Adds: runtime binary-size growth needs measurement (the
  current `wasamoc` is ~5 modules; not large but non-trivial), and
  source-location preservation must reach the host's error display.
  Net risk roughly equivalent to B.

**Recommendation:** **Option B.**

Three independent reasons:

1. **Binding workload.** Option A requires three generators by
   M2-Phase 6 and N for N future bindings. Option B requires zero
   per-binding work beyond the C ABI bindings each language already
   needs. The difference compounds across the project's lifetime;
   the predecessors that motivated wasamo (Slint and similar) report
   binding-author cost as a recurring drag, and Option A reproduces
   that pattern.

2. **Phase neutrality.** Option B leaves M2-Phase 3 a real decision.
   Option A pre-decides it (handlers must run host-side because
   that's where the generator wrote them). Pre-deciding a question
   we have explicitly scheduled to discuss is bad sequencing.

3. **Hot reload.** Hot reload is post-1.0, not M2 — but the
   constraint stated in [m2-plan §Out of scope](../../plan.md#out-of-scope-deferred-to-later-milestones)
   is "feasibility depends on M2-Phase 2". Option A forecloses it;
   Option B enables it cleanly; Option C makes it the default. We
   should pick the lowest option on this axis that doesn't lose on
   other axes — that's B, not C.

Option C loses to B on three minor but real axes (binary size, error
timing, IP surface) without winning on anything that matters at M2.
The "hot reload is the default" property is a feature of post-M2
work, not a current need; deferring it to a future ADR is cheaper
than paying its cost today. Option C remains a viable evolution of B
(move the parser into the runtime later, gated by a follow-up ADR)
if hot-reload-by-default becomes desirable in M3+.

**Technical-risk re-evaluation:** Adding the risk axis does not flip
this recommendation.

- Option A is genuinely lower per-component risk (predictable toil,
  no novel mechanism). But the toil is paid 3× (one generator per
  binding language now, one per future binding forever), and the
  three load-bearing arguments above (workload scaling, Phase 3
  neutrality, hot-reload feasibility) are unaffected by risk
  framing.
- Option B's headline risk — the reactive dependency tracker — is
  incurred by **M2-Phase 5 regardless** of what Phase 2 picks
  (acceptance A2 requires a reactive engine, full stop). The
  Phase 2 choice only decides where the tracker lives. Locating it
  in the runtime (Option B) means Phase 5 builds directly on it;
  locating it in generated host code (Option A) means Phase 5
  builds three reactive wirings, one per generator.
- Option C's risk is roughly equivalent to B with a different
  trade allocation; it does not win on the risk axis enough to
  justify its losses elsewhere.

**Pre-doc validation spike (agreement gate).** This ADR's package
is structurally conditioned on Option B (DD-M2-P2-002 explicitly
"Conditional on DD-M2-P2-001 = Option B"; DD-M2-P2-003 framed
around the runtime as evaluator). Agreeing the package without
validating B's architectural feasibility means agreeing on a
foundation that could collapse during M2-Phase 6 implementation,
forcing rewrite of two of the four DDs. The asymmetric cost
(half-day spike now vs weeks of phase-6 rework on failure) makes
the spike a precondition for agreement rather than an option.

Spike scope:
- Add an `experimental_ir_loader` module in `wasamo-runtime`
  (feature-gated).
- Hand-write a minimal `counter.uic` capturing `counter.ui` in a
  proposed textual-IR shape (grammar is throwaway, not part of the
  ADR commitment).
- Write a ~150-200 line Rust loader: minimal lex/parse + tree
  walker that calls existing internal builders with type-erased
  property values.
- Add a minimal driver crate under `wasamo-poc/` that invokes the
  loader and produces the same widget tree as
  `examples/counter-rust/`.
- Build and verify the counter renders.

Pass criteria:
- Existing internal builder API can be driven by a generic IR
  walker without modification.
- Property set works through a tagged-value (int / str / float /
  bool) value type.
- Resulting widget tree renders identically to the hand-written
  M1 example.

Fail criteria:
- Internal API generic-type / compile-time-type / ownership
  assumptions block the IR-walker pattern.

Disposition:
- **Pass** → spike outcome (3-5 line note) appended to this ADR's
  Context, status moves to **Accepted**, spike branch
  `exp/m2-p2-ir-loader-spike` pushed to origin as the Phase 6
  implementation reference point.
- **Fail** → DD-M2-P2-001 Recommendation flips to Option A,
  DD-M2-P2-002 is dropped, DD-M2-P2-003 is rewritten in the
  codegen context. Status remains **Pre-doc** and re-review begins.

The spike does **not** validate the residual risks identified in
the Option B analysis (reactive dependency tracker integration
with DD-P8-002 layout invalidation; full DSL type-system
formalization); those are M2-Phase 5 / M2-Phase 6 implementation
risks and are accepted as such, documented in this ADR but not
gated on pre-doc validation. The spike validates only the
**architectural shape** that the rest of the package depends on.

---
