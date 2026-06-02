### DD-P7-003 — Experimental layer surfacing in bindings

**Status:** Accepted

**Context:**
[abi_spec.md §5](../../../../docs/abi_spec.md) marks roughly half the C ABI surface
`WASAMO_EXPERIMENTAL` (the all-at-once widget constructors and
`wasamo_button_set_clicked`). Bindings must propagate this marker
in language-idiomatic ways, otherwise hosts learn the experimental
boundary only by reading the C header.

**Options:**

Option A — Module split: `wasamo::experimental` (Rust); equivalent in Zig (recommended)
- Rust: stable-core C ABI → safe wrapper at the crate root
  (`wasamo::Window`, `wasamo::Widget`, `wasamo::Value`, etc.).
  Experimental constructors → `wasamo::experimental` submodule
  (`wasamo::experimental::button`, `::vstack`, etc.) with a
  module-level docstring stating the M1 stability story.
- Zig: same shape — `wasamo.zig` exposes stable-core types at the
  top level; experimental constructors live in `wasamo.experimental`
  namespace.
- C: header inherits `WASAMO_EXPERIMENTAL` markers as-is. No
  separate header; the marker is the boundary signal.

- What you gain: `use wasamo::experimental::*` is a visible signal
  in source — code review and grep both see it. Symmetric across
  Rust and Zig. Costs nothing if a future `wasamoc`-codegen path
  retires the experimental layer (the module empties out, the
  stable-core API is unaffected).
- What you give up: Slightly more module-organization work in the
  wrapper crate. Negligible.

Option B — Cargo feature flag (`features = ["experimental"]`)
Experimental constructors are gated behind a non-default cargo
feature.

- What you gain: Hosts that don't enable the feature cannot
  accidentally call experimental functions.
- What you give up: For M1, **every** host needs the experimental
  layer (M1 `wasamoc` is parser-only, the stable core has no
  tree-construction primitive). Defaulting it off means defaulting
  every M1 host to a broken state. Defaulting it on makes the
  feature flag decorative. Cargo features add CI matrix complexity
  for no M1 protection.

Option C — Same crate, no separation; rely on docstrings
Experimental and stable wrappers sit side by side at the crate
root, distinguished only by `#[doc = "EXPERIMENTAL — ..."]`.

- What you gain: Smallest implementation cost.
- What you give up: Source-level visibility of the boundary is
  weak. `vstack(...)` and `Window::run(...)` look identical at the
  call site. The C side took pains to mark its experimental
  surface; bindings should match.

**Recommendation:** **Option A.** Module split is the cheapest way
to make the experimental boundary structurally visible in source,
and it survives both DSL-codegen-to-Rust and DSL-IR-to-runtime as
the M2 path (the experimental module either empties out or is
retained as a hand-builder escape hatch).

---
