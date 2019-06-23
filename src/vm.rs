mod error;
mod frame;
mod value;

use crate::compiler::{BinaryInstr, Chunk, Instruction, UnaryInstr};
pub use error::RuntimeError;
use frame::Frame;
use std::collections::HashMap;
use std::rc::Rc;
pub use value::Value;

pub type RuntimeResult<T> = Result<T, RuntimeError>;

pub struct Vm {
    frames: Vec<Frame>,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    current_chunk: Option<Chunk>,
}

impl Vm {
    pub fn new() -> Self {
        Vm {
            frames: Vec::new(),
            stack: Vec::new(),
            globals: HashMap::new(),
            current_chunk: None,
        }
    }

    pub fn run(&mut self, chunk: Chunk) -> RuntimeResult<Value> {
        self.current_chunk = Some(chunk);
        self.init_call();
        self.execute()
    }

    #[inline]
    fn init_call(&mut self) {
        let frame = Frame::new(0, None);
        self.frames.push(frame)
    }

    fn execute(&mut self) -> RuntimeResult<Value> {
        loop {
            let instr = self.next_instr()?;
            match instr {
                Instruction::Nil => self.stack.push(Value::Nil),
                Instruction::True => self.stack.push(Value::Bool(true)),
                Instruction::False => self.stack.push(Value::Bool(false)),
                Instruction::Constant { index } => {
                    let value = self.current_chunk().constants()[index as usize].clone();
                    self.stack.push(value)
                }
                Instruction::Pop => {
                    self.pop_stack()?;
                }
                Instruction::Return => match self.stack.pop() {
                    Some(value) => return Ok(value),
                    None => return Ok(Value::Nil),
                },
                Instruction::Bin(bin) => self.binary(bin)?,
                Instruction::Unary(unary) => self.unary(unary)?,
                Instruction::GetGlobal { index } => {
                    let name = self.current_chunk().constants()[index as usize].as_str()?;
                    match self.globals.get(name) {
                        Some(value) => self.stack.push(value.clone()),
                        None => return Err(RuntimeError::UndefinedVariable { name: name.to_string() }),
                    }
                }
                Instruction::SetGlobal { index } => {
                    let name = self.current_chunk().constants()[index as usize].as_str()?.to_string();
                    let value = self.stack.last().unwrap().clone();
                    self.globals.insert(name, value);
                }
                Instruction::GetLocal { index } => {
                    self.stack.push(self.stack[index as usize].clone());
                }
                Instruction::SetLocal { index } => {
                    self.stack[index as usize] = self.stack.last().unwrap().clone();
                }
                Instruction::Jump { offset } => self.jump(offset)?,
                Instruction::JumpIf { offset, when_true } => {
                    let value = self.pop_stack()?;
                    if value.to_bool() == when_true {
                         self.jump(offset)?;
                    }
                }
                _ => return Err(RuntimeError::UnsupportedInstruction(instr)),
            }
            let f = self.current_frame_mut()?;
            f.pc += 1;
        }
    }

    fn jump(&mut self, offset: i8) -> RuntimeResult<()> {
        let f = self.current_frame_mut()?;
        if offset > 0 {
            f.pc += (offset - 1) as usize
        } else {
            f.pc -= (-offset - 1) as usize
        }
        Ok(())
    }

    fn binary(&mut self, op: BinaryInstr) -> RuntimeResult<()> {
        let right = self.pop_stack()?;
        let left = self.pop_stack()?;
        if op == BinaryInstr::Eq {
            self.stack.push(Value::Bool(left == right));
        } else if op == BinaryInstr::Ne {
            self.stack.push(Value::Bool(left != right));
        } else {
            let new_value = match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(match op {
                    BinaryInstr::Add => Value::Number(a + b),
                    BinaryInstr::Sub => Value::Number(a - b),
                    BinaryInstr::Mul => Value::Number(a * b),
                    BinaryInstr::Div => Value::Number(a / b),

                    BinaryInstr::Gt => Value::Bool(a > b),
                    BinaryInstr::Lt => Value::Bool(a < b),
                    BinaryInstr::Ge => Value::Bool(a >= b),
                    BinaryInstr::Le => Value::Bool(a <= b),
                    _ => unreachable!(),
                }),
                (Value::Number(a), Value::Int(b)) => Ok(match op {
                    BinaryInstr::Add => Value::Number(a + b as f64),
                    BinaryInstr::Sub => Value::Number(a - b as f64),
                    BinaryInstr::Mul => Value::Number(a * b as f64),
                    BinaryInstr::Div => Value::Number(a / b as f64),

                    BinaryInstr::Gt => Value::Bool(a > b as f64),
                    BinaryInstr::Lt => Value::Bool(a < b as f64),
                    BinaryInstr::Ge => Value::Bool(a >= b as f64),
                    BinaryInstr::Le => Value::Bool(a <= b as f64),
                    _ => unreachable!(),
                }),
                (Value::Int(a), Value::Int(b)) => Ok(match op {
                    BinaryInstr::Add => Value::Int(a + b),
                    BinaryInstr::Sub => Value::Int(a - b),
                    BinaryInstr::Mul => Value::Int(a * b),
                    BinaryInstr::Div => Value::Int(a / b),

                    BinaryInstr::Gt => Value::Bool(a > b),
                    BinaryInstr::Lt => Value::Bool(a < b),
                    BinaryInstr::Ge => Value::Bool(a >= b),
                    BinaryInstr::Le => Value::Bool(a <= b),
                    _ => unreachable!(),
                }),
                (Value::Str(a), Value::Str(b)) => match op {
                    BinaryInstr::Add => {
                        let mut new_string = String::with_capacity(a.len() + b.len());
                        new_string.extend(a.chars());
                        new_string.extend(b.chars());
                        Ok(Value::Str(Rc::new(new_string)))
                    }
                    _ => Err(RuntimeError::TypeError),
                },
                (value, _) => Err(RuntimeError::UnsupportedBinary { value, op }),
            }?;
            self.stack.push(new_value);
        }
        Ok(())
    }

    fn unary(&mut self, op: UnaryInstr) -> RuntimeResult<()> {
        let value = self.pop_stack()?;
        match op {
            UnaryInstr::Negate => match value {
                Value::Int(i) => self.stack.push(Value::Int(-i)),
                Value::Number(f) => self.stack.push(Value::Number(-f)),
                _ => return Err(RuntimeError::TypeError),
            },
            UnaryInstr::Not => match value {
                Value::Bool(b) => self.stack.push(Value::Bool(!b)),
                _ => return Err(RuntimeError::TypeError),
            },
        }
        Ok(())
    }

    fn next_instr(&mut self) -> RuntimeResult<Instruction> {
        let f = self.current_frame()?;
        let instr = self.current_chunk().instructions()[f.pc];
        Ok(instr)
    }

    fn current_frame(&self) -> RuntimeResult<Frame> {
        match self.frames.last() {
            Some(frame) => Ok(*frame),
            None => Err(RuntimeError::EmptyFrame),
        }
    }

    fn current_frame_mut(&mut self) -> RuntimeResult<&mut Frame> {
        match self.frames.last_mut() {
            Some(frame) => Ok(frame),
            None => Err(RuntimeError::EmptyFrame),
        }
    }

    #[inline]
    fn current_chunk(&self) -> &Chunk {
        self.current_chunk.as_ref().unwrap()
    }

    #[inline]
    fn current_chunk_mut(&mut self) -> &mut Chunk {
        self.current_chunk.as_mut().unwrap()
    }

    fn pop_stack(&mut self) -> RuntimeResult<Value> {
        match self.stack.pop() {
            Some(value) => Ok(value),
            None => Err(RuntimeError::EmptyStack),
        }
    }
}
