use super::scanner::LexError;
use super::parser::ParserError;

pub type FluxResult<T> = std::result::Result<T, FluxError>;

#[derive(Debug, Clone)]
pub enum FluxError {
    ParserError(ParserError),
    LexError(LexError),
}

impl From<ParserError> for FluxError {
    fn from(error: ParserError) -> Self {
        FluxError::ParserError(error)
    }
}

impl From<LexError> for FluxError {
    fn from(error: LexError) -> Self {
        FluxError::LexError(error)
    }
}