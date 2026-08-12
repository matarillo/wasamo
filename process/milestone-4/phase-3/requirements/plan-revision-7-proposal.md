---
title: M4 plan Revision 7 proposal — a seventh Phase 3 decision record for what a dot means
status: landed
created: 2026-08-12
authorised: 2026-08-12
landed: 2026-08-12
landing-commit: dbd0aac
proposal-target:
  - process/milestone-4/phase-3/requirements/framing.md
  - process/milestone-4/plan.md
workflow-tier: tier 2 refining
initiator: owner
related:
  - process/procedures/workflow.md
  - process/cross-milestone/decisions/plan-revision-discipline.md
  - process/milestone-4/phase-3/requirements/framing.md
  - process/milestone-4/plan.md
  - process/candidate-pool.md
  - docs/dsl_spec.md
  - docs/notes/dsl-grammar.md
---

# M4 plan Revision 7 proposal — a seventh Phase 3 decision record for what a dot means

**State:** Landed on 2026-08-12 in `dbd0aac` after owner authorisation and the
agent critical check.

- **What / tier.** **Tier 2 refining.** Widen the Phase 3 reserved decision set
  from **DD-M4-P3-001〜006** to **001〜007**, where DD-007 settles what a dot
  means in an expression and which prefixes are admissible. Record the three
  questions DD-007 sends onward. AC9 wording, phase order, phase dependencies
  and the acceptance↔phase mapping are unchanged, and no other record's
  question moves.

- **Initiator.** Owner, 2026-08-12, in the Phase 3 pre-doc chat. The owner
  judged the current `root.` handling to be a defect rather than a design and
  asked whether a separate record should carry it. The agent proposed the
  record-set change and drafted DD-007.

- **Old premise.** The framing reserves six records and states that DD-001 is
  "共通の語彙・型・式位置を定める土台" — the foundation the other five consume.
  Its sub-issue list gives DD-001 name resolution, including the `root.`-
  qualified form. The premise is that the dot's meaning is a detail inside the
  expression-surface decision.

- **New evidence.** Measured against the release `wasamoc`:
  - A dotted prefix is **discarded, not validated**. `check_qualified_name`
    resolves the last segment and drops the rest, so `photos.count` and
    `a.b.c.count` both compile as a read of the state `count`, while
    `root.nope` fails with "undefined state `nope`".
  - The prefix is therefore **optional**: replacing every `root.count` in
    `examples/counter/counter.ui` with `count` passes `wasamoc check`.
  - **No document defines it.** `dsl_spec.md` never defines the identifier
    `root`; it appears only inside code examples, and in the spec's prose the
    word denotes the **content root widget** instead. The only recorded intent
    is a checker comment.
  - The two grammar statements disagree: §3 gives
    `qualified_name ::= IDENT ("." IDENT)*` while §2.4 says an interpolation
    placeholder takes one or two segments. The implementation follows §3.
  - The prefix is **absent from the IR**: `root.count` lowers to
    `(prop-read count)` (§8.9), so no representation, loader path or C ABI
    surface carries it.
  - The authored surface is 26 occurrences across four `.ui` files.

- **Why the old plan no longer holds — insufficient.** The reservation is not
  wrong; it is short by one question. DD-001 has to choose between a member-read
  interrogation (`photos.count`) and a method-call one (`photos.count()`), and
  the member-read spelling is **legal and already taken** under the unvalidated
  prefix. Deciding that choice inside DD-001 decides it against an accident
  rather than on merit, and freezes the outcome on a rationale that disappears
  the moment the prefix is validated. The question is also upstream of, and
  wider than, the expression surface: it is the phase's only change to what a
  currently legal `.ui` means, it reaches DD-006's assignment-target shape, and
  it bears on M4-Phase 7's host-state boundary spelling.

- **No-change option considered.** Keep six records and let DD-001 own the
  prefix question, or defer the whole matter to M4-Phase 7. Rejected on two
  grounds. First, review concern: DD-001's argument rests on every construct it
  adds being a parse error or a named diagnostic today, and the prefix change
  is the one thing in the phase that breaks that property — folding it in makes
  the record contradict itself. Second, ordering cost: both decisions are
  breaking, and settling DD-001 first means either living with a spelling
  chosen for a reason that no longer holds, or a second breaking change to
  correct it.

- **Critical check.** Agent-completed 2026-08-12, the owner being the initiator.
  - The measured claims above were each produced by running the compiler or
    reading the named source, not inferred from documents.
  - The strongest counter-argument is scope discipline: AC9 does not require
    this, the phase has already been revised six times, and DD-001's
    recommendation (S3) is safe under either outcome. It does not carry,
    because the cost of deferring is not "a spelling we might have preferred"
    but a 1.0-frozen spelling resting on a rationale this record removes.
  - The proposal does **not** claim the prefix question must be finished in
    Phase 3. DD-007 settles the rule and hands the retirement question forward;
    that split is what keeps the widening proportionate.
  - Limitation to note: the agent both drafted DD-007 and performed this check.
    The discipline assigns the check to the non-initiator side, which is
    satisfied, but an independent review of DD-007 before accept is the
    stronger reading of the same rule.

- **Owner authorisation.** Authorised 2026-08-12.

## Edits

The section-level record of what this revision touched. `proposal-target`
names the documents; this table names the sections.

| Target | Edit |
|---|---|
| framing §2.2 owner-agreed table, item ① | Reservation becomes **DD-M4-P3-001〜007**, with 007 named and the 状態 column recording this revision |
| framing §論点一覧 (`DD 番号の予約`) intro | Reservation restated as 001〜007; a dependency bullet states DD-007 is upstream of DD-001 and why |
| framing §論点一覧 | New `### DD-M4-P3-007` subsection — question, why it is separate, the measured starting state, and its sub-issues, in the same shape as the existing six |
| framing §論点の割り付け確認 | DD-007 row, with the lines that must not be mixed into it (the spelling is DD-001's; the host prefix is M4-Phase 7's) |
| framing §DD ごとの判定 (pre-ADR spike) | DD-007 row: **spike 不要** — the question is decided by measurement already performed and by normative judgement, not by an unknown mechanism |
| framing §DD と検証手段の対応 | DD-007 row: a firing reject case per rejected shape (non-member prefix, chained prefix, prefix where none is admitted, a `state` named after a prefix), the 26 existing `root.` occurrences as the positive control, no runtime path and no GUI evidence |
| framing §含まないもの | The prefix set's final form goes onward (table below) |
| framing §M4 acceptance criteria との対応 | AC9 row now reads 001〜007, noting DD-007 discharges no AC9 element directly |
| framing §AC9 の discharge matrix | Note under the table: DD-007 holds no row and why; its evidence sits in the verification table |
| framing §Revisions | One dated entry |
| `plan.md` Phase 3 progress row | "DD-M4-P3-001–006 reserved" → "001–007 reserved", with ADR status advanced to drafting |
| `plan.md` §Revision log | Revision 7 entry in the tier 2/3 template |

## Deferral-with-trigger table

DD-007 answers the rule and sends three questions onward. None is silently
dropped.

| Question sent onward | Owner | Activation trigger |
|---|---|---|
| The host-state prefix spelling itself | M4-Phase 7 | Its own host state boundary design, which cannot close without spelling host state somehow |
| The prefix set's final form — whether the expression-side set is emptied (retiring `root.`), and whether members carry a reserved symbol | M4-Phase 7, backed by a [candidate pool](../../../candidate-pool.md) row | M4-Phase 7's boundary decision; the pool row's per-planning disposition duty ([DD-V-028](../../../cross-milestone/decisions/pre-1.0-candidate-pool.md)) is the backstop, and the 1.0 compatibility commitment is the deadline |

The second row is the one that would otherwise lapse. Its trigger depends on a
**negative** outcome — if Phase 7 marks the host boundary on declarations rather
than with a prefix, `root` is left with no work to do, and a phase does not
document the prefix set it declined to use. Prose in the Phase 3 handoff alone
would leave nothing for anyone to act on; the pool row makes it an item a
planning pass has to dispose of.

## Impact check

- **Existing AC meanings** — unchanged. AC9's text is not touched and no AC ID
  is added, renumbered or superseded, so `process/_roadmap.md` needs no mirror.
- **Phase dependencies and order** — unchanged. DD-007 is upstream of DD-001
  inside Phase 3; no phase boundary moves.
- **Completed-phase evaluation** — unaffected. Phases 1 and 2 are closed and
  neither authored a dotted prefix beyond the examples counted above.
- **Retro / merge gates** — unchanged; the record set is larger by one, and the
  phase retrospective covers it like the rest.
- **Normative spec** — DD-007 moves `docs/dsl_spec.md` §2.4, §3, §4.16 and §5
  at implementation. `docs/abi_spec.md` does not move: the prefix never reaches
  the IR or the C ABI.
- **Authored `.ui`** — unchanged under DD-007's recommendation. `root` stays a
  valid prefix; the 26 occurrences continue to compile.
