---
title: M2-Phase 7 / DD-M2-P6-012 実装ノート
status: completed
created: 2026-05-10
---

# M2-Phase 7 / DD-M2-P6-012 実装ノート

このノートは、DD-M2-P6-012 の実装ステップで使った作業仮説と検証ログを置く場所である。
正式な決定は ADR に、step-end retrospective は
`docs/notes/m2-phase-7/dd-012-step-end-retrospective.md` に、進捗状態は
phase progress に蒸留する。

対象は、Accepted Option C (role-specified defense in depth) を既存実装に
反映することである。DD-M2-P6-012 は broad guard-token rewrite ではなく、
既存の guard 配置を次の責務分担に揃える実装ステップとして扱う。

- ABI boundary は diagnostic boundary として、公開 `wasamo_*` 関数名、
  `WasamoStatus`、`wasamo_last_error_message` を所有する。
- Internal runtime boundary は invariant boundary として、ABI を通らない
  runtime-owned entry からも state invariant を守る。
- Runtime-owned non-ABI entry は例外ではなく first-class runtime entry として
  internal invariant boundary を通る。
- Cleanup / destroy path は divergence 後に許可される明示的な lifecycle
  exception として扱う。

## 現在の作業仮説

- H1: 既存の `abi.rs` の guard helper / macro は、ほとんどの exported
  `wasamo_*` に対して diagnostic boundary として機能している。
- H2: `emit::drain_if_outermost()` は、Win32 message loop から呼ばれる
  non-ABI entry の internal invariant boundary として既に近い形で実装されている。
- H3: DD-012 の実装価値は、広い API rewrite よりも guard placement の穴を
  監査し、focused tests で境界を固定することにある。
- H4: Cleanup exception は実 window / widget を作らず、null destroy path を使えば
  OS side effect なしに divergence 後許可の性質だけを検証できる。

## 重要な未検証点

- Q1: exported `wasamo_*` の中に、state touch 前に divergence diagnostic boundary
  を通っていないものが残っていないか。
- Q2: void ABI (`wasamo_run`, `wasamo_quit`, `wasamo_shutdown`) は status を返せないが、
  divergence 後の no-op / diagnostic policy をどう表現すべきか。
- Q3: `drain_if_outermost()` の `IN_DRAIN` suppression と `RuntimeHealth::Diverged`
  suppression は unit test で OS / Compositor なしに固定できるか。
- Q4: cleanup exception の検証は、real Win32 destruction を伴わずに書けるか。

## 実装セッションの初手

1. `abi.rs` の exported `wasamo_*` を structural / mutating / read /
   lifecycle cleanup / void lifecycle に分類する。
2. `emit::drain_if_outermost()` に対して、Diverged suppression と nested-drain
   no-op を pure unit test で固定する。
3. ABI diagnostic boundary の薄いズレがあれば、status-returning ABI と
   void ABI の既存作法に合わせて補正する。
4. Phase progress は、code + tests が入ったあとに DD-012 implemented として更新する。

## 検証ログ

- 2026-05-10: Guard placement audit を実施。多くの exported ABI は
  `guard_structural!`, `guard_mutating!`, または explicit
  `check_owning_thread` / `check_not_diverged` 経由で既に diagnostic boundary を
  通っていることを確認した。
- 2026-05-10: `emit::drain_if_outermost()` は、`IN_DRAIN` 中の re-entrant call を
  no-op にし、`RuntimeHealth::Diverged` では全 phase を suppress するため、
  accepted Option C の internal invariant boundary として整合していることを確認した。
- 2026-05-10: Unit tests のために `reactive::set_runtime_health_for_test()` を追加した。
  これは test-only seam であり、production API surface は増やしていない。
- 2026-05-10: `emit` unit tests を追加。Diverged 中の `drain_if_outermost()` が
  queued callback work を drain せず、Phase 1 / Phase 3 flag を立てないことを確認した。
  また `IN_DRAIN` 中の nested drain が work を外側 drain に残し、outer flag を
  clear しないことを確認した。
- 2026-05-10: ABI audit で、void lifecycle entries の `wasamo_run` /
  `wasamo_quit` が divergence diagnostic boundary を通っていない薄いズレを確認した。
  両方に `check_not_diverged` を追加し、void return の既存 wrong-thread policy と同じく
  last-error を残して no-op で戻る形にした。
- 2026-05-10: ABI unit test を追加。Diverged 後の `wasamo_run` /
  `wasamo_quit` が last-error diagnostics を残して戻ること、null
  `wasamo_window_destroy` / null `wasamo_widget_destroy` が cleanup exception として
  `WASAMO_OK` を返すことを確認した。
- 2026-05-10: `cargo test --workspace`, `cargo build --release --workspace`,
  and clean rebuild sequence (`cargo clean` -> release build -> debug build ->
  workspace test) を実行し、すべて green。既存の `wasamo` crate-type warning と
  `wasamo-sys` import-library ordering warning は観測されたが、今回の DD-012 差分由来ではない。

## 実装中の決定

- `wasamo_run()` / `wasamo_quit()` は divergence 後に no-op とし、
  `check_not_diverged()` 経由で last-error diagnostics を残す。void ABI なので
  `WasamoStatus` は返せないが、wrong-thread void policy と同じ診断チャネルを使う。
- `wasamo_window_destroy(NULL)` / `wasamo_widget_destroy(NULL)` を cleanup exception の
  unit test stimulus として使う。real window / widget destruction は OS /
  Composition side effect を伴うため、この step の focused guard-placement test からは外す。
- `reactive::set_runtime_health_for_test()` は `#[cfg(test)]` 限定とする。
  Runtime health を直接操作する production API は作らない。
- `emit` tests は `Pending::Signal { token: 0, args: Vec::new() }` を queue に積む。
  Diverged / nested-drain suppression を見るテストなので、registry lookup が成功する必要はない。

## 蒸留先

- 実装が ADR どおり進んだ場合: `docs/plans/progress/m2-phase-7-progress.md` の
  DD-M2-P6-012 実装チェックを更新する。
- 実装中に Option C から逸脱する新しい guard placement rule が必要になった場合:
  `process/milestone-2/phase-7/decisions/preamble.md` の DD-012 へ戻すか、新しい pre-doc
  cycle を開く。
- Step-end gate の結果は `docs/notes/m2-phase-7/dd-012-step-end-retrospective.md` に記録する。

## 蒸留結果

- `wasamo-runtime/src/emit.rs`: `drain_if_outermost()` の Diverged suppression と
  re-entrant no-op を unit tests で固定済み。
- `wasamo-runtime/src/abi.rs`: `wasamo_run()` / `wasamo_quit()` が divergence 後に
  last-error diagnostics を残して no-op になるよう整合済み。cleanup exception も
  null destroy path で test 済み。
- `wasamo-runtime/src/reactive.rs`: test-only `set_runtime_health_for_test()` を追加済み。
- `docs/plans/progress/m2-phase-7-progress.md`: DD-M2-P6-012 を Accepted and implemented に更新済み。
- `docs/notes/m2-phase-7/dd-012-step-end-retrospective.md`: step-end retrospective を記録済み。
