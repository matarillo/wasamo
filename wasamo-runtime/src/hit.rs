//! Pure-logic hit resolution (DD-M4-P2-002 "Target selection: T2 over T1"
//! and "Containment is bounded by ancestor clips").
//!
//! No Compositor, no `windows` types: the resolver is generic over a
//! [`HitTree`] view, so it is testable with a plain in-memory tree
//! (AGENTS.md §Testing rules — pure layout math gets unit tests, not a
//! mocked OS surface). `WidgetNode` supplies the production `HitTree` impl
//! in `widget.rs`, reading `arranged_rect` (T1's store) and
//! `clips_children`.

use crate::widget::DipRect;

/// A point in the same space as [`DipRect`]: absolute DIP within the
/// window's client area.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DipPoint {
    pub x: f32,
    pub y: f32,
}

/// The tree view [`resolve_topmost`] needs. Implemented by `WidgetNode`
/// (production) and by a plain mirror struct in this module's tests (no
/// Compositor).
pub(crate) trait HitTree {
    /// This node's own arranged rectangle, or `None` when it has never
    /// been through a layout pass — DD-M4-P2-002's "not hit-testable" is
    /// the intended failure, not a bug to work around here.
    fn hit_rect(&self) -> Option<DipRect>;

    /// Whether this node clips its subtree to its own rectangle
    /// (`ScrollView`, `Grid`, `ZStack` install a zero-inset clip at
    /// construction; see `WidgetNode::clips_children`).
    fn hit_clips_children(&self) -> bool;

    /// Number of children, in paint order (first = bottom, last = top —
    /// the same order `sync_visuals` walks and the Composition tree
    /// paints).
    fn hit_child_count(&self) -> usize;

    /// The child at `index`, which is always `< hit_child_count()`.
    fn hit_child(&self, index: usize) -> &Self;
}

/// Does `rect` contain `point`?
///
/// **Left/top edges inclusive, right/bottom edges exclusive** — the same
/// half-open convention the pre-T2 `hit_test_click_inner` used
/// (`x >= abs_x && x < abs_x + vw`), kept because it is what makes two
/// abutting widgets partition the plane with no pixel claimed by both and
/// none claimed by neither.
pub(crate) fn contains(rect: DipRect, point: DipPoint) -> bool {
    point.x >= rect.x
        && point.x < rect.x + rect.width
        && point.y >= rect.y
        && point.y < rect.y + rect.height
}

/// The topmost node containing `point`, as the path of child indices from
/// `root` (an empty vec means `root` itself). `None` when nothing
/// contains it.
///
/// DD-M4-P2-002 "Target selection: T2 over T1" / "Containment is bounded
/// by ancestor clips" — the algorithm, in the order it is checked:
///
/// 1. **If `root` clips its children and `point` is not inside `root`'s
///    own rectangle (including the case where `root` has no rectangle at
///    all), resolve to `None` immediately**: neither `root` nor anything
///    in its subtree is a candidate. This is fail-closed by construction —
///    a clipping node with no rectangle has nothing to bound its subtree
///    with, so the safe answer is "nothing here" rather than "everything
///    here." A clipping container paints nothing of its subtree outside
///    its own rectangle, and hit-testing resolves what the screen shows.
/// 2. **Otherwise, visit children in reverse index order.** Later
///    children paint over earlier ones (document order = paint order,
///    the same order `sync_visuals` walks), so the reverse walk visits
///    topmost-first; the first child that resolves wins, and its index
///    is prefixed onto the path it returned.
/// 3. **Otherwise, `root` itself is the target if its own rectangle
///    contains `point`.**
/// 4. **Otherwise, `None`.**
///
/// **The asymmetry between clipping and non-clipping nodes with no
/// rectangle is deliberate.** A *non-clipping* node with no rectangle
/// still has its children visited at step 2 — its children's rectangles
/// are independent facts step 1 never gated — so its reachable subtree is
/// unaffected; only step 3 excludes the node itself from being a
/// candidate (`None` can never satisfy `contains`). A *clipping* node with
/// no rectangle is stopped at step 1: a clip is a bound on the whole
/// subtree, not just the node's own candidacy, and there is no rectangle
/// to bound it with.
pub(crate) fn resolve_topmost<T: HitTree>(root: &T, point: DipPoint) -> Option<Vec<usize>> {
    let own_rect = root.hit_rect();

    // Step 1: the clip bound. Only a clipping node's own rectangle gates
    // its subtree; a non-clipping node falls through to step 2
    // regardless of whether it has a rectangle.
    if root.hit_clips_children() {
        match own_rect {
            Some(rect) if contains(rect, point) => {}
            _ => return None,
        }
    }

    // Step 2: reverse-order descent — topmost child first.
    for i in (0..root.hit_child_count()).rev() {
        if let Some(mut path) = resolve_topmost(root.hit_child(i), point) {
            path.insert(0, i);
            return Some(path);
        }
    }

    // Step 3 / 4: the node itself, only if it has a rectangle that
    // contains the point.
    match own_rect {
        Some(rect) if contains(rect, point) => Some(Vec::new()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A plain, Compositor-free mirror of the two facts `resolve_topmost`
    /// reads off a `WidgetNode`: its own rectangle and whether it clips.
    /// Exists so the resolution algorithm is pinned with no Win32/WinRT
    /// dependency (AGENTS.md §Testing rules).
    struct TestNode {
        rect: Option<DipRect>,
        clips: bool,
        children: Vec<TestNode>,
    }

    impl TestNode {
        fn leaf(rect: DipRect) -> Self {
            Self {
                rect: Some(rect),
                clips: false,
                children: Vec::new(),
            }
        }

        fn container(rect: DipRect, clips: bool, children: Vec<TestNode>) -> Self {
            Self {
                rect: Some(rect),
                clips,
                children,
            }
        }

        fn no_rect(clips: bool, children: Vec<TestNode>) -> Self {
            Self {
                rect: None,
                clips,
                children,
            }
        }
    }

    impl HitTree for TestNode {
        fn hit_rect(&self) -> Option<DipRect> {
            self.rect
        }

        fn hit_clips_children(&self) -> bool {
            self.clips
        }

        fn hit_child_count(&self) -> usize {
            self.children.len()
        }

        fn hit_child(&self, index: usize) -> &Self {
            &self.children[index]
        }
    }

    fn rect(x: f32, y: f32, width: f32, height: f32) -> DipRect {
        DipRect {
            x,
            y,
            width,
            height,
        }
    }

    fn pt(x: f32, y: f32) -> DipPoint {
        DipPoint { x, y }
    }

    #[test]
    fn the_later_of_two_overlapping_siblings_wins_occlusion() {
        let tree = TestNode::container(
            rect(0.0, 0.0, 100.0, 100.0),
            false,
            vec![
                TestNode::leaf(rect(0.0, 0.0, 50.0, 50.0)),
                // Later in document order == painted on top; overlaps the
                // first child at the probed point.
                TestNode::leaf(rect(10.0, 10.0, 50.0, 50.0)),
            ],
        );
        assert_eq!(
            resolve_topmost(&tree, pt(20.0, 20.0)),
            Some(vec![1]),
            "the later (topmost-painted) sibling must win where both contain the point"
        );
    }

    #[test]
    fn a_child_wins_over_its_parent_when_both_contain_the_point() {
        let tree = TestNode::container(
            rect(0.0, 0.0, 100.0, 100.0),
            false,
            vec![TestNode::leaf(rect(10.0, 10.0, 20.0, 20.0))],
        );
        assert_eq!(
            resolve_topmost(&tree, pt(15.0, 15.0)),
            Some(vec![0]),
            "a child paints over its parent, so it must resolve ahead of the container"
        );
    }

    #[test]
    fn a_container_is_the_target_when_the_point_is_inside_it_but_in_none_of_its_children() {
        let tree = TestNode::container(
            rect(0.0, 0.0, 100.0, 100.0),
            false,
            vec![TestNode::leaf(rect(10.0, 10.0, 20.0, 20.0))],
        );
        assert_eq!(
            resolve_topmost(&tree, pt(80.0, 80.0)),
            Some(Vec::new()),
            "with no child under the point, the container itself is the target"
        );
    }

    /// The clip-bound arm and its agreement leg in one test, per the task's
    /// pairing requirement: a point inside a clipping ancestor's subtree
    /// but outside that ancestor's own rectangle must not resolve into the
    /// subtree, and the identical tree with `clips` false must resolve to
    /// the same child — proving the `None` above is the clip bound at
    /// work, not a geometry mistake that would also produce `None` under a
    /// non-clipping parent.
    #[test]
    fn a_clip_excludes_an_overflowing_childs_rect_and_the_same_tree_without_clip_resolves_it() {
        // Child's rectangle extends past its parent's.
        let child_rect = rect(40.0, 40.0, 40.0, 40.0);
        let parent_rect = rect(0.0, 0.0, 50.0, 50.0);
        let probe = pt(70.0, 70.0); // inside child_rect, outside parent_rect

        let clipping = TestNode::container(parent_rect, true, vec![TestNode::leaf(child_rect)]);
        assert_eq!(
            resolve_topmost(&clipping, probe),
            None,
            "a clipping ancestor's subtree must not resolve outside its own rectangle, \
             even where a child's own rectangle would otherwise contain the point"
        );

        // Agreement leg: same rectangles, `clips` false.
        let non_clipping =
            TestNode::container(parent_rect, false, vec![TestNode::leaf(child_rect)]);
        assert_eq!(
            resolve_topmost(&non_clipping, probe),
            Some(vec![0]),
            "agreement leg: with no clip, the identical overflowing child must resolve — \
             so the None above is the clip bound, not a coordinate error"
        );
    }

    #[test]
    fn a_node_with_no_rectangle_is_not_a_candidate_but_its_non_clipping_subtree_is_still_reachable()
    {
        let tree = TestNode::no_rect(false, vec![TestNode::leaf(rect(0.0, 0.0, 10.0, 10.0))]);
        assert_eq!(
            resolve_topmost(&tree, pt(5.0, 5.0)),
            Some(vec![0]),
            "the child's own rectangle is an independent fact step 1 never gates for a \
             non-clipping parent"
        );
        assert_eq!(
            resolve_topmost(&tree, pt(50.0, 50.0)),
            None,
            "a node with no rectangle can never be the target itself, even where no child \
             is under the point either"
        );
    }

    #[test]
    fn a_clipping_node_with_no_rectangle_makes_its_subtree_unreachable() {
        let tree = TestNode::no_rect(true, vec![TestNode::leaf(rect(0.0, 0.0, 10.0, 10.0))]);
        assert_eq!(
            resolve_topmost(&tree, pt(5.0, 5.0)),
            None,
            "fail closed: a clipping node with no rectangle cannot have its bound checked, \
             so its subtree is not a candidate at all"
        );
    }

    /// DD-V-029 boundary-condition red-test obligation: a deliberately
    /// wrong containment implementation (e.g. `<=` on the right/bottom
    /// edges, or excluding the left/top edges) must make this test fail.
    /// Verified by mutation at the T2 close gate.
    #[test]
    fn edge_containment_includes_the_left_and_top_edges_and_excludes_the_right_and_bottom() {
        // x in [10, 40), y in [20, 60).
        let r = rect(10.0, 20.0, 30.0, 40.0);

        // The four edges themselves.
        assert!(contains(r, pt(10.0, 30.0)), "left edge must be included");
        assert!(contains(r, pt(20.0, 20.0)), "top edge must be included");
        assert!(!contains(r, pt(40.0, 30.0)), "right edge must be excluded");
        assert!(!contains(r, pt(20.0, 60.0)), "bottom edge must be excluded");

        // Just-inside / just-outside neighbours of each edge.
        assert!(
            contains(r, pt(39.999, 30.0)),
            "just inside the right edge must still be included"
        );
        assert!(
            contains(r, pt(20.0, 59.999)),
            "just inside the bottom edge must still be included"
        );
        assert!(
            !contains(r, pt(9.999, 30.0)),
            "just outside the left edge must be excluded"
        );
        assert!(
            !contains(r, pt(20.0, 19.999)),
            "just outside the top edge must be excluded"
        );
    }

    #[test]
    fn the_returned_path_identifies_which_of_two_identically_shaped_sibling_subtrees_was_hit() {
        let subtree = |x_offset: f32| {
            TestNode::container(
                rect(x_offset, 0.0, 10.0, 10.0),
                false,
                vec![TestNode::leaf(rect(x_offset, 0.0, 10.0, 10.0))],
            )
        };
        let tree = TestNode::container(
            rect(0.0, 0.0, 100.0, 100.0),
            false,
            vec![subtree(0.0), subtree(50.0)],
        );
        assert_eq!(
            resolve_topmost(&tree, pt(5.0, 5.0)),
            Some(vec![0, 0]),
            "a point in the first identically-shaped subtree must name index 0"
        );
        assert_eq!(
            resolve_topmost(&tree, pt(55.0, 5.0)),
            Some(vec![1, 0]),
            "a point in the second identically-shaped subtree must name index 1, not 0"
        );
    }

    #[test]
    fn nothing_under_the_point_yields_none() {
        let tree = TestNode::container(
            rect(0.0, 0.0, 10.0, 10.0),
            false,
            vec![TestNode::leaf(rect(0.0, 0.0, 5.0, 5.0))],
        );
        assert_eq!(resolve_topmost(&tree, pt(500.0, 500.0)), None);
    }
}
