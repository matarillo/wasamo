---
title: M3-Phase 1 / T14 step-end retrospective
status: recorded
created: 2026-05-19
scope: step-end
task: T14 — Reject bool state interpolation in string bindings
---

# M3-Phase 1 / T14 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-1-progress.md` の **T14** ("Reject bool
state interpolation in string bindings") の step-end retrospective。
T14 は phase-end implicit-constraint review で見つかった最後の
language-surface gap を閉じるための retroactive task である。

目的は、`bool`-typed state を string interpolation に入れた source が
`wasamoc check` を通過し、後段の runtime evaluator で
`TypeMismatch` になる、という遅い失敗をなくすこと。Phase 1 の
language surface としては、`bool` は bool-typed property binding
(`Button.enabled`) と bool state への handler assignment だけに許可し、
string 表示変換は明示 surface が設計されるまで禁止する。

対象コミット:

- `75e4417 fix(m3-phase-1): reject bool string interpolation`
- `e4fe023 docs(m3-phase-1): close T14 bool interpolation follow-up`

対象成果物:

- `wasamoc/src/check.rs`
- `wasamoc/src/lower.rs`
- `docs/dsl_spec.md`
- `docs/plans/progress/m3-phase-1-progress.md`
- `docs/notes/m3-phase-1/phase-end-retrospective.md`
- `docs/notes/m3-phase-2/predoc-inputs.md`

## Current Judgment

T14 は達成済み。

- `wasamoc check` が string interpolation placeholder を走査するとき、
  `QualifiedName` の解決先が `bool` state なら compile-time
  diagnostic を出すようになった。
- 追加 test `check::tests::bool_state_in_string_interp_rejected` は、
  `state ready: bool = true; Text { text: "ready=\{root.ready}" }`
  形の fixture を拒否することを確認している。
- 旧 T4 lowering test
  `bool_state_ident_in_string_interp_lowered_to_bool_prop_read` は削除した。
  T14 後の正規 pipeline では `parse -> check -> lower` の `check`
  で拒否されるため、「bool interpolation が `BoolPropRead` に lower
  される」ことを期待する test は古い仕様を固定してしまう。
- `docs/dsl_spec.md` は document version 0.5 -> 0.6。M3-Phase 1 では
  `bool` state の string interpolation は compile-time error であり、
  implicit bool-to-string formatting/display conversion は存在しないと
  明記した。
- `docs/notes/m3-phase-1/phase-end-retrospective.md` と
  `docs/notes/m3-phase-2/predoc-inputs.md` に、後続 expression /
  formatting work への input として「bool の display conversion は
  explicit surface が必要」を前送りした。
- `docs/plans/progress/m3-phase-1-progress.md` の T14 checklist と
  verification log を complete にした。

## Main Learning

今回の学びは、**新しい scalar を既存 expression container に入れられる
ことと、その container が意味的に受け取ってよいことは別**、という点。

Phase 1 の T4 時点では、string interpolation parts も namespace を見て
typed `*PropRead` を選ぶようにしていたため、`bool` state placeholder は
機械的に `BoolPropRead` へ lower できた。型付き lowering としては自然
だが、string interpolation の evaluator は `i32` / `string` の表示値を
結合する surface であり、`bool` の display policy は ADR でも spec でも
まだ決めていなかった。

この状態で compile を許すと、source author にとっては「構文も型名も
通ったのに runtime で落ちる」挙動になる。T14 はここを language
surface の境界として明確化し、**formatting / display conversion は
implicit に生やさず、必要になった phase で明示 DD として設計する**
という方針を選んだ。

副次的な学び:

- `lower.rs` に defensive fallback が残っていること自体は問題ではない。
  unchecked caller が直接 lower helper を叩く可能性に対する保険であり、
  正規 pipeline の contract は `check` が source-level diagnostic を
  出すことで成立している。
- `dsl_spec.md` の version bump は小さく見えるが、step-end checklist
  項目 2 の「仕様文書変更あり」に該当する。T14 は fast-track ではなく、
  owner 報告対象として扱うのが正しい。
- Phase-end close 後に implicit constraint を task 化する場合、progress
  file だけでなく phase-end retrospective と次 phase predoc inputs の
  両方へ蒸留し直す必要がある。T14 はその小さな実例になった。

## Verification Notes

- `cargo fmt --all -- --check` — green。
- `cargo clean` — green (`Removed 3317 files, 919.2MiB total`)。
- `cargo build --release --workspace` — green (Finished release
  profile [optimized] target(s) in 40.03s)。
- `cargo build --workspace` — green (Finished dev profile
  [unoptimized + debuginfo] target(s) in 34.54s)。
- `cargo test --workspace` — green。
  - `wasamo-ir`: 7 unit tests passed。
  - `wasamoc`: 98 unit tests + 6 roundtrip tests passed, including
    `check::tests::bool_state_in_string_interp_rejected`。
  - `wasamo-runtime`: 165 unit tests + 9 integration tests passed,
    including `bool_binding_live_propagation` and `button_enabled`।
  - `wasamo-sys`: 1 unit test passed。
  - Host / binding crates with 0 tests passed; doc tests passed with
    0 tests where present.
  - No failures, no ignored tests.
- Known warnings unchanged: `wasamo` crate "provides no linkable target"
  notice and `wasamo-sys` import-library ordering note. These match the
  T12 phase-end clean rebuild warnings and are not build/test failures.

## Retrospective Checklist

scope = step-end (merge -> phase ブランチ)。

1. 主要な学び: 上記 Main Learning。

2. 仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更 —
   **あり**。`docs/dsl_spec.md` を 0.5 -> 0.6 に bump し、bool state
   interpolation rejection を明記した。これは Accepted な DD の単なる
   機械的転記ではなく、Phase 1 language surface の reject set を
   明文化する変更なので fast-track 対象外。

3. プロジェクトルートで `cargo fmt` を実行した上での local clean
   rebuild — **green**。`cargo fmt --all -- --check`、
   `cargo clean`、release/debug workspace build、`cargo test
   --workspace` を T14 後の HEAD で実行し、すべて green。

4. PO に相談すべき設計判断・トレードオフ — **あり**。T14 の判断
   (`bool` の implicit display conversion を入れない) は小さいが
   user-facing language surface の境界を決める。progress file と
   `dsl_spec.md` には反映済みで、owner には「Phase 1 は explicit
   formatting surface なし」として報告する。

5. plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・構造変更
   — **なし**。`lower.rs` の変更は obsolete test の削除のみ。

6. 現在の phase ADR への追加 DD 必要性 — **なし**。T14 は ADR の bool
   scalar binding 方針を変えず、未定義だった interpolation edge を
   compile-time reject に寄せたもの。将来 formatting surface を入れる
   場合は後続 phase の DD として扱う。

7. 既存 ADR の Proposed 項目の新規追加、または Proposed -> Accepted
   への昇格 — **なし**。

8. `m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・分割 —
   **なし**。T14 は progress file の mutable task list に追加された
   follow-up であり、上位 plan の AC や phase breakdown は変更して
   いない。

9. 後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告 —
   **なし**。`wasamoc` test は green、新規 unused code なし。

10. タスクリストの後続 step 見直し — **不要**。T14 で progress file 上の
    remaining implementation task は閉じた。残るのは phase branch の
    owner merge / push gate と、必要なら T14 後の full workspace
    verification をどこで実行するかの運用判断。

判定: 項目 3 は **green**。ただし項目 2 と 4 が **あり**なので、
**fast-track 対象外**。owner への報告と承認が必要。

## Follow-Up

- Owner に T14 の language-surface 判断を報告する:
  `bool` は bool binding / bool handler assignment のみ。string
  interpolation での display conversion は明示 surface ができるまで
  reject。
- T14 後の full workspace clean rebuild は green になったため、残る
  owner-facing gate は T14 の language-surface 判断の承認と phase
  branch の merge / push 判断。
- `docs/notes/m3-phase-2/predoc-inputs.md` §8 は、Phase 2 で即使用しない
  場合でも、Phase 6 以降の expression / formatting surface 設計時に
  参照する。
