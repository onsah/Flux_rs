mod chunk;
mod error;
mod instruction;

pub use self::chunk::{Chunk, JumpCondition};
use crate::parser::{BinaryOp, Expr, Literal, Statement, UnaryOp};
use crate::vm::Value;
pub(self) use error::CompileError;
pub use instruction::{BinaryInstr, Instruction, UnaryInstr};
use std::rc::Rc;

pub type CompileResult<T> = Result<T, CompileError>;

pub fn compile(stmts: Vec<Statement>) -> CompileResult<Chunk> {
    let mut chunk = Chunk::new();
    for stmt in stmts {
        compile_stmt(stmt, &mut chunk)?;
    }
    chunk.push_instr(Instruction::Return { return_value: false })?;
    Ok(chunk)
}

pub fn compile_stmt(stmt: Statement, chunk: &mut Chunk) -> CompileResult<()> {
    match stmt {
        Statement::Expr(expr) => expr_stmt(expr, chunk),
        Statement::Let { name, value } => let_stmt(name, value, chunk),
        Statement::Block(statements) => block_stmt(statements, chunk),
        Statement::If {
            condition,
            then_block,
            else_block,
        } => if_stmt(condition, then_block, else_block, chunk),
        Statement::While {
            condition,
            then_block,
        } => while_stmt(condition, then_block, chunk),
        Statement::Print(expr) => {
            compile_expr(expr, chunk)?;
            chunk.push_instr(Instruction::Print)
        }
        Statement::Return(expr) => {
            compile_expr(expr, chunk)?;
            chunk.push_instr(Instruction::Return { return_value: true })
        }
    }
}

fn expr_stmt(expr: Expr, chunk: &mut Chunk) -> CompileResult<()> {
    let is_assignment = match &expr {
        Expr::Set { .. } => true,
        _ => false,
    };
    compile_expr(expr, chunk)?;
    if !is_assignment {
        chunk.push_instr(Instruction::Pop)?;
    }
    Ok(())
}

fn let_stmt(name: String, value: Expr, chunk: &mut Chunk) -> CompileResult<()> {
    chunk.push_local(name);
    compile_expr(value, chunk)?;
    Ok(())
}

fn block_stmt(statements: Vec<Statement>, chunk: &mut Chunk) -> CompileResult<()> {
    chunk.scope_incr();
    for stmt in statements {
        compile_stmt(stmt, chunk)?;
    }
    for _ in 0..chunk.scope_decr() {
        chunk.push_instr(Instruction::Pop)?;
    }
    Ok(())
}

// TODO: Better PLEASE
fn if_stmt(
    condition: Expr,
    then_block: Box<Statement>,
    else_block: Option<Box<Statement>>,
    chunk: &mut Chunk,
) -> CompileResult<()> {
    compile_expr(condition, chunk)?;
    let patch_index = chunk.push_placeholder()?;
    compile_stmt(*then_block, chunk)?;
    let offset = chunk.instructions().len() - patch_index;
    if offset > std::i8::MAX as usize {
        return Err(CompileError::TooLongToJump);
    }
    chunk.patch_placeholder(patch_index, offset as i8, JumpCondition::WhenFalse)?;
    let has_else = if let Some(else_block) = else_block {
        let patch_index = chunk.push_placeholder()?;
        compile_stmt(*else_block, chunk)?;
        let offset = chunk.instructions().len() - patch_index;
        if offset > std::i8::MAX as usize {
            return Err(CompileError::TooLongToJump);
        }
        chunk.patch_placeholder(patch_index, offset as i8, JumpCondition::None)?;
        true
    } else {
        false
    };
    // Jump one further
    if has_else {
        chunk.patch_placeholder(patch_index, (offset + 1) as i8, JumpCondition::WhenFalse)?;
    }
    Ok(())
}

fn while_stmt(condition: Expr, then_block: Box<Statement>, chunk: &mut Chunk) -> CompileResult<()> {
    let start_index = chunk.instructions().len();
    compile_expr(condition, chunk)?;
    let patch_index = chunk.push_placeholder()?;
    compile_stmt(*then_block, chunk)?;
    let offset = chunk.instructions().len() - patch_index + 1;
    chunk.patch_placeholder(patch_index, offset as i8, JumpCondition::WhenFalse)?;
    chunk.push_instr(Instruction::Jump { 
        offset: -((chunk.instructions().len() - start_index) as i8)
    })
}

pub fn compile_expr(expr: Expr, chunk: &mut Chunk) -> CompileResult<()> {
    compile_impl(expr, chunk)
}

fn compile_impl(expr: Expr, chunk: &mut Chunk) -> CompileResult<()> {
    match expr {
        Expr::Literal(lit) => literal(lit, chunk),
        Expr::Identifier(name) => ident(name, chunk),
        Expr::Unary { op, expr } => unary(*expr, op, chunk),
        Expr::Binary { left, op, right } => binary(*left, *right, op, chunk),
        Expr::Grouping(expr) => compile_impl(*expr, chunk),
        Expr::Tuple(exprs) => tuple(exprs, chunk),
        Expr::Access { table, field } => access(*table, *field, chunk),
        Expr::Set { variable, value } => set(*variable, *value, chunk),
        Expr::TableInit { keys, values } => table_init(keys, values, chunk),
        Expr::Function { args, body } => function_def(args, body, chunk),
        Expr::Call { func, args } => call(*func, args, chunk),
        _ => Err(CompileError::UnimplementedExpr(expr)),
    }
}

fn literal(lit: Literal, chunk: &mut Chunk) -> CompileResult<()> {
    match lit {
        Literal::Nil => chunk.push_instr(Instruction::Nil),
        Literal::Bool(b) => chunk.push_instr(match b {
            true => Instruction::True,
            false => Instruction::False,
        }),
        Literal::Number(n) => match n.fract() == 0.0 {
            true => chunk.push_constant(Value::Int(n.trunc() as i32)),
            false => chunk.push_constant(Value::Number(n)),
        }
        .map(|_| ()),
        Literal::Str(string) => {
            let index = chunk.add_constant(string.into())?;
            chunk.push_instr(Instruction::Constant { index })?;
            Ok(())
        }
        Literal::Unit => chunk.push_instr(Instruction::Unit),
    }
}

fn ident(name: String, chunk: &mut Chunk) -> CompileResult<()> {
    if let Some((index, local)) = chunk.resolve_local(name.as_str()) {
        let index = index as u16;
        if local.in_function() {
            chunk.push_instr(Instruction::GetFnLocal { index })
        } else {
            chunk.push_instr(Instruction::GetLocal { index })
        }
    } else {
        if let Some(index) = chunk.has_string(name.as_str()) {
            chunk.push_instr(Instruction::GetGlobal { index })
        } else {
            Err(CompileError::UndefinedVariable { name })
        }
    }
}

fn unary(expr: Expr, op: UnaryOp, chunk: &mut Chunk) -> CompileResult<()> {
    compile_impl(expr, chunk)?;
    let unary = match op {
        UnaryOp::Minus => UnaryInstr::Negate,
        UnaryOp::Bang => UnaryInstr::Not,
    };
    chunk.push_instr(Instruction::Unary(unary))
}

fn binary(left: Expr, right: Expr, op: BinaryOp, chunk: &mut Chunk) -> CompileResult<()> {
    compile_impl(left, chunk)?;
    compile_impl(right, chunk)?;
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
    chunk.push_instr(Instruction::Bin(binary))
}

fn tuple(exprs: Vec<Expr>, chunk: &mut Chunk) -> CompileResult<()> {
    let len = exprs.len() as u8;
    for expr in exprs {
        compile_impl(expr, chunk)?
    }
    chunk.push_instr(Instruction::Tuple { len })
}

fn access(table: Expr, field: Expr, chunk: &mut Chunk) -> CompileResult<()> {
    compile_impl(table, chunk)?;
    let access_instr = match field {
        Expr::Literal(lit) => match lit {
            Literal::Str(string) => {
                let index = chunk.add_constant(string.into())?;
                Instruction::GetFieldImm { index }
            }
            _ => {
                literal(lit, chunk)?;
                Instruction::GetField
            }
        },
        expr => {
            compile_impl(expr, chunk)?;
            Instruction::GetField
        }
    };
    chunk.push_instr(access_instr)
}

fn set(variable: Expr, value: Expr, chunk: &mut Chunk) -> CompileResult<()> {
    // TODO: pattern matching for tuple expressions
    
    match variable {
        Expr::Identifier(name) => {
            let index = chunk.add_constant(name.clone().into())?;   
            compile_impl(value, chunk)?;
            if let Some((index, local)) = chunk.resolve_local(name.as_str()) {
                let index = index as u16;
                if local.in_function() {
                    chunk.push_instr(Instruction::SetFnLocal { index })
                } else {
                    chunk.push_instr(Instruction::SetLocal { index })
                }
            } else {
                chunk.push_instr(Instruction::SetGlobal { index })
            }
        }
        Expr::Access { table, field } => {
            compile_impl(value, chunk)?;
            compile_impl(*field, chunk)?;
            compile_impl(*table, chunk)?;
            chunk.push_instr(Instruction::SetField)
        }
        _ => Err(CompileError::InvalidAssignmentTarget(variable)),
    }
}

fn table_init(keys: Option<Vec<Expr>>, values: Vec<Expr>, chunk: &mut Chunk) -> CompileResult<()> {
    let len = values.len();
    let has_keys = match keys {
        Some(keys) => {
            for (k, v) in keys.into_iter().zip(values.into_iter()) {
                compile_impl(k, chunk)?;
                compile_impl(v, chunk)?;
            }
            true
        }
        None => {
            for value in values.into_iter().rev() {
                compile_impl(value, chunk)?;
            }
            false
        }
    };
    let len = len as u16;
    chunk.push_instr(Instruction::InitTable { len, has_keys })
}

fn function_def(args: Vec<String>, body: Vec<Statement>, chunk: &mut Chunk) -> CompileResult<()> {
    let patch_index = chunk.push_placeholder()?;  // To prevent accidentally entering a function
    let args_len = args.len() as u8;
    let code_start = chunk.instructions().len() - 1;
    chunk.enter_function();
    for arg in args {
        chunk.push_local(arg);
    }
    for stmt in body {
        compile_stmt(stmt, chunk)?;
    }
    let pop_count = chunk.exit_function();
    for _ in 0..pop_count {
        chunk.push_instr(Instruction::Pop)?;
    }
    chunk.push_instr(Instruction::Return { return_value: false })?;
    // Patch after inserting pop instructions
    let offset = chunk.instructions().len() - patch_index;
    chunk.patch_placeholder(patch_index, offset as i8, JumpCondition::None)?;
    chunk.push_instr(Instruction::FuncDef { args_len, code_start })
} 

fn call(func: Expr, args: Vec<Expr>, chunk: &mut Chunk) -> CompileResult<()> {
    for arg in args {
        compile_expr(arg, chunk)?;
    }
    compile_expr(func, chunk)?;
    chunk.push_instr(Instruction::Call)
}