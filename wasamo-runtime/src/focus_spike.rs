//! Projection from a live `WidgetNode` tree onto the pure [`focus_core`] tree.
//!
//! **Spike scaffolding for M4-Phase 2, with no production caller.** It exists
//! to answer one question the pure core cannot answer on its own: *can the
//! core's input be built from a real widget tree, and what does that require
//! the tree to carry?*
//!
//! The measured answer is the reason this module is separate from
//! [`focus_core`]. The core stays free of `windows` types; this module is
//! where the two meet, and the dependency runs one way — projection depends on
//! the core, never the reverse.
//!
//! # What the widget tree can and cannot supply today
//!
//! [`WidgetNode::focus_role`] derives a role from the widget kind alone.
//! It can produce exactly two of the six roles: `Stop` (Button family) and
//! `Container` (everything else). `Group`, `ModalScope`, `ActiveItemList` and
//! `ActiveItem` have **no representation in `WidgetData`** — nothing an author
//! writes today says "these three buttons are one Tab stop" or "this subtree
//! is modal.
//!
//! So the projection takes an explicit override map, and that map is a
//! stand-in for the authored annotations DD-M4-P2-005 has to design. Keying it
//! by pre-order index rather than by any existing identifier is deliberate:
//! there is no widget-name surface in M4 (`dsl-grammar.md` Q1 is an explicit
//! M4 defer), so the spike must not invent one.

use std::collections::BTreeMap;

use crate::focus_core::{FocusId, FocusRole, FocusTree};
use crate::widget::WidgetNode;

/// A projected tree plus the mapping back to the widgets it came from.
///
/// `FocusId` is the pre-order index of the walk, so `widgets[id]` is always
/// the node that produced focus node `id`.
pub struct Projection {
    pub tree: FocusTree,
    pub widgets: Vec<*mut WidgetNode>,
}

impl Projection {
    pub fn widget(&self, id: FocusId) -> *mut WidgetNode {
        self.widgets[id]
    }

    /// The focus id of a widget, by pointer identity.
    pub fn id_of(&self, widget: *mut WidgetNode) -> Option<FocusId> {
        self.widgets.iter().position(|&w| w == widget)
    }
}

/// Walk a live widget tree and build the annotated focus tree for it.
///
/// `overrides` supplies the roles the widget tree cannot express, keyed by
/// pre-order index.
pub fn project(root: &mut WidgetNode, overrides: &BTreeMap<usize, FocusRole>) -> Projection {
    let mut projection = Projection {
        tree: FocusTree::new(),
        widgets: Vec::new(),
    };
    walk(root, None, overrides, &mut projection);
    projection
}

fn walk(
    node: &mut WidgetNode,
    parent: Option<FocusId>,
    overrides: &BTreeMap<usize, FocusRole>,
    out: &mut Projection,
) {
    let index = out.widgets.len();
    let (derived_role, enabled) = node.focus_role();
    let role = overrides.get(&index).copied().unwrap_or(derived_role);
    let id = out.tree.push(parent, role, enabled);
    debug_assert_eq!(id, index, "FocusId is the pre-order index by construction");
    out.widgets.push(node as *mut WidgetNode);

    for child in &mut node.children {
        walk(child, Some(id), overrides, out);
    }
}
