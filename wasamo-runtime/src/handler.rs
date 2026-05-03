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
}
