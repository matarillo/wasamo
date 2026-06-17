## Decisions log

- **2026-06-17 / T9 review remediation 2 — finding #4 (commit-failure
  rollback registry leak) fixed.** Re-reviewing R-2, the second agent
  found a **production bug** the new rollback test exposed: the
  commit-stage failure branch in `mutate_for_loop_subtree`
  (`ir_loader.rs`) cleaned up only the **committed** prefix and `return`ed,
  leaving the **not-yet-committed** staged children (and, in production,
  the faulting child) to a bare `drop`. `WidgetNode` has **no** `Drop`
  impl (verified: `rg` 0 hits) — registry/binding release is
  `widget_destroy → dispose_subtree_bindings` + `registry::remove_for_widget`,
  call-site-only — so a bare drop leaks. The sibling **staging**-failure
  branch already disposes correctly (`for child in staged { widget_destroy }`);
  the commit-failure branch was asymmetric. The original R-2 test did not
  catch it (it observed only tree shape + retained prefix).

  **Fix (production).** In the Err branch, after rolling back + destroying
  the committed prefix tail-first, also `widget_destroy` the remaining
  un-committed staged children (mirrors the staging-failure branch). The
  loop is now `while let … = staged_iter.next()` so the staged iterator
  stays usable in the failure arm. The **faulting child** itself is
  consumed either by `widget_destroy` (test seam) or by `insert_child`'s
  by-value contract (production, on a near-unreachable WinRT failure —
  index always valid, child fresh-unattached); recovering the production
  faulting child would require changing `insert_child`'s signature across
  the conditional / ABI / static callers (a cross-cutting runtime-
  structural change), so it is documented in-code, not done here.

  **Fix (test).** `staged_for_insert_commit_failure_rolls_back_partial_inserts`
  now appends 5 (C…G) and faults the 2nd commit, so **3** children are
  left un-committed (the disposal loop runs over several). Added a
  registry-baseline assert via a new benign read helper
  `ffi::__registry_entry_count_for_test()` (→ `registry::__entry_count_for_test`):
  a fully-rolled-back failed write must leave the registry at its
  pre-write baseline. **Observability limit (honest disposition):** the
  generated `for`-body children are handler-free `Text` (a `for` body
  cannot author a handler — DD-M3-P7-003), so they carry no registry
  entry of their own; the baseline assert therefore guards retained-prefix
  integrity + any entry-bearing child directly, while the leftover-`Text`
  disposal rests on code symmetry with the proven staging-failure branch.

  **Source enumeration.**

  ```text
  rg -n "impl Drop for WidgetNode" wasamo-runtime/src            # 0 hits (no Drop)
  rg -n "for \(_, leftover\) in staged_iter|while let Some\(\(offset, child\)\)|__entry_count_for_test|__registry_entry_count_for_test" wasamo-runtime/src wasamo-runtime/tests/iteration_mutation_integration.rs
  ```

  **Implemented-branch test map (addendum).**

  | Implemented branch / behaviour | Category | Source query / diff cue | Direct test |
  |---|---|---|---|
  | Commit-failure rollback disposes the committed prefix **and** the un-committed leftover staged (no registry leak) | defensive invariant (finding #4) | diff cue `for (_, leftover) in staged_iter.by_ref()` + registry baseline | `staged_for_insert_commit_failure_rolls_back_partial_inserts` (5 appended, fault at 2nd commit, registry == baseline) |
  | Registry entry-count observability | test helper | `rg` hit `__registry_entry_count_for_test` / `registry::__entry_count_for_test` | used by the rollback test's baseline assert |

  **Carry note.** Faulting-child recovery in production (insert_child
  by-value consume-on-failure) is a **narrow, near-unreachable** residual
  documented in-code; it is *not* a new T10 carry by itself, but if a
  future task makes `insert_child` failure reachable on a valid child (or
  changes its signature) it should dispose the faulting child there. No
  owner-less open item.

  **Verification.**

  ```text
  cargo test -p wasamo-runtime --test iteration_mutation_integration   # 8 passed
  cargo test -p wasamo-runtime --lib                                    # 403 passed
  cargo fmt --all -- --check                                            # clean
  cargo test --workspace                                                # green
  cargo build --release -p wasamo-runtime                               # green (fix in non-gated path; seam still cfg-stripped)
  ```

- **2026-06-17 / T9 review remediation — #2 rollback / #3 Clear closed
  in-phase.** A second-agent review of the T9 end gate found two minor
  proof gaps (recorded in the T9 end-gate entry below as findings ① / ②
  before this remediation). The owner ruled both **closed in-phase on the
  T9 branch** (2026-06-17): both are *runtime* test additions that T10
  (doc-close) cannot discharge, so they land here. This updates the
  "T9 adds no Rust" premise — T9 now adds **test-only Rust** (two
  collection tests + a `debug_assertions`-gated fault seam). Production
  semantics are unchanged.

  **Implementation-gates (start + close).** Review lane =
  **branch/test-focused** ([implementation-gates.md §4](../../../procedures/implementation-gates.md);
  trap #4 = directly fire the untested branch). Traps #1/#2/#3 remain
  non-applicable: no enum/IR/schema variant, no new production runtime
  branch (the seam is `#[cfg(debug_assertions)]` test-only and the
  production commit-loop arm is byte-identical), no new parallel/derived
  data. Trap #6 standing. The rollback **contract** (prefix removed /
  tree unchanged / `live_children` not advanced / subsequent append
  succeeds) is held, not relaxed.

  **Source enumeration used for close artifacts.**

  ```text
  git diff --stat -- wasamo-runtime/src/handler.rs wasamo-runtime/src/ir_loader.rs wasamo-runtime/tests/iteration_mutation_integration.rs
  rg -n "collection_assignment_empty_literal_clear|reactive_for_empty_literal_clear_removes_all_then_regrows|staged_for_insert_commit_failure_rolls_back_partial_inserts|__arm_structural_insert_fault_for_test|structural_insert_fault_armed|cfg\(debug_assertions\)" wasamo-runtime/src/handler.rs wasamo-runtime/src/ir_loader.rs wasamo-runtime/tests/iteration_mutation_integration.rs
  ```

  **Implemented-branch test map.**

  | Implemented branch / behaviour | Category | Source query / diff cue | Direct test |
  |---|---|---|---|
  | Empty-literal clear handler atom (`Assign { rhs: ListLit(vec![]) }`) evaluates Ok, empties the collection, and non-empty→empty dirties | semantic branch (R-1a, finding ③) | `rg` hit `collection_assignment_empty_literal_clear`; subcase `ListLit(Vec::new())` + `set_i32_list(.., Vec::new()) == Ok(true)` | `handler::tests::collection_assignment_empty_literal_clear` |
  | `>1 → 0` structural shrink-to-zero (`labels = []`) removes every generated child + releases registry, then grow-from-empty regrows (member live) | size / semantic branch (R-1b, finding ③) | `rg` hit `reactive_for_empty_literal_clear_removes_all_then_regrows`; DESTROY_COUNT==3 then `0 → 1` regrow | `iteration_mutation_integration::reactive_for_empty_literal_clear_removes_all_then_regrows` |
  | Partial-insert rollback: commit-stage failure removes the committed prefix, leaves the tree unchanged, does not advance `live_children`, and recovers | defensive invariant (R-2, finding ①/#2) | `rg` hit `staged_for_insert_commit_failure_rolls_back_partial_inserts`; seam `__arm_structural_insert_fault_for_test`; production branch `for rollback in (0..inserted).rev()` | `iteration_mutation_integration::staged_for_insert_commit_failure_rolls_back_partial_inserts` |

  **Rollback contract — the four asserts (review condition c).**
  (1) tree unchanged: `text_children == ["before","A","B","after"]` +
  `assert_visual_order` (VisualCollection count == children → no orphaned
  Visual) + static-sibling pointers unchanged. (2) prefix removed: the
  committed `C` is gone from children and the VisualCollection;
  `DESTROY_COUNT == 0` proves the retained prefix `A`/`B` is **not**
  destroyed (the within-write-generated rolled-back child has no
  pre-attach window for its own destroy callback, so "no orphan" is
  proven via VisualCollection-count consistency rather than a per-child
  counter — noted for re-review). (3) `live_children` not advanced: a
  later disarmed append plans off `old_len == 2`, yielding exactly one
  child. (4) recovery: the subsequent append succeeds.

  **Seam release-absence evidence (review condition b/c).** The seam
  (`structural_insert_fault_armed`, `FAIL_STRUCTURAL_INSERT_AT`,
  `__arm/__disarm_structural_insert_fault_for_test`) and the rollback
  test are all `#[cfg(debug_assertions)]`. Verified absent from release
  by **compiler check** (spike discipline — a throwaway non-gated probe
  referencing the seam, built then reverted):
  `cargo build -p wasamo-runtime` (dev) compiled the probe;
  `cargo build --release -p wasamo-runtime` failed with
  `error[E0425]: cannot find function __arm_structural_insert_fault_for_test`
  / `__disarm_...` — proving the symbols are cfg-stripped from release.
  Corroborated by `cargo rustc --release -p wasamo-runtime --lib --
  --print cfg` emitting **no** `debug_assertions` and no `[profile.release]`
  / `.cargo/config` override. (A raw `grep` over the release `.rlib`
  *does* match the name in embedded `.rmeta` metadata, but `nm`/object
  symbols and the compiler probe confirm no codegen.) The production
  commit loop's `#[cfg(not(debug_assertions))]` arm is byte-identical to
  the prior direct `insert_structural_child` call.

  **Carry closure (R-3, revise-don't-workaround).** Both carries are now
  **closed**, not deferred: the T9 end-gate carry rows below and the
  earlier T7-remediation rollback rows are updated to point here.
  `plan.md` T10 no longer carries the rollback proof / clear fixture, and
  `t9.md` item 9 is corrected to in-phase close.

  **Verification runs.**

  ```text
  cargo test -p wasamo-runtime --lib                      # 403 passed
  cargo test -p wasamo-runtime --test iteration_mutation_integration   # 8 passed (incl. the 2 new)
  cargo fmt --all -- --check                              # clean
  cargo test --workspace                                  # green
  cargo build --release -p wasamo-runtime                 # green (production path intact)
  ```

  All green on 2026-06-17; the pre-existing `wasamo` linkable-target
  warning is unchanged.

- **2026-06-17 / T9 end gate — owner-manual GUI smoke accepted.** The
  owner ran `target/release/gallery-rust.exe` and captured an 8-frame
  sequence under [evidence/](../evidence/) (`t9-owner-smoke-1-init` …
  `t9-owner-smoke-8-scrolldown`). The owner reported the smoke as
  successful; the assistant independently analysed each frame against the
  per-step claim (the T8-retro GUI-evidence self-falsification step) and
  the positive control holds. T9 adds **no Rust**; the deliverable was
  the runnable host + the owner observation script (in chat) + the plan
  revision. T10 / phase handoff still owns Moment 2 spec re-sync and the
  three deferral-trigger rows; none of those close at T9. (The insert
  partial-failure rollback proof and the empty-`Clear` direct fixture
  were initially carried here as findings ① / #2-#3, then **closed
  in-phase** — see the 2026-06-17 T9 review-remediation entry above.)

  **Source enumeration used for close artifacts.**

  ```text
  git status --short
  git diff --name-only -- "*.rs"        # empty — no Rust branch
  ls process/milestone-3/phase-7/evidence/t9-owner-smoke-*.png   # 8 frames
  ```

  **Owner-observed positive control (frame-by-frame, claim ↔ pixel).**

  | Frame | Action | Pixel observation | Verdict |
  |---|---|---|---|
  | 1 `init` | — | 6 thumbnails `S01 #0`…`S06 #5` | pass |
  | 2 `add-3times` | `Add` ×3 | 9 thumbnails; prefix `S01 #0`…`S06 #5` unmoved + tail `NEW #6` `NEW #7` `NEW #8` (wrap to row 2) | pass — count tracks click, prefix undisturbed |
  | 3 `remove-4times` | `Remove` ×4 | 5 thumbnails `S01 #0`…`S05 #4`; `NEW #6/7/8` and `S06 #5` gone from the tail | pass — tail-first removal |
  | 4 `clear` | `Clear` | 0 thumbnails; generated area empty; no crash | pass — empty-case invariant |
  | 5 `add` | `Add` after clear | 1 thumbnail `NEW #0` (index re-derives from empty) | pass — member still live after clear (strong positive control) |
  | 6 `reset` | `Reset` | 6 thumbnails `S01 #0`…`S06 #5` restored | pass — static-literal reset |
  | 7 `narrowing` | shrink width | Photo WrapPanel reflows 10→5+5; `for`-set reflows to 4/row | pass — WrapPanel reflow |
  | 8 `scrolldown` | `Scroll down (+100)` | `for`-set content scrolls up; top-row labels clip past the ScrollView top; `S05 #4` `S06 #5` become fully visible at the bottom | pass — ScrollView behaviour around the generated set |

  Count trajectory 6 → 9 → 5 → 0 → 1 → 6 tracks every Button click; a
  hardcoded tree cannot make the count follow the click, so this
  distinguishes collection-driven cardinality from a static look-alike
  (the FD-B / ADR item-6 positive control). DPI is acceptable (no
  concerning blur); residual host DPI is the known M4 item, not a Phase 7
  failure.

  **Implemented-branch test map (item 5).**

  | Implemented branch / behaviour | Category | Source query / diff cue | Direct test or owner |
  |---|---|---|---|
  | No new reject / diagnostic / size / semantic **Rust** branch | n/a | `git diff --name-only -- "*.rs"` empty | T9 adds no branch; the for/collection branches are T2–T7-owned and tested there |
  | Owner human-visible GUI smoke (ADR evidence item 6) | GUI evidence (owner-observed) | `evidence/t9-owner-smoke-*.png` (8 frames) | **this task** — owner ran + accepted; assistant frame analysis above |
  | `append` / `drop-last` / static-literal reset render their cardinality change | observable behaviour (owner-visible) | frames 2/3/6; `gallery.ui` handlers `labels.append` / `labels.drop-last` / `labels = ["S01"…]` | runtime contract proven by `wasamo-runtime/tests/iteration_mutation_integration.rs` (T7: `reactive_for_tail_append_reset_remove_preserves_order_and_prefix_identity`); owner smoke shows it human-visibly |
  | **empty-literal `Clear` (`labels = []`, non-empty → empty shrink-to-zero)** | observable behaviour (owner-visible) + direct tests | frame 4; `gallery.ui` handler `labels = []` | **Closed in-phase** (review finding #3): owner frame 4 end-to-end **plus** the now-landed direct tests `handler::tests::collection_assignment_empty_literal_clear` (empty `ListLit` atom) and `iteration_mutation_integration::reactive_for_empty_literal_clear_removes_all_then_regrows` (`>1 → 0` shrink + grow-from-empty). See the 2026-06-17 review-remediation entry above. |
  | WrapPanel reflow + ScrollView behaviour around the generated set | layout invariant (owner-visible) | frames 7/8 | owner observation; underlying WrapPanel / ScrollView are Phase 3/4-owned, unchanged this phase |

  **Behaviour / invariant carry scan (item 6).**

  | Behaviour / invariant | Disposition |
  |---|---|
  | Owner human-visible smoke for collection-driven cardinality. | **Closed in T9.** ADR evidence item (6) owner half discharged; assistant baseline (T8) + owner smoke (T9) both green. |
  | Three deferral-trigger observations (structured-item / `TypedValue`; loop-external indexed read; bindable-`fill`) + Phase 7b owner reservation. | **Carried to T10 handoff.** Not touched by T9. |
  | Insert partial-failure rollback proof (`for rollback in (0..inserted).rev()`). | **Closed in-phase** (R-2; review-remediation entry above). `staged_for_insert_commit_failure_rolls_back_partial_inserts` fires it directly via the `debug_assertions`-gated fault seam. No longer a T10 carry. |
  | **Empty-literal `Clear` direct runtime fixture** (review finding #3). | **Closed in-phase** (R-1; review-remediation entry above). Direct unit (`collection_assignment_empty_literal_clear`) + integration (`reactive_for_empty_literal_clear_removes_all_then_regrows`) tests landed. No longer a T7/T10 carry. |
  | Moment 2 spec/architecture re-sync (gallery slice, four mutation forms, landed diagnostics). | **Owner = T10.** |
  | DPI blur. | **Owner = M4.** Known residual; not a fail criterion. |

  **Carry-forward ownership (item 7).**

  | Carry-forward | Owner task | Scope | Impact | Close condition |
  |---|---|---|---|---|
  | Three deferral-trigger observations + Phase 7b reservation | T10 handoff | Three deferred axes the gallery surfaced; routed M4+ unless Phase 7b is opened | M3 ships the single-attribute slice | T10 records all three + re-triggers + the reservation in `handoff.md` |
  | Insert partial-failure rollback proof | **Closed in-phase (R-2)** | Commit-stage failure rollback branch `for rollback in (0..inserted).rev()` | Was the only defensive branch lacking direct proof | **Done:** `staged_for_insert_commit_failure_rolls_back_partial_inserts` via the `debug_assertions`-gated fault seam (review-remediation entry above) |
  | Empty-literal `Clear` direct runtime fixture (review finding #3) | **Closed in-phase (R-1)** | `collection_assignment_empty_literal_clear` (empty `ListLit` atom) + `reactive_for_empty_literal_clear_removes_all_then_regrows` (`>1 → 0 → re-append`) | Was compositional + owner-smoke only | **Done:** both direct tests landed (review-remediation entry above) |
  | Moment 2 spec/architecture re-sync | T10 | Moment 2 docs | Final doc check for the landed gallery slice + four mutation forms | T10 re-syncs or records no divergence |
  | DPI blur | M4 ([DD-V-022/023](../../../cross-milestone/decisions/dpi-awareness-m4-deferral.md)) | Host DPI-unawareness | Cosmetic residual | M4 DPI work |

  No owner-less residual remains; T9's checklist is green pending the
  owner's explicit merge-gate approval after the retrospective.

  **Verification.** No Rust changed (`git diff --name-only -- "*.rs"`
  empty), so `cargo fmt` / workspace test ownership stays with T10's
  phase-end gates. The host build was re-confirmed green
  (`cargo build -p gallery-rust --release`) before the owner run; the
  pre-existing `wasamo` linkable-target warning is unchanged.

- **2026-06-17 / T9 start gate — owner-manual GUI smoke opened.**
  Started by reading the prior carry-forward rows in this log before
  treating [plan.md](./plan.md) T9 as a hypothesis. T8 closed the
  assistant-visible baseline (6-frame positive control) and explicitly
  left **one** item to T9: the **owner human-visible GUI smoke** — the
  separate gate ADR evidence item (6) names, which the assistant
  screenshot baseline does **not** replace ([CLAUDE.md §Testing
  rules](../../../../AGENTS.md)). T10 owns Moment 2 spec re-sync and the
  three deferral-trigger handoff rows; those do **not** close at T9.

  **Critical re-think of T9's responsibility (plan revised).** T9 is not
  an assistant implementation task: the assistant writes no production
  Rust. Its load-bearing deliverable is (a) a runnable host and (b) a
  detailed owner observation script that **arms the positive control**,
  so the owner can run the smoke and accept or fail it. The one T0-frozen
  divergence found by cross-checking the plan against the mid-phase T8
  owner decision: T8 grew the gallery to **four** body-external mutation
  Buttons (`Add` / `Remove` / `Clear` / `Reset`) on owner request, but
  the T0 plan T9 bullet only named `Add` / `Remove` + "the empty case."
  Revised plan T9 to cover all four authored mutation forms and to fold
  the T8-retro capture-mechanics carry (keep the mutated count legible —
  wider window, count on one row, avoid wrap-fold clip). Verified the
  host builds green (`cargo build -p gallery-rust --release`,
  `target/release/gallery-rust.exe` present).

  **Carry-over checked from prior tasks.**

  | Carry-over | T9 disposition hypothesis |
  |---|---|
  | T7/T8 carry: owner human-visible GUI smoke absent. | **T9 owns.** Owner runs `gallery-rust`, observes the four mutation forms with the collection-mutated positive control, and records acceptance or a fail. |
  | T8 retro → T9 capture-mechanics carry: the `for`-set sits deep in a non-scroll tall VStack inside a `ScrollView`; legible evidence needs a wide window and the mutated count on one row; Button y-coords are member-order-dependent. | **T9 folds into the owner script.** Reuse the 1280-wide window; tell the owner where the four Buttons and the generated set are; capture-script coords re-derive from a recon frame if member order changed (it has not since T8). |
  | T8 carry: three deferral-trigger observations (structured-item / `TypedValue`; loop-external indexed read; bindable-`fill`) with the Phase 7b owner reservation. | **Not T9.** Owner remains **T10 handoff**; T9 does not record or close them. |
  | T7 carry: insert partial-failure rollback proof (`for rollback in (0..inserted).rev()`). | **Not T9.** Owner remains **T10 / phase handoff**; the owner GUI smoke does not exercise this defensive branch. |
  | T6/T7/T8 carry: Moment 2 spec/architecture re-sync, DD-007 drain residuals 1-3. | **Not T9.** Owner remains **T10 / phase handoff**. |

  **Selected traps and non-applicable reasons.**

  | Trap | Applies? | Reason / close artifact hypothesis |
  |---|---|---|
  | #1 semantic migration | **Non-applicable.** | T9 adds no enum / IR / schema variant and no production Rust. It runs the already-landed T2–T8 pipeline and writes an observation script + a plan revision; there is no traversal call-site to audit. |
  | #2 side effects | **Non-applicable.** | T9 writes no production runtime code. The structural splice side-effects are T7-owned and tested; the owner only triggers them through the landed body-external handlers. |
  | #3 parallel / derived data drift | **Non-applicable.** | No new parallel vector / derived index / cache. |
  | #4 untested reject branch | **Non-applicable to T9 code, armed-as-carry.** | T9 adds no new reject / diagnostic / size branch in Rust. If the owner smoke surfaces a defect, it re-triggers trap #1/#4/#6 on the **owning** surface (T3 check / T6 loader / T7 runtime) and a fix lands there with a direct test — **not** worked around in the `.ui`; the T9 checklist re-runs to green before close. |
  | #5 carry-forward | **Applies.** | The gating open item is **owner acceptance**, which the assistant cannot self-certify. Close with owner / scope / impact / close-condition rows; the three trigger observations + the partial-failure rollback proof stay their existing owners (T10 / phase handoff), not T9. |
  | #6 deterministic failure | **Standing, not pre-selected.** | No recurring failure exists before the owner run. Known mechanics hazards (wrong-HWND, click-coordinate miss, member-order coord drift) are documented in the owner script, not re-rolled. |
  | #7 weak GUI evidence | **Applies — but discharged by owner observation, not an assistant screenshot.** | T9 **is** the owner human-visible smoke the assistant baseline cannot replace. The assistant does not produce new screenshot evidence here (that was T8 #7); the assistant's obligation is to **arm the positive control in the owner script**: the owner must see the item count *track each click* (not a static look-alike), across all four mutation forms, plus WrapPanel reflow / ScrollView behaviour. Optionally the owner captures frames as an audit artifact. |

  **Review lane.** **No special independent code-review lane for the
  assistant-side prep** (T9 adds no Rust, no new branch; the deliverable
  is a doc-side plan revision + an owner observation script). The
  standing review re-arms **only if** the owner smoke surfaces a defect
  and a fix lands on the T9 branch — that fix takes the review tier of
  its **owning** surface (full independent review for a T7 runtime
  fix / a T6 loader fix; branch/test-focused for a T3 check fix), per
  [implementation-gates.md §4](../../../procedures/implementation-gates.md).
  The user-initiated second-agent review of this T9 entry verifies (a)
  the plan revision faithfully reflects the four-mutation-form T8
  divergence, (b) the owner script arms the positive control across all
  four forms + WrapPanel/ScrollView, and (c) carry-forward ownership is
  intact (triggers / rollback proof not silently absorbed into T9).

  **Planned proof obligations before the owner runs (hypotheses).** T9
  proves nothing in code; the "proof" is the owner's human-visible
  observation. Expected observations, each a positive control the owner
  confirms or fails:

  | Planned observation | Category | Hypothesis before the owner run |
  |---|---|---|
  | `Add` → N+1, prefix thumbnails visually undisturbed | cardinality / positional-identity positive control | The appended `NEW #N` thumbnail appears at the tail; `S01 #0`…prefix unchanged. Count tracks the click. |
  | `Remove` → N-1, named tail item gone | cardinality positive control | `drop-last` removes the last thumbnail; the named tail item disappears. |
  | `Clear` → 0 children, member live, no crash | empty-case invariant | The `for` slot materialises zero children; the thumbnail area is empty; the app does not crash and the member stays live (later `Reset`/`Add` work). |
  | `Reset` → restored to the static-literal N | static-literal reset positive control | `labels = ["S01"…"S06"]` restores six thumbnails `S01 #0`…`S06 #5`. |
  | WrapPanel reflow / ScrollView behaviour stay correct across mutations | layout invariant around the generated set | As the count changes, the WrapPanel reflows without corrupting the surrounding static slices; the ScrollView scrolls the generated set correctly. |

  **Known carry-forward candidates before the owner runs.**

  | Candidate | Owner / scope / impact / close condition |
  |---|---|
  | Owner acceptance of the GUI smoke. | **Owner = the project owner, recorded at T9 close.** Scope: owner runs `gallery-rust` and observes the four mutation forms + the positive control. Impact: the assistant baseline does not substitute for owner judgment (ADR item 6). Close: owner records acceptance, or a fail observation that re-runs the checklist to green. |
  | A defect the owner smoke surfaces. | **Owner = the surfaced surface (T3 check / T6 loader / T7 runtime).** Scope: a check/loader/runtime defect. Impact: must be fixed on the owning surface with a direct test, not papered over in the `.ui`; the T9 checklist re-runs to green. Close: fix lands on the owning surface, or an explicit disposition recorded here. |
  | Three deferral-trigger observations (structured-item / `TypedValue`; loop-external indexed read; bindable-`fill`) + Phase 7b reservation. | **Owner = T10 handoff.** Not recorded or closed by T9. |
  | Insert partial-failure rollback proof. | **Owner = T10 / phase handoff** (T7 carry). The owner GUI smoke does not exercise this defensive branch. |
  | DPI blur. | **Owner = M4** ([DD-V-022/023](../../../cross-milestone/decisions/dpi-awareness-m4-deferral.md)). A known cosmetic residual, not a Phase 7 failure; noted to the owner, not a fail criterion. |

- **2026-06-17 / T8 start gate — gallery thumbnail slice + assistant
  build/launch opened.** Started by reading the prior carry-forward rows
  in this log before treating [plan.md](./plan.md) T8 as a hypothesis.
  T7 closed the runtime range mutation and explicitly left two items to
  T8: the **assistant-visible cardinality positive control** (gallery
  `.ui` → for-generated, 2+ frame screenshots) and the **structured-item
  / `TypedValue` deferral-trigger owner consult** (G-2 / T1 addendum 4).
  T9 owns the owner human-visible smoke; T10 owns Moment 2 spec re-sync
  and the handoff carry rows. Critical re-think of T8's responsibility
  kept the plan's three-part structure (owner consult → additive `.ui`
  growth → build/launch/2+ frame evidence) and **sharpened two
  under-specified load-bearing points** into plan T8: the GUI
  input-injection mechanism (the plan said "launch + screenshot" but not
  how `Add`/`Remove` are clicked — pinned to the proven Phase 6
  `capture-lightbox.ps1` `SetCursorPos`+`mouse_event` pattern) and the
  viewport-visibility constraint (the for-set lives inside a deep
  `ScrollView`, so the generated thumbnails *and* the driving Buttons
  must both fall in the captured frame). Append-value mechanics recorded
  as a composition input.

  **Carry-over checked from prior tasks.**

  | Carry-over | T8 disposition hypothesis |
  |---|---|
  | T7 carry: assistant-visible collection-cardinality positive control absent. | **T8 owns.** Grow the ScrollView-backed `S01…` WrapPanel into a `for`-generated set driven by body-external `Add`/`Remove` Buttons; record 2+ frame N → append → remove evidence. |
  | T1 addendum 4 / G-2: the gallery thumbnail varies **two** per-item attributes (`fill` colour + label) — record-like data a scalar `for` cannot express; the DD-M3-P7-002 structured-item / `TypedValue` trigger that **cannot be smuggled**. | **T8 owns the owner consult** (mandatory, owner-confirm-gated). First subtask before authoring: tell the owner the trigger fired, recommend **reduce-to-single-attribute for Phase 7** (label/id from the collection, static `fill`), route the trigger to M4/M5 (reopening structured items is against FD-C thesis-sequencing and revises M3 acceptance), record the decision here and queue the observation for the **T10 handoff**. Per memory, the consult is **plain Japanese chat, not `AskUserQuestion`**. |
  | T7 carry: owner human-visible GUI smoke absent. | **Not T8.** Owner remains **T9**; T8 supplies the assistant baseline only (Start-Process survival is a supporting signal, not the deliverable). |
  | T6/T7 carry: phase-end spec/architecture re-sync, DD-007 drain residuals 1-3, insert partial-failure rollback proof. | **Not T8.** Owner remains **T10 / phase handoff**; T8 does not close them. |

  **Selected traps and non-applicable reasons.**

  | Trap | Applies? | Reason / close artifact hypothesis |
  |---|---|---|
  | #1 semantic migration | **Non-applicable.** | T8 adds no enum / IR / schema variant. It authors a `.ui` and a capture script and consumes the already-landed T2–T7 for/collection pipeline; there is no traversal call-site to audit. |
  | #2 side effects | **Non-applicable.** | T8 writes no production runtime code. The structural splice side-effects are T7-owned and tested; T8 only triggers them through an authored handler. |
  | #3 parallel / derived data drift | **Non-applicable.** | No new parallel vector / derived index / cache is introduced by an authored `.ui` or a PowerShell capture script. |
  | #4 untested authored branch | **Non-applicable to T8 code, armed-as-carry.** | T8 adds no new reject / diagnostic / size branch in Rust. But T8 is the **first real end-to-end exercise** of the for-gallery pipeline through `gallery-rust/build.rs` (`wasamoc` check/lower/emit + loader + runtime); if it surfaces a gap, that re-triggers trap #1/#4/#6 on the **owning** surface (T3 check / T6 loader / T7 runtime) and is carried, not worked around in the `.ui`. |
  | #5 carry-forward | **Applies.** | T8 produces the structured-item / `TypedValue` trigger observation that **must** land in the T10 handoff (smuggle-forbidden), plus the T9 smoke handoff and any integration gap surfaced by the first real build. Close with owner / scope / impact / close-condition rows. |
  | #6 deterministic failure | **Standing, not pre-selected.** | No recurring failure exists before implementation. Known mechanics hazards (blank `PrintWindow` readback, wrong-HWND `MainWindowHandle`, click-coordinate miss, a deterministic gallery build/check failure) are root-caused, not re-rolled, if they appear. Obs-5-class teardown AV does **not** apply (single-process GUI host, not a multi-test Compositor binary). |
  | #7 weak GUI evidence | **Applies — the central trap of T8.** | The deliverable is GUI-host rendering. Close with screenshot + assistant analysis + a **positive control**: the 2+ frame N → `Add` (N+1, prefix undisturbed) → `Remove` mutation pair driven by body-external Buttons; a single static frame a hardcoded tree could equally produce is not evidence. Does not replace the T9 owner smoke. |

  **Review lane.** **Full independent review** (the GUI-render-evidence
  high-risk class, [implementation-gates.md §4](../../../procedures/implementation-gates.md)).
  Reason: T8's deliverable is assistant-visible GUI evidence (#7); the
  review must verify the positive control genuinely distinguishes
  collection-driven cardinality from a hardcoded look-alike, and that the
  structured-item trigger observation is correctly surfaced and routed
  (not smuggled). T8 adds no diagnostic / reject / size code branch, so
  the trap-#4 branch/test lane does not compose in.

  **Planned proof obligations before implementation.**

  | Planned branch / behaviour | Category | Hypothesis before implementation |
  |---|---|---|
  | The for-generated `gallery.ui` compiles and loads through the real host build. | integration smoke (item 3-adjacent) | `cargo build -p gallery-rust` runs `wasamoc` tokenize/parse/check/lower/emit on the rewritten `.ui` and the loader materialises the `for` slot — the first end-to-end exercise of the T2–T7 surfaces on a real multi-construct `.ui`. Proof: build green; a failure root-causes to the owning surface, not a `.ui` workaround. |
  | Assistant-visible 2+ frame mutation pair. | GUI evidence (item 5) + FD-B positive control | Launch `gallery-rust.exe`, capture initial N → click `Add` → N+1 (prefix thumbnails visually undisturbed) → click `Remove` → N; DPI-aware `CopyFromScreen`, title-enum HWND, `SetCursorPos`+`mouse_event` clicks. Assistant analyses pixels for the count delta and prefix stability. |
  | The captured region is legible. | GUI evidence prerequisite | Window sized / arranged so the ScrollView-backed generated set **and** the `Add`/`Remove` Buttons both fall in the captured frame; otherwise the positive control is unreadable. |
  | Structured-item trigger surfaced, decided, recorded. | carry-forward (#5) | Owner consult done in Japanese chat; the chosen single-attribute composition recorded here; the trigger observation queued for the T10 handoff. |
  | Assistant baseline ≠ owner smoke. | evidence-standard invariant | `Start-Process` survival is a supporting "no early crash" signal only; the owner human-visible smoke is T9's and is not discharged by T8. |

  **Known carry-forward candidates before implementation.**

  | Candidate | Owner / scope / impact / close condition |
  |---|---|
  | Structured-item / `TypedValue` trigger observation (G-2). | **Owner = T10 handoff** (recorded by T8 here). Scope: the gallery is the first concrete app case where scalar items cannot express per-item {colour, label}; routed to M4/M5 per DD-M3-P7-002. Impact: a `TypedValue` adoption revises M3 acceptance and cannot be smuggled. Close: owner decision recorded in this log at T8; T10 records the observation + re-trigger in `handoff.md`. |
  | Owner human-visible GUI smoke. | **Owner = T9.** Scope: owner runs `gallery-rust` and observes `Add`/`Remove` with the collection-mutated positive control. Impact: the assistant baseline does not substitute for owner judgment. Close: T9 records owner acceptance or a fail observation. |
  | Any integration gap surfaced by the first real for-gallery build. | **Owner = the surfaced surface (T3 check / T6 loader / T7 runtime).** Scope: a check/loader/runtime defect the `.ui` exposes. Impact: must be fixed on the owning surface, not papered over in the `.ui`. Close: defect fixed with a direct test on the owning surface, or an explicit disposition recorded here. |
  | DPI blur in the capture. | **Owner = M4** ([DD-V-022/023](../../../cross-milestone/decisions/dpi-awareness-m4-deferral.md)). Scope: host DPI-unawareness. Impact: a known cosmetic residual, not a Phase 7 failure. Close: M4 DPI work; T8 only notes it in the evidence analysis. |
  | Append-value composition form (constant vs counter `state`). | **Resolved at T8 authoring**, not a cross-task carry. Record the chosen form; the DD admits a scalar expr as the `append` argument. |

  **Owner reservation (2026-06-17) — routing is provisional, not
  settled.** During the T8 owner consult the owner examined a proposed
  parallel-collection / index-indirection composition
  (`for label, index in labels { Box { fill: colors[index]; Text { text:
  label } } }`). This is **not authorable in Phase 7**: `colors` read in
  the body is a loop-external collection read (rejected by
  `check::collection_external_read_segment` /
  `loop_external_collection_reads_rejected`, hint *"collection reads
  outside iteration not yet supported"*) and the `[index]` subscript is a
  deferred indexed-read syntax. It is the struct-of-arrays form of the
  same per-item record need; it routes to the FD-F正本
  **loop-external collection reads** row (Q5 uniform expression/reference
  extension), a different deferral axis from the structured-item /
  `TypedValue` row. Under the **current** plan both axes land **M4 or
  later** (M3's only remaining phase is Phase 8, which is editorial /
  assembly with no new grammar surface;
  [framing.md FD-F正本](../requirements/framing.md) routes loop-external
  reads to the Q5 expression extension, which has no M3 acceptance
  criterion). **The owner explicitly reserves the option to insert a new
  M3-Phase 7b** — ADR drafting + design judgment — to bring one or both
  of these capabilities into M3 instead of M4+. So the two trigger rows
  above carry an **"M4+ unless the owner exercises the reserved Phase 7b"**
  qualifier. This reservation does **not** block T8: T8 proceeds with the
  single-attribute reduction now, and the gallery composition would be
  revisited only if Phase 7b is actually opened. Phase 7b, if opened,
  would be its own task with its own start gate / ADR / review lane (not
  smuggled into T8). The T10 handoff records both trigger observations
  **with this reservation**, so the M4+ routing is not stated as settled.

  **Owner consult outcome (2026-06-17) — single-attribute reduction
  confirmed.** The owner explored three "vary the thumbnail fill /
  per-item record" compositions during the consult; each is blocked by a
  **distinct** deferred axis (verified in source, not asserted), and the
  owner accepted the reduction. Recorded for the T10 handoff as **three
  observation axes**, all carrying the Phase 7b reservation above:

  | # | Proposed composition | Blocker (source-verified) | Deferred axis / routing |
  |---|---|---|---|
  | 1 | per-item `{fill colour, label}` record | a scalar `for` item binds one value | structured-item / `TypedValue` (DD-M3-P7-002 §pressure → M4 showcase-spec / M5 data-surface) |
  | 2 | `for label, index in labels { fill: colors[index] }` | `colors` body read is a loop-external collection read (`check::loop_external_collection_reads_rejected`, *"collection reads outside iteration not yet supported"*); `[index]` is deferred indexed-read syntax | loop-external collection reads (FD-F正本 → Q5 uniform expression/reference extension) |
  | 3 | `for color, index in colors { fill: color }` | `Box.fill` is **constant-only** (DD-M3-P2-004; `check::box_fill_state_ident_rejected`, hint *"`Box.fill` is constant-only"* / `"Color"`); also no `Color[]` element type (DD-002 element set is `i32`/`string`/`bool`) | bindable `Box.fill` (DD-M3-P2-004; dynamic styling naturally M5 theming) |

  **Root insight recorded for the handoff:** the real root constraint
  behind "vary the thumbnail fill per item" is **not** the iteration
  grammar — it is that `Box.fill` is constant-only (DD-M3-P2-004). Every
  collection-side workaround still ends in a dynamic bind into `fill`,
  which is closed in M3. So the per-item-colour desire is recorded as the
  **bindable-`fill` trigger**, distinct from the two collection-read
  triggers.

  **Settled T8 composition (the authored gallery slice):**

  - `state labels: string[] = ["S01", … ]` (a modest initial N for a
    legible tail; the original 32 static `S01…S32` boxes are reduced to a
    `for`-generated set — the recorded additive deviation).
  - `ScrollView { offset-y: scroll_y; WrapPanel { for label, index in
    labels { Box { aspect: 1:1; fill: #336699cc; Text { text:
    "\{label} #\{index}" } } } } }` — single varying attribute (the
    `label` from the collection plus the positional `index`); **static
    uniform `fill`**; descriptive binder name `index` for the public
    example (the bind *shape* is the `wasamoc check` positive control
    `"\{label} #\{i}"`). Display reads `S01 #0`, `S02 #1`, … for
    owner-requested continuity.
  - **Body-external** text Buttons driving all four authored mutation
    forms (DD-M3-P7-002): `Add` → `labels = labels.append("NEW")`,
    `Remove` → `labels = labels.drop-last()`, `Clear` → `labels = []`
    (empty-literal whole-value set), `Reset` →
    `labels = ["S01", … , "S06"]` (static-literal reset). `append` /
    `drop-last` are the only contextual *methods*; clear / reset are
    *literal assignment*, not methods (the ADR deliberately kept the
    method vocabulary to two — DD-M3-P7-002 §RHS extent M3b). The `index`
    suffix makes appended items (`NEW #6`, …) visibly distinct without a
    counter `state`, so the prior append-value composition question is
    closed by the index display.

- **2026-06-17 / T8 end gate — gallery iteration slice landed +
  assistant-visible positive control captured.** Grew
  [examples/gallery/gallery.ui](../../../../examples/gallery/gallery.ui)
  additively into a `for`-generated thumbnail set and produced the
  6-frame assistant evidence (2 sequences × 3). T8 added **no Rust
  code**; it exercises the landed
  T2–T7 for/collection pipeline through the real host build and supplies
  the GUI positive control. T9 (owner human-visible smoke) and T10
  (Moment 2 spec re-sync + handoff carry rows) remain their owners.

  **Source enumeration used for close artifacts.**

  ```text
  git status --short
  git diff --name-only -- "*.rs"
  git diff --stat -- examples/gallery/gallery.ui
  cargo build -p gallery-rust --release
  ```

  `git diff --name-only -- "*.rs"` is **empty** (T8 adds no Rust branch);
  `git diff --stat` on `gallery.ui` shows 25 insertions / 159 deletions
  (the 32 static `S01…S32` boxes collapse to one `for`-generated `Box`,
  plus the four body-external mutation Buttons). The new untracked
  `process/milestone-3/phase-7/evidence/` holds the capture script, a
  README, and 6 PNG frames (2 sequences × 3).

  **Implemented-branch / behaviour map.**

  | Implemented branch / behaviour | Category | Source query / diff cue | Direct test or owner |
  |---|---|---|---|
  | The `for`-generated `gallery.ui` compiles + loads through the real host build | integration smoke | `cargo build -p gallery-rust --release` green; diff cue `for label, index in labels` | `gallery-rust/build.rs` runs `wasamoc` tokenize/parse/check/lower/emit; shape pinned by `check::tests::gallery_like_for_shape_and_body_external_handlers_accepted` + `lower::tests::gallery_like_for_shape_lowers_single_box_body_and_external_mutations` (T3); runtime mutation by `wasamo-runtime/tests/iteration_mutation_integration.rs` (T7) |
  | Assistant-visible collection-cardinality positive control (ADR item 5) | GUI evidence | evidence PNGs `t8-iteration-{init,add,remove}.png` (append/remove) + `t8-clearreset-{init,clear,reset}.png` (clear/reset) | **this task** — screenshots + analysis below |
  | No new reject / diagnostic / size / semantic **Rust** branch | n/a | `git diff --name-only -- "*.rs"` empty | the for/collection branches are T2–T7-owned and tested there; T8 does not add or alter one |

  **GUI evidence (trap #7 close artifact).**

  - **Capture mechanics:** per-monitor-DPI-aware
    (`SetProcessDpiAwarenessContext(-4)`), enumerate the top-level
    `Gallery` HWND by title, drive the body-external `Add` / `Remove`
    Buttons with `SetCursorPos` + `mouse_event` at window-relative
    coordinates, `CopyFromScreen` over `GetWindowRect` (not `PrintWindow`).
    Script:
    [evidence/capture-iteration.ps1](../evidence/capture-iteration.ps1)
    (adapted from the Phase 6 `capture-lightbox.ps1`). Window sized
    1280×1316 so the ScrollView-backed `for` set **and** the four
    body-external Buttons fall in one frame (the viewport-visibility
    constraint; the wider window keeps the appended items on a
    fully-visible row).
  - **Sequence A — append / remove (`t8-iteration-*.png`):** `init` = 6
    (`S01 #0` … `S06 #5`) → `add` = 7 (`+ NEW #6`, **fully legible on the
    same row** — 7 items per row at 1280 wide) → `remove` = 6 (the named
    `NEW #6` is **gone**, tail back to `S06 #5`). The sequence is held at
    ≤ 7 items so no item wraps below the fold: the drop-last "named item
    disappears" step is crisply readable (the T8-review-① fix; the
    earlier 4-frame `add2 = 8 / NEW #7` capture clipped the 8th item on a
    wrapped row and is replaced).
  - **Sequence B — clear / reset (`t8-clearreset-*.png`):** `init` = 6 →
    `clear` = **0** (the `for` slot materialises zero children; the
    thumbnail area is empty, member still live) → `reset` = 6
    (`S01 #0` … `S06 #5` restored from the static literal).
  - **Positive control (both sequences):** the item count **tracks the
    body-external Button clicks** (6 → 7 → 6, and 6 → 0 → 6); the prefix
    thumbnails `S01 #0`…`S06 #5` stay **visually stable** across the tail
    edits (the strict pointer-retention *invariant* is proven separately
    by T7's
    `iteration_mutation_integration::reactive_for_tail_append_reset_remove_preserves_order_and_prefix_identity`,
    not claimed from pixels — the screenshot shows visual stability, the
    unit test shows identity); the `index` binder **re-derives** the
    position for the appended item (`NEW #6`); the empty `clear` case is
    well-behaved (zero children, no crash); and the upper static slices
    (Grid, `Photo 1`…`Photo 10`) stay byte-identical frame-to-frame. A
    hardcoded tree cannot make the count track the click — this
    distinguishes collection-driven cardinality from a static look-alike
    (FD-B positive control). `Start-Process` survival is a supporting "no
    early crash" signal only and does **not** substitute for the T9 owner
    human-visible smoke.
  - **DPI:** the capture is DPI-aware; any residual host-side blur is the
    known M4 residual ([DD-V-022/023](../../../cross-milestone/decisions/dpi-awareness-m4-deferral.md)),
    not a Phase 7 failure.

  **Recorded additive deviation (plan T8 "record the decided
  deviation").** The original 32 static `S01…S32` boxes (6 cycling
  `fill` colours) are reduced to a single `for`-generated `Box` over
  `state labels: string[]` with initial N = 6 and a **uniform static
  `fill`** (`#336699cc`); four body-external text Buttons (`Add`,
  `Remove`, `Clear`, `Reset`) are inserted before the `ScrollView`,
  exercising all four DD-M3-P7-002 mutation forms (`append` /
  `drop-last` / empty-literal clear / static-literal reset). The Grid,
  the upper `Photo` WrapPanel, the scroll Buttons, and the lightbox
  subtree are byte-identical.

  **Behaviour / invariant carry scan.**

  | Behaviour / invariant | Disposition |
  |---|---|
  | The gallery now mutates widget-tree cardinality through authored body-external handlers (`labels.append` / `labels.drop-last`). | **Closed in T8** as visible evidence; the runtime contract itself is T7-owned and tested. |
  | Three deferral-trigger observations surfaced at the gallery (structured-item / `TypedValue`; loop-external indexed read; bindable-`fill`). | **Carried to T10 handoff**, each with the Phase 7b owner reservation. Recorded in the consult-outcome table above. |
  | Owner human-visible smoke. | **Owner = T9.** Not discharged by the assistant baseline. |
  | DPI blur. | **Owner = M4.** Known residual; noted in the evidence analysis only. |

  **Carry-forward ownership.**

  | Carry-forward | Owner task | Scope | Impact | Close condition |
  |---|---|---|---|---|
  | Structured-item / `TypedValue`, loop-external indexed read, and bindable-`fill` trigger observations | T10 handoff (Phase 7b reservation held by owner) | Three deferred axes the gallery surfaced; routed M4+ unless Phase 7b is opened | The per-item-richer gallery is deferred; M3 ships the single-attribute slice | T10 records all three observations + re-triggers + the Phase 7b reservation in `handoff.md`; or the owner opens Phase 7b as its own task |
  | Owner human-visible GUI smoke | T9 | Owner runs `gallery-rust`, observes Add/Remove with the collection-mutated positive control | Assistant baseline does not substitute for owner judgment | T9 records owner acceptance or a fail observation |
  | Phase-end spec/architecture re-sync (runtime list-setter names, landed diagnostics, gallery slice) | T10 | Moment 2 docs | Implementation details need a final doc check | T10 re-syncs or records no divergence |

  **Verification runs.**

  ```text
  cargo build -p gallery-rust --release   # green (for-gallery pipeline end-to-end)
  pwsh evidence/capture-iteration.ps1 ...  # Sequence A (init,add,remove)
  pwsh evidence/capture-iteration.ps1 ...  # Sequence B (init,clear,reset)
  ```

  The exact window size + click coordinates for both runs are recorded in
  [evidence/README.md](../evidence/README.md) so a third party can
  reproduce them without re-deriving the Button coordinates (T8-review-②).

  No Rust changed, so `cargo fmt` / workspace test ownership stays with
  T10's phase-end gates; the for-gallery **build** going green is the
  integration proof this task owns. The pre-existing `wasamo`
  linkable-target warning is unchanged.

- **2026-06-16 / T7 review remediation — branch proof gaps closed.**
  A second-agent review accepted the core T7 implementation but found two
  proof gaps before merge: string/bool collection assignment sub-branches
  were over-grouped under append-only evidence, and the breadth fixture did
  not directly observe non-divergence. T7 remains open until this remediation
  and its verification are green.

  **Source enumeration used for this remediation.**

  ```text
  rg -n "collection_assignment_append_drop_last_and_literal_reset_i32|collection_assignment_supports_string_and_bool_items|collection_assignment_string_bool_drop_last_and_literal_reset" wasamo-runtime\src\handler.rs
  rg -n "__runtime_health_for_test|__reactive_divergence_diagnostics_present_for_test|large_breadth_tail_append_converges_beyond_mutation_cap|after conditional removal through splice seam|after conditional reinsert through splice seam" wasamo-runtime\src\lib.rs wasamo-runtime\tests\iteration_mutation_integration.rs
  rg -n "for rollback in \(0\.\.inserted\)\.rev\(\)|plan_tail_range_change" wasamo-runtime\src\ir_loader.rs
  ```

  **Review finding disposition.**

  | Finding | Disposition |
  |---|---|
  | `string[]` / `bool[]` drop-last and literal-reset branches were not directly fired. | **Closed in T7 remediation.** Added `handler::tests::collection_assignment_string_bool_drop_last_and_literal_reset`, firing `ListDropLast` for both element types and `ListLit` through `list_literal_string` / `list_literal_bool`. |
  | Breadth/cap fixture only observed materialized child count and tail text. | **Closed in T7 remediation.** `large_breadth_tail_append_converges_beyond_mutation_cap` now performs a gallery-scale `0 -> 8` write and a larger `8 -> 64` write, and asserts runtime health is `Healthy` with no divergence diagnostics after each step. |
  | Conditional path now depends on the shared structural splice seam but T7 did not directly toggle a conditional fixture. | **Closed in T7 remediation.** `reactive_for_tail_append_reset_remove_preserves_order_and_prefix_identity` now toggles `show` false/true after range mutation and asserts child text order plus VisualCollection order. |
  | `plan_tail_range_change` still carried stale `#[allow(dead_code)]`. | **Closed in T7 remediation.** Removed the stale attribute; the function is production-live through `mutate_for_loop_subtree`. |
  | Insert partial-failure rollback branch was not listed in the close gate. | **Carried with owner below.** The branch is defensive after successful staging but before all child insertions complete. Current mock-free fixtures have no natural way to fault `WidgetNode::insert_child` after a prefix insert without mocking WinRT/VisualCollection or corrupting indices outside authored/runtime paths. |
  | Other loader-guarded defensive branches (`ensure_collection_assignment_target` element-type mismatch, `collection_len_tracked` non-list lookup, missing-slot / null-parent structural diagnostics) remain untested. | **Non-blocking defensive note.** These are unreachable through validated authored DSL under the current loader contracts or require corrupted runtime state; they are lower risk than the carried insert partial-failure rollback branch and do not add a merge-blocking owner. Re-trigger trap #1/#2 if a future task opens a production-like path to any of them. |

  **Implemented-branch test map addendum.**

  | Implemented branch / behavior | Category | Source query / diff cue | Direct test or owner |
  |---|---|---|---|
  | Handler `string[]` drop-last | semantic branch | `rg` hit: `collection_assignment_string_bool_drop_last_and_literal_reset`; subcase `ListDropLast { elem: IrType::Str }` | `handler::tests::collection_assignment_string_bool_drop_last_and_literal_reset` |
  | Handler `bool[]` drop-last | semantic branch | `rg` hit: `collection_assignment_string_bool_drop_last_and_literal_reset`; subcase `ListDropLast { elem: IrType::Bool }` | `handler::tests::collection_assignment_string_bool_drop_last_and_literal_reset` |
  | Handler `string[]` literal reset through `list_literal_string` | semantic branch | `rg` hits: `collection_assignment_string_bool_drop_last_and_literal_reset`, `IrLiteral::Str("X")`, `IrLiteral::Str("Y")` | `handler::tests::collection_assignment_string_bool_drop_last_and_literal_reset` |
  | Handler `bool[]` literal reset through `list_literal_bool` | semantic branch | `rg` hits: `collection_assignment_string_bool_drop_last_and_literal_reset`, `IrLiteral::Bool(true)`, `IrLiteral::Bool(false)` | `handler::tests::collection_assignment_string_bool_drop_last_and_literal_reset` |
  | Breadth fixture non-divergence at gallery scale and larger-than-cap scale | size / scheduler invariant | `rg` hits: `large_breadth_tail_append_converges_beyond_mutation_cap`, `__runtime_health_for_test`, `__reactive_divergence_diagnostics_present_for_test` | `iteration_mutation_integration::large_breadth_tail_append_converges_beyond_mutation_cap` |
  | Conditional removal/reinsert uses the shared structural splice seam after range mutation | semantic / observable behavior | `rg` hits: `after conditional removal through splice seam`, `after conditional reinsert through splice seam` | `iteration_mutation_integration::reactive_for_tail_append_reset_remove_preserves_order_and_prefix_identity` |
  | Insert partial-failure rollback after a prefix of staged children has committed | defensive invariant | `rg` hit: `for rollback in (0..inserted).rev()` | **Owner = T10 / phase handoff carry.** Scope: future mock-free fault surface or explicit fault-injection design for `WidgetNode::insert_child` after partial success. Impact: current authored paths still stage before commit and use valid indices; missing direct proof only affects the defensive cleanup branch if WinRT insertion fails mid-batch. Close condition: add a direct test when a production-like fault surface exists, or record an accepted infeasibility decision with review sign-off. |

  **Carry-forward ownership addendum.**

  | Carry-forward | Owner task | Scope | Impact | Close condition |
  |---|---|---|---|---|
  | Insert partial-failure rollback proof for `for rollback in (0..inserted).rev()` | T10 / phase handoff carry | Defensive cleanup branch after successful staging and partial child insertion | A future fallible insertion surface must not silently leave an orphaned prefix committed after later insertion failure | T10 records the carry in handoff unless this task gains a mock-free direct fault surface before merge; future owner closes with direct branch test or reviewed infeasibility disposition. |

  > **2026-06-17 addendum (supersede, not rewrite):** the owner ruled this
  > carry **closed in-phase on the T9 branch** rather than deferred to T10.
  > The rollback branch now has a direct test
  > (`staged_for_insert_commit_failure_rolls_back_partial_inserts`) driven by
  > a `debug_assertions`-gated Rust-side fault seam (the "no mock-free fault
  > surface" obstacle above was resolved by adding such a seam, gated out of
  > release). See the 2026-06-17 T9 review-remediation entry at the top of
  > this log. This row is retained as history; the carry no longer reaches T10.

  **Verification runs for this remediation.**

  ```text
  cargo test -p wasamo-runtime collection_assignment
  cargo fmt --all -- --check
  cargo test -p wasamo-runtime --test iteration_mutation_integration
  git diff --check
  cargo test -p wasamo-runtime
  cargo test --workspace
  ```

  All completed successfully on 2026-06-16. The first
  `cargo fmt --all -- --check` attempt failed only on formatting in
  `iteration_mutation_integration.rs`; `cargo fmt --all` was run and the
  final format check passed. `git diff --check` reported only the existing
  Windows line-ending warnings, not whitespace errors. `cargo test
  --workspace` still emits the pre-existing package `wasamo` linkable-target
  warning; no test failed.

- **2026-06-16 / T7 close gate — reactive range mutation landed.**
  Implemented the runtime half of iteration cardinality: handler-side
  collection writes, `BindingTarget::ForLoopSubtree`, the placement-aware
  structural splice seam used by conditional and `for` mutation paths,
  tail insert/remove reconciliation, mutation-time per-item binding reuse,
  and mock-free Windows runtime fixtures. T8 remains the owner of
  assistant-visible gallery screenshot evidence.

  **Source enumeration used for close artifacts.**

  ```text
  git status --short
  git diff --name-only
  rg -n "ForLoopSubtree|register_for_loop_binding|mutate_for_loop_subtree|insert_structural_child|remove_structural_child|ForLoopRuntimeState|DeclaredMemberSlot::ForLoop|set_if_changed|collection_element_type|set_.*_list" wasamo-runtime\src wasamo-runtime\tests\iteration_mutation_integration.rs
  rg -n "collection_assignment_append_drop_last_and_literal_reset_i32|collection_assignment_supports_string_and_bool_items|collection_assignment_rejects_wrong_lhs_at_runtime|collection_assignment_rejects_bare_collection_copy_at_runtime|evaluate_rejects_bare_collection_forms_in_integer_context|reactive_for_tail_append_reset_remove_preserves_order_and_prefix_identity|handler_collection_append_is_observable_before_click_returns|empty_drop_last_is_equal_value_and_does_not_dirty_range|reactive_for_zstack_tail_append_uses_child_carried_placement|large_breadth_tail_append_converges_beyond_mutation_cap|staged_for_insert_build_failure_leaves_tree_unchanged" wasamo-runtime\src\handler.rs wasamo-runtime\tests\iteration_mutation_integration.rs
  ```

  `git status --short` also shows the new untracked test file
  `wasamo-runtime/tests/iteration_mutation_integration.rs`; `git
  diff --name-only` lists only tracked-file edits, as expected.

  **Trap #1 call-site audit table.**

  | Surface / call-site class | Source query / diff cue | Classification |
  |---|---|---|
  | `BindingTarget` | `rg` hits: `ForLoopSubtree` in `reactive.rs`, construction in `ir_loader.rs` | **Extended.** Added `ForLoopSubtree { parent, declared_member_index }` and `register_for_loop_binding`; existing `register_binding` / `register_bool_binding` / `register_for_item_*` / `register_conditional_binding` still destructure only their target class and panic on wrong class. |
  | `HandlerExpr::ListAppend` / `ListDropLast` / `ListLit` in handler evaluation | `rg` hits: `collection_element_type`, `set_i32_list`, `set_string_list`, `set_bool_list`, handler tests | **Extended under `Assign` only.** Runtime handler evaluation now performs whole-value read-modify-write for append/drop-last/literal reset and uses `set_if_changed`. Bare collection forms still reject in integer context. |
  | `HandlerExpr::ListPropRead` copy | diff cue: `is_collection_expr` excludes `ListPropRead`; test `collection_assignment_rejects_bare_collection_copy_at_runtime` | **Deliberately rejects.** Author/loader bare-copy deferral remains closed; T7 did not reopen whole-list copy semantics. |
  | `DeclaredMemberSlot::ForLoop` / `ForLoopRuntimeState.live_children` | `rg` hits: static construction, `declared_slot_live_cardinality`, `mutate_for_loop_subtree` | **Extended.** T6's static live cardinality is now reconciled by the structural effect on collection changes. |
  | Widget-only filters (`IrNode::widget_children()` and runtime traversal helpers) | No T7 diff in `wasamo-ir`; runtime mutation uses declared-slot index + body template, not widget-only child filters | **Correctly unaffected.** T7 consumes already-validated `ControlFlowNode::For` bodies and does not add a new widget-only traversal. |

  **Trap #2 structural side-effect enumeration.**

  | DD-M3-P7-006 side effect | T7 disposition |
  |---|---|
  | Child splice with carried placement | **Implemented.** `insert_structural_child` / `remove_structural_child` are the single mutation seam; `for` tail insertion computes seam offsets through `materialized_offset_for_declared_slot`. |
  | Visual sibling order at seam-computed positions | **Implemented / tested.** The seam delegates to `WidgetNode::insert_child` / `insert_child_with_zstack_placement` / `remove_child`, whose VisualCollection order is asserted by `reactive_for_tail_append_reset_remove_preserves_order_and_prefix_identity` and `reactive_for_zstack_tail_append_uses_child_carried_placement`. |
  | Layout invalidation | **Implemented.** Conditional and `for` mutation mark `mark_layout_dirty_for(parent_ptr)` after successful structural change only; same-length reset and empty equal write perform no structural invalidation. |
  | Registry release / registration | **Reused for removal; existing registration for inserts.** Removed subtrees go through `widget_destroy`; `reactive_for_tail_append_reset_remove_preserves_order_and_prefix_identity` registers destroy callbacks on generated children and observes release. Newly built inserted children use the existing build/registry path. |
  | Effect disposal ahead of teardown / attach timing | **Reused + guarded.** Per-item effects remain child-owned; tail removal calls `widget_destroy` tail-first, disposing child bindings before the subtree is dropped. Staged inserted children are built before tree mutation; if staging fails, the staged children are destroyed before return (`staged_for_insert_build_failure_leaves_tree_unchanged`). |
  | No other parent-owned metadata | **Preserved.** ZStack placement remains child-carried; no parent-owned ZStack vector was reintroduced. Grid `cell_placements` remains static-only and untouched by T7. |

  **Trap #3 parallel / derived-data sync artifact.**

  | Parallel / derived structure | Source query / result | Disposition |
  |---|---|---|
  | `ForLoopRuntimeState.live_children` vs materialised generated range | `rg` hits: `state.borrow().live_children`, `plan_tail_range_change(old_len, new_len)`, state update after insert/remove | **Closed.** Count is read before mutation, tail plan executes, and `live_children` updates only after successful insert/remove; initial effect run sees old == new and is a no-op. |
  | `declared_slots` vs seam offset | `rg` hits: `materialized_offset_for_declared_slot(declared_member_index, &slots)` in static load and mutation | **Closed.** T7 reuses the T4 seam; static/conditional siblings around the `for` slot are asserted in `reactive_for_tail_append_reset_remove_preserves_order_and_prefix_identity`. |
  | Collection signal equality vs dirty effects | `rg` hits: `set_if_changed` production callers in `HandlerEvalContext` and test setters | **Closed.** Collection writes use `set_if_changed`; empty equal write returns false and leaves child pointers unchanged. |
  | Child-carried placement vs ZStack range insertion | `rg` hits: `insert_structural_child(..., placement)`, `reactive_for_zstack_tail_append_uses_child_carried_placement` | **Closed.** Mutation-time ZStack generated children carry explicit placement and layout at `h-align:end` / `v-align:start`. |
  | Per-item child-owned effects vs tail removal | `rg` hits: `register_for_item_*`, `widget_destroy(removed)`, destroy-log fixture | **Closed.** Tail removal destroys generated children tail-first; child-owned binding effects leave with the subtree. |

  **Implemented-branch test map.**

  | Implemented branch / behavior | Category | Source query / diff cue | Direct test or owner |
  |---|---|---|---|
  | Handler collection append / drop-last / literal reset evaluate under `Assign` for `i32[]` | semantic branch | `rg` hits: `ListAppend`, `ListDropLast`, `ListLit`, `collection_assignment_append_drop_last_and_literal_reset_i32` | `handler::tests::collection_assignment_append_drop_last_and_literal_reset_i32` |
  | Handler collection append supports `string[]` and `bool[]` values | semantic branch | `rg` hit: `collection_assignment_supports_string_and_bool_items` | `handler::tests::collection_assignment_supports_string_and_bool_items` |
  | Runtime collection assignment rejects wrong LHS / source mismatch | reject branch | `rg` hit: `collection_assignment_rejects_wrong_lhs_at_runtime` | `handler::tests::collection_assignment_rejects_wrong_lhs_at_runtime` |
  | Bare collection copy remains rejected at runtime | reject / owner-boundary branch | `rg` hit: `collection_assignment_rejects_bare_collection_copy_at_runtime`; diff cue: `is_collection_expr` excludes `ListPropRead` | `handler::tests::collection_assignment_rejects_bare_collection_copy_at_runtime` |
  | Bare collection forms still reject in integer handler context | reject branch | `rg` hit: `evaluate_rejects_bare_collection_forms_in_integer_context` | `handler::tests::evaluate_rejects_bare_collection_forms_in_integer_context` |
  | `ForLoopSubtree` initial run does not double-create T6 static children | size / semantic branch | `rg` hits: `register_for_loop_binding`, `old_len == new_len`; fixture starts with `["A", "B"]` and observes exactly two generated children before mutation | `iteration_mutation_integration::reactive_for_tail_append_reset_remove_preserves_order_and_prefix_identity` initial assertion |
  | Tail append inserts generated child before following static sibling and preserves prefix pointers | size / observable behavior | `rg` hit: `reactive_for_tail_append_reset_remove_preserves_order_and_prefix_identity`; diff cue: `TailRangePlan::Insert` | `iteration_mutation_integration::reactive_for_tail_append_reset_remove_preserves_order_and_prefix_identity` |
  | Same-length reset updates retained item bindings in place, no structural edit | semantic / invariant | fixture subcase `["A","B","C"] -> ["X","Y","Q"]`; prefix pointer checks include generated children | `iteration_mutation_integration::reactive_for_tail_append_reset_remove_preserves_order_and_prefix_identity` |
  | Tail removal disposes generated subtrees tail-first and releases registry entries | semantic / invariant | destroy-count + destroy-log subcase; diff cue: `TailRangePlan::Remove { tail_first_indices }` | `iteration_mutation_integration::reactive_for_tail_append_reset_remove_preserves_order_and_prefix_identity` subcase destroy log `[4, 3]` |
  | Same-batch removed-item binding guard skips out-of-range reads without panic | semantic branch | same fixture shrinks collection while item effects and structural effect are dirtied by the same signal; T6 registration guard remains the source branch | `iteration_mutation_integration::reactive_for_tail_append_reset_remove_preserves_order_and_prefix_identity`; T6 unit `reactive::tests::register_for_item_binding_writes_item_index_and_skips_out_of_range` remains the direct guard unit |
  | Authored handler append is observable before click returns | semantic / drain item 4 | `rg` hit: `handler_collection_append_is_observable_before_click_returns`; diff cue: `HandlerEvalContext` list setters call `set_if_changed` | `iteration_mutation_integration::handler_collection_append_is_observable_before_click_returns` |
  | Empty equal collection write produces no dirty range mutation | semantic / invariant | `rg` hit: `empty_drop_last_is_equal_value_and_does_not_dirty_range`; diff cue: `set_if_changed` false branch | `iteration_mutation_integration::empty_drop_last_is_equal_value_and_does_not_dirty_range`; `reactive::tests::signal_set_if_changed_skips_equal_value_dirtying` |
  | ZStack range append preserves child-carried placement and Visual order | observable behavior | `rg` hit: `reactive_for_zstack_tail_append_uses_child_carried_placement` | `iteration_mutation_integration::reactive_for_zstack_tail_append_uses_child_carried_placement` |
  | Breadth-heavy append larger than `MUTATION_CAP` converges | size / scheduler invariant | `rg` hit: `large_breadth_tail_append_converges_beyond_mutation_cap` | `iteration_mutation_integration::large_breadth_tail_append_converges_beyond_mutation_cap` (64 generated children from one collection write) |
  | Stage-then-commit build failure leaves tree unchanged and logs range-scoped diagnostic | diagnostic / invariant | `rg` hit: `staged_for_insert_build_failure_leaves_tree_unchanged`; diff cue: `for range insert build failed at position` | `iteration_mutation_integration::staged_for_insert_build_failure_leaves_tree_unchanged` (memory-IR invalid body, initial len 0, append faults staging) |

  **Deterministic-failure rerun / disposition.**

  | Failure | Root cause | Disposition |
  |---|---|---|
  | Initial integration test run failed 4/5 and poisoned the test lock. | Author-DSL handler syntax in the fixture used textual-IR-like `on clicked`; real DSL syntax is `clicked => { ...; }`. | Fixed fixture syntax; reran targeted handler test and full `iteration_mutation_integration` to green. |
  | Handler append fixture still failed after syntax fix. | Click coordinate `(4,4)` did not necessarily hit the laid-out Button. | Fixture now reads Button Visual offset/size and clicks its center; targeted test green. |
  | `cargo fmt --all -- --check` failed. | Import ordering in the new integration file. | Ran `cargo fmt --all`; final `cargo fmt --all -- --check` green. |

  **Behavior / invariant carry scan.**

  | Behavior / invariant | Disposition |
  |---|---|
  | Positional un-keyed identity is now observable under mutation: retained generated child pointers survive tail append and same-length reset. | **Closed in T7** for tail-only mutation. Reorder/keyed identity remains out of scope per ADR and framing; no new owner needed. |
  | Handler collection assignment is whole-value only and excludes bare collection copy. | **Closed in T7.** Runtime evaluator mirrors the T3/T6 deferral; whole-list copy remains deferred to the framing owner, not silently opened. |
  | Stage-then-commit currently builds child-owned effects during staging, then either commits the staged child or destroys it on staging failure. | **Closed for T7 behavior.** No owner-less residual: if a later task introduces more fallible staging after partial commit, re-trigger trap #2/#6 and add rollback proof. |
  | Reactive-drain residuals 1-3 (cycle detection policy, ordering ties, fan-out x cap) remain broader scheduler policy. | **Carried with owner.** T7 closes item 4 (synchronous non-batched drain proof for range mutation) and records breadth > cap convergence; T10/phase handoff carries DD-M3-P7-007 residual rows verbatim with their triggers. |
  | Assistant-visible cardinality proof is still absent. | **Owner = T8.** Scope: gallery N -> append -> remove screenshots; T7 only supplies runtime/headless evidence. |
  | Owner human-visible smoke is still absent. | **Owner = T9.** Scope: manual gallery smoke after T8. |

  **Carry-forward ownership.**

  | Carry-forward | Owner task | Scope | Impact | Close condition |
  |---|---|---|---|---|
  | DD-M3-P7-007 reactive-drain residuals 1-3 | T10 / phase handoff | Scheduler policy beyond T7's preserved synchronous drain and breadth fixture | Future structural fan-out work must not silently reinterpret scheduler policy | T10 records the verbatim carry rows and triggers in `handoff.md` / phase-close log. |
  | Assistant-visible collection-cardinality positive control | T8 | Gallery `.ui`, build/launch, screenshots | Runtime mutation is implemented but not yet human-visible evidence | T8 records 2+ frame screenshot evidence. |
  | Owner manual GUI smoke | T9 | Owner-run gallery smoke | Assistant evidence does not replace owner-visible check | T9 records owner acceptance or failure observation. |
  | Phase-end spec/architecture re-sync | T10 | Moment 2 docs | Implementation details such as runtime list setter names and landed diagnostics need final doc check | T10 re-syncs docs or records no divergence. |

  **Verification runs.**

  ```text
  cargo test -p wasamo-runtime --test iteration_mutation_integration
  cargo test -p wasamo-runtime
  cargo fmt --all -- --check
  cargo test --workspace
  ```

  All completed successfully on 2026-06-16. `cargo test --workspace`
  still emits the pre-existing package `wasamo` linkable-target warning;
  no test failed.

- **2026-06-16 / T7 start gate — reactive range mutation opened.**
  Started by reading the prior carry-forward rows in this log before
  treating [plan.md](./plan.md) as a hypothesis. T6 closed static
  materialisation and per-item binding registration, but deliberately
  left the structural `ForLoopSubtree` effect, handler-side collection
  writes, mutation-time guard/disposal, and cap convergence to T7. T8
  remains the owner of assistant-visible gallery screenshots; T7 owns
  mock-free Windows runtime evidence only.

  **Carry-over checked from prior tasks.**

  | Carry-over | T7 disposition hypothesis |
  |---|---|
  | T1 CF-3 / T6 carry: `BindingTarget::ForLoopSubtree` and the structural `for` effect do not exist yet. | **T7 owns.** Add the target/effect, preserve static children on the initial effect run, and execute tail insert/remove on later collection changes. |
  | T1 CF-4 / T6 carry: guarded loop-local reads are proven for static registration but not same-batch doomed bindings. | **T7 owns mutation-time proof.** Reuse `ForItemEvalContext` for staged inserted children and fire the out-of-range skip under removal. |
  | T1 CF-5 / T2 carry: collection equal-value no-dirty semantics have no production caller. | **T7 owns.** Handler collection writes must use `Signal::set_if_changed`; empty `drop-last` and same-value literal reset must return no change / no dirty rerun. |
  | T1 CF-6 / T3 carry: `xs = xs.append(e)` / `xs = xs.drop-last()` / list literal collection assignment still runtime-rejects. | **T7 owns.** Extend `HandlerEvalContext` / handler evaluator so authored handlers can mutate whole-value collection signals. |
  | T5/T6 carry: child-carried ZStack placement exists for static generated children, but mutation-time range splice is unproven. | **T7 owns.** Staged range children must carry placement as child metadata and preserve Visual sibling order through the splice. |
  | T6 carry: GUI-visible cardinality proof absent. | **Not T7.** Owner remains **T8**; T7 records headless/runtime evidence only. |

  **Critical responsibility re-check.**

  | Candidate responsibility | Decision before implementation |
  |---|---|
  | Collection handler writer | **In T7.** The structural effect needs a real authored writer to drive collection signals; deferring it would make T7 rely on test-only state mutation and leave Add/Remove impossible. |
  | Unified splice seam including conditional routing | **In T7, but conservative.** Route both conditional and `for` through one insertion/removal helper for side-effect ownership. Keep conditional observable behavior unchanged. |
  | Static `for` materialisation | **Out of T7.** Closed in T6; T7 only proves the new effect's initial run is a no-op against that already-materialised count. |
  | Gallery `.ui` and screenshot positive controls | **Out of T7.** Owner remains T8, including the structured-item trigger decision and 2+ frame evidence. |
  | New language/spec sync | **Out of T7.** T10 owns Moment 2 spec/architecture sync unless T7 uncovers an implementation divergence that must be carried. |

  **Selected traps and non-applicable reasons.**

  | Trap | Applies? | Reason / close artifact hypothesis |
  |---|---|---|
  | #1 semantic migration | **Applies.** | `BindingTarget` gains `ForLoopSubtree`; `HandlerExpr` collection assignment/list expressions stop being unconditional runtime rejects. Close with an `rg`-enumerated call-site audit for `BindingTarget`, `ForLoopSubtree`, `HandlerExpr::List*`, collection signals, and widget-only/control-flow filters touched by runtime mutation. |
  | #2 side effects | **Applies.** | T7 mutates the live widget tree. Close with the DD-M3-P7-006 six-item side-effect bundle: child splice, Visual sibling order, layout invalidation, registry release/registration, effect disposal/attach timing, and no other parent-owned metadata. |
  | #3 parallel / derived data drift | **Applies.** | `ForLoopRuntimeState.live_children`, `declared_slots`, materialised child range, dirty effect dependencies, and child-carried placement must move atomically with the splice. Close with a sync table. |
  | #4 untested authored branch | **Applies.** | T7 adds semantic / size / diagnostic branches: append, drop-last empty no-op, same-length reset, tail insert/remove, same-batch out-of-range skip, cap breadth, staging failure disposition if feasible. Each branch needs a direct test or explicit owner/disposition. |
  | #5 carry-forward | **Applies.** | Known residuals include reactive-drain items 1-3 and possible fault-injection infeasibility. Close with owner/scope/impact/close condition for every remaining item. |
  | #6 deterministic failure | **Standing, not pre-selected.** | No recurring failure exists before implementation. If a deterministic or >=2x recurring failure appears, rerun/root-cause/disposition becomes required before close. |
  | #7 weak GUI evidence | **Not applicable to T7.** | T7's deliverable is runtime/headless structural behavior. Assistant-visible screenshot + positive-control evidence is T8-owned. |

  **Review lane.** Full independent review. Reason: T7 is a runtime
  structural change and also adds semantic/diagnostic branches, so the
  full review must include the branch/test-focused trap-#4 check.

  **Planned proof obligations before implementation.**

  | Planned branch / behavior | Category | Hypothesis before implementation |
  |---|---|---|
  | Handler `Assign` to collection LHS with `ListAppend` mutates the matching whole-value collection signal. | semantic branch | Direct unit/integration test fires authored handler evaluation and observes collection change. |
  | Handler `Assign` to collection LHS with `ListDropLast` on empty collection is equal-value and does not dirty dependents. | semantic / invariant branch | Direct fixture counts no structural rerun / unchanged child pointers. |
  | Handler literal reset with same length re-evaluates retained item bindings in place and preserves prefix pointers. | semantic / observable invariant | Direct runtime fixture changes text values without structural child replacement. |
  | `ForLoopSubtree` initial effect run preserves T6 static children. | size / semantic branch | Load fixture proves no double-create after registering the structural effect. |
  | Tail append inserts generated children at the seam-computed offset with static / `if` siblings in declared order. | size / observable behavior | Windows runtime fixture checks child text order + Visual order. |
  | Tail removal disposes removed subtrees tail-first and releases registry/effect ownership. | semantic / invariant | Runtime fixture uses destroy-counted signal registration and/or effect-observable stale writes. |
  | Same-batch dirty removed-item binding reads out-of-range and skips without panic/write. | semantic branch | Direct fixture dirties a doomed item binding before tail removal in the same batch. |
  | ZStack range mutation preserves child-carried placement and Visual order. | observable behavior | Runtime fixture runs layout after append and checks aligned generated child offsets. |
  | Gallery-scale and >`MUTATION_CAP` breadth converge without divergence. | size / scheduler invariant | Cap fixture appends many items through one collection write and verifies runtime remains healthy. |
  | Stage-then-commit construction failure leaves the tree unchanged. | diagnostic / invariant | Attempt mock-free fault injection if feasible; otherwise record why no direct production branch can be fired yet and assign an owner/disposition. |

  **Known carry-forward candidates before implementation.**

  | Candidate | Owner / scope / impact / close condition |
  |---|---|
  | Reactive-drain residuals 1-3 from DD-M3-P7-007. | **T7 records carry.** Scope: scheduler policy beyond the preserved synchronous drain item 4. Impact: not fully solved by range mutation. Close condition: copied with triggers to the T7 close record / phase handoff owner. |
  | Stage-failure branch may be hard to fault-inject without mocking WinRT construction. | **T7 determines.** Scope: PF2 rollback evidence. Impact: missing direct production failure proof if no natural fault surface exists. Close condition: either direct test, pure planner test + explicit infeasibility disposition, or owner-confirmed follow-up. |
  | Assistant-visible cardinality proof. | **T8.** Scope: gallery screenshot positive controls. Impact: T7 runtime evidence is not human-visible proof. Close condition: T8 screenshots and analysis. |

- **2026-06-15 / T6 rereview closure — residual placement rows
  classified.** The second independent review found no merge blocker
  and left two low-priority questions. Both were re-checked against
  DD-M3-P7-007 and `docs/dsl_spec.md` §4.15 before close.

  **Source enumeration used for this closure.**

  ```text
  rg -n "for_member_rejects_component_body_surface_at_parse|validate_rejects_if_with_nested_control_flow_body|for_member_parses_binders_collection_and_body|for_member_rejects_direct_disallowed_containers|unexpected token in component body|validate_direct_for_parent" wasamo-runtime\src\ir_loader.rs
  ```

  **Rereview residual disposition.**

  | Residual | Disposition |
  |---|---|
  | `for` nested through `if` under Box/ScrollView might bypass the direct-container reject | **Closed in T6 by existing loader gate, now explicitly pinned.** Phase 6 validation rejects a control-flow member directly inside an `if` body before Box/ScrollView materialisation; `validate_rejects_if_with_nested_control_flow_body` now includes `node Box { if true { for x in xs { ... } } }` as a direct `nested control-flow` subcase. |
  | Root / component-level `for` admit-vs-reject ambiguity | **Classified, with a direct parse test.** Component body-level `for` remains rejected by the textual-IR component parser (`unexpected token in component body`) and is pinned by `for_member_rejects_component_body_surface_at_parse`. A `for` inside the root widget's body (for example `node WrapPanel { for ... }`) is not component-level and remains admitted; this is pinned by `for_member_parses_binders_collection_and_body`. |

  **Deterministic-failure root cause / disposition.** During this
  closure pass, a too-broad attempted fix treated `ParentKind::Root` as
  component-level and rejected every direct `for` inside the root
  widget. That failed `for_member_parses_binders_collection_and_body`
  and the Box/ScrollView reject test. Root cause: in runtime textual IR
  the component has exactly one root node; `ParentKind::Root` describes
  that node's parent, not a component-body member slot. The attempted
  reject was reverted, and the actual component-body surface is covered
  as a parse reject instead.

  **Implemented-branch test map addendum.**

  | Implemented branch / behavior | Category | Source query / diff cue | Direct test or owner |
  |---|---|---|---|
  | Component body-level `for` cannot appear in textual IR | reject / diagnostic branch | `rg` hits: `unexpected token in component body`, `for_member_rejects_component_body_surface_at_parse` | `ir_loader::tests::for_member_rejects_component_body_surface_at_parse` |
  | Root widget body-level `for` under an admitted container is not component-level | semantic / owner-boundary branch | `rg` hit: `for_member_parses_binders_collection_and_body` | `ir_loader::tests::for_member_parses_binders_collection_and_body` |
  | `if` body cannot directly contain `for`, including inside Box | reject / diagnostic branch | `rg` hit: `validate_rejects_if_with_nested_control_flow_body` | `ir_loader::tests::validate_rejects_if_with_nested_control_flow_body` subcase `node Box { if true { for ... } }` |

  **Verification run after rereview closure.**

  ```text
  cargo fmt --all
  cargo test -p wasamo-runtime
  ```

  Both completed successfully on 2026-06-15.

- **2026-06-15 / T6 review remediation — i32 item-read proof gap
  closed.** Claude's independent review found one real T6 defect and a
  matching close-gate hole: `HandlerExpr::ItemRead` in string/interp
  binding evaluation always used the string item reader, so `for n in
  nums { Text { text: "\{n}" } }` validated but failed at runtime with
  an i32 item. T6 now dispatches string-like item binding reads through
  `read_item_binding_tracked`, which stringifies i32 loop items, keeps
  string loop items as strings, and still rejects bool loop items in
  string display contexts.

  **Source enumeration used for this review remediation.**

  ```text
  rg -n "read_item_binding_tracked|register_for_item_binding_stringifies_i32_item_value|static_for_materialises_i32_item_and_index_bindings|static_for_composes_with_preceding_conditional_slot_offsets|static_for_under_zstack_preserves_child_carried_placement|loop_local_reads_are_scoped_to_for_body|bare_collection_read_outside_for_header_rejected|static_for_materialises_initial_children_and_loop_local_bindings|static_for_empty_initial_collection_materialises_zero_children" wasamo-runtime\src wasamo-runtime\tests
  git diff -- wasamo-runtime\src\handler.rs wasamo-runtime\src\reactive.rs wasamo-runtime\src\ir_loader.rs wasamo-runtime\tests\ir_loader_roundtrip.rs wasamo-runtime\tests\iteration_static_integration.rs
  ```

  **Review finding disposition.**

  | Finding | Disposition |
  |---|---|
  | A: i32 collection item read was broken and untested | **Fixed.** `EvalContext::read_item_binding_tracked` is now the string-like item-read entry point, and `ForItemEvalContext` dispatches by `IrType`. Direct tests: `reactive::tests::register_for_item_binding_stringifies_i32_item_value` and `iteration_static_integration::static_for_materialises_i32_item_and_index_bindings`. |
  | B: start gate planned string / i32 but close only proved string | **Fixed by this addendum.** The close map below is supplemented with explicit i32 runtime/unit rows; the original T6 close entry should be read together with this remediation entry. |
  | C: missing `validate_loop_local_binding_type` sub-branches | **Fixed.** `ir_loader::tests::loop_local_reads_are_scoped_to_for_body` now directly fires string item -> i32 target and index binder -> bool target rejects. |
  | D: missing production `if` + `for` adjacent slot-offset fixture | **Fixed.** `iteration_static_integration::static_for_composes_with_preceding_conditional_slot_offsets` exercises preceding conditional true/false plus a following static sibling. |
  | E: missing production ZStack + `for` child-carried placement fixture | **Fixed.** `iteration_static_integration::static_for_under_zstack_preserves_child_carried_placement` builds through wasamoc -> textual IR -> runtime loader and checks child text plus layout offsets. |
  | Minor: dead `validate_direct_for_parent` root arm / mostly unused parent | **Fixed.** The unused parent parameter and dead root arm were removed; `cargo test -p wasamo-runtime` is warning-clean. |
  | Minor: close traceability for for-external `list-prop-read` | **Clarified.** Existing direct loader test is `ir_loader::tests::bare_collection_read_outside_for_header_rejected`; the source enumeration above includes it. |

  **Implemented-branch test map addendum.**

  | Implemented branch / behavior | Category | Source query / diff cue | Direct test or owner |
  |---|---|---|---|
  | String-like `item-read` dispatch stringifies i32 loop items instead of forcing the string reader | semantic branch / defect fix | `rg` hits: `read_item_binding_tracked` in `handler.rs` and `reactive.rs`; diff cue: `HandlerExpr::ItemRead` now calls `ctx.read_item_binding_tracked` | `reactive::tests::register_for_item_binding_stringifies_i32_item_value`; `iteration_static_integration::static_for_materialises_i32_item_and_index_bindings` |
  | String item cannot bind to an i32 target property | reject / diagnostic branch | `rg` hit: `loop_local_reads_are_scoped_to_for_body`; diff cue: `element type \`string\`, not \`i32\`` | `ir_loader::tests::loop_local_reads_are_scoped_to_for_body` subcase `string_item_to_i32_target` |
  | Index binder cannot bind to a bool target property | reject / diagnostic branch | `rg` hit: `loop_local_reads_are_scoped_to_for_body`; diff cue: `index binder cannot be used in a bool binding` | `ir_loader::tests::loop_local_reads_are_scoped_to_for_body` subcase `index_read_to_bool_target` |
  | Static `for` composes with preceding `if` slot offsets for both true and false condition branches | size / semantic branch | `rg` hit: `static_for_composes_with_preceding_conditional_slot_offsets` | `iteration_static_integration::static_for_composes_with_preceding_conditional_slot_offsets` |
  | Static generated ZStack children preserve child-carried `h-align:end` / `v-align:start` placement through production loader and layout | observable behavior / invariant | `rg` hit: `static_for_under_zstack_preserves_child_carried_placement` | `iteration_static_integration::static_for_under_zstack_preserves_child_carried_placement` |
  | Textual-IR loop-external collection read remains a loader dual-gate reject | reject / owner-boundary branch | `rg` hit: `bare_collection_read_outside_for_header_rejected` | `ir_loader::tests::bare_collection_read_outside_for_header_rejected` |

  **Behavior / invariant carry scan after remediation.**

  | Behavior / invariant | Disposition |
  |---|---|
  | i32 loop items are display-formattable only in string-like binding contexts; bool loop items still are not display-formattable in T6. | **Closed in T6** for static generated children; T7 reuses the same evaluator for tail-inserted children. |
  | Slot-offset composition across `if` and static `for` is now production-observable before mutation effects exist. | **Closed in T6** for static load; T7 owns mutation-time range updates over the same seam. |
  | ZStack generated children use child-carried placement in production, not just in the static placement reducer. | **Closed in T6** for static load; T7 owns mutation-time ZStack range insertion/removal. |
  | No new unresolved owner-less item was created by the remediation. | Existing carry-forward ownership remains: T7 for structural `ForLoopSubtree`, handler collection writers, tail mutation / disposal / cap fixtures; T8 for assistant-visible gallery screenshots. |

  **Verification run after remediation.**

  ```text
  cargo fmt --all
  cargo test -p wasamo-runtime
  cargo fmt --all -- --check
  cargo test --workspace
  ```

  All four completed successfully on 2026-06-15. `cargo test
  --workspace` still emits the pre-existing package `wasamo` linkable
  target warning; no test failed.

- **2026-06-15 / T6 close gate — loader `for` static
  materialisation landed.** Replaced the deferred runtime-loader
  `ControlFlowNode::For` build reject with static materialisation:
  `append_static_member` now constructs the first production
  `DeclaredMemberSlot::ForLoop`, derives initial cardinality from the
  whole-value collection signal, builds one generated child per initial
  item through the T4 prefix-sum seam, and attaches per-item bindings
  through `ForItemEvalContext`. No `BindingTarget::ForLoopSubtree`
  structural effect or collection writer landed in T6; those remain
  T7-owned per the start-gate split.

  **Source enumeration used for close artifacts.**

  ```text
  git status --short
  git diff -- process\milestone-3\phase-7\implementation\preamble.md process\milestone-3\phase-7\implementation\plan.md process\milestone-3\phase-7\implementation\log.md wasamo-runtime\src\handler.rs wasamo-runtime\src\reactive.rs wasamo-runtime\src\ir_loader.rs wasamo-runtime\tests\ir_loader_roundtrip.rs
  rg -n "ForItemContext|ForItemEvalContext|register_for_item|evaluate_binding_optional|evaluate_bool_binding_optional|evaluate_tracked_optional|static_collection_cardinality|DeclaredMemberSlot::ForLoop|ForLoopRuntimeState|ControlFlowNode::For|validate_phase7_iteration|validate_loop_local_binding_type|loop-local|nested `for`|direct `for`|bool loop binder|iteration_emit_then_parse|static_for_|zstack_static_placement_reducer" wasamo-runtime\src wasamo-runtime\tests process\milestone-3\phase-7\implementation\plan.md process\milestone-3\phase-7\implementation\log.md
  rg -n "register_for_item_binding_writes_item_index_and_skips_out_of_range|register_for_item_bool_binding_tracks_bool_item_value|for_member_rejects_direct_disallowed_containers|for_member_rejects_handler_and_nested_for_inside_template|loop_local_reads_are_scoped_to_for_body|for_member_parses_binders_collection_and_body|for_member_rejects_scalar_collection_target|for_member_rejects_undeclared_collection_target|for_member_rejects_multi_child_body|for_member_rejects_nested_control_flow_body|zstack_static_placement_reducer_expands_for_cardinality_after_t6|iteration_emit_then_parse_preserves_for_member_and_collection_state|static_for_materialises_initial_children_and_loop_local_bindings|static_for_empty_initial_collection_materialises_zero_children" wasamo-runtime\src wasamo-runtime\tests
  ```

  **Trap #1 call-site audit table.**

  | Surface / call-site class | Source query / diff cue | Classification |
  |---|---|---|
  | `ControlFlowNode::For` parse / annotation / validation / render sites | `rg -n "ControlFlowNode::For" wasamo-runtime\src\ir_loader.rs wasamo-runtime\tests` | **Extended / already extended.** Parser and annotation already existed; T6 added the remaining runtime-load behavior and extra validation gates. Test renderer and cross-crate roundtrip preserve the textual shape. |
  | `append_static_member` `ControlFlowNode::For` arm | diff cue: removed deferred build reject; `rg` hits `static_collection_cardinality`, `ForLoopRuntimeState`, `DeclaredMemberSlot::ForLoop` | **Extended.** First production construction and static generated child insertion now live here. |
  | `DeclaredMemberSlot::ForLoop` / `ForLoopRuntimeState` | `rg -n "DeclaredMemberSlot::ForLoop|ForLoopRuntimeState" wasamo-runtime\src\ir_loader.rs` | **Extended.** No longer test-only / production-dead; `live_children` is set from initial collection cardinality at load. |
  | Loop-local `HandlerExpr::ItemRead` / `IndexRead` in validation | `rg -n "LoopReadScope|validate_loop_local_binding_type|loop-local|bool loop binder" wasamo-runtime\src\ir_loader.rs` | **Extended.** Loader now scopes loop-local reads to the current `for` body, rejects missing index binders, and rejects bool item interpolation. |
  | Loop-local runtime evaluation | `rg -n "ForItemContext|ForItemEvalContext|register_for_item|evaluate_.*optional" wasamo-runtime\src` | **Extended for static children.** New guarded optional evaluators and for-item registration entry points cover T6 static-load bindings. |
  | `BindingTarget::ForLoopSubtree` | `rg -n "ForLoopSubtree|BindingTarget" wasamo-runtime\src process\milestone-3\phase-7\implementation\plan.md` | **Deliberately not extended in T6.** Owner remains T7; plan was revised so T7 owns the structural effect and its initial-run no-double-create proof. |
  | Widget-only filters such as `IrNode::widget_children()` | `rg -n "widget_children\\(|ControlFlow\\(_\\)" wasamo-runtime\src wasamo-ir\src` (carried from T2/T4 audit) | **Correctly unaffected.** T6 materialises through `append_static_member`; widget-only filters continue to exclude control-flow bodies unless a validator explicitly recurses. |

  **Trap #2 structural side-effect enumeration.**

  | Derived effect / path | T6 disposition |
  |---|---|
  | Runtime child insertion at static load | **Implemented.** For each initial item, `append_static_member` builds the body with a `ForItemContext` and inserts at `base_index + position`, where `base_index` comes from `materialized_offset_for_declared_slot`. |
  | Visual sibling order | **Preserved.** Generated children use the existing `insert_child` / `insert_child_with_zstack_placement` paths, so the VisualCollection order follows the same child-vector insertion order as static widgets / conditionals. |
  | ZStack placement | **Preserved through child-carried placement.** Generated ZStack children call `insert_child_with_zstack_placement(..., extract_zstack_placement(body))`; no parent-owned ZStack placement vector was reintroduced. |
  | Layout dirty behavior | **No new static-load dirty mark.** Like existing static child construction, initial build establishes the tree before any layout pass; mutation-time invalidation remains T7's splice-seam responsibility. |
  | Binding ownership | **Implemented for static generated children.** Per-item EffectHandles are pushed into the generated child widget's own `bindings` during `build_node_with_loop_context`; parent-owned structural effect remains absent until T7. |
  | Registry / teardown | **No new teardown path in T6.** Static generated children use the same registry references and child-owned EffectHandles as normal bindings. Tail removal / disposal is T7-owned. |

  **Trap #3 parallel / derived-data sync artifact.**

  | Parallel / derived structure | Source query / result | Disposition |
  |---|---|---|
  | `declared_slots` vs generated child range | `rg` hits: `push(DeclaredMemberSlot::ForLoop(...))`, `materialized_offset_for_declared_slot(declared_member_index, &slots)`, `for position in 0..live_children` | **Closed for static load.** The slot is pushed once with `live_children = initial len`, then exactly that many children are inserted at the seam-derived base range. |
  | Collection signal cardinality vs `ForLoopRuntimeState.live_children` | `rg` hit: `static_collection_cardinality(collection, registry)` | **Closed for T6.** The count is read from the typed whole-value collection map at load. Mutation-time count changes remain T7. |
  | Child-carried placement vs generated ZStack children | `rg` hits: `insert_child_with_zstack_placement` in the `For` arm and no `zstack_placements` hits in runtime source | **Closed for T6.** Generated children reuse T5's child-carried placement path. |
  | For-item effect ownership vs generated child subtree | `rg` hits: `register_for_item_binding`, `register_for_item_bool_binding`, `widget.bindings.push(handle)` inside `build_node_with_loop_context` | **Closed for static load.** Generated child roots own their own per-item binding handles. T7 owns removal disposal proof. |

  **Implemented-branch test map.**

  | Implemented branch / behavior | Category | Source query / diff cue | Direct test or owner |
  |---|---|---|---|
  | Textual IR `for` parse/emit/load preserves binder, optional index binder, collection read, body, collection state, and loop-local reads | semantic branch | `rg` hits: `parse_for_member`, `render_node` `ControlFlowNode::For`, `iteration_emit_then_parse_preserves_for_member_and_collection_state` | `ir_loader::tests::for_member_parses_binders_collection_and_body`; `iteration_emit_then_parse_preserves_for_member_and_collection_state` |
  | Static nonempty collection materialises N generated children in declared order with static siblings | size / semantic branch | diff cue: `for position in 0..live_children`; test cue `static_for_materialises_initial_children_and_loop_local_bindings` | `iteration_static_integration::static_for_materialises_initial_children_and_loop_local_bindings` |
  | Empty initial collection materialises zero children and keeps surrounding siblings | size / invariant | diff cue: `ForLoopRuntimeState { live_children }`; test cue `static_for_empty_initial_collection_materialises_zero_children` | `iteration_static_integration::static_for_empty_initial_collection_materialises_zero_children` |
  | First production `DeclaredMemberSlot::ForLoop` construction closes the T4 dead-production allowance | semantic / owner-boundary branch | `rg` hits: `push(DeclaredMemberSlot::ForLoop(Rc::clone(&state)))` | Covered by the two `iteration_static_integration` build fixtures and existing T4 seam unit tests. |
  | Per-item string item + index reads write initial generated Text content | semantic branch | `rg` hits: `ForItemContext`, `ForItemEvalContext`, `register_for_item_binding` | `iteration_static_integration::static_for_materialises_initial_children_and_loop_local_bindings`; `reactive::tests::register_for_item_binding_writes_item_index_and_skips_out_of_range` |
  | Guarded out-of-range per-item read skips the write | semantic branch / invariant | `rg` hits: `Ok(None) => {}` in for-item registration | `reactive::tests::register_for_item_binding_writes_item_index_and_skips_out_of_range`; mutation-time same-batch removed item remains owner = T7 |
  | Bool item binding evaluates through the bool for-item registration path | semantic branch | `rg` hits: `register_for_item_bool_binding`, `read_item_bool_tracked` | `reactive::tests::register_for_item_bool_binding_tracks_bool_item_value` |
  | Direct `for` under ScrollView / Box / Grid is rejected by loader dual-gate | reject branch | `rg` hits: `validate_direct_for_parent`; test name cue | `ir_loader::tests::for_member_rejects_direct_disallowed_containers` |
  | Handler inside a `for` body and nested `for` inside the template are rejected by loader dual-gate | reject branch | `rg` hits: `handlers inside a \`for\` body`, `nested \`for\`` | `ir_loader::tests::for_member_rejects_handler_and_nested_for_inside_template` |
  | Loop-local reads outside body, missing index binder, and bool item interpolation are rejected by loader dual-gate | reject / diagnostic branch | `rg` hits: `LoopReadScope`, `validate_loop_local_binding_type`, `bool loop binder` | `ir_loader::tests::loop_local_reads_are_scoped_to_for_body` |
  | Existing collection/header/body reject rows remain direct | reject branch | `rg` hits: `for_member_rejects_scalar_collection_target`, `for_member_rejects_undeclared_collection_target`, `for_member_rejects_multi_child_body`, `for_member_rejects_nested_control_flow_body`, collection assignment tests | Existing `ir_loader::tests::*` rows listed by the source query above. |
  | Static ZStack placement reducer expands `for` cardinality after T6 | observable behavior / invariant | `rg` hit: `zstack_static_placement_reducer_expands_for_cardinality_after_t6` | `ir_loader::tests::zstack_static_placement_reducer_expands_for_cardinality_after_t6`; production ZStack placement path is exercised by the static insertion code path and T5 regressions. |
  | `BindingTarget::ForLoopSubtree` structural effect initial run / no double-create after effect registration | not implemented in T6 | Plan/log cue: T6 split; `rg` has no production `ForLoopSubtree` variant | **Owner = T7.** Scope: structural effect + initial reconcile + mutation; impact: T6 proves only pre-effect static single-pass load. |
  | Handler-side collection assignment writer | not implemented in T6 | Plan/log cue: T7 CF-6 bullet; handler evaluator still rejects collection forms | **Owner = T7.** Scope: authored `append` / `drop-last` / literal reset in handlers; impact: T6 static fixtures use defaults, not handler-driven cardinality change. |
  | Tail append/remove, same-length reset, same-batch doomed-binding, cap convergence | not implemented in T6 | Plan/log cue: T7 Windows-runtime and cap fixtures | **Owner = T7.** Scope: reactive range mutation; impact: static load works, but collection changes do not yet change cardinality. |

  **Behavior / invariant carry scan.**

  | Behavior / invariant discovered or created | Disposition |
  |---|---|
  | Static materialisation is now a single-pass loader operation, and no structural `for` effect exists in T6. | **Closed / carried split.** T6 closes pre-effect double-create by construction and integration tests; **T7** must prove no double-create when the structural effect is added. |
  | `ForItemEvalContext` is the static-load per-item binding seam and returns `Ok(None)` for out-of-range items. | **Closed for T6 static load.** **T7** reuses it for tail-inserted children and directly fires the same-batch removed-item guard. |
  | Generated child roots own their per-item binding EffectHandles. | **Closed for T6 static generated children.** **T7** must preserve the ownership on staged tail inserts and prove tail-removal disposal. |
  | Direct `for` admission is loader-gated independently of `wasamoc check`: ScrollView / Box / Grid / Cell reject; normal layout containers admit. | **Closed in T6** with direct parse tests for the reject side and integration build tests for the admitted side. |
  | Static generated ZStack children consume placement as child-carried metadata. | **Closed in T6** at the loader insertion/reducer level; **T7** must prove the mutation-time ZStack splice fixture. |
  | Handler-side collection assignment remains a deliberate runtime reject / non-feature until T7. | **Owner = T7.** Scope: collection writer evaluation; impact: no author Button can mutate cardinality yet; close when T7 handler-driven fixtures pass. |
  | GUI-visible collection cardinality proof is not produced by T6. | **Owner = T8.** Scope: gallery N → append → remove screenshot positive controls; impact: T6 proves headless runtime structure only. |

  **Carry-forward ownership.**

  | Carry-forward | Owner task | Scope | Impact | Close condition |
  |---|---|---|---|---|
  | Structural `ForLoopSubtree` effect and initial-effect no-double-create proof | T7 | Binding target + stage-then-commit tail mutation | Cardinality is static-only until T7 | T7 adds the effect, preserves T6's static children on initial run, and records branch/side-effect proof. |
  | Handler collection writes | T7 | `HandlerEvalContext` / evaluator assignment arm | Add/Remove Buttons cannot drive collection changes yet | T7 direct handler fixtures mutate whole-value collection signals. |
  | Mutation-time per-item guard / disposal | T7 | tail remove / same-length reset / same-batch removed binding | T6 guard is proven at registration level only | T7 Windows fixtures fire same-batch skip and child-owned effect disposal. |
  | Assistant-visible positive control | T8 | Gallery screenshots | No visual proof in T6 | T8 records 2+ frames showing collection-driven N changes. |

  **Verification run.**

  ```text
  cargo fmt --all -- --check
  cargo test -p wasamo-runtime
  cargo test --workspace
  ```

  All three completed successfully on 2026-06-15. `cargo test
  --workspace` emitted the existing warning that package `wasamo`
  provides no linkable target; no test failed.

- **2026-06-15 / T6 start gate — loader static materialisation
  responsibility re-challenged before implementation.** Checked the
  prior carry-over rows in this log, then re-read the Phase 7
  constraints, implementation preamble / plan, and
  [implementation-gates.md](../../../procedures/implementation-gates.md)
  before editing runtime code. The T6 plan was revised before
  implementation: T6 owns textual-IR `for` load, loader dual-gates,
  first production `DeclaredMemberSlot::ForLoop` construction, static
  initial materialisation, and initial per-item binding registration for
  generated children. T6 does **not** own `BindingTarget::ForLoopSubtree`,
  collection handler writes, tail append/remove mutation, cap fixtures,
  or GUI evidence; those remain T7/T8.

  **Carry-over checked from prior tasks.**

  | Carry-over | T6 disposition |
  |---|---|
  | T1 CF-1 / T2 close: loader `ControlFlowNode::For` is a deferred build reject. | **T6 owns.** Replace the reject in `append_static_member` with static materialisation and direct tests. |
  | T1 CF-2 / T4 close: `DeclaredMemberSlot::ForLoop` and `ForLoopRuntimeState` are production-dead until T6. | **T6 owns.** First production construction closes the bounded dead-production allowance. |
  | T1 CF-3: the original plan spoke about a no-op `ForLoopSubtree` effect, but T1 addendum corrected `BindingTarget::ForLoopSubtree` to T7 where it first has a meaningful mutation body. | **Plan revised.** T6 proves static materialisation is single-pass before the structural effect exists; T7 proves no double-create again when the effect lands. |
  | T1 CF-4 / addendum G-1: loop-local reads require new guarded per-item binding registration entry points, not reuse of `register_binding`. | **T6 owns the static-load half.** Initial generated children must resolve `item-read` / `index-read` through `{ collection, elem, position }`; T7 owns same-batch doomed-binding mutation coverage. |
  | T3 close/remediation: textual-IR loop-external `list-prop-read` / member-navigation rows remain loader dual-gates. | **T6 owns.** Direct `parse_ir` validation tests cover the textual-IR surface independent of `wasamoc check`. |
  | T5 close: ZStack placement is child-carried; the old test-only static placement reducer rejected `for` until T6. | **T6 owns.** Static generated children under ZStack must carry placement through the existing `insert_child_with_zstack_placement` path; no parent-owned ZStack vector may reappear. |

  **Responsibility re-check.**

  | Plan hypothesis | T6 decision |
  |---|---|
  | T6 should register a no-op `ForLoopSubtree` effect to satisfy the double-create wording. | Refuted. A dead structural effect adds a runtime target before its first meaningful behavior. T6 closes the static-load half; T7 owns the effect-initial-run half when it introduces `BindingTarget::ForLoopSubtree`. |
  | Per-item binding registration can wait entirely for T7. | Refuted. Static materialisation with `bind text = (item-read item)` cannot be correct unless the initially generated children can evaluate loop-local reads. T6 owns the static registration path; T7 reuses/extends it for mutation. |
  | T6 is only diagnostic / reject work, so branch-focused review is enough. | Refuted. Replacing a build reject with runtime tree materialisation is a runtime structural change. T6 takes full independent review, with the trap #4 branch/test check included. |

  **T6 start-gate selection.**

  | Trap | Applies? | Reason / planned close artifact |
  |---|---|---|
  | #1 semantic migration | **Applies.** | No new enum variant is added, but T6 changes the existing `ControlFlowNode::For` loader classification from deliberate reject to materialised control-flow member. Close with an `rg`-enumerated call-site table over `ControlFlowNode::For`, `DeclaredMemberSlot::ForLoop`, `ForLoopRuntimeState`, loop-local read expressions, and widget-only / control-flow filters. |
  | #2 side effects | **Applies.** | Static load now inserts 0..N generated child subtrees, with Visual order, ZStack child-carried placement, layout dirtiness at initial build, and child-owned binding handles. Close with a structural side-effect enumeration for static append paths and the no-mutation boundary. |
  | #3 parallel data drift | **Applies.** | `declared_slots` / `ForLoopRuntimeState.live_children` must stay in sync with the generated child range and the T4 prefix-sum seam; ZStack placement must remain child-carried. Close with a sync table showing where the slot is pushed and where generated children are appended. |
  | #4 untested authored branch | **Applies.** | T6 adds loader reject branches and semantic/size branches: nonempty initial materialisation, empty initial zero-child live slot, per-element collection cardinality reads, loop-local item/index reads, out-of-range guarded skip, and textual-IR dual-gate rejects. Each needs a direct test or explicit later owner. |
  | #5 carry-forward | **Applies.** | T7 inherits the structural effect / mutation body, collection writers, same-batch doomed-binding guard, cap fixtures, and the effect-initial-run no-double-create proof. Record owner/scope/impact/close condition. |
  | #6 root cause | Standing. | Any deterministic or recurring failure during T6 test runs must be root-caused and recorded rather than retried to green. |
  | #7 GUI evidence | Not applicable. | T6 has no GUI-host rendering deliverable. Mock-free Windows loader/build fixtures may be used as runtime evidence, but screenshot evidence belongs to T8. |

  **Review lane:** full independent review. T6 is a runtime structural
  load-path change, and the full review must include the trap #4
  branch/test-focused check for the loader rejects.

  **Planned proof obligations (implementation-time hypotheses).**

  | Branch / behavior to prove | Category | Planned proof |
  |---|---|---|
  | Textual IR `for` parse/emit/load preserves binder, optional index binder, collection read, and single-widget body. | semantic branch | Existing parser tests plus cross-crate emit → parse roundtrip for authored `for`. |
  | Static nonempty collection materialises exactly N generated children at the declared slot, preserving static sibling order. | size / semantic branch | Mock-free Windows runtime build fixture over static / `for` / static siblings. |
  | Empty initial collection materialises zero children while still constructing a live `ForLoop` slot. | size / invariant | Pure seam/unit assertion if exposed; otherwise build fixture proves zero generated children and no build reject. |
  | Initial generated children resolve loop-local item and index reads from their fixed positions. | semantic branch | Mock-free Windows build fixture reads generated Text content for string / i32 item and index cases. |
  | Guarded out-of-range item read writes nothing. | semantic branch / invariant | Direct registration-level unit test or a T7-owned carry if only observable under same-batch removal. |
  | Loader re-rejects textual-IR-only malformed `for` / collection / loop-local rows independently of `wasamoc check`. | reject / diagnostic branch | Direct `parse_ir` negative tests per row/subcase. |
  | ZStack generated static children consume child-carried placement and preserve Visual order. | observable behavior / invariant | Existing placement path plus a direct loader/build fixture if the code path is distinct enough from T5. |

  **Known carry-forward candidates before implementation.**

  | Carry-forward candidate | Owner / scope / impact / close condition |
  |---|---|
  | `BindingTarget::ForLoopSubtree` and the structural `for` effect are not implemented by T6. | **Owner = T7.** Scope: tail insert/remove effect, stage-then-commit, structural diagnostics. Impact: static initial render works, but collection changes do not yet change cardinality. Close when T7 routes collection signal changes through the splice seam and proves no initial double-create. |
  | Handler-side collection assignment evaluation is still absent. | **Owner = T7.** Scope: `xs = xs.append(e)` / `drop-last` / literal reset in handlers. Impact: T6 tests may set registry state only through test hooks or static defaults, not authored Button mutation. Close when T7 fires handler-driven mutation fixtures. |
  | Same-batch removed-item guarded read and empty-`drop-last` no-dirty behavior are mutation-time cases. | **Owner = T7.** Scope: runtime mutation fixtures. Impact: T6 can prove initial loop-local reads but not doomed-binding behavior under removal. Close when T7 directly fires those branches. |
  | Assistant-visible collection cardinality proof is not T6 evidence. | **Owner = T8.** Scope: gallery N → append → remove screenshots. Impact: T6 build tests prove runtime structure, not human-visible rendering. Close with T8 screenshot positive controls. |

- **2026-06-15 / T5 start gate — ST2 placement migration
  responsibility re-challenged before implementation.** Read the T4
  close/carry rows, the T1 sequencing addendum, the T3 carry-forward
  rows touching T5, the Phase 7 constraints/preamble/plan, DD-M3-P7-006,
  and
  [implementation-gates.md](../../../procedures/implementation-gates.md)
  before editing runtime code. The T5 plan was revised before
  implementation: T5 owns the ZStack child-carried placement migration
  across `WidgetNode` storage/mutation, `LayoutNode` arrange/build-tree
  transfer, and loader static/conditional insertion. It does **not** own
  the T7 unified splice primitive, `ForLoopSubtree`, collection writes,
  or GUI evidence.

  **Carry-over checked from prior tasks.**

  | Carry-over | T5 disposition |
  |---|---|
  | T4 close: ZStack placement storage remains parent-owned parallel data, with owner T5. | **T5 owns.** Delete/migrate the `zstack_placements` storage path and close the trap #3 greppable artifact. |
  | T1 sequencing: T5 must keep the conditional mutation path green under child-carried placement before the unified splice seam exists. | **T5 owns.** Preserve the existing conditional-under-ZStack behavior while changing the storage carrier. |
  | T3/T4 rows for textual-IR `for`, static materialisation, `ForLoopSubtree`, collection writers, guarded reads, and range side effects. | **Not T5.** Owners remain T6/T7. T5 only changes how existing ZStack child placement rides through current static and conditional child mutations. |
  | Grid placement migration trigger from DD-M3-P7-006. | **T5 records but does not migrate.** Grid `cell_placements` stays parallel and static-only with the DD trigger pointer. |

  **T5 responsibility re-check.**

  | Plan hypothesis | T5 decision |
  |---|---|
  | "Move ZStack placement onto the child slot" might imply implementing the T7 splice seam now. | Refuted. T5 changes the carrier and current insert/remove/replace semantics only; the single splice seam and range side-effect bundle remain T7-owned. |
  | Child-carried placement could live only in the layout tree. | Refuted. That would leave runtime mutation and staging still needing a parallel carrier before layout. T5 must carry placement on the runtime child slot and transfer it into `LayoutNode`. |
  | Grid should migrate at the same time for consistency. | Refuted by DD-M3-P7-006. Grid rejects direct structural mutation in Phase 7; `cell_placements` remains static-only and gains the trigger pointer. |

  **T5 start-gate selection.**

  | Trap | Applies? | Reason / planned close artifact |
  |---|---|---|
  | #1 semantic migration | Not applicable. | T5 does not add an IR/schema enum variant or field, nor widen a semantic traversal surface. It changes runtime storage fields and call sites already enumerated by R-C. |
  | #2 side effects | **Applies.** | T5 changes tree mutation storage semantics on the shipped ZStack path. Close with a structural side-effect table for child insertion/removal/replacement, Visual sibling order, layout-tree transfer, layout dirty behavior, and conditional mutation preservation. |
  | #3 parallel data drift | **Applies.** | This is T5's core: remove ZStack's parent-owned parallel placement vector from mutated paths. Close with an `rg`-enumerated table showing `zstack_placements` removed from runtime source and `cell_placements` intentionally static-only with the DD trigger pointer. |
  | #4 untested authored branch | **Applies.** | T5 adds/changes size/semantic branches: ZStack insert with explicit placement, ZStack default placement on placement-free insert, non-ZStack normalization to `None`, ZStack replace preserving placement, and layout arrange reading child-carried placement. Each needs a direct pure or existing Windows regression test owner. |
  | #5 carry-forward | **Applies.** | The T7 splice seam must consume child-carried placement and not reintroduce parallel metadata; Grid's deferred migration trigger remains open. Record owner/scope/impact/close condition. |
  | #6 root cause | Standing. | Any deterministic or recurring failure during T5 regression runs must be root-caused and recorded rather than retried to green. |
  | #7 GUI evidence | Not applicable. | T5 has no GUI-host rendering deliverable; Windows integration fixtures are regression tests, not assistant screenshot evidence. |

  **Review lane:** full independent review. T5 is a runtime structural
  change touching shipped ZStack arrange / loader / conditional mutation
  behavior, and includes the trap #4 branch/test check.

  **Planned proof obligations (implementation-time hypotheses).**

  | Branch / behavior to prove | Category | Planned proof |
  |---|---|---|
  | Runtime child slots carry ZStack placement as ordinary child data, and placement-free parent insertions normalize the carrier to `None`. | semantic branch / invariant | Pure test-module mirror for child-slot mutation state, avoiding WinRT construction. |
  | ZStack insert/appended child default placement is `Center/Center`; explicit loader/conditional placement overrides are preserved. | size / semantic branch | Pure mirror tests plus existing ZStack layout and conditional-under-ZStack Windows regressions. |
  | ZStack removal returns a detached child with no parent-interpreted placement left behind. | semantic branch / invariant | Pure mirror test. |
  | ZStack replacement preserves the old slot's placement on the new child, matching the old parent-owned-vector behavior. | semantic branch / invariant | Pure mirror test. |
  | Layout arrange reads placement from each child rather than a parallel vector. | observable behavior / invariant | Pure `layout.rs` ZStack unit tests updated to set child-carried placements and keep alignment/default branch assertions direct. |
  | Grid `cell_placements` remains parallel and static-only with the DD-M3-P7-006 trigger pointer. | owner boundary / carry-forward | Greppable source audit; existing Grid tests remain green. |
  | Conditional-under-ZStack insertion/removal still applies declared child placement. | observable behavior / invariant | Existing `conditional_zstack_reinsert_uses_declared_placement_metadata` Windows regression. |

  **Known carry-forward candidates before implementation.**

  | Carry-forward candidate | Owner / scope / impact / close condition |
  |---|---|
  | The unified placement-aware splice seam is not implemented by T5. | **Owner = T7.** Scope: splice primitive + `ForLoopSubtree` range mutation. Impact: current conditional path still uses direct insert/remove calls, though those now carry placement structurally. Close when T7 routes structural mutation through one seam and records trap #2 side effects. |
  | T7 must not reintroduce parent-owned ZStack placement metadata while staging generated range children. | **Owner = T7.** Scope: staged generated children and commit. Impact: range mutation correctness depends on carrying placement with staged children. Close when T7 consumes child-carried placement in the splice tests, including the ZStack range fixture. |
  | Grid `cell_placements` remains a parallel vector. | **Owner = future Grid structural-mutation task (triggered by DD-M3-P7-006).** Scope: direct `for` of `Cell`s, conditional `Cell`s, or another parent-owned per-child metadata kind. Impact: safe in Phase 7 because Grid rejects direct `for`; close when the trigger task migrates Grid before admitting structural mutation. |

- **2026-06-15 / T5 close gate — ZStack placement is child-carried.**
  Migrated ZStack placement storage from parent-owned parallel metadata
  to child-slot-carried placement in `wasamo-runtime`: `WidgetNode`
  now carries `Option<ZStackPlacement>` on each child slot,
  `WidgetData::ZStack` stores no placement vector, `build_layout_tree`
  transfers the child carrier into `LayoutNode`, and `arrange_zstack`
  reads each child carrier with `Center/Center` as the default. Loader
  static and conditional ZStack insertion continue to extract placement
  from the authored child and pass it through the existing insertion
  path. Grid remains static-only with its `cell_placements` trigger
  pointer.

  **Source enumeration used for close artifacts.**

  ```text
  git diff -- process\milestone-3\phase-7\implementation\plan.md process\milestone-3\phase-7\implementation\log.md wasamo-runtime\src\widget.rs wasamo-runtime\src\layout.rs wasamo-runtime\src\ir_loader.rs
  rg -n "zstack_placement|child_slot_zstack_placement|replacement_child_zstack_placement|clear_detached_child_zstack_placement|insert_child_with_zstack_placement|remove_child|replace_child|WidgetData::ZStack|LayoutNode::zstack|collect_static_zstack_child_placement_slots|cell_placements|DD-M3-P7-006" wasamo-runtime\src\widget.rs wasamo-runtime\src\layout.rs wasamo-runtime\src\ir_loader.rs
  rg -n "zstack_insert_default_placement_is_centered_on_production_logic|zstack_insert_explicit_placement_is_preserved_on_production_logic|non_zstack_insert_normalizes_child_slot_placement_to_none|zstack_remove_detaches_and_clears_child_slot_placement|zstack_replace_preserves_existing_slot_placement_on_new_child|non_zstack_replace_normalizes_replacement_placement_to_none|zstack_arrange_alignment_overrides|zstack_defaults_to_fill_fill_and_centers_children|conditional_zstack_reinsert_uses_declared_placement_metadata|zstack_rooted_fixture_preserves_live_visual_order_and_clip|zstack_static_placements_follow_materialized_member_order|zstack_static_placement_rejects_for_until_static_materialization_lands" wasamo-runtime\src wasamo-runtime\tests
  rg -n "zstack_placements" wasamo-runtime\src wasamo-runtime\tests
  ```

  **Trap #2 structural side-effect enumeration.**

  | Derived effect / path | T5 disposition |
  |---|---|
  | Runtime child insertion | **Changed storage carrier only.** `insert_child_inner` now sets the incoming child's `zstack_placement` through `child_slot_zstack_placement(is_zstack_parent, placement)`: `Some(explicit-or-centered)` for ZStack parents and `None` otherwise before inserting into `children`. Visual insertion order remains the existing `InsertAtTop` / `InsertBelow` logic. |
  | Runtime child removal | **Changed storage carrier only.** `remove_child` still detaches the child's Visual and removes from `children`; it now clears the removed child's parent-interpreted placement through `clear_detached_child_zstack_placement` instead of removing a parent vector entry. |
  | Runtime child replacement | **Changed storage carrier only.** `replace_child` preserves the old child slot's placement on the new child for ZStack parents through `replacement_child_zstack_placement`, preserving the prior parent-vector behavior; it clears the removed child's carrier. Existing Visual replacement behavior is unchanged. |
  | Layout-tree transfer | **Changed.** `WidgetNode::build_layout_tree` transfers each widget child's carried placement into the corresponding `LayoutNode`; `WidgetData::ZStack` no longer supplies a placement vector. |
  | ZStack arrange | **Changed.** `arrange_zstack` reads `child.zstack_placement.unwrap_or_else(ZStackPlacement::centered)` for each direct child. |
  | Conditional-under-ZStack mutation | **Preserved.** The loader/effect path still calls `insert_child_with_zstack_placement` for ZStack parents and `remove_child` for removal; the existing regression stayed green. |
  | Layout dirty / registry / effect teardown | **Preserved.** T5 did not change the conditional mutation site's `mark_layout_dirty_for`, `widget_destroy`, or effect/registry disposal sequence; T7 owns the unified splice side-effect bundle. |

  **Trap #3 parallel-data sync artifact.**

  | Parallel / derived structure | Source query / result | Disposition |
  |---|---|---|
  | ZStack parent-owned placement vector | `rg -n "zstack_placements" wasamo-runtime\src wasamo-runtime\tests` returned no hits after the migration. | **Closed for T5.** No ZStack parallel placement vector remains on runtime source/test paths. |
  | ZStack child-carried replacement | `rg` hits: `WidgetNode::zstack` stores `WidgetData::ZStack`; `WidgetNode::zstack_placement`; `LayoutNode::zstack_placement`; `arrange_zstack` child read; loader `insert_child_with_zstack_placement` call sites. | **Closed for T5.** Placement now rides with each child slot from runtime mutation into layout. |
  | Grid `cell_placements` | `rg` hits remain in `widget.rs`, `layout.rs`, and `ir_loader.rs`; comments include `DD-M3-P7-006` trigger pointer on the static-only path. | **Intentionally open with owner trigger.** Grid rejects direct structural mutation in Phase 7; migrate before any Grid structural mutation path lands. |

  **Implemented-branch test map.**

  | Implemented branch / behavior | Category | Source query / diff cue | Direct test or owner |
  |---|---|---|---|
  | ZStack child insertion with no explicit placement defaults to `Center/Center` | size / semantic branch | `rg` hits: production helper `child_slot_zstack_placement`; production call `child.zstack_placement = child_slot_zstack_placement(self.is_zstack(), zstack_placement)` | `widget::tests::zstack_insert_default_placement_is_centered_on_production_logic`; arrange-site fallback separately covered by `layout::tests::zstack_defaults_to_fill_fill_and_centers_children` |
  | ZStack child insertion with explicit placement stores that placement on the child slot | semantic branch | `rg` hits: `insert_child_with_zstack_placement`; production helper `child_slot_zstack_placement(true, Some(...))` | `widget::tests::zstack_insert_explicit_placement_is_preserved_on_production_logic`; production build/layout path covered by `zstack_rooted_fixture_preserves_live_visual_order_and_clip` |
  | Placement-free parent insertion normalizes the child carrier to `None` even if a placement value is supplied internally | semantic branch / invariant | `rg` hit: production helper `child_slot_zstack_placement(false, Some(...))` | `widget::tests::non_zstack_insert_normalizes_child_slot_placement_to_none` |
  | ZStack removal clears the removed child's parent-interpreted placement | semantic branch / invariant | `rg` hits: production helper `clear_detached_child_zstack_placement`; production calls in `remove_child` and old-child half of `replace_child` | `widget::tests::zstack_remove_detaches_and_clears_child_slot_placement`; conditional removal behavior covered by `conditional_zstack_reinsert_uses_declared_placement_metadata` |
  | ZStack replacement preserves old slot placement on the new child and clears the old child | semantic branch / invariant | `rg` hits: production helper `replacement_child_zstack_placement`; production call `new_child.zstack_placement = replacement_placement`; clear helper for old child | `widget::tests::zstack_replace_preserves_existing_slot_placement_on_new_child`; `widget::tests::non_zstack_replace_normalizes_replacement_placement_to_none` |
  | Widget layout-tree transfer copies child-carried placement into `LayoutNode` | semantic branch / invariant | `rg` hit: `layout_node.zstack_placement = self.zstack_placement`; diff cue removes `LayoutNode::zstack(vec![...])` construction | Production transfer is covered by `conditional_zstack_reinsert_uses_declared_placement_metadata` and `zstack_rooted_fixture_preserves_live_visual_order_and_clip`; `layout::tests::zstack_arrange_alignment_overrides` covers arrange read only. |
  | ZStack arrange reads placement from the child slot and defaults missing placement to centered | observable behavior / invariant | `rg` hit: `child.zstack_placement.unwrap_or_else(ZStackPlacement::centered)` in `arrange_zstack` | `layout::tests::zstack_arrange_alignment_overrides`; `layout::tests::zstack_defaults_to_fill_fill_and_centers_children`; `layout::tests::zstack_shrink_measure_uses_child_union_with_fill_child_zero` |
  | Loader static ZStack child placement extraction still follows materialized member order | observable behavior / invariant | `rg` hit: test-only reducer `collect_static_zstack_child_placement_slots`; production static path is the `append_static_member` + `insert_child_with_zstack_placement` path | Reducer branch: `ir_loader::tests::zstack_static_placements_follow_materialized_member_order`; production integration: `zstack_rooted_fixture_preserves_live_visual_order_and_clip` |
  | Loader static placement still rejects `for` until T6 materialisation | reject / owner boundary | `rg` hit: `ControlFlowNode::For { .. }` branch in the test-only reducer; production loader still rejects `for` materialisation before this placement path per the T2/T4 boundary | `ir_loader::tests::zstack_static_placement_rejects_for_until_static_materialization_lands`; owner for production `for` load remains T6 |
  | Conditional-under-ZStack insertion/removal still applies declared placement after storage migration | observable behavior / invariant | `rg` hits: `zstack_placement_for_parent`; `insert_child_with_zstack_placement`; `remove_child` | `conditional_zstack_reinsert_uses_declared_placement_metadata`; also covered by `cargo test -p wasamo-runtime` integration run |
  | Unified placement-aware splice seam and range mutation side-effect bundle | not implemented in T5 | Plan/log source cue: T5 explicitly does not own `ForLoopSubtree` or splice primitive | **Owner = T7.** Scope: splice primitive + `ForLoopSubtree`; impact: current conditional path still uses direct insert/remove calls; close when T7 routes structural mutation through one seam and records trap #2. |
  | Grid child-carried placement migration | not implemented in T5 | `rg` hits: `cell_placements`; DD-M3-P7-006 trigger pointer in comments | **Owner = future Grid structural-mutation task.** Scope: direct `for` of `Cell`s / conditional `Cell`s / new parent-owned metadata; impact: safe while Grid is static-only; close before any Grid structural mutation path lands. |

  **Behavior / invariant carry scan.**

  | Behavior / invariant discovered or created | Disposition |
  |---|---|
  | ZStack no longer has a parent-owned placement vector; child placement is part of the child slot and is cleared on detach. | **Closed in T5** for current static, conditional, layout, and mutation paths. |
  | `replace_child` under ZStack preserves the old slot's placement on the replacement child. | **Closed in T5.** This preserves the previous parent-vector behavior even though no current ZStack production path depends on replacement. |
  | Placement-free parents normalize child-carried ZStack placement to `None`. | **Closed in T5.** Prevents stale parent-interpreted placement from riding through non-ZStack attachment. |
  | Default placement is now applied at two layers: insertion normalizes ZStack child slots to `Some(Center/Center)`, while `arrange_zstack` still defaults `None` to centered. | **Closed / recorded in T5.** The insertion default is the production ZStack child path and has direct helper tests; the arrange fallback is defensive for manually constructed `LayoutNode`s and is covered by layout tests. |
  | DD-M3-P7-006's open value space was narrowed to a per-container optional child field: `zstack_placement: Option<ZStackPlacement>`, not a shared placement enum. | **Closed / recorded in T5.** This is the chosen implementation shape for ZStack. Future Grid migration may reuse the pattern but must still make its own triggered migration decision before admitting structural mutation. |
  | `build_layout_tree` copies `zstack_placement` on every widget node, but the field is parent-interpreted and meaningful only for direct children of a ZStack parent. | **Closed / recorded in T5.** The invariant is documented on `WidgetNode` / `LayoutNode`; placement-free parents normalize the runtime field to `None`, and non-ZStack layout code ignores the layout field. |
  | T7 staging must carry placement as ordinary child data and must not recreate ZStack parent metadata. | **Owner = T7.** Scope: staged range children + splice commit. Impact: reintroducing parallel metadata would reopen trap #3; close when T7's ZStack range mutation fixture proves placement and Visual order through the splice. |
  | Grid remains SoA/static-only with `cell_placements`. | **Owner = future Grid structural-mutation trigger task.** Scope: Grid admitting structural mutation. Impact: no Phase 7 range mutation crosses Grid; close condition is migration before the trigger path is built. |
  | The unified splice seam is still absent after T5. | **Owner = T7.** Scope: single structural mutation seam and six side effects. Impact: existing conditional path remains direct insert/remove until T7; close when the conditional path and `for` ranges route through the seam. |

  **Carry-forward ownership.**

  | Open point | Owner task | Scope | Impact | Close condition |
  |---|---|---|---|---|
  | T7 splice seam must consume child-carried placement | T7 | Structural splice + `ForLoopSubtree` | Range mutation depends on placement riding with staged children | T7 implements the seam without parent-owned ZStack metadata and proves the ZStack range fixture. |
  | Conditional path still uses direct insert/remove calls | T7 | Structural side-effect consolidation | Current behavior is green, but side effects are not yet centralized | T7 routes conditional and `for` mutation through the single seam and records the DD-006 side-effect table. |
  | Grid `cell_placements` is still parallel | Future Grid structural-mutation trigger task | Direct `for` of `Cell`s, conditional `Cell`s, or new parent-owned child metadata | Safe in Phase 7 because Grid is static-only; would drift if mutation is admitted first | Trigger task migrates Grid to child-carried placement before building the mutation path. |

  **Verification.** `cargo fmt --all`, `cargo fmt --all -- --check`,
  `cargo test -p wasamo-runtime`, and `cargo test --workspace` were
  green. The workspace run emitted the existing `wasamo` linkable-target
  warning but no failures. No deterministic or recurring failure was
  observed, so trap #6 required no rerun disposition.

  **Independent-review follow-up disposition.** The review found no
  defect in the ST2 storage migration itself, but flagged insufficient
  proof structure and over-broad branch-map ownership. T5 follow-up
  addressed the review points:

  | Review point | Disposition |
  |---|---|
  | Mirror tests duplicated the placement state logic instead of testing production logic. | **Closed.** Extracted `child_slot_zstack_placement`, `replacement_child_zstack_placement`, and `clear_detached_child_zstack_placement`; production code and unit tests now call the same helpers. Removed the placement mirror structs. |
  | Insert-time default placement was not directly pinned. | **Closed.** Added `widget::tests::zstack_insert_default_placement_is_centered_on_production_logic` and split explicit placement into `widget::tests::zstack_insert_explicit_placement_is_preserved_on_production_logic`. |
  | Branch map credited `layout::tests::zstack_arrange_alignment_overrides` for layout-tree transfer, but that test constructs `LayoutNode` directly. | **Closed.** Branch map now assigns transfer proof to `conditional_zstack_reinsert_uses_declared_placement_metadata` and `zstack_rooted_fixture_preserves_live_visual_order_and_clip`; the layout unit is listed only for arrange read. |
  | Test-only static placement reducer was not distinguished from production loader coverage. | **Closed.** Branch map now labels `collect_static_zstack_child_placement_slots` as a test-only reducer and separately names production integration coverage. |
  | Double default site and per-container field choice were not recorded. | **Closed.** Behavior / invariant carry scan now records insertion-default vs arrange fallback and the chosen per-container `zstack_placement` field shape. |

- **2026-06-14 / T4 start gate — C1 seam responsibility
  re-challenged before implementation.** Read the T3 close/carry rows,
  the T3 owner-follow-up addendum, the T2 carry-forward rows, the T1
  Seam B record, the Phase 7 constraints/preamble/plan, and
  [implementation-gates.md](../../../procedures/implementation-gates.md)
  before editing runtime code. The T4 plan was revised before
  implementation: T4 owns the pure declared-slot expansion seam and the
  Phase 6 conditional migration onto that seam; it does **not** own
  textual-IR `for` load/materialisation (T6), `ForLoopSubtree` effects
  or collection writers (T7), ZStack placement migration (T5), or GUI
  evidence (T8/T9).

  **Carry-over checked from prior tasks.**

  | Carry-over | T4 disposition |
  |---|---|
  | T1 Seam B: `DeclaredMemberSlot::ForLoop` must exist before T6 so the C1 seam can prove `For` cardinality in pure logic, but production loader construction remains T6-owned. | **T4 owns the variant + test-only construction.** Record the dead-production allowance as carry-forward to T6. |
  | T1/T3/T2 carry rows for static `for` materialisation, textual-IR loader dual gates, guarded loop-local reads, `ForLoopSubtree`, collection writer evaluation, and `set_if_changed` production use. | **Not T4.** Owners remain T6/T7. T4 only computes offsets/counts/plans from already-known slot cardinalities. |
  | R-B: C1 touches the shipped Phase 6 conditional path. | **T4 owns.** Migrate conditional insertion/removal index calculation onto the shared seam and run the Phase 6 declared-order regression fixture unchanged. |
  | R-C / ST2 placement storage migration. | **Not T4.** T5 owns child-carried placement; T4 must preserve the existing ZStack conditional placement call path while only changing index calculation. |

  **T4 responsibility re-check.**

  | Plan hypothesis | T4 decision |
  |---|---|
  | "Per-member live cardinality (For = collection length)" might imply T4 should read collection signals or materialise `for` children. | Refuted. T4 accepts cardinality as runtime slot state; T6/T7 own deriving that cardinality from collection signals and constructing/destroying subtrees. |
  | Tail insert/remove plan derivation could belong to T7 with the splice primitive. | Partially refuted. T7 owns executing the plan and side effects, but T4 owns the pure old-length/new-length range plan so T7 consumes a tested seam. |
  | The conditional path can keep using a bespoke materialised-index helper. | Refuted. C1 is the canonized seam; the existing conditional 0/1 path migrates now so later `ForLoop` does not fork offset logic. The old thin wrapper was removed in the review follow-up. |

  **T4 start-gate selection.**

  | Trap | Applies? | Reason / planned close artifact |
  |---|---|---|
  | #1 semantic migration | **Applies.** | T4 adds `DeclaredMemberSlot::ForLoop` and replaces bespoke offset math with a declared-slot expansion seam. Close with an `rg`-enumerated call-site table over `DeclaredMemberSlot`, `materialized_offset_for_declared_slot`, range-planner helpers, and conditional mutation call sites. |
  | #2 side effects | **Applies.** | The shipped conditional insert/remove path is a tree-structure mutation. T4 intends to change only offset calculation, but must enumerate the preserved derived effects: child insert/remove, ZStack placement lookup, widget_destroy/registry/effect teardown, layout dirty, and Visual sibling order through the existing widget primitive. |
  | #3 parallel data drift | Not applicable. | T4 does not add or migrate parent-owned parallel placement metadata, derived indices, or caches; ZStack placement remains T5-owned and current metadata paths are preserved. |
  | #4 untested authored branch | **Applies.** | T4 adds pure size/range branches: widget / conditional / for cardinality, boundary offsets, total count, tail insert, tail remove, and no-op/equal-length plans. Each branch must have a directly firing unit test. |
  | #5 carry-forward | **Applies.** | `ForLoop` remains unconstructed in production until T6; T7 consumes the tail plan but owns splice side effects. Both must be recorded with owner, scope, impact, and re-trigger. |
  | #6 root cause | Standing. | Any deterministic or recurring failure during the T4 regression runs must be root-caused and recorded rather than retried to green. |
  | #7 GUI evidence | Not applicable. | T4 has no GUI-host rendering deliverable; Windows integration fixtures are regression tests, not assistant screenshot evidence. |

  **Review lane:** full independent review. T4 is a runtime structural
  refactor of the shipped conditional path, even though its new seam is
  pure logic and adds no GUI deliverable.

  **Planned proof obligations (implementation-time hypotheses).**

  | Branch / behavior to prove | Category | Planned proof |
  |---|---|---|
  | Declared-slot live cardinality: widget = 1, absent/present `If` = 0/1, `ForLoop` = current length | size / semantic branch | Pure unit tests with interleaved static / conditional / `ForLoop` slots. |
  | Prefix materialised offsets and total count recompute from current slot state, with no cached offset | size / invariant | Pure unit tests mutate conditional/for slot cardinality and re-query offsets/counts. |
  | Tail plan: old < new inserts `[old, new)`, old > new removes `[new, old)` tail-first, old == new no-op | size / semantic branch | Direct pure unit tests for insert/remove/no-op and boundary zero-length cases. |
  | Phase 6 conditional insertion/removal still computes declared-order live index through the seam | observable behavior / invariant | Existing Phase 6 declared-order Windows fixture runs unchanged; pure unit covers conditional 0/1 offset. |
  | T2 deferred `for` load reject remains until T6 | reject branch / owner boundary | Existing loader reject tests remain green; no T4 code should construct production `ForLoop` slots. |

  **Known carry-forward candidates before implementation.**

  | Carry-forward candidate | Owner / scope / impact / close condition |
  |---|---|
  | `DeclaredMemberSlot::ForLoop` is test-constructed only after T4. | **Owner = T6.** Scope: loader static materialisation. Impact: production `for` remains deferred-load reject until T6. Close when T6 constructs `ForLoop` slots from textual-IR `for` members and removes/justifies the dead-production allowance. |
  | Tail range plan is pure only; it does not splice children or update Visual/layout/registry/effects. | **Owner = T7.** Scope: splice seam + `ForLoopSubtree` effect. Impact: no runtime range mutation yet. Close when T7 consumes the plan in the placement-aware stage-then-commit splice and records trap #2 side-effect proof. |
  | ZStack placement storage remains parent-owned parallel data during T4. | **Owner = T5.** Scope: ST2 child-carried placement migration. Impact: T4 must preserve existing conditional placement behavior, but does not solve range placement drift. Close when T5 deletes/migrates `zstack_placements` per its trap #3 artifact. |

- **2026-06-14 / T4 close gate — C1 seam canonized and conditional
  path migrated.** Implemented the runtime declared-slot expansion seam
  in `wasamo-runtime/src/ir_loader.rs`: `DeclaredMemberSlot::ForLoop`
  plus `ForLoopRuntimeState`, live cardinality dispatch, prefix
  materialised offsets, total materialised child count, and pure
  old-length/new-length tail range planning. The existing Phase 6
  conditional mutation path now reaches its live insertion/removal index
  through the seam. Production `for` construction remains T6-owned and
  production range splicing remains T7-owned.

  **Source enumeration used for close artifacts.**

  ```text
  git diff -- process\milestone-3\phase-7\implementation\plan.md process\milestone-3\phase-7\implementation\log.md wasamo-runtime\src\ir_loader.rs
  rg -n "DeclaredMemberSlot|ForLoopRuntimeState|TailRangePlan|declared_slot_live_cardinality|materialized_offset_for_declared_slot|total_materialized_children|plan_tail_range_change|mutate_conditional_subtree|insert_child_with_zstack_placement|remove_child|widget_destroy|mark_layout_dirty_for" wasamo-runtime\src\ir_loader.rs
  rg -n "expansion_seam_|tail_range_plan_|conditional_toggle_preserves_declared_visual_order|conditional_zstack_reinsert" wasamo-runtime\src wasamo-runtime\tests
  rg -n "ControlFlowNode::For \{ .. \}|static materialisation is owned by T6|materialised in T6|ForLoopSubtree|zstack_placements|cell_placements" wasamo-runtime\src wasamo-runtime\tests process\milestone-3\phase-7\implementation\plan.md
  ```

  **Trap #1 call-site audit (`rg`-enumerated).**

  | Surface | Sites classified | Disposition |
  |---|---|---|
  | `DeclaredMemberSlot` variants | enum definition, `append_static_member` pushes (`Widget`, `Conditional`), `mutate_conditional_subtree` lookup, pure tests | **Extended.** Added `ForLoop` and a live-cardinality arm. Production pushes are deliberately still `Widget` / `Conditional` only; T6 owns first production `ForLoop` construction. |
  | Offset calculation | `mutate_conditional_subtree` calls `materialized_offset_for_declared_slot` directly | **Migrated.** The shipped conditional path uses the shared seam as the 0/1 case; the old thin wrapper was removed after independent-review follow-up. |
  | Cardinality / count / tail planner helpers | `declared_slot_live_cardinality`, `total_materialized_children`, `plan_tail_range_change` | **Added.** Pure seam covers widget / conditional / for cardinality, total count, and tail insert/remove/no-op plan derivation. At T4, `total_materialized_children` and `plan_tail_range_change` were unused by production until T6/T7 and carried bounded dead-code allowances; T7 made `plan_tail_range_change` production-live and removed its stale allowance. |
  | Runtime `For` load arm | `append_static_member` `ControlFlowNode::For { .. }` build reject; `collect_static_zstack_placements` reject | **Correctly unaffected.** T4 does not static-materialise `for`; existing T2/T3 reject tests remain green. |
  | Placement metadata | `zstack_placements` / `cell_placements` hits in `widget.rs`, `layout.rs`, `ir_loader.rs` | **Correctly unaffected.** T4 does not migrate placement storage; T5 remains owner. |

  **Trap #2 structural side-effect enumeration.**

  | Conditional mutation derived effect | T4 disposition |
  |---|---|
  | Materialised child insertion/removal index | **Changed only through seam.** `mutate_conditional_subtree` still computes `live_index` immediately before mutation, now directly via `materialized_offset_for_declared_slot`. |
  | Visual sibling order | **Preserved.** Existing `WidgetNode::insert_child` / `insert_child_with_zstack_placement` / `remove_child` calls are unchanged; Phase 6 declared-order fixture remains green. |
  | ZStack placement lookup | **Preserved.** Existing `zstack_placement_for_parent(parent, body)` branch is unchanged; T5 owns storage migration. |
  | Registry/effect teardown on removal | **Preserved.** Existing `crate::widget::widget_destroy(removed)` call is unchanged. |
  | Layout invalidation | **Preserved.** Existing `crate::emit::mark_layout_dirty_for(parent_ptr)` calls after successful insert/remove are unchanged. |
  | Parent-owned parallel placement metadata | **Not changed in T4.** Existing ZStack parallel vector path remains, with owner T5. |

  **Implemented-branch test map.**

  | Implemented branch / behavior | Category | Source query / diff cue | Direct test or owner |
  |---|---|---|---|
  | `DeclaredMemberSlot::Widget` live cardinality = 1 | size branch | `rg` hit: `DeclaredMemberSlot::Widget => 1`; diff adds `declared_slot_live_cardinality` | `ir_loader::tests::expansion_seam_counts_interleaved_widgets_conditionals_and_for_loops`; `ir_loader::tests::expansion_seam_handles_boundaries_and_total_count` |
  | `DeclaredMemberSlot::Conditional` live cardinality = 0/1 and recomputes after state mutation | size branch / invariant | `rg` hit: `Conditional(state) => usize::from(state.borrow().live_child)`; test mutates `toggled.borrow_mut().live_child` | `ir_loader::tests::expansion_seam_counts_interleaved_widgets_conditionals_and_for_loops` |
  | `DeclaredMemberSlot::ForLoop` live cardinality = `live_children`, including zero-cardinality | size branch / invariant | `rg` hits: `ForLoopRuntimeState`, `DeclaredMemberSlot::ForLoop`, `state.borrow().live_children`; diff adds test-only construction | `ir_loader::tests::expansion_seam_counts_interleaved_widgets_conditionals_and_for_loops`; `ir_loader::tests::expansion_seam_handles_boundaries_and_total_count` |
  | Prefix materialised offsets over interleaved static / absent-if / present-if / for slots | semantic branch | `rg` hit: `materialized_offset_for_declared_slot`; diff shows `mutate_conditional_subtree` calls it directly | `ir_loader::tests::expansion_seam_counts_interleaved_widgets_conditionals_and_for_loops` |
  | Boundary offsets: first slot, leading zero-cardinality slot, offset after final declared slot | size branch | `rg` hits: test name `expansion_seam_handles_boundaries_and_total_count`; diff adds offset assertions for indices `0`, `1`, `4` | `ir_loader::tests::expansion_seam_handles_boundaries_and_total_count` |
  | Total materialised child count from current slot state | size branch / invariant | `rg` hit: `total_materialized_children`; diff adds assertions before/after for-loop cardinality mutation | `ir_loader::tests::expansion_seam_counts_interleaved_widgets_conditionals_and_for_loops`; `ir_loader::tests::expansion_seam_handles_boundaries_and_total_count` |
  | Tail growth plan: old < new inserts the new suffix | size / semantic branch | `rg` hit: `TailRangePlan::Insert`; diff cue `plan_tail_range_change(2, 5)` and `plan_tail_range_change(0, 1)` | `ir_loader::tests::tail_range_plan_derives_insert_remove_and_noop_cases` |
  | Tail shrink plan: old > new removes retained-boundary suffix tail-first | size / semantic branch | `rg` hit: `tail_first_indices: (new_len..old_len).rev().collect()`; diff cue `plan_tail_range_change(5, 2)` and `plan_tail_range_change(1, 0)` | `ir_loader::tests::tail_range_plan_derives_insert_remove_and_noop_cases` |
  | Same-length / empty no-op range plan | size / semantic branch | `rg` hit: `TailRangePlan::NoOp`; diff cue `plan_tail_range_change(3, 3)` and `plan_tail_range_change(0, 0)` | `ir_loader::tests::tail_range_plan_derives_insert_remove_and_noop_cases` |
  | For-slot absolute insertion index composes declared-slot base offset with for-local tail plan | semantic branch / invariant | `rg` hit: `expansion_seam_composes_for_slot_offset_with_tail_plan`; diff cue `[Widget, ForLoop(2), Widget]` + `Insert { start: 2, count: 1 }` | `ir_loader::tests::expansion_seam_composes_for_slot_offset_with_tail_plan` |
  | Existing conditional mutation computes declared-order live index through the seam and preserves behavior | observable behavior / invariant | `rg` hits: `mutate_conditional_subtree`, `materialized_offset_for_declared_slot`, `conditional_toggle_preserves_declared_visual_order`, `conditional_zstack_reinsert` | `conditional_toggle_preserves_declared_visual_order_and_disposes_registry`; `conditional_zstack_reinsert_uses_declared_placement_metadata`; also covered by `cargo test -p wasamo-runtime` integration run |
  | Production textual-IR `for` static materialisation remains deferred | reject / owner boundary | `rg` hits: `ControlFlowNode::For { .. }`, `"static materialisation is owned by T6"`, `"materialised in T6"` | `ir_loader::tests::zstack_static_placement_rejects_for_until_static_materialization_lands`; production construction owner = T6 |
  | Production range splice side effects from the tail plan | not implemented in T4 | `rg` hits in plan: `ForLoopSubtree`; diff adds pure `TailRangePlan` only | **Owner = T7.** Scope: splice seam + `ForLoopSubtree`; impact: no runtime collection mutation yet; close when T7 consumes the plan and records trap #2 side-effect proof. |
  | ZStack child-carried placement migration / parallel-vector deletion | not implemented in T4 | `rg` hits: `zstack_placements`, `cell_placements`; no diff changes these paths | **Owner = T5.** Scope: ST2 placement migration; impact: current placement metadata remains parallel until T5; close when T5 trap #3 artifact lands. |

  **Behavior / invariant carry scan.**

  | Behavior / invariant discovered or created | Disposition |
  |---|---|
  | Declared slot offsets are now a runtime seam over slot cardinality, not a bespoke conditional-only helper. | **Closed in T4** for pure offset/count behavior; T6/T7 must consume the seam rather than reintroducing parallel offset math. |
  | `DeclaredMemberSlot::ForLoop` and `ForLoopRuntimeState` exist but are production-dead until loader static materialisation. | **Owner = T6.** Scope: textual-IR `for` load + static materialisation. Impact: dead-production allowance is intentional; close when T6 constructs `ForLoop` slots and proves empty-initial zero-child member-live behavior. |
  | Tail range planning is pure and does not mutate the tree or Visual/layout/registry/effects. | **Owner = T7.** Scope: splice seam + `ForLoopSubtree` effect. Impact: T4 proves the plan behavior and a minimal enum shape only; close when T7 either consumes this shape or deliberately replaces it with an equivalent seam while preserving the T4 branch tests. |
  | T4 preserved existing conditional ZStack placement calls while placement metadata remains parallel. | **Owner = T5.** Scope: child-carried placement migration. Impact: range placement drift is not solved by T4; close when T5 migrates/deletes `zstack_placements` on mutated paths. |
  | Production `for` build reject remains after T4. | **Owner = T6.** Scope: replace deferred-load reject with static materialisation. Impact: authored/IR `for` still cannot build a runtime tree until T6. |

  **Carry-forward ownership.**

  | Open point | Owner task | Scope | Impact | Close condition |
  |---|---|---|---|---|
  | First production construction of `DeclaredMemberSlot::ForLoop` | T6 | Runtime loader static path | `ForLoop` variant is dead outside tests after T4 | T6 constructs it from validated textual-IR `for` members and removes/updates the dead-code allowance. |
  | Static `for` materialisation and empty-initial zero-child member-live proof | T6 | Loader static materialisation | Runtime still rejects `for` build | T6 replaces the deferred reject and proves static materialisation without double-creation. |
  | Tail plan execution and side-effect bundle | T7 | Splice seam + `ForLoopSubtree` effect | T4 has no runtime range mutation | T7 consumes `plan_tail_range_change` or an equivalent seam, confirms/replaces the minimal `TailRangePlan` shape, and records structural side effects. |
  | ZStack child-carried placement | T5 | ST2 placement storage | Parent-owned `zstack_placements` remains | T5 migration lands with trap #3 greppable artifact. |

  **Independent-review follow-up disposition.** The non-blocking review
  found no implementation defect and requested two minor cleanups plus
  one optional depth pin. T4 follow-up addressed all three:

  | Review point | Disposition |
  |---|---|
  | Optional depth: compose for-slot base offset with local tail plan. | **Closed.** Added `ir_loader::tests::expansion_seam_composes_for_slot_offset_with_tail_plan`, pinning `[Widget, ForLoop(2), Widget]` + `Insert { start: 2, count: 1 }` → absolute index `3`. |
  | Minor contract: `TailRangePlan` carried redundant `Remove.start` and `NoOp.len`. | **Closed for T4 shape.** Simplified to `Remove { tail_first_indices }` and `NoOp`; T7 still owns confirming or replacing the exact consumer-facing shape when it implements the splice. |
  | Minor cruft: `materialized_index_for_declared_member` was a thin pass-through. | **Closed.** Removed the wrapper and pointed `mutate_conditional_subtree` plus tests directly at `materialized_offset_for_declared_slot`. |

  **Trap #6 disposition.** One deterministic failure occurred on the
  first `cargo test -p wasamo-runtime` run:
  `ir_loader::tests::expansion_seam_counts_interleaved_widgets_conditionals_and_for_loops`
  expected offset `4` after setting the preceding `ForLoop` cardinality
  to zero. Root cause: fixture arithmetic, not implementation behavior;
  the correct prefix is `1 + 1 + 0 + 1 + 0 = 3`. The test expectation was
  corrected and the test suite was rerun to green; no retry was treated
  as a flake.

  **Trap #3 / #7 close confirmation.** T4 added no parallel vector,
  derived index cache, or placement metadata migration (#3), and has no
  GUI-render screenshot deliverable (#7).

  **Verification evidence.** `cargo fmt --all -- --check` passed.
  `cargo test -p wasamo-runtime` passed (384 unit tests plus the runtime
  integration fixtures, including
  `conditional_toggle_preserves_declared_visual_order_and_disposes_registry`
  and `conditional_zstack_reinsert_uses_declared_placement_metadata`).
  `cargo test --workspace` passed. **Review lane remains full
  independent review** because T4 refactors the shipped runtime
  structural conditional path.

- **2026-06-14 / T3 start gate — author surface responsibility
  re-challenged before implementation.** Read the T2 close/carry rows,
  the T2 implemented-branch addendum, T1 carry-forward rows, the Phase 7
  preamble/plan, DD-M3-P7-007, and
  [implementation-gates.md](../../../procedures/implementation-gates.md)
  before editing code. The T3 plan was revised because the previous
  wording under-specified the AST and binder-scope threading needed to
  make the author surface auditable. T3 owns only author-reachable
  parser/check/lower/emit behavior; textual-IR-only loader dual gates
  remain T6-owned, and runtime guarded reads / collection writers remain
  T7-owned.

  **Carry-over checked from prior tasks.**

  | Carry-over | T3 disposition |
  |---|---|
  | Author parser/check/lower/emit and full DD-007 compile-time matrix were not finished by T2. | **T3 owns.** Implement the author grammar, diagnostics, lowering, and emit pins. |
  | Loop-local `item-read` / `index-read` scoping is only represented in IR/textual forms after T2. | **T3 owns author-scope diagnostics.** T6/T7 keep loader/runtime context ownership. |
  | Loader scalar defaults became stricter in T2 (`state count: i32 = true` rejects in textual IR). | **T3 awareness.** Author collection/default checks must not weaken the scalar-default expectation. T6 preserves loader gate. |
  | Static `For` materialisation reject, `ForLoopSubtree`, guarded out-of-range runtime read, collection writer, and `set_if_changed` production use. | **Not T3.** Owners remain T6/T7 per CF-1..CF-6. |

  **T3 start-gate selection.**

  | Trap | Applies? | Reason / close artifact |
  |---|---|---|
  | #1 semantic migration | **Applies.** | T3 widens the `wasamoc` AST/expression/member/type surface and threads loop-local scope through check/lower. Close with `rg`-enumerated call-site audit over `TypeName`, `Expr`, `Member`, `Statement`, `HandlerExpr`, and `ControlFlowNode` construction/lowering sites. |
  | #2 side effects | Not applicable. | T3 performs no materialised widget-tree mutation and no runtime structural side effects. |
  | #3 parallel data drift | Not applicable. | T3 adds no parallel vectors, placement arrays, caches, or derived indices. |
  | #4 untested authored branch | **Applies.** | Every T3-owned reject / diagnostic / semantic branch gets a directly firing test or is mapped to owner T6/T7 in the implemented-branch test map. |
  | #5 carry-forward | **Applies.** | Any behavior/invariant discovered while authoring (for example loader-only rows or syntax that creates observable textual IR) must be recorded with owner, scope, impact, and re-trigger. |
  | #6 root cause | Standing. | Any deterministic failure is root-caused and recorded rather than retried to green. |
  | #7 GUI evidence | Not applicable. | T3 has no GUI-render evidence deliverable. |

  **Review lane:** branch/test-focused review. T3 is diagnostic /
  reject-branch heavy but does not perform a schema migration in the
  shared IR crate or runtime structural change.

- **2026-06-14 / T3 close gate — author surface implemented and branch
  map reconciled.** Implemented the `wasamoc` author surface for
  M3-Phase 7 iteration: `in` reservation, collection state types and
  literals, `for` members, loop-local expression reads in `for` bodies,
  collection assignment RHS forms, check-time matrix diagnostics,
  lowering to the T2 IR forms, and textual-IR emit pins. Loader-only
  structural re-validation and runtime mutation semantics remain
  deliberately outside T3.

  **Trap #1 call-site audit (`rg`-enumerated).** Commands run:

  ```text
  rg -n "TypeName|CollectionElemType|Expr::(QualifiedRef|ListLit|CollectionCall|Ident)|Member::For|BlockStatement|HandlerExpr::(ItemRead|IndexRead|ListAppend|ListDropLast|ListLit)|ControlFlowNode::For|Keyword::In|Token::(LParen|RParen|LBracket|RBracket|Comma)" wasamoc\src
  rg -n "collection|for`|loop binder|append|drop-last|nested `for`|component-level `for`|local state name|collection expressions|list literal|handler.*`for`|exactly one widget child|bare control flow|not a declared state or loop binder" wasamoc\src\check.rs wasamoc\src\parser.rs
  ```

  | Surface | Sites classified | Disposition |
  |---|---|---|
  | Lexer keywords / punctuation | `wasamoc/src/lexer.rs` keyword display, scanner punctuation, `scan_ident`, tests | **Extended.** `in` is a keyword; `in-out` / `in-outx` behavior is explicitly pinned as unaffected. |
  | AST / parser type and expression surface | `ast.rs`; `parser.rs` `parse_for_member`, `parse_statement`, `parse_expr`, `parse_type_name` | **Extended.** The author AST now carries collection types, list literals, qualified refs, collection calls, expression statements for reject-only collection calls, and `Member::For`. Chained collection calls reject at parse because T3 admits only one contextual method call. |
  | Check matrix | `check.rs` state defaults, property binds, handler statements, `for` placement/body/header/binder checks, loop-context expression checks | **Extended with direct diagnostics.** Author-reachable DD-007 rows are checked in `wasamoc`; textual-IR-only dual gates are mapped to T6. During double-loop review, the unknown-ident-in-`for` branch was narrowed to typed property targets so untyped keyword-like property values remain compatible with the existing DSL surface. |
  | Lowering / emit | `lower.rs` state lowering, member lowering, binding lowering, handler RHS lowering; `emit.rs` existing T2 IR emit arms plus authored-surface test | **Extended.** Authored collection defaults, `for`, `item-read` / `index-read`, `list-append`, `list-drop-last`, and `list-lit` lower and emit through the T2 IR forms. |
  | Runtime loader / evaluation | `wasamo-runtime` intentionally not touched | **Owned by T6/T7.** T3 emits IR shapes but does not materialise `For`, validate textual-IR-only scope rows, evaluate collection writes, or implement guarded loop-local runtime reads. |

  **Implemented-branch test map.** Source enumeration was the two `rg`
  commands above plus the named test inventory from `cargo test
  --workspace`. `cargo test` green is supporting evidence only; the rows
  below are the forcing map.

  | Branch / semantic pin | Direct test / owner |
  |---|---|
  | `in` reserved; punctuation admitted; `in-out` remains one hyphenated keyword | `lexer::tests::control_flow_family_keywords_reserved`; `lexer::tests::punctuation`; `lexer::tests::in_out_unaffected_by_in_keyword`; `lexer::tests::in_outx_lexes_as_kebab_ident` |
  | Collection state type / list literal / `for` parse; contextual `append` / `drop-last`; keyword binder reject; nested collection type reject; chained call parse reject | `parser::tests::collection_state_and_for_member_parse`; `parser::tests::collection_assignment_contextual_methods_parse`; `parser::tests::for_keyword_binder_rejected_at_identifier_position`; `parser::tests::nested_collection_type_rejected_at_parse`; `parser::tests::chained_collection_call_rejected_at_parse` |
  | Positive author collection default, empty initial value, `for` body loop-local read, and collection assignment forms | `check::tests::collection_state_default_and_for_body_accepted`; `check::tests::collection_assignment_forms_accepted`; lower/emit pins below |
  | `for` target reject rows: scalar target, undeclared target, qualified target, non-identifier / collection-expression target | `check::tests::for_target_must_be_collection_state`; `check::tests::for_target_must_be_declared`; `check::tests::for_target_rejects_qualified_reference`; `check::tests::for_target_rejects_collection_expression` |
  | Binder reject rows: binder state collision and value/index same name; keyword-as-binder parser reject | `check::tests::for_binder_collisions_rejected`; `parser::tests::for_keyword_binder_rejected_at_identifier_position` |
  | Placement reject rows: component-level, ScrollView, Box, Grid / Cell placement contexts | `check::tests::for_component_level_rejected`; `check::tests::for_disallowed_direct_containers_rejected` |
  | Body reject rows: non-widget member, multi-child body, bare control-flow body, nested `for` at any depth, handler inside `for` body | `check::tests::for_body_shape_rejects_non_widget_multi_child_and_bare_control_flow`; `check::tests::for_body_rejects_handler_and_nested_for_at_any_depth` |
  | Binder-read rows: outside body, handler position, `if` condition, undeclared typed binding inside body, and untyped keyword positive control | `check::tests::loop_binder_reads_rejected_outside_handler_and_if_condition`; `check::tests::for_body_rejects_unknown_typed_binding_but_keeps_untyped_keyword_values` |
  | Collection declaration / literal reject rows: collection requires list default, list-on-scalar, hetero / mismatched element, non-literal element, nested list | `check::tests::collection_declaration_literal_rejects_bad_shapes` |
  | Collection-assignment reject rows: qualified LHS, compound op, scalar LHS, collection expr outside RHS / as statement / property binding, arity, wrong receiver, qualified receiver, append element mismatch, drop-last arity, bare copy | `check::tests::collection_assignment_rejects_bad_shapes`; `parser::tests::chained_collection_call_rejected_at_parse` for chained-call syntax |
  | Lower semantic pins: collection state, `ControlFlowNode::For`, `ItemRead`, `IndexRead`, `ListAppend`, `ListDropLast`, `ListLit` | `lower::tests::collection_state_and_for_loop_lower_to_ir`; `lower::tests::for_index_binder_lowers_to_index_read`; `lower::tests::collection_assignment_lowers_to_handler_exprs` |
  | Emit semantic pins for authored `for` and collection assignment surface | `emit::tests::authored_for_surface_emits_loop_local_reads_and_collection_assignment`; existing T2 `emit::tests::collection_state_and_for_member_emit_in_textual_ir_shape` / string-bool spellings remain green |
  | Textual-IR loader dual-gates: malformed `For`, textual loop-local read position/scope, textual collection declaration/assignment validation beyond author syntax | **Owner = T6.** T3 only emits author-valid IR; T6 re-validates the loader surface. |
  | Runtime guarded reads, `ForLoopSubtree`, collection writer evaluation, equal-value no-dirty production use, splice/materialisation effects | **Owner = T7.** T3 only lowers handler expressions to IR. |

  **Single-loop / double-loop self-check.**

  | Check | Result |
  |---|---|
  | Past-task carry-over processed? | Yes. T2/T1 carry rows were read first; T3 closed author parser/check/lower/emit and left T6/T7 rows explicitly mapped. |
  | Audit table created? | Yes. Trap #1 audit table above is `rg`-enumerated; implemented branches are mapped to direct tests or owners. |
  | Unit-test breadth/depth sufficient for T3 pins? | Yes after widening: direct tests now cover non-identifier `for` targets, qualified receivers, property-position collection expressions, chained calls, and the unknown-typed-binding vs untyped-keyword distinction. |
  | Plan hypothesis challenged? | The starting plan was too implicit about AST ownership and T6/T7 boundaries; plan was revised before implementation. A later double-loop pass found the unknown-ident-in-`for` reject was too broad and corrected it. |
  | T3 behavior incorrectly pushed later? | No author-reachable parser/check/lower/emit row remains unowned. Loader-only textual IR and runtime mutation rows are correctly T6/T7, with scope and impact below. |

  **Plan-hypothesis challenge row.**

  | Hypothesis challenged | Refuted | Not refuted | Ownership moved / unresolved |
  |---|---|---|---|
  | T3 is "just parser/check/lower/emit" and can avoid AST changes. | Refuted: author evidence needs AST shapes for collection calls, list literals, block expression statements, and `Member::For`. | The bounded owner remains `wasamoc`; no runtime code needed. | None. |
  | The DD-007 matrix can be proven by broad happy-path tests plus `cargo test`. | Refuted: branch map found missing direct pins for chained call, qualified receiver, and collection expressions in property position. | Direct branch tests are now sufficient for T3-owned rows. | None. |
  | Unknown identifiers in a `for` body should all be rejected as undeclared binders. | Refuted: that would reject existing untyped keyword-like property values. | Typed property targets still reject undeclared loop-local reads. | T10 should sync this nuance into the spec if the external-reader text is ambiguous. |
  | Loader/runtime rows belong in T3 because T3 can emit the syntax. | Not refuted: emission alone does not prove loader static materialisation or runtime evaluation. | T6/T7 ownership remains correct. | T6/T7 carry rows preserved. |

  **Owner-correction count.** `0` at T3 close before owner report. The
  next task must revise this signal if owner feedback after this report
  causes extra implementation, extra tests, or plan-ownership correction.

  **Two-key exit check.**

  | Key | Status |
  |---|---|
  | Carry-forward key | Satisfied. Remaining unresolved work is assigned: T6 owns textual-IR loader dual gates and static `for` materialisation; T7 owns runtime guarded loop-local reads, collection writer evaluation, equal-value no-dirty production use, `ForLoopSubtree`, and splice effects; T10 owns spec sync for landed author nuances. Scope and impact are listed in the carry scan below. |
  | Proof key | Satisfied. Every T3-implemented branch / semantic pin is mapped to a direct test in the implemented-branch test map; no row relies on workspace green alone. |

  **Behavior / invariant carry scan.**

  | Behavior / invariant discovered or created | Owner / scope / impact |
  |---|---|
  | `in` is now reserved in the author lexer, while `in-out` and `in-outx` remain unaffected. | **T10 spec sync.** Ensure §2.1 / token examples match the landed boundary behavior. |
  | Author grammar admits collection defaults, list literals, `for`, and collection assignment syntax; chained collection methods are rejected by the parser, not by check. | **T10 spec sync.** External text should not imply chained calls reach semantic validation. |
  | Unknown identifiers in typed property binds inside a `for` body reject as undeclared state/binder, but untyped keyword-like property values still pass. | **T10 spec sync.** Prevents accidental overstatement of loop-body identifier strictness. |
  | Textual IR emitted by T3 may contain `list-append` / `list-drop-last` handler expressions before runtime evaluation exists. | **T7.** Runtime handler evaluation still deliberately rejects these forms until the collection writer lands; authored gallery mutation must wait for T7/T8. |
  | Loader must not trust author-only checks for `for` body shape, scope, container placement, or collection-assignment well-formedness. | **T6.** Textual IR remains an independent input surface and needs its dual-gate matrix. |

  **Trap #6 disposition.** No deterministic failure was rerun as a
  flake. The known implementation-time failure was a test fixture issue
  in `for_index_binder_lowers_to_index_read` using a string-typed target
  for an index read; the fixture was corrected to an integer property
  target. During close self-check, an over-broad unknown-identifier
  reject in `for` bodies was found and narrowed before final test
  evidence.

  **Trap #2 / #3 / #7 close confirmation.** T3 added no runtime
  materialised-tree side-effect bundle (#2), no child-parallel placement
  vector or cache (#3), and no GUI-render evidence deliverable (#7).

  **Verification evidence.** `cargo fmt --all -- --check` passed.
  `cargo test -p wasamoc` passed (349 unit tests + 6 roundtrip tests).
  `cargo test --workspace` passed. **Review lane remains
  branch/test-focused** because this task added diagnostics and author
  semantic branches but no shared IR schema migration or runtime
  structural change.

- **2026-06-14 / T3 owner-follow-up audit addendum — constraints and
  branch pins widened.** After the initial T3 completion report, the
  owner requested a critical re-check against
  [requirements/constraints.md](../requirements/constraints.md) and a
  deeper test-width review. Two read-only subagent audits were delegated:
  one for constraints/T3-boundary ownership, one for branch/test depth.
  Result: T6/T7 ownership remained correct, but the original T3 proof map
  over-claimed coverage for several author-reachable branches. This
  increments **Owner-correction count to `1`** for T3.

  **Constraints re-check result.** `constraints.md` items §1, §7, and
  §8 are the T3-relevant constraints: iteration must stay in the
  control-flow family, `TypedValue` / structured item pressure must not
  be smuggled, and semantic-migration proof must be a forcing artifact.
  Runtime ownership constraints (§2, §4, §5, §6, §9) remain T4–T9-owned.
  The concrete T3 miss was that qualified loop-local-looking reads such
  as `label.field` or `\{label.field}` could be resolved as ordinary
  state reads when `field` was a state, silently resembling structured
  item access. T3 now rejects qualified loop-local reads directly and
  records the structured-item / `TypedValue` deferral at author check.

  **Implemented-branch test map addendum.**

  | Branch / semantic pin added after follow-up | Direct test |
  |---|---|
  | Qualified loop-local reads (`label.field`, `\{label.field}`, `root.i`) reject as structured-item / loop-local qualification deferral | `check::tests::qualified_loop_local_reads_rejected_as_structured_item_deferral` |
  | Gallery-like author shape: `ScrollView { WrapPanel { for ... { Box { Text { ... } } } } }` plus body-external Add/Remove handlers compiles | `check::tests::gallery_like_for_shape_and_body_external_handlers_accepted` |
  | Gallery-like shape lowers to one Box body, interpolation `ItemRead` + `IndexRead`, and external collection mutation handler expressions | `lower::tests::gallery_like_for_shape_lowers_single_box_body_and_external_mutations` |
  | Empty list defaults / assignments accepted in typed collection contexts, including bool collection | `check::tests::collection_assignment_forms_accepted` |
  | Index binder colliding with a state name is directly fired, not only value-binder / same-name collision | `check::tests::for_binder_collisions_rejected` |
  | Direct `for` under `Cell` rejects, while a `for` inside a descendant `WrapPanel` under `Cell` is admitted | `check::tests::for_disallowed_direct_containers_rejected`; `check::tests::for_is_admitted_inside_cell_descendant_container` |
  | Collection assignment reject rows for undeclared LHS + collection RHS, collection LHS + scalar RHS, unknown method, and append element with unknown type | `check::tests::collection_assignment_rejects_bad_shapes` |
  | Keyword in index-binder position rejects at parse, not only keyword in element-binder position | `parser::tests::for_keyword_binder_rejected_at_identifier_position` |
  | Chained collection call now gets a named deferral diagnostic rather than a generic `expected ';'` parse error | `parser::tests::chained_collection_call_rejected_at_parse` |
  | Emit pin includes both `(item-read label)` and `(index-read i)` from authored interpolation | `emit::tests::authored_for_surface_emits_loop_local_reads_and_collection_assignment` |

  **Plan-hypothesis correction.** The plan's "gallery shape compile and
  lower" positive control was not sufficiently discharged by the
  minimal `Text { text: label }` fixture; T3 now has a gallery-like
  Box/Text/ScrollView/WrapPanel fixture with body-external mutation
  handlers. The earlier "every matrix row" proof row was also too coarse:
  collection-assignment and placement-context rows needed more direct
  sub-branch pins.

  **Verification evidence.** `cargo fmt --all -- --check` passed.
  `cargo test -p wasamoc` passed after the addendum (353 unit tests + 6
  roundtrip tests). Workspace verification is recorded in the updated
  T3 retrospective.

- **2026-06-14 / T3 in-task review remediation — loop-external read and
  bool-binder interpolation rows closed (commit `fccd277`).** A critical
  in-session review (assistant reviewer, distinct from the implementing
  pass) re-ran the DD-M3-P7-007 author matrix row-by-row against the
  shipped diagnostics and found **two author-reachable rows the close map
  had over-claimed as covered** — both confirmed by throwaway probe tests
  against the built compiler, not by inspection. Remediated inside T3
  rather than carried, per owner direction (close author-surface holes in
  the owning task). This increments **Owner-correction count to `2`** for
  T3.

  **Root failure (single-loop).** The `t3.md` corrective tests
  *Branch-map granularity* and *Smuggle scan*, written at the owner
  follow-up, were **recorded but not actually executed against every
  DD-007 row**. The loop-external collection-read row never appeared in
  the plan's prose enumeration, so it was absent from the branch map; the
  bool element interpolation contract was applied to scalar bool states
  but not to bool loop binders.

  **Rows closed.**

  | Author-reachable row (DD-007) | Pre-fix behavior (probed) | Post-fix behavior |
  |---|---|---|
  | bare collection ident in a property position (`bar: xs`) | **silently accepted**, lowered to `IrLiteral::Ident` | named "collection reads outside iteration not yet supported" deferral |
  | collection member navigation (`xs.length`) | misleading `undefined state \`length\`` | named loop-external read deferral |
  | whole-value qualified read (`root.xs`) | accepted (untyped prop) / type-mismatch only (typed prop) | named loop-external read deferral |
  | collection ident / navigation in interpolation (`\{xs}`, `\{xs.length}`) | accepted, lowered to `PropRead` | named loop-external read deferral |
  | collection read as scalar assignment RHS (`n = xs`) | silently accepted | named loop-external read deferral |
  | indexed read (`xs[i]`) | raw `expected member`/`expected ;` parse error | named deferral at parse |
  | bool loop binder in interpolation (`\{f}`, `f: bool[]` elem) | accepted, lowered to `ItemRead` | rejected, mirroring the scalar bool-in-interp contract |

  **Implemented-branch test map addendum (this remediation).**

  | Branch / semantic pin added | Direct test |
  |---|---|
  | Loop-external collection reads (bare, qualified whole-value, member navigation, interpolation, scalar-RHS, in-body read of the iterated collection) | `check::tests::loop_external_collection_reads_rejected` |
  | Indexed collection read deferral at parse | `parser::tests::indexed_collection_read_rejected_at_parse` |
  | Bool loop binder rejected in interpolation; string binder + i32 index positive control | `check::tests::bool_loop_binder_in_interpolation_rejected` |
  | No duplicate type-mismatch for a collection source in a typed property | covered by `loop_external_collection_reads_rejected` (`Text { text: xs }` asserts the single named diagnostic) |

  **Plan-hypothesis challenge row (this remediation).**

  | Hypothesis challenged | Refuted | Not refuted | Ownership moved / unresolved |
  |---|---|---|---|
  | The owner-follow-up widening made the T3 author matrix complete. | Refuted: two DD-007 author rows were still uncovered and silently/misleadingly handled. | The T6/T7 runtime/loader boundary stays correct; no runtime row was wrongly pulled forward. | None — both rows closed in T3. |
  | "Loop-external collection reads" can rely on existing scalar diagnostics (undefined-state / type-mismatch). | Refuted: bare and interpolated collection reads produced *no* diagnostic; `xs.length` produced the wrong one. | Indexed reads were always rejected — but as a generic parse error, not the named deferral. | None. |

  **Two-key exit check (this remediation).**

  | Key | Status |
  |---|---|
  | Carry-forward key | Satisfied. The named loop-external read deferral is now the T3 author surface; T6 must add the **textual-IR loader dual-gate** for the same rows (a `for`-external `list-prop-read` / member navigation in textual IR) — recorded in the carry scan below. No new unowned point. |
  | Proof key | Satisfied. Every newly closed branch fires a direct test; the silent-acceptance cases are asserted to now emit exactly the named diagnostic. |

  **Behavior / invariant carry scan (this remediation).**

  | Behavior / invariant created | Owner / scope / impact |
  |---|---|
  | `wasamoc check` now rejects every author-reachable loop-external collection read with one named deferral; the bool-element interpolation reject extends to loop binders. | **T10 spec sync.** The §4.15 invalid-examples / binder-scope text must list the loop-external read deferral and the bool-binder interpolation reject so the external-reader matrix matches the shipped diagnostics. |
  | Indexed reads (`xs[i]`) are a **parse-time** reject (no index grammar); all other loop-external reads are **check-time**. | **T6 loader dual-gate.** Textual IR has no `[i]` surface, but it *can* carry a `for`-external `list-prop-read`; T6 must re-reject that independently (the loader does not trust author checks). |
  | A collection source in a typed scalar property emits the loop-external read diagnostic only (type-mismatch suppressed). | T3-local; no carry. Prevents double diagnostics. |

  **Findings judged out of scope / not changed.** (a) `check_for_body`
  emits two diagnostics for a single non-widget body member — harmless
  redundancy, both true, left as-is. (b) The admitted-container set for
  `for` is a denylist (reject ScrollView/Box/Grid/Cell, admit the rest),
  mirroring the Phase 6 `if` family pattern, so a `for` under a non-layout
  leaf widget is admitted at check; this is a **family-level** behavior,
  not T3-specific, and is recorded as a Phase 8 / spec carry rather than
  changed under T3 (changing it would touch the shared `if` admission
  contract). Both are recorded here so they are not silently dropped.

  **Verification evidence.** `cargo fmt --all -- --check` passed (clean).
  `cargo test -p wasamoc` passed (356 unit tests + 6 roundtrip).
  `cargo test --workspace` passed (`wasamo-runtime` 381, `wasamo-ir` 23,
  `wasamoc` 356). No production behavior outside `wasamoc check` / parse
  diagnostics changed; lowering and emit are unaffected (the rejected
  inputs never reach lowering).

- **2026-06-13 / T2 post-close critical re-check — plan hypothesis
  challenged against the preamble.** Re-read the implementation
  preamble, the mutable task plan, constraints §8, T2 code diff, and the
  T2 carry-forward rows as if `plan.md` were only a hypothesis. A second
  subagent pass was also delegated to challenge the boundary. Result:
  no additional T2 implementation work is required. T2's bounded goal is
  the schema/parser/registry/audit foundation, not the full Phase 7
  iteration runtime.

  | Question challenged | Disposition |
  |---|---|
  | Does T2 need to finish the author surface or full DD-007 matrix? | No. T3 owns author parser/check/lower/emit and the compile-time matrix; T6 owns loader dual-gate rows. |
  | Does T2 need to materialise `for` in the runtime? | No. T2's build-time `For` reject is the intentional Seam A. T6 replaces it with static materialisation and proves no double-create. |
  | Does T2 need `BindingTarget::ForLoopSubtree` or handler-side collection writes? | No. T7 owns the runtime target, collection assignment evaluation, and first production `set_if_changed` use. |
  | Is `Signal::set_if_changed` sufficiently enforced? | Bounded carry. T2 lands the contract and tests it, but the collection maps remain plain `Signal<Vec<_>>`; T7 must use the contract and fire the empty-`drop-last` no-dirty fixture. |
  | Are `item-read` / `index-read` fully scoped in T2? | Bounded carry. T2 carries IR/textual forms and runtime rejects evaluation. T3 owns author-scope diagnostics; T6/T7 own loader/runtime context. To make this ownership auditable on the task list, the T6 loader dual-gate bullet was revised to explicitly name textual-IR loop-local read position/scope violations. |
  | Does constraints §8 require anything else? | Closed. Start gate, close audit, full independent review, and the compile-error-forcing preference minor edit are all recorded. |

- **2026-06-13 / T2 close gate — IR schema migration landed; residuals
  assigned.** Implemented the shared IR migration bundle as one buildable
  change: `IrStateType::{Scalar, Collection}`, `IrLiteral::List`,
  `ControlFlowNode::For`, collection/loop-local `HandlerExpr` forms,
  textual-IR emit/load support, loader validation/annotation, collection
  signal maps, and `Signal::set_if_changed`. Critical re-check outcome:
  T2 is still only the schema/parser/registry seam. Author syntax and
  compile-time matrix remain T3; static materialisation remains T6;
  loop-local binding evaluation, `ForLoopSubtree`, and handler-side
  collection assignment evaluation remain T7.

  **Trap #1 call-site audit (`rg`-enumerated).** Commands run:
  `rg -n "IrStateType|IrState \\{|\\.ty|state\\.ty" wasamo-ir wasamoc wasamo-runtime`;
  `rg -n "ControlFlowNode::|IrMember::ControlFlow|widget_children\\(|children\\.iter\\(\\).*filter_map|filter_map\\(IrNode::widget_children" wasamo-ir wasamoc wasamo-runtime`;
  `rg -n "HandlerExpr::(ListPropRead|ItemRead|IndexRead|ListAppend|ListDropLast|ListLit)|ListPropRead|ItemRead|IndexRead|ListAppend|ListDropLast|ListLit" wasamo-ir wasamoc wasamo-runtime`;
  `rg -n "BindingTarget|ForLoopSubtree|ConditionalSubtree|register_binding|register_bool_binding|register_conditional_binding" wasamo-runtime wasamo-ir wasamoc`.

  | Surface | Sites classified | Disposition |
  |---|---|---|
  | `IrState.ty` / `IrStateType` | `wasamo-ir/src/lib.rs`; `wasamoc/src/lower.rs`; `wasamoc/src/emit.rs`; `wasamo-runtime/src/ir_loader.rs`; `wasamo-runtime/tests/ir_loader_roundtrip.rs` | **Extended.** Scalar lowering now wraps existing types in `Scalar`; emitter/parser/loader handle `i32[]` / `string[]` / `bool[]`; registry builds scalar and collection maps. Loader rejects list-on-scalar, scalar-default-on-list, mismatched list elements, and nested list literals. |
  | `IrLiteral::List` | `wasamo-ir/src/lib.rs`; `wasamoc/src/emit.rs`; `wasamo-runtime/src/ir_loader.rs` parser / renderer / validation | **Extended with context restrictions.** List literals are valid only for collection defaults or collection assignment RHS. Bare scalar assignment list RHS is a direct reject. |
  | `ControlFlowNode` / `IrMember::ControlFlow` | `wasamo-ir/src/lib.rs`; `wasamoc/src/lower.rs` and `emit.rs`; `wasamo-runtime/src/ir_loader.rs` annotation / validation / parser / static-build paths / test renderer | **Extended or deliberately rejected.** `For` is emitted/parsed/validated as a one-widget body. Static runtime materialisation is a T6-owned deferred reject in `append_static_member` and ZStack placement collection, with tests. Existing `If` lowering remains correctly unaffected by author-surface ownership (T3 lowers `for`). |
  | `IrNode::widget_children()` | `wasamo-ir/src/lib.rs` | **Correctly unaffected widget-only filter.** It must continue to return only concrete `IrMember::Widget` children; control-flow bodies are structural members, not direct static widget children. Callers that need to recurse through `If`/`For` do so explicitly in loader validators. |
  | Widget-only / direct-control-flow filters | `wasamo-runtime/src/ir_loader.rs` ScrollView, Grid cell, placement contexts, `matches!(m, IrMember::ControlFlow(_))` guards | **Classified, not missed.** Direct control flow remains invalid in the existing widget-only contexts, so wildcard `ControlFlow(_)` guards are correct rejects for both `If` and `For` there. **Box is not in this reject class at T2:** the existing Box gate permits a single control-flow member (e.g. conditional-only body), and `for`-under-Box loader dual-gating remains T6-owned with the broader disallowed-container matrix. Structural invariant walkers that must enter bodies gained explicit `For` recursion. |
  | `HandlerExpr` collection and loop-local variants | `wasamo-ir/src/lib.rs`; `wasamoc/src/emit.rs`; `wasamo-runtime/src/handler.rs`; `wasamo-runtime/src/ir_loader.rs` | **Extended with deliberate runtime rejects.** Emit/load round-trip the forms. Loader permits collection reads only in `for` headers and collection edits/list literals only as collection assignment RHS; it also rejects obvious `list-append` element type mismatches. `handler.rs` rejects the new forms until T7 wires actual collection mutation and loop-local evaluation. |
  | `BindingTarget` pre-audit | `wasamo-runtime/src/reactive.rs`; call sites in `wasamo-runtime/src/ir_loader.rs` | **Deliberately unchanged.** Current targets are `WidgetProperty` and `ConditionalSubtree`; `register_binding`, `register_bool_binding`, and `register_conditional_binding` still panic on wrong target class. `ForLoopSubtree` is not a T2 type: it is T7-owned per CF-6 / T1 correction. |

  **Trap #4 direct branch tests.** Added/updated tests cover
  `IrStateType` scalar-vs-collection encoding, list literal storage,
  handler iteration variants, `ControlFlowNode::For` shape, textual emit
  of collection state + `for`, loader parsing/rejects for collection
  defaults, list-on-scalar, mismatched/nested list elements, `for` over
  scalar or undeclared target, `for` binder/index collisions,
  multi-child and nested-control-flow body rejects, collection
  assignment `append` / `drop-last` / literal RHS restrictions, wrong
  receiver, wrong RHS kind, compound assignment, collection edit outside
  assignment RHS, append value type mismatch, bare collection read
  outside a `for` header, and the T6-owned static materialisation
  reject. Added `Signal::set_if_changed` test for equal-value no-dirty
  semantics, a collection-map initialisation test for all three
  collection types, a handler-evaluator reject test for collection forms
  until T7, and a unit pin that `IrNode::widget_children()` does not
  recurse into `For` bodies.

  **Trap #5 carry-forward / ownership after T2.**

  | Residual | Owner / impact |
  |---|---|
  | `For` static materialisation is still a loader build reject | **T6.** Replaces the reject with static construction and declared-slot plumbing; current reject prevents accidental partial rendering. |
  | Loop-local binder scope/type matrix (`item-read`, `index-read`) | **T3** for author check rows, **T6/T7** for loader/runtime evaluation context. T2 carries variants and syntax only. |
  | `BindingTarget::ForLoopSubtree` and per-item binding registration | **T7.** T2 pre-audit confirms no current target can represent it. |
  | Handler-side collection assignment evaluation (`append`, `drop-last`, list literal whole-value write) | **T7.** T2 adds IR/load validation; `handler.rs` intentionally rejects evaluation until the collection write path and signal dirtying are wired together. |
  | `Signal::set_if_changed` has no production caller yet | **T7.** This is the CF-5 contract; the `#[allow(dead_code)]` is intentional and must close when the first collection writer lands. |
  | Full DD-M3-P7-007 negative matrix | **T3/T6.** T2 only covers schema-level and static-loader seam rejects needed to keep the migration buildable. |

  **Trap #6 disposition.** No deterministic implementation failure was
  rerun as a flake. Two local `collection_assignment_append` test
  failures were fixed as root causes: first the test used invalid textual
  IR (`next` / `label` instead of `(prop-read next)` /
  `(str-prop-read label)`), then the diagnostic path for mismatched
  scalar reads was sharpened. The final targeted run passed.

  **Trap #2 / #3 / #7 close confirmation.** No materialised widget-tree
  side-effect bundle was added (#2), no child-parallel placement vector
  was migrated (#3), and no GUI-visible behavior exists in T2 (#7).

  **Verification evidence.** `cargo test -p wasamo-runtime
  collection_assignment_append --lib` passed after the targeted fixes.
  Full workspace verification and clean-rebuild evidence are recorded in
  the T2 retrospective. **Review lane was full independent review**; the
  review disposition is recorded in the entry immediately below.

- **2026-06-13 / T2 independent review disposition.** Full independent
  review was delegated after owner approval. Review result: no high- or
  medium-severity code issue found in the T2 schema migration. One low
  documentation finding was accepted: the T2 close audit over-classified
  Box as a direct-control-flow reject context. The audit row above was
  corrected to state that Box currently permits single control-flow
  content and that `for`-under-Box loader dual-gating is T6-owned. One
  test-gap suggestion was also accepted: added a direct
  `build_signal_registry` test proving `i32[]`, `string[]`, and `bool[]`
  defaults populate the three collection signal maps. The Phase 7
  constraints §8 residual about compile-error-forcing preference was
  closed as a minor edit to
  [implementation-gates.md](../../../procedures/implementation-gates.md)
  trap #1 close-artifact guidance.

- **2026-06-14 / T2 test-depth addendum — trap #4 branch pins widened.**
  A dedicated test-coverage review challenged whether the tests were
  deep enough for the behavior T2 itself already implements. Accepted
  result: T2 was not missing implementation, but several T2-owned reject
  branches were only documented, not directly fired. Added tests for:
  `For` undeclared collection, binder/index state collisions,
  same binder/index, multi-child body, nested-control-flow body;
  collection state scalar default, collection compound assignment,
  collection edit outside assignment RHS, collection assignment wrong RHS
  kind; string/bool collection defaults; collection assignment list
  literal RHS; nested-list default validation; collection-form evaluator
  rejects until T7; `widget_children()` excluding `For` body widgets; and
  string/bool collection state emission.

- **2026-06-14 / T2 merge-prep addendum — implemented-branch test map
  made forcing.** A merge-readiness review found that the retrospective's
  "Implemented-branch test map" corrective had been recorded as prose but
  not applied as a mechanical reconciliation artifact. The concrete miss:
  three T2-owned `IrLoadError::Validate` branches were implemented but had
  no direct firing test. Added
  `scalar_state_rejects_type_mismatched_default`,
  `scalar_read_of_collection_state_rejected`, and
  `append_collection_state_as_element_rejected`. This closes the three
  missing branches and upgrades the close condition: `cargo test` green is
  supporting evidence only; the branch map below is the forcing artifact.

  Command used for the reconciliation:
  `rg -n 'IrLoadError::Validate' wasamo-runtime/src/ir_loader.rs`.
  Non-production hits for `Display`, comments, and test assertions are not
  branch arms. Production arms are mapped below by current line number.

  | Line(s) | Reject branch | Direct test / owner |
  |---|---|---|
  | 183 | duplicate state name | `malformed_duplicate_state_name` |
  | 251 | collection default list item mismatch / nested list | `collection_state_rejects_mismatched_list_element`; `collection_state_rejects_nested_list_default` |
  | 259 | scalar state uses list default | `scalar_state_rejects_list_default` |
  | 263 | collection state default is not a list | `collection_state_rejects_scalar_default` |
  | 267 | scalar state default type mismatch catch-all | `scalar_state_rejects_type_mismatched_default` |
  | 409 | host `title` is not string | `static_window_title_rejects_non_string_host_prop` |
  | 419 | host `backdrop` / `theme` typed literal | `host_surface_rejects_typed_literal_backdrop`; `host_surface_rejects_typed_literal_theme` |
  | 426 | unknown host attribute | `host_surface_rejects_unknown_host_prop` |
  | 435 | host binding | `host_surface_rejects_host_binding` |
  | 450 | root-squatted host prop | `old_root_squatted_host_prop_rejected` |
  | 461 | root-squatted host binding | `old_root_squatted_host_binding_rejected` |
  | 492 | `if` has more than one branch in memory IR | Prior Phase 6 invariant; not T2-owned. Existing direct textual IR cannot construct multi-branch `if`; owner remains prior Phase 6 memory-IR defense. |
  | 499 | `if` body not exactly one member | `validate_rejects_if_with_empty_body`; `validate_rejects_if_with_multi_child_body` |
  | 507 | nested control-flow directly in `if` body | `validate_rejects_if_with_nested_control_flow_body` |
  | 515 | `for` body not exactly one member | `for_member_rejects_multi_child_body` |
  | 523 | nested control-flow directly in `for` body | `for_member_rejects_nested_control_flow_body` |
  | 567 | Box more than one child/member | `malformed_box_with_two_children`; `validate_rejects_box_with_multiple_conditional_siblings`; `validate_rejects_box_with_widget_and_conditional_sibling` |
  | 583 | ratio literal outside `Box.aspect` | `malformed_ratio_outside_box_aspect_on_vstack`; `malformed_ratio_on_box_wrong_prop_name`; `malformed_ratio_in_nested_node` |
  | 592 | color literal outside `Box.fill` | `malformed_color_outside_box_fill_on_text`; `malformed_color_on_box_wrong_prop_name` |
  | 637 | direct control-flow in ScrollView content | `validate_rejects_scrollview_with_conditional_member`; `validate_rejects_scrollview_with_conditional_only_member` |
  | 643 | ScrollView child count not exactly one | `scroll_view_with_zero_children_rejected`; `scroll_view_with_two_children_rejected`; `scroll_view_with_three_children_rejected`; `scroll_view_nested_zero_child_is_rejected` |
  | 687 | WrapPanel negative item/layout attributes | `wrap_panel_rejects_negative_item_cross_size`; `wrap_panel_rejects_negative_item_spacing`; `wrap_panel_rejects_negative_line_spacing` |
  | 748 | `Cell` outside `Grid` | `cell_outside_grid_rejected` |
  | 810 | `ZStack` kind payload | `validate_rejects_zstack_with_kind_payload` |
  | 819 | `ZStack` attributes | `zstack_attribute_rejected_at_validate` |
  | 825 | `ZStack` binding | `zstack_binding_rejected_at_validate` |
  | 830 | `ZStack` handler | `zstack_handler_rejected_at_validate` |
  | 841 | placement prop outside ZStack child / Grid Cell | `placement_prop_outside_zstack_child_or_grid_cell_rejected_at_validate` |
  | 888 | invalid placement alignment literal | `zstack_child_unknown_alignment_rejected_at_validate`; `grid_cell_unknown_alignment_rejected` |
  | 898 | non-Grid node carries Grid payload | `validate_rejects_non_grid_kind_payload` |
  | 928 | Grid missing track payload | `grid_node_without_tracks_rejected` |
  | 936 | Grid missing columns | `grid_missing_column_track_rejected` |
  | 941 | Grid missing rows | `grid_missing_row_track_rejected` |
  | 953 | Grid fixed track below 1 | `grid_zero_fixed_track_rejected` |
  | 960 | Grid star weight outside range | `grid_star_weight_over_cap_rejected`; the `< 1` subcase is prior Phase 5 memory-IR defense and not T2-owned. |
  | 983 | Grid direct non-Cell widget | `grid_non_cell_child_rejected` |
  | 989 | Grid direct control-flow member | `validate_rejects_direct_conditional_grid_member` |
  | 1002 | Grid cell overlap | `grid_same_cell_conflict_rejected`; `grid_overlapping_span_conflict_rejected`; `grid_multi_cell_omitted_placement_collides_at_origin` |
  | 1029 | Cell content child count | `grid_cell_zero_content_children_rejected`; `grid_cell_two_content_children_rejected` |
  | 1039 | Cell direct control-flow content | `validate_rejects_direct_conditional_cell_member` |
  | 1059 | Cell row out of range | `grid_cell_row_out_of_range_rejected` |
  | 1064 | Cell column out of range | `grid_cell_column_out_of_range_rejected` |
  | 1072, 1077 | Cell span below 1 | `grid_cell_zero_span_rejected` |
  | 1082, 1088 | Cell span exceeds Grid | `grid_cell_span_exceeds_grid_rejected` |
  | 1108 | Cell placement/span non-integer literal | Prior Phase 5 memory-IR defense; not T2-owned. No new T2 test required. |
  | 1126 | Cell alignment vocabulary | `grid_cell_unknown_alignment_rejected` |
  | 1211 | collection read outside `for` header | `bare_collection_read_outside_for_header_rejected` |
  | 1220 | assignment to undeclared lhs | `malformed_assign_undeclared` |
  | 1226 | collection state uses compound assignment | `collection_compound_assignment_rejected` |
  | 1229 | compound assignment to undeclared lhs | `malformed_compound_assign_undeclared` |
  | 1233 | collection edit expression outside assignment RHS | `collection_edit_outside_assignment_rhs_rejected` |
  | 1237 | list literal outside collection default/assignment RHS | `scalar_assignment_list_rhs_rejected` |
  | 1265 | `if` condition resolves to non-bool | `validate_rejects_if_with_bool_read_resolving_to_non_bool_state` |
  | 1268 | `if` condition undeclared | `validate_rejects_if_with_unresolved_condition` |
  | 1273 | `if` condition is scalar read form | `validate_rejects_if_with_non_bool_condition` |
  | 1277 | `if` condition other invalid expression | `validate_rejects_if_with_non_bool_condition` |
  | 1290 | scalar expression reads collection state | `scalar_read_of_collection_state_rejected` |
  | 1293 | scalar expression reads undeclared state | `malformed_propread_undeclared`; `malformed_undeclared_inside_interpolation`; `malformed_undeclared_inside_block` |
  | 1305 | collection read elem tag mismatches state elem | T2 annotation makes textual IR path non-observable after `annotate_collection_expr_types`; direct memory-IR owner is T6/T7 if they construct collection expressions outside the parser. |
  | 1310 | collection expression references scalar state | `for_member_rejects_scalar_collection_target` |
  | 1313 | collection expression references undeclared state | `for_member_rejects_undeclared_collection_target` |
  | 1325 | collection assignment lhs no longer collection | Defensive branch after caller dispatch; not T2-owned beyond `collection_assignment_wrong_receiver_rejected`. Owner T7 if runtime constructs assignment RHS without loader dispatch. |
  | 1342 | collection assignment list literal item mismatch | `collection_assignment_append_literal_type_mismatch_rejected`; `collection_state_rejects_mismatched_list_element` covers helper message shape for defaults |
  | 1347 | collection assignment wrong receiver | `collection_assignment_wrong_receiver_rejected` |
  | 1351 | collection assignment wrong RHS kind | `collection_assignment_wrong_rhs_kind_rejected` |
  | 1389 | append value wrong literal / unsupported expression kind | `collection_assignment_append_literal_type_mismatch_rejected` |
  | 1405 | append scalar read type mismatch | `collection_assignment_append_scalar_read_type_mismatch_rejected` |
  | 1410 | append collection state as one element | `append_collection_state_as_element_rejected` |
  | 1413 | append undeclared read | `malformed_undeclared_inside_block` covers handler read; T7 owns runtime collection-assignment eval diagnostics if this is constructed outside textual IR. |
  | 1424 | empty `for` binder | Textual parser cannot emit empty binder; direct memory-IR owner T6/T7 if they construct headers outside parser. |
  | 1429 | `for` binder collides with state | `for_member_rejects_binder_state_collision` |
  | 1435 | empty `for` index binder | Textual parser cannot emit empty index binder; direct memory-IR owner T6/T7 if they construct headers outside parser. |
  | 1440 | `for` binder/index same | `for_member_rejects_same_binder_and_index` |
  | 1445 | `for` index binder collides with state | `for_member_rejects_index_state_collision` |
  | 1456 | `for` collection expression is not collection read | Textual parser's `for IDENT in IDENT` cannot emit this; direct memory-IR owner T6/T7 if alternate construction appears. |

  Line numbers above are a point-in-time reconciliation as of this commit;
  T3/T6/T7 edits to `ir_loader.rs` will drift them. Future readers should
  match on the arm description + test name, not the line number.

  **Carry-forward additions surfaced by this audit.**

  | Residual | Owner / impact |
  |---|---|
  | Collection `HandlerExpr.elem` is not serialized in textual IR. `list-prop-read`, `list-append`, and `list-drop-last` parse with a placeholder element type, then `annotate_collection_expr_types` re-derives the authoritative element type from state declarations. | **T7**, with T6 awareness. Invariant: a collection `HandlerExpr.elem` is authoritative only after annotation has run. Re-trigger when T7 builds or evaluates collection `HandlerExpr` through any path other than `parse_ir`'s parse → annotate → validate sequence. |
  | Loader scalar defaults are now strictly type-checked. `state count: i32 = true` and analogous scalar/scalar mismatches are rejected instead of flowing through as malformed-but-loaded textual IR. | **T3/T6 phase-sync awareness.** This is an observable direct-textual-IR strictness change, pinned by `scalar_state_rejects_type_mismatched_default`; author syntax should already reject through T3's typed defaults, and T6 must preserve the stricter loader gate. |

  Collection `elem` serialization decision: T2 chose **not** to put the
  element tag into textual spellings for `list-prop-read` /
  `list-append` / `list-drop-last`; the parser accepts the compact state
  reference and re-derives the tag from the declared collection state in
  `annotate_collection_expr_types`. The alternative was to serialize the
  element tag with every collection read/edit form. T2 rejected that
  because the state declaration is the single source of truth and a
  second textual tag would create an avoidable mismatch mode; the cost is
  the invariant above, which T7 must respect when it starts evaluating or
  constructing collection handlers outside the loader path. T1 §1 covered
  marker-like `item-read` / `index-read`, but did not record this
  `list-prop-read` serialization decision, so it is carried explicitly
  here.

  **Merge-prep validation.** `cargo fmt --all -- --check` passed;
  `cargo build --workspace` passed with the existing `wasamo` linkable
  target warning; `cargo test --workspace` passed. The targeted
  `wasamo-runtime --lib` run passed 381 tests, including the three new
  reject pins named above.

- **2026-06-13 / T2 start gate — IR schema migration traps selected
  before production edits.** Re-read
  [implementation-gates.md](../../../procedures/implementation-gates.md),
  the T1 carry-over rows, and the current source surfaces for
  `IrState`, `IrLiteral`, `ControlFlowNode`, `HandlerExpr`,
  `SignalRegistry`, and the textual-IR emit / load path. Critical
  re-check of the plan hypothesis: T2 remains the right buildable bundle
  boundary because `IrStateType`, `IrLiteral::List`, and
  `ControlFlowNode::For` break shared IR, `wasamoc` emit/lower/check,
  textual-IR parsing, runtime validation, and registry setup together;
  splitting would create an intentionally non-building intermediate
  state. No additional owner consult is needed at T2 start: the schema
  shapes and vocabulary are settled by the Accepted DDs, while the
  owner-visible structured-item trigger remains correctly owned by T8.

  **Selected traps and reasons:**

  - **#1 semantic migration — applies.** T2 widens IR/schema enums and
    fields. Close artifact: an `rg`-enumerated table over `IrState`,
    `IrMember`, `ControlFlowNode`, `HandlerExpr`, and a `BindingTarget`
    pre-audit, including compiler-silent wildcard filters such as
    `IrNode::widget_children()`.
  - **#2 side effects — not applicable.** T2 adds schema and signal
    storage only; it does not splice materialised widget children or
    mutate Visual/layout structure.
  - **#3 parallel data drift — not applicable.** T2 introduces registry
    maps keyed by state name, not a parallel vector/index coupled to a
    child list. Placement drift is T5-owned.
  - **#4 untested branch — applies narrowly.** The deferred-load
    `ControlFlowNode::For` branch and any loader-side list/state-type
    validation branches that T2 lands must have direct tests. The full
    author-surface / loader reject matrix remains T3/T6.
  - **#5 carry-forward — applies.** T2 establishes collection registry
    maps and equal-value no-dirty semantics relied on by T7; the
    deferred-load `For` rejection is CF-1 owned by T6.
  - **#6 root cause — standing.** If a deterministic or recurring failure
    appears during build/test, rerun only to diagnose and record the
    disposition, not to flake-roll.
  - **#7 GUI evidence — not applicable.** T2 has no GUI-rendering
    deliverable.

  **Review lane:** full independent review, because this is the phase's
  schema / IR migration task; the full review includes the trap-#4
  branch-test check.

- **2026-06-13 / T1 addendum 5 — compile-experiment: the trap-#1
  surface, compiler-verified (premise test).** To test the premise that
  the T2 migration surface is "enumerable by grep/reasoning" (my F-3),
  a throwaway `ControlFlowNode::For { binder, index_binder, collection,
  body }` variant was added to `wasamo-ir/lib.rs`, `cargo build
  --workspace` run, the compiler-forced breakage captured, then the
  variant **reverted** (no production code lands — a spike experiment,
  not a change). The premise **partly failed**, in three instructive
  ways:

  - **FE-1 — empirical ≠ grep.** The compiler forced exactly **9
    production sites**: `wasamoc/src/emit.rs:81`, and
    `wasamo-runtime/src/ir_loader.rs` 334 / 365 / 480 / 520 / 574 / 670 /
    971 / 1913. **`wasamoc/emit.rs:81` was *not* in my F-3 grep list**
    (I over-focused on `ir_loader.rs`) — a false negative. Conversely,
    several F-3-listed lines are *not* compiler-forced (below) — false
    positives. So grep over-and-under-counted; the compiler is the
    ground truth.
  - **FE-2 — `cargo build` does not compile `#[cfg(test)]`.** The
    test-module `ControlFlowNode::If` matches (e.g. `ir_loader.rs` ~2254,
    ~3179, the `materialized_index` tests ~2448–2517) did **not** break
    the build — they only break under `cargo test`. So the T2 trap-#1
    audit must run **both** `cargo build` *and* `cargo test` to surface
    the full match set; "release build green" is necessary-not-sufficient
    (the gate's core principle, here concrete).
  - **FE-3 — the dangerous sites are compiler-*invisible*.** The
    `_`-wildcard `ControlFlow` arms silently absorb `For` and the
    compiler says **nothing** — they were absent from the error list.
    Grep found **5**: `wasamo-ir/lib.rs:186` (`widget_children`,
    `IrMember::ControlFlow(_) => None` — the known Phase-6 hotspot,
    *confirmed compiler-silent*), and `ir_loader.rs` 352 / 788
    (`IrMember::ControlFlow(_) => { … }` handler arms — need
    classification) and 459 / 837 (`matches!(m,
    IrMember::ControlFlow(_))` boolean "is control-flow?" tests —
    likely correct under `For`, since `For` *is* control-flow, but must
    be confirmed). **This is the proof that compile-error-forcing does
    not protect the trap-#1 hotspots** — the audit cannot rely on "the
    compiler will enumerate the surface"; it must grep `ControlFlow(_)`
    separately. Validates DD-004 / preamble's `widget_children` emphasis
    with ground truth.

  **Compiler-verified trap-#1 map for `ControlFlowNode::For` (hand to
  T2):**
  | Class | Sites | Audit note |
  |---|---|---|
  | Compiler-forced (production) | `emit.rs:81`; `ir_loader.rs` 334/365/480/520/574/670/971/1913 | each gets a real `For` arm or a deliberate reject |
  | Compiler-silent wildcard (`ControlFlow(_)`) | `lib.rs:186` `widget_children`; `ir_loader.rs` 352/788 (arms), 459/837 (`matches!`) | **the dangerous half** — classify each *correct-filter* vs *bug-under-For*; grep-found, not compiler-found |
  | Parser string-dispatch (additive) | `ir_loader.rs` 1460/1544 (`Token::Ident == "if"`) | add a new `"for"` arm (additive, not a break) |
  | Test-only (need `cargo test`) | `ir_loader.rs` ~2254/~3179; `materialized_index` tests ~2448–2517 | FE-2: surfaced only under `cargo test` |

  **Scope honesty:** the experiment exercised the `ControlFlowNode`
  surface only (one variant). The `IrStateType` (R-A) and `HandlerExpr`
  collection/loop-local/assignment surfaces would each need their own
  scratch variant to be compiler-verified the same way; they remain
  **grep-predicted** until a T2-start experiment (recommended: repeat
  this technique per added variant before writing the audit table —
  cheaper and more accurate than grep, and it is *the* way to find the
  emit.rs:81-class false negatives).

  **Premise / framing conclusions (widening the frame):**
  - **"T1 lands no production code" did not bar the most rigorous
    check.** A reverted scratch experiment is a spike's proper tool and
    out-rigored every grep pass. The plan's no-code framing was read too
    literally as "no compiler runs"; the corrective is "no code *lands*,"
    not "no code *runs*."
  - **"Done = no surprises" is the wrong exit test** — it failed 4×. The
    correct T1 exit is **"every remaining unknown is owned by a named
    task and bounded"** (the coverage table + this map + the CF rows
    achieve that). Restating the exit criterion is the durable fix; the
    perpetual addenda were the symptom of using the wrong one.
  - **Diminishing returns / when to stop.** Past this experiment, further
    T1 deepening (e.g. scratch-verifying the `IrStateType` / `HandlerExpr`
    surfaces) is better done *at T2 start*, where it is that task's own
    audit, run once against the real edit — doing it now would re-incur
    the cost and is the same artifact-compulsion in a new guise. **T1's
    cross-task / plan-structure risk surface is now covered; T1 should
    close.** This stopping judgment is itself part of the discipline.

- **2026-06-13 / T1 addendum 4 — owner-agreement surface review across
  T2–T10.** Critical sweep of every task for decisions needing owner
  agreement (not delegated to the Accepted DDs). Conclusion: **one
  mandatory new consult (T8, re-framed below); no others** — the DD
  slate is owner-settled and the real owner gates already exist;
  manufacturing more consults would be the over-consultation the
  owner-confirm gate warns against. Three *conditional* escalation
  triggers are named (armed before, not after).

  - **Re-framed mandatory consult (T8): G-2 is the structured-item /
    `TypedValue` deferral trigger, not a demo-look choice.** The gallery
    thumbnail's per-item {colour, label} is **record-like data**;
    DD-M3-P7-002 §`TypedValue` pressure names exactly this ("a concrete
    app case where scalar items cannot express the data") as a
    trigger-backed defer that **cannot be smuggled**. Silently picking
    one attribute would consume the trigger without its named
    acceptance-revision path. So the T8 first subtask is upgraded: the
    owner is told the trigger fired, the recommendation is
    **reduce-to-single-attribute for Phase 7** (the trigger routes to
    M4/M5; reopening structured items is against FD-C thesis-sequencing
    and revises M3 acceptance), and the trigger observation lands in the
    **T10 handoff**. Plan T8 + T10 handoff updated. My earlier G-2 plan
    note treated this as composition aesthetics — under-framed; this is
    the correction.
  - **No other mandatory consult.** Confirmed owner-settled by Accepted
    DDs (no consult, implement + record in log): T2 schema shapes
    (DD-002/004, spellings adjustable), T3 author surface
    (`append`/`drop-last` + `in` owner-confirmed at the Accepted flip),
    T5 placement value-space (DD-006 "open for implementation"), T7
    PF2/cap (DD-005/007). Confirmed already-present owner gates (no new
    subtask needed): **T9** owner-manual smoke (an owner gate by
    construction), **T10** phase-end merge approval + the A12 spec
    review-before-commit + the conditional ABI-escalation bullet, and
    the Moment-1 spec drafts already owner-reviewed.
  - **Conditional escalation triggers (fire only if a delegated detail
    turns load-bearing — named here to arm the gate before the design
    decision, not at completion):**
    1. **T4 / T5 / T7 — observable change to the shipped Phase 6
       conditional behaviour.** `if` is a shipped, owner-smoked feature
       that the C1 migration (T4), the placement migration (T5), and the
       splice-seam routing (T7) all touch. Intent is zero observable
       change (regression fixtures are the guard); **if a regression
       fixture must *change* (real behaviour change), escalate**
       (owner-confirm criterion a/b — it alters shipped/accepted
       behaviour). Conditional, not a mandatory subtask.
    2. **T3 — a novel diagnostic-*philosophy* question** beyond
       DD-007's "name the deferral" (criterion b / author-visible) ⇒
       surface. Plain wording stays owner-reviewed at the T10 A12 gate
       (existing), not a T3 consult.
    3. **T7 — PF2 fault-injection infeasibility.** Already DD-005-
       mandated to be *recorded in this log, not silently skipped*;
       surfaced to the owner at the T7 retro (a disposition record, not
       a mid-task consult).
  - **Discipline recorded:** the bar for a *mandatory* owner-consult
    subtask is (product/author-visible **or** AC/phase/cross-task-
    constraint) **and** not settled by a DD **and** not covered by an
    existing gate (T9 smoke / T10 merge / review-before-commit). G-2
    meets all three; the conditional triggers meet only the first, so
    naming them as triggers — not subtasks — is the correct weight
    (over-consulting delegated detail is itself a gate violation).

- **2026-06-13 / T1 addendum 3 — applying the spike's own corrective to
  T1 (load-bearing verification + coverage table + break-first).** The
  retrospective's "目標・前提・計画仮説の再点検" prescribes two artifacts
  before declaring a spike done — (a) an evidence-set coverage table and
  (b) a "where does the plan-hypothesis break first" statement — and
  flags the failure mode of *proposing* a forcing artifact while
  exempting the current task. This entry applies (a)/(b) to T1 itself
  and verifies the single load-bearing assumption of §1.

  - **G-1 (load-bearing design correction — verified in code): the
    binding-eval context is constructed *internally*, so per-item reads
    need new registration entry points, not a reused one.** §1 assumed
    loop-local reads resolve "through a per-item `EvalContext`." But
    `register_binding_with_writer`
    ([reactive.rs:626](../../../../wasamo-runtime/src/reactive.rs#L626))
    and its bool sibling
    ([reactive.rs:666](../../../../wasamo-runtime/src/reactive.rs#L666))
    **construct `BindingEvalContext::new(&registry)` inside the effect
    closure** — the context is not a caller-supplied parameter, and the
    closure writes unconditionally (`Ok(value) => writer(value)`). So
    the instantiation context cannot be *injected* into the existing
    path. T6/T7 must add **new registration entry points** (one per
    element type — i32 / string / bool, mirroring the existing
    string/bool writer seams) whose closure (i) builds a
    `ForItemEvalContext { registry, collection, elem, position }` instead
    of `BindingEvalContext`, and (ii) is **guarded**:
    `Some(v) => writer(v)`, `None => skip` (the out-of-range "write
    nothing" — expressible only in a *new* closure, since the existing
    one always writes). This sharpens §1 and CF-4: the work is "a new
    per-item binding registration API ×3 element types with a guarded
    closure," materially more than §1's "extend the trait + impl."
    Verified directly, because §1 stood entirely on this assumption —
    the exact "load-bearing assumption left unverified" the reflection
    warns about. (Good news: the design holds — the seam is real and the
    guard is expressible; only the implementation shape was understated.)
    Plan T6/T7 updated.
  - **G-2 (T8 composition fact): the current gallery thumbnail varies
    two attributes; a scalar-item `for` binds one.** `gallery.ui`
    already has the `ScrollView { offset-y: scroll_y; WrapPanel { … } }`
    shape with `Box { aspect: 1:1; fill: #RRGGBBAA; Text { text: "S0N" } }`
    children
    ([gallery.ui:133](../../../../examples/gallery/gallery.ui#L133)) — so
    T8's additive `for`-growth into that WrapPanel is feasible
    (DD-007's assumed gallery shape is real). **But** each current
    thumbnail varies **two** per-item attributes — a distinct `fill`
    colour *and* a distinct label (`S01`..) — whereas a scalar
    collection item (`i32[]` / `string[]` / `bool[]`, single bound value
    per item, DD-002) can drive **one**. T1's remit is to **surface this
    constraint, not to pick the demo composition** (deciding the
    gallery's look here would be T1 overreaching into owner-visible demo
    aesthetics — the exact "freeze a complete-looking decision" failure
    the retrospective diagnoses). So the *resolution* (which single
    attribute the item drives — label/id with static fill, or a per-item
    colour with static label, or another) is **deferred to a first T8
    subtask surfaced to the owner** with an options-plus-recommendation
    (default: label/id + static fill), because the gallery is the A8
    positive-control vehicle the owner smokes at T9. Recorded now so the
    constraint is a T8 input, not a T8-time surprise; the choice is the
    owner's at T8 start. Plan T8 updated (the prior draft's prescriptive
    "reduce to label + static fill" is withdrawn as a T1 over-decision).

  **T1 evidence-set coverage table (corrective (a), applied to T1).**
  "Primary landing file" = where each downstream task's main change
  lands; status as of T1 close.

  | Task | Primary landing file(s) | T1 status |
  |---|---|---|
  | T2 IR schema | `wasamo-ir/lib.rs` ✓; `reactive.rs` registry/`Signal` ✓; `ir_loader.rs` `If`-match cluster ✓ (enumerated, addendum 2 F-3); `wasamoc/lower.rs` + `emit.rs` construction sites **✗ deferred** | read except lower/emit |
  | T3 wasamoc surface | `lexer.rs` ✓ (F-5a); `check.rs` namespace/threading ✓ (F-4); `parser.rs` **✗ deferred**; `ast.rs` **✗ deferred**; `lower.rs`/`emit.rs` **✗ deferred** | partial |
  | T4 C1 seam | `ir_loader.rs` `materialized_index*` / `DeclaredMemberSlot` ✓ | read |
  | T5 ST2 placement | `widget.rs` ZStack/insert/remove ✓; `layout.rs` zstack arrange **~ grep-only**; `ir_loader.rs` placement extraction **~ grep-only** | core read; arrange body skimmed |
  | T6 loader static | `ir_loader.rs` build + textual-IR parse ✓; binding-registration path ✓ (G-1) | read |
  | T7 splice + effect | `reactive.rs` effect/binding ✓; `widget.rs` destroy ✓; `handler.rs` eval contexts ✓; registration path ✓ (G-1) | read |
  | T8 gallery | `examples/gallery/gallery.ui` ✓ (G-2) | read |
  | T9 / T10 | docs/spec sync — N/A at T1 | n/a |

  **Deferral judgments (why ✗/~ is a judgment, not a gap):**
  - `lower.rs` / `emit.rs` (T2/T3): the exhaustive `IrState` /
    `HandlerExpr` construction-site enumeration **is the T2/T3 trap-#1
    close artifact** — reading them now to enumerate would duplicate that
    task's own audit. Boundary defensible *because* addendum 2 already
    pinned the highest-density cluster (`ir_loader.rs` `If`-matches).
  - `parser.rs` (T3): the plan's "LL(1) after the first `IDENT`" claim is
    sound **by construction** — `for` becomes a reserved keyword token
    (lexer `Keyword` enum, F-5a), so the member parser dispatches on it
    with no backtracking, exactly like the existing control-flow member.
    Reading 1279 lines to re-confirm a keyword-led dispatch is not
    warranted at T1; residual risk noted, owned by T3.
  - `layout.rs` zstack arrange (T5): read via grep to confirm it reads
    `zstack_placements` parallel-vector (the migration target); the full
    arrange body is T5's own regression surface (Phase 6 ZStack fixtures).

  **Break-first statement (corrective (b)).** If the T1 design/order is
  wrong, the first break was the **per-item binding registration path**
  (G-1) — now *verified* to support new guarded entry points, so that
  risk is closed. The top *residual* therefore shifts to the
  **deferred-unread** set: T3's `for`-header parse (mitigated:
  keyword-led ⇒ LL(1) by construction) and the T2/T3 exhaustive
  evaluator + `If`-match migration (mitigated: cluster pinned, audit is
  those tasks' close artifact). No residual rises to a T1-blocking
  unknown; each is owned by a named task with a recorded mitigation.

- **2026-06-13 / T1 addendum 2 — deeper code pass (wasamoc, disposal,
  loader parser).** The first spike read only the runtime *structural*
  side; a second critical pass read the previously-unread areas each
  later task actually lands in — `wasamoc` (`lexer.rs` / `check.rs`),
  the disposal/teardown path
  ([`widget.rs`](../../../../wasamo-runtime/src/widget.rs)
  `widget_destroy`), the textual-IR **parser** half of `ir_loader.rs`,
  and the two `EvalContext` impls. Five findings; F-1/F-3/F-4 are
  scope-relevant to T2/T3/T7 (no reorder), F-5 is confirmation.

  - **F-1 (T7 hard constraint): per-item binding effects must be owned
    by the generated *child* subtree, not the parent.** Teardown is
    `widget_destroy`
    ([widget.rs:1786](../../../../wasamo-runtime/src/widget.rs#L1786))
    → `dispose_subtree_bindings`
    ([widget.rs:1792](../../../../wasamo-runtime/src/widget.rs#L1792)),
    which clears `WidgetNode.bindings`
    ([widget.rs:327](../../../../wasamo-runtime/src/widget.rs#L327))
    recursively over the subtree, then severs the registry, then drops.
    The Phase 6 conditional stores its effect on the **parent**
    (`parent.bindings.push(handle)`,
    [ir_loader.rs:1969](../../../../wasamo-runtime/src/ir_loader.rs#L1969)).
    That pattern is correct for the `ForLoopSubtree` **structural**
    effect (it outlives individual items, like the conditional) but
    **wrong for the per-item value/index effects**: on a tail-removal
    T7 calls `widget_destroy(removed_child)`, which disposes only the
    *child subtree's* bindings — so a per-item effect parked on the
    parent would **leak** (and keep reading a freed position). So the
    ownership rule is split: `ForLoopSubtree` effect → parent.bindings;
    per-item value/index effects → the generated child subtree root's
    bindings. My §1 record (and the conditional analogy it leaned on)
    did not pin this; it is a correctness constraint, not a style note.
  - **F-2 (T7 trap-#2 sharpening): two of the six side-effects already
    exist as infrastructure.** `widget_destroy` already performs
    DD-006 side-effect #5 (effects disposed ahead of teardown) and #4
    (registry release) in the order bindings → registry → drop,
    recursively. So T7's removal path **reuses** `widget_destroy` per
    removed subtree (tail-first) rather than rebuilding #4/#5; the
    splice seam's *new* work is the children-vector splice (#1), Visual
    sibling order (#2), layout invalidation (#3), and staged-insert
    effect attach (#5 insert side). The trap-#2 close artifact should
    mark #4/#5-removal as **reused**, not re-implemented — my §2/§3
    treated all six as freshly enumerated.
  - **F-3 (T2 audit scope, materially larger): `ir_loader.rs` has ~12
    non-test `ControlFlowNode::If`-only match sites** that go
    non-exhaustive the moment T2 adds `For`: lines 336, 367, 482, 522,
    576, 672, 973, 1460 (parse dispatch), 1544 (nested-`if` in
    `parse_if_member`), 1931 (`append_static_member`), 2254, plus the
    emit site 3179. My §3 R-A list named only `append_static_member` +
    `materialized_index_for_declared_member`. **This is exactly the
    Phase-6 `widget_children` failure mode multiplied** — each of these
    must be classified (real `For` arm vs deliberate reject) in the T2
    trap-#1 audit table; several are in validation / counting /
    placement-collection helpers where a silently-missing `For` arm
    would mis-validate or mis-count. The §3 R-A site list is extended
    accordingly (below).
  - **F-4 (T3 scope, wider than "reject rows"): binder scope is new
    `check` machinery.** `check.rs` carries a **flat, state-only**
    `Namespace` (name→type,
    [check.rs:55](../../../../wasamoc/src/check.rs#L55)) built by
    `collect_state_namespace` and threaded immutably through
    `check_members_inner`
    ([check.rs:1335](../../../../wasamoc/src/check.rs#L1335) — which
    already gained a `parent_widget` param in Phase 6). DD-003's binder
    scope (binders added entering a `for` body, removed leaving it;
    binder-vs-state collision; index-vs-value collision; reads only
    inside the body) is a **new scoped dimension threaded alongside
    `ns`**, the direct analogue of the Phase-6 `parent_widget`
    threading — not just additional reject arms. The qualified-name
    resolver (`check.rs:1686`) already covers the DD-001/002
    qualified-reference rejects. T3 is "parser + binder-scope threading
    + reject rows," and the scope threading is the load-bearing part.
  - **F-5 (confirmations).** (a) The `.ui` lexer already lexes
    **kebab-case identifiers as single tokens** (`scan_ident`
    [lexer.rs:347](../../../../wasamoc/src/lexer.rs#L347), continuing on
    `-` + alpha, [lexer.rs:366](../../../../wasamoc/src/lexer.rs#L366)),
    with `in-out` pinned by an existing test
    (`in_outx_lexes_as_kebab_ident`) and a `Keyword` enum + reserved
    control-flow family already present — so T3's `in` reservation is
    mechanical and DD-002's contextual `append` / `drop-last` (and a
    *state* named `drop-last`) are lexically sound as single tokens.
    (b) The textual-IR parser is hand-rolled recursive descent:
    `parse_for_member` mirrors `parse_if_member`
    ([ir_loader.rs:1530](../../../../wasamo-runtime/src/ir_loader.rs#L1530),
    dispatched at the member loop's `Token::Ident == "if"` site
    [ir_loader.rs:1460](../../../../wasamo-runtime/src/ir_loader.rs#L1460)),
    and the collection atoms (`list-prop-read` / `list-append` /
    `list-drop-last` / `list`) slot into `parse_expr`'s atom-head
    dispatch ([ir_loader.rs:1645](../../../../wasamo-runtime/src/ir_loader.rs#L1645),
    beside `str-prop-read` / `bool-prop-read` at 1680/1684) — T2/T6
    additions are mechanical. (c) The instantiation-context impl is
    concrete: `BindingEvalContext<'a> { registry }`
    ([reactive.rs:416](../../../../wasamo-runtime/src/reactive.rs#L416))
    is a thin registry wrapper implementing `EvalContext`, so
    `ForItemContext` is the same wrapper plus `{ collection, elem,
    position }`; CF-6 extends `HandlerEvalContext`
    ([reactive.rs:493](../../../../wasamo-runtime/src/reactive.rs#L493)).

  **§3 R-A site list extended (F-3):** add the ~12 `ir_loader.rs`
  `ControlFlowNode::If` match sites above and the `wasamoc` lower/emit
  `IrState` / `HandlerExpr` construction sites
  ([`lower.rs`](../../../../wasamoc/src/lower.rs),
  [`emit.rs`](../../../../wasamoc/src/emit.rs)) — still to be read
  exhaustively at T2 start, where the trap-#1 audit table is the close
  artifact — to the previously-named lib.rs / reactive.rs / registry
  sites. The audit is **T2's** close artifact; T1's contribution is
  pinning that the `If`-match-site sweep over `ir_loader.rs` is the
  highest-density trap-#1 cluster.

  **Method learning (for the gate template / memory):** the original
  spike declared completion having read only the *runtime structural*
  files; the two corrections (addendum 1 evaluator seam, this pass's
  F-1/F-3/F-4) both came from the files a spike that audits "where does
  each later task land" would have read first. A pre-implementation
  spike's source set must cover **every task's primary landing file**,
  not just the phase's headline structural refactor.

- **2026-06-13 / T1 addendum — critical re-examination of the spike
  (evaluator seam + ownership gap).** A critical second pass over
  [`handler.rs`](../../../../wasamo-runtime/src/handler.rs) (the
  `EvalContext` trait + the five `evaluate_*` / `invoke_handler` match
  sites) and `Signal::set`
  ([reactive.rs:230](../../../../wasamo-runtime/src/reactive.rs#L230))
  found that the original spike entry below **overstated "no plan
  change"**. Four corrections, one of which **requires a plan revision**:

  1. **The instantiation-context seam is the `EvalContext` trait, not a
     bespoke closure (corrects §1).** Runtime expression evaluation goes
     through `trait EvalContext`
     ([handler.rs:12](../../../../wasamo-runtime/src/handler.rs#L12) —
     `get_i32` / `set_i32` / `read_i32_tracked` / `read_string_tracked` /
     `get_bool` / `read_bool_tracked` / `set_bool`, default-impl methods
     for additive back-compat) implemented by `BindingEvalContext`
     (reactive reads) and `HandlerEvalContext` (live writes). The
     faithful shape of `ForItemContext` is therefore **an `EvalContext`
     implementation that carries `position`**, resolving the loop-local
     reads via **new tracked trait methods** (e.g. `read_item_i32` /
     `read_item_string` / `read_item_bool`) that read
     `collection_signal.get()[position]` with the out-of-range guard
     returning the "write nothing" path — *not* a closure capturing a
     plain struct as §1 phrased it. The §1 datum (`collection`, `elem`,
     `position`) is unchanged; what changes is that the read resolves
     through the trait, so T6/T7's surface is "extend the `EvalContext`
     trait + its binding-context impl," wider than §1's "add a guarded
     read." DD spellings stay adjustable (no DD reopen).
  2. **Plan-ownership gap (requires revision): the handler-side
     collection-assignment evaluation is unowned.** The authored
     `thumbs = thumbs.append(x)` runs **inside a handler**
     (`invoke_handler` / `evaluate` → `HandlerEvalContext` →
     `Signal::set`). It is a whole-`Vec` read-modify-write needing a new
     `HandlerExpr` arm in the handler evaluator **and** a new
     `EvalContext` collection-write method. This is the *writer* that
     drives the signal T7's `for` effect (the *reader*) reacts to — so
     T7's mutation fixtures depend on it — yet the plan's T7 bullets name
     only the splice seam, `ForLoopSubtree` + effect, per-item bindings,
     Windows fixtures, and cap fixtures; **none names the handler-side
     assignment evaluation**, and T6 (loader static path) does not cover
     it. **Resolution: T7 gains an explicit bullet** for the handler-side
     collection-assignment evaluation (read-modify-write on the
     whole-value signal via an extended `HandlerEvalContext`; the
     equal-value no-dirty rule). Plan revised in this commit batch
     (retrospectives.md §11 ownership-on-Task-list mandate). New
     carry row **CF-6**.
  3. **§3 R-A omitted the handler.rs evaluator match sites (corrects
     §3).** When T2 widens `HandlerExpr`, the **five exhaustive matches**
     in `handler.rs` — `evaluate`
     ([handler.rs:95](../../../../wasamo-runtime/src/handler.rs#L95)),
     `evaluate_tracked`
     ([handler.rs:344](../../../../wasamo-runtime/src/handler.rs#L344)),
     `evaluate_binding`
     ([handler.rs:277](../../../../wasamo-runtime/src/handler.rs#L277)),
     `evaluate_binding_part`
     ([handler.rs:301](../../../../wasamo-runtime/src/handler.rs#L301)),
     `evaluate_bool_binding`
     ([handler.rs:328](../../../../wasamo-runtime/src/handler.rs#L328)),
     plus `invoke_handler`
     ([handler.rs:235](../../../../wasamo-runtime/src/handler.rs#L235)) —
     all break the build and are **among the most-affected trap-#1
     sites**. The T2 call-site audit table **must enumerate them**; §3's
     R-A site list (which named lower/emit/check/ir_loader/registry/
     `BindingTarget` only) is extended here.
  4. **`Signal::set` has no equal-value short-circuit today (sharpens
     CF-5).** `set`
     ([reactive.rs:230](../../../../wasamo-runtime/src/reactive.rs#L230))
     assigns unconditionally and always marks dependents dirty — so
     DD-002's "equal-value writes mark nothing dirty" is a **new**
     `PartialEq`-gated behaviour T2 adds for the collection signals, not
     a configuration of existing behaviour (CF-5 said "ship with," which
     understated it). This is exactly the DD-005 §Technical-risk "if
     absent, a bounded note lands in the plan" branch — confirmed
     **absent**. `Signal<T>` is `T: Clone + 'static`
     ([reactive.rs:213](../../../../wasamo-runtime/src/reactive.rs#L213)),
     so `Signal<Vec<_>>` is fine and `Vec: PartialEq` satisfies the gate
     (no blocker — recorded as the one *confirmed-clear* item).
  5. **Self-inflicted (corrects §2 Seam C):** §2 said
     `BindingTarget::ForLoopSubtree` "lands in T2's `BindingTarget`
     migration." Wrong mechanism — `register_binding`
     ([reactive.rs:607](../../../../wasamo-runtime/src/reactive.rs#L607))
     and `register_conditional_binding` destructure with **let-else, not
     exhaustive match**, so adding a `BindingTarget` variant does not
     force T2 changes; the plan's placement of `ForLoopSubtree` in **T7**
     (where first used) is correct and avoids a dead variant. Seam C is
     corrected inline below.

  **Net:** order T2→T7 unchanged; **one plan revision** (T7 gains the
  handler-side assignment-evaluation bullet, CF-6); design record
  corrected on the evaluator seam (1/3/5) and the equal-value rule (4).
  The original "no plan change" claim is withdrawn.

- **2026-06-13 / T1 — Pre-implementation spike: instantiation context,
  bisectable sequencing, risk sharpening, and the T2 gate selection.**
  T1 lands **no production code** (per
  [plan.md §T1](./plan.md)); its deliverables are the recorded design
  decisions below plus the plan revisions they imply. Task branch
  `feat/m3-phase-7-t1`. The three plan bullets are discharged in
  §1 (instantiation context), §2 (sequencing), §3 (risk sharpening +
  T2 gate selection). Sources read against current `HEAD` (`e97361c`):
  [`wasamo-ir/src/lib.rs`](../../../../wasamo-ir/src/lib.rs),
  [`wasamo-runtime/src/reactive.rs`](../../../../wasamo-runtime/src/reactive.rs),
  [`wasamo-runtime/src/ir_loader.rs`](../../../../wasamo-runtime/src/ir_loader.rs),
  [`wasamo-runtime/src/widget.rs`](../../../../wasamo-runtime/src/widget.rs),
  [`wasamo-runtime/src/layout.rs`](../../../../wasamo-runtime/src/layout.rs),
  [`wasamo-runtime/src/registry.rs`](../../../../wasamo-runtime/src/registry.rs).

  ### T1 start gate (implementation-gates selection for T1 itself)

  T1 produces no schema change, no branch, no tree mutation, and no GUI
  deliverable, so the structural traps do not apply to T1's own work;
  the one trap that does apply is the one T1 *is*:

  - **#1 semantic migration** — *not applicable to T1*: T1 adds no
    enum/schema variant (it only *designs* the T2 ones). The audit
    obligation it produces is recorded as the T2 gate selection (§3).
  - **#2 side effects / #3 parallel data / #7 GUI** — *not applicable*:
    no runtime mutation, no parallel vector touched, no GUI render in
    T1.
  - **#4 untested branch** — *not applicable*: no code branch is added.
  - **#5 carry-forward** — **applies; T1's entire output is
    carry-forward.** The instantiation-context shape, the sequencing
    seams, and the T2 gate selection are recorded here (log.md) and in
    [plan.md](./plan.md) with re-trigger criteria, so the downstream
    tasks consume a written record, not memory.
  - **#6 root cause** — standing; nothing recurring to disposition at
    T1.
  - **Review lane:** T1 is a design spike with no executable change; the
    task-end gate is owner review of the recorded design (no full code
    review, because there is no code). The high-risk review lanes it
    *assigns* (T2 schema, T5/T7 runtime structural) are recorded in §3.

  ### 1. Instantiation context type (plan T1 bullet 1)

  **Problem.** A `for` body template is one IR subtree shared by all N
  generated positions (DD-M3-P7-004 S1: `ControlFlowNode::For.body` is a
  single `Widget` member). A per-item binding such as `label: thumb`
  must therefore resolve its **value** and **index** from data supplied
  *at materialisation*, not baked into the shared template. The shipped
  binding path has no such per-instance datum: `register_binding`
  ([reactive.rs:601](../../../../wasamo-runtime/src/reactive.rs#L601))
  evaluates a `HandlerExpr` against the whole `SignalRegistry` and always
  writes the evaluated `String`; reads are by state name
  (`HandlerExpr::PropRead { path }`,
  [lib.rs:48](../../../../wasamo-ir/src/lib.rs#L48)). There is no concept
  of "the current item at position *i*".

  **Existing shape this generalises.** The Phase 6 conditional path is
  the structural precedent. `DeclaredMemberSlot`
  ([ir_loader.rs:92](../../../../wasamo-runtime/src/ir_loader.rs#L92)) is
  `Widget | Conditional(Rc<RefCell<ConditionalRuntimeState>>)` with
  `ConditionalRuntimeState { live_child: bool }`
  ([ir_loader.rs:97](../../../../wasamo-runtime/src/ir_loader.rs#L97));
  `materialized_index_for_declared_member`
  ([ir_loader.rs:1975](../../../../wasamo-runtime/src/ir_loader.rs#L1975))
  is the prefix-sum (Widget→1, Conditional→0/1) that the C1 seam
  generalises (T4). The conditional subtree is built once by
  `build_node` and re-built fresh on toggle; iteration needs **per-item**
  state because each position carries its own live read.

  **Decision — recommended shape.** The instantiation context is a
  *runtime* construct (not an IR construct): the body template stays
  position-agnostic, and the loader (T6) / `for` effect (T7) supply the
  per-instance context when materialising each position.

  ```rust
  /// Supplied once per generated subtree, at materialisation time.
  /// Fixes the subtree's position in the collection; the value-binder's
  /// reads resolve `collection[position]` *live* on the whole-value
  /// signal (DD-005 V2), the index-binder resolves to `position`
  /// (constant under tail-only mutation). DD/field spellings adjustable.
  struct ForItemContext {
      /// Registry key of the iterated collection state (DD-002 R1
      /// whole-value signal). Fixed for the whole body — lives here, not
      /// duplicated into every loop-local read.
      collection: String,
      /// Element scalar type. Selects the collection signal map and the
      /// evaluator/writer pair (mirrors the existing per-type writer
      /// seam, architecture.md §6.7.x). Fixed per `for`.
      elem: IrType,
      /// This item's fixed position = the materialised offset within the
      /// `for` slot's range. Read live; out-of-range → write nothing.
      position: usize,
  }
  ```

  Two IR-side companions (their exact spelling is T2/DD-002's, recorded
  here only as the shape the context serves):

  - **Loop-local reads are bare markers** in the body template —
    `ItemValueRead` (the current element) and `ItemIndexRead` (the
    current position) `HandlerExpr` variants carrying **no** collection
    name and **no** concrete index. Everything fixed-per-`for`
    (`collection`, `elem`) and everything fixed-per-instance (`position`)
    comes from the `ForItemContext` the binding effect closes over.
    Rationale: under flat scope with no nesting (DD-003), a body is
    instantiated under exactly one `for`, so the collection/elem are
    invariant across the body; keeping them out of the read markers keeps
    the *shared template* free of per-`for` data and lets the loader
    validate "a loop-local read appears only inside a `for` body"
    structurally.
  - **Considered alternative — reads carry the tag**
    (`ItemValueRead { collection, elem }`, context carries only
    `position`). Closer to DD-005's literal "`collection[i]`" phrasing
    and makes the IR self-describing for loader cross-checks, but
    duplicates the `for` header's collection/elem into every body read
    and splits the per-instance datum (`position`) from the
    per-`for` data across two carriers. **Rejected on merit:** the
    context already exists to carry `position`, so folding the two
    invariants into it costs nothing and keeps the template minimal.
    (DD variant spellings stay adjustable — if T6/T7 finds the loader
    validation wants the tag on the read, it may move without reopening
    a DD.)

  **Live / out-of-range guard (DD-005 V2).** The value read is
  `registry.collection_signal(elem)[collection].get().get(position)` →
  `Option<scalar>`. `None` (position ≥ current length) ⇒ the binding
  **writes nothing**. This is a real extension to the binding-evaluation
  path: today `register_binding_with_writer`
  ([reactive.rs:620](../../../../wasamo-runtime/src/reactive.rs#L620))
  unconditionally writes the evaluated `String`. The loop-local path
  needs a *guarded* evaluation that can yield "skip" (e.g. evaluate to
  `Option<String>`, write only `Some`). Recorded as the binding-path
  shape T6 (static reads) and T7 (the same-batch doomed-binding read,
  DD-005 / DD-007 cap row) must implement and directly test; it is the
  positive control for the doomed-binding no-panic fixture.

  **Per-`for` runtime slot.** Mirroring `DeclaredMemberSlot::Conditional`,
  iteration adds `DeclaredMemberSlot::ForLoop { … }` carrying the slot's
  live cardinality / generated-subtree state (so the C1 seam can sum it
  and the splice can address its range). Whether it stores a count plus
  external per-item effects, or a `Vec` of per-item records, is a
  T4/T6/T7 implementation choice; T1 fixes only that the **stable
  identity is the declared slot** and the materialised range
  `[offset, offset+cardinality)` is recomputed via the seam, never cached
  (DD-004 / DD-005). The `ForItemContext.position` is an index into that
  range.

  **Why a runtime context, not an IR field (thesis check).** Cardinality
  is runtime data (FD-A); the position cannot be lowered into the shared
  template without static expansion, which the thesis rejects (DD-004
  S3). The context is the minimal per-instance carrier that keeps the
  template shared and the reads live — it is the iteration analogue of
  the conditional's `ConditionalRuntimeState`, widened from a `bool` to a
  position.

  ### 2. Bisectable sequencing (plan T1 bullet 2)

  **Decision: keep the plan's default order**
  T2 (I2 schema) → T3 (`wasamoc` surface) → T4 (C1 seam) →
  T5 (ST2 placement) → T6 (loader static path) →
  T7 (splice primitive + `for` effect). It is dependency-correct and
  bisectable; T1 does **not** reorder. What T1 adds is the explicit
  record of the **three inter-task seams** that keep each intermediate
  commit building and each task reviewable in isolation:

  - **Seam A — T2's loader `For` arm is a deferred-load reject.** T2 is
    the compile-error-forcing schema bundle (R-A): adding
    `ControlFlowNode::For` makes the `IrMember::ControlFlow` match in
    `append_static_member`
    ([ir_loader.rs:1931](../../../../wasamo-runtime/src/ir_loader.rs#L1931))
    non-exhaustive. T2 keeps the build green by adding a `For` arm that
    returns an `IrLoadError` ("`for` not yet materialised") — a *real,
    directly-tested* reject branch (trap #4) that T6 replaces with static
    materialisation. The three registry collection signal maps land in T2
    (`SignalRegistry`,
    [reactive.rs:391](../../../../wasamo-runtime/src/reactive.rs#L391))
    but are only *read* by T6+ and *written* by T7; T2 ships them with
    the value-equality-on-set rule (DD-002: equal-value writes mark
    nothing dirty) and its unit test.
  - **Seam B — T4 introduces `DeclaredMemberSlot::ForLoop` ahead of its
    first construction.** The plan's T4 unit suite must cover
    interleaved `if`/`for`/static siblings and tail insert/remove plan
    derivation (plan T4 bullet 3), which requires the seam to handle a
    `For` cardinality arm. So T4 lands the `ForLoop` slot variant + the
    seam's cardinality arm and unit-tests it, even though the loader does
    not *construct* a `ForLoop` slot until T6. The variant is therefore
    **dead (unconstructed) between T4 and T6** — a recorded, bounded
    `dead_code` allowance whose closure is T6, not a smell (it exists to
    make the pure-logic seam testable before the WinRT loader path
    lands). Carry-forward row below.
  - **Seam C — T6 registers the `ForLoopSubtree` effect with a no-op
    initial reconcile; T7 fills its tail-edit body.** T6 owns static
    materialisation (walk the seam at load) **and** registering the
    `BindingTarget::ForLoopSubtree`
    ([reactive.rs:585](../../../../wasamo-runtime/src/reactive.rs#L585) —
    the `ForLoopSubtree` variant is added in **T7**, where first used:
    `register_binding` and `register_conditional_binding` destructure
    `BindingTarget` with let-else, not an exhaustive match, so the new
    variant forces no T2 change and needs no dead variant; see the T1
    addendum correction 5)
    effect, whose **initial run is reconciled to a no-op** so static load
    + first effect run do not double-create children (plan obligation 3 /
    plan T6 bullet 3, the explicit test). T7 fills the effect body with
    the stage-then-commit tail insert/remove through the splice seam. So
    between T6 and T7 the initial render is correct and no shipped
    example issues a collection mutation (the gallery `Add`/`Remove`
    arrives in T8; the mutation headless fixtures are T7's). The T6→T7
    effect-body stub is a carry-forward row below.

  **Cross-task dependency facts recorded for the reviewer:**

  - **Both T4 (offsets) and T5 (carried placement) precede T7**, because
    the T7 splice seam consumes both. T4 and T5 are mutually independent
    (index math vs placement storage) and build in either order; the plan
    keeps T4→T5.
  - **T5 must keep the conditional mutation path green under
    child-carried placement before the unified splice seam exists.** The
    Phase 6 conditional path calls `insert_child` / `remove_child` /
    `insert_child_with_zstack_placement` directly
    ([ir_loader.rs:2021-2054](../../../../wasamo-runtime/src/ir_loader.rs#L2021));
    T5 changes how those carry placement (child-slot, not parallel
    vector), so T5 updates that path and re-greens the Phase 6 ZStack
    fixtures *before* T7 wraps the six-effect bundle into one seam and
    routes both conditional and `for` through it (DD-006).
  - **T1's instantiation context is consumed by T6** (static per-item
    reads at load) **and T7** (live per-item reads under mutation).

  No plan reorder results; the plan's T1 bullets are checked and these
  seams are cited from the plan tasks (see plan edits in this commit
  batch).

  ### 3. Risk-table sharpening + T2 gate selection (plan T1 bullet 3)

  **Sharpened R-A / R-B / R-C hotspots (pinned to current source).**

  - **R-A (I2 compile-error-forcing schema migration).** Schema sites in
    [`wasamo-ir/src/lib.rs`](../../../../wasamo-ir/src/lib.rs):
    `IrState.ty: IrType` → `IrStateType`
    ([lib.rs:86](../../../../wasamo-ir/src/lib.rs#L86));
    `IrLiteral` gains `List(Vec<IrLiteral>)`
    ([lib.rs:14](../../../../wasamo-ir/src/lib.rs#L14));
    `HandlerExpr` gains the collection read + loop-local reads +
    assignment forms
    ([lib.rs:44](../../../../wasamo-ir/src/lib.rs#L44));
    `ControlFlowNode` gains `For`
    ([lib.rs:149](../../../../wasamo-ir/src/lib.rs#L149)). **Trap-#1
    hotspot:** `IrNode::widget_children()`
    ([lib.rs:176](../../../../wasamo-ir/src/lib.rs#L176)) — a widget-only
    filter that already drops `ControlFlow` members; every use must be
    classified *correct* (layout-time over materialised children) or *a
    bug under `For`* (traversal over declared members). Construction /
    match sites to migrate: `wasamoc` `lower.rs` / `emit.rs` / `check.rs`;
    runtime `ir_loader.rs` emit + parse + `append_static_member`
    ([ir_loader.rs:1904](../../../../wasamo-runtime/src/ir_loader.rs#L1904))
    + `materialized_index_for_declared_member`
    ([ir_loader.rs:1975](../../../../wasamo-runtime/src/ir_loader.rs#L1975));
    `SignalRegistry` + `new()`
    ([reactive.rs:391-399](../../../../wasamo-runtime/src/reactive.rs#L391));
    `BindingTarget`
    ([reactive.rs:585](../../../../wasamo-runtime/src/reactive.rs#L585));
    every `IrState { ty: … }` literal across `wasamoc` + runtime + tests.
  - **R-B (C1 seam touches the shipped conditional path).** Extract /
    generalise `materialized_index_for_declared_member`
    ([ir_loader.rs:1975](../../../../wasamo-runtime/src/ir_loader.rs#L1975));
    migrate `mutate_conditional_subtree`
    ([ir_loader.rs:1989](../../../../wasamo-runtime/src/ir_loader.rs#L1989))
    onto the seam as the 0/1 case; `DeclaredMemberSlot` /
    `ConditionalRuntimeState`
    ([ir_loader.rs:92-99](../../../../wasamo-runtime/src/ir_loader.rs#L92))
    gain `ForLoop`; `register_conditional_binding`
    ([reactive.rs:674](../../../../wasamo-runtime/src/reactive.rs#L674))
    is the `for`-effect analogue (T7). Regression gate: the
    materialised-index unit tests
    ([ir_loader.rs:2448-2517](../../../../wasamo-runtime/src/ir_loader.rs#L2448))
    and `wasamo-runtime/tests/conditional_toggle_integration.rs`.
  - **R-C (ST2 touches shipped arrange / loader).** Placement storage:
    `WidgetData::ZStack { zstack_placements }`
    ([widget.rs:181](../../../../wasamo-runtime/src/widget.rs#L181)),
    `WidgetNode::zstack`
    ([widget.rs:648](../../../../wasamo-runtime/src/widget.rs#L648)),
    the placement insert/remove inside `insert_child_inner`
    ([widget.rs:1373](../../../../wasamo-runtime/src/widget.rs#L1373)) /
    `remove_child`
    ([widget.rs:1400](../../../../wasamo-runtime/src/widget.rs#L1400)),
    and `insert_child_with_zstack_placement`
    ([widget.rs:1325](../../../../wasamo-runtime/src/widget.rs#L1325)).
    Arrange read: `LayoutNode.zstack_placements`
    ([layout.rs:252](../../../../wasamo-runtime/src/layout.rs#L252)),
    `LayoutNode::zstack`
    ([layout.rs:479](../../../../wasamo-runtime/src/layout.rs#L479)),
    the zstack arrange loop
    ([layout.rs:1382-1405](../../../../wasamo-runtime/src/layout.rs#L1382)),
    and the `WidgetData::ZStack` → `LayoutNode::zstack` bridge
    ([widget.rs:1634](../../../../wasamo-runtime/src/widget.rs#L1634)).
    Loader extraction re-targets: `collect_static_zstack_placements`
    ([ir_loader.rs:2246](../../../../wasamo-runtime/src/ir_loader.rs#L2246)),
    `zstack_placement_for_parent`
    ([ir_loader.rs:2057](../../../../wasamo-runtime/src/ir_loader.rs#L2057)),
    `extract_zstack_placement` (used at
    [ir_loader.rs:1922](../../../../wasamo-runtime/src/ir_loader.rs#L1922)).
    Grid stays parallel + static-only: `WidgetData::Grid { cell_placements }`
    ([widget.rs:170-173](../../../../wasamo-runtime/src/widget.rs#L170))
    SoA comment gains the DD-M3-P7-006 trigger pointer; the trap-#3 close
    artifact is "`zstack_placements` deleted (greppable); `cell_placements`
    static-only with DD pointer".
  - **R-E / R-F (T7).** The guarded `ItemRead` is the
    `ForItemContext.position` out-of-range branch (§1); the cap fixtures
    exercise `MUTATION_CAP = 16`
    ([reactive.rs:10](../../../../wasamo-runtime/src/reactive.rs#L10)) —
    DD-007 confirms the cap charges drain **depth**, so a ≫N
    tail-append (e.g. 64) converges in one non-empty drain iteration.
    No sharpening beyond DD-005/DD-007; recorded for completeness.

  **T2 implementation-gates selection (recorded before T2 opens —
  plan T1 bullet 3 / preamble obligation 2).** T2 = the schema /
  IR-migration full-review-lane task.

  - **#1 semantic migration — APPLIES (the task's core).** Close
    artifact: the `rg`-enumerated call-site audit table over `IrState` /
    `IrMember` / `ControlFlowNode` / `HandlerExpr` (+ a `BindingTarget`
    pre-audit for T7), each site classified
    extended / correctly-unaffected / deliberately-rejects, with
    `IrNode::widget_children()` and every widget-only filter explicitly
    classified (the exact Phase 6 failure mode). Recorded in this log at
    T2 close.
  - **#2 side effects — not applicable to T2.** T2 makes no materialised-
    tree mutation; the registry collection maps are a state-store
    addition, not a structural edit with derived layout/Visual effects
    (those live in T5/T7).
  - **#3 parallel data drift — not applicable to T2.** T2 touches no
    placement vector (T5 owns that); the new registry maps are keyed by
    state name, not parallel to a child list.
  - **#4 untested branch — APPLIES (narrowly).** T2's own new reject
    branches — the deferred-load `For` arm (Seam A), and any
    `IrLiteral::List` / `IrStateType` loader element-type / nesting /
    list-on-scalar rejects that land in T2 rather than T3/T6 — each ship
    with a directly-firing test (the full DD-007 matrix is T3/T6).
  - **#5 carry-forward — APPLIES.** Seams A/B/C and the registry
    value-equality-on-set contract are invariants T4/T6/T7 depend on;
    recorded as the carry rows below with re-triggers.
  - **#6 root cause — standing**, not pre-selected.
  - **#7 GUI evidence — not applicable** (T2 has no GUI deliverable).
  - **Review lane:** **full independent review** (schema / IR migration
    high-risk class), composing in the trap-#4 branch/test check.

  ### Carry-forward rows (re-trigger criteria)

  | # | Carry | Owner / re-trigger |
  |---|---|---|
  | CF-1 | Seam A: T2's loader `For` arm is a deferred-load reject with a direct test | **T6** replaces it with static materialisation; re-trigger = T6 opening |
  | CF-2 | Seam B: `DeclaredMemberSlot::ForLoop` is dead (unconstructed) between T4 and T6 | **T6** first constructs it; re-trigger = T6 opening (the `dead_code` allowance closes there) |
  | CF-3 | Seam C: T6 registers `ForLoopSubtree` with a no-op initial reconcile; effect body stubbed | **T7** fills the stage-then-commit tail-edit body; re-trigger = T7 opening |
  | CF-4 | Guarded loop-local read ("write nothing" on out-of-range position) is a new binding-eval branch | **T6** (static) + **T7** (same-batch doomed binding) implement + directly test it; re-trigger = first loop-local read lowering |
  | CF-5 | T2 **adds** a `PartialEq`-gated equal-value-no-dirty set for the collection signals (currently `Signal::set` has no short-circuit — confirmed absent, the DD-005 "if absent" branch) | **T7** cap accounting + the empty-`drop-last` no-dirty fixture rely on it; re-trigger = first collection write path |
  | CF-6 | Handler-side collection-assignment evaluation (whole-`Vec` read-modify-write via an extended `HandlerEvalContext` + a new `HandlerExpr` evaluator arm) is the *writer* the `for` effect reacts to | **T7** (new explicit plan bullet); re-trigger = T7 mutation fixtures need an authored `append` / `drop-last` to drive a signal change |

  ### T1 close gate (artifacts)

  - **#5 carry-forward (the only applying trap):** recorded above as the
    CF-1..CF-5 table with owners and re-triggers, plus the design record
    in §1 and the sequencing seams in §2. The downstream consumers (T2
    gate selection, T6/T7 seams) read this log, not memory.
  - **Build/test sanity (no production Rust changed by T1):** the
    workspace `cargo build --workspace` is green and `cargo test
    --workspace` was run as a baseline proxy (T1 adds no Rust, so the
    fmt / clean-rebuild gate is the merge-base state; per the preamble
    the local clean-rebuild gate is owned by a task only when it changes
    production Rust). Recorded in the T1 retrospective
    ([../retrospectives/t1.md](../retrospectives/t1.md)).
