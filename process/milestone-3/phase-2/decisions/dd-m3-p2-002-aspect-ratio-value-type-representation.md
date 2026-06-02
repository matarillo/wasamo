### DD-M3-P2-002 — `aspect: <ratio>` value-type representation

**Status:** Accepted

**Context:**
Box's `aspect` attribute requires a value type representing an
aspect ratio (width:height). The type threads through `wasamoc` (lex,
parse, check, lower), `wasamo-ir` (literal form, IR text grammar),
and `wasamo-runtime` (loader, measure-arrange pass per DD-005).

Per DD-M3-P2-004's recommendation below, `aspect` is **constant-only**
in Phase 2 — no state binding. This loosens the value-type
requirement: a new `IrType::Ratio` for state declarations is not
needed; only a new `IrLiteral::Ratio` for the literal attribute value.

**Options (numeric form / surface syntax):**

Option A — Rational pair `aspect: <num>:<den>`, with both sides
unsigned integer literals (recommended)
- Surface syntax: `aspect: 16:9` or `aspect: 1:1`. Ratio literals
  are written as two unsigned integer tokens joined by `:`;
  `wasamoc` lexer emits a new `Ratio` token; parser produces
  `IrLiteral::Ratio { num: i32, den: i32 }`. `wasamoc check`
  rejects zero on either side at compile time; negative sides are
  not ratio literals (a leading `-` parses as unary minus and does
  not form `Ratio`), so the runtime cannot see `num < 0` /
  `den < 0`.

  - What you gain: Exact arithmetic for measure-arrange (Box bounds
    derive from `parent_width * num / den` or `parent_height * den /
    num` with no float rounding). NaN / infinity are structurally
    unreachable — `wasamoc check` already excludes zero / negative
    sides, so the runtime cannot see them. Surface syntax matches
    conventional aspect-ratio notation (16:9, 4:3, 1:1).
  - What you give up: Two integer fields per literal vs one float
    field. Lexer gains one new token form.
  - **Technical risk:** Low.

Option B — Float literal `aspect: 1.7778`
- Adds `IrType::F32` / `IrLiteral::F32` / `HandlerExpr::F32Lit` to
  the type-suffix chain (per Phase 1 pattern;
  [predoc-inputs.md §7](../requirements/constraints.md#7-f32--f64-を-irtype-に入れるかの再評価)).

  - What you gain: One numeric form for all aspect surfaces.
    `TypeName::Float` already exists in `wasamoc`'s AST, so the lex /
    parse cost is largely paid.
  - What you give up: `wasamoc check` must guard against NaN /
    infinity / zero / negative / sub-epsilon values. Runtime
    measure-arrange must handle float rounding (e.g. a 1920×1080
    parent + `aspect: 1.7778` produces 1919.616×1080 — rounding
    direction must be normative). Introduces F32 to IR / runtime
    without a Phase-2-internal need beyond aspect — the cost is
    real, the payoff narrow. F5 deferral note: adding `IrType::F32`
    creates a third scalar type with no widget property and no
    binding admitting it (DD-004 says constant-only), so the type
    enters the IR for one attribute's literal form only. Defensible
    but asymmetric.
  - **Technical risk:** Medium. Float-rounding spec is the part Phase
    2 would have to write.

Option C — Compile-time-parsed string `aspect: "16:9"`
- `wasamoc` parses the string at check time, producing the same
  `IrLiteral::Ratio` as Option A.

  - What you give up: Cosmetic inconsistency with non-string ratio
    literals across DSL ancestry (CSS, XAML use bare 16:9). Per
    [predoc-inputs.md §1](../requirements/constraints.md#1-box-が新規-propertyvalue-variant-を入れるなら-abi-value-conversion-arm-は同じ-step-に-fold-する),
    parse failure becomes a `wasamoc check` diagnostic, not a runtime
    fallback — same as Option A — so the strictness is identical;
    only the surface form differs. The lexer save (no new token form)
    is real but small.
  - **Technical risk:** Low.

Option D — Twin i32 attributes `aspect-width: 16` and `aspect-height: 9`
- No new value type at all — reuse existing i32 attributes.

  - What you give up: Surface fragmentation (one logical attribute
    written as two). A reader can write one without the other; spec
    must define the default and the both-absent / one-absent cases.
    Measure-arrange's "aspect set" branch becomes "both aspect-*
    set", which is a weaker structural signal than a single Ratio
    literal.

**IR / runtime plumbing (consequent on Option A; ABI surface
deliberately not extended):**

- New `IrLiteral::Ratio { num: i32, den: i32 }` variant in
  `wasamo-ir`.
- New internal domain type `Ratio` in `wasamo-runtime` (private to
  the crate). `WidgetData::Box` stores `aspect: Option<Ratio>` as a
  Box-internal field. `Ratio` is **not** a `PropertyValue` variant.
- **No** `PropertyValue::Ratio` variant in
  `wasamo-runtime/src/widget.rs`. Aspect is constant-only per DD-004,
  so it never traverses the `PropertyValue`-mediated paths
  (`get_property` / `set_property` / observer payload / binding
  writer). The `ir_loader` materialises `IrLiteral::Ratio` directly
  into the Box's internal field, bypassing `PropertyValue`.
- **No** ABI changes: `wasamo-runtime/src/abi.rs`
  (`read_property_value` / `write_property_value` /
  `property_value_to_owned`) is untouched in Phase 2; no
  `WASAMO_VALUE_RATIO` tag is added.
- IR text grammar (`docs/dsl_spec.md` §8) gains the `Ratio` literal
  production.
- Rationale: adding `PropertyValue::Ratio` without an ABI tag would
  create a structurally unreachable variant at the C ABI boundary
  (since `read_property_value` / `write_property_value` exhaustively
  match `PropertyValue` and pass through `WasamoValue.tag`); adding
  the tag would widen the public ABI surface ahead of the bindable
  surface DD-004 explicitly defers. Keeping `Ratio` Box-internal
  matches DD-004's constant-only intent and the Phase 1 discipline
  of "build the seam in the phase that needs it". Were DD-004 to flip
  to bindable for aspect, that phase would land
  `PropertyValue::Ratio` + `IrType::Ratio` +
  `HandlerExpr::RatioLit` / `RatioPropRead` +
  `WASAMO_VALUE_RATIO` tag + `abi.rs` arms together as one coherent
  bindable-surface step. See
  [predoc-inputs.md §1](../requirements/constraints.md#1-box-が新規-propertyvalue-variant-を入れるなら-abi-value-conversion-arm-は同じ-step-に-fold-する):
  the fold-in-same-step obligation triggers when `PropertyValue` does
  gain a variant; Phase 2 satisfies it by not adding a variant.

**Recommendation:** Option A — rational pair `aspect: <num>:<den>`
with positive-integer sides, `IrLiteral::Ratio` literal form, no
new `IrType` extension. The decision turns on three factors:
exact arithmetic, structurally unreachable NaN / infinity, and
surface-syntax familiarity. Option B's float path forces a
rounding spec on Phase 2 (where the framing already nuances "no
novel measure-arrange algorithm" — DD-005); Option C's string path
saves a lexer token at the cost of cosmetic friction; Option D
fragments the surface.

**Forward-compat exposure:** Option A's exposure under foreseeable
future events (see Out of scope):

- Theming / styling system (M4+) may want to express aspects from a
  named palette. `IrLiteral::Ratio { num, den }` survives — a
  theme-resolved aspect becomes a Ratio literal at theme-resolve time.
  Option B's float form survives the same event symmetrically.
- A future bindable-aspect surface (rejected for Phase 2 by DD-004)
  would add `IrType::Ratio`, `HandlerExpr::RatioLit` /
  `RatioPropRead`, and `WASAMO_VALUE_RATIO` tag at that point —
  additive in the established type-suffix pattern. Option A's
  exposure here is symmetric with Option B's: both add one new type
  to the chain.

---
