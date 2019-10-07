use super::Result;
use super::{ParserError, ParserErrorKind, Token, TokenType};

const LOOKAHEAD_SIZE: usize = 3;

// Lol it turns out my syntax doesn't need lookahead :)
pub struct LookAhead<I>
where
    I: Iterator<Item = Token>,
{
    token_buffer: I,
    lookahead: [Token; LOOKAHEAD_SIZE],
    lookahead_index: usize,
}

impl<I> LookAhead<I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(mut buffer: I) -> Self {
        let mut lookahead = [Token::default(), Token::default(), Token::default()];
        lookahead[0] = buffer.next().unwrap();
        LookAhead {
            token_buffer: buffer,
            lookahead,
            lookahead_index: 0,
        }
    }

    pub(super) fn advance(&mut self) -> Result<Token> {
        match self.token_buffer.next() {
            Some(token) => {
                self.lookahead_insert(token.clone());
                self.current()
            }
            None => Err(self.make_error(ParserErrorKind::ExpectedToken)?),
        }
    }

    // Can be used both for matching and for asserting tokens
    pub(super) fn match_token(&mut self, typ: TokenType) -> Result<Token> {
        let current = self.current()?.clone();
        let next_type = current.get_type();
        if next_type == typ {
            self.advance()?;
            Ok(current)
        } else {
            Err(self.make_error(ParserErrorKind::NotMatched { typ })?)
        }
    }

    fn lookahead_insert(&mut self, token: Token) {
        let next_index = (self.lookahead_index + 1) % LOOKAHEAD_SIZE;
        self.lookahead[next_index] = token;
        self.lookahead_index = next_index;
    }

    pub(super) fn current(&self) -> Result<Token> {
        let i = self.lookahead_index;
        let token = self.lookahead[i].clone();
        let line = token.get_line();
        if token.is_invalid() {
            Err(ParserError {
                kind: ParserErrorKind::UnexpectedToken { token },
                line,
            })
        } else {
            // println!("current token: {:?}", token);
            Ok(token)
        }
    }

    #[allow(dead_code)]
    pub(super) fn peek_first(&self) -> Result<Token> {
        let i = (self.lookahead_index + 1) % LOOKAHEAD_SIZE;
        let token = self.lookahead[i].clone();
        println!("peek: {:?}", token);
        if token.is_invalid() {
            Err(self.make_error(ParserErrorKind::UnexpectedToken { token })?)
        } else {
            Ok(token)
        }
    }

    pub(super) fn make_error(&self, kind: ParserErrorKind) -> Result<ParserError> {
        Ok(ParserError {
            kind,
            line: self.current()?.get_line(),
        })
    }
}
