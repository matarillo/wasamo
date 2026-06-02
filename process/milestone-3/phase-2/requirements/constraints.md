---
title: M3-Phase 2 pre-doc inputs — carried forward from M3-Phase 1 close
status: live
created: 2026-05-19
related:
  - docs/notes/m3-phase-1/phase-end-retrospective.md
  - docs/notes/retrospectives.md
  - docs/plans/m3-plan.md
---

# M3-Phase 2 pre-doc inputs

このノートは M3-Phase 1 (`bool` scalar binding) の close 時点で
[docs/notes/retrospectives.md §Retrospective Main Learning の前送り](../retrospectives.md#retrospective-main-learning-の前送り)
に従って書き起こした、**M3-Phase 2 (Box layout primitive) の
pre-doc が input section として取り込むべき材料**である。

M3-Phase 2 の pre-doc 着手時に、このファイルを直接参照するか、内容を
pre-doc の Context / Inputs / 前提 section に折り込むか、要約して
ADR の Context に入れるかは Phase 2 owner-agreed framing 次第。
**書く / 取り込まないの判断はその時点で行ってよいが、本ファイル自体は
M3-Phase 1 close 内に確定している。**

参照元:

- [M3-Phase 1 phase-end retrospective](../m3-phase-1/phase-end-retrospective.md)
  §Main Learning #1–#3 が本ノートの一次蒸留元。
- [M3-Phase 1 progress file](../../phase-1/implementation/plan.md)
  の T1–T12 タスクログと CI / verification log が二次蒸留元。
- 個別の step-end retrospective
  ([t1](../m3-phase-1/t1-step-end-retrospective.md) –
  [t11](../m3-phase-1/t11-step-end-retrospective.md)) は本ノートが
  要約した内容の execution-level 細部を保持する。

---

## 1. Box が新規 `PropertyValue` variant を入れるなら、ABI value-conversion arm は同じ step に fold する

**根拠 (M3-Phase 1 Main Learning #2):** Rust の exhaustive `match` が
`PropertyValue::Bool` 追加時に
`wasamo-runtime/src/{abi.rs, emit.rs}` の value-conversion arms を
mechanically 強制した結果、T9 (ABI value-conversion arms) は独立
step として温存する意味がなくなり、T6 part 1 (commit `a550bd9`) に
fold された。CLAUDE.md §Commit rules の「implementation reveals a
tighter ordering」適用例。

**M3-Phase 2 への適用:** Box は仕様上 `aspect: <ratio>` と最小限の
`fill: <color>` 属性を持つ ([m3-plan.md A6](../../plan.md#acceptance-criteria))。
このうち、

- `aspect: <ratio>` は値型として **新しい数値型** (float / rational) を
  必要とするか、既存 `i32` で代用するかが Phase 2 pre-doc の DD 候補。
  新型を入れるなら `PropertyValue` / `WasamoValue` / `IrType` の三層
  追加が必要で、Phase 1 の bool-fold と同じく ABI arms は同 step に
  fold する。
- `fill: <color>` は **新しい `PropertyValue::Color`** (または既存の
  `Str` 経由で色名 / hex を渡す soft path) のどちらを選ぶかが Phase 2
  の DD 候補。Color variant を入れるなら同じく ABI arms fold が必須。

**判断基準:** ABI public 関数を**新規に増やさない**変更 (内部 variant
追加 + 既存 `wasamo_value_*` の payload 拡張) は、Phase 1 の T9-fold
原則に従って単一 step。ABI public 関数を**新規に増やす**変更
(DD-M3-P1-008 Option B 系) のときだけ独立 step として温存する。

---

## 2. 新しい bindable property は per-type writer seam を ir_loader call site で選ぶ

**根拠 (M3-Phase 1 Main Learning #1 / DD-M3-P1-007):** Phase 1 の
`Button.enabled` は `evaluate_bool_binding` + `widget_write_property_bool`
+ `register_bool_binding` の triple を `ir_loader::build_node` の
`match prop_ty` で選ぶことで、reactive engine 内部に `IrType` dispatch
を入れずに済んだ。これは F5 (`TypedValue`) deferral を構造的に支える
seam であり、Phase 7 反復文法で TypedValue 圧力を再評価するまで
維持する。

**M3-Phase 2 への適用:** Box の `aspect: <ratio>` / `fill: <color>` が
**reactive な値** (state binding) を受け付ける範囲を Phase 2 pre-doc
の DD で決める。受け付けるなら、

- 新しい `evaluate_<T>_binding` / `widget_write_property_<T>` /
  `register_<T>_binding` の triple を engine 外に追加し、
- `resolve_prop_key` の `IrType` 戻り値を拡張し、
- ir_loader の `build_node` で `match prop_ty` の新規 arm を足す。

reactive engine 自体は依然 type-agnostic に保つ。これは「per-type
binding writer seam の場所」決定の継続適用。

**判断基準:** Box の属性が **constant-only** (binding 不可) と
決めるなら、これらの seam 拡張は不要で `IrLiteral` の追加のみ。
constant か bindable かは Phase 2 pre-doc 内の DD として
明示的に固定する。

---

## 3. `cargo fmt` process gap — step checklist 改訂 / CI 強制のどちらを選ぶか

**根拠 (M3-Phase 1 Main Learning #3):** Phase 1 の T6–T8 期に
rustfmt drift (6 ファイル) が蓄積し、各 step retrospective の
「`cargo fmt` — green」表示と矛盾していたが、CI が `cargo fmt --check`
を強制していないため phase-end gate まで未検出だった。応急処置は
commit `1129aea` (fmt-only) で済ませた。

**M3-Phase 2 への適用:** Phase 2 の pre-doc 内、または Phase 2 開始時
の最初の協議材料として、以下のどちらか (または両方) を owner と
決める。

- **(a)** [docs/notes/retrospectives.md](../retrospectives.md) checklist
  項目 3 (clean rebuild) を、`cargo fmt --all -- --check` 単独の
  green 確認を含む形に改訂する。step retrospective の "green" 表記が
  「commit 後の状態に対する `--check` の green」を意味することを明示。
- **(b)** [.github/workflows/ci.yml](../../../.github/workflows/ci.yml)
  に `cargo fmt --all -- --check` の step を追加する。CLAUDE.md
  §CI rules の「Rust コードを既存 crate に追加する phase は CI 更新
  不要」原則の例外で、owner agreement が必要。

**判断基準:** (a) は docs 改訂のみ、(b) は CI YAML 変更を伴う。Phase 2
は新言語 / 新ビルド系を入れないので、CLAUDE.md §CI rules を厳格に
読むと (b) は本来 phase 内で扱えるネタではない。owner judgement で
どちらかを Phase 2 pre-doc DD として閉じる。

---

## 4. 可視 proof は既存 canonical example を太らせず sibling example を立てる

**根拠 (T11 retrospective):** M3-Phase 1 の visible proof は
`examples/counter-rust` を拡張する代わりに `examples/bool-demo-rust/`
を sibling example として立てた。理由は「`counter-rust` は M2
Hello Counter の reference として stable に残す」「sibling example
にすることで Phase の主張だけを抱えた最小差分になる」「workspace
member 登録で phase-end clean rebuild に自然に乗る」の三点。

**M3-Phase 2 への適用:** Box layout primitive の可視 proof は
`examples/box-demo-rust/` (もしくは Box + Text placeholder pattern が
gallery sub-screen の一部を成すなら `examples/gallery-box-rust/`) を
新規に立てるのが既定線。`examples/counter-rust` / `bool-demo-rust`
には触らない。

**判断基準:** sibling example を立てるか、Phase 6+ の gallery
sub-screen への部分組み込みを Phase 2 から始めるかは pre-doc DD。
ただし「既存 example を太らせない」原則は保持する。

---

## 5. GUI smoke は owner manual / Codex は launch command 成功までを記録

**根拠 (T11 retrospective §Follow-Up):** M3-Phase 1 close 時に、
visible window の click behavior 確認は owner 領域として明示分離し、
Codex 側は `Start-Process` の command 成功までを log に残すという
process correction を入れた。

**M3-Phase 2 への適用:** Box の `aspect: <ratio>` 視覚的正しさ、
`fill: <color>` 色の正しさは owner manual smoke の範囲。Phase 2 の
verification strategy に書くときも、headless integration test
(measure / arrange の数値 assertion) と owner GUI smoke を別 gate
として区別する。

---

## 6. Retroactive spec-gap fold は最小範囲で同じ phase に折り込む

**根拠 (T10 retrospective):** Phase 1 の T10 で `dsl_spec.md` の bool
追加と並行して、M2-Phase 6 で `state` 宣言が追加されたのに spec §1–7
に文書化されていなかった gap を、owner-agreed の最小範囲で T10
commit に fold した。

**M3-Phase 2 への適用:** Phase 2 の spec sync (`dsl_spec.md` の Box
記述追加) で、M2 / M3-Phase 1 由来の earlier-phase docs 漏れに気づい
たら、最小範囲で同じ Phase 2 spec sync commit に折り込む。owner
明示確認は必要。これは M3-P1 T10 で確立した規律の継続適用で、
memory entry `feedback_retroactive_spec_gap_fold` でも保持されている。

---

## 7. f32 / f64 を IrType に入れるかの再評価

**根拠 (M3-Phase 1 ADR Context / T4 注釈):** Phase 1 では「float-typed
state idents (if any ever appear) fall through to the static-ident
branch alongside non-state idents, because Phase 1 has no `*PropRead`
variant for `f32` / `f64` and the checker rejects float earlier」と
defensive fallback だけを置いた。実際 wasamoc の `TypeName::Float` は
存在するが、IR / runtime 側の対応はない。

**M3-Phase 2 への適用:** Box の `aspect: <ratio>` が rational (2 整数)
ではなく float literal を許容するなら、`IrType::F32` / `IrLiteral::F32`
/ `HandlerExpr::{F32Lit, F32PropRead}` を追加するかどうかが pre-doc
DD。Phase 1 の bool で確立した type-suffix pattern がそのまま
拡張可能なので、追加コストは予測可能。

**判断基準:** Phase 2 が float なしで閉じられる ( `aspect: "16:9"`
の文字列パース or `aspect: 16, 9` の整数 pair) なら追加しない。float
literal が user-facing surface として必要なら追加する。M3 全体での
float pressure は target-app pre-doc 上は明示されていないので、
default は「追加しない」。

---

## 8. `bool` の display conversion は明示 surface ができるまで禁止

**根拠 (M3-Phase 1 T14):** Phase 1 close 時の implicit-constraint review
で、`bool` state を string interpolation に入れると runtime
`TypeMismatch` まで進んでしまう gap が見つかった。T14 で
`wasamoc check` が `bool`-typed state interpolation を compile-time
error として拒否するようにした。

**M3-Phase 2 への適用:** Box 自体は formatting surface を持たない想定
だが、`fill` / `aspect` の値型設計で「表示用 string への暗黙変換」を
便利機能として足さない。Phase 1 のルールは、`bool` は bool-typed
property binding と bool handler assignment に限る、である。

**後続 phase への送り:** Phase 6 以降で conditional / expression
surface が広がる時、または target app 側で status text が必要になった
時に、`format(...)` / template filter / display trait 相当のどれを採る
かを明示 DD にする。暗黙の bool→string 変換は既定では入れない。

---

## 9. Bool live proof は現行の同期 non-batched drain に依存している

この項目の本体は M3 横断の reactive/drain 前提なので、
[docs/notes/m2-to-m3-handover.md](../m2-to-m3-handover.md) §3 item 4
に移した。M3-Phase 2 の Box layout primitive は通常この前提を直接
触らないが、Phase 2 pre-doc が event/input batching、layout scheduling、
または headless integration proof の boundary を扱う場合は、handover
側の M3-Phase 1 addendum を読む。

---

## 適用方法のサマリ

| Phase 2 pre-doc DD 候補 | 起源 | このノート §           |
|---|---|---|
| `aspect` / `fill` の値型と ABI fold 方針 | T6/T9 fold | §1                  |
| `aspect` / `fill` を bindable にするか | DD-M3-P1-007 | §2          |
| `cargo fmt` 強制方法 (checklist vs CI) | T12 fmt drift | §3        |
| Box 可視 proof の host 配置                  | T11 sibling | §4               |
| visual smoke gate の責任分離                   | T11 §Follow-Up | §5            |
| spec sync 中の retroactive fold 許容           | T10 | §6                       |
| `f32` を IrType に入れるかの再判定           | Phase 1 defensive fallback | §7      |
| bool display conversion / formatting surface を明示化するか | T14 | §8 |
| bool live proof の同期 drain 前提を preserve / 改訂するか | T13 / Follow-up B | §9 |

Phase 2 pre-doc が起こされる時、上表のうち pre-doc framing が触れる
ものを Context / Inputs / Open questions section に取り込む。触れな
かった項目は本ノートが durable な reference として残るので、Phase 5
(Grid) / Phase 6 (ZStack + conditional) などで必要になった時点で
再参照する。
