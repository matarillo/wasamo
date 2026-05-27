### DD-M2-P6-002 — Normative grammar of the textual IR

**Status:** Accepted

**Context:**
DD-M2-P2-001 = B settled "textual IR" as the M2 wasamoc output
shape. It did not specify the surface form normatively; the Phase 2
spike used an s-expression shape sufficient to pass the
`experimental_ir_loader` round-trip test. Phase 6 promotes the
textual IR from "what the spike happens to write" to "the contract
between `wasamoc` and `wasamo-runtime`". This DD picks the surface
form and where the spec lives.

**Options:**

Option A — Promote the Phase 2 spike s-expression form as-is
- Name the spike's grammar in a normative spec; freeze it.
- What you gain: zero design work; the spike's existing
  round-trip is the conformance test; no parser rewrite.
- What you give up: the spike grammar was sized for one
  round-trip on counter, not for human readability or
  versioning. Some node shapes (handler-body emission in
  particular) are tagged-value flavour with thin error
  reporting; promoting them locks in shapes the spike author
  did not optimise for.
- **Technical risk: Low.**

Option B — Design a new normative grammar (recommended)
- Specify a textual grammar fit for the contract: explicit
  productions for tree nodes, properties, bindings, handler
  bodies, and (per DD-M2-P6-002 sub-issue) a header line.
  Re-use the spike's parser implementation where it agrees
  with the new grammar; rewrite where it does not.
- What you gain: the grammar is the spec, not an
  implementation accident. Header line accommodates fail-fast
  on stale-`wasamoc` / new-runtime in post-M2 hot-reload-like
  scenarios. Diagnostics target the grammar, not parser
  internals. The grammar can be written to be diff-friendly
  (one node per line, indented children), which matters for
  reviewing generated IR during M3 binding development.
- What you give up: design + spec-writing time; some
  rewriting of `experimental_ir_loader` and `wasamoc` emit;
  a new freeze surface this ADR creates.
- **Technical risk: Low–medium** (parser rewrite scope; not
  conceptually novel).

Option C — Adopt a third format (JSON, TOML, custom binary stub)
- Replace s-expression with a different surface form.
- What you gain: structural validators may exist off-the-shelf
  (JSON Schema, etc.).
- What you give up: JSON is poor for handler bodies (no
  expression-tree sugar); TOML is structurally wrong for
  trees; binary is a non-goal for M2 (textual is the explicit
  Phase 2 choice). All three discard parser/emitter code that
  already works.
- **Technical risk: Medium** (replacing more than the
  grammar).

**Header / version contract sub-issue.** Whether the IR mandates a
magic + version line at file head (e.g. `;wasamo-ir v0`).
M2 co-builds `wasamoc` and `wasamo-runtime` in a single workspace,
so version skew is not a correctness concern *in M2*. Writing the
contract now is cheap and protects post-M2 scenarios (hot reload,
shipped pre-built IR) from silent acceptance of stale output.
**Recommended: include a header line.** Reject load on
mismatch; document the bump policy.

**Recommendation:** **Option B**, with a header line.

The textual IR is the contract Phase 6 makes load-bearing.
"Whatever the spike emits" is not a contract; specifying the
grammar normatively is the smallest change that makes the
artifact reviewable. Header-line cost is one line of parser
work and one paragraph of spec.

**Spec home.** Extend `docs/dsl_spec.md` with an IR chapter. A
separate `docs/ir_spec.md` is rejected because the IR is bound
tightly to DSL constructs (binding expressions, handler bodies);
splitting the document fragments the per-construct
documentation. The IR chapter cross-references the DSL chapter
where the lowering target maps directly to a DSL form.

**Forward-compat exposure:**

- Out-of-scope items engaged: post-1.0 hot reload, M3 binding
  features (Computed, conditional, for-loop), M5 LSP/diagnostics.
- B + header survives all three: header version bumps with
  grammar additions; M3 binding features add productions
  additively; LSP attaches to the grammar, not the parser
  implementation.
- A's "spike-shape as spec" carries hidden-decision risk into
  every later extension. C front-loads format change with no
  M2 acceptance benefit.

**Technical-risk re-evaluation:** B's risk is bounded to
parser/emitter rewrite scope; the conceptual change is small.
Risk reinforces B.

---
