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
        if self.match_token(TokenType::Let).is_ok() {
            self.let_stmt()
        } else if self.match_token(TokenType::If).is_ok() {
            self.if_stmt()
        } else if self.match_token(TokenType::Do).is_ok() {
            let stmt = Statement::Block(self.block_stmt()?);
            self.match_token(TokenType::End)?;
            Ok(stmt)
        } else if self.match_token(TokenType::While).is_ok() {
            self.while_stmt()
        } else if self.match_token(TokenType::Return).is_ok() {
            self.return_stmt()
        } else if self.match_token(TokenType::Fn).is_ok() {
            self.fn_stmt()
        } else {
            // self.expr_stmt()
            let expr = self.expression()?;
            if self.match_token(TokenType::Semicolon).is_ok() {
                Ok(Statement::Expr(expr))
            } else {
                Err(ParserError::UnexpectedExpr(expr))
            }
        }
    }

    fn let_stmt(&mut self) -> Result<Statement> {
        let token = self.match_token(TokenType::Identifier)?;
        let name = token.text();
        self.match_token(TokenType::Equal)?;
        let value = self.expression()?;
        self.match_token(TokenType::Semicolon)?;
        Ok(Statement::Let {
            name: name.to_string(),
            value,
        })
    }

    fn if_stmt(&mut self) -> Result<Statement> {
        let condition = self.expression()?;
        self.match_token(TokenType::Then)?;
        let then_block = self.block_expr_impl()?;
        if self.match_token(TokenType::Else).is_ok() {
            let else_block = if self.match_token(TokenType::If).is_ok() {
                let if_stmt = self.if_stmt()?;
                Some(Box::new(if_stmt.into_expr().unwrap()))
            } else {
                Some(Box::new(self.block_expr(TokenType::End)?))
            };
            // let else_block = self.block_expr(TokenType::End)?;
            Ok(Statement::If {
                condition,
                then_block: Box::new(then_block),
                else_block,
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
        let then_block = Statement::Block(self.block_stmt()?);
        self.match_token(TokenType::End)?;
        Ok(Statement::While {
            condition,
            then_block: Box::new(then_block),
        })
    }

    #[allow(dead_code)]
    fn print_stmt(&mut self) -> Result<Statement> {
        let expr = self.expression()?;
        Ok(Statement::Print(expr))
    }

    fn return_stmt(&mut self) -> Result<Statement> {
        let expr = self.expression().unwrap_or(Expr::Literal(Literal::Unit));
        let _ = self.match_token(TokenType::Semicolon);
        Ok(Statement::Return(expr))
    }

    fn fn_stmt(&mut self) -> Result<Statement> {
        let token = self.match_token(TokenType::Identifier)?;
        let name = token.extract_text();
        let value = self.function()?;
        Ok(Statement::Let { name, value })
    }

    pub(self) fn expression(&mut self) -> Result<Expr> {
        self.binary()
    }

    fn binary(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let left = self.comparasion()?;
        if self.match_token(TokenType::Equal).is_ok() {
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
        while let Some(token) = [
                TokenType::Less, 
                TokenType::Greater, 
                TokenType::LessEqual,
                TokenType::GreaterEqual,
                TokenType::EqualEqual,
                TokenType::BangEqual
            ].iter()
            .find_map(|t| self.match_token(*t).ok())
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
            .or_else(|_| self.match_token(TokenType::Colon))
            .or_else(|_| self.match_token(TokenType::LeftBracket))
            .or_else(|_| self.match_token(TokenType::LeftParen))
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
                TokenType::Colon => {
                    let token = self.match_token(TokenType::Identifier)?;
                    let name = token.text().to_string();
                    expr = Expr::SelfAccess {
                        table: Box::new(expr),
                        field: name,
                    }
                }
                TokenType::LeftBracket => {
                    let access_expr = self.expression()?;
                    expr = Expr::Access {
                        table: Box::new(expr),
                        field: Box::new(access_expr),
                    };
                    self.match_token(TokenType::RightBracket)?;
                }
                TokenType::LeftParen => {
                    let mut args = Vec::new();
                    if let Err(_) = self.match_token(TokenType::RightParen) {
                        args.push(self.expression()?);
                        while self.match_token(TokenType::Comma).is_ok() {
                            args.push(self.expression()?);
                        }
                        self.match_token(TokenType::RightParen)?;
                    }
                    expr = Expr::Call {
                        func: Box::new(expr),
                        args,
                    }
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
        } else if self.match_token(TokenType::Fn).is_ok() {
            self.function()
        } else if self.match_token(TokenType::Do).is_ok() {
            self.block_expr(TokenType::End)
        } else if self.match_token(TokenType::If).is_ok() {
            self.if_expr()
        } else {
            Err(ParserError::UnexpectedToken {
                token: self.current()?,
            })
        }
    }

    #[inline]
    fn grouping(&mut self) -> Result<Expr> {
        let expr = self.expression()?;
        if self.match_token(TokenType::Comma).is_ok() {
            self.tuple(expr)
        } else {
            self.match_token(TokenType::RightParen)?;
            Ok(Expr::Grouping(Box::new(expr)))
        }
    }

    fn tuple(&mut self, first_expr: Expr) -> Result<Expr> {
        let second_expr = self.expression()?;
        let mut elems = vec![first_expr, second_expr];
        while self.match_token(TokenType::Comma).is_ok() {
            let expr = self.expression()?;
            elems.push(expr);
        }
        self.match_token(TokenType::RightParen)?;
        Ok(Expr::Tuple(elems))
    }

    fn table_init(&mut self) -> Result<Expr> {
        if self.match_token(TokenType::RightCurly).is_ok() {
            Ok(Expr::TableInit {
                values: Vec::new(),
                keys: None,
            })
        } else {
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
            while self.match_token(TokenType::Comma).is_ok() {
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

    fn function(&mut self) -> Result<Expr> {
        let mut args = Vec::new();
        self.match_token(TokenType::LeftParen)?;
        if let Ok(token) = self.match_token(TokenType::Identifier) {
            args.push(token.extract_text());
            while self.match_token(TokenType::RightParen).is_err() {
                self.match_token(TokenType::Comma)?;
                let name = self.match_token(TokenType::Identifier)?;
                args.push(name.extract_text());
            }
        } else {
            self.match_token(TokenType::RightParen)?;
        }
        let body = self.block_stmt()?;
        self.match_token(TokenType::End)?;
        Ok(Expr::Function { args, body })
    }

    fn block_expr(&mut self, terminating_token: TokenType) -> Result<Expr> {
        //...
        let expr = self.block_expr_impl()?;
        self.match_token(terminating_token)?;
        Ok(expr)
    }

    fn block_expr_impl(&mut self) -> Result<Expr> {
        let mut stmts = Vec::new();
        let expr = loop {
            match self.statement() {
                Ok(stmt) => stmts.push(stmt),
                Err(err) => {
                    match err {
                        ParserError::UnexpectedExpr(expr) => break expr,
                        // This may omit some unexpected error
                        // TODO: check if matched with terminating token if so push literal expr
                        _ => {
                            // Check if last statement can be converted to expr
                            let last_stmt = stmts.last();
                            break if last_stmt.is_some() && last_stmt.unwrap().can_convert_expr() {
                                stmts.pop().unwrap().into_expr().unwrap()
                            } else {
                                Expr::Literal(Literal::Unit)
                            }
                        },
                    }
                }
            }
        };
        Ok(Expr::Block {
            stmts,
            expr: Box::new(expr),
        })
    }

    fn if_expr(&mut self) -> Result<Expr> {
        let condition = self.expression()?;
        self.match_token(TokenType::Then)?;
        let then_block = self.block_expr(TokenType::Else)?;
        let else_block = if self.match_token(TokenType::If).is_ok() {
            self.if_expr()?
        } else {
            self.block_expr(TokenType::End)?
        };

        Ok(Expr::If {
            condition: Box::new(condition),
            then_block: Box::new(then_block),
            else_block: Box::new(else_block),
        })
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
    use super::*;

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

        let source = "{}";
        let mut parser = Parser::new(source).unwrap();
        let parsed = parser.expression();
        assert_eq!(
            parsed,
            Ok(Expr::TableInit {
                values: Vec::new(),
                keys: None
            })
        );
    }

    #[test]
    fn call_works() {
        let source = "foo(5 + 2, bar[\"foo\"])";
        let mut parser = Parser::new(source).unwrap();
        let parsed = parser.expression().unwrap();
        assert_eq!(
            parsed,
            Expr::Call {
                func: Box::new(Expr::Identifier("foo".to_string())),
                args: vec![
                    Expr::Binary {
                        left: Box::new(Expr::Literal(Literal::Number(5.0))),
                        op: BinaryOp::Plus,
                        right: Box::new(Expr::Literal(Literal::Number(2.0))),
                    },
                    Expr::Access {
                        table: Box::new(Expr::Identifier("bar".to_string())),
                        field: Box::new(Expr::Literal(Literal::Str("foo".to_string()))),
                    }
                ]
            }
        );
        let source = "bar[\"foo\"].hello()";
        let mut parser = Parser::new(source).unwrap();
        let parsed = parser.expression().unwrap();
        assert_eq!(
            parsed,
            Expr::Call {
                func: Box::new(Expr::Access {
                    table: Box::new(Expr::Access {
                        table: Box::new(Expr::Identifier("bar".to_string())),
                        field: Box::new(Expr::Literal(Literal::Str("foo".to_string()))),
                    }),
                    field: Box::new(Expr::Literal(Literal::Str("hello".to_string())))
                }),
                args: vec![]
            }
        )
    }

    #[test]
    fn fn_stmt_works() {
        let source = "fn foo() end";
        let mut parser = Parser::new(source).unwrap();
        let parsed = parser.statement().unwrap();
        assert_eq!(
            parsed,
            Statement::Let {
                name: "foo".to_string(),
                value: Expr::Function {
                    args: vec![],
                    body: vec![]
                }
            }
        )
    }

    #[test]
    fn block_expr_works() {
        let source = "
            let foo = do
                let bar = foo;
                5
            end;
        ";
        let mut parser = Parser::new(source).unwrap();
        let parsed = parser.parse().unwrap();
        assert_eq!(
            parsed,
            vec![Statement::Let {
                name: "foo".to_string(),
                value: Expr::Block {
                    stmts: vec![Statement::Let {
                        name: "bar".to_string(),
                        value: Expr::Identifier("foo".to_string()),
                    }],
                    expr: Box::new(Expr::Literal(Literal::Number(5.0)))
                }
            }]
        );
        let source = "
            let square = fn(x)
                x * x
            end;
        ";
        let source = "
            let bar = fn()
                return 5;
            end;
        ";
    }
}
