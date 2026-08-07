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
    /// `anchors[id]` is the node-address anchor (M4-Phase 2 T7) that
    /// produced focus node `id`: the same raw pointer
    /// `WidgetNode::for_each_ptr` hands its visitor. Written by the same
    /// walk that writes `paths`, at the same index, so `anchors[id]` and
    /// `paths[id]` are two facts about one node rather than two things
    /// that could drift apart (implementation-gates trap #3 — a second
    /// walk that filled this vec independently could disagree with
    /// `paths` about which node is `id` after any edit to either walk).
    anchors: Vec<*mut WidgetNode>,
}

impl FocusProjection {
    /// Walk `root` in pre-order and build the annotated tree, taking each
    /// node's role and enabled state from `WidgetNode::focus_role` — the
    /// derivation DD-M4-P2-003 F3 fixes, with no authored override (that
    /// arrives at M4-Phase 2 T6).
    pub(crate) fn project(root: &WidgetNode) -> Self {
        let mut tree = FocusTree::new();
        let mut paths = Vec::new();
        let mut anchors = Vec::new();
        walk(root, None, Vec::new(), &mut tree, &mut paths, &mut anchors);
        Self {
            tree,
            paths,
            anchors,
        }
    }

    pub(crate) fn tree(&self) -> &FocusTree {
        &self.tree
    }

    /// The tree path that produced focus node `id`, or `None` when `id` is
    /// not explained by this projection.
    ///
    /// **`Option`, not a bare index.** [`WindowFocus`] retains a
    /// [`FocusId`] across messages while this projection is rebuilt fresh
    /// per operation (this type's own doc comment), so a structural
    /// removal between two messages can leave the retained id naming a
    /// node this projection never built — `docs/architecture.md` §13.3,
    /// DD-M4-P2-003. Returning `Option` makes that case unrepresentable at
    /// the call site: a caller cannot forget to check it the way a bare
    /// `&self.paths[id]` index invited back the panic this type exists to
    /// rule out. [`WindowFocus::rebase`] is what keeps the id this reads
    /// pointed at a node the *current* projection can explain in the
    /// first place (M4-Phase 2 T7); this `Option` is what remains once
    /// that primitive has run — "nothing is focused" and "the id could
    /// not be explained" collapsing to the one outcome a caller here needs
    /// to distinguish from "found".
    pub(crate) fn path(&self, id: FocusId) -> Option<&[usize]> {
        self.paths.get(id).map(Vec::as_slice)
    }

    /// The focus id of the widget at `path`, or `None` when nothing in
    /// this projection was built from it.
    pub(crate) fn id_of_path(&self, path: &[usize]) -> Option<FocusId> {
        self.paths.iter().position(|p| p.as_slice() == path)
    }

    /// This projection's anchor vector — `anchors[id]` is the node address
    /// that produced focus node `id`. [`WindowFocus::rebase`] is the one
    /// caller (M4-Phase 2 T7); see this struct's `anchors` field doc
    /// comment for why the vector exists at all.
    pub(crate) fn anchors(&self) -> &[*mut WidgetNode] {
        &self.anchors
    }

    /// The focus id whose anchor is `node`, or `None` when this
    /// projection's walk never produced one — the node has left the tree
    /// (removed), or has not yet entered it (not attached under the root
    /// this projection was built from).
    ///
    /// **Compared, never dereferenced.** A pointer surviving as *some*
    /// anchor in `self.anchors` says only that the address matches a node
    /// this walk visited; nothing here reads through it. That is what
    /// makes a stale entry *detectable* rather than a silent
    /// use-after-free: `rebase`'s caller never has a live `&WidgetNode` to
    /// hand back for an anchor this method returns `None` for, and this
    /// method itself never manufactures one.
    ///
    /// **What comparison does not rule out, stated rather than implied.**
    /// An address freed by `widget_destroy` can be handed back by a later
    /// allocation, so an anchor naming a removed node can in principle
    /// match a *different*, newly built node at the same address. The
    /// consequence is bounded to "focus lands on an unexpected widget" —
    /// never an unsound read, since nothing dereferences an anchor — and
    /// the window is narrow, because every structural mutation rebases at
    /// the end of the same drain (`emit::flush_layout`). It is the residual
    /// that remains after CF-T5-1's in-range case is closed, and is
    /// recorded as such rather than claimed away.
    pub(crate) fn id_of_anchor(&self, node: *mut WidgetNode) -> Option<FocusId> {
        self.anchors.iter().position(|&p| p == node)
    }
}

fn walk(
    node: &WidgetNode,
    parent: Option<FocusId>,
    path: Vec<usize>,
    tree: &mut FocusTree,
    paths: &mut Vec<Vec<usize>>,
    anchors: &mut Vec<*mut WidgetNode>,
) {
    let index = paths.len();
    let (role, enabled) = node.focus_role();
    let id = tree.push(parent, role, enabled);
    debug_assert_eq!(id, index, "FocusId is the pre-order index by construction");
    paths.push(path.clone());
    // Taken the same way `WidgetNode::for_each_ptr` takes it: a raw
    // pointer from the shared reference this walk already holds, so the
    // vector this one walk builds is what `paths` is indexed against too
    // (this function's own doc comment, and `FocusProjection::anchors`'s).
    anchors.push(node as *const WidgetNode as *mut WidgetNode);
    for (i, child) in node.children.iter().enumerate() {
        let mut child_path = path.clone();
        child_path.push(i);
        walk(child, Some(id), child_path, tree, paths, anchors);
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
///
/// `anchors` (M4-Phase 2 T7) is **the coordinate system the ids in `core`
/// are expressed in** — for each id `core` might retain, the node that
/// produced it in the projection those ids were last written against. It
/// is compared and never dereferenced (see
/// `FocusProjection::id_of_anchor`'s doc comment for why that is what
/// makes a stale entry *detectable* rather than a silent
/// use-after-free): a projection built later can walk a differently
/// shaped tree, and `anchors[old_id]` is what lets [`Self::rebase`] ask
/// "where is the node that used to be `old_id`" instead of trusting that
/// `old_id` still names it.
#[derive(Default)]
pub(crate) struct WindowFocus {
    core: focus_core::FocusState,
    anchors: Vec<*mut WidgetNode>,
}

impl WindowFocus {
    /// The currently focused node, or `None` when nothing is. A read-only
    /// escape hatch onto `core` for callers that need only the id and not
    /// the write access `move_focus` has — [`focused_path`] is the one
    /// today.
    pub(crate) fn focused(&self) -> Option<FocusId> {
        self.core.focused()
    }

    /// Re-express every id `core` retains in `projection`'s coordinate
    /// system, then adopt `projection`'s anchors as the new coordinate
    /// system going forward.
    ///
    /// **This is `discard_stale_focus`'s successor and closes the gap its
    /// own doc comment recorded as CF-T5-1.** That function only caught
    /// the *out-of-range* case: an id past the end of the new projection
    /// could not have come from it, so it was cleared. An id that stayed
    /// *in range* but, after a structural insert or removal, now named a
    /// *different* node was undetectable to it — the bounded exposure its
    /// doc comment left for this task. Rebasing through node addresses
    /// closes that gap: an id whose anchor is no longer in the tree maps
    /// to `None` (the old out-of-range case, generalised), and an id that
    /// shifted to
    /// a different position is *detected*, because its anchor's new
    /// position — not its old numeric value — is what the mapping looks
    /// up.
    ///
    /// **The two fields are written together, in this one function,
    /// deliberately** (implementation-gates trap #3): `core`'s ids and
    /// `anchors`, the coordinate system they are expressed in, are a
    /// derived pair, exactly as [`move_focus`] pairs the retained record
    /// and the painted indicator. A caller that rebased `core` without
    /// replacing `anchors` (or vice versa) would leave the pair
    /// inconsistent with no compiler check to catch it — `anchors` stays
    /// private to this module for the same reason `core` has no setter.
    ///
    /// Every call site must run this **before** anything reads `core`
    /// against `projection` — a caller that reads `core` first defeats the
    /// whole point, since the ids it would read are still expressed in the
    /// *previous* coordinate system.
    pub(crate) fn rebase(&mut self, projection: &FocusProjection) {
        // Snapshotted rather than borrowed for the closure below: `remap`
        // takes `&mut self.core`, and holding a borrow of `self.anchors`
        // across that call would need the disjoint-field argument this
        // clone sidesteps entirely (raw pointers are cheap to copy).
        let old_anchors = self.anchors.clone();
        self.core.remap(|old_id| {
            let ptr = *old_anchors.get(old_id)?;
            projection.id_of_anchor(ptr)
        });
        self.anchors = projection.anchors().to_vec();
    }
}

/// Project `root` and rebase `focus` against it — the call every seam that
/// materialises or removes a subtree makes, so the retained record is
/// re-expressed in the current tree's coordinate system as soon as the
/// tree changes, rather than only lazily at the next focus operation.
///
/// Two production call sites (M4-Phase 2 T7's seam enumeration):
/// `window::set_root`, right after the initial layout pass — the window's
/// tree has just been replaced wholesale, so `focus` (itself freshly
/// reset) needs a coordinate system for the tree that now exists — and
/// `emit::flush_layout`, which every reactive structural mutation reaches
/// through `mark_layout_dirty_for` (the four call sites in
/// `ir_loader.rs`: conditional insert / remove, `for`-range insert /
/// remove). See `flush_layout`'s own call site for why the rebase lives
/// there rather than at `insert_structural_child` / `remove_structural_child`
/// themselves.
pub(crate) fn rebase_to_current_tree(root: &WidgetNode, focus: &mut WindowFocus) {
    let projection = FocusProjection::project(root);
    focus.rebase(&projection);
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
    // `path` returning `None` for the previous id is not an error case to
    // route around: a previous id this projection cannot explain simply
    // means there is nothing to leave (`WindowFocus::rebase` already
    // re-expresses this at both call sites before `move_focus` runs, but
    // the `Option` handling here does not depend on that order).
    let prev_path = focus
        .core
        .focused()
        .and_then(|id| projection.path(id))
        .map(|p| p.to_vec());
    let next_path = next.and_then(|id| projection.path(id)).map(|p| p.to_vec());

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
    // Must run before anything below reads `focus.core`: `FocusTree::tab`
    // (via `stop_index_of` -> `nearest`) indexes `self.nodes[id]` directly
    // in `focus_core`, so a retained id must already be expressed in this
    // fresh projection's coordinate system before it reaches there.
    focus.rebase(&projection);
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

/// Move focus to the click landing at or above the resolved click target,
/// or leave focus unchanged when there is none (`docs/dsl_spec.md` §4.19
/// "clicking background never clears focus").
///
/// The chain is built from `hit::dispatch_chain` — **the same ancestor
/// walk DD-001's click dispatch uses** (DD-M4-P2-003 "The walk is DD-001's
/// ancestor walk, so this adds no second traversal") — rather than a
/// second walk up the tree, so the click's focus resolution and the
/// click's dispatch resolution read "at or above" off the identical
/// ancestor list and cannot disagree.
///
/// **The landing rule is `focus_core::FocusTree::focus_landing`, not
/// `tab_stops` + a nearest-stop search (M4-Phase 2 T7, closing CF-T6-5).**
/// The T5-era version enumerated Tab stops and found the nearest one on
/// the chain — which meant a `focus-group` container contributed *itself*
/// to that list and its members did not appear at all, so a click on a
/// widget inside a group moved focus to the group container rather than
/// the clicked member. `focus_landing` walks the chain directly against
/// node roles instead, so a member is reached before its enclosing group
/// ever is; see that function's own doc comment for the full rule and its
/// relationship to `tab_stops`.
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
    // Must run before anything below reads `focus.core`, for the same
    // reason `traverse_on_key` runs it first: `focus_landing` below takes
    // `&focus.core` and walks from `tree.traversal_root(&focus.core)`, and
    // a retained id must already be expressed in this fresh projection's
    // coordinate system before either is reached.
    focus.rebase(&projection);
    let chain: Vec<FocusId> = hit::dispatch_chain(&target_path)
        .into_iter()
        .filter_map(|path| projection.id_of_path(&path))
        .collect();
    let Some(next) = projection.tree().focus_landing(&focus.core, &chain) else {
        return Ok(());
    };
    move_focus(root, compositor, focus, &projection, Some(next))
}

/// The focused widget's path, or `None` when nothing is focused, or when
/// the retained id names a node this freshly built projection cannot
/// explain.
///
/// **Read-only, and does not call [`WindowFocus::rebase`] itself.** By the
/// time this runs, every seam that can change the tree shape has already
/// rebased `focus` against its own projection: `move_focus`'s two
/// production callers rebase before they read `focus.core`
/// (`traverse_on_key`, `focus_on_click`), and every structural mutation
/// reaches [`rebase_to_current_tree`] through `window::set_root` or
/// `emit::flush_layout` (M4-Phase 2 T7). So the id this reads is already
/// expressed in a coordinate system the *live* tree can explain, and the
/// projection built here agrees with it; the `Option` on
/// `FocusProjection::path` is what remains for the one case rebasing
/// cannot remove — nothing is focused at all — which collapses to the
/// same `None` a caller here needs to distinguish from "found" either way.
///
/// Used by the test seam (`lib.rs::ffi::__focus_path_for_test`). **A
/// reader, not a second store**: the window's one record is the
/// [`FocusId`] held by [`WindowFocus`], and this function derives a path
/// from it on demand by projecting the live tree, rather than reading a
/// path field that could drift from the id.
pub(crate) fn focused_path(root: &WidgetNode, focus: &WindowFocus) -> Option<Vec<usize>> {
    let id = focus.focused()?;
    let projection = FocusProjection::project(root);
    projection.path(id).map(|p| p.to_vec())
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
}
