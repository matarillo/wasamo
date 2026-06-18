---
title: DSL 文法 — 検討メモと未解決事項
status: live
created: 2026-05-07
last-updated: 2026-06-10
related-adrs:
  - process/milestone-2/phase-2/decisions/preamble.md
  - process/m2-phase-6-ir-loader.md
  - process/milestone-3/phase-1/decisions/preamble.md
related-specs:
  - docs/dsl_spec.md
related-notes:
  - docs/notes/top-layer-overlays.md
---

# DSL 文法 — 検討メモと未解決事項

このノートは Wasamo DSL の文法に関するオーナーの検討記録の live note。
確定済みの normative 仕様は `docs/dsl_spec.md` を参照。本ノートは
**未解決の文法上の論点** と、**将来 ADR に昇格しうる検討メモ** を残す。

特に M3 以降では、target app pre-doc / phase pre-doc / DD で直接扱う論点は
そちらを本線とする。本ノートは、M3 の採用案からこぼれた選択肢、M3 では
解かないことにした残余、または M4+ で再整理すべき open question の
受け皿として使う。M3-Phase 6 ADR で確定した内容は、ここでは再決定せず、
「吸収済み」と「残った再訪点」を分けて記録する。

---

## Open Questions

### Q1. Widget 識別子の IR 表現

**現状（M3-Phase 6 ADR 時点）:**

- DSL の `widget_decl ::= IDENT "{" member* "}"` の `IDENT` は widget の
  **type** 名（`VStack`, `Button` など）であり、widget instance を一意に
  指す **id** を付ける構文は存在しない。
- バインディング/ハンドラから参照される `root.count` の `root` は
  component の base（暗黙の self）を指し、`count` は `state` 宣言名。
  すなわち現行 IR で名前解決の対象になりうるのは **`state` 宣言のみ**。
- M3-Phase 5 Grid は `Cell` placement で足り、widget id は導入しなかった。
  M3-Phase 6 conditional rendering も `declared_member_index` と
  `ControlFlowNode` で slot を管理し、widget id は導入しない。

**論点:**

- DD-M2-P6-009（IR loader の防御的検証）のタスク文には
  「every binding/handler name resolves to a declared `state` or widget」
  と書かれており、forward-looking に「widget 名解決」の余地を含めていた。
  しかし現 IR には widget 識別子が無いため、M2 時点の reference resolution
  検証は **state 名のみ** を対象とする。
- 将来、widget instance を式から参照する必要が生じた場合（候補シナリオは
  下記）、IR / DSL の両側に widget id を持ち込む文法拡張が必要になる。

**候補シナリオ:**

- Phase 7 の `for` / iteration で per-item context、item identity、`key:`
  を導入する場合。これは「任意 widget id」というより、generated subtree の
  identity / item scope の問題として開く可能性が高い。
- 兄弟 widget の状態を直接参照したいケース（例：チェックボックスの状態に
  応じて別 widget の `enabled` をバインド）。M2 では state を経由するため
  不要だが、ボイラープレートが過剰になれば再考対象。
- top-layer / popover / anchor positioning で、anchor となる widget を参照したい
  ケース。詳細は [`top-layer-overlays.md`](./top-layer-overlays.md) を SSOT とする。

**未決:**

- そもそも widget id を許す方針を取るか、それとも全参照を `state` 経由に
  揃える方針を貫くか。後者は Elm/SwiftUI 的に「UI = state の純関数」を
  強制することになり、設計上の魅力はある。
- Id を導入する場合の構文（`node Button#submit { ... }` か、
  `id: "submit"` プロパティか、`@submit` か等）。
- 名前解決スコープ（component-local フラットか、ネスト可能か）。

**この議論を再訪する契機:**

Phase 7 iteration の pre-doc / DD で `item` / `index` / `key:` / retained
identity を扱う時、top-layer anchor 参照を開く時、または「state 経由のみ」
では表現力が足りない concrete app case が出た時。

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

**M3-Phase 6 ADR 後の整理（2026-06-01）:**

- conditional rendering は widget id ではなく member-level structural IR で扱う
  ことになった。Q1 は任意 widget id の導入問題としては未解決のままだが、
  Phase 6 の `if` には不要。
- 次の本命は Phase 7 iteration の item identity / key surface。ここで
  widget id と item key を混同しないこと。

**M3-Phase 7 owner 回答後（2026-06-10）:**

- iteration の per-item context として `item` / `index` を、**「全参照を `state`
  経由に揃える」主義への初の明文例外** ＝ **loop-local read-only binding** として
  認める（owner 回答、[owner-intent-answers §2 Q5a](../../process/milestone-3/phase-7/requirements/owner-intent-answers.md);
  思想は本ノート Q8）。例外は **式（binding）位置に限る**。
- **handler 位置から `item` を読めるか**（select-this-item 等）は **未決のまま
  残す** — 別 admission 判断として後続の設計文書で裁く（責務先は本ノートに
  書かない。現在の割当は phase-7 framing を参照）。
- これは「任意 widget id」を開くものではない。widget id ≠ item key の規律は不変。

---

### Q2. Window 由来の component-level prop の runtime 配線

**現状（M3-Phase 6 ADR 時点）:**

- DSL では `component Counter inherits Window { title: "Counter"; backdrop: mica;
  theme: system; ... }` のように Window 由来 prop を component 直下に書ける。
- wasamoc の lowering（[wasamoc/src/lower.rs](../../wasamoc/src/lower.rs) の
  `lower()`）は component-level の `PropertyBind` を root widget node の props に
  splice し、IR 上は root の `prop title = "Counter"` 等として正しく出力される。
- M3-Phase 6 DD-M3-P6-006 は、static `title:` だけを R1 として回収する方針を
  採った。`wasamo_load_ui` の ABI signature は変えず、root props から
  `title` literal を読み、内部の `window::create(title, width, height)` に渡す。
- `backdrop: mica` / `theme: system` / dynamic title / initial window size は、
  Phase 6 では引き続き未配線または deferred。

**Phase 6 で閉じること / 閉じないこと:**

- **閉じる:** `title: "Gallery"` のような static string literal は native window
  title bar に反映する。absent / empty title は default fallback、non-string IR
  title は malformed として reject する方針。
- **閉じない:** `title: some_string_state` のような dynamic title は defer。
  window は `WidgetNode` ではないため、`BindingTarget::WindowTitle` などの
  window-property binding seam と host effector が必要になる。
- **閉じない:** `backdrop` / `theme` は M4 の Mica / Acrylic / theme wiring で
  title と同じ Window-prop family として再評価する。

**initial window size についての補足（M3-Phase 6 検討）:**

- `wasamo_load_ui` は `DEFAULT_WINDOW_WIDTH` / `DEFAULT_WINDOW_HEIGHT`
  で native window を作るため、`.ui` から initial window size を指定する
  経路も現状はない。
- 一見すると static `title:` と同じ creation-time Window prop として
  `wasamo_load_ui` 内で読めそうだが、性質は少し違う。window size は
  root layout viewport の初期条件であり、client size vs outer window size、
  DPI、resize policy、将来の widget-level `width:` / `height:` size
  constraint surface と混同しやすい。
- そのため、M3-Phase 6 の R1 では **static title のみ**を回収し、
  initial size は Window-prop / WindowConfig 設計の候補として後続へ送るのが
  よい。もし先に surface を切るなら、一般 widget の `width:` / `height:` と
  区別できる名前（例: `viewport-width:` / `viewport-height:`、または
  `window-width:` / `window-height:`）を検討する。

**意図的に Phase 6 へ入れない理由:**

- static title は ABI 変更なしで閉じられるが、dynamic title / backdrop /
  theme / size は Window-prop binding、WindowConfig、host effector、DPI、
  client-size semantics と結びつく。これらを R1 に混ぜると Phase 6 の
  ZStack / conditional rendering の主戦場がぼやける。
- 一方で wasamoc は component-level props を root IR props に splice しており、
  surface と IR carrier は既に存在する。後続はその carrier を window-property
  seam としてどう解釈するかを決める段階。

**この議論を再訪する契機:**

- M4 の Mica / Acrylic 導入（[process/_roadmap.md](../../process/_roadmap.md) の M4） — backdrop /
  theme の実体実装に着手するタイミングで、static/dynamic title も含めた
  Window-prop binding / host effector / WindowConfig の形を見直す。
- dynamic title が v1 必須または gallery proof に必要になった時 —
  `BindingTarget::WindowTitle` と `wasamo_window_set_title` 相当の effector を
  独立 DD として切る。
- initial window size を `.ui` で指定したくなった場合 — title と同じ
  creation-time prop として扱うか、WindowConfig / viewport-size surface として
  backdrop/theme とまとめるかを別 DD で決める。
- multi-window / scene model を開く時 — component-level Window props が
  component root に属するのか、Window entity / Scene entity に属するのかを
  再整理する。

---

### Q3. M3 target app からこぼれうる文法残余

**位置づけ（M3-Phase 6 ADR 後）:**

M3 target app pre-doc / phase pre-doc / DD で直接扱う文法論点は、そちらを本線とする。
本節は、M3-Phase 6 までに採用しない、制限する、または M4+ / Phase 7 へ送った
grammar 残余だけを短く残す。

**候補残余:**

- **繰り返し生成:** Phase 6 は `if` のみ。Phase 7 の `for` で、item context、
  generated subtree identity、range insert/remove、keyed retention を決める。
- **条件表示の不採用形:** Phase 6 は `visible:` / `when:` ではなく structural
  `if` block を採った。`visible` 的な property toggling を将来入れる場合も、
  structural `if` とは別 semantics として扱う。
- **条件 body の制限:** Phase 6 の `if` body は single widget child。multi-widget
  branch、bare nested control flow、`else` / `switch` は structural family extension
  として後続で扱う。
- **template-local scope:** Phase 7 `for` で `item`, `index`, named context,
  nested template scope を決める。
- **TypedValue / evaluator 接続:** Phase 6 conditional は narrow bool-expr のまま。
  `!`, `&&`, comparison, qualified reads は uniform expression grammar extension として
  後続で扱う。

**この議論を再訪する契機:**

- Phase 7 iteration pre-doc / DD を開く時。
- `else` / `switch` / bare nested structural scope を導入したくなった時。
- expression grammar extension を開き、`if !ready` と `enabled: !ready` を同じ
  grammar で扱う時。
- M3 完了時に、この残余リストを Phase 7 / M4+ の note へ分配する時。

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
- ZStack / Grid / ScrollView など built-in primitive と user-defined component の
  境界を public DSL spec で説明する必要が出た時。

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

- Phase 6 では operators を導入しない方針が採られたため、次の再訪点は
  post-Phase-6 の uniform expression grammar extension。`if !ready` と
  `enabled: !ready` を同じタイミングで扱う時。
- `root.ready` のような qualified state reference を property RHS に許すかを
  決める時。
- 文字列以外の属性値に template interpolation 風 syntax を導入したい強い
  外部要件が出た時。

---

### Q6. 条件レンダリング構文の思想

**状態（M3-Phase 6 ADR 後）:**

この節の思想は M3-Phase 6 ADR に吸収された。Phase 6 は
`if <bool-expr> { <single-widget-child> }` を dedicated structural syntax として
採り、`visible:` / `when:` 的な property gating ではなく、subtree の
present / absent を runtime tree の insert/remove として扱う方針になった。

したがって以下は、Phase 6 の決定を覆す open question ではなく、将来
`else` / `switch` / `for` / keyed identity / host-language DSL を開くときの
設計背景として残す。

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
中心にする方針で ADR 化された。ただし、それは将来の
**言語構文型に近い自由度** を諦めるという意味ではない。

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

- 繰り返し生成構文を仕様化するとき。
- `else` / `switch` / nested structural scope を導入したくなったとき。
- `key:` / retained identity / Element-level reconciler を導入したくなったとき。
- `.ui` DSL とは別に、Rust / Swift / Zig などの言語内 DSL を本格的に
  扱うとき。

---

### Q7. Top-layer overlay / popover surface

**位置づけ:**

M3-Phase 6 の ZStack は lightbox overlay のための layout primitive だが、
親コンテンツの layout / clip 境界から隔離された popover / tooltip / dropdown /
menu / modal top layer までは閉じない。これは文法だけでなく、widget identity、
anchor resolution、coordinate conversion、clip、z-order、input / focus、
accessibility、ABI / host boundary を横断する論点である。

**このノートでの扱い:**

`dsl-grammar.md` では、`.ui` surface と widget id / structural rendering に
関係する入口だけを保持する。詳細な open question は
[`top-layer-overlays.md`](./top-layer-overlays.md) を SSOT とする。

**この議論を再訪する契機:**

- M3-Phase 8 の full gallery E2E / public draft で、v1 に root lightbox 以上の
  overlay が必要か判断するとき。
- M4 input / focus model で click-away close、focus trap、keyboard dismissal を
  仕様化するとき。
- Widget id / anchor 参照 surface（Q1）を開くとき。
- Window-level property / host-wiring / multi-window 設計（Q2）を開くとき。
- dropdown / menu / tooltip / popover が v1 必須に近づいたとき。

---

### Q8. イテレーション grammar の思想（初回surface）

**状態（M3-Phase 7 owner 回答後 2026-06-10）:**

Phase 7 の iteration（`for`）の初回surface の思想を owner 回答
（[owner-intent-answers](../../process/milestone-3/phase-7/requirements/owner-intent-answers.md)）
から蒸留する。Q6 の構造的制御構文ファミリー（`if` → `else` / `switch` / `for`）の
loop メンバに対応する。**スケジュール（責務先の M4 / M5 / ADR 割当）はここに
書かない** — 正本は phase-7 framing に置き、本ノートは思想と条件ベースの再訪契機
のみ保持する（計画は仮説であり、思想 note に埋めると計画改訂のたびに腐る。現在の
割当は phase-7 framing を参照）。

**初回surface の中核思想:**

- **collection binding が cardinality を駆動する。** iteration は静的展開では
  なく、実行時可変の collection binding が、生成される widget subtree の**個数**を
  駆動することを示す（凍結 acceptance「collection binding drives widget-tree
  generation」と無改訂で整合）。`if` が subtree の present/absent を駆動したのに
  対し、`for` は 0..N の cardinality を駆動する。
- **un-keyed base が baseline。** 初回surface は identity を保持しない
  fresh/positional base（collection 変化で rebuild）。これは Q6 の「軽量な
  declared tree と寿命を持つ entity tree の分離」の un-keyed 形を collection へ
  一般化したもの。
- **keyed identity / retained state は declared-tree anchor 上の opt-in。**
  declared tree（`for` メンバ + item template）が安定 anchor なので、将来の
  keyed retention は IR 変更なしの opt-in として後付けでき、baseline の
  destroy/rebuild を黙って変えない。
- **reorder は別問題（ordering contract）。** data-driven reorder は ordering
  contract + keyed diff を要し、cardinality 駆動の証明とは別の thesis。初回surface
  は append/truncate-only。
- **要素型は `TypedValue` と一体で育てる。** `item` は初回surface では scalar。
  複合 field（`item.filename` 等）は要素型を複合にし `TypedValue` 圧力を生むため、
  型システムの thesis と一緒に開く。
- **loop-local `item` / `index` は state 経由主義への初の明文例外**（Q1 追記参照）。
  例外は式（binding）位置に限り、handler 位置の可読性は未決。

**この議論を再訪する契機（条件ベース。スケジュールは phase-7 framing 参照）:**

- repeated subtree 内に focus / input / selected / user-editable state が入る、
  または reorder を許す → keyed identity / retained state を開く。
- sort / filter / drag reorder / keyed diff を要する UI → data-driven reorder。
- `item.filename` / caption fields / record-like state など scalar で足りない
  concrete case → structured item fields（`TypedValue` と一体）。
- select-this-item / delete-this-item 等の per-item interaction → handler 位置
  からの `item` 参照 / per-item handler admission。
- nested `for` / `else` / `switch` / template-local named scope の必要 → nested
  template scope（nested structural control flow と同じ波で開く）。
- N item 生成が `MUTATION_CAP` に触れる、または visible list が数十〜数百 item を
  acceptance として要求 → gallery-scale / cap / fan-out。
