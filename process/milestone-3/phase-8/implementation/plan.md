## Task list

Phase 8 closes M3 with three workstreams: the `ToggleButton` / `checked`
surface end-to-end (T3–T4), the Photo Gallery integration in three hosts
(T2, T5–T7), and the public-draft promotion + milestone-close inputs
(T8–T11) — preceded by a pre-implementation spike (T1). The five FD-8-G
owner checkpoints are staged across T2 / T5 / T7 / T9 / T10 (not piled
at phase end), and the final-task / phase-end / milestone-end ownership
split ([preamble.md §Obligations item 4](./preamble.md)) is represented
in T11 and the two close batches from the start.

Default to **one commit per task-list item** per
[AGENTS.md §Commit rules](../../../../AGENTS.md#commit-rules). Known
exceptions this phase:

- **T5** — the gallery restructure (wireframe assembly + verification-
  screen sweep + capture-coordinate re-derivation) is one review
  concern and intermediate states would leave the build or the evidence
  scripts broken, so it may bundle into one buildable commit (or split
  only at seams T1 identifies as independently green).
- **T3 / T4** — T1 found the widget-kind carrier is string-based rather
  than compile-error-forcing, so the default split holds: T3 owns the
  compiler/IR surface and T4 owns the runtime node/visual surface.

If implementation reveals an item should split or reorder, revise this
list so it stays an accurate record rather than a frozen prediction.
**Sub-task lists are planning-time hypotheses** — T1 may re-cut them
against the source, and any task may revise its own sub-list as work
surfaces.

Each task runs the implementation gates at **start** (record the trap
selection + review lane in [log.md](./log.md) before choosing an
approach) and **close** (the auditable artifacts), per
[implementation-gates.md](../../../procedures/implementation-gates.md).

---

### T0 — Moment 1 closure + implementation docs open

Opens execution after the ADR acceptance. Moment 1 is largely landed;
T0 completes the remaining re-sync set and lands the implementation
docs. Implementation (T1) begins only after T0 closes.

- [x] ADR set `Status: Accepted` (DD-001 2026-07-01; DD-002 2026-07-02;
      preamble synced 2026-07-02).
- [x] `docs/dsl_spec.md` §4.17 (`ToggleButton` / `checked` normative
      draft) + §4.18 (public-draft future notes: PM-2 / explicit sizing
      / defaults / spelling / bindability / DD-001 axes) +
      `docs/architecture.md` §6.7.7 — Moment 1 design draft landed
      (commits `3a1af26`, `28a991a`).
- [x] `process/milestone-3/plan.md` Phase 8 row populated + FD-8-F
      no-new-AC Revision-log entry (2026-07-02).
- [x] Problem B Vision DR raised and Accepted
      ([author-controllable-sizing-surface.md](../../../cross-milestone/decisions/author-controllable-sizing-surface.md),
      2026-07-02) — Phase 8 carries only the documentation posture.
- [x] **Remaining DD-001 item-6 re-sync** (separate review concern from
      this plan): `process/_roadmap.md` A1 / A9 / A10 / A12 +
      `process/milestone-3/plan.md` A1 / A10 (AC mirror) wording
      re-synced from the "Button `selected` state" shorthand to the
      accepted `ToggleButton` / `checked` lexeme; the framing packet-C
      `selected: bool` form recorded as DD-001 B1-rejected in a dated
      addendum (not overwritten — the framing is frozen). `docs/dsl_spec.md`
      §4.17 / `docs/architecture.md` §6.7.7 already carry the accepted
      lexeme from Moment 1. Executes the plan Revision-log 2026-07-02
      tier-1-factual entry (no new AC).
- [x] This `preamble.md` + `plan.md` owner-reviewed (owner accepted
      2026-07-03; the plan is explicitly a working hypothesis, revisable
      mid-implementation) and landed with `status: active`; skeleton
      [log.md](./log.md) / [handoff.md](./handoff.md) opened.

**Start gate:** none (doc-only). **End gate:** Moment 1 commit set
complete; T1 may open.

---

### T1 — Pre-implementation spike: ToggleButton landing recon + gallery/hosts recon + sequencing

Per the spike discipline
([implementation-gates.md](../../../procedures/implementation-gates.md)):
**no production code lands**; the compiler-verification edit is
throwaway and reverted before T1 closes. Landing artifacts are recorded
decisions in [log.md](./log.md) plus any revision of this plan. Exit
criterion: **every open point is assigned to a downstream task and its
scope is seen**, not "no surprises expected".

Critical responsibility cut (T1 re-check, 2026-07-03): T1 owns
**source-grounded uncertainty reduction and task-boundary repair**, not
early implementation. It must distinguish (a) facts that force a plan
revision before T2/T3, (b) downstream implementation choices that are
now scoped enough to leave with their owning task, and (c) owner-facing
UI judgments that T2/T5/T7 still own. T1 must not silently decide the
A1 placeholder agreement, change the author-facing `ToggleButton`
surface, or leave a production edit behind. A T1 "done" report is valid
only if the log makes those three buckets auditable.

- [x] **Read every landing file end-to-end** (not grep-sample):
      `wasamoc/src/{lexer,parser,ast,check,lower,emit}.rs` (widget-name
      admission; the attribute admission tables — e.g.
      `("Button", "enabled")` rows in `check.rs`; style/text lowering),
      `wasamo-ir/src/lib.rs` (how `IrNode` carries the widget kind —
      string vs enum decides whether T3 is additive or
      compile-error-forcing), `wasamo-runtime/src/ir_loader.rs` (kind
      dispatch, bool-binding registration on `enabled` as the model for
      `checked`), `wasamo-runtime/src/widget.rs` (Button visual
      construction, `enabled` propagation path, where the V-a checked
      background fill plugs in), `wasamo-runtime/src/layout.rs` (Button
      leaf measure reuse). Record the per-file touch points.
- [x] **Compiler-verify the widget-kind addition**: introduce a
      throwaway `ToggleButton` kind at the IR carrier, `cargo build` the
      workspace to enumerate every dispatch/match site by compiler error
      (or, if the kind is string-carried, enumerate the dispatch tables
      by targeted search *and* a deliberate wrong-kind probe), record
      the site list, then **revert**. This is the trap-#1 pre-audit; the
      authoritative audit table is T3/T4's close artifact.
- [x] **Fix and record the internal shape recommendations** the DD left
      to implementation: whether `ToggleButton` shares a Button-family
      code path internally (ButtonBase-style sharing is allowed; the
      *author-facing* taxonomy is DD-fixed), how `checked` reuses the
      existing single-boolean binding path, and how the V-a fill
      composes with `style` / `enabled` visuals (including the R-2
      ambiguity check plan against the final effective Gallery background;
      Mica/backdrop is a possible ambiguity factor, not a separate proof
      target).
- [x] **Gallery + hosts recon**: map the current
      `examples/gallery/gallery.ui` sections to the wireframe target
      (what folds where; what is swept); list retained capture scripts
      whose coordinates must re-derive (trap #2); read
      `examples/counter-c/{CMakeLists.txt,embed_uic.cmake,main.c}` and
      `examples/counter-zig/{build.zig,main.zig}` end-to-end and record
      the gallery-port deltas (component name, `.uic` path, window
      loop); confirm the CI step shapes to mirror
      ([ci.yml](../../../../.github/workflows/ci.yml)).
- [x] **Fix and record the bisectable sequencing** (default
      T2 → T3 → T4 → T5 → T6 → T7; T2 is independent of T3/T4 and may
      run in parallel or reorder) and the inter-task seams (T5 needs
      T3+T4 for the tab band; T6 needs T5's final `gallery.ui`; T7
      needs T5+T6). Revise this plan if the default order changes.
- [x] **Record the T1 responsibility buckets**: plan revisions required
      before work continues; downstream choices that are scoped but left
      to T2–T8; owner-facing judgments explicitly not decided by T1.
- [x] Sharpen the preamble §Technical risks table against the source;
      record the **T3 gate selection** (review lane + applicable traps
      with reasons for non-applicable ones) before T3 opens.

**Start gate:** T0 closed; T1's own gate selection recorded in
[log.md](./log.md). **End gate (spike-specific):** every open point
assigned + scoped; site list, shape recommendations, sequencing, and
recon deltas recorded in [log.md](./log.md); the throwaway edit
reverted.

---

### T2 — Wireframe skeleton + layout-skeleton technical smoke + FD-8-G(1) owner agreement

Discharges ADR verification-closure item (4) and the FD-8-G(1)
checkpoint. Independent of T3/T4 (the tab band is plain-Button
placeholder at this stage — swapped to `ToggleButton` in T5).

Critical responsibility cut (T2 re-check, 2026-07-03): T2 owns the
**first non-throwaway Photo Gallery skeleton** and the **wireframe-
fidelity / M3-placeholder agreement basis**. It does not own the final
A1 gallery assembly, verification-screen sweep, `ToggleButton` tab
band, host parity, or authoritative GUI evidence package. Its technical
smoke is deliberately early: it should expose layout collapse / clip /
aspect / scroll risks before owner UI review, then classify findings as
fixable by the existing surface, owner-agreement placeholder, or Problem
B residual. T2 must not silently absorb layout-engine changes or final UI
judgments that belong to T5/T7/owner checkpoints.

- [x] Restructure `examples/gallery/gallery.ui` to the **wireframe
      skeleton**: Grid overall frame (current T2 shape uses rows
      `56 / 1* / 28`, two star columns, direct `Cell` alignment in the
      header, and content/status `column-span: 2`),
      tab-band placeholder `HStack`, thumbnail area `ScrollView {
      WrapPanel { for … } }`, status strip `Text`, lightbox `ZStack` +
      `if` retained, Box `aspect` placeholders, `slot.*` placement —
      the first assembly step, not a throwaway (T5 completes it).
      Workspace stays green (build compiles `gallery.ui`).
- [x] **Technical smoke (assistant, before any owner UI review):**
      build + launch `gallery-rust`, DPI-aware capture + analysis;
      check 0-sizing / clip breakage / `aspect`-abort / scroll breakage
      (Problem B's Fill→0 collapse surfaced early — framing R7).
      Findings triage recorded in [log.md](./log.md): fix with existing
      surface, or route to the A1-table placeholder / G(1) agreement /
      Problem B note. **No layout-engine change.**
- [x] **FD-8-G(1) owner agreement packet** (informed by the smoke):
      the wireframe-fidelity / M3-placeholder table — real images →
      Box + Text; thumbnail click-to-open → explicit "Open lightbox"
      Button (hit-testing = M4); lightbox prev/next Button presence and
      what they do in M3 (spec.md Interaction) vs inert placeholder;
      scrollbar/wheel → scroll Buttons; thumbnail highlight → omitted
      (TH-a, DD-fixed); tab exclusion → α live, V-a background-only;
      status text static (no collection-length read exists in M3);
      which mutation Buttons (`Add` / `Remove` / `Clear` / `Reset`)
      survive as A1 minimal operation UI vs are swept; scrim/backdrop
      approximations. Owner agreement recorded in [log.md](./log.md).
- [x] **Update the A1 feature-mapping table** (the framing's table is
      the initial hypothesis): the agreed table lands in this plan (or
      log.md) as the audit basis for all later UI checks; deviations
      from the framing table noted in [log.md](./log.md).

**Start gate:** T1 recon (gallery mapping) recorded; T2 gate selection
recorded. **End gate:** skeleton renders without collapse (or triaged),
G(1) agreement + updated A1 table recorded; workspace green.

---

### T3 — `ToggleButton` / `checked`: compiler + IR surface

The cross-cutting surface task, compiler half (gates trap #1 + #4;
risk R-1). Discharges ADR evidence items (1) and (2)-emit, and
propagation-audit point (i). **Full independent review**, with the
branch/test-focused check folded in for the reject matrix.

Critical responsibility cut (T3 re-check, 2026-07-04): T3 owns the
**authoring/compiler/IR boundary** for `ToggleButton.checked`, not the
runtime node, visual state, gallery tab-band, or diagnostic-policy
reform. Because widget kind is string-carried and unknown widgets are
warning-only today, T3 must prove `ToggleButton` is known/admitted with a
no-unknown-warning positive fixture; it must not silently convert the
general unknown-widget policy to a hard error. T2's G(1) table and GUI
evidence carry forward to T5/T7 and do not change T3's scope except that
the alpha tab-band compile fixture should match the later Gallery shape.

- [x] `wasamoc` parse/check: `ToggleButton` admitted as a widget kind;
      attribute admission per [dsl_spec §4.17](../../../../docs/dsl_spec.md)
      — carries Button's `text` / `style` / `enabled` + `clicked`
      handler; `checked: <bool>` (literal or bool-state binding)
      admitted on `ToggleButton` **only**; default `false` when absent.
      Positive fixture must assert no unknown-widget warning for
      `ToggleButton`.
- [x] SI-3 reject matrix (named diagnostics, firing tests both
      directions, trap #4): `checked` on `Button` / `Text` / other
      non-supporting widgets → reject; unknown attributes on
      `ToggleButton` → existing unknown-attr path; non-bool `checked`
      RHS → type reject.
- [x] Lower + emit: `ToggleButton` kind + `checked` prop/binding
      through the existing single-boolean binding model (no new
      binding-target class); textual-IR emit shape per the spec; public
      wasamoc pipeline fixture proves emitted IR carries the kind,
      `checked` literal, and `checked` binding forms. Loader preservation
      remains T4's responsibility.
- [x] Positive fixtures: a `ToggleButton` with `text` / `style` /
      `enabled` / `clicked` / `checked` compiles and lowers; the α
      tab-band shape (3 ToggleButtons + 3 bool states + block-assign
      exclusion handlers) compiles (the stage-1 spike shape, now on the
      real surface) without introducing equality, group widgets,
      self-toggle, or two-way binding.
- [x] **Close artifact (trap #1):** the call-site audit table over
      widget-kind dispatch + attribute-admission sites (check / lower /
      emit / IR), each site classified, including sites deliberately left
      generic / unchanged and the preserved warning-only unknown-widget
      policy.

**Start gate:** T1 site list + T3 gate selection recorded. **End
gate:** workspace + tests green; audit table + firing tests recorded;
full independent review before merge.

---

### T4 — `ToggleButton` runtime node + V-a checked visual + Windows fixtures

The runtime half (gates traps #1 / #4 / #7-adjacent; risks R-1 / R-2).
Discharges ADR evidence item (2)-loader and propagation-audit point
(ii). **Full independent review** (runtime structural: new widget
node).

Critical responsibility cut (T4 re-check, 2026-07-04): T4 owns the
**runtime defensive-reader and widget-node boundary** for
`ToggleButton.checked`. It mirrors T3's authoring catalog in the runtime
loader (`text` / `style` / `enabled` / `checked`, with `checked` valid
only on `ToggleButton`), materializes absent `checked` as the runtime
default `false`, and proves bool-state writes drive the live visual. T4
does **not** own Gallery integration, final tab-band composition, C/Zig
host parity, screenshot-coordinate derivation, or the authoritative T7
GUI evidence package. Its R-2 work is the runtime cue and an
unambiguous-colour fixture; the final effective Gallery-background proof
remains a T5/T7 carry-forward if later UI work changes that background.

- [x] `wasamo-runtime` loader: `ToggleButton` kind dispatch; `checked`
      literal + binding registration reusing the `enabled` bool-binding
      path; loader **re-rejects** malformed IR (`checked` on a
      non-supporting kind, unknown `ToggleButton` attribute / binding,
      non-string `text`, non-keyword `style`, non-bool `enabled`, and
      non-bool `checked` values) with named diagnostics + firing tests
      (dual-gate pattern); absent `checked` stays absent in textual IR
      and defaults to runtime `false`.
- [x] Widget construction: `ToggleButton` reuses Button's visual
      (text / style / enabled / click) and leaf measure/arrange — a new
      node, not a new layout primitive; **V-a checked visual**:
      background colour change on `checked`, composing with `style` and
      `enabled` states; pure tests pin Default/Accent checked
      hover/press arms and disabled-over-checked priority, while T5/T7
      still verify the cue is unambiguous against the final effective
      Gallery background (R-2 — Mica/backdrop is a possible ambiguity
      factor, not a separate proof target; if ambiguous in practice, that
      is an SI-1 implementation-checkpoint revision recorded in log.md,
      not a new option choice).
- [x] Reactive path: a bool-state change through the existing binding
      drives the visual (propagation-audit point (ii)); `clicked`
      handler block-assignment writes the state (W1 controlled — the
      widget does not self-toggle).
- [x] Windows-runtime fixtures (CI-gated, fail-not-skip): checked
      visual state reflects the bound value at load; a state flip
      reaches the visual; the α exclusion shape drains to exactly-one-
      checked; `enabled: false` keeps the Phase-1 disabled contract
      (no `clicked` fire). Regression: existing Button fixtures
      unchanged; runtime default `false` is pinned when `checked` is
      absent.
- [x] **Close artifact:** trap-#1 audit rows for the runtime sites;
      trap-#4 loader-reject firing test recorded.

**Start gate:** T3 merged; T4 gate selection recorded. **End gate:**
workspace + integration + regression green; full independent review
before merge.

---

### T5 — Gallery integration (A1) on the Rust host + verification-screen sweep + FD-8-G(2)

Completes the T2 skeleton into the agreed Photo Gallery target app and
sweeps the per-phase verification surfaces
([constraints §6](../requirements/constraints.md)). Gates trap #2
(capture coordinates) + the R-5 regression risk.

Critical responsibility cut (T5 re-check, 2026-07-05): T5 owns the
**landed Rust-host Gallery surface** that T6 ports and T7 captures. It
replaces the T2 tab placeholder with the real `ToggleButton` alpha
exclusion band, keeps or explicitly dispositions each G(1) / A1 table row,
records the verification-screen sweep / retired-script status, and creates
or re-derives only the capture coordinates needed for the T5 first-render
direction check. T5 does **not** own the C / Zig hosts, the authoritative
assistant GUI evidence package, the strict `Box.aspect` positive-control
closure, the public-draft audit, or owner final smoke. It must not absorb
new layout-engine work, generic diagnostic-policy changes, real image /
thumbnail hit-testing work, or collection-mutation controls unless the A1
audit proves the existing Phase 7 coverage citation insufficient.

- [x] Tab band: 3 `ToggleButton`s (All / Albums / Favorites) with
      `checked:` bound to per-tab bool states and α block-assignment
      exclusion handlers; V-a visual distinguishes the selected tab.
      (**β fallback trigger:** if the live exclusion proves unworkable
      here, substitute β and record the SI-2 static-approximation
      accounting in this plan + the A1 table — preamble obligation 2.)
- [x] Assemble the remaining agreed surface: `for`-generated thumbnail
      WrapPanel inside ScrollView (iteration + wrap + viewport);
      lightbox ZStack + `if` with close (and agreed prev/next form)
      Buttons; status strip; Box `aspect` placeholders; `slot.*` /
      `Cell` placement per the frozen 7b surface; the agreed minimal
      operation UI (scroll / any retained mutation Buttons).
- [x] **Complete the per-phase verification-surface sweep**: T2 already
      removed the placement-demo sub-screen/state/button, the Grid
      footer-clip demo, the standalone static `Photo 1–10` WrapPanel,
      and mutation Buttons from the skeleton. T5 verifies no
      verification-only surface reappears while integrating
      `ToggleButton`, records the `phase-7b/.../evidence/capture-
      placement-demo.ps1` retirement note, and records the A1-table
      disposition for any operation UI it keeps/removes. No verification
      menu / dashboard remains (FD-8-E).
- [x] Regression gate: workspace + all fixtures green; every M3 surface
      in the A1 table is exercised by the integrated app (audit against
      the T2-updated table, row by row, recorded in log.md).
- [x] Re-derive layout-coupled coordinates for the T5 first-render /
      alpha-precheck capture script (trap #2); do not treat T2 coordinates
      as retained ground truth. Park superseded scripts with the evidence
      they supported, including the Phase 7b placement-demo retirement note.
- [x] **FD-8-G(2): owner first-render UI check** on the Rust host
      (early direction check; assistant pre-verifies α exclusion works
      live before presenting). Findings that exceed the plan go back to
      the A1 table / framing revisions (framing packet-G note), not
      silently absorbed. If owner feedback requires additive fixes, they
      land on the T5 branch and this checkpoint reruns before T5 close.

**Start gate:** T2 (agreed table) + T4 merged; T5 gate selection
recorded. **End gate:** integrated gallery green + A1-table audit
recorded + G(2) owner check passed; capture coordinates re-derived.

---

### T6 — `gallery-c` + `gallery-zig` hosts + CI steps (A1 three-host)

Creates the two missing hosts from the counter templates and closes the
A1 "all three hosts" requirement. Build ordering per
[AGENTS.md §Build ordering](../../../../AGENTS.md): `wasamoc` builds
before the C / Zig hosts.

Critical responsibility cut (T6 re-check, 2026-07-05): T6 owns the
**host-template ports and CI build coverage** for the already-landed T5
Gallery surface. It must port the final `examples/gallery/gallery.ui`
without changing the Gallery semantics that T7 will capture, keep the C /
Zig hosts declarative (`WASAMO_LOAD_MEMORY` / embedded `.uic`, no host-side
widget mutation), and mirror the proven counter build ordering. T6 may run
a default-render cross-host parity precheck to prove the new hosts actually
load the same integrated Gallery, but it does **not** own the authoritative
two-frame selected/exclusion, lightbox, wrap/overflow, or aspect evidence
package; those remain T7's full-review GUI evidence. If screenshot capture
is used for T6 parity, the twice-observed visible-desktop / outside-sandbox
constraint is recorded up front and any coordinates are treated as
task-local precheck coordinates, not T7 ground truth.

- [x] `examples/gallery-c/`: port `counter-c` (CMakeLists +
      `embed_uic.cmake` + `main.c`) to the Gallery component; builds
      and runs against `target/release/wasamo.dll`.
- [x] `examples/gallery-zig/`: port `counter-zig` (`build.zig` +
      `main.zig`); builds and runs.
- [x] CI: add `gallery-c` / `gallery-zig` build steps mirroring the
      counter steps in [ci.yml](../../../../.github/workflows/ci.yml)
      (per-example enumeration — not a new build system; `gallery-rust`
      is already workspace-covered). Optionally add
      `wasamoc check examples/gallery/gallery.ui` beside the counter
      check step.
- [x] **Cross-host parity (propagation-audit point (iii)):** launch all
      three hosts, DPI-aware default-view capture, assistant analysis —
      C / Rust / Zig render the same integrated gallery at first load
      (representative-host UI review stays Rust; C / Zig are identical-
      render / no-regression checks). Parity frames land under
      [evidence/](./evidence/) as `t6-parity-<host>.png`. T7 still
      re-captures the authoritative positive-control state set after T6.

**Start gate:** T5 merged; T6 gate selection recorded. **End gate:**
three hosts build + run; CI steps added and locally rehearsed in the
phase build order; remote CI run id recorded if the task branch is
available to GitHub Actions before merge (otherwise the run id remains
phase-branch / phase-end owned); parity analysis recorded.

---

### T7 — Assistant GUI evidence package + FD-8-G(3)

The authoritative assistant-visible evidence (ADR item (5); gates trap
#7; GUI-render class → **full independent review**). Assistant evidence
= launch + DPI-aware screenshot (`CopyFromScreen`) + analysis;
`Start-Process` survival is a supporting signal only. Owner smoke is
T10's separate gate.

Critical responsibility cut (T7 re-check, 2026-07-05): T7 owns the
**authoritative assistant-visible evidence package** for the final
post-T6 Gallery surface. It must re-capture the selected/exclusion,
lightbox, wrap/overflow, and aspect/citation evidence after the C/Zig
host additions, treating T5/T6 frames as prechecks only. Because T2 and
T5 both exposed sandboxed `CopyFromScreen` / coordinate fragility, T7's
capture is planned as a visible-desktop / outside-sandbox activity with
state-confirming frames after state-changing clicks. T7 does **not** own
new Gallery UI semantics, C/Zig host changes, T8's no-silently-deferred
audit, or T10's human-visible smoke. G(3) remains an explicit owner
confirmation over the captured positive-control frames before T7 can be
reported done.

- [x] Capture the agreed state set on the Rust host (labelled frames
      under [evidence/](./evidence/), analysis in
      `evidence/README.md`): **default view**; **lightbox open +
      closed** (subtree present/absent pair); **selected two-frame
      positive control** (click a different tab → its background
      changes **and** the previously-selected tab clears — exclusion in
      the same two frames); **wrap/overflow** (narrow-width reflow
      and/or scroll-offset frames per the agreed state set); **aspect**
      evidence that distinguishes a live `Box.aspect` constraint from a
      no-op look-alike, or records the T8 audit citation that proves the
      aspect surface elsewhere.
- [x] Verify each positive control distinguishes the intended behaviour
      from a static look-alike (AGENTS.md §Testing rules); note known
      M4 residuals (DPI blur, dynamic title) as residuals, not
      failures.
- [x] **FD-8-G(3): owner confirms the two-frame positive controls**
      (selected/exclusion + lightbox) over the captured frames.

**Start gate:** T6 merged (final surface + re-derived coordinates); T7
gate selection recorded. **End gate:** evidence set + analysis landed;
G(3) owner confirmation recorded; full independent review.

---

### T8 — A12 editorial pass + external-reader smoke + milestone audits

The spec-closure work (ADR item (6), minus the T11 marker flip).
Reads the **landed** implementation (pin-to-landed-source learning),
not the design draft.

Critical responsibility cut (T8 re-check, 2026-07-05): T8 owns the
**public-draft readiness audit before promotion**, not the promotion
mechanics themselves. It must read the landed T7 surface and source-backed
tests, then make `docs/dsl_spec.md` reproducible for an external reader for
all M3 surfaces. The external-reader smoke is the parent gate: the A11
ADR-to-spec trace, the no-silently-deferred-surface audit, the Phase 7
mutation-control citation, the T7 aspect visual + Phase 2 test citation, and
the architectural-family confirm entry are inputs to that smoke. T8 may make
editorial fixes in `docs/dsl_spec.md` and revise
`docs/notes/architectural-family.md`, but it must not flip
`status: public-draft`, add the public-draft change-history entry, change
`docs/architecture.md` status markers, draft M3 handoff, run owner G(4), or
reopen shipped UI semantics.

- [x] Editorial pass over `docs/dsl_spec.md` **whole-document** at the
      external-reader bar: §4.17 re-verified against the shipped
      implementation; the DD-002 accepted dispositions verified present
      and honestly worded — B-1b (PM-2 both forms + provisional wrapper
      rule), B-2c (sizing future note, no shape reservation, no M4/M5
      schedule in the public draft), B-3b (container-owned default
      semantics), B-4a (spelling affirmed), B-5b (constant-per-instance
      placement, no compatibility guarantee), B-6b (five DD-001 axes as
      future notes); the α M3-era exclusion note at the DD-002-coupling
      strength (items 1–3); no DD/option labels in spec prose.
- [x] **External-reader smoke** (milestone-end criterion 5): a
      structured walkthrough asking whether a reader with only
      `docs/dsl_spec.md` could reproduce each M3 surface (A2–A10, A13,
      grammar surfaces) against a hypothetical C-ABI host; per-surface
      verdict recorded in [log.md](./log.md); every "not yet" fixed as
      remaining editorial work in this task. **B-3b check:** if the
      defaults still read as arbitrary, trigger the separate B-3c
      revision procedure (owner-gated), not a silent rewording.
- [x] **A11 auditability check** (milestone-end criterion 4): every M3
      phase ADR names the `docs/dsl_spec.md` sections it updated;
      gaps listed + closed (pointer fixes in the ADRs' allowed
      sections or log-recorded disposition).
- [x] **No-silently-deferred-surface audit** (milestone-end criterion
      6): the M3 target-app pre-doc's 必要 surface list checked item by
      item — shipped, or recorded as a deviation in the M3 plan
      Revision log; result recorded for T9's handoff.
- [x] `docs/notes/architectural-family.md`: the Phase 8
      confirm-within-family entry lands revise-in-place (trigger 1 —
      M3 spec capstone; not yet landed at Moment 1).

**Start gate:** T7 merged (spec re-verified against shipped surface);
T8 gate selection recorded. **End gate:** smoke verdicts all "yes" (or
disposition recorded); audits recorded; spec editorial complete short
of the T11 marker flip.

---

### T9 — M3 handoff draft + FD-8-G(4) owner review

Prepares the milestone-close inputs early enough to review before the
close batches (FD-8-G(4): the public draft and the M3 handoff have the
largest owner judgment surface).

Critical responsibility cut (T9 re-check, 2026-07-05): T9 owns the
**milestone-level handoff draft and G(4) review packet**, not public-draft
promotion, final milestone-close recording, T10 human-visible smoke, or T11
Moment 2 sync. It must reuse the T8 no-silently-deferred-surface audit and
the T1-T8 carry-forward set rather than rediscovering residuals, and it must
separate (a) public-reader future notes already folded into `docs/dsl_spec.md`,
(b) pre-1.0 / M4-M6 residuals that the M3 handoff must carry, and (c) local
Phase 8 evidence-process learnings that T11 / phase-end may choose to put in
`implementation/handoff.md`. The milestone handoff remains `status: draft`
after T9; milestone close owns the final `status: recorded` flip.

- [x] Draft `process/milestone-3/handoff.md` (milestone-level; distinct
      from this phase's `implementation/handoff.md`): PM-2 wrapper-rule
      decision (pre-1.0, re-triggers); Problem B disposition (VDR
      Accepted: M4/M5 spike schedule + M6 backstop); DD-001's five
      deferred axes with triggers; default-alignment residual per the
      T8 B-3b outcome; spelling affirmed-keep record; DPI / dynamic
      title / modal focus M4 residuals; TypedValue / structured-item
      triggers (Phase 7 carry); host-state-boundary record; anything
      the T8 audits surfaced.
- [x] **FD-8-G(4): owner reviews the public draft (post-T8
      `docs/dsl_spec.md`) + the M3 handoff draft together** before the
      phase close batches open. The T9 branch is the fix container for
      additive corrections from that review; the task may not be reported
      done or merged until explicit owner acceptance is recorded.

**Start gate:** T8 merged; T9 carry-over check and gate selection recorded.
**End gate:** G(4) owner review passed; handoff draft parked at
`status: draft` (finalized at milestone close, not here); T9 close artifacts
recorded in [log.md](./log.md).

---

### T10 — FD-8-G(5) final owner human-visible smoke

The owner-performed gate (separate from T7's assistant baseline).

Critical responsibility cut (T10 re-check, 2026-07-06): T10 owns the
**FD-8-G(5) owner-performed human-visible smoke** over the agreed state
set and its additive fix container — nothing else. The gate evidence is
the owner's explicit per-state acceptance recorded in
[log.md](./log.md); assistant screenshots are **not** the T10 evidence
form, and the T7 frames / T10 script are prep material only
(AGENTS.md §Testing rules — the owner smoke is not replaced by the
assistant baseline). T10 does not own Moment 2 docs sync (T11), the CI
run id / phase-end batch, new Gallery semantics, or reopening the
T7/T8/T9 evidence and audits. Because T8/T9 landed documentation only,
T10's prep first confirms by git history that the built surface equals
the T7-captured surface; if a fail observation forces a fix touching
compiler / runtime / `gallery.ui` / hosts, the T10 gate selection and
review lane are re-evaluated before that fix lands.

- [x] Assistant prep: confirm the runtime surface is unchanged since
      the T7 capture (T8/T9 doc-only git check); rebuild the three
      hosts in the AGENTS.md build order and rehearse the script's
      build/launch commands (launch survival is a supporting no-early-
      crash signal only); author the owner observation script at
      `evidence/t10-owner-smoke-script.md` (launch / navigation /
      per-state observation + pass-fail criteria) covering the agreed
      state set — default view, lightbox open/close, tab selection with
      exclusion (positive control: clicking tabs moves the single
      selected highlight and clears the previous one), wrap/overflow
      (narrow resize reflow + scroll movement), window close without
      crash — on the representative Rust host, plus launch +
      default-view confirmation on C and Zig. The script names the
      known M4 residuals so they are observed as residuals, not
      recorded as fail observations, and cites the A1 table / T7
      evidence set rather than restating the state-set definition.
- [x] Owner runs the smoke on a visible Windows desktop session
      ([human-visible-smoke.md](../../../../docs/notes/human-visible-smoke.md)
      environment rules) and observes per the script; owner explicitly
      accepts, or records a fail observation → fixes land additively on
      the task branch → re-run to green (Phase 4 / 7b precedent).
      (Owner acceptance "G(5) OK" recorded 2026-07-06; no fail
      observation.)
- [x] T10 step-end retrospective recorded at
      `../retrospectives/t10.md`.

**Start gate:** T9 merged (surface frozen); T10 carry-over check and
gate selection recorded in [log.md](./log.md). **End gate:** owner
acceptance recorded in log.md.

---

### T11 — Step-end local gates + Moment 2 re-sync (public-draft promotion)

The final task's step-close half per the ownership split
([preamble.md §Obligations item 4](./preamble.md)). T11 is a
document-sync and step-close task, **not** the phase-close or
milestone-close task.

Critical responsibility cut (T11 re-check, 2026-07-06): T11 owns the
**Moment 2 public-draft promotion mechanics, the step-end local gates,
and the phase-end candidate ledger** — nothing else. Its docs diff is
bound to the T9-enumerated allowed surface ([log.md](./log.md) §T11
allowed diff surface): `docs/dsl_spec.md` top Status / §4.17
phase-status line / public-draft promotion change-history entry (the
public-draft anchor), `docs/architecture.md` top Status + §6.7.7 status
sentence, `CHANGELOG.md` M3 entry, and the `docs/abi_spec.md` no-op
confirmation. Two plan hypotheses were corrected against the source:
(a) §4.18 carries no `Phase status:` marker — only §4.17 does, so
there is no §4.18 marker flip; the §4.18 state is governed by the top
Status block; (b) the "divergence corrections folded" work was already
discharged by T8 (readiness editorial pass) and T9 (G(4) remediation)
— T11 folds no body-prose correction; any newly found divergence is a
separate owner-visible review concern per the T9 carry-forward. T11
does not own the CI run id, `implementation/handoff.md` finalization,
the phase-end retrospective, the preamble status flip (phase-end
batch), or the milestone-close batch. T11 must not touch product
surface (`wasamoc/` / `wasamo-runtime/` / `wasamo-ir/` / `examples/` /
`bindings/` / CI): the G(5) acceptance is bound to the surface
unchanged since the T7 capture commit (`5b66321`), and a forced
product fix re-triggers the owner smoke before it lands.

- [ ] T11 start gate recorded in [log.md](./log.md): carry-over from
      log.md and every Phase 8 task retrospective checked; T11 /
      phase-end / milestone-end ownership split made auditable.
- [ ] `cargo fmt --all -- --check` green locally.
- [ ] Local clean rebuild green (`cargo clean` → release build → debug
      build → `cargo test --workspace`), plus the C / Zig example hosts
      in the AGENTS.md build order. CI green is phase-end-owned.
- [ ] `docs/dsl_spec.md`: the §4.17 phase-status line and the top
      Status block flip to `M3-Phase 8 closed; implementation-synced`
      (§4.18 carries no phase-status marker — its state is governed by
      the top Status block; divergence corrections were already folded
      by T8/T9, and T11 adds no body-prose change); **the
      `status: public-draft` marker lands in the top Status block**
      (C-2 — gated on the surface running, which T5–T10 established);
      the public-draft promotion change-history entry lands as the
      public-draft anchor, distinct from the revision-history table
      (linking the M3 ADRs); the T8 external-reader smoke result
      recorded there; the revision-history table gains an appended
      Moment 2 row (append-only per the T9 carry-forward).
- [ ] **CHANGELOG entry for M3** (milestone-end criterion 3): links
      each M3 phase ADR and the public-draft anchor.
- [ ] `docs/architecture.md` §6.7.7 (+ any sections the gallery /
      ToggleButton work touched) re-synced to the landed shape; Status
      header updated.
- [ ] `docs/abi_spec.md` re-confirmed untouched; any forced surface
      escalates with owner confirmation.
- [ ] `process/milestone-3/plan.md` Phase 8 row flips to
      `implementation complete; phase-end pending`.
- [ ] ADR set touched only if a retrospectives.md §phase-sync
      ADR-touch case applies.
- [ ] [log.md](./log.md) records the phase-close evidence pointers +
      implementation summary distilled from T1–T10, and the phase-end
      handoff **candidate ledger**: the five DD-001 axes; PM-2 wrapper
      rule; Problem B disposition; default-alignment (per T8 outcome);
      spelling affirmed-keep; M4 residual cluster (DPI, dynamic title,
      hit-testing, modal focus, wheel/drag); anything mid-phase
      surfaced.
- [ ] **T11 step-end retrospective recorded** at
      `../retrospectives/t11.md` (items 1–11; owned by T11).

**Start gate:** T10 merged; T11 start gate recorded. **End gate (T11
step-close):** local gates green, Moment 2 synced (including the
public-draft marker + CHANGELOG), candidate ledger recorded, T11 retro
done — preamble `status` stays `active` (phase-end owns the flip).

---

### Phase-end batch (NOT owned by T11)

Lands on the phase branch after T11 merges in, by separate commits; the
precondition for the phase → main merge gate
([retrospectives.md](../../../procedures/retrospectives.md)).

- [ ] GitHub Actions CI run id recorded (`workflow_dispatch` on the
      phase branch, including the new gallery-c / gallery-zig steps).
- [ ] `implementation/handoff.md` finalized from the candidate ledger
      (this phase's cross-phase residuals; the milestone-level items
      flow to the M3 handoff, not duplicated here).
- [ ] Phase-end retrospective recorded at
      `../retrospectives/phase-end.md` (items 12–18); no open
      `phase-sync` items survive.
- [ ] [preamble.md](./preamble.md) front-matter `status` flips
      `active` → `closing`.
- [ ] Owner approval → phase branch no-ff merge to main; push is a
      separate gate.

### Milestone-close batch (workflow §7 — after the phase → main merge)

Phase 8 is the final M3 phase; the milestone close follows as its own
gated batch ([workflow.md §7](../../../procedures/workflow.md);
[plan.md §Milestone-end criteria](../../plan.md)).

- [ ] Milestone review: every phase-end retrospective re-read; A1–A13
      discharge confirmed and recorded (M3 plan Progress rows +
      criterion mapping), with the Phase 8 evidence pointers.
- [ ] `process/milestone-3/handoff.md` finalized (from the T9
      owner-reviewed draft) — `status: recorded`.
- [ ] `process/_roadmap.md` M3 flipped to complete with acceptance
      evidence links; M3 `plan.md` `status` → `completed`.
- [ ] Release step per workflow §7.4 (tagging / CHANGELOG shape) —
      owner decision at close time.
