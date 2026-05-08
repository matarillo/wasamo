---
title: DSL 文法 — 検討メモと未解決事項
status: live
created: 2026-05-07
last-updated: 2026-05-08
related-adrs:
  - docs/decisions/m2-phase-2-wasamoc-output-format.md
  - docs/decisions/m2-phase-6-ir-loader.md
related-specs:
  - docs/dsl_spec.md
---

# DSL 文法 — 検討メモと未解決事項

このノートは Wasamo DSL の文法に関するオーナーの検討記録の live note。
確定済みの normative 仕様は `docs/dsl_spec.md` を参照。本ノートは
**未解決の文法上の論点** と、**将来 ADR に昇格しうる検討メモ** を残す。

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
