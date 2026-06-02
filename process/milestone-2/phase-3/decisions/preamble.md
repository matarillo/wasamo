# M2-Phase 3 — Handler execution location: Architecture Decisions

**Phase:** M2-Phase 3 (DSL inline handler execution location)
**Date:** 2026-05-04
**Status:** Accepted (2026-05-04)

## Context

[Phase 6 ADR](../../../milestone-1/phase-6/decisions/preamble.md) explicitly deferred two questions to
M2 to keep the stable C ABI core neutral:

> **(a)** Where DSL inline handler bodies (`clicked => { … }`) execute
> — host-side trampoline vs runtime-side interpreter.

This ADR resolves question (a). [M2-Phase 2 ADR](../../phase-2/decisions/preamble.md)
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
  `wasamo_button_set_clicked` ([phase-6-c-abi.md DD-P6-002](../../../milestone-1/phase-6/decisions/preamble.md#dd-p6-002--signal-model))
  is a per-widget typed setter, not an inline-handler mechanism. It
  is preserved as-is for M1 hosts and is not in scope here.

---

## Summary of proposed decisions

| ID | Topic | Recommendation | Impl risk | Forward-compat exposure |
|---|---|---|---|---|
| DD-M2-P3-001 | Handler-body execution location | **Option A** — runtime-side interpreter (consistent with DD-M2-P2-001 = B layering) | Medium (handler-body evaluator; largely shared with M2-Phase 5) | Low |
| DD-M2-P3-002 | Coexistence with `wasamo_signal_connect` | **Option B** — separate paths; inline runs first, host listeners after | Low | Low |
| DD-M2-P3-003 | Handler error / panic policy | **Option A** — `catch_unwind` + stderr log; continue event loop; pluggable sink deferred | Low | Low |
| DD-M2-P3-004 | Source-location preservation | **Option B** — IR reserves optional span slot; M2 uses coarse identifiers; M3 tightens | Low | Medium |

The Impl-risk column reads the same as prior ADRs: feasibility within
this phase. The Forward-compat exposure column is new from this ADR
onwards (see [decisions/README.md](../../../README.md)) and rates how much
the recommended option is exposed to revision when post-M2 DSL or
C ABI extensions land. A per-DD `**Forward-compat exposure**`
paragraph is written only where options differ on this axis;
DD-M2-P3-002 and DD-M2-P3-003 have no such paragraph because their
options are equally additive-compatible with foreseeable extensions.

**Aggregate impl-risk picture.** The only non-trivial impl risk in
the recommended package is DD-M2-P3-001's handler-body evaluator,
and that work is largely shared with M2-Phase 5 (which evaluates
property-binding expressions on invalidation regardless of handler
location). No option in the recommended package introduces a
mechanism that has no prior art in similar projects (Slint /
SwiftUI / Vue all run UI-state mutations through an in-process
evaluator); the M2-Phase 2 spike already exercised the
IR-walker → internal-builder shape that Option A extends.

**Aggregate forward-compat exposure.** The dominant exposure in the
recommended package is DD-M2-P3-004's M3-deferral, mitigated by
reserving the IR span slot as optional rather than committing to its
shape now. DD-M2-P3-001's recommendation (Option A) is the
lowest-exposure option among its three; the alternative (Option B)
would have ratcheted forward-compat cost with every post-M2 DSL
extension by requiring per-binding-language emitter updates.

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
