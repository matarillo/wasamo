### DD-P8-001 — `counter.ui` positioning in Hello Counter

**Status:** Accepted

**Context:**
[`examples/counter/counter.ui`](../../../../examples/counter/counter.ui)
already exists from Phase 1 as the canonical reference example for
the `.ui` DSL. Phase 8's three host programs need to satisfy
"Hello Counter runs in three languages." How they should relate to
`counter.ui` is not free of choices.

[abi_spec §5.1](../../../../docs/abi_spec.md#51-what-m1-experimental-verifies-and-what-it-does-not)
already states the principle: "M1 wasamoc is parser-only by design;
host code constructs the equivalent tree directly through the
experimental layer. The lowering itself is M2 scope." This ADR
applies that principle to Phase 8 and decides how visible the
distinction is in the deliverables.

**Options:**

Option A — `.ui` is reference-only; host code is hand-written (recommended)
`counter.ui` is left untouched. Each `examples/counter-{c,rust,zig}/`
program constructs the same widget tree imperatively through the
experimental ABI / its safe wrapper. The example READMEs cross-link
to `counter.ui` and explicitly state that the lowering is M2.
`wasamoc check examples/counter/counter.ui` continues to pass and
is wired into CI.

- What you gain: Aligned with abi_spec §5.1 and VISION §7 M1
  (which scopes M1 to runtime-side ABI mechanics). Phase 8 ships
  the runtime/ABI/Visual-Layer hypothesis check that M1 is
  actually about. No new dependency on M2 design questions
  (codegen vs IR) that would have to be settled prematurely. The
  smallest implementation surface.
- What you give up: A reader who arrives at Phase 8 expecting to
  see `.ui → runtime` will find host-imperative code instead. This
  is honest about the M1/M2 split, but it puts a documentation
  burden on the example READMEs and the README Quick Start to
  explain it without sounding apologetic.

Option B — Hand-translation contract
Same as A, plus each `main.*` carries a header comment annotating
which `counter.ui` lines each block of imperative code corresponds
to. Reviews check the two against each other.

- What you gain: Makes the future `.ui → runtime` lowering visible
  as a structural mapping; readers see the M2 codegen target in
  rough shape without the codegen.
- What you give up: Ongoing review/maintenance overhead. If
  `counter.ui` evolves (e.g., signal body changes), three host
  files need synchronised edits or the comments rot. The benefit
  is documentation-only; the actual M2 lowering work is not made
  easier by these comments because M2 will operate on the AST,
  not the textual form.

Option C — Drop `counter.ui` from Phase 8 framing entirely
Treat `counter.ui` purely as a Phase 1 wasamoc artifact, and
drop the cross-linking from Phase 8 examples. M1 acceptance reads
"three host programs that produce the counter behavior."

- What you gain: No reader-expectation gap; what you see is what
  Phase 8 ships.
- What you give up: Severs the visible connection between the
  three M1 pillars (DSL, C ABI, Visual Layer). VISION §1 frames
  Wasamo as "DSL × C ABI × Visual Layer"; deleting `.ui` from the
  Hello Counter narrative makes one pillar invisible at the
  showcase moment.

**Recommendation: Option A.**

Option A is what abi_spec §5.1 already implies; Phase 8 just
applies it to a concrete deliverable. Option B's
hand-translation comments are documentation-shaped work whose
ongoing cost outweighs the documentation-shaped benefit, and they
risk reading as a partial codegen design that M2 might not
follow. Option C achieves cleanliness by removing one of the M1
pillars from view, which trades long-term framing for short-term
clarity — the wrong direction.

**Upstream document alignment.**

The choice does not require any *substantive* change to upstream
documents — the M1 / M2 split is already in abi_spec §5.1 and the
VISION §7 M1 paragraph. Two small wording tweaks reduce reader
friction:

1. **ROADMAP Phase 8 task list, item 1** currently reads
   `examples/counter/counter.ui`, which is misleading since the
   file already exists from Phase 1. Revise to make clear that
   Phase 8 only verifies it still parses, and add an item for the
   READMEs that carry the M1/M2 framing message:
   - `[ ] verify examples/counter/counter.ui still parses with wasamoc check (already exists from Phase 1)`
   - `[ ] each example README explains: this is the M1 host-imperative shape; .ui → runtime lowering is M2`
2. **VISION §7 M1 paragraph** is correct but stops short of
   naming the visible consequence. Add one sentence at the end of
   the M1 paragraph (after the existing "wasamoc output format…"
   sentence): "Concretely, the M1 Hello Counter examples
   construct the widget tree imperatively through the C ABI; the
   `.ui → runtime` lowering arrives in M2."

VISION §1 ("UI is written in an external DSL … and consumed from
any language through a stable C ABI") is the project's long-term
framing and is correct without M1 caveats. M1-specific qualifications
belong in §7 and in `abi_spec.md`, not in §1.

abi_spec §5.1 needs no change.

**Explicitly deferred:**
- Whether the `counter.ui` source file moves under
  `examples/counter/` or gets a sibling `examples/counter-ui/`
  with its own README. Current placement (file alone in
  `examples/counter/`) is fine; reorganization is M2's call when
  `wasamoc` produces an actual artifact from it.

---
