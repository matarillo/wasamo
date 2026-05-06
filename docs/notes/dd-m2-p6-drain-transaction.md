# DD-M2-P6-XXX (draft) — Drain transaction semantics

**Status:** Draft for M2-Phase 6 (DD 番号は Phase 6 ADR 起草時に確定)
**Date:** 2026-05-06
**Targets phase:** M2-Phase 6 (`.ui → runtime` lowering の pre-doc サイクルで正式採用予定)
**Affects:** DD-M2-P5-004 (drain stage 框組み, 部分 supersede), DD-P6-003 (queued emission, 不変), VISION §4 Principle 2 (任意で補足)
**Background note:** [reactive-drain-cascade-policy.md](./reactive-drain-cascade-policy.md) (open question 側)

---

## 1. 背景

### 1.1 プロジェクトと runtime モデル

Wasamo は Windows 専用の宣言的 UI フレームワーク。`.ui` という外部 DSL で UI を記述し、C ABI を介して任意言語ホスト (C / Rust / Zig 等) から runtime を呼び出す。runtime 本体は単一 DLL (`wasamo.dll`)、レンダリングは Windows.UI.Composition (Visual Layer)。現在は M2 (Foundation) マイルストーン Phase 5 (reactive engine) を実装し終えた段階。

### 1.2 UI スレッドモデルと queued emission

runtime は厳密な UI-thread affinity を持ち、すべての ABI 関数とすべてのコールバックは `wasamo_init` を呼んだスレッドで動く。再入性についての確定済ルール (DD-P6-003 = Option A):

> **ホストが `wasamo_*` 関数の中にいる間、runtime はそのスレッドでコールバックを発火しない。** その呼出から生じた発火は queue に積まれ、呼出から制御がホストに戻った時点で drain される。

このルールを実装する関数が `drain_if_outermost`。「最外」判定はスレッドローカルなネスト深さカウンタで行う。

### 1.3 Phase 5 の reactive engine

`Signal<T>` (観測可能な値セル) と `Effect` (再実行可能な依存追跡クロージャ) を導入。Signal の `get()` は実行中 Effect を依存先として記録し、`set()` は依存している Effect を dirty set にマークして即座に return (= deferred dispatch, DD-M2-P5-004 = Option B)。dirty Effect の再実行は `drain_if_outermost` 内で行う。

dirty Effect の再実行ボディは内部 `set_property` を呼び、property 値の書込みと同時に observer queue にエントリを積む (Phase 3 = Option A 確定済)。size に影響する property は window を「layout dirty」としてマーク。

### 1.4 顕在化した問題

DD-M2-P5-004 は drain を「Observer drain → Reactive drain → Layout drain」の 3 段直列と確定したが、ある経路の挙動を明文化していなかった:

> reactive 段で実行された Effect が `set_property` 経由で property を書込み、その結果として observer queue に新しいエントリが積まれる。本 cycle の observer drain は既に終わっているため、このエントリはいつ消化されるか?

直列 1 回なら次 outermost cycle (= 1 frame 遅延)。3 段全体をループするなら同 cycle。表面的には「1-pass か loop か」の二択に見えるが、後述するように **これは設計軸の選択** の問題であり、二択そのものが問いの立て方として誤っている可能性がある。

---

## 2. 概念整理 — 3 種類のコールバック

Wasamo の host ↔ runtime コールバックは機能的に異なる 3 種を含むが、現行実装ではこれらが queueing 機構を共有しており、混同されたまま設計が進んでいた。これが本問題の遠因である。

| 概念 | 役割 | 現行 mutation 可否 | VISION §4 P2 内の対応 |
|---|---|---|---|
| **Signal handler** | user input event (click, key) を host が処理する場 | 可 | 「events flow up as host-language callbacks」 |
| **Reactive Effect** | declarative な state→property binding | 可 (内部 set_property) | 「state flows down through property bindings」 |
| **Property observer** | host が property 変化を観測する場 (logging, telemetry, 外部 I/O 等) | 可 (現行 ABI) | **直接対応なし** — 派生的にしか導出されない |

VISION §4 Principle 2 は明示的に:

> "view is a pure function of state (`view = f(state)`), state flows down through property bindings, and user interactions flow up as events handled by host-language callbacks."

これに従えば mutation channel は「events up」と「bindings down」の 2 経路のみ。**「property 変化を host が観測してそれに応じて state を変える」経路は VISION の宣言的単方向モデルに本来含まれない**。observer を mutation channel として扱うのは VISION からの派生ではなく、現行 ABI が偶然許してしまっている自由度 (= 暗黙の C2: 「set_property は observer queue を必ず積む」) に過ぎない。

---

## 3. 設計軸 — 何を決めると drain が決まるか

選択肢を列挙する前に、独立な軸を 4 つ識別する。これらの組合せが drain を一意に定める。

### 軸 α — フレーム境界の所在

Wasamo は React Fiber や SwiftUI のような「render pass = フレーム」概念を持たず、**ABI 呼出境界**がフレームの単位となっている。これが多くの既存 UI フレームワークと異なる構造的特徴。同 frame / 次 frame という議論は実際には「同一 ABI 呼出内で観測可能か」という意味になる。本 DD はこの構造を変えない (変えると VISION C ABI 中心思想と衝突するため)。

### 軸 β — observer の意味論

- **β1**: observer = 同期 mutation channel (任意 ABI 再呼出可、観測内で state mutation 可)
- **β2**: observer = post-commit な mutation 可能 channel だが、次 frame に deferred (現状の「直列 1 回」に近い実装上の帰結)
- **β3**: observer = post-commit pure effect (state-mutating ABI 不可、外部 I/O のみ可)

### 軸 γ — set_property と observer enqueue の結合

- **γ1**: 即時 enqueue (現行 C2: set_property が呼ばれた瞬間に observer queue へ積む)
- **γ2**: deferred enqueue (set_property は property 値と dirty マークのみ更新; observer enqueue は別 phase で diff から構築)

### 軸 δ — 収束保証レイヤの分離

- **δ1**: dependency graph 収束 (reactive 内部) と event propagation 収束 (外側) を同一 pipeline で処理。単一 cap または重層 cap。
- **δ2**: 両者を別 phase に分離。reactive 段は dependency graph 収束のみ責任を持ち、event propagation は別 phase で処理。

これら軸は独立で組合せ可能だが、論理的に共起しやすい組合せがある (例: β3 なら observer の cascade が原理的に存在しないため δ は意味を失う)。

---

## 4. 選択肢

以下 6 つを検討する。

### Option A — 単純直列 1-pass [β2, γ1, δ1]

3 段を直列に 1 回だけ呼ぶ。reactive 起源の observer 通知は次 outermost cycle で消化 (1 frame 遅延)。

- **得るもの**: 現行 Phase 5 実装と一致 (実装コスト 0)。停止条件最も単純。ADR (DD-M2-P5-004) の文言に最も忠実。
- **失うもの**:
  - **経路非対称性 = 意味論の経路依存性**: host→set_property→observer は同 cycle、reactive→set_property→observer は次 cycle。これは単なる「観測タイミングの揺れ」や「FRP 系で起きやすいバグ」ではなく、**フレームワークの意味論が経路依存になる** ことを意味する。同じ property に同じ値を書いても、書いた経路によって observer に対する到達タイミングが変わる。これは declarative model における「property 変化」という概念の意味そのものを壊す。
  - **位置付けは「暫定」ではなく「仕様的逸脱」**: `view = f(state)` を標榜するフレームワークが、内部経路ごとに異なる通知遅延を持つ semantics を採用するのは技術的負債の許容ではなく **原理的な設計の不正** であり、本来は採用するならその逸脱を仕様として正面から記載しなければならない。
  - `drain_if_outermost` 戻り時にシステムが quiescent state にない (pending observer が存在しうる)。
- **位置付け**: 設計として正しい選択肢ではない。他のすべての選択肢が時間的・実装的に間に合わない場合の緊急避難ラインに留まる。
- **VISION 整合**: 弱い。observer が mutation channel である前提を維持し、かつ経路依存性を持ち込むため §4 P2 と単に緊張するのではなく **構造的に矛盾する**。

### Option B — 3 段外側ループ [β1, γ1, δ1]

「observer queue 空 ∧ dirty Effect 空 ∧ layout dirty 空」が同時成立するまで 3 段全体をループ。reactive 内部ループの cap (=16) と外側 cap が並存する二重構造。

- **得るもの**: reactive 起源 observer 通知が同 cycle で消化される (経路非対称性解消)。
- **失うもの**:
  - **収束レイヤ混在**: dependency graph 収束 (純粋関数的) と event propagation 収束 (副作用込み) を同じ pipeline に直列で載せることが制度化される。両者は性質が異なるため「どこで止めるか」の意味が曖昧になる。
  - **二重 cap**: 発散経路の診断が困難 (どちらの cap が effect しているか毎回判定要)。
  - 実装表面積 (code + 仕様文書) が増える。
- **VISION 整合**: 弱い (observer mutation 前提を維持)。

### Option C — 統合 side-effect drain [β1, γ1, δ2 (ただし pipeline 統合)]

Observer drain と Reactive drain を「副作用キュー」として統合し、両者を交互に消化する単一ループで収束まで回す。Layout はその後 1 回。

```
loop while observer_queue ≠ ∅ OR dirty_effects ≠ ∅:
    drain one observer entry  (host callback fires, may call ABI)
    drain one dirty Effect    (re-runs, calls internal set_property)
    iter += 1; if iter > CAP: error-break
layout drain (1 pass)
```

- **得るもの**: 経路非対称性解消。**単一 cap** (二重化問題消滅)。`drain_if_outermost` 戻り時の quiescent state を構造的に保証する。3 段構造を 2 段に圧縮するため表面積も減る。
- **失うもの**:
  - **mutation graph に「非宣言的エッジ」が混入する**: observer は `.ui` から見えない host-side の mutation 経路を作る。declarative な dependency graph (Signal の依存関係) と event propagation graph (observer chain) が **同じ pipeline で混ざる** ため、システム全体の振舞いが declarative graph 単独からは静的に決まらなくなる。M3 で binding 数が増えれば「動的に何が起きるか」の予測がツール (LSP, devtool) からも難しくなる。
  - **構造的不安定性**: React Fiber や Compose が observer (effect) を commit 後の別レーンに分離している理由はここにある。両者を同期トランザクションで混ぜることは、それらの先行例があえて避けてきた設計選択。Option C はその回避を**逆向きに踏み外す**選択であり、「中庸」ではなく**前例的に不利な道**。
  - **「動的決定論」と「構造的決定論」の混同**: 表面上は経路非対称性が消えて quiescent state も担保されるが、それは「実行時に収束する」という動的性質であって、「設計レベルで非対称性が定義不能である」という構造的性質ではない。前者は実装が変わると揺らぐ、後者は揺らがない。
- **VISION 整合**: 中弱。表面上の不変条件は満たすが、graph 構造を壊す方向であり、長期的には不安定。

### Option D — Declarative transaction + post-commit pure observer [β3, γ1 or γ2, δ2]

Observer を **post-commit pure effect** と再定義 (state-mutating ABI 不可)。drain は 3 phase:

```
Phase 1 (mutation convergence, loop):
    while signal_queue ≠ ∅ OR dirty_effects ≠ ∅:
        drain signal handler (host event handler, may mutate state)
        drain dirty Effect   (re-runs, calls internal set_property)
        cap = MUTATION_CAP

Phase 2 (layout, 1 pass)

Phase 3 (post-commit observers, terminal 1 pass):
    drain observer queue
    runtime sets TLS "in-observer" flag during each callback
    state-mutating ABI called while flag set → WASAMO_ERR_OBSERVER_MUTATION
```

- **得るもの**:
  - **VISION §4 P2 を ABI 表面で構造的に enforce**: mutation channel は events up / bindings down のみ。observer は read-only 観察 + 外部 I/O。
  - 経路非対称性が原理的に存在しない (どの経路の observer も Phase 3 で発火)。
  - 単一 MUTATION_CAP。
  - **収束レイヤの真の分離**: Phase 1 = state mutation 収束 (signal + reactive 統合)、Phase 2 = view consistency、Phase 3 = pure side effects。各 phase の責任が明確。
- **失うもの**:
  - **既存 UI モデルとの構造的断絶**: 「観測 → 状態更新」を許す既存パターン (MVVM の `INotifyPropertyChanged` → ViewModel mutation, Cocoa KVO callback → state update, DOM `MutationObserver` → DOM mutation など) は **すべて Wasamo では書けなくなる**。これは慣習からの移行コストではなく、**エコシステム設計レベルの制約**。binding 著者・外部統合コード・ツール系コードの三方面で確実に摩擦が発生する。VISION の「OSS 貢献」「多言語ホスト」原則と部分的に緊張する。
  - 「observer が次フレームでの mutation を post する」escape hatch は本 DD では未設計 (open question)。Option F を後追いで加える経路は自然だが、その間 host 側で workaround が必要になる期間が生じる。
  - 新 error code 1 個追加。TLS flag コード追加。
- **失うものの引き受け方**: 上記摩擦は無視できないが、Wasamo は「Slint × XAML × multi-language」の**新しい組合せ**を主張するフレームワークであり、既存の「観測 → 更新」パターンに迎合せず declarative model の構造的整合を選ぶことは、差別化の核心にむしろ整合する。摩擦は許容コストであり、binding 著者向けのドキュメントで「Wasamo では reactive Effect に書け」というガイドを明示することで緩和する。
- **VISION 整合**: 強い。Phase 3 で mutation 不可なので戻り時に確実に静止する quiescent state も同時に達成。ただし下記注記参照 — 採択は VISION の意味の **強化** を含む。

**注記 — VISION との関係 (双方向の正直さ)**: 「VISION が正しいから D を選ぶ」のではなく、**「D を選ぶことは VISION §4 P2 を convention から structural constraint へ昇格させる決定である」**。現状の VISION 文面は「declarative + unidirectional」を方向性として宣言しているが、observer の意味論を ABI 表面でどう扱うかまでは明示していない。本 DD はその空白を **強い側 (mutation 不可)** に埋める仕様強化判断。すなわち VISION の自然な解釈の中に既に含まれていたわけではなく、**設計判断によって VISION の意味を強く固定している**。このことを §11.1 の VISION 補足追加が表現する。

### Option E — Deferred enqueue + observer-can-mutate [β2, γ2, δ2]

set_property は **property 値の更新と dirty マークのみ** を行い、observer queue は積まない。Phase 1 (mutation convergence) 終了時点で diff を取り、変化した property に対する observer を queue へ。Phase 2 (layout) → Phase 3 (observer fire)。observer は state mutation 可だが、その mutation は直接 next outermost cycle に持ち越し (本 cycle 内では reentry しない)。

- **得るもの**:
  - **mutation phase と notification phase の真の構造的分離**: γ2 (enqueue 遅延) によって `set_property` から observer enqueue への即時結合を切り、両者を別 phase に追い出す。これは React Fiber の commit phase ↔ effect phase 分離に**構造的にかなり近い**設計であり、Option A〜C のように両者を同じ pipeline に置く設計と質的に異なる。
  - observer は常に「post-commit な何か」として一貫した意味論を持つ (経路非対称性なし)。
  - observer の自由度を維持 (mutation 可)。
- **失うもの**:
  - **中途半端の本質**: 構造的にはクリーンだが、observer に mutation を残しているため「mutation graph に非宣言的エッジが存在する」という Option C と同じ問題が、phase 分離されただけで残る。次 outermost cycle で処理される deferred mutation という形で graph の影響は伝搬する。**完全な declarative にはならない**。
  - **mental model がやや複雑**: ホスト視点では「observer から set_property してもすぐには反映されず、次の ABI 呼出後に処理される」という非自明な遅延が semantics に組込まれる。Option A の 1 frame 遅延を別位置に押し付けただけにも見える。
  - 実装複雑度: diff 計算または pending property set の管理が runtime 側に増える。
- **位置付け**: 「構造的にはクリーンだが、観測者から見て中途半端」。Option C の「同 cycle 完結」と Option D の「observer mutation 禁止」の中間に立とうとして、両者の利点を半分ずつしか取れていない。「declarative を完成させる」立場からは D に劣り、「observer の自由度を残す」立場からは C より複雑。**両軸で次善**。
- **VISION 整合**: 中。observer mutation の事実は隠蔽されないが、phase 分離によって observer が「同期 mutation channel」ではなく「deferred mutation channel」として整理される点は VISION 整合に寄与。

### Option F — Event-source observer [β3 + post-event API]

Option D を採るが、**初日から escape hatch を組込む**。observer は state-mutating ABI を呼べないが、`wasamo_post_event(event_id, payload)` によって**次 outermost cycle に処理される event を post できる**。投稿された event は signal handler キューに乗り、次 cycle の Phase 1 で処理される。

- **得るもの**: D の利点 (VISION 整合、Phase 分離、単一 cap) + 「observer 内で何かをトリガーしたい」ニーズへの構造化された経路。observer の表現力を D より高める。
- **失うもの**:
  - **概念表面積**: signal handler / property observer / posted event の 3 概念をホストが学ぶ必要。
  - posted event の意味論を本 DD で確定する必要 (D は将来 deferred mutation API として open のまま許容)。
  - 設計コスト前倒し。
- **VISION 整合**: 強い (D 同様)。

---

## 5. 比較表

| 観点 | A | B | C | D | E | F |
|---|---|---|---|---|---|---|
| 経路非対称性 | **あり (意味論経路依存)** | なし | なし | なし | なし | なし |
| 戻り時 quiescent state | × | ○ | ○ | ○ | ○ | ○ |
| iteration cap | 内部 1 個 | 内部 + 外側 (2 個) | 単一 | 単一 (MUTATION_CAP) | 単一 | 単一 |
| 収束レイヤ分離 (軸 δ) | δ1 | δ1 (劣化) | δ2 (pipeline 統合) | δ2 (真の分離) | δ2 (真の分離) | δ2 (真の分離) |
| observer mutation 可 | 可 | 可 | 可 | **不可** | 可 (deferred) | 不可 (post_event は可) |
| mutation graph の declarative 純度 | 破綻 (経路依存) | 維持されない | **非宣言的エッジ混入** | 完全に declarative | 部分的混入 (deferred) | 完全に declarative |
| VISION §4 P2 整合 | **構造的矛盾** | 弱 | 中弱 | **強 (ただし仕様強化を伴う)** | 中 | **強 (ただし仕様強化を伴う)** |
| 既存 UI モデル (MVVM/KVO) との互換 | ○ | ○ | ○ | **× (構造的断絶)** | ○ | △ (post_event 経由) |
| 実装コスト | 0 (現状維持) | 中 | 中 | 中 | 大 | 中大 |
| 既存 ABI 表面変更 | なし | なし | なし | error code 1 個追加 | 内部のみ | post_event API 追加 |
| 設計の位置付け | **仕様的逸脱** (緊急避難) | 表面積増・利得小 | 構造的に不安定 | declarative の構造的完成 | 両軸で次善 | D + 表現力確保 |

---

## 6. 推奨と批判的吟味

### 6.1 推奨 — Option D (仕様強化を伴う採択)

VISION §4 P2 (`view = f(state)`, 単方向) を ABI 表面で構造的に enforce することが、Wasamo を「declarative + 決定論的」フレームワークとして長期的に正しい位置に据えるための最小コストの選択。Option C は表面上の整合性 (経路非対称性解消, quiescent state) を回復するが、observer が mutation channel である限り mutation graph に非宣言的エッジが残り、構造的決定論には到達しない。Option D はその問題を構造的に消去する。

ただし本推奨は「VISION から自然に導かれる選択肢」ではなく、**「VISION の意味を強化することと一体になった設計判断」** であることを明示しておく。すなわち D を採るとは:

- VISION §4 P2 の「declarative + unidirectional」を **convention レベルから structural constraint レベルへ昇格** させる
- Wasamo は「観測 → 状態更新」を許す既存 UI モデル群 (MVVM, KVO 系) と **構造的に断絶した** declarative-first フレームワークである、という立場を確定する
- その断絶を Wasamo の **アイデンティティの一部** として引き受ける (= 慣習踏襲を捨てて差別化軸 「Slint 哲学 × XAML 語彙 × multi-language openness」に declarative-first を加える)

これは VISION の「自然な含意」ではなく、**VISION の解釈を強い側に固定する明示的判断**。この自覚なしに D を採ると、後で「VISION にはそう書いてない」と揺り戻しが起きる。本 DD の VISION §4 P2 補足追加 (§11.1) は、この解釈固定を文書として記録する手段である。

### 6.2 Option D に対する自己批判

ここで本案推奨に対し、可能な限り辛口に批判する:

1. **「observer mutation 不可」はエコシステム設計レベルの制約である**
   - 既存 UI モデルの「観測 → 状態更新」パターン群は、Wasamo では **すべて書けない**:
     - .NET MVVM の `INotifyPropertyChanged` → ViewModel mutation
     - Cocoa KVO callback → state update
     - DOM `MutationObserver` → DOM mutation
     - Reactive Extensions の Subject pattern (host 側で state 同期)
   - 影響範囲は単に「言語ホスト著者の慣習からの移行」ではなく、**3 方面のエコシステム摩擦**:
     - **Binding 著者**: Swift / Go / Nim 等のコミュニティ binding が慣習パターンを移植しようとして衝突
     - **外部統合コード**: state の永続化・ログ出力・analytics と双方向同期するコードを `set_property` 経由で常に書く必要が出る (= 既存の bidirectional sync ライブラリは流用不可)
     - **ツール系コード**: devtool や inspector が「state を読んで書き戻す」操作を行う場合、特別経路が必要
   - これは VISION の「OSS 貢献」「multi-language ホスト」原則と **部分的に正面衝突** する。Wasamo は「declarative-first を貫いた結果、既存ライブラリ群との互換性を一部捨てる」というアイデンティティを引き受ける。
   - **覚悟すべき結論**: D を採ることは「Wasamo は declarative-first フレームワークであり、imperative 統合は別経路 (signal handler or future post_event API) で解く」というスタンスを永続的にコミットすること。撤回するなら DD を分裂させる必要がある。緩和は binding 著者ガイドの提供で行うが、エッジケースで unconditional に解消できるとは保証できない。

2. **「deferred mutation API」が未設計のまま open にされる**
   - 反論可能性: 本 DD は open question として残すが、実際のユースケースが M3 で出てきた時に設計が間に合わない可能性 (= 結果的に Option F を後追いで作る羽目になる)。
   - **対抗オプション**: Option F を初日から採用する。F は D より概念数が増えるが、escape hatch が初日から定義されるため後追い設計のリスクがない。F vs D の本質は「設計コスト前倒し vs 後倒し」のトレードオフ。

3. **`WASAMO_ERR_OBSERVER_MUTATION` の検出は完全か**
   - TLS flag による検出は単純だが、observer callback 内から **別スレッドへ仕事を委譲し、そのスレッドから ABI を叩く** 経路は捕捉できない (Wasamo は UI-thread affinity なので別スレッドから ABI 呼出は元々 error だが、検出位置が変わる)。本検出は「同スレッド同期呼出」のみ。実用上はこれで十分だが、ドキュメント上の説明は注意が要る。

4. **VISION §4 P2 補足を「任意」にしている曖昧さ**
   - 推奨は補足 **追加** の方が首尾一貫する (= 実質 Option A を選んだ場合との差を文書化)。「補足なしでも導出可」と書いて選択を曖昧にしているのは中途半端。
   - **修正方針**: 本 DD 採択時は VISION §4 P2 への補足を **必須** とする。任意性を取り下げる。

5. **Phase 5 実装は既に Option A 相当で完了しており、refactor が要る**
   - これは事実だが、Phase 5 close は完了しており、本 DD は次 phase への申し送り的位置付け。M2-Phase 6 の冒頭で drain refactor を組み込めば実害は小さい。Phase 5 ADR の更新と implementation 修正は併記して 1 commit で済む規模。

### 6.3 Option C を落とす理由 — 構造的不安定性

Option C は表面上の整合性原則 (quiescent state, 単一 cap, 経路非対称性解消) をすべて満たす。しかしこれを推奨から外す理由は次の構造的問題:

**mutation graph に非宣言的エッジが混入する**: observer は `.ui` から見えない host-side の経路で property を書き換える。Option C ではこの observer mutation が reactive Effect の dirty propagation と **同じ pipeline で混ざる**。結果として:

- システムの dependency graph (Signal の依存関係) が静的に決まらない。実行時に observer が何をするかで graph が動的に変わる。
- M3 で binding 数が増えると「動的に何が起きるか」を tool (LSP, devtool) からも推論不能になる。
- React Fiber や Compose が effect を commit 後の別レーンに置く設計選択の根拠は、まさにこの graph 構造の純粋性確保にある。Option C はその先行事例があえて避けてきた構造を**逆向きに踏み外す**。

「動的決定論」 (実行時に収束する) と「構造的決定論」 (設計レベルで非対称性が定義不能) を区別するなら、Option C は前者にしか到達しない。前者は実装変更で揺らぎ、後者は揺らがない。Wasamo を「信頼に足る UI runtime」と位置付けるなら後者を選ぶべき。

**Option C を採るべき条件** (限定的):
- VISION §4 P2 を「方向性の表明」とのみ読み、ABI 表面での enforce を意図的に拒否する場合
- M2-Phase 6 までの implementation 帯域が極めて限定的で、TLS flag + error code 追加のコストすら許容できない場合
- Wasamo の差別化を Visual Layer + 多言語 + Mica に絞り、reactive モデルの構造的純度は二次的目標と位置付ける場合

これらは限定条件であり、Wasamo の現状 VISION (§4 P2 を含む declarative + unidirectional の明示) からは適合しにくい。

### 6.4 Option F は将来の保険

Option F は「D の VISION 整合 + observer 表現力の確保」を両立するが、概念数増加と前倒し設計コストを払う。M3 で observer の実用ニーズが見えた段階で D → F に拡張する経路は自然なので、本 DD では D を採り、F は将来の拡張余地として open question 化するのが穏当。

### 6.5 Option A を切り捨てる覚悟

Option A は「現状維持で M2 を出せる」という実装的魅力があるが、**設計選択としては採るべきでない**。理由:

- 経路非対称性は単なる「観測タイミングの揺れ」ではなく、フレームワークの意味論が経路依存になる **構造的な意味の破綻**。同じ property に同じ値を書いても経路で挙動が変わる semantics は、`view = f(state)` を標榜するフレームワークと矛盾する。
- 「短期的な技術的負債」として処理することは可能だが、それは「正しい設計が判明していて、実装が間に合わないから一時的に劣化版を出す」場合の枠組み。本件はそのような状況ではなく、**正しい設計 (D) が判明し、実装コストも限定的 (TLS flag + error code 1 個)** であるため、A を選ぶ実質的な根拠は乏しい。
- 採るとしても「暫定」「短期」と書くのは不誠実で、**「Wasamo は経路依存 semantics を仕様として持つ」という仕様逸脱を明記** する必要がある。これは VISION §4 P2 を実質的に放棄する宣言になり、フレームワークのアイデンティティに直接ダメージを与える。

**A が採られるべき例外条件**: M2 acceptance を 1 週間以内に出さねばならないなどの極端な期限制約があり、かつ 1 phase 後に必ず D へ移行する commit が文書化されている場合のみ。それ以外は採らない。

### 6.6 Option B / E を落とす理由

- **B**: 二重 cap と収束レイヤ混在を**制度化**する設計。C と同じ利点を表面積大で達成するため、C にも D にも劣る。**積極的に推奨しない**。
- **E**: 構造的にはクリーンだが (mutation/notification 分離は React commit phase に近い)、observer mutation を残しているため Option C と同じ「非宣言的エッジ混入」問題が deferred 形で残る。**両軸 (declarative 完成 / observer 自由度) で次善** であり、特定条件下で意味を持つが、本 DD では D / C / F のどれかが優先される。E が真に有用になるのは「observer mutation を残したいが Option C の同期混合は避けたい」という限定状況のみ。

---

## 7. ドレインの再定義 (Option D 採択時の確定仕様)

```
drain_if_outermost()
  │
  ├─ Phase 1: Mutation convergence  (loop until fixed point)
  │     while signal_queue ≠ ∅ OR dirty_effects ≠ ∅:
  │         if signal_queue ≠ ∅:
  │             pop signal handler, fire host callback
  │             (callback may freely mutate state via ABI)
  │         else if dirty_effects ≠ ∅:
  │             take one dirty Effect, re-run
  │             (effect body calls internal set_property)
  │         iter += 1
  │         if iter > MUTATION_CAP: error-log, break
  │
  ├─ Phase 2: Layout  (1 pass, terminal)
  │     for each layout-dirty window: run_layout
  │
  └─ Phase 3: Post-commit observers  (1 pass, terminal)
        IN_OBSERVER_CALLBACK := true
        drain observer queue;
        state-mutating ABI returns WASAMO_ERR_OBSERVER_MUTATION (panic in debug)
        IN_OBSERVER_CALLBACK := false
```

**戻り時の不変条件**: signal_queue 空 ∧ dirty_effects 空 ∧ layout-dirty 空 ∧ observer_queue 空。`drain_if_outermost` がホストに制御を返した瞬間、システムは完全な静止状態にある (cap 到達による打切り除く)。

---

## 8. 上位文書への影響

| 文書 | 影響 |
|---|---|
| **VISION.md §4 Principle 2** | **必須補足** (Option D 採択時): observer = post-commit pure effect であることを明記し、mutation channel の二重定義を排する。文案は §11.1 に記載。 |
| **DD-M2-P5-004** | 「3 段直列 (observer → reactive → layout)」框組みを supersede。reactive が layout に先行する意図 (size-affecting 変更を同一 layout pass に畳む) は維持。observer が「reactive 前段」から「layout 後段 terminal」へ移動。 |
| **DD-P6-003 (queued emission)** | 不変。「ABI 内コールバック発火しない」ルールは前提として維持。本 DD は callback の **firing timing** ではなく **mutation capability** を扱う。 |
| **DD-M2-P5-006 (mirror struct テストパターン)** | 不変。本 DD の単体テストにも適用可能。 |
| **architecture.md §6.8** | drain 構造を 3 phase + terminal 形式へ更新。observer mutation 制約を明記。 |
| **m2-phase-5-reactive-engine.md** | 「3 段直列」記述を本 DD で置換。 |

---

## 9. M2 acceptance との関係

acceptance criterion A2 (counter シナリオ: click → count++ → bound Text 更新) は全 option で満たされる。観測上、A〜F の差は M2 シナリオでは見えない:

- counter は signal handler 1 個 + reactive Effect 1 個 + layout 1 個 + observer 0 個。
- どの drain 構造でも 1 イテレーションで収束。
- 観測等価。

選択判断は M2 acceptance ではなく、**M3 以降で observer / multi-binding cascade が実用シナリオに乗った時の整合性** に基づく。

---

## 10. 実装影響 (Option D 採択時)

- **`wasamo-runtime/src/emit.rs`**: `drain_if_outermost` を 3 phase 構成へ書換え。signal_queue と observer_queue を概念的に分離 (現状同一 queue であれば種別タグ追加または queue 二本化)。
- **TLS flag**: `IN_OBSERVER_CALLBACK: Cell<bool>` を追加。Phase 3 内で各 callback fire 前後に true/false 切替え。
- **state-mutating ABI のエントリ**: `wasamo_set_property`, `wasamo_emit_signal` ほか、TLS flag check を追加して `WASAMO_ERR_OBSERVER_MUTATION` を返す。
- **新 error code**: `WASAMO_ERR_OBSERVER_MUTATION` を C ABI 仕様に追加。
- **テスト**: pure logic 側に下記を追加可能 (DD-M2-P5-006 mirror パターン):
  - Phase 1 の signal + Effect 交互収束
  - Phase 3 の mutation block 検出
- **C ABI 仕様文書 / architecture / Phase 5 ADR**: §8 の通り更新。

---

## 11. 補助文書

### 11.1 VISION §4 Principle 2 補足の文案 (Option D 採択時必須)

§4 Principle 2 末尾に追加:

> Property observers (host-registered watchers on property changes) are post-commit pure effects: they observe a fully-converged frozen state and perform external side effects (logging, telemetry, I/O) without mutating runtime state. State mutation flows exclusively through user events (signal handlers) and reactive bindings (declarative property bindings). This makes the unidirectional model structurally enforced rather than merely conventional.

---

## 12. 決定 (オーナー判断待ち)

> **Decision:** Option ___ — ___

主要選択肢:

- **D (推奨)**: declarative transaction + post-commit pure observer。VISION §4 P2 補足追加必須。M2-Phase 6 冒頭で実装。エコシステム摩擦 (MVVM/KVO 系との断絶) を Wasamo アイデンティティの一部として正面から引き受ける判断を含む。
- **F (declarative + 表現力両立)**: D + post_event API。設計コスト前倒しと引換えに observer 表現力を確保。エコシステム摩擦は D より緩和される。
- **C (declarative 純度妥協)**: 統合 side-effect drain。observer 自由度維持の代償として mutation graph に非宣言的エッジ混入。**起草者は構造的不安定性ゆえ推奨しない**。
- **A (緊急避難)**: 現状維持 + 経路依存 semantics を仕様逸脱として明記。**起草者は仕様的逸脱として推奨しない**。

起草者の批判的選好順: **D > F > C >> E > A > B**。

D と F の差は「observer 表現力」と「設計コスト前倒し」のトレードオフであり、両者とも declarative 構造的整合は達成する。C と D の間には大きなギャップ (graph 構造の declarative 純度) があり、A は本選択肢から事実上脱落させるべき。

---

## 13. open questions (本 DD では決めない)

- **observer から state mutation を要求するユースケースが M3 以降で見つかった場合の扱い** (D 採択時): deferred mutation API / event posting API / Option F 相当の拡張のいずれを採るかは、実シナリオを見て別 DD で。
- **animation の意味論**: Visual Layer の compositor-driven animation は runtime state を更新しないという前提なら本 DD と無関係。確認要。
- **MUTATION_CAP の値**: 暫定 16 (旧 reactive cap 引継ぎ)。signal handler を含めるとカウント基準が変わるので Phase 6 以降で実測調整。
- **`wasamo_emit_signal` を Effect 内から呼ぶ場合の意味論**: 本 DD では Phase 1 ループで自然に処理されると読めるが、意図的許容/禁止/警告のいずれかは別 DD で明示確定。
- **多言語バインディング著者へのドキュメント設計** (D 採択時): observer mutation 制約は VISION 由来であって ABI 制約ではないことを binding 著者ガイドで明示し、慣習との衝突を最小化する。
