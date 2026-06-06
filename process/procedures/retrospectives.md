---
title: レトロスペクティブ — マージ前の振り返り手順
status: SSOT
created: 2026-05-05
---

# レトロスペクティブ

ブランチをマージする前に retrospective を実施する。**scope はマージ先で
決まる**:

- merge 先 = phase ブランチ → **task retrospective** → オーナー明示承認後 no-ff merge
- merge 先 = main → **phase retrospective** → オーナー明示承認後 no-ff merge → オーナー明示承認後 push
- 1 task = 1 phase (task ブランチが phase 全体をカバー) の場合は
  task → phase merge 後に **続けて phase retrospective も回す**。
  task→phase と phase→main はどちらも no-ff merge で、それぞれ独立した
  gate (**オーナー確認は最低 2 回必要**、retrospective を 1 回にまとめて
  main まで一気に進めない)。push は no-ff merge とも別 gate。

task→phase merge も Phase 2 / Phase 3 実運用に合わせて no-ff
(`git log --merges feat/m3-phase-2` / `feat/m3-phase-3` で確認可)。
M3-Phase 3 までは本文に "ff merge" 表記が残っていたが、M3-Phase 4
prep で no-ff に揃えた。過去 task retrospective 内の "ff merge"
言及は当時の文面に従った歴史記録として残置。

## 進行手順

checklist 完了 = merge 許可ではない。順序を固定する:

1. 下記 checklist を実施
2. 結果をオーナーに報告 (CHANGELOG / plan diff、CI / rebuild 結果、
   未決事項、retrospective 所見)
3. オーナーが merge を承認 (task-end / phase-end ともに **明示承認
   必須**、merge 種別は no-ff 固定)
4. 承認後に merge を実行
5. (phase-end のみ) オーナーが push タイミングを別途承認
6. 承認後に push を実行
7. push 後 main CI green 確認 (項目 16)

task-end / phase-end ともに **オーナー明示承認なしでは merge しない**
(M3-Phase 4 prep で「ファストトラック (= 報告と同時に merge → 事後
通知)」を廃止。task 単位の意思決定可視性をオーナー側に揃えるため、
判定の自動化より毎回のレビューを優先する判断)。

## checklist

各項目「あり/なし」(または green/fail、必要/不要) で答える。
checklist は **オーナー報告の構造化テンプレ**であり、項目の値が
何であっても merge にはオーナー明示承認が必要。

### 共通 (両 scope)

1. 本作業の主要な学び (記述項目、判定対象外)
2. 仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の
   変更 — あり/なし。タイポ修正、または既に Accepted な DD の
   機械的転記は「なし」扱い。
3. プロジェクトルートで `cargo fmt --all -- --check` を **post-commit
   state に対して** 実行した上で、ローカル clean rebuild (`cargo
   clean` → release+debug build → `cargo test --workspace`) —
   green/fail。「green」は `--check` がゼロ終了することを
   指し、単に `cargo fmt --all` が走り終わったことではない。事前に
   `cargo fmt --all` を回すのは構わないが、ゲートは post-commit
   state での `--check` が ground truth。M3-Phase 1 phase-end で
   task 跨ぎの fmt ドリフトを見落とした事故 (commit `1129aea`) を踏まえ、
   M3-Phase 2 フェーズ計画 決定 E (a) で固定された discipline
   (see [m3-phase-2 framing §E](../milestone-3/phase-2/requirements/framing.md))。
4. PO に相談すべき設計判断・トレードオフ — あり/なし

### task-end 固有 (merge → phase ブランチ)

5. plan/ADR に記載の task 目的から外れた「ついで」のリファクタ・
   構造変更 — あり/なし
6. 現在の ADR への追加 DD 必要性 — あり/なし
7. 既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格 — あり/なし
8. 現行 milestone plan (`m<N>-plan.md`) の AC (Ax) 追加・変更、
   または Phase 構成の追加・統合・分割 — あり/なし
9. 後続 task に持ち越す仮実装・近似・新規 `dead_code` 警告 — あり/なし
10. 新たに発見・導入した cross-task / cross-phase の設計制約 —
    なし / あり。
    「あり」は、この task 以降の実装・仕様・テストが暗黙に依存
    しそうな規約、前提、不変条件、境界の意味論に限る。単なる
    実装詳細、既に ADR / framing.md / `architecture.md` / `dsl_spec.md`
    / `abi_spec.md` に明記済みの前提、本 task 内だけで閉じる一回限り
    の判断は「なし」扱い。
    「あり」の場合は 1 件につき次だけを書く (見落とし検出器であって
    思考メモではない):
    - **制約**: 1 文
    - **エビデンス**: どの実装・テスト・smoke・レビューで露見したか
    - **配置先** (1 つ選ぶ):
      - **設計文書反映 (`doc-folded`)**: 本 task で
        `architecture.md` / `dsl_spec.md` / `abi_spec.md` / ADR に
        反映済み。retro 本文には pointer のみ。
      - **フェーズ終了時反映 (`phase-sync`)**: phase-end の実装同期
        (Moment 2) spec sync 候補に積む (§phase-sync で触る doc セット
        を参照)。
      - **前送り候補 (`carry-forward`)**: retro 本文の項目 10 entry
        として記録する。ここでは `implementation/handoff.md` の
        確定成果物としては更新しない。phase-end retro の項目 15 で
        最終的な handoff 対象を確定し、§6.3 handoff の整理で
        `implementation/handoff.md` に清書する。
      - **ローカル限定 (`local-only`)**: 将来制約ではないため
        前送りしない。なぜ将来制約ではないかの理由を 1 文。
    参考 precedent: M3-Phase 3 T9 の pure-layout 絶対座標 /
    `Visual.Offset` parent-relative の境界規約 (`architecture.md`
    §6.5 に fold = `doc-folded`)。
11. タスクリストの後続 task 見直し (現在の ADR に影響しない
    範囲) — 必要/不要

    本 task で `[ ]` のまま残る ADR / 検証 evidence がある
    場合、それを **次に所有する task が `implementation/plan.md` 上で
    明示されているか** を点検する。owner-manual GUI smoke / 人手 CI
    確認 / 外部レビュー gate 等の人間系 gate は、phase-end checklist
    や `retrospectives.md` checklist への暗黙依存に置かず、
    **(a) 実行責任 task、(b) 失敗時の fix container (= 不具合発生
    時にどの task ブランチで additive 修正を積むか)** の両方を
    `implementation/plan.md` の Task list 上で明示する。次の task が
    現在の Task list に存在しない場合、merge 前に Task list を revise
    して task を挿入し直す (= ownership 修正としての plan revision)。
    単なる「次の task で扱う」言及は不可。

    precedent: M3-Phase 4 で T5 の "Leave visual correctness as
    owner-manual GUI smoke" `[ ]` が T5 merge 待ち / 旧 T6 (phase-end
    機械的 close) の両方で実行責任主体が不明という宙吊り状態だった
    ことを T5 close 時に検出し、T5/T6 split + 旧 T6 → T7 への
    renumber で plan revise した
    (`process/milestone-3/phase-4/implementation/log.md` Decisions log
    "T5/T6 split for owner-manual GUI smoke (2026-05-25)")。本観点
    が無かったため当初検出が遅れた。

CI green: 推奨 (PR を上げていれば PR CI、ローカルのみなら項目 3 の
clean rebuild が proxy)。CI YAML 変更は通常不要 (phase 内で発生したら
ADR に補足 DD)。

### phase-end 固有 (merge → main)

12. acceptance criteria (Ax) が本当に達成されているか — ADR の
    「discharged」表記と実装の乖離
13. `CHANGELOG.md` / `process/_roadmap.md` の記述と実装の整合
14. `VISION.md` / thesis-level claim への影響 — 影響あれば本 phase 内
    で更新するか、別 ADR に切るかを決める
15. 次 phase の framing への送り込み材料を整理したか。本 phase の
    task retros で項目 10 を `phase-sync` 分類した制約は、phase-end で
    **`doc-folded` 相当 (文書反映済み) / `carry-forward` / `local-only`
    のいずれかに閉じる** (open のまま phase close しない)。
    task retro の項目 10 で `carry-forward` とした entry はこの時点では
    handoff 候補として一覧し、最終的な handoff 対象に含めるかを確定
    する。`phase-sync` 分類から phase-end で `carry-forward` に閉じた
    制約も同じく対象に含める。確定したものは、出発点になる設計軸・
    未決の論点・引き継ぎ制約と並んで §6.3 handoff の整理で
    `implementation/handoff.md` に清書する。
16. CI green 確認 — phase ブランチ上で `workflow_dispatch` から CI を
    走らせ、**merge 前に GitHub Actions green を確認する** (proxy では
    なく実 CI を gate にする)。**push はオーナー明示承認後のみ**。
    push 後は main 上で CI green を再確認 (push トリガで自動実行)、
    失敗時の recovery (revert PR / force reset 等) はオーナー判断。
17. human-visible GUI smoke — 必要/不要。runtime / ABI / binding /
    wasamoc lowering / examples 等、ユーザー可視の挙動に影響しうる
    phase では必要。必要な場合は
    [human-visible GUI smoke](../../docs/notes/human-visible-smoke.md) に従い、
    `counter-c`, `counter-rust`, `counter-zig` を確認する。
18. CI YAML 変更要否の sanity check — 本 phase で新言語/新ビルド系を
    追加していれば CI 更新済みであること (AGENTS.md の CI rules)

CI green: **必須**。clean rebuild は **必須** (incremental cache の嘘を
main に持ち込まない)。

## phase 最終 step の retrospective 分割 (複数 step phase)

複数 step からなる phase の**最終 step**では、その step の progress
checklist で **step-end retrospective (上記 checklist items 1-11)** と
**phase-end retrospective (items 12-18)** を **別 bullet・別ファイル・別
commit** に分割する:

- **step-end retro (items 1-11)** は最終 step が所有し、step → phase
  merge gate で `[x]` にできる。
- **phase-end retro (items 12-18)** は phase → main merge gate が所有し、
  最終 step を phase ブランチに merge した**後**に、別 commit で記録する。
  最終 step close 時点では `[ ]` のままでよい。

単一 bullet にすると所有者・タイミングの曖昧さで reviewer が混乱する
(M3-Phase 4 T7 で検出、M3-Phase 5 が最初から分割運用して有効性を確認し、
Phase 5 phase-end で本節に規範化)。1 task = 1 phase (単一 step が phase
全体をカバー) の場合は §進行手順 の通り task → phase merge 後に続けて
phase retrospective を回す形で足り、本分割は不要。

## phase-sync (実装同期 / Moment 2) で触る doc セット

task retro item 10 で `phase-sync` 分類した制約は phase-end の
実装同期 (Moment 2) に積む。Moment 2 で **本 phase が実際に触れた
範囲に限り** sync する候補 doc セット:

- `docs/dsl_spec.md` — 該当 chapter の `**Phase status:**` marker を
  `M<N>-Phase <M> closed; implementation-synced` に flip、design draft
  と実装の divergence を反映
- `docs/architecture.md` — 上端 Status を `M<N>-Phase <M> complete`
  に flip、該当 paragraph block を realised impl と整合
- `docs/abi_spec.md` — 本 phase で C ABI 表面 (シグネチャ、定数、
  エラーコード等) を変更した場合、該当 section を impl と整合
- `process/milestone-N/phase-M/decisions/` (該当 ADR) — 以下の
  いずれかが発生した phase でのみ touch:
  - AC `discharged` 表記と impl の乖離を fold (item 12 由来)
  - out-of-phase residual を `implementation/handoff.md` に
    cross-reference 追記 (実運用、precedent: M3-Phase 3 commit
    `826d5b4`)
  - thesis-level 発見を本 ADR に追記 (item 14 由来; 別 ADR に
    切る判断なら本 phase commit には含めない)
- `process/_roadmap.md` — Progress 該当 row、AC 達成記録
- `process/milestone-N/plan.md` — Progress section 該当 row
- `process/milestone-N/phase-M/implementation/plan.md` — Moment 2
  task checkbox flip、CI evidence pointer、impl summary

Phase 2 / Phase 3 では abi_spec.md は touch しなかったため Moment 2
の実 commit には含まれていない。今後 ABI 表面を変更する phase が
来たら本リストに従って sync する。

ADR についても M3-Phase 3 では out-of-phase residual filing (R1 / R2
の cross-ref 追加) のみ touch し、substantive な DD 改訂は無かった。
ADR への substantive 追加・修正 (新 DD 追加、Proposed → Accepted
昇格等) は task-end (item 6 / 7 / item 10 `doc-folded`) が主経路で、
phase-end は上記 3 ケースの整合 touch に限定される。

commit shape は per-review-concern
([AGENTS.md §Commit rules](../../AGENTS.md#commit-rules))、Moment は
milestone label であって commit unit ではない。earlier-phase spec gap
の同 commit fold は owner 明示確認の上で最小範囲のみ。

> **運用注記:** Moment 2 (実装同期) の two-moment structure は
> [m3-phase-2 framing §D](../milestone-3/phase-2/requirements/framing.md)
> に由来する。実装同期の概念定義は
> [workflow.md §6.2](./workflow.md) を参照。

## Main Learning と設計制約の前送り

前送りは 3 段階で行う:

1. task retro の項目 10 で `carry-forward` entry を候補として記録する。
2. phase-end retro の項目 15 で候補を一覧し、最終的な handoff 対象を
   確定する。
3. §6.3 handoff の整理で、確定済み材料だけを
   `implementation/handoff.md` に清書する。

phase close 前に `implementation/handoff.md` へ整理する対象は次のもの:

- phase / task retrospectives の `Main Learning` のうち、次 phase の
  設計判断に効くもの
- 本 phase の task retrospective で項目 10 を `carry-forward` 候補として
  記録し、phase-end retro で最終的な handoff 対象に確定した制約
- `phase-sync` 分類とした制約のうち、phase-end で `carry-forward` に
  閉じた制約 (`doc-folded` 相当・`local-only` に閉じたものは含まない)

次 phase の framing 着手時に作業する、ではなく、**現 phase の merge
gate を通過する前に `implementation/handoff.md` へ書き終えていなければ
ならない**。retrospective 中は `implementation/handoff.md` を確定成果物
として更新しない。記載形態は次 phase の §2.1 で再構成しやすい形を
選んでよい (逐語引用 / 要約 / 明示リンク化など) が、**現 phase の
retrospective 本文だけで完結させるのは不可** (= 単にリンクを書いて
「次の人が読む」形は前送り未達)。

`doc-folded` 分類は本文転記不要。次 phase は当該 doc (`architecture.md`
/ `dsl_spec.md` / `abi_spec.md` / ADR) を読めば足りるので、
`implementation/handoff.md` には必要なら pointer のみ置く。

これは A11 (per-phase spec sync) と並ぶ phase 間連続性の gate であり、
checklist 項目 15「次 phase の framing への送り込み材料を整理したか」
は、上記 3 種を含む `implementation/handoff.md` が phase close commit に
含まれていることが達成条件。

`implementation/handoff.md` は前 phase 側の永続記録であり、
`requirements/constraints.md` は次 phase 側の解釈結果である。次 phase
開始時の **フェーズ計画 (§2.1 制約引き継ぎ)** は handoff を input として
読み、次 phase の論点設定 (§2.2) / スコープ確定 (§2.3) / 検証方針確認
(§2.4) に効く形へ `constraints.md` を再構成する。単純コピーを目的にせず、
次 phase の DD slate・scope・verification と関係づけて要約 / 分割 / 統合
する。次 ADR が `Status: Accepted` に flip した時点で `constraints.md` の
内容は ADR / 関連 spec / 該当 phase plan に消化される。

(workflow との対応: [workflow.md §6.3 handoff の整理](./workflow.md)
の `implementation/handoff.md` は次 phase の `constraints.md` の原典。
両者は phase boundary 前後で対をなすが、後者は前者のコピーではなく
次 phase 専用の再構成である。)
