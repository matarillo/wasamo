### DD-M2-P2-002 — IR artifact form

**Status:** Accepted

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
