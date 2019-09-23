use std::collections::HashMap;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum TokenType {
    /* Primitives */
    String,
    Number,
    True,
    False,
    Nil,

    Identifier,

    LeftParen,
    RightParen,
    LeftCurly,
    RightCurly,
    LeftBracket,
    RightBracket,

    /* Single char tokens */
    Dot,
    Comma,
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    Bang,
    Greater,
    Less,
    Question,
    Semicolon,
    Colon,

    /* Double char tokens */
    EqualEqual,
    BangEqual,
    GreaterEqual,
    LessEqual,
    RightArrow, // '=>'

    /* Keywords */
    Let,
    If,
    Else,
    While,
    For,
    Return,
    Break,
    Fn,
    Then,
    Do,
    End,
    Print,
    Import,
    As,
    In,
    Var,

    Eof,
    Invalid,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub(super) typ: TokenType,
    pub(super) text: String,
    pub(super) line: usize,
}

lazy_static! {
    pub static ref KEYWORDS: HashMap<&'static str, TokenType> = hashmap! {
        "nil" => TokenType::Nil,
        "true" => TokenType::True,
        "false" => TokenType::False,

        "let" => TokenType::Let,
        "if" => TokenType::If,
        "else" => TokenType::Else,
        "while" => TokenType::While,
        "for" => TokenType::For,
        "return" => TokenType::Return,
        "break" => TokenType::Break,
        "fn" => TokenType::Fn,
        "then" => TokenType::Then,
        "do" => TokenType::Do,
        "end" => TokenType::End,
        "import" => TokenType::Import,
        "as" => TokenType::As,
        "in" => TokenType::In,
        "var" => TokenType::Var,
        // "print" => TokenType::Print,
    };
}

impl Token {
    pub fn is_invalid(&self) -> bool {
        self.typ == TokenType::Invalid
    }

    pub fn get_type(&self) -> TokenType {
        self.typ
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn extract_text(self) -> String {
        self.text
    }

    pub fn get_line(&self) -> usize {
        self.line
    }
}

impl Default for Token {
    fn default() -> Self {
        Token {
            typ: TokenType::Invalid,
            text: String::new(),
            line: 0,
        }
    }
}
