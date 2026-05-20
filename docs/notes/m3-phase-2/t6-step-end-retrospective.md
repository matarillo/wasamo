---
title: M3-Phase 2 / T6 step-end retrospective
status: recorded
created: 2026-05-20
scope: step-end
task: T6 — wasamo-runtime widget catalog Box
---

# M3-Phase 2 / T6 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-2-progress.md` の **T6**
(`wasamo-runtime` Box widget catalog) の step-end retrospective。
T6 が discharge する材料は次:

- DD-M3-P2-001 の IR-node-shape 半分 — per-kind tag `WidgetKind::Box`
  (Option A) と `WidgetData::Box { aspect, fill }` の data slot。
- DD-M3-P2-002 の Box-internal `Ratio` 半分 — private domain type
  `Ratio { num: i32, den: i32 }`、`PropertyValue` には乗らず
  `WASAMO_VALUE_*` tag も足さない (Option A variant strategy)。
- DD-M3-P2-003 の Box-internal `Color` 半分 — private domain type
  `Color(u32)` (packed `0xAARRGGBB`)、同様に `PropertyValue` / ABI
  には乗らない。

対象コミット:

- `b4dff5d feat(wasamo-runtime): add Box widget catalog (M3-Phase 2 T6)`

これは step-end の gate であり、phase-end retrospective ではない。
本 step (T6) は単一 step = 単一 task 構造で、merge 先は phase ブランチ
`feat/m3-phase-2` (step→phase は ff)。

## Current Judgment

2026-05-20 時点で T6 step-end 基準は **達成済み**。

- 新規 module `wasamo-runtime/src/box_widget.rs` に `Ratio` と
  `Color` を `pub(crate)` で declare。命名は ADR / progress file 通り
  (`Ratio`, `Color`) — `windows::UI::Color` と衝突を避けるために
  module を分け、widget.rs 側からは `box_widget::Ratio` /
  `box_widget::Color` で参照する。これにより既存の `Color { A, R, G, B }`
  (Windows API) との use 競合は発生しない。
- `WidgetData::Box { aspect: Option<box_widget::Ratio>, fill: Option<box_widget::Color> }`
  を追加。**`child: Option<Box<WidgetNode>>` field は採用していない**
  — 子は既存の `WidgetNode.children: Vec<Box<WidgetNode>>` に乗せ、
  single-child 不変性は `wasamoc check` (T3) と
  `ir_loader::build_node` (T7) の 2 gate で守る (DD-M3-P2-001 Option A
  の "defense in depth" そのまま)。data shape で 1 を強制しないことで
  既存 widget の構造 (children Vec) と対称になり、`for_each_ptr` /
  `dispose_subtree_bindings` / `sync_visuals` 等の subtree-traversal
  helper はすべて変更不要。progress file T6 の文言
  "`child: Option<Box<WidgetNode>>` (or layout-equivalent shape)" の
  後者を採用。
- `WidgetNode::box_(compositor)` constructor: `aspect: None`,
  `fill: None`, `width = SizeConstraint::Fill`, `height = SizeConstraint::Fill`
  (DD-M3-P2-005 で "no-aspect bounded Box: matches parent bounds
  when empty" を満たす最小デフォルト)。`SpriteVisual` は生成するが
  brush は付けない (fill の paint は T8 / T11 の責任)。
- `WidgetKind::Box` + `LayoutNode::box_(width, height)` を `layout.rs`
  に追加。`measure` arm は `measure_leaf` placeholder、`arrange` arm
  は no-op (自身の offset/size は arm に入る前に書かれる)。T8 が
  DD-M3-P2-005 inscribed-fit measure-arrange と DD-M3-P2-001 child
  centring / clip overflow で置き換える。
- `build_layout_tree` の `WidgetData::Box { .. }` arm が
  `LayoutNode::box_` を生成し、`WidgetNode.children` を再帰展開して
  layout tree 側にも乗せる。
- `aspect` / `fill` field は `#[allow(dead_code)]` を付けてコメントで
  T7 (writer) / T8 / T11 (reader) を forward-pointer する。
  `cargo build` の dead_code 警告は出ていない。
- 新規テスト 5 件:
  - `box_widget::tests::ratio_construction_and_equality` — `Ratio`
    の construction と PartialEq。
  - `box_widget::tests::color_packs_alpha_in_msb` — `0xFF_CC_CC_CC` /
    `0x80_00_00_00` の alpha bit 配置 (MSB)。
  - `box_widget::tests::color_equality_distinguishes_alpha` — 同じ
    RGB / 異なる alpha は != であることを固定 (packing 仕様の
    side-effect として alpha が値の一部であることを明示)。
  - `widget::tests::box_variant_carries_optional_aspect_and_fill` —
    `WidgetData::Box { aspect: Some(...), fill: Some(...) }` の
    pattern-match 抽出。`WidgetNode::box_` 自体は Compositor 必須なので
    variant の data shape だけを直接構築して assert する pure-logic
    test (T11 が `WidgetNode::box_` の実機 path をカバーする)。
  - `widget::tests::box_variant_defaults_both_fields_to_none` —
    constructor の default 初期化と同じ shape を assert。
- `cargo fmt --all -- --check` (post-commit state) zero exit。
- `cargo clean` → `cargo build --release --workspace` (release,
  41.20s) → `cargo build --workspace` (debug, 39.75s) →
  `cargo test --workspace` すべて green。
  - `wasamo-runtime`: 170 passed (T6 で +5、box_widget +3 / widget +2)。
  - `wasamo-ir`: 12 passed (変化なし)。
  - `wasamoc`: 153 passed (変化なし)。
  - 他 crate 変化なし。

T6 の blocker は残っていない。

## Main Learning

中心的な学びは「**Box-internal domain type の data slot は、writer (T7)
と reader (T8/T11) が同じ step に来なくても安全に先行できる**」という
ことの設計確認。T1 で `IrLiteral::Ratio` / `IrLiteral::Color` を IR
variant に足したときは、arm-exhaustiveness が即時 callers を全部
押さえる安全網になっていた (= match 漏れがあると build が落ちる)。
T6 の `WidgetData::Box { aspect, fill }` には同等の compile-time
安全網が **存在しない** — field の未使用は dead_code lint で検出
されうるが、今回はそれを `#[allow(dead_code)]` で **意図的に抑制した**
ので、現状の build では writer / reader 漏れを検出する仕組みは無い。
従って `#[allow(dead_code)]` は安全網ではなく、純粋に
**forward-pointer comment と組になった hand-off marker** として
機能する:

- `aspect` / `fill` field を読む production code は今 T6 時点では
  存在しない (T8 / T11 で入る)。書く production code も今は存在しない
  (T7 で入る)。Rust の `dead_code` lint は基本的に「読まれない field」
  を検出するもので、書かれない field は対象外なので、いずれにせよ
  writer 漏れの自動検出は無い。
- 後続 step の作業者は **コードコメントと progress file の T7 / T8 /
  T11 行を信頼して wiring を入れる必要がある** (lint は補助になら
  ない)。これは前 step (T5) の "意図の明示は実装と同じくらい重要" の
  Main Learning と同じ精神 — 安全網が無いところでは意図表明を強化
  するしかない。
- T7 / T8 / T11 が完了して production read / write が入った時点で
  `#[allow(dead_code)]` を外す (= 本来の wiring の一部であり、別途の
  ついでリファクタではない)。`#[allow]` を外したときに reader 側が
  まだ繋がっていなければそこで lint が再点灯する可能性がある — その
  範囲では allow を外す瞬間が遅延した check point になる。

副次的な学びとして、**OS API 型 (`windows::UI::Color`) と DSL 値型
(`box_widget::Color`) の同名衝突は module 分割で安価に解消できる**
ことの再確認。widget.rs では `Color` を `windows::UI::Color` として
import しており、ここに `box_widget::Color` を持ち込むと use 競合が
出る。`box_widget::Color` を module-qualified path で参照することで、
- ADR / progress file が指定する名前 (`Color`) をそのまま保てる
  (alias rename 不要)、
- widget.rs 内の既存 `Color { A, R, G, B }` 構築 site (button 系) を
  一切触らずに済む、
の二点を同時に成立させた。同じ pattern は今後 `wasamo-runtime` が
さらに OS-API-collision-prone な名前を domain 型として導入する
場合 (Vector, Rectangle, Brush, ...) に再利用できる。

ただし `box_widget` という module 名は現時点で違和感を残している —
中身は `Ratio` / `Color` の二つの value 型だけで、Box widget の
constructor / layout tree 化 / SpriteVisual 配線は `widget.rs` /
`layout.rs` 側にある。**module 名と責務の粒度が今は不一致**。今 T6
で rename しないのは、T7 で `ir_loader::build_node` が
`WidgetData::Box` の field に値を直接書く際に `pub(crate)` の setter
ないし builder が必要になる可能性があり、その時に `box_widget` の
責務が広がるか、それとも純粋な value 型 module のまま残るかが
決まるため。**T7 着手時に再評価**し、まだ違和感が残るなら
`box_values.rs` への rename が最小・最も clean な対応 (Follow-Up に
記録)。

T7 / T8 / T11 への持ち越し:

- **T7 (`ir_loader::build_node`):** `Box-internal Color` への
  materialise で `0xAARRGGBB` packing に揃える (spec §8.2)。`#RRGGBB`
  surface 入力時は alpha implicit `0xFF` を MSB に詰める。
  `IrLiteral::Ratio` / `IrLiteral::Color` を読み、`WidgetData::Box`
  の field に直接書き込む — `PropertyValue` 経由ではない (DD-M3-P2-002 /
  DD-M3-P2-003 Option A boundary)。emit canonical policy (短縮優先) と
  ir_loader accept policy (両形受理) が **非対称** であることを T5 retro
  が要点として残している。
- **T8 (layout):** `LayoutNode::box_` constructor は今 T6 の段階で
  width/height だけ受ける。T8 で aspect を threading する必要が出たら
  (a) `LayoutNode::box_` の signature を拡張するか (b) parallel
  measure-arrange entry point を切るか、を T8 内で判断する。今 T6 で
  signature を先回りで広げると T8 の判断余地を狭めるので、現状の
  minimal signature を維持。
- **T11 (Windows-runtime integration test):** `fill` の paint は
  `SpriteVisual` の brush で行うが、`WidgetNode::box_` constructor は
  brush を付けていない。T11 (または T8) が `WidgetNode` 内で
  fill → brush 反映を実装する。T11 進行時に `#[allow(dead_code)]` の
  forward-pointer comment を見直す。

## Checklist

1. **本作業の主要な学び:** あり。
   - Box-internal domain type の data slot を writer/reader と
     非同期に立てる場合、`#[allow(dead_code)]` + forward-pointer
     comment が意図された hand-off marker として機能する。
     後続 step (T7 / T8 / T11) が wiring を完了した時点で外す
     (= 本来の writer/reader 実装の一部)。
   - OS API 型と DSL 値型の同名衝突は module 分割で解消でき、ADR
     指定の名前をそのまま保てる (上記 Main Learning に展開)。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:**
   **なし**
   - T6 は `wasamo-runtime` の内部 catalog のみ。dsl_spec §4.9 /
     §8.2 / §8.11 / §8.13 の Box 関連条項はすべて T5 までで draft
     済み、本 step での再記述は不要 (Moment 2 spec re-sync は T13 の
     責任範囲)。`PropertyValue` / ABI 表面は変えていないので
     abi_spec も触らない。`architecture.md` への影響もなし。

3. **ローカル clean rebuild:** **green**
   - `cargo fmt --all -- --check` (post-commit state): zero exit。
   - `cargo clean` → `cargo build --release --workspace`: green
     (release, 41.20s)。
   - `cargo build --workspace`: green (debug, 39.75s)。
   - `cargo test --workspace`: failure 0 件。
     - `wasamo-runtime`: 170 passed (T6 で +5)。
     - `wasamo-ir`: 12 passed (変化なし)。
     - `wasamoc`: 153 passed (変化なし)。
     - 他 crate 変化なし。
   - GitHub Actions 上の clean rebuild は phase-end gate (T13) で
     確認。

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - T6 範囲はすべて DD-M3-P2-001 / DD-M3-P2-002 / DD-M3-P2-003 の
     Option A 採択から機械的に降りる。今回の細目決定 (data shape
     の `child: Option<Box<WidgetNode>>` ではなく既存
     `WidgetNode.children` 採用 / `width = height = Fill` デフォルト /
     module 分割) はすべて ADR の言葉ないし既存 widget convention に
     対応がついており、新たな設計判断を要しない。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - 変更は `wasamo-runtime` 内の 4 file (新規 `box_widget.rs`、
     修正 `lib.rs` / `layout.rs` / `widget.rs`) のみ。既存 widget の
     constructor / property dispatch / hit-testing / mutation API /
     binding writer / subtree teardown には一切触れていない。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - 既存 DD-M3-P2-001..006 で T6 範囲は完全にカバー。実装細目
     (子の格納位置、constructor の default 値、module 分割) は
     spec/DD レベルの判断ではない。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格:** **なし**
   - 当該 ADR は全 DD Accepted 済み。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**
   - A6 / A11 文言変更なし。Phase 構成変更なし。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **あり (T8 boundary placeholder)**
   - `cargo build` 出力に新規の dead_code 警告は出ていない
     (`#[allow(dead_code)]` で抑制済み)。
   - ただし `layout.rs` の `WidgetKind::Box` 用 arm は明示的な
     **仮実装** が残っている:
     - `measure`: `WidgetKind::Box => measure_leaf(node)`
       (DD-M3-P2-005 inscribed-fit ではなく leaf 扱い)。
     - `arrange`: `WidgetKind::Box => {}` (DD-M3-P2-001 の子 centring /
       clip overflow を行わず no-op)。
     - これらは progress file の T8 行で置き換えられる前提の
       boundary placeholder で、コードコメントで T8 / DD-M3-P2-005 /
       DD-M3-P2-001 への forward-pointer を入れている。
   - `unimplemented!` / `todo!` stub は置いていない (build / test は
     panic せずに通る)。`#[allow(dead_code)]` 自体は警告ではなく
     抑制だが、Main Learning の通り **安全網としては機能していない**
     ため、後続 step の作業者は progress file と forward-pointer
     comment に依存する必要がある。
   - 上記 placeholder は T8 完了時に消える設計であり、T8 のタスク
     範囲そのもの。T6 単独で「次 step 以降に解決すべき技術的負債」
     として持ち越しているわけではないが、retrospective rule の
     文言 "仮実装・近似" には該当するため **あり** で記録する。

10. **タスクリストの後続 step 見直し:** **なし (進行通り)**
    - T7–T13 の構成・順序・依存関係に T6 実装から見て調整すべき点は
      出ていない。
    - T7 / T8 / T11 への follow-up は下記 "Follow-Up" 節と Main
      Learning に明示。

## Fast-Track Judgment

Fast-track criteria は **満たさない** (item 9 の "仮実装" 該当):

- item 2 (spec doc 変更): なし
- item 3 (local clean rebuild): green
- item 4 (PO 相談事項): なし
- item 5 (ついでリファクタ): なし
- item 6 (追加 DD): なし
- item 7 (Proposed 増加/昇格): なし
- item 8 (m3-plan AC 変更): なし
- item 9 (仮実装・近似・新規 dead_code 警告): **あり** —
  `layout.rs` の `WidgetKind::Box` arm が leaf-like measure /
  no-op arrange の placeholder。T8 で置き換えられる boundary
  placeholder だが、retrospective rule の "仮実装・近似" に該当する。
- item 10 (タスクリスト見直し): なし

step→phase ブランチへの ff merge はオーナー明示確認後に実行する
(retrospectives.md §3 のファストトラック基準は item 2–8 (FT 印つき)
が全て「なし」を要求し、本 step は item 9 で "あり" のためファスト
トラック不適格。FT 範囲を 2–8 と読むか 2–9 と読むかに関わらず、
item 9 の素直な意味で本 step は対象外と判定)。

## Verification Notes

T6 で追加したテストと、走らせた command を記録する。

新規 `box_widget` テスト (`wasamo-runtime/src/box_widget.rs` 内
`#[cfg(test)] mod tests`):

- `ratio_construction_and_equality`
- `color_packs_alpha_in_msb`
- `color_equality_distinguishes_alpha`

新規 `widget` テスト (`wasamo-runtime/src/widget.rs` 内
`#[cfg(test)] mod tests`):

- `box_variant_carries_optional_aspect_and_fill`
- `box_variant_defaults_both_fields_to_none`

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

T6 から後続 task への明示的な引き渡し:

- **T7 (`wasamo-runtime` ir_loader):** `IrLiteral::Ratio` を
  `box_widget::Ratio` に、`IrLiteral::Color` を `box_widget::Color`
  に materialise する path を書く。`PropertyValue` を経由せず
  `WidgetData::Box` の field に直接書き込む (DD-M3-P2-002 /
  DD-M3-P2-003 Option A boundary)。emit canonical (短縮優先) と
  ir_loader accept (両形受理) の非対称性は T5 retro の Main Learning
  に既出。T7 で `WidgetData::Box` の field に値を書く手段として
  `pub(crate)` setter / builder が `box_widget` 側に必要になるかを
  実装時に判断し、必要なら同 module に置く。
- **T7 module 名再評価:** 今 T6 時点で `box_widget` という module
  名は中身 (純粋な `Ratio` / `Color` value 型のみ) に対して広すぎる。
  T7 で setter / builder が追加されて module が "Box 関連の crate-
  internal API 集" に育つなら現状名のまま正当化できる。逆に T7 でも
  純粋な value 型しか足さない場合は `box_values.rs` への rename が
  最小・最も clean な対応。T7 retro 時の判定項目とする。
  rename 自体はファイル名 + `lib.rs` の `mod` 行 + `widget.rs` の
  `use crate::box_widget` 行のみで完結する小変更。
- **T8 (`wasamo-runtime` layout):** `WidgetKind::Box` の
  `measure` / `arrange` placeholder を DD-M3-P2-005 inscribed-fit
  algorithm + DD-M3-P2-001 child centring / clip overflow に
  置き換える。`LayoutNode::box_` constructor の signature 拡張
  (aspect threading) か parallel entry point かを T8 内で判断する
  (T6 では先回りしない)。
- **T11 (Windows-runtime integration test):** `WidgetNode::box_` は
  `SpriteVisual` を生成するが brush は付けていない。fill → brush
  の反映は T11 (または T8) で。`#[allow(dead_code)]` の forward-pointer
  comment は T11 完了時に reader 側が外れることで自然解消する想定。

上記のうち T7 ir_loader 本体 / T8 layout / T11 brush 反映の 3 つは
progress file の T7 / T8 / T11 として既に列挙済み。**`box_widget`
module 名再評価のみ T6 で新規発見した judgement call** で、progress
file の T7 checklist 5 項目には含まれていない (= progress file 外の
review-time consideration として retro 側にのみ記録)。T7 着手者は
ir_loader 実装の流れで `box_widget` に setter / builder を増やすか
の判断と同時にこの再評価を扱う想定。
