---
title: M4 interaction intake — lightbox and event/focus cases
status: live
created: 2026-05-31
last-updated: 2026-07-28
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
入力として読み、各項目を **M4 scope / M4 explicit defer / M5+ / 1.0 required** の
どれかに分類すること（分類枠の 4 つ目は当初 "v1 required" と綴っていたが、
1.0 = M6 というロードマップに合わせて 1.0 required と読む）。

**設計はケース 1〜6 とも M4-Phase 2 の ADR で確定した**（2026-08-05。
[M4-Phase 2 decisions](../../process/milestone-4/phase-2/decisions/preamble.md)）。
下の分類表は M4 範囲の判定として有効なまま、各ケースの設計論点は当該
ADR が持つ — 要求 4（`Button.clicked` と汎用ハンドラの関係）と ケース 6
（配送モデル）の 2 件は、本ノートが「該当フェーズの ADR へ」と送った先に
着地した。本ノートは実装フェーズの入力として `live` を維持する。

**分類は完了した**（2026-07-28）。結果は
[§分類の結果](#分類の結果2026-07-28-確定)、および本ノートに課された 6 つの
要求の消化先は [§M4 pre-doc / framing への要求](#m4-pre-doc--framing-への要求)
の消化表を参照。本ノートは `status: live` のまま各フェーズ ADR の入力として
残る（ケース本文の設計論点が ADR の材料であり、分類だけでは閉じないため）。

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

**分類（2026-07-28）: M4 scope。** generic hit handler まで開く（ケース 3 の
帰結）。thumbnail を Button で包む最小案は採らない。→ [§分類の結果](#分類の結果2026-07-28-確定)

### 2. Scrim click / click-away closes overlay

**User story:** lightbox の外側、または scrim をクリックすると閉じる。

**設計論点:**

- Scrim はただの `Box` なのか、modal overlay の dismissal region なのか。
- event bubbling / capture / stop-propagation が必要か。
- overlay 内部をクリックしたときに scrim の handler へ伝播しない規則をどうするか。
- top-layer / portal を採る場合、click-away 判定は通常 tree ではなく top-layer
  service の責務になりうる。

**M4 framing question:** click-away close を M4 input/focus の必須ケースに含めるか。

**分類（2026-07-28）: M4 scope。** 含める。ただし証明の場は top-layer に載る
overlay（Quick Capture Inbox の項目メニュー）であり、gallery の scrim は
「兄弟への click を遮る」遮蔽規則の側で扱う。→ [§分類の結果](#分類の結果2026-07-28-確定)

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

**分類（2026-07-28）: M4 scope。** generic な click handler を出す（機能リスト
P1 行「汎用クリック処理」）。高水準 signal に留めるか raw pointer event まで
出すか、hit-test eligibility の決め方は event routing phase の ADR。
→ [§分類の結果](#分類の結果2026-07-28-確定)

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

**分類（2026-07-28）: M4 scope（中核ケース）。** focus trap は「層に載ったものが
閉じ込める」という層の性質ではなく、任意の部分木に付く**構造非依存の modal な
focus scope** として設計し、ZStack の枝と top-layer の中身の 2 構造で実証する。
→ [§分類の結果](#分類の結果2026-07-28-確定)

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

**分類（2026-07-28）: M4 scope。** Escape close は含める。Arrow navigation は
gallery lightbox の前後移動として M4 で取る（polish 送りにしない）。overlay 内の
Tab 移動はケース 4 と一体。declarative shortcut surface を一般機構として開くかは
phase ADR の裁量で、M4 の acceptance は上記 3 操作で満たす。
→ [§分類の結果](#分類の結果2026-07-28-確定)

### 6. Event routing and propagation model

上記すべての土台として、M4 は event routing を決める必要がある。

**未決事項:**

- capture / target / bubble の三相を持つか。
- それとも Wasamo v1 は target-only + explicit high-level signals で始めるか。
- handler が state mutation したとき、既存の synchronous drain contract とどう結合するか。
- removed subtree へ event が配送されないことをどう保証するか。
- pointer capture / hover enter-leave / pressed state をどう扱うか。

**分類（2026-07-28）: M4 scope（app 非依存の土台）。** routing model そのものの
選択（三相か target-only + 高水準 signal か）は M4 最初期 phase の ADR が決める。
上記の未決事項はその ADR の論点表になる。
→ [§分類の結果](#分類の結果2026-07-28-確定)

## 分類の結果（2026-07-28 確定）

[M4 start framing](../../process/milestone-4/requirements/framing.md) と
[M4 target app spec](../../process/milestone-4/requirements/spec.md) の確定に
より、ケース 1〜6 は**全件 M4 scope**。分類枠のうち `M4 explicit defer` /
`M5+` / `1.0 required` に落ちたのはケース本体ではなく、下表の「M4 で開かない
部分」欄の粒度である。

| ケース | 分類 | M4 での消化先（どの app のどこが証明するか） | M4 で開かない部分 |
|---|---|---|---|
| 1. thumbnail click で lightbox を開く | M4 scope | Photo Gallery のサムネイル click → lightbox。ケース 3 の generic hit handler の最初の具体ケースで、per-item handler（handler 位置での item / index 参照）を伴う | — |
| 2. scrim click / click-away で閉じる | M4 scope | Quick Capture Inbox の項目行「…」menu（top-layer に載る overlay の外側 click）。gallery の scrim は ZStack 兄弟間の遮蔽規則として同じ ADR で扱う | 「lightbox の scrim を click したら閉じるか」は app 仕様の裁量（spec には持たせていない） |
| 3. Button 以外への hit handler | M4 scope | 機能リスト P1 行「汎用クリック処理」。Photo Gallery のサムネイルと Quick Capture Inbox の項目行 | raw pointer event surface（`pointer_down` 等）を出すかは phase ADR。出さない判断も可 |
| 4. modal focus trap | M4 scope（中核） | **2 構造で実証**: Photo Gallery の lightbox（root ZStack のまま）と Quick Capture Inbox の rename dialog（top-layer）。背景を読み上げから隠す規則も層ではなく focus scope に付ける | — |
| 5. keyboard close / navigation | M4 scope | Escape = overlay の規則一式、← → = gallery lightbox の前後移動、Tab = ケース 4 と一体。text input / IME と shortcut の優先順位は Quick Capture Inbox が具体ケースを供給する | declarative shortcut surface の一般機構化（phase ADR 裁量、M4 AC ではない） |
| 6. event routing model | M4 scope（app 非依存） | M4 最初期 phase の ADR。`Button.clicked` と generic hit handler の関係、hit-test eligibility を必須論点として含む | — |

分類枠の他の 3 つに落ちた隣接項目（本ノートのケース本体ではないが、読者が
同じ設計空間で探すもの）:

- **anchored popover（widget への宣言的な吸着・座標変換・配置規則）** —
  `M4 explicit defer`。M4 の top-layer は「構造 + 閉じ方 / focus の規則一式」
  までとし、吸着は含めない。[candidate pool](../../process/candidate-pool.md)
  に記録済みで、正規の買い手は M5 の公式 widget set（Menu / ComboBox）。
- **公式 widget としての Menu / Dialog** — `M5+`。M4 の menu / dialog は既存
  widget（Box / Text / Button / 一行入力欄）の作者合成であり、widget 製品面の
  完成を意味しない。
- **widget の名前参照（[dsl-grammar](./dsl-grammar.md) Q1）** — `M4 explicit
  defer`。anchored popover を M4 に含めない帰結として、M4 では開かない。

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
   / `M5+` / `1.0 required` のいずれかに分類する。
2. `process/_roadmap.md` の M4 acceptance "keyboard, mouse, touch; focus model and
   event routing" に対し、どの concrete proof で discharge するかを決める。
3. gallery / lightbox を M4 proof に使う場合、M3 の root ZStack lightbox を拡張する
   のか、別の minimal interaction fixture を作るのかを決める。
4. `Button.clicked` 既存 signal と generic hit handler の関係を明文化する。
5. focus trap / modal / accessibility を同時に扱うか、M4 内で phase 分割するかを
   決める。
6. top-layer / portal / popover を M4 で開くか、M4 では root overlay + focus/input
   までに留めるかを明示する。

### 消化の記録（2026-07-28）

6 件とも消化済み。着地先は次のとおり（`framing` =
[M4 start framing](../../process/milestone-4/requirements/framing.md)、
`spec` = [M4 target app spec](../../process/milestone-4/requirements/spec.md)）:

| 要求 | 消化先 |
|---|---|
| 1. ケース 1〜6 の分類 | 本ノート [§分類の結果](#分類の結果2026-07-28-確定) |
| 2. acceptance をどの concrete proof で discharge するか | spec §2 本の役割分担（機能ごとの担当 app と証明の形）+ spec §ROADMAP 達成条件との同期（AC 改訂案） |
| 3. M3 の root ZStack lightbox を拡張するか別 fixture か | **拡張する**。lightbox は ZStack のままとし、modal focus scope をそこで実証（framing §粒度の定義〔重ね表示〕、spec §アプリ仕様 A）。ただし focus 意味論の機構検証（group 内移動 / focus と活性項目の分離）には別途 app 非依存の最小 fixture を置く |
| 4. `Button.clicked` と generic hit handler の関係の明文化 | event routing phase の ADR の**必須論点**として予約（framing §粒度の定義〔フォーカスモデル〕）。hit-test eligibility を同じ論点に含める |
| 5. focus trap / modal / accessibility の phase 分割 | `milestone-4/plan.md`（workflow §1.4）で確定する。framing §初期フェーズ分割の仮説が依存方向の見立てを与える |
| 6. top-layer を M4 で開くか | **開く**。構造（宣言位置から window 水準の最前面層へ実体を移す）+ 閉じ方 / focus の規則一式まで。widget への吸着は含めない（→ [§分類の結果](#分類の結果2026-07-28-確定) の隣接項目） |

## M3 側への読み替え

M3 はこのノートの項目を実装しない。

- M3-Phase 6 は Button click handler による open/close を proof とする。
- M3-Phase 6 は arbitrary hit handler、click-away close、focus trap、keyboard close
  を約束しない。
- M3-Phase 8 の full gallery E2E で、これらが v1 必須に近づいた場合は、本ノートを
  参照して M4/M5 への明示的な carry-forward または scope revision を判断する。
