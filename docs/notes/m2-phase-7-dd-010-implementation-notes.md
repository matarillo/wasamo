---
title: M2-Phase 7 / DD-M2-P6-010 実装ノート
status: working
created: 2026-05-09
---

# M2-Phase 7 / DD-M2-P6-010 実装ノート

このノートは、DD-M2-P6-010 の実装ステップで使う作業仮説と検証ログを置く場所である。
正式な決定は ADR に、進捗状態は phase progress に、M3 へ渡す確定済みの残余は
`docs/notes/m2-to-m3-handover.md` に蒸留する。

対象は `dirty_effects` の `EffectId` 数値順近似を、真の dependency graph walk に置き換える実装である。
ADR は Option A を採用済みで、実装は次の形を満たす必要がある。

- Kahn-style walk を `wasamo-runtime/src/reactive.rs` の free function として抽出する。
- 入力は `ReactiveGraph::forward` / `ReactiveGraph::back` と dirty set、またはそれと同等の borrow に限定する。
- `drain_dirty_effects()` は抽出した walk を単一の production path として呼ぶ。
- 既存の `sort_unstable()` は fast path として残さない。
- release/debug の挙動差を作らない。
- chain / diamond / fan-out × `MUTATION_CAP` / out-of-ID-order の synthetic graph unit tests を実装受け入れ条件にする。

## 現在の作業仮説

- H1: `dirty_effects` の並び替え対象は、各 drain iteration で `DIRTY_EFFECTS` から drain された Effect set に限定する。
- H2: topological walk は、dirty set に含まれる Effect 間の dependency order だけを決める。dirty set 外の Effect はこの iteration の実行対象ではない。
- H3: topo 上の tie は M3 residual として契約化しない。ただしテストと診断の安定性のため、実装内では deterministic な候補順を持たせる。
- H4: cycle policy は DD-010 の M2 実装では未決定であり、M3 pre-doc へ渡された残余として扱う。M2 unit tests は acyclic graph を対象にする。

## 重要な未検証点

- Q1: 現行の `forward: SignalId -> EffectId set` と `back: EffectId -> SignalId set` だけで、Effect 間の topo edge を正しく導出できるか。
- Q2: topo edge が本来 `Effect A writes Signal X -> Effect B reads Signal X` を意味するなら、現在の graph は「Effect が読む Signal」しか保持していないのではないか。
- Q3: ADR の実装条件である `&forward` / `&back` / dirty set からの walk と、実際に必要な Effect-write 情報の間にギャップがある場合、それは実装内の調整で済むのか、pre-doc へ戻すべき逸脱なのか。
- Q4: divergence diagnostics の `offending_effect_id` は topo walk 後の先頭 Effect でよいか。既存の `sort_unstable()` 前提から診断の意味が変わらないか。

## 実装セッションの初手

1. Synthetic graph test helper を先に作り、`forward` / `back` / dirty set だけで chain と out-of-ID-order の意図を表現できるか確認する。
2. 表現できる場合は、その helper を使って mandatory tests を先に追加し、Kahn-style walk を実装する。
3. 表現できない場合は、`writes` map 追加などに進む前に、ADR の Required form とのズレをここへ記録し、設計確認に戻す。

## 検証ログ

- 2026-05-09: 実装前の作業仮説置き場として本ノートを新設。最初のリスクは、現行 `forward` / `back` だけで Effect-write-to-Effect-read edge を導出できるかどうか。

## 実装中の決定

未記録。

## 蒸留先

- 実装が ADR どおり進んだ場合: `docs/plans/progress/m2-phase-7-progress.md` の DD-M2-P6-010 実装チェックを更新する。
- 実装中に ADR 条件からの逸脱が必要になった場合: `docs/decisions/m2-phase-7-reactive-foundation.md` の DD-010 へ戻すか、新しい pre-doc cycle を開く。
- M3 に渡す確定済み残余が増えた場合: `docs/notes/m2-to-m3-handover.md` へ蒸留する。
