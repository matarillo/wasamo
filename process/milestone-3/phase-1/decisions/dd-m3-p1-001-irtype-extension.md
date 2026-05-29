### DD-M3-P1-001 — `IrType` extension

**Status:** Accepted

**Context:**
`IrType` is a two-variant enum (`I32 | Str`) that tags `state`
declarations and disambiguates the type-suffixed `HandlerExpr` variants.
A9 requires that `bool` becomes a first-class declaration type, so
`state foo: bool = false` parses and the resulting `IrState` carries
something distinct from `I32` and `Str`.

**Options:**

Option A — Add `IrType::Bool` variant (recommended)
- `IrType` becomes `I32 | Str | Bool`. Additive.

  - What you gain: One-to-one with the surface-level type vocabulary.
    Pattern-matching exhaustiveness in `wasamoc` / `wasamo-runtime`
    forces every site that branches on type to handle `Bool` —
    compiler-enforced completeness.
  - What you give up: Every existing `match` on `IrType` in the
    workspace needs a `Bool` arm. The set is small and discoverable;
    no abstraction debt.
  - **Technical risk:** Low. Pure enum extension; no FFI / wire format
    changes outside this phase's own work.

Option B — Encode `bool` as `IrType::I32` with a flag/refinement
- Reuse `I32`; treat `0`/`1` as falsy/truthy throughout.

  - What you gain: Zero new variant.
  - What you give up: Loses the type tag at the IR boundary, which is
    where M2 deliberately placed it (DD-M2-P6-002 chose tagged
    representation for exactly this reason). Re-opens the typing
    discipline of a settled DD.
  - **Technical risk:** Low to implement; high to live with — every
    later phase that touches `bool` would need to re-derive "is this
    bool-typed or i32-typed?" from context.

**Forward-compat exposure:**
Option A's exposure under foreseeable future events (see Out of scope):
when `TypedValue` is reconsidered after M3, an `IrType::Bool` variant
naturally maps to a `TypedValue::Bool` arm — strictly additive. Option
B would require *splitting* `I32` into `I32` + `Bool` retroactively,
which is exposure to the same future event but reversed.

**Recommendation:** Option A. The type-suffix pattern is the M2
discipline; extending it for `bool` is the additive path. Design
quality dominates here: a refinement-flag scheme would be a footgun
the rest of M3 has to navigate around.

---
