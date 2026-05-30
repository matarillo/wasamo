---
title: DSL 文法 — 検討メモと未解決事項
status: live
created: 2026-05-07
last-updated: 2026-05-31
related-adrs:
  - docs/decisions/m2-phase-2-wasamoc-output-format.md
  - docs/decisions/m2-phase-6-ir-loader.md
  - docs/decisions/m3-phase-1-bool-scalar.md
related-specs:
  - docs/dsl_spec.md
---

# DSL 文法 — 検討メモと未解決事項

このノートは Wasamo DSL の文法に関するオーナーの検討記録の live note。
確定済みの normative 仕様は `docs/dsl_spec.md` を参照。本ノートは
**未解決の文法上の論点** と、**将来 ADR に昇格しうる検討メモ** を残す。

特に M3 以降では、target app pre-doc / phase pre-doc / DD で直接扱う論点は
そちらを本線とする。本ノートは、M3 の採用案からこぼれた選択肢、M3 では
解かないことにした残余、または M3 完了時に再整理すべき open question の
受け皿として使う。

---

## Open Questions

### Q1. Widget 識別子の IR 表現

**現状（M2-Phase 6 時点）:**

- DSL の `widget_decl ::= IDENT "{" member* "}"` の `IDENT` は widget の
  **type** 名（`VStack`, `Button` など）であり、widget instance を一意に
  指す **id** を付ける構文は存在しない。
- バインディング/ハンドラから参照される `root.count` の `root` は
  component の base（暗黙の self）を指し、`count` は `state` 宣言名。
  すなわち現行 IR で名前解決の対象になりうるのは **`state` 宣言のみ**。

**論点:**

- DD-M2-P6-009（IR loader の防御的検証）のタスク文には
  「every binding/handler name resolves to a declared `state` or widget」
  と書かれており、forward-looking に「widget 名解決」の余地を含めていた。
  しかし現 IR には widget 識別子が無いため、M2 時点の reference resolution
  検証は **state 名のみ** を対象とする。
- 将来、widget instance を式から参照する必要が生じた場合（候補シナリオは
  下記）、IR / DSL の両側に widget id を持ち込む文法拡張が必要になる。

**候補シナリオ:**

- M3 の Grid / List で per-item context を導入する場合
  （[m2-plan.md](../plans/m2-plan.md) の M3 ロードマップでも binding 拡張
  として言及されている）。
- 兄弟 widget の状態を直接参照したいケース（例：チェックボックスの状態に
  応じて別 widget の `enabled` をバインド）。M2 では state を経由するため
  不要だが、ボイラープレートが過剰になれば再考対象。

**未決:**

- そもそも widget id を許す方針を取るか、それとも全参照を `state` 経由に
  揃える方針を貫くか。後者は Elm/SwiftUI 的に「UI = state の純関数」を
  強制することになり、設計上の魅力はある。
- Id を導入する場合の構文（`node Button#submit { ... }` か、
  `id: "submit"` プロパティか、`@submit` か等）。
- 名前解決スコープ（component-local フラットか、ネスト可能か）。

**この議論を再訪する契機:**

M3 の Grid / List 設計で per-item context が必要になった時、または、M2
完了後の retrospective で「state 経由のみ」では表現力が足りない事例が
出た時。

**M3 target app framing からの追記（2026-05-12）:**

- `docs/notes/m3/m3-target-app-wireframes.html` の候補整理により、List item
  template、Grid inside List item template、breadcrumb segments、gallery
  thumbnails のような **template-local / repeated child scope** が M3 の
  target app 候補に直接出てきた。
- これらは widget instance id を直ちに要求するとは限らないが、少なくとも
  `item`, `index`, selected state, nested template-local names などの名前解決を
  Q1 と同じ文脈で再確認する必要がある。
- Q3 はこの再訪からこぼれうる文法残余の置き場である。M3 target app pre-doc や
  phase pre-doc で直接扱う論点はそちらを本線とし、本ノートには M3 で未採用・
  defer・制限された選択肢だけを残す。

---

### Q2. Window 由来の component-level prop の runtime 配線

**現状（M2-Phase 6 / DD-M2-P6-008 時点）:**

- DSL では `component Counter inherits Window { title: "Counter"; backdrop: mica;
  theme: system; ... }` のように Window 由来 prop を component 直下に書ける。
- wasamoc の lowering（[wasamoc/src/lower.rs](../../wasamoc/src/lower.rs) の
  `lower()`）は component-level の `PropertyBind` を root widget node の props に
  splice し、IR 上は root の `prop title = "Counter"` 等として正しく出力される。
- しかし IR loader の `construct_widget`（[wasamo-runtime/src/ir_loader.rs:714](../../wasamo-runtime/src/ir_loader.rs#L714)）
  は `VStack`/`HStack`/`Text`/`Button` の各 widget が認識する prop しか参照せず、
  認識しない prop は黙って drop する（M3 diagnostic system 移送の予告コメントが
  [ir_loader.rs:684-685](../../wasamo-runtime/src/ir_loader.rs#L684) にある）。
- 加えて `wasamo_load_ui` は内部で `window::create("Wasamo", 800, 600)` を固定で
  呼ぶため、DSL の `title` を window のタイトルバーに反映する経路が ABI 上存在
  しない（[wasamo-runtime/src/abi.rs:1081-1083](../../wasamo-runtime/src/abi.rs#L1081)
  の `DEFAULT_WINDOW_TITLE` / `DEFAULT_WINDOW_WIDTH` / `DEFAULT_WINDOW_HEIGHT`）。

**結果として M2 で起きること:**

- M1 の counter examples では `wasamo_window_create("Counter", ...)` でタイトルが
  `"Counter"` だったが、DD-M2-P6-008 で `wasamo_load_ui` 経由に切り替わると
  default の `"Wasamo"` になる。
- `backdrop: mica` / `theme: system` も同様に未配線（M4 の Mica/Acrylic 導入で
  まとめて扱う想定）。
- これは A1/A2 acceptance（DSL drives / reactive propagation without host wiring）
  には抵触しない（acceptance 文面はタイトル文言を要求していない）。

**意図的な未実装である理由:**

- 配線するには ABI 拡張が必要（`wasamo_load_ui` への `WindowConfig` 引数追加か、
  新規 `wasamo_window_set_title` などの導入）。これは Phase 6 のスコープ
  「counter examples migration」を超え、abi_spec.md / wasamo.h / 新規 DD を
  伴うため、Phase 6 に含めない判断とした。
- 一方で wasamoc は正しく lowering しており、DSL surface としての記法は
  既に確定している。ABI 配線が追いついていないだけ。

**この議論を再訪する契機:**

- M3 の DSL spec drafting — Window 由来 prop の意味論を normative spec に
  書き起こす際に、配線も含めて整理する。
- M4 の Mica / Acrylic 導入（[ROADMAP.md](../../ROADMAP.md) の M4） — backdrop
  prop の実体実装に着手するタイミングで title も含めて ABI 設計を見直す。
- それより早く必要が生じた場合（counter 以外の demo で title が要件になる等）
  は、独立 DD を切って `wasamo_load_ui` の signature 拡張または sibling ABI
  追加を検討する。

---

### Q3. M3 target app からこぼれうる文法残余

**位置づけ:**

M3 target app pre-doc / phase pre-doc / DD で直接扱う文法論点は、そちらを本線とする。
本節は、M3 で採用しない、制限する、または M3 完了時に再整理すべき grammar 残余だけを
短く残す。

**候補残余:**

- **繰り返し生成:** M3 が `List` 吸収 / Repeater 型 / `for` 構文のどれかを採る場合、
  採らなかった形と rebuild / diff / invalidation の残余。
- **条件表示:** M3 が `visible` prop / conditional child / 回避のどれかを採る場合、
  採らなかった条件レンダリング方式と非表示 child semantics の残余。
- **template-local scope:** M3 が一段 item context に制限する場合、nested template scope、
  named context、List-owned selection model の残余。
- **TypedValue / evaluator 接続:** M3 proof が bool / comparison / item context を
  制限した場合の将来接続。

**この議論を再訪する契機:**

- M3 target app pre-doc / phase pre-doc が grammar 採用案を確定した時。
- M3 で制限・未採用にした形が M4+ で必要になった時。
- M3 完了時に、採用案と残余選択肢をこのノートから整理し直す時。

---

### Q4. Component extension model と DSL import surface

**背景（M3 target app framing 時点）:**

- M3 wireframe 検討では `layout primitive` という語を使っているが、これは
  Wasamo が M3 public draft で標準提供する built-in layout component を指す。
  Wasamo の component set が将来これらだけに閉じるという意味ではない。
- component extension model の本体は grammar だけでなく、component registry、
  measure / arrange protocol、native binding、package / import resolution を横断する。
  そのため詳細は `docs/notes/component-extension-model.md` に分離する。

**このノートでの扱い:**

- `dsl-grammar.md` では import / name resolution / reserved syntax に関係する
  ポインタだけを保持する。
- component extension model 自体は M3 acceptance scope に含めない。

**この議論を再訪する契機:**

- M3 ではなく、custom component / package / native binding を扱う milestone に入る時。
- M3 grammar が import 構文や component name resolution を予約する必要を持った時。

---

### Q5. Property RHS の式位置とテンプレート風参照構文

**背景（M3-Phase 1 / bool scalar 実装後）:**

- M3-Phase 1 では `Button.enabled: bool` が入り、次の 2 種類の RHS が
  `bool` として扱われる。
  - `enabled: ready` — `ready` は識別子。`state ready: bool = ...` に
    解決されると `BoolPropRead` に lower される。
  - `enabled: true` / `enabled: false` — `true` / `false` は識別子ではなく、
    lexer で予約された bool literal。`BoolLit` に lower される。
- `state false: bool = true` / `state true: bool = false` のような宣言は
  禁止される。これは実装先行の副作用ではなく、DD-M3-P1-002 の
  `true` / `false` keywords 採用と `docs/dsl_spec.md` §2.1 の予約語規則に
  よる明示的な設計判断。

**批判的検討:**

- `enabled: ${ready}` のようなテンプレート言語由来の参照構文は、
  Phase 1 では採らなかった。理由は、property binding の RHS はすでに
  `expr` 位置であり、識別子 `ready` をそのまま式として読めるため。
- `${...}` は HTML / text template / shell-like interpolation では自然だが、
  Wasamo DSL では文字列内 interpolation が既に `"\{root.count}"` で存在する。
  property RHS に `${ready}` を別途導入すると、
  「これは文字列 interpolation なのか」「式 interpolation なのか」
  「文字列化や coercion が起きるのか」という余計な区別を作る。
- M3-Phase 1 は `enabled: <bool-expr>` の `<bool-expr>` として
  bool literal と state identifier だけを認める狭い段階であり、
  将来の `!ready`, `root.ready`, comparison, logical operator は
  expression grammar の拡張として扱う方が一貫する。

**現時点の扱い:**

- Open question ではない。Phase 1 の `true` / `false` 予約と
  identifier resolution は DD-M3-P1-002 / DD-M3-P1-010 で閉じている。
- ただし、将来の expression grammar 拡張時に `${...}` 型のテンプレート風構文を
  再提案する場合は、文字列 interpolation との関係と coercion の有無を
  先に明文化すること。

**この議論を再訪する契機:**

- M3-Phase 6 conditional rendering で `if <expr>` / `enabled: !ready` /
  comparison / logical operator を導入する時。
- `root.ready` のような qualified state reference を property RHS に許すかを
  決める時。
- 文字列以外の属性値に template interpolation 風 syntax を導入したい強い
  外部要件が出た時。

---

### Q6. 条件レンダリング構文の思想

**背景:**

UI における「条件レンダリング」は、単に「表示する / 隠す」を切り替える
小機能ではない。UI DSL がどの程度まで画面構造を状態に従わせられるかを
決める、中核的な文法論点である。

大きく見ると、条件レンダリングには 3 つの思想がある。

1. **プロパティ制御型**

   UI ツリーそのものは先に作っておき、あとから `visible` や `enabled` の
   ようなプロパティを状態に応じて切り替える方式。

   実装しやすく、既存のプロパティバインディングにも乗せやすい。しかし、
   画面の構造と制御が分離しやすい。作者は「この状態ならこの部分が存在する」
   と宣言するのではなく、「この部品を表示して、あちらを隠す」という
   命令的な管理を抱えやすくなる。

2. **テンプレート + 独自構文・属性型**

   UI DSL の中に、条件付きで subtree を出すための専用構文や専用属性を
   持たせる方式。`if` 風の block、`when:` 風の属性、構造的 directive
   などがこの系統に入る。

   Wasamo は独立した `.ui` DSL を持つため、まずはこの方式が自然な中心になる。
   コンパイラとランタイムが条件 subtree の範囲、依存 state、生成・破棄
   される widget / effect の寿命を把握しやすく、public DSL spec としても
   説明しやすい。

   一方で、作者は Wasamo 独自の構文規則を覚える必要がある。そのため、この
   方式を採る場合でも、単発の特殊構文ではなく、将来の `else`、複数分岐、
   繰り返し生成と同じ文法ファミリーに育つ形が望ましい。

3. **言語構文型**

   UI レンダリングそのものがホスト言語や埋め込み DSL の中にあり、通常の
   `if`、`switch`、ループ、関数分割などをそのまま使って UI tree を生成する
   方式。

   自由度は最も高い。実用的な UI では単純な `if` だけでなく、複数分岐、
   繰り返し、局所スコープ、派生状態、item context などが自然に必要になる
   ため、この方向の表現力は強い。

   ただし Wasamo の中核仮説は、独立した `.ui` DSL と C ABI を持ち、複数
   言語から同じ UI 仕様を扱えることにある。ホスト言語の構文へ寄せすぎると、
   `.ui` の独立性や言語横断性が弱くなる。

**現時点の考え方:**

Wasamo の条件レンダリングは、まず **テンプレート + 独自構文・属性型** を
中心に考える。ただし、それは将来の **言語構文型に近い自由度** を諦める
という意味ではない。

少なくとも、次の方向性は守りたい。

- subtree の存在・非存在を、単なる表示プロパティではなく **構造的な
  present / absent** として扱う。
- 条件レンダリング、複数分岐、繰り返し生成を、ばらばらの特殊機能ではなく
  **構造的制御構文ファミリー** として考える。
- 将来 `else`、`switch` 相当、loop / iteration 相当へ広げるときに、初期の
  構文・IR・runtime が邪魔にならないようにする。
- 条件 subtree の中に binding / effect がある場合、その寿命を曖昧にしない。
  subtree が存在しないとき、内側の effect は存在するのか、止まるのか、
  破棄されるのか。再び存在するとき、再利用されるのか、再生成されるのか。
- 外部読者が DSL 仕様を読んだとき、条件レンダリングを単発の表示切替では
  なく、Wasamo DSL が tree shape を状態に従わせるための第一歩として理解
  できるようにする。

**ランタイム設計への含意:**

この方針は、条件レンダリング構文だけの問題ではない。将来、`.ui` DSL とは
別に、言語内 DSL から UI tree を生成する可能性を考えると、runtime は特定の
表面構文に依存しすぎない方がよい。

この観点では、Flutter の Widget / Element / RenderObject の分離は重要な
参照点になる。Widget に相当する軽量な宣言情報は、状態変化に応じて再生成
されてもよい。一方で、state、effect、layout 実体、focus、入力中の値などの
寿命は、runtime 側が identity に基づいて扱う必要がある。

Wasamo でも、条件 subtree の present / absent を構文上は簡潔に書けるように
しつつ、runtime 側では subtree identity、state scope、effect scope、
loop key、再利用 / 再生成の規則を後から仕様化できる余地を残したい。

つまり、v1 の表面構文がテンプレート + 独自構文・属性型であっても、runtime
内部では「軽量な宣言 tree」と「寿命を持つ実体 tree」を分けて考える。この
分離があると、将来 `if` / `switch` / loop をより言語構文型に近い形で扱う
場合にも、同じ runtime 上へ落とし込みやすい。

**この議論を再訪する契機:**

- 条件レンダリング構文を仕様化するとき。
- 繰り返し生成構文を仕様化するとき。
- `else` / `switch` / nested structural scope を導入したくなったとき。
- `.ui` DSL とは別に、Rust / Swift / Zig などの言語内 DSL を本格的に
  扱うとき。
