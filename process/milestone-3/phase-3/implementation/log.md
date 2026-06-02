## Decisions log

### 2026-05-21 — Lexer prerequisite for kebab-case attribute names and negative integer literals (T1)

dsl_spec §4.10's three WrapPanel attribute names are kebab-case
(`item-cross-size`, `item-spacing`, `line-spacing`). The
`wasamoc` lexer prior to T1 only recognised the `in-out` keyword
through a hardcoded special case in `scan_ident` and rejected `-`
in identifier position elsewhere; the top-level `-` handler
rejected `-` not followed by `=`. Both surfaces blocked T1:

- Without kebab-aware `scan_ident`, `item-cross-size: 88`
  tokenized as `item` Ident, then `-` "unexpected `-`" error.
  The PropertyBind never reached the checker, so the attribute
  diagnostics had no surface to fire on.
- Without negative-IntLit support, `item-spacing: -1` errored
  at the lexer. The DD-M3-P3-006 compile-time gate ("diagnostic
  names the rejected attribute") was unreachable.

The owner confirmed the lexer-extension approach on 2026-05-21
(over the alternative of revising the spec to snake_case
attribute names):

- `scan_ident` now loops over alphanumeric segments joined by `-`
  when the next character after `-` is alphabetic. `count -= 1`
  still tokenizes correctly because the `=` after `-` breaks the
  kebab continuation rule.
- The `in-out` keyword's hardcoded entry in `scan_ident` is
  removed; `in-out` is matched via the post-scan keyword table
  on parity with `component` / `inherits` / etc.
- The top-level `-` arm emits a negative `IntLit` when followed
  by an ASCII digit. Bare `-` remains an error; binary
  subtraction is not grammatical in the DSL, so the leading-sign
  reading is unambiguous in expression position.
- **The negative entry path is integer-only.** `scan_number`'s
  fractional (`-1.5`) and `px`-unit (`-12px`) branches are
  rejected at lex time when entered with `negative == true`,
  yielding diagnostics that point to integer literals as the
  only legal negative form. `FloatLit` and `Measurement` remain
  unsigned per dsl_spec §5 (AST table) and §2 (measurement
  surface). This bound was introduced after rev 1 of the T1
  retrospective surfaced a scope-widening issue in owner review;
  see commit `bf7aee0` and `t1-step-end-retrospective.md` rev 2.
- Ratio literals (`<num>:<den>`) remain unsigned by construction
  (dsl_spec §4.9 surface). The negative-entry path in
  `scan_number` skips the ratio fold.

**Behavioural deltas from the pre-T1 lexer:**

- `in-outx` previously errored at lex time (`scan_ident` saw
  `in`, hit `-` not followed by `out`-end, returned `in`, then
  the `-` arm errored). It now lexes as `Token::Ident("in-outx")`.
  No existing program shape relied on the previous rejection; the
  parser would still reject `in-outx` in any meaningful context.
- The lexer test `in_out_followed_by_alphanumeric_is_error` was
  replaced with `in_outx_lexes_as_kebab_ident` plus six new
  kebab / negative-literal tests:
  - `kebab_case_ident`
  - `kebab_ident_breaks_on_non_alpha_after_hyphen`
  - `negative_int_literal`
  - `negative_int_in_property_bind_position`
  - `negative_float_literal_rejected` (rev 2; `-1.5` reject)
  - `negative_measurement_literal_rejected` (rev 2; `-12px` reject)

The progress doc's "Phase 3 introduces no new parser grammar"
prose continues to hold — the parser's IDENT-keyed PropertyBind
shape is unchanged. The lexer's `Token::Ident` surface widens
(kebab segments admitted) and the integer-literal sign domain
widens (negative IntLit admitted); `FloatLit` / `Measurement` /
`RatioLit` surfaces remain unsigned. This is recorded so the
Phase 3 closing Moment 2 spec re-sync can decide whether the
lexer change deserves its own note in dsl_spec §2 (lexical
surface) — and specifically whether the "IntLit signed,
FloatLit / Measurement unsigned" split needs a one-line
confirmation in §2 / §5 — or is sufficiently absorbed by
§4.10's normative attribute-name list and the existing AST
table.

### 2026-05-22 — Gallery sub-screen numerics restored to ADR canonical `88 / 12 / 12` (T9 rev 2)

Initial T9 landing (commit `d1e5ba6`) used
`item-cross-size: 120; item-spacing: 16; line-spacing: 16` with
8 thumbnails because, on the default 800×600 window
(≈ 784 px client width), the ADR-canonical `88 / 12 / 12` with
8 thumbs produces a `7 + 1` wrap which is visually unbalanced. The
deviation was un-documented at landing time; owner review
([t9-step-end-retrospective.md rev 2](../retrospectives/t9.md))
flagged that the ADR
[§Phase 3 verification closure item 1 (sub-screen positive control)](../decisions/preamble.md#phase-3-verification-closure-what-counts-as-a3-evidence)
and item 4 (CI integration fixture) both reference `88` as the
canonical example, so the implementation should match unless the
deviation is recorded.

Resolution (commit set landed after `e89a423`): restore
`88 / 12 / 12` and grow the thumbnail count from 8 to 10 (still
within framing decision E's 5–10 ceiling). On the same default
window the layout becomes `7 + 3` — visible wrap is preserved and
the numerics match the ADR-canonical example. The choice "match
ADR canonical example, adjust count for visible balance" was
selected over the alternatives "keep `120 / 16 / 16` with a
deviation note" and "keep 8 thumbs with `7 + 1` wrap" after the
review. No ADR or spec change is required by this revert — the
ADR's normative content does not pin the gallery to specific
dimensions; it pins the integration fixture (T8, unaffected) and
references the sub-screen example as `item-cross-size: 88` in the
positive-control description.

---

## CI / verification log

(empty — populated as T1–T10 land; see Phase 2 progress file for
the shape.)
