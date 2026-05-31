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

## Options

### Static title host path

- **HS-1 — loader reads the root's `title` prop and passes it to
  `window::create` (recommended).** In `wasamo_load_ui`, before
  `window::create`, read the `title` literal from
  `component.root.props` (falling back to `DEFAULT_WINDOW_TITLE` when
  absent or empty) and pass it as the title argument. **No ABI
  signature change, no new export.**
- **HS-2 — extend the `wasamo_load_ui` ABI with a `WindowConfig`**
  (title / size struct) the host fills.
- **HS-3 — new `wasamo_window_set_title` ABI** the host calls after
  load.

### Dynamic (`String`-binding) title

- **HD-1 — evaluate, defer implementation (recommended).** Compare
  static-only vs static+dynamic; ship static-only this phase; record
  the dynamic title as deferred with its reason and its forward seam.
- **HD-2 — implement dynamic title now.** Add a window-property
  binding target so a `title: some_string_state` re-titles the window
  reactively.

## Comparison

### Static path

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

### Dynamic path

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

## Recommendation

**HS-1 (static) + HD-1 (dynamic evaluated, deferred).**

### Static title (required — R1 completion condition)

- In `wasamo_load_ui`, after `build_widget_tree` and before
  `window::create`, read the `title` literal from
  `component.root.props`:
  - if present and a non-empty `IrLiteral::Str`, pass it as the
    `window::create` title;
  - otherwise fall back to `DEFAULT_WINDOW_TITLE` ("Wasamo").
- **No ABI signature change, no new export, no `PropertyValue` tag.**
  `window::create` already accepts the title; only the argument
  changes. `abi_spec.md` is **no-touch** (preamble §Upstream
  revisions).
- The standalone `wasamo_window_create` ABI (used by hand-built host
  trees) is unaffected — it already takes a title.
- `backdrop` / `theme` remain unwired (Q2, M4); Phase 6 wires
  **`title` only**, the R1 scope.
- Verification: a `.ui` declaring `title: "Gallery"` produces a native
  window whose title bar reads `"Gallery"` — asserted in the
  Windows-runtime integration test (verification closure item 4) and
  visible in the assistant/owner smoke frames (the title bar is in
  every captured frame).

### Dynamic title (evaluated, deferred — FD-D)

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
- **No new failure mode**: an absent/empty title falls back to the
  existing default, so malformed input degrades to today's behaviour
  rather than erroring.
- **R1 is technically independent of the conditional/ZStack work**
  (constraints §1: co-located by timing via FD-E, not by dependency),
  so its risk does not couple to the grammar/runtime DDs; it can land
  as its own task.
- **Dynamic-title deferral risk** is the usual "deferred question
  reopened later" — mitigated by recording the seam (writer pattern +
  ABI) so the M4 work inherits a named landing point rather than a
  rediscovery.
