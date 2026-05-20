---
title: M3-Phase 2 / T10 step-end retrospective
status: recorded
created: 2026-05-20
scope: step-end
task: T10 — IR text round-trip evidence (ADR §Phase 2 verification closure item 2)
---

# M3-Phase 2 / T10 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-2-progress.md` の **T10**
(ADR §Phase 2 verification closure item 2 — IR text round-trip
evidence) の step-end retrospective。T10 が discharge する材料は次:

- 共通 fixture
  `Box { aspect: 16:9; fill: #00000080; Text { text: "Photo 12" } }`
  を cross-crate 単位で結ぶ round-trip 駆動。これまで
  `wasamoc::emit::tests::box_phase2_ir_text_emit_fixture` (T5) と
  `wasamo-runtime::ir_loader::tests::box_phase2_load_side_fixture`
  (T7) が、それぞれ独立に reference 文字列を assert していた接合点を、
  `wasamo-runtime/tests/box_round_trip.rs` の
  `box_phase2_emit_parses_back_to_ir_literal_variants` で
  `wasamoc::emit::emit` の実出力越しに連結。
- Emit-side: 出力 IR が `IrLiteral::Ratio { 16, 9 }` /
  `IrLiteral::Color(0x80_00_00_00)` を `parse_ir` 後にも保つことを
  cross-crate に再確認。
- Load-side build_node materialisation: `build_widget_tree` を経由
  して、最終 `WidgetData::Box` が Box-internal 領域型 (`Ratio` /
  `Color`) を carry し、`IrLiteral::*` が runtime state に survive
  しないこと (DD-M3-P2-002 / DD-M3-P2-003 variant strategy Option A)
  を Windows-only integration test で観測可能にした。
- 2+ children reject (DD-M3-P2-001) の defense-in-depth gate を
  cross-crate file 上でも明示。

対象コミット:

- `8d12f66 feat(wasamo-runtime): IR text round-trip evidence for Box
  (M3-Phase 2 T10)`

これは step-end の gate であり、phase-end retrospective ではない。
merge 先は phase ブランチ `feat/m3-phase-2` (step→phase は ff)。

## Current Judgment

2026-05-20 時点で T10 step-end 基準は **達成済み**。

- 進捗 file T10 checklist 4 項目すべてに対し、新規 cross-crate
  integration test (`wasamo-runtime/tests/box_round_trip.rs`、3 件)
  と新規 test-only accessor (`WidgetNode::__box_state_for_test`) を
  紐づけた。
- `cargo fmt --all -- --check` (post-commit state, `8d12f66`):
  zero exit。
- `cargo clean` → `cargo build --release --workspace` →
  `cargo build --workspace` → `cargo test --workspace`: green。
- `cargo test --workspace --lib` の per-crate 件数は T9 終了時点と
  同一 (wasamo-ir 12 / wasamo-runtime 200 / wasamoc 153)。T10 は
  in-crate ユニットテストを追加していないので変化なし。
- Integration test 側で新規 3 件:
  `box_phase2_emit_parses_back_to_ir_literal_variants` (pure logic)、
  `box_phase2_two_children_rejected_at_parse_ir` (pure logic)、
  `box_phase2_build_node_materialises_box_internal_state`
  (`#[cfg(windows)]`、skip-guard あり)。
- 既存 in-crate fixtures
  (`box_phase2_ir_text_emit_fixture` / `box_phase2_load_side_fixture`)
  は不変。T10 はそれらの reference 文字列の cross-crate な接合点を
  追加で外側に張った。
- `WidgetData::Box.fill` の `#[allow(dead_code)]` (T6 で導入、
  T8 / T9 で継続) を本 step で解除。T9 retro item 9 で「T11 で
  外す」と planned だったものを、accessor の必要性が T10 に
  前倒しになった結果として一段早く解消した — `__box_state_for_test`
  が production の `WidgetNode` から `fill` を読むため、`dead_code`
  抑止が言わば自然に不要になる。

T10 の blocker は残っていない。

## Main Learning

中心的な学びは「**ADR の verification closure 項目に書かれている
構造的境界 (Option A 変種戦略のように "IR 側は IrLiteral、runtime
側は domain type" を分けている境界) は、両側で別々に in-crate
fixture を持つだけでは不十分で、cross-crate な driver で実出力を
連結させることでようやく executable な evidence になる**」。

T5 / T7 段階では、emit-side と parse-side それぞれの in-crate
fixture が「同じ reference 文字列に対して assert する」形で twin
fixture を構成していた。これは単独で読めば自然な test だが、
「reference 文字列同士が一致している」という静的事実だけでは、
**実装が emit から parse へ実際に出力を流したときに同じ意味を
保つ**ことを保証しない (e.g. emit と parse のどちらか一方が
reference 文字列に追従して同じ間違いをした場合に検出できない)。
T10 のように emit::emit の出力を parse_ir に流す cross-crate test
を入れて初めて、変種境界が executable になる。

副次的な学びとして、**build_node 半 (Windows-only Compositor 要求)
を T11 にまとめて延期する案より、T10 で先に最小限の
materialisation test を入れる方が境界が綺麗になる**こと。T7 で
`box_phase2_load_side_fixture` のコメントに「the build_node half
is T11」と書いてしまっていたが、

- ADR item 2 と ADR item 3 はそれぞれ独立の verification 対象
  (item 2 = IR text round-trip、item 3 = layout integration evidence)。
- 両方を T11 に混ぜると "IR materialisation を確認する test" と
  "layout 結果を確認する test" が同じ file に並ぶことになり、
  item 2 と item 3 のスコープが濁る。
- 共通 infrastructure (Compositor skip-guard、`__box_state_for_test`
  accessor、`build_widget_tree` 呼び出しヘルパ) は T10 と T11 で
  自然に共用できる。T10 でそれを置き、T11 が layout assertion を
  足す形にすれば、infrastructure コストを増やさず境界が保てる。

これは「**ADR 上で別 item に分かれている verification は、
infrastructure 共用を口実に同じ step に統合せず、item 単位で
別 step に landing する**」という運用判断の原則として記録できる。
具体的には今後の M3 phases で、同種の「item X は parse まで、item
Y は build まで」境界が出てきたときに、accessor / Compositor guard
helper の duplication を恐れて step を統合しない (≒ accessor は
T10 で landed、Compositor guard helper は file 内ローカル定義、
file 単位の重複は許容)。

T7 / T9 で「build_node half is T11」と書いた箇所は本 step
で T10 に正しく付け替え済み (`box_phase2_load_side_fixture` の
コメント、`widget.rs` T6 test-module コメント)。 同じ kind の
"将来 step で landed する" コメントを書くときは、actually どの
verification item を扱っているのかと、それが本当に次 step まで
持ち越されるべきものかを、commit 前に再点検する。

これらは spec / ADR の文面を新たに足す必要はなく、test file 内の
module-level doc-comment + progress doc T10 本文 + 本 retrospective
として記録した。後続 phase の verification closure step (M3-Phase
3 以降に similar T が立つ場合) は、**(a) ADR item 単位で test
file を切る、(b) infrastructure (accessor / skip-guard helper) は
item ごとに重複定義してよい、(c) "build_node half is T<N>" の
ような forward-pointer は実際に T<N> 着手時点で改めて妥当性確認
する** — の 3 点を踏襲してよい。

## Checklist

1. **本作業の主要な学び:** あり。
   - ADR の variant-strategy Option A 境界は in-crate twin fixture
     だけでは executable にならない、cross-crate driver で初めて
     evidence 化される (Main Learning に展開)。
   - ADR verification item 単位で test file を分け、共通
     infrastructure の重複は許容するほうが境界が保てる (Main
     Learning 副次)。
   - T7 で書いた "build_node half is T11" の forward-pointer は
     誤りで、T10 に正しく付け替えた。"将来 step で landed する"
     コメントは commit 前に再点検する習慣。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`)
   の変更:** **なし**
   - T10 は test infrastructure と accessor の追加。spec doc には
     触れない。
   - dsl_spec.md §4.9 の Phase status marker flip は T13
     (Moment 2 spec re-sync) の責任範囲。
   - `__box_state_for_test` は test-only な `#[doc(hidden)] pub fn`
     accessor で、`abi_spec.md` の C ABI surface には登場しない
     (DD-M3-P2-003 / DD-M3-P2-004 が `aspect` / `fill` の
     `PropertyValue` / `WASAMO_VALUE_*` への進出を明示的に拒んで
     おり、本 accessor もその制約に従って Rust-側 test-only
     surface に留まる)。

3. **ローカル clean rebuild:** **green**
   - `cargo fmt --all -- --check` (post-commit state `8d12f66`):
     zero exit。
   - `cargo clean` → `cargo build --release --workspace`: green。
   - `cargo build --workspace` (debug): green。
   - `cargo test --workspace`: failure 0 件。per-crate lib test
     count は T9 終了時点と同一 (wasamo-ir 12 / wasamo-runtime 200
     / wasamoc 153)、integration test に T10 で 3 件追加 (上記
     Current Judgment 参照)。
   - GitHub Actions 上の clean rebuild は phase-end gate (T13) で
     確認。

4. **PO に相談すべき設計判断・トレードオフ:** **あり (1 件)**
   - T10 着手時に「ADR item 2 の build_node half は T10 か T11
     か」という境界線が、T7 retro / T9 retro / T10 progress 上で
     不一致だったため、step 開始前にオーナーへ確認した。回答は
     「T10 で最小の Windows-only `build_node` materialisation test
     を追加し、T11 はその infrastructure を再利用」。本 step は
     その方針通り着地。回答の判断は本 retro Main Learning に
     原則として吸い上げ済み。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - 変更は (a) `wasamo-runtime/tests/box_round_trip.rs` 新規追加、
     (b) `WidgetNode::__box_state_for_test` accessor 追加、
     (c) `WidgetData::Box.fill` の `#[allow(dead_code)]` 解除、
     (d) `box_phase2_load_side_fixture` / `widget.rs` T6 テスト
     コメントの forward-pointer 修正のみ。すべて ADR §verification
     closure item 2 の scope 内。
   - `#[allow(dead_code)]` 解除は T9 retro item 9 で T11 work と
     planned されていたものを、本 step の accessor 追加に伴って
     自然に解消したもの。「ついで」ではなく、accessor 導入の必然的
     副作用。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - T10 で発見された設計論点はゼロ。ADR Accepted DD は不変。
   - 唯一の判断 (T10 vs T11 境界) はオーナー確認で吸収済みで、
     ADR 本文の DD ではない (Phase 2 verification closure section
     の運用解釈)。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed →
   Accepted への昇格:** **なし**
   - 当該 ADR は全 DD Accepted 済み。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**
   - A6 / A11 文言変更なし。Phase 構成変更なし。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **なし**
   - T6 で導入、T8 / T9 で継続していた `WidgetData::Box.fill` の
     `#[allow(dead_code)]` は本 step で解除 (T9 retro item 9 の
     見越し通り、ただし T11 ではなく T10 で解消)。
   - 新規に `#[allow(dead_code)]` や `unimplemented!` / `todo!` は
     導入していない。
   - `__box_state_for_test` は `#[doc(hidden)] pub fn` で実体ある
     accessor。dead_code 警告は出ない (新規 cross-crate test が
     呼び出している)。

10. **タスクリストの後続 step 見直し:** **なし (進行通り)**
    - T11 (Windows-runtime layout integration test) は T10 が
      入れた `__box_state_for_test` accessor と
      `wasamo_init`-skip-guard pattern を再利用できる構造で、
      checklist の文面変更は不要。`fill` の SpriteVisual brush
      peek は T11 でも accessor 越し (packed `u32`) で観測すれば
      足り、T11 ADR-side の checklist 文面 ("`fill` verified via
      a Box-internal / test-only accessor or via the render model")
      の前者を選ぶ形になる。
    - T12 (examples/gallery seed)、T13 (phase-end gates) は T10 から
      見て調整すべき点なし。

## Fast-Track Judgment

Fast-track criteria は **満たさない** (item 4 (FT) で T10 着手前に
オーナー相談を要した設計判断があったため):

- item 2 (spec doc 変更): なし
- item 3 (local clean rebuild): green
- item 4 (PO 相談事項): **あり** — T10 vs T11 の build_node half
  境界。step 開始前にオーナー確認済みで方針確定、ただし FT 判定上は
  "あり" 扱い。
- item 5 (ついでリファクタ): なし
- item 6 (追加 DD): なし
- item 7 (Proposed 増加/昇格): なし
- item 8 (m3-plan AC 変更): なし
- item 9 (仮実装・近似・新規 dead_code 警告): なし (むしろ既存
  `#[allow(dead_code)]` を解消)
- item 10 (タスクリスト見直し): なし

step→phase ブランチへの ff merge はオーナー明示確認後に実行する
(item 4 で "あり" のためファストトラック不適格)。

## Verification Notes

T10 で追加された test 件数の集計 (post-commit `8d12f66` 状態):

- `wasamo-runtime/tests/box_round_trip.rs` (新規 file): 3 件
  - `box_phase2_emit_parses_back_to_ir_literal_variants` (pure
    logic、cross-crate、emit → parse_ir join)
  - `box_phase2_two_children_rejected_at_parse_ir` (pure logic、
    DD-M3-P2-001 defense-in-depth 再確認)
  - `box_phase2_build_node_materialises_box_internal_state`
    (`#[cfg(windows)]`、`__box_state_for_test` accessor 経由で
    `WidgetData::Box` の Box-internal `aspect` / `fill` 観測)
- in-crate lib test count は T9 と同一 (新規 unit test なし)。

実行コマンド (post-commit `8d12f66` 状態):

```text
cargo fmt --all -- --check
cargo clean
cargo build --release --workspace
cargo build --workspace
cargo test --workspace
```

いずれも green。clean rebuild は本 retro 時点で完了 (release
profile / debug profile / 全 test suite の三段、いずれも failure
0 件)。GitHub Actions 上の clean rebuild は phase-end gate (T13)
で改めて確認する。

Post-review correction 後の HEAD `bee916a` でも、同じ step-end gate
(`cargo fmt --all -- --check` → `cargo clean` →
`cargo build --release --workspace` → `cargo build --workspace` →
`cargo test --workspace`) は green。

skip-guard の動作:

- ローカル Windows 開発機 (本 retro 実行環境): Compositor 利用可、
  `wasamo_init` が `WASAMO_OK` を返したため skip-guard は発火せず
  `box_phase2_build_node_materialises_box_internal_state` 本体を
  実行し pass。
- skip-guard の実発火は SSH dev box 等で `wasamo_init` が
  `0x80070005` を返す環境で T11 着地時に併せて確認する
  (CLAUDE.md `verification-environments.md` 準拠)。**現時点では
  guard の発火経路は実観測されていない**ことを Out-of-phase
  residual / Follow-Up に明示。

## Follow-Up

T10 から後続 task / phase への明示的な引き渡し:

- **T11 (Windows-runtime layout integration test, ADR §Phase 2
  verification closure item 3, CI-gated):** T10 が land した
  `WidgetNode::__box_state_for_test` accessor と `box_round_trip.rs`
  の skip-guard pattern を T11 で再利用する。T11 ADR-checklist の
  「`fill` verified via a Box-internal / test-only accessor」は
  accessor (packed `u32`) 経由で観測可能。新規 accessor を T11 で
  追加する必要はない見込み。
- **T11 / phase-end gate (T13) の skip-guard 実発火確認:** T10 の
  Windows-only test は本機 (Compositor 利用可) では skip-guard を
  発火させない経路でのみ pass しており、`0x80070005`-skip 経路は
  未観測。T11 で同 pattern を使う際、SSH dev box 等で実発火を確認
  する (Verification Notes 末尾の通り)。
- **T13 (Phase-end gates):** T10 から phase-end Out-of-phase scan
  に追加項目を出していない。T8 で記録した `WASAMO_ERR_*` 拡張の
  residual のみが scope 内。

これらは progress file の T11 / T13 に引き渡し済み。skip-guard
実発火確認は T11 の skip-guard checklist に明示し、T10 から新たに
発生した out-of-phase 項目は無い。

## Post-Review Corrections

T10 step-end review 後、phase-end retrospective で拾うべき記録整合性
修正を追加で行った:

- `0d601750cdb68dcf9aeac9ed636d7c8a60d0714d`
  `docs(m3-phase-2): align T10 follow-up records`
  - `ir_loader.rs` の stale な T11 forward-pointer を T10 に修正。
  - T11 progress checklist に `0x80070005` skip path 実発火確認を明示。
  - 本 retrospective の Follow-Up 締め文を progress file の引き渡し状態に
    合わせた。
- `027ba68 docs(process): record evidence-boundary open question`
  - phase progress file / step retrospective / phase acceptance evidence の
    責務境界が未定義であることを
    `docs/notes/process-rules-ssot.md` Q6 に open question として記録。
- `56608da docs(m3-phase-2): compact completed progress records`
  - `docs/plans/progress/m3-phase-2-progress.md` の T8–T10 完了済み
    checklist を discharge 要約に圧縮。
  - T9 の full test-name inventory と T10 の判断理由は、それぞれ
    step-end retrospective 側を詳細記録として扱う形に整理。

Phase-end retrospective では、T10 本体の ADR 適合だけでなく、
"phase progress file に execution log / retrospective detail /
acceptance evidence が混ざり始めた" 運用論点を
`process-rules-ssot.md` Q6 として再確認する。
