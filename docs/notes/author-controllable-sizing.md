<!--
docs/notes/ は owner-authored の探索メモ／live open question 置き場（日本語可）。
本ファイルは規範スペックではない。確定したら decisions/ へ移し、ここからはリンクに置き換える。
-->

# ウィジェットの明示サイズ指定 — 未解決の future surface

**状態:** Open question（未割当）。実装フェーズ未定。
**最終更新:** 2026-06-24（M3-Phase 7b T5/T6b 発見を起点に起票）。

---

## 1. これは何か（一言で）

今の DSL には「このウィジェットを **幅 200 / 高さ 150 にする**」のような
**明示的なサイズ指定の書き方が無い**。ウィジェットの大きさは、部品の種類ごとの
**決め打ち（既定の制約）**で決まり、`.ui` の作者は上書きできない。

これは古くから認識されている「将来の surface」だが、**どのマイルストーンが実装を
担当するかも、いつ再訪するかのトリガーも、正本に記録されていない**。本メモは
その欠落を一箇所に集約し、再訪の足場にするための公式記録である。

---

## 2. 背景 — 部品のサイズはどう決まっているか

各レイアウト部品は、軸ごとに次のいずれかの「サイズ制約」を**種類固有の既定値**として持つ。
作者が `.ui` で変更する手段は無い。

| 制約 | 意味（たとえ） | これを既定に持つ部品（例） |
|---|---|---|
| **Fill** | 「親の割り当てを**いっぱいに埋める**」。自分からの希望サイズは持たない。 | Grid（両軸）、ZStack（両軸）、ScrollView（両軸）、VStack の幅、HStack の高さ |
| **Shrink** | 「**中身のぶんだけ**の大きさ」。中身が無ければ 0。 | Text 等のリーフ（両軸）、VStack の高さ、HStack の幅、WrapPanel の高さ |
| **Fixed(px)** | 「固定の px」。**現状、内部表現としては存在するが、author surface からは到達できない。** | （Grid のトラック幅 `100` など内部的な経路のみ） |

要点は2つ:

1. **Grid と ZStack は両軸 Fill**＝「親をいっぱいに埋める」型。
   「あなたの本来の大きさは？」と測られると **0 を返す**。
2. **多くの入れ物は片軸が Shrink**＝「中身の大きさを子に聞く」。

この2つが噛み合うと事故が起きる（次節）。

---

## 3. 表面化した症状 — ZStack 内の Grid が消える（Phase 7b 発見）

「中身ぴったり型（Shrink）」の入れ物の中に「ゴム風船型（Fill）」の Grid を入れると、
入れ物が Grid に大きさを聞き、Grid が 0 を返すため、**Grid が 0×0 になって消える**。

```
VStack {                 # 高さは Shrink（中身の高さを子に聞く）
    ZStack {             # Fill（高さを聞かれて 0 を返す）
        Grid {
            columns: 1*  rows: 1*
            Cell { Text { text: "中身" } }
        }
    }
}
# → VStack→ZStack→Grid と「0」が伝播し、縦が潰れて表示されない
```

**この症状は placement（`slot.h-align` 等の配置）の問題ではなく、サイズの問題である。**
配置（左/中央/右寄せ）が見た目に効くのは「中身ぴったり型」の部品だけで、Fill の Grid は
配置値に関わらず常に膨らむ（または 0 で潰れる）。詳しい切り分けと回避レシピは
[the manager-facing explainer in `private/`](../../private/m3-phase-7b-grid-in-zstack-explainer.md)
（非公開）にある。

Phase 7b では、Grid を ZStack の直下に `slot.*` 付きで書くこと自体が checker で
誤って弾かれていた問題（=「問題A」）を T6b で修正する。ただし**この消失（=「問題B」）は
サイズの話なので T6b では直さない**。「親に具体的な大きさがある」普通のケース
（全画面オーバーレイ等）は問題A の修正だけで正しく表示される。

---

## 4. これは新しい欠落ではない — Phase 2 からの defer 履歴

明示サイズ（`width` / `height`）は **M3-Phase 2 以降、繰り返し defer されてきた既知の
future surface** であり、設計は部分的に予約済みである。

- [DD-M3-P2-005 (aspect measure-arrange)](../../process/milestone-3/phase-2/decisions/dd-m3-p2-005-aspect-constraint-measure-arrange-algorithm.md):
  「明示 width/height があればそちらが優先（aspect は警告付き情報扱い）」という競合規則を
  **spec text として既に記述**。ただし「`width`/`height` は Phase 2 surface に無いので
  spec text のみ」「width/height が surface 化した時に適用」と明記。
- [Phase 3 constraints §12](../../process/milestone-3/phase-3/requirements/constraints.md):
  「Box の future-width/height rule を WrapPanel の item sizing と混ぜない」という規律。
- [DD-M3-P4-002 (viewport size source)](../../process/milestone-3/phase-4/decisions/dd-m3-p4-002-viewport-size-source.md):
  「author-controlled viewport sizing を要する future phase」に言及。

つまり「いつか surface 化する」前提は各所にあるのに、**deferred items の正本
（per-phase の framing 表）には載っておらず、責務先・トリガーが複数フェーズに散在**している。
Phase 7b の Grid 消失は、この古い欠落が初めて「部品が消える」という**可視症状**を出した例。

---

## 5. なぜ今まで発火しなかったか

1. どの in-repo `.ui` も「Fill の部品を Shrink 軸の中に置く」書き方をしていなかった。
2. Phase 7b 以前の checker は Grid-in-ZStack の配置を別形で扱い、そもそも author できなかった。

Phase 7b T5 で初めて、検証用の positive-control デモとして Grid-in-ZStack を意図的に
組もうとして顕在化した。**潜在していたが発火しなかった**というのが正確な経緯である。

---

## 6. 責務と締切

- **責務:** 現時点で **どのマイルストーンの acceptance criteria もこれを所有していない**
  （[roadmap](../../process/_roadmap.md) の M3–M6 / Post-1.0 を確認）。最寄りの受け皿は
  roadmap の defer 群（"generic modifier system" / "Layout algorithm changes" /
  将来の DSL ergonomics phase）で、いずれも未スケジュール。→ **自動では決まらない論点**。
- **締切:** `width`/`height` は `.ui → IR → runtime` を通るので **IR surface 波及は確実**。
  host の imperative 構築にも要るなら **C ABI 波及もあり得る**。M6 で ABI freeze + SemVer が
  効くため、ABI 波及があるなら **post-freeze は append-only でしか足せず、pre-1.0 が事実上の
  締切**になる（multi-window / DPI が ABI 波及ゆえ前倒しされた前例
  [DD-V-022 (DPI deferral)](../../process/cross-milestone/decisions/dpi-awareness-m4-deferral.md)
  と同型）。**この ABI 波及の有無評価**が、締切を hard（pre-1.0 必須）にするか
  post-1.0 可にするかを分ける鍵で、まだ未評価。

---

## 7. 想定する対応（記録としての方針）

1. **activation trigger:** 「concrete app/layout が kind-default 以外のサイズ
   （fixed か content-sized）を要求する、**または** M6 ABI-freeze 準備が始まる、の
   いずれか早い方」。後者を **hard backstop（遅くとも pre-1.0）**とする。
2. **公式な責務割り当て**は roadmap（SSOT）を触る**構造的変更**なので、
   `process/cross-milestone/decisions/` の **Vision DR** で行う（DD-V-022 と同パターン）。
   起票タイミングは **M3-Phase 8（public draft freeze）の framing 段階**が forcing function:
   Phase 8 は public draft 上の「予約（reservation）」を明示的に判断するフェーズなので、
   `width`/`height` を「予約済み future surface」として draft にどう書くかをそこで決める。
   ただし Vision DR は phase に属さない cross-milestone ガバナンスであり、確定は急がない。
3. **本メモ**は、その Vision DR の証拠ベースであり、確定までの live open question の home。

---

## 関連

- Phase 7b 実装ログ／candidate ledger（carry-forward 記録）:
  `process/milestone-3/phase-7b/implementation/log.md`
- レイアウトエンジンのその他 open questions:
  [layout-engine notes](./layout-engine.md)

## 改訂履歴

- 2026-06-24: 初版。M3-Phase 7b T5（Grid-in-ZStack 消失）/ T6b（checker 修正）を
  起点に、Phase 2 以降の `width`/`height` defer 履歴を集約し、責務・締切・対応方針を記録。
