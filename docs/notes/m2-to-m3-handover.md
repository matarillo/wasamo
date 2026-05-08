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
- They are not in the live ADR set for any later phase (DD-M2-P6-010
  / 011 / 012 are scoped to Phase 7 and orthogonal to these
  structural choices).

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

## Re-evaluation triggers

Drop or revise this note if:

- M3 introduces a binding language that diverges enough from the
  handler language that the unified `HandlerExpr` becomes a
  structural mismatch (e.g. a typed expression IR with explicit
  effect tracking — currently not on the M3 acceptance list).
- A future phase splits the IR shared crate into compiler-only and
  runtime-only halves (currently no driver for this; the shared
  crate is small and the dependency direction is correct).

Otherwise, M3 pre-doc cycles for Grid / List / ScrollView and the
DSL spec public draft should consume this note as design input
and not re-open the structural questions it records.
