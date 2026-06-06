---
title: 設計判断（DD/ADR）レビューのすすめかた — 3段階レビューのガイドラインとプロンプトテンプレート
status: guideline
created: 2026-05-31
---

# 設計判断（DD/ADR）レビューのすすめかた

本ドキュメントは、[workflow.md §3 設計判断](./workflow.md) で書いた DD/ADR を
**Accepted に進める前に複数視点でレビューする**ときの、おすすめ手順とプロンプト
テンプレート集である。

> **位置づけ — これは強制ゲートではない。**
> 本ドキュメントは **guideline**（おすすめ手順）であり、守るかどうかは自由。
> 実際のゲートは別にある：
> - DD/ADR の確定は **owner の明示 accept**（[workflow.md §3](./workflow.md)）。
> - merge は **retrospective + owner 明示承認**（[retrospectives.md](retrospectives.md)）。
> - commit 形は [AGENTS.md §Commit rules](../../AGENTS.md#commit-rules)。
>
> 本手順はこれらのゲートを**置き換えない**。owner が accept 判断を下すための
> **input（より良く練れた DD と、論点の見える化）を整える**ための対話補助である。
> したがって本ドキュメントは process-rule の SSOT ではなく、設計判断段階の SSOT
> である [workflow.md §3](./workflow.md) に従属する。新たな強制ルールを足したく
> なった場合は、ここを編集するのではなく
> [AGENTS.md §Process rule lifecycle](../../AGENTS.md#process-rule-lifecycle) の
> 構造的変更フロー（vision decision record）に乗せること。

主な対象読者は **AI エージェント**だが、reviewer も reviewee も AI エージェントに
なりうることを前提にしている（→ [§AI 同士で回すときの注意](#ai-同士で回すときの注意)）。

origin: M3-Phase 6（ZStack + conditional rendering）の DD slate を Accepted に
進める前に実施した複数パスのレビュー（strategic コア → strategic 全体 →
recommendation-choice → implementation-readiness）。本ドキュメントはそれを
**3 段階**に整理し、再利用可能なテンプレートにしたものである。

---

## なぜ 3 段階か

DD レビューを 1 回の「いい感じに見て」で済ませると、3 種類の異なる失敗が混ざって
どれも十分に拾えない。3 段階は **広げる → 選ぶ → 落とす** という別々の関心事に
レビューを分割する。各段階は「防ぎたい失敗」が異なる。

| 段階 | 関心事 | 防ぎたい失敗 |
|---|---|---|
| 1. strategic design / owner-alignment | **広げる** | 既存実装に早く収束し、owner が判断すべき設計空間を閉じすぎる |
| 2. recommendation-choice | **選ぶ** | 個々の DD は妥当でも、推奨案が全体で噛み合わない／比較軸が誘導的／trade-off が暗黙 |
| 3. implementation-readiness | **落とす** | 設計は良いが実装者が迷う（境界・順序・evidence・診断・doc-sync が未定義） |

### 順序が効く理由

各段階は**前段の収束を前提**にする。

- 段階 1 が設計空間を動かしている（Options が増減する）うちに段階 3 を走らせても、
  落とし込みの議論はやり直しになる。
- 段階 2 で推奨が差し替わると、段階 3 で詰めた task 分解・evidence は前提が崩れる。

だから **1 → 2 → 3 の順で、前段が落ち着いてから次段**へ進むのが推奨。これは
[「DD 確定を急がない」原則](#dd-確定を急がない)の運用形でもある。ただし強制では
ない：小さな DD なら段階を畳んでよいし、逆に大きな DD は **1 段階を複数パスに
割ってよい**（M3-Phase 6 は段階 1 を「コア DD 先行 → 全 DD」の 2 パスに割った）。
畳む／割る判断をしたら、なぜそうしたかを 1 行残すと後から読める。

---

## 共通の約束事

3 段階すべてに共通する前提・出力・改訂規律。各段階のテンプレートはこれを内包する
形で書いてあるので、テンプレ単体をコピーして使える。

### 前提資料（context pack）

reviewer に渡す（reviewer が AI エージェントなら、これを読める状態で起動する）：

- [workflow.md](./workflow.md)（段階の意味と DD の構造）
- 対象 phase の `requirements/framing.md` / `requirements/constraints.md`
- 関連する `docs/notes/` / `docs/*_spec.md` / 先行 DD / `handoff.md`
- `decisions/preamble.md`（phase 全体の Context / Out of scope / Inputs absorbed）
- **対象 DD**（`dd-*.md`）本体

### 出力形式（findings + verdict）

- **重大 findings 優先**。各 finding に次の 3 つを含める：
  - **問題** — 何が・どこで
  - **影響** — owner 判断（段階 1/2）または実装（段階 3）にどう効くか
  - **修正方向** — どう直すか（断定せず方向を示す）
- 末尾に**段階別の verdict**（各段階の節を参照）。
- 重大 finding が無い場合はその旨を明示し、次段／owner に渡してよい状態かを短く評価
  する（reviewer が勝手に「合意」しない）。

### reviewee 側の改訂規律

指摘を受けて DD を直すときは：

- **Revision history は初版との差分を簡潔に**示す。DD はレビュー中で、合意形成の
  ために何度も更新するが、**Revision history への追加は 1 行に集約**する（レビュー
  1 パスにつき 1 行が目安。各 finding ごとに行を増やさない）。
- finding は **fold（取り込む）/ reject（採らない）/ defer（先送り）** に分類し、
  reject・defer にも 1 行理由を残す（黙って無視しない）。
- 取り込んだ結果が他の DD・preamble の語彙と矛盾しないか相互参照を確認する
  （特に段階 2 はクロス DD 整合が主眼）。

### DD 確定を急がない

レビュー中の DD は **`Status: Proposed` を維持**する。reviewer/reviewee が AI
エージェントどうしで「直し切った」と判断しても、それは Accepted ではない。
全段階を走らせ、**owner の明示 accept** を待ってから Accepted-flip・設計同期
（Moment 1）・§4 実装計画へ進む（[workflow.md §3 / §3.1](./workflow.md)）。

---

## 段階 1: strategic design / owner-alignment review

**狙う失敗：** 既存実装に乗る案へ早く収束し、owner が判断すべき設計空間を閉じすぎる。

**関心事：** 広げる。owner の意図・Wasamo の設計思想・将来拡張性に対して、DD が
小さく閉じすぎていないか。implementation-readiness は見ない。

**観点：**

- **Options** が既存実装に引っ張られすぎず、owner が判断できる選択肢空間を開けて
  いるか。必要なら IR / runtime / grammar / reactive architecture を変える案も
  公平に並んでいるか。
- **Recommendation** が「実装が小さいから」ではなく「設計思想に合うから」選ばれて
  いるか。
- **Out of scope / deferred** が本当に妥当か（oversight による黙殺ではないか）。
- 既存実装に乗る案へ**早く収束しすぎていない**か。

### reviewer プロンプト

```text
このフェーズの DD を strategic design / owner-alignment review してください。

implementation-readiness ではなく、owner の意図・Wasamo の設計思想・将来拡張性に
対して、DD が小さく閉じすぎていないかを見てください。

特に、
- Options が既存実装に引っ張られすぎず owner が判断できる選択肢空間を開けているか
- Recommendation が「実装が小さいから」ではなく「設計思想に合うから」選ばれているか
- Out of scope / deferred が本当に妥当か
をレビューしてください。既存実装に乗る案へ早く収束しすぎていないか、必要なら
IR / runtime / grammar / reactive architecture を変える案も公平に比較されているかを
見てください。

前提資料: workflow.md、phase framing、constraints、関連 notes/spec/prior DD/handoff、
decisions preamble、対象 DD。

出力は重大 findings 優先で、各 finding に【問題 / owner 判断への影響 / 修正方向】を
含めてください。最後に「設計空間は十分に開かれているか（十分 / 要拡張）」を短く
判定してください。

（任意の重点）今回は特に ___ を重点的に見てください。
  例: Options derivation / Out of scope・deferred の妥当性 / Recommendation の根拠
```

### reviewee プロンプト

```text
このフェーズの DD を strategic design / owner-alignment review しました。
owner の意図・Wasamo の設計思想・将来拡張性に対して、DD が小さく閉じすぎていないかを
見ています（implementation-readiness ではありません）。

指摘を受けてファイルを修正する場合は、finding を fold / reject / defer に分類し、
reject・defer には 1 行理由を残してください。Revision history では初版との差分を
簡潔に示すこと。DD はレビュー中であり合意形成のため何度も更新しますが、
Revision history への追加は 1 行に集約します。Status は Proposed のまま維持。

指摘は以下です。
...
```

**推奨 verdict：** 設計空間は **十分に開かれている / 要拡張（既存実装に閉じすぎ）**。

---

## 段階 2: recommendation-choice review

**狙う失敗：** 個々の DD は妥当でも、推奨案が全体で噛み合わない／比較軸が特定案へ
誘導的／owner が受け入れる trade-off が暗黙。

**関心事：** 選ぶ。段階 1 で広げた選択肢空間から、「この選択で進むべきか」を owner
判断として確認する。implementation-readiness ではない。**前提として段階 1 の全体
整合の重大 finding は解消済み**であること。

**観点：**

- Options space が十分に開かれているか（段階 1 の確認の引き継ぎ）。
- 比較軸が fair か、特定案へ不自然に誘導していないか。
- Recommendation の根拠が **owner intent / acceptance criteria / constraints /
  prior decisions** に接続しているか。
- 推奨案が**他 DD/ADR の推奨案と矛盾していない**か。
- rejected / deferred / out-of-scope option の扱いが**強すぎ／弱すぎ**ないか。
- fallback や alternate path を残す場合、**Accepted 時の意味が曖昧でない**か
  （何を選んだことになるのか）。
- owner が受け入れる **trade-off・将来コスト・後戻りコスト**が明示されているか。
- 将来拡張性を理由に**過剰設計**していないか。
- 実装容易性を理由に**設計意図を狭めすぎ**ていないか。
- verification / documentation / downstream plan が Recommendation と整合しているか。

### reviewer プロンプト

```text
このフェーズの DD を recommendation-choice review してください。

目的: 各 DD/ADR の Options 比較と Recommendation が妥当かを、owner 判断として
確認する。implementation-readiness ではなく、「この選択で進むべきか」を見るレビュー
です。前提として strategic design / owner-alignment review の全体整合の重大 finding
は解消済みとします。

レビュー観点:
- Options space が十分に開かれているか
- 比較軸が fair か、特定案へ不自然に誘導していないか
- Recommendation の根拠が owner intent / acceptance criteria / constraints /
  prior decisions に接続しているか
- 推奨案が他 DD/ADR の推奨案と矛盾していないか
- rejected / deferred / out-of-scope option の扱いが強すぎ／弱すぎないか
- fallback や alternate path を残す場合、Accepted 時の意味が曖昧でないか
- owner が受け入れる trade-off、将来コスト、後戻りコストが明示されているか
- 将来拡張性を理由に過剰設計していないか
- 実装容易性を理由に設計意図を狭めすぎていないか
- verification / documentation / downstream plan が Recommendation と整合しているか

前提資料: workflow.md、phase framing、constraints、関連 notes/spec/prior DD/handoff、
decisions preamble、対象 DD。

出力は重大 findings 優先で、各 finding に【問題 / owner 判断への影響 / 修正方向】を
含めてください。最後に、重大 finding が無い場合はその旨を明示し、全体として owner が
Accepted 判断に進める状態かを短く評価してください。

（任意の重点）今回は特に ___ を重点的に見てください。
  例: クロス DD の推奨整合 / fallback の Accepted 時の意味 / trade-off の明示
```

### reviewee プロンプト

```text
このフェーズの DD を recommendation-choice review しました。
strategic design / owner-alignment review は実施済みで、全体整合の重大 finding は
解消済みです。

目的: 各 DD/ADR の Options 比較と Recommendation が妥当かを、owner 判断として確認する。
implementation-readiness ではなく、「この選択で進むべきか」を見るレビューです。

指摘を受けてファイルを修正する場合は、finding を fold / reject / defer に分類し、
reject・defer には 1 行理由を残してください。取り込んだ結果が他 DD/preamble の語彙と
矛盾しないか相互参照を確認してください。Revision history では初版との差分を簡潔に
示すこと。Revision history への追加は 1 行に集約します。Status は Proposed のまま維持。

指摘は以下です。
...
```

**推奨 verdict：** owner が Accepted 判断に進める状態か（**進める / 要修正**）を短く
評価。

---

## 段階 3: implementation-readiness review

**狙う失敗：** 設計は良いが、実装者が task 分解・コード変更・検証に落とせず迷う。

**関心事：** 落とす。DD から安全に implementation plan（[workflow.md §4](./workflow.md)）
へ進めるか。**設計思想の再検討はしない**（それは段階 1/2 で終わっている前提）。

**観点：**

- Recommendation の**曖昧さ・未決事項**。
- 触るべき **IR / runtime / API / grammar / spec 境界**。
- **実装順序**（依存方向、green workspace を保てるか）。
- **test / evidence への落とし込み**（何が positive control か、何が CI-gated か）。
- **diagnostics / error handling**（どの段で何を reject するか）。
- **doc-sync の抜け**（Moment 1 / Moment 2 の対象 doc）。

### reviewer プロンプト

```text
このフェーズの DD を implementation-readiness review してください。

目的は、DD から安全に implementation plan へ進めるかを見ることです。設計思想の
再検討ではなく、実装者が迷わず task 分解・コード変更・検証に落とせるかを重視して
ください。

特に、Recommendation の曖昧さ、未決事項、触るべき IR/runtime/API/grammar/spec 境界、
実装順序、test/evidence への落とし込み、diagnostics/error handling、doc-sync の抜けを
レビューしてください。

前提資料: workflow.md、phase framing、constraints、関連 notes/spec/prior DD/handoff、
decisions preamble、対象 DD。

出力は重大 findings 優先で、各 finding に【問題 / 実装上のリスク / 修正方向】を
含めてください。最後に Ready / Ready with clarifications / Not ready を判定して
ください。

（任意の重点）今回は特に ___ を重点的に見てください。
  例: diagnostics / 実装順序と green workspace / doc-sync 対象の網羅
```

### reviewee プロンプト

```text
このフェーズの DD を implementation-readiness review しました。
目的は、DD から安全に implementation plan へ進めるかを見ることです。実装者が迷わず
task 分解・コード変更・検証に落とせるかを重視しています（設計思想の再検討ではあり
ません）。

指摘を受けてファイルを修正する場合は、finding を fold / reject / defer に分類し、
reject・defer には 1 行理由を残してください。Revision history では初版との差分を
簡潔に示すこと。Revision history への追加は 1 行に集約します。Status は Proposed の
まま維持。

指摘は以下です。
...
```

**推奨 verdict：** **Ready / Ready with clarifications / Not ready**。

---

## AI 同士で回すときの注意

reviewer も reviewee も AI エージェントになりうる。その場合の運用上の留意点：

- **reviewer は独立コンテキストで走らせると価値が出る。** reviewee（DD を書いた
  側）の合理化に引きずられないため、可能なら fresh なエージェント／別セッションで
  前提資料を読ませて起動する。同一セッションの続きで「自分が書いた DD を自分で
  レビュー」すると、批判が甘くなりやすい。
- **reviewer は「合意」しない。** AI 同士で finding を出し合い、reviewee が直し
  切ったように見えても、それは Accepted ではない。reviewer の出力はあくまで
  owner judgment への input。
- **重大 finding ゼロでも owner 判断を代行しない。** verdict は「owner が判断
  できる状態になった」という報告であって、accept の代理ではない。
- **reviewee は reject/defer を明示する。** AI どうしだと finding を機械的に全部
  取り込みがちだが、採らない判断にも owner が後から読める理由を残す（段階 2 の
  「rejected option の扱いが強すぎ／弱すぎ」を自分の改訂にも適用する）。
- **段階をまたいで前提を引き継ぐ。** 段階 2 の reviewer プロンプトは「段階 1 の
  整合 finding は解消済み」を前提として述べる。前段を飛ばした場合はその旨を
  reviewer に伝える（前提が崩れた状態でのレビューは findings の意味が変わる）。

---

## このガイドラインを使った記録の残し方（任意）

レビューを実施したら、何を・どの観点で見て・どう畳んだかは DD の **Revision
history** 1 行と、必要なら `implementation/log.md` の Decisions log に残ると、後から
「なぜこの設計になったか」を追える。M3-Phase 6 の preamble Revision history が
precedent（「strategic-design / recommendation-choice / implementation-readiness
review の findings を反映、Status: Proposed のまま」を 1 行で記録）。

繰り返しになるが、これらは**おすすめ**であって gate ではない。プロジェクトの規模や
DD の重さに応じて、段階を畳む・パスを増やす・重点を変えるのは自由。判断したら理由を
1 行残すこと、それだけが（後から読めるようにするための）唯一のお願いである。
