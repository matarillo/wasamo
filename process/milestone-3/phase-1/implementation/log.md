## Decisions log

- **T11 host choice (2026-05-19):** Added a dedicated
  `examples/bool-demo-rust/` host instead of extending
  `examples/counter-rust`. Rationale: `counter-rust` remains the M2
  Hello Counter reference, while T11's bool proof gets a fixture whose
  behavior is exactly the Phase 1 closure item (`Button.enabled`
  driven by `state ready: bool` and disabled by the button's own click
  handler).
- **T13 retroactive insertion (2026-05-19):** Owner review at T12
  phase-end surfaced that the original T6 acceptance bullet had been
  reading the
  [m3-phase-1-bool-scalar.md ADR §Verification item 3](../../decisions/m3-phase-1-bool-scalar.md#verification-strategy)
  as discharged by `button_enabled.rs`, but that test bypasses the
  binding pipeline by design. T13 was added to discharge the actual
  `.ui → load → click → state → bound widget property` chain that
  ADR item 3 requires. CLAUDE.md §Commit rules permits this kind of
  retroactive task insertion ("implementation reveals that an item
  should be split"); the deviation is recorded here rather than the
  task list being silently rewritten.
- **Finding 2 resolution (2026-05-19):** Owner chose option (a) —
  amend m3-plan §Phase-end criteria item 5 with a foundational-phase
  exception clause rather than recording a one-off deviation in the
  retrospective. Rationale: the ADR §Verification item 4 explicitly
  permitted the `bool-demo` substitute path from the start, so the
  plan ↔ ADR mismatch was a plan-side drafting gap (the gallery
  sub-screen criterion implicitly assumed `examples/gallery/` would
  exist before any phase closed). Foundational phase status does not
  recur, so the exception's scope is statically bounded to Phase 1.
  Aligns with `feedback_revise_dont_workaround` (revise the document
  rather than working around it) and `feedback_doc_cost_not_a_factor`
  (plan revision size is not a design factor).

---

## CI / verification log

- **2026-05-19 / T11 local:** `cargo fmt` — green.
- **2026-05-19 / T11 local:** `cargo test -p wasamoc --test roundtrip`
  — green, 6 passed including
  `bool_demo_ui_contains_bool_binding_and_handler`.
- **2026-05-19 / T11 local:** `cargo build -p bool-demo-rust` — green.
- **2026-05-19 / T11 local:** `cargo build --release -p bool-demo-rust`
  — green.
- **2026-05-19 / T11 local GUI smoke:** `Start-Process
  .\target\release\bool-demo-rust.exe` — command succeeded. Manual
  visible-window smoke was owner-confirmed. Full workspace release
  build, full workspace tests, and CI run remain T12 phase-end gates.
- **2026-05-19 / T12 local fmt drift fix:** `cargo fmt --all --
  --check` surfaced rustfmt drift in `wasamo-runtime/src/{emit,
  reactive,widget}.rs`, `wasamo-runtime/tests/button_enabled.rs`,
  `wasamoc/src/{check,lower}.rs`. `cargo fmt --all` applied; fmt-only
  commit `1129aea`. Re-run `cargo fmt --all -- --check` — green.
- **2026-05-19 / T12 local clean rebuild:**
  - `cargo clean` — green (`Removed 3834 files, 973.9MiB total`).
  - `cargo build --release --workspace` — green (`Finished
    `release` profile [optimized] target(s) in 43.73s`).
  - `cargo build --workspace` — green (`Finished `dev` profile
    [unoptimized + debuginfo] target(s) in 38.14s`).
  - `cargo test --workspace` — green. `wasamo-ir` 7 unit, `wasamoc`
    98 unit + 6 roundtrip, `wasamo-runtime` 165 unit + 8 integration
    (incl. `abi_load_ui`, `button_enabled` Windows-only live test,
    `ir_loader_roundtrip` 5, `live_widgetnode_headless`), `wasamo-sys`
    1 unit, plus host crates with 0 tests. No failures, no ignored.
  - Known warnings unchanged: `wasamo` crate "provides no linkable
    target" notice and `wasamo-sys` import-library ordering note
    (DD-M2-P1-006-era; not build/test failures).
- **2026-05-19 / T12 CI:** `workflow_dispatch` run
  [26094510225](https://github.com/matarillo/wasamo/actions/runs/26094510225)
  on `feat/m3-phase-1` — **green** (cargo build job: success). This
  run includes both T6's `button_enabled` and T13's
  `bool_binding_live_propagation` mock-free Windows integration tests
  in `cargo test --workspace`, so it serves as the CI inclusion link
  for both T12 and T13.
- **2026-05-19 / T13 local:** `cargo build -p wasamo-runtime --tests` —
  green after a one-line refactor of the `read_bool_property` helper
  (`unsafe { value.payload.v_bool } != 0` is parsed as
  `unsafe-block; != 0` rather than as the comparison; bound the union
  read into a local first).
- **2026-05-19 / T13 local:** `cargo test -p wasamo-runtime --test
  bool_binding_live_propagation` — green, 1 passed (the new
  `bool_binding_propagates_state_write_through_inline_handler_to_widget_property`).
- **2026-05-19 / T13 local:** `cargo fmt --all -- --check` — initial
  diff in the new test file; `cargo fmt --all` applied; re-run green.
- **2026-05-19 / T13 local:** `cargo test --workspace` — green.
  `wasamo-runtime` integration count rose from 8 → 9 (added
  `bool_binding_live_propagation`); other crates unchanged
  (`wasamo-ir` 7 unit, `wasamoc` 98 unit + 6 roundtrip,
  `wasamo-runtime` 165 unit + 9 integration, `wasamo-sys` 1 unit).
  No failures, no ignored.
- **2026-05-19 / T13 CI:** folded into the same
  `feat/m3-phase-1` `workflow_dispatch` run as T12 —
  [26094510225](https://github.com/matarillo/wasamo/actions/runs/26094510225)
  green.
- **2026-05-19 / phase-close doc-edit step:** Findings 1–4 closed.
  - m3-plan.md §Phase-end criteria item 5: added foundational-phase
    exception clause (Finding 2 / option (a)).
  - phase-end retrospective: §Current Judgment rewritten to reflect
    findings closure; §Checklist 11 A9 evidence anchored to T13 with
    T6 widget-setter slice as auxiliary; §Checklist 16
    (human-visible GUI smoke) added as required and completed through
    the Phase 1 host; original §16 (CI YAML sanity check) renumbered
    to §17 (Findings 1 / 3).
  - progress file frontmatter: `status: active` →
    `status: closing` + `closing: 2026-05-19` (Finding 4); T13 box 4
    and T12 "Phase-end retrospective entry added" checkbox ticked;
    Owner-review follow-ups section heading updated to "(closed at
    T12 phase-end)".
  - No code changes in this step; only doc/spec/plan synchronization
    per CLAUDE.md §Document categories. `cargo test --workspace` not
    re-run because no source files changed.
- **2026-05-19 / T14 local:** `cargo fmt --all -- --check` — green.
- **2026-05-19 / T14 local:** `cargo test -p wasamoc` — green. 98 unit
  tests, 6 roundtrip tests, and 0 doc tests passed. Added
  `check::tests::bool_state_in_string_interp_rejected` and removed the
  obsolete lowering test that expected bool interpolation to lower to
  `BoolPropRead`.
- **2026-05-19 / T14 local clean rebuild:**
  - `cargo clean` — green (`Removed 3317 files, 919.2MiB total`).
  - `cargo build --release --workspace` — green (`Finished
    `release` profile [optimized] target(s) in 40.03s`).
  - `cargo build --workspace` — green (`Finished `dev` profile
    [unoptimized + debuginfo] target(s) in 34.54s`).
  - `cargo test --workspace` — green. No failures, no ignored tests.
    Counts: `wasamo-ir` 7 unit, `wasamoc` 98 unit + 6 roundtrip,
    `wasamo-runtime` 165 unit + 9 integration, `wasamo-sys` 1 unit,
    host/binding crates and doc tests with 0 tests where present. Known
    warnings unchanged: `wasamo` crate "provides no linkable target"
    notice and `wasamo-sys` import-library ordering note.
- **2026-05-19 / remaining follow-ups doc close:** Follow-up A resolved
  by documenting that `Button.enabled`'s internal property key remains
  outside the public experimental ABI in `docs/abi_spec.md`; Follow-up B
  resolved by adding the synchronous non-batched drain proof contract to
  `docs/notes/m2-to-m3-handover.md` §3 item 4, with
  `docs/notes/m3-phase-2/predoc-inputs.md` §9 kept as a back-pointer.
  No code changes; tests not re-run.
- **2026-05-19 / post-T14 + follow-up A/B + retro/CHANGELOG fold-in CI:**
  `workflow_dispatch` run
  [26100232039](https://github.com/matarillo/wasamo/actions/runs/26100232039)
  on `feat/m3-phase-1` (HEAD `6c97459`) — **green**
  (conclusion=success, 2m34s). First CI run on top of the post-T12
  commits: T14 (`fix(m3-phase-1): reject bool string interpolation`,
  code change introducing
  `wasamoc::check::tests::bool_state_in_string_interp_rejected`), the
  implicit-constraint follow-up A/B doc closure, the retrospective
  fold-in (`docs(m3-phase-1): fold follow-up A/B closure into
  phase-end retro`), and the CHANGELOG fold-in (`docs(m3-phase-1):
  reflect T13 binding-pipeline evidence and bool surface rule`). The
  T12 / T13 entry above remains the CI evidence for the original
  phase close on HEAD `f6b6d74`; this entry covers the post-T12
  doc-finalization and language-rule tightening commits added on top.
