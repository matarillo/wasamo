---
title: M3-Phase 5 phase-end 振り返り
status: recorded
created: 2026-05-30
scope: phase-end
phase: M3-Phase 5 — Grid layout primitive
---

# M3-Phase 5 phase-end 振り返り

## 対象範囲

M3-Phase 5 は Grid 2D layout primitive (1 cell 1 child / fixed +
weighted-star track sizing / row-column spanning) を shipped とし、M3
acceptance criterion **A2** と **A11** の Phase 5 owner-acceptance slice
を close した。phase 内訳:

- `wasamoc` の Grid / Cell surface・narrow track-list parser・diagnostics
  (T1)
- pure-data layout engine の Grid measure-arrange (fixed-first +
  weighted-star resolution / spanning / per-Cell alignment /
  document-order z-order) (T2)
- runtime IR loader materialisation + `validate()` defense-in-depth と
  Cell flatten (T3)
- Windows-runtime Visual tree evidence (Grid outer-bounds `InsetClip` +
  Cell content offset / production root shape) (T4)
- gallery `.ui` Grid slice 追加と assistant build / launch + screenshot
  baseline (T5)
- owner-manual visible smoke (T6; リサイズ陽性対照で star 柔軟性と
  outer-bounds clip を確認)
- Moment 2 spec / architecture / plan re-sync と T7 step-end retro (T7)

phase-end (本 retro) は T7 を phase ブランチに merge した後に、別 commit
で items 12-18 を記録する ([retrospectives.md §phase 最終 step の
retrospective 分割](../../../procedures/retrospectives.md))。

## 主な学び

一つ目は、**ADR-local / local-only として始まった検証制約が、実は
project-wide に再利用される規範であることがあり、その正しい close 先は
『最も近い手順書』ではなく『ジャンル / アクターで一致する常時ロード
SSOT』への doc-fold である**こと。#2 (assistant-visible GUI evidence)
と #4 (陽性対照原理) は、一見 `docs/notes/human-visible-smoke.md` (owner
の目視手順書) が近いが、これらは **assistant の証拠生成への指示**であり
アクターが違う。ゼロベースで genre を詰めた結果、規範核を
[CLAUDE.md §Testing rules](../../../../CLAUDE.md) (= Claude への指示の
SSOT、常時ロード) に、操作詳細を
[verification-environments.md](../../../../docs/notes/verification-environments.md)
Obs 4 に fold した。「近いから畳む」ではなく「誰宛のどの規範か」で
fold 先を決める。

二つ目は、**T0 で凍結した task list は、phase 途中の owner 決定で所有が
動いた項目について stale な ownership を持ちうる**こと (T7 Main Learning
の phase-end への持ち上げ)。最終 step の close では凍結 list を
rubber-stamp せず mid-phase owner 決定と cross-check し、mutable phase
plan を workaround せず revise する (plan revise A)。

三つ目は、**最終 step の step-end / phase-end retrospective を最初から
別 bullet・別ファイル・別 commit に分割する運用が有効だった**こと。
Phase 4 で事後検出した曖昧さ (所有者 / タイミング) を Phase 5 は最初から
分割して回避でき、reviewer friction が下がった。この lived experience を
根拠に、本 phase-end で constraints §5 を
[retrospectives.md §phase 最終 step の retrospective 分割](../../../procedures/retrospectives.md)
へ **project-wide rule として昇格させる判断を確定** (commit `09b6273`)。

## チェックリスト

12. **Acceptance criteria (Ax) 達成確認:** **達成**
    - **A2** は T1-T6 により discharged。ADR
      [§Phase 5 verification closure](../decisions/preamble.md) の
      automated / CI-gated evidence items (1)-(4) が landed:
      (1) `wasamoc check` surface + diagnostics (T1) + gallery 正例
      (T5); (2) measure-arrange pure-logic (T2); (3) IR loader /
      `validate()` defense-in-depth (T3); (4) Windows-runtime Grid
      Visual evidence (T4)。item (5) は gallery slice (T5) +
      owner-manual smoke (T6) で discharged。
    - **A11** の Phase 5 slice は T7 Moment 2 re-sync + T6
      owner-acceptance で discharged。`docs/dsl_spec.md` §4.12 は
      `M3-Phase 5 closed; implementation-synced`、`architecture.md` は
      M3-Phase 5 complete。
    - ADR の "discharged" 表記と実装の乖離は **なし**。dsl_spec v1.4
      revision history が "No §4.12 design / implementation divergence
      found during the close re-sync" を記録。観測点 #6 (document-order
      paint) は可視 fixture では非観測だが、layout-side substrate +
      Visual-tree insertion-order を根拠に許容判断として記録済み
      (T7 retro)。

13. **`CHANGELOG.md` / `process/_roadmap.md` 整合:** **整合**
    - [`CHANGELOG.md`](../../../../CHANGELOG.md) Unreleased に
      `### M3-Phase 5 — Grid layout primitive (2026-05-30)` entry を
      追加。A2 / A11 slice discharge、generic IR (carrier c1) /
      no-new-ABI、wasamoc surface、runtime measure-arrange + loader +
      Visual、gallery 可視証跡 + T6 owner smoke、Moment 2 spec sync、
      DPI residual → M4、R1 → Phase 6 を要約。
    - [`process/_roadmap.md`](../../../_roadmap.md) は A2 記述が安定で
      変更不要。M4 の per-monitor DPI AC は本 phase の DPI governance
      commit (`2162867`、DD-V-022) で既に landed。

14. **`README.md` / thesis-level claim への影響:** **あり (独立
    governance commit で処理済み)**
    - T6 owner smoke が、`README.md` の "high-DPI composition ... out
      of the box" が現状 (runtime は M1 から per-monitor DPI 非対応) と
      ずれる readability gap を surface。これは Phase 5 が実装した内容
      ではない cross-milestone ガバナンスのため、**Moment-2 sync には
      束ねず独立 commit** (`2162867`) として処理:
      [DD-V-022 / DD-V-023](../../../cross-milestone/decisions/dpi-awareness-m4-deferral.md)
      を起票し、`_roadmap.md` M4 に per-monitor DPI awareness AC を追加、
      `README.md` に vision-state note を追加。Phase 5 ADR (Grid) には
      影響せず (DPI は Grid design に直交)。
    - process rule の structural 変更 (retrospectives.md への §5 昇格)
      も同様に独立 commit (`09b6273`) で処理 (上記 主な学び 三つ目)。

15. **次 phase の framing への送り込み材料:** **整理済み (open な
    phase-sync 残ゼロ)**
    - task retro 項目 10 の制約を phase-end で以下に **全て close**:

      | 制約 | 出所 | disposition |
      |------|------|-------------|
      | Grid carrier-c1 textual IR §8 grammar | T1/T3/T7 | `doc-folded` (dsl_spec §8.5) |
      | assistant-visible GUI evidence standard | T5/T7 | `doc-folded` (CLAUDE.md + verif-env Obs 4) |
      | 陽性対照 discipline | T6 | `doc-folded` (CLAUDE.md + verif-env Obs 4) |
      | per-monitor DPI 非対応 | T6/T7 | `carry-forward` → M4 |
      | R1 Window-title wiring | (Phase 4 継続) | out-of-phase residual → Phase 6 |
      | T2 parallel-vector / T1 R-C / T5 no-comment 他 | — | `local-only` |

    - `carry-forward` (DPI) と out-of-phase residual (R1) は
      [`implementation/handoff.md`](../implementation/handoff.md) に
      §6.3 清書済み。`doc-folded` は本文転記せず pointer のみ
      (handoff の Pointers 節)。
    - **ADR cross-ref (phase-sync ADR-touch case 2):** DPI residual は
      handoff + VDR で forward 済み。Grid ADR ([preamble.md](../decisions/preamble.md))
      への cross-ref は **不発火** (DPI は Grid design に直交し、VDR +
      roadmap M4 AC + handoff で一意な owner を既に持つため、Grid ADR
      読者には noise; owner 確認 2026-05-30)。ADR set は Moment 1
      Accepted のまま。

16. **CI green 確認:** **green (phase ブランチ実 CI 確認済み)**
    - ローカル clean rebuild は T7 working tree で green (proxy):
      `cargo fmt --all -- --check` exit 0 → `cargo clean` → debug +
      release `--workspace` build green → `cargo test --workspace`
      **627 passed / 0 failed** (`grid_layout_integration` 2 を含む)。
      phase-end の doc 追加 (CHANGELOG / handoff / 本 retro / CLAUDE.md /
      verif-env) は全て非コードのため build / test state 不変。
    - **実 CI (item 16 必須 gate):** phase ブランチ `feat/m3-phase-5`
      (headSha `ca711bd`) で `workflow_dispatch` から CI を実行し
      **GitHub Actions green を merge 前に確認済み**:
      [run `26683352589`](https://github.com/matarillo/wasamo/actions/runs/26683352589)
      conclusion **success** (2026-05-30 12:04→12:07 UTC, ~2m31s)。
      `cargo build` job の build / test / C ABI / CMake / Zig / counter
      smoke steps 全 green (annotation のみ: `mlugg/setup-zig@v2` の
      Node.js deprecation、`windows-latest` → `windows-2025-vs2026`
      redirect notice)。**push はオーナー明示承認後のみ**; push 後は
      main 上で CI green を再確認 (push トリガで自動実行)。
    - CI YAML 変更不要 (新言語 / 新ビルド系なし)。

17. **human-visible GUI smoke:** **必要 → phase-specific gallery smoke
    で代替実施 (明示的例外判断)**
    - retrospectives.md item 17 の既定は
      [human-visible-smoke.md](../../../../docs/notes/human-visible-smoke.md)
      の `counter-c/rust/zig` 確認だが、Phase 5 の可視対象は **Grid**
      であり counter 系は Grid を使わない。よって本 phase は既定の
      counter smoke ではなく、**phase-specific な `gallery-rust` owner
      smoke (T6) をもって human-visible smoke の達成とする明示的例外
      判断**を取る:
      - T6 owner-manual GUI smoke を rebuilt `gallery-rust.exe` で実施、
        spanning header / footer、3 つの star-sized 中段列 (`C2` ≈ 2×
        `C1`、リサイズ陽性対照で維持)、outer-bounds clip (footer 4:1
        box が thin strip に clip され Photos に bleed しない) を owner
        が accept。
      - 既定の `counter-c/rust/zig` smoke は **追加不要**: counter 系は
        Grid 非使用で本 phase の可視挙動変化を含まず、非 Grid path 不変は
        CI の counter build / smoke steps green (run `26683352589`) +
        627 test green が裏付ける。
    - phase-end 以降は doc-only で新規 GUI surface 無し。

18. **CI YAML 変更要否 sanity check:** **不要**
    - Phase 5 は新言語 / 外部ビルド系 / CI matrix 次元を追加しない。
      既存 Windows CI `cargo test --workspace` が Grid の
      Windows-runtime integration test を covers。

## 検証メモ

phase-end working tree (doc 追加のみ) の build / test state は T7 close
時と同一 (627 passed / 0 failed)。実 CI を phase ブランチ
`workflow_dispatch` から実行し green を確認済み (item 16 merge gate):

```text
git push origin feat/m3-phase-5          (0feb9d4..ca711bd)
gh workflow run ci.yml -r feat/m3-phase-5
gh run watch 26683352589 --exit-status
  -> https://github.com/matarillo/wasamo/actions/runs/26683352589
  -> conclusion success; ~2m31s; all build / test / C ABI / CMake /
     Zig / counter smoke steps green
```

実 CI は code state (headSha `ca711bd`) を gate する。これより後の
phase-end doc commit (CHANGELOG / 本 retro の run id 反映 / plan.md
flips / log.md) は全て doc-only で build / test state を変えないため、
本 CI green が merge gate の有効な ground truth。

## フォローアップ

- **merge gate:** 本 retro checklist 完了 + 実 CI green (item 16) を
  オーナーに報告 → オーナー明示承認後に `feat/m3-phase-5` を main へ
  no-ff merge → オーナー明示承認後に push → main CI green 再確認。
- **plan.md close:** phase-end-owned bullets (cargo gates / Windows
  integration evidence on CI / phase-sync close / log.md 清書 /
  handoff.md / front-matter `active` → `closing` / 本 phase-end retro
  recorded) を merge gate で flip。`closing` → `retired` は
  phase → main merge commit / post-merge distillation の所有物で本
  retro commit では実行しない。
- **Phase 6:** R1 (Window-title wiring) は Phase 6 所管 (m3-plan Phase 6
  行 Notes 既載)。DPI は M4 (handoff + VDR)。次 phase pre-doc は
  handoff.md を input に読む。
