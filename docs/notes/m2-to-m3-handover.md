---
title: M2 → M3 handover — design prerequisites carried forward from Phase 6
status: live
created: 2026-05-08
---

# M2 → M3 handover

Structural decisions that landed inside M2-Phase 6 implementation
steps (rather than as standalone DDs) and that M3 DSL-extension work
(Grid / ScrollView / List per [ROADMAP.md M3](../../ROADMAP.md#m3-dsl-surface)
and the M3 DSL spec public-draft acceptance criterion) must inherit
as design premises, not re-litigate.

These are recorded here because:

- They are not in the Phase 6 ADR's nine Accepted DDs (they were
  absorbed under DD-M2-P6-006 implementation).
- They are not derivable from the post-Phase-6 codebase as
  *intentional* design prerequisites — `git log` shows the change but
  not the M3-facing commitment.
- They are not in the live ADR set for any later phase. (Sections
  1–2 originated as Phase 6 plan deviations; section 3 records the
  M3-facing residuals from DD-M2-P6-010, which is Accepted but whose
  M3 obligations are pre-doc inputs rather than ADR successor work.)

## 1. `wasamo-ir` is the shared IR crate; compiler and runtime both
   depend on it

**Premise.** The IR text grammar (`;wasamo-ir v0`, `docs/dsl_spec.md`
§8) is matched by a single in-memory representation hosted in the
`wasamo-ir` crate (`IrComponent`, `IrNode`, `IrProperty`,
`HandlerExpr`, …). `wasamoc` emits this representation; `wasamo-runtime`
consumes it via `ir_loader.rs`. The dependency direction is
`wasamoc → wasamo-ir ← wasamo-runtime`; no `runtime → wasamoc` edge.

**Origin.** Phase 6 implementation (commit `f8f7d3d`, recorded as
DD-M2-P6-006 plan deviation). Before extraction the IR types lived
inside `wasamoc`, which the runtime would have had to depend on
(wrong direction).

**Why it matters for M3.** Grid / List / ScrollView each adds new
IR node forms (cell positions for Grid, item-template binding for
List, viewport descriptors for ScrollView). The DSL spec public
draft must extend the grammar in `docs/dsl_spec.md` §8 *and* the
type set in `wasamo-ir`. Adding IR forms in only the compiler or
only the runtime is a category error — a Phase 7 / M3 reviewer
should reject it on sight. The M3 spec drafting work should treat
"new IR node form" as a triple of (grammar production, `wasamo-ir`
variant, loader/emitter wiring).

## 2. `HandlerExpr` is unified across handler bodies and binding
   expressions

**Premise.** A single `HandlerExpr` enum (in `wasamo-ir`) represents
both DSL handler bodies (`clicked => { count += 1 }`) and DSL binding
expressions (`text: "Count: \{root.count}"`). Variants —
`Assign`, `CompoundAssign(CompoundOp)`, `PropRead { path }`,
`IntLiteral`, `Block`, `Interpolation`, etc. — are shared. The
text-grammar surface uses `+= -= *= /=`; the in-memory variant set is
`CompoundOp::{Add, Sub, Mul, Div}`. Binding evaluation is the
read-only subset (rejects `Assign` / compound-assign at evaluation
time, per DD-M2-P5-006).

**Origin.** Phase 6 implementation (commit `00246ce`, recorded as
DD-M2-P6-006 plan deviation). Before unification the compiler's
`wasamo_ir::HandlerExpr` and the runtime's
`wasamo_runtime::handler::HandlerExpr` were structurally identical
modulo cosmetic differences (`PropRead.name` vs `.path`;
`CompoundOp::PlusEq` vs `Add`); maintaining two types served no
purpose and would have required a conversion pass through the
loader.

**Why it matters for M3.** New expression forms surfaced by M3 DSL
work — list-item context references (`item.foo`), per-cell
positioning expressions, possibly a `match` / conditional shape if
Grid spans introduce branching — go into the *single*
`HandlerExpr` enum. New variants must be evaluable both as binding
expressions (read-only) and as handler statements (read/write)
unless they are intrinsically one-sided, in which case the
read/write distinction is encoded by which `EvalContext` impl
accepts them, not by splitting the enum.

The M3 DSL spec draft should describe the expression grammar once
and note the two evaluation modes, rather than describing handler
expressions and binding expressions as separate languages.

## 3. `dirty_effects` topological walk — M3 residuals from DD-M2-P6-010

**Premise.** `drain_dirty_effects()` in `wasamo-runtime` orders the
dirty Effect set with a Kahn-style topological walk over
`ReactiveGraph::forward` / `back` plus the Effect write-edge map,
restricted to the dirty set. `forward` / `back` encode read
dependencies; the write-edge map is required to derive writer-Effect
to reader-Effect ordering edges. The walk is implemented as a free
function and covered by pure-logic unit tests on synthetic dependency
graphs (chain, diamond, fan-out vs `MUTATION_CAP`, out-of-ID-order).
There is a single drain code path; no debug/release behavioural
asymmetry.

**Origin.** Phase 7 ADR DD-M2-P6-010 (Accepted 2026-05-09) — Option A.
A5's literal reading required a structural correctness guarantee in
the shipped release binary, ruling out the M2-deferral and verified-
approximation alternatives.

**Why it matters for M3.** Adopting the walk in M2 settled the
*ordering primitive*, but did not by itself settle every property M3
multi-binding will require of it. The following obligations are
inherited as M3 pre-doc material; they are not absorbed into M3
implementation silently.

1. **Cycle detection policy.** A Kahn-style walk is well-defined only
   on a DAG. The M2 counter case has no cycles by construction; the
   M2 unit tests assert acyclic shapes. M3 multi-binding can in
   principle introduce cycles (e.g. two Signals that bind through
   each other's expressions). The M3 pre-doc must decide whether
   cycles are (a) prevented at IR-load time by a structural rule,
   (b) detected at runtime and surfaced as
   `WASAMO_ERR_REACTIVE_DIVERGED` (or a new error code), or (c)
   rejected at `wasamoc` lowering time. Until M3 chooses, the M2
   walk's behaviour on a cyclic input is **undefined-but-bounded**:
   the unit tests cover acyclic inputs; if a cycle reaches the walk
   in production, the runtime is in a state DD-010 did not specify.

2. **Ordering ties.** Multiple Effects with no dependency relationship
   between them have no topologically-required order; the walk
   currently picks one. M3 must decide whether the chosen order is
   observable contract (e.g. by Signal-creation order, or
   ABI-explicit) or remains implementation-defined.

3. **Fan-out interaction with `MUTATION_CAP`.** The walk runs inside
   a drain loop bounded by `MUTATION_CAP = 16`. M3 multi-binding may
   legitimately produce dirty sets large enough to probe this
   interaction; the cap may need to grow, become per-shape, or be
   replaced by a different convergence guarantee. This was already
   named as an open question in DD-M2-P6-001's divergence semantics;
   M3 inherits it alongside the residual above.

The M3 DSL spec drafting work and the M3 multi-binding implementation
step are the natural places these obligations are discharged. They
are not roadmap acceptance criteria — they are pre-doc inputs.

## Re-evaluation triggers

Drop or revise this note if:

- M3 introduces a binding language that diverges enough from the
  handler language that the unified `HandlerExpr` becomes a
  structural mismatch (e.g. a typed expression IR with explicit
  effect tracking — currently not on the M3 acceptance list).
- A future phase splits the IR shared crate into compiler-only and
  runtime-only halves (currently no driver for this; the shared
  crate is small and the dependency direction is correct).
- Section 3's residuals are discharged by the M3 multi-binding
  pre-doc cycle (cycle / ties / fan-out policies decided and
  recorded in the relevant M3 ADR); at that point section 3 can be
  trimmed to a back-pointer to the M3 decision.

Otherwise, M3 pre-doc cycles for Grid / List / ScrollView and the
DSL spec public draft should consume this note as design input
and not re-open the structural questions it records.
