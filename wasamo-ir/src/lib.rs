//! Wasamo IR types — the in-memory representation shared between the compiler
//! (`wasamoc`) and the runtime loader (`wasamo-runtime`).
//!
//! Grammar spec: DD-M2-P6-002 / DD-M2-P6-003.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrType {
    I32,
    Str,
    Bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrLiteral {
    Int(i32),
    Str(String),
    Ident(String),
    Bool(bool),
    /// Homogeneous collection literal used only for collection state defaults
    /// and collection-assignment RHS forms. Element type compatibility is
    /// enforced by wasamoc check and by the runtime IR loader.
    List(Vec<IrLiteral>),
    /// Ratio literal — surface form `<num>:<den>`, both sides positive
    /// integer literals (DD-M3-P2-002). Phase 2 admits this literal only
    /// as the RHS of `Box.aspect`; value-validity (`num > 0`, `den > 0`)
    /// is enforced at `wasamoc check`, not at the variant.
    Ratio {
        num: i32,
        den: i32,
    },
    /// Color literal — surface forms `#RRGGBB` / `#RRGGBBAA`, packed in
    /// `0xAARRGGBB` layout with alpha in the most-significant byte
    /// (DD-M3-P2-003; dsl_spec §8.2 `COLOR` token). Phase 2 admits this
    /// literal only as the RHS of `Box.fill`.
    Color(u32),
}

/// HandlerExpr — the tagged-value expression form (DD-M2-P6-003 = Option A).
/// Maps 1-to-1 to the IR text grammar §8.9.
///
/// This is the single source of truth shared between `wasamoc` (lowering /
/// emit) and `wasamo-runtime` (loader / evaluator). Field / variant naming
/// follows the runtime evaluator's prior conventions: `PropRead { path }`
/// (the field carries a dot-separated path like `root.count`); `CompoundOp`
/// variants are operator names without an `Eq` suffix because the assignment
/// is implicit in the enclosing `CompoundAssign`.
#[derive(Debug, Clone, PartialEq)]
pub enum HandlerExpr {
    IntLit(i32),
    StrLit(String),
    BoolLit(bool),
    PropRead {
        path: String,
    },
    StrPropRead {
        path: String,
    },
    BoolPropRead {
        path: String,
    },
    ListPropRead {
        path: String,
        elem: IrType,
    },
    ItemRead {
        binder: String,
    },
    IndexRead {
        binder: String,
    },
    ListAppend {
        path: String,
        elem: IrType,
        value: Box<HandlerExpr>,
    },
    ListDropLast {
        path: String,
        elem: IrType,
    },
    ListLit(Vec<IrLiteral>),
    Assign {
        lhs: String,
        rhs: Box<HandlerExpr>,
    },
    CompoundAssign {
        lhs: String,
        op: CompoundOp,
        rhs: Box<HandlerExpr>,
    },
    Interpolation(Vec<InterpolationPart>),
    Block(Vec<HandlerExpr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompoundOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationPart {
    Literal(String),
    Expr(HandlerExpr),
}

/// A `state` node in the IR component.
#[derive(Debug, Clone, PartialEq)]
pub struct IrState {
    pub name: String,
    pub ty: IrStateType,
    pub default: IrLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrStateType {
    Scalar(IrType),
    Collection(IrType),
}

/// A static property set (`prop name = value`).
#[derive(Debug, Clone, PartialEq)]
pub struct IrProp {
    pub name: String,
    pub value: IrLiteral,
}

/// Grid track sizing form (DD-M3-P5-002). Phase 5 admits fixed integer
/// pixels and weighted-star tracks only; `auto` / `minmax` / named lines
/// are deferred — this enum is their additive extension point (adding a
/// future `Auto` variant lowers existing star/fixed track lists
/// unchanged). Value validity (`Fixed` `>= 1`; `Star` weight in
/// `[1, 1024]`) is enforced at `wasamoc check` and runtime `validate()`,
/// not at the variant. Unit star `*` lowers to `Star(1)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrackSize {
    Fixed(i32),
    Star(u32),
}

/// Grid kind-specific payload carried on a still-generic `IrNode`
/// (DD-M3-P5-001 carrier decision **c1**). Grid's `columns:` / `rows:`
/// track lists live here rather than in `IrProp`, so `IrProp.value`
/// stays strictly `IrLiteral` for every kind (the M2 / Phase 1..4
/// invariant). `IrNode.kind_payload` is `None` for every non-Grid kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KindPayload {
    Grid {
        columns: Vec<TrackSize>,
        rows: Vec<TrackSize>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrAlignment {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrSlotData {
    Grid {
        row: u32,
        column: u32,
        row_span: u32,
        column_span: u32,
        h_align: IrAlignment,
        v_align: IrAlignment,
    },
    ZStack {
        h_align: IrAlignment,
        v_align: IrAlignment,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrChildSlot {
    pub node: IrNode,
    pub slot_data: Option<IrSlotData>,
}

/// A reactive binding (`bind name = expr`).
#[derive(Debug, Clone, PartialEq)]
pub struct IrBinding {
    pub prop_name: String,
    pub expr: HandlerExpr,
}

/// A signal handler (`on signal { expr }` / `on signal("arg") { expr }`).
#[derive(Debug, Clone, PartialEq)]
pub struct IrHandler {
    pub signal: String,
    /// The signal's string argument, if any (DD-M4-P2-005). A separate
    /// field rather than baked into `signal` so the loader can map it
    /// without re-parsing. `key-down` is the only signal that currently
    /// carries one; every other signal's handler has `arg: None`.
    pub arg: Option<String>,
    pub expr: HandlerExpr,
}

/// The recognised key names for `key-down("<key>")` handlers (dsl_spec
/// §4.19 "Keyboard input") — the named non-character keys, in the
/// spec's own order. Exactly 22 entries. `"Tab"` is deliberately absent:
/// Tab always belongs to focus traversal (dsl_spec §4.19 "Which keys the
/// runtime keeps") and can never reach a `key-down` handler. Character
/// keys and modifier combinations (e.g. `"Ctrl+S"`) are likewise absent
/// — they are simply unrecognised names, needing no separate rule.
///
/// Single source of truth (DD-M4-P2-005): `wasamoc::check` and a later
/// runtime virtual-key map both read this table rather than each
/// carrying their own copy.
pub const RECOGNISED_KEY_NAMES: &[&str] = &[
    "Escape",
    "ArrowLeft",
    "ArrowRight",
    "ArrowUp",
    "ArrowDown",
    "Home",
    "End",
    "PageUp",
    "PageDown",
    "Enter",
    "F1",
    "F2",
    "F3",
    "F4",
    "F5",
    "F6",
    "F7",
    "F8",
    "F9",
    "F10",
    "F11",
    "F12",
];

/// Whether `name` is one of the 22 `RECOGNISED_KEY_NAMES` (dsl_spec
/// §4.19). Used by both `wasamoc check` and the runtime IR loader's
/// defense-in-depth gate to reject an unrecognised `key-down` argument.
pub fn is_recognised_key_name(name: &str) -> bool {
    RECOGNISED_KEY_NAMES.contains(&name)
}

/// The canonical handler storage-key spelling for a `(signal, arg)` pair
/// — the DSL/IR surface spelling verbatim: `clicked` for
/// `("clicked", None)`, `key-down("ArrowLeft")` for
/// `("key-down", Some("ArrowLeft"))`. The `None` case returns the bare
/// signal name unchanged so every existing `clicked` / `dismiss` handler
/// keeps its current storage key.
///
/// Single source of truth (DD-M4-P2-005) shared by the IR loader (which
/// writes this as the inline-handler storage key at attachment time) and
/// the runtime dispatcher (which looks handlers up by the same key) — no
/// other code composes this string, so the two sides cannot drift.
pub fn signal_key(signal: &str, arg: Option<&str>) -> String {
    match arg {
        None => signal.to_string(),
        Some(arg) => format!("{signal}(\"{arg}\")"),
    }
}

/// A member in a widget node body.
#[derive(Debug, Clone, PartialEq)]
pub enum IrMember {
    Widget(IrChildSlot),
    ControlFlow(ControlFlowNode),
}

/// Structural control-flow member. Phase 6 ships only the single-branch
/// `If` form; the branch list is the family extension point for `else`.
#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlowNode {
    If {
        branches: Vec<ControlFlowBranch>,
    },
    For {
        binder: String,
        index_binder: Option<String>,
        collection: HandlerExpr,
        body: Vec<IrMember>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ControlFlowBranch {
    pub condition: HandlerExpr,
    pub body: Vec<IrMember>,
}

/// A widget node in the IR tree.
#[derive(Debug, Clone, PartialEq)]
pub struct IrNode {
    pub widget_type: String,
    pub props: Vec<IrProp>,
    pub bindings: Vec<IrBinding>,
    pub handlers: Vec<IrHandler>,
    pub children: Vec<IrMember>,
    /// Grid kind-specific payload (DD-M3-P5-001 carrier c1); `None` for
    /// every non-Grid widget kind. Set explicitly at each construction
    /// site (the IR types deliberately derive no `Default`, so adding
    /// this field surfaces every site at compile time — R-C
    /// construction-site discipline).
    pub kind_payload: Option<KindPayload>,
}

impl IrNode {
    pub fn widget_children(&self) -> impl Iterator<Item = &IrNode> {
        self.children.iter().filter_map(|member| match member {
            IrMember::Widget(slot) => Some(&slot.node),
            IrMember::ControlFlow(_) => None,
        })
    }

    pub fn widget_child_slots(&self) -> impl Iterator<Item = &IrChildSlot> {
        self.children.iter().filter_map(|member| match member {
            IrMember::Widget(slot) => Some(slot),
            IrMember::ControlFlow(_) => None,
        })
    }
}

/// Top-level IR component.
#[derive(Debug, Clone, PartialEq)]
pub struct IrComponent {
    pub name: String,
    pub base: String,
    pub host_props: Vec<IrProp>,
    pub host_bindings: Vec<IrBinding>,
    pub states: Vec<IrState>,
    pub root: IrNode,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn child_slot(node: IrNode) -> IrChildSlot {
        IrChildSlot {
            node,
            slot_data: None,
        }
    }

    #[test]
    fn ir_type_bool_distinct_from_i32_and_str() {
        assert_ne!(IrType::Bool, IrType::I32);
        assert_ne!(IrType::Bool, IrType::Str);
        assert_eq!(IrType::Bool, IrType::Bool);
    }

    #[test]
    fn ir_literal_bool_round_trip_values() {
        let t = IrLiteral::Bool(true);
        let f = IrLiteral::Bool(false);
        assert_ne!(t, f);
        assert_eq!(t.clone(), IrLiteral::Bool(true));
        assert_eq!(f.clone(), IrLiteral::Bool(false));
    }

    #[test]
    fn ir_literal_bool_distinct_from_int_zero_one() {
        assert_ne!(IrLiteral::Bool(false), IrLiteral::Int(0));
        assert_ne!(IrLiteral::Bool(true), IrLiteral::Int(1));
    }

    #[test]
    fn handler_expr_bool_lit_round_trip() {
        let t = HandlerExpr::BoolLit(true);
        let f = HandlerExpr::BoolLit(false);
        assert_ne!(t, f);
        assert_eq!(t.clone(), HandlerExpr::BoolLit(true));
    }

    #[test]
    fn handler_expr_bool_prop_read_carries_path() {
        let e = HandlerExpr::BoolPropRead {
            path: "root.ready".into(),
        };
        match &e {
            HandlerExpr::BoolPropRead { path } => assert_eq!(path, "root.ready"),
            _ => panic!("expected BoolPropRead"),
        }
    }

    #[test]
    fn handler_expr_bool_variants_distinct_from_other_scalars() {
        assert_ne!(HandlerExpr::BoolLit(true), HandlerExpr::IntLit(1));
        assert_ne!(
            HandlerExpr::BoolLit(false),
            HandlerExpr::StrLit("false".into())
        );
        assert_ne!(
            HandlerExpr::BoolPropRead {
                path: "ready".into()
            },
            HandlerExpr::PropRead {
                path: "ready".into()
            }
        );
        assert_ne!(
            HandlerExpr::BoolPropRead {
                path: "ready".into()
            },
            HandlerExpr::StrPropRead {
                path: "ready".into()
            }
        );
    }

    #[test]
    fn ir_literal_ratio_round_trip_values() {
        let r = IrLiteral::Ratio { num: 16, den: 9 };
        match &r {
            IrLiteral::Ratio { num, den } => {
                assert_eq!(*num, 16);
                assert_eq!(*den, 9);
            }
            _ => panic!("expected Ratio"),
        }
        assert_eq!(r.clone(), IrLiteral::Ratio { num: 16, den: 9 });
    }

    #[test]
    fn ir_literal_ratio_distinct_by_components() {
        assert_ne!(
            IrLiteral::Ratio { num: 16, den: 9 },
            IrLiteral::Ratio { num: 9, den: 16 }
        );
        assert_ne!(
            IrLiteral::Ratio { num: 16, den: 9 },
            IrLiteral::Ratio { num: 32, den: 18 }
        );
    }

    #[test]
    fn ir_literal_color_round_trip_value() {
        let c = IrLiteral::Color(0x80_00_00_00);
        match &c {
            IrLiteral::Color(v) => assert_eq!(*v, 0x80_00_00_00),
            _ => panic!("expected Color"),
        }
        assert_eq!(c.clone(), IrLiteral::Color(0x80_00_00_00));
    }

    #[test]
    fn ir_literal_color_distinct_by_packed_value() {
        // #cccccc with implicit alpha 0xFF → 0xFFCCCCCC
        let opaque = IrLiteral::Color(0xFF_CC_CC_CC);
        // #cccccc00 (fully transparent same RGB) → 0x00CCCCCC
        let transparent = IrLiteral::Color(0x00_CC_CC_CC);
        assert_ne!(opaque, transparent);
    }

    #[test]
    fn ir_literal_ratio_and_color_distinct_from_other_variants() {
        assert_ne!(IrLiteral::Ratio { num: 1, den: 1 }, IrLiteral::Int(1));
        assert_ne!(IrLiteral::Color(0), IrLiteral::Int(0));
        assert_ne!(
            IrLiteral::Color(0xFFFFFFFF),
            IrLiteral::Ident("white".into())
        );
    }

    #[test]
    fn track_size_variants_distinct() {
        assert_ne!(TrackSize::Fixed(1), TrackSize::Star(1));
        assert_ne!(TrackSize::Fixed(180), TrackSize::Fixed(181));
        assert_ne!(TrackSize::Star(1), TrackSize::Star(2));
        assert_eq!(TrackSize::Star(3), TrackSize::Star(3));
    }

    #[test]
    fn kind_payload_grid_round_trip() {
        let p = KindPayload::Grid {
            columns: vec![
                TrackSize::Fixed(180),
                TrackSize::Star(1),
                TrackSize::Star(2),
            ],
            rows: vec![TrackSize::Star(1), TrackSize::Star(1)],
        };
        match &p {
            KindPayload::Grid { columns, rows } => {
                assert_eq!(
                    columns,
                    &vec![
                        TrackSize::Fixed(180),
                        TrackSize::Star(1),
                        TrackSize::Star(2)
                    ]
                );
                assert_eq!(rows, &vec![TrackSize::Star(1), TrackSize::Star(1)]);
            }
        }
        assert_eq!(p.clone(), p);
    }

    #[test]
    fn ir_node_kind_payload_defaults_none_for_generic_kinds() {
        let n = IrNode {
            widget_type: "VStack".into(),
            props: vec![],
            bindings: vec![],
            handlers: vec![],
            children: vec![],
            kind_payload: None,
        };
        assert_eq!(n.kind_payload, None);
    }

    #[test]
    fn ir_member_encodes_widget_and_control_flow() {
        let text = IrNode {
            widget_type: "Text".into(),
            props: vec![],
            bindings: vec![],
            handlers: vec![],
            children: vec![],
            kind_payload: None,
        };
        let control = ControlFlowNode::If {
            branches: vec![ControlFlowBranch {
                condition: HandlerExpr::BoolLit(true),
                body: vec![IrMember::Widget(child_slot(text.clone()))],
            }],
        };

        assert!(matches!(
            IrMember::Widget(child_slot(text)),
            IrMember::Widget(_)
        ));
        assert!(matches!(
            IrMember::ControlFlow(control),
            IrMember::ControlFlow(ControlFlowNode::If { .. })
        ));
    }

    #[test]
    fn ir_state_type_distinguishes_scalar_and_collection() {
        assert_eq!(
            IrStateType::Scalar(IrType::I32),
            IrStateType::Scalar(IrType::I32)
        );
        assert_ne!(
            IrStateType::Scalar(IrType::I32),
            IrStateType::Collection(IrType::I32)
        );
        assert_ne!(
            IrStateType::Collection(IrType::I32),
            IrStateType::Collection(IrType::Str)
        );
    }

    #[test]
    fn ir_literal_list_round_trip_values() {
        let lit = IrLiteral::List(vec![IrLiteral::Int(1), IrLiteral::Int(2)]);
        match &lit {
            IrLiteral::List(items) => {
                assert_eq!(items, &vec![IrLiteral::Int(1), IrLiteral::Int(2)]);
            }
            _ => panic!("expected List"),
        }
        assert_eq!(
            lit.clone(),
            IrLiteral::List(vec![IrLiteral::Int(1), IrLiteral::Int(2)])
        );
    }

    #[test]
    fn handler_expr_iteration_variants_carry_bindings_and_types() {
        let collection = HandlerExpr::ListPropRead {
            path: "thumbs".into(),
            elem: IrType::I32,
        };
        let item = HandlerExpr::ItemRead {
            binder: "thumb".into(),
        };
        let index = HandlerExpr::IndexRead { binder: "i".into() };
        let append = HandlerExpr::ListAppend {
            path: "thumbs".into(),
            elem: IrType::I32,
            value: Box::new(HandlerExpr::IntLit(3)),
        };
        let drop_last = HandlerExpr::ListDropLast {
            path: "thumbs".into(),
            elem: IrType::I32,
        };

        assert!(matches!(collection, HandlerExpr::ListPropRead { .. }));
        assert!(matches!(item, HandlerExpr::ItemRead { .. }));
        assert!(matches!(index, HandlerExpr::IndexRead { .. }));
        assert!(matches!(append, HandlerExpr::ListAppend { .. }));
        assert!(matches!(drop_last, HandlerExpr::ListDropLast { .. }));
    }

    #[test]
    fn control_flow_for_encodes_binders_collection_and_body() {
        let body = IrMember::Widget(child_slot(IrNode {
            widget_type: "Text".into(),
            props: vec![],
            bindings: vec![],
            handlers: vec![],
            children: vec![],
            kind_payload: None,
        }));
        let flow = ControlFlowNode::For {
            binder: "thumb".into(),
            index_binder: Some("i".into()),
            collection: HandlerExpr::ListPropRead {
                path: "thumbs".into(),
                elem: IrType::I32,
            },
            body: vec![body],
        };

        match flow {
            ControlFlowNode::For {
                binder,
                index_binder,
                collection,
                body,
            } => {
                assert_eq!(binder, "thumb");
                assert_eq!(index_binder.as_deref(), Some("i"));
                assert!(matches!(collection, HandlerExpr::ListPropRead { .. }));
                assert_eq!(body.len(), 1);
            }
            _ => panic!("expected For"),
        }
    }

    #[test]
    fn widget_children_excludes_for_body_widgets() {
        let direct = IrNode {
            widget_type: "Button".into(),
            props: vec![],
            bindings: vec![],
            handlers: vec![],
            children: vec![],
            kind_payload: None,
        };
        let repeated = IrNode {
            widget_type: "Text".into(),
            props: vec![],
            bindings: vec![],
            handlers: vec![],
            children: vec![],
            kind_payload: None,
        };
        let parent = IrNode {
            widget_type: "WrapPanel".into(),
            props: vec![],
            bindings: vec![],
            handlers: vec![],
            children: vec![
                IrMember::Widget(child_slot(direct.clone())),
                IrMember::ControlFlow(ControlFlowNode::For {
                    binder: "item".into(),
                    index_binder: None,
                    collection: HandlerExpr::ListPropRead {
                        path: "items".into(),
                        elem: IrType::I32,
                    },
                    body: vec![IrMember::Widget(child_slot(repeated))],
                }),
            ],
            kind_payload: None,
        };

        let children: Vec<&IrNode> = parent.widget_children().collect();
        assert_eq!(children, vec![&direct]);
    }

    #[test]
    fn child_slot_carries_optional_slot_data() {
        let node = IrNode {
            widget_type: "Text".into(),
            props: vec![],
            bindings: vec![],
            handlers: vec![],
            children: vec![],
            kind_payload: None,
        };
        let slot = IrChildSlot {
            node,
            slot_data: Some(IrSlotData::ZStack {
                h_align: IrAlignment::End,
                v_align: IrAlignment::Center,
            }),
        };
        assert!(matches!(
            slot.slot_data,
            Some(IrSlotData::ZStack {
                h_align: IrAlignment::End,
                v_align: IrAlignment::Center
            })
        ));
    }

    #[test]
    fn ir_component_separates_host_surface_from_content_root() {
        let component = IrComponent {
            name: "C".into(),
            base: "Window".into(),
            host_props: vec![IrProp {
                name: "title".into(),
                value: IrLiteral::Str("Counter".into()),
            }],
            host_bindings: vec![],
            states: vec![],
            root: IrNode {
                widget_type: "ZStack".into(),
                props: vec![],
                bindings: vec![],
                handlers: vec![],
                children: vec![],
                kind_payload: None,
            },
        };

        assert_eq!(component.host_props[0].name, "title");
        assert!(component.root.props.is_empty());
    }

    #[test]
    fn ir_node_carries_grid_kind_payload() {
        let n = IrNode {
            widget_type: "Grid".into(),
            props: vec![],
            bindings: vec![],
            handlers: vec![],
            children: vec![],
            kind_payload: Some(KindPayload::Grid {
                columns: vec![TrackSize::Fixed(180), TrackSize::Star(1)],
                rows: vec![TrackSize::Star(1)],
            }),
        };
        assert!(matches!(n.kind_payload, Some(KindPayload::Grid { .. })));
    }

    #[test]
    fn ir_state_bool_declaration() {
        let s = IrState {
            name: "ready".into(),
            ty: IrStateType::Scalar(IrType::Bool),
            default: IrLiteral::Bool(false),
        };
        assert_eq!(s.ty, IrStateType::Scalar(IrType::Bool));
        assert_eq!(s.default, IrLiteral::Bool(false));
    }

    // --- M4-Phase 2 T8: key-down argument (DD-M4-P2-005) ----------------

    #[test]
    fn recognised_key_names_has_22_unique_entries() {
        assert_eq!(RECOGNISED_KEY_NAMES.len(), 22);
        let mut seen = std::collections::HashSet::new();
        for name in RECOGNISED_KEY_NAMES {
            assert!(seen.insert(*name), "duplicate key name: `{name}`");
        }
    }

    #[test]
    fn is_recognised_key_name_accepts_every_table_entry() {
        for name in RECOGNISED_KEY_NAMES {
            assert!(
                is_recognised_key_name(name),
                "`{name}` should be recognised"
            );
        }
    }

    #[test]
    fn is_recognised_key_name_rejects_tab_and_character_keys() {
        // `Tab` always belongs to focus traversal (dsl_spec §4.19) and
        // must never be in the recognised set; character keys / modifier
        // combinations are simply unrecognised names.
        assert!(!is_recognised_key_name("Tab"));
        assert!(!is_recognised_key_name("Ctrl+S"));
        assert!(!is_recognised_key_name("a"));
        assert!(!is_recognised_key_name(""));
    }

    #[test]
    fn signal_key_no_arg_returns_bare_signal_name() {
        assert_eq!(signal_key("clicked", None), "clicked");
        assert_eq!(signal_key("dismiss", None), "dismiss");
    }

    #[test]
    fn signal_key_with_arg_returns_dsl_spelling() {
        assert_eq!(
            signal_key("key-down", Some("ArrowLeft")),
            "key-down(\"ArrowLeft\")"
        );
    }
}
