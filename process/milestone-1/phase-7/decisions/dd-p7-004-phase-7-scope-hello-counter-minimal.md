### DD-P7-004 — Phase 7 scope: Hello-Counter-minimal

**Status:** Accepted

**Context:**
The Phase 6 stable core has 13 functions; the experimental layer
adds 6 more. A "complete" binding wraps all of them. A
"Hello-Counter-sufficient" binding wraps only what `examples/counter`
will actually call. The acceptance criterion explicitly references
Hello Counter, not full ABI coverage.

**Options:**

Option A — Bind only what Hello Counter needs (recommended)
The Rust safe wrapper and Zig wrapper expose: lifecycle (init/run/
shutdown/quit), window create/show/destroy, the four experimental
constructors (`text`/`button`/`vstack`/`hstack`), `window_set_root`,
`button_set_clicked`, and `set_property`/`get_property` for at
least Button label and Text content. Other ABI entries (observers,
generic signal connect/disconnect, value packing for non-Counter
types) are added only if Phase 8 demonstrates they are needed.

- What you gain: Smallest binding surface. Phase 7 stays scoped.
  Anything not used by Phase 8 is by definition unverified, so
  binding it speculatively in Phase 7 is busywork that has to be
  re-checked anyway when a real consumer appears.
- What you give up: Hosts wanting more than Counter can do are
  blocked until a follow-up. Acceptable — M1 does not promise a
  complete binding, only enough binding for the milestone demo.

Option B — Full ABI coverage in M1
Wrap every `wasamo.h` entry in Rust and Zig.

- What you gain: Hosts have a complete surface from day one.
- What you give up: Phase 7 scope balloons. Most of the surface
  has no test consumer in M1 — observers and generic signal
  connect/disconnect are not used by Hello Counter, so wrapping
  them produces unverified code in the binding crate.

**Recommendation:** **Option A.** "Phase 7 produces what Phase 8
exercises" is the disciplined scope. Add to the bindings *during*
Phase 8 if a need surfaces; document the unbound entries in
`CONTRIBUTING.md` as "open for contribution." This also aligns
the binding's experimental surface area with what abi_spec §5.1
already commits to verifying in M1.

---
