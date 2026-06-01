# DD-M3-P6-006 — Window-title host-wiring (R1) surface

**Status:** Proposed
**Phase:** M3-Phase 6
**Carries:** R1 (Gallery host Window-title wiring), Phase 4 residual
assigned to Phase 6 as owning phase (Phase 5 FD-E,
[../requirements/constraints.md §1](../requirements/constraints.md))

## Context

R1 is a host-wiring gap, not a binding feature. The `.ui` declares a
component-level `title:` (`component Gallery inherits Window { title:
"Gallery"; … }`); `wasamoc` lowers it correctly — component-level
prop-binds are spliced into the root IR node's props
([wasamoc/src/lower.rs:58-59](../../../../wasamoc/src/lower.rs)), so
`component.root.props` carries `title` as an `IrLiteral::Str`. But the
runtime drops it: `wasamo_load_ui` builds the widget tree, then calls

```rust
let mut window = crate::window::create(DEFAULT_WINDOW_TITLE,   // "Wasamo"
    DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)?;
```

([abi.rs:1220](../../../../wasamo-runtime/src/abi.rs)), so every loaded
window shows `"Wasamo"` regardless of the declared title (the Phase 4
smoke observation:
[constraints.md §1](../requirements/constraints.md), Q2). The IR
loader's `construct_widget` only reads props each widget kind
recognises, and the window is created with a constant.

**Required completion condition** (owner intent, FD-D / constraints
§1): the runtime/host path **applies the component-level static
`title:` to the native window** — not "title is declared unsupported".

The framing (FD-D) additionally asks Phase 6 to **evaluate** a
**dynamic** (`String`-binding-driven) title — because Phase 6 is the
phase where binding reaches from property into tree structure — but
does **not** require committing to it. The question must be evaluated
and its disposition recorded, not silently closed.

Relevant shapes:

- `IrComponent { name, base, states, root }`; `component.root.props`
  holds the spliced `title` (and `backdrop` / `theme`, also unwired —
  Q2).
- `wasamo_load_ui` owns window creation internally
  ([abi.rs:1220](../../../../wasamo-runtime/src/abi.rs)); `component`
  is still in scope at that point (`build_widget_tree` borrows it).
- `window::create(title: &str, width, height)` already takes the title
  ([window.rs:57](../../../../wasamo-runtime/src/window.rs)) — the
  plumbing exists; only the call site passes a constant.

## Decision dependency summary

**No cross-DD coupling.** R1 is technically independent of the
conditional / ZStack work (constraints §1: co-located by timing via
FD-E, not by dependency), so neither sub-issue here constrains or is
constrained by another DD; it does not appear in the preamble §Cross-DD
decision dependencies index. Both sub-issues (static path, dynamic
title) are decided wholly within this DD.

## Sub-issues

- **Static title host path** (R1 required completion condition): by
  what mechanism does the static, declared component-level `title:`
  reach the native window?
- **Dynamic (`String`-binding) title** (FD-D evaluation obligation):
  does Phase 6 implement a reactively-driven window title, or evaluate
  and defer it?

## Static title host path

R1's narrow requirement: the static `title:` literal — already sitting
on `component.root.props` next to the `window::create` call — must
reach the native window title bar.

### Options

- **HS-1 — loader reads the root's `title` prop and passes it to
  `window::create`**
  - In `wasamo_load_ui`, before `window::create`, read the `title`
    literal from `component.root.props` (falling back to
    `DEFAULT_WINDOW_TITLE` when absent or empty) and pass it as the
    title argument. No ABI signature change, no new export.
  - What you gain: routes a value the runtime **already has in hand**
    with a few lines and **no host-facing surface change**
    (`window::create` already accepts a title); `abi_spec.md` stays
    no-touch (FD-H).
  - What you give up: nothing for the static case — only that it does
    not pre-build the broader window-config seam (which is M4 work, not
    R1).

- **HS-2 — extend the `wasamo_load_ui` ABI with a `WindowConfig`**
  - A title / size struct the host fills and passes into load.
  - What you gain: the right shape **if and when** size / backdrop /
    theme also need host control (the M4 Q2 work).
  - What you give up: expands the ABI surface (`abi_spec.md` +
    `wasamo.h` + a new contract) to wire a single string the runtime
    already holds — over-building a one-string fix ahead of its
    milestone.

- **HS-3 — new `wasamo_window_set_title` ABI the host calls after load**
  - A standalone exported setter.
  - What you gain: this is the natural effector a **dynamic** title
    would later reuse (see the Dynamic title sub-issue).
  - What you give up: ships an ABI export with **no Phase 6 consumer**
    for the static case; unnecessary surface now.

### Comparison

HS-2 and HS-3 both **expand the ABI surface** (`abi_spec.md` +
`wasamo.h` + a new DD-level contract) to wire a value the runtime
**already has in hand** at the `window::create` call site. R1's
requirement is narrow — a static declared title reaching the window —
and the title is a plain `IrLiteral::Str` on `component.root.props`
sitting right next to the `window::create` call. HS-1 routes it with a
few lines and **no host-facing surface change**: `window::create`
already accepts a title; only the constant argument is replaced. HS-2
(WindowConfig) is the right shape **if and when** size / backdrop /
theme also need host control, but that is the M4 backdrop/theme work
(Q2), not R1 — pulling it in now would over-build a one-string fix.
HS-3 (set_title) is the seam a **dynamic** title needs (see below) but
is unnecessary for the static case and would ship an ABI export with
no Phase 6 consumer.

So the static-vs-ABI judgment is clean: **R1's static requirement is
satisfiable entirely inside the existing internal path (HS-1), and the
ADR therefore makes `abi_spec.md` a no-touch** (preamble §Upstream
revisions / FD-H).

### Recommendation

**HS-1.** R1's static requirement is satisfiable entirely inside the
existing internal path, so the ADR makes `abi_spec.md` a **no-touch**
(preamble §Upstream revisions / FD-H). Concretely:

- In `wasamo_load_ui`, after `build_widget_tree` and before
  `window::create`, read the `title` literal from
  `component.root.props` and resolve it by kind:
  - **present, non-empty `IrLiteral::Str`** → pass as the
    `window::create` title;
  - **absent or empty string** → fall back to `DEFAULT_WINDOW_TITLE`
    ("Wasamo") — a benign default, not an error;
  - **present but a non-string `IrLiteral`** (e.g. `title = 123` in a
    hand-authored IR that `wasamoc check` would have caught but a
    direct IR loader can still receive) → **`WASAMO_ERR_IR_MALFORMED`**,
    consistent with the loader defense-in-depth posture (DD-M3-P6-004) —
    **not** a silent fallback. The fallback covers *missing* title;
    a *wrong-typed* title is malformed IR.
- **No ABI signature change, no new export, no `PropertyValue` tag.**
  The standalone `wasamo_window_create` ABI (hand-built host trees) is
  unaffected — it already takes a title.
- `backdrop` / `theme` remain unwired (Q2, M4); Phase 6 wires
  **`title` only**, the R1 scope.
- Verification: a `.ui` declaring `title: "Gallery"` produces a native
  window whose title bar reads `"Gallery"` — asserted in the
  Windows-runtime integration test (verification closure item 4) and
  visible in the assistant/owner smoke frames (the title bar is in
  every captured frame). A loader-level test also asserts a **non-string
  IR `title` is rejected with `WASAMO_ERR_IR_MALFORMED`** (absent/empty
  falls back), pinning the split above.

## Dynamic (`String`-binding) title

FD-D asks Phase 6 to evaluate — but not necessarily implement — a
title driven reactively by a `String` state, since Phase 6 is where
binding first reaches from property into tree structure.

### Options

- **HD-1 — evaluate, defer implementation**
  - Compare static-only vs static+dynamic; ship static-only this phase;
    record the dynamic title as deferred with its reason and forward
    seam.
  - What you gain: keeps Phase 6 scoped to R1 and the lightbox (which
    need only a static title); the question is recorded, not silently
    closed (FD-D).
  - What you give up: authors cannot bind the window title reactively
    until the Window-prop seam lands (M4) — acceptable, no Phase 6
    driver exists.

- **HD-2 — implement dynamic title now**
  - Add a window-property binding target so `title: some_string_state`
    re-titles the window reactively.
  - What you gain: full reactive title this phase.
  - What you give up: requires a **new reactive target class**
    (`BindingTarget::WindowTitle`, since the window is not a
    `WidgetNode`) **and** a new ABI/host effector
    (`SetWindowText`-equivalent / HS-3) — half of the M4 backdrop/theme
    Window-prop seam (Q2) built in isolation, ahead of its milestone,
    for a feature the lightbox does not need.

### Comparison

HD-2 requires a **window-property binding seam** — the window is not a
`WidgetNode`, so a `String` binding to the title needs either a new
`BindingTarget::WindowTitle` variant whose writer calls a
host/Win32 `SetWindowText`-equivalent, or the `wasamo_window_set_title`
ABI (HS-3) as the writer's effector. That is a genuinely new reactive
target class and a new ABI/host effector — and it **overlaps the M4
backdrop / theme wiring** (Q2 lists `title` alongside `backdrop` /
`theme` as the Window-derived props to wire together when the
`WindowConfig` / window-prop work is done). Implementing dynamic title
in isolation now would build half of that seam ahead of its milestone,
for a feature the lightbox does **not** need (the gallery title is
static). The evaluation conclusion is therefore **defer** — but the
question is recorded with its seam identified, not closed silently
(FD-D).

### Recommendation

**HD-1 (evaluated, deferred).**

- **Disposition: deferred from Phase 6**, not closed. Reason: it needs
  a window-property binding seam (a `BindingTarget::WindowTitle`
  writer backed by a `SetWindowText`-equivalent host effector / a
  `wasamo_window_set_title` ABI) that is a new reactive target class
  and ABI surface overlapping the M4 backdrop/theme Window-prop wiring
  (Q2). The lightbox / gallery title is static, so Phase 6 has no
  driver.
- **Forward seam recorded:** when the Window-prop binding work lands
  (with or after M4 backdrop/theme), a `String`-binding title reuses
  the same per-type writer-seam pattern (DD-M3-P1-007) with a window
  effector, and `wasamo_window_set_title` (HS-3) is the natural ABI
  for it. No Phase 6 decision forecloses this.

## Forward-compat exposure

- **Dynamic title** — lands via a `BindingTarget::WindowTitle` writer
  + window effector when Window-prop binding is built (M4 Q2). The
  static HS-1 path is unaffected (it sets the initial title; a dynamic
  binding would later overwrite it reactively).
- **`WindowConfig` ABI (HS-2)** — if host-controlled size / backdrop /
  theme become needed, a `WindowConfig` extension to `wasamo_load_ui`
  is the shape; the static-title read (HS-1) folds into it naturally
  (the loader would still resolve declared props, with host overrides
  layered on).
- **`backdrop` / `theme` wiring** — M4 (Q2); same internal-vs-ABI
  judgment will be re-made then. Phase 6 records only that they remain
  unwired.

## Technical risk re-evaluation

- **HS-1 is a contained internal change** (read one prop, replace one
  argument) with no ABI/host surface, so the blast radius is the
  `wasamo_load_ui` body plus one integration test — far smaller than
  the ABI-extension options.
- **Failure modes are split, not blanket-tolerant**: an absent/empty
  title falls back to the existing default (no new failure mode), while
  a **non-string** title is surfaced as `WASAMO_ERR_IR_MALFORMED` via
  the existing loader defense-in-depth path (not a new bespoke error).
  `.ui` authors never reach the malformed path — `wasamoc check` rejects
  a non-string `title` earlier; the loader check guards only the
  direct-IR-loader entry (hand-authored IR).
- **R1 is technically independent of the conditional/ZStack work**
  (constraints §1: co-located by timing via FD-E, not by dependency),
  so its risk does not couple to the grammar/runtime DDs; it can land
  as its own task.
- **Dynamic-title deferral risk** is the usual "deferred question
  reopened later" — mitigated by recording the seam (writer pattern +
  ABI) so the M4 work inherits a named landing point rather than a
  rediscovery.
