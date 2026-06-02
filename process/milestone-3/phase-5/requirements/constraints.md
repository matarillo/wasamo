---
title: M3-Phase 5 pre-doc inputs — M3-Phase 4 close からの前送り
status: live
created: 2026-05-25
source-phase: M3-Phase 4
target-phase: M3-Phase 5
---

# M3-Phase 5 pre-doc inputs

この note は、M3-Phase 4 (ScrollView minimal) close の学びを
M3-Phase 5 (Grid layout primitive) の pre-doc へ前送りするもの。単なる
retrospective ではなく、Phase 5 が Phase 4 の全 commit を読み直さなく
ても、ここから着手できるように action-oriented に書く。

入力源:

- [docs/plans/progress/m3-phase-4-progress.md](../../phase-4/implementation/log.md)
  の Decisions log 全体 (T4 fixture WrapPanel substitution / T5/T6
  split / T6 smoke failure mode A / T7 Q1/Q2/Q3 dispositions)。
- [t4-step-end-retrospective.md](../m3-phase-4/t4-step-end-retrospective.md) /
  [t5-step-end-retrospective.md](../m3-phase-4/t5-step-end-retrospective.md) /
  [t6-step-end-retrospective.md](../m3-phase-4/t6-step-end-retrospective.md) の
  Item 10 と Follow-Up 群。
- T7 progress doc Decisions log entry "T7 Moment 2 dispositions for
  follow-up bullets (2026-05-25)"。

## 1. integration test fixture parent shape は production root shape を必ずカバーする

T6 retrospective Item 10 (2) `carry-forward`。

Phase 4 T4 で landed した
`wasamo-runtime/tests/scroll_view_layout_integration.rs` の元 fixture
(`FIXTURE_SRC`) は ScrollView を component 直下 (Fill/Fill default) に
置く構成で、`gallery.ui` / `counter.ui` / `bool-demo.ui` が踏む
VStack-rooted production 経路を 1 件も pin できていなかった。T6 の
owner-manual GUI smoke で **failure mode A** (`scroll_y = 0` で
ScrollView 領域が完全に空、`+100/-100` で画面変化なし) として顕在
化し、`WidgetNode::run_layout_as_window_root` 新 entry point + 1 件の
runtime integration test 追加で resolve した
([t6-step-end-retrospective.md](../m3-phase-4/t6-step-end-retrospective.md))。

具体的な Phase 5 pre-doc 反映:

- Phase 5 Grid ADR の verification closure には、Grid root を VStack
  または直接 window-root に置く integration fixture を **必ず 1 件以上**
  含める。ScrollView と違って Grid 自身が `width / height` declared
  constraints を取りうるなら、両方の root shape (Grid-rooted +
  VStack(Grid)-rooted) をカバーするのが望ましい。
- 一般則: 後続 phase で新 widget が catalog に入る度に「production
  `.ui` で root に置かれる shape の少なくとも一つを integration test
  fixture parent として常時カバーする」方針を ADR verification closure
  にひと言で書く。これは pure-logic layout test では捕捉できない
  Visual-layer / runtime-boundary collapse を予防する gate。

## 2. non-root の Shrink container が Fill 子を持つ場合の挙動

T6 retrospective Item 10 (3) `carry-forward`。

T6 の window-root Fill/Fill 上書きは **window-root のみ**を対象とし、
非 root の Shrink container (例: 入れ子の VStack inside Box inside
VStack root) が Fill 子を持つ場合は既存 convention
(`degenerate_fill_in_shrink_parent_clamps_to_zero`) で潰れる挙動を
そのまま残した。Phase 4 範囲外。

具体的な Phase 5 pre-doc 反映:

- Grid が non-root の Shrink container 経路で使われる shape
  (`VStack { Grid {...} }` で外側 VStack が Shrink) を **design space
  として明示**し、以下の 3 択を Phase 5 ADR で判断する:
  1. 現状維持 (Shrink + Fill 子 collapse をそのまま継承)
  2. Grid 内部で Shrink-parent → Fill-子 をある範囲で許容する例外則
  3. Grid を含む全 layout primitive で non-root Fill-子 を許容する
     より広い convention 変更
- Phase 5 が (1) を選ぶ場合は ADR に 1 行残すだけで足りる。(2)(3) を
  選ぶ場合は `degenerate_fill_in_shrink_parent_clamps_to_zero` の
  semantic 変更を伴うため、別途 layout DD を起こすことになる。

## 3. M4 handoff: `scroll_y` Signal drift

T6 retrospective Follow-Up #4。**Phase 5 pre-doc では反映不要、M4
phase plan の input として残置**。

Phase 4 では `arrange_scroll_view` が layout 時に `applied_offset_y`
を clamp するだけで、`scroll_y` Signal 自体は drift する。owner smoke
で「逆方向を 4 回押して初めて画面が動き始める」現象として顕在化した。
M4 で `in-out offset-y` write-back が入れば Signal 側にも clamp 後値
が書き戻されて drift は解消する設計。

具体的な扱い:

- Phase 5 (Grid) は Signal-direction の変更を伴わない layout primitive
  なので、本項は **Phase 5 design に影響しない**。M4 phase pre-doc が
  着手される時点でここから input として読む (Phase 5 ADR に登らせる
  対象ではない)。
- ただし Phase 5 が万一 ScrollView と Grid の composition (Grid inside
  ScrollView、ScrollView inside Grid cell) を verification closure に
  含めるなら、その composition も drift 挙動を再現することを認識して
  おく。

## 4. R1 Window-title wiring の owning-phase 割当 — **Phase 5 pre-doc 内で必須完了**

T7 Q2 disposition (2026-05-25 owner-confirmed framing)。

Phase 4 close 時点で
[m3-phase-4-progress.md §Out-of-phase residuals](../../phase-4/implementation/handoff.md#out-of-phase-residuals)
に **R1 — Gallery host Window title wiring** が登録された。

- 観測: smoke 全 screenshot で `MainWindowTitle = "Wasamo"` (framework
  default) で `examples/gallery/gallery.ui` の `title: "Gallery"` を
  反映していない。現行 `.ui` lowering は component-level `title:`
  surface を保持するが、runtime/ABI host 経路は framework default の
  title で Window を生成している。
- **owner intent (2026-05-25):** `.ui` `title:` が実 native Window
  title を駆動しなければならない。これは **M3 residual** であり、M4
  theming/chrome handoff **ではない**。
- 解決条件: 「title attribute is declared unsupported」ではなく、
  「runtime/ABI host path applies component-level `title:` to the
  native window」。
- 期限: 遅くとも **M3-Phase 8 Gallery E2E close まで**に実装完了。

**Phase 5 pre-doc が完了するべき作業:**

1. R1 の owning phase を Phase 5 / Phase 6 / Phase 7 / Phase 8 のうち
   いずれかに **明示的に割り当て**、m3-plan.md の該当 phase 行 Notes
   に追記する (1 行)。
2. R1 を Phase 5 自身が owning する選択肢は **強くは推奨されない**。
   Phase 5 thesis (Grid layout primitive) と無関係であり、Phase 5 が
   抱え込むと thesis 集中が崩れる。**Phase 6 (ZStack + conditional
   rendering)** が natural candidate である理由は、lightbox UX の
   登場で Window-level metadata の visibility が高まるため。
3. 万一 Phase 5 pre-doc の時点で owning phase を確定できない場合
   (例: Phase 6 / Phase 7 / Phase 8 の thesis scope が pre-doc input
   段階で fix できていない)、**Phase 5 pre-doc では owning phase
   候補を 2-3 件に narrow** し、Phase 5 close 時点で再判断 → Phase 6
   pre-doc 着手前に確定する、という二段 gate を許容する。ただし
   Phase 8 close を超えて確定しないことは許容しない (Phase 8 close
   が implementation 期限のため)。
4. owning phase が Phase 5 / Phase 6 のいずれかに確定した時点で、
   Phase 4 progress doc の R1 entry に「owning phase: M3-Phase N」を
   追記 (R1 行を本 phase 内で再 commit するか、新 phase の progress
   doc 上で cross-reference するかは Phase 5 owner と相談)。

**Phase 5 pre-doc agenda checklist (R1 関連):**

- [ ] R1 の owning phase 候補を 1-2 件に narrow し、Phase 5 ADR の
      pre-doc framing に記録する。
- [ ] m3-plan.md の Phase 6 / Phase 7 / Phase 8 行のうち、R1 を
      assign した phase の Notes に "M3-Phase 4 R1 (Window title
      wiring) owning phase" 等の cross-reference を追記。
- [ ] Phase 5 ADR draft に「R1 は Phase 5 thesis scope **外**」と
      1 行明示し、Phase 5 自身が owning しないことを記録 (Phase 5
      が owning する判断になった場合のみ別途 DD を起こす)。

## 5. phase 最終 step の retrospective / progress checklist は step-end と phase-end を分割する

T7 retrospective Item 10 `carry-forward`。

T7 close で、progress checklist の "Phase-end retrospective recorded"
という単一 bullet が reviewer にとって危険な曖昧さを持つことが
分かった。retrospectives.md の checklist items 1-11 は step-end
retro (step → phase merge gate) で、items 12-18 は phase-end retro
(phase → main merge gate) であり、所有者もタイミングも異なる。
Phase 4 T7 では bullet を二段に分割し、`t7-step-end-retrospective.md`
は T7 が所有、`phase-end-retrospective.md` は `feat/m3-phase-4` へ
T7 merge 後の phase → main merge gate が所有する、と明示した。

具体的な Phase 5 pre-doc 反映:

- Phase 5 progress file の最終 step checklist は、最初から
  **step-end retrospective** と **phase-end retrospective** を別 bullet
  にする。step-end bullet は最終 step が `[x]` にできる。phase-end
  bullet は phase branch 上の phase → main merge gate が所有し、最終
  step close 時点では `[ ]` のままでよいことを明記する。
- Phase 5 が同じ構造で問題なく回った場合、Phase 5 phase-end retro で
  `retrospectives.md` 本文へ規範化するか判断する。Phase 4 close 時点
  では一段先送りし、過剰一般化を避ける。

## 前送り対象に含まれないもの (`doc-folded` 相当 / `local-only`)

以下は本 file には載せない (`doc-folded` または `local-only` 相当):

- **T6 fix bundle の実装詳細** (`WidgetNode::run_layout_as_window_root`
  の存在、`scroll_y` Signal drift 挙動など) — `architecture.md §6.3`
  と `§6.5` に T7 Moment 2 で fold 済み、または上記 §3 に M4 handoff
  として記述済み。Phase 5 ADR が `architecture.md` を read する前提で
  良い。
- **T4 retrospective Item 10 配置先訂正** (`doc-folded` → `phase-sync`
  への昇格) — T7 Moment 2 で `architecture.md §6.5` に fold 済み。
- **T5 副次学び #3** (`;` member separator) — T7 Moment 2 で
  `dsl_spec.md §4.9` notation note として fold 済み (post-Phase-4
  open question として明示)。Phase 5 grammar 変更を要するなら別途
  ADR が起こる。

これらは Phase 5 pre-doc が `architecture.md` / `dsl_spec.md` / Phase 4
ADR を読めば足りるので、本 file には pointer のみ置く。
