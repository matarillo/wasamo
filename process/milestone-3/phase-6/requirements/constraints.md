---
title: M3-Phase 6 制約引き継ぎ — ZStack + 条件レンダリング
status: accepted
created: 2026-05-31
source-phase: M3-Phase 5
target-phase: M3-Phase 6
---

# M3-Phase 6 制約引き継ぎ

ワークフロー §2.1 のアウトプット。前フェーズの永続記録
[M3-Phase 5 handoff.md](../../phase-5/implementation/handoff.md) から本
フェーズ（**ZStack primitive + 条件レンダリング grammar**）に効く制約を
切り出し、本 phase の論点・スコープ・検証方針に合わせて再構成する。単純
コピーではなく、各項目に「Phase 6 でどう効くか」を付す。

Phase 6 thesis（[plan.md](../../plan.md) Phase 6 行）の前提:

- ZStack（兄弟要素の Z 順 overlay、document order）と条件レンダリング
  （`bool` 駆動の subtree present/absent 切替）を **一体で出荷**する。
  gallery の lightbox が両者を unit として使うため。
- **M3 初の grammar surface**。binding が property 値だけでなく
  widget-tree の構造（subtree の存否）に届く。
- `bool` scalar は M3-Phase 1 で landed 済み（hard prereq 充足）。

引き継ぎ源は phase-5 handoff.md だが、論点に効く milestone-plan レベルの
義務（§7 reactive-drain residual）も併記する。各項目末尾に **採否**（本
phase の constraints とするか / 別 owner へ送るか）を明示する。

---

## 1. R1 — Gallery host Window-title wiring（**Phase 6 が owning**）

[phase-5 handoff §Out-of-phase residuals](../../phase-5/implementation/handoff.md)
の R1。Phase 4 から carry され、Phase 5 FD-E で owning phase = **Phase 6**
と確定済み（[plan.md](../../plan.md) Phase 6 行 Notes に記録）。

- **観測（Phase 4 由来）:** smoke 全 screenshot で
  `MainWindowTitle = "Wasamo"`（framework default）のまま、
  `examples/gallery/gallery.ui` の `title: "Gallery"` を反映していない。
  `.ui` lowering は component-level `title:` surface を保持するが、
  runtime/ABI host 経路が framework default title で Window を生成する。
- **解決条件（owner intent 2026-05-25）:** 「title attribute is declared
  unsupported」ではなく、**runtime/ABI host path applies
  component-level `title:` to the native window**。
- **期限:** 遅くとも **M3-Phase 8 Gallery E2E close まで**。Phase 6 が
  owning するので本 phase で実装するのが既定線。

**Phase 6 への効き方:**

- **本質は component metadata / host wiring であり、binding 駆動ではない。**
  R1 が要求するのは「component 宣言時の **静的** `title:` 文字列を
  runtime/ABI host 経路が native window に適用する」という host-wiring の
  欠落の修復であって、`bool`/binding 駆動ではない。Phase 6 と同居させる
  のは時期的な都合（FD-E が「最初の条件レンダリング / `bool` 駆動
  property-update slice と一緒に land」と置いた）であり、技術的依存では
  ない点に注意。
- **スコープ（→ §2.3）:** R1（静的 title の host-wiring）を Phase 6 の
  スコープに明示的に含める。**Phase 6 の最小達成条件は R1 解決条件
  そのもの（静的 `title:` が native window に乗る）に閉じる。**
- **論点（→ §2.2、optional）:** 将来 `String`/`bool` binding 駆動の
  **動的** title まで射程に入れるかは DD slate で開きうる論点だが、
  **本 phase の必須ではない**。動的 title を開かない判断なら ADR に
  1 行残すだけで足りる。
- **検証（→ §2.4）:** owner-visible GUI smoke の screenshot 上で
  native window title bar が `"Gallery"` を表示することを確認する
  （§2 の assistant-visible evidence でも title bar をフレームに含める）。
- **採否:** **採用**（Phase 6 の実装制約）。

## 2. Assistant-visible GUI evidence 標準（screenshot + 解析）

[phase-5 handoff §Pointers](../../phase-5/implementation/handoff.md)（T5/T6
由来、`doc-folded`）。規範核は
[CLAUDE.md §Testing rules](../../../../CLAUDE.md)、capture mechanics /
環境要件は
[docs/notes/verification-environments.md Observation 4](../../../../docs/notes/verification-environments.md)
に fold 済み。

- GUI host が **実際に描画した**ことが evidence の task では、assistant の
  自動 evidence は **launch + screenshot capture + assistant の画像解析**
  でなければならない。`Start-Process` 生存は「早期 crash なし」の補助
  signal にすぎず、画面が非空で描画されたこと・意図した sub-screen が
  view に入っていることは示せない。
- capture は `Graphics.CopyFromScreen`（`PrintWindow` 不可 —
  DirectComposition client area は `PrintWindow` で blank に読み戻る）。
- これは owner の human-visible GUI smoke を **代替しない** pre-owner
  check。

**Phase 6 への効き方:**

- Phase 6 は M3 で最も「見える」surface を出荷する（lightbox overlay の
  ZStack、条件レンダリングの present/absent）。handoff は「M3-Phase 6
  gallery visible work will exercise this rule」と明記。各 visible task の
  verification closure（→ §2.4）に screenshot + 解析を最初から組み込む。
- **採否:** **採用**（Phase 6 の検証方針の前提）。

## 3. 陽性対照（positive control）規律 — **Phase 6 で最も load-bearing**

[phase-5 handoff §Pointers](../../phase-5/implementation/handoff.md)（T6
owner-smoke 由来、`doc-folded`）。fold 先は §2 と同じ
[CLAUDE.md §Testing rules](../../../../CLAUDE.md) +
[verification-environments.md Obs 4](../../../../docs/notes/verification-environments.md)。

- 単発の static frame（誤実装でも同じ見た目を出しうるもの）は evidence に
  ならない。意図した挙動を look-alike から **区別する陽性対照**を含める:
  star track の柔軟性は resize で比率保持を見せて証明、clip は source に
  対し**何が欠けているか**で証明、**条件 / stateful rendering は state を
  toggle して証明**（初期 state だけでは不可）。

**Phase 6 への効き方（直撃）:**

- handoff が名指しする通り「M3-Phase 6 conditional rendering（初期 frame
  だけでは `bool` 駆動 slice が toggle することを証明できない）」に直結。
  Phase 6 の verification は **必ず state を toggle** して present → absent
  （およびその逆）の両 frame を撮り、解析する設計にする。
- **ZStack にも適用:** z-order は「何が何を occlude するか」で証明する。
  重なりのない単発 frame は flat layout でも同じに見えるため、overlay の
  上下関係が見える構図（後ろの要素が前の要素に隠れる、scrim が下を覆う）
  を陽性対照として撮る。
- **採否:** **採用**（Phase 6 検証方針の中核制約）。§2 と合わせ、
  conditional/ZStack の各 visible task は「toggle 前後 2 frame」を最小
  evidence とする。

## 4. screenshot capture の per-monitor-DPI awareness（assistant tooling）

[phase-5 handoff §Carry-forward](../../phase-5/implementation/handoff.md) の
DPI 項のうち、runtime 修正ではない **assistant-tooling 側**の制約。
[verification-environments.md Obs 4](../../../../docs/notes/verification-environments.md)。

- assistant-visible GUI evidence 用の screenshot capture は
  per-monitor-DPI-aware でなければならない（高 DPI display 上で client
  area とフレーム座標がずれないように）。

**Phase 6 への効き方:**

- §2 / §3 で screenshot evidence を多用するため、capture コードが
  DPI-aware であることは Phase 6 の evidence 取得の前提。runtime の DPI
  awareness 欠如（§5）とは別レイヤ。
- **採否:** **採用**（evidence 取得の前提）。

## 5. runtime の per-monitor DPI awareness 欠如 — **本 phase の制約にしない（M4 へ）**

[phase-5 handoff §Carry-forward](../../phase-5/implementation/handoff.md) の
DPI 項（runtime 側）。vision/roadmap 決定は
[DD-V-022 / DD-V-023](../../../cross-milestone/decisions/dpi-awareness-m4-deferral.md)
で M4 acceptance criterion として landed 済み。

- runtime は M1 以来 DPI-unaware。高 DPI monitor で DWM が window 全体を
  bitmap-scale する（125% → logical 800×600 が physical 1000×750 で一様に
  blur）。layout は client pixel を logical unit として 1:1 消費し、
  `WM_DPICHANGED` handler も無い。

**不採用の理由（§2.1 に従い明記）:**

- ZStack / 条件レンダリングの設計と **直交**する。両者は logical pixel で
  正しく計算され、DPI は orthogonal な runtime-quality 軸。
- gap は **runtime-wide かつ pre-existing**で、Phase 6 固有でない。
- 既に **専任 owner**（DD-V-022/023 + roadmap M4 AC + handoff の
  engineering input）を持つ。Phase 6 ADR から cross-ref すると読者に
  noise を足すだけ（Phase 5 が Grid ADR を DPI に cross-ref しなかった
  判断と同じ）。
- **採否:** **不採用**（M4 owned）。本 phase は logical-pixel 正しさのみを
  検証対象とし、DPI blur は evidence 解析時に「既知の M4 残課題」として
  注記するに留める。

## 6. phase 最終 task の retrospective / progress checklist 分割（プロセス学び）

[phase-5 handoff §Main learnings](../../phase-5/implementation/handoff.md)。
T0 で凍結した task list は、mid-phase の owner 決定が項目を動かすと stale
ownership を抱えうる（Phase 5 T7 で phase-sync close と handoff clean-up が
T7 に割り当たっていたが、retrospectives.md §15/§6.3 により phase-end retro
へ再割当 → plan revise A で解決）。

**Phase 6 への効き方:**

- Phase 6 の `implementation/plan.md` 最終 task checklist は、最初から
  **task-end retrospective** と **phase-end retrospective** を別 bullet に
  する（task-end は最終 task が `[x]` 可、phase-end は phase → main merge
  gate が所有し最終 task close 時点で `[ ]` のまま）。
- **最終 task close の直前に、T0 凍結 task list を mid-phase owner 決定と
  cross-check** し、ズレがあれば mutable phase plan を revise する（work
  around しない）。手順は
  [retrospectives.md](../../../procedures/retrospectives.md)。
- **採否:** **採用**（実装計画 §4 / クロージング §6 の進め方の前提）。

## 7. reactive-drain residual の fix-or-carry 判断義務（plan-level 制約）

引き継ぎ源は phase-5 handoff ではなく [plan.md §Risks](../../plan.md)。
plan は「Phase 6 / 7 が dirty-Effect drain residuals（DD-M2-P6-010
follow-ons）に触れる最有力候補」とし、**phase pre-doc が
[process/milestone-2/handoff.md §3](../../../milestone-2/handoff.md) を
参照して fix するか carry-forward するかを明示判断する**ことを要求する
（silent carry-forward は不可）。M2 handoff §3 は **4 項目**の inherited
obligation を挙げる：

1. **cycle detection policy**（IR-load 構造則で防止 / runtime 検出 /
   `wasamoc` lowering 拒否のいずれか）。
2. **ordering ties**（依存関係のない Effect 間の順序を observable
   contract とするか implementation-defined のままにするか）。
3. **fan-out × `MUTATION_CAP`**（cap 拡大 / per-shape 化 / 別の収束
   保証への置換）。
4. **synchronous non-batched drain proof contract（M3-Phase 1 addendum）**
   — `BATCH_DEPTH == 0` での write が `hit_test_click` 戻りまでに dirty
   Effect を drain する、という M3-Phase 1 T13 が依拠した observable
   contract。M2 handoff は後続 phase の **bool-dependent display
   structure（notably 条件レンダリング）と Button selected state** が
   これに直撃すると名指しし、その proof contract を **保つか、
   "test/host が bound widget property を最新と期待できる境界" を明示
   revise するか**を要求する。

**Phase 6 への効き方:**

- 条件レンダリングは binding を widget-tree 構造に届かせる初の grammar で、
  subtree の生成/破棄が reactive drain と干渉しうる。Phase 6 ADR /
  framing は M2 handoff §3 の 4 項目を参照し、本 phase の
  multi-binding/subtree-toggle 経路が具体的 failure を surface するか
  判断し、fix または carry-forward を **明示記録**する。
- 特に **item 4 は Phase 6 直撃**：条件レンダリングの subtree が
  present/absent するとき、**`bool` を toggle した直後に host/test が
  いつ subtree presence を観測できるか**（drain 後 quiescence の観測
  境界）を Phase 6 の判断対象に含める。これは §3 の陽性対照（toggle
  前後 2 frame）の verification が「toggle 後どの時点の frame を撮れば
  確定状態か」という形でも効く。
- **採否:** **採用**（DD slate §2.2 で参照必須の論点入力。item 4 は
  Phase 6 の verification 設計にも波及）。

---

## 前送り対象に含めないもの（pointer のみ）

- **Grid carrier-c1 textual IR grammar**（Phase 5 T1/T3、`doc-folded`）—
  [docs/dsl_spec.md](../../../../docs/dsl_spec.md) §8.5（`track_decl`）+
  §5/§2.2/§3 に fold 済み。Grid 固有で Phase 6 thesis（ZStack /
  conditional）とは無関係。Phase 6 が spec を read する際の前提として
  pointer のみ。
- **DPI runtime 修正の実装詳細**（§5）— DD-V-022/023 + roadmap M4 AC が
  owner。本 constraints では不採用理由のみ記録し、engineering 詳細は
  phase-5 handoff と VDR を直接読む。
