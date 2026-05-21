---
title: M3-Phase 3 / T9 step-end retrospective
status: recorded
created: 2026-05-22
scope: step-end
task: T9 — `examples/gallery/` + `examples/gallery-rust/` additive growth (ADR §Phase 3 verification closure evidence item 5)
---

# M3-Phase 3 / T9 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-3-progress.md` の **T9**
(ADR §Phase 3 verification closure evidence item 5 — visible smoke;
m3-plan §Phase-end criteria item 5 — gallery sub-screen per phase) の
step-end retrospective。

T9 は framing decision E (uniform 1:1 placeholders, additive growth,
5–10 items, Rust host only) と framing decision G (visible
correctness is owner-manual GUI smoke; assistant does not assert on
pixel-level correctness) の制約下で、Phase 2 の単一 Box sub-screen
を `WrapPanel` of Box thumbnails に成長させ、`gallery-rust` host が
build + 起動できることまでを確認する step である。

対象コミット (3 件; 本 doc 含まず):

- `570d08a fix(wasamo-runtime): sync_visuals writes parent-relative offset (M3-Phase 3 T9)`
- `d1e5ba6 feat(examples): grow gallery sub-screen to WrapPanel of 8 thumbnails (M3-Phase 3 T9)`
- `e89a423 docs(m3-phase-3): close T9 gallery sub-screen step + visible-smoke evidence`

T9 が本 step で landed した材料:

- `examples/gallery/gallery.ui` を additive に成長 — Phase 2 の単一
  `Box { aspect: 16:9; … }` を `WrapPanel` of 8 uniform 1:1 Box
  thumbnails (`item-cross-size: 120; item-spacing: 16;
  line-spacing: 16`) に置き換え。Default 800×600 client window で
  5+3 wrap、より狭い window で 2-col / 1-col に rewrap することを
  smoke 確認 (framing decision E)。
- `examples/gallery-rust/README.md` を Phase 3 のサブスクリーン形状
  に追記。`Start-Process` で .exe を起動して visible smoke が
  取れるところまで確認 (framing decision G; assistant は GUI 正しさ
  そのものを assert しない)。
- 3 枚の visible-smoke スクリーンショットを
  `docs/references/m3-phase-3/` 配下に保存
  (`t9-gallery-smoke-default-window.png`,
   `t9-gallery-smoke-medium-rewrap.png`,
   `t9-gallery-smoke-narrow-rewrap.png`) — Phase 3 close 後に
  ADR / progress / 本 retrospective から evidence として参照
  可能な形に固定。
- `wasamo-runtime/src/widget.rs` の `sync_visuals` を
  parent-relative offset 書き込みに修正 (詳細は Main Learning)。
  Phase 2 から潜在していた visual-tree offset 二重加算バグが
  Phase 3 T9 の visible smoke で初めて顕在化した。

これにより ADR §Phase 3 verification closure evidence item 5
(visible smoke via WrapPanel-of-Boxes sub-screen) と
m3-plan §Phase-end criteria item 5 (gallery sub-screen per phase;
Rust host only at Phase 3) が **landed** 状態に達する。最終
discharge は T10 phase-end gates 内で `Start-Process` smoke 再確認 +
spec status marker flip を経て成立する。

step-end の gate であり phase-end retrospective ではない。merge 先は
phase ブランチ `feat/m3-phase-3` (ff)。本 step (T9) は単一 task =
単一 step 構造で、現在のブランチは `feat/m3-phase-3-t9`。

## Current Judgment

2026-05-22 時点で T9 step-end 基準は **達成済み**。

- progress file T9 checklist 4 項目はすべて `[x]` に flip 済み。
- `cargo build --release -p gallery-rust` 成功。`Start-Process
  .\target\release\gallery-rust.exe` が `MainWindowTitle: Wasamo`
  の window を立ち上げることを `Get-Process` で確認。
- owner-manual GUI smoke で 3 width state (default / medium /
  narrow) を撮影し、すべての `Photo N` ラベルが対応 Box thumbnail
  の中央に正しく描画されることを確認。WrapPanel 自体の line
  breaker は 5+3 / 2×4 / 1×8 と width 変化に追随して再 wrap
  している。3 枚は `docs/references/m3-phase-3/` 配下に
  evidence として保存済み。
- clean rebuild gate (post-commit hash `e89a423`) も green:
  `cargo fmt --all -- --check` (前後とも zero exit) →
  `cargo clean` (1043 files / 236.1 MiB 削除) →
  `cargo build --release --workspace` (37.09s) →
  `cargo build --workspace` (32.99s) →
  `cargo test --workspace` (workspace 全 test green; 内訳は
  Verification Notes 参照)。
- T9 由来の warning / failure はなし。既存の `wasamo-sys` import
  library order warning のみ既存通り。

## Main Learning

中心的な学びは「**pure-data layout engine の絶対座標と WinRT
Composition `Visual.Offset` の parent-relative セマンティクスの
ギャップは、Phase 2 では非ゼロ offset のネストが発生せず潜在化
していた**。Phase 3 で WrapPanel が非ゼロ offset の Box を孫として
配置した瞬間に visible smoke で露出し、layout engine 側ではなく
**`sync_visuals` 境界**で 1 箇所の引き算を入れることで解決した」。

`wasamo-runtime/src/layout.rs` の `arrange*` 系はすべて呼び出し元
から受けた `(x, y)` をそのまま `node.offset` に書き込み、子へは
`x + child_local_x`, `y + child_local_y` を絶対座標として渡す。
`run_layout` の最上層が `(0.0, 0.0)` で root を arrange するので、
全 `LayoutNode.offset` は **window-client-origin 基準の絶対座標**
になる。これは pure layout のテスト容易性 (`x + width > parent_w`
のような不等式が単純に書ける) を支えている設計で、Phase 2 / T7
のテスト群もこの規約に依存している。

一方 WinRT Composition の `Visual.Offset` は **直近の親 Visual
からの相対オフセット**で、`InsertAtTop(child_visual)` で挿入された
子は親の Offset をベースに描画される。Phase 2 の gallery では
Window root → 単一 Box → 内側 Text という visual tree の各段が

- Window container は (0, 0)
- Box は Window-root であり absolute (0, 0)
- Text の絶対座標 (Box の中央) ≈ Box の絶対座標 + 中央オフセット

となり、Box の absolute offset が `(0, 0)` だったため
「absolute を relative として書く」誤読が結果的に正しく描画する
偶然が成立していた。Phase 3 T9 では WrapPanel が Box thumbnail を
非ゼロ offset で並べた瞬間に、Text の SetOffset(absolute) が
parent Box の SetOffset(absolute) と二重加算され、ラベルが
WrapPanel rect の外まで押し出されて表示された (例: Box 6 が
row 2 col 1 (0, 136) のとき Text label は (30, 306) に出るのが
正解だが、`(0, 136) + (30, 170)` で `(30, 306)` ではなく
`(30, 306)` … 実際の screen 上では `(0+30, 136+170) = (30, 306)`
が表示位置になり、文字は wrap panel の下端を超えた領域に流れた)。

修正は `sync_visuals` に `parent_abs_offset: (f32, f32)` 引数を
増やし、

- `visual.SetOffset(computed.offset - parent_abs_offset)` で
  parent-relative に変換した値だけを Composition に書く
- 子再帰には `computed.offset`(= 子から見た parent の absolute
  offset) を渡す

の 1 段ローカルな引き算で済ませた。`run_layout` のエントリ点は
`(0.0, 0.0)` でシードするので、root visual の relative offset は
absolute offset と一致したまま (= 既存挙動を regress させない)。
**layout engine 側 (`LayoutNode.offset` の絶対座標規約) は触らない**
ことで、T7 が pin した line-breaker / arrange の不等式テスト群が
そのまま残り、235 runtime + 202 wasamoc + integration test は
no-change で green を維持した。

副次的な学び:

- **Phase 2 で導入した nested-visual パターンは「親が `(0, 0)` の
  ときだけ偶然動く」コードを含んでいた**。これを T9 visible smoke
  が捕まえたのは、framing decision G が「owner-manual GUI smoke で
  pixel-level correctness を見る」と定めていた効果。runtime test
  side では Phase 2 / Phase 3 とも `WidgetNode` を Compositor 込み
  で組み立ててから `LayoutNode.offset` の値を assert する path は
  存在せず、Visual.Offset の意味論が assert される現行 test は
  T8 oversized-child fixture (overflow assertion で `x + width >
  parent_w` を見るが、これは pure-layout 値の話で Visual.Offset
  ではない) を含めて存在しない。`sync_visuals` ↔ pure-layout の
  境界に「relative vs absolute 規約」の test がないという穴は
  follow-up に記録する。
- **DSL の child separator は `;` ではなく改行**。T8 で 1 回踏んだ
  落とし穴を T9 で 1 度繰り返した (初稿の `Box { aspect: 1:1;
  fill: #336699cc; ... }` が `expected member, found `;``)。
  T8 retrospective に同種の記述が残っており、T9 ではすぐに
  複数行構文に直して green に到達したので process learning は
  反復済み — 個別の DD は不要。
- **テキスト形式 `.uic` の手動生成は in-tree に置かない**。step 中に
  `wasamoc build examples\gallery\gallery.ui examples\gallery\gallery.uic`
  を debug 目的で実行し `.uic` がリポジトリ直下に生成された。
  `.gitignore` には `.uic` パターンが無く、commit 前に手で削除した。
  `examples/*/build.rs` 経由の OUT_DIR 出力と異なり、in-tree への
  直接出力が想定外であることに気づくきっかけになった。
  `.gitignore` への追加は T10 ではなく out-of-phase residual として
  記録 (Phase 3 scope 外、本 step で発生したが本 step では fix
  しない)。

## Checklist

1. **本作業の主要な学び:** あり。
   - `sync_visuals` の absolute / relative offset 規約ギャップが
     Phase 2 のテスト網と framing decision G の owner-manual GUI
     smoke の隙間に潜んでいた、という構造的気付き (Main Learning
     に展開)。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`)
   の変更:** **なし** **(FT)**
   - T9 は example sub-screen + runtime fix + 進捗記録のみ。
   - `architecture.md` の visual-tree / layout-engine 境界の記述に
     parent-relative セマンティクスを 1 行明記する余地はあるが、
     これは T10 (Moment 2 architecture re-sync) の方が適切。
     T9 の検出事実は本 retrospective + commit message が
     primary record として残る。

3. **ローカル clean rebuild:** **green** **(FT)**
   - `cargo fmt --all -- --check` (pre-clean): zero exit。
   - `cargo clean`: success (1043 files / 236.1 MiB 削除)。
   - `cargo build --release --workspace`: success (37.09s)。
   - `cargo build --workspace`: success (32.99s; 既存の
     `wasamo-sys` import library order warning のみ)。
   - `cargo test --workspace`: success — 内訳 (零件数の result
     行は省略):
     - bool-demo-rust integration: 1 passed
     - counter-rust integration: 12 passed
     - wasamo-runtime unit + integration: 235 passed (内
       wrap_panel / box / ir_loader 各 fixture 群を含む)
     - wasamo-runtime tests/box_layout_integration: 1 passed
     - wasamo-runtime tests/wrap_panel_layout_integration: 2 passed
       (Compositor available; skip path に入らず実行 pass)
     - wasamo-runtime tests/ir_loader_roundtrip 系: 1 + 1 + 6 = 8 passed
     - wasamo-runtime tests/abi_load_ui / bool_binding_live_propagation /
       button_enabled / live_widgetnode_headless: 1 + 3 + 1 + 1 = 6 passed
     - wasamoc unit: 202 passed
     - wasamoc bins (check / build) と CLI tests: 各 0–6
   - `cargo fmt --all -- --check` (post-rebuild): zero exit。
   - 全 gate は post-commit hash `e89a423` 上で実行。

4. **PO に相談すべき設計判断・トレードオフ:** **あり** **(FT)**
   - **`sync_visuals` の parent-relative 変換を T9 内で fix するか
     out-of-phase residual に回すかの判断**を session 中に確認
     (選択肢提示の結果、owner 選択は "T9 内で fix")。本判断は
     framing decision G の "assistant does not assert on pixel-
     level correctness" と "visible smoke で owner が見つけた
     不具合をどう扱うか" の境界事例で、T9 scope 拡張に該当する
     ため事前 PO 確認を経た。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **あり** **(FT)**
   - `wasamo-runtime/src/widget.rs` の `sync_visuals` 修正は
     T9 progress doc の checklist 文言には含まれていなかった
     "Phase 2 由来の visual-tree offset 二重加算 bug の fix"
     である。framing decision G が requires the assistant to
     surface owner-found visible smoke failures に該当し、
     上記 item 4 で PO 確認済み。Phase 3 scope 内 (Phase 3 で
     visible smoke を成立させるための必要条件) であり、新 ADR
     や DD を要しない実装修正なので step-end fast-track 判定の
     対象には残るが、checklist 上は "あり" として明示する。

6. **現在の phase ADR への追加 DD 必要性:** **なし** **(FT)**
   - `sync_visuals` fix は ADR の DD レベルの設計判断ではなく
     既存 visual-tree 構築規約 (DD-M2-P4-001/002 の Visual.Children
     親子関係) の整合性 fix。Accepted DD の解釈変更はなし。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed →
   Accepted への昇格:** **なし** **(FT)**
   - 当該 ADR は T6 closing 時点で全 DD Accepted 済み。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし** **(FT)**
   - A3 文言変更なし。Phase 構成変更なし。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **なし** **(FT)**
   - `todo!` / `unimplemented!` / 新規 `#[allow(dead_code)]` は
     導入していない。`sync_visuals` 修正は fully implemented;
     placeholder ではない。

10. **タスクリストの後続 step 見直し:** **不要**
    - T10 (phase-end gates) は現行順序で進行可。`sync_visuals` の
      fix が landed 済みなので、T10 中の architecture.md 再 sync で
      visual-tree / layout-engine 境界の relative-vs-absolute 規約を
      1 行明記するかどうかが新規論点として加わるが、これは T10 内
      の判断であり T9 step list 側の改訂は要さない。

## Fast-Track Judgment

Fast-track criteria は **満たさない** (item 4 / item 5 が「あり」):

- item 2: なし
- item 3: green
- **item 4: あり** (PO 確認済みだが flip 自体は本 retrospective で
  明示。session 中の確認内容を merge gate として再確認する余地が
  残る)
- **item 5: あり** (`sync_visuals` fix; PO 確認済みだが checklist
  上は明示)
- item 6: なし
- item 7: なし
- item 8: なし
- item 9: なし
- item 10: 不要

本作業では merge は実行していない。step→phase の扱いはオーナーの
次アクションに従う。

## Verification Notes

実行コマンド (post-commit hash `e89a423` 上):

```text
cargo fmt --all -- --check
cargo clean
cargo build --release --workspace
cargo build --workspace
cargo test --workspace
cargo fmt --all -- --check
```

すべて green。`cargo build --release --workspace` /
`cargo build --workspace` では既存の `wasamo-sys` import library
order warning が表示されたが T9 由来の warning / failure はなし。

`cargo build --release --workspace` の前段 (1 回目 `cargo build
--release --workspace`) で rustc が `windows` crate compile 中に
`Allocation failed` / `STATUS_STACK_BUFFER_OVERRUN` を返した事象が
1 回発生したが、これはローカル開発機の RAM 枯渇 (並列 rustc が
windows-0.58 の巨大 lib.rs を opt-level=3 で複数本同時に処理した
ことが原因) であり、開発機を再起動した後の再実行 (本記録の上記
コマンド列) は green。T9 の commit graph に副作用なし。

Visible smoke (framing decision G; owner-manual):

- `examples/gallery-rust/target/release/gallery-rust.exe` を
  `Start-Process` で起動し、`Get-Process gallery-rust` が
  `MainWindowTitle: Wasamo` の window を 1 つ確認。
- 3 width state (default 約 800×600 / medium / narrow) の
  smoke 撮影を owner が実施。`docs/references/m3-phase-3/` に
  3 枚を evidence として保存:
  - `t9-gallery-smoke-default-window.png` — 5+3 wrap
  - `t9-gallery-smoke-medium-rewrap.png` — 2-col × 4 row
  - `t9-gallery-smoke-narrow-rewrap.png` — 1-col
- 3 枚すべてで Box thumbnail と Photo label が正しく入れ子で
  描画されていること、WrapPanel の line breaker が width 変化に
  追随して再 wrap していることを owner が目視で確認。
- 第 1 round の smoke (`sync_visuals` fix 前) では Photo label が
  WrapPanel rect の外に押し出される失敗を観測し、その screenshot
  を起点に Main Learning の解析と fix が成立した。失敗側の
  screenshot は private/ に owner が保持し、git には含めない。

## Follow-Up

T9 から発生した out-of-phase residual:

- **(R1) `.gitignore` への `.uic` パターン追加。** 本 step 中に
  `wasamoc build examples\gallery\gallery.ui examples\gallery\gallery.uic`
  を debug 目的で実行した結果、in-tree に `.uic` が生成された。
  本 step では手動削除で対処したが、`.uic` を debug 出力として
  in-tree に書く誘惑は再発しうる。`.gitignore` に `*.uic` を
  追加する変更は **Phase 3 scope 外** (build path は OUT_DIR 経由
  であり in-tree `.uic` は production path に存在しない) なので、
  T9 commit set には含めず、別 step (Phase 3 T10 の "Out-of-phase
  residuals filed" 枠、もしくは Phase 4 以降の cross-cutting 整理)
  で扱う。本 retrospective に記録するに留める。
- **(R2) `sync_visuals` ↔ pure-layout の境界 test 欠落。** Phase 2
  の test 網は `LayoutNode.offset` の絶対座標規約は pin している
  が、その絶対値が Composition の `Visual.Offset` (parent-relative)
  に渡る際の変換が正しいことを assert する path が無かった。
  T9 visible smoke で偶然見つかった形だが、ネストされた非ゼロ
  offset visual tree の relative-offset 計算を assert する pure
  もしくは Compositor-backed test を Phase 4 以降のどこかで設けると
  類似 bug の回帰検出が visible-smoke 依存でなくなる。T9 内では
  実装しない (Phase 3 scope 外; framing decision G は visible
  smoke を canonical 検出路として認容しており、追加 test は
  optional)。

T10 への引き継ぎ:

- **T10:** phase-end gates で full workspace clean rebuild +
  CI green 確認 + spec status marker flip (`dsl_spec.md` §4.10
  draft → closed-impl-synced) + architecture.md §6.8 reconcile を
  実施。本 retrospective の "副次的な学び" に挙げた "visual-tree /
  layout-engine 境界の relative-vs-absolute 規約を architecture.md
  に 1 行明記するか" は Moment 2 architecture re-sync 内の小判断
  として T10 で扱う。
- **T10:** Out-of-phase residuals filed の枠で R1 (`.gitignore`
  への `.uic` 追加検討) を progress doc の §Out-of-phase residuals
  に追記。
