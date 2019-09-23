use super::Expr;
use crate::scanner::{LexError, LexErrorKind, Token, TokenType};
use std::fmt::{Display, Formatter};

// TODO: ParserErrorKind and ParserError

#[derive(Clone, Debug, PartialEq)]
pub struct ParserError {
    pub kind: ParserErrorKind,
    pub line: usize
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParserErrorKind {
    ExpectedToken,
    UnexpectedToken { token: Token },
    NotMatched { typ: TokenType },
    // mixing array and table initialization
    // Ex: let t = { 3, foo = 5 }
    InitError,
    UnexpectedExpr(Expr),
    Lex(LexErrorKind),
    ReservedIdentifier(String),
    Redeclaration { name: String },
    Undeclared { name: String },
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Parsing Error: {:?}", self.line, self.kind)
    }
}

impl From<LexError> for ParserError {
    fn from(lex_error: LexError) -> Self {
        ParserError {
            kind: ParserErrorKind::Lex(lex_error.kind),
            line: lex_error.line,
        }
    }
}