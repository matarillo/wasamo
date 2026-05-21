---
title: M3-Phase 3 / T4 step-end retrospective
status: recorded
created: 2026-05-21
scope: step-end
task: T4 — wasamoc IR text emit
---

# M3-Phase 3 / T4 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-3-progress.md` の **T4**
(`wasamoc` IR text emit) の step-end retrospective。T4 が discharge
する範囲は progress doc preamble の「Phase 3 introduces no new
emit grammar」を test として固定すること、および emit / re-parse
round-trip 安定性の確認。

対象コミット (2 件):

- `2eb7f7a test(wasamoc): cover WrapPanel IR text emit + cross-crate round-trip (M3-Phase 3 T4)`
- `a42cb3c docs(m3-phase-3): flip T4 checkboxes (wasamoc IR text emit)`

step-end の gate であり phase-end retrospective ではない。merge 先は
phase ブランチ `feat/m3-phase-3` (ff)。本 step (T4) は単一 task =
単一 step 構造で、現在のブランチは `feat/m3-phase-3-t4`。

## Current Judgment

2026-05-21 時点で T4 step-end 基準は **達成済み**。fast-track 判定は
**適用可** (checklist item 2–9 すべて「なし」/「不要」、item 3 green)。

- `emit.rs` の production コードに変更なし。`emit_node` は
  `widget_type` 文字列を保持する generic shape、`emit_prop` は
  `prop {name} = {literal}` を出力する generic shape、`emit_literal`
  の `IrLiteral::Int` arm は `n.to_string()` で decimal 整数を生成
  する既存経路で WrapPanel の 3 属性をそのまま吸収。「no new emit
  grammar」が成立。
- `wasamo-runtime/src/ir_loader.rs` の lexer (line 465-477) が
  identifier の非先頭位置で `-` を許容しているため、emit された
  `prop item-cross-size = 96` がそのまま IR loader を逆走できる。
  これも production コード変更なしで成立。
- 追加した unit test (emit.rs::tests) は 5 件:
  - `wrap_panel_zero_children_no_attributes_emitted`
  - `wrap_panel_all_three_attributes_emitted_as_decimal_ints`
  - `wrap_panel_only_item_cross_size_omits_other_attributes`
  - `wrap_panel_only_spacings_omits_item_cross_size`
  - `wrap_panel_zero_valued_attributes_emitted_as_zero_ints`
- 追加した round-trip test (`wasamo-runtime/tests/ir_loader_roundtrip.rs`)
  は 1 件: `wrap_panel_emit_then_parse_yields_equal_ir`。
  `wasamoc::emit::emit` → `wasamo_runtime::ir_loader::parse_ir` が
  `IrComponent` を完全に再構成することを `assert_eq!` で固定。
- 「absent attributes are omitted from the IR text」契約は T3 の
  lowering 段で IR から省略済みのため emit 側は自動的にこの契約を
  満たす。`only_item_cross_size_omits_other_attributes` / `only_
  spacings_omits_item_cross_size` でそれを emit 出力上でも明示。
- DD-M3-P3-006 zero-handling (`< 0` reject, `<= 0` ではない) を
  `zero_valued_attributes_emitted_as_zero_ints` でピン留め。
  emit 側でも「explicitly zero」と「absent」を区別すること
  (`prop ... = 0` が出力される) を test で保証。
- **Clean rebuild gate:**
  `cargo clean` (3621 files removed, 980.6 MiB) → `cargo build
  --release --workspace` (53.14s, green) → `cargo build --workspace`
  (debug; 44.45s, green) → `cargo test --workspace` (failure 0 件;
  wasamoc lib 198 passed [T3 の 193 から +5]、`ir_loader_roundtrip`
  6 passed [T3 の 5 から +1]、他 crate も全 green) →
  `cargo fmt --all -- --check` (post-commit state; zero exit)。

T4 の blocker は残っていない。T5 (`wasamo-runtime` widget catalog) へ
進める。

## Main Learning

中心的な学びは **「T4 は emit 経路と IR loader 経路の generic 性が
WrapPanel surface を吸収しきれることを test として固定する step
だった」** という確認。production コードを 1 行も足さずに済んだこと
自体が、Phase 3 framing — 「WrapPanel は generic な `IrNode` +
`IrProp` + `IrLiteral::Int` で表現できる」 — の verification であり、
T3 (lowering) と T4 (emit) はその同じ命題を入出力の両端から押さえる
対称な test 群を生む。

副次的な学び:

- **「emit / re-parse round-trip test は cross-crate seam を保護する
  唯一の場所」。** `wasamoc/src/emit.rs` の unit test だけでは
  「emit 出力が IR loader で読み取れる形になっているか」は検証で
  きない。kebab-case attribute 名 (`item-cross-size`) が IR loader
  の identifier lexer (line 465-477) で `-` を非先頭位置に許容する
  既存挙動に依存していることは、`wasamo-runtime/tests/ir_loader_
  roundtrip.rs::wrap_panel_emit_then_parse_yields_equal_ir` で
  cross-crate に固定して初めて regression として守られる。
  既存の counter / bool / string round-trip がカバーしていない
  「kebab-case prop name」サーフェスをここで追加。
- **「『emit 側の omission』は lowering 側の omission の自動的な
  帰結であり、emit 側で別途分岐を書く必要はない」。** `emit_node`
  は `node.props` を素直に iterate するだけで、IR に存在しない prop
  は出力に出ない。これは T3 で固定した「absent → IR omission」契約
  が emit 段に自然に伝播することを意味する。将来 emit 側に「default
  を逆算して書き出す」変更圧力 (例: human-readable form を狙って
  すべての省略可能 prop を明示的に書く) が来た場合、emit 出力の差分
  と round-trip 安定性のどちらが破綻するかを test で検知できる。
- **「`Box { aspect: 1:1 } Box { aspect: 1:1 }` を round-trip
  fixture に置けるのは T2 の warning surface 設計に依存している」。**
  T2 で確定した「aspect-only-Box warning は `has_errors()` を返さ
  ない」性質により、warning 発火 fixture も check pass → lower →
  emit → parse_ir まで通る。`wrap_panel_emit_then_parse_yields_
  equal_ir` の fixture は `item-cross-size: 96` を明示している
  ため warning は発火しないが、T3 の `Box` 子複数 fixture と同じ
  pattern を採用したことで test surface の連続性が取れている。

## Checklist

1. **本作業の主要な学び:** あり (記述項目)。
   - T4 が production コード変更ゼロで完了したこと自体が、Phase 3
     framing (no new emit grammar) の verification である。
   - emit / re-parse round-trip test は cross-crate seam (とくに
     kebab-case prop name) を守る唯一の場所。
   - emit 側の omission は lowering 側 omission の自動的帰結で、
     T3 → T4 の test surface は対称構造を取る。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:** **なし**
   - emit 側に新規 grammar / 新規 ABI 差分は発生していない。
     Moment 2 spec sync (T10) で記載すべき factual gap も生じていない。
     T1 の Decisions log に記録済みの「IntLit signed / FloatLit
     Measurement RatioLit unsigned」の §2/§5 表記決定は T10 マター。

3. **ローカル clean rebuild:** **green**
   - `cargo clean`: 3621 files removed, 980.6 MiB。
   - `cargo build --release --workspace`: green (53.14s)。
   - `cargo build --workspace` (debug): green (44.45s)。
   - `cargo test --workspace`: failure 0 件。
     - `wasamoc` lib test: **198 passed** (T3 の 193 から +5: T4 で
       追加した 5 件の emit test)。
     - `wasamo-runtime` integration `ir_loader_roundtrip`:
       **6 passed** (T3 の 5 から +1: `wrap_panel_emit_then_parse_
       yields_equal_ir`)。
     - `wasamo-runtime` lib: 200 passed。
     - `wasamo-ir`: 12 passed。
     - ABI / DLL / binding / counter-rust / gallery-rust crate 群も
       全 green。
   - `cargo fmt --all -- --check` (post-commit state): zero exit。

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - T4 で発生した design call はゼロ。progress doc preamble
     (no new emit grammar) が既に Accepted で、実装はそれに従う
     だけ。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - 追加したのは `emit.rs::tests` 末尾の T4 section (5 件の test)
     と `wasamo-runtime/tests/ir_loader_roundtrip.rs` の helper +
     test 1 件 (`build_wrap_panel_ir` / `wrap_panel_emit_then_parse_
     yields_equal_ir`) のみ。production コードに 1 行も触れていない。
     format / rename 系の churn なし。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - DD-M3-P3-001 / DD-M3-P3-003 / DD-M3-P3-004 / DD-M3-P3-006 の
     framing 範囲内で完結。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格:** **なし**
   - 当該 ADR の DD はすべて既に Accepted。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:** **なし**
   - `unimplemented!` / `todo!()` stub なし。
   - 新規 `dead_code` 警告観測なし。
   - production コードに変更がないため持ち越し技術的負債は構造的に
     発生しえない。

10. **タスクリストの後続 step 見直し:** **不要**
    - progress file の T4 行 3 項目をすべて `[x]` に更新済み。
    - T5–T10 の task 構成・順序・依存関係に T4 実装から見て調整す
      べき点は出ていない。
    - T5 (runtime widget catalog) は IR loader 経由で
      `WidgetData::WrapPanel` を materialise する step。T4 で
      cross-crate round-trip が IrComponent レベルで通ることを固定
      したので、T5 は「IrNode → WidgetData」変換層に集中できる。

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

T4 で追加したテストと、走らせた command を記録する。

新規テスト (emit, wasamoc): 5 件

- `wrap_panel_zero_children_no_attributes_emitted` (no-attr 出力)
- `wrap_panel_all_three_attributes_emitted_as_decimal_ints`
  (3 属性 + 複数 Box 子、decimal 整数形)
- `wrap_panel_only_item_cross_size_omits_other_attributes`
  (presence/absence の混在)
- `wrap_panel_only_spacings_omits_item_cross_size`
  (presence/absence の対称 case)
- `wrap_panel_zero_valued_attributes_emitted_as_zero_ints`
  (DD-M3-P3-006 zero は出力に明示)

新規テスト (ir_loader_roundtrip, wasamo-runtime): 1 件

- `wrap_panel_emit_then_parse_yields_equal_ir`
  (3 属性 + Box 子複数、cross-crate emit → parse_ir → IrComponent
  完全再構成)

実行コマンド:

```text
cargo clean                                (3621 files, 980.6 MiB)
cargo build --release --workspace          (53.14s, green)
cargo build --workspace                    (debug; 44.45s, green)
cargo test --workspace                     (failure 0)
cargo test -p wasamoc --lib emit::tests    (16 passed [+5 vs T3 ベース])
cargo test -p wasamo-runtime --test ir_loader_roundtrip
                                           (6 passed [+1 vs T3 ベース])
cargo fmt --all -- --check                 (post-commit state; zero exit)
```

いずれも green。`wasamoc` lib test は **198 passed** (T3 の 193 から
+5)、`ir_loader_roundtrip` は **6 passed** (T3 の 5 から +1)。

## Follow-Up

T4 から後続 task への明示的な引き渡し:

- **T5 (`wasamo-runtime` widget catalog):** T4 で IrComponent
  レベルの round-trip 安定性が固定されたので、T5 は IrNode →
  `WidgetData::WrapPanel { item_cross_size: Option<i32>,
  item_spacing: i32, line_spacing: i32 }` の materialise dispatch
  に集中できる。`props.iter().find(|p| p.name == "item-spacing")`
  パターン (T3 follow-up で予告済み) を `wasamo-runtime/src/
  ir_loader.rs::build_widget_tree` の WrapPanel arm として実装。
  defaults: `item_cross_size: None`, `item_spacing: 0`,
  `line_spacing: 0` (DD-M3-P3-003 / DD-M3-P3-004)。
- **T6 (`wasamo-runtime` IR loader + `validate()` defense-in-depth):**
  T4 round-trip test は IR text 経由で valid な IrComponent を
  作る経路を pin しているが、`wasamo_load_ui` の memory-IR path
  は wasamoc を経由しない。T6 で `validate()` に「negative
  `item_cross_size` / `item_spacing` / `line_spacing` を
  `WASAMO_ERR_IR_MALFORMED` で reject」する arm を追加する。
- **T10 (Phase-end Moment 2 spec re-sync):** T1 の Decisions log
  に記録済みの「IntLit signed / FloatLit Measurement RatioLit
  unsigned」が dsl_spec §2 / §5 に一言の確認を要するかどうか
  (T1 Decisions log 末尾参照) は依然として T10 で判断する事項。
  T4 では新たな factual gap は生じていない。

これらはすべて progress file の T5–T10 として既に列挙済み。T4 単体で
新たに発見された follow-up は無し。
