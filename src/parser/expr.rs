use crate::scanner::TokenType;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Tuple(Vec<Expr>),
    Access {
        table: Box<Expr>,
        field: Box<Expr>,
    },
    Set {
        variable: Box<Expr>,
        value: Box<Expr>,
    },
    TableInit {
        keys: Option<Vec<Expr>>,
        values: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Str(String),
    Number(f64),
    Bool(bool),
    Nil,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UnaryOp {
    Minus,
    Bang,
}

impl From<TokenType> for UnaryOp {
    fn from(typ: TokenType) -> UnaryOp {
        match typ {
            TokenType::Minus => UnaryOp::Minus,
            TokenType::Bang => UnaryOp::Bang,
            _ => panic!("Unexpected type to convert to UnaryOp: {:?}", typ),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BinaryOp {
    Plus,
    Minus,
    Star,
    Slash,
    Greater,
    Less,
    EqualEqual,
    BangEqual,
    GreaterEqual,
    LessEqual,
}

impl From<TokenType> for BinaryOp {
    fn from(typ: TokenType) -> BinaryOp {
        match typ {
            TokenType::Plus => BinaryOp::Plus,
            TokenType::Minus => BinaryOp::Minus,
            TokenType::Star => BinaryOp::Star,
            TokenType::Slash => BinaryOp::Slash,
            TokenType::Greater => BinaryOp::Greater,
            TokenType::Less => BinaryOp::Less,
            TokenType::GreaterEqual => BinaryOp::GreaterEqual,
            TokenType::LessEqual => BinaryOp::LessEqual,
            TokenType::EqualEqual => BinaryOp::EqualEqual,
            TokenType::BangEqual => BinaryOp::BangEqual,
            _ => panic!("Unexpected type to convert to BinaryOp: {:?}", typ),
        }
    }
}
