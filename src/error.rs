use super::parser::ParserError;
use super::scanner::LexError;

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
