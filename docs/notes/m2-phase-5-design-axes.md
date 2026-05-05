---
title: M2-Phase 5 (Reactive engine) — 設計の 2 軸
status: live
created: 2026-05-05
related-plans:
  - docs/plans/m2-plan.md
related-notes:
  - docs/notes/headless-verification.md
---

# M2-Phase 5 — 設計の 2 軸

Phase 5 (Reactive engine) は M2 で唯一の technical-thesis 検証点
(A2: host 配線なしで reactive 伝搬) であり、リスクが集中する。
pre-doc ADR を起草する前に、オーナーと方向感を擦り合わせておきたい
2 つの軸をここに置く。pre-doc 起草時の出発点として参照する。

## 軸 (a): Dependency tracker の設計深度

Reactive engine の中核は「property 変更 → 依存先 binding/UI を再評価」
の依存グラフ追跡。深度の選択肢:

### Option A — Minimum viable
- counter 1 つの binding が動けば良い
- グローバル dirty flag + 全 binding 再評価でも可
- **Pro**: Phase 5 を最短で閉じられる。M2 acceptance (A2) は通る
- **Con**: M3 で本格的な fine-grained tracking に書き直し確実

### Option B — 構造的 (fine-grained from the start)
- Signal/Computed/Effect の 3 層 (Solid.js / Vue ref 系)
- 依存は読み取り時に自動収集
- **Pro**: M3 以降の拡張に直結。書き直しコスト無し
- **Con**: Phase 5 のスコープが膨らむ。dependency cycle / disposal /
  glitch-free update など考慮事項が一気に増える

### 中間案
- Signal + Effect の 2 層のみ (Computed は M3 へ)
- 依存収集は手動 register ではなく、read 時の thread-local stack で自動

## 軸 (b): Headless verification への踏み込み

Reactive engine は pure logic 部分が大きいので unit test の余地が広い。
ただし「state 変更 → UI 再評価が呼ばれた」を確認するには、Compositor を
要求する WidgetNode に触れる必要があり、現状はテスト不能。

### Option A — Pure logic のみ test
- Signal 値の伝搬・依存収集・dirty 判定だけを test
- 「UI 側の再評価が呼ばれた」は GUI 手動 + integration で確認
- **Pro**: CLAUDE.md の testing rule にそのまま乗る
- **Con**: A2 (host 配線なしで reactive 伝搬) の検証が GUI 手動依存

### Option B — Mirror struct で WidgetNode 周辺も test
- M2-Phase 4 で確立した Slot/Children mirror パターンを Phase 5 にも適用
- Reactive 部分が WidgetNode 更新を呼ぶ箇所を mirror 経由で検証
- **Pro**: A2 の close 条件を unit test で satisfy できる可能性
- **Con**: Mirror が production 型と乖離するリスク

### Option C — Headless verification 環境を整える
- [headless-verification.md](./headless-verification.md) で議論中
- runtime を起こして state transition だけ観察する中間層を整備
- **Pro**: Phase 5 以降ずっと使える基盤になる
- **Con**: Phase 5 のスコープ外に膨らむ。別 ADR を先に切る必要

## オーナーへの問い

1. 軸 (a): minimum viable / 構造的 / 中間 — 腰を据える方向は?
2. 軸 (b): pure logic のみ / mirror 拡張 / headless 環境整備 — Phase 5 で
   どこまで踏むか?
3. これら以外に Phase 5 で先に決めておきたい論点はあるか?

回答は pre-doc 起草時に DD として展開する。本 note は資料置き場で、
ここで決定はしない。
