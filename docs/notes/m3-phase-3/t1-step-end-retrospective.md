---
title: M3-Phase 3 / T1 step-end retrospective
status: recorded
created: 2026-05-21
scope: step-end
task: T1 — wasamoc check WrapPanel known-widget + reject set
---

# M3-Phase 3 / T1 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-3-progress.md` の **T1**
("`wasamoc check`: WrapPanel validity and reject set") の step-end
retrospective。T1 が discharge する DD は DD-M3-P3-001 (child count +
surface registration)、DD-M3-P3-006 (compile-time half of two-gate
defense for non-negative integers)、および DD-M3-P3-003 / DD-M3-P3-004
の constant-only halves。

対象コミット (3 件):

- `a3dba9a feat(wasamoc): generalize kebab-case ident, admit negative integer literals`
- `b92a876 feat(wasamoc): WrapPanel known-widget registration and attribute reject set`
- `f505459 docs(m3-phase-3): flip T1 checkboxes and record lexer prerequisite`

step-end の gate であり phase-end retrospective ではない。merge 先は
phase ブランチ `feat/m3-phase-3` (ff)。本 step (T1) は単一 task = 単一
step 構造で、現在のブランチは `feat/m3-phase-3-t1`。

## Current Judgment

2026-05-21 時点で T1 step-end 基準は **達成済み**。

- WrapPanel が `KNOWN_WIDGET_TYPES` に追加され、unknown-widget warning
  を出さない (DD-M3-P3-001 surface registration)。
- `widget_prop_type` catalog に `(WrapPanel, item-cross-size) /
  (item-spacing) / (line-spacing) = TypeName::Int` の 3 行を追加。既存の
  `TypeName::Int` を再利用し、新 `TypeName` variant は導入していない。
- `check_wrappanel_const_only_bind` で `IntLit` shape 強制と
  `value >= 0` 不変条件の両方を gate。state-backed Ident, Ratio /
  Color / String / Bool literal, Measurement (`px`) のいずれも
  diagnostic に attribute 名を埋めて reject。Zero は valid
  (`item-cross-size: 0 / item-spacing: 0 / line-spacing: 0` を
  accept) — dsl_spec §4.10 の zero-handling と一致。
- `check_wrappanel_attr_outside_wrappanel` で attribute-position
  rejection を実装。`Box { item-cross-size: 88 }` /
  `VStack { item-spacing: 12 }` / component-level `line-spacing: 12`
  はすべて attribute 名と offending position 名で reject。
- 0-child / 1-child / multi-child の 3 形すべて accept (DD-M3-P3-001
  の no upper bound)。
- 23 件の T1 unit test を `check.rs` に追加。すべて green。
- 追加 lexer test 6 件 (kebab-case ident generalization + negative
  IntLit) も green。`in_out_followed_by_alphanumeric_is_error` は
  動作変化に伴い `in_outx_lexes_as_kebab_ident` へ書き換え。
- `cargo fmt --all -- --check` (post-commit state)、`cargo build
  --workspace`、`cargo test --workspace` どれも local で green
  (180 wasamoc + 12 wasamo-ir + 他 crate の test result が全 ok)。

T1 の blocker は残っていない。T2 (aspect-only-Box warning) へ進める。

## Main Learning

中心的な学びは **「ADR/progress doc が "no new parser grammar" と
書いていても、kebab-case 属性名や負整数リテラルなど lexical surface
の変更を要求する spec はある」** という発見。実装着手時に `item-
cross-size: 88` を tokenize したら `item` Ident → `-` "unexpected `-`"
で停止することを確認し、owner 確認のうえ lexer を以下 2 点拡張した:

1. `scan_ident` を kebab-aware にし、`in-out` を一般 ident lexing +
   keyword 判定経路に統一。
2. 先頭 `-` の後ろが ASCII 数字なら scan_number 経由で負の IntLit を
   発行。binary subtraction が DSL grammar に無いため leading-sign
   読みは曖昧ではない。

この発見は **「pre-doc framing がカバーしていない実装前提が T1 着手の
最初の commit で surface する」** パターン。Phase 1 / Phase 2 の T1
retro が「設計が既に閉じている mechanical 反映」だったのと対照的に、
Phase 3 T1 は最小の lexical-surface 改変を要した。これは progress
doc の「Phase 3 introduces no new parser grammar」prose を
**factually 正確に保ちつつ** lexer extension を Decisions log に明文
記録した形で吸収している。

副次的な学び:

- `WrapPanel.<attribute>` 系を `Box.aspect` / `Box.fill` と同じ
  「constant-only literal pattern, dedicated checker」discipline で
  追加できた。`widget_prop_type` catalog 行は Phase 3 では type-mismatch
  table に到達しない (constant-only gate が先に走る) が、future
  bindable phase で catalog 行が即座に再利用できるよう残した。これは
  Phase 2 の `Box.aspect: Ratio` / `Box.fill: Color` 行が catalog に
  存在しないのと方針が異なるため、check.rs 内コメントで意図を明記。

- 3 commit 構成 (lexer / check / docs) は Phase 1 / Phase 2 の単一
  task = 単一 commit を逸脱しているが、CLAUDE.md §Commit rules の
  例外条件「Implementation reveals that an item should be split」に
  該当 (lexer 拡張は T1 着手後に発覚した前提)。progress doc の
  Decisions log にその経緯を残した。

## Checklist

1. **本作業の主要な学び:** あり。
   - Phase 3 の dsl_spec §4.10 が要求する kebab-case 属性名と
     `: -1` の negative-literal reachability は lexer 拡張が前提。
     pre-doc framing がこれを明示していなかった。
   - WrapPanel 属性は `Box.aspect/fill` と同じ constant-only literal
     pattern + dedicated checker で実装できる。catalog 行は Phase 3
     では unreachable だが future-proof のため残す方針を選択。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:** **なし**
   - T1 の対象は `wasamoc::lexer` / `wasamoc::check` および
     `docs/plans/progress/m3-phase-3-progress.md` のみ。spec sync
     (Moment 2) は T10 の責任範囲。dsl_spec §4.10 は既に Moment 1
     ドラフト済みで T1 から factual 矛盾は出ていない。lexer 拡張に
     対する §2 (lexical surface) サイドの記述要否は T10 で
     再判断する旨を Decisions log に記録した。

3. **ローカル clean rebuild:** **green**
   - `cargo fmt --all -- --check` (post-commit state): zero exit。
   - `cargo build --workspace`: green (35.99s)。
   - `cargo test --workspace`: failure 0 件。
     - `wasamoc` lib test: 180 passed (T1 で +29: lexer +5,
       check +23, それと既存 `in_out_followed_by_alphanumeric_is_error`
       の置換 1)。
     - `wasamo-ir`, `wasamo-runtime`, ABI / DLL / binding crate 群も
       全部 green。
   - GitHub Actions 上の clean rebuild は phase-end gate (T10) で確認。

4. **PO に相談すべき設計判断・トレードオフ:** **あり (解消済み)**
   - T1 着手時に **kebab-case lex の方針** を 2026-05-21 に owner と
     合意 (lexer 拡張、`in-out` 特例の一般化、negative IntLit lexer
     到達)。spec 側変更でなく lexer 拡張で吸収する方針が確定済み。
   - 新規発生の design call は無し。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし (ただし lexer 拡張あり)**
   - `wasamoc::lexer` の kebab + negative IntLit 拡張は **T1 を実施
     するための前提**であり「ついで」ではない。owner 合意のうえ
     先頭 commit として切り出した。lexer / check / docs の 3 commit
     構成は CLAUDE.md §Commit rules の split 許容条件を満たす。
   - 既存 test `in_out_followed_by_alphanumeric_is_error` を
     `in_outx_lexes_as_kebab_ident` に置換 (lexer behavior 変化に
     直接対応する書き換え; format / scope creep 系の churn ではない)。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - lexer 拡張は ADR DD-001..006 のいずれの design call にも
     抵触しない (DD は IR shape / 属性意味論 / measure-arrange /
     two-gate defense をカバーしており、lexical surface は ADR 範囲外)。
   - Decisions log に T1 着手時の経緯と spec sync 側の処理予定を
     記録済み。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格:** **なし**
   - 当該 ADR (`docs/decisions/m3-phase-3-wrap-panel.md`) は
     全 DD Accepted 済み。T1 では昇格対象なし。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**
   - A3 / A11 の文言は変更なし。Phase 構成変更なし。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **なし**
   - `unimplemented!` / `todo!()` stub は一切置いていない。
   - 新規 `dead_code` 警告は観測していない (cargo build / test の
     warning 出力に新規ノイズ無し)。
   - WrapPanel-side catalog 行は Phase 3 では unreachable だが
     `dead_code` 警告は出ていない (match arm 内で参照されるため)。

10. **タスクリストの後続 step 見直し:** **不要**
    - progress file の T1 行 8 項目をすべて `[x]` に更新済み
      (lexer prerequisite 行を新規追加して合計 8 行に増えた)。
    - T2–T10 の task 構成・順序・依存関係に T1 実装から見て調整す
      べき点は出ていない。
    - T2 (aspect-only-Box warning) は T1 で用意した
      `check_members_inner` の dispatch site を再利用するだけで
      足りる見込み。

## Fast-Track Judgment

Fast-track criteria を満たしている。

- item 2 (spec doc 変更): なし
- item 3 (local clean rebuild): green
- item 4 (PO 相談事項): あり / **合意済み** (kebab lexer 方針を
  2026-05-21 に確定)
- item 5 (ついでのリファクタ): なし (lexer 拡張は T1 前提)
- item 6 (追加 DD 必要性): なし
- item 7 (Proposed → Accepted 昇格): なし
- item 8 (plan AC / Phase 構成変更): なし
- item 9 (持ち越し): なし

item 4 が「あり」だったが本 step 内で owner 合意済みのため fast-track
を阻害しない。とはいえ step → phase ブランチへの ff merge 判断は
オーナーに通知のうえで進める (retrospectives.md §進行手順 3 に従い、
本 retrospective を fast-track 報告として提示)。

## Verification Notes

T1 で追加したテストと、走らせた command を記録する。

新規テスト (lexer): 5 件 + 既存 1 件の置換

- `kebab_case_ident`
- `kebab_ident_breaks_on_non_alpha_after_hyphen`
- `negative_int_literal`
- `negative_int_in_property_bind_position`
- `in_outx_lexes_as_kebab_ident` (旧 `in_out_followed_by_alphanumeric_is_error`)

新規テスト (check): 23 件

- accept 側 (10): `wrappanel_known_widget_no_warning`,
  `wrappanel_zero_child_accepted`, `wrappanel_one_child_accepted`,
  `wrappanel_multi_child_accepted`,
  `wrappanel_with_item_cross_size_accepted`,
  `wrappanel_with_item_spacing_accepted`,
  `wrappanel_with_line_spacing_accepted`,
  `wrappanel_zero_values_accepted`,
  `wrappanel_full_accept_shape`
- 負値 reject (3): `wrappanel_negative_item_cross_size_rejected`,
  `wrappanel_negative_item_spacing_rejected`,
  `wrappanel_negative_line_spacing_rejected`
- bind reject (3): `wrappanel_item_cross_size_state_ident_rejected`,
  `wrappanel_item_spacing_state_ident_rejected`,
  `wrappanel_line_spacing_state_ident_rejected`
- 非 IntLit RHS reject (5):
  `wrappanel_item_cross_size_ratio_literal_rejected`,
  `wrappanel_item_cross_size_string_literal_rejected`,
  `wrappanel_item_cross_size_bool_literal_rejected`,
  `wrappanel_item_cross_size_color_literal_rejected`,
  `wrappanel_item_spacing_measurement_rejected`
- 位置 reject (3): `wrappanel_attr_on_box_rejected`,
  `wrappanel_attr_on_vstack_rejected`,
  `wrappanel_attr_at_component_level_rejected`

実行コマンド:

```text
cargo fmt --all -- --check   (post-commit state)
cargo build --workspace
cargo test --workspace
cargo test -p wasamoc --lib lexer
cargo test -p wasamoc --lib check
```

いずれも green。`wasamoc` lib test は 180 passed (T1 で +29 net)。

## Follow-Up

T1 から後続 task への明示的な引き渡し:

- **T2 (`wasamoc check`: aspect-only-Box warning):** WrapPanel の
  直下子に `Box { aspect: <ratio>; … }` があり `item-cross-size` 未
  設定なら warning。T1 で追加した
  `check_members_inner` の dispatch site と
  `enclosing_widget == Some("WrapPanel")` 判定をそのまま利用できる。
- **T3 (`wasamoc` lowering):** `Expr::IntLit` (正負どちらも) を
  `IrLiteral::Int` に lower。T1 の lexer 変更で負値 IntLit が
  自然に IR まで届く。
- **T6 (`wasamo-runtime` validate() runtime gate):** memory-IR で
  bypass される compile-time gate と対になる runtime 半分。T1 は
  compile-time gate のみを discharge。
- **Phase 3 closing Moment 2 (T10) spec re-sync 時の判断材料:**
  - dsl_spec §2 (lexical surface) に kebab ident + negative IntLit
    の記載が必要かどうかを再判断する (現状 §2 は ident に明示的な
    surface 文法を載せていないので、§4.10 attribute table の存在で
    充分とも判定しうる)。
  - lexer test の `in-outx` 動作変化を §2 / §4.7 に反映するか。

これらはすべて progress file の T2–T10 として既に列挙済み。T1 単体で
新たに発見された follow-up は **lexer prerequisite** の Decisions log
反映以外にない。
