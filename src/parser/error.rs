use super::{Expr, Statement};
use crate::scanner::{Token, TokenType};

#[derive(Clone, Debug, PartialEq)]
pub enum ParserError {
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
}
