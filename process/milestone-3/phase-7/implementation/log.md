## Decisions log

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
