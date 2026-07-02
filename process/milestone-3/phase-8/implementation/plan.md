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
- **T3 / T4** — if T1's compiler-verification shows the widget-kind
  addition is compile-error-forcing across crates (non-additive kind
  carrier), the affected sites bundle into one buildable commit;
  otherwise the default split (compiler/IR, then runtime) holds.

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

- [ ] **Read every landing file end-to-end** (not grep-sample):
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
- [ ] **Compiler-verify the widget-kind addition**: introduce a
      throwaway `ToggleButton` kind at the IR carrier, `cargo build` the
      workspace to enumerate every dispatch/match site by compiler error
      (or, if the kind is string-carried, enumerate the dispatch tables
      by targeted search *and* a deliberate wrong-kind probe), record
      the site list, then **revert**. This is the trap-#1 pre-audit; the
      authoritative audit table is T3/T4's close artifact.
- [ ] **Fix and record the internal shape recommendations** the DD left
      to implementation: whether `ToggleButton` shares a Button-family
      code path internally (ButtonBase-style sharing is allowed; the
      *author-facing* taxonomy is DD-fixed), how `checked` reuses the
      existing single-boolean binding path, and how the V-a fill
      composes with `style` / `enabled` visuals (including the R-2
      ambiguity check plan against Mica/backdrop).
- [ ] **Gallery + hosts recon**: map the current
      `examples/gallery/gallery.ui` sections to the wireframe target
      (what folds where; what is swept); list retained capture scripts
      whose coordinates must re-derive (trap #2); read
      `examples/counter-c/{CMakeLists.txt,embed_uic.cmake,main.c}` and
      `examples/counter-zig/{build.zig,main.zig}` end-to-end and record
      the gallery-port deltas (component name, `.uic` path, window
      loop); confirm the CI step shapes to mirror
      ([ci.yml](../../../../.github/workflows/ci.yml)).
- [ ] **Fix and record the bisectable sequencing** (default
      T2 → T3 → T4 → T5 → T6 → T7; T2 is independent of T3/T4 and may
      run in parallel or reorder) and the inter-task seams (T5 needs
      T3+T4 for the tab band; T6 needs T5's final `gallery.ui`; T7
      needs T5+T6). Revise this plan if the default order changes.
- [ ] Sharpen the preamble §Technical risks table against the source;
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

- [ ] Restructure `examples/gallery/gallery.ui` to the **wireframe
      skeleton**: Grid overall frame (rows ≈ tab band 40 / content 1* /
      status 20), tab-band placeholder `HStack`, thumbnail area
      `ScrollView { WrapPanel { for … } }`, status strip `Text`,
      lightbox `ZStack` + `if` retained, Box `aspect` placeholders,
      `slot.*` placement — the first assembly step, not a throwaway
      (T5 completes it). Workspace stays green (build compiles
      `gallery.ui`).
- [ ] **Technical smoke (assistant, before any owner UI review):**
      build + launch `gallery-rust`, DPI-aware capture + analysis;
      check 0-sizing / clip breakage / `aspect`-abort / scroll breakage
      (Problem B's Fill→0 collapse surfaced early — framing R7).
      Findings triage recorded in [log.md](./log.md): fix with existing
      surface, or route to the A1-table placeholder / G(1) agreement /
      Problem B note. **No layout-engine change.**
- [ ] **FD-8-G(1) owner agreement packet** (informed by the smoke):
      the wireframe-fidelity / M3-placeholder table — real images →
      Box + Text; thumbnail click-to-open → explicit "Open lightbox"
      Button (hit-testing = M4); lightbox prev/next Button presence and
      what they do in M3 (spec.md Interaction) vs inert placeholder;
      scrollbar/wheel → scroll Buttons; thumbnail highlight → omitted
      (TH-a, DD-fixed); tab exclusion → α live, V-a background-only;
      status text static (no collection-length read exists in M3);
      which mutation Buttons (`Add` / `Remove` / `Clear` / `Reset`)
      survive as A1 minimal operation UI vs are swept; scrim/backdrop
      approximations. Owner agreement recorded.
- [ ] **Update the A1 feature-mapping table** (the framing's table is
      the initial hypothesis): the agreed table lands in this plan (or
      log.md) as the audit basis for all later UI checks; deviations
      from the framing table noted.

**Start gate:** T1 recon (gallery mapping) recorded; T2 gate selection
recorded. **End gate:** skeleton renders without collapse (or triaged),
G(1) agreement + updated A1 table recorded; workspace green.

---

### T3 — `ToggleButton` / `checked`: compiler + IR surface

The cross-cutting surface task, compiler half (gates trap #1 + #4;
risk R-1). Discharges ADR evidence items (1) and (2)-emit, and
propagation-audit point (i). **Full independent review**, with the
branch/test-focused check folded in for the reject matrix.

- [ ] `wasamoc` parse/check: `ToggleButton` admitted as a widget kind;
      attribute admission per [dsl_spec §4.17](../../../../docs/dsl_spec.md)
      — carries Button's `text` / `style` / `enabled` + `clicked`
      handler; `checked: <bool>` (literal or bool-state binding)
      admitted on `ToggleButton` **only**; default `false` when absent.
- [ ] SI-3 reject matrix (named diagnostics, firing tests both
      directions, trap #4): `checked` on `Button` / `Text` / other
      non-supporting widgets → reject; unknown attributes on
      `ToggleButton` → existing unknown-attr path; non-bool `checked`
      RHS → type reject.
- [ ] Lower + emit: `ToggleButton` kind + `checked` prop/binding
      through the existing single-boolean binding model (no new
      binding-target class); textual-IR emit shape per the spec; IR
      roundtrip tests (emit → load preserves kind + `checked` literal
      and binding forms).
- [ ] Positive fixtures: a `ToggleButton` with `text` / `style` /
      `enabled` / `clicked` / `checked` compiles and lowers; the α
      tab-band shape (3 ToggleButtons + 3 bool states + block-assign
      exclusion handlers) compiles (the stage-1 spike shape, now on the
      real surface).
- [ ] **Close artifact (trap #1):** the call-site audit table over
      widget-kind dispatch + attribute-admission sites (check / lower /
      emit / IR), each site classified.

**Start gate:** T1 site list + T3 gate selection recorded. **End
gate:** workspace + tests green; audit table + firing tests recorded;
full independent review before merge.

---

### T4 — `ToggleButton` runtime node + V-a checked visual + Windows fixtures

The runtime half (gates traps #1 / #4 / #7-adjacent; risks R-1 / R-2).
Discharges ADR evidence item (2)-loader and propagation-audit point
(ii). **Full independent review** (runtime structural: new widget
node).

- [ ] `wasamo-runtime` loader: `ToggleButton` kind dispatch; `checked`
      literal + binding registration reusing the `enabled` bool-binding
      path; loader **re-rejects** malformed IR (`checked` on a
      non-supporting kind) with a named diagnostic + firing test
      (dual-gate pattern).
- [ ] Widget construction: `ToggleButton` reuses Button's visual
      (text / style / enabled / click) and leaf measure/arrange — a new
      node, not a new layout primitive; **V-a checked visual**:
      background colour change on `checked`, composing with `style` and
      `enabled` states; verify the cue is unambiguous against the Mica
      backdrop (R-2 — if ambiguous in practice, that is an SI-1
      implementation-checkpoint revision recorded in log.md, not a new
      option choice).
- [ ] Reactive path: a bool-state change through the existing binding
      drives the visual (propagation-audit point (ii)); `clicked`
      handler block-assignment writes the state (W1 controlled — the
      widget does not self-toggle).
- [ ] Windows-runtime fixtures (CI-gated, fail-not-skip): checked
      visual state reflects the bound value at load; a state flip
      reaches the visual; the α exclusion shape drains to exactly-one-
      checked; `enabled: false` keeps the Phase-1 disabled contract
      (no `clicked` fire). Regression: existing Button fixtures
      unchanged.
- [ ] **Close artifact:** trap-#1 audit rows for the runtime sites;
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

- [ ] Tab band: 3 `ToggleButton`s (All / Albums / Favorites) with
      `checked:` bound to per-tab bool states and α block-assignment
      exclusion handlers; V-a visual distinguishes the selected tab.
      (**β fallback trigger:** if the live exclusion proves unworkable
      here, substitute β and record the SI-2 static-approximation
      accounting in this plan + the A1 table — preamble obligation 2.)
- [ ] Assemble the remaining agreed surface: `for`-generated thumbnail
      WrapPanel inside ScrollView (iteration + wrap + viewport);
      lightbox ZStack + `if` with close (and agreed prev/next form)
      Buttons; status strip; Box `aspect` placeholders; `slot.*` /
      `Cell` placement per the frozen 7b surface; the agreed minimal
      operation UI (scroll / any retained mutation Buttons).
- [ ] **Sweep the per-phase verification surfaces**: placement-demo
      sub-screen + its state/button (P7b) and
      `phase-7b/.../evidence/capture-placement-demo.ps1` retirement
      note; the Grid footer-clip demo (P5); the standalone static
      `Photo 1–10` WrapPanel; verification-only Buttons not in the
      agreed operation UI. No verification menu / dashboard remains
      (FD-8-E).
- [ ] Regression gate: workspace + all fixtures green; every M3 surface
      in the A1 table is exercised by the integrated app (audit against
      the T2-updated table, row by row, recorded in log.md).
- [ ] Re-derive layout-coupled coordinates in retained capture scripts
      (trap #2); park superseded scripts with the evidence they
      supported.
- [ ] **FD-8-G(2): owner first-render UI check** on the Rust host
      (early direction check; assistant pre-verifies α exclusion works
      live before presenting). Findings that exceed the plan go back to
      the A1 table / framing revisions (framing packet-G note), not
      silently absorbed.

**Start gate:** T2 (agreed table) + T4 merged; T5 gate selection
recorded. **End gate:** integrated gallery green + A1-table audit
recorded + G(2) owner check passed; capture coordinates re-derived.

---

### T6 — `gallery-c` + `gallery-zig` hosts + CI steps (A1 three-host)

Creates the two missing hosts from the counter templates and closes the
A1 "all three hosts" requirement. Build ordering per
[AGENTS.md §Build ordering](../../../../AGENTS.md): `wasamoc` builds
before the C / Zig hosts.

- [ ] `examples/gallery-c/`: port `counter-c` (CMakeLists +
      `embed_uic.cmake` + `main.c`) to the Gallery component; builds
      and runs against `target/release/wasamo.dll`.
- [ ] `examples/gallery-zig/`: port `counter-zig` (`build.zig` +
      `main.zig`); builds and runs.
- [ ] CI: add `gallery-c` / `gallery-zig` build steps mirroring the
      counter steps in [ci.yml](../../../../.github/workflows/ci.yml)
      (per-example enumeration — not a new build system; `gallery-rust`
      is already workspace-covered). Optionally add
      `wasamoc check examples/gallery/gallery.ui` beside the counter
      check step.
- [ ] **Cross-host parity (propagation-audit point (iii)):** launch all
      three hosts, DPI-aware capture, assistant analysis — C / Rust /
      Zig render the same integrated gallery (representative-host UI
      review stays Rust; C / Zig are identical-render / no-regression
      checks). Parity frames land under [evidence/](./evidence/) as
      `t6-parity-<host>.png`.

**Start gate:** T5 merged; T6 gate selection recorded. **End gate:**
three hosts build + run; CI steps green on the phase branch
(`workflow_dispatch` or push run recorded in log.md); parity analysis
recorded.

---

### T7 — Assistant GUI evidence package + FD-8-G(3)

The authoritative assistant-visible evidence (ADR item (5); gates trap
#7; GUI-render class → **full independent review**). Assistant evidence
= launch + DPI-aware screenshot (`CopyFromScreen`) + analysis;
`Start-Process` survival is a supporting signal only. Owner smoke is
T10's separate gate.

- [ ] Capture the agreed state set on the Rust host (labelled frames
      under [evidence/](./evidence/), analysis in
      `evidence/README.md`): **default view**; **lightbox open +
      closed** (subtree present/absent pair); **selected two-frame
      positive control** (click a different tab → its background
      changes **and** the previously-selected tab clears — exclusion in
      the same two frames); **wrap/overflow** (narrow-width reflow
      and/or scroll-offset frames per the agreed state set).
- [ ] Verify each positive control distinguishes the intended behaviour
      from a static look-alike (AGENTS.md §Testing rules); note known
      M4 residuals (DPI blur, dynamic title) as residuals, not
      failures.
- [ ] **FD-8-G(3): owner confirms the two-frame positive controls**
      (selected/exclusion + lightbox) over the captured frames.

**Start gate:** T6 merged (final surface + re-derived coordinates); T7
gate selection recorded. **End gate:** evidence set + analysis landed;
G(3) owner confirmation recorded; full independent review.

---

### T8 — A12 editorial pass + external-reader smoke + milestone audits

The spec-closure work (ADR item (6), minus the T11 marker flip).
Reads the **landed** implementation (pin-to-landed-source learning),
not the design draft.

- [ ] Editorial pass over `docs/dsl_spec.md` **whole-document** at the
      external-reader bar: §4.17 re-verified against the shipped
      implementation; the DD-002 accepted dispositions verified present
      and honestly worded — B-1b (PM-2 both forms + provisional wrapper
      rule), B-2c (sizing future note, no shape reservation, no M4/M5
      schedule in the public draft), B-3b (container-owned default
      semantics), B-4a (spelling affirmed), B-5b (constant-per-instance
      placement, no compatibility guarantee), B-6b (five DD-001 axes as
      future notes); the α M3-era exclusion note at the DD-002-coupling
      strength (items 1–3); no DD/option labels in spec prose.
- [ ] **External-reader smoke** (milestone-end criterion 5): a
      structured walkthrough asking whether a reader with only
      `docs/dsl_spec.md` could reproduce each M3 surface (A2–A10, A13,
      grammar surfaces) against a hypothetical C-ABI host; per-surface
      verdict recorded in [log.md](./log.md); every "not yet" fixed as
      remaining editorial work in this task. **B-3b check:** if the
      defaults still read as arbitrary, trigger the separate B-3c
      revision procedure (owner-gated), not a silent rewording.
- [ ] **A11 auditability check** (milestone-end criterion 4): every M3
      phase ADR names the `docs/dsl_spec.md` sections it updated;
      gaps listed + closed (pointer fixes in the ADRs' allowed
      sections or log-recorded disposition).
- [ ] **No-silently-deferred-surface audit** (milestone-end criterion
      6): the M3 target-app pre-doc's 必要 surface list checked item by
      item — shipped, or recorded as a deviation in the M3 plan
      Revision log; result recorded for T9's handoff.
- [ ] `docs/notes/architectural-family.md`: the Phase 8
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

- [ ] Draft `process/milestone-3/handoff.md` (milestone-level; distinct
      from this phase's `implementation/handoff.md`): PM-2 wrapper-rule
      decision (pre-1.0, re-triggers); Problem B disposition (VDR
      Accepted: M4/M5 spike schedule + M6 backstop); DD-001's five
      deferred axes with triggers; default-alignment residual per the
      T8 B-3b outcome; spelling affirmed-keep record; DPI / dynamic
      title / modal focus M4 residuals; TypedValue / structured-item
      triggers (Phase 7 carry); host-state-boundary record; anything
      the T8 audits surfaced.
- [ ] **FD-8-G(4): owner reviews the public draft (post-T8
      `docs/dsl_spec.md`) + the M3 handoff draft together** before the
      phase close batches open.

**Start gate:** T8 merged. **End gate:** G(4) owner review passed;
handoff draft parked at `status: draft` (finalized at milestone close,
not here).

---

### T10 — FD-8-G(5) final owner human-visible smoke

The owner-performed gate (separate from T7's assistant baseline).

- [ ] Assistant prep: rebuild the three hosts; author the owner
      observation script at `evidence/t10-owner-smoke-script.md`
      (launch / navigation / per-state observation + pass-fail
      criteria) covering the agreed state set — default view, lightbox
      open/close, tab selection with exclusion (positive control:
      clicking tabs moves the single selected highlight), wrap/overflow
      — on the representative Rust host, plus launch + default-view
      confirmation on C and Zig.
- [ ] Owner runs the smoke and observes per the script; owner
      explicitly accepts, or records a fail observation → fixes land
      additively on the task branch → re-run to green (Phase 4 / 7b
      precedent).
- [ ] T10 step-end retrospective recorded at
      `../retrospectives/t10.md`.

**Start gate:** T9 merged (surface frozen). **End gate:** owner
acceptance recorded.

---

### T11 — Step-end local gates + Moment 2 re-sync (public-draft promotion)

The final task's step-close half per the ownership split
([preamble.md §Obligations item 4](./preamble.md)). T11 is a
document-sync and step-close task, **not** the phase-close or
milestone-close task.

- [ ] T11 start gate recorded in [log.md](./log.md): carry-over from
      log.md and every Phase 8 task retrospective checked; T11 /
      phase-end / milestone-end ownership split made auditable.
- [ ] `cargo fmt --all -- --check` green locally.
- [ ] Local clean rebuild green (`cargo clean` → release build → debug
      build → `cargo test --workspace`), plus the C / Zig example hosts
      in the AGENTS.md build order. CI green is phase-end-owned.
- [ ] `docs/dsl_spec.md`: §4.17 / §4.18 markers flip to `M3-Phase 8
      closed; implementation-synced`; divergence corrections folded
      (landed shapes, not design-draft sketches); **`status:
      public-draft` frontmatter marker lands** (C-2 — gated on the
      surface running, which T5–T10 established); the public-draft
      promotion change-history entry lands (linking the M3 ADRs and the
      public-draft anchor); the T8 external-reader smoke result
      recorded.
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
