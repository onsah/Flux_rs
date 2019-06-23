use crate::compiler::Instruction;
use crate::parser::Expr;

#[derive(Clone, Debug)]
pub enum CompileError {
    TooManyConstants,
    UnimplementedExpr(Expr),
    UndefinedVariable { name: String },
    InvalidAssignmentTarget(Expr),
    WrongPatch(Instruction),
    TooLongToJump,
}
