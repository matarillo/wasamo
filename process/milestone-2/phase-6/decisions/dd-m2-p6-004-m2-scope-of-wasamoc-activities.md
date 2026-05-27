### DD-M2-P6-004 — M2 scope of `wasamoc` activities

**Status:** Accepted

**Context:**
DD-M2-P2-003 enumerates seven candidate `wasamoc` activities:
parse → check → type inference → property-binding lowering →
handler-body lowering → IR emit → file write-out. M1 `wasamoc`
implements only the first two. M2 acceptance A1 requires whatever
subset is needed to drive `counter.ui`. This DD picks the subset.

A coupled question lives inside the Phase 2 spec deferral: whether
`.ui` carries `state` declarations (Signal ownership in `.ui`) or
leaves Signal ownership on the host. That question directly
determines whether the host needs an element-identity API
(DD-M2-P6-005's sub-issue), so it is decided here, not in
DD-M2-P6-005.

**Options:**

Option A — Full activity set (1–7) including general type inference
- Implement all seven activities; type inference is general
  (not restricted to counter's two types).
- What you gain: M3 binding features gain full type checking
  out of the gate; no follow-up DD on activity scope.
- What you give up: type inference for an unfinalised DSL
  surface (M3 grammar is not done); inference rules locked in
  before there is a spec to align them against; Phase 6 scope
  expands well beyond A1.
- **Technical risk: High** for M2 — designing inference for a
  language whose surface still moves.

Option B — Restricted scope: 1, 2, 4, 5, 6, 7 + minimal type inference (recommended)
- Activities: parse, check, property-binding lowering,
  handler-body lowering, IR emit, file write-out. Type
  inference is restricted to fixed `i32` and string for M2;
  errors-out on anything else.
- `.ui` carries `state` declarations (Signal ownership in
  `.ui`). The IR includes Signal nodes; the runtime
  instantiates Signals from the IR.
- What you gain: covers A1 (counter has only `i32` count and
  string label content); the lowering paths exist as written;
  M3 type-inference design is unconstrained by an M2 inference
  rule set.
- What you give up: handlers/bindings that use other types
  fail at `wasamoc` time. Acceptable for M2; M3 expands.
- **Technical risk: Low–medium** (lowering design for two
  shapes; small).

Option C — Minimum viable: 1, 2, 5, 6, 7 (skip property-binding lowering as a distinct pass; do it during emit)
- Property-binding lowering is folded into IR emit; no
  intermediate lowered form.
- What you gain: smaller `wasamoc` internal pipeline.
- What you give up: handler-body lowering (5) and binding
  lowering (4) share substantial machinery (HandlerExpr
  construction); folding 4 into emit duplicates that machinery
  in the emit step. Saves no work in practice; complicates
  diagnostics.
- **Technical risk: Low**, but design-quality regression.

**Coupled consequence — Signal ownership.**

- Option A and B both place Signal ownership in `.ui` (`.ui`
  declares `state`; host references state by binding name).
  This means the host does **not** need an
  `wasamo_find_element_by_id`-style identity API: the host's
  interaction surface is the named Signal, not the widget tree.
  DD-M2-P6-005 is freed from element-identity scope.
- "Signal ownership stays host-side" (an alternative not
  enumerated above) would require an element-identity API for
  the host to attach Signals to widgets, expanding
  DD-M2-P6-005's surface. Rejected here as it leaks DSL
  responsibility (state declaration) into host-language code,
  defeating A2's "no host-side property-set plumbing"
  acceptance.

**Coupled consequence — name resolution.**

Signal ownership in `.ui` makes name resolution a `wasamoc`
responsibility, not a runtime one. The rules are fixed here so
that DD-M2-P6-007 (`SignalRegistry`) and DD-M2-P6-009 (loader
validation) inherit a defined contract:

- **Scope:** the `.ui` document is a single flat namespace for
  M2. Counter has one `state count: i32` declaration and a
  small set of references; flat scope is trivially sufficient.
- **Resolution time:** compile-time. `wasamoc` rejects undefined
  references and duplicate `state` names at parse/check time.
  The IR carries already-resolved names; "binding references
  state X" appears in IR as a resolved reference to declared
  state X, not as a pending lookup.
- **Shadowing:** prohibited in M2. Two `state` declarations
  with the same name are a `wasamoc` error. M3 component
  scoping (when introduced) revisits this; M2's prohibition is
  the conservative starting point that does not foreclose any
  M3 scoping shape.
- **Runtime side:** the loader (DD-M2-P6-006) reads
  already-resolved names from the IR and indexes
  `SignalRegistry` (DD-M2-P6-007) by them. Reference-resolution
  validation at load (DD-M2-P6-009 = C) verifies every IR-side
  name resolves to a declared registry entry; "unresolved name
  at runtime" is not a possible failure mode beyond malformed
  IR detection.

Component-level scoping, dotted access (`component.state`),
and renaming-on-import are out of scope for M2; they live in
M3's binding-feature DDs.

**Recommendation:** **Option B**, with `state` declarations in
`.ui` (Signal ownership in DSL).

Counter requires `i32` and string only; restricting type
inference to those two avoids designing an inference rule set
for a language whose grammar M3 still moves. Property-binding
lowering as a distinct pass keeps the pipeline diagnosable.
Signal ownership in `.ui` keeps the host surface narrow and
discharges A2's "no host-side plumbing" requirement
structurally rather than by host convention.

**Forward-compat exposure:**

- Out-of-scope items engaged: M3 DSL spec finalisation
  (general type inference, expanded type set); M5 LSP /
  diagnostics; post-1.0 hot reload.
- B's restricted inference is replaced (not extended) in M3;
  the replacement is straightforward because B's lowering
  passes are already structured for general types — only the
  inference rule set needs filling in.
- A locks in inference rules likely to conflict with M3 spec.
  C complicates diagnostics for no compensating benefit.

**Technical-risk re-evaluation:** B's risk is the smallest
that satisfies A1; A is high-risk for M2; C is low-risk but
design-degraded. Risk reinforces B.

---
