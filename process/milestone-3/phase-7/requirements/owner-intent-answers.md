---
title: M3-Phase 7 owner-intent answers — イテレーション grammar
status: draft (owner 回答 — 1b 書き戻し・蒸留の入力)
created: 2026-06-10
answers: owner-intent-questions.md
---

# M3-Phase 7 owner-intent answers

> **位置づけ.** owner-intent-questions.md の Q1–Q7 へのオーナー回答。
> §5 の手順で 1b へ confirm / revise を書き戻し、耐久部分を
> docs/notes/dsl-grammar.md へ蒸留したのち、framing 着手の入力とする。
> 本文書自体は SSOT ではない。

---

## 0. Owner-supplied prior（評価軸の重み付け — cross-cutting）

本回答および Phase 7 以降の DD 比較に適用する前提:

- **過去の合意は仮説である。** milestone の目的達成に資するなら、適切な
  手順（acceptance-revision exception、各文書の revisions 規律）で改訂できる。
- **比較の主軸は product merit**（実用性・thesis 整合）。実装・改訂コストは
  独立の軸ではなく tie-breaker。**却下する選択肢は merit で却下する**
  （コストで却下しない）。
- ただし「コスト非重視」は maximalism ではない。将来拡張性を理由とした
  過剰設計は引き続き失敗モード（段階 2 レビュー観点は生きている）。
- コストが判断に効く唯一の経路は「M3 の他 AC（特に Phase 8 / public
  draft）への schedule リスク」であり、それは DD の比較軸ではなく
  **framing §Risks** で扱う。

この §0 は framing の owner alignment packet に「評価軸の重み付け FD」と
して載せ、明示合意の対象とする。

---

## 1. Phase 7 thesis（回答の総括）

Phase 7 の初回 iteration surface は、Wasamo の構造的制御構文ファミリーを
`if` から `for` へ拡張する段階と位置づける。静的展開ではなく、**実行時
可変の collection binding が、生成される widget subtree の個数を駆動する**
ことを証明する（凍結 A8「collection binding drives widget-tree generation」
と無改訂で整合する回答である）。

keyed identity / retained state / data-driven reorder / structured item
fields は、Wasamo の将来方向として**肯定的に**残す。ただし初回 surface の
acceptance proof には含めない。初回 surface は un-keyed base
(fresh/positional — 規範文言の精密化は §3 注記 1 のとおり DD へ委譲)、
append/truncate-only、scalar item、flat scope で閉じ、deferred items は
条件ベースの activation trigger 付きで送る（§4）。

**deferral の理由は一貫して thesis-sequencing であり、コスト回避ではない**:
初回 surface の証明対象は「collection が cardinality を駆動する」ことで
ある。identity / reorder / fields / scale はそれぞれ別の thesis
（per-item 入力状態の保持、collection UX と ordering contract、型システム、
大規模 list 性能）に属し、それらが観測可能になる driver（per-item の
state・interaction、並べ替え UI、複合 field、大 N）と一緒に開く方が
良い設計判断ができる。

---

## 2. 回答表

選択肢の並び順・最小性に意味はない。理由はすべて merit / sequencing の
語彙で記す。

| 問い | 回答 | 理由（merit / sequencing） |
| --- | --- | --- |
| **Q1** collection は静的か実行時可変か | **実行時可変 collection + 最小変更経路** | 実用 UI の collection は実行時に変わる。静的展開は A8 / Phase 7 thesis の証明対象（cardinality 駆動）に届かない |
| **Q7** mutation 経路 | **最小 DSL handler 演算**（append / truncate 相当）。**mutation を起こす Button は `for` body の外に置く**（グローバルな add / remove。per-item handler は §4 の deferred 行で扱う） | `.ui` 内で自己完結する方が DSL thesis に合う。陽性対照（item 数の増減 2 frame）を authored 経路で直接示せる |
| **Q2** identity | 初回は **un-keyed base（fresh/positional）**。keyed identity は**明示 defer**（silent 乖離禁止 — §5 で 1b の期待を revise として書き戻す） | identity 意味論が author に観測可能になるのは item が state / interaction を持つ時で、その driver は M4 input 系で到来する。driver と一緒に開く方が retention の設計判断の質が上がる |
| **Q3** reorder | **append/truncate-only** | reorder は ordering contract + keyed diff を要する collection UX の thesis に属し、初回の証明対象とは別。sort / filter は将来必要（肯定的 defer、trigger は §4） |
| **Q4** scale | **小 N で機構を証明**。cap は fix-or-carry を明示記録（**§3 注記 3 の会計確定義務つき**） | 規模性能は LazyList 系（M5+）の別 thesis。初回の証明対象は cardinality 駆動であり、規模はその上に積む |
| **Q5a** item / index | 「全参照 state 経由」主義への**初の明文例外**として、**loop-local read-only binding** を認める | 例外は式（binding）位置に限る。handler 位置からの `item` 参照は別判断（§4 deferred 行） |
| **Q5b** item fields | **scalar-only**。structured fields は `TypedValue` と一体で defer | 要素型を ad hoc に生やさず、型システムの thesis（TypedValue）と一緒に育てる。plan が課す Phase 7 での明示判断義務は ADR で履行する（§4） |
| **Q6** scope | **flat**。`item` / `index` と state 名の collision は **error**。nesting は defer | 初回は平坦で十分。nested template scope は nested structural control flow（`else` / `switch` / bare nested）と同じ波で開く方が一貫した scope 規則を設計できる |

---

## 3. DD / slate への委譲・注記（本回答が決めないこと）

1. **Q2 の規範文言の選択は DD に委譲する。** 初回surfaceスコープ（stateless item +
   append/truncate-only）では fresh（変化毎 full rebuild）と
   positional-stable（append が先行 item を乱さない）は**観測上等価**。
   ただし dsl_spec に書く author-visible 規範文言は異なり、将来の keyed
   opt-in が「黙って変えてはいけない baseline」として参照する正文になる。
   DD はどちらを normative に書くか**明示的に**決めること（無自覚な
   既成事実化を禁ずる）。
2. **Q7 の演算形は Q5 一斉拡張規律と明示的に裁くこと。** collection 変更
   演算（既存 `+=` の collection overload か、新しい handler statement 形か）
   は「演算子は全 expr 位置で uniform に育てる」判例（E1 / Q5）と衝突しうる。
   collection 専用の演算ポケットを作るなら「これは expression ではなく
   handler statement である」という線引きを意識的に置き、ADR に記録する。
3. **cap の fix-or-carry は会計モデルの確定を前提とする。** Phase 7 DD は
   `MUTATION_CAP` の会計（drain 反復数か effect 数か、N item 一括
   materialise がどう数えられるか）を確定し、**Phase 7 自身の E2E 規模が
   cap に触れないことを示してから** carry を宣言すること。触れるなら
   carry は選べず fix が強制される。silent carry-forward は不可
   （constraints §6 の義務）。
4. 以下は elicitation 付録のとおり §2.2 slate で扱う（本回答は態度を
   決めない）: per-container × `for` 許容スイープ、DD-007 留保
   （member emission の canonize 可否）、body 形状（single vs range）、
   collection 式の語彙・初期値構文・`in` 予約、`ForLoopSubtree` slot
   bookkeeping、placement 格納モデル、architectural-family トリガー発火の
   confirm-or-strain 記録。

---

## 4. Deferred items（条件ベース activation trigger 付き）

このテーブルの**正本は Phase 7 framing（scope / out-of-scope 節）に置き**、
ADR forward-compat と implementation handoff へ流す。dsl-grammar.md には
スケジュール（責務先）を書かない（§5-2）。

| Deferred item | 責務を置く先 | activation trigger | 理由 |
| --- | --- | --- | --- |
| keyed identity / retained state | **M4 input / focus / TextField pre-doc で必ず再評価**。LazyList / 大規模 list は **M5 or later** | repeated subtree 内に focus / input / selected / user-editable state が入る。あるいは reorder を許す | retention が本当に痛むのは「同じ item の入力中状態・focus・selection を保持したい」時。M4 が input を開くので自然な再点火点。大規模 list 性能は VISION 上 LazyList が M5+ のため、そこに寄せる |
| data-driven reorder | **原則 M5 / collection UX DD**（Phase 7 で reorder を入れる判断をするなら即時） | sort / filter / drag reorder / user-authored order mutation / keyed diff を要求する UI | reorder は input stack ではなく collection UX / reconciler / ordering contract の問題。M4 に載せると焦点がぼやける |
| structured item fields / `TypedValue` | **Phase 7 ADR で明示判断**。不要なら trigger 付きで **M4 showcase spec または M5 widget/data-surface DD** へ carry | `item.filename`、caption fields、image metadata、record-like state、scalar で足りない concrete app case | plan は Phase 7 を `TypedValue` 圧力の最有力点と名指しし、explicit DD を要求。「不要」とする場合も trigger を残す |
| **per-item handler / handler 内 `item` 参照** | **Phase 7 ADR で admission を明示判断**（reject するなら trigger 付き defer） | select-this-item / delete-this-item 等の per-item interaction が要る UI（自然には M4 input で到来） | Q5a の例外は式（binding）位置のみ。`for` body 内の handler 持ち widget の admission と handler からの `item` 可読性は別判断で、spec が沈黙すると A12 の external-reader bar に響く |
| nested template scope / shadowing | **次に nested structural control flow を開く phase**（暫定 M4+ grammar residual） | nested `for`、`else` / `switch`、bare nested control flow、template-local named scope の必要 | dsl-grammar の再訪契機と同じ波。scope 規則は family 拡張と一緒に設計する方が一貫する |
| gallery-scale / cap / fan-out | **Phase 7 framing §Risks + reactive-drain residual carry**（§3 注記 3 の会計確定が前提）。大規模化は **M5+ LazyList / performance DD** | N item 生成が `MUTATION_CAP` に触れる（**触れるか否かの判定自体が Phase 7 の検証義務**）。visible list が数十〜数百 item を acceptance として要求する。CI / E2E で convergence failure | constraints §6 は fan-out × cap を Phase 7 直撃とし fix-or-carry の明示記録を義務化。silent carry-forward 不可 |
| item key と widget id の境界 | **Phase 7 ADR で確定的に記録**。widget id 自体は top-layer / anchor / concrete app case まで carry | `key:` 導入、top-layer anchor 参照、state 経由で表現力不足となる concrete case | dsl-grammar Q1「widget id と item key を混同しないこと」の規律の履行 |

---

## 5. 書き戻し・蒸留・sync の指示

1. **1b（期待）への書き戻し**:
   - 「Phase 7 = keyed identity を足す」→ **revise**（Phase 7 は un-keyed
     base の collection 一般化。keyed は §4 の条件 trigger 付き defer）。
   - 「`for` が ID-2 reconciler の first real driver」→ **revise**
     （first driver は M4 input 系の per-item state へ移る）。
   - 「data-driven reorder が ordering-contract driver」→ **confirm**
     （条件付き予測のまま維持。trigger を M5 / collection UX に精密化）。
   - 「M3 target は WrapPanel-backed thumbnail collections」→ **confirm**。
   - 「member emission を canonize しない留保」→ slate へ（態度は DD で）。
2. **dsl-grammar.md への蒸留 — 思想と条件は書く、スケジュールは書かない**:
   - **Q1 への追記**: loop-local read-only binding を「全参照 state 経由」
     主義への初の明文例外として記録する（handler 位置の可読性は未決の
     まま残す）。
   - **新 Q（または Q3 / Q6 追記）**: iteration thesis（collection binding
     が cardinality を駆動する。静的展開ではない）、un-keyed base が
     baseline で keyed / retention は declared-tree anchor 上の opt-in、
     reorder は ordering contract という別問題、要素型は `TypedValue` と
     一体で育てる。再訪契機は**条件ベース**で書く（§4 trigger 列の語彙）。
   - **責務先（M4 / M5 / Phase 7 ADR の割当）は書かない。** 正本は
     phase-7 framing に置き、dsl-grammar 側は「現在の割当は phase-7
     framing 参照」の 1 行に留める。計画は仮説であり、思想 note に
     埋め込むと計画改訂のたびに腐る。
3. **live 文書 sync（Phase 7 Moment 2）**: architecture.md §9 の
   「keyed item identity and state retention (the Phase 7 `for` driver …)」
   は本回答で stale になる。spec sync で「keyed は opt-in future、Phase 7
   は un-keyed base を collection へ一般化した」へ改訂する。dsl_spec 内の
   `for` 前方参照も同様に点検する。
4. **凍結記録は編集しない**: DD-M3-P6-004 / preamble の forward-compat は
   Phase 6 の凍結記録であり、本回答との乖離は「期待の revise」として
   Phase 7 framing の明文 + live 文書 sync（上記 3）で表現する。accepted
   ADR を遡って修正しに行かないこと。
5. **§0 の評価軸 prior** を framing の owner alignment packet に FD として
   載せ、明示合意の対象とする。
