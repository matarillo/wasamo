# Wasamo DSL Specification

**Document version:** 0.4
**Last updated:** 2026-05-11
**Status:** M2 complete; covers the M2 `.ui` surface and `;wasamo-ir v0`

---

## 1. Overview

The Wasamo DSL is an external domain-specific language for declaring UI component structure.
Source files use the `.ui` extension.

M2 scope covers lexing, parsing, checking, IR text emission, runtime IR
loading, inline handler evaluation, and reactive property binding for the
Foundation counter surface.

### Reference example (`examples/counter/counter.ui`)

```
component Counter inherits Window {
    title: "Counter"
    backdrop: mica
    theme: system

    in-out property <int> count: 0

    VStack {
        spacing: 12px
        padding: 24px

        Text {
            text: "Count: \{root.count}"
            font: title
        }
        Button {
            text: "Increment"
            style: accent
            clicked => { root.count += 1; }
        }
    }
}
```

---

## 2. Lexical Elements

### 2.1 Keywords

| Keyword     | Description                              |
|-------------|------------------------------------------|
| `component` | Starts a component declaration           |
| `inherits`  | Names the base type                      |
| `in-out`    | Property modifier: readable and writable |
| `property`  | Starts a property declaration            |

`in-out` is lexed as a **single keyword token** (not `in`, `-`, `out`).

### 2.2 Token types

| Token       | Lexical pattern                        | Examples                     |
|-------------|----------------------------------------|------------------------------|
| `Keyword`   | See §2.1                               | `component`, `in-out`        |
| `Ident`     | `[A-Za-z_][A-Za-z0-9_]*`              | `Counter`, `VStack`, `count` |
| `IntLit`    | `[0-9]+`                               | `0`, `12`, `24`              |
| `FloatLit`  | `[0-9]+\.[0-9]+`                       | `1.5`, `0.0`                 |
| `StringLit` | `"` string content `"`                 | `"Counter"`, `"Count: \{…}"` |
| `Unit`      | `px`                                   | `px`                         |
| `LBrace`    | `{`                                    |                              |
| `RBrace`    | `}`                                    |                              |
| `LAngle`    | `<`                                    |                              |
| `RAngle`    | `>`                                    |                              |
| `Colon`     | `:`                                    |                              |
| `Arrow`     | `=>`                                   |                              |
| `Dot`       | `.`                                    |                              |
| `Semicolon` | `;`                                    |                              |
| `PlusEq`    | `+=`                                   |                              |
| `MinusEq`   | `-=`                                   |                              |
| `StarEq`    | `*=`                                   |                              |
| `SlashEq`   | `/=`                                   |                              |
| `Eq`        | `=`                                    |                              |

### 2.3 Whitespace and comments

Whitespace (space, tab, `\r`, `\n`) is ignored between tokens.
M1 does not support comments. Line comments (`//`) are M2 scope.

### 2.4 String literals

String literals are delimited by double quotes `"…"` and may contain:

- Ordinary Unicode characters (except unescaped `"` and `\`).
- Escape sequences: `\\`, `\"`.
- Interpolation placeholder: `\{` *qualified\_name* `}`.

Interpolation syntax: `\{` followed by one or two `IDENT` segments separated by `.`,
followed by `}`.

In M1 the entire string content (including placeholders) is stored **as-is** in the AST.
No evaluation or reactive binding is performed at parse time.

---

## 3. Grammar

Notation: `::=` defines a rule; `|` is alternation; `*` zero-or-more;
`+` one-or-more; `?` optional; `( )` grouping;
terminals appear in `"quotes"` or ALL_CAPS.

```
file             ::= component_def EOF

component_def    ::= "component" IDENT "inherits" IDENT
                     "{" member* "}"

member           ::= property_decl
                  |  property_bind
                  |  widget_decl
                  |  signal_handler

property_decl    ::= "in-out" "property" "<" type_name ">" IDENT
                     ":" expr

property_bind    ::= IDENT ":" expr

widget_decl      ::= IDENT "{" member* "}"

signal_handler   ::= IDENT "=>" block

block            ::= "{" statement* "}"

statement        ::= assign_stmt ";"

assign_stmt      ::= qualified_name assign_op expr

assign_op        ::= "+=" | "-=" | "*=" | "/=" | "="

qualified_name   ::= IDENT ("." IDENT)*

expr             ::= STRING_LIT
                  |  number_with_unit
                  |  IDENT

number_with_unit ::= (INT_LIT | FLOAT_LIT) UNIT?

UNIT             ::= "px"

type_name        ::= "int" | "string" | "float" | "bool"
```

### Disambiguation

Within `member`, a 2-token lookahead resolves the alternative:

| First token | Second token | Rule matched      |
|-------------|--------------|-------------------|
| `in-out`    | `property`   | `property_decl`   |
| `IDENT`     | `:`          | `property_bind`   |
| `IDENT`     | `{`          | `widget_decl`     |
| `IDENT`     | `=>`         | `signal_handler`  |

---

## 4. Semantics (M1 Scope)

### 4.1 `component` declaration

```
component <Name> inherits <Base> { … }
```

Declares a named UI component. `<Base>` is stored as a string; no base-type validation
is performed in M1.

Each `.ui` file contains exactly **one** top-level `component` declaration.
Multiple components per file are M2 scope.

### 4.2 `in-out property` declaration

```
in-out property <type> <name>: <default>
```

Declares a component-level mutable property with a type annotation and a default value.

Supported types in M1: `int`, `string`, `float`, `bool`.

`in` (read-only from outside) and `out` (write-only from outside) modifiers are M2 scope.

### 4.3 Property binding

```
<name>: <expr>
```

Associates a value with a named property. In M1 all bindings are **static**: they are
evaluated once at construction time. Reactive re-evaluation is M2 scope.

### 4.4 Widget declaration

```
<WidgetType> { … }
```

Declares a child widget. Widget type names are PascalCase identifiers.
`wasamoc check` validates the type name against the M1 widget registry below:

| Widget name | Description              |
|-------------|--------------------------|
| `VStack`    | Vertical stack container |
| `HStack`    | Horizontal stack container |
| `Text`      | Text display             |
| `Button`    | Clickable button         |
| `Rectangle` | Solid rectangle          |

Unknown widget type names produce a warning (not an error) in M1,
to allow forward-compatibility with user-defined components.

### 4.5 Signal handler

```
<signal_name> => { <statements> }
```

Attaches a handler to a named signal. The body is parsed for **structural correctness only**
(balanced braces, valid statement syntax). No type-checking or name resolution is performed
inside `{ }` in M1.

The only recognized signal name in M1 is `clicked`.

### 4.6 Expressions

| Expression form    | AST representation                          |
|--------------------|---------------------------------------------|
| `"…"` string       | `Expr::StringLit(String)` — raw content     |
| `42` integer       | `Expr::IntLit(i64)`                         |
| `3.14` float       | `Expr::FloatLit(f64)`                       |
| `12px` measurement | `Expr::Measurement { value: f64, unit: Unit }` |
| `mica` identifier  | `Expr::Ident(String)` — no resolution       |

---

## 5. AST Structure (M1)

The Rust type definitions live in `wasamoc/src/ast.rs`.

```
ComponentDef {
    name:    String,
    base:    String,
    members: Vec<Member>,
}

Member (enum) {
    PropertyDecl  { name: String, ty: TypeName, default: Expr },
    PropertyBind  { name: String, value: Expr },
    WidgetDecl    { type_name: String, members: Vec<Member> },
    SignalHandler { signal: String, body: Block },
}

StringPart (enum) {
    Text(String),
    Interp(QualifiedName),
}

Expr (enum) {
    StringLit   { parts: Vec<StringPart> },
    IntLit      { value: i64 },
    FloatLit    { value: f64 },
    Measurement { value: f64, unit: Unit },
    Ident       { name: String },
}

Unit (enum) { Px }

TypeName (enum) { Int, Str, Float, Bool }

Block { statements: Vec<Statement> }

Statement {
    target: QualifiedName,
    op:     AssignOp,
    value:  Expr,
}

QualifiedName { segments: Vec<String> }

AssignOp (enum) { Eq, PlusEq, MinusEq, MulEq, DivEq }
```

All AST nodes carry a `span: Span` field (byte offset, line, col) for error reporting.

---

## 6. `wasamoc check` Command

```
wasamoc check <file.ui>
```

- Parses the given `.ui` file against the M1 grammar.
- Exits with code `0` and no output on success.
- Exits with code `1` and prints diagnostics to stderr on any error.

Error output format:

```
error: <message>
  --> <filename>:<line>:<column>
   |
8  |     Buttun {
   |     ^
```

Warnings use the same format with `warning:` in place of `error:`.
Warnings are printed to stderr but do not affect the exit code.

---

## 7. Scope Out (M2 and Later)

The following are explicitly **out of scope for M1**:

| Feature                                             | Deferred to |
|-----------------------------------------------------|-------------|
| `in` / `out` property modifiers                     | M2          |
| Reactive property bindings (auto-update on change)  | M2          |
| `\{…}` interpolation evaluation                     | M2          |
| Signal body type-checking and name resolution       | M2          |
| Line comments (`//`)                                | M2          |
| Multiple components per file                        | M2          |
| Import / module system                              | M2          |
| Code generation (runtime call emission)             | M2          |
| Conditional widgets (`if`, `for`)                   | M2+         |

---

---

## 8. Wasamo IR — Normative Specification (M2)

The **Wasamo IR** is the textual file format emitted by `wasamoc` and consumed
by the `wasamo-runtime` loader.  It is the contract between the two tools;
this chapter specifies it normatively (DD-M2-P6-002 = Option B).

The IR is not intended for hand-authoring.  Its surface form is optimised for
diff-readability and machine parsability, not ergonomics.

### 8.1 File header

Every IR file begins with a magic + version line:

```
;wasamo-ir v0
```

- The line starts with `;` (semicolon), which also serves as the IR comment
  character.
- `wasamo-ir` is the format name; `v0` is the format version.
- The loader **rejects** any file whose first line does not match this literal
  exactly, returning `WASAMO_ERR_IR_MALFORMED`.
- When the grammar evolves incompatibly, the version is bumped (e.g. `v1`).
  The bump policy is: any change to the grammar that would cause a v0 file to
  parse differently under a v1 parser requires a version bump.

### 8.2 Notation

Grammar rules use the same notation as §3:

- `::=` defines a rule; `|` alternation; `*` zero-or-more; `+` one-or-more;
  `?` optional; `( )` grouping.
- Terminals appear in `"quotes"` or ALL_CAPS token names.
- `IDENT` matches `[A-Za-z_][A-Za-z0-9_.\-]*` (dots and hyphens allowed for
  path segments and widget-type names).
- `INT` matches `[0-9]+` with an optional leading `-`.
- `STRING` matches a double-quoted string with `\"` and `\\` escapes.
- Whitespace (space, tab, `\r`, `\n`) is ignored between tokens.
- A `;` outside the header line begins a line comment; the rest of that line
  is ignored.

### 8.3 Top-level grammar

```
ir_file        ::= header component_def EOF

header         ::= ";wasamo-ir v0" NEWLINE

component_def  ::= "component" IDENT "inherits" IDENT
                   "{" component_body "}"

component_body ::= state_decl* widget_node
```

One `component_def` per IR file (matches the M2 single-component restriction
from DD-M2-P6-004).

### 8.4 State declarations

`state` declarations encode the Signal ownership transferred from the DSL
(DD-M2-P6-004 = B).  The runtime allocates a `Signal<T>` for each one.

```
state_decl ::= "state" IDENT ":" type_name "=" literal
```

| Element     | Meaning                                          |
|-------------|--------------------------------------------------|
| `IDENT`     | Signal name; unique within the component (flat namespace) |
| `type_name` | `"i32"` or `"string"` (M2 type set)             |
| `literal`   | Default value: `INT` for `i32`; `STRING` for string |

Example:

```
state count: i32 = 0
```

### 8.5 Widget nodes

```
widget_node ::= "node" IDENT "{" node_body "}"

node_body   ::= (property_set | binding | handler | widget_node)*
```

`IDENT` is the widget type (e.g. `Window`, `VStack`, `Text`, `Button`).
Children appear as nested `node` blocks in document order.

### 8.6 Property sets

A `property_set` writes a static value to a widget property at load time.
It is used for properties whose value is a plain literal (not reactive).

```
property_set ::= "prop" IDENT "=" literal

literal      ::= INT | STRING | IDENT
```

The third `literal` alternative (`IDENT`) encodes keyword-valued properties
such as `mica`, `system`, `accent`, `title` (see §4.3).

Examples:

```
prop title = "Counter"
prop backdrop = mica
prop spacing = 12
prop padding = 24
```

### 8.7 Reactive bindings

A `binding` wires a `HandlerExpr` to a widget property reactively.  Every
time a referenced Signal changes, the expression is re-evaluated and the
property is updated.

```
binding ::= "bind" IDENT "=" expr
```

`IDENT` is the property name on the enclosing widget node.

`expr` is a `HandlerExpr` in the tagged-value form defined in §8.9.

Example (the `text` property of `Text`, reactive on `count`):

```
bind text = (interp "Count: " (prop-read count))
```

### 8.8 Signal handlers

A `handler` attaches a `HandlerExpr` body to a named signal on the enclosing
widget.

```
handler ::= "on" IDENT "{" expr "}"
```

`IDENT` is the signal name (e.g. `clicked`).

The body is one `expr`.  Multiple top-level statements are encoded as a
`(block ...)` expression (§8.9).

Example (the `clicked` handler on `Button`):

```
on clicked {
    (compound-assign += count (lit 1))
}
```

### 8.9 Expressions (`HandlerExpr` tagged-value form)

Expressions are written in a parenthesised prefix form.  Each form maps
1-to-1 to a `HandlerExpr` variant (DD-M2-P6-003 = Option A).

**Bare-literal shorthand.** Where the position is unambiguous (i.e. the
parser expects an expression and the next token is `INT` or `STRING`), a
bare literal may be written without the `(lit ...)` wrapper.  The grammar
below calls these positions out with `atom`.

```
expr  ::= atom
        | "(" "lit"             INT ")"
        | "(" "str"             STRING ")"
        | "(" "prop-read"       IDENT ")"
        | "(" "str-prop-read"   IDENT ")"
        | "(" "assign"          IDENT expr ")"
        | "(" "compound-assign" compound_op IDENT expr ")"
        | "(" "interp"          interp_part+ ")"
        | "(" "block"           expr* ")"

atom  ::= INT
        | STRING

compound_op ::= "+=" | "-=" | "*=" | "/="

interp_part ::= STRING         ; literal text fragment
              | "(" expr ")"   ; embedded expression (re-uses the expr rule)
```

**Mapping to `HandlerExpr` variants:**

| IR form | `HandlerExpr` variant | Notes |
|---|---|---|
| `INT` / `(lit INT)` | `IntLit(i32)` | Bare `INT` is equivalent to `(lit INT)` |
| `STRING` / `(str STRING)` | `StrLit(String)` | Binding-only |
| `(prop-read NAME)` | `PropRead { path }` | `NAME` is the Signal name from `state` |
| `(str-prop-read NAME)` | `StrPropRead { path }` | String-typed binding read; `NAME` is the Signal name from `state` |
| `(assign NAME expr)` | `Assign { lhs, rhs }` | Handler-only |
| `(compound-assign OP NAME expr)` | `CompoundAssign { lhs, op, rhs }` | Handler-only |
| `(interp part+)` | `Interpolation(Vec<InterpolationPart>)` | Binding-only |
| `(block expr*)` | `Block(Vec<HandlerExpr>)` | Empty block evaluates to `0` |

**`interp_part` mapping:**

| Part | `InterpolationPart` variant |
|---|---|
| `STRING` | `Literal(String)` |
| `(expr)` | `Expr(HandlerExpr)` |

### 8.10 Complete annotated example

The following is the full IR for `examples/counter/counter.ui`:

```
;wasamo-ir v0

component Counter inherits Window {
    ; Signal declarations (DD-M2-P6-004 = B: state ownership in .ui)
    state count: i32 = 0

    ; Root window node — static properties only
    node Window {
        prop title = "Counter"
        prop backdrop = mica
        prop theme = system

        node VStack {
            prop spacing = 12
            prop padding = 24

            node Text {
                ; Reactive binding: re-evaluates whenever `count` changes
                bind text = (interp "Count: " (prop-read count))
                prop font = title
            }

            node Button {
                prop text = "Increment"
                prop style = accent

                ; Signal handler body: count += 1
                on clicked {
                    (compound-assign += count (lit 1))
                }
            }
        }
    }
}
```

### 8.11 Loader validation policy (DD-M2-P6-009 = C)

The runtime loader (`wasamo-runtime/src/ir_loader.rs`) applies
defense-in-depth validation:

| Check | Enforced at load | On failure |
|---|---|---|
| Header line matches `;wasamo-ir v0` | Yes | `WASAMO_ERR_IR_MALFORMED` |
| Top-level structure is `component_def` | Yes | `WASAMO_ERR_IR_MALFORMED` |
| Every `prop-read` / `str-prop-read` / `assign` / `compound-assign` name resolves to a declared `state` | Yes | `WASAMO_ERR_IR_MALFORMED` |
| Binding expression result type matches target property type | **No** (trusted from `wasamoc`) | Undefined behaviour |
| Per-node emitter invariants (e.g. `on` only on signal-capable widgets) | **No** (trusted from `wasamoc`) | Undefined behaviour |

The loader trusts type-level invariants established by `wasamoc`'s check pass.
Type mismatches indicate a `wasamoc` bug, not a recoverable load-time error.

### 8.12 Scope out (post-M2)

| Feature | Deferred to |
|---|---|
| `(computed ...)` expression form | M3 |
| `(if ...)` / `(for ...)` binding forms | M3+ |
| M3 expanded type set (`float`, `bool`, user types) | M3 |
| Binary IR format | Post-M2 |
| Grammar version `v1` (first incompatible change) | When required |
| `(post-event ...)` escape hatch for observer callbacks | M3 (DD-M2-P6-001 Option F) |

---

## Appendix A: Design Decisions

### DD-001 — `in-out` is a single keyword token

**Decision:** The lexer emits a single `Token::InOut` for the literal string `in-out`.
It does not split it into `Ident("in")`, `Minus`, `Ident("out")`.

**Rationale:**
The only property modifier in M1 is `in-out`. Treating it as a single token keeps the
grammar unambiguous without context-sensitivity. The alternative (3-token split) would
make `-` serve double duty as both an arithmetic operator and a keyword separator, which
complicates the grammar as soon as expression syntax expands in M2.

**Explicitly deferred:** `in` (read-only from outside) and `out` (write-only from outside)
as standalone modifiers. These remain post-M2 scope.

**Future impact:** When `in` and `out` are introduced as standalone modifiers, the
lexer will need to be updated. Two viable paths at that point:

- Promote `in` and `out` to separate keywords and keep `in-out` as a third compound keyword.
- Drop the compound `InOut` token and instead have the parser recognize `In Minus Out`.

The right choice depends on whether the future expression grammar also adds `-`
inside property bindings. That decision belongs to the milestone that expands
the DSL expression surface.

---

### DD-002 — String interpolation is parsed structurally but not evaluated

**Decision:** String literals that contain `\{…}` placeholders are stored in the AST as
`Expr::StringLit(Vec<StringPart>)`, where `StringPart` is either `Text(String)` or
`Interp(QualifiedName)`. The interpolation is parsed into structure at M1, but the
resulting value is never computed — `Interp` nodes are inert data.

**Rationale:**
Three options were considered:

| Option | AST type | M1 error detection | M2 compatibility |
|--------|----------|--------------------|------------------|
| Raw string | `String` | None — malformed `\{root.}` silently accepted | M2 must re-parse strings |
| Structured (chosen) | `Vec<StringPart>` | Syntax errors in placeholders caught | M2 evaluates existing `Interp` nodes |
| Raw string + validation pass | `String` | Caught, but via a second parse | M2 must still re-parse |

Parsing the structure once at lex/parse time avoids re-parsing in M2 and catches obvious
mistakes (e.g. `\{root.}`) early without adding significant complexity — the lexer merely
switches to a mini-mode inside `\{…}` to tokenize a `qualified_name`.

**Discharged in M2:** Reactive evaluation of `Interp` nodes for the Foundation
counter surface. M2 consumes the structured interpolation nodes when lowering
property bindings to IR.

**M2 impact:** The M2 reactive engine consumes `StringPart::Interp(QualifiedName)`
nodes directly. It resolves the `QualifiedName` against the component's property scope,
subscribes to changes, and re-evaluates the concatenated string on each change. No AST
schema change was required; M2 added lowering/evaluation logic, not a new source
representation. String-typed interpolation lowers to `str-prop-read` in
`;wasamo-ir v0`.

---

## Revision history

| Version | Date       | Notes                                                                             |
|---------|------------|-----------------------------------------------------------------------------------|
| 0.1     | 2026-04-27 | Initial draft (Phase 1, pending owner agreement)                                  |
| 0.2     | 2026-04-27 | Phase 1 Accepted; added missing tokens (MinusEq/StarEq/SlashEq); corrected AST types (StringLit → Vec<StringPart>, Statement as struct); corrected error output format |
| 0.3     | 2026-05-07 | M2-Phase 6 Accepted; added §8 Wasamo IR normative spec (DD-M2-P6-002 + DD-M2-P6-003) |
| 0.4     | 2026-05-11 | M2 complete; added `str-prop-read` IR form from DD-M2-P6-011 and updated M2/post-M2 status language |
