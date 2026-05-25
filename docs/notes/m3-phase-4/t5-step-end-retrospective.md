---
title: M3-Phase 4 / T5 step-end retrospective
status: recorded
created: 2026-05-25
scope: step-end
task: T5 — End-to-end gallery visible smoke
---

# M3-Phase 4 / T5 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-4-progress.md` の **T5**
("End-to-end gallery visible smoke") の step-end retrospective。
T5 が discharge する ADR 検証は Phase 4 verification closure
**evidence item 5** (gallery sub-screen の `.ui` 追加 + アシスタント側
build / launch / `Start-Process` success) と phase-close / A11 gallery
proof のうち **アシスタント自動化部分**。残りの
visible-correctness 部分 (viewport clip 鋭さ、Button-driven content
motion、clipped 領域の非表示、off-viewport thumbnail が scroll で
viewport に進入する) はオーナー手動 GUI smoke として T6 phase-end
gate (`retrospectives.md` checklist item 17 / [human-visible GUI
smoke](../human-visible-smoke.md)) に持ち越す。これは ADR 規定通り。

T5 は runtime / IR loader / wasamoc に新規コードを追加しない step。
T4 で intermediate Visual + `Visual.Offset = (0, -applied_y, 0)`
shift + `set_property` ScrollView arm の string-to-`i32` parse +
`update_scroll_view_offset_y` writer + `applied_offset_y` cache + IR
loader binding wiring がすべて wire 済み (T4 retrospective Main
Learning)。T5 は **wire 済み chain を実際の `.ui` に exercise させ、
gallery-rust host から process が立ち上がることを観測する** だけの
step。

対象実装コミット (`feat/m3-phase-4-t5` 上、本 retrospective ファイル
自体を除く 1 件。progress checkbox flip は独立 commit を切らず実装
commit に fold):

- `256dbc0 feat(examples): gallery ScrollView slice + scroll-y buttons (M3-Phase 4 T5)`

merge 先は phase ブランチ `feat/m3-phase-4` (no-ff、`feedback_workflow`
§1 / `retrospectives.md` §進行手順)。phase → main は T6 の phase-end
gate に持ち越す。

## Current Judgment

2026-05-25 時点で T5 step-end 基準は **達成済み (owner 明示承認待ち)**。
fast-track は廃止 (`feedback_workflow` §2(b) / 2026-05-25 `49b49fb`)
のため、判定にかかわらず owner 明示承認待ちで停止する。

- **`examples/gallery/gallery.ui` の additive 拡張:**
  - `state scroll_y: i32 = 0` を component-level に declare
    (dsl_spec §4.7 / §4.11 Attributes の bare-identifier RHS pattern)。
  - 既存の Phase 3 standalone WrapPanel slice (Box × 10、
    `item-cross-size: 88`) は文字どおり untouched で残置 (10 行ぶん
    1:1 文字列保存)。
  - 新規 root container を `VStack { spacing: 12px; padding: 12px }`
    に置き、3 つの sibling 要素を持つ:
    (1) 既存 WrapPanel (untouched) ;
    (2) Button "Scroll down (+100)" (style: accent) +
        Button "Scroll up (-100)" (default style)。
        各 Button の `clicked` handler は
        `root.scroll_y += 100;` / `root.scroll_y -= 100;` ;
    (3) `ScrollView { offset-y: scroll_y; WrapPanel {
        item-cross-size: 64; item-spacing: 8; line-spacing: 8;
        Box × 32 } }` slice。32 thumbnail は ADR の "Box × 30–40"
        範囲内、`item-cross-size: 64` で WrapPanel content_h を
        viewport remainder より十分大きく確保。
  - `offset-y: scroll_y` は dsl_spec §4.11 Attributes の bare state
    identifier RHS。`\{…}` interpolation でも `bind` / `in-out`
    keyword でもなく、T1 wasamoc check で受理される唯一の bound 形。
- **Button → ScrollView の writer chain は T4 wired 経路を再利用:**
  - Button `clicked` handler → handler runtime → `root.scroll_y +=
    100` の Signal::set → effect re-fire → ScrollView の bound
    `offset-y` property に `widget_write_property` → ScrollView arm
    の string-to-`i32` parse (DD-M3-P4-003 narrow bridge) →
    `update_scroll_view_offset_y(new_value)` → mark_layout_dirty →
    next `run_layout` で `arrange_scroll_view` が clamp →
    `applied_offset_y.set(...)` → `sync_visuals` ScrollView arm が
    intermediate Visual に `Visual.Offset = (0, -applied_y, 0)` を
    書く + children の `parent_abs_offset` を `(x, y -
    applied_y)` に shift。
  - T5 では runtime / IR loader / wasamoc いずれにも 1 文字も触らない。
- **HStack vs ScrollView Fill 競合の回避 (実装上の選択):**
  - Button row を `HStack { ... }` で水平に並べる方が見栄えするが、
    HStack は DD-M3-P3-002 Option A 由来で `height: SizeConstraint::
    Fill` がデフォルト。ScrollView も `height: SizeConstraint::Fill`
    がデフォルト。両者を VStack の sibling に置くと
    `arrange_vstack` の `fill_count` が 2 になり、`remaining /
    fill_count = remaining / 2` で **HStack 行が vertical 半分を専有
    し、ScrollView の viewport も半分しか取れない**。
  - 回避: Button 2 つを HStack でラップせず VStack の直接 sibling
    として垂直に並べる。Button の `width / height` は
    `widget::button` constructor で `SizeConstraint::Fixed(btn_w) /
    Fixed(btn_h)` (= ラベル + padding) で固定なので、非 Fill 子として
    `non_fill_h` に算入され、ScrollView の Fill 子が
    `remaining = inner_h - (WrapPanel content_h + Button × 2 +
    spacing×4 + padding×2)` をまるごと取れる。
  - 美的な選択ではなく **layout 競合の構造的回避**。M4 で input
    handling が入って scrollbar widget が来れば標準の scroll 制御は
    そちらに移るので、T5 限定の workaround として恒久化しない。
- **`Start-Process` launch success:**
  - `Start-Process target/release/gallery-rust.exe -PassThru` で
    PID = 11832、`MainWindowTitle = "Wasamo"`、3 秒待機後に
    `HasExited = $false` を確認、`Stop-Process -Id 11832 -Force`
    で正常終了。process-stayed-alive signal で assistant-side の
    "Start-Process launch success" bullet を discharge。
  - `MainWindowTitle = "Wasamo"` は `.ui` 上の `title: "Gallery"`
    と一致しないが、これは現行の wasamo-runtime が Window title を
    `.ui` から拾わず framework 既定値で上書きしていることに由来
    (Phase 4 範囲外、別 phase の wiring に委ねる残置)。T5 の
    Start-Process 観測には無関係。
- **Clean rebuild gate:**
  `cargo clean` (3565 files, 1.0 GiB) → `cargo build --release
  --workspace` (40.93s、green) → `cargo build --workspace` (debug;
  34.40s、green) → `cargo test --workspace` (failure 0、
  `wasamo-runtime` lib 257 passed = T4 値で不変、integration test
  `scroll_view_layout_integration` 2 passed = T4 値で不変、他 crate
  全 green) → `cargo fmt --all -- --check` (post-commit state;
  zero exit)。
- **CI / GitHub Actions:** T6 phase-end gate (`workflow_dispatch`) で
  実 CI green を確認する。T5 では local clean rebuild が proxy。

T5 の blocker は残っていない。owner 明示承認後に
`feat/m3-phase-4-t5` を `feat/m3-phase-4` に no-ff merge して
T6 (phase-end / Moment 2 re-sync) に進める。

## Main Learning

中心的な学びは **「T4 で wire した writer chain を gallery `.ui` に
exercise させるとき、runtime 側に new code path が一切不要だった
ことが構造的に正しい」** という確認。Phase 4 ADR の Moment 1 設計
(DD-M3-P4-003 + DD-M3-P4-004 + architecture.md §6.5) が T1 → T4 で
端から端まで実装され、T5 は author 視点で `.ui` を書くだけで完結
する。これは Phase 2 / Phase 3 で確立した「ADR の Moment 1 design
draft が impl と divergence なく着地する」 doc-driven cycle が
Phase 4 でも維持されていることの動作確認。

副次的な学び:

- **VStack の Fill 子が viewport 提供器になる pattern.**
  ScrollView は dsl_spec §4.11 Sizing mental model #1 で「viewport
  size comes from parent」と規定されている。VStack 直下に Fill
  ScrollView を置けば、`arrange_vstack` の Fill 計算
  (`remaining / fill_count`) が viewport を提供する。Phase 4 ADR の
  整合性は維持されているが、`fill_count` が 2 以上になると **複数
  の Fill 子が viewport remainder を等分** することに注意。Gallery
  での HStack-vs-ScrollView 競合回避はその一例。これは新しい制約
  ではなく DD-M3-P3-002 と arrange_vstack の自然な合成なので、
  cross-step constraint 化はしない (Item 10 参照)。
- **HStack 既定 `height: Fill` の局所的不都合.** Button row を
  水平に並べたい authoring 直感に対して、HStack が Fill 高さを
  消費する既定は M3 Phase 3 で固まった shape (DD-M3-P3-002 Option
  A、HStack は cross-axis = Fill)。T5 では vertical stacking で
  迂回したが、より良い解は M4 の input handling 統合で scrollbar
  widget が来れば自然に解消する見込み (author は明示的に scroll
  Button を並べる必要がなくなる)。Phase 4 内では迂回で十分。
- **DSL syntax の note: `;` セパレータは parser 非対応.** dsl_spec
  §4.9 の例で `Box { aspect: <ratio>; fill: <color>; Text {...} }`
  と書かれている stylized 一行表記は **spec 上の説明的略記** で、
  実 parser は member の区切りに改行を必要とする
  (`expected member, found `;`` で T5 ビルド初回失敗)。spec 側を
  修正するかは T6 Moment 2 spec sync の判断材料。T5 では `.ui`
  を multi-line に展開して回避。Item 2 (spec changes) には登る
  可能性があるが Item 10 cross-phase constraint としては小さい
  (新規 widget の `.ui` を書くたびに同じ症状に当たるが、対処は
  multi-line への展開だけ)。

## Checklist

1. **本作業の主要な学び:** あり。
   - T4 で wire した writer chain が runtime 変更なしで gallery に
     exercise できることを確認 (Phase 4 doc-driven cycle の動作)。
   - VStack Fill 子による ScrollView viewport 提供 pattern と
     fill_count 競合の挙動 (`remaining / fill_count` 等分)。
   - HStack 既定 `height: Fill` で button row が ScrollView Fill と
     remainder を争う構造、vertical stacking で迂回可能。
   - dsl_spec §4.9 一行表記 `Box { ...; ...; ... }` は spec 説明用
     略記、実 parser は改行必須 (post-commit state では multi-line
     `.ui` で回避済み)。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:** **なし**
   - T5 の対象は `examples/gallery/gallery.ui` および
     `docs/plans/progress/m3-phase-4-progress.md` のみ。
   - dsl_spec §4.11 / architecture.md §6.5 / abi_spec.md は T5 では
     touch しない。`offset-y: scroll_y` の bare-identifier RHS は
     §4.11 Attributes で既に Moment 1 で規定済み、T5 の `.ui` は
     その規定の literal 適用。
   - 副次学び #3 (`;` セパレータの spec 例) は **T6 Moment 2 で
     dsl_spec §4.9 例示文の修正可否を判断する候補**。retrospective
     本 step では doc-driven 反映を実行せず、phase-end の判断に
     委ねる。

3. **ローカル clean rebuild:** **green**
   - `cargo clean`: 3565 files, 1.0 GiB removed。
   - `cargo build --release --workspace`: 40.93s, green。
   - `cargo build --workspace` (debug): 34.40s, green。windows crate
     compile 中の `STATUS_STACK_BUFFER_OVERRUN` sporadic rustc crash
     は T5 でも観測されず。
   - `cargo test --workspace`: failure 0 件、`wasamo-runtime` lib
     test = **257 passed** (T4 値で不変、T5 は新規 test なし)、
     integration test `scroll_view_layout_integration` = **2 passed**
     (T4 値で不変)、他 crate 全 green。
   - `cargo fmt --all -- --check` (post-commit state): zero exit。
   - GitHub Actions 上の clean rebuild は phase-end gate (T6) で
     確認。

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - すべて ADR DD-M3-P4-003 / DD-M3-P4-004 と dsl_spec §4.11 の
     範囲内で完結。
   - HStack 既定 `height: Fill` と ScrollView Fill 子の競合は
     DD-M3-P3-002 と dsl_spec §4.11 の自然な合成で、T5 限定の
     vertical stacking 迂回はオーナー相談不要と判定。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - `.ui` 1 ファイルに sibling slice を additive に追加しただけ。
     Phase 3 標準 WrapPanel slice は 10 box とも文字どおり untouched
     で残置。
   - ランタイム / wasamoc / IR loader / バインディングのコード変更は
     一切なし。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - T5 で発生した実装判断 (VStack root、Button vertical stacking、
     32 thumbnail、`item-cross-size: 64`) はいずれも `.ui` author
     スタイルの選択で、ADR 決定面に登る材料ではない。
   - dsl_spec §4.9 例示文の `;` セパレータ表記の見直しは T6
     Moment 2 sync 候補に残し、ADR には登らせない。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格:** **なし**
   - 当該 ADR (`docs/decisions/m3-phase-4-scroll-view.md`) は
     全 DD Accepted 済み。T5 では昇格対象なし。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**
   - A5 / A11 の文言変更なし。Phase 構成変更なし。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:** **なし**
   - `unimplemented!` / `todo!()` stub は一切なし。
   - 新規 `dead_code` 警告なし (T5 は runtime コードに触らないため
     構造的に発生し得ない)。
   - T4 で追加した `BuiltUi::__set_i32_state_for_test` 等の
     `#[doc(hidden)] pub fn` test helper は T5 では未使用 (T5 は
     gallery host 経由で実際の Button click に依存し、test-only
     helper は integration test 専用)。production code path に
     影響なし、dead-code 警告対象外。

10. **新たに発見・導入した cross-step / cross-phase の設計制約:** **なし**

    - HStack 既定 `height: Fill` と ScrollView Fill 子の合成は
      DD-M3-P3-002 と dsl_spec §4.11 にそれぞれ独立に規定済み。
      arrange_vstack の `remaining / fill_count` 等分挙動も
      pure-logic layout test (Phase 1) で既に押さえられている。
      合成の結果として「複数 Fill 子の viewport remainder 競合」が
      起こることは新たな normative claim ではなく、3 つの既存規定
      の機械的な合成。T5 では vertical stacking で迂回したのみで、
      cross-phase constraint 化する素材ではない。
    - dsl_spec §4.9 例示文の `;` セパレータ表記は spec 内の
      説明的略記の問題で、parser 側の規範は §3 grammar (改行
      区切り) が ground truth。T6 Moment 2 で例示文を改行表記に
      揃えるかどうかを判断する candidate に留め、cross-phase
      constraint としては立てない。

11. **タスクリストの後続 step 見直し:** **不要**
    - progress file の T5 行 6 項目を 5 件 `[x]` に flip 済み、
      残 1 件 (owner-manual GUI smoke) は ADR 規定通り T6 phase-end
      gate に持ち越し。
    - T6 (phase-end / Moment 2 re-sync) の task 構成・順序・依存
      関係に T5 実装から見て調整すべき点は出ていない。
    - Moment 2 spec sync 候補として `dsl_spec.md` §4.9 例示文の
      `;` セパレータ表記見直しが副次的に浮上 (本 retro Item 2 /
      副次学び #3)。T6 で取り扱う / 取り扱わないをオーナーと共に
      判断。

## Verification Notes

T5 で追加した test と、走らせた command を記録する。

新規テスト: **なし** (T5 は `.ui` の additive 拡張のみ。runtime /
ir_loader / wasamoc に変更なし)。T4 で landed した
`scroll_view_layout_integration` 2 件はそのまま green を維持。

実行コマンド:

```text
cargo clean                                       (3565 files, 1.0 GiB)
cargo build -p gallery-rust                       (初回 build; gallery.ui parse-error → multi-line 修正後 green)
cargo build --release --workspace                 (40.93s, green)
cargo build --workspace                           (debug; 34.40s, green)
cargo test --workspace                            (failure 0; wasamo-runtime lib 257 passed = T4 値で不変、scroll_view_layout_integration 2 passed = T4 値で不変)
cargo fmt --all -- --check                        (post-commit state; zero exit)
Start-Process target/release/gallery-rust.exe -PassThru  (PID 11832, MainWindowTitle "Wasamo", HasExited=$false after 3s; Stop-Process -Force で正常終了)
```

いずれも green。Start-Process は process-stayed-alive signal で
discharge (visible GUI smoke は owner 手動の T6 範囲)。

## Follow-Up

T5 から後続 task への明示的な引き渡し:

- **T6 (phase-end / Moment 2 re-sync):**
  - T5 残置の **owner-manual GUI smoke** (viewport clip 鋭さ、
    Button-driven content motion、clipped 領域非表示、off-viewport
    thumbnail 進入) を `retrospectives.md` checklist item 17 /
    [human-visible GUI smoke](../human-visible-smoke.md) に沿って
    T6 で消化。Phase 4 では `gallery-rust` のみ確認すれば足り、
    `counter-c` / `counter-zig` までは ADR 規定で out-of-phase
    (T5 checklist item 6)。
  - Moment 2 sync 候補として 2 件浮上:
    1. dsl_spec.md §4.9 例示文の `;` セパレータ表記を改行表記に
       揃えるかどうか (T5 副次学び #3)。判断は T6 でオーナーと共に。
    2. architecture.md §6.5 への "child の parent_abs_offset shift"
       明示追記の要否 (T4 retro Item 10 で `doc-folded` 判定済みだが
       T5 visible smoke で readability が話題になる場合は再検討、
       との note を T4 follow-up が残している)。T5 の `.ui` 追加で
       新たな materialな材料は出ていないので、T6 で覆す積極材料は
       現時点ではない。
  - dsl_spec §4.11 / architecture.md / abi_spec.md の現行 draft と
     T5 実装は整合済み。Moment 2 で sync 必要な substantive な
     divergence は無し。
- **Window title wiring (out-of-phase residual):**
  - gallery host の `MainWindowTitle = "Wasamo"` は `.ui` の
    `title: "Gallery"` と一致しない。これは現行 wasamo-runtime が
    Window title を `.ui` から拾わず framework 既定値で上書き
    している実装に由来し、Phase 4 ADR / dsl_spec §4.11 とは無関係。
    T6 で Out-of-phase residuals に登録するか、別 phase の wiring
    として残置するかをオーナーと共に判断。T5 内 closing にはしない。
- **将来 phase**:
  - M4 input handling で scrollbar widget / wheel handler / drag
    handler が入れば、Button-driven scroll controls は author が
    明示的に書く必要がなくなる。gallery sub-screen は M4 で再構成
    する可能性が高いが、T5 で landed した `.ui` 形は M3-Phase 4
    範囲では十分。

T6 の作業に直接効くのは Follow-Up 1 件目 (Moment 2 sync candidates)
と 2 件目 (Window title wiring の置き場判断)。
