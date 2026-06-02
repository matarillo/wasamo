# M2-Phase 2 — wasamoc output format: Architecture Decisions

**Phase:** M2-Phase 2 (wasamoc output format decision)
**Date:** 2026-05-03
**Status:** Accepted (2026-05-04; spike passed — see DD-M2-P2-001 spike note)

## Context

[Phase 6 ADR](../../../milestone-1/phase-6/decisions/preamble.md) explicitly deferred two questions to
M2 to keep the stable C ABI core neutral:

> **(b)** `wasamoc`'s M2 output format — host-language codegen vs IR +
> runtime interpretation.

This ADR resolves question (b). M2-Phase 3 (handler execution location,
Phase 6's deferred (a)) is a separate ADR; the relationship between the
two is recorded in DD-M2-P2-004 below.

### What is "the output format"?

M1 `wasamoc` is parser-only ([wasamoc/src/main.rs](../../../../wasamoc/src/main.rs):
`check` subcommand only). It builds an AST
([wasamoc/src/ast.rs](../../../../wasamoc/src/ast.rs)) and runs static checks
([wasamoc/src/check.rs](../../../../wasamoc/src/check.rs)) but produces no
artifact a host can consume. M1 hosts therefore reproduce
`counter.ui`'s tree by hand against the experimental C ABI
([examples/counter-rust/](../../../../examples/counter-rust/),
[examples/counter-c/](../../../../examples/counter-c/),
[examples/counter-zig/](../../../../examples/counter-zig/)).

M2 acceptance criterion **A1** requires `counter.ui` itself to drive
the running counter in all three host languages
([m2-plan.md A1](../../plan.md#acceptance-criteria)). Something
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
  [DD-P6-001](../../../milestone-1/phase-6/decisions/preamble.md#dd-p6-001--stable-core-scope-at-function-granularity)
  was sized to survive either resolution of (b). This ADR must
  therefore not require the stable core to grow new shapes; growth
  is allowed only in M2-Phase 4 and only as a separate decision.
- **Hot reload (post-1.0 deferral).** [m2-plan §Out of scope](../../plan.md#out-of-scope-deferred-to-later-milestones)
  records hot reload as post-1.0 with feasibility "depending on
  M2-Phase 2's wasamoc output format decision". The decision below
  must not foreclose hot reload, though it is not required to enable
  it in M2 itself.
- **Binding workload scaling.** Official bindings at 1.0 are
  C / Rust / Zig ([VISION §11](../../../../VISION.md)); Swift / Go are
  post-1.0 community track. The output format determines whether
  adding a new binding language is "wire up the C ABI"
  (mostly mechanical) or "wire up the C ABI **and** write a
  language-specific code generator" (a new artifact requiring its
  own tests and maintenance).

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

## Spike result

**Pass** — 2026-05-04, branch
[`exp/m2-p2-ir-loader-spike`](https://github.com/matarillo/wasamo/tree/exp/m2-p2-ir-loader-spike),
commit `b7ab4dc`.

Spike scope: `experimental_ir_loader` module (feature-gated
`experimental-ir`) added to `wasamo-runtime`; hand-written
`experiments/ir-spike/counter.uic` in throwaway s-expression IR;
~200-line loader (tokenizer + parser + tree walker); driver crate
`experiments/ir-spike/` renders the counter window.

Pass criteria confirmed:
1. **Internal builder API unchanged** — `WidgetNode::vstack`,
   `text`, `button`, `append_child`, `set_clicked` driven by IR
   walker without modification to `widget.rs` or any other runtime
   file.
2. **Tagged-value property set** — `PropertyValue::String` used
   in the click handler via `set_property(PROP_TEXT_CONTENT, &val)`;
   existing two-variant `PropertyValue` enum sufficient (no new
   variants needed).
3. **GUI verified locally** — counter window rendered; `Count: 0`
   → `Count: N` on click; hover/press animation intact.

This ADR is now **Accepted**. M2-Phase 2 task list is written in
[m2-plan.md Progress](../../plan.md#progress) and M2-Phase 3
pre-doc is the next phase to enter.
