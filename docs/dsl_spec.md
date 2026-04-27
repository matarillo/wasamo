# Wasamo DSL Specification — M1 Scope

**Document version:** 0.2
**Last updated:** 2026-04-27
**Status:** Phase 1 agreed

---

## 1. Overview

The Wasamo DSL is an external domain-specific language for declaring UI component structure.
Source files use the `.ui` extension.

M1 scope covers **lexing, parsing, and syntax checking only**.
Code generation and reactive evaluation are M2 scope.

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
as standalone modifiers. These are M2 scope.

**Future impact (M2):** When `in` and `out` are introduced as standalone modifiers, the
lexer will need to be updated. Two viable paths at that point:

- Promote `in` and `out` to separate keywords and keep `in-out` as a third compound keyword.
- Drop the compound `InOut` token and instead have the parser recognize `In Minus Out`.

The right choice depends on whether M2 also adds `-` to expression syntax inside property
bindings. That decision belongs in the M2 DSL spec revision.

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

**Explicitly deferred:** Reactive evaluation of `Interp` nodes (observing state changes and
re-rendering affected text). This is M2 scope.

**Future impact (M2):** The M2 reactive engine consumes `StringPart::Interp(QualifiedName)`
nodes directly. It resolves the `QualifiedName` against the component's property scope,
subscribes to changes, and re-evaluates the concatenated string on each change. No AST
schema change is required; M2 adds evaluation logic, not a new representation.

---

## Revision history

| Version | Date       | Notes                                                                             |
|---------|------------|-----------------------------------------------------------------------------------|
| 0.1     | 2026-04-27 | Initial draft (Phase 1, pending owner agreement)                                  |
| 0.2     | 2026-04-27 | Phase 1 agreed; added missing tokens (MinusEq/StarEq/SlashEq); corrected AST types (StringLit → Vec<StringPart>, Statement as struct); corrected error output format |
