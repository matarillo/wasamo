//! Per-window focus record and the projection that lets it read a live
//! widget tree (M4-Phase 2 T5; DD-M4-P2-003 "the spike's traversal core is
//! adopted as the implementation").
//!
//! This is where `crate::focus_core`'s pure traversal core meets
//! `crate::widget::WidgetNode`: [`FocusProjection::project`] walks the tree
//! once per operation and builds the annotated `focus_core::FocusTree` the
//! core needs — the same relationship `crate::hit` has to hit-test
//! rectangles. [`WindowFocus`] is the one record `docs/architecture.md`
//! §13.3 puts on `WindowState` ("A `WindowState` owns one focus record");
//! `window::WindowState::focus` is its only production home.
//!
//! **Not `pub`.** The phase's cross-task obligation is "no new ABI
//! function" (`process/milestone-4/phase-2/requirements/constraints.md`
//! §2), and nothing outside this crate needs a `FocusId`.

use crate::focus_core::{self, FocusId, FocusTree, TabDirection};
use crate::hit::{self, DipPoint};
use crate::widget::WidgetNode;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_TAB;
use windows::UI::Composition::Compositor;

/// A projection of a live widget tree onto [`FocusTree`], plus the mapping
/// back to the tree paths that produced each node.
///
/// Rebuilt fresh for every keyboard or click operation rather than cached
/// on [`WindowFocus`]: a handler's synchronous rebuild can invalidate a
/// path between two calls into `widget.rs` (`node_at_path_mut`'s doc
/// comment in that module), so a projection cached across such a rebuild
/// would carry exactly the staleness hazard bounds-checked descent exists
/// to avoid. The tree itself is small (one per window's widget count), so
/// rebuilding it per operation is the same cost class as
/// `hit::resolve_topmost`'s per-message walk.
pub(crate) struct FocusProjection {
    tree: FocusTree,
    /// `paths[id]` is the path of child indices from the projection root
    /// to the widget that produced focus node `id`; `paths[0]` is empty
    /// (the root). Indexed by [`FocusId`], which is the pre-order index of
    /// the walk in [`walk`] by construction — the same pairing
    /// `focus_spike::Projection`'s `widgets` vec keeps for its override-map
    /// projection.
    paths: Vec<Vec<usize>>,
}

impl FocusProjection {
    /// Walk `root` in pre-order and build the annotated tree, taking each
    /// node's role and enabled state from `WidgetNode::focus_role` — the
    /// derivation DD-M4-P2-003 F3 fixes, with no authored override (that
    /// arrives at M4-Phase 2 T6).
    pub(crate) fn project(root: &WidgetNode) -> Self {
        let mut tree = FocusTree::new();
        let mut paths = Vec::new();
        walk(root, None, Vec::new(), &mut tree, &mut paths);
        Self { tree, paths }
    }

    pub(crate) fn tree(&self) -> &FocusTree {
        &self.tree
    }

    /// The tree path that produced focus node `id`.
    pub(crate) fn path(&self, id: FocusId) -> &[usize] {
        &self.paths[id]
    }

    /// The focus id of the widget at `path`, or `None` when nothing in
    /// this projection was built from it.
    pub(crate) fn id_of_path(&self, path: &[usize]) -> Option<FocusId> {
        self.paths.iter().position(|p| p.as_slice() == path)
    }
}

fn walk(
    node: &WidgetNode,
    parent: Option<FocusId>,
    path: Vec<usize>,
    tree: &mut FocusTree,
    paths: &mut Vec<Vec<usize>>,
) {
    let index = paths.len();
    let (role, enabled) = node.focus_role();
    let id = tree.push(parent, role, enabled);
    debug_assert_eq!(id, index, "FocusId is the pre-order index by construction");
    paths.push(path.clone());
    for (i, child) in node.children.iter().enumerate() {
        let mut child_path = path.clone();
        child_path.push(i);
        walk(child, Some(id), child_path, tree, paths);
    }
}

/// This window's retained focus record (M4-Phase 2 T5; `docs/architecture.md`
/// §13.3 "A `WindowState` owns one focus record..." — DD-M4-P2-003's L2:
/// one `FocusState` per `WindowState`, no global, no static).
///
/// `core` is private to this module and carries no setter — the same
/// visibility discipline `crate::widget::HoverState` uses for `target`,
/// and for the identical reason (T4 trap #3, adopted by DD-M4-P2-003 as
/// "the focus pointer... is not independently writable"): [`move_focus`]
/// is the only function in the crate that calls `FocusState::set_focus`,
/// so the focused id and the painted indicator can never be written apart.
#[derive(Default)]
pub(crate) struct WindowFocus {
    core: focus_core::FocusState,
}

impl WindowFocus {
    /// The currently focused node, or `None` when nothing is. A read-only
    /// escape hatch onto `core` for callers that need only the id and not
    /// the write access `move_focus` has — [`focused_path`] is the one
    /// today.
    pub(crate) fn focused(&self) -> Option<FocusId> {
        self.core.focused()
    }
}

/// `Tab` / `Shift+Tab` are the only keys traversal takes at T5
/// (`docs/dsl_spec.md` §4.19 §Which keys the runtime keeps): arrow keys and
/// `Escape` are conditioned on a `focus-group` / an entered `modal-scope`,
/// and neither annotation exists before M4-Phase 2 T6/T7.
///
/// `VK_TAB` is imported and compared against, rather than hard-coded as
/// `0x09`, so the constant this compares is the OS's, not a copy of it.
pub(crate) fn tab_direction(vk: u16, shift_down: bool) -> Option<TabDirection> {
    if vk != VK_TAB.0 {
        return None;
    }
    Some(if shift_down {
        TabDirection::Backward
    } else {
        TabDirection::Forward
    })
}

/// The first entry of `chain` (target-first, root-last — `hit::dispatch_chain`'s
/// order) that is also a Tab stop.
///
/// **Deliberately defined against the very list traversal enumerates**, so
/// "what is focusable" has exactly one definition in the runtime: this
/// function and `FocusTree::tab_stops` cannot drift apart the way two
/// independent predicates could (implementation-gates trap #3).
pub(crate) fn nearest_focusable(chain: &[FocusId], stops: &[FocusId]) -> Option<FocusId> {
    chain.iter().copied().find(|id| stops.contains(id))
}

/// Write the window's focused id and the painted indicator together
/// (M4-Phase 2 T5; DD-M4-P2-003 "the focus pointer and the group memory
/// are written by one primitive"). **This is the single primitive**: no
/// other function in the crate calls both `WidgetNode::set_button_focused_at`
/// and `FocusState::set_focus`, which is what keeps the retained record and
/// the painted flag from being edited apart — the same shape
/// `WidgetNode::update_hover` gives `HoverState` (T4 trap #3).
///
/// Body order: the previous node's paint is cleared first, and only when
/// its path actually differs from the next node's — moving focus off and
/// back onto the same widget in one call must not flash it, mirroring
/// `hit::hover_leave_target`'s `previous == next` guard. The next node's
/// paint is set second, and the record is written last.
///
/// **A Composition failure can leave the pair out of step, and that is
/// stated rather than claimed away.** `set_button_focused_at` flips the
/// node's flag before it starts the colour animation, so an animation
/// that fails propagates out of here with the flag already moved and the
/// record not yet written. The exposure is inherited rather than
/// introduced: `set_button_state_at` and `WidgetNode::update_hover` carry
/// exactly the same one for `ButtonState` and `HoverState` (T4). What the
/// ordering does buy is that no *successful* call sequence can write one
/// half without the other.
pub(crate) fn move_focus(
    root: &mut WidgetNode,
    compositor: &Compositor,
    focus: &mut WindowFocus,
    projection: &FocusProjection,
    next: Option<FocusId>,
) -> windows::core::Result<()> {
    let prev_path = focus.core.focused().map(|id| projection.path(id).to_vec());
    let next_path = next.map(|id| projection.path(id).to_vec());

    if prev_path != next_path {
        if let Some(path) = prev_path.as_deref() {
            root.set_button_focused_at(compositor, path, false)?;
        }
    }
    if let Some(path) = next_path.as_deref() {
        root.set_button_focused_at(compositor, path, true)?;
    }

    focus.core.set_focus(projection.tree(), next);
    Ok(())
}

/// Consume `Tab` / `Shift+Tab` ahead of the host key slot; every other key
/// is left untouched so the caller can fall it through
/// (`docs/architecture.md` §13.2 "a key nothing consumes is not
/// swallowed").
///
/// Returns **`true` when the key was consumed by traversal**, including
/// when the domain has no stop at all (`FocusTree::tab` returns `None`):
/// `docs/dsl_spec.md` §4.19 "`Tab` / `Shift+Tab` — Always the runtime;
/// traversal cannot be overridden" makes no exception for an empty
/// traversal domain, so `Tab` over a window with nothing focusable is
/// still Tab's to consume — it just moves focus nowhere.
pub(crate) fn traverse_on_key(
    root: &mut WidgetNode,
    compositor: &Compositor,
    focus: &mut WindowFocus,
    vk: u16,
    shift_down: bool,
) -> bool {
    let Some(dir) = tab_direction(vk, shift_down) else {
        return false;
    };
    let projection = FocusProjection::project(root);
    let next = projection.tree().tab(&focus.core, dir);
    // The `Result` is dropped rather than returned, because the answer
    // this function owes its caller is "was the key consumed", and it was:
    // traversal claimed `Tab` before anything could fail. A failed
    // indicator repaint must not turn into the key escaping to
    // `DefWindowProc`. This matches how `window.rs`'s pointer arms already
    // treat `update_hover`'s `Result` — a Composition failure is not a
    // reason to re-route the message.
    let _ = move_focus(root, compositor, focus, &projection, next);
    true
}

/// Move focus to the nearest focusable widget at or above the resolved
/// click target, or leave focus unchanged when there is none
/// (`docs/dsl_spec.md` §4.19 "clicking background never clears focus").
///
/// The chain is built from `hit::dispatch_chain` — **the same ancestor
/// walk DD-001's click dispatch uses** (DD-M4-P2-003 "The walk is DD-001's
/// ancestor walk, so this adds no second traversal") — rather than a
/// second walk up the tree, so the click's focus resolution and the
/// click's dispatch resolution read "at or above" off the identical
/// ancestor list and cannot disagree.
pub(crate) fn focus_on_click(
    root: &mut WidgetNode,
    compositor: &Compositor,
    focus: &mut WindowFocus,
    x: f32,
    y: f32,
) -> windows::core::Result<()> {
    let Some(target_path) = hit::resolve_topmost(root, DipPoint { x, y }) else {
        return Ok(());
    };
    let projection = FocusProjection::project(root);
    let chain: Vec<FocusId> = hit::dispatch_chain(&target_path)
        .into_iter()
        .filter_map(|path| projection.id_of_path(&path))
        .collect();
    let tree = projection.tree();
    let stops = tree.tab_stops(&focus.core, tree.traversal_root(&focus.core));
    let Some(next) = nearest_focusable(&chain, &stops) else {
        return Ok(());
    };
    move_focus(root, compositor, focus, &projection, Some(next))
}

/// The focused widget's path, or `None` when nothing is focused.
///
/// Used by the test seam (`lib.rs::ffi::__focus_path_for_test`). **A
/// reader, not a second store**: the window's one record is the
/// [`FocusId`] held by [`WindowFocus`], and this function derives a path
/// from it on demand by projecting the live tree, rather than reading a
/// path field that could drift from the id.
pub(crate) fn focused_path(root: &WidgetNode, focus: &WindowFocus) -> Option<Vec<usize>> {
    let id = focus.focused()?;
    let projection = FocusProjection::project(root);
    Some(projection.path(id).to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    // An arbitrary non-Tab virtual-key code (VK_A), used only to show
    // `tab_direction` yields `None` for a key that is not Tab.
    const VK_A: u16 = 0x41;

    #[test]
    fn tab_without_shift_is_forward() {
        assert_eq!(tab_direction(VK_TAB.0, false), Some(TabDirection::Forward));
    }

    #[test]
    fn tab_with_shift_is_backward() {
        assert_eq!(tab_direction(VK_TAB.0, true), Some(TabDirection::Backward));
    }

    #[test]
    fn a_non_tab_key_is_not_traversals_regardless_of_shift() {
        assert_eq!(tab_direction(VK_A, false), None);
        assert_eq!(tab_direction(VK_A, true), None);
    }

    #[test]
    fn nearest_focusable_returns_the_first_chain_entry_that_is_a_stop() {
        // chain is target-first, root-last: [2, 1, 0]; only 1 and 0 are
        // stops, and 1 is nearer the target than 0.
        assert_eq!(nearest_focusable(&[2, 1, 0], &[0, 1]), Some(1));
    }

    #[test]
    fn nearest_focusable_returns_the_target_itself_when_it_is_a_stop() {
        assert_eq!(nearest_focusable(&[3, 2, 1, 0], &[3]), Some(3));
    }

    #[test]
    fn nearest_focusable_is_none_when_no_ancestor_is_a_stop() {
        assert_eq!(nearest_focusable(&[2, 1, 0], &[5, 6]), None);
    }

    #[test]
    fn nearest_focusable_over_an_empty_chain_is_none() {
        assert_eq!(nearest_focusable(&[], &[0, 1, 2]), None);
    }
}
