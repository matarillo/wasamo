---
title: M3-Phase 1 / T11 step-end retrospective
status: recorded
created: 2026-05-19
scope: step-end
task: T11 — `.ui` fixture and end-to-end host evidence
---

# M3-Phase 1 / T11 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-1-progress.md` の **T11**
(".ui fixture and end-to-end host evidence") の step-end
retrospective。目的は、T1-T10 で通した bool scalar / `Button.enabled`
経路を、実際の `.ui -> wasamoc -> IR -> wasamo_load_ui -> window` の
host 経路で触れる形にすること。

対象コミット:

- `b2433eb feat(examples): add bool binding Rust demo (M3-Phase 1 T11)`

## Current Judgment

T11 は達成済み。

- `examples/bool-demo/bool-demo.ui` を追加。`state ready: bool = true`
  を宣言し、`Button.enabled: ready` を張り、`clicked => {
  root.ready = false; }` で自分自身を disabled にする。
- `examples/bool-demo-rust/` を追加。`counter-rust` と同じ
  build-time compiler path を使い、`build.rs` が `.ui` を
  `wasamoc` 経由で IR にし、host は `wasamo_load_ui` に渡すだけに
  している。
- `wasamoc/tests/roundtrip.rs` に
  `bool_demo_ui_contains_bool_binding_and_handler` を追加し、fixture が
  `state ready: bool = true` / `bool-prop-read ready` /
  `(assign ready false)` を emit することを確認した。
- `Cargo.toml` workspace に `examples/bool-demo-rust` を登録したため、
  T12 の workspace build/test gate に自然に乗る。

## Main Learning

今回の学びは、**可視 proof 用の example は、既存 canonical example を
太らせるより、phase の主張だけを持つ小さな sibling example にした方が
後で読み返しやすい**ということ。

`examples/counter-rust` を拡張すればファイル数は減るが、M2 の
Hello Counter 証拠に M3-Phase 1 の bool 証拠が混ざり、counter が
「何を代表しているか」がぼやける。専用 `bool-demo-rust` にすると、
追加されるものは多いが、差分の意味は非常に単純になる:

- `.ui` fixture が Phase 1 の bool path そのもの。
- Rust host は counter-rust と同型で、新しい runtime imperative
  呼び出しを足していない。
- workspace member なので phase-end の clean rebuild で漏れない。

副次的には、T11 の「visible window」証拠は自動 test に寄せすぎない
方がよい、という整理も残った。`Button.enabled` の mock-free
Windows integration test は T6 で既に CI-gated。T11 はそれと別に、
人間が `.ui` fixture 由来の window を起動し、クリックで grey/inert
になる経路を確認できる host artifact を置くのが役割。

## Verification Notes

- `cargo fmt` — green.
- `cargo test -p wasamoc --test roundtrip` — green (6 passed)。
- `cargo build -p bool-demo-rust` — green。
- `cargo build --release -p bool-demo-rust` — green。
- `Start-Process .\target\release\bool-demo-rust.exe` — command
  succeeded。GUI の目視 click smoke は owner 確認が必要な範囲であり、
  owner から `private/m3-p1-t11 screenshot 2026-05-19 190441.png`
  が証跡として提供された。対応する baseline として
  `private/m3-p1-t10 screenshot 2026-05-19 190455.png` も残っている。

## Retrospective Checklist

1. 主要な学び: 上記 Main Learning。
2. 仕様文書変更: なし。progress file と notes の記録のみ。
3. clean rebuild: T11 では targeted build/test まで実施。full
   `cargo clean` -> release/debug workspace build -> workspace test は
   T12 phase-end gate。
4. PO に相談すべき設計判断: なし。
5. ついでのリファクタ: なし。`wasamoc/tests/roundtrip.rs` の helper
   汎用化は bool-demo fixture を同じ test path に乗せるための局所変更。
6. phase ADR への追加 DD 必要性: なし。
7. Proposed DD 追加/昇格: なし。
8. plan AC 追加/変更: なし。
9. 後続 step に持ち越す仮実装/近似/dead_code: なし。
10. 後続 step 見直し: 不要。T12 は予定通り phase-end gates。

## Follow-Up

- T12 で full workspace release build / full workspace test / CI green /
  Windows-only `button_enabled` integration test の CI pass を確認する。
- T12 の phase-end retrospective では、この `bool-demo-rust` が
  M3-Phase 1 の gallery-sub-screen 相当の可視 proof として十分だったかを
  再確認する。
- Process correction: GUI の「動作した」確認は owner/manual 領域なので、
  Codex 側は launch command 成功までを記録し、クリック後の visible
  behavior は owner に明示確認する。今回の T11 記録は owner-provided
  screenshot を証跡として補正した。
