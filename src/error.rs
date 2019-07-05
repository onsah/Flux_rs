use super::scanner::LexError;
use super::parser::ParserError;
use super::compiler::CompileError;
use super::vm::RuntimeError;

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