---
title: Top-layer overlays / popover — open questions
status: live
created: 2026-05-31
last-updated: 2026-05-31
related-specs:
  - docs/dsl_spec.md
related-roadmap:
  - process/_roadmap.md#m4-interaction-stack
related-adrs:
  - process/milestone-3/phase-6/decisions/preamble.md
related-notes:
  - docs/notes/dsl-grammar.md
  - docs/notes/m4-interaction-intake.md
---

# Top-layer overlays / popover — open questions

このノートは、Wasamo で **親コンテンツの layout / clip 境界から隔離された
overlay**、特に popover / tooltip / dropdown / menu / modal top layer を
いつ・どの surface として取り入れるかを後で判断するための live note。

M3-Phase 6 の ZStack は、gallery lightbox のための overlay primitive である。
ただしその意味は「通常の widget tree の中で children を重ねる」ことであり、
親の layout / clip / Visual hierarchy から独立した top-layer surface ではない。
root に置いた ZStack なら window 全体の lightbox overlay は表現できるが、
任意の親 subtree から境界を突き抜ける popover とは別問題である。

## Boundary

- この note は top-layer / portal / popover の **構造・配置・境界越え** を扱う。
- click-away close、keyboard close、focus trap などの具体的な M4 UX 要求は
  [`m4-interaction-intake.md`](./m4-interaction-intake.md) が受ける。
- `.ui` 文法入口、widget id / anchor 参照、structural rendering family との関係は
  [`dsl-grammar.md`](./dsl-grammar.md) Q1 / Q7 が受ける。
- Benchmark framework の一般比較は private survey または将来の
  `docs/notes/interaction-surface-survey.md` / `layout-surface-survey.md` に蒸留する。

## 現時点の位置づけ

- **v1 で絶対必要とはまだ決定しない。**
- ただし、v1 までに「必要か / いつ入れるか / 最小 surface は何か」を検討する
  open question として残す。
- M3-Phase 6 の ZStack / conditional rendering ADR は、この論点を閉じない。
  ZStack は root overlay の足場にはなるが、top-layer / portal / anchored overlay
  の設計判断を代替しない。

## なぜ ZStack だけでは足りないか

ZStack は通常 layout tree 内の container である。したがって:

- 親の layout slot に配置される。
- 親や自身の clip の影響を受ける。
- z-order は同じ parent 内の document order に閉じる。
- anchor となる widget の screen/window 座標を解決する surface を持たない。
- click-away close、focus capture、modal focus trap、keyboard dismissal などの
  input / focus policy を持たない。

このため、次のような UI は ZStack の単純な将来拡張ではなく、別の top-layer
概念を必要とする可能性が高い。

- Button から開く dropdown menu。
- thumbnail / icon / text selection に anchored する popover。
- layout subtree の clip を超えて表示される tooltip。
- window-level modal / sheet / command palette。

## 候補 surface

まだ採用しない。後続 ADR で比較する候補を残す。

### A. Root overlay convention

author が root 直下に `ZStack` / `if` を置き、window 全体の overlay を作る。

```wasamo
ZStack {
    MainContent { ... }

    if is_lightbox_open {
        ZStack { ... }
    }
}
```

最小で、M3-Phase 6 の設計と整合する。lightbox には十分。ただし任意の subtree
から親境界を突き抜ける popover ではない。

### B. `Portal` / `TopLayer`

通常 tree 上の宣言位置から、実体の描画先だけ window-level top layer に移す。

```wasamo
Portal {
    if is_menu_open {
        Menu { ... }
    }
}
```

親 clip から隔離しやすい一方、document order、lifecycle、focus、hit-testing、
accessibility の扱いを明文化する必要がある。

### C. Anchored `Popover`

anchor となる widget / rect に結びついた overlay surface を持つ。

```wasamo
Button {
    id: menu_button
    clicked => { root.is_menu_open = true; }
}

Popover {
    anchor: menu_button
    open: is_menu_open
    VStack { ... }
}
```

author ergonomics は良いが、widget identity / anchor resolution / coordinate
conversion / placement fallback / collision avoidance が必要になる。`id:` surface
を要求するなら `dsl-grammar.md` Q1 と結びつく。

### D. Host-managed overlay API

`.ui` は open state と content だけ宣言し、placement / top-layer への移送は host
または runtime service に寄せる。

ABI / host responsibility の境界が大きくなるため、M4+ の input / focus / multi-window
設計と一緒に見る必要がある。

## 主な未解決事項

- **必要時期:** v1 必須にするか、M4 input/focus 後に送るか、M3-Phase 8 の
  public draft で reserved/non-normative として触れるだけにするか。
- **最小 surface:** root overlay convention で v1 を満たすか、`Portal` /
  `TopLayer` を先に入れるか、anchored `Popover` まで入れるか。
- **anchor identity:** anchor widget をどう参照するか。`id:` / `@name` /
  state-driven anchor model など。これは `dsl-grammar.md` Q1 の widget id 論点と
  関係する。
- **layout boundary:** overlay はどの coordinate space で measure / arrange されるか。
  親 layout に desired size を返すのか、top-layer 側で独立に配置するのか。
- **clip policy:** 親 clip / scroll viewport / ZStack clip を越えることをどこまで
  許すか。
- **z-order policy:** top-layer 内の ordering、modal vs non-modal、複数 popover の
  stacking rule。
- **input / focus:** click-away close、Escape close、focus capture、modal focus trap、
  pointer capture。M4 input/focus と強く結合する。
- **accessibility:** top-layer に移された content の semantic order / focus order /
  modality をどう扱うか。M4 AccessKit / UIA と結合する。
- **lifecycle:** conditional rendering の absent=fresh-on-return semantics と合わせるか、
  popover retention / keyed identity を許すか。
- **ABI / host boundary:** window-level top layer を runtime 内部だけで扱うか、
  host-facing API を作るか。

## 再訪する契機

- M3-Phase 8 で full gallery E2E / public draft をまとめるとき、v1 gallery に
  root lightbox 以上の overlay が必要か判断する。
- M4 input / focus model の pre-doc で、click-away close、focus trap、keyboard
  dismissal を扱うとき（具体ケースは
  [`m4-interaction-intake.md`](./m4-interaction-intake.md)）。
- Widget id / anchor 参照 surface を開くとき（`dsl-grammar.md` Q1）。
- Window-level property / host-wiring / multi-window 設計を開くとき。
- 実アプリ要件として dropdown / menu / tooltip / popover が v1 必須に近づいたとき。

## M3-Phase 6 への読み替え

M3-Phase 6 の ADR を Accept しても、この open question は閉じない。

- Phase 6 は root lightbox overlay を ZStack + `if` で証明する。
- Phase 6 は親境界を突き抜ける popover / portal を約束しない。
- ただし Phase 6 の member-level structural IR と conditional subtree lifecycle は、
  将来 `Portal` / `Popover` の content を `if` / `for` と組み合わせる土台になる。
