use crate::ir::{
    CompoundOp, ControlFlowNode, HandlerExpr, InterpolationPart, IrBinding, IrComponent, IrHandler,
    IrLiteral, IrMember, IrNode, IrProp, IrState, IrType, KindPayload, TrackSize,
};

/// Serialise an IrComponent to the normative Wasamo IR text format (§8, DD-M2-P6-002).
pub fn emit(component: &IrComponent) -> String {
    let mut out = String::new();
    out.push_str(";wasamo-ir v0\n");
    out.push('\n');
    emit_component(&mut out, component, 0);
    out
}

fn emit_component(out: &mut String, comp: &IrComponent, indent: usize) {
    let i = ind(indent);
    out.push_str(&format!(
        "{}component {} inherits {} {{\n",
        i, comp.name, comp.base
    ));
    for state in &comp.states {
        emit_state(out, state, indent + 1);
    }
    for prop in &comp.host_props {
        emit_host_prop(out, prop, indent + 1);
    }
    for binding in &comp.host_bindings {
        emit_host_binding(out, binding, indent + 1);
    }
    if !comp.states.is_empty() || !comp.host_props.is_empty() || !comp.host_bindings.is_empty() {
        out.push('\n');
    }
    emit_node(out, &comp.root, indent + 1);
    out.push_str(&format!("{}}}\n", i));
}

fn emit_state(out: &mut String, state: &IrState, indent: usize) {
    let type_str = match state.ty {
        IrType::I32 => "i32",
        IrType::Str => "string",
        IrType::Bool => "bool",
    };
    let default_str = emit_literal(&state.default);
    out.push_str(&format!(
        "{}state {}: {} = {}\n",
        ind(indent),
        state.name,
        type_str,
        default_str
    ));
}

fn emit_node(out: &mut String, node: &IrNode, indent: usize) {
    let i = ind(indent);
    out.push_str(&format!("{}node {} {{\n", i, node.widget_type));
    // Grid kind payload (DD-M3-P5-001 carrier c1): the `columns:` /
    // `rows:` track lists emit as `tracks <axis> = <track-list>` lines at
    // the top of the node body, NOT as `prop` entries. This is the
    // Phase-5 implementation textual shape the runtime loader parses in
    // T3 and the dsl_spec §8 fold pins in T7.
    if let Some(KindPayload::Grid { columns, rows }) = &node.kind_payload {
        emit_track_list(out, "columns", columns, indent + 1);
        emit_track_list(out, "rows", rows, indent + 1);
    }
    for prop in &node.props {
        emit_prop(out, prop, indent + 1);
    }
    for binding in &node.bindings {
        emit_binding(out, binding, indent + 1);
    }
    for handler in &node.handlers {
        emit_handler(out, handler, indent + 1);
    }
    for child in &node.children {
        emit_member(out, child, indent + 1);
    }
    out.push_str(&format!("{}}}\n", i));
}

fn emit_member(out: &mut String, member: &IrMember, indent: usize) {
    match member {
        IrMember::Widget(node) => emit_node(out, node, indent),
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            let i = ind(indent);
            for branch in branches {
                out.push_str(&format!("{}if {} {{\n", i, emit_expr(&branch.condition)));
                for body_member in &branch.body {
                    emit_member(out, body_member, indent + 1);
                }
                out.push_str(&format!("{}}}\n", i));
            }
        }
    }
}

/// Emit a Grid track list as `tracks <axis> = <t0> <t1> …` (DD-M3-P5-002).
/// Track elements use their surface forms: a fixed track is its integer,
/// a star track is `<weight>*`. Unit star is written canonically as `1*`
/// (the IR weight is explicit), mirroring the canonical color-emit policy.
fn emit_track_list(out: &mut String, axis: &str, tracks: &[TrackSize], indent: usize) {
    let rendered: Vec<String> = tracks.iter().map(emit_track_size).collect();
    out.push_str(&format!(
        "{}tracks {} = {}\n",
        ind(indent),
        axis,
        rendered.join(" ")
    ));
}

fn emit_track_size(t: &TrackSize) -> String {
    match t {
        TrackSize::Fixed(n) => n.to_string(),
        TrackSize::Star(weight) => format!("{}*", weight),
    }
}

fn emit_prop(out: &mut String, prop: &IrProp, indent: usize) {
    out.push_str(&format!(
        "{}prop {} = {}\n",
        ind(indent),
        prop.name,
        emit_literal(&prop.value)
    ));
}

fn emit_host_prop(out: &mut String, prop: &IrProp, indent: usize) {
    out.push_str(&format!(
        "{}host prop {} = {}\n",
        ind(indent),
        prop.name,
        emit_literal(&prop.value)
    ));
}

fn emit_binding(out: &mut String, binding: &IrBinding, indent: usize) {
    out.push_str(&format!(
        "{}bind {} = {}\n",
        ind(indent),
        binding.prop_name,
        emit_expr(&binding.expr)
    ));
}

fn emit_host_binding(out: &mut String, binding: &IrBinding, indent: usize) {
    out.push_str(&format!(
        "{}host bind {} = {}\n",
        ind(indent),
        binding.prop_name,
        emit_expr(&binding.expr)
    ));
}

fn emit_handler(out: &mut String, handler: &IrHandler, indent: usize) {
    let i = ind(indent);
    out.push_str(&format!("{}on {} {{\n", i, handler.signal));
    out.push_str(&format!("{}    {}\n", i, emit_expr(&handler.expr)));
    out.push_str(&format!("{}}}\n", i));
}

fn emit_literal(lit: &IrLiteral) -> String {
    match lit {
        IrLiteral::Int(n) => n.to_string(),
        IrLiteral::Str(s) => format!("\"{}\"", escape_string(s)),
        IrLiteral::Ident(id) => id.clone(),
        IrLiteral::Bool(b) => (if *b { "true" } else { "false" }).to_string(),
        IrLiteral::Ratio { num, den } => format!("{}:{}", num, den),
        IrLiteral::Color(value) => emit_color_lit(*value),
    }
}

/// Render a `Color(u32)` packed as `0xAARRGGBB`.
///
/// **Canonical emit policy (M3-Phase 2 T5).** Both `#RRGGBB` and
/// `#RRGGBBAA` are valid surface forms (dsl_spec §8.2 `COLOR`) and
/// both are accepted by `wasamoc` lex and by `ir_loader`; the choice
/// of which form the emitter writes is a separate decision. The
/// emitter normalises alpha = `0xFF` to the short `#RRGGBB` form
/// (implicit-opaque) and writes the full `#RRGGBBAA` form otherwise.
/// This keeps the common opaque case readable (`#cccccc`) while
/// reserving the 8-digit form for values where alpha actually
/// carries information (e.g. `#00000080` for a structural scrim).
/// `#RRGGBBFF` written explicitly in surface does **not** survive
/// emit byte-for-byte — it round-trips through `IrLiteral::Color`
/// and re-emits as `#RRGGBB`.
fn emit_color_lit(value: u32) -> String {
    let alpha = (value >> 24) & 0xFF;
    let rgb = value & 0x00FF_FFFF;
    if alpha == 0xFF {
        format!("#{:06x}", rgb)
    } else {
        format!("#{:06x}{:02x}", rgb, alpha)
    }
}

fn emit_expr(expr: &HandlerExpr) -> String {
    match expr {
        HandlerExpr::IntLit(n) => n.to_string(),
        HandlerExpr::StrLit(s) => format!("\"{}\"", escape_string(s)),
        HandlerExpr::BoolLit(b) => (if *b { "true" } else { "false" }).to_string(),
        HandlerExpr::PropRead { path } => format!("(prop-read {})", path),
        HandlerExpr::StrPropRead { path } => format!("(str-prop-read {})", path),
        HandlerExpr::BoolPropRead { path } => format!("(bool-prop-read {})", path),
        HandlerExpr::Assign { lhs, rhs } => {
            format!("(assign {} {})", lhs, emit_expr(rhs))
        }
        HandlerExpr::CompoundAssign { op, lhs, rhs } => {
            let op_str = match op {
                CompoundOp::Add => "+=",
                CompoundOp::Sub => "-=",
                CompoundOp::Mul => "*=",
                CompoundOp::Div => "/=",
            };
            format!("(compound-assign {} {} {})", op_str, lhs, emit_expr(rhs))
        }
        HandlerExpr::Interpolation(parts) => {
            let parts_str: Vec<String> = parts.iter().map(emit_interp_part).collect();
            format!("(interp {})", parts_str.join(" "))
        }
        HandlerExpr::Block(exprs) => {
            if exprs.is_empty() {
                "(block)".to_string()
            } else {
                let inner: Vec<String> = exprs.iter().map(emit_expr).collect();
                format!("(block {})", inner.join(" "))
            }
        }
    }
}

fn emit_interp_part(part: &InterpolationPart) -> String {
    match part {
        InterpolationPart::Literal(s) => format!("\"{}\"", escape_string(s)),
        InterpolationPart::Expr(expr) => format!("({})", emit_expr(expr)),
    }
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn ind(level: usize) -> String {
    "    ".repeat(level)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::check;
    use crate::lexer::tokenize;
    use crate::lower::lower;
    use crate::parser::parse;

    fn emit_src(src: &str) -> String {
        let tokens = tokenize(src, "<test>").unwrap();
        let ast = parse(&tokens, "<test>").unwrap();
        let result = check(&ast, "<test>");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        let comp = lower(&ast, &result.namespace);
        emit(&comp)
    }

    #[test]
    fn header_line_present() {
        let out = emit_src("component C inherits W { VStack {} }");
        assert!(out.starts_with(";wasamo-ir v0\n"), "got: {}", out);
    }

    #[test]
    fn component_structure() {
        let out = emit_src("component C inherits W { VStack {} }");
        assert!(out.contains("component C inherits W {"), "got: {}", out);
        assert!(out.contains("node VStack {"), "got: {}", out);
    }

    #[test]
    fn state_emitted() {
        let out = emit_src("component C inherits W { state count: i32 = 0 VStack {} }");
        assert!(out.contains("state count: i32 = 0"), "got: {}", out);
    }

    #[test]
    fn prop_emitted() {
        let out = emit_src(r#"component C inherits W { VStack { spacing: 12px } }"#);
        assert!(out.contains("prop spacing = 12"), "got: {}", out);
    }

    #[test]
    fn binding_interp_emitted() {
        let out = emit_src(
            r#"component C inherits W { state count: i32 = 0 VStack { text: "Count: \{root.count}" } }"#,
        );
        assert!(
            out.contains(r#"bind text = (interp "Count: " ((prop-read count)))"#),
            "got: {}",
            out
        );
    }

    #[test]
    fn string_state_binding_emits_str_prop_read() {
        let out = emit_src(
            r#"component C inherits W { state label: string = "Ready" VStack { text: "State: \{root.label}" } }"#,
        );
        assert!(
            out.contains(r#"bind text = (interp "State: " ((str-prop-read label)))"#),
            "got: {}",
            out
        );
    }

    #[test]
    fn handler_compound_assign_emitted() {
        let out = emit_src(
            "component C inherits W { state count: i32 = 0 VStack { clicked => { root.count += 1; } } }",
        );
        assert!(out.contains("on clicked {"), "got: {}", out);
        assert!(out.contains("(compound-assign += count 1)"), "got: {}", out);
    }

    #[test]
    fn bool_state_emitted() {
        let out = emit_src("component C inherits W { state ready: bool = false VStack {} }");
        assert!(out.contains("state ready: bool = false"), "got: {}", out);
    }

    #[test]
    fn bool_literal_prop_emitted() {
        let out = emit_src("component C inherits W { Button { enabled: true } }");
        assert!(out.contains("prop enabled = true"), "got: {}", out);
    }

    #[test]
    fn bool_literal_in_handler_emitted() {
        let out = emit_src(
            "component C inherits W { state ready: bool = true Button { clicked => { root.ready = false; } } }",
        );
        assert!(out.contains("on clicked {"), "got: {}", out);
        assert!(out.contains("(assign ready false)"), "got: {}", out);
    }

    // --- T5: Box ratio / color literal IR text emit ---------------------
    //
    // Verifies that ratio and color literals appear in `prop` literal
    // position in their surface forms (`<num>:<den>`, `#RRGGBB` /
    // `#RRGGBBAA`), that the full Box widget node shape from dsl_spec
    // §4.9 emits intact, and that the canonical alpha = `0xFF`
    // normalisation documented on `emit_color_lit` holds end-to-end
    // from surface input through IR text emit. The corresponding
    // IR-text-side fixture for ADR §Phase 2 verification closure item 2
    // is pinned by `box_phase2_ir_text_emit_fixture` below; the load-
    // side half lives in `wasamo-runtime::ir_loader` (T7 / T10).

    #[test]
    fn box_aspect_ratio_emitted_in_surface_form() {
        let out = emit_src("component C inherits W { Box { aspect: 16:9 } }");
        assert!(out.contains("node Box {"), "got: {}", out);
        assert!(out.contains("prop aspect = 16:9"), "got: {}", out);
    }

    #[test]
    fn box_fill_opaque_color_emitted_in_short_form() {
        // `#cccccc` lowers to `IrLiteral::Color(0xFF_CC_CC_CC)`; emit
        // normalises alpha = `0xFF` back to the short `#cccccc` form
        // (canonical emit policy on `emit_color_lit`).
        let out = emit_src("component C inherits W { Box { fill: #cccccc } }");
        assert!(out.contains("prop fill = #cccccc"), "got: {}", out);
        assert!(!out.contains("#ccccccff"), "got: {}", out);
    }

    #[test]
    fn box_fill_color_with_alpha_emitted_in_full_form() {
        // `#00000080` lowers to `IrLiteral::Color(0x80_00_00_00)`;
        // alpha != `0xFF`, so emit writes the full 8-hex form.
        let out = emit_src("component C inherits W { Box { fill: #00000080 } }");
        assert!(out.contains("prop fill = #00000080"), "got: {}", out);
    }

    #[test]
    fn color_emit_normalises_alpha_ff_input_to_short_form() {
        // Surface `#ffffffff` (explicit alpha = `0xFF`) round-trips
        // through `IrLiteral::Color(0xFF_FF_FF_FF)` and re-emits as
        // `#ffffff` — the canonical-policy normalisation direction.
        // This is the byte-for-byte case the policy intentionally
        // does **not** preserve.
        let out = emit_src("component C inherits W { Box { fill: #ffffffff } }");
        assert!(out.contains("prop fill = #ffffff\n"), "got: {}", out);
        assert!(!out.contains("#ffffffff"), "got: {}", out);
    }

    #[test]
    fn box_phase2_placeholder_widget_node_shape_emitted() {
        // dsl_spec §4.9 normative placeholder shape:
        //   Box { aspect: 16:9; fill: #cccccc; Text { text: "Photo 12" } }
        let out = emit_src(
            r#"component C inherits W { Box { aspect: 16:9 fill: #cccccc Text { text: "Photo 12" } } }"#,
        );
        assert!(out.contains("node Box {"), "got: {}", out);
        assert!(out.contains("prop aspect = 16:9"), "got: {}", out);
        assert!(out.contains("prop fill = #cccccc"), "got: {}", out);
        assert!(out.contains("node Text {"), "got: {}", out);
        assert!(out.contains(r#"prop text = "Photo 12""#), "got: {}", out);
    }

    #[test]
    fn box_phase2_ir_text_emit_fixture() {
        // ADR §Phase 2 verification closure item 2 (emit-side gate):
        // for the fixture `Box { aspect: 16:9; fill: #00000080;
        // Text { text: "Photo 12" } }`, an in-process test inspects
        // both the underlying `IrLiteral` variants and the emitted
        // IR text. DD-M3-P2-002 / DD-M3-P2-003 require the literals
        // travel as `IrLiteral::Ratio` / `IrLiteral::Color` directly
        // (not via `PropertyValue`); §8.2 fixes the packed `u32`
        // layout as `0xAARRGGBB`. The load-side half (the same
        // fixture re-entering `wasamo-runtime::ir_loader`) lives in
        // T7 / T10.
        let src = r#"component C inherits W { Box { aspect: 16:9 fill: #00000080 Text { text: "Photo 12" } } }"#;
        let tokens = tokenize(src, "<test>").unwrap();
        let ast = parse(&tokens, "<test>").unwrap();
        let result = check(&ast, "<test>");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        let comp = lower(&ast, &result.namespace);

        let b = &comp.root;
        assert_eq!(b.widget_type, "Box");
        let aspect = &b
            .props
            .iter()
            .find(|p| p.name == "aspect")
            .expect("aspect prop")
            .value;
        let fill = &b
            .props
            .iter()
            .find(|p| p.name == "fill")
            .expect("fill prop")
            .value;
        assert_eq!(*aspect, IrLiteral::Ratio { num: 16, den: 9 });
        assert_eq!(*fill, IrLiteral::Color(0x80_00_00_00));
        assert_eq!(b.children.len(), 1);
        assert!(matches!(
            &b.children[0],
            IrMember::Widget(child) if child.widget_type == "Text"
        ));

        let out = emit(&comp);
        assert!(out.starts_with(";wasamo-ir v0\n"), "got: {}", out);
        assert!(out.contains("node Box {"), "got: {}", out);
        assert!(out.contains("prop aspect = 16:9"), "got: {}", out);
        assert!(out.contains("prop fill = #00000080"), "got: {}", out);
        assert!(out.contains("node Text {"), "got: {}", out);
        assert!(out.contains(r#"prop text = "Photo 12""#), "got: {}", out);
    }

    // --- T4: WrapPanel widget + i32 attribute IR text emit -------------
    //
    // Phase 3 introduces no new emit grammar — `WrapPanel { ... }` emits
    // via the generic `emit_node` path and the three
    // `item-cross-size` / `item-spacing` / `line-spacing` attributes emit
    // via the existing `prop <name> = <int>` form (kebab-case prop names
    // traverse the loader unchanged; cf. ir_loader's identifier lexer
    // which already admits `-` in non-leading position). Attributes
    // absent on the IR side never enter `IrNode.props` (cf. T3 lowering),
    // so they are absent from the emitted text by construction.
    // Round-trip fidelity is exercised cross-crate in
    // `wasamo-runtime/tests/ir_loader_roundtrip.rs`.
    //
    // Listed cases below cover the presence/absence combinations called
    // out by the progress doc's T4 checklist plus the DD-M3-P3-006
    // zero-valid edge.

    #[test]
    fn wrap_panel_zero_children_no_attributes_emitted() {
        let out = emit_src("component C inherits W { WrapPanel {} }");
        assert!(out.contains("node WrapPanel {"), "got: {}", out);
        assert!(!out.contains("prop item-cross-size"), "got: {}", out);
        assert!(!out.contains("prop item-spacing"), "got: {}", out);
        assert!(!out.contains("prop line-spacing"), "got: {}", out);
    }

    #[test]
    fn wrap_panel_all_three_attributes_emitted_as_decimal_ints() {
        let out = emit_src(
            "component C inherits W { WrapPanel { item-cross-size: 96 item-spacing: 8 line-spacing: 12 Box { aspect: 1:1 } Box { aspect: 1:1 } } }",
        );
        assert!(out.contains("node WrapPanel {"), "got: {}", out);
        assert!(out.contains("prop item-cross-size = 96"), "got: {}", out);
        assert!(out.contains("prop item-spacing = 8"), "got: {}", out);
        assert!(out.contains("prop line-spacing = 12"), "got: {}", out);
    }

    #[test]
    fn wrap_panel_only_item_cross_size_omits_other_attributes() {
        let out = emit_src(
            "component C inherits W { WrapPanel { item-cross-size: 64 Box { aspect: 1:1 } Box { aspect: 4:3 } } }",
        );
        assert!(out.contains("prop item-cross-size = 64"), "got: {}", out);
        assert!(!out.contains("prop item-spacing"), "got: {}", out);
        assert!(!out.contains("prop line-spacing"), "got: {}", out);
    }

    #[test]
    fn wrap_panel_only_spacings_omits_item_cross_size() {
        let out = emit_src(
            "component C inherits W { WrapPanel { item-spacing: 4 line-spacing: 6 Text {} Text {} } }",
        );
        assert!(out.contains("prop item-spacing = 4"), "got: {}", out);
        assert!(out.contains("prop line-spacing = 6"), "got: {}", out);
        assert!(!out.contains("prop item-cross-size"), "got: {}", out);
    }

    // The four cases below complete the 2^3 = 8 presence combinations
    // demanded by the T4 progress-doc gate ("each combination of
    // attribute presence / absence"). Together with `zero_children`
    // (000), `all_three` (111), `only_item_cross_size` (100), and
    // `only_spacings` (011) above, the eight presence vectors are
    // covered exhaustively. The emitter is generic so the test value
    // is in pinning the contract surface, not in stressing branches.

    #[test]
    fn wrap_panel_only_item_spacing_present() {
        // presence = 010
        let out =
            emit_src("component C inherits W { WrapPanel { item-spacing: 5 Text {} Text {} } }");
        assert!(out.contains("prop item-spacing = 5"), "got: {}", out);
        assert!(!out.contains("prop item-cross-size"), "got: {}", out);
        assert!(!out.contains("prop line-spacing"), "got: {}", out);
    }

    #[test]
    fn wrap_panel_only_line_spacing_present() {
        // presence = 001
        let out =
            emit_src("component C inherits W { WrapPanel { line-spacing: 7 Text {} Text {} } }");
        assert!(out.contains("prop line-spacing = 7"), "got: {}", out);
        assert!(!out.contains("prop item-cross-size"), "got: {}", out);
        assert!(!out.contains("prop item-spacing"), "got: {}", out);
    }

    #[test]
    fn wrap_panel_item_cross_size_and_item_spacing_present() {
        // presence = 110
        let out = emit_src(
            "component C inherits W { WrapPanel { item-cross-size: 80 item-spacing: 4 Box { aspect: 1:1 } } }",
        );
        assert!(out.contains("prop item-cross-size = 80"), "got: {}", out);
        assert!(out.contains("prop item-spacing = 4"), "got: {}", out);
        assert!(!out.contains("prop line-spacing"), "got: {}", out);
    }

    #[test]
    fn wrap_panel_item_cross_size_and_line_spacing_present() {
        // presence = 101
        let out = emit_src(
            "component C inherits W { WrapPanel { item-cross-size: 80 line-spacing: 10 Box { aspect: 1:1 } } }",
        );
        assert!(out.contains("prop item-cross-size = 80"), "got: {}", out);
        assert!(out.contains("prop line-spacing = 10"), "got: {}", out);
        assert!(!out.contains("prop item-spacing"), "got: {}", out);
    }

    #[test]
    fn wrap_panel_zero_valued_attributes_emitted_as_zero_ints() {
        // DD-M3-P3-006 zero-handling: zero is a *valid* attribute value
        // (rejection threshold is `< 0`, not `<= 0`). The emitter must
        // emit `prop <name> = 0` rather than omitting the prop, so the
        // load side cannot conflate "explicitly zero" with "absent /
        // apply default".
        let out = emit_src(
            "component C inherits W { WrapPanel { item-cross-size: 0 item-spacing: 0 line-spacing: 0 Text {} } }",
        );
        assert!(out.contains("prop item-cross-size = 0"), "got: {}", out);
        assert!(out.contains("prop item-spacing = 0"), "got: {}", out);
        assert!(out.contains("prop line-spacing = 0"), "got: {}", out);
    }

    // --- M3-Phase 5 T1: Grid carrier c1 IR text emit (DD-M3-P5-001) -----
    //
    // Grid track lists emit as `tracks <axis> = <track-list>` lines (NOT
    // `prop` entries), preserving the carrier-c1 invariant. Cell wrappers
    // emit as ordinary `node Cell { … }` subtrees with placement props.
    // This is the Phase-5 implementation textual shape that feeds the
    // runtime loader parse (T3) and the dsl_spec §8 fold (T7).

    #[test]
    fn grid_track_lists_emitted_as_tracks_lines() {
        let out = emit_src(
            r#"component C inherits W {
                Grid {
                    columns: 180 1* 2*
                    rows: 1* 1*
                    Cell { row: 0 column: 0 Text { text: "x" } }
                }
            }"#,
        );
        assert!(out.contains("node Grid {"), "got: {}", out);
        assert!(out.contains("tracks columns = 180 1* 2*"), "got: {}", out);
        assert!(out.contains("tracks rows = 1* 1*"), "got: {}", out);
        // Track lists must NOT appear as prop entries (carrier c1).
        assert!(!out.contains("prop columns"), "got: {}", out);
        assert!(!out.contains("prop rows"), "got: {}", out);
    }

    #[test]
    fn grid_unit_star_emitted_canonically_as_one_star() {
        // Bare `*` lowers to Star(1) and emits canonically as `1*`.
        let out = emit_src("component C inherits W { Grid { columns: * rows: * } }");
        assert!(out.contains("tracks columns = 1*"), "got: {}", out);
        assert!(out.contains("tracks rows = 1*"), "got: {}", out);
    }

    #[test]
    fn grid_cell_emitted_as_node_with_placement_props() {
        let out = emit_src(
            r#"component C inherits W {
                Grid {
                    columns: 1* 1*
                    rows: 1*
                    Cell { row: 0 column: 1 h-align: center Text { text: "x" } }
                }
            }"#,
        );
        assert!(out.contains("node Cell {"), "got: {}", out);
        assert!(out.contains("prop row = 0"), "got: {}", out);
        assert!(out.contains("prop column = 1"), "got: {}", out);
        assert!(out.contains("prop h-align = center"), "got: {}", out);
        assert!(out.contains("node Text {"), "got: {}", out);
    }

    // --- M3-Phase 6 T1: ZStack textual IR emit (DD-M3-P6-001) ----------

    #[test]
    fn zstack_emitted_as_node_with_direct_children_in_order() {
        let out = emit_src(
            r#"component C inherits W {
                ZStack {
                    Box { fill: #00000080 }
                    Text { h-align: center v-align: end text: "caption" }
                }
            }"#,
        );

        assert!(out.contains("node ZStack {"), "got: {}", out);
        assert!(!out.contains("tracks columns"), "got: {}", out);
        assert!(!out.contains("tracks rows"), "got: {}", out);
        assert!(!out.contains("node Cell {"), "got: {}", out);
        let box_pos = out.find("node Box {").expect("Box child emitted");
        let text_pos = out.find("node Text {").expect("Text child emitted");
        assert!(box_pos < text_pos, "got: {}", out);
        assert!(out.contains("prop fill = #00000080"), "got: {}", out);
        assert!(out.contains("prop h-align = center"), "got: {}", out);
        assert!(out.contains("prop v-align = end"), "got: {}", out);
    }

    #[test]
    fn conditional_emitted_as_control_flow_member() {
        let out = emit_src(
            "component C inherits W { state ready: bool = true VStack { if ready { Text { text: \"Shown\" } } } }",
        );
        assert!(out.contains("if (bool-prop-read ready) {"), "got: {}", out);
        assert!(out.contains("node Text {"), "got: {}", out);
        let if_pos = out.find("if (bool-prop-read ready) {").expect("if emitted");
        let text_pos = out.find("node Text {").expect("Text emitted");
        assert!(if_pos < text_pos, "got: {}", out);
    }

    #[test]
    fn full_counter_ir_roundtrip() {
        let src = r#"component Counter inherits Window {
    title: "Counter"
    backdrop: mica
    theme: system
    state count: i32 = 0
    VStack {
        spacing: 12px
        padding: 24px
        Text {
            text: "Count: \{root.count}"
            font: title
        }
        Button {
            text: "Increment"
            style: accent
            clicked => { root.count += 1; }
        }
    }
}"#;
        let out = emit_src(src);
        // Header
        assert!(out.starts_with(";wasamo-ir v0\n"));
        // Component
        assert!(out.contains("component Counter inherits Window {"));
        // State
        assert!(out.contains("state count: i32 = 0"));
        // Host static props
        assert!(out.contains("host prop title = \"Counter\""));
        assert!(out.contains("host prop backdrop = mica"));
        assert!(out.contains("host prop theme = system"));
        // VStack children
        assert!(out.contains("node VStack {"));
        assert!(out.contains("prop spacing = 12"));
        assert!(out.contains("prop padding = 24"));
        // Text binding
        assert!(out.contains("node Text {"));
        assert!(out.contains("bind text ="));
        assert!(out.contains("prop-read count"));
        // Button handler
        assert!(out.contains("node Button {"));
        assert!(out.contains("on clicked {"));
        assert!(out.contains("compound-assign += count 1"));
    }

    #[test]
    fn host_binding_emitted_on_component_surface() {
        // `host_bindings` is a *structural* surface this phase: the Phase-6
        // catalog admits no bindable host attribute (the runtime `validate()`
        // rejects any host binding), but the surface must still round-trip
        // canonically. This pins the emit half — `host bind ...`, on the
        // component surface, never spliced onto the content root. The parse
        // half is covered by `wasamo-runtime`'s
        // `host_surface_rejects_host_binding`, which reaches `validate()`
        // (proving the parser populated `host_bindings`) before rejecting.
        use crate::ir::{HandlerExpr, IrBinding, IrComponent, IrNode};
        let comp = IrComponent {
            name: "C".into(),
            base: "W".into(),
            host_props: vec![],
            host_bindings: vec![IrBinding {
                prop_name: "title".into(),
                expr: HandlerExpr::StrPropRead { path: "s".into() },
            }],
            states: vec![],
            root: IrNode {
                widget_type: "V".into(),
                props: vec![],
                bindings: vec![],
                handlers: vec![],
                children: vec![],
                kind_payload: None,
            },
        };
        let out = emit(&comp);
        assert!(
            out.contains("host bind title = (str-prop-read s)"),
            "got: {out}"
        );
        // Never on the content root.
        assert!(!out.contains("node V {\n    bind"), "got: {out}");
    }
}
