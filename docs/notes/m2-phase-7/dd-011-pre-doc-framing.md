# M2-Phase 7 / DD-M2-P6-011 pre-doc framing

**Status:** framing draft started; F1 aligned with owner (2026-05-10)
**Date:** 2026-05-10
**Targets DD:** DD-M2-P6-011 - String-typed property binding
**Targets phase:** M2-Phase 7 (Reactive Foundation Hardening & Contract Finalization)
**ADR housing:** [docs/decisions/m2-phase-7-reactive-foundation.md](../../decisions/m2-phase-7-reactive-foundation.md)
**Progress tracker:** [docs/plans/progress/m2-phase-7-progress.md](../../plans/progress/m2-phase-7-progress.md)

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

Consequence: an implementation that only adds `get_string` unit tests without
an end-to-end binding demonstration does not discharge A6.

### F3 - Avoid broad evaluator churn unless it buys M2-visible correctness

Changing every integer read/write call site to a `TypedValue` API increases
blast radius across handler evaluation, binding evaluation, test stubs, and
IR tooling. If Option B or a narrow Option A can satisfy F1 and F2, Option C
should be treated as M3+ revisit material rather than required M2 work.

Consequence: the ADR should compare Option C honestly as the future-friendly
shape, but should not recommend it solely because its name sounds more
"type-agnostic."

### F4 - String binding should preserve the existing integer binding behavior

The DD-011 implementation must not regress bare integer bindings or
interpolation of integer expressions. Existing `PropRead` behavior and tests
are part of the Phase 6 acceptance surface that Phase 7 is hardening, not
reopening wholesale.

Consequence: whichever option is accepted should include regression tests for
the current i32 binding path alongside the new String path.

---

## Option pressure after framing

- **Option A - Type-tag `PropRead`:** viable if the loader / lowering layer can
  reliably attach type information at every property-read expression site.
  It keeps one read variant but forces construction-site churn.
- **Option B - `StrPropRead`:** likely still sufficient for M2 if F1 is
  accepted. It is additive, keeps existing `PropRead` semantics stable, and
  gives the evaluator an explicit String read path.
- **Option C - `TypedValue`:** strongest long-term abstraction, but broader
  than M2 needs if "type-agnostic" is read demonstratively. Keep as a
  documented M3+ revisit trigger unless implementation evidence shows Option
  A / B cannot support the `.ui` String binding path cleanly.

---

## Draft acceptance shape for DD-011

DD-M2-P6-011 can move from Proposed to Accepted when the ADR:

1. records the A6 interpretation from F1;
2. chooses the property-read disambiguation strategy;
3. requires an end-to-end `.ui` String binding demonstration;
4. records Option C as either accepted now or explicitly deferred as the
   typed-value generalization revisit point.

Implementation can then proceed against that accepted shape.
