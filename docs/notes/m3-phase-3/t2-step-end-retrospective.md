---
title: M3-Phase 3 / T2 step-end retrospective
status: recorded
created: 2026-05-21
scope: step-end
task: T2 — wasamoc check aspect-only-Box warning
---

# M3-Phase 3 / T2 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-3-progress.md` の **T2**
(`wasamoc check`: aspect-only-Box warning) の step-end retrospective。
T2 が discharge する DD は DD-M3-P3-004 Recommendation の companion
judgement (Checkpoint 2 ship-warning pick)。

対象コミット (2 件):

- `b942bce feat(wasamoc): warn on WrapPanel aspect-only-Box without item-cross-size`
- `51a8e5d docs(m3-phase-3): flip T2 checkboxes (aspect-only-Box warning)`

step-end の gate であり phase-end retrospective ではない。merge 先は
phase ブランチ `feat/m3-phase-3` (ff)。本 step (T2) は単一 task = 単一
step 構造で、現在のブランチは `feat/m3-phase-3-t2`。

## Current Judgment

2026-05-21 時点で T2 step-end 基準は **達成済み**。fast-track 判定は
**適用可** (checklist item 2–9 すべて「なし」/「不要」、item 3 green)。

- `check_wrappanel_aspect_only_box_warning` が WrapPanel 直下の Box
  with `aspect:` を検出し、`item-cross-size` 未設定時に warning
  1 件を WrapPanel 位置で発行する。dsl_spec §4.10 Common pitfalls
  への参照を text 内に含む。
- Guard scope は **direct child のみ**: nested container 内の
  aspect-only Box は scan しない (DD-M3-P3-004 "the warning does not
  classify all possible child shapes")。
- WrapPanel あたり warning 1 件 (matching Box が複数あっても重複しない)。
- 既存の `KNOWN_WIDGET_TYPES` chain (T1) に dispatch site を追加した
  だけで、Member walk の構造は無変更。
- 5 件の T2 unit test を `check.rs` に追加 (firing / positive control /
  nested non-firing / no-aspect / multi-matching-single-warning)。
  すべて green。
- **Clean rebuild gate:**
  `cargo clean` (3531 files removed, 977.5 MiB) → `cargo build --release
  --workspace` (38.27s, green) → `cargo build --workspace` (debug;
  35.13s, green) → `cargo test --workspace` (failure 0 件; wasamoc lib
  187 passed [T1 の 182 から +5]、他 crate も全 green) →
  `cargo fmt --all -- --check` (post-commit state; zero exit)。

T2 の blocker は残っていない。T3 (`wasamoc` lowering: AST → IR) へ進める。

## Main Learning

中心的な学びは **「T1 で築いた check_members_inner dispatch site が
T2 の warning 1 件追加だけで済むほど well-shaped だった」** という
確認。T1 の Follow-Up で予測していた「`check_members_inner` の
dispatch site と `enclosing_widget == Some("WrapPanel")` 判定を再利用」
が額面通り成立し、追加コードは `check_wrappanel_aspect_only_box_warning`
ヘルパ 1 つと WidgetDecl arm への呼び出し 1 行のみ。

副次的な学び:

- **「aspect-only」の判定基準は spec 文言 (`Box { aspect: <ratio>; …}`)
  を字句通り読んで「Box 子に `aspect` PropertyBind があれば match」**
  とした。ADR Option C 案にあった「no other size source」の精密分類は
  Phase 3 では vacuous (Box には width/height surface がまだない)
  なので採用していない。warning は false positive 寄りに振っている
  形だが、Phase 3 sub-screen の wireframe shape (`Box { aspect: 1:1
  fill: #cccccc }`) も明示的に同じ pitfall pattern に該当するため、
  DD-M3-P3-004 が想定する narrow guard の趣旨と一致。

- **「警告を出す単位は WrapPanel ごとに 1 件」** とした。matching Box
  が複数あっても重複しない方が author 体験として親切で、spec 文言
  「one or more `Box { aspect: <ratio>; … }` children」も WrapPanel
  単位の現象として記述している。

## Checklist

1. **本作業の主要な学び:** あり (記述項目)。
   - T1 で築いた `check_members_inner` dispatch site の再利用性が
     確認できた。warning 1 件の追加が helper 関数 1 つ + 呼び出し
     1 行で済んだ。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:** **なし**
   - dsl_spec §4.10 Common pitfalls は Moment 1 ドラフトで既に
     warning の存在を予告済み。実装側は spec 文言と一致しており、
     Moment 2 spec sync (T10) で追記する factual gap は生じていない。

3. **ローカル clean rebuild:** **green**
   - `cargo clean`: 3531 files removed, 977.5 MiB。
   - `cargo build --release --workspace`: green (38.27s)。
   - `cargo build --workspace` (debug): green (35.13s)。
   - `cargo test --workspace`: failure 0 件。
     - `wasamoc` lib test: **187 passed** (T1 の 182 から +5: T2 で
       追加した 5 件の warning test)。
     - `wasamo-runtime`: 200 passed。
     - `wasamo-ir`: 12 passed。
     - ABI / DLL / binding / counter-rust / gallery-rust crate 群も
       全 green。
   - `cargo fmt --all -- --check` (post-commit state): zero exit。

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - T2 の design call は DD-M3-P3-004 Recommendation companion で
     既に Accepted。実装側で新規に PO 裁定を求めた判断はない。
   - 「aspect-only」の判定を Box 子の `aspect` PropertyBind 存在で
     gate する読みは DD-M3-P3-004 Option C 「no other size source」
     から narrow に振った形 (Phase 3 では Box に width/height surface
     なし)、spec 文言 (`Box { aspect: <ratio>; … }`) とも一致する
     ため設計判断としては機械的。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - 追加コードは `check_wrappanel_aspect_only_box_warning` ヘルパと
     WidgetDecl arm への呼び出し 1 行、および 5 件の unit test のみ。
   - 既存コード (T1 で追加した dispatch site, KNOWN_WIDGET_TYPES,
     WRAPPANEL_INT_ATTRS など) はそのまま再利用。format / rename
     系の churn なし。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - DD-M3-P3-004 Recommendation の範囲内で完結。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格:** **なし**
   - 当該 ADR の DD はすべて既に Accepted。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:** **なし**
   - `unimplemented!` / `todo!()` stub なし。
   - 新規 `dead_code` 警告観測なし。

10. **タスクリストの後続 step 見直し:** **不要**
    - progress file の T2 行 3 項目をすべて `[x]` に更新済み。
    - T3–T10 の task 構成・順序・依存関係に T2 実装から見て調整す
      べき点は出ていない。
    - T3 (lowering) は warning ロジックに依存しないため、独立に
      進められる。

## Fast-Track Judgment

**Fast-track 適用可。** retrospectives.md §進行手順 / §ファストトラック
基準を満たす:

- item 2 (spec doc 変更): なし **(FT)**
- item 3 (local clean rebuild): green **(FT)**
- item 4 (PO 相談事項): なし **(FT)**
- item 5 (ついでのリファクタ): なし **(FT)**
- item 6 (追加 DD 必要性): なし **(FT)**
- item 7 (Proposed → Accepted 昇格): なし **(FT)**
- item 8 (plan AC / Phase 構成変更): なし **(FT)**
- item 9 (持ち越し): なし **(FT)**

全 (FT) 項目が「なし」/ green。本 retrospective を report と同時に
ff merge を実行し、事後にオーナーへ通知する形を取る。

## Verification Notes

T2 で追加したテストと、走らせた command を記録する。

新規テスト (check): 5 件

- `wrappanel_aspect_only_box_without_item_cross_size_warns` (firing)
- `wrappanel_aspect_only_box_with_item_cross_size_does_not_warn`
  (positive control — Phase 3 gallery sub-screen の shape)
- `wrappanel_aspect_only_box_nested_does_not_warn` (non-direct-child)
- `wrappanel_box_without_aspect_does_not_warn` (non-firing — Box
  に aspect なし)
- `wrappanel_multi_aspect_only_box_emits_single_warning` (重複しない)

実行コマンド:

```text
cargo clean                                (3531 files, 977.5 MiB)
cargo build --release --workspace          (38.27s, green)
cargo build --workspace                    (debug; 35.13s, green)
cargo test --workspace                     (failure 0)
cargo test -p wasamoc --lib check::tests   (76 passed)
cargo fmt --all -- --check                 (post-commit state; zero exit)
```

いずれも green。`wasamoc` lib test は **187 passed** (T1 の 182 から
+5)。

## Follow-Up

T2 から後続 task への明示的な引き渡し:

- **T3 (`wasamoc` lowering: AST → IR):** T2 の warning は check 層で
  完結しており、lowering 経路に影響しない。T3 着手時は既存の
  `Expr::IntLit` lowering と generic widget decl lowering を確認し、
  `WrapPanel` の 3 属性を `IrProp` として記録するだけで済む見込み。
- **T5 (`wasamo-runtime` widget catalog):** runtime 側の `WrapPanel`
  default は `item_cross_size: Option<i32>` で `None` が "passthrough"
  を意味する設計。T2 の warning は author に "None になる前に
  item-cross-size を設定せよ" と促す compile-time gate で、runtime
  側の Option 型表現と概念的に整合している。
- **T10 (Phase-end Moment 2 spec re-sync):** dsl_spec §4.10 Common
  pitfalls の warning 文言と実装側 diagnostic message に明示的な
  字句一致は要求していないが、Moment 2 で両者を読み比べて drift が
  ないか確認する。現状 implementation 側の text は spec 趣旨
  (aspect-only Box / item-cross-size 未設定 / 巨大 thumbnail footgun)
  を full に含んでいる。

これらはすべて progress file の T3–T10 として既に列挙済み。T2 単体で
新たに発見された follow-up は無し。
