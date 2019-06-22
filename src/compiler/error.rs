pub use crate::parser::Expr;

#[derive(Clone, Debug)]
pub enum CompileError {
    TooManyConstants,
    UnimplementedExpr(Expr),
    UndefinedVariable { name: String },
    InvalidAssignmentTarget(Expr),
}