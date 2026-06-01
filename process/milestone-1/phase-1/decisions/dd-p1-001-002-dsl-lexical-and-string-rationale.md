### DD-P1-001 — `in-out` is a single keyword token

**Status:** Accepted

**Context:**
The only property modifier in M1 is `in-out`.

**Decision:** The lexer emits a single `Token::InOut` for the literal
string `in-out`. It does not split it into `Ident("in")`, `Minus`,
`Ident("out")`.

**Rationale:**
Treating `in-out` as a single token keeps the grammar unambiguous without
context-sensitivity. The alternative (3-token split) would make `-` serve
double duty as both an arithmetic operator and a keyword separator, which
complicates the grammar as soon as expression syntax expands in M2.

**Explicitly deferred:** `in` (read-only from outside) and `out`
(write-only from outside) as standalone modifiers. These remain post-M2
scope.

**Future impact:** When `in` and `out` are introduced as standalone
modifiers, the lexer will need to be updated. Two viable paths at that
point:

- Promote `in` and `out` to separate keywords and keep `in-out` as a
  third compound keyword.
- Drop the compound `InOut` token and instead have the parser recognize
  `In Minus Out`.

The right choice depends on whether the future expression grammar also
adds `-` inside property bindings. That decision belongs to the milestone
that expands the DSL expression surface.

---

### DD-P1-002 — String interpolation is parsed structurally but not evaluated

**Status:** Accepted

**Context:**
M1 string literals may contain `\{...}` placeholders, but M1 does not
evaluate bindings.

**Options:**

Option A — Raw string
- AST type: `String`
- M1 error detection: none; malformed `\{root.}` is silently accepted.
- M2 compatibility: M2 must re-parse strings.

Option B — Structured string parts
- AST type: `Vec<StringPart>`
- M1 error detection: syntax errors in placeholders are caught.
- M2 compatibility: M2 evaluates existing `Interp` nodes.

Option C — Raw string plus validation pass
- AST type: `String`
- M1 error detection: caught, but via a second parse.
- M2 compatibility: M2 must still re-parse.

**Decision:** Option B — string literals that contain `\{...}`
placeholders are stored in the AST as `Expr::StringLit(Vec<StringPart>)`,
where `StringPart` is either `Text(String)` or `Interp(QualifiedName)`.
The interpolation is parsed into structure at M1, but the resulting value
is never computed; `Interp` nodes are inert data.

**Rationale:**
Parsing the structure once at lex/parse time avoids re-parsing in M2 and
catches obvious mistakes (for example `\{root.}`) early without adding
significant complexity. The lexer merely switches to a mini-mode inside
`\{...}` to tokenize a `qualified_name`.

**Discharged in M2:** Reactive evaluation of `Interp` nodes for the
Foundation counter surface. M2 consumes the structured interpolation
nodes when lowering property bindings to IR.

**M2 impact:** The M2 reactive engine consumes
`StringPart::Interp(QualifiedName)` nodes directly. It resolves the
`QualifiedName` against the component's property scope, subscribes to
changes, and re-evaluates the concatenated string on each change. No AST
schema change was required; M2 added lowering/evaluation logic, not a new
source representation. String-typed interpolation lowers to
`str-prop-read` in `;wasamo-ir v0`. M3-Phase 1 rejects `bool`-typed state
interpolation at `wasamoc check` time rather than lowering it to a
runtime `TypeMismatch`; an explicit formatting/display-conversion surface
is a future design item.
