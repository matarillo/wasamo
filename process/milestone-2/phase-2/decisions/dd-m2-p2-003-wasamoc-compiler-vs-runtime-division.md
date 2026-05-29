### DD-M2-P2-003 — wasamoc 責務境界 (compiler vs runtime division)

**Status:** Accepted

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
