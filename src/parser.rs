mod error;
mod expr;
mod lookahead;
mod statement;

pub use super::scanner::{Token, TokenType};
use crate::error::FluxResult;
use crate::scanner::Scanner;
pub use error::ParserError;
pub use expr::{BinaryOp, Expr, Literal, UnaryOp};
use lookahead::LookAhead;
pub use statement::Statement;
use std::ops::{Deref, DerefMut};

type Result<'a, T> = std::result::Result<T, ParserError>;

pub struct Parser<I>
where
    I: Iterator<Item = Token>,
{
    lookahead: LookAhead<I>,
}

impl Parser<std::vec::IntoIter<Token>> {
    pub fn new(source: &str) -> FluxResult<Self> {
        let mut scanner = Scanner::new(source);
        scanner.scan()?;
        let lookahead = LookAhead::new(scanner.extract_tokens().into_iter());
        Ok(Parser { lookahead })
    }
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>,
{
    pub fn parse(&mut self) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();
        while self.current()?.get_type() != TokenType::Eof {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    pub fn statement(&mut self) -> Result<Statement> {
        if let Ok(_) = self.match_token(TokenType::Let) {
            self.let_stmt()
        } else if let Ok(_) = self.match_token(TokenType::If) {
            self.if_stmt()
        } else if let Ok(_) = self.match_token(TokenType::Do) {
            let stmt = self.block_stmt().map(|v| Statement::Block(v))?;
            self.match_token(TokenType::End)?;
            Ok(stmt)
        } else if let Ok(_) = self.match_token(TokenType::While) {
            self.while_stmt()
        } else {
            self.expr_stmt()
        }
    }

    fn let_stmt(&mut self) -> Result<Statement> {
        let token = self.match_token(TokenType::Identifier)?;
        let name = token.text();
        self.match_token(TokenType::Equal)?;
        let value = self.expression()?;
        Ok(Statement::Let {
            name: name.to_string(),
            value,
        })
    }

    fn if_stmt(&mut self) -> Result<Statement> {
        let condition = self.expression()?;
        self.match_token(TokenType::Then)?;
        let then_block = Statement::Block(self.block_stmt()?);
        if let Ok(_) = self.match_token(TokenType::Else) {
            let else_block = if let Ok(_) = self.match_token(TokenType::If) {
                self.if_stmt()?
            } else {
                let block = self.block_stmt()?;
                self.match_token(TokenType::End)?;
                Statement::Block(block)
            };
            Ok(Statement::If {
                condition,
                then_block: Box::new(then_block),
                else_block: Some(Box::new(else_block)),
            })
        } else {
            self.match_token(TokenType::End)?;
            Ok(Statement::If {
                condition,
                then_block: Box::new(then_block),
                else_block: None,
            })
        }
    }

    fn block_stmt(&mut self) -> Result<Vec<Statement>> {
        let mut stmts = Vec::new();
        while self.current()?.get_type() != TokenType::End
            && self.current()?.get_type() != TokenType::Else
        {
            let stmt = self.statement()?;
            stmts.push(stmt)
        }
        Ok(stmts)
    }

    fn while_stmt(&mut self) -> Result<Statement> {
        let condition = self.expression()?;
        self.match_token(TokenType::Then)?;
        let then_block = self.statement()?;
        Ok(Statement::While {
            condition,
            then_block: Box::new(then_block),
        })
    }

    fn expr_stmt(&mut self) -> Result<Statement> {
        Ok(Statement::Expr(self.expression()?))
    }

    pub(self) fn expression(&mut self) -> Result<Expr> {
        self.binary()
    }

    fn binary(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let left = self.comparasion()?;
        if let Ok(_) = self.match_token(TokenType::Equal) {
            let right = self.comparasion()?;
            Ok(Expr::Set {
                variable: Box::new(left),
                value: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    fn comparasion(&mut self) -> Result<Expr> {
        let mut left = self.addition()?;
        while let Ok(token) = self
            .match_token(TokenType::Less)
            .or_else(|_| self.match_token(TokenType::Greater))
            .or_else(|_| self.match_token(TokenType::LessEqual))
            .or_else(|_| self.match_token(TokenType::GreaterEqual))
            .or_else(|_| self.match_token(TokenType::EqualEqual))
            .or_else(|_| self.match_token(TokenType::BangEqual))
        {
            let binop: BinaryOp = token.get_type().into();
            let right = self.addition()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: binop,
                right: Box::new(right),
            }
        }
        Ok(left)
    }

    fn addition(&mut self) -> Result<Expr> {
        let mut left = self.multiplication()?;
        while let Ok(token) = self
            .match_token(TokenType::Plus)
            .or_else(|_| self.match_token(TokenType::Minus))
        {
            let binop: BinaryOp = token.get_type().into();
            let right = self.multiplication()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: binop,
                right: Box::new(right),
            }
        }
        Ok(left)
    }

    fn multiplication(&mut self) -> Result<Expr> {
        let mut left = self.unary()?;
        while let Ok(token) = self
            .match_token(TokenType::Star)
            .or_else(|_| self.match_token(TokenType::Slash))
        {
            let binop: BinaryOp = token.get_type().into();
            let right = self.unary()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: binop,
                right: Box::new(right),
            }
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr> {
        if let Ok(token) = self
            .match_token(TokenType::Plus)
            .or_else(|_| self.match_token(TokenType::Minus))
            .or_else(|_| self.match_token(TokenType::Bang))
        {
            let unop: UnaryOp = token.get_type().into();
            let expr = self.unary()?;
            Ok(Expr::Unary {
                op: unop,
                expr: Box::new(expr),
            })
        } else {
            self.access()
        }
    }

    // TODO: test this
    fn access(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;
        while let Ok(token) = self
            .match_token(TokenType::Dot)
            .or_else(|_| self.match_token(TokenType::LeftBracket))
        {
            match token.get_type() {
                TokenType::Dot => {
                    let token = self.match_token(TokenType::Identifier)?;
                    let name = token.text().to_string();
                    expr = Expr::Access {
                        table: Box::new(expr),
                        field: Box::new(Expr::Literal(Literal::Str(name))),
                    };
                }
                TokenType::LeftBracket => {
                    let access_expr = self.expression()?;
                    expr = Expr::Access {
                        table: Box::new(expr),
                        field: Box::new(access_expr),
                    };
                    self.match_token(TokenType::RightBracket)?;
                }
                _ => unreachable!(),
            }
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr> {
        // println!("primary: {}", self.current()?.text());
        if let Ok(token) = self.match_token(TokenType::String) {
            let string = token.text().to_string();
            Ok(Expr::Literal(Literal::Str(string)))
        } else if let Ok(token) = self.match_token(TokenType::Number) {
            let number: f64 = token.text().parse().unwrap();
            Ok(Expr::Literal(Literal::Number(number)))
        } else if let Ok(token) = self.match_token(TokenType::Identifier) {
            let name = token.text();
            Ok(Expr::Identifier(name.to_string()))
        } else if self.match_token(TokenType::True).is_ok() {
            Ok(Expr::Literal(Literal::Bool(true)))
        } else if self.match_token(TokenType::False).is_ok() {
            Ok(Expr::Literal(Literal::Bool(false)))
        } else if self.match_token(TokenType::Nil).is_ok() {
            Ok(Expr::Literal(Literal::Nil))
        } else if self.match_token(TokenType::LeftParen).is_ok() {
            self.grouping()
        } else if self.match_token(TokenType::LeftCurly).is_ok() {
            self.table_init()
        } else {
            Err(ParserError::UnexpectedToken {
                token: self.current()?,
            })
        }
    }

    #[inline]
    fn grouping(&mut self) -> Result<Expr> {
        let expr = self.expression()?;
        if let Ok(_) = self.match_token(TokenType::Comma) {
            self.tuple(expr)
        } else {
            self.match_token(TokenType::RightParen)?;
            Ok(Expr::Grouping(Box::new(expr)))
        }
    }

    fn tuple(&mut self, first_expr: Expr) -> Result<Expr> {
        let second_expr = self.expression()?;
        let mut elems = vec![first_expr, second_expr];
        while let Ok(_) = self.match_token(TokenType::Comma) {
            let expr = self.expression()?;
            elems.push(expr);
        }
        self.match_token(TokenType::RightParen)?;
        Ok(Expr::Tuple(elems))
    }

    fn table_init(&mut self) -> Result<Expr> {
        let mut values = Vec::new();
        let mut keys: Option<Vec<Expr>> = {
            let expr = self.expression()?;
            match expr {
                Expr::Set { variable, value } => {
                    values.push(*value);
                    Some(vec![*variable])
                }
                other => {
                    values.push(other);
                    None
                }
            }
        };
        while let Ok(_) = self.match_token(TokenType::Comma) {
            if let Some(keys) = keys.as_mut() {
                let expr = self.expression()?;
                match expr {
                    Expr::Set { variable, value } => {
                        keys.push(*variable);
                        values.push(*value);
                    }
                    _ => return Err(ParserError::InitError),
                }
            } else {
                let expr = self.expression()?;
                match expr {
                    Expr::Set { .. } => return Err(ParserError::InitError),
                    _ => values.push(expr),
                }
            }
        }
        self.match_token(TokenType::RightCurly)?;
        Ok(Expr::TableInit { keys, values })
    }
}

impl<I> Deref for Parser<I>
where
    I: Iterator<Item = Token>,
{
    type Target = LookAhead<I>;

    fn deref(&self) -> &Self::Target {
        &self.lookahead
    }
}

impl<I> DerefMut for Parser<I>
where
    I: Iterator<Item = Token>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lookahead
    }
}

#[cfg(test)]
mod tests {
    use super::{BinaryOp, Expr, Literal, Parser, ParserError, UnaryOp};

    #[test]
    fn binary_works() {
        let source = "3 + 4 * 2 < 20 - 4";
        let mut parser = Parser::new(source).unwrap();
        let parsed = parser.expression().unwrap();
        assert_eq!(
            parsed,
            Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Literal(Literal::Number(3.0))),
                    op: BinaryOp::Plus,
                    right: Box::new(Expr::Binary {
                        left: Box::new(Expr::Literal(Literal::Number(4.0))),
                        op: BinaryOp::Star,
                        right: Box::new(Expr::Literal(Literal::Number(2.0)))
                    }),
                }),
                op: BinaryOp::Less,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Literal(Literal::Number(20.0))),
                    op: BinaryOp::Minus,
                    right: Box::new(Expr::Literal(Literal::Number(4.0)))
                })
            }
        )
    }

    #[test]
    fn grouping_works() {
        let source = "(3 + 4) * 2";
        let mut parser = Parser::new(source).unwrap();
        let parsed = parser.expression().unwrap();
        assert_eq!(
            parsed,
            Expr::Binary {
                left: Box::new(Expr::Grouping(Box::new(Expr::Binary {
                    left: Box::new(Expr::Literal(Literal::Number(3.0))),
                    op: BinaryOp::Plus,
                    right: Box::new(Expr::Literal(Literal::Number(4.0)))
                }))),
                op: BinaryOp::Star,
                right: Box::new(Expr::Literal(Literal::Number(2.0)))
            }
        );
    }

    #[test]
    fn tuple_works() {
        let source = "(3, \"hello\")";
        let mut parser = Parser::new(source).unwrap();
        let parsed = parser.expression().unwrap();
        assert_eq!(
            parsed,
            Expr::Tuple(vec![
                Expr::Literal(Literal::Number(3.0)),
                Expr::Literal(Literal::Str("hello".to_string()))
            ])
        );

        let source = "((3 + 2, \"hello\", !false, (nil)))";
        let mut parser = Parser::new(source).unwrap();
        let parsed = parser.expression().unwrap();
        assert_eq!(
            parsed,
            Expr::Grouping(Box::new(Expr::Tuple(vec![
                Expr::Binary {
                    left: Box::new(Expr::Literal(Literal::Number(3.0))),
                    op: BinaryOp::Plus,
                    right: Box::new(Expr::Literal(Literal::Number(2.0)))
                },
                Expr::Literal(Literal::Str("hello".to_string())),
                Expr::Unary {
                    op: UnaryOp::Bang,
                    expr: Box::new(Expr::Literal(Literal::Bool(false))),
                },
                Expr::Grouping(Box::new(Expr::Literal(Literal::Nil)))
            ])))
        );
    }

    #[test]
    fn table_init_works() {
        let source = "{3 = 6, \"foo\" = bar, \"xd\" = 5 + 3}";
        let mut parser = Parser::new(source).unwrap();
        let parsed = parser.expression().unwrap();
        assert_eq!(
            parsed,
            Expr::TableInit {
                keys: Some(vec![
                    Expr::Literal(Literal::Number(3.0)),
                    Expr::Literal(Literal::Str("foo".to_string())),
                    Expr::Literal(Literal::Str("xd".to_string())),
                ]),
                values: vec![
                    Expr::Literal(Literal::Number(6.0)),
                    Expr::Identifier("bar".to_string()),
                    Expr::Binary {
                        left: Box::new(Expr::Literal(Literal::Number(5.0))),
                        op: BinaryOp::Plus,
                        right: Box::new(Expr::Literal(Literal::Number(3.0)))
                    },
                ]
            }
        );

        let source = "{3 = 6, \"foo\", \"xd\" = 5 + 3}";
        let mut parser = Parser::new(source).unwrap();
        let parsed = parser.expression();
        assert_eq!(parsed, Err(ParserError::InitError));
    }
}
