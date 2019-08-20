use super::{Expr, Statement};
use crate::scanner::{Token, TokenType};
use std::fmt::{Display, Formatter};

// TODO: ParserErrorKind and ParserError

#[derive(Clone, Debug, PartialEq)]
pub struct ParserError {
    pub(super) kind: ParserErrorKind,
    pub(super) line: usize
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParserErrorKind {
    ExpectedToken,
    UnexpectedToken { token: Token },
    NotMatched { typ: TokenType },
    // mixing array and table initialization
    // Ex: let t = { 3, foo = 5 }
    InitError,
    // Expectes return statement or an expression at the end of the block
    ExprOrReturn,
    UnexpectedStmt(Statement),
    UnexpectedExpr(Expr),
    ExpectedMethod,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Parsing Error: {:?}", self.line, self.kind)
    }
}