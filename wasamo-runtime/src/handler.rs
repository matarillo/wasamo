//! Runtime-side DSL inline handler evaluator (DD-M2-P3-001 = Option A).
//!
//! `HandlerExpr` is the in-memory IR for handler bodies. Textual IR ↔
//! `HandlerExpr` translation is Phase 6 work; this module is handler-only
//! evaluation. The binding expression evaluator (Phase 5) will share the
//! core once it is built on top of this foundation.

/// A single expression node in a DSL handler body.
///
/// M2 scope: the subset of expression forms that appear in `counter.ui`.
/// Phase 5 extends this with the read-only binding expression forms.
#[derive(Debug, Clone)]
pub enum HandlerExpr {
    /// Integer literal.
    IntLit(i32),

    /// Read a named property from the evaluation context.
    /// `path` is a dot-separated widget-path + property name, e.g. `"root.count"`.
    PropRead { path: String },

    /// Assign a value to a property: `lhs = rhs`.
    Assign { lhs: String, rhs: Box<HandlerExpr> },

    /// Compound-assign: `lhs op= rhs`.
    CompoundAssign { lhs: String, op: CompoundOp, rhs: Box<HandlerExpr> },

    /// A sequential block of statements; the value of the last expression is
    /// the value of the block (unit / discarded for statement blocks).
    Block(Vec<HandlerExpr>),
}

/// Compound-assignment operators supported in M2 handler bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompoundOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// Evaluation context: property read / write access for a specific component.
///
/// Phase 5 will unify this with the binding evaluator context; for Phase 3 the
/// trait stays minimal (integer properties only, matching `counter.ui` needs).
pub trait EvalContext {
    /// Read an integer property by dot-separated path.
    fn get_i32(&self, path: &str) -> Result<i32, EvalError>;

    /// Write an integer property by dot-separated path.
    fn set_i32(&mut self, path: &str, value: i32) -> Result<(), EvalError>;
}

/// Errors that the evaluator can produce.
#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    UnknownProperty(String),
    TypeMismatch { path: String },
    DivisionByZero,
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::UnknownProperty(p) => write!(f, "unknown property: {p}"),
            EvalError::TypeMismatch { path } => write!(f, "type mismatch at: {path}"),
            EvalError::DivisionByZero => write!(f, "division by zero"),
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

        HandlerExpr::PropRead { path } => ctx.get_i32(path),

        HandlerExpr::Assign { lhs, rhs } => {
            let v = evaluate(rhs, ctx)?;
            ctx.set_i32(lhs, v)?;
            Ok(v)
        }

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
        Self { name: name.into(), index: None }
    }
    pub fn indexed(name: impl Into<String>, index: usize) -> Self {
        Self { name: name.into(), index: Some(index) }
    }
}

/// Invoke a `HandlerExpr` against `ctx`, catching any panic that the evaluator
/// might raise (DD-M2-P3-003 = Option A). On error or panic, logs one line to
/// stderr in the form:
/// `wasamo: handler error in <location>: <message>`
/// where `location` is a caller-supplied coarse identifier
/// (see `format_handler_location` in this module).
/// Returns `true` if the handler completed without error.
pub fn invoke_handler(
    expr: &HandlerExpr,
    ctx: &mut dyn EvalContext,
    location: &str,
) -> bool {
    // RefUnwindSafe is not automatically satisfied for trait objects, so we
    // evaluate inside a wrapper that AssertUnwindSafe asserts the invariant.
    // The safety argument: `ctx` releases any interior borrows before this
    // call (see DD-M2-P3-003 risk note); no RefCell is live across the boundary.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        evaluate(expr, ctx)
    }));
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

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Simple in-memory context for unit testing.
    struct MapCtx(HashMap<String, i32>);

    impl MapCtx {
        fn new(pairs: &[(&str, i32)]) -> Self {
            Self(pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect())
        }
        fn get(&self, key: &str) -> i32 {
            *self.0.get(key).unwrap_or(&0)
        }
    }

    impl EvalContext for MapCtx {
        fn get_i32(&self, path: &str) -> Result<i32, EvalError> {
            self.0
                .get(path)
                .copied()
                .ok_or_else(|| EvalError::UnknownProperty(path.to_string()))
        }
        fn set_i32(&mut self, path: &str, value: i32) -> Result<(), EvalError> {
            self.0.insert(path.to_string(), value);
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
        let expr = HandlerExpr::PropRead { path: "root.count".into() };
        assert_eq!(evaluate(&expr, &mut ctx), Ok(7));
    }

    #[test]
    fn prop_read_unknown() {
        let mut ctx = MapCtx::new(&[]);
        let expr = HandlerExpr::PropRead { path: "root.count".into() };
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
            HandlerExpr::PropRead { path: "root.count".into() },
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
        let loc = format_handler_location(
            "Counter",
            &[WidgetPathSegment::named("button")],
            "clicked",
        );
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
        let loc = format_handler_location(
            "?",
            &[WidgetPathSegment::named("button")],
            "clicked",
        );
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
            fn get_i32(&self, _: &str) -> Result<i32, EvalError> { Ok(0) }
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
}
