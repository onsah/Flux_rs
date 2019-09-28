use super::Statement;
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
    SelfAccess {
        table: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    TableInit {
        keys: Option<Vec<Expr>>,
        values: Vec<Expr>,
    },
    Function {
        args: Vec<String>,
        body: BlockExpr,
        /** (keys, values) */
        env: Option<(Vec<Expr>, Vec<Expr>)>,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    Block(BlockExpr),
    If {
        condition: Box<Expr>,
        then_block: Box<Expr>,
        // Else is mandatory when if is expression
        else_block: Box<Expr>,
    },
    Rec,    // Function calls itself
}

// TODO: use these function instead of manually wrapping
impl Expr {
    pub fn string(s: String) -> Self {
        Expr::Literal(Literal::Str(s))
    }

    pub fn integer(i: f64) -> Self {
        Expr::Literal(Literal::Number(i))
    }

    pub fn bool(b: bool) -> Self {
        Expr::Literal(Literal::Bool(b))
    }

    pub fn unit() -> Self {
        Expr::Literal(Literal::Unit)
    }

    pub fn nil() -> Self {
        Expr::Literal(Literal::Nil)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockExpr {
    pub stmts: Vec<Statement>,
    pub expr: Box<Expr>,
}

impl Into<Expr> for BlockExpr {
    fn into(self) -> Expr {
        Expr::Block(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Str(String),
    Number(f64),
    Bool(bool),
    Unit,
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
