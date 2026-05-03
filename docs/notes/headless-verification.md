---
title: ヘッドレス検証 — 必要性の批判的検討
status: live
created: 2026-05-04
related-adrs:
  - docs/decisions/m2-phase-3-handler-exec-location.md
related-notes:
  - docs/notes/verification-environments.md
---

# ヘッドレス検証 — 必要性の批判的検討

## 背景

[verification-environments.md](./verification-environments.md) が整理した通り、
wasamo の検証環境は 3 種類:

| 種別 | 環境 |
|---|---|
| Build | 任意の Rust toolchain (CI runner / SSH dev box / local) |
| Link / static export | MSVC toolchain (SSH 可) |
| GUI / interactive | 可視 Windows desktop (RDP / 物理) |

GUI 検証は人間がピクセルを見てマウス/キーボードを操作する必要がある。
CI では実行できず、SSH 経由でも不可。

M2-Phase 3 の検証計画 (handler evaluator) を立てる際にこの制約が再浮上した。
unit test (pure logic) と GUI 手動検証の間に **「runtime を起こして state
transition だけ観察する」中間層が無い** ため、phase 完了の close 条件が
"unit test 緑 + 次 phase で統合確認" に圧縮されがち。

本ノートは「ヘッドレス検証機構を整備すべきか」を批判的に検討する。
**結論先出し: M2 内では構築しない。長期的にも一般目的のヘッドレス
backend は反対。phase ごとに narrow な test fixture を足す方針を支持。**

---

## 候補

### (i) Win32 mock backend (HWND / Compositor / DirectWrite を全 stub)

**反対.**

- [CLAUDE.md](../../CLAUDE.md) Testing rules:
  > Win32/WinRT code (window creation, Compositor, Visual Layer,
  > DirectWrite): do **not** mock the OS API surface.
  との直接衝突。
- mock と実 backend の divergence は典型的な「mock では緑、実機で破綻」
  パターンを生む。M1 で Visual Layer 採用を決めた理由 (DD-V-001 系列で
  確認された "Visual Layer に近接させ抽象を入れない" 方針) とも矛盾。
- maintenance 二重化 — 全 widget / 全 ABI surface に対し mock を保つ必要が
  あり、コストが線形に増える。Slint / Iced のヘッドレス backend は
  別レンダリングパスをライブラリ作者が維持する重い投資で成り立つ。
  wasamo の規模ではペイしない。

### (ii) "no-Compositor" runtime mode (tree + property + signal だけ動かす)

**部分的に肯定可能だが M2 では不採用.**

- 「Visual Layer 抜きで内部 state machine だけ駆動する mode」は概念的に
  成立する。signal dispatch, property storage, layout 計算 (の純粋部) は
  Compositor 不要。
- ただし `wasamo-runtime` に「Visual あり」「Visual なし」二系統を抱える
  ことは DD-V-001 系列で守ってきた posture を侵す。導入には独立 ADR が
  必要。
- 必要性は phase ごとに異なる:
  - Phase 3 (handler evaluator): pure logic は unit test で足りる。
    Visual Layer を起こす必要はそもそもない。→ 不要。
  - Phase 4 (tree-mutation ABI): C ABI 経路の検証は dumpbin + 既存 DLL
    で satisfiable。→ 不要。
  - Phase 5 (reactive engine): 「property 書込 → binding 再評価 → 新値が
    widget に書込まれる」までは Visual Layer 不要で観察可能。**ここで
    初めて検証ギャップが顕在化する可能性**。
  - Phase 6 (`.ui → runtime` lowering): full pipeline の e2e 検証。
    GUI 観察が本質 (ボタン押下で表示文字が変わる、というのが
    acceptance A1/A2 そのもの)。ヘッドレスでは A1/A2 を満たせない。
- → Phase 5 着手時に「reactive 経路の検証が unit test 単独で足りるか」
  を再評価し、足りなければそこで初めて "no-Compositor" mode の独立 ADR を
  起こす。**先回り構築は反対**。

### (iii) Pure-logic 層を直接 Rust API で叩く unit-test 戦略

**支持. これが正解.**

- handler evaluator (Phase 3), binding evaluator (Phase 5), textual IR
  parser (Phase 6), tree-mutation primitive (Phase 4) — いずれも pure
  logic として切り出せる surface を持ち、Visual Layer 抜きで unit test
  可能。
- CLAUDE.md testing rule に正対:
  > Pure Rust logic (parsers, layout algorithms, coordinate math):
  > write unit tests.
- general-purpose な「ヘッドレス backend」を組まず、phase ごとに必要な
  test fixture (fake `EvalContext`, fake listener list, in-memory
  property store 等) を都度整備する。
- 各 fixture は phase 固有 — backend として共通化しない。共通化を狙うと
  (i) や (ii) に寄り、上記の問題を抱える。

---

## なぜ「ヘッドレス backend を作りたい」気持ちが起こるか

phase 完了 close 条件が薄く感じる時に「test 環境の不足」と誤帰属しがち
だが、実際には:

- **pure logic の test surface を切り出せていない** ことが多い
  (Phase 3 の dispatcher 順序を fake listener list で test するアイデアは
  ADR 検討中に後付けで出てきた。最初の plan では unit test の対象に
  入っていなかった)
- **Visual Layer 越しでないと見えない property** (実際の rendering 結果,
  hover animation timing, IME 結合) と **見えなくてもいい property**
  (state transition, dispatch order, formatter 出力) を混同している

→ phase の verification gap を analyze する時はまず「これは Visual Layer
   越しでしか見えないか?」を問う。NO ならヘッドレス backend ではなく
   pure-logic test fixture で閉じる。

---

## 当面の方針 (M2)

- ヘッドレス backend は構築しない。
- phase ごとに verification gap を identify し、pure-logic 部分は narrow な
  test fixture で覆う。
- Visual Layer 越しでしか見えない部分は GUI 手動検証 (RDP / 物理) で残す。
- CI は build 緑 + unit test 緑のみ保証。GUI 検証は phase close 時に owner
  が手動実施。

## 再評価トリガ

以下が起きたら本ノートを再評価し、必要なら ADR (新規 DD) を起こす:

1. **Phase 5 (reactive engine) 着手時** — reactive 経路の検証が unit test
   単独で覆えないと判明したら、"no-Compositor" mode の独立 ADR を検討。
2. **M3+ の DSL surface 拡大時** — widget 数 / signal 種類が増え、test
   fixture の維持コストが pure-logic 切り出しでは追いつかなくなった場合。
3. **post-M2 hot-reload 検討時** — hot-reload の loop を CI で回したい
   要求が出た場合 (M2 では out of scope だが、要求が顕在化したら別問題
   として再検討)。
4. **bindings 自動 conformance test 要求** — Swift / Go 等の community
   binding が公式入りする post-1.0 で、各 binding が同じ振る舞いをする
   ことを CI で証明する仕組みが必要になった場合。これはヘッドレスより
   binding-level の e2e test の話なので、本ノートとは別の検討になる
   見込み。

## 参考

- [verification-environments.md](./verification-environments.md) —
  3 種別 (build / link / GUI) の整理。本ノートはここに「ヘッドレス
  state-only」という第 4 種を足すかの検討。
- [m2-phase-3-handler-exec-location.md](../decisions/m2-phase-3-handler-exec-location.md) —
  検討の発端となった ADR。Phase 3 verification gap を契機に本ノート起草。
- [CLAUDE.md](../../CLAUDE.md) Testing rules — Win32/WinRT mock 禁止の
  根拠。
