### DD-M3-P1-002 — `IrLiteral` extension and surface syntax

**Status:** Accepted

**Context:**
Once Option A of DD-M3-P1-001 is taken, `IrLiteral` needs to carry a
bool constant for `state foo: bool = <default>` and (eventually) for
literal-driven bindings. Two sub-questions: which variant shape, and
which surface syntax does `wasamoc`'s lexer/parser accept.

**Options for the IR literal variant:**

Option A — Add `IrLiteral::Bool(bool)` (recommended)
- Parallel to `IrLiteral::Int(i32)` and `IrLiteral::Str(String)`.

  - What you gain: Pattern symmetry with the rest of `IrLiteral`.
  - What you give up: Nothing structural.
  - **Technical risk:** Low.

Option B — Encode bool literals as `IrLiteral::Int(0|1)`
- Reuse the integer literal variant; type-context distinguishes.

  - What you give up: Same objection as DD-M3-P1-001 Option B: erases
    type at the IR boundary. Inconsistent with `Str` being its own
    variant.

**Options for the surface syntax:**

Option A — `true` / `false` keywords (recommended)
- Add two reserved identifiers to the lexer; parser produces
  `IrLiteral::Bool(true|false)`.

  - What you gain: Universal across DSL ancestry (XAML, SwiftUI, Slint,
    QML, CSS, every C-family language). Zero ambiguity with
    `IrLiteral::Ident` because the keywords are recognised at the
    lexer.
  - What you give up: Two reserved words. `true` and `false` are not
    plausible identifier choices in `.ui` (they're keywords or builtin
    constants in nearly every reasonable target language a `.ui` author
    is also literate in), so the reservation is essentially free.
  - **Technical risk:** Low. Lexer additions are localised in
    [wasamoc/src/lexer.rs](../../../../wasamoc/src/lexer.rs).

Option B — Reuse integer literals (`0` / `1`) with type coercion
- No new lexer tokens; `state foo: bool = 1` parses.

  - What you give up: Forces every reader of `.ui` (human and tooling)
    to remember a bool-context convention. Diverges from every prior
    DSL the project is influenced by.

Option C — Defer surface syntax entirely; expose `bool` only through
the host (e.g. component default set from C)
- `IrType::Bool` and `IrLiteral::Bool` exist; `.ui` cannot write a
  bool literal yet.

  - What you give up: Phase 1's E2E proof becomes awkward (the `.ui`
    side cannot declare `state foo: bool = false`). M3-Phase 6
    inherits the surface-syntax decision under deadline pressure when
    conditional rendering needs `if foo` (and `if true` for spec
    examples). No real saving — defers irreducible work.

**Recommendation:**
- IR variant: Option A (`IrLiteral::Bool(bool)`).
- Surface syntax: Option A (`true` / `false` keywords).

**Forward-compat exposure:** Adding `true` / `false` keywords is
irreversible only in the sense that removing them later would be a
breaking change; doing so is not on any plausible roadmap, so exposure
is nil.

---
