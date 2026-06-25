---
title: M3-Phase 8 制約引き継ぎ — selected state + Gallery E2E + DSL spec public draft
status: draft
created: 2026-06-25
source-phase: M3-Phase 7b
target-phase: M3-Phase 8
---

# M3-Phase 8 制約引き継ぎ

ワークフロー [§2.1](../../../procedures/workflow.md) のアウトプット。前
フェーズの永続記録
[M3-Phase 7b handoff.md](../../phase-7b/implementation/handoff.md) から本
フェーズ（**`selected` state + Gallery E2E + DSL spec public draft**）に効く
制約を切り出し、本 phase の論点・スコープ・検証方針に合わせて再構成する。
単純コピーではなく、各項目に「Phase 8 でどう効くか」を付す。本 phase の制約
として引き込むものと、別 owner / 後続 milestone へ前送りするものを理由付きで
分ける。

Phase 8 thesis（[plan.md Phase 8 行](../../plan.md)）の前提:

- 3 つの workstream が M3 を**まとめて閉じる**: (i) A10 = Button `selected`
  state の具体構文を確定し gallery の tab 風セクションを駆動、(ii) A1 = full
  `examples/gallery/gallery.ui` を 3 host（C / Rust / Zig）で end-to-end に
  組み立て、(iii) A12 = `docs/dsl_spec.md` を **first public draft** へ昇格。
- Phase 1–7 の per-phase spec 更新があるため、Phase 8 の spec 作業は
  **editorial**（ゼロから書くのではなく、既出 surface を public draft 品質へ
  磨く）。
- **Phase 8 は M3 の最終フェーズ**である。よって handoff が「後続フェーズへ
  carry」と書いた項目でも、その責務の一部が **M3 内で discharge すべきもの**
  なら Phase 8 の制約として引き込む。§2–§7 が「引き込む」項目、末尾の表が
  「前送り維持」項目を理由付きで分ける。

引き継ぎ源は phase-7b handoff だが、Phase 8 が M3 close フェーズである以上、
milestone-plan レベルの義務（future surface reservation の明示判断、
milestone-end criteria、A11 sync）も §8 に併記する。

---

## 1. placement surface は Phase 7b で凍結済み — Phase 8 は読むだけで再決定しない（**foundational**）

[phase-7b handoff §Phase 8 (editorial) note](../../phase-7b/implementation/handoff.md)
と §Main Learnings 第2項。Phase 7b は parent-interpreted placement の
author surface（`slot.*` + Grid `Cell`、PM-2）を **public draft の手前で
凍結する目的で**実装・同期した。

**Phase 8 への効き方:**

- Phase 8 は placement surface を **再決定しない**。`docs/dsl_spec.md` §4.16
  と `docs/architecture.md` §6.8.6 / §6.8.4 に Moment 2 同期済みの surface を
  **直接読んで** editorial に磨く。設計を draft から再導出せず、landed した
  *source* に pin する（handoff Main Learning 第2項: 「living spec は design
  draft でなく landed source に pin する」）。
- これは「凍結＝触らない」ではない。§2–§5 のとおり、**凍結した surface を
  public draft でどう提示するか**（provisional / reserved / 既定の非対称）は
  Phase 8 の editorial 判断であり、surface 自体の re-litigation とは別。これが
  A12 editorial pass の出発点。

---

## 2. author-controllable `width` / `height` sizing（Problem B）— Phase 8 framing で Vision DR を起票（**引き込み**）

[phase-7b handoff §Mid-phase-surfaced residuals](../../phase-7b/implementation/handoff.md)
（Problem B）と live note
[docs/notes/author-controllable-sizing.md](../../../../docs/notes/author-controllable-sizing.md)。
Fill-default container（Grid / ZStack）を Shrink 軸の祖先に入れると 0×0 に
潰れる。これは M3-Phase 2 以降 defer されてきた明示サイズ surface の欠落で
あり、placement redesign の regression ではない（7b は `measure_grid` /
`axis_is_stretchy` を変更していないと git 確認済み）。

**Phase 8 への効き方:**

- handoff と live note §7.2 がともに、**起票タイミングを「M3-Phase 8
  framing 段階」**と名指ししている（Phase 8 = public draft freeze は
  reservation を明示判断する forcing function）。責務割り当ては roadmap
  （SSOT）を触る構造的変更なので **`process/cross-milestone/decisions/` の
  Vision DR**（DD-V-022 と同パターン）で行う。
- したがって Phase 8 の制約は 2 つ:
  1. **Phase 8 framing（§2.2/§2.3）で Problem B の Vision DR を起票する**
     （milestone home + activation trigger + hard backstop=pre-1.0/M6
     ABI-freeze prep を割り当てる）。Vision DR は cross-milestone ガバナンス
     なので**確定を急がない**——起票と論点固定が Phase 8 の責務で、最終
     確定は別ゲート。
  2. **A12 public draft は Fill-default sizing を final として提示しない**。
     明示 `width` / `height` を **future surface** として draft にどう
     位置づけるかを明記する。これは plan の **M4 material reservation
     とは別根拠**である——M4 material の syntax 予約ではなく、Phase 2 以降
     残る author-controllable sizing の未解決問題で、根拠は本 handoff と
     [author-controllable-sizing.md](../../../../docs/notes/author-controllable-sizing.md)
     にある（§8 で別 bullet として分離）。
- `aspect`-in-cell arrange abort（`BoxAspectUnboundedBoth` で subtree が
  silent drop）は同じ sizing gap の facet なので、**この Problem B triage に
  fold** する（独立項目にしない）。
- **sizing surface 自体の実装は Phase 8 では行わない**——Vision DR が割り当てる
  milestone へ送る。Phase 8 が負うのは framing 起票と A12 documentation。

## 3. PM-2 provisional two-form Grid — public draft が「pre-1.0 未確定」と flag（**引き込み**）

[phase-7b handoff §Known carry-forward candidates 第1項（PM-2 wrapper-rule）](../../phase-7b/implementation/handoff.md)
と §Phase 8 note。Phase 7b は Grid placement に **2 form**（`Cell` と直接
`slot.*`）を出荷した——**正準形を規範化しない deliberately provisional な
状態**。どの widget / container が wrapper form を使えるかの規則（Grid を
PM-1=「`Cell` 維持・直接 `slot.*` 廃止」か PM-3=「`Cell` 廃止」へ寄せる）は
**1.0 前に**決める。

**Phase 8 への効き方:**

- wrapper-rule **決定そのもの**は pre-1.0 residual で、carry path は
  handoff → **M3 `handoff.md`（milestone close）** → pre-1.0。これは Phase 8
  では決めない（hard deadline = Wasamo 1.0 到達）。
- ただし handoff §Phase 8 note が明示的に「**public draft は wrapper-rule
  決定が pre-1.0 で未確定であることを flag せねばならない**」と Phase 8 の
  責務として名指ししている。public draft が 2 form を「確定した正準」として
  提示すると、後で PM-1/PM-3 に寄せたとき published surface の breaking
  change になる。
- Phase 8 の制約: public draft は accept-set（両 form 受理）を述べつつ、
  wrapper-rule を「pre-1.0 未確定の provisional」と明記する。決定自体は M3
  milestone handoff へ fold し、Phase 8 では決めない。

## 4. default-alignment の非対称（Grid `stretch` / ZStack `center`）— public draft で explicability debt を判断（**引き込み**）

[phase-7b handoff §Known carry-forward candidates 第5項（Default-alignment unification）](../../phase-7b/implementation/handoff.md)。
surface を `slot.*` に統一しても **default semantics は統一されない**：Grid
の align default は `stretch`、ZStack は `center`、default は container ごとに
所有される。

**Phase 8 への効き方:**

- handoff の re-trigger は**条件付き**である: 「public draft を書くときに、
  この default 非対称（Grid=`stretch` / ZStack=`center`）が外部読者にとって
  **real explicability debt だと判断されたら**、default-alignment unification
  を再検討する」。つまり Phase 8 到達で自動的に unification 検討が**発火する
  わけではない**——public draft（A12）を書く Phase 8 でこそ、**この非対称が
  説明可能かを評価する責務が発生する**、というのが正確な読み。
- Phase 8 の制約（評価フロー）:
  - 現状 default（Grid=`stretch` / ZStack=`center`）を public draft に
    **正確に記述**する。
  - その非対称が外部読者にとって説明可能か / 説明負債かを**明示判断**する。
  - **説明可能**なら Phase 8 は documentation accuracy で閉じる。
  - **説明負債**と判断したら、unification を future layout-behavior phase へ
    **residual として送る**。
  - いずれにせよ **Phase 8 では default 統一（=layout-behavior change）の
    実装はしない**。

## 5. placement key/value の綴り（`h-align` → `hAlign` 等）— public-draft stabilization で affirmative に keep/revise を決める（**引き込み**）

[phase-7b handoff §Known carry-forward candidates 第6項（Placement key/value spelling revision）](../../phase-7b/implementation/handoff.md)。
既存の綴り（`h-align` / `v-align` / `row-span` 等）は変更なく継承された。
re-trigger は「DSL naming-convention / ergonomics pass、**または
public-draft stabilization**」。

**Phase 8 への効き方:**

- Phase 8 が出す **first public draft** が、外部読者が依存し始める最初の
  公開 surface である。public draft 後に綴りを変えるのは breaking change に
  なるため、Phase 8 は **pre-publication 最後の改訂チャンス**。re-trigger の
  「public-draft stabilization」はここで発火すると解釈する。
- Phase 8 の制約: inherited spelling を**維持するか revise するかを
  affirmatively 判断**する（silent carry 不可）。これは plan の M4-material
  reservation 規律「Phase 8 must act affirmatively / 沈黙は default 維持だが
  permission 行使は積極的行為」（[plan.md Phase 8 行](../../plan.md)）と同型。
  判断は framing/ADR（綴り変更が author surface を変えるなら A13/A12 への
  影響を評価）で確定する。Phase 8 推奨は「inherited spelling 維持」が default
  だが、それを**沈黙でなく明示**で選ぶ。

## 6. per-phase verification surface の Phase 8 close cleanup（placement-demo ほか）（**引き込み**）

[phase-7b handoff §Mid-phase-surfaced residuals 第2項（Phase-8 removal of placement-demo）](../../phase-7b/implementation/handoff.md)
と第4項（capture-driver coordinates）。T5 placement-demo sub-screen
（`is_placement_demo_open` state + button + `if`-overlay）と capture driver
`evidence/capture-placement-demo.ps1` は throwaway な検証足場で、**Phase 8
removal がマーク済み**。

**Phase 8 への効き方:**

- handoff が re-trigger を **「Phase 8 close cleanup sweep that removes the
  per-phase gallery verification surfaces」**と明示。対象は P5 Footer clip /
  P6/7 lightbox / P7 reactive list / **P7b placement-demo** の各 per-phase
  検証 sub-screen。これは A1 = full gallery assembly の一部であり、Phase 8
  in-scope の実装作業。
- 連動 sub-item: A1 assembly は gallery layout を変えるため、**残す capture
  script があれば layout-coupled coordinates を再導出**する（handoff 第4項。
  placement-demo capture driver 自体は除去対象だが、原理は他の retained
  script に効く）。full gallery 組み立て時に per-phase 検証 surface を sweep
  し、coordinate 依存 script を再導出する（A1 assembly の実装作業）。

## 7. placement の bindability / backward-compat positioning — public draft の記述正確性（**引き込み（軽量）**）

[phase-7b handoff §Known carry-forward candidates 第4項（Bindable placement）](../../phase-7b/implementation/handoff.md)
と reconciliation 表「Backward-compatibility guarantee」行。placement は
constant-per-instance で binding RHS は named diagnostic で reject。bare
ZStack `h-align` / `v-align` も reject（firing test）。

**Phase 8 への効き方:**

- A12 public draft は placement を **constant**（将来 bindable になりうる
  concept だが現状は literal/constant、binding RHS は reject）と**正確に**
  記述する。これは再決定ではなく editorial accuracy。
- public draft は **「first public draft」**であって stability commitment
  ではない（公開互換コミットは M6）。draft を「綴り・surface が永続確定」と
  誤読させない positioning にする（§3 PM-2 provisional flag と同じ系統）。
  bindable 化実装・compat policy 確立そのものは前送り維持。

## 8. plan-level の Phase 8 義務（M3 close フェーズとして）（**併記**）

引き継ぎ源は handoff ではなく [plan.md](../../plan.md) だが、Phase 8 が M3
最終フェーズである以上、§2–§5 の判断はこれらの plan-level 規律の上に乗る。

- **public draft で future surface を Phase 8 が affirmatively に明示判断する**
  という**作法は共通**だが、**根拠は項目ごとに別**であり一つの規律の
  application として束ねない:
  - **M4 material reservation**（根拠 = [plan.md Phase 8 行](../../plan.md)）:
    M4 material の syntax を public draft に予約するかを、Phase 8 が「予約する
    / declined と記録する」で明示判断する。沈黙は「予約しない」default。
  - **`width` / `height` sizing reservation**（§2。根拠 = 本 handoff +
    [author-controllable-sizing.md](../../../../docs/notes/author-controllable-sizing.md)）:
    M4 material ではなく Phase 2 以降残る author-controllable sizing 問題。
    Phase 8 framing で Vision DR を起票し、public draft で Fill-default sizing
    を final と誤読させず future surface として位置づける。
  - **placement spelling**（§5。根拠 = 本 handoff）: public-draft
    stabilization が綴り変更の最後の機会なので、keep/revise を沈黙でなく
    明示で決める。
- **milestone-end criteria**（[plan.md §Milestone-end criteria](../../plan.md)）:
  A1–A13 全 discharge / 3 deliverable（A10・A1・A12）/ CHANGELOG / per-phase
  spec sync の auditable 性 / **external-reader smoke**（dsl_spec だけで M3
  surface を再現できるか）/ **silently deferred M3 surface が無いこと** /
  clean rebuild CI green。とくに「silently deferred surface 無し」は §2–§5 の
  「public draft で final と書かない」判断と表裏。
- **A11 per-phase sync** は Phase 8 でも継続: `.ui` / IR / `wasamoc` /
  runtime / `dsl_spec` / `examples/gallery/` が同期して閉じる。
- milestone close では handoff の deferred 群が **M3 `handoff.md`** へ fold
  される（PM-2 wrapper-rule、Problem B Vision DR の最終確定先など）。Phase 8
  close（§6.3）→ milestone close（§7.2）の引き継ぎ経路を維持する。

これらは framing §2.2–2.4 と closing §6/§7 の進め方に反映する。

## 9. プロセス学び（final-step ownership 分割 / pin-to-landed-source / 一見一制約=二問題）

[phase-7b handoff §Main Learnings](../../phase-7b/implementation/handoff.md)。

- **last-task / phase-end ownership split は維持**: 最終 task が Moment 2
  docs sync / local clean rebuild / plan 行 flip / candidate ledger を所有、
  phase-end batch が CI run id / handoff finalization / phase-end retro /
  preamble flip を所有。Phase 8 の `implementation/plan.md` 最終 task
  checklist は最初からこの分割で書く。
- **living spec は landed source に pin する**（§1 と同根）: Phase 8 の
  editorial pass は dsl_spec / architecture の **landed 定義を読む**。status
  flip だけで draft の sketch を凍結しない。
- **「constraint finding」は独立した 2 問題でありうる**: 7b の Grid-in-ZStack
  は checker bug（問題A、T6b で fix 済み）**と** sizing gap（問題B、§2 で
  carry）だった。Phase 8 が triage する際、checker/surface の問題と sizing の
  問題を 1 ラベルに束ねない。

**反映先:** 実装計画 §4 / クロージング §6。

---

## 前送り維持 — Phase 8 action なし（理由付き）

handoff / framing 正本の deferred items のうち、final-phase 批判検討の結果
**Phase 8 で discharge すべき M3 責務を持たない**ものを、理由付きで前送り
維持する。正本は
[phase-7b framing §Out of scope](../../phase-7b/requirements/framing.md) /
[phase-7b handoff reconciliation 表](../../phase-7b/implementation/handoff.md)。

| Deferred item | 前送り維持の理由（なぜ Phase 8 制約にしないか） | 責務先 |
|---|---|---|
| Public code-construction API / ABI（FD-7b-D） | Phase 8 は API/ABI を追加しない。public draft は author-facing DSL surface であり code-construction surface を記述しない。非コミット制約（generic child property setter で placement を表さない）は M6 ABI prep が owner。 | Future code-construction phase / M6 ABI freeze prep |
| VS-2 / VS-3 `SlotData` carrier trigger | 内部 model（`architecture.md` に同期済み）の話で author surface（A12）に影響しない。trigger（第3 placement container / non-layout parent-data）は未発火。 | Future container / M4+ input-accessibility phase |
| Grid structural-mutation trigger（DD-M3-P7-006 recursive） | storage は 7b で `SlotData` へ移行済みだが mutation path（`for`/`if` of `Cell`）は未構築。Phase 8 は新 mutation を作らない。 | Future Grid mutation phase |
| Generic modifier system | handoff reconciliation が「no Phase 8 action」明示。styling/behavior modifier を placement 構文に混ぜない。 | Future DSL ergonomics / styling phase |
| User-defined containers / custom slot attrs | 同上「no Phase 8 action」。custom slot 衝突予約は PM-2 wrapper-rule（§3）と VS-2 trigger に乗る。 | Component / custom-layout phase |
| Non-layout parent-data | VS-3 trigger（hit-test / focus / accessibility）。M4+。A12 author surface 不変。 | M4+ input / accessibility phase |
| Keyed child metadata / retained identity | placement（structural parent-child edge）とは別問題。`key:` surface は child-slot record が開くものでない。 | Future keyed-identity / reorder phase |
| Layout algorithm changes | **Problem B（§2）とは別物**: Problem B は既存アルゴリズム上の missing author *surface*、こちらは geometry 自体の変更。Phase 8 は新 measure-arrange を作らない。混同しない。 | Future layout-primitive-refinement phase |
| Backward-compat **guarantee**（旧 placement syntax の stable 宣言） | first public draft は stability commitment ではない（§7）。公開互換コミットは M6。external user 依存 / 公開 docs が stable 宣言したとき再オープン。 | Pre-1.0 compatibility-policy phase / M6 |

---

## 前送り対象に含めないもの（pointer のみ）

- **placement surface の doc-folded semantics**（spec を直接読む）— `slot.*`
  author surface、Grid `Cell` + 直接 `slot.*`（PM-2）、child-slot
  `SlotData`、bare ZStack `h-align`/`v-align` の reject はいずれも
  [docs/dsl_spec.md](../../../../docs/dsl_spec.md) §4.16 /
  [docs/architecture.md](../../../../docs/architecture.md) §6.8.6 / §6.8.4 に
  fold 済み。Phase 8 が editorial に磨く base であり、constraints へ転記せず
  spec を直接読む（§1）。
- **DPI runtime 修正 / dynamic Window title / lightbox modal 入力**（いずれも
  M4 owned）— [DD-V-022/023](../../../cross-milestone/decisions/dpi-awareness-m4-deferral.md)
  + roadmap M4 AC。full gallery assembly の evidence 解析時に「既知の M4
  残課題」として注記するに留める（Phase 6/7 と同じ扱い）。Phase 8 の A1 は
  Box + Text placeholder + plain text Button で構造経路を証明し、real image /
  click-to-open / modal focus は M4 のまま。
