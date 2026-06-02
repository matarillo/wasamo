---
title: M4 interaction intake — lightbox and event/focus cases
status: live
created: 2026-05-31
last-updated: 2026-05-31
related-roadmap:
  - process/_roadmap.md#m4-interaction-stack
related-notes:
  - docs/notes/top-layer-overlays.md
  - docs/notes/dsl-grammar.md
---

# M4 interaction intake — lightbox and event/focus cases

このノートは、M4 Interaction stack の spec / framing で忘れずに扱うべき
input・hit-testing・focus・modal interaction の具体ケースを、M3 の作業中に
先に拾っておくための intake note。

`process/_roadmap.md` の M4 は "keyboard, mouse, touch; focus model and event
routing" を acceptance として持つ。ただしその粒度では、M3 gallery / lightbox
から出てきた具体的な UX 要求が埋もれやすい。M4 pre-doc / framing は、本ノートを
入力として読み、各項目を **M4 scope / M4 explicit defer / M5+ / v1 required** の
どれかに分類すること。

Benchmark framework の一般比較はここに入れない。一般比較は private survey
（例: `private/ui-layoutsystem.md`, `private/ui-inputsystem.md`）または将来の
`docs/notes/interaction-surface-survey.md` に蒸留し、この note は M3 gallery /
lightbox 由来の M4 要求に絞る。

## 背景

M3-Phase 6 の lightbox proof は、`Button.clicked` handler で
`is_lightbox_open` を切り替える最小経路を使う。これは conditional rendering と
ZStack の proof としては十分だが、ユーザーが自然に期待する lightbox UX の全体を
閉じるものではない。

M3 では次を意図的に扱わない:

- thumbnail そのものをクリックして lightbox を開く。
- scrim click / click-away で lightbox を閉じる。
- `Box` や画像など Button 以外の任意 visual element に hit handler を付ける。
- focus trap / modal input / Escape close / keyboard navigation。

これらは M4 interaction stack の自然な候補であり、M4 framing で再評価する。

## M4 framing で扱うべき具体ケース

### 1. Thumbnail click opens lightbox

**User story:** gallery thumbnail をクリックすると lightbox が開く。

**設計論点:**

- Clickable surface を Button 以外へ広げるか。
- `Box` / future `Image` / arbitrary widget に `clicked` や `on pointer_down`
  を許すか。
- Button semantics と generic hit handler semantics を分けるか。
- Accessibility role はどう付くか。見た目が thumbnail でも、操作上は button
  なのか、image + action なのか。

**M4 framing question:** v1 gallery で thumbnail click を必須にするか。必須なら、
generic hit handler まで開くのか、thumbnail を Button-like surface で包む最小案に
するのか。

### 2. Scrim click / click-away closes overlay

**User story:** lightbox の外側、または scrim をクリックすると閉じる。

**設計論点:**

- Scrim はただの `Box` なのか、modal overlay の dismissal region なのか。
- event bubbling / capture / stop-propagation が必要か。
- overlay 内部をクリックしたときに scrim の handler へ伝播しない規則をどうするか。
- top-layer / portal を採る場合、click-away 判定は通常 tree ではなく top-layer
  service の責務になりうる。

**M4 framing question:** click-away close を M4 input/focus の必須ケースに含めるか。

### 3. Hit handlers on non-Button widgets

**User story:** `Box`、Image placeholder、将来の `Image`、任意 container に
pointer/click handler を付けたい。

**候補 surface:**

```wasamo
Box {
    clicked => { root.is_lightbox_open = true; }
}
```

または:

```wasamo
Box {
    pointer_down => { ... }
}
```

**設計論点:**

- `clicked` を Button 固有 signal から generic widget signal へ広げるか。
- pointer events を raw event surface として出すか、それとも high-level
  `clicked` / `pressed` / `hovered` に留めるか。
- hit-test eligibility をどう決めるか。visual を持つ widget はすべて hit-testable
  なのか、明示属性が必要なのか。
- disabled / selected / hover / pressed state とどう関係するか。

**M4 framing question:** generic event handler surface を M4 で出すか、v1 は
specific widgets の signals に限定するか。

### 4. Modal focus trap

**User story:** lightbox / modal が開いている間、Tab focus は overlay 内に閉じる。
背景側の操作はできない。

**設計論点:**

- Focus scope / focus root / modal scope の surface。
- Background subtree を inert にするか。
- Accessibility tree で background を hidden/inert 扱いにするか。
- Conditional subtree が absent になったとき、focus をどこへ戻すか。
- Top-layer overlay と通常 root ZStack overlay で同じ focus policy を使うか。

**M4 framing question:** focus trap を M4 acceptance の中核ケースにするか。

### 5. Keyboard close and keyboard navigation

**User story:** Escape で lightbox を閉じる。ArrowLeft / ArrowRight で前後移動する。
Tab / Shift+Tab で overlay 内を移動する。

**設計論点:**

- Key events は focused widget に配送するか、window / focus scope に配送するか。
- Declarative shortcut surface を作るか。
- Text input / IME と key shortcut の優先順位。
- Modal scope が Escape を consume する規則。

**M4 framing question:** Escape close を M4 の最小 keyboard proof に含めるか。
Arrow navigation は v1 gallery 必須か、M5+ polish か。

### 6. Event routing and propagation model

上記すべての土台として、M4 は event routing を決める必要がある。

**未決事項:**

- capture / target / bubble の三相を持つか。
- それとも Wasamo v1 は target-only + explicit high-level signals で始めるか。
- handler が state mutation したとき、既存の synchronous drain contract とどう結合するか。
- removed subtree へ event が配送されないことをどう保証するか。
- pointer capture / hover enter-leave / pressed state をどう扱うか。

## top-layer overlay との関係

[`top-layer-overlays.md`](./top-layer-overlays.md) は、親 layout / clip 境界を
越える overlay の open question を扱う。このノートは、そのうち **input / focus
側の具体ケース** を M4 intake として展開する。

両者の関係:

- root ZStack lightbox でも、thumbnail click / scrim click / Escape close /
  focus trap は必要になりうる。
- `Portal` / `Popover` を採るなら、click-away、focus trap、anchor hit-testing は
  さらに強く結合する。
- M4 framing は top-layer overlay を採用するかどうかとは独立に、generic
  hit-testing / event routing / focus scope の最小方針を決める必要がある。

## M4 pre-doc / framing への要求

M4 を開くとき、pre-doc / framing は少なくとも次を行う:

1. 本ノートの具体ケース 1〜6 を読み、各ケースを `M4 scope` / `M4 explicit defer`
   / `M5+` / `v1 required` のいずれかに分類する。
2. `process/_roadmap.md` の M4 acceptance "keyboard, mouse, touch; focus model and
   event routing" に対し、どの concrete proof で discharge するかを決める。
3. gallery / lightbox を M4 proof に使う場合、M3 の root ZStack lightbox を拡張する
   のか、別の minimal interaction fixture を作るのかを決める。
4. `Button.clicked` 既存 signal と generic hit handler の関係を明文化する。
5. focus trap / modal / accessibility を同時に扱うか、M4 内で phase 分割するかを
   決める。
6. top-layer / portal / popover を M4 で開くか、M4 では root overlay + focus/input
   までに留めるかを明示する。

## M3 側への読み替え

M3 はこのノートの項目を実装しない。

- M3-Phase 6 は Button click handler による open/close を proof とする。
- M3-Phase 6 は arbitrary hit handler、click-away close、focus trap、keyboard close
  を約束しない。
- M3-Phase 8 の full gallery E2E で、これらが v1 必須に近づいた場合は、本ノートを
  参照して M4/M5 への明示的な carry-forward または scope revision を判断する。
