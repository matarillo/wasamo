### DD-M2-P6-003 — IR representation of `HandlerExpr` and binding expressions

**Status:** Accepted

**Context:**
DD-M2-P3-001 = A established `HandlerExpr` as the in-runtime AST
for handler bodies. DD-M2-P5-005 = A reuses `HandlerExpr` as the
binding-expression AST. The IR must serialise both in a form the
runtime parser can rebuild as `HandlerExpr` values. The Phase 2
spike used a tagged-value form for handler bodies (e.g.
`(assign root.count (add (read root.count) 1))`); whether to
promote that form, replace it, or unify it with property values
is the question here.

**Options:**

Option A — Promote the Phase 2 spike's tagged-value form (recommended)
- Each `HandlerExpr` variant has a distinct head tag (`assign`,
  `add`, `read`, literal forms). Serialisation is a direct
  recursive walk of the AST. Binding expressions and handler
  bodies share the form; the difference between them is the
  *target* (DD-M2-P6-007), not the expression shape.
- What you gain: parser/emitter pair already exists in the
  spike; conceptual fit with `HandlerExpr` is exact (the AST
  was designed for this lowering); diff-friendly when the
  grammar (DD-M2-P6-002 Option B) puts one node per line.
  Sharing between bindings and handlers exercises the
  evaluator-core sharing established in DD-M2-P5-002.
- What you give up: the form is verbose for trivial property
  literals (a number `1` becomes `(lit 1)` rather than `1`);
  acceptable for M2, addressable in DD-M2-P6-002's grammar by
  permitting bare literals where the position is unambiguous.
- **Technical risk: Low.**

Option B — Custom expression mini-language with infix syntax
- Use infix `+`, `=`, `.` etc.; parse to `HandlerExpr` via a
  small precedence parser.
- What you gain: handler bodies read like the source DSL.
- What you give up: a second precedence parser to maintain
  alongside `wasamoc`'s; ambiguity with property literal
  values that contain operators (strings); tooling cost
  outweighs M2 ergonomic gain.
- **Technical risk: Medium.**

Option C — Distinct schemes for bindings vs handlers
- Bindings serialise with one shape, handler bodies with
  another.
- What you gain: each can be optimised independently.
- What you give up: defeats the evaluator-core sharing
  (DD-M2-P3-001/DD-M2-P5-002); two parsers and two emitters
  to maintain; the runtime evaluator must accept either
  origin.
- **Technical risk: Medium.**

**Recommendation:** **Option A.**

The tagged-value form's verbosity is a trivially addressable
artefact of DD-M2-P6-002's grammar choice; everything else aligns
with prior decisions. Bindings and handlers use one expression
shape, mapping 1:1 to `HandlerExpr` variants.

**Forward-compat exposure:**

- Out-of-scope items engaged: M3 binding features (Computed,
  conditional, for-loop expressions); M3 DSL spec finalisation
  (richer expression forms — function call, ternary).
- A is additive: M3 expression forms add `HandlerExpr`
  variants and IR head tags in lockstep; no parser-shape
  change. B's precedence parser fights M3 syntax additions
  (each new operator is a precedence-table edit). C's parallel
  shapes amplify the M3 change cost across both schemes.

**Technical-risk re-evaluation:** A's incremental risk is
near-zero. Risk reinforces A.

---
