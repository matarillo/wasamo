---
title: Release distribution — open questions
status: live
created: 2026-05-22
related:
  - docs/architecture.md
  - docs/abi_spec.md
  - docs/decisions/m2-phase-2-wasamoc-output-format.md
---

# Release distribution — open questions

このノートは、Wasamo を初めて外部向けに release する際に決めなければならない
配布形態についての open questions を inventory する parking lot である。
現時点では「いずれ議論する」ことだけが合意されており、各 question に対する
答えは未定である。

ノートのスコープは **ユーザー利便性に直接影響する観点** に絞っている。
release 運用の内部プロセス（cadence / 権限 / CHANGELOG フォーマット / yank
仕組み 等）や bikeshed 寄りの細目（artifact 命名 / bundle 単位 / PDB 同梱 等）
は外してある。これらは ADR 起こし段階で別途 inventory する。

現状の配布前提については
[../architecture.md §1 "DSL build pipeline"](../architecture.md#dsl-build-pipeline-m2-phase-6-onward)
が "provisional / M2 acceptance gate のための expedient" と明示しており、
M3 DSL spec drafting / multi-`.ui` host / hot reload を re-evaluation
triggers として挙げている。本ノートは同 triggers を踏まえつつ、配布
artifact と downstream consumer 経路まで対象を広げた question inventory
である。ADR 化の trigger は §G で論じる。

---

## A. 何を配布するか (what)

1. `wasamoc.exe` を release artifact に含めるか
2. `wasamo.dll` + `wasamo.h` + `wasamo.lib`（C ABI バンドル）を release artifact に含めるか
3. Rust 向けバインディングを別配布物として用意するか（公開 lib crate / build helper crate / CLI 経由のみ のいずれか）
4. Zig 向けバインディング配布形態は何か（Zig package manager / source bundle / 配布しない）

## B. どのチャネルで配るか (channel)

5. GitHub Releases を canonical channel とするか
6. 言語別 package registry（crates.io / Zig package index 等）も併用するか

## C. ホスト側からどう取得・発見するか (discovery)

7. 各ホスト言語の build system が `wasamoc.exe` をどう discovery するか統一規約を置くか（PATH / env var / per-build download）
8. エンドユーザに明示的な bootstrap install step を要求するか、build system 経由で transitively pull させるか

## D. version / 互換性 (compatibility)

9. version 番号を全 artifact 単一にするか、artifact ごとに semver を切るか
10. wasamoc IR format version と wasamo.dll runtime version の互換性をどう release に表現・gate するか
11. ABI 安定性 promise を release version にどう紐付け、[../abi_spec.md](../abi_spec.md) にどう書くか

## E. プラットフォーム / 法務 (constraints)

12. サポート アーキ・OS の宣言（x64 Windows のみか、arm64 / 非 Windows の扱い）
13. 最低 Windows version と MSVC runtime / VC++ Redistributable 依存をどう宣言するか
14. 第三者ライセンス attribution および project LICENSE / NOTICE の同梱方法

## F. CI / 配布運用 (release ops minimum)

15. tag push で release artifact を build する CI workflow を新設するか
16. Windows Authenticode による code signing を要件にするか

## G. 再評価トリガ (when)

17. このノートを 決定 / ADR 化する trigger は何か。候補：
    - first external user の発生
    - M4 = 1.0 cut の準備
    - hot reload 着手（post-1.0、[M2-Phase 2 wasamoc-output-format decisions](../../process/milestone-2/phase-2/decisions/preamble.md) 参照）
    - multi-`.ui` host の登場（[../architecture.md §1](../architecture.md#dsl-build-pipeline-m2-phase-6-onward) 既存 trigger）
    - 上記のいずれが先か、複数の組合せで起動するかは未決

---

## 質問間の相互依存（読み手注意）

これらは独立な軸として並べているが、いくつかは相互依存している。
ADR 化の際は分離可能な部分集合に切り分ける必要がある：

- **A3 ↔ C7**: Rust 配布形態を決めると `wasamoc.exe` discovery 規約が
  制約される（lib crate を公開すれば Rust は CLI discovery 不要、CLI 経由
  のみなら統一 discovery 規約が必要）
- **A1/A2/B5/B6 → C8**: 配布する artifact と channel の組合せが、エンド
  ユーザに bootstrap install step が要るかどうかを実質決める
- **F16 → signing 周辺**: code signing を要件にしなければ signing key
  管理体制の question は moot になる
- **D9 → D10/D11**: version 番号方針が IR × runtime 互換表現と ABI promise
  紐付けの前提となる

## 非スコープ

以下は本ノートでは意図的に扱わない。必要になった時点で別 note または ADR
で起こす：

- release 運用の内部プロセス（cadence / 権限 / CHANGELOG / yank 仕組み）
- bikeshed 細目（artifact 命名 / bundle 構造 / PDB 同梱 / examples 同梱）
- reproducible build / telemetry 宣言 / auto-update（YAGNI または答えが自明）
- 新規 contributor の clone → first-build 体験（development UX で release UX とは別軸）
- vcpkg / Conan port 個別検討（B6 の language registry 一般項に subsume）
