use super::compiler::CompileError;
use super::parser::ParserError;
use super::scanner::LexError;
use super::vm::RuntimeError;
use std::fmt::{Display, Formatter};

pub type FluxResult<T> = std::result::Result<T, FluxError>;

#[derive(Debug, Clone)]
pub enum FluxError {
    Lex(LexError),
    Parse(ParserError),
    Compile(CompileError),
    Runtime(RuntimeError),
}

impl From<LexError> for FluxError {
    fn from(error: LexError) -> Self {
        FluxError::Lex(error)
    }
}

impl From<ParserError> for FluxError {
    fn from(error: ParserError) -> Self {
        FluxError::Parse(error)
    }
}

impl From<CompileError> for FluxError {
    fn from(error: CompileError) -> Self {
        FluxError::Compile(error)
    }
}

impl From<RuntimeError> for FluxError {
    fn from(error: RuntimeError) -> Self {
        FluxError::Runtime(error)
    }
}

impl Display for FluxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            // TODO: format
            FluxError::Lex(l) => write!(f, "{}", l),
            FluxError::Compile(c) => write!(f, "{:?}", c),
            FluxError::Runtime(r) => write!(f, "{:?}", r),
            FluxError::Parse(c) => write!(f, "{}", c),
        }
    }
}