### DD-M3-P1-010 — `wasamoc` type-checker scope for `bool`

**Status:** Accepted

**Context:**
Phase 1 introduces a new scalar type. The `wasamoc` checker
([wasamoc/src/check.rs](../../wasamoc/src/check.rs))
has to decide which type combinations to accept and reject. The
question is well-bounded: which of the following do we want to be
*compile-time errors* (caught by `wasamoc check`), and which are
*runtime errors* (caught by `EvalError::TypeMismatch` at IR load /
evaluation)?

Examples, paired with the desired classification:

| Pattern | Static? | Reason |
|---|---|---|
| `state ready: bool = false` | ✓ valid | declaration matches default literal type |
| `state ready: bool = 0` | error | i32 literal in bool default |
| `state ready: bool = "false"` | error | string literal in bool default |
| `bind enabled: ready` (both `bool`) | ✓ valid | source/target types agree |
| `bind enabled: 1` (target `bool`) | error | i32 RHS into bool target |
| `bind enabled: true` (target `bool`) | ✓ valid | bool literal into bool target |
| `bind label: true` (target `String`) | error | bool RHS into string target |
| `bind label: ready` (target `String`, source `bool`) | error | bool source into string target |
| `state x: i32 = 5; bind enabled: x` | error | i32 source into bool target |
| identifier `ready` resolves to which `*PropRead`? | depends | resolve via the declared `state` type table |

The shape of "static vs runtime" matters because it affects how much
the M3 `wasamoc` checker grows in Phase 1 (and how much Phase 6
inherits as foundation).

**Options:**

Option A — Phase 1 checker rejects all of the above at
`wasamoc check`, using the state-name → declared-type table built
from parsed `state` decls (recommended)
- `wasamoc` builds a `HashMap<String, IrType>` from `state`
  declarations at parse time. The identifier-lowering pass resolves
  `ready` to `BoolPropRead` (if the table says `bool`), `PropRead`
  (if i32), or `StrPropRead` (if String). Binding LHS types come
  from the widget-property catalog (DD-M3-P1-009). Mismatch is a
  diagnostic with line/column.

  - What you gain: Mismatch errors land where they belong — at
    compile time, on the line they originate from, before any IR
    loader / runtime evaluator ever sees them. The same table
    Phase 6 needs to type-check conditional rendering's `if <expr>`
    becomes available now. Identifier resolution becomes
    deterministic at lowering time rather than at runtime
    evaluator dispatch.
  - What you give up: A typed lowering pass in `wasamoc` —
    bounded, but not zero. The current checker does not thread
    parent-widget context or target-property type down through
    member-checking; making `bind` LHS type-aware requires a
    signature change in `check_members` (or its equivalent) so the
    binding pass knows the widget catalog entry for the enclosing
    widget. This is a real (small) structural change to the
    checker, not just an additional rule.
  - **Technical risk:** Low–medium. `wasamoc` already has a checker
    framework (DD-M2-P6-001 onwards), so the rule additions are
    straightforward; the medium part is the `check_members`
    signature widening to carry widget context, which touches every
    existing member-check call site. Not large, not zero.

Option B — Phase 1 checker validates `state` default-vs-declared-type
only; defer binding type-checking to the IR loader / runtime
- `state ready: bool = 0` is rejected by `wasamoc`. But
  `bind enabled: 1` slips through `wasamoc` and dies at IR-load /
  evaluator with `EvalError::TypeMismatch`.

  - What you give up: Tooling story (M5 LSP) inherits a partly-typed
    `wasamoc` — type errors are not consistently surfaced. Phase 6
    will need the binding type-checker anyway (it needs to know
    that `if <expr>` is bool-typed); deferring forces Phase 6 to
    write the checker that Phase 1 could have written first.

Option C — No static type checking for bool in Phase 1; everything
becomes a runtime error
- All the rejections in the table above land at runtime as
  `EvalError::TypeMismatch`.

  - What you give up: Worst error ergonomics; same Phase 6 debt as
    Option B but larger.

**Recommendation:** Option A. The cost of building the state-type
table in `wasamoc` is the same as Phase 6 would pay anyway for
conditional rendering's expression typing; doing it now keeps
diagnostics consistent and removes a Phase 6 prerequisite. M3 spec
public draft (A12) is also better served by a checker that rejects
malformed type combinations at the source line.

Identifier resolution is the load-bearing detail: in Option A,
`ready` becomes `BoolPropRead` at *lowering* time by consulting the
state-type table, not at runtime by trial-and-error on `EvalContext`
methods. This locks in DD-M3-P1-003's expression shape (one
identifier → one typed PropRead variant).

---
