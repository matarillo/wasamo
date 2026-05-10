---
title: M2-Phase 7 / DD-M2-P6-011 実装ノート
status: completed
created: 2026-05-10
---

# M2-Phase 7 / DD-M2-P6-011 実装ノート

このノートは、DD-M2-P6-011 の実装ステップで使った作業仮説と検証ログを置く場所である。
正式な決定は ADR に、進捗状態は phase progress に蒸留する。

対象は Accepted Option B (`HandlerExpr::StrPropRead`) によって、String-typed
property binding を `.ui` から runtime binding evaluator まで通すことである。

## 現在の作業仮説

- H1: 既存の `PropRead` は i32 read form として残し、String read は
  `StrPropRead` で構造的に分ける。
- H2: `.ui` からの String read form は、wasamoc lowering が state namespace の
  declared type を見て interpolation part を `StrPropRead` に落とす。
- H3: Runtime loader は `str-prop-read` S-expression を parse / validate するが、
  full typed-expression validation は M2 では導入しない。
- H4: Cross-type read は coercion せず、既存の registry/error shape に従って
  missing typed bucket lookup (`UnknownProperty`) として失敗させる。

## 重要な未検証点

- Q1: `EvalContext` に String read surface を追加しても、既存 handler tests と
  `NullEvalContext` / test stubs の blast radius は小さく保てるか。
- Q2: String interpolation の一部として `StrPropRead` を評価しつつ、既存の
  integer interpolation (`PropRead`) をそのまま regression-protect できるか。
- Q3: `.ui` source から emitted IR の `str-prop-read` を経て runtime loader の
  `HandlerExpr::StrPropRead` へ round-trip できるか。
- Q4: Runtime widget property state の確認は、new Visual Layer CI fixture なしで
  binding writer surface まで自動化できるか。

## 検証ログ

- 2026-05-10: `wasamo-ir::HandlerExpr` に `StrPropRead { path }` を追加。
  `wasamoc::emit` / runtime IR parser / test renderer は `(str-prop-read name)` を扱う。
- 2026-05-10: `EvalContext` に `get_string` と `read_string_tracked` を追加。
  default 実装は untracked / unknown とし、既存の test-only context への影響を抑えた。
- 2026-05-10: `BindingEvalContext` が `SignalRegistry.strings` から String を
  untracked / tracked に読むようにした。`HandlerEvalContext` には untracked
  `get_string` を足し、handler mutation の i32 path は変更していない。
- 2026-05-10: `evaluate_binding` が bare `StrPropRead` と interpolation 内
  `StrPropRead` を String read path に dispatch するようにした。
  `evaluate` / integer `evaluate_tracked` では `StrPropRead` を TypeMismatch として拒否する。
- 2026-05-10: wasamoc lowering は interpolation の state ref を namespace で確認し、
  `string` state なら `StrPropRead`、それ以外なら既存 `PropRead` を出す。
- 2026-05-10: `.ui` String binding の cross-crate round-trip test を追加。
  `state label: string` と `text: "State: \{root.label}"` が emitted IR の
  `(str-prop-read label)` になり、runtime parser で同じ IR に戻る。
- 2026-05-10: Runtime binding evaluator の String propagation は
  `register_binding_with_writer` の pure-logic test で固定した。これは production
  `register_binding` と同じ evaluator / Effect / tracked Signal path を使うが、Win32 /
  Composition の live widget construction は既存 phase-close GUI checkpoint に委ねる。
- 2026-05-10: `cargo check --workspace` は green。既存の `wasamo` crate-type warning は
  観測されたが今回の DD-011 差分由来ではない。

## 実装中の決定

- IR text tag は既存 `(prop-read name)` と並ぶ `(str-prop-read name)` とした。
- `get_string` / `read_string_tracked` は trait default を持つ。String を扱う production
  context (`BindingEvalContext`, `HandlerEvalContext`) は明示実装し、その他の context は
  existing unknown-property behavior にフォールバックする。
- Bare `.ui` identifier (`text: label`) は既存の keyword / ident literal ambiguity があるため、
  DD-011 の `.ui` proof は interpolation syntax (`"\{root.label}"`) を使う。
- `lower_rhs_expr` の handler-side dynamic string interpolation は M2 handler evaluator が
  String expression mutationを扱わないため、今回の A6 proof path には含めない。

## 蒸留結果

- `wasamo-ir/src/lib.rs`: `HandlerExpr::StrPropRead` を追加。
- `wasamoc/src/lower.rs`: declared state type に基づき String interpolation ref を
  `StrPropRead` へ lowering。
- `wasamoc/src/emit.rs`: `(str-prop-read ...)` emission と regression test を追加。
- `wasamo-runtime/src/handler.rs`: String read surface と binding evaluator dispatch を追加。
- `wasamo-runtime/src/reactive.rs`: `BindingEvalContext` の tracked String read と
  String binding propagation tests を追加。
- `wasamo-runtime/src/ir_loader.rs`: `(str-prop-read ...)` parse / validate と parser test を追加。
- `wasamo-runtime/tests/ir_loader_roundtrip.rs`: `.ui` String binding の emitted-IR round-trip test を追加。
