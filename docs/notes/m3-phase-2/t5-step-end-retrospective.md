---
title: M3-Phase 2 / T5 step-end retrospective
status: recorded
created: 2026-05-20
scope: step-end
task: T5 — wasamoc IR text emit for Ratio / Color literals
---

# M3-Phase 2 / T5 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-2-progress.md` の **T5**
(`wasamoc` IR text emit) の step-end retrospective。T5 が discharge
する材料は次の二本:

- DD-M3-P2-002 の IR-text-spelling 半分 — `IrLiteral::Ratio { num, den }`
  → `<num>:<den>` (surface `RATIO_LIT` と同形)。
- DD-M3-P2-003 の IR-text-spelling 半分 — `IrLiteral::Color(u32)` →
  `#RRGGBB` または `#RRGGBBAA` (surface `COLOR_LIT` と同形)。

加えて T5 で **canonical emit policy** を確定:

- alpha = `0xFF` の `IrLiteral::Color` は短い `#RRGGBB` 形に
  normalise。
- alpha != `0xFF` は full 8-hex `#RRGGBBAA` で出す。
- `#RRGGBBFF` の surface 入力は byte-for-byte 保存されない
  (`#RRGGBB` に再正規化される) — これは意図的選択。

対象コミット:

- `935b5d0 feat(wasamoc): IR text emit for Ratio / Color literals (M3-Phase 2 T5)`

これは step-end の gate であり、phase-end retrospective ではない。
本 step (T5) は単一 step = 単一 task 構造で、merge 先は phase ブランチ
`feat/m3-phase-2` (step→phase は ff)。

## Current Judgment

2026-05-20 時点で T5 step-end 基準は **達成済み**。

- `emit_literal` の `IrLiteral::Ratio` / `IrLiteral::Color` arm は
  T1 で arm-exhaustiveness のために既に書かれていたが、T5 で
  `emit_color_lit` の doc-comment を「canonical emit policy」として
  明示。pre-T5 の comment は「surface form fixed by DD-M3-P2-003 /
  dsl_spec §8.2」と書いており、これは正確ではなかった — dsl_spec
  と DD は **surface accept** を fix しており **emit form** は別
  選択。新 comment はその区別と canonical 選択 (alpha = `0xFF` は
  短縮、それ以外は full) と、`#RRGGBBFF` が byte-for-byte 保存
  されないという policy の非対称性を明文化。
- 新規テスト 6 件 (`wasamoc::emit::tests`):
  - `box_aspect_ratio_emitted_in_surface_form` — `prop aspect = 16:9`
    が IR text に出ることを確認。
  - `box_fill_opaque_color_emitted_in_short_form` — `#cccccc` →
    `IrLiteral::Color(0xFF_CC_CC_CC)` → `#cccccc` (alpha-FF 短縮)。
  - `box_fill_color_with_alpha_emitted_in_full_form` — `#00000080`
    → `IrLiteral::Color(0x80_00_00_00)` → `#00000080` (full form)。
  - `color_emit_normalises_alpha_ff_input_to_short_form` —
    `#ffffffff` → `#ffffff` の非保存方向を明示テスト化。canonical
    policy 文言の "intentional" 性をテスト粒度で固定する。
  - `box_phase2_placeholder_widget_node_shape_emitted` — dsl_spec
    §4.9 normative placeholder shape
    (`Box { aspect: 16:9 fill: #cccccc Text { text: "Photo 12" } }`)
    の IR text が node / prop / child 全部含めて期待通り出る。
  - `box_phase2_ir_text_emit_fixture` — ADR §Phase 2 verification
    closure item 2 の emit-side gate。`#00000080` 版 fixture を
    `IrLiteral::Ratio { 16, 9 }` / `IrLiteral::Color(0x80_00_00_00)`
    の variant レベルと、emit 後の IR text レベルの両方で assert。
    load-side (T7 / T10) と対称になる "in-process roundtrip-shaped"
    test の本ファイル側担当分。
- `cargo fmt --all -- --check` (post-commit state) zero exit。
- `cargo clean` → `cargo build --release --workspace` (release,
  43.22s) → `cargo build --workspace` (debug, 37.35s) →
  `cargo test --workspace` すべて green。
  - `wasamoc`: 153 passed (T5 で +6、すべて `emit::tests`)。
  - `wasamo-ir`: 12 passed (変化なし)。
  - `wasamo-runtime`: 165 passed (変化なし)。
  - 他 crate 変化なし。

T5 の blocker は残っていない。

## Main Learning

中心的な学びは「**arm-exhaustiveness で書かれた既存実装と
canonical policy としての意図的選択は別概念**」ということ。T1 で
`IrLiteral::Ratio` / `IrLiteral::Color` を IR variant に追加したとき、
emit は match の arm coverage のために何か書かないとコンパイルが
通らない。当時の emit 実装は「とりあえず surface form と同じ形を
出す」を選んだが、これは defaulting (= 選択肢を比較せず惰性で
書いた形) であって canonical policy として書かれてはいなかった —
コメントは「surface form fixed by DD」と書いており、まるで spec が
emit form も決めているかのように読めた。

T4 retro が T5 への follow-up として明示した
「alpha = `0xFF` のとき `#RRGGBB` か `#RRGGBBAA` か」という設計
判断は、まさにこの "defaulting" を policy 化する役を担った。owner
との合意 (本会話: 短縮形を canonical とし、ADR の "alpha-yes" /
"surface forms #RRGGBB and #RRGGBBAA only" のどちらとも矛盾しない
読み) を pre-impl で取り、それを emit_color_lit の doc-comment と
`color_emit_normalises_alpha_ff_input_to_short_form` テスト名で
固定した。

これは "doc-driven development" (pre-doc → agreement → impl →
post-doc) の最小ループの綺麗な実演になった: agreement の段階で、
既存実装は変えなくてもよいことが分かり、変えるべきは「意図の
明示化」だった。**意図の明示は実装と同じくらい重要**で、書かれて
いないと defaulting と区別できないという観察。

副次的な学びとして、**three-stage verification の per-stage 責務**
が emit 側でも維持できた:

- check (T3): surface positional / value validity の reject。
- lower (T4): structural translation only (`IrLiteral::Ratio` /
  `IrLiteral::Color` への素直な変換)。
- emit (T5): IR text への spelling decision。**ここで初めて
  canonical form の選択が起きる** (= 選択は最後段にある)。
- ir_loader (T7): runtime 側 defense-in-depth。surface 両形を
  受理しつつ runtime state では区別しない。

この並びで、emit canonical policy は ir_loader の accept policy
とは独立であり (load 側は両形受理 / emit 側だけ正規化)、
**対称ではないことが意図**であることも明文化できた。T7 設計時に
この非対称を再確認する手がかりとして本 retro と
`emit_color_lit` の doc-comment が機能する想定。

T6 / T7 / T10 に持ち越した境界:

- **T6 (`wasamo-runtime` Box catalog):** 影響なし。Box-internal
  `Ratio` / `Color` 型は emit policy と独立。
- **T7 (`wasamo-runtime` ir_loader):** load 側は `#RRGGBB` /
  `#RRGGBBAA` の **両形** を受理し、Box-internal `Color` への
  materialise で `0xAARRGGBB` packing に揃える (alpha = `0xFF` を
  6-hex 入力でも 8-hex 入力でも同じ packed `u32` に。spec §8.2
  packing 規則そのまま)。emit canonical policy (= 短縮優先) と
  ir_loader accept policy (= 両形受理) が **非対称** であることを
  T7 実装時に再確認する。
- **T10 (IR text round-trip evidence):** emit→load の round-trip
  で `#RRGGBBFF` 入力が `#RRGGBB` 出力に正規化されることに留意。
  IR text の文字列一致を ground truth にする場合、入力 surface
  そのままではなく **emit normalisation 後の text** を ground
  truth にする。T10 fixture は `#00000080` (alpha != `0xFF`) で
  あるため、本件は T10 自身には影響しない (8-hex のまま round-
  trip)。`#ffffffff` 系のテストを T10 に追加する場合のみ留意。

## Checklist

1. **本作業の主要な学び:** あり。
   - arm-exhaustiveness で書かれた既存実装と canonical policy
     としての意図的選択は別概念。意図を明示しないと defaulting
     と区別できない (上記 Main Learning に展開)。
   - three-stage verification (check → lower → emit) の最後段で
     初めて canonical form の選択が起き、emit policy と
     ir_loader accept policy は **非対称が意図** であることを
     明文化。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:**
   **なし**
   - T5 の対象は `wasamoc::emit` のみ。dsl_spec §8.2 の `COLOR`
     packing 規則・surface accept ルールは既に確定済みで、
     emit canonical policy は **spec が定めない範囲** の実装側
     決定 (`emit_color_lit` の doc-comment に局所化)。
   - Moment 2 spec re-sync は T13 の責任範囲。

3. **ローカル clean rebuild:** **green**
   - `cargo fmt --all -- --check` (post-commit state): zero exit。
   - `cargo clean` → `cargo build --release --workspace`: green
     (release, 43.22s)。
   - `cargo build --workspace`: green (debug, 37.35s)。
   - `cargo test --workspace`: failure 0 件。
     - `wasamoc`: 153 passed (T5 で +6: 全て `emit::tests` Box /
       Color 系)。
     - `wasamo-ir`: 12 passed (変化なし)。
     - `wasamo-runtime`: 165 passed (変化なし)。
     - 他 crate 変化なし。
   - GitHub Actions 上の clean rebuild は phase-end gate (T13) で
     確認。

4. **PO に相談すべき設計判断・トレードオフ:** **あり (step 内で解決済み)**
   - T4 retro が T5 follow-up として明示した
     「alpha = `0xFF` のとき `#RRGGBB` か `#RRGGBBAA` か」の選択を
     pre-impl で owner と合意。owner 判断: 現行実装どおり
     `#RRGGBB` 短縮形を canonical emit form とする。ADR の
     "alpha-yes" (alpha チャンネル admit) と
     "surface forms #RRGGBB and #RRGGBBAA only" (両形 surface
     accept) のどちらとも矛盾しない読み。
   - 合意内容は `emit_color_lit` の doc-comment と
     `color_emit_normalises_alpha_ff_input_to_short_form` テストで
     固定。ADR 改訂や追加 DD は不要 (spec が定めない範囲の実装側
     決定)。
   - 本項目は "あり" であるため fast-track 不適格。step→phase
     merge はオーナー明示確認待ち。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - 変更は `wasamoc/src/emit.rs` の `emit_color_lit` の
     doc-comment 改訂 (5 行 → 14 行) と新規テスト 6 件のみ。
     emit ロジック本体 (arm body) は無変更。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - 既存 DD-M3-P2-001..006 で T5 範囲は完全にカバー。canonical
     emit policy は DD レベルの判断ではなく実装側の細目で、
     spec の "alpha-yes" / "両形 surface accept" のどちらとも
     矛盾しない (= DD が許す範囲内での実装選択)。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格:** **なし**
   - 当該 ADR は全 DD Accepted 済み。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**
   - A6 / A11 文言変更なし。Phase 構成変更なし。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **なし**
   - emit 側の policy が canonical 化されたので、`#RRGGBBFF` 系
     テストの "非保存方向" は明示済み。T7 / T10 で再確認する
     観点 (emit canonical / ir_loader accept の非対称) は Main
     Learning と Follow-Up に書き出し済みで、技術的負債ではない。
   - 新規 `dead_code` 警告: なし。

10. **タスクリストの後続 step 見直し:** **なし (進行通り)**
    - progress file の T5 行を `[x]` に更新し、本 retrospective への
      link を追加した。
    - T6–T13 の構成・順序・依存関係に T5 実装から見て調整すべき
      点は出ていない。
    - T7 / T10 への follow-up は下記 "Follow-Up" 節と Main Learning
      に明示。

## Fast-Track Judgment

Fast-track criteria は **満たさない** (item 4 に "あり")。

- item 2 (spec doc 変更): なし
- item 3 (local clean rebuild): green
- item 4 (PO 相談事項): **あり** — canonical emit policy 判断を
  step 内で owner に上げて決着。retrospective 内で明文化済みで
  あっても、`retrospectives.md` §3 のファストトラック基準は
  "checklist 項目 2–8 がすべて『なし』" を要求するため、本 step は
  fast-track 不適格。
- item 5–8: なし
- item 9: なし

step→phase ブランチへの ff merge はオーナー明示確認後に実行する。

## Verification Notes

T5 で追加したテストと、走らせた command を記録する。

新規 emit テスト (`wasamoc/src/emit.rs` 内 `#[cfg(test)] mod tests`):

Box ratio / color IR text emit:
- `box_aspect_ratio_emitted_in_surface_form`
- `box_fill_opaque_color_emitted_in_short_form`
- `box_fill_color_with_alpha_emitted_in_full_form`
- `color_emit_normalises_alpha_ff_input_to_short_form`
- `box_phase2_placeholder_widget_node_shape_emitted`
- `box_phase2_ir_text_emit_fixture`

実行コマンド:

```text
cargo fmt --all -- --check   (post-commit state)
cargo clean
cargo build --release --workspace
cargo build --workspace
cargo test --workspace
```

いずれも green。

## Follow-Up

T5 から後続 task への明示的な引き渡し:

- **T6 (`wasamo-runtime` Box catalog):** 影響なし。emit canonical
  policy は runtime 側の domain 型と独立。
- **T7 (`wasamo-runtime` ir_loader):** load 側は `#RRGGBB` /
  `#RRGGBBAA` の **両形** を受理する (defense-in-depth)。
  Box-internal `Color` への materialise で `0xAARRGGBB` packing
  に揃える (spec §8.2 packing 規則)。emit canonical policy
  (= 短縮優先) と ir_loader accept policy (= 両形受理) が
  **非対称** であることを T7 実装時に再確認する。本 retro の
  Main Learning が手がかり。
- **T10 (IR text round-trip evidence):** emit→load round-trip
  で `#RRGGBBFF` 入力が `#RRGGBB` 出力に正規化されることに留意。
  T10 fixture (`#00000080`, alpha != `0xFF`) は 8-hex のまま
  round-trip するため本件は T10 自身には影響しないが、`#ffffffff`
  系の round-trip テストを T10 に追加する場合は **emit
  normalisation 後の text** を ground truth にする。

これらはすべて progress file の T6–T10 として既に列挙済み。T5
単体で新たに発見された follow-up は無い。
