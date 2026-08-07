//! Focus traversal core — the Win32-independent half of the M4-Phase 2 focus
//! model (DD-M4-P2-003 "the spike's traversal core is adopted as the
//! implementation").
//!
//! **Consumed through a projection, never directly.** `crate::focus`
//! projects a live `WidgetNode` tree onto the [`FocusTree`] this module
//! defines and drives it from `WindowState`'s keyboard and pointer arms
//! (M4-Phase 2 T5) — that projection, not this module, is what a caller
//! outside this file reaches for. `focus_spike` is a second, override-map
//! projection kept only for `tests/focus_mechanism_fixture.rs` until
//! M4-Phase 2 T7 retires it. Neither caller changes this module's logic:
//! the core stays exactly what the Phase 2 ADR compared options against.
//!
//! # Why an annotated tree of its own rather than annotations on `WidgetNode`
//!
//! Putting focus annotations on `WidgetData` would drag the OS types into
//! what is deliberately pure logic, and would make every focus rule a
//! method on a type this crate also uses for Composition and layout. The
//! core therefore consumes its own index-based tree; a caller projects
//! whatever it has onto that shape. This is the mirror-structure allowance
//! in `CLAUDE.md` §Testing rules, used here at design time rather than
//! only at test time.
//!
//! # What the core deliberately does not know
//!
//! It does not know how an event reached it (the routing model is
//! DD-M4-P2-001), where a node is on screen (hit geometry is DD-M4-P2-002),
//! or how any of this is spelled in `.ui` (DD-M4-P2-005, landing at
//! M4-Phase 2 T6). It answers exactly one question: given a tree, its
//! annotations, and the current focus state, what is the next focus
//! target.

// `Group`, `ActiveItemList` / `ActiveItem`, and `ModalScope` have no
// production caller yet: `crate::focus`'s T5 projection derives only
// `Stop` / `Container` from the widget kind (DD-M4-P2-003 F3), and the
// authored annotations that would produce the other four roles arrive at
// M4-Phase 2 T6, projected into production at T7. The allow stays until
// T7 makes every variant reachable from production.
#![allow(dead_code)]

use std::collections::BTreeMap;

/// Index into [`FocusTree::nodes`]. Stable only for one tree instance; a
/// rebuilt subtree gets new ids, which is why removal is planned *before* the
/// mutation (see [`FocusTree::focus_after_removing`]).
pub type FocusId = usize;

/// What a node is, for traversal purposes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FocusRole {
    /// Structure only: never focused, always descended into.
    Container,
    /// A single Tab stop. Treated as a traversal leaf — see the module's
    /// recorded limit about focusable containers.
    Stop,
    /// A group of stops that Tab treats as **one** stop, and that arrow keys
    /// move within (radio-button-like). Entering the group lands on the
    /// remembered member.
    Group,
    /// A modal scope: while entered, traversal is confined to this subtree.
    /// While *not* entered, its stops are not enumerated at all.
    ModalScope,
    /// A list whose items are activated without moving focus: focus stays on
    /// the owner (the nearest enclosing `Stop`), and arrows move this list's
    /// active item (dropdown-like).
    ActiveItemList,
    /// An item inside an [`FocusRole::ActiveItemList`].
    ActiveItem,
}

#[derive(Clone, Debug)]
pub struct FocusNode {
    pub parent: Option<FocusId>,
    pub children: Vec<FocusId>,
    pub role: FocusRole,
    /// A disabled stop is skipped by traversal. Mirrors the existing narrow
    /// `Button.enabled` contract (DD-M3-P1-005).
    pub enabled: bool,
}

/// The annotated tree. Node 0 is the root by construction.
#[derive(Clone, Debug, Default)]
pub struct FocusTree {
    nodes: Vec<FocusNode>,
}

/// One entered modal scope, with the focus to restore when it leaves.
///
/// The restore target is captured **at entry** because it cannot be recovered
/// from the tree afterwards: nothing in the structure records which node was
/// focused before the scope opened.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ModalEntry {
    pub scope: FocusId,
    pub restore_to: Option<FocusId>,
}

/// The mutable focus state for one window.
#[derive(Clone, Debug, Default)]
pub struct FocusState {
    focused: Option<FocusId>,
    /// Per-group memory of the last member focused inside it (roving).
    /// **Derived data parallel to `focused`** — written by the same primitive
    /// that writes `focused`, never separately (implementation-gates trap 3).
    group_memory: BTreeMap<FocusId, FocusId>,
    /// Per-list active item. Independent of `focused` by construction: this
    /// is the separation M5's dropdown-like widgets require.
    active_item: BTreeMap<FocusId, FocusId>,
    /// Entered modal scopes, innermost last.
    modal_stack: Vec<ModalEntry>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TabDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arrow {
    Prev,
    Next,
}

/// What an arrow key did. `NotHandled` is load-bearing: it is what lets an
/// application-level handler (the gallery's Left/Right stepping between
/// photos) coexist with core-level arrow semantics inside a group, without
/// the core knowing anything about the application.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArrowOutcome {
    /// Focus moved within a group.
    MovedFocus(FocusId),
    /// The active item of `list` moved to `item`; focus did not move.
    MovedActiveItem { list: FocusId, item: FocusId },
    /// The core has no arrow semantics here; the caller decides.
    NotHandled,
}

impl FocusTree {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a node and return its id. The first pushed node is the root and
    /// must have `parent == None`.
    pub fn push(&mut self, parent: Option<FocusId>, role: FocusRole, enabled: bool) -> FocusId {
        let id = self.nodes.len();
        assert_eq!(
            parent.is_none(),
            id == 0,
            "node 0 is the root and every later node has a parent"
        );
        self.nodes.push(FocusNode {
            parent,
            children: Vec::new(),
            role,
            enabled,
        });
        if let Some(p) = parent {
            self.nodes[p].children.push(id);
        }
        id
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn role(&self, id: FocusId) -> FocusRole {
        self.nodes[id].role
    }

    pub fn parent(&self, id: FocusId) -> Option<FocusId> {
        self.nodes[id].parent
    }

    // ── Traversal domain ─────────────────────────────────────────────────

    /// The subtree traversal is confined to: the innermost entered modal
    /// scope, or the tree root.
    ///
    /// **This one function is the whole of modal containment.** The walk below
    /// has no modal-specific branch; confinement falls out of enumerating from
    /// a different root.
    pub fn traversal_root(&self, state: &FocusState) -> FocusId {
        state.modal_stack.last().map(|e| e.scope).unwrap_or(0)
    }

    /// The Tab stops inside `root`, in tree order.
    ///
    /// A `Group` contributes itself, once, and is not descended into — that is
    /// "Tab treats the group as one stop". A not-entered `ModalScope`
    /// contributes nothing, so a modal subtree that exists but has not been
    /// entered cannot be tabbed into from outside.
    pub fn tab_stops(&self, state: &FocusState, root: FocusId) -> Vec<FocusId> {
        let mut out = Vec::new();
        self.collect_stops(state, root, true, &mut out);
        out
    }

    fn collect_stops(
        &self,
        state: &FocusState,
        id: FocusId,
        is_root: bool,
        out: &mut Vec<FocusId>,
    ) {
        let node = &self.nodes[id];
        if !is_root {
            match node.role {
                FocusRole::Stop if node.enabled => {
                    out.push(id);
                    return;
                }
                FocusRole::Stop => return,
                FocusRole::Group if node.enabled => {
                    out.push(id);
                    return;
                }
                FocusRole::Group => return,
                FocusRole::ModalScope if !state.is_entered(id) => return,
                // An ActiveItemList's items are never Tab stops: reaching them
                // is what the active-item pointer is for.
                FocusRole::ActiveItemList => return,
                FocusRole::ActiveItem => return,
                _ => {}
            }
        }
        for &child in &node.children {
            self.collect_stops(state, child, false, out);
        }
    }

    /// The members of a group, in tree order. Members are the enabled `Stop`
    /// descendants; nested groups are not flattened.
    pub fn group_members(&self, group: FocusId) -> Vec<FocusId> {
        let mut out = Vec::new();
        for &child in &self.nodes[group].children {
            self.collect_group_members(child, &mut out);
        }
        out
    }

    fn collect_group_members(&self, id: FocusId, out: &mut Vec<FocusId>) {
        let node = &self.nodes[id];
        match node.role {
            FocusRole::Stop if node.enabled => out.push(id),
            FocusRole::Stop | FocusRole::Group => {}
            _ => {
                for &child in &node.children {
                    self.collect_group_members(child, out);
                }
            }
        }
    }

    /// The nearest ancestor (or self) with the given role.
    fn nearest(&self, id: FocusId, role: FocusRole) -> Option<FocusId> {
        let mut cur = Some(id);
        while let Some(c) = cur {
            if self.nodes[c].role == role {
                return Some(c);
            }
            cur = self.nodes[c].parent;
        }
        None
    }

    fn is_descendant_of(&self, id: FocusId, ancestor: FocusId) -> bool {
        let mut cur = Some(id);
        while let Some(c) = cur {
            if c == ancestor {
                return true;
            }
            cur = self.nodes[c].parent;
        }
        false
    }

    /// The list whose active item the focused node's arrows would move, if
    /// any: an `ActiveItemList` that is a descendant of the focused stop.
    fn owned_active_list(&self, focused: FocusId) -> Option<FocusId> {
        let mut found = None;
        self.find_active_list(focused, &mut found);
        found
    }

    fn find_active_list(&self, id: FocusId, found: &mut Option<FocusId>) {
        if found.is_some() {
            return;
        }
        for &child in &self.nodes[id].children {
            if self.nodes[child].role == FocusRole::ActiveItemList {
                *found = Some(child);
                return;
            }
            self.find_active_list(child, found);
        }
    }

    fn active_items(&self, list: FocusId) -> Vec<FocusId> {
        self.nodes[list]
            .children
            .iter()
            .copied()
            .filter(|&c| self.nodes[c].role == FocusRole::ActiveItem && self.nodes[c].enabled)
            .collect()
    }

    // ── Focus movement ───────────────────────────────────────────────────

    /// The initial focus inside the current traversal domain: the first Tab
    /// stop, resolved through group memory. `None` when nothing is focusable.
    pub fn initial_focus(&self, state: &FocusState) -> Option<FocusId> {
        let root = self.traversal_root(state);
        let stops = self.tab_stops(state, root);
        stops.first().map(|&s| self.resolve_stop(state, s))
    }

    /// Move by Tab / Shift+Tab. Returns the new focus, or `None` when the
    /// domain has no stop at all.
    ///
    /// Wrap-around is at the ends of the stop list: the last stop's forward
    /// neighbour is the first. Inside an entered modal scope this is exactly
    /// focus containment — the same wrap, over a smaller list.
    pub fn tab(&self, state: &FocusState, dir: TabDirection) -> Option<FocusId> {
        let root = self.traversal_root(state);
        let stops = self.tab_stops(state, root);
        if stops.is_empty() {
            return None;
        }
        let current = state
            .focused
            .and_then(|f| self.stop_index_of(&stops, f))
            .or_else(|| {
                // Focus is outside the domain (a scope was just entered):
                // Tab starts at the domain's first stop.
                None
            });
        let next_index = match (current, dir) {
            (None, TabDirection::Forward) => 0,
            (None, TabDirection::Backward) => stops.len() - 1,
            (Some(i), TabDirection::Forward) => (i + 1) % stops.len(),
            (Some(i), TabDirection::Backward) => (i + stops.len() - 1) % stops.len(),
        };
        Some(self.resolve_stop(state, stops[next_index]))
    }

    /// Which entry of `stops` the focused node belongs to: itself if it is a
    /// stop, or its enclosing group if it is a group member.
    fn stop_index_of(&self, stops: &[FocusId], focused: FocusId) -> Option<usize> {
        if let Some(i) = stops.iter().position(|&s| s == focused) {
            return Some(i);
        }
        let group = self.nearest(focused, FocusRole::Group)?;
        stops.iter().position(|&s| s == group)
    }

    /// Landing on a stop: a `Group` resolves to its remembered member (or its
    /// first), anything else is itself.
    fn resolve_stop(&self, state: &FocusState, stop: FocusId) -> FocusId {
        if self.nodes[stop].role != FocusRole::Group {
            return stop;
        }
        let members = self.group_members(stop);
        match state.group_memory.get(&stop) {
            Some(&remembered) if members.contains(&remembered) => remembered,
            _ => members.first().copied().unwrap_or(stop),
        }
    }

    /// Arrow-key semantics. Two rules, in this order:
    ///
    /// 1. If the focused stop owns an `ActiveItemList`, the arrow moves that
    ///    list's active item and **focus does not move**.
    /// 2. Otherwise, if focus is inside a `Group`, the arrow moves focus
    ///    within the group.
    ///
    /// Anything else is `NotHandled`, which is how an application keeps its
    /// own arrow meaning (the gallery's Left/Right between photos).
    pub fn arrow(&self, state: &FocusState, arrow: Arrow) -> ArrowOutcome {
        let Some(focused) = state.focused else {
            return ArrowOutcome::NotHandled;
        };
        if let Some(list) = self.owned_active_list(focused) {
            let items = self.active_items(list);
            if items.is_empty() {
                return ArrowOutcome::NotHandled;
            }
            let current = state
                .active_item
                .get(&list)
                .and_then(|a| items.iter().position(|&i| i == *a));
            let next = match (current, arrow) {
                (None, Arrow::Next) => 0,
                (None, Arrow::Prev) => items.len() - 1,
                (Some(i), Arrow::Next) => (i + 1) % items.len(),
                (Some(i), Arrow::Prev) => (i + items.len() - 1) % items.len(),
            };
            return ArrowOutcome::MovedActiveItem {
                list,
                item: items[next],
            };
        }
        if let Some(group) = self.nearest(focused, FocusRole::Group) {
            let members = self.group_members(group);
            if members.is_empty() {
                return ArrowOutcome::NotHandled;
            }
            let current = members.iter().position(|&m| m == focused);
            let next = match (current, arrow) {
                (None, Arrow::Next) => 0,
                (None, Arrow::Prev) => members.len() - 1,
                (Some(i), Arrow::Next) => (i + 1) % members.len(),
                (Some(i), Arrow::Prev) => (i + members.len() - 1) % members.len(),
            };
            return ArrowOutcome::MovedFocus(members[next]);
        }
        ArrowOutcome::NotHandled
    }

    // ── Modal scopes ─────────────────────────────────────────────────────

    /// The scope that would consume an Esc, if any.
    ///
    /// The core answers *which* scope; what closing means (removing the
    /// subtree, clearing a flag) belongs to the caller — the core never
    /// mutates the tree.
    pub fn esc_target(&self, state: &FocusState) -> Option<FocusId> {
        state.modal_stack.last().map(|e| e.scope)
    }

    /// Where focus must go when `subtree` is about to be removed.
    ///
    /// Computed **before** the mutation, because ids do not survive a rebuild:
    /// the conditional / `for` paths materialise fresh subtrees, so a stored
    /// id could not be matched against the new tree afterwards.
    pub fn focus_after_removing(&self, state: &FocusState, subtree: FocusId) -> Option<FocusId> {
        let focused = state.focused?;
        if !self.is_descendant_of(focused, subtree) {
            return Some(focused);
        }
        // If the removed subtree is (or contains) the entered modal scope,
        // restoration wins over structural succession.
        if let Some(entry) = state.modal_stack.last() {
            if self.is_descendant_of(entry.scope, subtree) {
                return entry.restore_to;
            }
        }
        let root = self.traversal_root(state);
        let stops = self.tab_stops(state, root);
        let survivor = stops
            .iter()
            .copied()
            .find(|&s| !self.is_descendant_of(s, subtree));
        survivor.map(|s| self.resolve_stop(state, s))
    }
}

impl FocusState {
    pub fn focused(&self) -> Option<FocusId> {
        self.focused
    }

    pub fn active_item_of(&self, list: FocusId) -> Option<FocusId> {
        self.active_item.get(&list).copied()
    }

    pub fn remembered_member_of(&self, group: FocusId) -> Option<FocusId> {
        self.group_memory.get(&group).copied()
    }

    pub fn modal_depth(&self) -> usize {
        self.modal_stack.len()
    }

    fn is_entered(&self, scope: FocusId) -> bool {
        self.modal_stack.iter().any(|e| e.scope == scope)
    }

    /// Set focus and update every state derived from it, atomically.
    ///
    /// The group memory is written **here and only here**, in the same
    /// primitive that writes `focused` — the parallel-data discipline of
    /// implementation-gates trap 3. A caller that assigns `focused` directly
    /// cannot exist, because the field is private.
    pub fn set_focus(&mut self, tree: &FocusTree, id: Option<FocusId>) {
        self.focused = id;
        if let Some(id) = id {
            if let Some(group) = tree.nearest(id, FocusRole::Group) {
                self.group_memory.insert(group, id);
            }
        }
    }

    pub fn set_active_item(&mut self, list: FocusId, item: FocusId) {
        self.active_item.insert(list, item);
    }

    /// Enter a modal scope, capturing the focus to restore on exit. Returns
    /// false — and changes nothing — when `scope` is not annotated as one.
    ///
    /// **The role check is load-bearing, and was added because a mutation
    /// showed its absence was undetectable** (spike finding S-3). Confinement
    /// comes from the stack, not from the role: with no check, pushing *any*
    /// container confines traversal to it, and the mechanism fixture's
    /// containment assertions passed against a projection that had dropped the
    /// annotation entirely. The role's own contribution is the separate
    /// property that an un-entered scope is not tabbable; the two are
    /// separable mechanisms, and only this check ties them to one concept.
    pub fn enter_modal(&mut self, tree: &FocusTree, scope: FocusId) -> bool {
        if tree.role(scope) != FocusRole::ModalScope {
            return false;
        }
        self.modal_stack.push(ModalEntry {
            scope,
            restore_to: self.focused,
        });
        let entering = tree.initial_focus(self);
        self.set_focus(tree, entering);
        true
    }

    /// Leave the innermost modal scope, restoring the focus captured at entry.
    pub fn exit_modal(&mut self, tree: &FocusTree) -> Option<FocusId> {
        let entry = self.modal_stack.pop()?;
        self.set_focus(tree, entry.restore_to);
        entry.restore_to
    }

    /// Apply an arrow outcome. Kept beside the query so the two derived
    /// stores are only ever written through a primitive that knows both.
    pub fn apply_arrow(&mut self, tree: &FocusTree, outcome: ArrowOutcome) {
        match outcome {
            ArrowOutcome::MovedFocus(id) => self.set_focus(tree, Some(id)),
            ArrowOutcome::MovedActiveItem { list, item } => self.set_active_item(list, item),
            ArrowOutcome::NotHandled => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The A-gallery-shaped fixture:
    ///
    /// ```text
    /// root
    ///  ├ toolbar (Container)
    ///  │   ├ tabs (Group)          <- one Tab stop, arrows move inside
    ///  │   │   ├ all (Stop)
    ///  │   │   ├ albums (Stop)
    ///  │   │   └ favorites (Stop)
    ///  │   └ view_toggle (Stop)
    ///  ├ grid (Container)
    ///  │   ├ thumb0 (Stop)
    ///  │   └ thumb1 (Stop)
    ///  └ lightbox (ModalScope)
    ///      ├ prev (Stop)
    ///      └ next (Stop)
    /// ```
    struct Fixture {
        tree: FocusTree,
        tabs: FocusId,
        all: FocusId,
        albums: FocusId,
        favorites: FocusId,
        view_toggle: FocusId,
        thumb0: FocusId,
        thumb1: FocusId,
        lightbox: FocusId,
        prev: FocusId,
        next: FocusId,
    }

    fn gallery() -> Fixture {
        let mut tree = FocusTree::new();
        let root = tree.push(None, FocusRole::Container, true);
        let toolbar = tree.push(Some(root), FocusRole::Container, true);
        let tabs = tree.push(Some(toolbar), FocusRole::Group, true);
        let all = tree.push(Some(tabs), FocusRole::Stop, true);
        let albums = tree.push(Some(tabs), FocusRole::Stop, true);
        let favorites = tree.push(Some(tabs), FocusRole::Stop, true);
        let view_toggle = tree.push(Some(toolbar), FocusRole::Stop, true);
        let grid = tree.push(Some(root), FocusRole::Container, true);
        let thumb0 = tree.push(Some(grid), FocusRole::Stop, true);
        let thumb1 = tree.push(Some(grid), FocusRole::Stop, true);
        let lightbox = tree.push(Some(root), FocusRole::ModalScope, true);
        let prev = tree.push(Some(lightbox), FocusRole::Stop, true);
        let next = tree.push(Some(lightbox), FocusRole::Stop, true);
        Fixture {
            tree,
            tabs,
            all,
            albums,
            favorites,
            view_toggle,
            thumb0,
            thumb1,
            lightbox,
            prev,
            next,
        }
    }

    // ── Q1 / Q2: group traversal ─────────────────────────────────────────

    #[test]
    fn tab_treats_a_group_as_one_stop() {
        let f = gallery();
        let state = FocusState::default();
        // The group contributes itself once; its three members do not appear.
        assert_eq!(
            f.tree.tab_stops(&state, 0),
            vec![f.tabs, f.view_toggle, f.thumb0, f.thumb1],
            "a Group is one Tab stop and its members are not stops"
        );
    }

    #[test]
    fn tab_from_inside_a_group_leaves_the_whole_group() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.albums));
        assert_eq!(
            f.tree.tab(&state, TabDirection::Forward),
            Some(f.view_toggle),
            "Tab from the middle member exits the group rather than stepping to its sibling"
        );
    }

    #[test]
    fn a_group_remembers_the_member_that_was_focused() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.favorites));
        // Tab out to the end of the list and wrap back round to the group.
        state.set_focus(&f.tree, f.tree.tab(&state, TabDirection::Forward));
        assert_eq!(state.focused(), Some(f.view_toggle));
        state.set_focus(&f.tree, f.tree.tab(&state, TabDirection::Backward));
        assert_eq!(
            state.focused(),
            Some(f.favorites),
            "re-entering the group lands on the remembered member, not the first"
        );
        assert_eq!(state.remembered_member_of(f.tabs), Some(f.favorites));
    }

    #[test]
    fn a_group_never_entered_lands_on_its_first_member() {
        let f = gallery();
        let state = FocusState::default();
        assert_eq!(
            f.tree.initial_focus(&state),
            Some(f.all),
            "with no memory the group resolves to its first member"
        );
    }

    #[test]
    fn arrows_move_within_the_group_and_wrap() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.all));
        for expected in [f.albums, f.favorites, f.all] {
            let outcome = f.tree.arrow(&state, Arrow::Next);
            assert_eq!(outcome, ArrowOutcome::MovedFocus(expected));
            state.apply_arrow(&f.tree, outcome);
        }
        let outcome = f.tree.arrow(&state, Arrow::Prev);
        assert_eq!(
            outcome,
            ArrowOutcome::MovedFocus(f.favorites),
            "Prev from the first member wraps to the last"
        );
    }

    #[test]
    fn arrows_outside_a_group_are_not_handled() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.thumb0));
        assert_eq!(
            f.tree.arrow(&state, Arrow::Next),
            ArrowOutcome::NotHandled,
            "the core yields arrows it has no meaning for, so an app handler can take them"
        );
    }

    // ── Q3: focus / active-item separation ───────────────────────────────

    /// A dropdown-shaped fixture: focus sits on `combo`, and the open list's
    /// items are activated without focus moving.
    fn dropdown() -> (FocusTree, FocusId, FocusId, Vec<FocusId>) {
        let mut tree = FocusTree::new();
        let root = tree.push(None, FocusRole::Container, true);
        let combo = tree.push(Some(root), FocusRole::Stop, true);
        let list = tree.push(Some(combo), FocusRole::ActiveItemList, true);
        let items = (0..3)
            .map(|_| tree.push(Some(list), FocusRole::ActiveItem, true))
            .collect();
        let after = tree.push(Some(root), FocusRole::Stop, true);
        (tree, combo, list, {
            let _ = after;
            items
        })
    }

    #[test]
    fn arrows_move_the_active_item_without_moving_focus() {
        let (tree, combo, list, items) = dropdown();
        let mut state = FocusState::default();
        state.set_focus(&tree, Some(combo));

        let outcome = tree.arrow(&state, Arrow::Next);
        assert_eq!(
            outcome,
            ArrowOutcome::MovedActiveItem {
                list,
                item: items[0]
            }
        );
        state.apply_arrow(&tree, outcome);
        assert_eq!(
            state.focused(),
            Some(combo),
            "focus stayed on the owner while the active item moved"
        );
        assert_eq!(state.active_item_of(list), Some(items[0]));

        let outcome = tree.arrow(&state, Arrow::Next);
        state.apply_arrow(&tree, outcome);
        assert_eq!(state.active_item_of(list), Some(items[1]));
        assert_eq!(state.focused(), Some(combo));
    }

    #[test]
    fn active_items_are_not_tab_stops() {
        let (tree, combo, _list, _items) = dropdown();
        let state = FocusState::default();
        let stops = tree.tab_stops(&state, 0);
        assert_eq!(stops.len(), 2, "combo and the stop after it, and no items");
        assert_eq!(stops[0], combo);
    }

    // ── Q4: modal scope ──────────────────────────────────────────────────

    #[test]
    fn an_unentered_modal_scope_is_not_reachable_by_tab() {
        let f = gallery();
        let state = FocusState::default();
        let stops = f.tree.tab_stops(&state, 0);
        assert!(
            !stops.contains(&f.prev) && !stops.contains(&f.next),
            "a modal subtree that exists but was not entered contributes no stops"
        );
    }

    #[test]
    fn entering_a_modal_scope_confines_tab_to_it() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.thumb1));
        state.enter_modal(&f.tree, f.lightbox);

        assert_eq!(
            state.focused(),
            Some(f.prev),
            "entry lands on the first stop"
        );
        assert_eq!(
            f.tree.tab_stops(&state, f.tree.traversal_root(&state)),
            vec![f.prev, f.next]
        );

        // Tab all the way round: it must never leave the scope.
        for expected in [f.next, f.prev, f.next] {
            state.set_focus(&f.tree, f.tree.tab(&state, TabDirection::Forward));
            assert_eq!(state.focused(), Some(expected));
        }
    }

    #[test]
    fn leaving_a_modal_scope_restores_the_focus_captured_at_entry() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.thumb1));
        state.enter_modal(&f.tree, f.lightbox);
        state.set_focus(&f.tree, Some(f.next));

        assert_eq!(state.exit_modal(&f.tree), Some(f.thumb1));
        assert_eq!(state.focused(), Some(f.thumb1));
        assert_eq!(state.modal_depth(), 0);
    }

    #[test]
    fn esc_targets_the_innermost_entered_scope_only() {
        let f = gallery();
        let mut state = FocusState::default();
        assert_eq!(
            f.tree.esc_target(&state),
            None,
            "with nothing entered the core claims no Esc"
        );
        state.enter_modal(&f.tree, f.lightbox);
        assert_eq!(f.tree.esc_target(&state), Some(f.lightbox));
    }

    // ── Q5: the focused node disappears ──────────────────────────────────

    #[test]
    fn removing_the_focused_subtree_falls_to_the_next_surviving_stop() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.thumb0));
        // The grid (thumb0's parent) is node 7 in the fixture's build order.
        let grid = f.tree.nodes[f.thumb0].parent.expect("thumb0 has a parent");
        assert_eq!(
            f.tree.focus_after_removing(&state, grid),
            Some(f.all),
            "the first surviving stop, resolved through the group"
        );
    }

    #[test]
    fn removing_an_unrelated_subtree_leaves_focus_alone() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.thumb0));
        assert_eq!(
            f.tree.focus_after_removing(&state, f.lightbox),
            Some(f.thumb0)
        );
    }

    #[test]
    fn removing_the_entered_scope_restores_rather_than_succeeds() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.thumb1));
        state.enter_modal(&f.tree, f.lightbox);
        assert_eq!(
            f.tree.focus_after_removing(&state, f.lightbox),
            Some(f.thumb1),
            "closing the lightbox by removing its subtree must restore, not fall through"
        );
    }

    // ── Branch tests: the authored edge arms ─────────────────────────────

    #[test]
    fn a_domain_with_no_stops_yields_no_focus() {
        let mut tree = FocusTree::new();
        let root = tree.push(None, FocusRole::Container, true);
        tree.push(Some(root), FocusRole::Container, true);
        let state = FocusState::default();
        assert_eq!(tree.tab(&state, TabDirection::Forward), None);
        assert_eq!(tree.initial_focus(&state), None);
    }

    #[test]
    fn a_disabled_stop_is_skipped() {
        let mut tree = FocusTree::new();
        let root = tree.push(None, FocusRole::Container, true);
        let a = tree.push(Some(root), FocusRole::Stop, true);
        tree.push(Some(root), FocusRole::Stop, false);
        let c = tree.push(Some(root), FocusRole::Stop, true);
        let state = FocusState::default();
        assert_eq!(tree.tab_stops(&state, 0), vec![a, c]);
    }

    #[test]
    fn an_empty_group_is_a_stop_that_resolves_to_itself() {
        let mut tree = FocusTree::new();
        let root = tree.push(None, FocusRole::Container, true);
        let g = tree.push(Some(root), FocusRole::Group, true);
        let state = FocusState::default();
        assert_eq!(
            tree.initial_focus(&state),
            Some(g),
            "a group with no members must not vanish from traversal"
        );
    }

    #[test]
    fn a_node_that_is_not_a_scope_cannot_be_entered_as_one() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.thumb0));
        assert!(
            !state.enter_modal(&f.tree, f.thumb1),
            "only a node annotated as a scope may confine traversal"
        );
        assert_eq!(state.modal_depth(), 0, "the rejected entry changed nothing");
        assert_eq!(state.focused(), Some(f.thumb0));
    }

    #[test]
    fn arrows_with_nothing_focused_are_not_handled() {
        let f = gallery();
        let state = FocusState::default();
        assert_eq!(f.tree.arrow(&state, Arrow::Next), ArrowOutcome::NotHandled);
    }

    #[test]
    fn a_group_memory_pointing_at_a_gone_member_falls_back_to_the_first() {
        // The hazard this pins is id reuse across a rebuild, which is real:
        // the conditional / `for` paths materialise fresh subtrees, so an id
        // remembered against the old tree can name a *different* node in the
        // new one. The rebuilt tree below keeps the group at the same id (2)
        // with a single member (3), and id 5 — the remembered one — now names
        // an unrelated top-level stop rather than a group member.
        //
        // Written this way deliberately: an earlier version rebuilt a tree in
        // which the group landed at a *different* id, so the memory lookup
        // missed entirely and the test passed without ever reaching the
        // membership check it claims to exercise. The mutation battery caught
        // that (spike finding S-2).
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.favorites));
        assert_eq!(
            state.remembered_member_of(f.tabs),
            Some(f.favorites),
            "precondition: the memory names the id this test will make stale"
        );

        let mut rebuilt = FocusTree::new();
        let root = rebuilt.push(None, FocusRole::Container, true); // 0
        let toolbar = rebuilt.push(Some(root), FocusRole::Container, true); // 1
        let tabs = rebuilt.push(Some(toolbar), FocusRole::Group, true); // 2
        let only_member = rebuilt.push(Some(tabs), FocusRole::Stop, true); // 3
        rebuilt.push(Some(toolbar), FocusRole::Stop, true); // 4
        let unrelated = rebuilt.push(Some(root), FocusRole::Stop, true); // 5
        assert_eq!(tabs, f.tabs, "the group kept its id across the rebuild");
        assert_eq!(
            unrelated, f.favorites,
            "the remembered id now names a node outside the group"
        );

        assert_eq!(
            rebuilt.initial_focus(&state),
            Some(only_member),
            "a remembered id that is no longer a member of this group must not \
             be returned as focus"
        );
    }

    // ── Boundary condition: the wrap (DD-V-029 red-test target) ──────────

    #[test]
    fn tab_wraps_at_both_ends_of_the_stop_list() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.thumb1)); // the last stop
        assert_eq!(
            f.tree.tab(&state, TabDirection::Forward),
            Some(f.all),
            "forward from the last stop wraps to the first, resolved through the group"
        );
        state.set_focus(&f.tree, Some(f.all)); // inside the first stop
        assert_eq!(
            f.tree.tab(&state, TabDirection::Backward),
            Some(f.thumb1),
            "backward from the first stop wraps to the last"
        );
    }
}
