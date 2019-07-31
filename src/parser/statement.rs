use super::Expr;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Expr(Expr),
    Let {
        name: String, // TODO: pattern matching with tuples
        value: Expr,
    },
    Block(Vec<Statement>),
    If {
        condition: Expr,
        then_block: Box<Statement>, // block_expr
        else_block: Option<Box<Statement>>, // block_expr
    },
    While {
        condition: Expr,
        then_block: Box<Statement>,
    },
    Print(Expr),
    Return(Expr),
}
