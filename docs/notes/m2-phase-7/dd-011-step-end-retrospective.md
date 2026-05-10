---
title: M2-Phase 7 / DD-M2-P6-011 step-end retrospective
status: recorded
created: 2026-05-10
scope: step-end
dd: DD-M2-P6-011
---

# M2-Phase 7 / DD-M2-P6-011 step-end retrospective

## Scope

DD-M2-P6-011 (String-typed property binding) の step-end retrospective。
対象は Accepted Option B (`HandlerExpr::StrPropRead`) の実装であり、M2-Phase 7
の A6 (Type-Agnostic Reactive Binding) を閉じるための step である。

対象コミット:

- `70a0cde feat(ir): add DD-011 string property read form`
- `73df424 feat(wasamoc): lower string state reads for DD-011`
- `07f6be4 feat(runtime): track DD-011 string bindings`
- `fb59bb4 docs(m2): record DD-011 implementation`
- `5c59d81 test(runtime): add live WidgetNode headless proof`
- `d07e5de docs(runtime): record live WidgetNode verification`

この retrospective は step-end の gate であり、phase-end retrospective ではない。

## Current Judgment

2026-05-11 時点では、DD-011 の step-end 基準は **達成済み** と判定する。

当初の blocking item は、DD-011 / A6 の evidence が
`register_binding_with_writer` までで十分か、live `WidgetNode` property state まで
要求するかの判断だった。その後、後者を採用し、
`wasamo-runtime/tests/live_widgetnode_headless.rs` を追加した。この test は
`GITHUB_ACTIONS` 上では runtime-compositor-unavailable skip path を failure にするため、
CI green を `wasamo_init -> build_widget_tree -> wasamo_get_property` まで到達した
evidence として扱える。

したがって、step-end retrospective の unresolved owner decision は解消済みであり、
残る作業は Phase 7 closing items であって DD-011 step-end の blocker ではない。

## Main Learning

今回の中心的な学びは、DD-011 の Option B が「型付き evaluator 全体の導入」
ではなく、「String read form を既存の integer read form と並べて、A6 に必要な
経路だけを実証する」変更として実装できたことだった。

特に有効だった分割は次の通り。

- `PropRead` は i32 専用の既存経路として残す。
- String state read は `StrPropRead` として IR 上で構造的に分ける。
- `.ui` からの String read form は wasamoc lowering が checked namespace の
  declared type を見て選ぶ。
- runtime evaluator は `BindingEvalContext` の tracked String read を
  `SignalRegistry.strings` / `Signal<String>::get()` に流す。

この形により、既存の counter-style handler mutation、integer `PropRead`、
integer interpolation を大きく触らずに String binding を追加できた。

同時に、検証粒度については注意点が残った。当初の自動テストは
`register_binding_with_writer` までの production binding core を通して
`Signal<String>` の初回値・更新値が writer に渡ることを確認している。一方で、
live `WidgetNode` を構築して `widget_write_property` から実 property state を読む
テストはまだ追加していなかった。DD の「runtime widget property state」という表現に対し、
この writer-surface proof を十分な step evidence とみなすかは owner-facing な確認点だった。
その後、`wasamo-runtime/tests/live_widgetnode_headless.rs` を採用した。これは live
`WidgetNode` まで到達する Windows-only headless integration test であり、Local physical
machine と GitHub Actions `windows-latest` では `"State: Ready"` の property state
まで確認できた。GitHub Actions では `GITHUB_ACTIONS` guard により
runtime-compositor-unavailable skip path が failure になるため、CI green は live proof が
実行された evidence として扱える。

`docs/notes/verification-environments.md` に照らすと、ここで混同してはいけない
環境区分がある。GitHub Actions は Windows runner なので build / link / headless
runtime verification には使える。一方で、visible desktop session が必要な GUI /
interactive verification (window visibility, hover, click, keyboard, animation,
pixel observation) は CI では満たせない。したがって、DD-011 の追加 evidence を
検討するなら、「visible desktop を要しない deterministic な Windows-only
headless integration test」と「既存の GUI checkpoint」のどちらに属するかを
先に分類する必要がある。

## Checklist

1. **本作業の主要な学び:** あり。
   - `StrPropRead` を `PropRead` と並べることで、M2 では TypedValue evaluator
     rewrite を避けつつ A6 の String read path を実証できた。
   - `.ui` 由来の型情報は runtime ではなく wasamoc lowering 側で反映するのが
     もっとも局所的だった。
   - 検証は cross-crate round-trip と runtime binding core の pure-logic tests に加え、
     Windows-only headless runtime integration test で live widget property state まで
     自動化した。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:** **なし**
   - 変更した文書は Phase 7 progress、DD-011 implementation notes、
     DD-011 retrospective、verification environment note、project convention note。
   - `dsl_spec.md` などの仕様文書は変更していない。

3. **ローカル clean rebuild:** **green**
   - `cargo clean`: green (`Removed 3049 files, 841.7MiB total`)
   - `cargo build --release --workspace`: green
   - `cargo build --workspace`: green
   - `cargo test --workspace`: green

   Notes:

   - release/debug build と test で既存の `wasamo` crate-type warning が出た。
   - clean 後の release/debug build で既存の `wasamo-sys` import-library ordering
     warning が出た。
   - いずれも今回の DD-011 差分由来ではない。

4. **PO に相談すべき設計判断・トレードオフ:** **解消済み**
   - DD-011 の implementation requirement 4 は「runtime widget property state」
     への到達を求めている。
   - 当初の自動テストは `.ui -> emitted IR -> runtime parser` と
     `Signal<String> -> BindingEvalContext -> evaluate_binding -> binding writer`
     を確認しているが、live `WidgetNode` property state そのものは確認していない。
   - `widget_write_property` は production `register_binding` で使われる既存 writer
     だが、今回の新規テストは Win32 / Composition 依存を避けるため
     `register_binding_with_writer` の mock writer を使った。
   - `docs/notes/verification-environments.md` の区分では、CI Windows runner は
     build / link / headless runtime verification に適しているが、visible desktop
     session を要する GUI / interactive verification には適さない。
   - したがって追加 evidence は、まず headless で deterministic にできるかを
     判定する必要がある。`WidgetNode::text` / `WidgetNode::button` の construction が
     `Compositor` / `TextRenderer` / DirectWrite state に依存するため、CI で安定に
     作れるかは別途確認が必要。
   - 追加実験として `exp/m2-p7-live-widgetnode-headless-test` 上で
     Windows-only headless integration test を作成した。SSH dev box 相当の環境では
     `wasamo_init` が `0x80070005 (Access denied)` で失敗し、`RoInitialize` の
     明示追加でも解消しなかったため、この環境では "runtime compositor unavailable"
     と分類する。
   - 同じ test を SSH ではない Local physical machine で再実行したところ、
     `cargo test -p wasamo-runtime --test live_widgetnode_headless -- --nocapture` は
     green (`1 passed`) で、runtime compositor unavailable の skip message は出なかった。
     したがってこの環境では `wasamo_init` / `build_widget_tree` / live `WidgetNode`
     property read が通り、`wasamo_get_property` で `"State: Ready"` まで確認できた。
   - そのため、SSH dev box は build / link / pure runtime logic の確認には使えるが、
     live `WidgetNode` construction を含む headless proof には十分ではない可能性がある。
     一方で Local physical machine は、この Windows-only headless proof に必要な
     runtime compositor capability を満たすことを確認済みである。CI runner で同じ
     capability があるかは、必要なら別途再分類する。
   - その後、実験ブランチを origin に push し、GitHub Actions manual CI run
     <https://github.com/matarillo/wasamo/actions/runs/25630928372/job/75234360458>
     で `cargo test --workspace` 内の `tests\live_widgetnode_headless.rs` を確認した。
     `string_binding_reaches_live_widgetnode_property_state` は `ok` / `1 passed` だった。
     通常の CI log は test output を capture するため、この時点では
     runtime-compositor-unavailable skip path が通らなかったことまでは log 単体から
     断定しない。
   - `GITHUB_ACTIONS` guard 追加後の manual CI run
     <https://github.com/matarillo/wasamo/actions/runs/25631782149/job/75236689367>
     でも `cargo test --workspace` 内の同 test は `ok` / `1 passed` だった。
     guard 後は runtime-compositor-unavailable path が GitHub Actions 上で failure になるため、
     GitHub Actions `windows-latest` はこの headless proof に必要な runtime compositor
     capability を満たす環境として分類できる。
   - 採用後の扱い: live `WidgetNode` まで到達する headless Windows test を
     `wasamo-runtime/tests/live_widgetnode_headless.rs` として追加したため、DD-011 / A6 の
     automatic evidence は writer surface だけでなく runtime widget property state まで
     到達する。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・構造変更:** **なし**
   - 意味的な変更は DD-011 の IR / lowering / evaluator / binding tests に限定。
   - ただし、触った Rust file に `rustfmt` をかけたため、一部ファイルでは既存の
     未整形箇所も formatting churn として差分に含まれている。これは構造変更ではないが、
     review 時にはノイズとして注意が必要。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - TypedValue evaluator rewrite は既に post-M2 open question として
     `docs/notes/typed-value-evaluator.md` に残っている。
   - 今回新しく phase ADR に切るべき DD は見つかっていない。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed -> Accepted への昇格:** **なし**
   - DD-M2-P6-011 は実装前に Accepted 済み。
   - 今回の step で新しい Proposed DD は追加していない。

8. **`m2-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・分割:** **なし**
   - A6 の内容変更なし。
   - Phase 7 構成変更なし。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:** **なし**
   - `StrPropRead` は runtime parser / evaluator / wasamoc lowering / emitter に実装済み。
   - 新規 `dead_code` warning は観測していない。
   - live widget property state までの追加 gate は
     `wasamo-runtime/tests/live_widgetnode_headless.rs` として採用済み。

10. **タスクリストの後続 step 見直し:** **必要**
    - `docs/plans/progress/m2-phase-7-progress.md` は DD-011 を implemented として
      更新済み。
    - item 4 の owner disposition は、live `WidgetNode` headless integration test を
      採用する形で解消済み。
    - Phase 7 closing items (`CHANGELOG.md`, `ROADMAP.md`, `m2-plan.md` completed 化、
      phase progress の蒸留) はまだ残っている。

## Fast-Track Judgment

Fast-track criteria は、当初記録時点では item 4 により **満たしていない**。
2026-05-11 に live `WidgetNode` headless integration test を採用した後の現在判定では
**満たしている**。

- item 2: なし
- item 3: green
- item 4: 記録時点では **あり**。2026-05-11 に
  `wasamo-runtime/tests/live_widgetnode_headless.rs` を採用して解消。
- item 5: なし
- item 6: なし
- item 7: なし
- item 8: なし
- item 9: なし

blocking item は item 4 だった。DD-011 / A6 の自動検証を
`register_binding_with_writer` までで十分とみなすか、live widget property state
への追加 evidence を要求するかについて owner 判断が必要だったが、2026-05-11 に
後者を採用した。

現在は DD-011 step-end の blocker は残っていない。

## Verification Notes

DD-011 実装で追加した主なテスト:

- evaluator:
  - `binding_bare_string_prop_read`
  - `binding_interpolation_string_prop_read`
  - `binding_rejects_string_read_in_integer_context`
  - `evaluate_rejects_str_prop_read_in_handler_context`
- reactive runtime:
  - `binding_ctx_string_reads_are_tracked`
  - `binding_ctx_get_string_untracked_vs_tracked`
  - `register_binding_writes_string_signal_initial_and_updates`
- runtime IR parser:
  - `binding_with_str_prop_read`
- wasamoc lowering / emission:
  - `dynamic_string_interp_uses_str_prop_read_for_string_state`
  - `string_state_binding_emits_str_prop_read`
- cross-crate `.ui` / emitted-IR / runtime parser:
  - `string_state_binding_emits_and_parses_str_prop_read`
- adopted Windows-only live `WidgetNode` headless proof:
  - `string_binding_reaches_live_widgetnode_property_state`
  - SSH dev box では `wasamo_init` が `0x80070005 (Access denied)` を返し、
    test は runtime compositor unavailable として通過した。
  - Local physical machine では skip されずに `wasamo_init` から
    `build_widget_tree` / `wasamo_get_property` まで到達し、`"State: Ready"` の
    property state を確認した。
  - GitHub Actions `windows-latest` の manual CI run
    <https://github.com/matarillo/wasamo/actions/runs/25630928372/job/75234360458>
    でも `cargo test --workspace` 内で `ok` / `1 passed` だった。この通常 CI log
    だけでは captured skip output の有無までは evidence にしない。
  - `GITHUB_ACTIONS` guard 追加後の manual CI run
    <https://github.com/matarillo/wasamo/actions/runs/25631782149/job/75236689367>
    でも `ok` / `1 passed` だった。guard 後は runtime-compositor-unavailable path が
    GitHub Actions 上で failure になるため、この CI green は live proof が実行された
    evidence として扱える。

Commands run for this retrospective:

```text
cargo clean
cargo build --release --workspace
cargo build --workspace
cargo test --workspace
cargo test -p wasamo-runtime --test live_widgetnode_headless -- --nocapture
```

すべて成功した。`live_widgetnode_headless` command は当初
`exp/m2-p7-live-widgetnode-headless-test` 上で実行し、その後
`wasamo-runtime/tests/live_widgetnode_headless.rs` として採用した。まず SSH dev box を
runtime-compositor-unavailable と分類し、その後 Local physical machine では
その skip path を通らずに成功した。さらに GitHub Actions manual CI run でも
同じ test が `ok` / `1 passed` となった。`GITHUB_ACTIONS` guard 追加後は、CI 上で
runtime-compositor-unavailable path が通ると failure になる。

## Follow-Up

Owner 確認事項だった A6 evidence の扱いは、live `WidgetNode` を作る
Windows-only headless integration test を採用する形で解消した。

- `wasamo-runtime/tests/live_widgetnode_headless.rs` は visible desktop session を要求しない。
- SSH dev box では `0x80070005` により runtime compositor unavailable として skip する。
- GitHub Actions `windows-latest` では、`GITHUB_ACTIONS` guard により同じ skip path が
  failure になるため、CI green は live proof 実行の evidence になる。
- production `widget_write_property` の手前までを M2 自動テストの上限とする案は採らず、
  `wasamo_init -> build_widget_tree -> wasamo_get_property` までを自動 test gate に含める。
