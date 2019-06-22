mod chunk;
mod instruction;
mod error;

use std::rc::Rc;
pub use instruction::{Instruction, BinaryInstr, UnaryInstr};
pub use self::chunk::Chunk;
pub(self) use error::CompileError;
use crate::parser::{Expr, Literal, UnaryOp, BinaryOp};
use crate::vm::Value;

pub type CompileResult<T> = Result<T, CompileError>;

pub fn compile(expr: Expr) -> CompileResult<Chunk> {
    let mut chunk = Chunk::new();
    compile_impl(expr, &mut chunk)?;
    chunk.push_instr(Instruction::Return)?;
    Ok(chunk)
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
        _ => Err(CompileError::UnimplementedExpr(expr))
    }
}

fn literal(lit: Literal, chunk: &mut Chunk) -> CompileResult<()> {
    match lit {
        Literal::Nil => chunk.push_instr(Instruction::Nil),
        Literal::Bool(b) => {
            chunk.push_instr(match b {
                true => Instruction::True,
                false => Instruction::False,
            })
        }
        Literal::Number(n) => {
            match n.fract() == 0.0 {
                true => chunk.push_constant(Value::Int(n.trunc() as i32)),
                false => chunk.push_constant(Value::Number(n))
            }.map(|_| ())
        }
        Literal::Str(string) => {
            let index = chunk.add_constant(string.into())?;
            chunk.push_instr(Instruction::Constant { index })?;
            Ok(())
        }
    }
}

fn ident(name: String, chunk: &mut Chunk) -> CompileResult<()> {
    match chunk.has_string(&name) {
        Some(index) => chunk.push_instr(Instruction::Constant { index }),
        None => chunk.push_constant(Value::Str(Rc::new(name))).map(|_| ())
    }
}

fn unary(expr: Expr, op: UnaryOp, chunk: &mut Chunk) -> CompileResult<()> {
    compile_impl(expr, chunk)?;
    let unary = match op {
        UnaryOp::Minus => UnaryInstr::Negate,
        UnaryOp::Bang => UnaryInstr::Not
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
        Expr::Literal(lit) => { 
            match lit {
                Literal::Str(string) => {
                    let index = chunk.add_constant(string.into())?;
                    Instruction::GetFieldImm { index }
                },
                _ => {
                    literal(lit, chunk)?;
                    Instruction::GetField
                }
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
    compile_impl(value, chunk)?;
    match variable {
        Expr::Identifier(name) => {
            let index = chunk.add_constant(name.into())?;
            chunk.push_instr(Instruction::SetGlobal { index })
        }
        Expr::Access { table, field } => {
            compile_impl(*field, chunk)?;
            compile_impl(*table, chunk)?;
            chunk.push_instr(Instruction::SetField)
        }
        _ => Err(CompileError::InvalidAssignmentTarget(variable))
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
            for value in values {
                compile_impl(value, chunk)?;
            }
            false
        }
    };
    let len = len as u8;
    chunk.push_instr(Instruction::InitTable { len, has_keys })
}