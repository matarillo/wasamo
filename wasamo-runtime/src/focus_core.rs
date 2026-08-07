//! Focus traversal core — the Win32-independent half of the M4-Phase 2 focus
//! model (DD-M4-P2-003 "the spike's traversal core is adopted as the
//! implementation").
//!
//! **Consumed through a projection, never directly.** `crate::focus`
//! projects a live `WidgetNode` tree onto the [`FocusTree`] this module
//! defines and drives it from `WindowState`'s keyboard and pointer arms
//! (M4-Phase 2 T5) — that projection, not this module, is what a caller
//! outside this file reaches for. This module's own logic is unchanged by
//! who calls it: it stays exactly what the Phase 2 ADR compared options
//! against.
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

// Measured at M4-Phase 2 T7's close gate by removing this attribute and
// reading what a plain `cargo build -p wasamo-runtime` actually reports,
// rather than by guessing: exactly seven methods across the two `impl`
// blocks below have no production caller and would otherwise warn —
// `FocusTree::is_empty` / `parent` / `focus_after_removing`, and
// `FocusState::active_item_of` / `remembered_member_of` / `modal_depth` /
// `exit_modal`. Every one of them is exercised only by this module's own
// `#[cfg(test)]` unit tests, which a plain build does not compile, so a
// build with no `#[cfg(test)]` reader still sees them as dead.
//
// `FocusRole::Group` and `FocusRole::ModalScope` are not on this list —
// T6's authored `focus-group` / `modal-scope` annotations gave
// `WidgetNode::focus_role` a production constructor for both, and T7's
// `focus::sync_scopes_to_tree` / `arrow_on_key` / `dismiss_on_key` are
// production consumers, so the enum variants and the tree/state methods
// they drive (`tab`, `arrow`, `enter_modal`, `focus_landing`, …) are
// reachable and warn-free without this attribute.
// `FocusRole::ActiveItemList` / `ActiveItem` have no authored `.ui` source
// in M4 at all (`plan.md` §T7 records the narrowed coverage as
// deliberate), so `active_item_of` — the read half of the state they
// drive — stays on this list alongside them; `remembered_member_of` and
// `modal_depth` are read-only diagnostics over state a production caller
// writes but never itself reads back; `exit_modal` is superseded in
// production by `sync_scopes_to_tree`'s removal-driven exit
// (`WindowFocus::rebase` plus the outermost-dropped-scope restoration in
// `focus.rs`), which never calls it directly.
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

    // ── Click landing (M4-Phase 2 T7, CF-T6-5) ────────────────────────────

    /// The first legal click landing in `chain` (target-first, root-last —
    /// `hit::dispatch_chain`'s order), or `None` when nothing in it is one.
    ///
    /// **Relationship to [`Self::collect_stops`], stated explicitly
    /// because the two differ in exactly one place and that difference is
    /// the rule, not drift.** `collect_stops` decides what **Tab**
    /// enumerates: a `Group` contributes itself, once, and is not
    /// descended into (`docs/dsl_spec.md` §4.19 "Tab treats the group as
    /// one stop"). This decides what a **click** lands on, and a click
    /// already names a point, not a position in a linear order — so a
    /// `Group`'s members are reachable directly here, and only fall back
    /// to the group itself (via [`Self::resolve_stop`]) when the click did
    /// not land on one of them (its own padding, or a disabled member
    /// below it in the chain).
    ///
    /// Checked in this order, walking `chain` front to back:
    ///
    /// 1. Skip any id that is not a descendant of (or equal to)
    ///    [`Self::traversal_root`] — a click must never move focus outside
    ///    an entered modal scope, the same containment `tab_stops` already
    ///    gives Tab.
    /// 2. An enabled [`FocusRole::Stop`] is the landing, directly. This is
    ///    the whole fix for CF-T6-5: a group's member is a `Stop`, and is
    ///    reached here before its enclosing `Group` is ever considered —
    ///    unlike the T5-era `focus::nearest_focusable` this replaces,
    ///    which was defined against `tab_stops` and so could never see a
    ///    member at all.
    /// 3. An enabled [`FocusRole::Group`] resolves through
    ///    [`Self::resolve_stop`] — the group's remembered member, the same
    ///    landing Tab gives it. Reached only when the click did not land
    ///    on a member.
    /// 4. Everything else (`Container`, `ModalScope`, `ActiveItemList`,
    ///    `ActiveItem`, a disabled `Stop` or `Group`) is not a landing;
    ///    the walk continues to the next, further-out entry of `chain`.
    /// 5. Nothing in `chain` is legal → `None`, which is "clicking
    ///    background never clears focus" (`docs/dsl_spec.md` §4.19): the
    ///    caller leaves the previous focus exactly where it was rather
    ///    than writing `None` through it.
    pub fn focus_landing(&self, state: &FocusState, chain: &[FocusId]) -> Option<FocusId> {
        let root = self.traversal_root(state);
        for &id in chain {
            if !self.is_descendant_of(id, root) {
                continue;
            }
            match self.nodes[id].role {
                FocusRole::Stop if self.nodes[id].enabled => return Some(id),
                FocusRole::Group if self.nodes[id].enabled => {
                    return Some(self.resolve_stop(state, id));
                }
                _ => {}
            }
        }
        None
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

    /// Read-only view of the entered modal scopes, innermost last. Exists
    /// for the caller that closes a scope by removal (M4-Phase 2's later
    /// work on this task) and doubles as what makes [`Self::remap`]'s
    /// stack-order behaviour unit-testable from outside this module.
    pub fn modal_entries(&self) -> &[ModalEntry] {
        &self.modal_stack
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

    /// Re-express every retained id through `f`, the mapping from the
    /// coordinate system this state's ids were last written against to a
    /// new one. **This is the primitive that makes "a `FocusId` is a
    /// coordinate, not an identity" true** (M4-Phase 2 T7): every store
    /// keyed or valued by a `FocusId` is rewritten here, in one place,
    /// rather than trusted to still resolve to the same node once the
    /// projection that produced it has been rebuilt.
    ///
    /// - `focused` is remapped; an unmappable id becomes `None`.
    /// - `group_memory` remaps both the key and the value; the entry is
    ///   **dropped** when either is unmappable — a group whose id moved
    ///   and a group whose id vanished collapse to the same case here, and
    ///   the same is true of the remembered member.
    /// - `active_item` follows the identical key/value rule, for the same
    ///   reason.
    /// - `modal_stack` remaps each entry's `scope`; the entry is
    ///   **dropped** when the scope is unmappable, because an unmappable
    ///   scope means its subtree is gone — there is no scope left to have
    ///   been entered. `restore_to` is remapped independently and becomes
    ///   `None` when unmappable, because "restore to nothing" is a legal
    ///   state (the previously focused node is itself gone, not the
    ///   scope). Stack order is preserved.
    ///
    /// **What this function does not do.** Dropping a modal entry here is
    /// mechanical coordinate bookkeeping, not a policy decision: the
    /// *restoration* a vanished scope owes is not this function's job.
    /// `focus::sync_scopes_to_tree`'s exit step is the caller that owes it,
    /// and it writes the dropped entry's remapped `restore_to` to focus
    /// exactly as captured — when that is `None`, focus becomes `None`, not
    /// the domain's first stop (`docs/dsl_spec.md` §4.19: entry "remembers
    /// the focused widget", which may be nothing; DD-M4-P2-004: entry
    /// "captures the restore target: the widget focused at that moment,
    /// possibly none"). The domain's first stop is a different branch's
    /// answer — `sync_scopes_to_tree`'s separate structural-succession
    /// step, reached only when no scope exit explains the lost focus. This
    /// function only keeps every retained id checkable against whatever
    /// tree comes next.
    pub fn remap(&mut self, f: impl Fn(FocusId) -> Option<FocusId>) {
        self.focused = self.focused.and_then(&f);

        self.group_memory = self
            .group_memory
            .iter()
            .filter_map(|(&k, &v)| Some((f(k)?, f(v)?)))
            .collect();

        self.active_item = self
            .active_item
            .iter()
            .filter_map(|(&k, &v)| Some((f(k)?, f(v)?)))
            .collect();

        self.modal_stack = self
            .modal_stack
            .iter()
            .filter_map(|entry| {
                let scope = f(entry.scope)?;
                let restore_to = entry.restore_to.and_then(&f);
                Some(ModalEntry { scope, restore_to })
            })
            .collect();
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

    // ── Nesting (M4-Phase 2 T7) ────────────────────────────────────────────
    //
    // `plan.md` §T7 asks for this explicitly: "scope nesting is supported,
    // unexercised by any M4 app, so its ordering and innermost-addressing
    // keep — or gain — pure-logic pins for Phase 9 to inherit." Every Q4
    // test above exercises exactly one entered scope; these pin what
    // happens with two, nested.

    /// A tree with a modal scope nested inside another: `before` is a stop
    /// outside both, `outer` contains its own stop plus the nested `inner`
    /// scope, and `inner` contains its own stop. The `before` stop is what
    /// lets [`exiting_both_nested_scopes_restores_each_entrys_own_capture`]
    /// give the outer entry a restore target that is not itself inside
    /// either scope.
    fn nested_scopes() -> (FocusTree, FocusId, FocusId, FocusId, FocusId, FocusId) {
        let mut tree = FocusTree::new();
        let root = tree.push(None, FocusRole::Container, true);
        let before = tree.push(Some(root), FocusRole::Stop, true);
        let outer = tree.push(Some(root), FocusRole::ModalScope, true);
        let outer_stop = tree.push(Some(outer), FocusRole::Stop, true);
        let inner = tree.push(Some(outer), FocusRole::ModalScope, true);
        let inner_stop = tree.push(Some(inner), FocusRole::Stop, true);
        (tree, before, outer, outer_stop, inner, inner_stop)
    }

    #[test]
    fn entering_nested_scopes_stacks_outer_then_inner() {
        let (tree, before, outer, _outer_stop, inner, _inner_stop) = nested_scopes();
        let mut state = FocusState::default();
        state.set_focus(&tree, Some(before));
        assert!(
            state.enter_modal(&tree, outer),
            "outer must be a real scope"
        );
        assert!(
            state.enter_modal(&tree, inner),
            "inner must be a real scope"
        );
        assert_eq!(
            state
                .modal_entries()
                .iter()
                .map(|e| e.scope)
                .collect::<Vec<_>>(),
            vec![outer, inner],
            "the stack is materialisation order — outer entered first, inner second — \
             and stays in that order; traversal_root / esc_target read the *last* entry, \
             so the order here is what makes 'innermost wins' well-defined"
        );
    }

    #[test]
    fn traversal_root_and_esc_target_name_the_innermost_entered_scope() {
        let (tree, before, outer, _outer_stop, inner, _inner_stop) = nested_scopes();
        let mut state = FocusState::default();
        state.set_focus(&tree, Some(before));
        state.enter_modal(&tree, outer);
        state.enter_modal(&tree, inner);
        assert_eq!(
            tree.traversal_root(&state),
            inner,
            "traversal confines to the innermost entered scope, not the outer one"
        );
        assert_eq!(
            tree.esc_target(&state),
            Some(inner),
            "Escape addresses the innermost scope only (docs/dsl_spec.md §4.19 \
             \"dismiss\": \"addressed to the innermost scope and stops there\")"
        );
    }

    #[test]
    fn exiting_both_nested_scopes_restores_each_entrys_own_capture() {
        let (tree, before, outer, outer_stop, inner, inner_stop) = nested_scopes();
        let mut state = FocusState::default();
        state.set_focus(&tree, Some(before));
        state.enter_modal(&tree, outer); // captures `before`; lands on outer_stop
        assert_eq!(state.focused(), Some(outer_stop), "precondition");
        state.enter_modal(&tree, inner); // captures outer_stop; lands on inner_stop
        assert_eq!(state.focused(), Some(inner_stop), "precondition");

        // `exit_modal` unwinds one level and restores *that level's own*
        // capture — not the outer scope's, and not the domain's first stop.
        assert_eq!(
            state.exit_modal(&tree),
            Some(outer_stop),
            "unwinding the inner scope restores what was focused when it was entered"
        );
        assert_eq!(state.focused(), Some(outer_stop));
        assert_eq!(state.modal_depth(), 1, "only one level unwound");
        assert_eq!(
            tree.traversal_root(&state),
            outer,
            "back inside the outer scope, not confined to the (now-exited) inner one"
        );

        // Exiting the remaining (outer) scope restores what was focused
        // *before either scope opened* — the outer entry's own capture.
        assert_eq!(state.exit_modal(&tree), Some(before));
        assert_eq!(state.focused(), Some(before));
        assert_eq!(state.modal_depth(), 0);
    }

    #[test]
    fn entering_a_scope_with_no_focus_stop_leaves_focus_unset() {
        let mut tree = FocusTree::new();
        let root = tree.push(None, FocusRole::Container, true);
        let scope = tree.push(Some(root), FocusRole::ModalScope, true);
        // No children under `scope`: nothing inside it can be focused.
        let mut state = FocusState::default();
        assert!(state.enter_modal(&tree, scope));
        assert_eq!(
            state.focused(),
            None,
            "docs/dsl_spec.md §4.19: \"A scope with no focusable widget leaves focus \
             unset\""
        );
        assert_eq!(
            tree.esc_target(&state),
            Some(scope),
            "the scope is still entered, and still claims Escape, even though nothing \
             inside it can be focused"
        );
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

    // ── `remap`: the id coordinate-system primitive (M4-Phase 2 T7) ──────

    #[test]
    fn remap_with_the_identity_mapping_changes_nothing() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.favorites)); // writes group_memory[tabs] = favorites
        state.enter_modal(&f.tree, f.lightbox); // pushes the stack, restore_to = favorites
        state.remap(Some);
        assert_eq!(
            state.focused(),
            Some(f.prev),
            "entering the scope moved focus; an identity remap must not move it again"
        );
        assert_eq!(state.remembered_member_of(f.tabs), Some(f.favorites));
        assert_eq!(
            state.modal_entries(),
            &[ModalEntry {
                scope: f.lightbox,
                restore_to: Some(f.favorites),
            }]
        );
    }

    #[test]
    fn remap_with_a_shifting_mapping_moves_focused() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.thumb0));
        state.remap(|id| Some(id + 100));
        assert_eq!(state.focused(), Some(f.thumb0 + 100));
    }

    #[test]
    fn remap_drops_focused_when_it_is_unmappable() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.thumb0));
        state.remap(|_| None);
        assert_eq!(state.focused(), None);
    }

    #[test]
    fn remap_drops_a_group_memory_entry_whose_key_is_unmappable() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.favorites));
        assert_eq!(
            state.remembered_member_of(f.tabs),
            Some(f.favorites),
            "precondition: the memory names the id this test will make unmappable"
        );
        state.remap(move |id| if id == f.tabs { None } else { Some(id) });
        assert_eq!(
            f.tree.initial_focus(&state),
            Some(f.all),
            "a group-memory entry whose key could not be remapped must be dropped, so \
             re-entering the group lands on its first member rather than the stale memory"
        );
    }

    #[test]
    fn remap_drops_a_group_memory_entry_whose_value_is_unmappable() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.favorites));
        state.remap(move |id| if id == f.favorites { None } else { Some(id) });
        assert_eq!(
            f.tree.initial_focus(&state),
            Some(f.all),
            "a group-memory entry whose remembered member could not be remapped must be \
             dropped too, or a vanished member would still be looked up by a live group id"
        );
    }

    #[test]
    fn remap_moves_an_active_item_entry() {
        let (tree, combo, list, items) = dropdown();
        let mut state = FocusState::default();
        state.set_focus(&tree, Some(combo));
        state.apply_arrow(
            &tree,
            ArrowOutcome::MovedActiveItem {
                list,
                item: items[0],
            },
        );
        assert_eq!(state.active_item_of(list), Some(items[0]), "precondition");
        state.remap(|id| Some(id + 100));
        assert_eq!(state.active_item_of(list + 100), Some(items[0] + 100));
    }

    #[test]
    fn remap_drops_a_modal_entry_whose_scope_is_unmappable() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.thumb1));
        state.enter_modal(&f.tree, f.lightbox);
        assert_eq!(state.modal_depth(), 1, "precondition");
        state.remap(move |id| if id == f.lightbox { None } else { Some(id) });
        assert_eq!(
            state.modal_entries(),
            &[],
            "a modal entry whose scope could not be remapped names a subtree that is \
             gone, and must be dropped rather than kept pointing at nothing"
        );
    }

    #[test]
    fn remap_keeps_a_modal_entry_whose_restore_to_is_unmappable_with_restore_to_cleared() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.thumb1));
        state.enter_modal(&f.tree, f.lightbox); // restore_to = Some(thumb1)
        state.remap(move |id| if id == f.thumb1 { None } else { Some(id) });
        assert_eq!(
            state.modal_entries(),
            &[ModalEntry {
                scope: f.lightbox,
                restore_to: None,
            }],
            "restoring to nothing is a legal state: the entry survives with restore_to \
             cleared rather than being dropped alongside a scope whose own id is fine"
        );
    }

    #[test]
    fn remap_preserves_modal_stack_order() {
        // A second scope nested inside the gallery fixture would need a
        // bespoke tree anyway, so this builds the nesting directly.
        let mut tree = FocusTree::new();
        let root = tree.push(None, FocusRole::Container, true);
        let outer = tree.push(Some(root), FocusRole::ModalScope, true);
        let outer_stop = tree.push(Some(outer), FocusRole::Stop, true);
        let inner = tree.push(Some(outer), FocusRole::ModalScope, true);
        tree.push(Some(inner), FocusRole::Stop, true);
        let mut state = FocusState::default();
        state.enter_modal(&tree, outer);
        state.enter_modal(&tree, inner);
        assert_eq!(state.modal_depth(), 2, "precondition");
        state.remap(Some);
        assert_eq!(
            state.modal_entries(),
            &[
                ModalEntry {
                    scope: outer,
                    restore_to: None,
                },
                ModalEntry {
                    scope: inner,
                    restore_to: Some(outer_stop),
                },
            ],
            "identity remap must not disturb stack order: outermost first, innermost last"
        );
    }

    // ── `focus_landing` (M4-Phase 2 T7, CF-T6-5) ──────────────────────────

    #[test]
    fn focus_landing_returns_the_target_itself_when_it_is_a_stop() {
        let f = gallery();
        let state = FocusState::default();
        // target-first, root-last, as `hit::dispatch_chain` produces it.
        let chain = [f.thumb0, f.tree.parent(f.thumb0).unwrap(), 0];
        assert_eq!(f.tree.focus_landing(&state, &chain), Some(f.thumb0));
    }

    #[test]
    fn focus_landing_climbs_to_an_ancestor_stop_when_the_target_is_a_non_focusable_container() {
        let mut tree = FocusTree::new();
        let root = tree.push(None, FocusRole::Container, true);
        let stop = tree.push(Some(root), FocusRole::Stop, true);
        // A container nested under a Stop, standing in for whatever a
        // resolved hit target can be below the widget that actually
        // claims focus.
        let inner = tree.push(Some(stop), FocusRole::Container, true);
        let state = FocusState::default();
        let chain = [inner, stop, root];
        assert_eq!(
            tree.focus_landing(&state, &chain),
            Some(stop),
            "a click that resolved below a Stop must still land on the Stop, walking \
             outward from the target"
        );
    }

    #[test]
    fn focus_landing_on_a_group_member_focuses_that_member_not_the_remembered_one() {
        let f = gallery();
        let mut state = FocusState::default();
        // Establish a remembered member deliberately different from the
        // one this click targets, so a `resolve_stop`-based landing (the
        // approach the T7 start gate rules out) would fail this test.
        state.set_focus(&f.tree, Some(f.favorites));
        let toolbar = f.tree.parent(f.tabs).unwrap();
        let chain = [f.albums, f.tabs, toolbar, 0];
        assert_eq!(
            f.tree.focus_landing(&state, &chain),
            Some(f.albums),
            "a click on a group member must land on that member, not on group memory"
        );
    }

    #[test]
    fn focus_landing_on_the_group_container_itself_falls_back_to_the_remembered_member() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.favorites));
        let toolbar = f.tree.parent(f.tabs).unwrap();
        // No member appears in the chain: the click resolved to the
        // group's own rectangle (its padding), not to a member.
        let chain = [f.tabs, toolbar, 0];
        assert_eq!(
            f.tree.focus_landing(&state, &chain),
            Some(f.favorites),
            "a click on the group container falls back to Tab's landing rule"
        );
    }

    #[test]
    fn focus_landing_on_a_disabled_member_falls_back_to_the_groups_first_member() {
        let mut tree = FocusTree::new();
        let root = tree.push(None, FocusRole::Container, true);
        let group = tree.push(Some(root), FocusRole::Group, true);
        let first = tree.push(Some(group), FocusRole::Stop, true);
        let disabled = tree.push(Some(group), FocusRole::Stop, false);
        let state = FocusState::default();
        let chain = [disabled, group, root];
        assert_eq!(
            tree.focus_landing(&state, &chain),
            Some(first),
            "a disabled member is not a legal landing; the walk continues outward to \
             the group, which (with no memory) resolves to its first member"
        );
    }

    #[test]
    fn focus_landing_over_a_chain_with_nothing_focusable_is_none() {
        let mut tree = FocusTree::new();
        let root = tree.push(None, FocusRole::Container, true);
        let child = tree.push(Some(root), FocusRole::Container, true);
        let state = FocusState::default();
        assert_eq!(tree.focus_landing(&state, &[child, root]), None);
    }

    #[test]
    fn focus_landing_outside_an_entered_modal_scope_is_none() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.thumb1));
        state.enter_modal(&f.tree, f.lightbox);
        let grid = f.tree.parent(f.thumb1).unwrap();
        let chain = [f.thumb1, grid, 0];
        assert_eq!(
            f.tree.focus_landing(&state, &chain),
            None,
            "a click outside the entered scope must never move focus out of it"
        );
    }

    #[test]
    fn focus_landing_inside_an_entered_modal_scope_lands_normally() {
        let f = gallery();
        let mut state = FocusState::default();
        state.set_focus(&f.tree, Some(f.thumb1));
        state.enter_modal(&f.tree, f.lightbox);
        let chain = [f.next, f.lightbox, 0];
        assert_eq!(f.tree.focus_landing(&state, &chain), Some(f.next));
    }
}
