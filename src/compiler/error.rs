use crate::compiler::Instruction;
use crate::parser::{Expr, ParserError};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    TooManyConstants,
    UnimplementedExpr(Expr),
    UndefinedVariable {
        name: String,
    },
    InvalidAssignmentTarget(Expr),
    WrongPatch(Instruction),
    TooLongToJump,
    Parse(ParserError),
    IoError(io::ErrorKind),
    ModuleError {
        name: String,
        error: Box<CompileError>,
    },
}

impl From<ParserError> for CompileError {
    fn from(pe: ParserError) -> Self {
        CompileError::Parse(pe)
    }
}
