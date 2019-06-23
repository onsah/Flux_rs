use super::Expr;

#[derive(Clone, Debug)]
pub enum Statement {
    Expr(Expr),
    Let {
        name: String, // TODO: pattern matching with tuples
        value: Expr,
    },
    Block(Vec<Statement>),
    If {
        condition: Expr,
        then_block: Box<Statement>,
        else_block: Option<Box<Statement>>,
    },
    While {
        condition: Expr,
        then_block: Box<Statement>,
    },
}
