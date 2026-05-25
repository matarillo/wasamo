---
title: レトロスペクティブ — マージ前の振り返り手順
status: live
created: 2026-05-05
---

# レトロスペクティブ

ブランチをマージする前に retrospective を実施する。**scope はマージ先で
決まる**:

- merge 先 = phase ブランチ → **step retrospective** → オーナー明示承認後 no-ff merge
- merge 先 = main → **phase retrospective** → オーナー明示承認後 no-ff merge → オーナー明示承認後 push
- 1 step = 1 phase (step ブランチが phase 全体をカバー) の場合は
  step → phase merge 後に **続けて phase retrospective も回す**。
  step→phase と phase→main はどちらも no-ff merge で、それぞれ独立した
  gate (**オーナー確認は最低 2 回必要**、retrospective を 1 回にまとめて
  main まで一気に進めない)。push は no-ff merge とも別 gate。

step→phase merge も Phase 2 / Phase 3 実運用に合わせて no-ff
(`git log --merges feat/m3-phase-2` / `feat/m3-phase-3` で確認可)。
M3-Phase 3 までは本文に "ff merge" 表記が残っていたが、M3-Phase 4
prep で no-ff に揃えた。過去 step retrospective 内の "ff merge"
言及は当時の文面に従った歴史記録として残置。

## 進行手順

checklist 完了 = merge 許可ではない。順序を固定する:

1. 下記 checklist を実施
2. 結果をオーナーに報告 (CHANGELOG / plan diff、CI / rebuild 結果、
   未決事項、retrospective 所見)
3. オーナーが merge を承認 (step-end / phase-end ともに **明示承認
   必須**、merge 種別は no-ff 固定)
4. 承認後に merge を実行
5. (phase-end のみ) オーナーが push タイミングを別途承認
6. 承認後に push を実行
7. push 後 main CI green 確認 (項目 16)

step-end / phase-end ともに **オーナー明示承認なしでは merge しない**
(M3-Phase 4 prep で「ファストトラック (= 報告と同時に merge → 事後
通知)」を廃止。step 単位の意思決定可視性をオーナー側に揃えるため、
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
   step 跨ぎの fmt ドリフトを見落とした事故 (commit `1129aea`) を踏まえ、
   M3-Phase 2 pre-doc framing 決定 E (a) で固定された discipline
   (see [m3-phase-2 pre-doc framing §E](./m3-phase-2/m3-phase-2-pre-doc-framing.md))。
4. PO に相談すべき設計判断・トレードオフ — あり/なし

### step-end 固有 (merge → phase ブランチ)

5. plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更 — あり/なし
6. 現在の phase ADR への追加 DD 必要性 — あり/なし
7. 既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格 — あり/なし
8. 現行 milestone plan (`m<N>-plan.md`) の AC (Ax) 追加・変更、
   または Phase 構成の追加・統合・分割 — あり/なし
9. 後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告 — あり/なし
10. 新たに発見・導入した cross-step / cross-phase の設計制約 —
    なし / あり。
    「あり」は、この step 以降の実装・仕様・テストが暗黙に依存
    しそうな規約、前提、不変条件、境界の意味論に限る。単なる
    実装詳細、既に ADR / pre-doc / `architecture.md` / `dsl_spec.md`
    / `abi_spec.md` に明記済みの前提、本 step 内だけで閉じる一回限り
    の判断は「なし」扱い。
    「あり」の場合は 1 件につき次だけを書く (見落とし検出器であって
    思考メモではない):
    - **制約**: 1 文
    - **エビデンス**: どの実装・テスト・smoke・レビューで露見したか
    - **配置先** (1 つ選ぶ):
      - **設計文書反映 (`doc-folded`)**: 本 step で
        `architecture.md` / `dsl_spec.md` / `abi_spec.md` / ADR に
        反映済み。retro 本文には pointer のみ。
      - **フェーズ終了時反映 (`phase-sync`)**: phase-end の Moment 2
        spec sync 候補に積む (§phase-sync で触る doc セット を参照)。
      - **前送り (`carry-forward`)**: 次 phase pre-doc input に
        本文として前送りする (§前送り を参照)。
      - **ローカル限定 (`local-only`)**: 将来制約ではないため
        前送りしない。なぜ将来制約ではないかの理由を 1 文。
    参考 precedent: M3-Phase 3 T9 の pure-layout 絶対座標 /
    `Visual.Offset` parent-relative の境界規約 (`architecture.md`
    §6.5 に fold = `doc-folded`)。
11. タスクリストの後続 step 見直し (現在の phase ADR に影響しない
    範囲) — 必要/不要

    本 step で `[ ]` のまま残る ADR / progress evidence がある
    場合、それを **次に所有する step が progress file 上で明示
    されているか** を点検する。owner-manual GUI smoke / 人手 CI
    確認 / 外部レビュー gate 等の人間系 gate は、phase-end checklist
    や `retrospectives.md` checklist への暗黙依存に置かず、
    **(a) 実行責任 step、(b) 失敗時の fix container (= 不具合発生
    時にどの step ブランチで additive 修正を積むか)** の両方を
    progress file の Task list 上で明示する。次の step が現在の
    Task list に存在しない場合、merge 前に Task list を revise
    して step を挿入し直す (= ownership 修正としての plan revision)。
    単なる「次の step で扱う」言及は不可。

    precedent: M3-Phase 4 で T5 の "Leave visual correctness as
    owner-manual GUI smoke" `[ ]` が T5 merge 待ち / 旧 T6 (phase-end
    機械的 close) の両方で実行責任主体が不明という宙吊り状態だった
    ことを T5 close 時に検出し、T5/T6 split + 旧 T6 → T7 への
    renumber で plan revise した
    (`docs/plans/progress/m3-phase-4-progress.md` Decisions log
    "T5/T6 split for owner-manual GUI smoke (2026-05-25)")。本観点
    が無かったため当初検出が遅れた。

CI green: 推奨 (PR を上げていれば PR CI、ローカルのみなら項目 3 の
clean rebuild が proxy)。CI YAML 変更は通常不要 (phase 内で発生したら
ADR に補足 DD)。

### phase-end 固有 (merge → main)

12. acceptance criteria (Ax) が本当に達成されているか — ADR の
    「discharged」表記と実装の乖離
13. `CHANGELOG.md` / `ROADMAP.md` の記述と実装の整合
14. `VISION.md` / thesis-level claim への影響 — 影響あれば本 phase 内
    で更新するか、別 ADR に切るかを決める
15. 次 phase の pre-doc への送り込み材料を `docs/notes/` に整理したか。
    本 phase の step retros で項目 10 を `phase-sync` 分類した制約は、
    phase-end で **`doc-folded` 相当 (文書反映済み) / `carry-forward`
    / `local-only` のいずれかに閉じる** (open のまま phase close
    しない)。`carry-forward` 分類した制約、および `phase-sync` 分類
    から phase-end で `carry-forward` に閉じた制約は、出発点になる
    設計軸・未決の論点・引き継ぎ制約と並んで次 phase pre-doc input に
    含める。
16. CI green 確認 — phase ブランチ上で `workflow_dispatch` から CI を
    走らせ、**merge 前に GitHub Actions green を確認する** (proxy では
    なく実 CI を gate にする)。**push はオーナー明示承認後のみ**。
    push 後は main 上で CI green を再確認 (push トリガで自動実行)、
    失敗時の recovery (revert PR / force reset 等) はオーナー判断。
17. human-visible GUI smoke — 必要/不要。runtime / ABI / binding /
    wasamoc lowering / examples 等、ユーザー可視の挙動に影響しうる
    phase では必要。必要な場合は
    [human-visible GUI smoke](./human-visible-smoke.md) に従い、
    `counter-c`, `counter-rust`, `counter-zig` を確認する。
18. CI YAML 変更要否の sanity check — 本 phase で新言語/新ビルド系を
    追加していれば CI 更新済みであること (CLAUDE.md の CI rules)

CI green: **必須**。clean rebuild は **必須** (incremental cache の嘘を
main に持ち込まない)。

## phase-sync (Moment 2) で触る doc セット

step retro item 10 で `phase-sync` 分類した制約は phase-end の
Moment 2 spec re-sync に積む。Moment 2 で **本 phase が実際に触れた
範囲に限り** sync する候補 doc セット:

- `docs/dsl_spec.md` — 該当 chapter の `**Phase status:**` marker を
  `M<N>-Phase <M> closed; implementation-synced` に flip、design draft
  と実装の divergence を反映
- `docs/architecture.md` — 上端 Status を `M<N>-Phase <M> complete`
  に flip、該当 paragraph block を realised impl と整合
- `docs/abi_spec.md` — 本 phase で C ABI 表面 (シグネチャ、定数、
  エラーコード等) を変更した場合、該当 section を impl と整合
- `docs/decisions/m<N>-phase-<M>-*.md` (該当 phase ADR) — 以下の
  いずれかが発生した phase でのみ touch:
  - AC `discharged` 表記と impl の乖離を fold (item 12 由来)
  - out-of-phase residual を residual / handover section に
    cross-reference 追記 (実運用、precedent: M3-Phase 3 commit
    `826d5b4`)
  - thesis-level 発見を本 phase ADR に追記 (item 14 由来; 別 ADR に
    切る判断なら本 phase commit には含めない)
- `docs/plans/m<N>-plan.md` — Progress section 該当 row
- `docs/plans/progress/m<N>-phase-<M>-progress.md` — Moment 2 task
  checkbox flip、CI evidence pointer、impl summary

Phase 2 / Phase 3 では abi_spec.md は touch しなかったため Moment 2
の実 commit には含まれていない。今後 ABI 表面を変更する phase が
来たら本リストに従って sync する。

ADR についても M3-Phase 3 では out-of-phase residual filing (R1 / R2
の cross-ref 追加) のみ touch し、substantive な DD 改訂は無かった。
ADR への substantive 追加・修正 (新 DD 追加、Proposed → Accepted
昇格等) は step-end (item 6 / 7 / item 10 `doc-folded`) が主経路で、
phase-end は上記 3 ケースの整合 touch に限定される。

commit shape は per-review-concern
([CLAUDE.md §Commit rules](../../CLAUDE.md#commit-rules))、Moment は
milestone label であって commit unit ではない。earlier-phase spec gap
の同 commit fold は owner 明示確認の上で最小範囲のみ。

> **運用注記:** Moment 2 の two-moment structure は
> [m3-phase-2 pre-doc framing §D](./m3-phase-2/m3-phase-2-pre-doc-framing.md#d-upstream-document-revision-timing-two-sync-moments)
> に由来する。ただし、Phase 2 / Phase 3 以降の実運用で doc セットと
> commit shape は更新されているため、retrospective / phase-end gate
> の実施時は本 `docs/notes/retrospectives.md` の記述を living rule
> として優先する。
>
> プロセスルール全体を notes 外へ正式 SSOT 化するか
> (`CLAUDE.md` / `docs/conventions/` / 領域別 README のいずれにする
> か等) は [process-rules-ssot.md](./process-rules-ssot.md) の open
> question (Q1-Q6) として未決。本節はその整理が完了するまでの
> 暫定的な運用上の正とする。

## Main Learning と設計制約の前送り

phase close 前に、次 phase の pre-doc input
(`docs/notes/m<N>-phase-<M>/predoc-inputs.md` など) へ次を整理する:

- phase / step retrospectives の `Main Learning` のうち、次 phase の
  設計判断に効くもの
- 本 phase の step retrospective で項目 10 を `carry-forward` 分類
  した制約
- `phase-sync` 分類とした制約のうち、phase-end で `carry-forward` に
  閉じた制約 (`doc-folded` 相当・`local-only` に閉じたものは含まない)

次 phase の pre-doc 着手時に作業する、ではなく、**現 phase の merge
gate を通過する前に書き終えていなければならない**。記載形態は次
phase の framing に合わせて選んでよい (逐語引用 / 要約 / 明示リンク
化など) が、**現 phase の retrospective 本文だけで完結させるのは
不可** (= 単にリンクを書いて「次の人が読む」形は前送り未達)。

`doc-folded` 分類は転記不要。次 phase は当該 doc (`architecture.md`
/ `dsl_spec.md` / `abi_spec.md` / ADR) を読めば足りるので、pre-doc
input には必要なら pointer のみ置く。

これは A11 (per-phase spec sync) と並ぶ phase 間連続性の gate であり、
checklist 項目 15「次 phase の pre-doc への送り込み材料を
`docs/notes/` に整理したか」は、上記 3 種を含む前送りファイルが
phase close commit に含まれていることが達成条件。

前送りファイルは次 phase 開始時の **pre-doc framing が input として
読み、次 phase ADR の DD slate / framing 決定の素材**になる
(pre-doc → owner agreement → impl → post-doc サイクルの「pre-doc」
側入力)。次 phase ADR が `Status: Accepted` に flip した時点で前送り
内容は ADR / 関連 spec / 該当 phase plan に消化され、predoc-inputs
ファイル自体は live status から retired/archive に転じる。
