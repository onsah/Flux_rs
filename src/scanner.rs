mod error;
mod token;

pub use error::LexError;
use std::iter::Peekable;
use std::str::CharIndices;
use token::KEYWORDS;
pub use token::{Token, TokenType};

type Result<T> = std::result::Result<T, LexError>;

pub struct Scanner<'a> {
    source: &'a str,
    chars: Peekable<CharIndices<'a>>,
    line: usize,
    tokens: Vec<Token>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source,
            chars: source.char_indices().peekable(),
            line: 1,
            tokens: Vec::new(),
        }
    }

    pub fn extract_tokens(self) -> Vec<Token> {
        self.tokens
    }

    pub fn scan(&mut self) -> Result<&Vec<Token>> {
        loop {
            match self.scan_next() {
                Ok(token) => self.tokens.push(token),
                Err(e) => match e {
                    LexError::Eof => {
                        self.tokens.push(self.new_token(TokenType::Eof, 0, 0));
                        return Ok(&self.tokens);
                    }
                    _ => return Err(e),
                },
            }
            // println!("scanned: {:?}", self.tokens.last().unwrap());
        }
    }

    fn scan_next(&mut self) -> Result<Token> {
        loop {
            let (start, c) = self.advance()?;
            match c {
                '.' => return Ok(self.new_token(TokenType::Dot, start, start + 1)),
                ',' => return Ok(self.new_token(TokenType::Comma, start, start + 1)),
                ';' => return Ok(self.new_token(TokenType::Semicolon, start, start + 1)),
                ':' => return Ok(self.new_token(TokenType::Colon, start, start + 1)),
                '?' => return Ok(self.new_token(TokenType::Question, start, start + 1)),

                '(' => return Ok(self.new_token(TokenType::LeftParen, start, start + 1)),
                ')' => return Ok(self.new_token(TokenType::RightParen, start, start + 1)),
                '{' => return Ok(self.new_token(TokenType::LeftCurly, start, start + 1)),
                '}' => return Ok(self.new_token(TokenType::RightCurly, start, start + 1)),
                '[' => return Ok(self.new_token(TokenType::LeftBracket, start, start + 1)),
                ']' => return Ok(self.new_token(TokenType::RightBracket, start, start + 1)),

                '+' => return Ok(self.new_token(TokenType::Plus, start, start + 1)),
                '-' => return Ok(self.new_token(TokenType::Minus, start, start + 1)),
                '*' => return Ok(self.new_token(TokenType::Star, start, start + 1)),
                '/' => return Ok(self.new_token(TokenType::Slash, start, start + 1)),

                '=' => match self.peek() {
                    '=' => {
                        let (end, _) = self.advance().unwrap();
                        return Ok(self.new_token(TokenType::EqualEqual, start, end));
                    }
                    '>' => {
                        let (end, _) = self.advance().unwrap();
                        return Ok(self.new_token(TokenType::RightArrow, start, end));
                    }
                    _ => return Ok(self.new_token(TokenType::Equal, start, start + 1)),
                },
                '!' => {
                    return self.double_char_token(
                        TokenType::Bang,
                        TokenType::BangEqual,
                        '=',
                        start,
                    )
                }
                '>' => {
                    return self.double_char_token(
                        TokenType::Greater,
                        TokenType::GreaterEqual,
                        '=',
                        start,
                    )
                }
                '<' => {
                    return self.double_char_token(
                        TokenType::Less,
                        TokenType::LessEqual,
                        '=',
                        start,
                    )
                }

                '\"' => return self.string(start + 1),

                ' ' | '\t' => {}
                '\n' => self.line += 1,
                c => {
                    if c.is_alphabetic() {
                        let token = self.identifier(start)?;
                        match KEYWORDS.get(token.text.as_str()) {
                            Some(&typ) => {
                                return Ok(Token {
                                    typ,
                                    text: token.text,
                                    line: token.line,
                                })
                            }
                            None => return Ok(token),
                        }
                    } else if c.is_numeric() {
                        return self.number(start);
                    } else {
                        return Err(LexError::InvalidChar {
                            ch: c,
                            line: self.line,
                        });
                    }
                }
            }
        }
    }

    fn string(&mut self, start: usize) -> Result<Token> {
        let end = loop {
            match self.match_char('\"') {
                Ok((i, _)) => break i,
                Err(LexError::UnexpectedChar { .. }) => {
                    self.advance()?;
                }
                Err(err) => return Err(err),
            }
        };
        Ok(self.new_token(TokenType::String, start, end))
    }

    fn identifier(&mut self, start: usize) -> Result<Token> {
        let end = self.skip_while(start, |c| c.is_alphanumeric());
        Ok(self.new_token(TokenType::Identifier, start, end + 1))
    }

    fn number(&mut self, start: usize) -> Result<Token> {
        let end = self.skip_while(start, |c| c.is_numeric());
        match self.peek() {
            '.' => {
                self.advance().unwrap();
                let end = self.skip_while(end + 1, |c| c.is_numeric());
                Ok(self.new_token(TokenType::Number, start, end + 1))
            }
            _ => Ok(self.new_token(TokenType::Number, start, end + 1)),
        }
    }

    #[inline]
    fn double_char_token(
        &mut self,
        single_type: TokenType,
        double_type: TokenType,
        second_char: char,
        start: usize,
    ) -> Result<Token> {
        match self.match_char(second_char) {
            Ok((end, _)) => Ok(self.new_token(double_type, start, end)),
            Err(LexError::UnexpectedChar { .. }) => {
                Ok(self.new_token(single_type, start, start + 1))
            }
            Err(err) => Err(err),
        }
    }

    fn advance(&mut self) -> Result<(usize, char)> {
        match self.chars.next() {
            Some(t) => Ok(t),
            None => Err(LexError::Eof),
        }
    }

    fn match_char(&mut self, pred: char) -> Result<(usize, char)> {
        match self.chars.peek() {
            Some(&(_, c)) => {
                if c == pred {
                    self.advance()
                } else {
                    Err(LexError::UnexpectedChar { line: self.line })
                }
            }
            None => Err(LexError::TooShort { line: self.line }),
        }
    }

    fn match_pred(&mut self, pred: impl Fn(char) -> bool) -> Result<(usize, char)> {
        match self.chars.peek() {
            Some(&(_, c)) => {
                if pred(c) {
                    self.advance()
                } else {
                    Err(LexError::UnexpectedChar { line: self.line })
                }
            }
            None => Err(LexError::TooShort { line: self.line }),
        }
    }

    fn skip_while<F>(&mut self, start: usize, pred: F) -> usize
    where
        F: Fn(char) -> bool + Copy,
    {
        let mut result = start;
        while self.match_pred(pred).is_ok() {
            result += 1
        }
        result
    }

    fn peek(&mut self) -> char {
        self.chars.peek().map(|(_, c)| *c).unwrap_or(' ')
    }

    fn new_token(&self, typ: TokenType, start: usize, end: usize) -> Token {
        Token {
            typ,
            text: self.source[start..end].to_string(),
            line: self.line,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::token::{Token, TokenType};
    use super::Scanner;

    #[test]
    fn string_works() {
        let source = "\"abcd\"";
        let mut scanner = Scanner::new(source);
        let token = scanner.scan_next().unwrap();
        assert_eq!(
            token,
            Token {
                typ: TokenType::String,
                text: "abcd".to_string(),
                line: 1,
            }
        );
    }

    #[test]
    fn number_works() {
        let source = " 43.23ab";
        let mut scanner = Scanner::new(source);
        let token = scanner.scan_next().unwrap();
        assert_eq!(
            token,
            Token {
                typ: TokenType::Number,
                text: "43.23".to_string(),
                line: 1,
            }
        );
    }

    #[test]
    fn ident_works() {
        let source = " \nvariable";
        let mut scanner = Scanner::new(source);
        let token = scanner.scan_next().unwrap();
        assert_eq!(
            token,
            Token {
                typ: TokenType::Identifier,
                text: "variable".to_string(),
                line: 2,
            }
        )
    }

    #[test]
    fn keyword_works() {
        let source = " while ";
        let mut scanner = Scanner::new(source);
        let token = scanner.scan_next().unwrap();
        assert_eq!(
            token,
            Token {
                typ: TokenType::While,
                text: "while".to_string(),
                line: 1,
            }
        );
    }
}
