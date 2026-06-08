## Decisions log

- **2026-06-08 / T7b start gate — A2a `IrComponent` host surface IR migration:**
  selected implementation-gate traps before choosing the approach. Applies:
  **#1 semantic migration** (`IrComponent` gains `host_props` /
  `host_bindings`; every component construction, textual-IR parse / emit,
  traversal, validation, title-resolution, and test fixture site must be
  audited); **#2 missed side effects** (component-level host attributes must
  stop contaminating the content root, the static title path must move to the
  new surface, ZStack root exemptions must be removed, old root-squatted IR
  must be rejected, and the gallery/counter examples must still lower and
  load); **#4 untested authored branch** (new catalog rejects for unknown
  host props / all host bindings, non-string title validation, and old
  root-squatted title/binding rejection branches require tests that fire
  them); **#5 carry-forward underweighted** (the host-owned-attribute /
  content-root separation is an M4-facing invariant and must be recorded with
  evidence and a re-trigger criterion if not fully folded into the T9 Moment 2
  sync). **#6 deterministic-failure disposition** is armed if a failure
  recurs or vanishes on retry during migration. Not applicable: **#3 parallel
  data drift** (the migration adds component-owned lists but no derived index,
  cache, or parallel vector whose source must be atomically mutated); **#7 GUI
  positive control** (T7b changes IR / validation / loader behavior; T8 owns
  the owner-visible GUI smoke after this lands). Review lane: **full
  independent review** because this is a schema / textual-IR migration.
- **2026-06-08 / T7b A2a `IrComponent` host surface IR migration:** landed
  DD-M3-P6-008's A2a code path. `IrComponent` now owns
  `host_props` / `host_bindings`; `wasamoc` lowering stores component-level
  host attributes there instead of splicing them onto the content root;
  textual IR emits / parses `host prop ...` and `host bind ...`; `wasamoc
  check` validates component-level host attributes through the Window-only
  Phase-6 host catalog (`title` / `backdrop` / `theme`) and rejects host
  bindings; the runtime validates the mirrored catalog, resolves the static
  title from `host_props`, rejects host bindings, and rejects old
  root-squatted host props / bindings. The ZStack root no longer carries the
  temporary window-prop exemption from T7.
  - **Close-gate #1 call-site audit:** `rg "struct IrComponent|IrComponent|root\\.props|root\\.bindings|resolve_static_window_title|validate_phase6_zstack_node_invariants|title|host_props|host_bindings" wasamo-ir wasamoc wasamo-runtime examples -n` and `rg "IrComponent \\{" -n` were used to classify migration sites. `wasamo-ir/src/lib.rs` = must-dispatch schema owner; added fields and a host/content-root separation unit test. `wasamoc/src/lower.rs` = must-dispatch construction site; component-level static/dynamic attrs now lower to `host_props` / `host_bindings`, not `root`. `wasamoc/src/emit.rs` = must-dispatch textual writer; emits `host prop` / `host bind` before the root node. `wasamoc/src/check.rs` = must-dispatch compiler gate; added `HOST_STATIC_ATTRS`, known-host accept, unknown-host reject, and host-binding reject. `wasamo-runtime/src/ir_loader.rs` = must-dispatch parser / validator / title / ZStack gate; parses host members, validates the runtime mirror, moves title resolution to `host_props`, rejects old root-squatted host attrs, and removes the ZStack root exemption. `wasamo-runtime/tests/abi_load_ui.rs` and `wasamo-runtime/tests/ir_loader_roundtrip.rs` = must-dispatch external seam tests; updated to canonical `host prop` shape. `examples/*/*.ui` = ignore-OK source surface; unchanged because A2a is internal IR lowering, verified by `wasamoc check` / build / roundtrip.
  - **Close-gate #2 structural side effects:** content roots are pure widget roots again; static window-title resolution moved from `component.root.props` to `component.host_props`; ZStack validation now treats any root `title` / `backdrop` / `theme` prop as malformed old IR rather than a root-only exemption; ABI `wasamo_load_ui` still calls `resolve_static_window_title` with the same signature and no new ABI surface.
  - **Close-gate #4 branch tests:** `component_level_host_attrs_accepted`, `component_level_unknown_host_attr_rejected`, and `component_level_host_binding_rejected` pin compiler catalog behavior; `component_host_prop_lowers_to_host_surface` pins no-splice lowering; `full_counter_ir_roundtrip` and `ir_loader_roundtrip::counter_ui_emit_then_parse_yields_equal_ir` pin canonical emit/parse; `host_prop_parses_on_component_surface`, `host_attribute_catalog_mirrors_wasamoc`, `host_surface_rejects_unknown_host_prop`, `host_surface_rejects_host_binding`, `static_window_title_rejects_non_string_host_prop`, `root_zstack_accepts_host_props_on_component_surface`, `old_root_squatted_host_prop_rejected`, and `old_root_squatted_host_binding_rejected` pin runtime parsing / mirror / title / old-shape rejection; `abi_load_ui` pins the malformed canonical host-title shape at the ABI boundary.
  - **Close-gate #5 carry-forward:** the new cross-task invariant is "host-owned attributes stay separated from the content root; future host/base modeling may replace the carrier but must preserve the separation." It is already in DD-M3-P6-008 and the T9 Moment 2 plan bullets; re-trigger criterion: any M4/M5 work that adds host/base attributes, dynamic host bindings, base-name validation, or an ABI-facing window descriptor must re-check that it does not put host attributes back on the content root. T7b retro item 10 classifies this as `phase-sync`.
  - **Close-gate #6 disposition:** no deterministic runtime / native failure recurred. Expected migration-test failures from old assertions (`root.props` / interim divergence pins) were fixed by updating those tests to the A2a canonical shape; no failure was re-rolled to green without a code or expectation change.
- **2026-06-08 / T7b local verification:** `cargo check --workspace` —
  green; `cargo test -p wasamo-ir` — green (18 tests); `cargo test -p
  wasamoc --lib` — green (319 tests); `cargo test -p wasamo-runtime --lib
  ir_loader::tests` — green (141 tests); `cargo test -p wasamo-runtime
  --test ir_loader_roundtrip` — green (7 tests); `cargo test -p
  wasamo-runtime --test abi_load_ui` — green; `cargo run -p wasamoc --
  check examples\gallery\gallery.ui` — green; `cargo run -p wasamoc --
  build examples\gallery\gallery.ui` — green; `cargo fmt --all --
  --check` — green; post-commit `cargo clean` completed (`5603 files,
  1.5GiB` removed); `cargo build --release --workspace` — green; `cargo
  build --workspace` — green; `cargo test --workspace` — green (workspace
  unit / integration / doc tests). Existing Cargo warnings about the
  `wasamo` linkable target / `wasamo-sys` import-library ordering were
  observed.
- **2026-06-07 / T7 review round 2 (third-party re-review corrections):**
  (1) The binding facet of the DD-M3-P6-008 divergence was **empirically
  verified**, not just inferred — `wasamoc build` emits `bind title =
  (str-prop-read s)` on a ZStack root and `wasamoc check` accepts it (exit 0),
  while the runtime rejects the same IR. (2) Pinned the divergence on **both
  gates**: accept side `zstack_root_component_window_attrs_accepted`
  (`wasamoc`), reject side now includes the faithful binding pin
  `root_zstack_rejects_spliced_component_window_binding` (exact emitted IR)
  rather than only the proxy `zstack_binding_rejected_at_validate`. (3)
  **Corrected a false commit message**: the reindent commit had claimed "CRLF
  normalization", but the repo has no `.gitattributes`, `core.autocrlf=true`,
  and the gallery.ui blob is LF — the working-tree "mixed endings" was a
  checkout artifact, not a committed defect; the message was amended to
  reindent-only. (4) retro/plan: clarified the screenshot positive control is
  **z-order / dimming only** (structural present/absent + dirty-layout path
  are proven by T5's headless tests, corroborated — not proven — by the
  screenshot); made **T8 smoke run after T7b**; refreshed `Refs` and
  `Merge Readiness` to record the shipped interim + open DD-M3-P6-008.
- **2026-06-07 / T7 review follow-up — component-root window-attribute
  boundary (DD-M3-P6-008) + interim pins:** the T7 review found that the
  two validator-fix branches were the visible symptom of a deeper
  divergence: component **window** attributes are spliced onto the **root
  widget's** `props` / `bindings` ([`wasamoc/src/lower.rs`](../../../../wasamoc/src/lower.rs#L59)),
  so `wasamoc check` (pre-splice AST, accepts any component-level name) and
  the runtime loader (post-splice IR, ZStack root rejects outside a
  three-name allowlist) disagree. The scope is broader than props — a
  component-level dynamic bind (`bind title = …`, FD-D) hits the same
  unconditional ZStack binding rejection. The ZStack-child-of-ZStack
  placement branch was separately a **T3 validator coverage gap** (`wasamoc
  check` already accepted it). Disposition: raised as **DD-M3-P6-008
  (Proposed)** with options A (IR-schema separation) / D (compiler-owned
  catalog mirrored by runtime) / C (rejected); tracked at **plan.md T7b**,
  time-boxed before phase close. Interim runtime behavior (reject outside the
  allowlist) is pinned by `nested_zstack_rejects_component_window_prop`,
  `root_zstack_rejects_non_window_component_prop`,
  `root_zstack_rejects_placement_prop`, and `zstack_binding_rejected_at_validate`.
  Also recorded: the T7 commit was amended to add the missing
  `Co-Authored-By: codex` trailer (codex authored the T7 implementation);
  the "existing slices byte-identical" item carries the caveat that the
  VStack gained `h-align`/`v-align: stretch` to fill the new root ZStack
  (structural wrapping, not a content change); a geometry positive-control
  observation (photo `aspect: 4:3` / centre-column width, caption-row fit)
  was added to T8 since the assistant analysis covered z-order / dimming but
  not geometry; and the wrapped VStack block was re-indented under the root
  ZStack (whitespace-only).
- **2026-06-07 / T7 start gate — gallery lightbox slice + assistant GUI evidence:**
  selected implementation-gate traps before editing. Applies: **#2 missed
  side effects** (the authored slice must wire the visible `Open lightbox` /
  `x` Buttons through `is_lightbox_open`, materialise the conditional
  subtree, preserve overlay z-order, and corroborate the static `"Gallery"`
  title in the launched host); **#5 carry-forward underweighted** (only if
  the GUI proof or additive gallery growth surfaces a later-task invariant,
  in which case it must be recorded with evidence and a re-trigger criterion);
  **#7 weak GUI evidence** (T7's deliverable is GUI rendering, so close
  evidence must include launch + `Graphics.CopyFromScreen` screenshots +
  assistant analysis with the closed/open toggle pair as positive control).
  Not applicable: **#1 semantic migration** (no enum / IR / schema variant or
  field is added); **#3 parallel data drift** (no parallel vector / map /
  index is introduced or mutated by this task); **#4 untested authored branch**
  (no new diagnostic / reject / size branch is authored; existing compiler and
  runtime branches are exercised through the gallery positive control);
  **#6 root cause** (no recurring or vanished failure observed at task start).
  Review lane: **full independent review** because the task closes
  GUI-render evidence.
- **2026-06-07 / T7 gallery lightbox slice + assistant GUI evidence:** T7
  grows `examples/gallery/gallery.ui` with a root `ZStack` whose first child
  is the existing gallery content and whose second child is a
  `bool`-controlled `if is_lightbox_open { ZStack { ... } }` overlay.
  `Open lightbox` and `x` are plain text Buttons that write
  `root.is_lightbox_open`, proving the event handler → bool state →
  conditional subtree path. The overlay uses a full-viewport scrim
  (`Box.fill = #10182099`) below a centered lightbox panel with a
  `Box { aspect: 4:3 }` + `Text` photo placeholder, caption text, and
  `<` / `>` / `x` nav Buttons. The assistant-visible screenshots are:
  `implementation/evidence/t7-lightbox-closed.png`,
  `implementation/evidence/t7-lightbox-open.png`, and
  `implementation/evidence/t7-lightbox-closed-after-click.png`; the capture
  helper is `implementation/evidence/capture-lightbox.ps1`.
  - **Close-gate artifacts:** #2 side effects — the visible Button writes
    materialise and remove the conditional subtree without resize; the open
    frame shows the overlay above the thumbnail gallery, and the close-after
    frame shows the overlay gone. The `"Gallery"` title bar is visible in all
    frames, corroborating T6's static-title host path. #4 branch tests
    (surfaced during implementation) — `root_zstack_accepts_component_window_props`
    pins that component-root window props (`title` / `backdrop` / `theme`)
    do not count as ZStack widget attributes when the root widget is ZStack;
    `root_zstack_still_rejects_widget_attribute` keeps ordinary ZStack attrs
    rejected at the root; `zstack_child_zstack_accepts_placement_props` pins
    that `h-align` / `v-align` remain legal parent-owned placement props when
    the ZStack direct child is itself a ZStack. #5 carry-forward — none new;
    the surfaced invariants are T7-owned validator bugs now fixed and pinned
    by tests rather than deferred. #6 deterministic-failure disposition —
    first launch failed deterministically with `wasamo_load_ui: IR validation
    error: ZStack accepts no Phase-6 attributes; found title`; after fixing
    the root window-prop boundary, the next deterministic failure was the same
    validator class for `h-align` on a ZStack direct-child ZStack. Both were
    root-caused to the Phase 6 ZStack validator conflating widget attributes
    with wrapper-carried root/window props or parent-owned placement props,
    fixed in `validate_phase6_zstack_node_invariants`, and rerun through the
    gallery launch path to green. A separate assistant capture failure
    (`Graphics.CopyFromScreen` / `BitBlt` returned "The handle is invalid")
    was isolated to sandboxed capture, not runtime rendering: the owner
    reported PID 13748 visibly displayed Gallery, and the same
    `CopyFromScreen` capture succeeded when rerun with GUI escalation. #7 GUI
    evidence — escalated `CopyFromScreen` over the visible `"Gallery"` HWND
    captured the closed/open/closed-after-click positive-control triplet at
    800x600. Assistant analysis: closed frame shows the gallery thumbnails and
    `Open lightbox`; open frame shows the scrim dimming (not replacing) the
    thumbnails, the photo placeholder / caption / nav painted over the scrim,
    and the `"Gallery"` title bar; close-after-click shows the overlay gone
    immediately after the `x` click, without a resize.
- **2026-06-07 / T6 start gate — R1 Window-title host-wiring:** selected
  implementation-gate traps before coding. Applies: **#2 missed side
  effects** (the static component-level `title` must affect native window
  creation, while dynamic `bind title` remains deferred); **#4 untested
  authored branch** (the loader adds a non-`Str` `title` rejection branch and
  the absent / empty fallback branch must be pinned); **#7 positive-control
  discipline for visible state** (T6's CI-gated evidence is live HWND title
  state, while screenshot / assistant analysis of the title bar is owned by
  T7 and owner-visible corroboration by T8). Not applicable: **#1 semantic
  migration** (no enum / IR schema variant or field is added); **#3 parallel
  data drift** (no parallel vector / map / index is introduced or mutated);
  **#6 root cause** (no recurring failure observed at task start). **#5
  carry-forward was reclassified during review follow-up**: dynamic title was
  already ADR-deferred, but T6 did add a title-specific loader invariant that
  should inform the later Window-prop seam. Review lane: **full independent
  review** because the task includes Windows-runtime evidence, with the trap
  #4 branch/test check folded into that review.
- **2026-06-07 / T6 R1 static Window-title host-wiring:** T6 confirms the
  R-D extraction point: the static component-level `title:` has already been
  spliced onto `component.root.props`, while a dynamic `bind title = ...`
  remains in `root.bindings` and is still the DD-M3-P6-006 deferred
  window-property-binding seam. The runtime now validates that any root
  `title` prop is an `IrLiteral::Str`, resolves an absent or empty title to
  `DEFAULT_WINDOW_TITLE`, and passes the non-empty string literal to
  `window::create` in `wasamo_load_ui`. The malformed-title rejection is
  intentionally single-sourced in `validate_static_window_title`;
  `resolve_static_window_title` is a crate-local infallible projection over a
  validated component. No ABI signature, export, `PropertyValue` tag, or
  `docs/abi_spec.md` text changed. The stale counter example README notes
  that said DSL titles were still dropped were refreshed to match the
  implemented host path.
  - **Close-gate artifacts:** #2 side effects — static `title` now affects
    native window creation; dynamic title remains deliberately deferred and
    unwired; `backdrop` / `theme` remain untouched. #4 branch tests —
    `static_window_title_resolves_string_or_default` pins absent / empty /
    string resolution, `static_window_title_rejects_non_string_root_prop`
    pins the loader rejection branch, and `abi_load_ui` pins
    `WASAMO_ERR_IR_MALFORMED` at the ABI boundary. #5 carry-forward — later
    Window-derived props should reuse this validate-then-resolve split rather
    than adding silent fallback for wrong-typed direct IR. #7 Windows-runtime
    state evidence — `static_component_title_reaches_native_window` lowers a
    `.ui` declaring `title: "Gallery"`, loads it through `wasamo_load_ui`,
    then reads the live HWND title via `GetWindowTextW`; the positive control
    is `"Gallery"` rather than the prior `"Wasamo"` default. T7/T8 still own
    the screenshot / human-visible title-bar corroboration.
- **2026-06-05 / Observation 5 remediation step 1 — marshal onto owning
  thread + abbreviated retro (branch `test/obs5-step1-marshal-owning-thread`
  → `feat/m3-phase-6`):** step 1 — owner-scheduled at the step-2 close — is
  now done. The keep-alive `tests/common/mod.rs` park thread became a
  work-queue executor; `run_on_owning_runtime_thread_or_skip` replaces
  `init_runtime_or_skip` and runs each Compositor test body on the single
  owning thread (panic caught there + re-raised on the libtest thread so
  `#[test]` still fails correctly). The five ≥2-Compositor binaries wrap
  their bodies in the helper closure. This eliminates the cross-apartment
  residual step 2 only tolerated. Abbreviated retro (out-of-band step, no
  numbered task slot → folded here, not a `tN.md`), per retrospectives.md
  items 1–11:
  - **Main learning:** the "does this one helper hold too many
    responsibilities?" question resolved not to *one responsibility* but to
    **shared change/deletion locality + coupling avoidance** — init, skip
    policy, marshalling, and panic-relay have *different* change drivers yet
    are added and deleted together, and splitting the skip check back to the
    callers would re-introduce a two-calls-must-agree coupling on the same
    process-global init outcome. Recorded in the helper's own rationale
    comment so the design intent travels with the code.
  - **Items 2 / 6 / 7 / 8 = none:** no spec-doc (`abi_spec` / `architecture`
    / `dsl_spec`) change; no new or promoted DD / ADR; no milestone-AC or
    phase-structure change. Changes are limited to the test harness and
    `docs/notes/`.
  - **Item 9 (carry-forward):** none new — step 1 *was* the carry-forward
    from the step-2 close and is now discharged. The helper's only remaining
    lifecycle is its deletion condition (process-per-test runner, e.g.
    `cargo nextest`, or libtest ceasing per-test thread spawn), documented in
    [docs/notes/verification-environments.md Observation 5](../../../../docs/notes/verification-environments.md)
    §Remediation status and the helper module doc.
  - **Item 10 (cross-task constraint), `doc-folded`:** unchanged in substance
    — ≥2-Compositor binaries route through the shared helper; the helper now
    additionally *executes* bodies on the owning thread rather than only
    keeping the apartment alive. Folded into Observation 5 and the helper's
    module doc; pointer only here.
  - **Item 11 (ownership):** Observation 5 §Remediation status flips step 1
    to DONE in the same change; no open `[ ]` left implicit.
- **2026-06-05 / Observation 5 teardown-AV investigation — abbreviated
  retrospective (branch `investigate/obs5-scrollview-teardown-av` →
  `feat/m3-phase-6`):** an out-of-band investigation, not a numbered plan
  task, so it gets an abbreviated retro folded here rather than a
  `retrospectives/tN.md` file (no task slot to invent). The clean-rebuild
  gate is in the CI/verification log above (green, first run, no AV).
  Per the task-end checklist (retrospectives.md items 1–11):
  - **Main learning:** the original symptom framing can be wrong — the AV
    was filed for two phases as a "process-exit teardown" fault on the
    assumption that a printed `... ok` meant the crash was in teardown. A
    minidump (`procdump -e -ma` + `cdb`) showed it is in the *next* test's
    `build_widget_tree` → `CreateSpriteVisual`, dispatching through a vtable
    in an unloaded `dcomp.dll`. The diff-independent recurrence plus the
    "capture the dump, don't re-roll" standing rule is what eventually
    forced the correct diagnosis. Method (repro matrix → minidump → faulting
    module) generalises to future native-COM AVs.
  - **Items 2 / 6 / 7 / 8 = none:** no spec-doc (`abi_spec` / `architecture`
    / `dsl_spec`) change; no new or promoted DD / ADR; no milestone-AC or
    phase-structure change. Changes are limited to `docs/notes/` and the
    test harness.
  - **Item 9 (carry-forward):** remediation **step 1** (marshal Compositor
    work onto the owning thread) is deferred to a separate owner decision —
    *no hard deadline*, with revisit triggers — recorded in
    [docs/notes/verification-environments.md Observation 5](../../../../docs/notes/verification-environments.md)
    §Remediation status. The residual it addresses (test bodies calling
    non-agile Composition objects cross-apartment, benign only while
    `dcomp.dll` is held resident) is UB-adjacent but test-harness-only;
    production is unaffected (hypothesis A confirmed, B excluded).
  - **Item 10 (cross-task constraint), `doc-folded`:** Compositor
    integration binaries with two or more Compositor tests must initialize
    the runtime on a process-lifetime thread via the shared
    `wasamo-runtime/tests/common/mod.rs` keep-alive helper, so the
    Compositor's apartment is not torn down between tests. Folded into
    Observation 5 and the helper module's own doc comment (which states the
    rationale, when the helper is/ isn't required, and its deletion
    conditions); pointer only here.
  - **Item 11 (ownership):** the phase plan's Phase 7 handoff bullet for the
    teardown-AV investigation is revised in the same change to reflect
    root-cause-done + step 2 landed + step 1 owner-deferred (no hanging
    `[ ]` left implicit).
- **2026-06-05 / T5 conditional reactive runtime:** T5 fills the
  DD-M3-P6-004 / 005 structural binding seam without adding any IR / ABI /
  grammar or host-facing error surface. `BindingTarget::ConditionalSubtree { parent,
  declared_member_index }` is registered through a new
  `register_conditional_binding` wrapper over `EffectHandle::new`; property
  binding entry points now destructure `BindingTarget` refutably. The
  runtime tracks each parent's declared member slots as
  `DeclaredMemberSlot::{Widget, Conditional(state)}` while iterating
  `IrMember` in declared order; the materialised insertion/removal index is
  recomputed from preceding declared slots and each conditional's live
  presence bit on every mutation. This closes the T4 carry-forward
  constraint for T5's positional path: the traversal that materialises
  conditionals and computes positional metadata dispatches on `IrMember`,
  not `widget_children()`.
- **2026-06-05 / T5 positional Visual + ZStack placement update:** The
  `WidgetNode::insert_child` Visual operation is now index-aware: append
  still uses `InsertAtTop`, while mid-list insertion uses `InsertBelow`
  relative to the current child at the target index so live Visual sibling
  order matches `WidgetNode.children`. Because ZStack stores
  parent-owned child placement metadata parallel to materialised children,
  dynamic insert/remove also updates the ZStack placement vector at the same
  index (`insert_child_with_zstack_placement` / `remove_child`). This is the
  T5 R-F closure and preserves the T4 traversal-audit rule for positional
  metadata.
- **2026-06-05 / T5 ZStack placement construction refactor:** The T5
  positional-mutation fix also moved static ZStack placement construction
  from a precomputed `collect_static_zstack_placements` vector to the same
  per-child insertion path used for dynamic members:
  `append_static_member` calls `insert_child_with_zstack_placement` whenever
  the parent is ZStack. The old static reducer helpers
  (`evaluate_static_condition`, `collect_static_zstack_placements`) are now
  `#[cfg(test)]`; their unit tests still pin reducer logic and are now
  commented as such, but no longer guard a production call-site directly.
  The new load-bearing index reducer
  `materialized_index_for_declared_member` has headless unit coverage in
  `materialized_index_counts_preceding_widgets_and_live_conditionals`,
  including the preceding-conditional removal shift. Production placement
  evidence is covered by the ZStack Windows integration fixtures and T5's
  `conditional_zstack_reinsert_uses_declared_placement_metadata`.
- **2026-06-05 / T5 parent-owned metadata mutation constraint:** The
  ZStack placement-vector fix surfaced a future-structural constraint:
  under the current SoA model, any structural mutation primitive that changes
  a materialised child list under a container with parent-owned positional
  metadata must update that metadata atomically with `WidgetNode.children`
  and the live Visual sibling order. T5 implements the single-child case for
  conditional insert/remove, but this invariant is a cost of the current
  parallel-vector representation, not a law that Phase 7 must preserve.
  Phase 7 must decide the placement storage model before `ForLoopSubtree`:
  keep SoA parallel vectors (affirm DD-M3-P6-002's implementation shape),
  move placement onto child nodes / child records (AoS, superseding the
  current shape), or use a `WidgetId`-keyed metadata map. Children ↔ Visual
  order synchronisation is unavoidable in every model; the reducible
  parallel structure is the placement vector itself, and the value of
  removing it grows linearly with future parent-owned per-child metadata
  kinds. T5 is sample 1 for dynamic parallel-vector sync; `ForLoopSubtree`
  would be sample 2, so the ≥2-sample discipline makes Phase 7 the decision
  point. T5's `append_child` consolidation is a local guard and remains
  subordinate to that Phase 7 storage-model decision.
- **2026-06-05 / T5 structural-binding handover constraints:** Conditional
  initial presence is now established by `EffectHandle::new`'s eager initial
  run; a future reactive-engine change that delays initial Effects must
  preserve this loader materialisation contract or add an explicit
  initialisation path. ZStack-aligned structural insertion must use the
  placement-carrying API; T5 guards the former two-path footgun by making
  `append_child` delegate to `insert_child_inner(len, child, None)`, so the
  centered ZStack default is concentrated in one insertion primitive.
  Conditional mutation build / insert / remove / slot-missing failures
  remain log-only (`eprintln!`) and are not surfaced through runtime health;
  Phase 7 range mutation should re-check whether log-only structural failure
  remains sufficient for multi-child edits. The final API consolidation shape
  remains dependent on the Phase 7 placement-storage model decision.
- **2026-06-05 / T5 self-review layout invalidation fix:** The initial T5
  implementation inserted/removed conditional children synchronously but did
  not mark the owning window layout-dirty on structural success. Self-review
  classified that as T5-owned, because a conditionally-present subtree can
  affect parent measurement/allocation even when no size-affecting property
  write occurs. `mutate_conditional_subtree` now marks dirty via the parent
  widget after successful insert/remove. The same pass added
  `conditional_zstack_reinsert_uses_declared_placement_metadata`, which
  drives conditional insert/reinsert under `ZStack` through
  `run_layout_as_window_root` and asserts the dynamic child's declared
  `h-align` / `v-align` placement.
- **2026-06-05 / T5 follow-on classification for dirty-layout evidence:**
  This is **not** a Phase 7 carry-forward. T5 fixed the structural mutation
  primitive, but the full real-window path (`mark_layout_dirty_for` →
  `drain_if_outermost` → `flush_layout` under `WindowState`) must be pinned
  by Phase 6 GUI evidence. T7 now owns the assistant screenshot before/after
  pair captured immediately after the click-driven lightbox toggle, without
  relying on resize; T8 owns the same owner-visible smoke criterion and the
  Phase 6 fix slot if the path fails. T9 Moment 2 architecture sync must
  include `docs/architecture.md` §6.6 so layout invalidation is no longer
  documented as property-change-only.
- **2026-06-05 / T5 closes T4b DD-M3-P6-007 comment handoff:** T4b left a
  narrow source-comment follow-up for the next `ir_loader.rs` touch: refresh
  `validate_phase4_node_invariants` from the "interim / open DD-007" wording
  to the accepted-(a) ScrollView direct-conditional rejection. T5 performed
  that refresh; the handoff is closed without reopening DD-M3-P6-007.
- **2026-06-05 / T5 reactive-drain items 1–3 disposition:** T5 implements
  the DD-M3-P6-005 DB-1 item-4 proof and does **not** revise the inherited
  reactive-drain items 1–3. Cycle detection, ordering ties, and fan-out ×
  `MUTATION_CAP` remain the DD-M3-P6-005 SM-1 carry-forward exactly as the
  ADR records: the conditional insertion Effect writes the widget tree, not
  its own Signal; quiescent child order is fixed by declared member order;
  and large-subtree cap strategy is deferred until the structural family
  (`for` / larger repeated subtrees) reveals the real budget requirement.
  This entry covers only the reactive-drain items 1–3 disposition; the
  separate parent-owned metadata mutation constraint above is the T5-specific
  carry-forward candidate.
- **2026-06-05 / T5 surfaced known issue — ScrollView teardown AV (carry-forward):**
  T5's follow-up clean rebuild re-observed the `scroll_view_layout_integration`
  process-exit access violation (see the CI/verification entry below). It is
  diff-independent (same fault recorded in Phase 5 T1 with a `wasamoc`-only
  diff) and therefore **not** a T5 regression, so it does not gate the T5
  merge. It is **not** settled as benign either: the fault is in COM/Compositor
  teardown at process exit and a real runtime teardown defect (hypothesis B)
  is not excluded. Disposition recorded as
  [docs/notes/verification-environments.md Observation 5](../../../../docs/notes/verification-environments.md)
  (hypotheses A/B; "capture a minidump on the next occurrence rather than
  re-rolling to green"; the faulting module decides the fix). **Carry-forward:
  promote this into the phase-end `handoff.md` (T9) as a Phase 7 / runtime
  investigation item** — root-cause the teardown AV from a captured dump and
  decide the permanent fix (never-dropped global Compositor + no
  `RoUninitialize`, vs a `widget_destroy` teardown-order fix). This has now
  recurred ≥2 times, so by the project's ≥2-sample discipline it graduates
  from "transient" to a tracked known issue.
- **2026-06-03 / T4 IrMember schema migration:** T4 landed the accepted
  DD-M3-P6-004 O1 shape directly: `IrNode.children` is now
  `Vec<IrMember>`, with `IrMember::Widget(IrNode)` and
  `IrMember::ControlFlow(ControlFlowNode::If { branches })`.
  Construction-site migration used a narrow helper discipline rather than
  a broad abstraction: production walkers use `IrNode::widget_children()`
  when an invariant is widget-child-only, and explicit
  `IrMember` dispatch where control flow is semantically relevant
  (`wasamoc` lower / emit; runtime parse / validate / static member
  append). The schema change, `wasamoc` emit/lower, textual IR parser,
  validators, and static load-time presence reducer were bundled in one
  buildable implementation commit per the T4 R-A/R-B risk note.
- **2026-06-03 / T4 review follow-up traversal audit:** owner review
  found two places where the initial `widget_children()` split silently
  changed semantics: phase-specific runtime validators did not descend
  into `ControlFlow` bodies, and ZStack placement vectors were built from
  widget-only declared children while materialised children came from
  static `IrMember` expansion. The follow-up fixes both by dispatching
  phase validators through `IrMember`, building ZStack placements through
  `collect_static_zstack_placements` with the same load-time condition
  evaluation as `append_static_member`, and rejecting direct runtime
  `ControlFlow` under `Grid` / `Cell` because those IR-only wrappers are
  flattened by a Grid-specific build path. This amends the T4
  retrospective's original "no new constraint" statement: declared-member
  traversal that affects validation or positional metadata must dispatch
  `IrMember`; widget-only helpers are valid only when dropping
  `ControlFlow` is explicitly part of the invariant.
- **2026-06-03 / T4 review follow-up #2 — `Vec<IrMember>` traversal
  call-site audit (semantic-migration audit):** a second review pass
  produced the explicit traversal-contract audit the original migration
  should have carried. Each production `IrMember`-bearing traversal was
  classified `must-dispatch` / `ignore-OK (+ proof)` /
  `defer-with-approval`; every `ignore-OK` carries a reject test or an
  impossibility note (the bar that makes the no-constraint claim
  falsifiable).
  - **`must-dispatch ✓` (already correct):** control-flow shape
    (`validate_phase6_control_flow_invariants`); the Phase 2/3/4/5/ZStack
    validator `*_member_invariants` body recursions; `validate_node_references`
    → `validate_member_references` (condition validation); the non-Grid
    build append (`append_static_member`); ZStack placements
    (`collect_static_zstack_placements`); `wasamoc` `lower` / `emit`
    member dispatch. Evidence: `validate_rejects_*`,
    `zstack_static_placements_follow_materialized_member_order`,
    `conditional_lowers/emitted_*`.
  - **`ignore-OK` (ControlFlow legitimately dropped):** Grid/Cell build
    + validate widget-only iterations — proof: `Grid`/`Cell` reject all
    direct `ControlFlow` upstream (`validate_rejects_direct_conditional_{grid,cell}_member`),
    so no `ControlFlow` reaches those sites. `WidgetNode.children` walks
    in `widget.rs` — impossibility note: the materialised widget tree has
    no `ControlFlow` variant (it is expanded at build).
  - **FINDING (was mis-classified `ignore-OK`, corrected to
    `must-dispatch`):** the **Box at-most-one** and **ScrollView
    exactly-one** child-count gates counted `widget_children()` only, so a
    conditional sibling (`Box { Content  if c }` / `ScrollView { Content
    if c }`) under-counted and slipped past **both** `wasamoc check` and
    runtime `validate()`, materialising two children. This is the same
    widget-only-vs-materialised root as the review-#1 findings; review #1
    fixed validator *descent* but left the *count basis* widget-only.
    Fixed: Box counts every child member (`node.children.len()`;
    `WidgetDecl | Conditional` at check); ScrollView rejects any direct
    conditional member (interim, symmetric with Cell). The
    conditional-only ScrollView case (`ScrollView { if c { … } }`) is left
    rejected pending **DD-M3-P6-007** (the conditionally-empty-container
    relaxation is a Phase 6 design decision, owner-gated).
  - **Rule candidate (carry-forward, not yet ruled):** "any traversal
    that validates declared structure, computes positional metadata, or
    materialises declared members must dispatch on `IrMember` unless it
    has a documented, tested widget-only invariant; prefer compile-error-
    forcing mechanisms (exhaustive `match`, no `Default`) over
    silent-absorb helpers (filtering iterators)." Precedent: the
    `kind_payload` migration (DD-M3-P5-001) used no-`Default` to force
    construction-site compile errors (success); the `IrMember` filtering
    helper bypassed that discipline (this failure). Recorded as a
    handoff carry-forward; rule-ification (workflow.md / retrospectives.md
    + a vision decision record) deferred to the next semantic migration so
    the rule is designed against ≥ 2 samples, not over-fit to one.
- **2026-06-03 / T3 skip-guard disposition:** ZStack live Visual
  integration introduces no new runtime capability path beyond the
  existing `wasamo_init` → Compositor creation surface. The
  `init_runtime_or_skip` guard in
  `wasamo-runtime/tests/zstack_layout_integration.rs` therefore reuses
  the Phase 5 Grid pattern byte-for-byte in behavior: local
  `0x80070005` returns `None` (developer-laptop skip), while GitHub
  Actions fails rather than silently skipping. This records the
  inheritance disposition requested by T3 instead of re-proving the
  already inherited missing-Compositor path.
- **2026-06-03 / T3 VisualCollection evidence seam:** The ZStack live
  Visual-order fixture needs to enumerate `VisualCollection`; the
  runtime crate's existing `windows` dependency now enables the
  `Foundation_Collections` feature so the test can read the live child
  collection directly. This is an API-feature enablement for the
  existing dependency, not a new build system / CI surface.
- **2026-06-04 / T4b DD-M3-P6-007 accepted (a):** the open ScrollView
  conditional-content question closed **(a) — reject a direct conditional
  member; defer conditionally-empty content**, after a multi-pass
  design-decision review (strategic / recommendation-choice /
  implementation-readiness). Doc/process only: DD-007 `Proposed → Accepted`,
  preamble §Decisions index (+ Revisions), `docs/dsl_spec.md` §4.11 sentence
  + §4.14 diagnostics row, plan.md T4b. **No code change** — the T4
  review-follow-up interim is the final rule, so the existing dual-gate
  tests (`scrollview_conditional_member_rejected` /
  `scrollview_conditional_only_member_rejected`;
  `validate_rejects_scrollview_with_conditional_member` /
  `validate_rejects_scrollview_with_conditional_only_member`,
  `IrLoadError::Validate` → `WASAMO_ERR_IR_MALFORMED`) are the final
  evidence. The review found and corrected a stub citation error
  (ScrollView exact-one = DD-M3-P4-001, not DD-M3-P4-003).
  - *Deferred (low harm, not a checklist item):* `ir_loader.rs`
    `validate_phase4_node_invariants` still narrates the rejection as the
    "interim / open DD-M3-P6-007 ... until that is decided" state. Refresh
    that provenance comment to "accepted (a); conditionally-empty direction
    deferred" at the **next `ir_loader.rs` touch (T5)** or phase-end — left
    now to keep T4b a code-no-touch close; harm is low (the comment still
    links DD-M3-P6-007, and behaviour / diagnostic / public spec are
    correct).

---

## CI / verification log

- **2026-06-07 / T6 local verification:** scoped checks green —
  `cargo test -p wasamo-runtime --lib static_window_title` (2 tests),
  `cargo test -p wasamo-runtime --test abi_load_ui` (1 test), and
  `cargo test -p wasamo-runtime --test window_title_integration` (1 test).
  Final clean-rebuild gate green: `cargo fmt --all -- --check`;
  `cargo clean` (`4329 files, 1.4GiB` removed);
  `cargo build --release --workspace` (37.12s);
  `cargo build --workspace` (35.67s); `cargo test --workspace` (included the
  new `window_title_integration` fixture). Existing Cargo warnings about the
  `wasamo` linkable target / `wasamo-sys` import-library ordering were
  observed.
- **2026-06-07 / T6 skip-guard inheritance disposition (review follow-up):**
  the new `window_title_integration` Windows-runtime fixture reuses the
  shared `run_on_owning_runtime_thread_or_skip` entry point byte-identically;
  it introduces **no new runtime-capability path**. That helper enforces the
  CI skip-guard policy itself: on `Runtime::CompositorUnavailable` it asserts
  `!github_actions()`, so the test **fails on GitHub Actions** when the
  Compositor cannot be created and only skips on a local dev box without a
  usable session (`wasamo_init` → `0x80070005`). This inherits the T3 / T5
  disposition (Phase 4 / 5 pattern) — no separate skip-guard verification was
  needed because the fail-on-CI branch lives in the reused helper, not in a
  T6-authored guard.
- **2026-06-07 / T6 example-host title observation (review follow-up):** the
  T6 runtime change flips every host's window title from the
  `DEFAULT_WINDOW_TITLE` (`"Wasamo"`) to the DSL-declared
  `title: "Counter"`, so the three counter example READMEs were re-asserted
  as a positive observable rather than left as the prior "title is dropped"
  caveat. All three were built and launched and their live window title read
  back (`Process.MainWindowTitle` / Win32 `GetWindowTextW` on the launched
  process's HWND): **counter-rust** (`cargo build -p counter-rust`),
  **counter-c** (CMake / MSVC 19.51, Release, `wasamoc.exe` custom build
  step), and **counter-zig** (`zig build` 0.16.0, ReleaseSafe, `@embedFile`)
  each reported `"Counter"` (not `"Wasamo"`) with a live HWND. The three
  build paths differ but share the runtime `wasamo_load_ui` →
  `resolve_static_window_title` → `window::create` seam. No dedicated
  positive control was required: the observed value `"Counter"` directly
  falsifies the only realistic wrong-implementation output (`"Wasamo"`), and
  the mechanism's input-varied discrimination was already proven by the
  `static_component_title_reaches_native_window` Gallery fixture
  (`"Gallery"` ≠ `"Wasamo"`). This is a title-bar (DWM / HWND-state)
  observation only; in-window content smoke remains T7 (assistant
  screenshot) / T8 (owner) on the gallery slice.
- **2026-06-07 / T7 local verification:** `cargo run -p wasamoc -- check
  examples\gallery\gallery.ui` — green; `cargo build --release -p
  gallery-rust` — green; escalated assistant GUI capture
  `process\milestone-3\phase-6\implementation\evidence\capture-lightbox.ps1`
  — green, saving the closed/open/closed-after-click triplet under
  `implementation/evidence/`. Targeted runtime: `cargo fmt --all --
  --check` — green; `cargo test -p wasamo-runtime --lib zstack` — green
  (17 tests, including the new root-ZStack/window-prop and child-ZStack
  placement validator tests); `cargo build --release --workspace` — green.
  First `cargo test --workspace` hit the known debug import-library ordering
  race (`wasamo-sys` warning followed by `wasamo-dll` LNK1356:
  `target\debug\libwasamo_runtime.rlib` missing). Disposition: this matches
  the pre-existing DD-M2-P1-006 ordering race already recorded in prior Phase
  6 clean rebuilds, not a T7 regression; after `cargo build --workspace`
  created the debug import library, `cargo test --workspace` reran green
  (wasamo-runtime 338 unit tests, wasamoc 316 unit tests, wasamo-ir 17, all
  integration suites and doc-tests green). Existing Cargo warnings about the
  `wasamo` linkable target / `wasamo-sys` import-library ordering were
  observed.
- **2026-06-07 / T7 task-end clean rebuild:** `cargo fmt --all -- --check`
  — green; `cargo clean` completed (`3090 files, 1.1GiB` removed);
  `cargo build --release --workspace` — green; `cargo build --workspace` —
  green; `cargo test --workspace` — green (wasamo-runtime 338 unit tests,
  wasamoc 316 unit tests, wasamo-ir 17, all integration suites and doc-tests
  green). Existing Cargo warnings about the `wasamo` linkable target /
  `wasamo-sys` import-library ordering were observed.
- **2026-06-05 / Observation 5 remediation step 1 — local gate + GitHub
  Actions CI (branch `test/obs5-step1-marshal-owning-thread`, commit
  `4d2cb3e`):** local clean-rebuild gate green — `cargo fmt --all -- --check`
  green; `cargo clean` (3764 files, 1.2GiB removed); `cargo build --release
  --workspace` green (40.6s); `cargo build --workspace` green;
  `cargo test --workspace` green. (A direct `cargo test --workspace` straight
  from `cargo clean` first hit the known LNK1356 `wasamo-sys → wasamo-dll`
  `/WHOLEARCHIVE` ordering race (DD-M2-P1-006); building the workspace first,
  as CI does, then testing was green — not a regression from this change.)
  Targeted: full `wasamo-runtime` suite (333 unit + all integration) green
  under `--test-threads=1`, the form that previously crashed deterministically
  with `0xC0000005`. **Positive control:** a temporary thread-identity probe
  showed the marshalled test body running on `wasamo-test-runtime-owner` while
  its caller ran on the libtest thread named after the test — distinguishing
  real owning-thread execution from a no-op wrapper (which would have printed
  the same name twice); the probe was reverted before commit. **GitHub Actions
  CI:** run
  [27014203528](https://github.com/matarillo/wasamo/actions/runs/27014203528)
  (`workflow_dispatch` on the branch, headSha `4d2cb3e`) — conclusion
  **success** (~3m8s); `Test (workspace)` and all binding / example smoke
  steps green on the windows-latest runner, confirming the executor-thread
  marshalling works on the actual CI runner (default multi-threaded
  `cargo test --workspace`). Existing Cargo warnings about the `wasamo`
  linkable target / `wasamo-sys` import-library ordering were observed.
- **2026-06-05 / Observation 5 remediation step 2 — task-end clean rebuild
  (branch `investigate/obs5-scrollview-teardown-av`, post-commits
  `02ff614`, `a304dc5`, `83aadb7`):** `cargo fmt --all -- --check` — green
  (post-commit state); `cargo clean` completed (`5067 files, 1.4GiB`
  removed); `cargo build --release --workspace` — green (44.4s);
  `cargo build --workspace` — green (41.3s); `cargo test --workspace` —
  green on the **first** run (23s; `wasamo-runtime` lib 333, `wasamoc` 316,
  `wasamo-ir` 17, all integration suites, 0 failed). The process-exit
  access violation that forced a `--workspace` rerun at the T5 follow-up
  clean rebuild did **not** recur: the keep-alive apartment helper
  (`wasamo-runtime/tests/common/mod.rs`) keeps `dcomp.dll` resident for the
  whole test binary. Positive control: before the fix, `scroll_view` /
  `wrap_panel` / `grid` crashed 5/5 · 3/3 · 3/3 under `--test-threads=1`;
  after, the full `wasamo-runtime` suite is green under `--test-threads=1`
  as well. Existing Cargo warnings about the `wasamo` linkable target /
  `wasamo-sys` import-library ordering were observed.
- **2026-06-05 / T5 follow-up clean rebuild (post-commits `cc5d130`,
  `35c2d88`, `f7a2281`):** `cargo clean` completed (`5311 files,
  1.4GiB` removed); `cargo fmt --all -- --check` — green;
  `cargo build --release --workspace` — green (57.88s);
  `cargo build --workspace` — green (47.89s). First
  `cargo test --workspace` run hit a `scroll_view_layout_integration`
  process-exit access violation **after individual assertions had passed**
  (the fault is in COM/Compositor teardown at process exit, not in the
  asserted ScrollView behaviour); the three ScrollView integration tests
  were rerun individually and were green, and the subsequent
  `cargo test --workspace` rerun was green (`wasamo-runtime` lib 333,
  `wasamoc` 316, `wasamo-ir` 17, integration suites all green, 0 failed).
  This matches the **same teardown AV recorded in Phase 5 T1**
  ([phase-5/t1.md](../../phase-5/retrospectives/t1.md)), where the diff was
  `wasamoc`-only and never touched the insertion path — so it is
  diff-independent and not a T5 regression (T5's `append_child` delegation
  is behaviour-identical for ScrollView). It is **not** dismissed as a mere
  flake: the known-issue disposition (hypotheses + "capture a minidump on
  next occurrence rather than re-rolling"; production teardown defect not
  yet excluded) is recorded as
  [docs/notes/verification-environments.md Observation 5](../../../../docs/notes/verification-environments.md)
  and carried forward below. Existing Cargo warnings about the `wasamo`
  linkable target / `wasamo-sys` import-library ordering were observed.
- **2026-06-05 / T5 local scoped:** `cargo test -p wasamo-runtime --test
  conditional_toggle_integration` — green (2 tests). Added
  `conditional_toggle_preserves_declared_visual_order_and_disposes_registry`
  for declared sibling order, two sibling conditionals, preceding removal
  index shift, true→true / false→false no-op, live VisualCollection order,
  and registry teardown through `widget_destroy`; added
  `conditional_toggle_drains_fresh_subtree_effects_before_return` for
  same-drain present/absent observation and freshly-created subtree Effects
  observing the latest state before the toggling setter returns.
- **2026-06-05 / T5 local scoped runtime:** `cargo test -p wasamo-runtime
  --lib ir_loader::tests` — green (127 tests);
  `cargo test -p wasamo-runtime --lib reactive::tests` — green (39 tests);
  `cargo test -p wasamo-runtime` — green (runtime lib 332 plus all
  integration suites, including the new conditional toggle fixture).
- **2026-06-05 / T5 local pre-retro:** `cargo fmt --all -- --check` —
  green; `cargo build --release --workspace` — green; `cargo build
  --workspace` — green; `cargo test --workspace` — green; `cargo test -p
  wasamo-runtime` — green. Existing Cargo warnings about the `wasamo`
  linkable target / `wasamo-sys` import-library ordering were observed.
- **2026-06-03 / T4 local scoped:** `cargo fmt --all -- --check`
  — green; `cargo test -p wasamo-ir` — green (17 tests);
  `cargo test -p wasamoc --lib` — green (308 tests);
  `cargo test -p wasamo-runtime --lib` — green (322 tests).
  Covered `IrMember` schema encoding, control-flow keyword / parser /
  check / lower / emit diagnostics, runtime textual IR parsing /
  roundtrip, and static conditional presence / validator rejection
  evidence.
- **2026-06-03 / T4 local pre-commit:** `cargo build --release
  --workspace` — green; `cargo build --workspace` — green;
  `cargo test --workspace` — green; `cargo test -p wasamo-runtime`
  — green (runtime lib 322 plus integration tests). Existing Cargo
  warnings about the `wasamo` linkable target were observed.
- **2026-06-03 / T4 task-end clean rebuild (post-commit
  `774b567`):** `cargo fmt --all -- --check` — green; `cargo clean`
  completed (`4862 files, 1.3GiB` removed); `cargo build --release
  --workspace` — green; `cargo build --workspace` — green;
  `cargo test --workspace` — green; `cargo test -p wasamo-runtime`
  — green (runtime lib 322 plus integration tests). Existing Cargo
  warnings about the `wasamo` linkable target / `wasamo-sys`
  import-library ordering were observed.
- **2026-06-03 / T4 review follow-up local:** `cargo test -p
  wasamo-runtime --lib ir_loader::tests` — green (122 tests);
  `cargo test -p wasamoc --lib check::tests::conditional` — green
  (11 tests); `cargo test -p wasamoc --lib` — green (310 tests);
  `cargo test -p wasamo-runtime --lib` — green (327 tests). Covered
  Grid direct conditional diagnostics, literal-condition diagnostics,
  runtime bool-read/non-bool condition rejection, ControlFlow-body
  validator descent, Grid/Cell direct-ControlFlow runtime rejection, and
  ZStack static placement order matching materialised member order.
  Final follow-up verification: `cargo fmt --all -- --check` — green;
  `cargo build --workspace` — green; `cargo test --workspace` —
  green; `cargo test -p wasamo-runtime` — green (runtime lib 327 plus
  integration tests). Existing Cargo warnings about the `wasamo`
  linkable target were observed.
- **2026-06-03 / T4 Cell conditional check follow-up local:**
  added the source-level dual gate for `Cell { <widget> if ... }`,
  matching the runtime `validate_rejects_direct_conditional_cell_member`
  defense-in-depth rejection. `cargo test -p wasamoc --lib
  check::tests::conditional_cell_sibling_rejected` — green; `cargo test
  -p wasamoc --lib check::tests::conditional` — green (12 tests);
  `cargo test -p wasamoc --lib` — green (311 tests); `cargo fmt --all
  -- --check` — green.
- **2026-06-03 / T4 review follow-up #2 — Box/ScrollView count fix
  (clean rebuild):** the `Vec<IrMember>` traversal call-site audit fix.
  Box (`node.children.len()`; `WidgetDecl | Conditional` at check) and
  ScrollView (reject direct conditional member, interim) single-child
  gates now count a conditional sibling at both `wasamoc check` and
  runtime `validate()`. Added tests
  `box_widget_and_conditional_sibling_rejected`,
  `box_conditional_only_child_accepted`,
  `scrollview_conditional_member_rejected` (`wasamoc`),
  `validate_rejects_box_with_widget_and_conditional_sibling`,
  `validate_accepts_box_with_conditional_only_child`, and
  `validate_rejects_scrollview_with_conditional_member` (runtime).
  `cargo fmt --all -- --check` — green; `cargo clean` completed
  (`5038 files, 1.3GiB` removed); `cargo build --release --workspace`
  — green (46.84s); `cargo build --workspace` — green (41.02s);
  `cargo test --workspace` — green (`wasamoc` 314, `wasamo-runtime` lib
  330, `wasamo-ir` 17, integration suites all green, 0 failed). Existing
  Cargo warnings about the `wasamo` linkable target / `wasamo-sys`
  import-library ordering were observed.
- **2026-06-03 / T4 review follow-up #2 — Codex review additions (clean
  rebuild):** Codex re-review returned no blocker; one should-fix (pin the
  DD-M3-P6-007 centre case `ScrollView { if c { … } }` directly) and one
  nit (a multiple-conditional-sibling Box reject as the shortest
  `children.len()` count-basis proof). Added
  `scrollview_conditional_only_member_rejected`,
  `box_multiple_conditional_siblings_rejected` (`wasamoc`),
  `validate_rejects_scrollview_with_conditional_only_member`, and
  `validate_rejects_box_with_multiple_conditional_siblings` (runtime).
  `cargo fmt --all -- --check` — green; `cargo clean` completed
  (`3150 files, 1.1GiB` removed); `cargo build --release --workspace`
  — green (44.08s); `cargo build --workspace` — green (35.29s);
  `cargo test --workspace` — green (`wasamoc` 316, `wasamo-runtime` lib
  332, `wasamo-ir` 17, integration suites all green, 0 failed). Existing
  Cargo warnings about the `wasamo` linkable target / `wasamo-sys`
  import-library ordering were observed.
- **2026-06-02 / T1 local:** `cargo fmt --all -- --check` — green.
- **2026-06-02 / T1 local:** `cargo test -p wasamoc` — green;
  covered the ZStack check / lower / emit evidence with tests including
  `zstack_known_widget_no_warning`,
  `zstack_direct_child_alignment_accepted`,
  `zstack_unknown_attribute_rejected`,
  `zstack_reserved_layering_attribute_rejected`,
  `zstack_grid_track_attribute_rejected`,
  `zstack_child_bad_alignment_value_rejected`,
  `placement_attr_outside_zstack_child_or_cell_rejected`,
  `placement_attr_on_zstack_itself_rejected_with_container_position`,
  `zstack_lowers_as_direct_children_without_kind_payload`, and
  `zstack_emitted_as_node_with_direct_children_in_order`.
- **2026-06-02 / T1 local:** `cargo clippy -p wasamoc` — green.
- **2026-06-02 / T1 task-end clean rebuild:** `cargo clean`
  completed (`2993 files, 1012.3MiB` removed);
  `cargo build --release --workspace` green; `cargo build --workspace`
  green; `cargo test --workspace` green. Existing Cargo warnings about
  the `wasamo` linkable target / `wasamo-sys` import-library ordering
  were observed.
- **2026-06-02 / T2 local:** `cargo fmt --all -- --check` — green.
- **2026-06-02 / T2 local:** `cargo test -p wasamoc` — green.
- **2026-06-02 / T2 local:** `cargo test -p wasamo-runtime zstack` —
  green; added pure-logic ZStack layout tests
  `zstack_defaults_to_fill_fill_and_centers_children`,
  `zstack_shrink_measure_uses_child_union_with_fill_child_zero`,
  `zstack_arrange_alignment_overrides`, and
  `zstack_arrange_preserves_document_order_substrate`.
- **2026-06-02 / T2 local:** `cargo build --release --workspace` —
  green. Existing Cargo warnings about the `wasamo` linkable target /
  `wasamo-sys` import-library ordering were observed.
- **2026-06-02 / T2 task-end clean rebuild:** `cargo clean` completed
  (`4163 files, 1.1GiB` removed); `cargo build --release --workspace`
  green; `cargo build --workspace` green; `cargo test --workspace`
  green. Existing Cargo warnings about the `wasamo` linkable target /
  `wasamo-sys` import-library ordering were observed.
- **2026-06-02 / T2 review follow-up local:** tightened the
  `zstack_arrange_preserves_document_order_substrate` evidence so the two
  children have distinguishable overlapping geometry, corrected the T2
  retrospective's limited helper-rename classification, and renamed
  `align_in_rect` parameters from cell-specific to rect-specific names.
  `cargo fmt --all -- --check` green; `cargo test -p wasamo-runtime
  zstack` green (4 passed); `cargo build` green. Clean follow-up
  verification: `cargo clean` completed (`3707 files, 1.1GiB` removed);
  `cargo build --release --workspace` green; `cargo build --workspace`
  green; `cargo test --workspace` green. Existing Cargo warnings about
  the `wasamo` linkable target / `wasamo-sys` import-library ordering
  were observed.
- **2026-06-03 / T3 local scoped:** `cargo fmt --all -- --check` —
  green after formatting; `cargo test -p wasamo-runtime zstack` —
  green. Covered runtime validate tests
  `zstack_positive_control_validates_direct_children`,
  `zstack_attribute_rejected_at_validate`,
  `zstack_binding_rejected_at_validate`,
  `zstack_child_unknown_alignment_rejected_at_validate`,
  `placement_prop_outside_zstack_child_or_grid_cell_rejected_at_validate`,
  and `validate_rejects_zstack_with_kind_payload`; roundtrip test
  `zstack_emit_then_parse_preserves_direct_children_and_order`; live
  Visual fixtures
  `zstack_rooted_fixture_preserves_live_visual_order_and_clip` and
  `zstack_vstack_root_fixture_pins_production_root_shape`.
- **2026-06-03 / T3 local pre-commit:** `cargo test -p wasamo-runtime`
  — green (included the new ZStack live Visual fixtures, plus existing
  Grid / ScrollView / WrapPanel integration coverage); `cargo build
  --release --workspace` — green; `cargo build --workspace` — green;
  `cargo test --workspace` — green. Existing Cargo warnings about the
  `wasamo` linkable target / `wasamo-sys` import-library ordering were
  observed.
- **2026-06-03 / T3 task-end clean rebuild (post-commit
  `63d6262`):** `cargo fmt --all -- --check` — green; `cargo clean`
  completed (`7195 files, 2.2GiB` removed); `cargo build --release
  --workspace` — green; `cargo build --workspace` — green; `cargo test
  --workspace` — green. Existing Cargo warnings about the `wasamo`
  linkable target and `wasamo-sys` import-library ordering were observed.
- **2026-06-03 / T3 review follow-up local:** pinned empty ZStack as a
  valid runtime shape (`zstack_zero_children_validates`) and strengthened
  the live Visual fixture so the aligned child's `Visual.Offset` proves
  `h-align: end` / `v-align: start` through the runtime
  `WidgetData::ZStack` → `LayoutNode::zstack` boundary. `cargo fmt
  --all -- --check` passed after formatting; `cargo test -p
  wasamo-runtime zstack` — green.
- **2026-06-03 / T3 review follow-up clean rebuild (post-commit
  `395da0f`):** `cargo fmt --all -- --check` — green; `cargo clean`
  completed (`3755 files, 1.2GiB` removed); `cargo build --release
  --workspace` — green; `cargo build --workspace` — green; `cargo test
  --workspace` — green. Existing Cargo warnings about the `wasamo`
  linkable target and `wasamo-sys` import-library ordering were observed.
- **2026-06-03 / T1+T2 cross-task review follow-up clean rebuild
  (post-commit `4616e48`):** a T1/T2 re-review on the test-breadth and
  cross-phase-constraint lenses pinned three deliberate diagnostic/size
  branches that had no test —
  `zstack_child_non_keyword_alignment_value_rejected` (T1 `wasamoc`
  `check_zstack_child_align` non-identifier arm),
  `zstack_handler_rejected_at_validate` (T3 runtime `validate` ZStack
  handler arm), and
  `zstack_fixed_size_measure_reports_declared_extent_not_child_union`
  (T2 `measure_zstack` `Fixed` size arm) — and corrected t1.md item 5 /
  10 and t2.md item 10 to record the placement/alignment constraint as a
  single `carry-forward` with three implementation sites. `cargo fmt
  --all -- --check` — green; `cargo clean` completed (`4935 files,
  1.3GiB` removed); `cargo build --release --workspace` — green
  (39.27s); `cargo build --workspace` — green (35.80s); `cargo test
  --workspace` — green (`wasamoc` 293, `wasamo-runtime` lib 314).
  Existing Cargo warnings about the `wasamo` linkable target and
  `wasamo-sys` import-library ordering were observed. This is the single
  SSOT record for the follow-up verification; the t1 / t2 / t3 retro
  item-3 sections stay scoped to their own original commits.
