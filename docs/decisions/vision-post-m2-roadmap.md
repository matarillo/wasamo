# Vision ADR — Post-M2 roadmap restructuring

**Status:** Agreed 2026-05-02

**Scope:** ROADMAP.md (M2 onward) and VISION.md §7 / §10.2 / §11.

This vision ADR records the decisions that rebalanced post-M2
milestones after M2 was redefined as a Foundation milestone (see
[docs/plans/m2-plan.md](../plans/m2-plan.md)). The redefinition moved
the original M2 Alpha wishlist out of M2 without committing to where
each item lands; this ADR settles that allocation and the structural
questions surfaced alongside it.

The discipline applied: each post-M2 milestone carries a single
**thesis** (a hypothesis the milestone closes), not a feature list.
This is the same discipline that drove the M2 redefinition — applied
prospectively to M3–M6 so the wishlist failure mode does not recur.

## Context

The M2 redefinition (m2-plan, 2026-05-02) deferred eight questions to
a follow-up revision: post-M2 milestone structure, Grid + DSL spec
placement, IME / AccessKit / VS Code rebalance, Mica/Acrylic
positioning, C ABI freeze position, 1.0 binding list, showcase
placement, and ADR identifier discipline. These questions are
interdependent and were settled together rather than piecewise.

Two structural alternatives were considered before this ADR was
written:

- **Alt A (chosen, with refinement)** — four thesis-driven
  milestones M3–M6, each closing one hypothesis. Mica/Acrylic and
  multi-window pulled forward into M4 so identity is demonstrable for
  the first showcase and so cross-cutting ABI is settled pre-freeze.
  VS Code LSP runs as a parallel track gated on M3's DSL spec draft,
  with M5 as its acceptance gate.

- **Alt A' (rejected)** — three milestones (M3 / M4 / M5 = 1.0),
  collapsing Identity & tooling into 1.0. Rejected because the
  resulting M4 would re-bundle input + IME + AccessKit + Mica +
  multi-window + showcase, reproducing the wishlist failure mode the
  M2 redefinition was meant to prevent.

## DD-V-005 — Post-M2 milestone structure

**Status:** Accepted

**Context:** The original ROADMAP listed four post-M1 milestones (M2
Alpha / M3 Beta / M4 1.0 / M5+) defined by feature lists. The M2
redefinition exposed that list-based milestones obscure verification
scope. A successor structure must avoid re-bundling the same wishlist
under new labels.

**Options:**

Option A — Thesis-driven four-milestone structure (M3 DSL surface /
M4 Interaction stack / M5 Identity & tooling / M6 1.0)
- What you gain: each milestone has a verifiable thesis; matches the
  M2 redefinition discipline; preserves a separate milestone for
  freeze-readiness work.
- What you give up: 1.0 is one milestone farther out than the
  original ROADMAP; "Alpha / Beta" labels familiar from OSS
  conventions are dropped.

Option B — Three milestones with VS Code LSP and identity work
folded into 1.0
- What you gain: 1.0 arrives one milestone sooner.
- What you give up: M4 inflates to roughly the size of the rejected
  original M2; thesis discipline weakens.

Option C — Preserve Alpha / Beta labels with rebalanced contents
- What you gain: continuity with established OSS milestone vocabulary.
- What you give up: the labels signal release maturity, but with
  M2 = Foundation already breaking the pattern, retaining Alpha /
  Beta on M3–M5 conflicts with their thesis-driven framing.

**Decision:** Option A — thesis-driven structure. Alpha / Beta labels
are dropped; milestones are named after their theses.

## DD-V-006 — Multi-window placement

**Status:** Accepted

**Context:** The original ROADMAP placed multi-window in M5+
(post-1.0). Multi-window has cross-cutting ABI implications
(per-window state, cross-window focus, event routing) that an
append-only post-freeze additive surface cannot accommodate cleanly.

**Options:**

Option A — Keep in post-1.0 with reserved extension points in the 1.0
ABI design
- What you gain: smaller pre-1.0 surface; faster path to freeze.
- What you give up: forces the 1.0 ABI design to predict multi-window
  shape without a working implementation; high risk of needing a
  breaking change post-freeze.

Option B — Move to M4 alongside the input / focus model
- What you gain: focus model and cross-window event routing are
  designed together; 1.0 freeze rests on a verified multi-window ABI.
- What you give up: M4 grows by one acceptance criterion.

Option C — Dedicated multi-window milestone between M4 and M6
- What you gain: isolates the multi-window investigation.
- What you give up: a milestone for what is essentially one feature
  is structurally heavy.

**Decision:** Option B — M4. Multi-window is part of the Interaction
stack thesis because it shares the focus model with input, IME, and
accessibility.

## DD-V-007 — Mica/Acrylic and showcase placement

**Status:** Accepted

**Context:** VISION §1 / §4.1 / §5 frame Mica / Acrylic as an
identity feature differentiating Wasamo from cross-platform
alternatives. The original ROADMAP placed full Mica / Acrylic in M3
Beta. VISION §10.2 commits to "shipping showcase apps early" as a
strategic-risk mitigation, but no milestone anchored that commitment.

**Options:**

Option A — Mica in M5; showcase implicitly tied to 1.0 quality
- What you gain: keeps Mica with the broader theming work.
- What you give up: identity feature is undemonstrable for three
  milestones after M2; first showcase has no identity to point at;
  VISION §10.2's "early" commitment is in tension with the timing.

Option B — Mica in M4; first showcase ships in M4; polished
showcase ships in M6 (1.0)
- What you gain: identity is demonstrable from the first
  contributor-outreach showcase; VISION §10.2 has a concrete
  milestone anchor; full theming work in M5 builds on Mica already
  shipped.
- What you give up: M4 grows by one acceptance criterion; theming
  work is split across M4 (Mica) and M5 (full surface).

Option C — Mica in M3 alongside the DSL spec draft
- What you gain: identity demonstrable as early as possible.
- What you give up: M3's thesis (DSL is expressive enough to write
  real layouts) is diluted by a non-DSL concern; the spec draft would
  need to commit to material rendering semantics earlier than
  necessary.

**Decision:** Option B — Mica / Acrylic in M4, first showcase in
M4, polished showcase in M6. M3 reserves DSL syntax for material
without committing to its rendering semantics.

## DD-V-008 — 1.0 binding list and bindings governance

**Status:** Accepted

**Context:** The original ROADMAP M4 listed "Rust / Swift / Zig / Go
bindings mature" as a 1.0 acceptance criterion. M1 verified C /
Rust / Zig end-to-end. Swift and Go have zero verification investment
to date. VISION §11 separately stated "Official bindings are limited
to C and Rust", inconsistent with §7's M4 list.

**Options:**

Option A — 1.0 = Rust / Swift / Zig / Go; add a pre-1.0 milestone
for Swift / Go prototyping
- What you gain: preserves the original 1.0 ambition.
- What you give up: a binding-prototyping milestone repeats the
  wishlist pattern; commits to language coverage that is not
  empirically validated.

Option B — 1.0 = C / Rust / Zig (M1-verified set); Swift / Go
reclassified as post-1.0 community bindings
- What you gain: 1.0 acceptance reflects actual investment; honest
  positioning; aligns VISION §7 and §11.
- What you give up: pulls back the apparent reach of "1.0 supports
  these languages".

Option C — 1.0 = C / Rust only (literal current §11 wording)
- What you gain: maximum conservatism on the 1.0 commitment.
- What you give up: contradicts M1's investment in Zig (Phase 7 ADR
  DD-P7-005 made Zig a first-class binding); demotes verified work.

**Decision:** Option B — 1.0 binding list is C / Rust / Zig. VISION
§11 is updated to match. Swift / Go are welcomed as community
bindings post-1.0 and are not on the 1.0 critical path.

## DD-V-009 — Hot reload and ADR identifier discipline

**Status:** Accepted

**Context:** Two smaller questions deferred from the M2 redefinition,
recorded here for completeness:

- The original ROADMAP placed hot reload in M3 Beta. Hot reload's
  feasibility depends on the wasamoc output format decided in
  M2-Phase 2 (codegen vs IR + interpretation). It does not affect ABI
  freeze.
- Phase numbering became milestone-local at M2, creating an ADR
  identifier collision risk between M1's `DD-P<N>-<seq>` and future
  M2+ phase ADRs.

**Decision (hot reload):** Hot reload moves to post-1.0. It is pure
tooling / runtime work that does not constrain the ABI freeze, and
its design is gated by M2-Phase 2's output format decision. Placing
it in 1.0's critical path would couple freeze readiness to a tooling
choice that has nothing to do with ABI stability.

**Decision (ADR identifier):** ADR identifier scope from M2 onward
is `M<N>-P<n>` (e.g. `DD-M2-P2-001`); see
[README.md](./README.md#file-naming). M1 phase ADRs remain
`DD-P<N>-<seq>` as historical record and are not renumbered.
