---
title: Reactive drain cascade policy — open question
status: superseded
created: 2026-05-06
superseded: 2026-05-07
superseded-by: process/milestone-2/phase-6/decisions/preamble.md#dd-m2-p6-001--drain-transaction-semantics
related-plans:
  - docs/plans/m2-plan.md (DD-M2-P5-004 = B)
related-decisions:
  - process/milestone-2/phase-5/decisions/preamble.md
  - process/milestone-2/phase-6/decisions/preamble.md (DD-M2-P6-001 = D, resolves this question)
---

> **Resolution (2026-05-07)**: DD-M2-P6-001 = Option D により本問は解決。observer は post-commit pure effect として再定義され、reactive Effect 起源の `set_property` から発生する observer 通知は同じ outermost cycle の Phase 3 で消化される (経路非対称性なし)。Phase 1 (mutation 収束) → Phase 2 (layout) → Phase 3 (post-commit observers, mutation 不可) の 3-phase + terminal 構造。詳細は ADR 参照。以下は問いが立った時点の記録として残置。


# Reactive drain cascade policy

## 問い

`drain_if_outermost` の 3 段ドレイン（observer → reactive → layout）において、
reactive Effect 起源の `set_property` 呼び出しが `enqueue_property_change` で
observer queue に積んだ通知を、**同じ outermost cycle 内で消化するか、
次の cycle に持ち越すか**。

## 背景

DD-M2-P5-004 = B は「observer drain → reactive drain → layout drain」という
3 段の直列を決定している。しかし reactive drain（dirty Effect の flush）が
`set_property` を呼ぶと、そこから `enqueue_property_change` が走り observer
queue に新たなエントリが積まれる可能性がある。このフィードバックパスについて
ADR は明文化していない。

## 2 案

### 案 1 — 同 cycle 完全消化（ループ）

「observer queue が空 ∧ dirty Effects が空 ∧ layout dirty が空」が
同時に成立するまで 3 段をループで回す。

- **Pro**: reactive 起源の observer 通知が同じ frame で消化される。UI の
  一貫性が高い。
- **Con**: ループの停止条件が複雑になる。observer → reactive → observer
  のフィードバックサイクルが存在する場合、iteration cap が必要になる（
  reactive 内の cap とは別に）。DD 追加に近いスコープ拡大。

### 案 2 — 単純直列（1 回のみ）

「observer → reactive → layout」を 1 回だけ直列で回す。reactive 起源の
observer 通知は **次の outermost cycle**（次の ABI call 末尾または
`wasamo_run` の次の dispatch 後）で消化される。

- **Pro**: 実装が単純。DD-M2-P5-004 = B の記述に忠実。
- **Con**: reactive Effect が書き換えた property の observer 通知が 1 frame
  遅延する。M2 acceptance シナリオ（counter + 単一 binding）でこの遅延が
  問題になるかは **未確認**（明示的な合意なし）。

## 現状の暫定実装

DD-M2-P5-004 実装（wip/step）では **案 2** を採用する。

理由: ADR が明文化した形に忠実であること、M2 acceptance シナリオが
1-frame 遅延を問題にするかどうかは Phase 6 実装前に判断できないこと、
および ADR が「最も likely な DD 追加源は DD-M2-P5-007/008」と
見込んでいることから、今は単純な実装を選び、問題が顕在化したら
その時点で refinement DD を追加する方針とする。

## オーナーへの問い

- M2 acceptance シナリオ（counter + 単一 binding の end-to-end）において、
  1-frame 遅延が許容されるか？
- 許容されない場合、案 1 的なループを Phase 5 内で導入するか、
  あるいは Phase 6 / DD-M2-P5-007 として扱うか？
- 結論が出たら本ノートを更新し、必要であれば DD を追記する。

## 解決方針 (2026-05-06)

- M2 acceptance では 1-frame 遅延は観測されない (counter シナリオでは 1 イテレーションで収束)。
  Phase 5 close は現在の暫定実装 (案 2 = 直列 1 回) のまま実施する。
- 本問題は「案 1 vs 案 2」の二択ではなく **drain pipeline の設計軸そのもの** に関する
  問題であることが判明 (経路非対称性、observer の意味論、収束レイヤ分離 など)。
  これを反映したドラフト DD を [dd-m2-p6-drain-transaction.md](../retrospectives/dd-m2-p6-drain-transaction.md)
  に作成済。Phase 6 pre-doc サイクルで正式採用予定 (DD 番号は Phase 6 ADR 起草時に確定)。
- 採択候補は 6 option (A〜F)、起草者推奨は **Option D**
  (declarative transaction + post-commit pure observer)。本ノートはオーナー判断後に
  最終 status を更新する。
