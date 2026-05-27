### DD-P4-004 — Button visual structure

**Status:** Accepted

**Context:**
`Button` needs a background layer (fill color that changes on hover/press)
and a text label. Both must be `SpriteVisual` objects parented into the
Visual Layer tree.

**Options:**

Option A — `SpriteVisual` container (background brush) + child text `SpriteVisual`
- The button's root is a `SpriteVisual` with a `CompositionColorBrush`
  as background. A child `SpriteVisual` (created the same way as a `Text`
  widget) is added as an overlay for the label.
  `SpriteVisual` already supports `Children()` (it inherits from
  `ContainerVisual`), so the `append_child` pattern from Phase 3 applies.
- What you gain: Background and label are independent visuals; changing
  hover/press state only requires swapping the background brush on the root
  visual. Consistent with the existing `WidgetNode.visual: SpriteVisual`
  type contract.
- What you give up: Two SpriteVisuals per button.

Option B — Single `ICompositionDrawingSurface` with background + text co-drawn
- Background and label are drawn into one surface via D2D.
- What you gain: One GPU object per button.
- What you give up: Every state change (hover, press) requires redrawing
  the entire surface, including text layout. More complex and slower.

**Decision:** Option A — layered `SpriteVisual` structure.
Background brush swap on the root visual covers all state transitions cheaply.

**Button states and colors (M1):**

| State   | Default style (background)          | Accent style (background)          |
|---------|-------------------------------------|------------------------------------|
| Normal  | `#20FFFFFF` (20% white glass)       | System accent color (`UISettings`) |
| Hover   | `#33FFFFFF` (33% white)             | Accent color lightened by 10%      |
| Pressed | `#10FFFFFF` (10% white)             | Accent color darkened by 10%       |

Text color: `#FFFFFFFF` (always white) for both styles in M1.

**Known limitation (deferred to M2):** The color table above was designed for dark mode.
In light mode, `#20FFFFFF` on a light Mica surface provides insufficient contrast —
the Default button is nearly invisible and white text is unreadable.
The correct fix is theme-aware color sets (dark semi-transparent background + dark text
in light mode, matching WinUI 3 conventions). Deferred to M2 as part of broader
theme-aware widget styling work.

**Animation scope:** Button state transitions in M1 are instant brush swaps,
consistent with DD-V-001 (default behavior is instant; animation is opt-in).
Phase 5's dev-only implicit animation helper covers `Offset`, `Size`, and
`Opacity`; it does not animate `CompositionColorBrush` color changes.
Animated hover/press feedback requires `ColorKeyFrameAnimation` and is
deferred to M5 (public animation API).

---
