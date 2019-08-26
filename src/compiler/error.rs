use crate::compiler::Instruction;
use crate::parser::{ParserError, Expr};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    TooManyConstants,
    UnimplementedExpr(Expr),
    UndefinedVariable { name: String },
    InvalidAssignmentTarget(Expr),
    WrongPatch(Instruction),
    TooLongToJump,
    Parse(ParserError),
    IoError(io::ErrorKind),
}

impl From<ParserError> for CompileError {
    fn from(pe: ParserError) -> Self {
        CompileError::Parse(pe)
    }
}