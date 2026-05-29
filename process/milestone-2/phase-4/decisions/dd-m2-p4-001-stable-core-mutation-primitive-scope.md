### DD-M2-P4-001 — Stable-core mutation primitive scope

**Status:** Accepted

**Context:**
A4 explicitly puts tree-mutation primitives in the C ABI. The question
is which of the six operations enumerated above are exposed at the C
ABI in M2, and which stay internal-Rust. Internal Rust API is required
for all six regardless (Phase 5 / Phase 6 consumers). The decision is
about which subset crosses the boundary.

**Options:**

Option A — Mutation only; constructors stay experimental (recommended)
- Stable-core promotion: `wasamo_widget_append_child`,
  `wasamo_widget_insert_child`, `wasamo_widget_remove_child`,
  `wasamo_widget_replace_child`, `wasamo_widget_destroy`.
- Property batching: see DD-M2-P4-004; this DD does not commit either
  way.
- Construction primitives (`wasamo_text_create`, `wasamo_button_create`,
  `wasamo_vstack_create`, `wasamo_hstack_create`,
  `wasamo_window_set_root`) stay in the **M1 experimental** layer
  unchanged. Hosts that mutate trees use experimental constructors to
  obtain widgets and stable mutators to compose them.

- What you gain: Cleanly satisfies A4's "no longer the only way to
  construct UI" clause via the DSL path (Phase 6 makes
  `wasamoc`-emitted IR the primary construction route; the M1
  experimental layer is the secondary route, and the new mutation
  primitives are the tertiary route layered on top of either).
  Construction is the design-loaded surface (spacing / padding /
  alignment / typography style — every constructor is a parameter
  set that is going to grow); deferring its stable-core promotion to
  the phase that actually has DSL-level vocabulary (M3 DSL spec
  draft) avoids freezing parameter shapes at M2 for a surface no M2
  acceptance criterion exercises.
- What you give up: Hosts that want to construct trees imperatively
  in M2 still depend on `WASAMO_EXPERIMENTAL` symbols. Acceptable —
  no acceptance criterion demands a stable construction surface in
  M2; A1 routes construction through the DSL.
- **Technical risk: Low.** All five mutator functions are mechanical
  wrappers over Rust API that Phase 4 implements anyway. The
  promotion adds no new failure modes beyond the boundary checks
  every C ABI function does (null pointer, valid widget handle,
  index in range). Header generation method (DD-P6-006 = A,
  hand-written) absorbs the additions as edits to `wasamo.h` plus a
  CI smoke-test extension.

Option B — Mutation + stable constructors (deprecate experimental layer)
- Stable-core promotion: all of Option A's mutators **plus** stable
  versions of `wasamo_text_create`, `wasamo_button_create`,
  `wasamo_vstack_create`, `wasamo_hstack_create`,
  `wasamo_window_set_root`. Experimental constructors are marked
  superseded; the `WASAMO_EXPERIMENTAL` block shrinks toward empty.

- What you gain: A4's "no longer the only way to construct UI"
  reads more strongly — host code can construct UI from the stable
  core alone, with no experimental dependency.
- What you give up: Constructor parameter shape becomes a stable
  commitment in M2. `wasamo_vstack_create` today takes only
  children (spacing / padding / alignment defaulted at the runtime
  side per `abi_spec.md §5`). Promoting it to the stable core means
  either (a) freezing the "no parameters beyond children" shape
  until M4 (and adding setters for each axis post-construction —
  fine but pre-commits the parameter axes) or (b) expanding the
  constructor signatures now (and freezing those expansions at M4).
  Both paths overrun M2-Phase 4's scope: parameter design belongs
  with the DSL spec work in M3.
- **Technical risk: Medium.** Mechanically the same as Option A,
  but the design surface is bigger. The risk is "shapes locked at
  M2 turn out wrong by M3 DSL spec time" — a forward-compat shape
  more than an implementation shape. M3's DSL spec draft is the
  natural place to settle constructor parameter axes (it has to
  enumerate them anyway for grammar reasons); committing them in
  M2-Phase 4 with no DSL grammar to align against is premature.

Option C — Mutation + detach/destroy only; no append/insert/replace promotion
- Stable-core promotion: `wasamo_widget_destroy` (detach optional —
  see DD-M2-P4-003). All `append`/`insert`/`replace` stay internal,
  reachable only through the experimental construction path
  (`wasamo_vstack_create` etc.).

- What you gain: Smallest stable-core growth. Avoids committing to
  an attach API shape until a host requirement appears.
- What you give up: A4 unsatisfied. The experimental layer remains
  the *only* way to compose a tree, just with the additional ability
  to dispose of one. The M2 acceptance criterion requires that
  experimental construction is no longer the only way to construct
  UI; Option C is at best a partial answer.
- **Technical risk: Low** (smallest surface), but the acceptance
  argument is weak — Option C would need the owner to either accept
  a narrowed-A4 reading or open a vision decision record redrafting A4. Not
  recommended without that prior step.

**Recommendation:** **Option A.**

The four-mutator + destroy stable-core surface is the smallest set
that satisfies A4 without pre-committing to the constructor design
work that belongs with the DSL spec in M3. Hosts in M2 obtain widgets
through experimental constructors (acknowledged as transient by the
`WASAMO_EXPERIMENTAL` marker) and compose them through the new stable
mutators; the DSL-driven path (Phase 6) is the primary route for
production code and does not touch the stable-core mutators at all.
Option B's constructor promotion is rejected on premature-freeze
grounds (parameter shape decisions land cleaner alongside DSL grammar
in M3). Option C is rejected on A4-coverage grounds.

The split between Option A's stable-core mutators and the experimental
constructor layer is intentionally asymmetric: mutation primitives are
a small, narrow ABI surface (parent + child + index/anchor + handle
out-param), while constructors carry every per-widget design decision.
Asymmetric promotion lets us freeze the structural-but-design-light
half now without freezing the design-heavy half.

**Forward-compat exposure:** Options differ. The relevant out-of-scope
items are M3 DSL spec finalisation (constructor parameter axes) and
post-M2 hosts that want imperative tree construction without the DSL.

- Option A leaves M3 free to add stable constructors with whatever
  parameter axes the DSL spec settles on, with no prior commitment to
  unwind. The new mutators survive trivially because their signatures
  are about widget *handles*, not widget *types* or *parameters*.
- Option B commits constructor parameter shapes at M2. If M3 DSL spec
  grammar needs different axes, the M2 stable constructors either
  get parallel "v2" siblings (ABI bloat) or get superseded by them
  (ABI churn, defeats the M4 freeze story).
- Option C delays the question one phase at the cost of leaving A4
  unsatisfied; it doesn't reduce forward-compat exposure compared to
  Option A.

This axis reinforces the Option A recommendation: minimum forward-
compat exposure for the M2 phase that has no DSL grammar to align
against yet.

**Technical-risk re-evaluation:** Option A is the lowest-impl-risk
option that meets A4. Option B's risk is design-quality (forward-
compat exposure on constructor shape), not implementability. Option
C is acceptance-coverage-deficient.

---
