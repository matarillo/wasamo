# Wasamo DSL Specification

**Document version:** 1.21
**Last updated:** 2026-08-09
**Status:** `public-draft` (M3) — this document is the first public
draft of the Wasamo DSL specification, promoted at M3-Phase 8 close;
the promotion record and the M3 decision links live in the
[public-draft change history](#public-draft-change-history). A public
draft is **not** a backward-compatibility guarantee (§4.18). Phase
state: M3-Phase 2 closed (implementation-synced); M3-Phase 3
closed (implementation-synced); M3-Phase 4 closed
(implementation-synced); M3-Phase 5 closed (implementation-synced);
M3-Phase 6 closed (implementation-synced); M3-Phase 7 closed
(implementation-synced); M3-Phase 7b closed (implementation-synced —
`slot.*` placement surface, §4.16); M3-Phase 8 closed
(implementation-synced — the `ToggleButton` / `checked` selected-state
surface, §4.17, and the public-draft future-surface notes, §4.18,
match the landed implementation; external-reader smoke verified in
M3-Phase 8 T8). M4-Phase 1 implementation-synced: the unit of every authored
length is defined as DIP (§1 *Units and the layout coordinate system*),
replacing the previously undefined "pixel extents in the layout
coordinate system" wording; the grammar, AST, IR, and authored numeric values
are unchanged, and the landed runtime keeps layout and font-size inputs in DIP.
M4-Phase 2 closed (implementation-synced): the interaction surface (§4.19) —
`clicked` on any widget, one-target hit resolution with consume-on-handle
propagation, per-item handlers inside `for` with invocation-time binder reads,
the `focus-group` / `modal-scope` container attributes, the `dismiss` request,
and the `key-down("<key>")` command surface — matches the landed runtime.
Covers the M2 `.ui` surface, the `state` surface keyword
retroactively, the M3-Phase 1 `bool` scalar binding additions, the
M3-Phase 2 Box layout primitive (with `aspect` / `fill` literal
attributes), the M3-Phase 3 WrapPanel layout primitive (with
`item-cross-size` / `item-spacing` / `line-spacing` constant-only
integer attributes), the M3-Phase 4 ScrollView layout primitive
(vertical-only viewport + clip + `offset-y` binding), the M3-Phase 5
Grid layout primitive (fixed + weighted-star track sizing, `Cell`
wrapper with explicit placement / span / alignment, both-axis
spanning, Grid outer-bounds clip), the M3-Phase 6 ZStack overlay
primitive (union sizing with `Fill/Fill` default, document-order
z-order, per-child alignment, outer-bounds clip) and conditional
rendering (the `if` structural control-flow member — the first chapter
of Wasamo's structural rendering model), the M3-Phase 6 component host
attribute surface, the M3-Phase 7 iteration grammar (the `for`
structural control-flow member — the second chapter of the structural
rendering model — with collection state types `i32[]` / `string[]` /
`bool[]`, list literals, whole-value collection assignment, and
author-named loop-local binders; see §4.15), the M3-Phase 7b
parent-interpreted placement surface (the shared `slot.*` namespace
unifying Grid and ZStack child placement, with Grid retaining a `Cell`
grouped form; see §4.16), the M3-Phase 8 `ToggleButton` selected /
toggle-state surface (a controlled one-way `checked` boolean attribute;
see §4.17) with its public-draft future-surface notes (§4.18), and
`;wasamo-ir v0`.

---

## 1. Overview

The Wasamo DSL is an external domain-specific language for declaring UI component structure.
Source files use the `.ui` extension.

M2 scope covers lexing, parsing, checking, IR text emission, runtime IR
loading, inline handler evaluation, and reactive property binding for the
Foundation counter surface.

<a id="units-and-the-layout-coordinate-system"></a>

### Units and the layout coordinate system

Every length an author writes in a `.ui` file — a dimension attribute, a
spacing, an offset, a Grid track size, a font size — is expressed in
**device-independent pixels (DIP)**, where **1 DIP is 1/96 inch**. This
is the unit of the layout coordinate system, and it is the only unit the
language has. Where a `px` suffix appears (a measurement expression such
as `12px`, §2.2 / §4.6) it names this unit; the dimension attributes
added in M3 take bare integers in the same unit and admit no suffix.

Two consequences follow, and they are the reason the unit is worth
stating rather than assuming.

- **An authored layout is identical at every display scale factor.** A
  `.ui` file laid out in a window of a given DIP size produces the same
  layout — the same element positions, the same sizes, the same
  wrap positions — whether that window is on a 100%, a 150%, or a 200%
  monitor. What changes is only how many device pixels each DIP occupies
  on the way to the screen. An author never writes a resolution-
  dependent number and never has a reason to ask what scale factor a
  display is at.
- **A DIP is a length, not a device pixel.** `24px` of padding is 24/96
  inch at 100%, and occupies 48 device pixels on a display set to 200%.
  Text laid out at that scale is rasterized at the device's resolution
  rather than magnified, so it stays crisp; the surface-resolution
  contract behind that is runtime architecture, not language surface,
  and is normative in
  [architecture.md §12](./architecture.md#coordinate-spaces).

  The physical size a DIP resolves to is the display's scale factor,
  which on Windows is the setting the user chose for that monitor. It
  tracks the monitor's real pixel density closely enough for authored
  sizes to be perceptually stable across displays, but it is a user
  preference rather than a measurement — a display set to 150% renders
  one DIP as 1.5 device pixels whatever its actual pixel density.

**Font sizes are DIP too.** The `font:` attribute selects from a named
typography ramp rather than carrying a number, and the ramp's four sizes
— 12, 14, 20, and 28 — are DIP font sizes, so type scales with the
display exactly as layout does. The ramp itself (its names, sizes,
weights, and family) is normative in
[architecture.md §7.3](./architecture.md#73-typographystyle-type-ramp).

Dimension-bearing sections below refer to this definition rather than
restating it. Layout arithmetic is `f32` throughout with no integer
pixel snapping (§4.9, §4.10, §4.11, §4.12); a DIP value is therefore not
required to be a whole number of device pixels at any scale factor, and
a non-integer scale factor does not round authored values.

Deciding record:
[M4-Phase 1 decisions](../process/milestone-4/phase-1/decisions/preamble.md).

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
| `state`     | Starts a state declaration (see §4.7)    |
| `true`      | Bool literal — reserved identifier (M3-Phase 1) |
| `false`     | Bool literal — reserved identifier (M3-Phase 1) |
| `if`        | Conditional rendering block (M3-Phase 6; see §4.14) |
| `else`      | Reserved control-flow keyword — no production yet (M3-Phase 6) |
| `switch`    | Reserved control-flow keyword — no production yet (M3-Phase 6) |
| `for`       | Iteration block (reserved M3-Phase 6; production M3-Phase 7; see §4.15) |
| `in`        | Iteration-header separator between binders and the collection reference (M3-Phase 7; see §4.15) |

`in-out` is lexed as a **single keyword token** (not `in`, `-`, `out`).

`true` and `false` are reserved by the lexer and may not appear as
identifiers (state names, property names, widget type names, qualified-
name segments). Using either in identifier position is a parse error.

**Structural control-flow family reservation (M3-Phase 6).** `if`,
`else`, `switch`, and `for` are reserved by the lexer and may not appear
as identifiers, mirroring the `true` / `false` reservation. `if`
(M3-Phase 6, §4.14) and `for` (M3-Phase 7, §4.15) have productions;
`else` / `switch` remain reserved ahead of their productions so the
structural control-flow family (§4.14) lands additively without a
future source break. Using any of the four in identifier position is a
parse error; a bare `else` / `switch` **block** in member position is a
separate "reserved / not yet supported" parse error that names the
construct.

**`in` reservation (M3-Phase 7).** `in` is reserved by the lexer when
its production lands: the iteration header (§4.15) needs a
non-ambiguous separator token between the binder slots and the
collection reference. `in` may no longer appear as an identifier
(state, property, widget, or binder name). The existing `in-out`
property token is a distinct single hyphenated lexeme and is
unaffected. No shipped `.ui` uses `in` as an identifier, so the
reservation breaks nothing today. Contextual sub-tokens of
not-yet-designed productions — `case` / `default` (`switch` arms) —
are still **not** reserved; each is reserved when its production is
specified. The collection method names `append` / `drop-last` (§4.15)
are **contextual names**, not reserved keywords — they remain valid
identifiers outside a collection-assignment right-hand side.

### 2.2 Token types

| Token       | Lexical pattern                        | Examples                     |
|-------------|----------------------------------------|------------------------------|
| `Keyword`   | See §2.1                               | `component`, `in-out`        |
| `Ident`     | `[A-Za-z_][A-Za-z0-9_]*(?:-[A-Za-z][A-Za-z0-9_]*)*` | `Counter`, `count`, `item-cross-size` |
| `IntLit`    | `-?[0-9]+`                             | `0`, `12`, `-1`              |
| `FloatLit`  | `[0-9]+\.[0-9]+`                       | `1.5`, `0.0`                 |
| `BoolLit`   | `true` \| `false`                      | `true`, `false`              |
| `StringLit` | `"` string content `"`                 | `"Counter"`, `"Count: \{…}"` |
| `RatioLit`  | `[0-9]+` `:` `[0-9]+`                  | `16:9`, `1:1`                |
| `ColorLit`  | `#` `[0-9A-Fa-f]{6}` \| `#` `[0-9A-Fa-f]{8}` | `#cccccc`, `#00000080` |
| `Unit`      | `px`                                   | `px` — names the DIP unit ([§1](#units-and-the-layout-coordinate-system)) |
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
| `Star`      | `*` (not followed by `=`)              | `*` (M3-Phase 5; bare `*` inside a Grid `columns:` / `rows:` track list, §4.12; `2*` lexes as `IntLit(2)` + adjacent `Star`, not one token; a bare `*` outside a track list is a parse error) |
| `SlashEq`   | `/=`                                   |                              |
| `Eq`        | `=`                                    |                              |
| `LBracket`  | `[` (M3-Phase 7; collection type suffix `i32[]` and list literal `[1, 2]`, §4.15) |  |
| `RBracket`  | `]` (M3-Phase 7)                       |                              |
| `LParen`    | `(` (M3-Phase 7; collection method-call expression `xs.append(e)` / `xs.drop-last()`, §4.15) |  |
| `RParen`    | `)` (M3-Phase 7)                       |                              |
| `Comma`     | `,` (M3-Phase 7; iteration-header binder separator and list-literal element separator, §4.15) |  |

A leading `-` is part of the `IntLit` token only; it does not extend
the `FloatLit`, measurement, or `RatioLit` surfaces, and the DSL does
not introduce a subtraction or unary-minus operator here.

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

M3-Phase 1 supports interpolation over `i32` and `string` state values.
Interpolation over a `bool`-typed `state` is a compile-time error:
`bool` may be used in bool-typed property bindings and bool handler
assignments, but no implicit bool-to-string formatting/display
conversion is defined.

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
                  |  placement_bind            ; M3-Phase 7b; see §4.16
                  |  widget_decl
                  |  signal_handler
                  |  state_decl
                  |  grid_track_list_member   ; M3-Phase 5; Grid body only
                  |  conditional_member        ; M3-Phase 6; see §4.14
                  |  iteration_member          ; M3-Phase 7; see §4.15

property_decl    ::= "in-out" "property" "<" type_name ">" IDENT
                     ":" expr

state_decl       ::= "state" IDENT ":" state_type "="
                     (expr | collection_literal)
                     ; collection_literal is M3-Phase 7 and is valid
                     ; only when state_type is a collection type (§4.7)

property_bind    ::= IDENT ":" expr

; M3-Phase 7b. A parent-interpreted placement key (§4.16). The key
; carries the reserved `slot.` prefix; `slot` is a contextual prefix,
; significant only as the head of a dotted placement key, and stays a
; valid ordinary identifier elsewhere. The RHS PARSES as a general `expr`
; (same as `property_bind`), so a state-read RHS is well-formed at the
; parse level; the placement-specific rules are CHECK-LAYER, not grammar:
; (1) the value must be a constant resolved against the closed
; placement-keyword set (alignment keyword for slot.h-align / slot.v-align,
; integer literal for slot.row / slot.column / slot.row-span /
; slot.column-span), NOT the state namespace; (2) a binding-expression RHS
; is a `wasamoc check` reject (placement is constant per instance);
; (3) admission (which parent admits which key) is a `wasamoc check` rule.
; Only a malformed KEY shape (`slot:` / `slot..h-align` / `slot.`) is a
; parser reject (§4.16).
placement_bind   ::= "slot" "." IDENT ":" expr

widget_decl      ::= IDENT "{" member* "}"

; M3-Phase 5. A Grid `columns:` / `rows:` track list. The parser routes
; to this rule ONLY inside a `Grid` widget body (a widget_decl whose
; IDENT is "Grid"); elsewhere `columns:` / `rows:` stay ordinary
; property_binds. This is a narrow Grid-specific path, not a general
; list grammar. The "*" must be ADJACENT to the INT for a weighted star:
; "1*" is one weighted-star track, but "1 *" is Fixed(1) then a unit
; star. Value-range checks (Fixed >= 1, weight in [1, 1024]) and the
; reserved-future "auto" rejection are wasamoc check's job (§4.12).
grid_track_list_member
                 ::= ("columns" | "rows") ":" grid_track grid_track*

grid_track       ::= INT_LIT "*"   ; weighted star (INT_LIT adjacent to "*")
                  |  INT_LIT       ; fixed track
                  |  "*"           ; unit star (= "1*")

; M3-Phase 6. A conditional rendering block (§4.14). The body admits
; EXACTLY ONE widget child this phase — no property/bind/handler/state/
; track-list member, no nested conditional_member, no multiple children.
; The grammar admits conditional_member wherever `member` appears, but
; `wasamoc check` restricts it semantically to INSIDE a widget body (a
; component-level `if` gating/multiplying the single content root is rejected).
; The condition is the same narrow bool-expr as Button.enabled.
conditional_member
                 ::= "if" cond_expr "{" conditional_body "}"

conditional_body ::= widget_decl                ; M3-Phase 6: exactly one widget child

cond_expr        ::= BOOL_LIT | IDENT           ; M3-Phase 6: bool literal, or an
                                                ; IDENT resolving to a bool-typed state
                                                ; (loop-local binders are NOT admitted
                                                ; here — §4.15)

; M3-Phase 7. An iteration block (§4.15). The first IDENT is the
; author-named element binder; the optional second IDENT is the
; author-named index binder. The post-`in` IDENT must resolve to a
; collection-typed state declared in the same component (bare state
; name only; collection expressions are not admitted in this position).
; The body admits EXACTLY ONE widget child per iteration. `wasamoc
; check` restricts placement semantically: admitted under VStack /
; HStack / WrapPanel / ZStack; rejected under ScrollView / Box / Grid
; and at component level (§4.15).
iteration_member ::= "for" IDENT ("," IDENT)? "in" IDENT
                     "{" iteration_body "}"

iteration_body   ::= widget_decl                ; M3-Phase 7: exactly one widget
                                                ; child per iteration

signal_handler   ::= IDENT ("(" STRING_LIT ")")? "=>" block
                    ; the optional argument is admitted only by signals
                    ; whose contract defines it; M4-Phase 2 defines it for
                    ; key-down("<key>") (§4.19)

block            ::= "{" statement* "}"

statement        ::= assign_stmt ";"

assign_stmt      ::= qualified_name assign_op expr
                  |  IDENT "=" collection_expr  ; M3-Phase 7; collection-typed
                                                ; LHS, "=" only; see §4.15

assign_op        ::= "+=" | "-=" | "*=" | "/=" | "="

; M3-Phase 7. The collection-assignment RHS (§4.15). The method
; receiver IDENT must be the assigned state itself; `append` and
; `drop-last` are contextual names, not keywords.
collection_expr  ::= IDENT "." "append" "(" expr ")"
                  |  IDENT "." "drop-last" "(" ")"
                  |  collection_literal         ; static reset / clear

collection_literal ::= "[" (collection_scalar_literal
                       ("," collection_scalar_literal)*)? "]"

collection_scalar_literal ::= INT_LIT | STRING_LIT | BOOL_LIT
                       ; each element literal must match the declared
                       ; element type; no idents / operators / nesting

qualified_name   ::= IDENT ("." IDENT)*

expr             ::= STRING_LIT
                  |  number_with_unit
                  |  BOOL_LIT
                  |  RATIO_LIT
                  |  COLOR_LIT
                  |  IDENT

BOOL_LIT         ::= "true" | "false"

RATIO_LIT        ::= INT_LIT ":" INT_LIT

COLOR_LIT        ::= "#" HEX_DIGIT{6}
                  |  "#" HEX_DIGIT{8}

HEX_DIGIT        ::= [0-9A-Fa-f]

number_with_unit ::= (INT_LIT | FLOAT_LIT) UNIT?

UNIT             ::= "px"

type_name        ::= "int" | "string" | "float" | "bool"

state_type       ::= "i32" | "string" | "bool"
                  |  "i32[]" | "string[]" | "bool[]"   ; M3-Phase 7; see §4.7
```

### Disambiguation

Within `member`, a 2-token lookahead resolves the alternative:

| First token | Second token | Rule matched         |
|-------------|--------------|----------------------|
| `in-out`    | `property`   | `property_decl`      |
| `state`     | `IDENT`      | `state_decl`         |
| `if`        | (keyword)    | `conditional_member` |
| `for`       | (keyword)    | `iteration_member`   |
| `IDENT("slot")` | `.`      | `placement_bind`     |
| `IDENT`     | `:`          | `property_bind`      |
| `IDENT`     | `{`          | `widget_decl`        |
| `IDENT`     | `=>`         | `signal_handler`     |
| `IDENT`     | `(`          | `signal_handler`     |

`if` (M3-Phase 6) and `for` (M3-Phase 7) are keywords, not `IDENT`s, so
the `member` dispatch resolves a leading `if` / `for` on the first
token alone — there is no collision with `property_bind` /
`widget_decl` / `signal_handler` (all of which begin with an `IDENT`).

`slot` (M3-Phase 7b) is **not** a keyword — it is a contextual prefix
(§4.16). The 2-token lookahead distinguishes a `placement_bind` from a
`property_bind` by the second token: a leading `IDENT` whose text is
`slot` followed by `.` (the `Dot` token) routes to `placement_bind`
(`slot.<key>: <expr>`); a leading `IDENT` followed by `:` stays a
`property_bind`. The RHS parses as a general `expr` in both cases — the
placement-specific constraint that a valid checked value is a *constant*
(not a binding expression) is a check-layer rule, not a parse-level one
(§4.16). `slot` therefore remains a valid ordinary identifier everywhere
it is **not** immediately followed by `.` in member position; a property
literally named `slot` would still bind as `slot: <expr>` (second token
`:`, not `.`).
Within the iteration header, the comma-optional second binder is
resolved with one token of lookahead after the first `IDENT`. A leading
`else` / `switch` keyword is a "reserved / not yet supported" parse
error (no production yet, §4.14).

Inside a `Grid` widget body (M3-Phase 5), a `columns:` / `rows:` member
takes the `grid_track_list_member` rule instead of `property_bind`; the
routing is by enclosing widget type (`Grid`), resolved in
`wasamoc/src/parser.rs`, not by the 2-token lookahead above.

---

## 4. Semantics (M2 Surface and M3 Additions)

### 4.1 `component` declaration

```
component <Name> inherits <Base> { … }
```

Declares a named UI component. `<Base>` is stored as a string; no base-type validation
is performed in M1.

Each `.ui` file contains exactly **one** top-level `component` declaration.
Multiple components per file are M2 scope.

**Window host attributes (M3-Phase 6).** The component-level `title:`,
`backdrop:`, and `theme:` attributes on a `Window`-derived component
belong to the **host surface**, not to the content root widget. In
textual IR they are emitted as `host prop` entries on the component
(§8.3) and are never stored as `prop` entries on the root `node Window`.

`title:` is a static string literal that **reaches the native window
title bar** — the loader passes it through to window creation in place
of the default title. An absent or empty `title:` falls back to the
default window title; a non-string `title:` is a `wasamoc check` error
(and `WASAMO_ERR_IR_MALFORMED` at the loader, §8.11). `backdrop:` and
`theme:` are catalogued static host attributes this phase and lower to
the same host surface. Dynamic host attributes are rejected in M3-Phase
6; the first dynamic Window host attribute opens the host-binding seam
additively.

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

Associates a value with a named property. M1 treated all bindings as
static construction-time values. M2 added reactive re-evaluation for
state-backed bindings; M3-Phase 1 adds the bool cases described below.
The right-hand side is an expression position, not a template
interpolation position: state references are written directly
(`enabled: ready`), without an additional interpolation or embedding
wrapper around the expression. String interpolation remains confined
to string literals (see §2.4).

`<expr>` may be a `BOOL_LIT` (`true` / `false`) or an identifier that
resolves to a `bool`-typed `state` declaration (M3-Phase 1) when the
target property is itself `bool`-typed. `wasamoc check` validates the
LHS/RHS type pair against the widget-property catalog (see §4.8) and
the state-name → declared-type table built from `state_decl`s; type
mismatches such as `enabled: 1` (i32 RHS into `bool` target) or
`text: ready` (`bool` source into `string` target) are
compile-time errors.

### 4.4 Widget declaration

```
<WidgetType> { … }
```

Declares a child widget. Widget type names are PascalCase identifiers.
`wasamoc check` validates the type name against the widget registry below:

| Widget name | Description              |
|-------------|--------------------------|
| `VStack`    | Vertical stack container |
| `HStack`    | Horizontal stack container |
| `Text`      | Text display             |
| `Button`    | Clickable button         |
| `Rectangle` | Solid rectangle          |
| `Box`       | Layout container with optional `aspect` / `fill` (M3-Phase 2; see §4.9) |
| `WrapPanel` | Wrapping layout container (M3-Phase 3; see §4.10) |
| `ScrollView` | Vertical scroll viewport with exactly one content child (M3-Phase 4; see §4.11) |
| `Grid`      | 2D layout container with declared track lists per axis; children carry placement via a `Cell` wrapper or direct `slot.*` keys (M3-Phase 5; placement surface M3-Phase 7b; see §4.12 / §4.16) |
| `ZStack`    | Overlay layout container; children overlap and paint back-to-front in document order; per-child overlay alignment via `slot.*` (M3-Phase 6; placement surface M3-Phase 7b; see §4.13 / §4.16) |
| `ToggleButton` | Button carrying a persistent selected / `checked` boolean state (M3-Phase 8; see §4.17) |

`Cell` is **not** a free-standing widget registry entry. It is a
Grid-specific child wrapper construct (one content child per `Cell`,
carrying explicit placement / span / alignment metadata) consumed by
Grid's lowering; `Cell` outside a `Grid` parent is rejected at
`wasamoc check`. It is retained as a grouped convenience over the
parent-interpreted `slot.*` placement model — a Grid child may be
authored as a `Cell` wrapper *or* with direct `slot.*` keys (§4.16).
See §4.12.

`Rectangle`, `Text`, `Button`, and `ToggleButton` admit **no widget
children**. The checker and runtime loader both reject such a child;
their authored content is carried by their own properties. The remaining
registry entries are containers whose child-count and placement rules are
defined in their respective sections.

`if` is **not** a widget registry entry either. It is a **structural
control-flow construct** (the first member of Wasamo's structural
rendering model), not a widget — it materialises no widget of its own;
it makes its body subtree present or absent. It is defined in §4.14,
not here. (This mirrors the `Cell` treatment: a construct the toolchain
*interprets* rather than *renders* is kept out of the registry and
specified in its own section, with only a pointer from here.)

Unknown widget type names produce a warning (not an error) in M1,
to allow forward-compatibility with user-defined components.

### 4.5 Signal handler

```
<signal_name> => { <statements> }
```

Attaches a handler to a named signal. The body is parsed for **structural correctness only**
(balanced braces, valid statement syntax). No type-checking or name resolution is performed
inside `{ }` in M1.

The signals defined by this specification are `clicked`, `dismiss`, and
`key-down("<key>")`; their admission rules are in §4.19. `clicked` is
admitted on **any** widget, not only on the Button family. Semantics and
diagnostic requirements for any other signal name are unspecified.

### 4.6 Expressions

| Expression form    | AST representation                          |
|--------------------|---------------------------------------------|
| `"…"` string       | `Expr::StringLit(String)` — raw content     |
| `42` integer       | `Expr::IntLit(i64)`                         |
| `3.14` float       | `Expr::FloatLit(f64)`                       |
| `true` / `false`   | `Expr::BoolLit(bool)` (M3-Phase 1)          |
| `12px` measurement | `Expr::Measurement { value: f64, unit: Unit }` |
| `mica` identifier  | `Expr::Ident(String)` — no resolution       |

**Handler-side expressions over `bool` (M3-Phase 1).** Inside a
`signal_handler` body, an assignment whose right-hand side is a
`BOOL_LIT` (`true` / `false`) or an identifier resolving to a
`bool`-typed `state` declaration is well-typed, e.g.
`ready = false;`. This lowers to a `HandlerExpr::Assign` with a
`BoolLit` or `BoolPropRead` RHS (see §8.9). Compound assignment
(`+=`, `-=`, `*=`, `/=`) is **not** defined over `bool` and is
rejected by `wasamoc check`.

**Condition expressions (M3-Phase 6).** The condition of an `if` block
(§4.14) is an expression position with the **same narrow bool-expr** as
`Button.enabled`: a `BOOL_LIT` or an identifier resolving to a
`bool`-typed `state`. It admits **no operators** — `!ready`,
comparisons, and logical operators are not in the `expr` grammar and are
rejected in the condition with a diagnostic pointing at the deferred,
uniform expression-grammar extension (it grows across all `expr`
positions at once, not condition-only). See §4.14.

**Loop-local binder reads (M3-Phase 7).** Inside a `for` body (§4.15),
an identifier in a property-binding or interpolation expression
position may resolve to a **loop-local binder** declared by the
enclosing iteration header — the first names in the DSL that do not
resolve to a component `state`. Binder reads are admitted in those
expression positions within the body's widget subtree, and — since
M4-Phase 2 — in **handler position** inside the same body (§4.19). They
are not admitted in `if` conditions (whose identifiers remain
state-only) or outside the body. See §4.15.

**Collection-valued expressions (M3-Phase 7).** The expression grammar
gains its first collection-valued forms — the pure `append` /
`drop-last` method-call expressions and the static collection literal —
admitted **type-driven, not positional**: a collection-valued
expression is valid only where a collection value is expected, and the
only such author-reachable position this phase is the
collection-assignment RHS (§4.15). State defaults stay literal-only;
the `for` header takes a bare state name. Operators remain absent from
every `expr` position.

### 4.7 State declarations (M2 surface; bool added in M3-Phase 1; collections added in M3-Phase 7)

```
state <name>: <state_type> = <default>
```

Declares a per-component reactive `Signal<T>` whose value is owned by
the `.ui` source. `state` declarations are a
component-level member, parallel to `in-out property`. Multiple
`state` declarations may appear in any order; names share a flat
namespace within the component.

Supported `state_type`s:

| `state_type` | Runtime store                            |
|--------------|------------------------------------------|
| `i32`        | `Signal<i32>` — integer reactive value   |
| `string`     | `Signal<String>` — string reactive value |
| `bool`       | `Signal<bool>` — bool reactive value (M3-Phase 1) |
| `i32[]`      | `Signal<Vec<i32>>` — whole-value integer collection (M3-Phase 7) |
| `string[]`   | `Signal<Vec<String>>` — whole-value string collection (M3-Phase 7) |
| `bool[]`     | `Signal<Vec<bool>>` — whole-value bool collection (M3-Phase 7) |

The `<default>` expression must be a literal of the matching type:
`INT_LIT` for `i32`, `STRING_LIT` for `string`, `BOOL_LIT` for
`bool`. `wasamoc check` rejects mismatches (e.g.
`state ready: bool = 0` or `state ready: bool = "false"`) as
compile-time errors with line/column.

Example:

```
state count: i32 = 0
state label: string = "Click me"
state ready: bool = true
```

**Collection state types (M3-Phase 7).** The three collection types are
the postfix-array forms of the existing scalars; elements are
**homogeneous and scalar-only**. The default value is a **collection
literal**: `[` scalar literals, comma-separated, possibly empty `]`.
Each element literal must match the declared element type; the empty
literal `[]` types itself from the declaration. Element expressions —
identifiers, operators, nested lists — are rejected: collection-literal
elements must be scalar literals (collection expressions are a recorded
deferral, §4.15). Nested collection types (`i32[][]`) are rejected with
a named diagnostic. `f64[]` is deferred until a concrete
`f64`-element case arrives; it is an additive fourth element type, not
a shape change.

```
state thumbs: i32[] = [101, 102, 103]
state captions: string[] = []
state flags: bool[] = [true, false]
```

A collection `state` is one **whole-value** reactive signal: any change
to the collection — the authored whole-value assignments of §4.15 —
replaces the whole value and marks the one signal dirty. There is no
per-element signal and no element identity beyond position (§4.15,
identity baseline). Collection values are value-semantic; a collection
assignment whose new value equals the current value performs no dirty
propagation (§4.15).

State declarations lower to the IR `state_decl` form (§8.4).

### 4.8 Widget property catalog (M3-Phase 1)

Per-widget typed property entries that may be bound (`prop: <expr>`
or reactively from a `state` declaration). M2 widget properties
remain bindable through the M2 string-baked path; the entries below
are the ones whose declared type is checked at `wasamoc check` and
dispatched through a per-type binding writer at the runtime loader.

| Widget | Property  | Type   | Default | Notes |
|--------|-----------|--------|---------|-------|
| `Button` | `enabled` | `bool` | `true`  | M3-Phase 1; see contract below |
| `ToggleButton` | `checked` | `bool` | `false` | M3-Phase 8; one-way controlled — see §4.17 |
| `ToggleButton` | `enabled` | `bool` | `true`  | M3-Phase 8; same contract as `Button.enabled` above |

**`Button.enabled` Phase 1 contract.** When bound to `false`:

- The button suppresses click-handler dispatch (no host callback, no
  inline `clicked` handler invocation, no `enqueue_signal("clicked", …)`).
- Hover / press visual transitions are frozen; the background paints a
  flat disabled grey directly (no `ColorKeyFrameAnimation` runs).
- The layout slot is **preserved** — the button still measures and
  arranges identically to its enabled form; there is no
  `display: none` semantics.
- The button remains a **hit-test target**: it occludes whatever is
  beneath it, and a click on it does not reach a lower sibling (§4.19).
  Having dispatched nothing, it also does not stop propagation, so the
  event continues to its ancestors.
- It is **not a focus stop**: traversal skips it, so Tab cannot reach it
  (§4.19). Button keyboard activation is not part of the current widget
  surface.

**Explicitly deferred to later milestones.** AccessKit / `aria-disabled`
accessibility tree state, and hover and focus visual variations for the
disabled state. M5 (accessibility, theming) owns those; the contract
above is structured to be additive under that widening, not superseded
by it.

### 4.9 Box layout primitive (M3-Phase 2)

**Phase status:** M3-Phase 2 closed; implementation-synced.

`Box` is a layout container that admits **zero or one child**.
Multi-child overlap is ZStack's responsibility
(M3-Phase 6); a Box declared with two or more children
is rejected at compile time by `wasamoc check` **and** independently
rejected by the runtime IR loader (`wasamo-runtime/src/ir_loader.rs`)
at IR-load time. The two rejection gates are required because
`wasamo_load_ui`'s memory-IR entry point does not pass through
`wasamoc`; the runtime gate is the last line of defence for the
spec invariant.

#### Attributes

| Attribute | Surface form                | Bindable in Phase 2 | Default                            |
|-----------|-----------------------------|---------------------|------------------------------------|
| `aspect`  | `<num>:<den>` (`RATIO_LIT`) | No                  | absent (no aspect constraint)      |
| `fill`    | `#RRGGBB` or `#RRGGBBAA` (`COLOR_LIT`) | No       | absent (transparent rectangle)     |

Both attributes are constant-only in M3-Phase 2;
`wasamoc check` rejects any non-literal RHS for `aspect` or `fill`
— including state-backed bindings such as `aspect: <state-ident>`
or `fill: <state-ident>` — with a diagnostic naming the rejected
attribute. Symmetrically, `RATIO_LIT` and `COLOR_LIT` literals are
only accepted as the RHS of `Box.aspect` and `Box.fill` respectively;
`wasamoc check` rejects them in any other syntactic position (a
`state` default, a handler RHS, or a non-Box property assignment)
with a diagnostic naming the offending position. The first phase
to need a reactive aspect or fill opens the per-type writer seam
triple at that point — Phase 2's literal plumbing is
forward-compatible and is not revised, only extended.

**`aspect` literal form.** `<num>:<den>` with both sides positive
integer literals. `wasamoc check` rejects `num <= 0` or `den <= 0`
at compile time; NaN and infinity are structurally unreachable.
The ratio is preserved exactly as a pair of `i32`s through `wasamoc`
lowering and IR; the projection onto `f32` parent bounds is the
only floating-point step in the measure-arrange pass below.

**`fill` literal form.** `#RRGGBB` carries three 8-bit channels with
alpha implicitly `0xFF`; `#RRGGBBAA` carries the alpha channel
explicitly. The value-layer admits alpha so that the structural
scrim use case (`Box { fill: #00000080 }`) is expressible. The
M3 *styling* layer does not expose alpha-styling controls beyond
the literal hex form — theming, named palette, and dynamic alpha
adjustment all remain M4+ work.

#### Child layout contract

When Box has zero children, it still produces a sized rectangle
filled with `fill` (or transparent if `fill` is absent). The
`aspect`-derived rectangle is the structural support for the scrim
shape (`Box { fill: <color> }`) and the placeholder-shape subsection
below.

When Box has a single child:

- **Measure pass.** Box's resolved outer bounds are passed through
  to the child as the child's measure constraint, unchanged.
- **Alignment.** The child is centred horizontally and vertically
  inside Box. M3-Phase 2 provides no per-child alignment override
  attribute; later phases that need other alignments open their own
  DD without revising Box's default.
- **Overflow.** A child measuring larger than Box bounds is
  visually clipped to Box's rectangle. Box's layout slot does not
  grow to accommodate an oversized child. This is consistent with
  Phase 4 ScrollView's separate scrollable-viewport surface
  (ScrollView's contribution is the *viewport*, not the clipping
  primitive — Box clips already).

#### Aspect-constraint measure-arrange

When Box carries `aspect`, its outer bounds are resolved from parent
bounds via **inscribed fit**: Box's resolved rectangle is the largest
aspect-correct rectangle that fits inside the parent. Given parent
width `W` and height `H` and `aspect: num:den`, the branch is
selected by integer comparison `(W as f64) * (den as f64)` vs
`(H as f64) * (num as f64)`; once the branch is chosen the derived
axis is computed in `f32`. No pixel-snapping in Phase 2: the derived
extent is a DIP value that need not be a whole number of device pixels
([§1 *Units and the layout coordinate system*](#units-and-the-layout-coordinate-system)).

Edge cases:

- **Unbounded parent on one axis** — the unbounded axis derives
  from the bounded axis × aspect (bounded-axis-wins). The Box has a
  defined intrinsic size in intrinsic-sizing contexts such as Phase
  3 WrapPanel-of-Boxes and Phase 4 ScrollView's inner measure.
- **Unbounded parent on both axes** — a layout-time runtime error.
  The diagnostic names the missing input as *"aspect with no bounded
  parent axis"*. NaN / silent-zero outcomes are structurally
  excluded.
- **No aspect, no children, unbounded parent on both axes** — the
  same layout-time runtime error class. The diagnostic wording
  names the missing input as *"neither aspect nor parent bounds"*.
  A scrim-only Box in a fully-unbounded context is an author error
  worth surfacing.
- **No aspect, single child** — Box shrink-to-fits the child's
  intrinsic size on each axis where the parent is bounded; the Box
  collapses to zero on each unbounded axis.

`width` / `height` are **not** in the M3-Phase 2 DSL surface, so the
"explicit dimensions vs `aspect`" rule lands as spec text only:
when those attributes are introduced by a later phase, explicit
dimensions win and `aspect` becomes informational (with a
`wasamoc check` warning landed by that phase, not by Phase 2).

#### Image placeholder pattern (M3)

The Box + Text-child shape is the **normative** M3 substitute for the
deferred Image widget surface. Phase 3 (WrapPanel of thumbnails) and
Phase 6 (ZStack lightbox) consume this pattern verbatim.

```
Box {
    aspect: <ratio>
    fill: <color>
    Text { text: <label> }
}
```

Examples from the M3 gallery:

```
Box {
    aspect: 1:1
    fill: #cccccc
    Text { text: "Photo 12" }
}
Box {
    aspect: 16:9
    fill: #cccccc
    Text { text: "photo-23.jpg" }
}
```

The scrim shape, used in compositions Phase 6 ZStack assembles
(a semi-transparent overlay over a lightbox), is:

```
Box { fill: #00000080 }
```

> **Notation note (M3-Phase 4 close).** The examples above use the
> parser-accepted multi-line member-per-line form. In the current
> implementation `;` is a statement terminator inside handler blocks
> (§4.5 / §3 grammar) and is **not** accepted as a widget/member
> separator; the grammar uses newlines between members. Accepting
> `;` as an optional member separator remains a **post-Phase-4 open
> question**; the multi-line presentation above is the parser-accurate
> form and does not foreclose that future extension.

When an `<Image>` widget lands (M4 or later), it supersedes this
pattern; this subsection then gains a "Superseded by `<Image>`
widget" header. The Box + Text shape remains as a back-compat
form for pre-Image authors, and Phase 3 / Phase 6 spec citations
to this subsection remain valid (the cited pattern is still
spec-recorded; downstream phases migrate to `<Image>` syntactically
when it ships).

### 4.10 WrapPanel layout primitive (M3-Phase 3)

**Phase status:** M3-Phase 3 closed; implementation-synced.

`WrapPanel` is a layout container that places its children along a
**main axis** in document order and breaks onto a new **line** when
the next child does not fit within the parent-supplied main-axis
bound. In M3-Phase 3 the main axis is **horizontal** unconditionally;
a `vertical` orientation is reserved for a later additive phase
(no `orientation` attribute is exposed in Phase 3).

WrapPanel admits **zero or more children**; the IR loader and
`wasamoc check` impose no child-count restriction. The line-formation
algorithm is two-stage measure-arrange (the first M3 measure-arrange
primitive whose outer cross-axis size depends on its children).

#### Sizing mental model

WrapPanel sizing follows four facts:

1. **Main-axis intrinsic measure.** WrapPanel measures each child
   against an **unbounded main-axis constraint**; line membership is
   decided by the child's reported main-axis intrinsic extent.
2. **Cross-axis bound source.** Each child receives a cross-axis
   bound from one of two sources — `item-cross-size` when set on the
   WrapPanel, or the parent's cross-axis constraint passed through
   unchanged when `item-cross-size` is unset. WrapPanel does **not**
   synthesise a cross-axis bound out of nowhere.
3. **Aspect-only Box requires a cross-axis bound.** A
   `Box { aspect: <ratio> }` child has no intrinsic size of its own.
   Without a finite cross-axis bound (either from `item-cross-size`
   or from a bounded parent), Phase 2's
   `LayoutError::BoxAspectUnboundedBoth` fires at runtime.
4. **No wrap boundary ⇒ one-line flow.** When the parent supplies no
   main-axis bound, there is no boundary against which to break
   lines; all children flow on a single line in document order.

**Ecosystem contrast.** `item-cross-size` has no clean analogue in
WPF, Compose, or CSS; readers arriving from those frameworks should
note the following:

- **WPF `ItemHeight` / `ItemWidth`** are orientation-coupled fixed
  cell extents. `item-cross-size` is orientation-neutral and
  conceptually a **bound passed to child measure**, not a cell
  extent the child is laid into. In the uniform case the *visible*
  outcome matches WPF (the line's cross-axis extent equals
  `item-cross-size`), but the underlying primitive differs.
- **Flutter / Jetpack Compose natural child size.** WrapPanel's
  default behaviour (when `item-cross-size` is unset) is *closer*
  to natural sizing — parent constraints pass through and children
  measure naturally. Children with no natural cross-axis size (the
  aspect-only Box) are supported by setting `item-cross-size`, not
  by a "compute natural size" fallback.
- **CSS `gap`.** Applies to container *spacing* between items, not
  to item *sizing*. `item-cross-size` is **not** a `gap` analogue;
  `item-spacing` / `line-spacing` are.

#### Attributes

| Attribute         | Surface form     | Bindable in Phase 3 | Default            |
|-------------------|------------------|---------------------|--------------------|
| `item-cross-size` | `<i32>` literal  | No                  | absent (parent cross-axis constraint passes through) |
| `item-spacing`    | `<i32>` literal  | No                  | `0` (touching items on the main axis) |
| `line-spacing`    | `<i32>` literal  | No                  | `0` (touching lines on the cross axis) |

All three attributes are constant-only integer literals in M3-Phase 3.
The values are extents in DIP
([§1 *Units and the layout coordinate system*](#units-and-the-layout-coordinate-system));
they reuse the existing `i32` literal plumbing from M2 (no new
`IrType`, no new `IrLiteral` variant, no new `PropertyValue` variant).

**Non-negative integer range.** All three attributes admit
**non-negative** integer values. `wasamoc check` rejects a negative
literal at compile time, naming the rejected attribute; the runtime
IR loader's `validate()` independently rejects negative IR
(two-gate defense, mirroring Phase 2's `RATIO_LIT`
rejection). Both gates are required because `wasamo_load_ui`'s
memory-IR entry point does not pass through `wasamoc`.

**Zero is a valid setting** for all three attributes, not a silent
footgun. `item-spacing: 0` / `line-spacing: 0` is the default —
touching items / lines, visible by construction. `item-cross-size: 0`
is an *author-requested degenerate layout* — each line collapses to
zero cross-axis extent (line count is still computed; no thumbnails
are visible). This is distinct from the `LayoutError` cases below;
the absence of any bound source is the error, not a written-out
zero.

The first phase to need a reactive WrapPanel attribute admits
binding for that attribute at that point. Phase 3 reuses the
existing `i32` literal plumbing; a future bindable phase can
reuse the M2 string-baked path that `IrType::I32` properties
currently dispatch to (`register_binding` +
`widget_write_property`), or open a typed-`i32` evaluator/writer
pair if that phase warrants it. Phase 3's constant-only `i32`
surface is forward-compatible and is extended, not revised.

#### Measure-arrange algorithm

WrapPanel's measure-arrange pass operates on pure data
(`wasamo-runtime/src/layout.rs`); the algorithm is Win32/WinRT-free.

**Bounded main-axis parent (happy path).** Children are measured
against an unbounded main-axis constraint and a cross-axis
constraint defined by `item-cross-size` (when set) or the parent's
cross-axis constraint (when unset). The line breaker greedily
appends children to the current line in document order. The
acceptance rule is **two-cased**:

- **First child of a line (`line_empty == true`).** The candidate is
  placed unconditionally — no inequality is consulted. The line's
  recorded main-axis extent equals the child's intrinsic main-axis
  extent and may exceed `parent_main_bound`. See *Oversized
  first-child and visible overflow* below.
- **Subsequent children of the same line (`line_empty == false`).**
  The candidate is placed on the current line iff

  ```
  current_line_main + item_spacing
    + next_child_main_intrinsic
    <= parent_main_bound
  ```

  When the inequality fails, a new line starts and the candidate
  becomes the first child of that new line (the unconditional-
  placement rule then applies).

No trailing `item_spacing` accrues after the last child of a line.

**Cross-axis line sizing.** Depends on whether `item-cross-size` is
set:

- **When set.** Each child receives `item-cross-size` as its
  cross-axis bound. The line's cross-axis extent is exactly
  `item-cross-size`. A `Box { aspect: num:den }` child derives its
  main-axis extent as `item-cross-size × num / den` per §4.9's
  aspect-constraint rule (bounded-axis-wins). Smaller children
  align centred within the line; larger children clip against the
  per-line cross-axis bound consistent with Box's overflow rule.
- **When unset.** Each child receives the parent's cross-axis
  constraint as its cross-axis bound (the WrapPanel-level
  passthrough). The line's cross-axis extent is the max of the
  children's reported cross-axis sizes. A `Box { aspect: num:den }`
  child derives its main-axis extent as `parent_cross × num / den`,
  which is the "huge thumbnail" path covered in *Common pitfalls*
  below.

**Per-line cross-axis item alignment.** Heterogeneous-cross-axis
line members are **centred** within the line (no per-child override
attribute in Phase 3, consistent with §4.9 Box's centred default).

**WrapPanel outer cross-axis size.** Sum of line cross-axis extents
plus `line_spacing × (line_count − 1)`. No trailing margin after
the last line.

**WrapPanel outer main-axis size.** Equals `parent_main_bound` —
**unconditionally**, even when one or more lines contain an
oversized first child whose intrinsic extent exceeds
`parent_main_bound`. The WrapPanel does not grow upward to
accommodate oversized children (see *Oversized first-child and
visible overflow* below).

**Unbounded main-axis parent.** When the parent supplies no
main-axis bound — the realistic context is an outer intrinsic-sizing
measure pass — the line breaker has no boundary to compare against
and WrapPanel **degenerates to one-line flow**: every child sits on
a single line in document order. The line's cross-axis extent
follows the same per-line rule above (fixed item cross-size or passthrough).
The WrapPanel's outer main-axis size is the cumulative content
extent (sum of children's intrinsic main-axis extents plus
`item_spacing × (n − 1)`). This branch raises **no new
`LayoutError`**: the one-line outcome is visible, not silent.
(Phase 4 ScrollView is *not* an example of this branch — ScrollView
bounds the main axis and unbounds the cross axis, so the bounded
happy path applies inside a vertical-scroll ScrollView.)

**Unbounded cross-axis parent (with `item-cross-size` unset).**
Each child receives an unbounded cross-axis constraint. A
`Box { aspect: <ratio> }` child in this state has both axes
unbounded and hits Phase 2's existing
`LayoutError::BoxAspectUnboundedBoth` — surfaced with the Box's IR
location. WrapPanel does not add a layered diagnostic variant in
Phase 3; the author must set `item-cross-size` or wrap the
WrapPanel in a sized parent.

**Rounding contract.** Inherits §4.9's discipline: parent bounds
enter as `f32`; the integer attributes (`item-cross-size`,
`item-spacing`, `line-spacing`) are promoted to `f32` for the
overflow inequality and the cross-axis sum; child intrinsic sizes
are `f32` from the layout engine. No pixel-snapping in Phase 3.

**`LayoutError` surface.** Phase 3 introduces **no new
`LayoutError` variant**. The unbounded-main-axis branch is one-line
flow (not an error); the unbounded-cross-axis-with-aspect-child case
fires Phase 2's existing `LayoutError::BoxAspectUnboundedBoth`. The
ABI / host-visible surface is unchanged in Phase 3 (no new
`WASAMO_LAYOUT_ERROR_*` extension; layout error class remains
host-internal per the Phase 2 precedent).

#### Oversized first-child and visible overflow

When the first child of a line has an intrinsic main-axis extent
that already exceeds `parent_main_bound`, the line breaker still
places it on that line (unconditional first-child placement above).
The line's recorded extent then exceeds the parent's main-axis
bound, and the *visible* rendering proceeds as follows:

- **WrapPanel's outer main-axis size is `parent_main_bound`** —
  the WrapPanel does not grow upward to its content. Its
  parent-facing rectangle is unchanged from the no-oversized case.
- **The oversized child paints at its measured extent**, with its
  main-axis-end edge extending past WrapPanel's outer rectangle.
- **WrapPanel installs no clip surface** on the oversized line.
  Whether visible clipping occurs is the responsibility of an
  enclosing parent that supplies one. Phase 4 ScrollView clips by
  definition; a plain HStack / VStack / parent component does not
  clip and visible overflow remains visible.

This matches the WPF / Slint / Compose "overflow is visible unless
someone clips" convention and avoids propagating a parent-bound
violation up the layout tree.

#### Common pitfalls

1. **Aspect-only Box children without `item-cross-size`.** When a
   WrapPanel directly contains one or more `Box { aspect: <ratio> … }`
   children and `item-cross-size` is unset on the WrapPanel, each
   child inherits the parent's cross-axis constraint as its
   cross-axis bound. In an 800×600 window with no other cross-axis-
   bounding container, a 1:1 thumbnail becomes ~600×600 — a single
   huge thumbnail with subsequent children pushed onto new lines.
   `wasamoc check` emits a **warning** on this pattern (the warning
   does not classify all possible child shapes, only the known
   aspect-only-Box footgun); the fix is to set `item-cross-size`
   on the WrapPanel.

2. **Oversized children paint past the WrapPanel rectangle.**
   See *Oversized first-child and visible overflow* above —
   WrapPanel does not clip the overflow itself; an enclosing
   ScrollView or other clipping parent is required to truncate
   the on-screen rendering.

### 4.11 ScrollView layout primitive (M3-Phase 4)

**Phase status:** M3-Phase 4 closed; implementation-synced.

`ScrollView` is a vertical-only layout primitive that exposes a
bounded **viewport** over one scrollable **content** child. It clips at
the viewport rectangle and translates the content upward by the
clamped `offset-y` value. The widget itself does not synthesize
content, scrollbars, wheel handling, drag handling, or author-declared
viewport dimensions in M3-Phase 4.

#### Sizing mental model

ScrollView sizing follows five facts:

1. **Viewport size comes from parent.** ScrollView fills its parent
   slot on both axes; there is no `viewport-width`, `viewport-height`,
   `width`, or `height` attribute on ScrollView in Phase 4. To control
   viewport size, the parent's slot must be sized through the parent's
   own attribute or layout role.
2. **Content measures with viewport-bounded cross axis and unbounded
   scroll axis.** The single content child receives the viewport width
   as a horizontal bound and an unbounded vertical constraint. Content
   along the scroll axis may therefore be arbitrarily tall and is
   scrollable when it exceeds the viewport; content shorter than the
   viewport is anchored at the top and does not scroll.
3. **Content offset is clamped to `[0, max(0, content_height -
   viewport_height)]`.** Out-of-range bound values (e.g. a `scroll_y`
   state that runs past the scrollable range) are silently clamped on
   every layout pass. The bound state is read-only-bound under the
   Phase 4 default (see *Attributes* below), so the source state's
   written value and the applied offset may diverge — the author
   observes the displayed scroll position, not the bound value, as
   ground truth.
4. **The clip is owned by ScrollView, not by the content.** Content
   widgets remain unclipped; only the ScrollView Visual installs a
   clip surface. Composing two ScrollViews around the same content
   stacks two clips; wrapping ScrollView around an HStack around
   content does not install an HStack-level clip. The Visual-layer
   shape is given normatively in *Visual-layer contract* below.
5. **`offset-y` is the Phase 4 external control surface, not the only
   future scroll model.** The bindable `offset-y` attribute is how
   Phase 4 exposes scroll position to author code; it is not a
   commitment that state-driven offset is the canonical way to scroll.
   Input-driven scrolling (wheel, drag, keyboard) and scrollbar-driven
   scrolling are M4 or later surfaces and land additively without
   redefining `offset-y`.

**Ecosystem contrast.** Readers arriving from WPF, CSS, or SwiftUI
should map Phase 4 ScrollView to "one child in a clipped viewport",
not to a full scroll-control stack:

- **WPF `ScrollViewer`.** Carries scrollbar visibility attributes
  and a built-in scrollbar widget. Wasamo's Phase 4 conceptual
  primitive (clip + offset + measure-arrange) matches, but the
  surface is narrower: no scrollbar, viewport-from-parent, and the
  scroll position is exposed as a bindable `offset-y` attribute.
- **CSS `overflow: scroll`.** The viewport-plus-clipped-content
  shape applies, but Phase 4 is not a general overflow style
  property — it is a concrete widget kind with exactly one content
  child. Phase 4 ships no scrollbar in any state and no input-driven
  internal scrolling, and content size does not back-propagate to
  ScrollView's outer size (which stays at viewport).
- **SwiftUI `ScrollView`.** Carries the viewport-from-parent default,
  axis selection, and gesture-driven offset as the familiar
  associations. Wasamo's Phase 4 hardcodes vertical, exposes scroll
  position as the bindable `offset-y` attribute against a bare state
  identifier (e.g. `offset-y: scroll_y` with `state scroll_y: i32 = 0`
  declared per §4.7), and defers gesture / wheel input to M4 or later.
  The `.scrollPosition($state)` SwiftUI surface is conceptually
  closest to the future in-out / write-back direction, which Phase 4
  defers.

#### Children

ScrollView admits **exactly one child**. `wasamoc check` rejects
zero-child and multi-child ScrollView declarations; the runtime IR
loader's `validate()` independently rejects malformed memory IR with
0 or more than 1 child. The runtime rejection uses the existing
`WASAMO_ERR_IR_MALFORMED` surface.

A **direct conditional member** under ScrollView — `ScrollView { if c { …
} }`, or an `if` beside the content child — is rejected at both gates: a
conditional's presence is dynamic, so it cannot satisfy the
exactly-one-content-child contract. A conditional-only body
(`ScrollView { if c { … } }`) materializes 0 or 1 children, and a
conditional beside the content child (`ScrollView { Content  if c { … } }`)
materializes 1 or 2 — neither is guaranteed-exactly-one. Wrap the
conditional inside the single content widget instead
(`ScrollView { Box { if c { … } } }`). This is symmetric with the `Cell`
direct-conditional rejection (§4.12); a conditionally-empty ScrollView is a
deferred future direction, not a supported Phase 6 shape.

#### Attributes

| Attribute | Surface form | Bindable in Phase 4 | Default |
|---|---|---|---|
| `offset-y` | `<i32>` literal or a bare state identifier such as `scroll_y` (declared as `state scroll_y: i32 = 0` per §4.7) | Read-only binding | `0` |

`offset-y` is a signed integer offset in DIP
([§1 *Units and the layout coordinate system*](#units-and-the-layout-coordinate-system)).
Per §4.3, the bound form is a bare identifier RHS (for example
`offset-y: scroll_y`), not a `\{…}` interpolation — string interpolation
is confined to string literals (see §2.4). Literal values and bound
`i32` state values may be negative or larger than the scrollable range
in source; the runtime clamps the applied offset during layout. Absent
`offset-y` materializes as `0` at the runtime layer.

The attribute reuses the existing `i32` surface: no grammar token, AST
variant, `IrType`, `IrLiteral`, or scalar value type is added in Phase
4. The bindable path reuses the existing i32 reader / binding-effect
machinery. Runtime writes reach ScrollView through a narrow
ScrollView-specific string-to-`i32` parse / write bridge; the general
typed-`i32` writer pair remains deferred to M4 or later.

Unknown ScrollView attributes are rejected in Phase 4. In particular,
`viewport-width`, `viewport-height`, `scroll-axis`, and `padding` are
out of scope.

#### Measure-arrange algorithm

ScrollView's measure-arrange pass operates on pure data
(`wasamo-runtime/src/layout.rs`); the algorithm is Win32/WinRT-free.

**Bounded vertical parent (happy path).** ScrollView's viewport size is
the parent-supplied bound. Its single content child is measured with:

```
content_width_constraint  = viewport_width
content_height_constraint = unbounded
```

ScrollView's outer arranged size equals the viewport size regardless
of the measured content height. The content is arranged at the
top-leading origin of the ScrollView content space; ScrollView does
not center content smaller than the viewport.

**Offset clamp.** Let:

```
max_offset_y = max(0, content_height - viewport_height)
applied_y    = clamp(offset_y, 0, max_offset_y)
```

The visible content translation is `(0, -applied_y)`. Negative
`offset-y` clamps to `0`; values larger than `max_offset_y` clamp to
`max_offset_y`; content smaller than or equal to the viewport also
clamps to `0`.

**Unbounded vertical parent.** A ScrollView whose scroll axis is
unbounded has no viewport boundary to scroll within. Layout fails with
`LayoutError::ScrollViewUnboundedAxis`. This error is runtime-internal
in Phase 4; no new C ABI layout-error tag is added.

**Rounding contract.** `offset-y` is an `i32` value promoted to `f32`
for layout arithmetic and Visual offset writes. Phase 4 introduces no
pixel-snapping rule.

#### Visual-layer contract

ScrollView owns two Visual-layer surfaces:

- The outer ScrollView Visual carries the viewport size and installs
  `Visual.Clip = InsetClip { 0, 0, 0, 0 }`.
- A ScrollView-owned intermediate content Visual sits between the
  outer Visual and the single content child's widget Visual. It carries
  the scroll translation:

  ```
  Visual.Offset = (0, -applied_y, 0)
  ```

The child widget's own Visual continues to receive its layout-derived
parent-relative offset through the normal `sync_visuals()` path. The
ScrollView-owned intermediate content Visual and the child Visual do
not carry the viewport clip.

#### Common pitfalls

1. **Expecting ScrollView to size itself.** ScrollView fills the slot
   its parent/root allocates. Use an enclosing layout shape to control
   the viewport; `viewport-*` attributes do not exist in Phase 4.
2. **Putting several children directly inside ScrollView.** Wrap them
   explicitly in a layout container. `ScrollView { WrapPanel { ... } }`
   is the canonical Phase 4 gallery composition.
3. **Expecting user input or scrollbar behavior.** Phase 4 demonstrates
   programmatic scrolling through `offset-y`. Wheel, drag, scrollbar,
   in-out write-back, and imperative scroll commands are M4 or later
   surfaces.

### 4.12 Grid layout primitive (M3-Phase 5)

**Phase status:** M3-Phase 5 closed; implementation-synced. The child
placement surface is extended in M3-Phase 7b (closed;
implementation-synced; see §4.16): a Grid child may be authored as a
`Cell` wrapper **or** with direct `slot.*` placement keys, both
expressing the same parent-interpreted placement.

`Grid` is a 2D layout primitive that arranges children across a
declared row × column track matrix. Tracks are declared once on
`Grid` via the `columns:` and `rows:` attributes. Each child carries
explicit `row` / `column` placement, optional `row-span` /
`column-span`, and optional per-cell `h-align` / `v-align` — authored
**either** grouped in a `Cell` wrapper (`Cell { row: … column: … }`)
**or** directly on the child in the `slot.*` namespace (`Box { slot.row:
… slot.column: … }`, §4.16). Both forms express the same
parent-interpreted placement and lower to the same model; a single Grid
child uses one form, never both. Content widgets carry no Grid-specific
*widget* property — placement is parent-interpreted metadata, not a
widget attribute (§4.16).

Grid admits **zero or more children** (each a `Cell` wrapper or a
directly-placed content widget). The minimum valid Grid shape is
`columns.len() >= 1` and `rows.len() >= 1`; a Grid with no children
resolves to its outer rectangle with no drawn cell content.

#### Sizing mental model

Grid sizing follows six facts:

1. **One track list per axis, declared on Grid.** `columns:` and
   `rows:` carry whitespace-separated sequences of track-sizing
   tokens. Track sharing across rows / columns is automatic; there
   is no per-row column-width or per-column row-height surface.
2. **Fixed tracks consume definite space first.** Each
   `<integer>`-px track contributes its declared size — in DIP,
   [§1 *Units and the layout coordinate system*](#units-and-the-layout-coordinate-system)
   — to the axis's resolved extent.
3. **Weighted-star tracks divide remaining bounded space by integer
   weight.** `*` (sugar for `1*`) and `n*` (positive integer in
   `[1, 1024]`) take fractional shares of the bounded space the
   parent allocates after fixed tracks are honoured. `auto` /
   intrinsic and floating-point weights are reserved for a future
   phase (see *Reserved future surface* below).
4. **Each child declares its placement — grouped in a `Cell` or
   directly via `slot.*`.** A child declares `row` + `column`
   (zero-based) and optionally spans cells via `row-span` +
   `column-span` (default `1`), authored either as a `Cell` wrapper's
   bare keys or as direct `slot.*` keys on the child (§4.16). Same-cell
   occupancy — two children whose resolved
   `(row, column, row-span, column-span)` rectangles share any resolved
   cell — is rejected at `wasamoc check` and at runtime `validate()`,
   regardless of which form authored each. Intentional overlay is not
   Grid's responsibility; Phase 6 ZStack owns overlay.
5. **Grid does not grow to fit its content.** On a bounded axis,
   Grid's outer rectangle equals the parent's allocation on that
   axis. Fixed-track sums that exceed the parent's allocation
   produce paint overflow that is clipped at Grid's outer
   rectangle (see *Arrange, overflow, and z-order* below). On an
   unbounded axis with no star tracks, Grid's outer rectangle on
   that axis equals the fixed-track sum.
6. **Star tracks on an unbounded parent axis are an error.** A
   Grid whose star direction meets an unbounded parent constraint
   has no finite space to divide; the layout pass fails with
   `LayoutError::GridUnboundedStarAxis`. This is consistent with
   §4.11 ScrollView's `LayoutError::ScrollViewUnboundedAxis`
   precedent.

**Ecosystem contrast.** Readers arriving from WPF, CSS, or
Compose/SwiftUI grids should note the following:

- **WPF `Grid`.** WPF declares `RowDefinition` /
  `ColumnDefinition` and routes child placement through attached
  `Grid.Row` / `Grid.Column` properties on arbitrary content
  widgets. Wasamo declares tracks the same way conceptually and
  routes child placement either through an explicit `Cell` wrapper
  or through the `slot.*` placement namespace on the child (§4.16) —
  the latter is the closest analogue to WPF's attached properties,
  but neutrally namespaced rather than qualified by the parent type.
  Either way placement is parent-interpreted metadata, kept out of
  the content widget's own property set. Star sizing, spanning, and
  zero-based indexing match WPF.
- **CSS Grid.** Wasamo's Phase 5 Grid borrows the *tracks +
  placed children* shape but ships a deliberately narrower surface:
  fixed pixels and weighted-star only (no `auto`, no `minmax`, no
  `fr` unit, no named lines, no `grid-template-areas`, no
  `auto-flow`, no `gap`). Placement is by explicit
  `(row, column, row-span, column-span)` coordinates, not by line
  names or area names. Same-cell overlap is rejected, not stacked.
- **Jetpack Compose / SwiftUI grids.** Those ecosystems typically
  model adaptive or data-driven grids (`LazyVerticalGrid`,
  `LazyVGrid`). Wasamo's Phase 5 Grid is a **static 2D composition
  primitive**: every `Cell` is explicit in the source, and Grid is
  not an M3 iteration target. (M3's data-driven collection surface
  is WrapPanel + the iteration grammar; see §4.10 and §4.15.) Iteration
  generating `Cell`s is not foreclosed but is post-M3.
- **ZStack / overlay models.** Grid does not provide intentional
  overlay. A Cell whose content paints past the cell rectangle may
  cross into a sibling cell's region — that is governed by the
  document-order paint rule below — but two `Cell`s may not
  deliberately occupy the same resolved cell. Phase 6 ZStack is the
  surface for intentional overlay.

#### Children

Grid admits zero or more children, each authored in one of two
mutually-exclusive forms:

1. **A `Cell` wrapper** — a Grid-owned single-child layout wrapper
   carrying the placement as its own bare keys.
2. **A directly-placed content widget** carrying `slot.*` placement
   keys (§4.16) — the content widget is itself the Grid child, with no
   `Cell`.

**`Cell` wrapper form:**

- Each `Cell` accepts **exactly one content child**. `wasamoc check`
  rejects `Cell { }` (0 children) and `Cell { X Y }` (2+ children).
  This is a `.ui`-source / checker rule on the wrapper: lowering
  normalises the `Cell` to a child-slot placement record (§4.16), so a
  `Cell` node does **not** reach the runtime loader — the loader's
  `Cell`-specific gate is the **stale-form rejection** of a surviving
  `node Cell { … }` IR (§8.5), not a child-count re-check. Authors who
  want multiple widgets in one cell wrap them explicitly
  (`Cell { VStack { Text { } Text { } } }`).
- `Cell` outside a `Grid` parent is rejected at `wasamoc check`. `Cell`
  is not a free-standing widget; it has no meaning outside Grid's
  lowering (and, being normalised away, never appears as a node in the
  loaded IR).
- `Cell` itself does not materialise as a runtime widget Visual.
  The Visual tree contains one Visual for Grid plus one Visual per
  `Cell`'s content child — the existing **1 WidgetNode = 1 Visual**
  convention from §6.5 of `architecture.md` is preserved.

**Direct `slot.*` form:** a content widget placed directly under Grid
(no `Cell`) carries its placement in the `slot.*` namespace (§4.16); the
widget *is* the Grid child and materialises one Visual, the same 1
WidgetNode = 1 Visual convention. The two forms are **mutually exclusive
per child** and are kept unambiguous by two distinct `wasamoc check`
rejects (both operate on the `Cell` wrapper before normalization, so
they are checker-side; the loader's parallel guard is the stale-form
rejection of a surviving `node Cell`, §8.5):

- **mixing reject** — `slot.*` appearing among a `Cell` node's own
  attributes (a `Cell` carries placement through its bare keys, not
  `slot.*`);
- **non-admitting-parent reject** — `slot.*` on a widget *inside* a
  `Cell` (the widget's immediate parent is the `Cell` wrapper, which
  admits no placement of its own).

A direct child of Grid that is neither a `Cell` wrapper nor carries
`slot.*` placement is permitted only under the single-child escape
clause below (it lowers to `(0, 0)`); a multi-child Grid requires every
child to declare placement (in either form).

**No normative canonical form** is declared for Grid this milestone;
this spec writes Grid examples with `Cell` by convention and shows
direct `slot.*` where it illustrates the shared placement surface
(§4.16). The convention is **provisional** (a future pre-1.0 decision
fixes whether a wrapper form is retained) and is not an acceptance
criterion.

#### Attributes

**On `Grid`:**

| Attribute  | Surface form        | Bindable in Phase 5 | Default |
|------------|---------------------|---------------------|---------|
| `columns:` | track-list (see below) | No                  | required |
| `rows:`    | track-list (see below) | No                  | required |

Both `columns:` and `rows:` are required and must declare at least
one track. Grid has no `width` / `height` / `gap` / `auto-flow`
attribute in Phase 5; unknown Grid attributes are rejected at
`wasamoc check`.

A **track-list** is a whitespace-separated sequence of track tokens
parsed by a narrow Grid-specific parser path that does **not** open
a general list / collection grammar; the sequence is terminated by
the existing attribute-termination rule (`;` or newline).

The track tokens admitted in Phase 5:

| Token                      | Meaning                                  | Validation |
|----------------------------|------------------------------------------|------------|
| `<integer>` (positive)     | Fixed track of that width / height in DIP ([§1](#units-and-the-layout-coordinate-system)) | `value >= 1` at `wasamoc check` and `validate()` |
| `*`                        | Weighted-star track with weight `1`      | (passes by construction) |
| `<integer>*` (`1..=1024`)  | Weighted-star track with that integer weight | `1 <= weight <= 1024` at `wasamoc check` and `validate()` |

Phase 5 rejects (with a `wasamoc check` diagnostic naming the
offending shape, and `WASAMO_ERR_IR_MALFORMED` if the malformed
shape reaches `validate()`):

- non-positive fixed values (`0`, `-5`);
- non-positive or out-of-range star weights (`0*`, `-2*`,
  `2048*`);
- `auto` — reserved for a future phase. The diagnostic names it
  as a **reserved-future** token rather than as an unknown
  identifier, so authors who try `auto` see a deferral hint rather
  than a typo hint;
- floating-point fixed values or weights (`1.5`, `1.5*`).

Examples:

```
Grid {
  columns: 180 1* 2*
  rows: 64 1*
  ...
}
```

```
Grid {
  columns: 96 1* 96
  rows: 64 1* 120
  ...
}
```

**Placement keys (in a `Cell`, written bare; directly on a child,
written `slot.*`):** the six placement keys are identical across both
authoring forms — the `Cell` wrapper writes them bare (`row:` /
`column:` / …), a directly-placed child writes them with the `slot.`
prefix (`slot.row:` / `slot.column:` / …, §4.16). Types, defaults, and
ranges are the same in both forms:

| Placement key          | In a `Cell` | Direct on child | Type   | Default                              | Valid range                                | Violations |
|------------------------|-------------|-----------------|--------|--------------------------------------|--------------------------------------------|------------|
| row                    | `row:`         | `slot.row:`         | `i32`  | `0` (single-child Grid only; see below) | `[0, rows.len())`                          | `wasamoc check` + `validate()` reject |
| column                 | `column:`      | `slot.column:`      | `i32`  | `0` (single-child Grid only; see below) | `[0, columns.len())`                       | `wasamoc check` + `validate()` reject |
| row span               | `row-span:`    | `slot.row-span:`    | `i32`  | `1`                                  | `[1, rows.len() - row]`                    | `wasamoc check` + `validate()` reject |
| column span            | `column-span:` | `slot.column-span:` | `i32`  | `1`                                  | `[1, columns.len() - column]`              | `wasamoc check` + `validate()` reject |
| horizontal alignment   | `h-align:`     | `slot.h-align:`     | ident  | `stretch`                            | `{ start, center, end, stretch }`          | `wasamoc check` + `validate()` reject |
| vertical alignment     | `v-align:`     | `slot.v-align:`     | ident  | `stretch`                            | `{ start, center, end, stretch }`          | `wasamoc check` + `validate()` reject |

Unknown `Cell` attributes, and unknown `slot.*` keys on a Grid child,
are rejected at `wasamoc check`. Mixing the two forms on one child — a
`slot.*` key among a `Cell`'s own attributes, or a `slot.*` key on a
widget inside a `Cell` — is rejected by the two distinct diagnostics in
*Children* above. Grid has no `clip:` / `z-index:` / `area:` surface in
either form.

**Placement-attribute presence rule.** In a Grid with two or more
children, every child must declare both row and column explicitly
(bare `row` / `column` in a `Cell`, or `slot.row` / `slot.column`
directly); omitting either is a `wasamoc check` diagnostic. In a Grid
with exactly one child, missing row and/or column is permitted and
lowers to `0`. The single-child Grid escape clause exists for minimal
demo cases; multi-child Grids are required to be self-describing so the
diagnostic surface for "missed placement" stays local.

**Same-cell / overlapping-rectangle conflict rejection.** For every
pair of children within a Grid, the algorithm checks whether their
resolved `(row, column, row-span, column-span)` rectangles share
any cell — independent of which form (a `Cell` wrapper or direct
`slot.*`) authored each. Conflicts are rejected at `wasamoc check` and
at `validate()`, with a diagnostic naming both conflicting children
and the shared resolved cell coordinate.

**Indexing convention.** All row / column values are **zero-based** at
the `.ui` boundary and zero-based internally. `row: 0` (or `slot.row:
0`) is the first row, `rows.len() - 1` is the last.

**Constant-only.** Grid `columns:` / `rows:` and the placement / span /
alignment keys (in either authoring form) are constant-only literals;
none of them are bindable, and a binding-expression RHS on a placement
key is a `wasamoc check` reject (§4.16, placement is constant per
instance). No new `IrType`, `IrLiteral`, or `PropertyValue` variant is
introduced for placement. A future phase may admit bindable track lists
or bindable placement; this milestone does not foreclose it but does not
implement it.

#### Track-resolution algorithm

Track resolution operates on pure data
(`wasamo-runtime/src/layout.rs`); the algorithm is Win32/WinRT-free.

Per axis (rows and columns are symmetric), given a validated
`tracks: &[TrackSize]` (non-empty; only `Fixed(>=1)` and
`Star(1..=1024)` variants reach this point — `auto` is rejected at
`wasamoc check` / `validate()` and does not appear) and an
`axis_bound` that is either `Bounded(f32)` (the parent's available
space on this axis) or `Unbounded`:

```
function resolve_axis(tracks, axis_bound) -> Result<Vec<f32>, LayoutError>:
    fixed_sum: f32        = sum of declared px over fixed tracks
    star_weight_sum: u64  = sum of (weight as u64) over star tracks
    has_star: bool        = star_weight_sum > 0

    if has_star and axis_bound is Unbounded:
        return Err(LayoutError::GridUnboundedStarAxis)

    // Reserved `auto` demand-distribution slot — no-op in Phase 5.
    // A future phase that admits `auto` inserts a measure-side
    // demand pass here that grows auto tracks to fit content;
    // the pass must execute BEFORE star distribution so star
    // tracks divide the space remaining after fixed + auto
    // consumption.

    bound: f32 = match axis_bound:
        Bounded(b) => b
        Unbounded  => fixed_sum   // unreachable with star tracks

    remaining_after_fixed: f32 = max(0.0, bound - fixed_sum)

    resolved: Vec<f32> = tracks.map(t =>
        match t:
            Fixed(px)     => px as f32
            Star(weight)  => remaining_after_fixed
                             * (weight as f32 / star_weight_sum as f32)
    )

    Ok(resolved)
```

The per-weight cap `[1, 1024]` (per the `Cell` and Grid attribute
tables above) combined with the `u64` star-weight-sum accumulator
bounds the per-axis sum at the type level: the sum is at most
`1024 * track_count`, and `u64` tolerates any structurally feasible
track count (each `TrackSize` allocates memory, so a count
approaching `2^32` is structurally impossible). Star arithmetic is
therefore overflow-safe without a "realistic input" assumption.

**Prefix boundaries** consumed by the arrange pass and by spanning
reconciliation:

```
boundary[0] = 0.0
boundary[n] = boundary[n - 1] + resolved[n - 1]
boundary[tracks.len()] = sum of resolved
                         // = total resolved track extent;
                         // NOT Grid's outer rect — see below
```

**Grid's outer rectangle.** Grid's outer extent on a **bounded**
axis equals the parent's allocation, **not** `boundary[tracks.len()]`.
This matches §4.10 WrapPanel ("outer main-axis size does not grow
to accommodate oversized children") and §4.11 ScrollView ("outer
size = viewport size, regardless of content size"). On an
**unbounded** axis (only reachable with no star tracks per the
unbounded-star branch above), Grid's outer extent equals
`fixed_sum`. The following table summarises the relationship
between the parent allocation, the track-resolved sum, and Grid's
outer rectangle:

| Axis bound | Tracks | Grid outer extent | Cell rectangles |
|------------|--------|-------------------|-----------------|
| `Bounded(b)` | mixed fixed + star | `b` | Sum of resolved = `b`; Cells fit exactly |
| `Bounded(b)`, `fixed_sum <= b` | fixed only | `b` | Sum of resolved = `fixed_sum <= b`; trailing space inside Grid |
| `Bounded(b)`, `fixed_sum > b` | fixed only | `b` | Sum of resolved = `fixed_sum > b`; rightmost Cells overflow, clipped by Grid's outer-bounds clip |
| `Bounded(b)`, `fixed_sum > b` | mixed fixed + star | `b` | Star tracks resolve to `0`; sum of resolved = `fixed_sum > b`; rightmost Cells overflow, clipped by Grid's outer-bounds clip |
| `Unbounded` | fixed only (star + unbounded is an error above) | `fixed_sum` | Sum of resolved = `fixed_sum`; Cells fit exactly |

**Spanning reconciliation.** A `Cell` with
`(row, column, row-span, column-span)` resolves to:

```
left   = column_boundary[column]
right  = column_boundary[column + column-span]
top    = row_boundary[row]
bottom = row_boundary[row + row-span]
```

The cell rectangle is `(left, top, right - left, bottom - top)`.
Spanning `Cell`s are measured against the combined resolved span
extent; **the spanned tracks are not grown** to accommodate a
larger child (there is no `auto`-style demand back-propagation in
Phase 5). A spanning child that exceeds its combined span overflows
its rectangle and is governed by the paint-overflow rule below.

**Negative remaining space.** When fixed tracks alone exceed the
parent's bound on an axis (`fixed_sum > bound`),
`remaining_after_fixed` clamps to `0.0` and every star track on
that axis resolves to width `0`. Fixed tracks retain their declared
sizes in the prefix boundaries; this is not a fault. The resulting
overflow is contained by Grid's outer-bounds clip below.

**Rounding contract.** Track resolution operates in `f32` layout
space; prefix boundaries are deterministic `f32` cumulative sums.
No integer pixel snap. This matches §4.9 Box / §4.10 WrapPanel /
§4.11 ScrollView.

**`LayoutError` surface.** Phase 5 introduces one new variant:

```
LayoutError::GridUnboundedStarAxis
```

fired by the unbounded-star branch above. The variant is
**runtime-internal** in Phase 5; no `WASAMO_LAYOUT_ERROR_*` C ABI
tag is added, consistent with the Phase 4
`LayoutError::ScrollViewUnboundedAxis` precedent.

#### Arrange, overflow, and z-order

Placement is resolved before arrange, so the algorithm below operates on
each child's resolved `(row, column, row-span, column-span, h-align,
v-align)` regardless of which authoring form produced it; "`Cell`" in
this section denotes a placed Grid child (a `Cell` wrapper or a child
carrying direct `slot.*`, §4.16).

After track resolution, each child's resolved rectangle is placed
relative to Grid. The content widget is then arranged inside that
rectangle per the child's alignment:

- **`h-align: stretch` (default).** Content is measured with the
  cell's resolved width as its horizontal bound; the content's
  arranged horizontal extent equals the cell width.
- **`h-align: start | center | end`.** Content is measured at its
  natural horizontal extent and anchored at the start (leading
  edge), center, or end (trailing edge) of the cell rectangle.
- **`v-align: stretch` (default).** Symmetric on the vertical axis.
- **`v-align: start | center | end`.** Symmetric on the vertical
  axis with start = top, end = bottom.

The default `stretch / stretch` makes a `Cell { Box { fill: ... } }`
fill its resolved cell rectangle, which is the common pattern for
visible composition slices.

**Per-cell clipping is out of scope in Phase 5.** A `Cell`'s
resolved rectangle is a measure-arrange rectangle, **not** a paint-
clip rectangle. Content that exceeds the cell rectangle paints past
the cell boundary and may be visible in a sibling cell's region if
no later child paints over it. A future phase may admit a per-cell
`clip:` attribute if author demand warrants; Phase 5 does not
foreclose this.

**Grid outer-bounds clip is on.** Grid's own Visual installs

```
Visual.Clip = InsetClip { 0, 0, 0, 0 }
```

applied to Grid's outer rectangle (per the table in *Track-
resolution algorithm* above). A Cell rectangle that extends past
Grid's outer rectangle (the `fixed_sum > bound` case) has its
overflowing paint cut off at Grid's outer boundary, so the
oversized Grid never bleeds into sibling layout regions. The clip
is a structural commitment; there is no author-facing attribute to
disable it.

**Paint order is document order.** Children paint in source order:
the first `Cell` paints first, the last paints last. When two
`Cell`s have paint regions that incidentally overlap (a Cell's
content overflowed past its rectangle and crossed into another
Cell's region), the later `Cell` paints on top. There is no
`z-index` attribute; intentional overlay is not Grid's
responsibility — Phase 6 ZStack owns overlay, and same-cell
occupancy is rejected at `wasamoc check` / `validate()` so the
"paint order between deliberately-overlapping siblings" question
does not arise.

**Visual ownership.** Grid uses the existing **1 WidgetNode = 1
Visual** convention. Grid's own Visual carries the outer-bounds
clip; each Cell's content widget Visual is a direct child of Grid's
Visual via the normal `sync_visuals()` path. Grid does **not**
introduce an intermediate Visual (unlike §4.11 ScrollView, which
extended the convention to carry a scroll-offset translation; Grid
has no analogous translation).

#### Loader rejection

The runtime IR loader's `validate()` independently rejects malformed
memory IR for every invariant the `wasamoc check` rules above
enforce. The dual-gate pattern matches §4.9 Box (single content
child + `RATIO` sign), §4.10 WrapPanel (negative-literal
rejection), and §4.11 ScrollView (single content child).

**The loader validates the normalized placement, not a `Cell` node.**
Under M3-Phase 7b the `Cell` wrapper is an **author-surface grouping
form** that lowering normalises away (§4.16); the loaded IR carries each
placed Grid child as a content node plus a `SlotData::Grid` placement
payload (architecture.md §6.8.6), and a stale `node Cell { … }` form is
**rejected and regenerated**, not slot-ised (IR-B, §8.5). The `Cell`
**structural** rules — exactly one content child per `Cell`, `Cell` only
under a `Grid` parent — are therefore `.ui` source / `wasamoc check`
rules (they constrain the wrapper before it is normalised), not loader
invariants; the loader's `Cell`-specific job is the stale-form rejection
below. The placement **value** invariants apply to the normalized
`SlotData::Grid` payload of every placed child (whichever form authored
it):

| Invariant (on the normalized `SlotData::Grid` payload, unless noted) | On failure |
|-----------|------------|
| Grid declares at least one row and at least one column | `WASAMO_ERR_IR_MALFORMED` |
| Each fixed track value `>= 1`; each star weight in `[1, 1024]` | `WASAMO_ERR_IR_MALFORMED` |
| A placed child's `row` in `[0, rows.len())`; `column` in `[0, columns.len())` | `WASAMO_ERR_IR_MALFORMED` |
| `row-span >= 1`; `column-span >= 1`; `row + row-span <= rows.len()`; `column + column-span <= columns.len()` | `WASAMO_ERR_IR_MALFORMED` |
| No two placed children within a Grid share any resolved cell | `WASAMO_ERR_IR_MALFORMED` |
| `h-align` / `v-align` values in `{ start, center, end, stretch }` | `WASAMO_ERR_IR_MALFORMED` |
| A stale `node Cell { … }` wrapper or bare-placement-`prop` form in the IR (pre-normalization shape) | `WASAMO_ERR_IR_MALFORMED` (named stale-placement-form diagnostic; reject + regenerate, §8.5) |

All Grid invariants are **reject-at-validate**, not clamp-at-arrange.
Placement / span values have no defensible clamped interpretation: a
silently-clamped `column: 5` in a 2-column Grid would displace a
legitimately-placed Cell at `column: 1` and produce order-dependent
layout. The only layout-time gate is the unbounded-star error
above; negative-remaining-space is not a fault.

#### Reserved future surface

The following surfaces are explicitly **deferred** from Phase 5 and
named here so authors who try them get spec-grounded diagnostics
rather than "unknown token" / "unknown attribute" feedback:

- **`auto` / intrinsic track sizing.** Reserved at the `TrackSize`
  vocabulary level. The track-resolution algorithm above contains a
  documented no-op slot before star distribution where the future
  demand pass will execute.
- **`minmax(min, max)` track sizing.** Additive at the `TrackSize`
  vocabulary level.
- **Floating-point star weights** (e.g. `1.5*`). Integer-weight
  ratios cover the practical proportions (express `1.5 : 1` as
  `3* 2*`); floating-point weights are a future generalisation.
- **Named lines and `grid-template-areas`-style 2D shorthand.**
  CSS Grid-style line names and area names are out of Phase 5
  scope. The `Cell` placement surface does not foreclose a future
  `area:` attribute; such an attribute would lower to the same
  `(row, column, row-span, column-span)` rectangle.
- **Bindable track lists / placement.** Phase 5 is constant-only.
- **Iteration-generated Grid children** (e.g.
  `for item in items { Cell { row: … } }` or a `for` body whose root
  child carries `slot.*`). Grid is not an M3 iteration target —
  M3-Phase 7's `for` member rejects a Grid parent as a recorded
  deferral (§4.15); the iteration grammar's M3 target is
  WrapPanel-backed thumbnail collections. Future admission is
  structurally possible because each child's placement is explicit in
  either authoring form.
- **Per-cell clipping** (`Cell { clip: true ... }`) and any
  author-facing per-cell clip surface.
- **Author-facing `z-index:` / paint-order attribute on `Cell`.**
  Paint order is fixed to document order in Phase 5; intentional
  overlay is Phase 6 ZStack's responsibility.
- **`gap` / `column-gap` / `row-gap`.** No spacing surface on
  Grid in Phase 5; tracks are touching.
- **Drag-resizable splitters / pointer-driven column drag.**
  Pointer-driven track resize is an M4+ input-handling concern.

None of these deferrals require modifying Phase 5's IR shape,
`Cell` contract, default behaviour, or measure-arrange algorithm;
all are additive on top of the Phase 5 surface.

#### Common pitfalls

1. **Star tracks under an unbounded parent.** Placing a Grid with
   star tracks inside a parent that does not bound the corresponding
   axis (e.g. a Grid with `rows: 1*` directly inside a ScrollView's
   scroll axis, or any intrinsic-measure context) fails layout with
   `LayoutError::GridUnboundedStarAxis`. The fix is to bound the
   axis at the parent — replace star tracks with fixed tracks, or
   wrap the Grid in a sized parent.
2. **Fixed-track sum exceeds parent bound.** When fixed tracks
   alone exceed the parent's allocation, star tracks on that axis
   resolve to `0` and the rightmost cells overflow Grid's outer
   rectangle. The overflow is contained by the Grid outer-bounds
   clip — paint is truncated, not propagated to siblings — but
   trailing Cells become invisible. The fix is to reduce the
   declared fixed widths or grow the parent's allocation.
3. **Forgetting row / column in a multi-child Grid.** The
   single-child escape clause does **not** apply once a Grid has
   two or more children. `wasamoc check` rejects the omission with
   a local diagnostic; the fix is to add the missing placement
   explicitly (bare in a `Cell` or as `slot.row` / `slot.column`).
4. **Two children with overlapping rectangles.** Two children that
   resolve to overlapping `(row, column, row-span, column-span)`
   rectangles are rejected with a diagnostic naming both and
   the shared resolved cell — regardless of authoring form.
   Intentional overlay is Phase 6 ZStack's responsibility; the fix
   is to relocate one child or wait for ZStack.
5. **Mixing `Cell` and `slot.*` on one child.** A `slot.*` key among
   a `Cell`'s own attributes, or `slot.*` on a widget inside a `Cell`,
   is rejected by the two distinct diagnostics in *Children*. Pick one
   form per child: a `Cell` wrapper with bare keys, or a directly-placed
   child with `slot.*`.
6. **Expecting per-cell clipping.** A cell whose content paints
   past the cell rectangle may cross into a sibling cell's region
   (until Grid's outer-bounds clip cuts it off). The fix is to
   wrap the oversized content in a clipping parent (e.g.
   ScrollView).
7. **Expecting Grid to grow with its tracks.** Grid's outer
   rectangle equals the parent's allocation on each bounded axis,
   not the sum of resolved track sizes. Authors who want a Grid
   sized by its tracks must size the parent's allocation
   accordingly.

### 4.13 ZStack layout primitive (M3-Phase 6)

**Phase status:** M3-Phase 6 closed; implementation-synced. The
per-child alignment surface is revised to the `slot.*` placement
namespace in M3-Phase 7b (closed; implementation-synced; see §4.16) —
bare `h-align` / `v-align` on a ZStack child becomes `slot.h-align` /
`slot.v-align`.

`ZStack` is an **overlay-dedicated** layout container: its children
occupy the **same** overlap region and paint **back-to-front** in
document order (the first child at the bottom, the last on top). It is
the surface for intentional overlay that Grid deliberately does not
provide — "same-cell overlap is ZStack's responsibility" (§4.12). The
lightbox (a scrim, a centered photo, a caption, and nav buttons stacked
over a thumbnail gallery) is the motivating composition.

ZStack admits **zero or more children**, each a widget declared
directly in the ZStack body (no wrapper construct — unlike Grid's
`Cell`):

```
ZStack {
    Box { fill: #00000080 }       // scrim (bottom)
    Box {                         // photo (on top)
        aspect: 4:3
        Text { text: "photo" }
    }
}
```

#### Mental model

ZStack is **stacked, centered, back-to-front, clipped to bounds**:

1. **Stacked / overlap.** Every child is arranged within the **same**
   ZStack content rect — that is the defining property of the
   primitive. Children do not flow or tile; they overlap.
2. **Centered by default.** Each child is anchored at `center` on both
   axes unless it sets `slot.h-align` / `slot.v-align` (see
   *Attributes*). A ZStack layer is an overlay that should sit at its
   natural size, so `center` — not `stretch` — is the right default.
3. **Back-to-front in document order.** Paint order = document order;
   the later child paints on top. There is no `z-index` (*Out of
   scope*).
4. **Clipped to bounds.** ZStack installs an outer-bounds clip on its
   own Visual; a child that overflows the ZStack rect is cut off at the
   ZStack boundary.

**Ecosystem contrast.** Readers arriving from SwiftUI or WPF should
note:

- **SwiftUI `ZStack`.** The overlap + center-default + back-to-front
  model matches. **One behaviour does not transfer:** SwiftUI's
  full-screen overlay via a flexible child does not apply here, because
  Wasamo's `Fill` measures `0.0` (it does not report the offered size).
  A full-viewport scrim therefore comes from the **ZStack's own
  `Fill/Fill` default** taking the parent allocation, not from a `Fill`
  child inflating the stack (see *Sizing*).
- **WPF overlapping `Grid` children / a bare `Panel`.** WPF stacks by
  placing children in the same cell. Wasamo gives overlay its own
  primitive (`ZStack`) rather than overloading Grid; document-order
  paint matches.

#### Sizing

ZStack's **default size constraint is `Fill/Fill`** (overlay-first,
like Grid / ScrollView):

- On a **bounded** parent axis, ZStack takes the full parent
  allocation.
- On a **Shrink / unbounded** axis, ZStack's desired size is the
  **union** — the per-axis **max** of its children's measured desired
  sizes.

A `Fill` child contributes **`0.0`** to the union (the engine's `Fill`
measure rule) and fills its allocated rect during *arrange*; it does
**not** inflate the ZStack's measured size. So a ZStack of only
intrinsic children sizes to its largest child on a Shrink / unbounded
axis, while on a bounded axis the `Fill/Fill` default fills the parent.
The lightbox's full-viewport scrim (`Box { fill: #00000080 }`, a `Fill`
child) is visible because the **ZStack itself** fills the parent
allocation and the scrim then fills that content rect — not because the
scrim drives the stack's size.

ZStack introduces **no new `LayoutError`**: it has no intrinsic sizing
pass that diverges on an unbounded axis (it defers entirely to each
child's Fill / Shrink resolution), so the unbounded-axis errors that
Grid (`LayoutError::GridUnboundedStarAxis`) and ScrollView
(`LayoutError::ScrollViewUnboundedAxis`) raise do not recur for ZStack.

**Owner-visible trade-off.** Phase 6 ships **no author-facing
`width:` / `height:` size-constraint surface**, so an author cannot opt
a ZStack back to **intrinsic** sizing (size-to-largest-child on a
bounded axis) this phase. ZStack is therefore an **overlay-first**
container: its default fills the parent, and the SwiftUI-style intrinsic
ZStack (badges sized to the icon they overlay) is weaker in the default
experience until a future per-widget size-constraint surface lands.
This is a deliberate trade-off, accepted because the Phase 6 driver is
the lightbox overlay, for which fill-the-parent is the wanted default.

#### Attributes

ZStack admits **no ZStack-level attributes** in Phase 6 — no `spacing`,
`padding`, `z-index`, `columns` / `rows`, or background `fill`. Unknown
ZStack attributes are rejected at `wasamoc check` and at runtime
`validate()`. The scrim is a child `Box { fill: #RRGGBBAA }` (§4.9),
not a ZStack attribute.

**Per-child alignment.** Each ZStack **direct child** may carry
`slot.h-align` and `slot.v-align` — parent-interpreted placement keys in
the shared `slot.*` namespace (§4.16):

| Placement key   | Type  | Default  | Valid range                       |
|-----------------|-------|----------|-----------------------------------|
| `slot.h-align:` | ident | `center` | `{ start, center, end, stretch }` |
| `slot.v-align:` | ident | `center` | `{ start, center, end, stretch }` |

The default is **`center`** on both axes (contrast Grid, which defaults
to `stretch` because a grid cell is a *slot* the content fills; a ZStack
layer is an *overlay* that sits at its natural size). `stretch` (or a
`Fill` size constraint on the child) expands the child to the full
content rect; `start` / `center` / `end` anchor the child's measured
size within the content rect.

`slot.h-align` / `slot.v-align` are **parent-interpreted placement**,
not widget properties (§4.16): they are admitted only on a **ZStack
direct child**, read by the ZStack context as child-placement metadata
*before* the child's own attribute check, so the child's normal
unknown-attribute rejection never sees them. The same keys placed under
a non-admitting parent — or written **bare** (without the `slot.`
prefix, the M3-Phase 6 spelling) on a ZStack child — are rejected by a
named `wasamoc check` diagnostic, re-checked by the loader. Grid's
placement uses the same keys (§4.12), authored grouped inside a `Cell`
(bare) or directly as `slot.*`.

#### Measure-arrange and Visual contract

ZStack's measure-arrange operates on pure data
(`wasamo-runtime/src/layout.rs`); the algorithm is Win32/WinRT-free.
After ZStack resolves its outer rect (per *Sizing*), each child is
measured against the ZStack content rect and anchored within it by its
`slot.h-align` / `slot.v-align`. All children share the **same** content
rect — the overlap region.

**Paint order is document order.** The first child paints first
(bottom), the last paints last (top). This rides the existing
document-order `sync_visuals` insertion — ZStack adds no separate
z-order mechanism.

**Outer-bounds clip on, per-child clip out.** ZStack's own Visual
installs `Visual.Clip = InsetClip { 0, 0, 0, 0 }` applied to its outer
rect; each child Visual has `Visual.Clip = null` (clip-absence
regression guard, symmetric with WrapPanel / ScrollView / Grid). An
overflowing overlay child is cut off at the ZStack boundary. ZStack uses
the existing **1 WidgetNode = 1 Visual** convention — it introduces
**no intermediate Visual** (unlike §4.11 ScrollView, which added one to
carry a scroll translation; ZStack has no analogous translation).

#### Out of scope (Phase 6)

- **Explicit `z-index` / author-facing layering** — paint order is
  fixed to document order.
- **Per-child `clip:` surface** — only the ZStack outer-bounds clip
  ships; per-child clip is a future additive child attribute.
- **ZStack background `fill`** — the scrim is a child `Box`; a future
  ZStack-level `fill` would grow the attribute allow-list additively.
- **Author-facing `width:` / `height:` size constraint** — see the
  *Sizing* owner-visible trade-off; until it lands, ZStack's
  `Fill/Fill` default cannot be overridden to intrinsic sizing.
- **Iteration-generated children** — landed in M3-Phase 7: a `for`
  block is admitted as a ZStack direct member (§4.15), with per-child
  placement handled by the runtime's child-carried placement storage.

### 4.14 Conditional rendering and the structural rendering model (M3-Phase 6)

**Phase status:** M3-Phase 6 closed; implementation-synced.

This chapter introduces **conditional rendering** — a `binding` that
drives the **present / absent state of a subtree** rather than a
property value. It is the **first chapter of Wasamo's structural
rendering model**: `if` is the first member of a **structural
control-flow grammar family** whose later members — `else` / `else if`
(more branches), `switch` (more discriminants), and `for` (iteration —
landed M3-Phase 7, §4.15) — arrive in the **same** family with the
**same** structural runtime machinery. An external reader should be
able to predict the family's growth from this chapter alone.

#### Why a structural directive, not a presence property

There are three ways a UI language could express conditionality; Wasamo
chooses the second deliberately:

1. **Property toggling** — an always-built subtree with a `visible:` /
   `enabled:` property that hides it. This proves *property toggling*,
   not structural presence; the subtree, its widgets, and its effects
   all still exist while "hidden". Wasamo does **not** use this model
   for conditional rendering.
2. **A dedicated structural directive** (Wasamo's choice) — a template
   construct (`if`) that makes a subtree **genuinely present or
   absent**. When absent, the subtree's widgets, Visuals, and effects
   do not exist. This is the model that can host the `if` → `else` /
   `switch` / `for` family.
3. **Host-language control flow** — `if` / `switch` embedded in a
   general-purpose host language. This is more than M3 needs and is
   **not** the v1 surface, but Wasamo's IR is shaped so a future
   language-construct surface could lower into the **same** structural
   construct — approach 3 is not foreclosed.

`if` is a structural directive (model 2). It reads as structure — a
block that exists or does not — not as a property of a widget.

#### The `if` block

```
if <cond-expr> {
    <widget>
}
```

`if` is a new **member** form (§3), admitted wherever a widget body's
members appear. When `<cond-expr>` is true the body is **present** as a
child of the enclosing widget at the block's document position; when
false it is **absent**. `if true { … }` and `if false { … }` are
well-typed but degenerate (always present / always absent) — permitted,
not special-cased.

The visible-proof shape is the lightbox toggle:

```
state is_lightbox_open: bool = false
…
if is_lightbox_open {
    ZStack { /* scrim + photo + caption + nav */ }
}
```

driven by a text-Button click handler that writes `is_lightbox_open`.

`if`, `else`, `switch`, and `for` are **all reserved keywords** as of
M3-Phase 6 (§2.1), even though only `if` has a production this phase —
see *The structural control-flow family* below.

#### Condition expressions

The `if` condition admits **exactly the narrow bool-expr that
`Button.enabled` already accepts** (§4.8): a `BOOL_LIT` (`true` /
`false`) or an identifier that resolves to a `bool`-typed `state`
declaration. There are **no operators** — `!ready`, comparisons
(`count > 0`), and logical operators (`a && b`) are **not** admitted in
the condition this phase.

This is a uniformity choice, not a size one: the condition position is
one `expr` position among many (every property RHS is an `expr`), and
operators are reserved to grow **uniformly across all `expr` positions
at once** (a future expression-grammar extension), rather than as a
condition-only pocket. Until then, an author inverts a condition by
introducing a complementary `bool` state. `wasamoc check` rejects an
operator condition with a diagnostic that points at the deferred
extension — a *recorded deferral*, not a silent gap.

#### Conditional body — a single widget child

In Phase 6 the `if` body admits **exactly one widget child** — one
`widget_decl`, nothing else:

- **no** property / bind / handler / `state` / track-list member
  directly in the body (the body is structural, not a place to apply
  conditional properties or scope branch-local state);
- **no** second child;
- **no** nested `if` directly in the body.

A multi-widget or nested-conditional body is authored by **wrapping** in
a container the author usually wants anyway:

```
if open { VStack { Text { … } Text { … } } }   // multi-widget → wrap
if a    { VStack { if b { … } } }               // nested if → wrap
```

The single-widget body always materialises **exactly one** widget,
which is what lets present / absent be a single subtree insert / remove.
This single-widget body discipline is shared by the whole structural
family — the M3-Phase 7 `for` body carries the same rule (§4.15); the
multi-member range body is a deferred family-wide generalisation, and
nested control flow directly in a body lands with the family extension.

**What is and is not in scope here, precisely:** what an `if` *body*
admits is one `widget_decl`. But **sibling** conditionals (`if a { … }
if b { … }` under one parent) and **descendant** conditionals nested
inside the wrapped widget (`if a { VStack { if b { … } } }`) are fully
in scope — they are ordinary `if` members at a deeper position, and
their present / absent, child ordering, and effect lifecycle are all
Phase 6 runtime semantics.

#### Placement — inside a widget body only

An `if` block is admitted only **inside a widget body**. A
**component-level `if`** — one that would gate or multiply the single
content root — is rejected at `wasamoc check`: the runtime makes a
subtree present / absent by inserting / removing it into a **parent**,
and a root-level conditional has no parent slot. (A conditional /
multiplexed content root is a distinct design not opened this phase.)
The lightbox `if` sits inside the root container, the in-scope shape.

#### Present / absent is structural

When the condition becomes true, the runtime **builds a fresh subtree**
from the declared body and **inserts** it into the parent at the block's
position. When the condition becomes false, the runtime **removes and
destroys** the subtree — its widgets and Visuals are dropped and its
effects disposed. This is genuine structural mutation, not visibility
toggling: an absent subtree does not exist in the widget tree.

The conditional re-inserts at the position matching its **declared**
document order, so when a conditional has static siblings on either
side, its subtree lands in the correct sibling order (and, inside a
ZStack, the correct document-order z-order), not merely on top.

#### Identity: an absent subtree returns fresh

**A conditional subtree that goes absent and returns is a _fresh_
subtree; any state inside it resets.** This is **normative
author-visible semantics**, not an implementation detail. The lightbox
photo is stateless, so reopening fresh is correct; an author who needs
state to persist across toggles keeps that state in a **component-level
`state`** (declared outside the conditional), the established Wasamo
pattern.

Future **state retention** across absent → present (preserving
in-progress input, focus, or scroll position) will arrive as **opt-in**
semantics — a `key:` / retention marker on the construct — so existing
`if` blocks keep destroy-and-recreate behaviour and retention never
silently alters observable behaviour. Destroy-and-recreate is the
baseline that does not change. (Internally, the declared `if` construct
is the stable identity anchor across toggles; only the materialised
subtree is recreated — see [architecture.md §9](./architecture.md).)

#### Effect lifecycle and the toggle-then-observe contract

The effect lifecycle inside a conditional subtree is the one-line rule:
**an absent subtree has no live effects; a present subtree's effects are
freshly created and run.** On absent, the subtree's reactive effects are
disposed through the structural teardown (and every hit-test target's
widget-pointer registry entry is severed); on present, fresh effects are
registered on the fresh widgets. There is no "paused effect" state.

The **toggle-then-observe** contract holds: a condition write outside a
batch (for example inside a Button click handler) **drains before
control returns**, so immediately after the toggling call the subtree's
present / absent change is complete **and** the freshly-inserted
subtree's bound properties have been initialised — no one-frame-stale
window. This preserves the synchronous non-batched drain contract
established in M3-Phase 1. (For an inserted subtree large enough that its
fresh effects exhaust the drain's convergence budget before quiescence,
the existing divergence backstop fires — the same behaviour as any other
effect fan-out, not silent staleness.)

#### Child order with multiple conditionals

When several sibling or wrapped-descendant conditionals are present at
the same time, the parent's child order at quiescence is a function of
**declared document order alone** — whichever conditionals are present
appear among the static siblings in declared order, **independent of the
order in which the condition effects evaluate**. The transient
evaluation order of independent effects is unspecified (as it already is
for property bindings), but the final, observable layout is fully fixed
by the declared tree.

#### The structural control-flow family

`if` is the first member of a family that grows additively:

- **`else` / `else if`** — chains the `if` block (`if c { … } else { …
  }`); additional branches on the same construct.
- **`switch`** — a sibling block keyword over a non-bool discriminant.
- **`for <binder> in <collection> { … }`** — landed in M3-Phase 7
  (§4.15): iteration as a sibling block keyword, reusing the same
  structural-subtree machinery generalised from presence (0/1) to
  cardinality (0..N). Its identity baseline is **positional and
  un-keyed**; keyed identity / state retention is a future opt-in
  surface, not something `for` ships (§4.15, identity baseline).
- **operator conditions** (`!ready`, comparisons, logical operators) —
  arrive through the uniform expression-grammar extension that widens
  every `expr` position, including `cond-expr`.

All four family keywords (`if` / `else` / `switch` / `for`) are reserved
now (§2.1) so the family lands without a future source break. `in` is
reserved as of M3-Phase 7, when its production landed (§2.1, §4.15);
contextual sub-tokens of still-undesigned productions (`case` /
`default` for `switch`) are **not** reserved yet — each is reserved
when its production is specified. `else if` is `else` followed by `if`
(two keywords), not a separate token.

#### Diagnostics (rejected shapes)

`wasamoc check` rejects each of the following, and the runtime IR loader
independently re-checks them (`WASAMO_ERR_IR_MALFORMED`, §8.11) because
the memory-IR entry point does not pass through `wasamoc`:

| Rejected shape | Example | Diagnostic |
|---|---|---|
| Non-bool condition | `if count { … }` (`count: i32`); `if "x" { … }` | type error |
| Undeclared condition name | `if missing { … }` | name-resolution error |
| Operator condition | `if !ready { … }`; `if a && b { … }`; `if count > 0 { … }` | "operators in `if` conditions are not yet supported in M3-Phase 6" (points at the deferred expression-grammar extension) |
| Non-structural body member | `if open { fill: red }`; `if open { state x: bool = true }` | "`if` body admits only a single widget child; properties, bindings, handlers, state declarations, and track lists are not structural body members" |
| Nested `if` directly in body | `if a { if b { … } }` | "a bare nested `if` is not admitted directly in an `if` body in M3-Phase 6; wrap it in a widget container" |
| Multiple children in body | `if open { Box{} Text{} }` | "`if` body admits exactly one widget child in M3-Phase 6; wrap multiple widgets or nested control flow in a container" |
| Component-level `if` | an `if` at component body level | "component-level `if` is not supported in M3-Phase 6; put the `if` inside a widget body" (a root-level conditional has no parent slot) |
| Direct conditional under ScrollView | `ScrollView { if c { … } }`; `ScrollView { Content  if c { … } }` | "`ScrollView` content child must be a single widget; a conditional member is not valid directly in ScrollView (wrap it in the content widget)" — the exactly-one-content-child cardinality cannot absorb a dynamic member; parallels the `Cell` direct-conditional rejection (§4.12) |
| Bare `else` / `switch` | `else { … }`; `switch x { … }` | "reserved / not yet supported" (names the construct) |

The reserved-but-unsupported diagnostic for `else` / `switch`
(a *block* in member position) is distinct from the identifier-position
rejection that fires when one of the four family keywords is used as a
name (§2.1). A `for` block is no longer in this class — it has a
production as of M3-Phase 7 (§4.15) and is rejected only where §4.15's
placement / shape rules reject it.

#### Out of scope (Phase 6)

- **`else` / `else if` / `switch`** — reserved family members, not yet
  implemented.
- **`for` / iteration** — landed M3-Phase 7 (§4.15).
- **Nested control flow directly in an `if` body** (`if a { if b { … }
  }`) — reached meanwhile by wrapping the inner `if` in a widget; lands
  with the family extension.
- **Operators in the condition** — deferred to the uniform
  expression-grammar extension.
- **State retention / `key:` across absent → present** — Phase 6 ships
  the fresh-on-return base case; retention is future opt-in.
- **Property / state / handler conditionality** — the body is
  structural only; conditional property application, branch-local
  state, and conditional handlers are not opened by A7.

### 4.15 Iteration and collection-driven generation (M3-Phase 7)

**Phase status:** M3-Phase 7 closed; implementation-synced.

This chapter introduces **iteration** — a collection binding that
drives the **number of generated widget subtrees**. It is the **second
chapter of Wasamo's structural rendering model** (§4.14): `if` makes a
binding drive a subtree's presence (0/1); `for` makes a binding drive a
subtree count (0..N). Both are member-level structural control-flow
constructs of the same grammar family, with the same expansion model —
declared members expand to materialised children in document order,
each member contributing its live cardinality (a widget contributes 1,
an `if` contributes 0 or 1, a `for` contributes the current collection
length).

This is **not static template expansion**: the collection is a
runtime-mutable `state`, and mutating it at runtime inserts / removes
generated subtrees while the rest of the tree is retained.

#### The `for` block

```
for <binder> in <collection> { <widget> }
for <binder>, <index-binder> in <collection> { <widget> }
```

`for` is a **member** form (§3 `iteration_member`), admitted wherever a
widget body's members appear (subject to the per-container admission
rules below). Per element of the collection, the body instantiates as a
child of the enclosing container at the block's document position:

```
state thumbs: i32[] = [101, 102, 103]
…
WrapPanel {
    for thumb in thumbs {
        Box {
            aspect: 1:1
            fill: #cccccc
            Text { text: "Photo \{thumb}" }
        }
    }
}
```

- The first `IDENT` is the **element binder**; the optional second
  `IDENT` after a comma is the **index binder**. Both are
  **author-named** — there are no fixed magic names. `item` and `index`
  are perfectly good conventional *choices* of binder name; they are
  not keywords and never enter scope implicitly.
- The post-`in` position must be a **bare identifier resolving to a
  collection-typed `state`** declared in the same component (§4.7).
  Collection *expressions* in this position (literals, slices, computed
  collections) are not admitted this phase; the position widens with
  the uniform expression-grammar extension. Qualified references
  (`for x in root.xs`) are likewise rejected — new collection-reference
  positions use local component state by name.
- An **empty collection is legal**: the `for` member stays live and
  materialises zero children. The admitted containers all tolerate zero
  children.

#### Iteration body — a single widget child per iteration

The `for` body admits **exactly one widget child** — one `widget_decl`,
nothing else — the same body discipline as the `if` body (§4.14):

- **no** property / bind / `state` / track-list member directly in the
  body;
- **no** second child;
- **no** bare control-flow member as the immediate body;
- a `signal_handler` may appear inside the body template under the
  admission and binder-read rules in *Handlers inside a `for` body*
  below;
- **no** `for` member at **any depth** inside a `for` body template
  (see *Nested control flow* below).

Multi-widget items wrap in a container the author usually wants anyway
(`for t in xs { VStack { Box { … } Text { … } } }`). N elements
materialise exactly N children — the cardinality contract this grammar
exists to provide.

**Nested control flow.** A descendant `if` member inside the body's
widget subtree is admitted (`for t in thumbs { Box { if flag { … } } }`)
— its condition resolves to `bool` state exactly as in §4.14; a
loop-local binder is **not** admitted in that condition (see *Loop-local
binders*). A `for` member anywhere inside a `for` body template is
rejected: a descendant `for` introduces nested template scope (outer
binders visible inside an inner template), and scope nesting /
shadowing is deferred to the phase that opens the next structural
control-flow extension. A `for` nested inside an `if` body's widget
subtree is admitted when that `if` is not itself inside a `for`
template.

#### Loop-local binders

Iteration introduces the DSL's first **template-local names**. Inside a
`for` body, the element binder (and the index binder, when declared) is
readable so generated subtrees can differ per item.

- **Types.** The element binder has the collection's element type
  (`i32` / `string` / `bool`); the index binder is **`i32`, read-only,
  zero-based**.
- **Read positions.** A binder is readable in **property-binding and
  interpolation expression positions** within the `for` body's widget
  subtree (`text: thumb`, `text: "Photo \{thumb}"`), and — since
  M4-Phase 2 — in **handler position** inside the same body (§4.19). It
  is **not** readable in an `if` condition (condition identifiers
  resolve to `bool` state only), in property *literal* positions, or
  anywhere outside the body. This is the first — and only — codified
  exception to the rule that every dynamic reference in a binding
  resolves to a component `state`.
- **Scope.** Flat: binders are visible from the `{` to the matching `}`
  of their `for` body, in the admitted read positions only.
- **Collisions are errors** at `wasamoc check`: a binder may not share
  a name with any declared `state` (collection or scalar); the element
  binder may not equal the index binder; reserved keywords are not
  valid binder names.
- **No shadowing rule is defined** — nothing can nest this phase
  (nested `for` is rejected at any template depth), so no shadowing
  semantics ships. This absence is by design, not undefined behaviour:
  nested template scope and shadowing are specified together with the
  next structural control-flow extension.
- **A binder is not a widget id and not an item key.** It names a value
  in scope during template instantiation; it confers no identity on the
  generated subtree (identity is positional — see *Identity baseline*)
  and no addressable handle on any widget.

A binder read evaluates as a **live positional read** of the collection
signal: the position is fixed per instantiation, the *value* is read
from the current collection. Under a same-length whole-value reset
(below), retained positions re-evaluate their bound properties in
place.

#### Collection mutation — whole-value assignment

Collection mutation is **whole-value assignment**: a handler statement
assigns the collection state its next value. Assignment remains the
**only** statement form in the DSL — there are no method-call
statements — and the expression grammar gains no operators. The
assignment RHS admits exactly three forms (§3 `collection_expr`):

```
add_thumb    => { thumbs = thumbs.append(next_id); }
remove_thumb => { thumbs = thumbs.drop-last(); }
clear_thumbs => { thumbs = []; }
reset_thumbs => { thumbs = [101, 102, 103]; }
```

- **`xs.append(expr)`** — a pure expression evaluating to a new
  collection with one element appended; the element expression is
  type-checked against the declared element type.
- **`xs.drop-last()`** — a pure expression evaluating to the collection
  minus its last element. **`drop-last` is total: on an empty
  collection it evaluates to the empty collection (the identity)** —
  matching the Swift / Kotlin `dropLast` precedent exactly. A boundary
  Remove action is therefore idempotent by function semantics, not by a
  statement special case.
- **A static collection literal** — whole-value reset / clear. Element
  typing follows the state-default rules (§4.7): homogeneous,
  element-type-checked against the LHS, `[]` typed from the LHS, no
  nesting, no identifiers inside.

Restrictions, each rejected at `wasamoc check` (and re-checked at the
loader):

- The method **receiver must be the assigned state itself**:
  `xs = ys.append(a)`, chained applications
  (`xs = xs.append(a).append(b)`), and a bare state copy (`xs = ys`)
  are rejected with a diagnostic naming the deferred general
  collection-expression surface.
- **`=` only** on a collection LHS — compound assignment operators have
  no collection meaning.
- The LHS and the receiver are **bare state names**; qualified forms
  (`root.xs = …`) are rejected.
- A bare collection expression as a statement (`xs.append(a);`) is
  rejected — the diagnostic points at the assignment form.
- A `collection_expr` outside a collection-assignment RHS is rejected.

`append` and `drop-last` are **contextual method names**, not reserved
keywords: a state or widget named `append` or `drop-last` still parses.
`;` placement is unchanged — `;` terminates handler-block statements
only (the collection assignment is an `assign_stmt` alternative);
member positions (state declarations, property settings) carry none.

The Phase 7 verification slice used all four authored mutation forms in
the gallery thumbnail set:

```
state labels: string[] = ["S01", "S02", "S03", "S04", "S05", "S06"]

Button {
    text: "Add"
    clicked => { labels = labels.append("NEW"); }
}
Button {
    text: "Remove"
    clicked => { labels = labels.drop-last(); }
}
Button {
    text: "Clear"
    clicked => { labels = []; }
}
Button {
    text: "Reset"
    clicked => {
        labels = ["S01", "S02", "S03", "S04", "S05", "S06"];
    }
}

ScrollView {
    offset-y: scroll_y
    WrapPanel {
        for label, index in labels {
            Box {
                aspect: 1:1
                fill: #336699cc
                Text { text: "\{label} #\{index}" }
            }
        }
    }
}
```

This example deliberately varies only the scalar label (plus the
loop-local index). Varying both label and colour per item requires
deferred surfaces — record-like item data / `TypedValue`,
loop-external indexed collection reads, and a bindable `Box.fill`
surface — so the Phase 7 proof keeps `fill` static.

The final M3 Photo Gallery target app still uses a collection state and
`for label, index in labels` to generate thumbnails, but it does not keep
the Phase 7 Add / Remove / Clear / Reset controls as end-user UI. Those
controls were verification scaffolding for the collection-assignment
surface; their coverage remains this section's authored mutation example
and the Phase 7 implementation evidence, while the integrated Gallery
keeps the target-app surface focused on generated thumbnails, scrolling,
selection, and lightbox state.

**Equal-value writes propagate nothing.** A collection assignment whose
new value equals the current value performs no dirty propagation — a
`drop-last()` on an empty collection, or a reset to the identical
current value, writes an equal value and re-runs no effects.

#### Identity baseline: positional, un-keyed

**A generated subtree's identity is its position in the collection. A
tail append materialises only the new tail subtrees; a tail removal
disposes only the removed tail subtrees; subtrees at retained positions
are retained — their bound properties re-evaluate; they are not
rebuilt.** This is normative author-visible semantics: appending item
N+1 does not disturb items 0..N.

The explicit non-promise beside it: **positions confer no
element-tracking identity; no state is preserved across removal; keyed
retention is a future opt-in surface.** When keyed identity arrives (an
opt-in `key:`-like marker), it changes which materialised subtree maps
to which *element*; it never silently changes this positional baseline.

A whole-value reset (`xs = [..]`) follows the same rule: its structural
delta is exactly its length delta; value changes at retained positions
flow through the live positional binder reads (a same-length reset
re-evaluates item bindings in place with no structural edit).

#### Runtime mutation timing and failure contract

- **Mutation-then-observe.** A collection assignment outside a batch
  (for example inside a Button click handler) drains before control
  returns: immediately after the mutating call, the new subtrees exist
  with their bound properties written (or the removed subtrees are
  gone). This generalises §4.14's toggle-then-observe contract to range
  mutation; the synchronous non-batched drain contract established in
  M3-Phase 1 is preserved.
- **Insertion is all-or-unchanged.** All fallible construction for a
  range insert happens before any tree mutation (subtrees are staged
  fully, then committed). A staging failure aborts the whole mutation
  with the materialised tree observably unchanged, plus a range-scoped
  diagnostic. A failure in the commit stage itself (an OS-level
  inconsistency) is logged with range context rather than promised as
  undoable.
- **Disposal order.** Removed subtrees dispose tail-first; their
  reactive effects are disposed ahead of structural teardown, and their
  widget-registry entries are released — the same lifecycle rule as
  §4.14's absent subtree, applied per removed item.
- **Quiescent order.** At quiescence, the parent's children are the
  declared members expanded by live cardinality in document order —
  static siblings, `if` members, and `for` members interleave by
  declared position, independent of effect evaluation order.

#### Where a `for` member is admitted

| Container | Direct `for` child | Reason |
|---|---|---|
| `VStack` / `HStack` / `WrapPanel` | **admitted** | arbitrary-children contract |
| `ZStack` | **admitted** | arbitrary-children contract; per-child `slot.*` placement (§4.16) rides the body's root child (CF on the body root) |
| `ScrollView` | **rejected** | exactly-one-content-child contract (§4.11); wrap the `for` inside the single content widget — `ScrollView { WrapPanel { for … } }` is the canonical gallery shape |
| `Box` | **rejected** | at-most-one-child contract (§4.9); a `for` can produce more than one |
| `Grid` | **rejected** | Grid children carry placement (`Cell` or direct `slot.*`, §4.12); `for`-generated Grid placement is a recorded deferral |
| component level | **rejected** | no parent slot for a 0..N root, same ground as the component-level `if` reject |

Each rejection is a named diagnostic, not a silent gap; the ScrollView
/ Grid rejections record deferrals (a conditionally-/ iteratively-
shaped ScrollView content model, `for`-generated Grid placement), not
permanent exclusions.

#### Diagnostics (rejected shapes)

`wasamoc check` rejects each of the following, and the runtime IR
loader independently re-checks the structural rows
(`WASAMO_ERR_IR_MALFORMED`, §8.11):

| Rejected shape | Example | Diagnostic |
|---|---|---|
| `for` over a non-collection | `for x in count { … }` (`count: i32`); `for x in missing { … }` | type / name-resolution error |
| `for` over a non-identifier | `for x in [1, 2] { … }` | "collection expressions are not yet supported" (recorded deferral) |
| Qualified collection reference | `for x in root.xs { … }` | "the loop collection must be a local state name" |
| Binder collides with a state | `state thumb: i32 = 0 … for thumb in xs { … }` | name-collision error |
| Element binder = index binder | `for a, a in xs { … }` | name-collision error |
| Keyword in binder position | `for in in xs { … }`; `for if in xs { … }` | parse error (§2.1) |
| Disallowed container | `ScrollView { for … }`; `Box { for … }`; `Grid { for … }` | names the container contract |
| Component-level `for` | a `for` at component body level | "component-level `for` is not supported; put the `for` inside a widget body" |
| Non-widget body member | `for t in xs { fill: red }` | "`for` body admits only a single widget child per iteration" |
| Multiple body children | `for t in xs { Box{} Text{} }` | same — wrap in a container |
| Bare control flow as immediate body | `for t in xs { if c { … } }` | wrap rule, same as §4.14 |
| Nested `for` at any depth | `for a in xs { VStack { for b in ys { … } } }` | "nested `for` is not yet supported" (nested-template-scope deferral) |
| Binder read in an `if` condition | `for t in flags { Box { if t { … } } }` | "`if` conditions resolve to `bool` state only" (per-item conditional presence deferral) |
| Binder read outside its body; undeclared binder | `text: thumb` outside the `for` | name-resolution error |
| Nested collection type | `state xs: i32[][] = []` | "nested collection types are not supported" |
| Heterogeneous / mismatched literal | `state xs: i32[] = [1, "a"]`; `xs = [true]` on `i32[]` | element-type error |
| Non-literal collection element | `state xs: i32[] = [a, b]` | "collection literal elements must be scalar literals" (recorded deferral) |
| List literal on a scalar state (and vice versa) | `state n: i32 = []`; `state xs: i32[] = 0` | type error |
| Collection assignment on a non-collection LHS (and vice versa) | `count = count.append(1);`; `xs = 0;` | type error |
| Compound assign on a collection | `xs += 1;` | "compound assignment is not defined over collections" |
| Wrong receiver / chained / bare copy | `xs = ys.append(a);`; `xs = xs.append(a).append(b);`; `xs = ys;` | "general collection expressions are not yet supported" (recorded deferral) |
| `append` arity / `drop-last(expr)` | `xs = xs.append();`; `xs = xs.drop-last(1);` | arity error |
| Bare collection statement | `xs.append(a);` | "collection mutation is written as assignment: `xs = xs.append(…)`" |
| Qualified LHS / receiver | `root.xs = root.xs.append(1);` | "collection mutation requires a local state name" |
| Loop-external collection read | `text: "\{xs}"`; length / element reads outside the loop | "collection reads outside iteration are not yet supported" (recorded deferral) |

#### Handlers inside a `for` body (admitted in M4-Phase 2)

A `signal_handler` member inside a `for` body template was rejected in
M3-Phase 7, on the reasoning that admitting handlers without binder
reads would ship per-item widgets whose handlers can only mutate global
state, and that handler admission, handler-position binder reads,
registration lifecycle and identity interaction had to be designed
together.

**They are, in §4.19.** A handler inside a `for` body is admitted, its
binders are readable in the handler body, its registration is released
with the generated subtree, and its relation to the positional identity
baseline above is stated there. The M3-era workaround — keeping the
mutation Buttons outside the `for` body — is no longer required.

#### Out of scope (Phase 7)

- **Keyed identity / retained state** — future opt-in over the same
  declared-slot anchor; the positional baseline above is the cited
  contract it must not silently change.
- **Data-driven reorder** — excluded by construction (the authored
  mutations are tail edits and static resets); reorder arrives with an
  ordering contract + keyed diff.
- **Structured item fields** (`item.field`, record-like elements) —
  elements are scalars this phase.
- **`f64[]`** — additive fourth element type, deferred.
- **Host-supplied initial collections / host replace / write-back** —
  collection state is runtime-owned; the host state boundary is a
  separate deferred surface (the whole-value representation keeps it
  unblocked).
- **Loop-external collection reads** (`length`, emptiness, element
  index reads) — with the uniform expression / reference extension.
- **Per-item handlers and handler-position binder reads** — see above.
- **Per-item conditional presence** (a loop-local `bool` binder in an
  `if` condition) — reopens with the first concrete per-item branching
  case.
- **Nested `for` / template scope and shadowing** — with the next
  structural control-flow extension.
- **Member-range bodies** (multiple members per iteration) — the
  deferred family-wide body generalisation.
- **General collection expressions** (computed lists, `xs = ys`,
  slices) — the static literal is the only whole-value RHS.
- **Grid / Box / ScrollView direct `for`** — see the admission table.
- **Large-N performance / lazy materialisation** — deliberately out of
  scope at gallery N.

### 4.16 Parent-interpreted placement (`slot.*`) (M3-Phase 7b)

**Phase status:** M3-Phase 7b closed; implementation-synced.

Some layout containers interpret metadata *about how a child sits inside
them*: Grid reads a child's row / column / span / alignment, and ZStack
reads a child's alignment within the overlap region. This metadata is
**parent-interpreted placement** — it is **not an attribute of the child
widget itself**. `Text.text` and `Button.enabled` are properties of the
widget; a child's `row` / `column` or its overlay `h-align` describe how
the child is *treated by its immediate parent container*. Placement is
therefore authored in a dedicated **`slot.*`** namespace on the child,
never as an ordinary widget property.

The prefix exists so the parent-interpreted nature is legible at the
call site — `slot.h-align` reads as "the slot this child occupies in its
parent", not as a property the child carries — and so placement keys can
never collide with a widget's own property names as the widget
vocabulary grows. The two placement-bearing containers this milestone
ships — **Grid** (§4.12) and **ZStack** (§4.13) — share this one
placement grammar; Grid additionally keeps a `Cell` grouped form as a
convenience over the same model (below).

#### The `slot.` placement-key prefix

A placement key is a property bind whose key carries the reserved
**`slot.`** prefix:

```
ZStack {
    Box { slot.h-align: end  slot.v-align: start  Text { text: "badge" } }
}

Grid {
    columns: 1* 1*
    rows: 64
    Button { slot.row: 0  slot.column: 1  text: "ok" }
}
```

`slot` is a **contextual prefix**, not a reserved keyword: it is
significant only as the head of a dotted placement key and remains a
valid ordinary identifier everywhere else. The right-hand side is a
**placement constant** — an integer literal (`slot.row` / `slot.column`
/ `slot.row-span` / `slot.column-span`) or an alignment keyword
(`slot.h-align` / `slot.v-align`) — resolved against the closed
placement-keyword set for that key (below), not against the state
namespace.

The grammar is a member alternative (§3 `placement_bind`); the key is a
dotted placement key (`slot` `.` *placement-name*), distinct from an
ordinary `property_bind`. Whether the lexer emits one token or the
parser folds `Ident("slot") Dot Ident(name)` into a placement key is an
internal encoding; the spec fixes the author-visible accepted / rejected
set and the rejecting stage (below).

#### Admission — which container admits which keys

Placement keys are admitted **only** on a child of a placement-bearing
parent, and only the keys that parent interprets. Every other position
is a named `wasamoc check` error (re-checked by the loader):

| Parent  | Admitted placement keys (on a direct child)                                            | Default alignment |
|---------|----------------------------------------------------------------------------------------|-------------------|
| `Grid`  | `slot.row` / `slot.column` / `slot.row-span` / `slot.column-span` / `slot.h-align` / `slot.v-align` | `stretch` (§4.12) |
| `ZStack`| `slot.h-align` / `slot.v-align`                                                         | `center` (§4.13)  |
| any other (VStack / HStack / WrapPanel / ScrollView / Box / component) | none — placement is rejected | — |

Grid's keys are admitted **two ways** (§4.12): grouped inside a `Cell`
wrapper (where they are written *bare* — `row` / `column` / … — as the
`Cell`'s own attributes), **or** directly on the child as `slot.*`. The
two forms are mutually exclusive per child (below). ZStack admits only
the direct `slot.*` form. Unifying the surface does **not** unify the
defaults: an omitted Grid alignment falls to `stretch` (a cell is a slot
the content fills), an omitted ZStack alignment to `center` (an overlay
sits at its natural size).

#### Accepted / rejected examples (the author-visible boundary)

| Example | Disposition | Stage |
|---|---|---|
| `slot.h-align: end` on a ZStack child | accepted | — |
| `Cell { row: 1  column: 0  Box {} }` Grid child | accepted (grouped form) | — |
| `Box { slot.row: 1  slot.column: 0 }` directly under `Grid` (no `Cell`) | accepted (direct form; the widget is the Grid child) | — |
| `Box { h-align: end }` on a ZStack child — **bare** alignment, no `slot.` prefix (the M3-Phase 6 spelling) | rejected — placement must use the `slot.*` prefix; bare `h-align` reads as an unknown widget property | `wasamoc check` |
| `Cell { row: 1  slot.column: 0  Box {} }` — `slot.*` among a `Cell`'s own attributes | rejected — **mixing**: a `Cell` carries placement via its own keys, not `slot.*` | `wasamoc check` |
| `Cell { row: 1  Box { slot.column: 0 } }` — `slot.*` on a widget *inside* a `Cell` | rejected — **non-admitting parent**: the widget's parent is the `Cell` wrapper | `wasamoc check` |
| `slot.h-align: end` under a non-admitting parent (e.g. VStack) | rejected — parent admits no placement | `wasamoc check` |
| `slot.foo: …` on a ZStack child | rejected — unknown slot key | `wasamoc check` |
| `slot.h-align: some_state` (binding RHS) | rejected — placement is constant per instance | `wasamoc check` |
| `slot.h-align: end` *where a state named `end` exists* | accepted — `end` is the placement keyword, **not** the state | `wasamoc check` |
| `slot:` / `slot..h-align` / `slot.` (malformed key) | rejected — malformed placement key | parser |

Parser-stage rejects (a malformed key *shape*) are distinguished from
`wasamoc check`-stage rejects (admission / mixing / unknown key /
constant-RHS). Each row is a named diagnostic with a firing test, and
each surviving invariant is re-checked by the runtime loader.

The **mixing** reject and the **non-admitting-parent** reject are two
*distinct* diagnostics: the first fires when `slot.*` appears among a
`Cell` node's own attributes (a `Cell` carries placement through its
bare `row` / `column` / `row-span` / `column-span` / `h-align` /
`v-align` keys); the second fires when `slot.*` appears on a widget
whose immediate parent is a `Cell` (which admits no placement of its
own).

#### Placement values resolve against a closed keyword set

A placement key's right-hand side is resolved against the **closed
placement-keyword set** for that key, **not** through the state
namespace:

- `slot.h-align` / `slot.v-align` → one of `start` / `center` / `end` /
  `stretch`;
- `slot.row` / `slot.column` / `slot.row-span` / `slot.column-span` →
  integer literals (the §4.12 range rules apply).

A bare keyword like `end` is therefore **always** the placement constant
even if a state of the same name exists; placement values do not shadow
or resolve through state, so the same `.ui` cannot flip
accepted/rejected by checker ordering. Reading a state into placement
would require explicit binding-expression syntax — which is the
constant-per-instance reject below.

#### Placement is constant per instance

A placement key whose right-hand side is a state- or loop-local binding
*expression* (rather than a literal / keyword constant) is a named
`wasamoc check` error. This keeps `slot.*` from reading as a *bindable*
parent-data grammar before that surface is designed: placement is fixed
when the child is constructed and does not re-bind reactively. A `for`
body may give each generated child its own *literal* placement (CF on
the body's root child, below), but no placement key takes a binding RHS
this milestone. The rejection is explicit, not a silent drop, so the
surface does not pre-promise bindability it cannot yet honour.

#### Placement on `for` / `if`-generated children

Placement on a generated child is written exactly where it is written on
a static child: **on the body's root widget** of the `for` / `if` block.
The body root *is* a child of the placement-bearing parent, so the
generated-child placement surface is identical to the static-child
surface; a multi-widget item wraps in a container that then carries the
placement, the same as static. There is no separate placement locus on
the `for` / `if` block itself.

The placement keys available are **the admitting parent's** keys (the
admission table above) — *not* a fixed set. Of the two placement-bearing
containers, only **ZStack** admits a `for` member (§4.15: Grid rejects a
direct `for`), so a `for`-generated child carries the ZStack keys:

```
ZStack {
    for t in overlays {
        Box { slot.h-align: end  slot.v-align: start  Text { text: "\{t}" } }
    }
}
```

A `for`-generated **Grid** child (which would carry `slot.row` /
`slot.column`) is therefore **not** an M3 surface: Grid admits no `for`,
so the combination cannot be authored this milestone (§4.15 Grid `for`
reject; the deferral is recorded there). `if`-generated placement follows
the same rule — the body root carries whatever the admitting parent's
keys are.

#### Grid's `Cell` grouped form over the same model

Grid keeps the `Cell` wrapper (§4.12) as a **grouped convenience** over
this placement model: `Cell { row / column / row-span / column-span /
h-align / v-align }` and direct `slot.*` on the child express the *same*
parent-interpreted placement and are both accepted. This milestone
declares **no normative canonical form** for Grid; the examples in this
spec and in `examples/gallery/` write Grid placement with `Cell` by
convention, showing direct `slot.*` where it illustrates the shared
surface. That convention is **provisional** — a future pre-1.0 decision
fixes whether a wrapper form is retained — and is not an acceptance
criterion. ZStack has no wrapper form; its placement is always direct
`slot.*`.

#### Out of scope (M3-Phase 7b)

- **Bindable placement** — placement is constant per instance (above).
  A state- or loop-local placement that *varies after construction*
  reopens with the binding-target machinery, not as a `slot.*`-local
  addition.
- **Custom-container / custom slot keys and non-layout parent-data**
  (hit-test / focus / accessibility) — the `slot.*` namespace reserves
  these additive paths but this milestone builds neither; only Grid and
  ZStack placement keys are admitted.
- **Default-alignment unification** — Grid `stretch` / ZStack `center`
  stay per-container; each is natural for its container.
- **Placement key/value spelling revision** (e.g. `h-align` → `hAlign`)
  — existing spelling is inherited unchanged.

### 4.17 `ToggleButton` and selected / toggle state (M3-Phase 8)

**Phase status:** M3-Phase 8 closed; implementation-synced.

`ToggleButton` is a button that carries a persistent **selected /
`checked`** state. Phase 1 already proved that a boolean binding can drive a
typed widget attribute through `Button.enabled` (§4.8); `ToggleButton.checked`
is the first persistent selected-state attribute. Unlike `enabled`, which
gates interaction and disabled visuals, `checked` keeps an author-controlled
visual selection state alive across frames. An ordinary `Button` (§4.4) keeps
a single momentary / action meaning and carries **no** selected state; the
persistent toggle state lives only on `ToggleButton`, so a reader can tell a
stateful toggle from an action button by the widget type alone.

`ToggleButton` carries the **same author-facing attributes `Button`
provides** — `text`, `style`, and `enabled` (§4.8) — plus a `clicked`
handler, and it reuses Button's leaf measure / arrange. It is a new widget
*node*, **not** a new layout primitive, and it adds exactly one attribute of
its own, `checked`.

#### The `checked` attribute

| Widget | Property | Type | Default | Bindable |
|--------|----------|------|---------|----------|
| `ToggleButton` | `checked` | `bool` | `false` | yes — one-way boolean binding (§4.3) |

`checked` is bound exactly as any other bool property (§4.3): a `BOOL_LIT`
or an identifier resolving to a `bool`-typed `state`. Binding `checked` to a
`state` makes the selected visual **reactive** — when the state changes, the
visual follows.

#### Controlled, one-way

The toggle is **controlled**: a `ToggleButton` does **not** flip its own
`checked` on click. The author owns the transition — a `clicked` handler
writes the driving `state`, exactly as any other bool state is written
(§4.6). There is no widget-owned selected state and no write-back from the
widget into the bound state this milestone. The click → value → state write
is always author code.

```
component TabBar inherits Window {
    state on_photos: bool = true
    state on_albums: bool = false

    HStack {
        ToggleButton {
            text: "Photos"
            checked: on_photos
            clicked => { on_photos = true; on_albums = false; }
        }
        ToggleButton {
            text: "Albums"
            checked: on_albums
            clicked => { on_photos = false; on_albums = true; }
        }
    }
}
```

#### Selected visual

The selected state is shown as a **background-colour change only** on the
selected button; there is no border or other cue this milestone. The
selected visual is **minimal and provisional** — the full theme / styling
surface (named palettes, borders, focus rings) is a later-milestone (M5)
concern and may absorb or override the M3 selected look. Selected visuals
are not a stability commitment (§4.18).

#### `checked` admission — accepted on `ToggleButton` only

`checked` is a `ToggleButton` attribute. It is **rejected on any other
widget**, mirroring the placement-key admission model (§4.16):

| Example | Disposition | Stage |
|---|---|---|
| `ToggleButton { checked: is_on }` (`is_on` a bool state) | accepted | — |
| `ToggleButton { checked: true }` | accepted | — |
| `Button { checked: … }` | rejected — `checked` is not a `Button` attribute | `wasamoc check` |
| `Text { checked: … }` | rejected — `checked` is not a `Text` attribute | `wasamoc check` |
| `ToggleButton { checked: 1 }` (i32 RHS into a `bool` target) | rejected — type mismatch (§4.3) | `wasamoc check` |

Each row is a named diagnostic with a firing test. The surviving invariant
(`checked` on a non-`ToggleButton` node) is re-checked by the runtime IR
loader — the two-gate defence of §4.9 / §4.16, because `wasamo_load_ui`'s
memory-IR entry point does not pass through `wasamoc`.

#### Exactly-one-selected exclusion is author-composed (M3-era pattern)

There is **no built-in group / exclusive-selection construct** in M3. A tab
band where *exactly one* button is selected is expressed by composing **one
boolean state per option** and assigning them together in each handler —
each `clicked` sets its own state `true` and the others `false` (the example
above). This is an **M3-era authoring pattern**, not a canonical long-term
language design: it grows as O(N²) hand-written assignments in the number of
options. A future equality operator could allow a single-discriminant form
(one state, `checked: tab == value`); this milestone provides none, and the
per-option assignment pattern must **not** be read as a reserved or
long-term idiom (§4.18).

#### Future directions (not reserved)

Richer selection models are known and deliberately left un-designed here.
None is reserved syntax or a stability commitment (§4.18); each is a
recorded future direction:

- **Equality / single-discriminant selection** — a discriminant state with
  `checked: tab == value`, once an equality operator enters the expression
  grammar (§4.6 admits no operators today).
- **Group / exclusive-selection widgets** — a `RadioGroup` / `TabBar` /
  segmented parent that manages exclusion so the author writes no per-option
  assignment.
- **Two-way binding** — a `checked` bound two-way so a click writes the
  state with no handler; M3's binding is one-way (§4.3).
- **Widget-owned (self-toggling) state** — a `ToggleButton` that flips its
  own `checked`; M3 state is explicit and lifted, not owned inside a widget.
- **Generic toggle appearance** — a single control whose appearance
  (button / switch / checkbox) is selected by a property.

### 4.18 Public-draft future surface and provisional notes (M3-Phase 8)

This section maps surface that M3 either keeps **provisional** before 1.0 or
names as a **future direction that is deliberately not yet designed**. It
exists so a reader treats the current shape honestly: the items below are
**not reserved syntax** and **not stability commitments**. Nothing here
promises a spelling, an IR shape, or an ABI; each names a known open
question. Process triggers for reopening these questions are outside this
public draft. The M3 surface in §4.1–§4.17 is what M3 ships; this section is
the map of what is intentionally left open.

A public draft of this spec is **not** a backward-compatibility guarantee.
Public-compatibility commitments are a later-milestone (M6) concern;
documenting a current M3 shape does not freeze it.

#### Author-controllable sizing (explicit `width` / `height`)

M3 sizing is **kind-default**: each layout primitive sizes by its own rule
(Fill / Shrink / aspect-derived / track-allocated, §4.9–§4.16). Explicit,
author-controllable sizing (`width` / `height`, or an equivalent) is a known
**pre-1.0 unresolved future surface**. Its exact syntax, IR, and ABI shape
are **not reserved** — whether it is grammar-only, a modifier, layout-parent
data, runtime state, a host-construction API, or some combination is left
open for the milestone that designs it. The current kind-default behaviour
is **not** presented as final. The `aspect`-in-a-Grid-cell arrange
interaction (§4.9 / §4.12) folds into this same open question, not a
separate future feature.

#### Grid two-form placement (`Cell` vs direct `slot.*`)

Grid child placement may be authored two ways — a `Cell` wrapper or direct
`slot.*` keys — and both are valid M3 surface (§4.16). Which form, if any,
is canonical is a **pre-1.0 decision carried forward**, not settled by this
draft; the spec declares no normative canonical form.

#### Default-alignment asymmetry (Grid `stretch` / ZStack `center`)

The differing default alignments are **container-owned semantics**, not a
global rule (§4.16): a Grid cell fills its allocated track (`stretch`); a
ZStack overlay has no track-fill contract and sits at its natural size
(`center`). The asymmetry is judged **explicable** and kept. A future
layout-behavior phase could unify defaults; that is not reserved here.

#### Placement spelling and bindability

The inherited kebab-case placement spellings (`slot.h-align` /
`slot.v-align` / `slot.row-span` / `slot.column-span`, §4.16) are kept — an
**affirmative keep** for the public draft, not a silent carry. Placement is
**constant per instance**: a binding-expression RHS is rejected (§4.16), and
this draft does not promise that placement stays permanently constant or
permanently non-bindable — a bindable-placement surface, if ever designed,
is a separate future decision.

---

### 4.19 Interaction: click handling, focus, and modal focus scopes (M4-Phase 2)

**Phase status:** M4-Phase 2 closed; implementation-synced.

M4-Phase 2 makes an authored interface respond. It adds no widget kind
and no new value type: what it adds is one generalised signal, the
availability of loop binders where a handler can read them, and two
container attributes that describe how the keyboard moves.

#### Click handling on any widget

`clicked` (§4.5) is admitted on **every** widget, not only on the
Button family:

```wasamo
Box {
    fill: #2f343b
    clicked => { root.lightbox_open = true; }
}
```

The signal means the same thing everywhere: the user activated this
widget. Button-family widgets additionally paint hover / pressed states;
those are Button behaviours (§4.8), not part of the signal's meaning.

**Which widget receives a pointer event.** A pointer event resolves to
exactly **one** target: the topmost widget whose arranged rectangle
contains the point, where "topmost" is the paint order a container
already defines — within a container, later children paint over earlier
ones. Every widget with a visual is a candidate, whether or not it
carries a handler.

A layout container is therefore a candidate across its own arranged
rectangle too. If a non-clipping container overflows its parent, that
overflow remains painted and reachable and can occlude an overlapping
sibling; this is an input consequence of the candidate rule, not a
separate overflow-layout policy.

A widget is reachable only where it is painted. A container that clips
its content — `ScrollView` (§4.11), `Grid` (§4.12), `ZStack` (§4.13) —
bounds its whole subtree to its own rectangle for hit-testing as well as
for painting, so content scrolled out of a viewport receives nothing. A
container that does not clip paints its overflowing children, and those
children stay reachable where they are drawn.

Two consequences follow from those rules rather than from any additional
one:

- **A covering widget occludes what is beneath it.** A full-bleed `Box`
  declared after its siblings in a `ZStack` receives clicks that fall
  on it, and the siblings below do not — no attribute marks a widget as
  a blocker, and none is needed.
- **A disabled Button still occludes.** It does not dispatch (§4.8),
  and it does not let the click reach whatever is behind it. Having run
  no handler, it also does not end propagation: the event continues to
  its ancestors as it would from any widget without a handler.

**Propagation.** The event fires at the target and then walks its
ancestors until a handler runs. **A handler that runs consumes the
event**: propagation ends there, and no ancestor sees it. There is no
descending phase and no separate verb for stopping propagation.

Keyboard events start at the focused widget — or, when nothing is
focused, at the innermost modal focus scope — and walk the same
way.

**Handlers and state.** A handler's state writes are applied and
propagated to quiescence **once, after propagation completes**, not
between steps. The ancestor chain is fixed when the event is
dispatched, so a widget removed by a handler's own state write does not
receive the event afterwards.

#### Per-item handlers

A `signal_handler` is admitted inside a `for` body (§4.15), and the
body's binders are readable in the handler:

```wasamo
for photo, i in photos {
    Box {
        clicked => {
            root.selected_index = i;
            root.lightbox_open = true;
        }
    }
}
```

- The binders are spelled exactly as in binding position — no
  qualification, no separate namespace.
- A binder read **resolves when the handler runs**, not when the
  subtree was generated. With the positional, un-keyed identity
  baseline of §4.15, a handler therefore belongs to a *position*: after
  a collection mutation, the handler at position `n` reads whatever
  item is now at position `n`.
- A handler's registration is released with the generated subtree, on
  the same path that releases that subtree's bindings.

#### Focus

At most one widget per window holds focus. Button-family widgets are
focusable; other widget kinds are not, and the set is widened by later
milestones rather than by an authored attribute in M4. A Button with
`enabled: false` is **not** focusable — it is skipped by traversal and
cannot be reached by Tab. Button keyboard activation is not part of the
current widget surface.

- **Nothing is focused when a window opens.** No widget shows a focus
  indicator until the keyboard is used or a click places focus.
- **Tab / Shift+Tab** move focus in declaration order, wrapping at both
  ends; the first Tab lands on the first stop.
- **A click** moves focus to the nearest focusable widget at or above
  the widget it resolved to, and leaves focus unchanged when there is
  none — clicking background never clears focus.
- Losing and regaining the window's activation does not change which
  widget is focused.

A focus stop that is scrolled out of view is still a stop: traversal
reads the tree, and clipping does not change it.

#### `focus-group`

```wasamo
HStack {
    focus-group: true
    ToggleButton { text: "All" }
    ToggleButton { text: "Albums" }
    ToggleButton { text: "Favorites" }
}
```

A container marked `focus-group: true` is **one Tab stop**. Tab enters
the group and leaves it; it does not step between the members. Arrow
keys move focus **within** the group, wrapping at its ends.
`ArrowLeft` and `ArrowUp` move to the previous member;
`ArrowRight` and `ArrowDown` move to the next. Both axes are accepted.

A group remembers the member last focused inside it: leaving the group
and returning lands on that member, not on the first. Entering a group
that has not been focused before lands on its first member.

#### `modal-scope`

```wasamo
if lightbox_open {
    Box {
        modal-scope: true
        fill: #101820cc
        // ... lightbox content
    }
}
```

A container marked `modal-scope: true` confines the keyboard. While the
scope is present, Tab cycles only within its subtree, and no widget
outside it can be reached by the keyboard. Scopes nest; the innermost
one is in force.

**Being there is being open.** The attribute does not switch a scope on;
the subtree's **presence** does. A scope is entered as it appears — when
the `if` that produces it becomes true, or when the window is first
built — and left when it is removed. There is no separate act to
perform and nothing to keep in step with the tree: the way to close a
scope is to stop rendering it, which is what the `dismiss` handler below
does.

Entering does two things beyond confining traversal:

- **It remembers the focused widget**, so focus can return there. That
  is the one fact the tree cannot supply afterwards — nothing in the
  structure records what was focused before.
- **It moves focus to the scope's first stop**, so the keyboard is
  inside the scope from the moment it opens and the scope's own key
  handlers are live without the user pressing Tab first. A scope with no
  focusable widget leaves focus unset, and keys start at the scope
  itself.

When the scope leaves, focus **returns to the remembered widget**, in
preference to whatever the structure would otherwise succeed to.

What is remembered is what was focused, which is not always the widget
that opened the scope. A click on a plain `Box` does not move focus
(§Focus), so a lightbox opened by clicking a `Box` thumbnail restores to
whatever the keyboard was on beforehand — possibly nothing. Restoring to
the clicked widget requires that widget to be focusable, which arrives
with the focusability attribute a later milestone adds.

The attribute's job is to say **which** subtrees behave this way: a
container without it is an ordinary container no matter what it
contains, and only an annotated subtree becomes a scope by appearing.

#### `dismiss` — the request to close

A scope receives a **dismissal request** when the user asks for it to go
away, and the author decides what closing means:

```wasamo
if lightbox_open {
    Box {
        modal-scope: true
        dismiss => { root.lightbox_open = false; }
        // ...
    }
}
```

The request is **addressed to the innermost scope** and stops there; it
does not continue to outer scopes, so a dialog that ignores it does not
close the menu underneath. Writing no handler means the scope does not
close by dismissal — that is how a confirmation the user must answer is
expressed. Nothing is vetoed or prevented: the runtime never mutates the
tree, so not writing the state is not closing.

`dismiss` is admitted **only on a container that carries
`modal-scope: true`**. Written anywhere else it could never be raised,
so it is rejected at `wasamoc check` rather than silently never firing.

**Esc is a source of the request, not the request itself.** It is the
only source in this surface; a click outside the scope and a widget-set
dialog's close control are later sources that raise the same `dismiss`.
An author binds the intent rather than the key.

#### Keyboard input

`key-down` reacts to a physical key press. The key is named in the
declaration:

```wasamo
Box {
    modal-scope: true
    dismiss                => { root.lightbox_open = false; }
    key-down("ArrowLeft")  => { root.selected_index -= 1; }
    key-down("ArrowRight") => { root.selected_index += 1; }
}
```

It is admitted on any widget and delivered by the propagation walk
above, starting at the focused widget: the first matching handler runs
and consumes the key.

**This is a command surface, not a text-input surface.** Text reaches a
widget through the editable-text path, never through `key-down`, and
**while an input method composition is active the keyboard belongs to
the composition** — no `key-down` handler fires. **Auto-repeat is
delivered**, so a held key repeats the handler.

The recognised key names are the **named non-character keys**:
`"Escape"`, `"ArrowLeft"`, `"ArrowRight"`, `"ArrowUp"`, `"ArrowDown"`,
`"Home"`, `"End"`, `"PageUp"`, `"PageDown"`, `"Enter"`, and `"F1"` …
`"F12"`. An unrecognised name is rejected at `wasamoc check` rather
than silently never firing. Character keys and modifier combinations
(`"Ctrl+S"`) are **not** in this surface.

#### Which keys the runtime keeps

Some keys are consumed by the focus machinery before any handler sees
them:

| Key | Recipient |
|---|---|
| `Tab` / `Shift+Tab` | Always the runtime — traversal cannot be overridden |
| Arrow keys, while focus is inside a `focus-group` | The runtime (movement within the group) |
| Arrow keys, otherwise | The propagation walk |
| `Escape`, while a modal scope is present | Becomes a dismissal request on the innermost one |
| `Escape`, otherwise | The propagation walk |

The rule underneath is the ordinary one: a built-in behaviour consumes
at the focused widget, and only unconsumed keys walk to ancestors.

A key that reaches the end of the walk without a handler running is
**not** consumed by the runtime: it continues to the window's default
handling, so system keyboard behaviour is unaffected by widgets that do
not use it.

**What a scope does not do.** It confines the **keyboard** only. It
does not block pointer input: a click on content behind an open scope
is stopped by a covering widget inside the scope (the occlusion rule
above), not by the scope itself. A scope with no covering child traps
Tab and passes clicks through. Such a click cannot move focus outside
the entered scope: focus landing is bounded by the scope's traversal
root, so focus remains unchanged.

A scope also does not decide *what closing is* — see `dismiss` below.

**Accessibility.** A screen reader sees only the innermost scope's
subtree; background content is hidden by focus scope rather than by any
layering or per-widget attribute. The reading surface itself arrives
with the accessibility work in a later phase.

#### Attribute admission

| Attribute | Type | Default | Admitted on |
|---|---|---|---|
| `focus-group` | `bool` | `false` | any container |
| `modal-scope` | `bool` | `false` | any container |

Both are **constant-only**: the value must be a `true` / `false`
literal, and a binding-expression RHS is rejected — the same rule
`Box.fill` and the `WrapPanel` attributes carry (§4.9, §4.10). A modal
subtree is turned on and off by the `if` that produces it, not by
binding the attribute.

The signals this section adds are admitted as follows:

| Signal | Admitted on |
|---|---|
| `clicked` | any widget |
| `key-down("<key>")` | any widget |
| `dismiss` | a container carrying `modal-scope: true` |

Neither attribute changes layout: an annotated container measures and
arranges exactly as an unannotated one.

#### Not in this surface

Raw pointer events (`pointer-down` and siblings), hover and pressed as
authored signals (they remain Button presentation, §4.8), a `key-up`
signal, character keys and modifier combinations, a handler that
receives *every* key and decides in its body which key it was, a
structured key value, a declarative keyboard-shortcut table, a
dismissal-policy attribute distinguishing which gestures close a scope,
an attribute making a non-Button widget focusable, click-through
(opting a widget out of hit-testing), a minimum hit-target size,
scrolling a focused widget into view, and pointer capture for drag are
all outside M4-Phase 2. None is reserved by this section, and each
would arrive additively.

Two of those carry a question rather than only work. Character keys need
a choice between the **logical key** a layout produces and the
**physical position** pressed — the named non-character keys above avoid
it because for them the two coincide. A dismissal-policy attribute
becomes meaningful only once a scope has more than one dismissal
source.

The runtime-side model — how hit rectangles are obtained, where focus
state lives, how the scope stack is maintained — is normative in
[architecture.md](./architecture.md). Design provenance:
[M4-Phase 2 decisions](../process/milestone-4/phase-2/decisions/preamble.md).

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
    PropertyBind  { name: String, value: Expr },                 // M3-Phase 7b: also carries `slot.<key>` placement (§4.16)
    WidgetDecl    { type_name: String, members: Vec<Member> },
    SignalHandler { signal: String, body: Block },
    StateMember   { name: String, ty: TypeName, default: Expr },  // M2
    GridTracks    { axis: TrackAxis, tracks: Vec<TrackSize> },    // M3-Phase 5
    Conditional   { condition: Expr, body: Box<Member /* WidgetDecl only */> }, // M3-Phase 6; `if` (§4.14)
    Iteration     { binder: String, index_binder: Option<String>,
                    collection: String,
                    body: Box<Member /* WidgetDecl only */> },    // M3-Phase 7; `for` (§4.15)
}

// M3-Phase 7b. A parent-interpreted placement key `slot.<key>` (§4.16) is
// stored in the existing `PropertyBind` variant — there is NO separate
// `PlacementBind` AST variant. The parser folds `IDENT("slot") Dot IDENT(key)
// Colon expr` into `PropertyBind` when the leading IDENT is the contextual
// prefix `slot`, canonicalizing `.name` to the full dotted key WITH the
// `slot.` prefix retained (e.g. "slot.row", "slot.h-align"); `.value` is an
// `Expr` — the RHS parses as a general expr, so a state-read RHS is
// well-formed at parse time. The placement-specific rules are check-layer,
// not parser concerns: admission (which parent admits which key), the
// closed-keyword value resolution, and the constant-RHS requirement (a
// binding-expression value is a `wasamoc check` reject) all run in
// `wasamoc check` (§4.16). Only a malformed KEY shape is a parser reject.

TrackAxis (enum) { Columns, Rows }                  // M3-Phase 5

// M3-Phase 5. Permissive parse shape: the Grid track-list parser records
// the parsed token shape for every track position (including out-of-range
// and reserved-future forms) so `wasamoc check` emits precise diagnostics.
// Value-range validation (Fixed >= 1, Star weight 1..=1024) and the
// reserved-future `auto` rejection are the check layer's job, not the
// parser's; raw values are carried at the IntLit width (i64).
TrackSize (enum) {
    Fixed        { value: i64 },                    // bare integer (fixed px)
    Star         { weight: i64 },                   // `n*` or bare `*` (unit star)
    InvalidFloat { },                               // float in track position (1.5 / 1.5*)
    Word         { name: String },                  // bare word; `auto` = reserved-future
}

StringPart (enum) {
    Text(String),
    Interp(QualifiedName),
}

Expr (enum) {
    StringLit   { parts: Vec<StringPart> },
    IntLit      { value: i64 },
    FloatLit    { value: f64 },
    BoolLit     { value: bool },             // M3-Phase 1
    RatioLit    { num: i32, den: i32 },      // M3-Phase 2
    ColorLit    { value: u32 },              // M3-Phase 2; packed 0xAARRGGBB (alpha in MSB)
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

The `Member::Conditional` variant (M3-Phase 6) holds the `if` block: its
`condition` is restricted at `wasamoc check` to the narrow bool-expr
(`Expr::BoolLit` or an `Expr::Ident` resolving to a `bool`-typed state,
§4.14), and its `body` is a single `Member::WidgetDecl` (the exactly-one-
widget-child rule — a non-structural, multi-child, or nested-conditional
body is rejected). This is the landed M3-Phase 6 shape; `else` /
`switch` / `for` extend the structural-control-flow family additively
(a branch list / sibling variants) without re-shaping the existing
members.

The `Member::Iteration` variant (M3-Phase 7) holds the `for` block: the
author-named binders, the collection state name, and a single
`Member::WidgetDecl` body (§4.15). The collection literal (state
defaults and assignment RHS) and the `append` / `drop-last` method-call
expressions are companion M3-Phase 7 AST additions. Their landed
lowering maps to the textual-IR / `HandlerExpr` forms in §8.9 without
changing the §3 / §4.15 author surface.

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
| Conditional rendering (`if`)                        | Landed M3-Phase 6 as structural rendering (§4.14) |
| Iteration (`for`)                                   | Landed M3-Phase 7 as structural rendering (§4.15) |

---

---

## 8. Wasamo IR — Normative Specification (M2)

The **Wasamo IR** is the textual file format emitted by `wasamoc` and consumed
by the `wasamo-runtime` loader.  It is the contract between the two tools;
this chapter specifies it normatively.

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
- `BOOL` matches the literal text `true` or `false` (M3-Phase 1). In
  positions where the grammar accepts a `literal` or an `atom`, a bare
  `true` / `false` IDENT is recognised as `BOOL` rather than `IDENT`.
- `STRING` matches a double-quoted string with `\"` and `\\` escapes.
- `RATIO` matches `[0-9]+":"[0-9]+` (M3-Phase 2). Both sides are
  positive integer literals; zero or negative sides are rejected by
  `wasamoc check` and independently by the runtime IR loader
  (see §8.11), and are structurally unreachable in valid IR.
- `COLOR` matches `"#"[0-9A-Fa-f]{6}` or `"#"[0-9A-Fa-f]{8}` (M3-Phase
  2). The 6-digit form encodes RGB with alpha implicitly `0xFF`; the
  8-digit form encodes RGBA with explicit alpha. Both surface forms
  lower to a packed `u32` in `0xAARRGGBB` layout (alpha in the most
  significant byte): `#RRGGBB` → `0xFFRRGGBB`,
  `#RRGGBBAA` → `0xAARRGGBB`.
- Whitespace (space, tab, `\r`, `\n`) is ignored between tokens.
- A `;` outside the header line begins a line comment; the rest of that line
  is ignored.

### 8.3 Top-level grammar

```
ir_file        ::= header component_def EOF

header         ::= ";wasamo-ir v0" NEWLINE

component_def  ::= "component" IDENT "inherits" IDENT
                   "{" component_body "}"

component_body ::= component_member*
component_member ::= state_decl | host_property_set | host_binding | widget_node
```

One `component_def` per IR file (matches the M2 single-component restriction).
Exactly one top-level `widget_node` is admitted; it is the **content
root**. Component-level host attributes live beside it on the component
host surface; they are not children or properties of the content root.
`wasamoc` emits states first, host entries next, and the content root
last.

```
host_property_set ::= "host" "prop" IDENT "=" literal
host_binding      ::= "host" "bind" IDENT "=" expr
```

In M3-Phase 6 the Window host-attribute catalog is `title`, `backdrop`,
and `theme`. `title` must be a string literal; `backdrop` and `theme`
must be keyword identifiers. `host bind` is part of the structural IR
surface so the component host surface is explicit, but every host
binding is rejected in M3-Phase 6 (`WASAMO_ERR_IR_MALFORMED`, §8.11).

### 8.4 State declarations

`state` declarations encode the Signal ownership transferred from the DSL.
The runtime allocates a `Signal<T>` for each one.

```
state_decl   ::= "state" IDENT ":" type_name "=" (literal | list_literal)

list_literal ::= "[" (literal ("," literal)*)? "]"    ; M3-Phase 7
```

| Element     | Meaning                                          |
|-------------|--------------------------------------------------|
| `IDENT`     | Signal name; unique within the component (flat namespace) |
| `type_name` | `"i32"`, `"string"`, `"bool"` (M3-Phase 1 adds `bool`), or `"i32[]"` / `"string[]"` / `"bool[]"` (M3-Phase 7) |
| `literal`   | Default value: `INT` for `i32`; `STRING` for string; `BOOL` (`true`/`false`) for `bool` |
| `list_literal` | Collection default (M3-Phase 7): scalar literals matching the declared element type; possibly empty; no nesting, no idents |

Examples:

```
state count: i32 = 0
state ready: bool = false
state thumbs: i32[] = [101, 102, 103]
state captions: string[] = []
```

The M3-Phase 7 collection forms above are the landed token spellings.
Collection state declarations and list-literal defaults **roundtrip
losslessly** through emit → load, and the loader rejects an
element-type mismatch, a nested list, a list default on a scalar state,
and a scalar default on a collection state as
`WASAMO_ERR_IR_MALFORMED` (§8.11).

### 8.5 Widget nodes and control-flow members

A `node_body` holds two kinds of structural member: **widget nodes**
(`node` blocks, which materialise a runtime widget) and, from
M3-Phase 6, **control-flow members** (the `if` construct, which the
runtime *interprets* rather than *renders* — it materialises no widget).

```
widget_node ::= "node" IDENT "{" node_body "}"

node_body   ::= ( track_decl
                | property_set
                | binding
                | handler
                | widget_node
                | placed_child                 ; M3-Phase 7b; see below
                | control_flow_member )*       ; control_flow_member is M3-Phase 6

; M3-Phase 7b. A child carrying parent-interpreted placement (§4.16).
; The child node is wrapped in a `child` record carrying its placement
; payload; a placement-free child stays a bare `widget_node`. Both Grid
; authoring forms (a `Cell` wrapper and direct `slot.*`) and the ZStack
; `slot.*` form lower to this one record — the `Cell` wrapper does NOT
; survive as a node in textual IR.
placed_child ::= "child" "{" placement_decl widget_node "}"
placement_decl
             ::= "placement" placement_kind "{" placement_entry* "}"
placement_kind ::= "grid" | "zstack"
placement_entry ::= IDENT "=" ( INT | IDENT )   ; constant only; key in the
                                                ; kind's admitted set (§4.16)
```

`IDENT` is the widget type (e.g. `Window`, `VStack`, `Text`, `Button`,
`ToggleButton`). A `ToggleButton` (§4.17) is an ordinary `node` whose
`checked` boolean rides the existing bool binding forms (§8.6 / §8.7); it
introduces no new IR grammar.
Children appear as nested `node` blocks in document order; a child that
carries parent-interpreted placement (a Grid or ZStack child, §4.16)
appears as a `child { placement … node … }` record instead. A
`control_flow_member` is **not** a `node` — it is a structural operator
over members (see *Control-flow members* below).

**Grid track declarations (M3-Phase 5).** A `track_decl` carries a
`Grid` node's `columns:` / `rows:` track lists. It is emitted only on
`Grid` nodes; the loader rejects a `tracks` line on any non-`Grid`
node (track lists live in a
Grid-specific kind payload on the IR node, never in a `prop` entry, so
`IrLiteral` stays the sole `property_set` carrier):

```
track_decl ::= "tracks" axis "=" track_list
axis       ::= "columns" | "rows"
track_list ::= track ( track )*     ; whitespace-separated, >= 1 track
track      ::= INT                  ; fixed track, INT >= 1
             | INT "*"              ; weighted-star track, weight in [1, 1024]
             | "*"                  ; unit star, sugar for "1*"
```

`wasamoc` emits each axis on its own line at the top of the Grid node
body, before any `prop` / child `node`, with the unit star
canonicalised to `1*`:

```
node Grid {
    tracks columns = 180 1* 2*
    tracks rows = 1* 1*
    child {
        placement grid { row = 0  column = 0 }
        node Text { prop text = "header" }
    }
}
```

Unlike the author-surface DSL (§4.12), the runtime IR `tracks` grammar
is **whitespace-insensitive**: `INT "*"` lowers to `Star(weight)`
whether or not the `*` is adjacent, because the author-surface
`1*`-vs-`1 *` distinction is resolved at `wasamoc` compile time and the
canonical machine format always emits the explicit weight.

**Placement is normalised to the child-slot record (M3-Phase 7b).** Both
Grid authoring forms — the `Cell` wrapper and direct `slot.*` — and the
ZStack `slot.*` form lower to **one** `child { placement <kind> { … }
node … }` record (`<kind>` ∈ `{ grid, zstack }`); the placement keys ride
`= `-separated constant entries (`INT` / `IDENT`) in the admitted set for
that kind (§4.16). The `Cell` wrapper does **not** survive as a node in
textual IR — it is an author-surface grouping form (§4.12), normalised
away at emit, so the runtime model carries placement on the child slot
with no parallel vector (the storage model is normative in
[architecture.md §6.8.6](./architecture.md#686-child-slot-placement-storage-slotdata-m3-phase-7b)). A
ZStack child's record uses `placement zstack { h-align = … v-align = … }`:

```
node ZStack {
    node Box { prop fill = #00000080 }
    child {
        placement zstack { h-align = end  v-align = start }
        node Text { prop text = "badge" }
    }
}
```

The grammar skeleton (the `child` / `placement <kind>` / `node` keywords
and nesting, the `<kind>` set, and constant-only values) is normative;
inter-entry separators and key ordering inside a placement block are
emitter trivia.

**Stale-form rejection (reject + regenerate).** The textual IR is a
build-internal artifact `wasamoc` regenerates every build, so the loader
is **single-form**: old-form placement IR — a `node Cell { prop row = …
}` wrapper, or bare ZStack placement `prop` lines on a child — is
rejected with a **named loader diagnostic** (not silently slot-ised), and
the build re-emits the canonical `child { … }` record. No dual-parse
transition window is carried, matching the no-long-lived-alias migration
stance.

**Control-flow members (M3-Phase 6; `for` added M3-Phase 7).** A
`control_flow_member` encodes a structural control-flow construct in
the node body. Phase 6 ships the single-branch `if`; Phase 7 adds the
`for` member:

```
control_flow_member ::= "if" cond "{" widget_node "}"   ; Phase 6: exactly one
                                                        ; widget node — no else,
                                                        ; no nested control flow
                     |  "for" IDENT ("," IDENT)? "in" IDENT
                        "{" widget_node "}"             ; M3-Phase 7: element
                                                        ; binder, optional index
                                                        ; binder, collection
                                                        ; state; exactly one
                                                        ; widget node
cond                ::= BOOL | IDENT   ; BOOL → bool-literal condition
                                       ; IDENT → bool-typed state read
```

The control-flow member is **IR-only**: like `Cell` it materialises no
runtime widget and no Visual — the loader *interprets* it to build a
conditional binding that makes the body subtree present / absent when
the condition Signal changes (the reactive mechanism is normative in
[architecture.md](./architecture.md)). Its condition rides the existing
`HandlerExpr` machinery (`BoolLit` for a literal, `BoolPropRead` for a
bool-typed state read, §8.9), so `IrProp.value` stays strictly
`IrLiteral`. In the loaded IR the control-flow member sits **alongside**
widget members in the parent's ordered child list — control flow is a
first-class structural member, not a widget node — carrying its branch
condition and its single-widget body; the schema shape is in
[architecture.md](./architecture.md). The construct is designed to carry
a branch list so `else` / `switch` (more branches) are same-family
additions; `for` landed in M3-Phase 7 as the second control-flow member
(its own sibling variant — see *The `for` member* below). Phase 6 emits
and loads **exactly one branch** with a **single-widget body**.

Worked example — `.ui` → textual IR for the lightbox slice
(`if is_lightbox_open { ZStack { … } }`):

`.ui`:

```
component Gallery inherits Window {
    state is_lightbox_open: bool = false
    title: "Gallery"
    WrapPanel { /* thumbnails */ }
    if is_lightbox_open {
        ZStack {
            Box { fill: #00000080 }
            Box {
                aspect: 4:3
                Text { text: "photo" }
            }
        }
    }
}
```

textual IR (`wasamoc` emit):

```
component Gallery inherits Window {
    state is_lightbox_open: bool = false
    host prop title = "Gallery"
    host prop backdrop = mica
    host prop theme = system

    node Window {
        node WrapPanel { /* … */ }
        if is_lightbox_open {
            node ZStack {
                node Box { prop fill = #00000080 }
                node Box {
                    prop aspect = 4:3
                    node Text { prop text = "photo" }
                }
            }
        }
    }
}
```

loaded IR (the runtime carries control flow as a member-level construct
alongside widget members in the parent's ordered child list — the
normative schema is in [architecture.md](./architecture.md)):

```
IrNode { widget_type: "Window", children: [
    Widget(IrChildSlot { node: IrNode { widget_type: "WrapPanel", … }, slot_data: None }),
    ControlFlow(ControlFlowNode::If { branches: [
        Branch {
            condition: HandlerExpr::BoolPropRead("is_lightbox_open"),
            // body member is a Widget(IrChildSlot { node, slot_data }); the ZStack
            // is a Window child, so slot_data: None (placement-free parent, §4.16)
            body: [ Widget(IrChildSlot { node: IrNode { widget_type: "ZStack", … }, slot_data: None }) ],
        },
    ] }),
] }
```

The `if` member appears at the same document position inside the parent
node body as it does in the `.ui` source, between the static
`node WrapPanel` and the end of the `Window` body — in the loaded IR, as
the `ControlFlow(…)` member between the `Widget(WrapPanel)` member and
the end of `Window`'s child list. `wasamoc` emit and the runtime loader
both preserve the branch condition and the single-widget body across an
emit → load roundtrip. Loader validation of malformed control-flow
members (multi-branch, multi-child / non-structural / nested-control-flow
body, non-bool / unresolved condition) is in §8.11.

**The `for` member (M3-Phase 7).** Like the `if` member, a `for` member
is IR-only: it materialises no widget and no Visual of its own — the
loader interprets it. The landed textual-IR spelling is:

```
for <binder> in <collection> { node ... }
for <binder>, <index-binder> in <collection> { node ... }
```

The normative properties are:

- the **binders, the collection reference, and the body roundtrip
  losslessly** through emit → load;
- the loader enforces — the post-`in` reference resolves to a declared
  **collection-typed state**; the body is **exactly one widget node**;
  the binders are well-formed (non-empty, distinct, not colliding with
  a state name) — each violation `WASAMO_ERR_IR_MALFORMED` (§8.11), the
  dual gate with `wasamoc check`;
- a `for` member's **declared slot is present at load time** with its
  initial cardinality materialised from the collection's initial value
  (0..N children at load). The empty-initial case materialises **zero
  children with the member still live** — it must not be conflated with
  "member absent".

Worked example — `.ui` → textual IR → loaded IR, with an `if` sibling
so declared-slot offsets are exemplified:

`.ui`:

```
component Gallery inherits Window {
    state thumbs: i32[] = [101, 102]
    state show_footer: bool = true

    VStack {
        Text { text: "Header" }
        for thumb in thumbs {
            Box { Text { text: "Photo \{thumb}" } }
        }
        if show_footer {
            Text { text: "Footer" }
        }
    }
}
```

textual IR (`wasamoc` emit):

```
component Gallery inherits Window {
    state thumbs: i32[] = [101, 102]
    state show_footer: bool = true
    host prop backdrop = mica
    host prop theme = system

    node Window {
        node VStack {
            node Text { prop text = "Header" }
            for thumb in thumbs {
                node Box {
                    node Text { bind text = (interp "Photo " (item-read thumb)) }
                }
            }
            if show_footer {
                node Text { prop text = "Footer" }
            }
        }
    }
}
```

loaded IR (schema shape normative in
[architecture.md](./architecture.md)):

```
IrNode { widget_type: "VStack", children: [
    Widget(IrChildSlot { node: IrNode { widget_type: "Text", … }, slot_data: None }),  // declared slot 0
    ControlFlow(ControlFlowNode::For {                          // declared slot 1
        binder: "thumb",
        index_binder: None,
        collection: HandlerExpr::ListPropRead { path: "thumbs", elem: I32 },
        // body member is a Widget(IrChildSlot { node, slot_data }); the Box is a
        // VStack child, so slot_data: None (placement-free parent, §4.16)
        body: [ Widget(IrChildSlot { node: IrNode { widget_type: "Box", … }, slot_data: None }) ],
    }),
    ControlFlow(ControlFlowNode::If { … }),                     // declared slot 2
] }
```

At load, the VStack materialises "Header" at offset 0, two `Box`
subtrees at offsets 1..2 (the `for` slot's initial cardinality is the
initial collection length, 2), and — `show_footer` being true —
"Footer" at offset 3. The materialised offset of each declared slot is
the sum of the live cardinalities of the declared members before it
(widget = 1, `if` = 0/1, `for` = current collection length), recomputed
at every structural mutation, never cached: after
`thumbs = thumbs.append(103);` the `for` slot covers offsets 1..3 and
"Footer" sits at offset 4.

### 8.6 Property sets

A `property_set` writes a static value to a widget property at load time.
It is used for properties whose value is a plain literal (not reactive).

```
property_set ::= "prop" IDENT "=" literal

literal      ::= INT | STRING | BOOL | RATIO | COLOR | IDENT
```

The `IDENT` alternative encodes keyword-valued properties such as
`mica`, `system`, `accent`, `title` (see §4.3). The `BOOL`
alternative is M3-Phase 1: a bare `true` or `false` in literal
position is interpreted as `IrLiteral::Bool` (per §8.2 Notation),
not as an `IDENT`-valued literal. The `RATIO` and `COLOR`
alternatives are M3-Phase 2: a literal in `<num>:<den>` form is
`IrLiteral::Ratio { num, den }` and a literal in `#RRGGBB` /
`#RRGGBBAA` form is `IrLiteral::Color(<packed u32>)`.

**Box-internal materialisation (M3-Phase 2 only).** When the
enclosing `node` is `Box` and the property is `aspect` or `fill`, the
runtime IR loader materialises the literal into Box-internal domain
types (`Ratio` / `Color` on `WidgetData::Box`) **directly**, without
constructing a `PropertyValue` variant. M3-Phase 2 deliberately does
not add `PropertyValue::Ratio` or `PropertyValue::Color`, so the C
ABI surface (`read_property_value` / `write_property_value` /
`property_value_to_owned` and the `WASAMO_VALUE_*` tag space) is
untouched. When a later phase opens bindable `aspect` or `fill`,
the corresponding `PropertyValue` variants, `WASAMO_VALUE_*` tags,
and `abi.rs` arms land together in that phase.

Examples:

```
prop spacing = 12
prop padding = 24
prop text = "Counter"
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
handler ::= "on" IDENT ("(" STRING ")")? "{" expr "}"
```

`IDENT` is the signal name (e.g. `clicked`). The optional string
argument is the signal argument; M4-Phase 2 defines it for
`key-down("<key>")`.

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
1-to-1 to a `HandlerExpr` variant.

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
        | "(" "bool-prop-read"  IDENT ")"     ; M3-Phase 1
        | "(" "list-prop-read"  IDENT ")"     ; M3-Phase 7
        | "(" "item-read"       IDENT ")"     ; M3-Phase 7
        | "(" "index-read"      IDENT ")"     ; M3-Phase 7
        | "(" "list-append"     IDENT expr ")"  ; M3-Phase 7
        | "(" "list-drop-last"  IDENT ")"     ; M3-Phase 7
        | list_literal                        ; M3-Phase 7 (§8.4)
        | "(" "assign"          IDENT expr ")"
        | "(" "compound-assign" compound_op IDENT expr ")"
        | "(" "interp"          interp_part+ ")"
        | "(" "block"           expr* ")"

atom  ::= INT
        | STRING
        | BOOL                                ; M3-Phase 1

compound_op ::= "+=" | "-=" | "*=" | "/="

interp_part ::= STRING         ; literal text fragment
              | "(" expr ")"   ; embedded expression (re-uses the expr rule)
```

**Mapping to `HandlerExpr` variants:**

| IR form | `HandlerExpr` variant | Notes |
|---|---|---|
| `INT` / `(lit INT)` | `IntLit(i32)` | Bare `INT` is equivalent to `(lit INT)` |
| `STRING` / `(str STRING)` | `StrLit(String)` | Binding-only |
| `BOOL` | `BoolLit(bool)` | M3-Phase 1. Bare `true` / `false` in atom position. No `(bool …)` wrapper form is defined |
| `(prop-read NAME)` | `PropRead { path }` | `NAME` is the Signal name from `state` |
| `(str-prop-read NAME)` | `StrPropRead { path }` | String-typed binding read; `NAME` is the Signal name from `state` |
| `(bool-prop-read NAME)` | `BoolPropRead { path }` | M3-Phase 1. Bool-typed binding read; `NAME` is the Signal name from `state` |
| `(assign NAME expr)` | `Assign { lhs, rhs }` | Handler-only. M3-Phase 1: `rhs` may now be `BoolLit` or `BoolPropRead` when the LHS state is `bool`-typed. M3-Phase 7: when the LHS state is collection-typed, `rhs` is exactly one of `(list-append …)` / `(list-drop-last …)` (receiver = the LHS state) or a `list_literal` |
| `(compound-assign OP NAME expr)` | `CompoundAssign { lhs, op, rhs }` | Handler-only. **Not defined over `bool`** — no `CompoundOp` is naturally bool-typed. **Not defined over collections** (M3-Phase 7) |
| `(interp part+)` | `Interpolation(Vec<InterpolationPart>)` | Binding-only |
| `(block expr*)` | `Block(Vec<HandlerExpr>)` | Empty block evaluates to `0` |
| `(list-prop-read NAME)` | `ListPropRead { path, elem }` | M3-Phase 7. Whole-value collection read carrying the element type tag; the `for` member's collection reference |
| `(item-read NAME)` | `ItemRead { binder }` | M3-Phase 7. Loop-local element-binder read; binding / interpolation positions inside a `for` body only. Evaluates as a live positional read of the collection signal at the subtree's instantiation position; an out-of-range position writes nothing (the defined same-batch-removal case) |
| `(index-read NAME)` | `IndexRead { binder }` | M3-Phase 7. Loop-local index-binder read (`i32`, zero-based); same positions as `item-read` |
| `(list-append NAME expr)` | collection tail-append expression | M3-Phase 7. Pure: evaluates to a new collection with one element appended; collection-assignment RHS only |
| `(list-drop-last NAME)` | collection tail-removal expression | M3-Phase 7. Pure and total: the collection minus its last element; identity on empty; collection-assignment RHS only |
| `list_literal` | static collection value | M3-Phase 7. State defaults (§8.4) and collection-assignment RHS (whole-value reset / clear) |

The M3-Phase 7 rows are the landed spellings and map to the single
unified `HandlerExpr` enum (no side enum). A collection assignment
evaluates as read-modify-write (or, for the literal, a direct
whole-value set) on the whole-value collection signal; an assignment
whose new value equals the current value performs no dirty propagation
(§4.15).

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
    ; Signal declarations: state ownership in .ui
    state count: i32 = 0

    ; Host-owned component attributes
    host prop title = "Counter"
    host prop backdrop = mica
    host prop theme = system

    ; Content root node
    node Window {
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

### 8.11 Loader validation policy

The runtime loader (`wasamo-runtime/src/ir_loader.rs`) applies
defense-in-depth validation:

| Check | Enforced at load | On failure |
|---|---|---|
| Header line matches `;wasamo-ir v0` | Yes | `WASAMO_ERR_IR_MALFORMED` |
| Top-level structure is `component_def` | Yes | `WASAMO_ERR_IR_MALFORMED` |
| Every `prop-read` / `str-prop-read` / `assign` / `compound-assign` name resolves to a declared `state` | Yes | `WASAMO_ERR_IR_MALFORMED` |
| `Box` node has at most one child (M3-Phase 2) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| `RATIO` literal has `num > 0` and `den > 0` (M3-Phase 2) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| `WrapPanel` `item-cross-size`, `item-spacing`, and `line-spacing` are non-negative `i32` (M3-Phase 3) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| `ScrollView` node has exactly one content child (M3-Phase 4) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| `Grid` declares at least one row and at least one column; each fixed track value is `>= 1`; each star weight is in `[1, 1024]` (M3-Phase 5) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| **Grid placement payload** (normalized `SlotData::Grid`, M3-Phase 7b; not a `Cell` node): a placed child's row in `[0, rows.len())`, column in `[0, columns.len())`, row-span/column-span `>= 1` with resolved rectangle within declared track count; no two placed children in the same Grid share any resolved cell; alignment in `{ start, center, end, stretch }` (M3-Phase 5 invariants on the M3-Phase 7b storage) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| **ZStack payload**: `ZStack` declares no ZStack-level attributes; its per-child placement (`slot.h-align` / `slot.v-align`, M3-Phase 7b) is in `{ start, center, end, stretch }` (M3-Phase 6) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| **Shared placement admission** (§4.16, M3-Phase 7b): a placement payload is admitted only under a placement-admitting parent (Grid, ZStack) and rejected elsewhere; one placement form per Grid child (mixing rejected); a placement value is a constant, not a binding expression (loaded-IR placement representation normative in [architecture.md §6.8.6](./architecture.md#686-child-slot-placement-storage-slotdata-m3-phase-7b)) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| **Stale placement-IR form** (IR-B, M3-Phase 7b): a `node Cell { … }` wrapper or a bare-placement-`prop` form surviving in the loaded IR (the pre-normalization shape) is rejected with a named diagnostic and regenerated, not slot-ised (§8.5). The `Cell` **structural** rules (exactly one content child; `Cell` only under `Grid`) are `wasamoc check` / `.ui`-source rules, not loader invariants — the loader never sees a `Cell` node | Yes | `WASAMO_ERR_IR_MALFORMED` |
| A control-flow (`if`) member carries **exactly one branch** (no `else` until specified), a **single-widget body** (not empty, not multiple children, no non-structural body member, no nested control-flow member), and a **bool-typed, resolved** condition; an `if` appears only where a member is admitted inside a widget body (not at component level) (M3-Phase 6) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| Component host attributes are only `title`, `backdrop`, and `theme`; `title` must be a string literal; `backdrop` and `theme` must be keyword identifiers; host bindings are rejected; the same names are rejected if squatted as props or bindings on the content root (M3-Phase 6) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| Collection state declarations: a list default appears only on a collection-typed state (and a scalar default only on a scalar state); list elements are scalar literals matching the declared element type; no nested lists (M3-Phase 7) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| A `for` member carries a collection read resolving to a declared collection-typed state, a **single-widget body**, and well-formed binders (non-empty, distinct, not colliding with a state name); a `for` appears only under VStack / HStack / WrapPanel / ZStack (not ScrollView / Box / Grid, not at component level); no `for` member at any depth inside a `for` body; no handler member inside a `for` body (M3-Phase 7) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| Collection assignment: `=` only on a collection-typed LHS; the RHS is a single self-receiver tail-edit form or a static list literal with matching element types; loop-local reads (`item-read` / `index-read`) appear only in binding positions inside the owning `for` body (M3-Phase 7) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| Binding expression result type matches target property type | **No** (trusted from `wasamoc`) | Undefined behaviour |
| Per-node emitter invariants (e.g. `on` only on signal-capable widgets) | **No** (trusted from `wasamoc`) | Undefined behaviour |

The loader trusts type-level invariants established by `wasamoc`'s check pass.
Type mismatches indicate a `wasamoc` bug, not a recoverable load-time error.

The M3 rows above (Phase 2 `Box` child count, Phase 2 `RATIO` sign,
Phase 3 WrapPanel non-negative attributes, Phase 4 ScrollView
single-content-child rule, Phase 5 Grid track / placement-payload /
span / conflict / alignment-vocabulary invariants, Phase 6 ZStack
attribute / placement-payload invariants, the Phase 7b shared placement
admission + stale-form rejection, Phase 6 control-flow
(`if`) branch / body / condition invariants, Phase 6 host-surface
catalog / value-shape / binding / content-root-separation invariants,
and Phase 7 collection-state / `for`-member / collection-assignment
invariants) are explicitly dual-gated rather than trusted because
`wasamo_load_ui`'s memory-IR entry point does not pass through
`wasamoc`; the runtime gate is the last line of defence for these spec
invariants. See §4.9 for the Box child-count rationale, §8.2 for the
`RATIO` surface constraint, §4.10 for the WrapPanel attribute range,
§4.11 for the ScrollView child-count rule, §4.12 for the Grid
placement-payload / loader rejection rules, §4.13 for the ZStack
attribute / placement rules, §4.16 for the shared placement admission /
constant-RHS / stale-form rules, and §4.14 for the conditional `if`
branch / body / condition rules — all of which `wasamoc check` already
enforces (the `Cell` **structural** rules — one content child, `Cell`
only under `Grid` — are `wasamoc check`-only, since the loader sees the
normalized placement, not a `Cell` node). The host-surface
rules guard the direct-IR-loader entry as well as `.ui` lowering: `.ui`
authors see the compiler diagnostics first, while hand-authored textual
IR reaches the same malformed-IR boundary.

Phase 5 Grid invariants are **reject-at-validate**, not
clamp-at-arrange: placement and span values have no defensible
clamped interpretation (a silently-clamped placement would displace
legitimately-placed siblings and produce order-dependent layout).
The only layout-time gate Phase 5 introduces is
`LayoutError::GridUnboundedStarAxis` (§4.12), which is not a
`validate()`-time concern because it depends on the parent's axis
bound.

### 8.12 Scope out (post-M2)

| Feature | Deferred to |
|---|---|
| `(computed ...)` expression form | M3 |
| Conditional rendering | **Landed M3-Phase 6** — not as an `(if …)` expression / binding form, but as a structural **control-flow member** in the node body (§8.5; `if` only, single branch, single-widget body). `else` / `switch` are reserved family members; `for` iteration **landed M3-Phase 7** as the second control-flow member (§8.5) |
| M3 expanded type set (`float`, user types; `bool` landed in M3-Phase 1) | M3 |
| Generic `TypedValue` value union | Post-M3 |
| Bindable surface for Box `aspect` / `fill` (M3-Phase 2 admits the literals only) | Future phase that first needs reactive aspect or fill |
| `IrType::Ratio` / `IrType::Color` (M3-Phase 2 stores them Box-internal, not as `PropertyValue` variants) | Same future phase as above |
| Binary IR format | Post-M2 |
| Grammar version `v1` (first incompatible change) | When required |
| `(post-event ...)` escape hatch for observer callbacks | M3 |

**`TypedValue` deferral.** M3-Phase 1 admits `bool` as a third
tagged scalar (`IrType::Bool`, `IrLiteral::Bool`, `HandlerExpr::BoolLit` /
`BoolPropRead`) using the same type-suffixed variant pattern M2
adopted for `i32` and `String`. It does not introduce
a generic `TypedValue` value union; the per-type binding evaluator and
per-type widget property writer (see `architecture.md` §6.7.7)
are the structural form of that deferral. This deferral is recorded in
[notes/m3/m3-start-framing.md §F5](../process/milestone-3/requirements/framing.md#f5--typedvalue-は再評価候補だが開始時点の-m3-acceptance-ではない).
M3-Phase 7 re-judged the deferral at the collection surface and did not
adopt: collection elements are the existing three scalars, every value
position stays monomorphic at lowering time, and element typing rides
type tags (a single typed collection read carrying its element tag),
not a runtime union. The genuine `TypedValue` driver — structured item
fields (`item.field`, record-like values) — is out of the Phase 7
surface.

## Public draft change history

This section is the public-draft promotion record — the public-draft
anchor — distinct from the per-edit revision-history table below.

- **2026-07-06 — promoted to `public-draft` at M3 close (M3-Phase 8
  Moment 2).** The M3 surface (§4.1–§4.18) matches the landed
  implementation, and the M3-Phase 8 external-reader smoke recorded a
  "yes" verdict for every M3 surface (Grid, WrapPanel, ZStack,
  ScrollView, Box `aspect` / `fill`, conditional rendering, iteration
  and collections, `bool` scalar binding, `ToggleButton` / `checked`,
  parent-interpreted placement, and the integrated Gallery path): a
  reader with only this document can reproduce each surface against a
  hypothetical host that already provides the C ABI. A public draft is
  **not** a backward-compatibility guarantee; public-compatibility
  commitments are a later-milestone concern (§4.18). Deciding records
  (per M3 phase):
  [Phase 1 — `bool` scalar binding](../process/milestone-3/phase-1/decisions/preamble.md);
  [Phase 2 — Box](../process/milestone-3/phase-2/decisions/preamble.md);
  [Phase 3 — WrapPanel](../process/milestone-3/phase-3/decisions/preamble.md);
  [Phase 4 — ScrollView](../process/milestone-3/phase-4/decisions/preamble.md);
  [Phase 5 — Grid](../process/milestone-3/phase-5/decisions/preamble.md);
  [Phase 6 — ZStack + conditional rendering](../process/milestone-3/phase-6/decisions/preamble.md);
  [Phase 7 — iteration](../process/milestone-3/phase-7/decisions/preamble.md);
  [Phase 7b — parent-interpreted placement](../process/milestone-3/phase-7b/decisions/preamble.md);
  [Phase 8 — selected state + Gallery + public draft](../process/milestone-3/phase-8/decisions/preamble.md).

## Revision history

| Version | Date       | Notes                                                                             |
|---------|------------|-----------------------------------------------------------------------------------|
| 0.1     | 2026-04-27 | Initial draft (Phase 1, pending owner agreement)                                  |
| 0.2     | 2026-04-27 | Phase 1 Accepted; added missing tokens (MinusEq/StarEq/SlashEq); corrected AST types (StringLit → Vec<StringPart>, Statement as struct); corrected error output format |
| 0.3     | 2026-05-07 | M2-Phase 6 Accepted; added the §8 Wasamo IR normative spec. |
| 0.4     | 2026-05-11 | M2 complete; added the `str-prop-read` IR form and finalised M2 status language. |
| 0.5     | 2026-05-19 | M3-Phase 1: `bool` scalar binding — `true`/`false` keywords, bool IR forms, `bool` state type, the `Button.enabled` entry, and the retroactive `state` surface (§4.7); recorded the `TypedValue` deferral (§8.12). |
| 0.6     | 2026-05-19 | M3-Phase 1: documented that bool-typed state interpolation is rejected until a display-conversion surface exists. |
| 0.7     | 2026-05-20 | M3-Phase 2 design draft: §4.9 Box layout primitive (`aspect`/`fill`, single-child centred-and-clipped, aspect inscribed-fit) plus the `RatioLit`/`ColorLit` token, grammar, AST, and IR additions. |
| 0.8     | 2026-05-20 | M3-Phase 2 close: §4.9 implementation-synced; no divergence found. |
| 0.9     | 2026-05-21 | M3-Phase 3 design draft: §4.10 WrapPanel layout primitive (line-flow measure-arrange; `item-cross-size`/`item-spacing`/`line-spacing`); reuses existing `i32` plumbing. Dropped the stale `M1` qualifier from the §4.4 registry lead-in. |
| 1.0     | 2026-05-22 | M3-Phase 3 close: §4.10 implementation-synced; folded the §2.2 lexer surface (kebab-case `Ident`, optional leading `-` on `IntLit`). |
| 1.1     | 2026-05-25 | M3-Phase 4 design draft: §4.11 ScrollView layout primitive (parent-supplied viewport, clip, `offset-y`, intermediate content Visual); reuses existing `i32` plumbing. |
| 1.2     | 2026-05-25 | M3-Phase 4 close: §4.11 implementation-synced; fixed the §4.9 Box examples to the parser-accepted member-per-line form (semicolon member separator left as an open question). |
| 1.3     | 2026-05-29 | M3-Phase 5 design draft: §4.12 Grid layout primitive (`Cell` placement/span/alignment, fixed + weighted-star tracks, track resolution, outer-bounds clip); reuses existing plumbing (track lists ride a Grid kind payload). Made §8.11 the full M3 loader-validation aggregate (added the Phase 3/4/5 rows). |
| 1.4     | 2026-05-30 | M3-Phase 5 close: §4.12 implementation-synced; folded the deferred Grid textual-IR grammar (§8.5 `track_decl`) and re-synced §5 AST / §2.2 tokens / §3 grammar to the landed parser. `abi_spec.md` untouched. |
| 1.5     | 2026-06-02 | M3-Phase 6 design draft (Moment 1): added §4.13 ZStack overlay primitive (union sizing + `Fill/Fill` default, document-order z-order, per-child alignment, outer-bounds clip) and §4.14 conditional rendering — the first chapter of the structural rendering model (`if` block, structural present/absent, absent=fresh-on-return with opt-in future retention). Supporting: §2.1 `if`/`else`/`switch`/`for` keyword reservation, §3 grammar, §5 AST, §8.5 control-flow member with textual + loaded IR examples, §8.11 validation rows. No new `IrType`/`IrLiteral`/`PropertyValue` or C ABI change; `abi_spec.md` untouched (the conditional + runtime-mechanism schema is normative in `architecture.md`). Also slimmed this revision history and applied the Living-spec vocabulary discipline retroactively — removed DD / option / process labels from the spec body and these notes, keeping the `M3-Phase N` identifiers (full provenance lives in the process documents). Pending implementation re-sync at Phase 6 close. |
| 1.6     | 2026-06-02 | Moved the historical M1 lexical rationale appendix into `process/milestone-1/phase-1/decisions/`; this spec now keeps only the normative DSL surface. |
| 1.7     | 2026-06-08 | M3-Phase 6 close: §4.13 and §4.14 marked implementation-synced; textual IR re-synced to the landed control-flow member and component host surface. Component-level Window host attributes (`title` / `backdrop` / `theme`) now lower to `host prop` entries beside the content root, host bindings are rejected, and the old shape that placed host attributes on the content root is malformed IR. |
| 1.8     | 2026-06-13 | M3-Phase 7 design draft (Moment 1): added §4.15 iteration — the second chapter of the structural rendering model (`for` block with author-named binders; collection state types `i32[]` / `string[]` / `bool[]` with list-literal defaults; whole-value collection assignment over pure `append` / `drop-last` expressions and static-literal reset / clear; positional un-keyed identity baseline with the keyed non-promise; mutation-then-observe timing; all-or-unchanged insertion; per-container admission; diagnostics matrix). Supporting: §2.1 `in` reservation (`for` now has a production), §2.2 bracket / paren / comma tokens, §3 grammar, §4.6 / §4.7 binder-read and collection-state notes, §5 AST, §8.4 / §8.5 / §8.9 textual-IR collection and `for` forms with a worked offsets example, §8.11 validation rows. Swept the stale §4.14 `for` forward references (the `for` body ships single-widget, not member-range; the identity baseline ships positional un-keyed, with keyed as future opt-in) per the live-doc-sync rule. No ABI change; `abi_spec.md` untouched. Pending implementation re-sync at Phase 7 close. |
| 1.9     | 2026-06-18 | M3-Phase 7 implementation sync (Moment 2): flipped Phase 7 status markers to closed / implementation-synced; confirmed the landed textual-IR spellings (`for`, `list-prop-read`, `item-read`, `index-read`, `list-append`, `list-drop-last`, list literals) and unified `HandlerExpr` mapping; added the gallery slice example showing all four authored collection mutation forms (`append`, `drop-last`, empty clear, static reset) and recorded why per-item colour richness remains deferred. No ABI change; `abi_spec.md` remains untouched. |
| 1.10    | 2026-06-21 | M3-Phase 7b design draft (Moment 1): added §4.16 parent-interpreted placement — the shared `slot.*` namespace for Grid and ZStack child placement. ZStack per-child alignment moves from bare `h-align` / `v-align` to `slot.h-align` / `slot.v-align` (§4.13); Grid gains a direct `slot.*` form alongside the retained `Cell` grouped form (§4.12), one form per child with two distinct mixing / non-admitting-parent rejects, no normative canonical form (provisional `Cell`-default examples convention). Supporting: §3 `placement_bind` production, §4.4 registry note, §4.15 `for`-placement (placement on the body root child), §8.11 validation rows (placement admission / constant-RHS). Placement is constant per instance; a binding-expression RHS is rejected. No new `IrType` / `IrLiteral` / `PropertyValue` or C ABI change; `abi_spec.md` untouched. The loaded-IR placement representation and storage model are normative in `architecture.md` (DD-002 / Moment 1, landing in the sibling architecture commit), so the textual-IR placement emit form (§8.5) re-syncs there. Pending implementation re-sync at Phase 7b close. |
| 1.11    | 2026-06-24 | M3-Phase 7b implementation sync (Moment 2): flipped §4.12 / §4.13 / §4.16 status markers to closed / implementation-synced. Pinned the §5 AST to the landed parser — `slot.<key>` rides the existing `PropertyBind` variant (name canonicalized **with** the `slot.` prefix retained, e.g. `slot.h-align`); there is **no** separate `PlacementBind` AST variant. Pinned the §8 loaded-IR examples to the landed IR member spelling `Widget(IrChildSlot { node, slot_data })` (tuple variant wrapping `IrChildSlot`, not the struct-variant draft sketch). The §3 `placement_bind` author-surface production and the §8.5 `child { placement <kind> { … } node … }` textual-IR skeleton matched the landed `wasamoc` emit / loader and were confirmed unchanged. No ABI change; `abi_spec.md` remains untouched. |
| 1.12    | 2026-07-02 | M3-Phase 8 design draft (Moment 1): added §4.17 `ToggleButton` selected / toggle-state surface (controlled one-way `checked` bool attribute; background-colour-only minimal / provisional visual; `checked` admitted on `ToggleButton` only with a two-gate reject; exactly-one-selected exclusion as an author-composed M3-era pattern; five future selection directions kept as non-reserved future notes) and §4.18 public-draft future-surface notes (author-controllable sizing as a pre-1.0 unresolved surface whose shape is not reserved — no schedule published; Grid two-form placement provisional; default-alignment asymmetry as container-owned / explicable; placement-spelling affirmative keep; placement bindability). §4.4 registry gains the `ToggleButton` row and §4.8 property catalog gains the `ToggleButton` `checked` and shared `enabled` rows (`enabled` carries the same Phase-1 disabled contract as `Button.enabled`); §8.5 notes `ToggleButton` as an ordinary node reusing the bool binding forms. No new `IrType` / `IrLiteral` / `PropertyValue` or token; `abi_spec.md` untouched. The `status: public-draft` marker, the public-draft promotion change-history entry (distinct from this revision-history table), and the external-reader smoke are deferred to Phase 8 close (Moment 2). Pending implementation re-sync at Phase 8 close. |
| 1.13    | 2026-07-05 | M3-Phase 8 public-draft readiness editorial pass: removed stale pre-Phase-6 wording from §4.9 and clarified that the Phase 7 collection mutation Buttons were verification scaffolding, while the final integrated Gallery keeps generated thumbnails without retaining Add / Remove / Clear / Reset as end-user UI. The public-draft marker and promotion change-history entry remain deferred to the Phase 8 Moment 2 sync. |
| 1.14    | 2026-07-06 | M3-Phase 8 T9 G(4) review remediation: normalised the Phase 8 verification-status wording after T8 external-reader smoke, corrected §4.17 so `ToggleButton.checked` is framed as the first persistent selected-state bool attribute rather than the first bool-driven widget attribute, and kept public-draft reopen triggers out of §4.18 prose. No public-draft marker, promotion change-history entry, or ABI change. |
| 1.15    | 2026-07-06 | M3-Phase 8 implementation sync (Moment 2): flipped the top Status block and the §4.17 phase-status marker to closed / implementation-synced, promoted the document to `public-draft`, and added the public-draft change-history anchor (promotion record + M3 decision links + T8 external-reader smoke result). No body-prose semantic change (divergence corrections were folded in 1.13 / 1.14); no new `IrType` / `IrLiteral` / `PropertyValue` or token; `abi_spec.md` untouched. |
| 1.16    | 2026-07-28 | M4-Phase 1 design draft (Moment 1): added §1 *Units and the layout coordinate system* — every authored length and font size is DIP (`1 DIP = 1/96 inch`), an authored layout is identical at every display scale factor, and a DIP is a physical length rather than a device pixel. The previously undefined "pixel extents in the layout coordinate system" wording is **replaced** at each dimension-bearing site (§4.10 WrapPanel `item-cross-size` / `item-spacing` / `line-spacing`, §4.11 ScrollView `offset-y`, §4.12 Grid fixed track sizes), which now reference the definition instead of restating it; §2.2 notes that the `px` unit suffix names DIP, and §4.9's rounding note is restated in DIP terms. No grammar, token, AST, `IrType`, `IrLiteral`, or `PropertyValue` change — the unit is a semantic statement about existing literals, so `wasamoc` and the IR are untouched. At 100% every existing `.ui` file is unchanged in behaviour. The runtime-side coordinate-space model (the two spaces, the conversion seams, the text-surface resolution contract, scale invariance) is normative in [architecture.md §12](./architecture.md#coordinate-spaces); the ABI argument unit is in [abi_spec.md](./abi_spec.md) §4.2. Pending implementation re-sync at M4-Phase 1 close. |
| 1.17    | 2026-08-04 | M4-Phase 1 implementation sync (Moment 2): flipped the phase status to implementation-synced after re-verifying the authored-length and font-size DIP statements against the landed runtime. No grammar, token, AST, `IrType`, `IrLiteral`, `PropertyValue`, or authored-value change; runtime coordinate projection and raster resolution remain normative in [architecture.md §12](./architecture.md#coordinate-spaces), and the outer-window ABI unit remains normative in [abi_spec.md §4.2](./abi_spec.md). |
| 1.18    | 2026-08-05 | M4-Phase 2 design draft (Moment 1): added §4.19 *Interaction* — `clicked` admitted on any widget (§4.5 updated from "the only recognized signal name in M1"); a pointer event resolves to exactly one target, the topmost containing widget, from which occlusion of lower siblings and of content behind a disabled Button follow as consequences rather than as separate rules; propagation is target-then-ancestors with **consume on handle** and no descending phase; a handler's state writes drain once after propagation completes. Per-item handlers are admitted inside `for` bodies with binder reads in handler position — **reversing the M3-Phase 7 deferral** in §4.15, whose "handlers inside a `for` body" subsection now points here, and updating the binder read-position statements in §4.6 and §4.15; a binder resolves at invocation time, so under the positional identity baseline a handler belongs to a slot rather than to an item, and its registration is released with the generated subtree. Added the constant-only `focus-group` and `modal-scope` boolean container attributes (same non-bindable rule as `Box.fill` / the `WrapPanel` attributes), Tab / arrow / group-memory semantics, scope entry / restoration / Esc delivery, and the statement that a scope confines the keyboard only — pointer confinement comes from the occlusion rule plus an authored covering widget. Screen-reader modality is stated as attaching to the focus scope, binding on the later accessibility phase. No new token, grammar production for expressions, `IrType`, `IrLiteral`, or `PropertyValue`; `abi_spec.md` untouched (no new ABI entry point). Pending implementation re-sync at M4-Phase 2 close. |
| 1.19    | 2026-08-05 | M4-Phase 2 design sync, keyboard half: §4.19 gains the **`dismiss`** request — addressed to the innermost entered scope, not bubbled, with the author deciding what closing means and Esc named as one *source* rather than as the concept, so a later click-away or widget-set close control reuses the same signal — and the **`key-down("<key>")`** command surface, whose key is named in the declaration because the recognised set is validated at `check`. `key-down` is the physical-key-press half and is stated as **not** a text-input path: an active input-method composition owns the keyboard, and auto-repeat is delivered. The recognised names are **non-character keys only** (`"Escape"`, the arrows, `"Home"` / `"End"`, `"PageUp"` / `"PageDown"`, `"Enter"`, `"F1"`…`"F12"`), which keeps the logical-key versus physical-position question closed; character keys and modifier combinations such as `"Ctrl+S"` are outside the surface. Added the table of keys the runtime keeps (`Tab` always; arrows while focus is inside a `focus-group`; `Escape` while a scope is entered). §Not in this surface now also lists `key-up`, a catch-all key handler, a structured key value, a shortcut table, and a dismissal-policy attribute, with the two that carry an open question named. No new token, `IrType`, `IrLiteral`, or `PropertyValue`; `key-down`'s argument is the one new grammar production. `abi_spec.md` untouched. |
| 1.20    | 2026-08-06 | M4-Phase 2 design sync, revision: §4.19 states that a modal scope is entered by **being present** — the subtree's appearance pushes it, remembers the widget that had focus, and moves focus to the scope's first stop, so a scope opened by a conditional is confined and its key handlers live without a separate act; closing is deleting. Restoration is stated against what was actually focused, since a click on a non-focusable widget leaves focus where it was. Hit resolution is **bounded by ancestor clips**, so content clipped out of a `ScrollView` / `Grid` / `ZStack` receives nothing while a non-clipping container's overflow stays reachable. Focus gains its opening state (nothing focused; the first Tab lands on the first stop), the rule that a click focuses the nearest focusable widget **at or above** the resolved target, and the statement that `enabled: false` removes a focus stop — which discharges the tab-order half §4.8 deferred to M4 and brings §4.8's disabled contract into agreement with §4.19's occlusion rule (a disabled Button occludes, dispatches nothing, and does not stop propagation). `dismiss` is admitted **only** on a container carrying `modal-scope: true`, with a signal-admission table beside the attribute table. A key no handler consumes now reaches the window's default handling rather than being swallowed. §Not in this surface adds hover / pressed as authored signals and scrolling a focused widget into view. Corrected the per-item handler example to §4.15's `for <binder>, <index-binder> in <collection>` form. No new token, `IrType`, `IrLiteral`, or `PropertyValue`; `abi_spec.md` untouched. Pending implementation re-sync at M4-Phase 2 close. |
| 1.21    | 2026-08-09 | M4-Phase 2 implementation sync (Moment 2): flipped the phase markers to closed / implementation-synced and corrected the normative text against the landed runtime. Added the optional string argument to the authored and textual-IR handler grammars; removed the stale §4.15 handler-rejection rows; documented the four childless widget kinds; narrowed the Button keyboard wording to the shipped Tab behaviour; fixed group arrow direction, outside-scope click focus, and container hit candidacy; and re-synced architecture §12.3 / §13 to the landed mouse/touch conversion, touch activation, focus repaint/composition, and scope restoration/succession paths. Unknown signal names remain unspecified; string assignment remains a normative compiler-enforcement divergence assigned to later M4 phases. No ABI change; `abi_spec.md` untouched. |
