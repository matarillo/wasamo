# M3-Phase 8 GUI Evidence

This directory stores assistant-visible screenshot evidence for Phase 8.
The T7 frames are the authoritative assistant GUI evidence package for the
final post-T6 Gallery surface. Earlier T2/T5/T6 frames are prechecks.

## T7 frame set

Capture script:
`process/milestone-3/phase-8/implementation/evidence/capture-t7-gallery.ps1`

Environment requirement: visible Windows desktop, outside the filesystem
sandbox, per `docs/notes/verification-environments.md` Observation 4.

Frames:

| Frame | Purpose |
|---|---|
| `t7-gallery-default-all.png` | Default Rust-host Gallery view after T6; All tab selected. |
| `t7-gallery-selected-albums.png` | Selected/exclusion positive control, frame 2: clicking Albums moves the checked background to Albums and clears All. |
| `t7-gallery-selected-favorites.png` | Selected/exclusion positive control, frame 3: clicking Favorites moves the checked background to Favorites and clears Albums. |
| `t7-gallery-lightbox-open.png` | Lightbox conditional subtree present: scrim, 4:3 placeholder, caption, and nav/close controls visible. |
| `t7-gallery-lightbox-closed.png` | Lightbox conditional subtree absent after the close click; this state-confirming frame prevents later captures from accidentally running under the modal. |
| `t7-gallery-narrow-before-scroll.png` | Narrow viewport wrap/overflow before scrolling; fewer thumbnail columns are visible than in the default frame. |
| `t7-gallery-narrow-after-scroll.png` | Narrow viewport wrap/overflow after Scroll down; thumbnail labels advance from the initial range, proving live scroll offset rather than a static narrow frame. |

## T7 assistant analysis

Selected/exclusion: the default frame shows the All `ToggleButton` with the
checked background and Albums/Favorites with the normal background. The
Albums frame shows the checked background on Albums and All cleared. The
Favorites frame shows the checked background on Favorites and Albums cleared.
This is a two-transition positive control for live alpha exclusion: a static
implementation with one preselected tab cannot produce all three frames.

Lightbox: `lightbox-open` contains the conditional overlay subtree (scrim,
large 4:3 placeholder, caption text, side buttons, and close button).
`lightbox-closed` shows the Gallery content without the overlay. This
present/absent pair proves the `if is_lightbox_open` subtree toggled rather
than merely rendering a default screen.

Wrap/overflow: the default 1200x760 frame shows nine thumbnail columns, while
the 760x420 narrow frame reflows to five columns. After clicking Scroll down,
the visible labels advance from the `IMG 001`-anchored range to the
`IMG 006`-anchored range. Resize plus scroll movement distinguishes real
WrapPanel/ScrollView behaviour from a fixed static thumbnail grid.

Aspect: the thumbnail placeholders in the default and narrow frames are
uniform square 1:1 boxes produced by `Box { aspect: 1:1 }` inside a
`WrapPanel` with `item-cross-size: 88`. Without the aspect constraint, these
Box-with-Text children would shrink to their text instead of producing uniform
square image placeholders. The lightbox frame also shows the 4:3 placeholder
from `Box { aspect: 4:3 }`. These Gallery frames are supported by the Phase 2
aspect tests cited in the T7 log; no new source change is introduced here.

Known M4 residuals: real images, thumbnail hit-testing, wheel/drag scroll,
modal focus, dynamic title/status, and DPI-awareness remain out of M3 or
explicitly deferred.
