# M2-Phase 7 / DD-M2-P6-011 pre-doc framing

**Status:** framing completed; F1/F2/F3/F4 aligned with owner (2026-05-10)
**Date:** 2026-05-10
**Targets DD:** DD-M2-P6-011 - String-typed property binding
**Targets phase:** M2-Phase 7 (Reactive Foundation Hardening & Contract Finalization)
**ADR housing:** [process/milestone-2/phase-7/decisions/preamble.md](../decisions/preamble.md)
**Progress tracker:** `docs/plans/progress/m2-phase-7-progress.md`
(retired at M2 close; summary remains in `docs/plans/m2-plan.md`)

このノートは、DD-M2-P6-011 の Option A / B / C を選ぶ前に、A6
("Type-Agnostic Reactive Binding") をどう読むかを揃えるための
pre-doc framing である。DD-010 / DD-012 と同じく、ここでの結論は
直接 ADR へ昇格するものではなく、ADR drafting の入力 artefact として扱う。

結論から言うと、DD-011 でも framing は必要である。ただし DD-010 / DD-012
より軽量でよい。理由は、DD-011 の未確定点が広い実装探索ではなく、
「A6 の type-agnostic を、将来型まで吸収する抽象化として読むのか、
それとも少なくとも i32 専用化でないことを M2 の String 経路で証明する
こととして読むのか」に集中しているためである。

---

## DD-011 question (restated)

DD-M2-P6-011 の問いは、`.ui` の String property binding を
`Signal<String>` から visible widget まで end-to-end に通すことである。

現状の Phase 7 ADR は Phase 6 draft を継承しており、Option B
(`HandlerExpr::StrPropRead`) を recommended としている。一方で、
ADR 自身が A6 framing の下では Option C (`TypedValue` unification)
が有利になる可能性を明記している。

したがって、pre-doc で決めるべき問いは次の一点である。

> A6 の "Type-Agnostic Reactive Binding" は、M2 の acceptance として
> `String` が既存の binding path を通ることを求めるのか、それとも
> evaluator / IR API を今すぐ fully typed-value 化することを求めるのか。

---

## Implementation evidence carried into DD-011

1. **IR already has String expression forms, but property reads are untyped.**
   `HandlerExpr` has `StrLit(String)` and `Interpolation(Vec<...>)`,
   but `PropRead { path }` carries no type tag. Interpolation currently
   evaluates embedded expressions through the integer tracked path.

2. **`SignalRegistry` already stores String Signals.**
   `SignalRegistry` has both `i32s` and `strings`. DD-M2-P6-007 landed the
   storage shape and pure-logic tests, but DD-011 is still the deferred step
   that makes `strings` participate in binding evaluation.

3. **`BindingEvalContext` is currently i32-only.**
   `EvalContext` exposes `get_i32`, `set_i32`, and `read_i32_tracked`.
   `BindingEvalContext` implements tracked i32 reads through `Signal::get()`;
   it does not yet expose `get_string` / `read_string_tracked`.

4. **The `.ui -> IR -> runtime` path lowers identifiers without type
   disambiguation at expression sites.**
   `wasamoc` lowers an identifier expression to `HandlerExpr::PropRead`.
   Runtime IR loading also parses `(prop-read NAME)` into the same variant.
   Therefore the DD-011 choice must decide where the property-read type is
   recovered: in the IR node, in a parallel variant, or in a typed-value
   evaluator API.

5. **DD-010 / DD-012 establish a useful precedent.**
   Phase 7 acceptance should not merely describe a future-correct design.
   It should land the production path that demonstrates the acceptance
   property. For DD-011, that means the accepted option must be implemented
   far enough that `.ui` String binding visibly updates through the reactive
   path.

---

## Why framing is still needed

Skipping framing would make the ADR choice look like a normal implementation
preference, but DD-011 is the only remaining A6 gate. The phrase
"Type-Agnostic Reactive Binding" can pull the design in two directions:

- **Minimal type expansion:** add the String read path while preserving the
  existing i32 path and the current `EvalContext` shape.
- **General typed-value abstraction:** introduce a `TypedValue`-style
  evaluator surface now, so future scalar types do not each add parallel
  methods or variants.

Both can plausibly claim to satisfy "not silently i32-specialized." The
framing step should fix which interpretation M2 actually needs before the
ADR recommendation is changed or reaffirmed.

---

## Proposed framing decisions

### F1 - A6 acceptance is demonstrative, not fully generic

For M2, A6 requires the shipped binding path to prove that it is not i32-only
by carrying a non-i32 property type (`String`) end-to-end. It does not require
the evaluator API to be fully generalized for all future scalar types before
M2 can close.

Consequence: Option C remains valid and documented, but it is not forced by
the wording of A6 alone. The `TypedValue` unification should be recorded as
a post-M2 revisit trigger, not forgotten as a rejected idea. The earliest
possible revisit point is M3 if the DSL surface work introduces another
scalar property type, item/context-typed binding expressions for List/Grid,
or a binding-language feature whose evaluator result cannot be cleanly
represented by the current parallel typed read methods. If M3's Grid /
ScrollView / List and public spec draft do not create that pressure, the
revisit should remain open for M4/M5/post-1.0 rather than being forced into
M3 merely because it is the next milestone. The live open question is tracked
in [docs/notes/typed-value-evaluator.md](../typed-value-evaluator.md).

### F2 - The accepted option must cover the `.ui` path, not only pure logic

DD-M2-P6-007 already proved that `SignalRegistry.strings` can store and read
`Signal<String>`. DD-011 must go further: a `.ui` String property bound to a
`Signal<String>` must flow through lowering / IR loading / `BindingEvalContext`
/ binding evaluation to the visible widget.

Owner alignment (2026-05-10): yes, pure evaluator tests alone are not enough.
However, the automated test boundary must respect the project testing policy
in `CLAUDE.md`: unit / integration tests should cover logic that has no
Win32 / WinRT FFI dependency, and must not mock HWND / Compositor / Visual
Layer / DirectWrite.

Consequence: DD-011 should include an automated test that starts from a `.ui`
fixture or its emitted IR and proves the String binding reaches the runtime
widget property state through the real lowering / loading / binding-evaluator
path. That test should not require a visible window, pixel inspection, or a
mock Visual Layer. Actual on-screen visibility remains part of the existing
Phase 6 GUI counter regression / phase-close manual verification, not a new
CI fixture.

An implementation that only adds `get_string` unit tests without this
`.ui -> runtime widget property state` demonstration does not discharge A6.

### F3 - Avoid broad evaluator churn unless it buys M2-visible correctness

Changing every integer read/write call site to a `TypedValue` API increases
blast radius across handler evaluation, binding evaluation, test stubs, and
IR tooling. If Option B or a narrow Option A can satisfy F1 and F2, Option C
should be treated as M3+ revisit material rather than required M2 work.

Owner alignment (2026-05-10): F3 is accepted. M2 should not adopt
`TypedValue` unification for DD-011. The preferred M2 recommendation is
Option B (`HandlerExpr::StrPropRead`) because it is additive, preserves the
existing integer `PropRead` path, and is sufficient to satisfy F1 / F2.

Consequence: the ADR should compare Option C honestly as the future-friendly
shape, but should not recommend it solely because its name sounds more
"type-agnostic." The ADR should instead recommend Option B for M2, while
recording `TypedValue` as the post-M2 open question tracked in
[docs/notes/typed-value-evaluator.md](../typed-value-evaluator.md).

F2 still constrains Option B: a hand-written `StrPropRead` unit test is not
enough. The implementation must include a `.ui` / emitted-IR path that reaches
`StrPropRead` (or the accepted String-read dispatch shape) based on the
declared state type, then proves the binding reaches runtime widget property
state.

### F4 - String binding should preserve the existing integer binding behavior

The DD-011 implementation must not regress bare integer bindings or
interpolation of integer expressions. Existing `PropRead` behavior and tests
are part of the Phase 6 acceptance surface that Phase 7 is hardening, not
reopening wholesale.

Owner alignment (2026-05-10): F4 is accepted. Under the Option B direction,
existing `PropRead { path }` remains the integer read form. String reads use
the accepted String-read form (`StrPropRead` under the current framing).

Consequences:

- Bare integer binding remains supported and continues to stringify the i32
  result for text binding.
- Integer interpolation remains supported and continues to track dependencies
  through `read_i32_tracked()`.
- Handler-side integer mutation (`Assign` / `CompoundAssign`) is not reopened
  by DD-011; existing counter-style handler behavior is regression-protected.
- DD-011 does not introduce broad implicit conversions. Cross-type reads must
  fail rather than silently coerce. The exact diagnostic (`UnknownProperty`
  vs `TypeMismatch`) may follow the existing registry / error shape unless
  the implementation can report `TypeMismatch` without broad churn.

Whichever option is accepted should include regression tests for the current
i32 binding / interpolation path alongside the new String path. As with F2,
those tests should target runtime state that can be verified without Win32 /
WinRT FFI; actual on-screen confirmation remains part of phase-close manual
GUI regression.

---

## Option pressure after framing

- **Option A - Type-tag `PropRead`:** viable if the loader / lowering layer can
  reliably attach type information at every property-read expression site.
  It keeps one read variant but forces construction-site churn.
- **Option B - `StrPropRead`:** recommended for M2. It is additive, keeps
  existing `PropRead` semantics stable, and gives the evaluator an explicit
  String read path. The accepted implementation must still provide a real
  `.ui` / emitted-IR path into this variant, not only hand-written runtime
  tests.
- **Option C - `TypedValue`:** strongest long-term abstraction, but broader
  than M2 needs if "type-agnostic" is read demonstratively. Keep as a
  documented M3+ revisit trigger unless implementation evidence shows Option
  A / B cannot support the `.ui` String binding path cleanly.

---

## Draft acceptance shape for DD-011

DD-M2-P6-011 can move from Proposed to Accepted when the ADR:

1. records the A6 interpretation from F1;
2. recommends Option B (`StrPropRead`) for M2 as the property-read
   disambiguation strategy;
3. requires a `.ui` / emitted-IR String binding demonstration through runtime
   widget property state, without adding a new Visual Layer CI fixture;
4. preserves the existing integer `PropRead` / interpolation / handler
   mutation behavior with focused regression tests;
5. records Option C as explicitly deferred to the post-M2 `TypedValue`
   open question.

Implementation can then proceed against that accepted shape.
