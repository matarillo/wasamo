# M2-Phase 2 — wasamoc output format: Architecture Decisions

**Phase:** M2-Phase 2 (wasamoc output format decision)
**Date:** 2026-05-03
**Status:** Pre-doc — on hold pending feasibility spike (see DD-M2-P2-001 Recommendation)

## Context

[Phase 6 ADR](./phase-6-c-abi.md) explicitly deferred two questions to
M2 to keep the stable C ABI core neutral:

> **(b)** `wasamoc`'s M2 output format — host-language codegen vs IR +
> runtime interpretation.

This ADR resolves question (b). M2-Phase 3 (handler execution location,
Phase 6's deferred (a)) is a separate ADR; the relationship between the
two is recorded in DD-M2-P2-004 below.

### What is "the output format"?

M1 `wasamoc` is parser-only ([wasamoc/src/main.rs](../../wasamoc/src/main.rs):
`check` subcommand only). It builds an AST
([wasamoc/src/ast.rs](../../wasamoc/src/ast.rs)) and runs static checks
([wasamoc/src/check.rs](../../wasamoc/src/check.rs)) but produces no
artifact a host can consume. M1 hosts therefore reproduce
`counter.ui`'s tree by hand against the experimental C ABI
([examples/counter-rust/](../../examples/counter-rust/),
[examples/counter-c/](../../examples/counter-c/),
[examples/counter-zig/](../../examples/counter-zig/)).

M2 acceptance criterion **A1** requires `counter.ui` itself to drive
the running counter in all three host languages
([m2-plan.md A1](../plans/m2-plan.md#acceptance-criteria)). Something
must turn the `.ui` source into the runtime calls that build the tree
and wire reactive bindings. **Where that translation happens, and what
intermediate artifact (if any) it produces, is the question this ADR
answers.**

### Constraints carried in from prior decisions

- **Acceptance A1** (`counter.ui` drives 3 host languages) is the
  primary load-bearing constraint. Whatever shape we pick must
  reach M2-Phase 6 in a form usable from C, Rust, and Zig.
- **Acceptance A4** (tree-mutation ABI primitives at the stable core)
  is decided by M2-Phase 4. The output format must be expressible in
  terms of *some* set of runtime calls; whether those calls live in
  the stable core or stay internal is a Phase 4 question, not this
  one.
- **Phase 6 stable-core neutrality.** The five-area minimum from
  [DD-P6-001](./phase-6-c-abi.md#dd-p6-001--stable-core-scope-at-function-granularity)
  was sized to survive either resolution of (b). This ADR must
  therefore not require the stable core to grow new shapes; growth
  is allowed only in M2-Phase 4 and only as a separate decision.
- **Hot reload (post-1.0 deferral).** [m2-plan §Out of scope](../plans/m2-plan.md#out-of-scope-deferred-to-later-milestones)
  records hot reload as post-1.0 with feasibility "depending on
  M2-Phase 2's wasamoc output format decision". The decision below
  must not foreclose hot reload, though it is not required to enable
  it in M2 itself.
- **Binding workload scaling.** Official bindings at 1.0 are
  C / Rust / Zig ([VISION §11](../../VISION.md)); Swift / Go are
  post-1.0 community track. The output format determines whether
  adding a new binding language is "wire up the C ABI"
  (mostly mechanical) or "wire up the C ABI **and** write a
  language-specific code generator" (a new artifact requiring its
  own tests and maintenance).

---

### DD-M2-P2-001 — Where the .ui→tree work happens

**Status:** Pre-doc

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
   constraint stated in [m2-plan §Out of scope](../plans/m2-plan.md#out-of-scope-deferred-to-later-milestones)
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
  Context, status moves to **Agreed**, spike branch
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

### DD-M2-P2-002 — IR artifact form

**Status:** Pre-doc

**Context:**
Conditional on DD-M2-P2-001 = Option B. The IR has to be serialized
to a file that the runtime loads at host startup. Three candidate
serializations:

**Options:**

Option A — Hand-designed binary IR
- A custom binary format with a magic header, version field,
  fixed-shape sections (string table, type table, widget tree,
  expression bytecode).
- The runtime ships a deserializer; `wasamoc` ships a serializer.

- What you gain: Smallest on disk. Fastest to load. Forces version
  discipline by construction (the version field in the header is
  the contract). No exposure of compiler-internal shapes.
- What you give up: Most design work up front. Not human-inspectable
  — debugging a "weird tree" in production requires a separate
  dump tool. Test fixtures are binary blobs, hard to diff. Premature
  for M2-scale UIs (Hello Counter compiles to a handful of nodes;
  the size win is meaningless).
- **Technical risk: Medium.** Binary format conventions (magic /
  version field / string table / section layout) are textbook, but
  first-version designs often turn out to need a v2 break once real
  use surfaces edge cases. Versioning discipline must be enforced
  from day one. For Hello-Counter scale the size/load wins do not
  justify the risk-for-design-investment trade.

Option B — Textual IR (recommended)
- A plain-text canonical form, distinct from the surface DSL —
  e.g. an s-expression-like form that maps 1:1 to the typed,
  checked tree:

  ```
  (component Counter (base Window)
    (property count int 0)
    (children
      (widget VStack (props (spacing 12px) (padding 24px))
        (children
          (widget Text (props (text (interp "Count: " (ref root.count)))))
          (widget Button (props (text "Increment") (style accent))
            (on clicked (block
              (assign-add (ref root.count) (int 1)))))))))
  ```

- The runtime ships a small parser; `wasamoc` ships a printer.

- What you gain: Diff-friendly (PR review, golden-file tests).
  Hand-inspectable when debugging a runtime tree mismatch. Test
  fixtures readable. No version-field ceremony — additive changes
  produce trivially-mergeable diffs; breaking changes show up
  obviously. Grammar of the IR is its own normative artifact in a
  future ABI spec.
- What you give up: Larger on disk than binary (irrelevant at M2
  scale). Slower to parse than binary (irrelevant at M2 scale). Risk
  that someone hand-edits a `.uic` file rather than the source —
  mitigated by a `wasamoc-generated; do not edit` header line and a
  CI check that `.uic` matches its `.ui` source.
- **Technical risk: Low.** Small s-expression-style grammar, lexer
  can borrow patterns from `wasamoc/src/lexer.rs`, parser is a few
  hundred lines. Round-trip property
  (`parse(print(x)) == x`) is easy to test. Local grammar choices
  (interpolated string shape, type literal syntax) are revisable
  cheaply because nothing serialized in this format ships frozen
  before M4.

Option C — Serialized AST (e.g. via serde + bincode/postcard)
- Persist the existing `wasamoc/src/ast.rs` types using a Rust
  serialization framework. Runtime deserializes back into the same
  types.

- What you gain: Zero IR-design work — reuse what exists. Format
  evolves automatically as the AST evolves.
- What you give up: The AST is an internal compiler representation.
  Persisting it as an artifact freezes its shape into the
  wasamoc↔runtime contract; changes to AST shape become breaking
  changes to the artifact format. The compiler can no longer
  refactor freely. This is the wrong layering — the IR should be
  designed once, deliberately, with its own evolution policy. Also
  ties the artifact format to Rust serde implementation choices
  (bincode versions, etc.), which is not a contract we want to
  carry.
- **Technical risk: Very low (operational); high (long-term coupling).**
  `serde` derive on existing types is near-free to make work; almost
  zero risk of "it doesn't work". The risk is entirely on the
  long-term axis: every refactor of `ast.rs` becomes a breaking
  change to the artifact format, and serde-format choices
  (bincode major version, postcard variant flags) become part of
  the de-facto contract.

**Recommendation:** **Option B (textual IR).**

For Hello-Counter-scale work, size and parse cost are negligible.
The wins from text — debuggability, test-fixture quality, normative
grammar as documentation — are real. Option C loses on layering
(AST is internal; IR is contract); the temptation to take it
because it's "free" is the kind of shortcut that creates a bad
contract that's hard to undo.

Binary IR (Option A) remains a non-breaking later optimization: if
M3+ adds a `--binary` flag to `wasamoc` and the runtime gains a
binary loader, hosts can opt in. The textual IR's semantics are the
contract; serialization swap is a code change, not a redesign.

The concrete grammar of the textual IR is **out of scope for this
ADR**. It is part of M2-Phase 6 implementation and will be drafted
there; this ADR commits only to "textual, distinct from surface
DSL, normative grammar to be defined".

**Technical-risk re-evaluation:** Risk and layering align here.
Option B is the lowest-risk choice that builds the right contract.
Option C is even lower operational risk but pays it back in
compounding coupling cost (AST refactor = artifact-format break).
Option A carries genuine design risk (first-version binary format
likely needs revision) for a benefit M2 does not need. The risk
axis reinforces, rather than complicates, the recommendation.

---

### DD-M2-P2-003 — wasamoc 責務境界 (compiler vs runtime division)

**Status:** Pre-doc

**Context:**
With DD-M2-P2-001 = B, the work of going from `.ui` source to a
running tree is split between `wasamoc` (compile-time) and the
runtime interpreter (host-runtime). The split is not free — every
piece of work pushed into `wasamoc` keeps the runtime smaller and
errors earlier; every piece kept in the runtime keeps `wasamoc`
simpler and the IR less constrained.

The question is: of the activities listed below, which does
`wasamoc` perform before emitting IR?

Activities:
1. Lex + parse to AST. (Already in `wasamoc`.)
2. Static check (warnings). (Already in `wasamoc`.)
3. Type-check property assignments against widget property
   declarations.
4. Type-check property bindings (`text: "Count: \{root.count}"` —
   verify `root.count` exists and is int-coercible to string).
5. Type-check handler bodies (`{ root.count += 1 }` — verify
   `root.count` is int and `+=` is defined on int).
6. Lower property bindings into a typed expression form (an IR
   subtree the interpreter can evaluate, with explicit dependency
   set).
7. Lower handler bodies into typed IR expressions.
8. Component instantiation flattening (resolve `inherits Window`,
   inline component definitions into a single tree). (Out of scope
   for M2 — components are not user-defined yet; only `Counter`
   exists.)
9. Optimization passes (constant folding, dead-binding elimination).

**Options:**

Option A — Minimal: parse + check, emit AST-shaped IR
Activities 1–2 only. The IR carries untyped expressions; the
interpreter does type resolution at load time.

- What you gain: Smallest `wasamoc` change from M1. IR shape stays
  close to AST.
- What you give up: Type errors surface at host startup, not at
  host build time — defeats one of the main wins of having a
  compile step. Interpreter grows a typer (duplicating logic that
  belongs in `wasamoc`). Wrong layering.
- **Technical risk: Low (wasamoc) / High (runtime).** The risk is
  *moved*, not removed: the runtime must grow a typer, which is
  harder to test (no host-build-time integration), surfaces failures
  at app startup, and lives in the wrong place architecturally. Net
  risk total exceeds Option B because the typer is at least as much
  work and is now in a worse spot to maintain.

Option B — Standard: full type-check and IR lowering (recommended)
Activities 1–7. The IR carries typed, checked expressions; the
interpreter is a pure evaluator over a known-good IR.

- What you gain: All static errors caught at host build time. IR
  is a clean evaluable form — the interpreter does no inference.
  Future passes (optimization, hot-reload diffing) attach to a
  stable typed IR. Test fixtures (textual IR) are typed and
  meaningful.
- What you give up: Most of the M2-Phase 6 implementation surface
  lands in `wasamoc` rather than the runtime. (This is correct
  layering, not a downside; calling it out for visibility.)
- **Technical risk: Medium.** This option forces the **DSL type
  system to be formalized for the first time**. M1's `TypeName`
  ([wasamoc/src/ast.rs](../../wasamoc/src/ast.rs)) covers only
  Int / Str / Float / Bool, and no rules currently exist for:
  interpolated strings (`"Count: \{root.count}"` — int→string
  coercion), assignment-operator semantics (`+=` `-=` `*=` `/=`
  on int / float), `Length(px)` arithmetic and unit propagation,
  or property-binding type inference. These will be settled inside
  M2-Phase 6 implementation; edge cases are likely to spawn one or
  two follow-up ADRs and a `docs/notes/wasamoc-types.md` live note.
  Each rule is locally simple; the risk is **volume of small
  decisions**, not depth of any single one. The risk is also
  largely shared with Option A of DD-M2-P2-001 (codegen also needs
  typed expressions to emit host-language code from), so this is
  not a B-specific cost.

Option C — Aggressive: standard + optimization (1–7 + 9)
Standard plus constant folding, dead-binding elimination, perhaps
component inlining.

- What you gain: Smaller IR; fewer interpreter cycles at runtime.
- What you give up: Premature optimization at M2 scale. Each
  optimization pass is its own surface to test. No measurement says
  the runtime cost matters at Hello-Counter scope. Optimizations
  are non-breaking additions; can land in M3+ when there's a real
  bottleneck.
- **Technical risk: Medium–high.** All of B, plus per-pass
  correctness arguments. First-attempt constant folders typically
  miss corner cases (operator overloading, side-effecting
  expressions, type coercions); debugging *incorrect* optimization
  output is harder than debugging missing optimization. Carries the
  highest risk-for-no-acceptance-criterion-benefit ratio of the
  three options.

**Recommendation:** **Option B.**

This is the correct layering: `wasamoc` is the compiler, the
runtime is the evaluator. Option A inverts this and pushes
compiler work into the runtime; Option C does compiler work that
no current acceptance criterion demands. M2-Phase 6 implementation
will fill out the activities 3–7 surface; this ADR commits only to
the responsibility boundary, not to the concrete typing rules
(those are Phase 6 implementation detail).

**Technical-risk re-evaluation:** Risk reinforces this
recommendation rather than challenging it.

- Option A is **not** actually low-risk: the typer must exist
  somewhere, and "somewhere" under A is the runtime, where it is
  harder to test and surfaces errors at app startup. A relocates
  risk to a worse location.
- Option B's medium risk (volume of small DSL-typing decisions) is
  unavoidable — *any* path that enables M2 acceptance A1 needs
  these rules settled. The only question is where the typer
  lives; B puts it in the right place.
- Option C adds risk for benefit M2 does not require.

Activity 8 (component instantiation flattening) is explicitly out
of scope: `counter.ui` has only one component declaration and no
nested user-defined components. When user-defined components arrive
(M3 DSL surface or later), a follow-up ADR decides whether
flattening happens in `wasamoc` or in the interpreter.

---

### DD-M2-P2-004 — Sequencing relative to M2-Phase 3

**Status:** Pre-doc

**Context:**
[m2-plan phase dependencies](../plans/m2-plan.md#phase-dependencies)
says M2-Phase 2 and M2-Phase 3 are parallelizable decision phases,
both gating M2-Phase 4. In practice the two interact:

- DD-M2-P2-001 = Option A (codegen) **forecloses** Phase 3 to
  host-side execution: the handler body lives in host-language
  source, so the host is the only place that can execute it.
- DD-M2-P2-001 = Option B (IR) leaves Phase 3 open: the IR carries
  handler bodies as typed expressions; either the runtime
  interpreter evaluates them or the runtime emits a synthetic
  signal that a host-side trampoline (also generated, or written
  by the binding) handles.
- DD-M2-P2-001 = Option C (runtime parse) is similar to B w.r.t.
  Phase 3.

The reverse direction (Phase 3 outcome constraining Phase 2) is
weaker. A Phase 3 strong preference for runtime-side execution
would argue against Phase 2's Option A; a Phase 3 preference for
host-side leaves all three Phase 2 options viable.

**Options:**

Option A — Sequential: this ADR (Phase 2) lands first; Phase 3 ADR
follows once Phase 2 is Agreed (recommended)

- What you gain: Phase 2's outcome reduces Phase 3's option space
  before Phase 3 review begins. Owner reviews one ADR at a time.
  Lower cognitive load; faster total review time. Matches the
  user's stated preference.
- What you give up: A Phase 3 surprise could in principle force
  Phase 2 reopen. Mitigated by this ADR explicitly enumerating the
  Phase 3 implication of each Phase 2 option (table below) so the
  Phase 2 decision is not made blind.
- **Technical risk: None** (process choice).

Option B — Parallel: both ADRs filed and reviewed together
- What you gain: A coherent joint shape can be reviewed in one
  pass; no risk of "Phase 2 agreed then Phase 3 reopens it".
- What you give up: Two ADRs in flight at once; doubled review
  surface; risk that Phase 3 disagreements re-litigate Phase 2
  mid-discussion.
- **Technical risk: None** (process choice).

Option C — Joint: one ADR covers both Phase 2 and Phase 3
- What you gain: No artificial split where the questions interact.
- What you give up: Conflates two phases the milestone plan
  separated for a reason. Larger ADR. Future readers lose the
  one-decision-per-ADR property.
- **Technical risk: None** (process choice).

**Recommendation:** **Option A (sequential).**

The interaction is one-directional in practice: Phase 2 → Phase 3.
Sequential capitalizes on this; parallel does not. Joint is
overkill given the questions are conceptually separable.

To insulate the sequential path against a Phase 3 surprise, the
following table records each Phase 2 option's downstream Phase 3
implication. If a future Phase 3 outcome is incompatible with the
Phase 2 option agreed here, this ADR is reopened (per
[docs/decisions/README.md supersede policy](./README.md)) rather
than Phase 3 silently working around it.

| DD-M2-P2-001 outcome | DD-M2-P3 (handler exec) implication |
|---|---|
| Option A (codegen) | Forecloses to host-side execution. Phase 3 becomes a sequencing/contract refinement, not a real fork. |
| Option B (IR + interpreter) | Both host-side trampoline and runtime-side interpreter remain viable. Phase 3 is a real decision. |
| Option C (runtime parse) | Same as B — both viable. |

---

## Summary of recommended decisions

| ID | Topic | Recommendation | Risk of recommended |
|---|---|---|---|
| DD-M2-P2-001 | Where the .ui→tree work happens | **Option B** — compile to IR, runtime interpreter | Medium (reactive tracker; shared with M2-Phase 5) |
| DD-M2-P2-002 | IR artifact form (conditional on B) | **Option B** — textual IR, normative grammar to be drafted in M2-Phase 6 | Low |
| DD-M2-P2-003 | wasamoc 責務境界 | **Option B** — full type-check and IR lowering (activities 1–7); component flattening and optimization deferred | Medium (DSL type-system formalization; shared with all DD-001 options) |
| DD-M2-P2-004 | Sequencing vs Phase 3 | **Option A** — sequential (this ADR first; Phase 3 follows) | None (process) |

**Aggregate risk picture.** The two non-trivial risks the
recommended package carries — a reactive dependency tracker and a
formalized DSL type system — are **both unavoidable at M2 scope
regardless of which Phase 2 path is chosen** (the first by A2
acceptance, the second by any path that produces typed code).
Choosing the recommendation does not introduce new risk; it
locates the risk where it is most addressable. No option in the
recommended package is "we don't know if it works" in the strong
sense; all are "well-understood pattern, no prior art in this
repo".

## Agreement gate

This ADR is currently **on hold** pending the feasibility spike
described in DD-M2-P2-001's Recommendation. Agreement (status
move to **Agreed**) does not happen until the spike runs and
passes. On failure, the recommendation flips and the ADR is
rewritten before re-review.

The M2-Phase 2 task list in
[m2-plan.md Progress](../plans/m2-plan.md#progress) is **not**
written until after agreement, per the order
spike → ADR agree → m2-plan task list adopted as the M2 default.

Once agreed, this ADR moves to **Agreed** and the M2-Phase 2
checkbox in [m2-plan.md Progress](../plans/m2-plan.md#progress) is
ticked with the agreement commit. Pre-doc → agreement → impl
post-doc lifecycle. M2-Phase 3 pre-doc is the next phase to enter.

A live note (`docs/notes/wasamoc-ir.md`) capturing open questions
about the textual IR grammar and `wasamoc` responsibility-boundary
edge cases is created in M2-Phase 6 implementation, not here — this
ADR has no implementation work, so there is nothing to feed it yet.
