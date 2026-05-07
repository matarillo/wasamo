---
title: DSL 文法 — 検討メモと未解決事項
status: live
created: 2026-05-07
last-updated: 2026-05-07
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
