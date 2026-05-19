use crate::ir::{
    CompoundOp, HandlerExpr, InterpolationPart, IrBinding, IrComponent, IrHandler, IrLiteral,
    IrNode, IrProp, IrState, IrType,
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
    if !comp.states.is_empty() {
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
        emit_node(out, child, indent + 1);
    }
    out.push_str(&format!("{}}}\n", i));
}

fn emit_prop(out: &mut String, prop: &IrProp, indent: usize) {
    out.push_str(&format!(
        "{}prop {} = {}\n",
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
        // Root node static props
        assert!(out.contains("prop title = \"Counter\""));
        assert!(out.contains("prop backdrop = mica"));
        assert!(out.contains("prop theme = system"));
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
}
