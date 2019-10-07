use super::{Ast, BlockExpr, Expr, Parser, ParserErrorKind, Result, Statement, Token};
use crate::vm::lib::PREDEFINED_CONSTANTS;
use std::collections::HashSet;

pub struct Analyzer<'a, I>
where
    I: Iterator<Item = Token>,
{
    parser: &'a Parser<I>,
    scopes: Vec<Scope>,
    globals: HashSet<String>,
}

struct Scope {
    name: Option<String>, // Function name for recursion
    locals: HashSet<String>,
    environment: Option<HashSet<String>>,
}

const ENV_NAME: &str = "env";

impl Scope {
    fn block() -> Self {
        Scope {
            name: None,
            locals: HashSet::new(),
            environment: None,
        }
    }

    fn function(name: Option<String>) -> Self {
        Scope {
            name,
            locals: HashSet::new(),
            environment: Some(HashSet::new()),
        }
    }

    fn global() -> Self {
        let locals: HashSet<String> = PREDEFINED_CONSTANTS
            .iter()
            .map(|(name, _)| name.to_string())
            .collect();
        Scope {
            name: None,
            locals,
            environment: None,
        }
    }
}

impl<'a, I> Analyzer<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn analyze(ast: Ast, parser: &'a Parser<I>) -> Result<Ast> {
        let mut analyzer = Self::new(parser);
        analyzer.visit_ast(ast)
    }

    fn new(parser: &'a Parser<I>) -> Self {
        Analyzer {
            parser,
            scopes: vec![Scope::global()],
            globals: HashSet::new(),
        }
    }

    fn visit_ast(&mut self, Ast(mut block_expr): Ast) -> Result<Ast> {
        self.visit_block_expr(&mut block_expr)?;
        Ok(Ast(block_expr))
    }

    fn visit_block_expr(&mut self, block_expr: &mut BlockExpr) -> Result<()> {
        for stmt in block_expr.stmts.iter_mut() {
            self.visit_stmt(stmt)?;
        }
        self.visit_expr(block_expr.expr.as_mut(), None)
    }

    fn visit_stmt(&mut self, stmt: &mut Statement) -> Result<()> {
        match stmt {
            Statement::Let { name, value } => {
                match value {
                    // Also block?
                    Expr::Function { .. } => {
                        self.add_local(&*name)?;
                        self.visit_expr(value, Some(name.clone()))
                    }
                    _ => {
                        self.visit_expr(value, None)?;
                        self.add_local(&*name)
                    }
                }
            }
            Statement::Var { name, value } => {
                if self.is_top_level() {
                    self.visit_expr(value, None)?;
                    self.globals.insert(name.to_string());
                    Ok(())
                } else {
                    Err(self
                        .parser
                        .make_error(ParserErrorKind::InnerVarDeclaration {
                            name: name.to_string(),
                        })?)
                }
            }
            Statement::Set { variable, value } => {
                self.visit_expr(variable, None)?;
                self.visit_expr(value, None)
            }
            Statement::Block(stmts) => stmts
                .into_iter()
                .fold(Ok(()), |result, stmt| result.and(self.visit_stmt(stmt))),
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.visit_expr(condition, None)?;
                self.visit_expr(then_block.as_mut(), None)?;
                match else_block {
                    Some(expr) => self.visit_expr(expr.as_mut(), None),
                    None => Ok(()),
                }
            }
            Statement::While {
                condition,
                then_block,
            } => {
                self.visit_expr(condition, None)?;
                self.visit_stmt(then_block.as_mut())
            }
            Statement::Return(expr) => self.visit_expr(expr, None),
            Statement::Import { name, .. } => {
                self.add_local(name)?;
                Ok(())
            }
            Statement::Expr(expr) => self.visit_expr(expr, None),
            _ => unimplemented!(),
        }
    }

    fn visit_expr(&mut self, expr: &mut Expr, func_name: Option<String>) -> Result<()> {
        use Expr::*;
        match expr {
            Identifier(name) => {
                // TODO: seems like not the best way to do it

                let is_rec = {
                    let env_name = self
                        .scopes
                        .iter()
                        .rev()
                        .find(|s| s.environment.is_some())
                        .and_then(|s| s.name.as_ref());
                    env_name == Some(name)
                };

                if is_rec {
                    *expr = Rec;
                    Ok(())
                } else {
                    let mut is_local_somewhere = false;

                    for env in self
                        .scopes
                        .iter_mut()
                        .rev()
                        .take_while(|s| {
                            is_local_somewhere = s.locals.contains(name);
                            !is_local_somewhere
                        })
                        .filter_map(|s| s.environment.as_mut())
                    {
                        env.insert(name.to_string());
                    }
                    if is_local_somewhere {
                        if !self.has_local(name) {
                            *expr = Expr::Access {
                                table: Box::new(Expr::Identifier("env".to_owned())),
                                field: Box::new(Expr::string(name.to_owned())),
                            }
                        }
                        Ok(())
                    } else {
                        let is_global = self.globals.contains(name);
                        if !is_global {
                            Err(self.parser.make_error(ParserErrorKind::Undeclared {
                                name: name.to_string(),
                            })?)
                        } else {
                            Ok(())
                        }
                    }
                }

                // TODO: get the name of the env from scope
                // Add to the env until find local
                /* match func_name.map(|n| &n == name) {
                    // Call self, ignore
                    Some(true) => {
                        *expr = Rec;
                        Ok(())
                    },
                    _ => {
                        for env in self.scopes.iter_mut()
                            .rev()
                            .take_while(|s| {
                                is_local_somewhere = s.locals.contains(name);
                                !is_local_somewhere
                            })
                            .filter_map(|s| s.environment.as_mut())
                        {
                            env.insert(name.to_string());
                        }
                        if is_local_somewhere {
                            if !self.has_local(name) {
                                *expr = Expr::Access {
                                    table: Box::new(Expr::Identifier("env".to_owned())),
                                    field: Box::new(Expr::string(name.to_owned()))
                                }
                            }
                            Ok(())
                        } else {
                            let is_global = self.globals.contains(name);
                            if !is_global {
                                Err(self.parser.make_error(ParserErrorKind::Undeclared { name: name.to_string() })?)
                            } else {
                                Ok(())
                            }
                        }
                    }
                }    */
            }
            Unary { expr, .. } => self.visit_expr(expr.as_mut(), None),
            Binary { left, right, .. } => self
                .visit_expr(left.as_mut(), None)
                .and(self.visit_expr(right.as_mut(), None)),
            Grouping(expr) => self.visit_expr(expr.as_mut(), None),
            Tuple(exprs) => exprs
                .into_iter()
                .fold(Ok(()), |acc, e| acc.and(self.visit_expr(e, None))),
            Access { table, field } => self
                .visit_expr(table.as_mut(), None)
                .and(self.visit_expr(field.as_mut(), None)),
            SelfAccess { table, args, .. } => {
                self.visit_expr(table.as_mut(), None)?;
                args.into_iter()
                    .fold(Ok(()), |res, arg| res.and(self.visit_expr(arg, None)))
            }
            TableInit { keys, values } => {
                if let Some(keys) = keys {
                    for key in keys {
                        self.visit_expr(key, None)?;
                    }
                }
                values
                    .into_iter()
                    .fold(Ok(()), |res, value| res.and(self.visit_expr(value, None)))
            }
            Function { body, args, env } => {
                self.enter_env(func_name);

                for arg in args.iter() {
                    self.add_local(arg)?;
                }

                self.visit_block_expr(body)?;

                let env_vars = self.exit_env().expect("expected a lexical environment");
                match env_vars.len() {
                    0 => (),
                    _ => {
                        let keys = env_vars.iter().map(|v| Expr::string(v.clone())).collect();
                        let mut values: Vec<Expr> =
                            env_vars.iter().map(|v| Identifier(v.clone())).collect();
                        for value in values.iter_mut() {
                            self.visit_expr(value, None)?;
                        }
                        *env = Some((keys, values));

                        args.push(ENV_NAME.to_owned());
                    }
                }
                Ok(())
            }
            Call { func, args } => {
                self.visit_expr(func.as_mut(), None)?;
                args.into_iter()
                    .fold(Ok(()), |res, arg| res.and(self.visit_expr(arg, None)))
            }
            Literal(_) => Ok(()),
            Block(block_expr) => {
                self.enter_scope();
                self.visit_block_expr(block_expr)?;
                self.exit_scope();
                Ok(())
            }
            If {
                condition,
                then_block,
                else_block,
            } => {
                self.visit_expr(condition.as_mut(), None)?;
                self.visit_expr(then_block.as_mut(), None)?;
                self.visit_expr(else_block.as_mut(), None)
            }
            Rec => Ok(()),
        }
    }

    fn add_local(&mut self, name: &str) -> Result<()> {
        let inserted = self
            .scopes
            .last_mut()
            .unwrap()
            .locals
            .insert(name.to_owned());
        if !inserted {
            Err(self.parser.make_error(ParserErrorKind::Redeclaration {
                name: name.to_owned(),
            })?)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn has_local(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.locals.contains(name) {
                return true;
            }
            if scope.environment.is_some() {
                break;
            }
        }
        false
    }

    fn enter_scope(&mut self) {
        self.scopes.push(Scope::block())
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn enter_env(&mut self, name: Option<String>) {
        self.scopes.push(Scope::function(name))
    }

    fn exit_env(&mut self) -> Option<HashSet<String>> {
        self.scopes.pop().and_then(|s| s.environment)
    }

    #[inline]
    fn is_top_level(&self) -> bool {
        self.scopes.len() == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_desugar_simple_works() {
        let source = "
            fn(x)
                fn(y)
                    x + y
                end
            end
        ";
        let mut parser = Parser::new(source).unwrap();
        let ast = parser.parse().unwrap();
        let desugared = Analyzer::analyze(ast, &parser).unwrap();
        // Complete this
        /*  let ast = Expr::Block(BlockExpr {
            stmts: vec![],
            expr: Box::new(Expr::Function {
                args: vec!["x".to_owned()],
                body: BlockExpr {
                    stmts: vec![],
                    expr: Box::new(Expr::Function {
                        args: vec!["y".to_owned(), "env".to_owned()],
                        body: BlockExpr {
                            stmts: vec![],
                            expr: Box::new(Expr::Binary {
                                left: Expr::Literal("x".to)
                            })
                        },
                        env: Some((vec!["x".to_owned()], vec!["x"]))
                    })
                },
                env: None,
            })
        }); */
        println!("{:#?}", desugared);
    }

    #[test]
    fn function_desugar_complex_works() {
        let source = "
            let foo = fn(x)
                fn(y)
                    fn()
                        x + y
                    end
                end
            end;
        ";
        let mut parser = Parser::new(source).unwrap();
        let ast = parser.parse().unwrap();
        println!("{:#?}", ast);
    }

    #[test]
    fn inner_var_declaration_is_forbidden() {
        let source = "
        fn()
            var a = nil;
        end
        ";
        let parse_result = Parser::parse_str(source);
        assert_eq!(
            parse_result.unwrap_err().kind,
            ParserErrorKind::InnerVarDeclaration {
                name: "a".to_string()
            }
        )
    }
}
