use crate::ast::{ComponentDef, Member};
use crate::diagnostic::Diagnostic;

const M1_WIDGET_TYPES: &[&str] = &["VStack", "HStack", "Text", "Button", "Rectangle"];

pub fn check(ast: &ComponentDef, filename: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    check_members(&ast.members, filename, &mut diags);
    diags
}

fn check_members(members: &[Member], filename: &str, diags: &mut Vec<Diagnostic>) {
    for member in members {
        if let Member::WidgetDecl { type_name, members: children, span } = member {
            if !M1_WIDGET_TYPES.contains(&type_name.as_str()) {
                diags.push(Diagnostic::warning(
                    filename,
                    span.line,
                    span.col,
                    format!(
                        "unknown widget type `{}`; known M1 types: {}",
                        type_name,
                        M1_WIDGET_TYPES.join(", ")
                    ),
                ));
            }
            check_members(children, filename, diags);
        }
    }
}
