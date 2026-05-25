---
title: M3-Phase 4 / T6 step-end retrospective
status: recorded
created: 2026-05-25
scope: step-end
task: T6 — Owner-manual GUI smoke and any visible-correctness fix
---

# M3-Phase 4 / T6 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-4-progress.md` の **T6**
("Owner-manual GUI smoke and any visible-correctness fix") の step-end
retrospective。T6 は T5 close 時の split (Decisions log "T5/T6 split
for owner-manual GUI smoke (2026-05-25)") で新規挿入された step で、
Phase 4 verification closure **evidence item 5** の visible-correctness
半分 (owner-manual GUI smoke + 不具合の T6 内 fix iteration) と A11
gallery proof の owner acceptance 半分を discharge する。assistant 側
の自動化部分 (`.ui` + Build + Start-Process) は T5 で landed 済み。

T6 は owner-manual smoke で **failure mode A** が判明し、T6 ブランチ
内 additive fix iteration を回した step。原始 T6 は smoke pass のみを
想定していたが、smoke fail → fix bundle → re-smoke green までを一つの
step として閉じる構造になった (progress doc Decisions log "T6 smoke
failure mode A disposition (2026-05-25)" 参照)。

対象実装コミット (`feat/m3-phase-4-t6` 上、本 retrospective ファイルを
除く):

- `e812813 docs(m3-phase-4): record T6 smoke failure mode A + selected fix bundle`
- `ed78d6c fix(wasamo-runtime): force window-root WidgetNode to Fill/Fill (M3-Phase 4 T6)`

`e812813` は **progress doc small update を impl 前に切り出した**
commit (owner critical review にて推奨された進行順を反映)。`ed78d6c`
が runtime split + gallery.ui 調整 + 2 件の test 追加を含む fix bundle
本体。本 retrospective + smoke evidence の commit が close commit に
なる。

merge 先は phase ブランチ `feat/m3-phase-4` (no-ff、`feedback_workflow`
§1 / `retrospectives.md` §進行手順)。phase → main は T7 の phase-end
gate に持ち越す。

## Current Judgment

2026-05-25 時点で **T6 step-end 基準は達成済み (owner 明示承認待ち)**。
fast-track は廃止 (`feedback_workflow` §2(b) / `49b49fb`) のため、判定
にかかわらず owner 明示承認待ちで停止する。

- **Failure mode A diagnosis (initial smoke):**
  - 初回 owner smoke (T5 release artifacts on
    `target/release/gallery-rust.exe`, build 2026-05-25 19:49) は
    (1) ScrollView 領域が `scroll_y = 0` で完全に空、
    (2) "Scroll down (+100)" を 5 回押下しても画面変化なし、で fail。
  - 原因: gallery.ui の component root は VStack で default
    `width: Fill, height: Shrink`。`layout::measure_vstack` の
    Shrink ブランチは Fill 子の desired_h を除外する慣習 (基底挙動
    として `layout::tests::degenerate_fill_in_shrink_parent_clamps_to_zero`
    が pin 済み) を踏むので、root VStack は `desired_h ≈ 312`
    (Phase 3 WrapPanel + Button × 2 + spacing/padding) に解決される。
    `arrange_vstack` の `remaining = max(0, inner_h − non_fill_h −
    spacing_total) = 0` で、Fill ScrollView 子は `child_h = 0` を
    受け取り outer Visual が `(w, 0)`、`InsetClip{0,0,0,0}` の
    auto-track で clip 高さ 0 → content が一度も描画されない。
    writer chain も `viewport_h = 0 → max_offset = 0 → applied = 0`
    で見た目に何も起こせない。
  - T4 integration test (`scroll_view_layout_integration.rs`
    FIXTURE_SRC) は ScrollView を component 直下に置く構成
    (Fill/Fill default) で、gallery / counter / bool-demo が踏む
    VStack-rooted 経路をテストしていなかった。T4 close 時の
    "fixture WrapPanel substitution vs ADR primary VStack" の補足
    commit (`57f2366`) が divergence を既に flag していたが、
    visible-correctness consequence は T6 smoke で初めて顕在化。

- **Fix bundle (no normative spec change):**
  - (a) `WidgetNode::run_layout` を分割。**新 entry point**
    `WidgetNode::run_layout_as_window_root` は root LayoutNode の
    `width`/`height` を `SizeConstraint::Fill` に上書きしてから
    `layout::run_layout` に delegate。`window.rs` の `WM_SIZE`
    handler と `set_root` 初期 layout を新メソッド呼び出しに切替。
    plain `WidgetNode::run_layout` は既存 semantic 不変なので
    `wrap_panel_layout_integration.rs` (declared sizing constraints
    を直接 exercise する mock-free integration) は無修正で green
    維持。pure-logic `layout::run_layout` も触らず、
    `degenerate_fill_in_shrink_parent_clamps_to_zero` も含む既存
    convention は pin 維持。
  - (b) `examples/gallery/gallery.ui` ScrollView 内 WrapPanel
    `item-cross-size: 64 → 128`。Fix (a) 後でも `64` だと
    800×600 〜 1280×900 の window 幅で `content_h < viewport_h`
    (max_offset = 0) となり Button 押下後も画面が動かない問題が
    残るため。128 で `content_h > viewport_h` を確保。ADR の
    "Box × 30–40" range と Phase 3 standalone WrapPanel slice は
    untouched。
  - (c) pure-logic pinning unit test
    `layout::tests::shrink_vstack_root_with_fill_scroll_view_child_collapses`。
    gallery-shaped VStack root (mixed Shrink-height + Fixed-height +
    Fill-height children including ScrollView) で
    `degenerate_fill_in_shrink_parent_clamps_to_zero` の結果を
    再現した上で、同 root を Fill height に pre-set し直すと Fill
    子の allocated height が非ゼロに反転することを assert。
  - (d) mock-free runtime integration test
    `scroll_path_vstack_root_fixture_pins_window_root_fill_override`
    in `scroll_view_layout_integration.rs`。VStack-rooted gallery-
    shaped `.ui` (Button + ScrollView) を `wasamoc` で lower し、
    `run_layout_as_window_root(200, 200)` を driving。
    (i) ScrollView outer Visual height > 0 at `scroll_y = 0`
    (Fix (a) の regression gate)、
    (ii) intermediate content Visual Y offset が `scroll_y = 100`
    で負 (writer chain end-to-end on the production path) を assert。
  - (c) (d) で layout engine 不変条件と runtime-boundary 上書きの
    両層を独立に pin。

- **Re-smoke green:**
  - Owner が 2026-05-25 に rebuilt `gallery-rust.exe` で再 smoke を
    実施し、4 観察項目 + 2 reference 項目をすべて green と報告。
    smoke evidence は
    [docs/references/m3-phase-4/](../../references/m3-phase-4/)
    配下に Phase 3 規則 (`t<N>-gallery-smoke-<axis>.png`) に揃えた
    命名で commit:
    - `t6-gallery-smoke-scroll-y-0.png` — 起動直後
      (`scroll_y = 0` で S01–S10 が viewport 内に描画、Photo 1–10
      と区別されている)
    - `t6-gallery-smoke-scroll-y-100.png` — "Scroll down (+100)" 1
      回押下後 (S01–S05 上部 100px が pixel-clean に clip、S11–S15
      が下端から進入)
    - `t6-gallery-smoke-scroll-y-800.png` — `+100` を上限まで連打
      した後 (S26–S30 / S31–S32 が viewport 上端寄りに到達、
      `applied_offset_y` が max_offset に clamp)
    - `t6-gallery-smoke-scroll-y-back-to-0.png` — 続いて `-100` を
      下限まで連打した後 (`scroll_y_0` と pixel-identical、0 clamp
      対称性確認)
  - window close (Alt+F4 / ×) で crash dialog なし、smoke 暗黙要件
    green。

- **Clean rebuild gate:**
  `cargo fmt --all -- --check` (post-commit state; zero exit) →
  `cargo build --release --workspace` (green) → `cargo build
  --workspace` (debug; green) → `cargo test --workspace`
  (failure 0; `wasamo-runtime` lib 258 passed = T5 baseline 257 + 1
  new pure-logic pinning test、
  `scroll_view_layout_integration` 3 passed = T5 baseline 2 + 1 new
  root-VStack integration test、
  `wrap_panel_layout_integration` 2 passed = `run_layout` 分割で
  override の影響範囲外に戻り green 維持、他 crate 全 green)。

- **CI / GitHub Actions:** **T7** phase-end gate
  (`workflow_dispatch`) で実 CI green を確認する。T6 では local
  clean rebuild が proxy。

**assistant-side T6 blocker は残っていない**。owner 明示承認後に
`feat/m3-phase-4-t6` を `feat/m3-phase-4` に no-ff merge し、続けて
T7 (phase-end / Moment 2 re-sync) に進む。

## Main Learning

中心的な学びは **「production path の `.ui` shape を実際に踏まないと
集積された latent 制約は表に出ない」** ことの再確認。Phase 1〜3 の
全 ADR / 全 layout test が積み上げてきた "Fill 子の意味論は parent
bounded を前提とする" convention は、Phase 2 / Phase 3 の counter /
bool-demo / gallery (Phase 3 WrapPanel slice 単独) では root container
に Fill 子が無いまま済んでいた。Phase 4 で初めて root VStack に Fill
ScrollView 子を載せた結果、convention の機械的帰結として "Shrink
parent が Fill 子を 0 に潰す" 古典的な layout collapse が顕在化した。
これは **新しい bug ではなく、既存規約の合成的帰結が visible 表面に
出ただけ** の事象である。

副次的な学び (cross-phase 制約候補は item 10 で再判断):

- **Test fixture parent shape の divergence が引き起こす blind spot.**
  T4 integration test は ScrollView を component 直下 (Fill/Fill
  default) に置く構成で landed していた (T4 retro / `57f2366`
  "fixture WrapPanel substitution vs ADR primary VStack" で
  divergence は doc 上明示済み)。production の `.ui` は全て
  container を root に置くので、fixture parent shape を production
  shape に揃えるのが安全な default だが、Phase 4 は ScrollView 自身の
  Visual 構造に集中したいという理由で意図的に divergence を許容
  していた。T6 の root-VStack integration test 追加で divergence は
  解消したが、今後の widget catalog 拡張で同じ罠を避けるための
  方針 ("**production の `.ui` で root に置かれる shape の少なくとも
  一つを integration test fixture parent として常時カバーする**") を
  T7 carry-over として残す価値がある (item 10 / item 15 で判断)。

- **`layout::run_layout` の semantic を変えない選択が正しかった.**
  当初の fix 案は `WidgetNode::run_layout` ひとつに root Fill 上書きを
  足す構造だったが、owner critical review で **既存 `wrap_panel_
  layout_integration.rs` が WidgetNode::run_layout を直接 drive する
  test を持っていた** ことが判明 (test 失敗で具体化)。`run_layout` /
  `run_layout_as_window_root` の split で「pure-logic layout engine
  は完全不変」「`WidgetNode::run_layout` は declared sizing
  constraints を honour」「`run_layout_as_window_root` は window-root
  policy を担う」の 3 階層が明確に分離。**「override は最も狭い boundary
  に置く」** という設計原則の具体的な適用例として記録。

- **Phase 4 仕様内 `scroll_y` drift の visible 表れ.**
  Owner re-smoke で「`Scroll down/up` 連打で画面が止まった後、逆方向
  を 4 回押して初めて画面が動き始める」観察が報告された。これは
  `scroll_y` Signal 自体が clamp されず drift し、`applied_offset_y`
  だけが layout 時に clamp される構造の自然な帰結。Phase 4 ADR の
  scope ([m3-phase-4-progress.md:18-22](../../plans/progress/m3-phase-4-progress.md#L18-L22))
  で `in-out offset-y write-back` を M4 scope に明示的に外している
  ので **bug ではなく仕様内 limitation**。owner-manual smoke が
  visible に表れる "Phase 4 と M4 の境界" を画面越しに確認できた、
  という意味で M4 design input としても意味のある観察。

- **`progress doc small update → impl → close commit` の進行順が
  「smoke fail → fix iteration」局面で実効的に機能した.** Owner
  critical review が「実装前に progress doc を 1 commit 切ること」を
  推奨した直接的な動機は、(1) failure mode A の disposition が
  Decisions log に明示される、(2) fix bundle の 4 件 (runtime split
  + gallery 調整 + 2 件 test) が "T6 check list を具体化した検証戦略"
  として progress 上で見える、という 2 点。実装後にまとめて記述する
  だと、後から見た時に「smoke fail を T6 でどう吸収したのか」が
  retro 本文だけからは見えにくく、進行記録としては痩せていた可能性
  がある。step ブランチを切ってから (a) 進行記録の追加 (b) impl
  (c) 検証 (d) close という order は今後も smoke iteration を伴う
  step では default として採用する価値がある。

## Checklist

1. **本作業の主要な学び:** あり。
   - production `.ui` shape を踏まない integration test fixture は
     既存規約の合成的帰結 (Shrink VStack root + Fill ScrollView 子
     collapse) を pin できない。
   - layout policy の override は最も狭い boundary に置く設計が
     既存 test の semantic を保護する (`run_layout` /
     `run_layout_as_window_root` split)。
   - Phase 4 ADR scope 内の `scroll_y` drift は visible smoke で
     初めて画面越しに表れる M4 design input。
   - smoke iteration を伴う step では progress doc small update を
     impl 前に 1 commit で切り出す進行順が retro 価値を高める。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の
   変更:** **なし**
   - `runtime split` も `gallery item-cross-size 調整` も normative
     spec surface に影響しない。runtime "window-root WidgetNode is
     sized to client rect" policy は `architecture.md §6` 一般 layout
     section に 1 文の `phase-sync` 候補として **T7 で fold** する
     (本 retro Follow-Up 参照)。本 step では doc 反映なし。

3. **ローカル clean rebuild:** **green** (詳細は Verification Notes
   参照)。`cargo fmt --all -- --check` post-commit state で zero exit。
   `cargo test --workspace` failure 0、wasamo-runtime lib 258 (T5
   baseline +1)、`scroll_view_layout_integration` 3 (T5 baseline +1)、
   `wrap_panel_layout_integration` 2 (`run_layout` split で
   override の影響範囲外に戻り green 維持)。GitHub Actions 上の
   clean rebuild は T7 phase-end gate で確認。

4. **PO に相談すべき設計判断・トレードオフ:** **あり (T6 内で
   review-and-go 済み)**
   - **fix の granularity と置き場所** (`WidgetNode::run_layout` 単一
     に override を足すか split するか、gallery `item-cross-size` を
     どの値にするか、test を pure-logic / integration の片方だけに
     するか両方か) を smoke fail 直後に owner critical review で
     確認し、4 件 fix bundle + progress doc small-update-first の
     進行順を確立した。本 retro Main Learning の 2 つ目・4 つ目の
     学びは review からの直接的な反映。
   - Phase 4 仕様内 `scroll_y` drift の取り扱い: T6 内 fix にせず M4
     handoff item として retro / progress doc に記録、で合意済み
     (本 retro Follow-Up 参照)。
   - smoke evidence の `docs/references/m3-phase-4/` への移動可否:
     owner は当初「step-end なら不要 / phase-end なら移動」の直感を
     提示。Phase 3 T9 step-end 先例
     ([t9-step-end-retrospective.md](../m3-phase-3/t9-step-end-retrospective.md))
     が step-end でも `docs/references/m3-phase-3/` に commit して
     いることを根拠に **Phase 3 precedent に倣って T6 step-end で
     commit** を選択。process consistency を優先。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - `WidgetNode::run_layout` の split は visible-correctness fix
     のための最小手当て。`run_layout_as_window_root` 新 API 追加と
     `window.rs` 2 call site 切替に閉じている。
   - `examples/gallery/gallery.ui` の `item-cross-size: 64 → 128` も
     smoke 可視化のための 1 行の調整。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - failure mode A は既存 layout convention (DD-M3-P3-002 +
     `degenerate_fill_in_shrink_parent_clamps_to_zero`) の機械的
     帰結であり、新 design decision を要しない。
   - `run_layout` / `run_layout_as_window_root` split は runtime-
     boundary policy の実装詳細であり、Phase 4 ADR の DD 構造には
     登らない (architecture.md §6 への 1 文追加で十分、T7 carry
     over)。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格:** **なし**
   - Phase 4 ADR (`docs/decisions/m3-phase-4-scroll-view.md`) 内
     DD はすべて Accepted 済み、T6 では昇格対象なし。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**
   - A5 / A11 文言不変。Phase 構成不変。T5 close で確立済みの
     T5/T6/T7 構成のまま進行。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:** **なし**
   - `unimplemented!` / `todo!()` stub なし。新規 `dead_code` 警告
     なし。
   - `run_layout_as_window_root` の追加で plain `run_layout` が
     production path から外れたが、`wrap_panel_layout_integration`
     ほか mock-free integration test がまだ exercise しているので
     dead-code ではない。public API として今後も維持。
   - `BuiltUi::__set_i32_state_for_test` 等の T4 で追加した
     `#[doc(hidden)] pub fn` test helper も
     `scroll_path_vstack_root_fixture_pins_window_root_fill_override`
     で active に使われたので status 変化なし。

10. **新たに発見・導入した cross-step / cross-phase の設計制約:**
    **あり** (3 件、内訳: `phase-sync` 1 件 + `carry-forward` 2 件)

    - **制約 (1):** **window-root WidgetNode は declared sizing
      constraints に関わらず client rect に Fill される** という
      runtime-boundary policy。
      - **エビデンス:** T6 failure mode A diagnosis、
        `WidgetNode::run_layout_as_window_root` 実装、
        `scroll_path_vstack_root_fixture_pins_window_root_fill_override`
        integration test の存在。
      - **配置先:** **`phase-sync`**。`architecture.md §6` (general
        layout、§6.5 ScrollView ではない) に 1 文追加: "the window-
        root WidgetNode is sized to the client rect regardless of
        its declared width/height constraints; this is enacted by
        `WidgetNode::run_layout_as_window_root` and called by
        `window.rs`'s `WM_SIZE` handler and `set_root` initial
        layout"。T7 Moment 2 spec re-sync の Moment 2 candidate
        として処理。本 step では doc 反映なし。

    - **制約 (2):** **integration test fixture の parent shape は
      production `.ui` で root に置かれる shape の少なくとも一つを
      常時カバーする**。
      - **エビデンス:** T4 integration test (ScrollView-rooted)
        が VStack-rooted 経路を pin できず T6 smoke で latent
        collapse が顕在化した事実。T6 で
        `scroll_path_vstack_root_fixture_pins_window_root_fill_override`
        を追加して divergence を解消した。
      - **配置先:** **`carry-forward`**。後続 phase で新 widget が
        catalog に入る度に「production root shape を 1 つ以上
        fixture parent として常時カバー」する方針を次 phase pre-doc
        input に明示。Phase 5 の reactive engine、Phase 6 の ZStack
        等で再判断する素材。

    - **制約 (3):** **non-root の Shrink container が Fill 子を
      持つ場合の挙動は M3 では未対処**。
      - **エビデンス:** Phase 4 fix は window-root のみを
        Fill/Fill に上書きする。非 root container (例: 入れ子の
        VStack inside Box inside VStack root, 中間の VStack が
        Shrink で Fill 子を持つ) は依然として
        `degenerate_fill_in_shrink_parent_clamps_to_zero` の
        convention で潰される。
      - **配置先:** **`carry-forward`**。Phase 4 範囲外。後続
        phase で non-root container hierarchy を扱う catalog 拡張
        が来た時に design 議論として再判断する候補。次 phase pre-
        doc input に「non-root Shrink + Fill 子の design space:
        現状 collapse、後続 phase で対処要否を決める」と明示。

11. **タスクリストの後続 step 見直し:** **不要**
    - 本 retro と同 commit で T6 行 6 件すべてを `[x]` flip 済み
      (smoke 結果 + fix bundle commit ハッシュ + 再 smoke 結果 +
      retro pointer + smoke evidence pointer を含めて反映)。
    - T7 (phase-end / Moment 2 re-sync) の checklist 内容は不変。
      ただし item 10 由来の `phase-sync` 制約 (1) が新規に T7
      Moment 2 sync 候補に積まれる (本 retro Follow-Up #2 参照)。
    - 本 step `[ ]` 残り未完 evidence なし (= 後続 step が所有する
      `[ ]` も無し)。`retrospectives.md` §step-end 固有 item 11
      の "未完 evidence の ownership lens" 要件は trivially 満足。

## Verification Notes

T6 で追加した test と、走らせた command を記録する。

新規テスト 2 件:

- `wasamo-runtime/src/layout.rs::tests::shrink_vstack_root_with_fill_scroll_view_child_collapses`
  (pure-logic、Fix 3(c))。gallery-shaped VStack root で Fill
  ScrollView 子が `child_h = 0` に潰されることを再現した上で、
  root を Fill height に pre-set すると非ゼロ allocation に
  反転することを assert。
- `wasamo-runtime/tests/scroll_view_layout_integration.rs::scroll_path_vstack_root_fixture_pins_window_root_fill_override`
  (mock-free runtime integration、Fix 3(d))。VStack-rooted gallery-
  shaped `.ui` を `wasamoc` で lower → `build_widget_tree` で実
  Compositor 配下に build → `run_layout_as_window_root(200, 200)`。
  (i) ScrollView outer Visual height > 0 at `scroll_y = 0`、
  (ii) `__set_i32_state_for_test("scroll_y", 100)` 後の re-layout
  で intermediate Visual Y offset が負、の 2 件を assert。

実行コマンド:

```text
cargo fmt --all                                     (post-impl fmt run)
cargo fmt --all -- --check                          (post-commit state; zero exit)
cargo build --release --workspace                   (green)
cargo build --workspace                             (debug; green)
cargo test --workspace                              (failure 0;
                                                     wasamo-runtime lib 258 passed (T5+1),
                                                     scroll_view_layout_integration 3 passed (T5+1),
                                                     wrap_panel_layout_integration 2 passed (= 元値で green 維持),
                                                     他 crate 全 green)
cargo build --release -p gallery-rust               (再 build for owner re-smoke)
Start-Process target/release/gallery-rust.exe -PassThru
                                                    (PID 3916, MainWindowTitle "Wasamo",
                                                     HasExited=$false after 3s; Stop-Process -Force OK)
```

Owner-manual GUI smoke (2026-05-25, re-run on `ed78d6c`-built
binary): 4 観察項目 + 2 reference 項目すべて green、window close
(Alt+F4 / ×) crash-free。smoke evidence:

- `docs/references/m3-phase-4/t6-gallery-smoke-scroll-y-0.png`
- `docs/references/m3-phase-4/t6-gallery-smoke-scroll-y-100.png`
- `docs/references/m3-phase-4/t6-gallery-smoke-scroll-y-800.png`
- `docs/references/m3-phase-4/t6-gallery-smoke-scroll-y-back-to-0.png`

(Phase 3 T9 retro の `t9-gallery-smoke-*.png` 規則に倣った命名。
Phase 3 が step-end で `docs/references/m3-phase-3/` に commit
していた precedent に沿って同じく step-end で commit。)

## Follow-Up

T6 から後続 task / 後続 phase への明示的な引き渡し:

1. **T7 (phase-end / Moment 2 re-sync) に直接効く Moment 2 sync 候補
   (本 retro item 10 由来):**
   - **`architecture.md §6` general layout** に 1 文追加。"the
     window-root WidgetNode is sized to the client rect regardless of
     its declared width/height constraints; this is enacted by
     `WidgetNode::run_layout_as_window_root` and called by `window.rs`'s
     `WM_SIZE` handler and `set_root` initial layout"。`phase-sync`
     分類なので T7 Moment 2 で fold する。

2. **T7 で他に効く Moment 2 sync 候補 (T5 retro から引き継ぎ済み、
   再掲):**
   - dsl_spec §4.9 例示文の `;` セパレータ表記を改行表記に揃えるか
     どうか (T5 副次学び #3 由来)。T7 で owner 判断。
   - `architecture.md §6.5` への "ScrollView child の
     `parent_abs_offset` shift" 明示追記の要否 (T5 close 時に T4
     retro Item 10 配置先を `doc-folded` → `phase-sync` に訂正済み)。
     T7 で owner 判断。
   - `architecture.md §6` general layout への item 10 制約 (1) の
     追記 (本 retro 由来、上記 #1)。

3. **後続 phase pre-doc input への carry-forward (本 retro item 10
   由来):**
   - **integration test fixture の parent shape を production `.ui`
     root shape にカバーさせる方針** (制約 2)。Phase 5 / Phase 6 の
     新 widget catalog 着手時に明示的に input として読む。
   - **non-root の Shrink container が Fill 子を持つ場合の挙動**
     (制約 3)。M3 では未対処、後続 phase で非 root container
     hierarchy 拡張時に design 判断。

4. **M4 handoff item:**
   - **`scroll_y` Signal drift**。Phase 4 では `arrange_scroll_view`
     が layout 時に `applied_offset_y` を clamp するのみで、Signal
     自体は drift する (owner smoke で「逆方向を 4 回押して初めて
     画面が動く」現象として顕在化)。M4 で **`in-out offset-y`
     write-back** が入れば Signal 側にも clamp 後値が書き戻されて
     drift は解消。Phase 4 ADR ([m3-phase-4-progress.md:18-22](../../plans/progress/m3-phase-4-progress.md#L18-L22))
     で明示的に M4 scope に外しているので Phase 4 内 fix にせず、
     本 retro / progress doc に「expected Phase 4 limitation;
     M4 handoff item」として記録するのみ。

5. **Window title wiring (T5 retro から引き継ぎ済み、依然 out-of-
   phase residual candidate):**
   - smoke 全 screenshot で MainWindowTitle が `"Wasamo"` のまま
     `.ui` の `title: "Gallery"` を反映していない件は依然未解決。
     T7 で Out-of-phase residuals に登録するか、別 phase の wiring
     に委ねるかを owner 判断。T6 では closing しない (visible
     correctness scope 外)。

直接 T7 作業に効くのは Follow-Up #1 (item 10 制約 1 の Moment 2
fold)、#2 の 3 件 (再掲 + 新規)、#5 の Out-of-phase residual 判断。
後続 phase へは Follow-Up #3、#4 が input となる。
