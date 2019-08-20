use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug)]
pub struct LexError {
    pub(super) kind: LexErrorKind,
    pub(super) line: usize,
}

#[derive(Copy, Clone, Debug)]
pub enum LexErrorKind {
    // Expected different char
    UnexpectedChar(char),
    // source is unfinished
    TooShort,
    // Invalid character
    InvalidChar(char),
    Eof,
}

impl Display for LexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Lex Error: {:?}", self.line, self.kind)
    }
}