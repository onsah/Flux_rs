use super::Expr;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Expr(Expr),
    Let {
        name: String, // TODO: pattern matching with tuples
        value: Expr,
    },
    // Declaration
    Var {
        name: String,
        value: Expr,
    },
    // Set declaration
    Set {
        variable: Expr,
        value: Expr,
    },
    Block(Vec<Statement>),
    If {
        condition: Expr,
        then_block: Box<Expr>,         // block_expr
        else_block: Option<Box<Expr>>, // block_expr
    },
    While {
        condition: Expr,
        then_block: Box<Statement>,
    },
    Print(Expr),
    Return(Expr),
    Import {
        path: Vec<String>,
        name: String,
    },
}

impl Statement {
    pub fn can_convert_expr(&self) -> bool {
        match self {
            Statement::Expr(expr) => match expr {
                Expr::If { .. } | Expr::Function { .. } | Expr::Block(..) => true,
                _ => false,
            },
            Statement::If { else_block, .. } => else_block.is_some(),
            _ => false,
        }
    }

    pub fn into_expr(self) -> Option<Expr> {
        match self {
            Statement::Expr(expr) => Some(expr),
            Statement::If {
                condition,
                then_block,
                else_block,
            } => else_block.map(|else_block| Expr::If {
                condition: Box::new(condition),
                then_block,
                else_block,
            }),
            _ => None,
        }
    }
}
