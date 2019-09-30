mod chunk;
mod error;
mod instruction;
mod io;

use std::convert::TryInto;
use crate::parser::{Parser, Ast, BinaryOp, Expr, BlockExpr, Literal, Statement, UnaryOp};
use crate::vm::{Value, Integer};
use crate::sourcefile::{SourceFile, MetaData};
pub use chunk::{Chunk, FuncProto, JumpCondition};
pub use error::CompileError;
pub use instruction::{BinaryInstr, Instruction, UnaryInstr};
use self::io::absolute_path;

pub type CompileResult<T> = Result<T, CompileError>;

pub struct Compiler {
    chunk: Chunk,
    locals: Vec<Local>,
    depth: u8,
    closure_scopes: Vec<ClosureScope>,
    metadata: MetaData,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Local {
    name: String,
    depth: u8,
    // n means the function in compiler.closure_scopes[n]
    closure: Option<u8>,
}

#[derive(Clone, Debug, PartialEq)]
struct ClosureScope {
    depth: u8,
    local_start: usize,
    instructions: Vec<Instruction>,
}

/**
 * Compiling
 */
impl Compiler {

    pub fn compile(
        SourceFile {
            ast,
            metadata,
        }: SourceFile
    ) -> CompileResult<Chunk> {
        let mut compiler = Self::new(metadata);
        compiler.compile_module(ast)?;
        Ok(compiler.chunk)
    }

    fn compile_module(&mut self, ast: Ast) -> CompileResult<()> {
        self.compile_ast(ast)?;
        self.add_instr(Instruction::Return {
            return_value: true,
        })
    }

    fn new(metadata: MetaData) -> Self {
        Compiler {
            chunk: Chunk::new(),
            locals: Vec::new(),
            depth: 0,
            closure_scopes: Vec::new(),
            metadata
        }
    }

    fn compile_ast(&mut self, ast: Ast) -> CompileResult<()> {
        self.enter_function();
        
        let body = ast.get_expr();
        self.func_body(body)?;

        let closure_scope = self.exit_function()?;
        self.define_func(closure_scope, 0, false)?;
        self.add_instr(Instruction::Call { args_len: 0 })
    }

    fn compile_stmt(&mut self, stmt: Statement) -> CompileResult<()> {
        match stmt {
            Statement::Expr(expr) => self.expr_stmt(expr),
            Statement::Var { name, value } => self.var_stmt(name, value),
            Statement::Let { name, value } => self.let_stmt(name, value),
            Statement::Set { variable, value } => self.set_stmt(variable, value),
            Statement::Block(statements) => self.block_stmt(statements),
            Statement::If {
                condition,
                then_block,
                else_block,
            } => self.if_stmt(condition, *then_block, else_block.map(|x| *x)),
            Statement::While {
                condition,
                then_block,
            } => self.while_stmt(condition, *then_block),
            Statement::Print(expr) => {
                self.compile_expr(expr)?;
                self.add_instr(Instruction::Print)
            }
            Statement::Return(expr) => {
                self.compile_expr(expr)?;
                self.add_instr(Instruction::Return { return_value: true })
            }
            Statement::Import { path, name } => self.import_stmt(path, name),
        }
    }

    fn expr_stmt(&mut self, expr: Expr) -> CompileResult<()> {
        self.compile_expr(expr)?;
        self.add_instr(Instruction::Pop)?;
        Ok(())
    }

    fn var_stmt(&mut self, name: String, expr: Expr) -> CompileResult<()> {
        let index = self.add_constant(name.into(), false)?;
        self.compile_expr(expr)?;
        self.add_instr(Instruction::SetGlobal { index })
    }

    fn let_stmt(&mut self, name: String, value: Expr) -> CompileResult<()> {
        match value {
            Expr::Function { .. } => {
                self.push_local(name);
                self.compile_expr(value)
            },
            _ => {
                self.compile_expr(value)?;
                self.push_local(name);
                Ok(())
            }
        }
    }

    fn set_stmt(&mut self, variable: Expr, value: Expr) -> CompileResult<()> {
        // TODO: pattern matching for tuple expressions
        match variable {
            Expr::Identifier(name) => {
                let index = self.add_constant(name.clone().into(), false)?;
                self.compile_expr(value)?;
                if let Some((index, frame)) = self.resolve_local(name.as_str()) {
                    let index = index as u16;
                    if frame > 1 {
                        // Since upvalues desugared by analyzer this should be impossible
                        unreachable!()
                    } else {
                        self.add_instr(Instruction::SetLocal { index, frame })
                    }
                } else {
                    self.add_instr(Instruction::SetGlobal { index })
                }
            }
            Expr::Access { table, field } => {
                self.compile_expr(value)?;
                self.compile_expr(*field)?;
                self.compile_expr(*table)?;
                self.add_instr(Instruction::SetField)
            }
            _ => Err(CompileError::InvalidAssignmentTarget(variable)),
        }?;
        Ok(())
    }

    fn block_stmt(&mut self, statements: Vec<Statement>) -> CompileResult<()> {
        self.enter_scope();
        for stmt in statements {
            self.compile_stmt(stmt)?;
        }
        self.exit_scope(false)
    }

    fn if_stmt(
        &mut self,
        condition: Expr,
        then_block: Expr,
        else_block: Option<Expr>,
    ) -> CompileResult<()> {
        self.compile_expr(condition)?;

        let patch_index = self.add_placeholder()?;
        self.compile_expr(then_block)?;

        let offset = self.get_offset(patch_index)?;
        if let Some(else_block) = else_block {
            // We need to patch old jump to one forward since now in this place there will be an unconditional jump
            self.patch_placeholder(patch_index, (offset + 1) as i8, JumpCondition::WhenFalse)?;

            let patch_index = self.add_placeholder()?;
            self.compile_expr(else_block)?;
            let offset = self.get_offset(patch_index)?;
            self.patch_placeholder(patch_index, offset as i8, JumpCondition::None)?;
        } else {
            self.patch_placeholder(patch_index, offset as i8, JumpCondition::WhenFalse)?;
        }
        Ok(())
    }

    fn while_stmt(&mut self, condition: Expr, then_block: Statement) -> CompileResult<()> {
        let start_index = self.instructions().len();
        self.compile_expr(condition)?;
        let patch_index = self.add_placeholder()?;
        self.compile_stmt(then_block)?;
        let offset = self.instructions().len() - patch_index + 1;
        self.patch_placeholder(patch_index, offset as i8, JumpCondition::WhenFalse)?;
        self.add_instr(Instruction::Jump {
            offset: -((self.instructions().len() - start_index) as i8),
        })
    }

    fn import_stmt(&mut self, path: Vec<String>, name: String) -> CompileResult<()> {
        use std::path::PathBuf;
        // Get source file
        let abs_path = if Self::is_std(&path) {
            let current_dir = PathBuf::from(file!());
            let src_dir = current_dir.parent().expect("Expected parent be 'src'");
            let mut lib_dir = PathBuf::from(src_dir);
            lib_dir.push("vm");
            lib_dir.push("lib");

            let last = path.len() - 2;
            for (i, p) in path.into_iter().skip(1).enumerate() {
                if i == last {
                    lib_dir.push(p + ".flux");
                } else {
                    lib_dir.push(p);
                }
            }
            println!("{:#?}", lib_dir);
            lib_dir
        } else {
            absolute_path(self.metadata.current_dir(), path.as_slice())
        };
        let source = io::read_file(abs_path.clone())?;
        // Parse and store
        let ast = Parser::parse_str(source.as_str())?;
        let metadata = MetaData {
            dir: abs_path.parent().expect("Expected a parent directory").to_owned(), 
        };
        // Compile the module
        let chunk = Compiler::compile(SourceFile { ast, metadata })
            .map_err(|error| CompileError::ModuleError {
                name: name.clone(),
                error: Box::new(error),
            })?;
        // Add import to table ad push instruction
        self.chunk.add_import(chunk, name)
    }

    #[inline]
    fn is_std(path: &[String]) -> bool {
        &path[0] == "std"
    }

    fn compile_expr(&mut self, expr: Expr) -> CompileResult<()> {
        #[allow(unreachable_patterns)]
        match expr {
            Expr::Literal(lit) => self.literal(lit),
            Expr::Identifier(name) => self.ident(name),
            Expr::Unary { op, expr } => self.unary(*expr, op),
            Expr::Binary { left, op, right } => self.binary(*left, *right, op),
            Expr::Grouping(expr) => self.compile_expr(*expr),
            Expr::Tuple(exprs) => self.tuple(exprs),
            Expr::Access { table, field } => self.access(*table, *field),
            Expr::SelfAccess { table, method, args } => self.self_access(*table, method, args),
            Expr::TableInit { keys, values } => self.table_init(keys, values),
            Expr::Function { args, body, env } => self.function_def(args, body, env),
            Expr::Call { func, args } => self.call(*func, args),
            Expr::Block(BlockExpr { stmts, expr }) => self.block_expr(stmts, *expr),
            Expr::If {
                condition,
                then_block,
                else_block,
            } => self.if_expr(*condition, *then_block, *else_block),
            Expr::Rec => self.add_instr(Instruction::Rec),
            _ => Err(CompileError::UnimplementedExpr(expr)),
        }
    }

    fn literal(&mut self, lit: Literal) -> CompileResult<()> {
        match lit {
            Literal::Nil => self.add_instr(Instruction::Nil),
            Literal::Bool(b) => self.add_instr(if b {
                Instruction::True
            } else {
                Instruction::False
            }),
            Literal::Number(n) => match n.fract() == 0.0 {
                true => self.int_literal(n as i64),
                false => self.add_constant(Value::Number(n), true).map(|_| ()),
            }
            Literal::Str(string) => {
                self.add_constant(string.into(), true)?;
                Ok(())
            }
            Literal::Unit => self.add_instr(Instruction::Unit),
        }
    }

    fn int_literal(&mut self, value: Integer) -> CompileResult<()> {
        if let Ok(value) = value.try_into() {
            self.add_instr(Instruction::Integer(value))
        } else {
            self.add_constant(Value::Int(value), true).map(|_| ())
        }
    }

    fn ident(&mut self, name: String) -> CompileResult<()> {
        if let Some((index, frame)) = self.resolve_local(name.as_str()) {
            let index = index as u16;
            if frame > 1 {
                // Since upvalues desugared by analyzer this should be impossible
                unreachable!();
            } else {
                self.add_instr(Instruction::GetLocal { index, frame })
            }
        } else {
            let index = self.add_constant(Value::new_str(name), false)?;
            self.add_instr(Instruction::GetGlobal { index })
            // TODO: make error if not repl
            // Err(CompileError::UndefinedVariable { name })
        }
    }

    fn unary(&mut self, expr: Expr, op: UnaryOp) -> CompileResult<()> {
        self.compile_expr(expr)?;
        let unary = match op {
            UnaryOp::Minus => UnaryInstr::Negate,
            UnaryOp::Bang => UnaryInstr::Not,
        };
        self.add_instr(Instruction::Unary(unary))
    }

    fn binary(&mut self, left: Expr, right: Expr, op: BinaryOp) -> CompileResult<()> {
        self.compile_expr(left)?;
        self.compile_expr(right)?;
        let binary = match op {
            BinaryOp::Plus => BinaryInstr::Add,
            BinaryOp::Minus => BinaryInstr::Sub,
            BinaryOp::Star => BinaryInstr::Mul,
            BinaryOp::Slash => BinaryInstr::Div,
            BinaryOp::Rem => BinaryInstr::Rem,

            BinaryOp::Greater => BinaryInstr::Gt,
            BinaryOp::Less => BinaryInstr::Lt,
            BinaryOp::GreaterEqual => BinaryInstr::Ge,
            BinaryOp::LessEqual => BinaryInstr::Le,

            BinaryOp::EqualEqual => BinaryInstr::Eq,
            BinaryOp::BangEqual => BinaryInstr::Ne,
        };
        self.add_instr(Instruction::Bin(binary))
    }

    fn tuple(&mut self, exprs: Vec<Expr>) -> CompileResult<()> {
        let len = exprs.len() as u8;
        for expr in exprs {
            self.compile_expr(expr)?
        }
        self.add_instr(Instruction::Tuple { len })
    }

    fn access(&mut self, table: Expr, field: Expr) -> CompileResult<()> {
        self.compile_expr(table)?;
        let access_instr = match field {
            Expr::Literal(lit) => match lit {
                Literal::Str(string) => {
                    let index = self.add_constant(string.into(), false)?;
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
        self.add_instr(access_instr)
    }

    fn self_access(&mut self, table: Expr, method: String, args: Vec<Expr>) -> CompileResult<()> {
        let index = self.add_constant(method.into(), false)?;
        let table_stack_index: u8 = args.len().try_into().unwrap();
        let args_len = (args.len() + 1).try_into().unwrap();

        self.compile_expr(table)?;
        self.compile_args(args)?;
        self.add_instr(Instruction::GetMethodImm { index, table_stack_index })?;
        self.add_instr(Instruction::Call { args_len })
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
        self.add_instr(Instruction::InitTable { len, has_keys })
    }

    // Compiles function definition
    fn function_def(&mut self, args: Vec<String>, body: BlockExpr, env: Option<(Vec<Expr>, Vec<Expr>)>) -> CompileResult<()> {
        // if env is some compile as table literal
        // define function that has env
        let has_env = 
            if let Some((keys, values)) = env {
                self.table_init(Some(keys), values)?;
                true
            } else {
                false
            };

        let args_len = args.len() as u8;
        self.enter_function();
        for arg in args {
            self.push_local(arg);
        }

        self.func_body(body)?;

        // Add new func proto
        let closure_scope = self.exit_function()?;
        self.define_func(closure_scope, args_len, has_env)
    }

    fn func_body(&mut self, body: BlockExpr) -> CompileResult<()> {
        for stmt in body.stmts {
            self.compile_stmt(stmt)?;
        }
        self.compile_expr(*body.expr)?;
        self.add_instr(Instruction::Return {
            return_value: true,
        })
    }

    // Creates function prototype
    fn define_func(&mut self, scope: ClosureScope, args_len: u8, has_env: bool) -> CompileResult<()> {
        let instructions = scope.instructions;
        let args_len = if has_env { args_len - 1 } else { args_len };
        let proto_index = self.chunk.push_proto(args_len, instructions).try_into().unwrap();
        self.add_instr(Instruction::FuncDef { proto_index, has_env })
    }

    fn call(&mut self, func: Expr, args: Vec<Expr>) -> CompileResult<()> {
        let args_len = args.len() as u8;
        self.compile_args(args)?;
        self.compile_expr(func)?;
        self.add_instr(Instruction::Call { args_len })
    }

    fn block_expr(&mut self, stmts: Vec<Statement>, expr: Expr) -> CompileResult<()> {
        self.enter_scope();
        for stmt in stmts {
            self.compile_stmt(stmt)?;
        }
        self.compile_expr(expr)?;
        self.exit_scope(true)
    }

    fn if_expr(
        &mut self,
        condition: Expr,
        then_block: Expr,
        else_block: Expr,
    ) -> CompileResult<()> {
        self.compile_expr(condition)?;

        let patch_index = self.add_placeholder()?;
        self.compile_expr(then_block)?;

        let offset = self.get_offset(patch_index)?;
        self.patch_placeholder(patch_index, (offset + 1) as i8, JumpCondition::WhenFalse)?;

        let patch_index = self.add_placeholder()?;
        self.compile_expr(else_block)?;
        let offset = self.get_offset(patch_index)?;
        self.patch_placeholder(patch_index, offset as i8, JumpCondition::None)?;

        Ok(())
    }

    fn compile_args(&mut self, args: Vec<Expr>) -> CompileResult<()> {
        for arg in args {
            self.compile_expr(arg)?;
        }
        Ok(())
    }
}

/**
 * Utility
 */

impl Compiler {
    fn add_instr(&mut self, instruction: Instruction) -> CompileResult<()> {
        self.instructions_mut().push(instruction);
        Ok(())
    }

    fn instructions(&self) -> &[Instruction] {
        match self.closure_scopes.last() {
            Some(closure_scope) => &closure_scope.instructions,
            None => self.chunk.instructions(),
        }
    }

    fn instructions_mut(&mut self) -> &mut Vec<Instruction> {
        match self.closure_scopes.last_mut() {
            Some(closure_scope) => &mut closure_scope.instructions,
            None => self.chunk.instructions_mut(),
        }
    }

    fn add_placeholder(&mut self) -> CompileResult<usize> {
        self.add_instr(Instruction::Placeholder)?;
        Ok(self.instructions().len() - 1)
    }

    fn patch_placeholder(
        &mut self,
        index: usize,
        jump_offset: i8,
        jump_cond: JumpCondition,
    ) -> CompileResult<()> {
        let offset = jump_offset;
        let instr = match jump_cond {
            JumpCondition::None => Instruction::Jump { offset },
            JumpCondition::WhenTrue => Instruction::JumpIf {
                when_true: true,
                offset,
            },
            JumpCondition::WhenFalse => Instruction::JumpIf {
                when_true: false,
                offset,
            },
        };
        match self.instructions()[index] {
            Instruction::Placeholder | Instruction::Jump { .. } | Instruction::JumpIf { .. } => {
                self.instructions_mut()[index] = instr;
                Ok(())
            }
            _ => Err(CompileError::WrongPatch(self.instructions()[index])),
        }
    }

    fn add_constant(&mut self, constant: Value, push_stack: bool) -> CompileResult<u8> {
        let index = self.chunk.add_constant(constant)?;
        if push_stack {
            self.add_instr(Instruction::Constant { index })?;
        }
        Ok(index)
    }

    #[inline]
    fn get_offset(&self, patch_index: usize) -> CompileResult<i8> {
        let offset = self.instructions().len() - patch_index;
        if offset > std::i8::MAX as usize {
            Err(CompileError::TooLongToJump)
        } else {
            Ok(offset as i8)
        }
    }
}

/**
 * Locals and scoping
 */
impl Compiler {
    // TODO: Write tests for local scoping
    // Returns the stack index and the frame offset
    // TODO: a local is now always at the same frame
    pub fn resolve_local(&self, name: &str) -> Option<(usize, u8)> {
        self.locals.iter().enumerate().rev().find_map(|(i, l)| {
            if l.name == name {
                let (offset, closure_depth) = match l.closure {
                    Some(i) => (
                        self.closure_scopes[i as usize].local_start,
                        self.closure_scopes.len() as u8 - i,
                    ),
                    None => (0, 0),
                };
                Some((i - offset, closure_depth))
            } else {
                None
            }
        })
    }

    fn push_local(&mut self, name: String) {
        self.locals.push(Local {
            name,
            depth: self.depth,
            closure: match self.closure_scopes.len() {
                0 => None,
                i => Some(i as u8 - 1),
            },
        })
    }

    fn enter_function(&mut self) {
        self.scope_incr();
        self.closure_scopes.push(ClosureScope {
            depth: self.depth,
            local_start: self.locals.len(),
            instructions: Vec::new(),
        })
    }

    fn exit_function(&mut self) -> CompileResult<ClosureScope> {
        let pop_count = self.scope_decr();
        for _ in 0..pop_count {
            self.add_instr(Instruction::Pop)?;
        } 
        Ok(self.closure_scopes.pop().unwrap())
    }

    fn enter_scope(&mut self) {
        self.scope_incr()
    }

    fn exit_scope(&mut self, return_value: bool) -> CompileResult<()> {
        let pop = self.scope_decr() as u16;
        self.add_instr(Instruction::ExitBlock { pop, return_value })
    }

    fn scope_incr(&mut self) {
        self.depth += 1
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
}
