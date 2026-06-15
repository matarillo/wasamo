//! Runtime-side DSL inline handler evaluator (DD-M2-P3-001 = Option A).
//!
//! `HandlerExpr` is the in-memory IR for handler bodies and binding
//! expressions. The type is defined once in `wasamo-ir` and shared by both
//! the compiler (`wasamoc::lower` / `emit`) and this evaluator; this module
//! re-exports it for backwards-compatible call-sites and contributes the
//! evaluator, error types, and diagnostic-location helpers.

pub use wasamo_ir::{CompoundOp, HandlerExpr, InterpolationPart};

/// Evaluation context: property read / write access for a specific component.
pub trait EvalContext {
    /// Read an integer property by dot-separated path (untracked).
    fn get_i32(&self, path: &str) -> Result<i32, EvalError>;

    /// Read a string property by dot-separated path (untracked).
    fn get_string(&self, path: &str) -> Result<String, EvalError> {
        Err(EvalError::UnknownProperty(path.to_string()))
    }

    /// Write an integer property by dot-separated path.
    fn set_i32(&mut self, path: &str, value: i32) -> Result<(), EvalError>;

    /// Read an integer property and report the read to the active reactive
    /// scope (if any). Default impl is untracked — forwards to `get_i32`.
    /// `BindingEvalContext` overrides this to route through `Signal::get()`.
    fn read_i32_tracked(&self, path: &str) -> Result<i32, EvalError> {
        self.get_i32(path)
    }

    /// Read a string property and report the read to the active reactive
    /// scope (if any). Default impl is untracked — forwards to `get_string`.
    fn read_string_tracked(&self, path: &str) -> Result<String, EvalError> {
        self.get_string(path)
    }

    /// Read a bool property by dot-separated path (untracked).
    ///
    /// Default impl returns `UnknownProperty` so existing `EvalContext`
    /// impls without bool support continue to compile (DD-M3-P1-004
    /// Option B + DD-M3-P1-008 Option A).
    fn get_bool(&self, path: &str) -> Result<bool, EvalError> {
        Err(EvalError::UnknownProperty(path.to_string()))
    }

    /// Read a bool property and report the read to the active reactive
    /// scope (if any). Default impl is untracked — forwards to `get_bool`.
    /// `BindingEvalContext` overrides this to route through `Signal::get()`.
    fn read_bool_tracked(&self, path: &str) -> Result<bool, EvalError> {
        self.get_bool(path)
    }

    /// Read the current `for` item as an integer value. Returns `Ok(None)`
    /// when the item position is no longer live.
    fn read_item_i32_tracked(&self, binder: &str) -> Result<Option<i32>, EvalError> {
        Err(EvalError::UnknownProperty(binder.to_string()))
    }

    /// Read the current `for` item as a string value. Returns `Ok(None)`
    /// when the item position is no longer live.
    fn read_item_string_tracked(&self, binder: &str) -> Result<Option<String>, EvalError> {
        Err(EvalError::UnknownProperty(binder.to_string()))
    }

    /// Read the current `for` item as a bool value. Returns `Ok(None)`
    /// when the item position is no longer live.
    fn read_item_bool_tracked(&self, binder: &str) -> Result<Option<bool>, EvalError> {
        Err(EvalError::UnknownProperty(binder.to_string()))
    }

    /// Read the current `for` item for a string-like binding context.
    /// Numeric items are stringified by the context; bool items deliberately
    /// remain a type error unless a later phase defines display formatting.
    fn read_item_binding_tracked(&self, binder: &str) -> Result<Option<String>, EvalError> {
        self.read_item_string_tracked(binder)
    }

    /// Read the current `for` index binder as an integer value.
    fn read_index_tracked(&self, binder: &str) -> Result<Option<i32>, EvalError> {
        Err(EvalError::UnknownProperty(binder.to_string()))
    }

    /// Write a bool property by dot-separated path. Default impl returns
    /// `UnknownProperty`; live impls (the runtime's `HandlerEvalContext`)
    /// override this to drive `Signal<bool>::set`.
    fn set_bool(&mut self, path: &str, _value: bool) -> Result<(), EvalError> {
        Err(EvalError::UnknownProperty(path.to_string()))
    }
}

/// Errors that the evaluator can produce.
#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    UnknownProperty(String),
    TypeMismatch {
        path: String,
    },
    DivisionByZero,
    /// A write expression (`Assign` / `CompoundAssign`) appeared in a
    /// read-only binding context where only property reads are permitted.
    WriteInBindingContext {
        path: String,
    },
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::UnknownProperty(p) => write!(f, "unknown property: {p}"),
            EvalError::TypeMismatch { path } => write!(f, "type mismatch at: {path}"),
            EvalError::DivisionByZero => write!(f, "division by zero"),
            EvalError::WriteInBindingContext { path } => {
                write!(f, "write expression not allowed in binding context: {path}")
            }
        }
    }
}

/// Evaluate a `HandlerExpr` against a mutable context.
///
/// Arithmetic follows Rust wrapping semantics for M2 (DD-M2-P3-003: overflow
/// is not classified as an error; wrapping is the documented behaviour).
/// Division by zero returns `EvalError::DivisionByZero`.
pub fn evaluate(expr: &HandlerExpr, ctx: &mut dyn EvalContext) -> Result<i32, EvalError> {
    match expr {
        HandlerExpr::IntLit(v) => Ok(*v),

        // String-typed forms are only valid in binding context.
        HandlerExpr::StrLit(_)
        | HandlerExpr::StrPropRead { .. }
        | HandlerExpr::ListPropRead { .. }
        | HandlerExpr::ItemRead { .. }
        | HandlerExpr::IndexRead { .. }
        | HandlerExpr::ListAppend { .. }
        | HandlerExpr::ListDropLast { .. }
        | HandlerExpr::ListLit(_)
        | HandlerExpr::Interpolation(_) => Err(EvalError::TypeMismatch {
            path: "<non-integer expression in integer context>".into(),
        }),

        // A bare bool literal / bool property-read in integer context is a
        // type mismatch — only `Assign { rhs: BoolLit | BoolPropRead }` is
        // admitted (DD-M3-P1-004 Option B; the bool-typed `Assign` arm
        // below handles that pairing). `CompoundAssign` over bool is out of
        // scope per ADR §Out of scope.
        HandlerExpr::BoolLit(_) | HandlerExpr::BoolPropRead { .. } => {
            Err(EvalError::TypeMismatch {
                path: "<bool expression in integer context>".into(),
            })
        }

        HandlerExpr::PropRead { path } => ctx.get_i32(path),

        // `Assign` peeks at `rhs` to pick the typed write path.
        // DD-M3-P1-004 Option B + DD-M3-P1-008 Option A admit
        // `rhs ∈ { BoolLit, BoolPropRead }` for bool-typed targets.
        // Other variants stay on the i32 path. The return value of a
        // bool-typed assign is unused (handlers run for side effects);
        // returning 0 keeps `evaluate()`'s `Result<i32, _>` contract
        // without implicit bool→i32 coercion (DD-M3-P1-001 Option B
        // explicitly rejected).
        HandlerExpr::Assign { lhs, rhs } => match rhs.as_ref() {
            HandlerExpr::BoolLit(b) => {
                ctx.set_bool(lhs, *b)?;
                Ok(0)
            }
            HandlerExpr::BoolPropRead { path } => {
                let v = ctx.get_bool(path)?;
                ctx.set_bool(lhs, v)?;
                Ok(0)
            }
            _ => {
                let v = evaluate(rhs, ctx)?;
                ctx.set_i32(lhs, v)?;
                Ok(v)
            }
        },

        HandlerExpr::CompoundAssign { lhs, op, rhs } => {
            let current = ctx.get_i32(lhs)?;
            let rhs_val = evaluate(rhs, ctx)?;
            let result = match op {
                CompoundOp::Add => current.wrapping_add(rhs_val),
                CompoundOp::Sub => current.wrapping_sub(rhs_val),
                CompoundOp::Mul => current.wrapping_mul(rhs_val),
                CompoundOp::Div => {
                    if rhs_val == 0 {
                        return Err(EvalError::DivisionByZero);
                    }
                    current.wrapping_div(rhs_val)
                }
            };
            ctx.set_i32(lhs, result)?;
            Ok(result)
        }

        HandlerExpr::Block(stmts) => {
            let mut last = 0i32;
            for stmt in stmts {
                last = evaluate(stmt, ctx)?;
            }
            Ok(last)
        }
    }
}

/// Format the coarse handler-location identifier used in diagnostic messages
/// (DD-M2-P3-004 = Option B).
///
/// Format: `<component>.<widget-path>.<signal>`
/// - `component`: name of the UI component that declared the inline handler
///   (e.g. `"Counter"`). Supplied by the IR loader at widget-tree build time;
///   Phase 3 callers pass `"?"` as a placeholder until Phase 6 fills it in.
/// - `widget_path`: slash-free widget path segments joined by `.`, with an
///   optional `[index]` suffix for repeated siblings (e.g. `"button[1]"`).
/// - `signal`: the signal name (e.g. `"clicked"`).
///
/// This is pure formatting logic with no runtime dependencies; all inputs are
/// caller-supplied strings or index values.
pub fn format_handler_location(
    component: &str,
    widget_path: &[WidgetPathSegment],
    signal: &str,
) -> String {
    if widget_path.is_empty() {
        return format!("{component}.{signal}");
    }
    let path_str = widget_path
        .iter()
        .map(|seg| match seg.index {
            None => seg.name.clone(),
            Some(i) => format!("{}[{}]", seg.name, i),
        })
        .collect::<Vec<_>>()
        .join(".");
    format!("{component}.{path_str}.{signal}")
}

/// One segment of a dot-path widget identifier.
#[derive(Debug, Clone, PartialEq)]
pub struct WidgetPathSegment {
    /// Widget type or instance name (e.g. `"button"`, `"label"`).
    pub name: String,
    /// Positional index among siblings of the same name, if disambiguation is
    /// needed. `None` when the name is unique at that level.
    pub index: Option<usize>,
}

impl WidgetPathSegment {
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            index: None,
        }
    }
    pub fn indexed(name: impl Into<String>, index: usize) -> Self {
        Self {
            name: name.into(),
            index: Some(index),
        }
    }
}

/// Invoke a `HandlerExpr` against `ctx`, catching any panic that the evaluator
/// might raise (DD-M2-P3-003 = Option A). On error or panic, logs one line to
/// stderr in the form:
/// `wasamo: handler error in <location>: <message>`
/// where `location` is a caller-supplied coarse identifier
/// (see `format_handler_location` in this module).
/// Returns `true` if the handler completed without error.
pub fn invoke_handler(expr: &HandlerExpr, ctx: &mut dyn EvalContext, location: &str) -> bool {
    // RefUnwindSafe is not automatically satisfied for trait objects, so we
    // evaluate inside a wrapper that AssertUnwindSafe asserts the invariant.
    // The safety argument: `ctx` releases any interior borrows before this
    // call (see DD-M2-P3-003 risk note); no RefCell is live across the boundary.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| evaluate(expr, ctx)));
    match result {
        Ok(Ok(_)) => true,
        Ok(Err(e)) => {
            eprintln!("wasamo: handler error in {location}: {e}");
            false
        }
        Err(payload) => {
            let msg = payload
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                .unwrap_or("unknown panic");
            eprintln!("wasamo: handler error in {location}: {msg}");
            false
        }
    }
}

/// Evaluate a `HandlerExpr` in binding (read-only) context and return a
/// string representation of the result.
///
/// - `StrLit` returns the literal unchanged.
/// - `Interpolation` concatenates literal parts with stringified expression
///   parts evaluated via `evaluate_tracked`.
/// - Integer-typed leaf expressions (`IntLit`, `PropRead`, integer `Block`)
///   are evaluated via `evaluate_tracked` and stringified.
/// - `Assign` and `CompoundAssign` are rejected at the AST level with
///   `EvalError::WriteInBindingContext` regardless of nesting depth.
///
/// Reads go through `ctx.read_i32_tracked()`, so `BindingEvalContext` (which
/// routes that method through `Signal::get()`) causes dependency collection
/// to happen automatically as a side effect of evaluation.
pub fn evaluate_binding(
    expr: &HandlerExpr,
    ctx: &mut dyn EvalContext,
) -> Result<String, EvalError> {
    match expr {
        HandlerExpr::StrLit(s) => Ok(s.clone()),
        HandlerExpr::Interpolation(parts) => {
            let mut out = String::new();
            for part in parts {
                match part {
                    InterpolationPart::Literal(s) => out.push_str(s),
                    InterpolationPart::Expr(e) => {
                        out.push_str(&evaluate_binding_part(e, ctx)?);
                    }
                }
            }
            Ok(out)
        }
        HandlerExpr::StrPropRead { path } => ctx.read_string_tracked(path),
        // Integer-typed top-level binding (e.g. a bare `root.count` binding).
        _ => evaluate_tracked(expr, ctx).map(|v| v.to_string()),
    }
}

pub(crate) fn evaluate_binding_optional(
    expr: &HandlerExpr,
    ctx: &mut dyn EvalContext,
) -> Result<Option<String>, EvalError> {
    match expr {
        HandlerExpr::StrLit(s) => Ok(Some(s.clone())),
        HandlerExpr::Interpolation(parts) => {
            let mut out = String::new();
            for part in parts {
                match part {
                    InterpolationPart::Literal(s) => out.push_str(s),
                    InterpolationPart::Expr(e) => match evaluate_binding_part_optional(e, ctx)? {
                        Some(value) => out.push_str(&value),
                        None => return Ok(None),
                    },
                }
            }
            Ok(Some(out))
        }
        HandlerExpr::StrPropRead { path } => ctx.read_string_tracked(path).map(Some),
        HandlerExpr::ItemRead { binder } => ctx.read_item_binding_tracked(binder),
        HandlerExpr::IndexRead { binder } => ctx
            .read_index_tracked(binder)
            .map(|value| value.map(|index| index.to_string())),
        _ => evaluate_tracked_optional(expr, ctx).map(|value| value.map(|v| v.to_string())),
    }
}

fn evaluate_binding_part(
    expr: &HandlerExpr,
    ctx: &mut dyn EvalContext,
) -> Result<String, EvalError> {
    match expr {
        HandlerExpr::StrPropRead { path } => ctx.read_string_tracked(path),
        _ => evaluate_tracked(expr, ctx).map(|v| v.to_string()),
    }
}

fn evaluate_binding_part_optional(
    expr: &HandlerExpr,
    ctx: &mut dyn EvalContext,
) -> Result<Option<String>, EvalError> {
    match expr {
        HandlerExpr::StrPropRead { path } => ctx.read_string_tracked(path).map(Some),
        HandlerExpr::ItemRead { binder } => ctx.read_item_binding_tracked(binder),
        HandlerExpr::IndexRead { binder } => ctx
            .read_index_tracked(binder)
            .map(|value| value.map(|index| index.to_string())),
        _ => evaluate_tracked_optional(expr, ctx).map(|value| value.map(|v| v.to_string())),
    }
}

/// Evaluate a `HandlerExpr` in bool-typed binding (read-only) context.
///
/// Per DD-M3-P1-007 Option A, bool bindings travel a separate per-type
/// evaluator/writer pair rather than funnelling through a `PropertyValue`
/// union — this keeps F5 (`TypedValue` deferral) structurally enforced.
///
/// Accepted shapes (DD-M3-P1-003):
/// - `BoolLit(b)` returns the literal.
/// - `BoolPropRead { path }` reads through `ctx.read_bool_tracked`, so
///   `BindingEvalContext` registers the read with the active reactive
///   scope and the binding subscribes to the source `Signal<bool>`.
///
/// All other variants are rejected with `EvalError::TypeMismatch`. This
/// mirrors the way `evaluate()` rejects string-typed forms — binding
/// lowering for a bool-typed target property only produces the two
/// shapes above (DD-M3-P1-010), so any other variant reaching this
/// evaluator is an IR-level type error rather than a user-syntax error.
pub fn evaluate_bool_binding(
    expr: &HandlerExpr,
    ctx: &mut dyn EvalContext,
) -> Result<bool, EvalError> {
    match expr {
        HandlerExpr::BoolLit(b) => Ok(*b),
        HandlerExpr::BoolPropRead { path } => ctx.read_bool_tracked(path),
        _ => Err(EvalError::TypeMismatch {
            path: "<non-bool expression in bool binding context>".into(),
        }),
    }
}

pub(crate) fn evaluate_bool_binding_optional(
    expr: &HandlerExpr,
    ctx: &mut dyn EvalContext,
) -> Result<Option<bool>, EvalError> {
    match expr {
        HandlerExpr::BoolLit(b) => Ok(Some(*b)),
        HandlerExpr::BoolPropRead { path } => ctx.read_bool_tracked(path).map(Some),
        HandlerExpr::ItemRead { binder } => ctx.read_item_bool_tracked(binder),
        _ => Err(EvalError::TypeMismatch {
            path: "<non-bool expression in bool binding context>".into(),
        }),
    }
}

/// Integer-typed evaluation in binding (read-only) mode.
///
/// Like `evaluate()` but:
/// - Uses `ctx.read_i32_tracked()` for `PropRead` (enabling dependency tracking).
/// - Rejects `Assign` and `CompoundAssign` with `WriteInBindingContext`.
/// - Rejects string-typed forms (`StrLit`, `Interpolation`) with `TypeMismatch`.
fn evaluate_tracked(expr: &HandlerExpr, ctx: &mut dyn EvalContext) -> Result<i32, EvalError> {
    match expr {
        HandlerExpr::IntLit(v) => Ok(*v),

        HandlerExpr::PropRead { path } => ctx.read_i32_tracked(path),

        HandlerExpr::Assign { lhs, .. } | HandlerExpr::CompoundAssign { lhs, .. } => {
            Err(EvalError::WriteInBindingContext { path: lhs.clone() })
        }

        HandlerExpr::Block(stmts) => {
            let mut last = 0i32;
            for stmt in stmts {
                last = evaluate_tracked(stmt, ctx)?;
            }
            Ok(last)
        }

        HandlerExpr::StrLit(_)
        | HandlerExpr::StrPropRead { .. }
        | HandlerExpr::ListPropRead { .. }
        | HandlerExpr::ItemRead { .. }
        | HandlerExpr::IndexRead { .. }
        | HandlerExpr::ListAppend { .. }
        | HandlerExpr::ListDropLast { .. }
        | HandlerExpr::ListLit(_)
        | HandlerExpr::Interpolation(_) => Err(EvalError::TypeMismatch {
            path: "<string expression in integer context>".into(),
        }),

        // M3-Phase 1 T7 / T8 will provide a bool-typed binding evaluator
        // (`evaluate_bool_binding`); until that lands, a bool expression in
        // integer binding context is a type mismatch.
        HandlerExpr::BoolLit(_) | HandlerExpr::BoolPropRead { .. } => {
            Err(EvalError::TypeMismatch {
                path: "<bool expression in integer context>".into(),
            })
        }
    }
}

fn evaluate_tracked_optional(
    expr: &HandlerExpr,
    ctx: &mut dyn EvalContext,
) -> Result<Option<i32>, EvalError> {
    match expr {
        HandlerExpr::IntLit(v) => Ok(Some(*v)),

        HandlerExpr::PropRead { path } => ctx.read_i32_tracked(path).map(Some),
        HandlerExpr::ItemRead { binder } => ctx.read_item_i32_tracked(binder),
        HandlerExpr::IndexRead { binder } => ctx.read_index_tracked(binder),

        HandlerExpr::Assign { lhs, .. } | HandlerExpr::CompoundAssign { lhs, .. } => {
            Err(EvalError::WriteInBindingContext { path: lhs.clone() })
        }

        HandlerExpr::Block(stmts) => {
            let mut last = Some(0i32);
            for stmt in stmts {
                last = evaluate_tracked_optional(stmt, ctx)?;
                if last.is_none() {
                    return Ok(None);
                }
            }
            Ok(last)
        }

        HandlerExpr::StrLit(_)
        | HandlerExpr::StrPropRead { .. }
        | HandlerExpr::ListPropRead { .. }
        | HandlerExpr::ListAppend { .. }
        | HandlerExpr::ListDropLast { .. }
        | HandlerExpr::ListLit(_)
        | HandlerExpr::Interpolation(_) => Err(EvalError::TypeMismatch {
            path: "<string expression in integer context>".into(),
        }),

        HandlerExpr::BoolLit(_) | HandlerExpr::BoolPropRead { .. } => {
            Err(EvalError::TypeMismatch {
                path: "<bool expression in integer context>".into(),
            })
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Simple in-memory context for unit testing.
    struct MapCtx {
        i32s: HashMap<String, i32>,
        strings: HashMap<String, String>,
        bools: HashMap<String, bool>,
    }

    impl MapCtx {
        fn new(pairs: &[(&str, i32)]) -> Self {
            Self {
                i32s: pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                strings: HashMap::new(),
                bools: HashMap::new(),
            }
        }
        fn with_strings(mut self, pairs: &[(&str, &str)]) -> Self {
            self.strings = pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            self
        }
        fn with_bools(mut self, pairs: &[(&str, bool)]) -> Self {
            self.bools = pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect();
            self
        }
        fn get(&self, key: &str) -> i32 {
            *self.i32s.get(key).unwrap_or(&0)
        }
        fn get_b(&self, key: &str) -> bool {
            *self.bools.get(key).unwrap_or(&false)
        }
    }

    impl EvalContext for MapCtx {
        fn get_i32(&self, path: &str) -> Result<i32, EvalError> {
            self.i32s
                .get(path)
                .copied()
                .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
        }
        fn get_string(&self, path: &str) -> Result<String, EvalError> {
            self.strings
                .get(path)
                .cloned()
                .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
        }
        fn get_bool(&self, path: &str) -> Result<bool, EvalError> {
            self.bools
                .get(path)
                .copied()
                .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
        }
        fn set_i32(&mut self, path: &str, value: i32) -> Result<(), EvalError> {
            self.i32s.insert(path.to_string(), value);
            Ok(())
        }
        fn set_bool(&mut self, path: &str, value: bool) -> Result<(), EvalError> {
            self.bools.insert(path.to_string(), value);
            Ok(())
        }
    }

    #[test]
    fn int_lit() {
        let mut ctx = MapCtx::new(&[]);
        assert_eq!(evaluate(&HandlerExpr::IntLit(42), &mut ctx), Ok(42));
    }

    #[test]
    fn prop_read() {
        let mut ctx = MapCtx::new(&[("root.count", 7)]);
        let expr = HandlerExpr::PropRead {
            path: "root.count".into(),
        };
        assert_eq!(evaluate(&expr, &mut ctx), Ok(7));
    }

    #[test]
    fn prop_read_unknown() {
        let mut ctx = MapCtx::new(&[]);
        let expr = HandlerExpr::PropRead {
            path: "root.count".into(),
        };
        assert_eq!(
            evaluate(&expr, &mut ctx),
            Err(EvalError::UnknownProperty("root.count".into()))
        );
    }

    #[test]
    fn assign() {
        let mut ctx = MapCtx::new(&[("root.count", 0)]);
        let expr = HandlerExpr::Assign {
            lhs: "root.count".into(),
            rhs: Box::new(HandlerExpr::IntLit(5)),
        };
        assert_eq!(evaluate(&expr, &mut ctx), Ok(5));
        assert_eq!(ctx.get("root.count"), 5);
    }

    #[test]
    fn compound_add() {
        let mut ctx = MapCtx::new(&[("root.count", 3)]);
        // root.count += 1
        let expr = HandlerExpr::CompoundAssign {
            lhs: "root.count".into(),
            op: CompoundOp::Add,
            rhs: Box::new(HandlerExpr::IntLit(1)),
        };
        assert_eq!(evaluate(&expr, &mut ctx), Ok(4));
        assert_eq!(ctx.get("root.count"), 4);
    }

    #[test]
    fn compound_sub() {
        let mut ctx = MapCtx::new(&[("root.count", 10)]);
        let expr = HandlerExpr::CompoundAssign {
            lhs: "root.count".into(),
            op: CompoundOp::Sub,
            rhs: Box::new(HandlerExpr::IntLit(3)),
        };
        assert_eq!(evaluate(&expr, &mut ctx), Ok(7));
        assert_eq!(ctx.get("root.count"), 7);
    }

    #[test]
    fn compound_mul() {
        let mut ctx = MapCtx::new(&[("root.count", 4)]);
        let expr = HandlerExpr::CompoundAssign {
            lhs: "root.count".into(),
            op: CompoundOp::Mul,
            rhs: Box::new(HandlerExpr::IntLit(3)),
        };
        assert_eq!(evaluate(&expr, &mut ctx), Ok(12));
        assert_eq!(ctx.get("root.count"), 12);
    }

    #[test]
    fn compound_div() {
        let mut ctx = MapCtx::new(&[("root.count", 12)]);
        let expr = HandlerExpr::CompoundAssign {
            lhs: "root.count".into(),
            op: CompoundOp::Div,
            rhs: Box::new(HandlerExpr::IntLit(4)),
        };
        assert_eq!(evaluate(&expr, &mut ctx), Ok(3));
        assert_eq!(ctx.get("root.count"), 3);
    }

    #[test]
    fn division_by_zero() {
        let mut ctx = MapCtx::new(&[("root.count", 5)]);
        let expr = HandlerExpr::CompoundAssign {
            lhs: "root.count".into(),
            op: CompoundOp::Div,
            rhs: Box::new(HandlerExpr::IntLit(0)),
        };
        assert_eq!(evaluate(&expr, &mut ctx), Err(EvalError::DivisionByZero));
        // Value unchanged on error.
        assert_eq!(ctx.get("root.count"), 5);
    }

    #[test]
    fn wrapping_overflow_add() {
        let mut ctx = MapCtx::new(&[("x", i32::MAX)]);
        let expr = HandlerExpr::CompoundAssign {
            lhs: "x".into(),
            op: CompoundOp::Add,
            rhs: Box::new(HandlerExpr::IntLit(1)),
        };
        // Wrapping: i32::MAX + 1 == i32::MIN
        assert_eq!(evaluate(&expr, &mut ctx), Ok(i32::MIN));
        assert_eq!(ctx.get("x"), i32::MIN);
    }

    #[test]
    fn wrapping_overflow_sub() {
        let mut ctx = MapCtx::new(&[("x", i32::MIN)]);
        let expr = HandlerExpr::CompoundAssign {
            lhs: "x".into(),
            op: CompoundOp::Sub,
            rhs: Box::new(HandlerExpr::IntLit(1)),
        };
        assert_eq!(evaluate(&expr, &mut ctx), Ok(i32::MAX));
        assert_eq!(ctx.get("x"), i32::MAX);
    }

    #[test]
    fn nested_block() {
        let mut ctx = MapCtx::new(&[("a", 0), ("b", 0)]);
        // { a = 1; b = 2; }
        let expr = HandlerExpr::Block(vec![
            HandlerExpr::Assign {
                lhs: "a".into(),
                rhs: Box::new(HandlerExpr::IntLit(1)),
            },
            HandlerExpr::Assign {
                lhs: "b".into(),
                rhs: Box::new(HandlerExpr::IntLit(2)),
            },
        ]);
        assert_eq!(evaluate(&expr, &mut ctx), Ok(2));
        assert_eq!(ctx.get("a"), 1);
        assert_eq!(ctx.get("b"), 2);
    }

    #[test]
    fn block_with_compound_then_read() {
        let mut ctx = MapCtx::new(&[("root.count", 5)]);
        // { root.count += 1; root.count }  → 6
        let expr = HandlerExpr::Block(vec![
            HandlerExpr::CompoundAssign {
                lhs: "root.count".into(),
                op: CompoundOp::Add,
                rhs: Box::new(HandlerExpr::IntLit(1)),
            },
            HandlerExpr::PropRead {
                path: "root.count".into(),
            },
        ]);
        assert_eq!(evaluate(&expr, &mut ctx), Ok(6));
        assert_eq!(ctx.get("root.count"), 6);
    }

    #[test]
    fn empty_block() {
        let mut ctx = MapCtx::new(&[]);
        assert_eq!(evaluate(&HandlerExpr::Block(vec![]), &mut ctx), Ok(0));
    }

    // ── format_handler_location tests (DD-M2-P3-004) ─────────────────────────

    #[test]
    fn location_no_path_segments() {
        let loc = format_handler_location("Counter", &[], "clicked");
        assert_eq!(loc, "Counter.clicked");
    }

    #[test]
    fn location_single_segment_no_index() {
        let loc =
            format_handler_location("Counter", &[WidgetPathSegment::named("button")], "clicked");
        assert_eq!(loc, "Counter.button.clicked");
    }

    #[test]
    fn location_single_segment_with_index() {
        let loc = format_handler_location(
            "Counter",
            &[WidgetPathSegment::indexed("button", 1)],
            "clicked",
        );
        assert_eq!(loc, "Counter.button[1].clicked");
    }

    #[test]
    fn location_nested_path() {
        let loc = format_handler_location(
            "App",
            &[
                WidgetPathSegment::named("vstack"),
                WidgetPathSegment::indexed("button", 0),
            ],
            "clicked",
        );
        assert_eq!(loc, "App.vstack.button[0].clicked");
    }

    #[test]
    fn location_placeholder_component() {
        // Phase 3 placeholder: component not yet known from IR.
        let loc = format_handler_location("?", &[WidgetPathSegment::named("button")], "clicked");
        assert_eq!(loc, "?.button.clicked");
    }

    // ── invoke_handler tests (DD-M2-P3-003) ──────────────────────────────────

    #[test]
    fn invoke_handler_success() {
        let mut ctx = MapCtx::new(&[("x", 0)]);
        let expr = HandlerExpr::Assign {
            lhs: "x".into(),
            rhs: Box::new(HandlerExpr::IntLit(7)),
        };
        let ok = invoke_handler(&expr, &mut ctx, "Counter.button.clicked");
        assert!(ok);
        assert_eq!(ctx.get("x"), 7);
    }

    #[test]
    fn invoke_handler_eval_error_returns_false() {
        let mut ctx = MapCtx::new(&[("x", 5)]);
        // Division by zero → EvalError, not a panic.
        let expr = HandlerExpr::CompoundAssign {
            lhs: "x".into(),
            op: CompoundOp::Div,
            rhs: Box::new(HandlerExpr::IntLit(0)),
        };
        let ok = invoke_handler(&expr, &mut ctx, "Counter.button.clicked");
        assert!(!ok);
        // Value unchanged on error.
        assert_eq!(ctx.get("x"), 5);
    }

    #[test]
    fn invoke_handler_catches_panic() {
        // EvalContext implementation that panics on set_i32.
        struct PanicCtx;
        impl EvalContext for PanicCtx {
            fn get_i32(&self, _: &str) -> Result<i32, EvalError> {
                Ok(0)
            }
            fn set_i32(&mut self, _: &str, _: i32) -> Result<(), EvalError> {
                panic!("injected panic for testing")
            }
        }
        let expr = HandlerExpr::Assign {
            lhs: "x".into(),
            rhs: Box::new(HandlerExpr::IntLit(1)),
        };
        let mut ctx = PanicCtx;
        // Must not propagate the panic; returns false.
        let ok = invoke_handler(&expr, &mut ctx, "Counter.button.clicked");
        assert!(!ok);
    }

    /// DD-M2-P3-002 Option B: inline handler runs and mutates state *before*
    /// the host listener observes it. Simulated here by recording the order
    /// of side-effects through a shared event log: the inline handler writes
    /// to "x", then the "host" reads "x" after — the read sees the updated value.
    #[test]
    fn inline_before_host_ordering() {
        let mut ctx = MapCtx::new(&[("x", 0)]);
        let mut event_log: Vec<String> = Vec::new();

        // Inline handler: x += 10
        let inline = HandlerExpr::CompoundAssign {
            lhs: "x".into(),
            op: CompoundOp::Add,
            rhs: Box::new(HandlerExpr::IntLit(10)),
        };

        // Step 1: inline handler fires.
        evaluate(&inline, &mut ctx).unwrap();
        event_log.push(format!("inline: x={}", ctx.get("x")));

        // Step 2: host listener fires (reads x, which is now 10).
        let host_saw = ctx.get("x");
        event_log.push(format!("host: x={host_saw}"));

        assert_eq!(event_log, ["inline: x=10", "host: x=10"]);
    }

    // ── evaluate_binding / evaluate_tracked tests (DD-M2-P5-006) ─────────────

    #[test]
    fn binding_str_lit() {
        let mut ctx = MapCtx::new(&[]);
        let expr = HandlerExpr::StrLit("hello".into());
        assert_eq!(evaluate_binding(&expr, &mut ctx), Ok("hello".into()));
    }

    #[test]
    fn binding_interpolation_counter() {
        // Simulates `"Count: \{root.count}"` with root.count = 7.
        let mut ctx = MapCtx::new(&[("root.count", 7)]);
        let expr = HandlerExpr::Interpolation(vec![
            InterpolationPart::Literal("Count: ".into()),
            InterpolationPart::Expr(HandlerExpr::PropRead {
                path: "root.count".into(),
            }),
        ]);
        assert_eq!(evaluate_binding(&expr, &mut ctx), Ok("Count: 7".into()));
    }

    #[test]
    fn binding_bare_int_prop_read() {
        // A bare integer property binding (not wrapped in interpolation).
        let mut ctx = MapCtx::new(&[("root.count", 3)]);
        let expr = HandlerExpr::PropRead {
            path: "root.count".into(),
        };
        assert_eq!(evaluate_binding(&expr, &mut ctx), Ok("3".into()));
    }

    #[test]
    fn binding_bare_string_prop_read() {
        let mut ctx = MapCtx::new(&[]).with_strings(&[("root.label", "Ready")]);
        let expr = HandlerExpr::StrPropRead {
            path: "root.label".into(),
        };
        assert_eq!(evaluate_binding(&expr, &mut ctx), Ok("Ready".into()));
    }

    #[test]
    fn binding_interpolation_string_prop_read() {
        let mut ctx = MapCtx::new(&[("root.count", 3)]).with_strings(&[("root.label", "Ready")]);
        let expr = HandlerExpr::Interpolation(vec![
            InterpolationPart::Literal("State: ".into()),
            InterpolationPart::Expr(HandlerExpr::StrPropRead {
                path: "root.label".into(),
            }),
            InterpolationPart::Literal(" #".into()),
            InterpolationPart::Expr(HandlerExpr::PropRead {
                path: "root.count".into(),
            }),
        ]);
        assert_eq!(
            evaluate_binding(&expr, &mut ctx),
            Ok("State: Ready #3".into())
        );
    }

    #[test]
    fn binding_rejects_string_read_in_integer_context() {
        let mut ctx = MapCtx::new(&[]).with_strings(&[("root.label", "Ready")]);
        let expr = HandlerExpr::Block(vec![HandlerExpr::StrPropRead {
            path: "root.label".into(),
        }]);
        assert!(matches!(
            evaluate_binding(&expr, &mut ctx),
            Err(EvalError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn binding_rejects_assign() {
        let mut ctx = MapCtx::new(&[("x", 0)]);
        let expr = HandlerExpr::Assign {
            lhs: "x".into(),
            rhs: Box::new(HandlerExpr::IntLit(1)),
        };
        assert_eq!(
            evaluate_binding(&expr, &mut ctx),
            Err(EvalError::WriteInBindingContext { path: "x".into() }),
        );
    }

    #[test]
    fn binding_rejects_compound_assign() {
        let mut ctx = MapCtx::new(&[("x", 5)]);
        let expr = HandlerExpr::CompoundAssign {
            lhs: "x".into(),
            op: CompoundOp::Add,
            rhs: Box::new(HandlerExpr::IntLit(1)),
        };
        assert_eq!(
            evaluate_binding(&expr, &mut ctx),
            Err(EvalError::WriteInBindingContext { path: "x".into() }),
        );
    }

    #[test]
    fn binding_rejects_write_nested_in_interpolation() {
        // A write expression hidden inside an interpolation part should also
        // be rejected at evaluation time.
        let mut ctx = MapCtx::new(&[("x", 0)]);
        let expr = HandlerExpr::Interpolation(vec![
            InterpolationPart::Literal("v=".into()),
            InterpolationPart::Expr(HandlerExpr::Assign {
                lhs: "x".into(),
                rhs: Box::new(HandlerExpr::IntLit(99)),
            }),
        ]);
        assert_eq!(
            evaluate_binding(&expr, &mut ctx),
            Err(EvalError::WriteInBindingContext { path: "x".into() }),
        );
        // The value must be unchanged.
        assert_eq!(ctx.get("x"), 0);
    }

    #[test]
    fn evaluate_rejects_str_lit_in_handler_context() {
        let mut ctx = MapCtx::new(&[]);
        let expr = HandlerExpr::StrLit("oops".into());
        assert!(matches!(
            evaluate(&expr, &mut ctx),
            Err(EvalError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn evaluate_rejects_interpolation_in_handler_context() {
        let mut ctx = MapCtx::new(&[]);
        let expr = HandlerExpr::Interpolation(vec![InterpolationPart::Literal("x".into())]);
        assert!(matches!(
            evaluate(&expr, &mut ctx),
            Err(EvalError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn evaluate_rejects_str_prop_read_in_handler_context() {
        let mut ctx = MapCtx::new(&[]).with_strings(&[("root.label", "Ready")]);
        let expr = HandlerExpr::StrPropRead {
            path: "root.label".into(),
        };
        assert!(matches!(
            evaluate(&expr, &mut ctx),
            Err(EvalError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn evaluate_rejects_collection_forms_until_t7_writer_lands() {
        let mut ctx = MapCtx::new(&[]);
        let exprs = [
            HandlerExpr::ListPropRead {
                path: "xs".into(),
                elem: wasamo_ir::IrType::I32,
            },
            HandlerExpr::ItemRead {
                binder: "item".into(),
            },
            HandlerExpr::IndexRead { binder: "i".into() },
            HandlerExpr::ListAppend {
                path: "xs".into(),
                elem: wasamo_ir::IrType::I32,
                value: Box::new(HandlerExpr::IntLit(1)),
            },
            HandlerExpr::ListDropLast {
                path: "xs".into(),
                elem: wasamo_ir::IrType::I32,
            },
            HandlerExpr::ListLit(vec![wasamo_ir::IrLiteral::Int(1)]),
        ];

        for expr in exprs {
            assert!(matches!(
                evaluate(&expr, &mut ctx),
                Err(EvalError::TypeMismatch { .. })
            ));
        }
    }

    // ── Bool surface tests (M3-Phase 1 T7) ───────────────────────────────────

    /// Default trait impl: a context without bool support reports
    /// `UnknownProperty`, paralleling the `get_string` default.
    #[test]
    fn eval_context_default_get_bool_is_unknown() {
        struct OnlyI32;
        impl EvalContext for OnlyI32 {
            fn get_i32(&self, _: &str) -> Result<i32, EvalError> {
                Ok(0)
            }
            fn set_i32(&mut self, _: &str, _: i32) -> Result<(), EvalError> {
                Ok(())
            }
        }
        let ctx = OnlyI32;
        assert_eq!(
            ctx.get_bool("ready"),
            Err(EvalError::UnknownProperty("ready".into()))
        );
        assert_eq!(
            ctx.read_bool_tracked("ready"),
            Err(EvalError::UnknownProperty("ready".into()))
        );
    }

    #[test]
    fn eval_context_default_set_bool_is_unknown() {
        struct OnlyI32;
        impl EvalContext for OnlyI32 {
            fn get_i32(&self, _: &str) -> Result<i32, EvalError> {
                Ok(0)
            }
            fn set_i32(&mut self, _: &str, _: i32) -> Result<(), EvalError> {
                Ok(())
            }
        }
        let mut ctx = OnlyI32;
        assert_eq!(
            ctx.set_bool("ready", true),
            Err(EvalError::UnknownProperty("ready".into()))
        );
    }

    /// Default `read_bool_tracked` forwards to `get_bool`. Overriding
    /// `get_bool` is enough for the default tracking shim to surface the
    /// value.
    #[test]
    fn read_bool_tracked_default_forwards_to_get_bool() {
        let ctx = MapCtx::new(&[]).with_bools(&[("ready", true)]);
        assert_eq!(ctx.read_bool_tracked("ready"), Ok(true));
    }

    #[test]
    fn assign_bool_lit_writes_through_set_bool() {
        let mut ctx = MapCtx::new(&[]).with_bools(&[("ready", true)]);
        let expr = HandlerExpr::Assign {
            lhs: "ready".into(),
            rhs: Box::new(HandlerExpr::BoolLit(false)),
        };
        assert_eq!(evaluate(&expr, &mut ctx), Ok(0));
        assert_eq!(ctx.get_b("ready"), false);
    }

    #[test]
    fn assign_bool_prop_read_copies_value() {
        // ready = other  where other = true
        let mut ctx = MapCtx::new(&[]).with_bools(&[("ready", false), ("other", true)]);
        let expr = HandlerExpr::Assign {
            lhs: "ready".into(),
            rhs: Box::new(HandlerExpr::BoolPropRead {
                path: "other".into(),
            }),
        };
        assert_eq!(evaluate(&expr, &mut ctx), Ok(0));
        assert_eq!(ctx.get_b("ready"), true);
        // Source untouched.
        assert_eq!(ctx.get_b("other"), true);
    }

    #[test]
    fn assign_bool_prop_read_unknown_source_propagates_error() {
        let mut ctx = MapCtx::new(&[]).with_bools(&[("ready", false)]);
        let expr = HandlerExpr::Assign {
            lhs: "ready".into(),
            rhs: Box::new(HandlerExpr::BoolPropRead {
                path: "missing".into(),
            }),
        };
        assert_eq!(
            evaluate(&expr, &mut ctx),
            Err(EvalError::UnknownProperty("missing".into()))
        );
        // Target unchanged.
        assert_eq!(ctx.get_b("ready"), false);
    }

    #[test]
    fn invoke_handler_drives_bool_assign() {
        // End-to-end via `invoke_handler` (the path inline click handlers
        // take): `Button { on click { ready = false } }` shape.
        let mut ctx = MapCtx::new(&[]).with_bools(&[("ready", true)]);
        let expr = HandlerExpr::Assign {
            lhs: "ready".into(),
            rhs: Box::new(HandlerExpr::BoolLit(false)),
        };
        let ok = invoke_handler(&expr, &mut ctx, "Demo.button.clicked");
        assert!(ok);
        assert_eq!(ctx.get_b("ready"), false);
    }

    /// Bare bool literal in handler (integer) context is still a type
    /// mismatch — only the `Assign { rhs: BoolLit | BoolPropRead }`
    /// shape is admitted (ADR §Out of scope: no implicit bool→i32).
    #[test]
    fn evaluate_rejects_bare_bool_lit_in_handler_context() {
        let mut ctx = MapCtx::new(&[]);
        let expr = HandlerExpr::BoolLit(true);
        assert!(matches!(
            evaluate(&expr, &mut ctx),
            Err(EvalError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn evaluate_rejects_bare_bool_prop_read_in_handler_context() {
        let mut ctx = MapCtx::new(&[]).with_bools(&[("ready", true)]);
        let expr = HandlerExpr::BoolPropRead {
            path: "ready".into(),
        };
        assert!(matches!(
            evaluate(&expr, &mut ctx),
            Err(EvalError::TypeMismatch { .. })
        ));
    }

    /// `CompoundAssign` over bool is out of scope (no naturally bool-typed
    /// `CompoundOp`). The bool rhs of a CompoundAssign falls through the
    /// existing i32 path, where the bare BoolLit/BoolPropRead arm rejects
    /// it as a TypeMismatch — proving compound-bool is not silently
    /// admitted.
    #[test]
    fn evaluate_rejects_compound_assign_with_bool_rhs() {
        let mut ctx = MapCtx::new(&[("x", 1)]);
        let expr = HandlerExpr::CompoundAssign {
            lhs: "x".into(),
            op: CompoundOp::Add,
            rhs: Box::new(HandlerExpr::BoolLit(true)),
        };
        assert!(matches!(
            evaluate(&expr, &mut ctx),
            Err(EvalError::TypeMismatch { .. })
        ));
        // Target unchanged.
        assert_eq!(ctx.get("x"), 1);
    }

    // ── evaluate_bool_binding tests (M3-Phase 1 T8 / DD-M3-P1-007) ──────────

    #[test]
    fn bool_binding_accepts_bool_lit_true() {
        let mut ctx = MapCtx::new(&[]);
        assert_eq!(
            evaluate_bool_binding(&HandlerExpr::BoolLit(true), &mut ctx),
            Ok(true)
        );
    }

    #[test]
    fn bool_binding_accepts_bool_lit_false() {
        let mut ctx = MapCtx::new(&[]);
        assert_eq!(
            evaluate_bool_binding(&HandlerExpr::BoolLit(false), &mut ctx),
            Ok(false)
        );
    }

    #[test]
    fn bool_binding_accepts_bool_prop_read() {
        let mut ctx = MapCtx::new(&[]).with_bools(&[("ready", true)]);
        let expr = HandlerExpr::BoolPropRead {
            path: "ready".into(),
        };
        assert_eq!(evaluate_bool_binding(&expr, &mut ctx), Ok(true));
    }

    #[test]
    fn bool_binding_bool_prop_read_unknown_is_unknown_property() {
        let mut ctx = MapCtx::new(&[]);
        let expr = HandlerExpr::BoolPropRead {
            path: "missing".into(),
        };
        assert_eq!(
            evaluate_bool_binding(&expr, &mut ctx),
            Err(EvalError::UnknownProperty("missing".into()))
        );
    }

    #[test]
    fn bool_binding_rejects_int_lit() {
        let mut ctx = MapCtx::new(&[]);
        assert!(matches!(
            evaluate_bool_binding(&HandlerExpr::IntLit(1), &mut ctx),
            Err(EvalError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn bool_binding_rejects_prop_read() {
        let mut ctx = MapCtx::new(&[("root.count", 1)]);
        let expr = HandlerExpr::PropRead {
            path: "root.count".into(),
        };
        assert!(matches!(
            evaluate_bool_binding(&expr, &mut ctx),
            Err(EvalError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn bool_binding_rejects_str_lit() {
        let mut ctx = MapCtx::new(&[]);
        let expr = HandlerExpr::StrLit("true".into());
        assert!(matches!(
            evaluate_bool_binding(&expr, &mut ctx),
            Err(EvalError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn bool_binding_rejects_str_prop_read() {
        let mut ctx = MapCtx::new(&[]).with_strings(&[("root.label", "true")]);
        let expr = HandlerExpr::StrPropRead {
            path: "root.label".into(),
        };
        assert!(matches!(
            evaluate_bool_binding(&expr, &mut ctx),
            Err(EvalError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn bool_binding_rejects_interpolation() {
        let mut ctx = MapCtx::new(&[]);
        let expr = HandlerExpr::Interpolation(vec![InterpolationPart::Literal("true".into())]);
        assert!(matches!(
            evaluate_bool_binding(&expr, &mut ctx),
            Err(EvalError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn bool_binding_rejects_assign() {
        let mut ctx = MapCtx::new(&[]).with_bools(&[("ready", false)]);
        let expr = HandlerExpr::Assign {
            lhs: "ready".into(),
            rhs: Box::new(HandlerExpr::BoolLit(true)),
        };
        assert!(matches!(
            evaluate_bool_binding(&expr, &mut ctx),
            Err(EvalError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn bool_binding_rejects_block() {
        let mut ctx = MapCtx::new(&[]);
        let expr = HandlerExpr::Block(vec![HandlerExpr::BoolLit(true)]);
        assert!(matches!(
            evaluate_bool_binding(&expr, &mut ctx),
            Err(EvalError::TypeMismatch { .. })
        ));
    }

    /// Existing i32 `Assign` arm still works — the bool peek does not
    /// regress the M2 path.
    #[test]
    fn assign_i32_lit_still_works_after_bool_arm() {
        let mut ctx = MapCtx::new(&[("count", 0)]);
        let expr = HandlerExpr::Assign {
            lhs: "count".into(),
            rhs: Box::new(HandlerExpr::IntLit(9)),
        };
        assert_eq!(evaluate(&expr, &mut ctx), Ok(9));
        assert_eq!(ctx.get("count"), 9);
    }
}
