mod chunk;
mod error;
mod instruction;

pub use self::chunk::{Chunk, JumpCondition};
use crate::parser::{BinaryOp, Expr, Literal, Statement, UnaryOp};
use crate::vm::Value;
pub(self) use error::CompileError;
pub use instruction::{BinaryInstr, Instruction, UnaryInstr};

pub type CompileResult<T> = Result<T, CompileError>;

pub struct Compiler {
    chunk: Chunk,
    locals: Vec<Local>,
    depth: u8,
    closure_depths: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Local {
    name: String,
    depth: u8,
    // n means the function in compiler.closure_depths[n]
    closure: Option<u8>,
}

/**
 * Compiling
 */
impl Compiler {
    pub fn compile(stmts: Vec<Statement>) -> CompileResult<Chunk> {
        let mut compiler = Self::new();
        for stmt in stmts {
            compiler.compile_stmt(stmt)?;
        }
        compiler.chunk.push_instr(Instruction::Return {
            return_value: false,
        })?;
        Ok(compiler.chunk)
    }

    fn new() -> Self {
        Compiler {
            chunk: Chunk::new(),
            locals: Vec::new(),
            depth: 0,
            closure_depths: Vec::new(),
        }
    }

    fn compile_stmt(&mut self, stmt: Statement) -> CompileResult<()> {
        match stmt {
            Statement::Expr(expr) => self.expr_stmt(expr),
            Statement::Let { name, value } => self.let_stmt(name, value),
            Statement::Block(statements) => self.block_stmt(statements),
            Statement::If {
                condition,
                then_block,
                else_block,
            } => self.if_stmt(condition, then_block, else_block),
            Statement::While {
                condition,
                then_block,
            } => self.while_stmt(condition, then_block),
            Statement::Print(expr) => {
                self.compile_expr(expr)?;
                self.chunk.push_instr(Instruction::Print)
            }
            Statement::Return(expr) => {
                self.compile_expr(expr)?;
                self.chunk
                    .push_instr(Instruction::Return { return_value: true })
            }
        }
    }

    fn expr_stmt(&mut self, expr: Expr) -> CompileResult<()> {
        let is_assignment = match &expr {
            Expr::Set { .. } => true,
            _ => false,
        };
        self.compile_expr(expr)?;
        if !is_assignment {
            self.chunk.push_instr(Instruction::Pop)?;
        }
        Ok(())
    }

    fn let_stmt(&mut self, name: String, value: Expr) -> CompileResult<()> {
        self.push_local(name);
        self.compile_expr(value)?;
        Ok(())
    }

    fn block_stmt(&mut self, statements: Vec<Statement>) -> CompileResult<()> {
        self.scope_incr();
        for stmt in statements {
            self.compile_stmt(stmt)?;
        }
        for _ in 0..self.scope_decr() {
            self.chunk.push_instr(Instruction::Pop)?;
        }
        Ok(())
    }

    // TODO: Better PLEASE
    fn if_stmt(
        &mut self,
        condition: Expr,
        then_block: Box<Statement>,
        else_block: Option<Box<Statement>>,
    ) -> CompileResult<()> {
        self.compile_expr(condition)?;
        let patch_index = self.chunk.push_placeholder()?;
        self.compile_stmt(*then_block)?;
        let offset = self.chunk.instructions().len() - patch_index;
        if offset > std::i8::MAX as usize {
            return Err(CompileError::TooLongToJump);
        }
        self.chunk
            .patch_placeholder(patch_index, offset as i8, JumpCondition::WhenFalse)?;
        let has_else = if let Some(else_block) = else_block {
            let patch_index = self.chunk.push_placeholder()?;
            self.compile_stmt(*else_block)?;
            let offset = self.chunk.instructions().len() - patch_index;
            if offset > std::i8::MAX as usize {
                return Err(CompileError::TooLongToJump);
            }
            self.chunk
                .patch_placeholder(patch_index, offset as i8, JumpCondition::None)?;
            true
        } else {
            false
        };
        // Jump one further
        if has_else {
            self.chunk.patch_placeholder(
                patch_index,
                (offset + 1) as i8,
                JumpCondition::WhenFalse,
            )?;
        }
        Ok(())
    }

    fn while_stmt(&mut self, condition: Expr, then_block: Box<Statement>) -> CompileResult<()> {
        let start_index = self.chunk.instructions().len();
        self.compile_expr(condition)?;
        let patch_index = self.chunk.push_placeholder()?;
        self.compile_stmt(*then_block)?;
        let offset = self.chunk.instructions().len() - patch_index + 1;
        self.chunk
            .patch_placeholder(patch_index, offset as i8, JumpCondition::WhenFalse)?;
        self.chunk.push_instr(Instruction::Jump {
            offset: -((self.chunk.instructions().len() - start_index) as i8),
        })
    }

    fn compile_expr(&mut self, expr: Expr) -> CompileResult<()> {
        match expr {
            Expr::Literal(lit) => self.literal(lit),
            Expr::Identifier(name) => self.ident(name),
            Expr::Unary { op, expr } => self.unary(*expr, op),
            Expr::Binary { left, op, right } => self.binary(*left, *right, op),
            Expr::Grouping(expr) => self.compile_expr(*expr),
            Expr::Tuple(exprs) => self.tuple(exprs),
            Expr::Access { table, field } => self.access(*table, *field),
            Expr::Set { variable, value } => self.set(*variable, *value),
            Expr::TableInit { keys, values } => self.table_init(keys, values),
            Expr::Function { args, body } => self.function_def(args, body),
            Expr::Call { func, args } => self.call(*func, args),
            _ => Err(CompileError::UnimplementedExpr(expr)),
        }
    }

    fn literal(&mut self, lit: Literal) -> CompileResult<()> {
        match lit {
            Literal::Nil => self.chunk.push_instr(Instruction::Nil),
            Literal::Bool(b) => self.chunk.push_instr(match b {
                true => Instruction::True,
                false => Instruction::False,
            }),
            Literal::Number(n) => match n.fract() == 0.0 {
                true => self.chunk.push_constant(Value::Int(n.trunc() as i32)),
                false => self.chunk.push_constant(Value::Number(n)),
            }
            .map(|_| ()),
            Literal::Str(string) => {
                let index = self.chunk.add_constant(string.into())?;
                self.chunk.push_instr(Instruction::Constant { index })?;
                Ok(())
            }
            Literal::Unit => self.chunk.push_instr(Instruction::Unit),
        }
    }

    fn ident(&mut self, name: String) -> CompileResult<()> {
        if let Some((index, local)) = self.resolve_local(name.as_str()) {
            let index = index as u16;
            if local.is_in_function() {
                self.chunk.push_instr(Instruction::GetFnLocal { index })
            } else {
                self.chunk.push_instr(Instruction::GetLocal { index })
            }
        } else {
            if let Some(index) = self.chunk.has_string(name.as_str()) {
                self.chunk.push_instr(Instruction::GetGlobal { index })
            } else {
                Err(CompileError::UndefinedVariable { name })
            }
        }
    }

    fn unary(&mut self, expr: Expr, op: UnaryOp) -> CompileResult<()> {
        self.compile_expr(expr)?;
        let unary = match op {
            UnaryOp::Minus => UnaryInstr::Negate,
            UnaryOp::Bang => UnaryInstr::Not,
        };
        self.chunk.push_instr(Instruction::Unary(unary))
    }

    fn binary(&mut self, left: Expr, right: Expr, op: BinaryOp) -> CompileResult<()> {
        self.compile_expr(left)?;
        self.compile_expr(right)?;
        let binary = match op {
            BinaryOp::Plus => BinaryInstr::Add,
            BinaryOp::Minus => BinaryInstr::Sub,
            BinaryOp::Star => BinaryInstr::Mul,
            BinaryOp::Slash => BinaryInstr::Div,

            BinaryOp::Greater => BinaryInstr::Gt,
            BinaryOp::Less => BinaryInstr::Lt,
            BinaryOp::GreaterEqual => BinaryInstr::Ge,
            BinaryOp::LessEqual => BinaryInstr::Le,

            BinaryOp::EqualEqual => BinaryInstr::Eq,
            BinaryOp::BangEqual => BinaryInstr::Ne,
        };
        self.chunk.push_instr(Instruction::Bin(binary))
    }

    fn tuple(&mut self, exprs: Vec<Expr>) -> CompileResult<()> {
        let len = exprs.len() as u8;
        for expr in exprs {
            self.compile_expr(expr)?
        }
        self.chunk.push_instr(Instruction::Tuple { len })
    }

    fn access(&mut self, table: Expr, field: Expr) -> CompileResult<()> {
        self.compile_expr(table)?;
        let access_instr = match field {
            Expr::Literal(lit) => match lit {
                Literal::Str(string) => {
                    let index = self.chunk.add_constant(string.into())?;
                    Instruction::GetFieldImm { index }
                }
                _ => {
                    self.literal(lit)?;
                    Instruction::GetField
                }
            },
            expr => {
                self.compile_expr(expr)?;
                Instruction::GetField
            }
        };
        self.chunk.push_instr(access_instr)
    }

    fn set(&mut self, variable: Expr, value: Expr) -> CompileResult<()> {
        // TODO: pattern matching for tuple expressions

        match variable {
            Expr::Identifier(name) => {
                let index = self.chunk.add_constant(name.clone().into())?;
                self.compile_expr(value)?;
                if let Some((index, local)) = self.resolve_local(name.as_str()) {
                    let index = index as u16;
                    if local.is_in_function() {
                        self.chunk.push_instr(Instruction::SetFnLocal { index })
                    } else {
                        self.chunk.push_instr(Instruction::SetLocal { index })
                    }
                } else {
                    self.chunk.push_instr(Instruction::SetGlobal { index })
                }
            }
            Expr::Access { table, field } => {
                self.compile_expr(value)?;
                self.compile_expr(*field)?;
                self.compile_expr(*table)?;
                self.chunk.push_instr(Instruction::SetField)
            }
            _ => Err(CompileError::InvalidAssignmentTarget(variable)),
        }
    }

    fn table_init(&mut self, keys: Option<Vec<Expr>>, values: Vec<Expr>) -> CompileResult<()> {
        let len = values.len();
        let has_keys = match keys {
            Some(keys) => {
                for (k, v) in keys.into_iter().zip(values.into_iter()) {
                    self.compile_expr(k)?;
                    self.compile_expr(v)?;
                }
                true
            }
            None => {
                for value in values.into_iter().rev() {
                    self.compile_expr(value)?;
                }
                false
            }
        };
        let len = len as u16;
        self.chunk
            .push_instr(Instruction::InitTable { len, has_keys })
    }

    fn function_def(&mut self, args: Vec<String>, body: Vec<Statement>) -> CompileResult<()> {
        let patch_index = self.chunk.push_placeholder()?; // To prevent accidentally entering a function
        let args_len = args.len() as u8;
        let code_start = self.chunk.instructions().len() - 1;
        self.enter_function();
        for arg in args {
            self.push_local(arg);
        }
        for stmt in body {
            self.compile_stmt(stmt)?;
        }
        let pop_count = self.exit_function();
        for _ in 0..pop_count {
            self.chunk.push_instr(Instruction::Pop)?;
        }
        self.chunk.push_instr(Instruction::Return {
            return_value: false,
        })?;
        // Patch after inserting pop instructions
        let offset = self.chunk.instructions().len() - patch_index;
        self.chunk
            .patch_placeholder(patch_index, offset as i8, JumpCondition::None)?;
        self.chunk.push_instr(Instruction::FuncDef {
            args_len,
            code_start,
        })
    }

    fn call(&mut self, func: Expr, args: Vec<Expr>) -> CompileResult<()> {
        for arg in args {
            self.compile_expr(arg)?;
        }
        self.compile_expr(func)?;
        self.chunk.push_instr(Instruction::Call)
    }
}

/**
 * Locals and scoping
 */
impl Compiler {
    pub fn resolve_local(&self, name: &str) -> Option<(usize, Local)> {
        self.locals
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, l)| match l.name == name {
                true => {
                    if l.is_in_function() {
                        self.resolve_fn_local(name)
                    } else {
                        Some((i, l.clone()))
                    }
                }
                false => None,
            })
    }

    fn resolve_fn_local(&self, name: &str) -> Option<(usize, Local)> {
        self.locals
            .iter()
            .filter(|l| l.depth >= *self.closure_depths.last().unwrap())
            .enumerate()
            .find_map(|(i, l)| match l.name == name {
                true => Some((i, l.clone())),
                false => None,
            })
    }

    fn push_local(&mut self, name: String) {
        self.locals.push(Local {
            name,
            depth: self.depth,
            closure: match self.closure_depths.len() {
                0 => None,
                i => Some(i as u8 - 1),
            },
        })
    }

    fn scope_incr(&mut self) {
        self.depth += 1
    }

    fn enter_function(&mut self) {
        self.scope_incr();
        self.closure_depths.push(self.depth)
    }

    fn scope_decr(&mut self) -> usize {
        self.depth -= 1;
        let mut pop_count = 0;
        while self.locals.last().is_some() && self.locals.last().unwrap().depth > self.depth {
            self.locals.pop().unwrap();
            pop_count += 1;
        }
        pop_count
    }

    fn exit_function(&mut self) -> usize {
        self.closure_depths.pop().unwrap();
        self.scope_decr()
    }
}

impl Local {
    pub fn is_in_function(&self) -> bool {
        self.closure.is_some()
    }
}
