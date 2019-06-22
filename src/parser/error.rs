use crate::scanner::{Token, TokenType};

#[derive(Clone, Debug, PartialEq)]
pub enum ParserError {
    ExpectedToken,
    UnexpectedToken { token: Token },
    NotMatched { typ: TokenType },
    // mixing array and table initialization
    // Ex: let t = { 3, foo = 5 } 
    InitError,
}
