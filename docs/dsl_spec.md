# Wasamo DSL Specification

**Document version:** 1.3
**Last updated:** 2026-05-29
**Status:** M3-Phase 2 closed (implementation-synced); M3-Phase 3
closed (implementation-synced); M3-Phase 4 closed
(implementation-synced); M3-Phase 5 design accepted (implementation
pending).
Covers the M2 `.ui` surface, the `state` surface keyword
retroactively, the M3-Phase 1 `bool` scalar binding additions, the
M3-Phase 2 Box layout primitive (with `aspect` / `fill` literal
attributes), the M3-Phase 3 WrapPanel layout primitive (with
`item-cross-size` / `item-spacing` / `line-spacing` constant-only
integer attributes), the M3-Phase 4 ScrollView layout primitive
(vertical-only viewport + clip + `offset-y` binding), the M3-Phase 5
Grid layout primitive (fixed + weighted-star track sizing, `Cell`
wrapper with explicit placement / span / alignment, both-axis
spanning, Grid outer-bounds clip), and `;wasamo-ir v0`.

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
| `state`     | Starts a state declaration (see §4.7)    |
| `true`      | Bool literal — reserved identifier (M3-Phase 1) |
| `false`     | Bool literal — reserved identifier (M3-Phase 1) |

`in-out` is lexed as a **single keyword token** (not `in`, `-`, `out`).

`true` and `false` are reserved by the lexer and may not appear as
identifiers (state names, property names, widget type names, qualified-
name segments). Using either in identifier position is a parse error.

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
                  |  widget_decl
                  |  signal_handler
                  |  state_decl

property_decl    ::= "in-out" "property" "<" type_name ">" IDENT
                     ":" expr

state_decl       ::= "state" IDENT ":" state_type "=" expr

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
```

### Disambiguation

Within `member`, a 2-token lookahead resolves the alternative:

| First token | Second token | Rule matched      |
|-------------|--------------|-------------------|
| `in-out`    | `property`   | `property_decl`   |
| `state`     | `IDENT`      | `state_decl`      |
| `IDENT`     | `:`          | `property_bind`   |
| `IDENT`     | `{`          | `widget_decl`     |
| `IDENT`     | `=>`         | `signal_handler`  |

---

## 4. Semantics (M2 Surface, M3-Phase 1 / Phase 2 Additions)

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
| `Grid`      | 2D layout container with declared track lists per axis; children are `Cell`-wrapped (M3-Phase 5; see §4.12) |

`Cell` is **not** a free-standing widget registry entry. It is a
Grid-specific child wrapper construct (one content child per `Cell`,
carrying explicit placement / span / alignment metadata) consumed by
Grid's lowering; `Cell` outside a `Grid` parent is rejected at
`wasamoc check`. See §4.12.

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

### 4.7 State declarations (M2 surface; bool added in M3-Phase 1)

```
state <name>: <state_type> = <default>
```

Declares a per-component reactive `Signal<T>` whose value is owned by
the `.ui` source (DD-M2-P6-004). `state` declarations are a
component-level member, parallel to `in-out property`. Multiple
`state` declarations may appear in any order; names share a flat
namespace within the component.

Supported `state_type`s:

| `state_type` | Runtime store                            |
|--------------|------------------------------------------|
| `i32`        | `Signal<i32>` — integer reactive value   |
| `string`     | `Signal<String>` — string reactive value |
| `bool`       | `Signal<bool>` — bool reactive value (M3-Phase 1) |

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

State declarations lower to the IR `state_decl` form (§8.4).

### 4.8 Widget property catalog (M3-Phase 1)

Per-widget typed property entries that may be bound (`prop: <expr>`
or reactively from a `state` declaration). M2 widget properties
remain bindable through the M2 string-baked path; the entries below
are the ones whose declared type is checked at `wasamoc check` and
dispatched through a per-type binding writer at the runtime loader
(DD-M3-P1-007, DD-M3-P1-009).

| Widget | Property  | Type   | Default | Notes |
|--------|-----------|--------|---------|-------|
| `Button` | `enabled` | `bool` | `true`  | M3-Phase 1; see contract below |

**`Button.enabled` Phase 1 contract.** When bound to `false`:

- The button suppresses click-handler dispatch (no host callback, no
  inline `clicked` handler invocation, no `enqueue_signal("clicked", …)`).
- Hover / press visual transitions are frozen; the background paints a
  flat disabled grey directly (no `ColorKeyFrameAnimation` runs).
- The layout slot is **preserved** — the button still measures and
  arranges identically to its enabled form; there is no
  `display: none` semantics.
- Child hit-test traversal is preserved.

**Explicitly deferred to later milestones.** Keyboard focusability and
tab-order semantics when disabled, AccessKit / `aria-disabled`
accessibility tree state, hover and focus visual variations, and key
activation suppression. M4 (input/focus) and M5 (accessibility) own
the full interaction-state contract for disabled controls; the Phase 1
contract above is structured to be additive under that widening, not
superseded by it.

### 4.9 Box layout primitive (M3-Phase 2)

**Phase status:** M3-Phase 2 closed; implementation-synced.

`Box` is a layout container that admits **zero or one child**
(DD-M3-P2-001). Multi-child overlap is ZStack's responsibility
(Phase 6, not yet shipped); a Box declared with two or more children
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

Both attributes are constant-only in M3-Phase 2 (DD-M3-P2-004);
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
forward-compatible and is not revised, only extended (see
[m3-phase-2-box-layout.md DD-M3-P2-004](./decisions/m3-phase-2-box-layout.md)).

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

When Box has a single child (DD-M3-P2-001):

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

#### Aspect-constraint measure-arrange (DD-M3-P2-005)

When Box carries `aspect`, its outer bounds are resolved from parent
bounds via **inscribed fit**: Box's resolved rectangle is the largest
aspect-correct rectangle that fits inside the parent. Given parent
width `W` and height `H` and `aspect: num:den`, the branch is
selected by integer comparison `(W as f64) * (den as f64)` vs
`(H as f64) * (num as f64)`; once the branch is chosen the derived
axis is computed in `f32`. No pixel-snapping in Phase 2; rasterisation
/ DPI hinting is unaffected by this section.

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

#### Image placeholder pattern (M3, DD-M3-P2-006)

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
(no `orientation` attribute is exposed in Phase 3, see
[m3-phase-3-wrap-panel.md DD-M3-P3-002](./decisions/m3-phase-3-wrap-panel.md)).

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

All three attributes are constant-only integer literals in M3-Phase 3
([m3-phase-3-wrap-panel.md DD-M3-P3-003 / DD-M3-P3-004](./decisions/m3-phase-3-wrap-panel.md)).
The values are pixel extents in the layout coordinate system; they
reuse the existing `i32` literal plumbing from M2 (no new `IrType`,
no new `IrLiteral` variant, no new `PropertyValue` variant).

**Non-negative integer range.** All three attributes admit
**non-negative** integer values. `wasamoc check` rejects a negative
literal at compile time, naming the rejected attribute; the runtime
IR loader's `validate()` independently rejects negative IR
(DD-M3-P3-006 two-gate defense, mirroring Phase 2's `RATIO_LIT`
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
follows the same per-line rule above (DD-004-bound or passthrough).
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

#### Attributes

| Attribute | Surface form | Bindable in Phase 4 | Default |
|---|---|---|---|
| `offset-y` | `<i32>` literal or a bare state identifier such as `scroll_y` (declared as `state scroll_y: i32 = 0` per §4.7) | Read-only binding | `0` |

`offset-y` is a signed integer pixel offset in the layout coordinate
system. Per §4.3, the bound form is a bare identifier RHS (for example
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

**Phase status:** M3-Phase 5 design accepted; implementation pending.

`Grid` is a 2D layout primitive that arranges children across a
declared row × column track matrix. Tracks are declared once on
`Grid` via the `columns:` and `rows:` attributes. Each child is
wrapped in a `Cell` carrying explicit `row` / `column` placement,
optional `row-span` / `column-span`, and optional per-cell `h-align`
/ `v-align`. Content widgets inside a `Cell` carry no Grid-specific
metadata.

Grid admits **zero or more `Cell` children**. The minimum valid Grid
shape is `columns.len() >= 1` and `rows.len() >= 1`; a Grid with no
`Cell` children resolves to its outer rectangle with no drawn cell
content.

#### Sizing mental model

Grid sizing follows six facts:

1. **One track list per axis, declared on Grid.** `columns:` and
   `rows:` carry whitespace-separated sequences of track-sizing
   tokens. Track sharing across rows / columns is automatic; there
   is no per-row column-width or per-column row-height surface.
2. **Fixed tracks consume definite space first.** Each
   `<integer>`-px track contributes its declared pixel size to the
   axis's resolved extent.
3. **Weighted-star tracks divide remaining bounded space by integer
   weight.** `*` (sugar for `1*`) and `n*` (positive integer in
   `[1, 1024]`) take fractional shares of the bounded space the
   parent allocates after fixed tracks are honoured. `auto` /
   intrinsic and floating-point weights are reserved for a future
   phase (see *Reserved future surface* below).
4. **Children go in `Cell` wrappers, not directly in Grid.** Each
   `Cell` declares its placement as `row` + `column` (zero-based)
   and optionally spans cells via `row-span` + `column-span`
   (default `1`). Same-cell occupancy — two `Cell`s whose resolved
   `(row, column, row-span, column-span)` rectangles share any
   resolved cell — is rejected at `wasamoc check` and at runtime
   `validate()`. Intentional overlay is not Grid's responsibility;
   Phase 6 ZStack owns overlay.
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
  widgets. Wasamo's Grid declares tracks the same way conceptually
  but routes child placement through an explicit `Cell` wrapper
  rather than attached properties on content widgets — content
  widgets stay free of Grid-specific metadata. Star sizing,
  spanning, and zero-based indexing match WPF.
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
  is WrapPanel + the iteration grammar; see §4.10.) Iteration
  generating `Cell`s is not foreclosed but is post-M3.
- **ZStack / overlay models.** Grid does not provide intentional
  overlay. A Cell whose content paints past the cell rectangle may
  cross into a sibling cell's region — that is governed by the
  document-order paint rule below — but two `Cell`s may not
  deliberately occupy the same resolved cell. Phase 6 ZStack is the
  surface for intentional overlay.

#### Children

Grid admits zero or more **`Cell`** children directly. `Cell` is a
Grid-owned single-child layout wrapper:

- Each `Cell` accepts **exactly one content child**. `wasamoc check`
  rejects `Cell { }` (0 children) and `Cell { X Y }` (2+ children);
  the runtime IR loader's `validate()` independently rejects
  malformed memory IR through `WASAMO_ERR_IR_MALFORMED`. Authors
  who want multiple widgets in one cell wrap them explicitly
  (`Cell { VStack { Text { } Text { } } }`).
- `Cell` outside a `Grid` parent is rejected at `wasamoc check` and
  at `validate()`. `Cell` is not a free-standing widget; it has no
  meaning outside Grid's lowering.
- `Cell` itself does not materialise as a runtime widget Visual.
  The Visual tree contains one Visual for Grid plus one Visual per
  `Cell`'s content child — the existing **1 WidgetNode = 1 Visual**
  convention from §6.5 of `architecture.md` is preserved.

Non-`Cell` direct children of Grid are rejected at `wasamoc check`.

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
| `<integer>` (positive)     | Fixed track of that pixel width / height | `value >= 1` at `wasamoc check` and `validate()` |
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

**On `Cell`:**

| Attribute       | Type   | Default                              | Valid range                                | Violations |
|-----------------|--------|--------------------------------------|--------------------------------------------|------------|
| `row:`          | `i32`  | `0` (single-`Cell` Grid only; see below) | `[0, rows.len())`                          | `wasamoc check` + `validate()` reject |
| `column:`       | `i32`  | `0` (single-`Cell` Grid only; see below) | `[0, columns.len())`                       | `wasamoc check` + `validate()` reject |
| `row-span:`     | `i32`  | `1`                                  | `[1, rows.len() - row]`                    | `wasamoc check` + `validate()` reject |
| `column-span:`  | `i32`  | `1`                                  | `[1, columns.len() - column]`              | `wasamoc check` + `validate()` reject |
| `h-align:`      | ident  | `stretch`                            | `{ start, center, end, stretch }`          | `wasamoc check` + `validate()` reject |
| `v-align:`      | ident  | `stretch`                            | `{ start, center, end, stretch }`          | `wasamoc check` + `validate()` reject |

Unknown Cell attributes are rejected at `wasamoc check`. Phase 5
has no `Cell` `clip:` / `z-index:` / `area:` surface.

**Placement-attribute presence rule.** In a Grid with two or more
`Cell` children, every `Cell` must declare both `row` and `column`
explicitly; omitting either is a `wasamoc check` diagnostic. In a
Grid with exactly one `Cell`, missing `row` and/or `column` is
permitted and lowers to `0`. The single-Cell Grid escape clause
exists for minimal demo cases; multi-Cell Grids are required to be
self-describing so the diagnostic surface for "missed placement"
stays local.

**Same-cell / overlapping-rectangle conflict rejection.** For every
pair of `Cell`s within a Grid, the algorithm checks whether their
resolved `(row, column, row-span, column-span)` rectangles share
any cell. Conflicts are rejected at `wasamoc check` and at
`validate()`, with a diagnostic naming both conflicting `Cell`s
and the shared resolved cell coordinate.

**Indexing convention.** All `row` / `column` values are
**zero-based** at the `.ui` boundary and zero-based internally.
`row: 0` is the first row, `row: rows.len() - 1` is the last.

**Constant-only.** Grid `columns:` / `rows:` and Cell placement /
span / alignment attributes are constant-only literals in Phase 5;
none of them are bindable. No new `IrType`, `IrLiteral`, or
`PropertyValue` variant is introduced. A future phase may admit
bindable track lists or bindable Cell placement; Phase 5 does not
foreclose this but does not implement it.

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

After track resolution, each `Cell`'s resolved rectangle is placed
relative to Grid. The content widget is then arranged inside that
rectangle per `Cell`'s alignment attributes:

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

Phase 5 Grid invariants checked at `validate()`:

| Invariant | On failure |
|-----------|------------|
| Grid declares at least one row and at least one column | `WASAMO_ERR_IR_MALFORMED` |
| Each fixed track value `>= 1`; each star weight in `[1, 1024]` | `WASAMO_ERR_IR_MALFORMED` |
| Each `Cell` has exactly one content child | `WASAMO_ERR_IR_MALFORMED` |
| `Cell.row` in `[0, rows.len())`; `Cell.column` in `[0, columns.len())` | `WASAMO_ERR_IR_MALFORMED` |
| `Cell.row-span >= 1`; `Cell.column-span >= 1`; `row + row-span <= rows.len()`; `column + column-span <= columns.len()` | `WASAMO_ERR_IR_MALFORMED` |
| No two `Cell`s within a Grid share any resolved cell | `WASAMO_ERR_IR_MALFORMED` |
| `Cell.h-align` and `Cell.v-align` values in `{ start, center, end, stretch }` | `WASAMO_ERR_IR_MALFORMED` |

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
- **Iteration-template-generated `Cell`s** (e.g.
  `for item in items { Cell { row: ... ... } }`). Grid is not an
  M3 iteration target; the iteration grammar's M3 target is
  WrapPanel-backed thumbnail collections. Future admission is
  structurally possible because every `Cell` is explicit.
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
3. **Forgetting `row:` / `column:` in a multi-`Cell` Grid.** The
   single-Cell escape clause does **not** apply once a Grid has
   two or more `Cell`s. `wasamoc check` rejects the omission with
   a local diagnostic; the fix is to add the missing placement
   explicitly.
4. **Two `Cell`s with overlapping rectangles.** Two Cells that
   resolve to overlapping `(row, column, row-span, column-span)`
   rectangles are rejected with a diagnostic naming both Cells and
   the shared resolved cell. Intentional overlay is Phase 6
   ZStack's responsibility; the fix is to relocate one Cell or
   wait for ZStack.
5. **Expecting per-cell clipping.** A Cell whose content paints
   past the cell rectangle may cross into a sibling cell's region
   (until Grid's outer-bounds clip cuts it off). The fix is to
   wrap the oversized content in a clipping parent (e.g.
   ScrollView).
6. **Expecting Grid to grow with its tracks.** Grid's outer
   rectangle equals the parent's allocation on each bounded axis,
   not the sum of resolved track sizes. Authors who want a Grid
   sized by its tracks must size the parent's allocation
   accordingly.

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
    StateMember   { name: String, ty: TypeName, default: Expr },  // M2
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
| `type_name` | `"i32"`, `"string"`, or `"bool"` (M3-Phase 1 adds `bool`) |
| `literal`   | Default value: `INT` for `i32`; `STRING` for string; `BOOL` (`true`/`false`) for `bool` |

Examples:

```
state count: i32 = 0
state ready: bool = false
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
and `abi.rs` arms land together in that phase
(DD-M3-P2-002 / DD-M3-P2-003 / DD-M3-P2-004).

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
        | "(" "bool-prop-read"  IDENT ")"     ; M3-Phase 1
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
| `(assign NAME expr)` | `Assign { lhs, rhs }` | Handler-only. M3-Phase 1: `rhs` may now be `BoolLit` or `BoolPropRead` when the LHS state is `bool`-typed |
| `(compound-assign OP NAME expr)` | `CompoundAssign { lhs, op, rhs }` | Handler-only. **Not defined over `bool`** — no `CompoundOp` is naturally bool-typed |
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
| `Box` node has at most one child (M3-Phase 2, DD-M3-P2-001) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| `RATIO` literal has `num > 0` and `den > 0` (M3-Phase 2) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| `WrapPanel` `item-cross-size`, `item-spacing`, and `line-spacing` are non-negative `i32` (M3-Phase 3, DD-M3-P3-006) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| `ScrollView` node has exactly one content child (M3-Phase 4, DD-M3-P4-006) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| `Grid` declares at least one row and at least one column; each fixed track value is `>= 1`; each star weight is in `[1, 1024]` (M3-Phase 5, DD-M3-P5-006) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| Each `Cell` has exactly one content child; `Cell.row` in `[0, rows.len())`; `Cell.column` in `[0, columns.len())`; `Cell.row-span`/`column-span >= 1` with resolved rectangle within declared track count; no two `Cell`s in the same Grid share any resolved cell; `h-align`/`v-align` in `{ start, center, end, stretch }` (M3-Phase 5, DD-M3-P5-006) | Yes | `WASAMO_ERR_IR_MALFORMED` |
| Binding expression result type matches target property type | **No** (trusted from `wasamoc`) | Undefined behaviour |
| Per-node emitter invariants (e.g. `on` only on signal-capable widgets) | **No** (trusted from `wasamoc`) | Undefined behaviour |

The loader trusts type-level invariants established by `wasamoc`'s check pass.
Type mismatches indicate a `wasamoc` bug, not a recoverable load-time error.

The M3 rows above (Phase 2 `Box` child count, Phase 2 `RATIO` sign,
Phase 3 WrapPanel non-negative attributes, Phase 4 ScrollView
single-content-child rule, Phase 5 Grid structural / track / placement /
span / conflict / alignment-vocabulary invariants) are explicitly
dual-gated rather than trusted because `wasamo_load_ui`'s memory-IR
entry point does not pass through `wasamoc`; the runtime gate is the
last line of defence for these spec invariants. See §4.9 for the
Box child-count rationale, §8.2 for the `RATIO` surface constraint,
§4.10 for the WrapPanel attribute range, §4.11 for the ScrollView
child-count rule, and §4.12 for the full Grid / Cell invariant set —
all of which `wasamoc check` already enforces.

Phase 5 Grid invariants are **reject-at-validate**, not
clamp-at-arrange: placement and span values have no defensible
clamped interpretation (a silently-clamped Cell would displace
legitimately-placed siblings and produce order-dependent layout).
The only layout-time gate Phase 5 introduces is
`LayoutError::GridUnboundedStarAxis` (§4.12), which is not a
`validate()`-time concern because it depends on the parent's axis
bound.

### 8.12 Scope out (post-M2)

| Feature | Deferred to |
|---|---|
| `(computed ...)` expression form | M3 |
| `(if ...)` / `(for ...)` binding forms | M3+ |
| M3 expanded type set (`float`, user types; `bool` landed in M3-Phase 1) | M3 |
| Generic `TypedValue` value union (F5 deferral) | Post-M3 |
| Bindable surface for Box `aspect` / `fill` (M3-Phase 2 admits the literals only) | Future phase that first needs reactive aspect or fill |
| `IrType::Ratio` / `IrType::Color` (M3-Phase 2 stores them Box-internal, not as `PropertyValue` variants) | Same future phase as above |
| Binary IR format | Post-M2 |
| Grammar version `v1` (first incompatible change) | When required |
| `(post-event ...)` escape hatch for observer callbacks | M3 (DD-M2-P6-001 Option F) |

**F5 (`TypedValue`) deferral.** M3-Phase 1 admits `bool` as a third
tagged scalar (`IrType::Bool`, `IrLiteral::Bool`, `HandlerExpr::BoolLit` /
`BoolPropRead`) using the same type-suffixed variant pattern M2
adopted for `i32` and `String` (DD-M2-P6-003). It does not introduce
a generic `TypedValue` value union; the per-type binding evaluator and
per-type widget property writer (see `architecture.md` §6.8.7 and
[m3-phase-1-bool-scalar.md DD-M3-P1-007](./decisions/m3-phase-1-bool-scalar.md))
are the structural form of that deferral. F5 is recorded in
[notes/m3/m3-start-framing.md §F5](./notes/m3/m3-start-framing.md#f5--typedvalue-は再評価候補だが開始時点の-m3-acceptance-ではない).

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
`;wasamo-ir v0`. M3-Phase 1 rejects `bool`-typed state interpolation
at `wasamoc check` time rather than lowering it to a runtime
`TypeMismatch`; an explicit formatting/display-conversion surface is a
future design item.

---

## Revision history

| Version | Date       | Notes                                                                             |
|---------|------------|-----------------------------------------------------------------------------------|
| 0.1     | 2026-04-27 | Initial draft (Phase 1, pending owner agreement)                                  |
| 0.2     | 2026-04-27 | Phase 1 Accepted; added missing tokens (MinusEq/StarEq/SlashEq); corrected AST types (StringLit → Vec<StringPart>, Statement as struct); corrected error output format |
| 0.3     | 2026-05-07 | M2-Phase 6 Accepted; added §8 Wasamo IR normative spec (DD-M2-P6-002 + DD-M2-P6-003) |
| 0.4     | 2026-05-11 | M2 complete; added `str-prop-read` IR form from DD-M2-P6-011 and updated M2/post-M2 status language |
| 0.5     | 2026-05-19 | M3-Phase 1 (`bool` scalar binding): added `true`/`false` keywords, `BoolLit` token, `BoolLit`/`BoolPropRead` IR expression forms, `bool` to `state_decl` type set, `Button.enabled` widget-catalog entry, and `state` surface declaration §4.7 (retroactive M2 gap); recorded F5 (`TypedValue`) deferral in §8.12 |
| 0.6     | 2026-05-19 | M3-Phase 1 T14: documented that string interpolation over `bool`-typed state is rejected until an explicit formatting/display-conversion surface exists |
| 0.7     | 2026-05-20 | M3-Phase 2 ADR-accepted design draft: added §4.9 Box layout primitive chapter (Phase status marker; `aspect` / `fill` attribute surface; single-child centred-and-clipped layout contract; aspect inscribed-fit measure-arrange with edge cases; image placeholder pattern subsection per DD-M3-P2-006); added `RatioLit` / `ColorLit` tokens (§2.2), grammar rules (§3), AST variants (§5), §8.2 terminals, and §8.6 literal alternatives + Box-internal materialisation note. Pending implementation re-sync at Phase 2 close. |
| 0.8     | 2026-05-20 | M3-Phase 2 close: flipped §4.9 Phase status marker and document status to implementation-synced after T1-T13 landed and local / CI phase-end gates passed. No implementation/spec divergence was found during the close re-sync. |
| 0.9     | 2026-05-21 | M3-Phase 3 ADR-accepted design draft: added §4.10 WrapPanel layout primitive chapter (Phase status marker; sizing mental-model subsection with four-fact anchor and WPF / Compose / CSS ecosystem contrast per framing decision H; `item-cross-size` / `item-spacing` / `line-spacing` constant-only `i32` attribute surface; two-stage measure-arrange algorithm with bounded happy path, unbounded-main-axis one-line-flow branch, and unbounded-cross-axis-with-aspect-child propagation to Phase 2's `LayoutError::BoxAspectUnboundedBoth`; oversized-first-child + visible-overflow subsection; common-pitfalls note); added `WrapPanel` row to the §4.4 widget registry and dropped the stale `M1` qualifier from the registry's lead-in (the registry grew beyond M1 once `Box` landed in Phase 2; folded into this commit as a minimal retroactive fix with owner confirmation). No new tokens, grammar rules, AST variants, or IR forms — Phase 3 reuses existing `i32` plumbing. Pending implementation re-sync at Phase 3 close. |
| 1.0     | 2026-05-22 | M3-Phase 3 close: flipped §4.10 Phase status marker and document status to implementation-synced after T1–T9 landed and the local clean-rebuild gate passed. Folded the T1 Decisions-log lexer-surface item into §2.2: generalised the `Ident` lexical pattern to admit kebab-case continuations (`-[A-Za-z]`-prefixed segments) and the `IntLit` pattern to admit an optional leading `-`; added a one-line note that the negative-sign surface is `IntLit`-only (does not extend `FloatLit` / measurement / `RatioLit` and does not introduce a subtraction or unary-minus operator). `§5` AST shapes unchanged (`IntLit { value: i64 }` already holds the signed surface). No other implementation / spec divergence found during the close re-sync. |
| 1.1     | 2026-05-25 | M3-Phase 4 ADR-accepted design draft: added §4.11 ScrollView layout primitive chapter (Phase status marker; viewport/content/offset mental model with WPF / CSS `overflow: scroll` / SwiftUI ecosystem contrast; exactly-one-child contract; `offset-y` signed `i32` literal or read-only `i32` state binding; parent-supplied viewport with no `viewport-*` attributes; pure-data measure-arrange algorithm including inner unbounded vertical measure, offset clamp, `LayoutError::ScrollViewUnboundedAxis`, and rounding contract; Visual-layer contract for the ScrollView-owned intermediate content Visual carrying `Visual.Offset = (0, -applied_y, 0)`; common-pitfalls note). Added `ScrollView` row to the §4.4 widget registry. No new grammar tokens, AST variants, IR literal/type variants, or scalar value types — Phase 4 reuses existing `i32` plumbing plus a narrow ScrollView string-to-`i32` parse / write bridge. Pending implementation re-sync at Phase 4 close. |
| 1.2     | 2026-05-25 | M3-Phase 4 close: flipped §4.11 Phase status marker and document status to implementation-synced after T1–T6 landed (including T6 window-root Fill/Fill fix bundle) and the T7 local clean-rebuild + GitHub Actions phase-end gates passed. Folded one Phase 4 close-time spec consistency fix: §4.9 Box examples switched from the `;`-separated single-line form (parser-invalid; surfaced by T5 first build) to the parser-accepted multi-line member-per-line form, with an adjacent notation note recording that **accepting `;` as an optional member separator remains a post-Phase-4 open question** — parser-accepted examples; semicolon member separator left as post-Phase-4 open question. §4.10 common-pitfalls example dropped its `; …` continuation to match the new multi-line convention. No other implementation / spec divergence found during the close re-sync. |
| 1.3     | 2026-05-29 | M3-Phase 5 ADR-accepted design draft: added §4.12 Grid layout primitive chapter (Phase status marker; sizing mental model with six-fact anchor and WPF / CSS Grid / Compose-SwiftUI grids / ZStack-overlay ecosystem contrast per framing decision FD-K; `Cell` single-content-child wrapper with explicit zero-based placement, both-axis spanning, and per-cell `h-align` / `v-align`; `columns:` / `rows:` track lists carrying fixed integer pixels and weighted-star tokens `n*` with `n in [1, 1024]`, parsed by a Grid-specific narrow parser path; `auto` deferred with reserved-future diagnostic; pure-data track-resolution algorithm with fixed-first + weighted-star distribution over `f32` prefix boundaries, `LayoutError::GridUnboundedStarAxis` on unbounded-star, and a reserved no-op slot for a future `auto` demand pass; Grid outer-bounds clip on Grid's own Visual via `Visual.Clip = InsetClip{0,0,0,0}` preserving the 1 WidgetNode = 1 Visual convention; document-order paint with no `z-index`; reject-at-validate dual-gate for placement / span / conflict invariants; reserved-future / common-pitfalls subsections). Added `Grid` row to the §4.4 widget registry plus a §4.4 pointer noting that `Cell` is a Grid-owned child wrapper defined in §4.12 (not a free-standing registry entry). No new grammar tokens, AST variants, `IrType` / `IrLiteral` / `PropertyValue` variants, or C ABI value tags — Grid's `Vec<TrackSize>` lives in a Grid-specific kind payload on `IrNode` and `IrProp.value` stays strictly `IrLiteral`. Folded a retroactive §8.11 spec-gap fix at owner request (review of the Moment 1 §4.12 + §6.8.7 thesis draft on 2026-05-29): the §8.11 *Loader validation policy* table previously listed only the Phase 2 `Box` child-count and `RATIO` sign dual-gates; added the Phase 3 WrapPanel non-negative attribute range (DD-M3-P3-006), the Phase 4 ScrollView single-content-child rule (DD-M3-P4-006), and the Phase 5 Grid / Cell invariant rows (DD-M3-P5-006) so the §8.11 table is now the true runtime-loader policy aggregate for M3. Pending implementation re-sync at Phase 5 close. |
