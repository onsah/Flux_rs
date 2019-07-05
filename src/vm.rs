mod error;
mod frame;
mod natives;
#[cfg(test)]
mod tests;
mod value;

use crate::compiler::{BinaryInstr, Chunk, Instruction, UnaryInstr, FuncProto};
pub use error::RuntimeError;
use frame::Frame;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
pub use value::{Function, ArgsLen, Table, Value, UpValue};
pub use natives::PREDEFINED_CONSTANTS;

pub type RuntimeResult<T> = Result<T, RuntimeError>;

pub struct Vm {
    frames: Vec<Frame>,
    stack: Vec<Value>,
    globals: HashMap<Value, Value>,
    current_chunk: Option<Chunk>,
}

impl Vm {
    pub fn new() -> Self {
        Vm {
            frames: Vec::new(),
            stack: Vec::new(),
            current_chunk: None,
            globals: PREDEFINED_CONSTANTS.iter()
                .map(|(s, f)| (Value::Embedded(s), f.clone()))
                .collect(),
        }
    }

    pub fn run(&mut self, chunk: Chunk) -> RuntimeResult<Value> {
        self.current_chunk = Some(chunk);
        self.init_call();
        self.execute()
    }

    #[inline]
    fn init_call(&mut self) {
        let frame = Frame::default();
        self.frames.push(frame)
    }

    fn execute(&mut self) -> RuntimeResult<Value> {
        loop {
            let instr = self.next_instr()?;
            match instr {
                Instruction::Nil => self.stack.push(Value::Nil),
                Instruction::Unit => self.stack.push(Value::Unit),
                Instruction::True => self.stack.push(Value::Bool(true)),
                Instruction::False => self.stack.push(Value::Bool(false)),
                Instruction::Constant { index } => {
                    let value = self.current_chunk().constants()[index as usize].clone();
                    self.stack.push(value)
                }
                Instruction::Pop => {
                    self.pop_stack()?;
                }
                Instruction::Return { return_value } => {
                    // println!("Call Stack:\n{:?}", self.frames);
                    let mut value = match return_value {
                        true => self.pop_stack()?,
                        false => Value::Unit,
                    };
                    match &mut value {
                        Value::Function(function) => {
                            if let Function::User(func) = function {
                                let frame = self.current_frame().unwrap();
                                for upval in func.upvalues_mut() {
                                    let close = match upval {
                                        UpValue::Open { index } => {
                                            match frame.upvalues[*index as usize] {
                                                UpValue::This { index } => Some(index),
                                                _ => None,
                                            }
                                        },
                                        _ => None,
                                    };
                                    if let Some(index) = close {
                                        let index = frame.stack_top + index as usize;
                                        let value = self.stack[index].clone();
                                        *upval = UpValue::Closed(value);
                                    }
                                }
                            }
                        },
                        _ => (),
                    } 
                    while self.stack.len() > self.current_frame()?.stack_top() {
                        self.pop_stack()?;
                    }
                    self.stack.push(value);
                    self.frames.pop().unwrap();
                    if self.frames.is_empty() {
                        return self.pop_stack();
                    }
                }
                Instruction::Bin(bin) => self.binary(bin)?,
                Instruction::Unary(unary) => self.unary(unary)?,
                Instruction::GetGlobal { index } => {
                    let name = &self.current_chunk().constants()[index as usize];
                    match self.globals.get(name) {
                        Some(value) => self.stack.push(value.clone()),
                        None => {
                            return Err(RuntimeError::UndefinedVariable {
                                name: name.to_string(),
                            })
                        }
                    }
                }
                Instruction::SetGlobal { index } => {
                    let name = self.current_chunk().constants()[index as usize].clone();
                    let value = self.stack.pop().unwrap().clone();
                    self.globals.insert(name, value);
                }
                Instruction::GetLocal { index, frame } => {
                    let frame_index = if frame != 0 { self.frames.len() - frame as usize } else { 0 };
                    let index = self.frames[frame_index].stack_top() + index as usize;
                    self.stack.push(self.stack[index as usize].clone());
                }
                Instruction::SetLocal { index, frame } => {
                    let frame_index = if frame != 0 { self.frames.len() - frame as usize } else { 0 };
                    let index = self.frames[frame_index].stack_top() + index as usize;
                    if self.stack.len() != index as usize {
                        self.stack[index as usize] = self.pop_stack()?;
                    }
                }
                Instruction::Jump { offset } => self.jump(offset)?,
                Instruction::JumpIf { offset, when_true } => {
                    let value = self.pop_stack()?;
                    if value.to_bool() == when_true {
                        self.jump(offset)?;
                    }
                }
                Instruction::InitTable { len, has_keys } => self.init_table(len, has_keys)?,
                Instruction::GetField => self.get_field()?,
                Instruction::GetFieldImm { index } => self.get_field_imm(index)?,
                Instruction::SetField => self.set_field()?,
                Instruction::SetFieldImm { index } => self.set_field_imm(index)?,
                Instruction::Print => {
                    let value = self.pop_stack()?;
                    println!("{}", value)
                }
                Instruction::Tuple { len } => {
                    let mut values = Vec::with_capacity(len as usize);
                    for _ in 0..len {
                        values.push(self.pop_stack()?)
                    }
                    let tuple = Value::Tuple(values.into_iter().rev().collect());
                    self.stack.push(tuple)
                }
                Instruction::FuncDef { proto_index } => {
                    let FuncProto {
                        code_start,
                        args_len,
                        upvalues
                    } = &self.current_chunk().prototypes()[proto_index];
                    self
                    .stack
                    .push(Value::Function(Function::new_user(*args_len, *code_start, upvalues)))
                },
                Instruction::Call { args_len } => {
                    let function = self.pop_stack()?;
                    match function {
                        Value::Function(function) => self.call(function, args_len)?,
                        _ => return Err(RuntimeError::TypeError),
                    }
                }
                Instruction::GetUpval { index } => {
                    let mut upval_index = index;
                    for frame in self.frames.iter().rev() {
                        let upvalue = &frame.upvalues[upval_index as usize];
                        match upvalue {
                            UpValue::Open { index } => {
                                upval_index = *index;
                            }
                            UpValue::This { index } => {
                                let value_index = frame.stack_top + *index as usize;
                                self.stack.push(self.stack[value_index].clone());
                                break;
                            }
                            UpValue::Closed(value) => {
                                self.stack.push(value.clone());
                                break;
                            }
                        }
                    }
                }
                _ => return Err(RuntimeError::UnsupportedInstruction(instr)),
            }
            let f = self.current_frame_mut()?;
            f.pc += 1;
            // self.print_call_stack();
            // self.print_stack()
        }
    }

    fn get_field(&mut self) -> RuntimeResult<()> {
        let key = self.pop_stack()?;
        let table = self.pop_stack()?;
        match table {
            Value::Table(rc) => {
                let table = rc.borrow_mut();
                let value = table.get(&key).clone();
                self.stack.push(value);
                Ok(())
            }
            _ => Err(RuntimeError::TypeError),
        }
    }

    fn get_field_imm(&mut self, index: u8) -> RuntimeResult<()> {
        let table = self.pop_stack()?;
        let key = &self.current_chunk().constants()[index as usize];
        match table {
            Value::Table(rc) => {
                let table = rc.borrow_mut();
                let value = table.get(&key).clone();
                self.stack.push(value);
                Ok(())
            }
            _ => Err(RuntimeError::TypeError),
        }
    }

    fn set_field(&mut self) -> RuntimeResult<()> {
        let table = self.pop_stack()?;
        let key = self.pop_stack()?;
        let value = self.pop_stack()?;
        match table {
            Value::Table(rc) => {
                let mut table = rc.borrow_mut();
                table.set(key, value);
                Ok(())
            }
            _ => Err(RuntimeError::TypeError),
        }
    }

    fn set_field_imm(&mut self, index: u8) -> RuntimeResult<()> {
        let value = self.pop_stack()?;
        let table = self.pop_stack()?;
        let key = &self.current_chunk().constants()[index as usize];
        match table {
            Value::Table(rc) => {
                let mut table = rc.borrow_mut();
                table.set(key.clone(), value);
                Ok(())
            }
            _ => Err(RuntimeError::TypeError),
        }
    }

    fn init_table(&mut self, len: u16, has_keys: bool) -> RuntimeResult<()> {
        let table = match has_keys {
            true => {
                let mut table = Table::new();
                for _ in 0..len {
                    let value = self.pop_stack()?;
                    let key = self.pop_stack()?;
                    table.set(key, value)
                }
                table
            }
            false => {
                let mut values = Vec::new();
                for i in 0..len {
                    values.push((Value::Int(i as i32), self.pop_stack()?))
                }
                Table::from_array(values)
            }
        };
        self.stack.push(Value::Table(Rc::new(RefCell::new(table))));
        Ok(())
    }

    fn jump(&mut self, offset: i8) -> RuntimeResult<()> {
        let f = self.current_frame_mut()?;
        if offset > 0 {
            f.pc += (offset - 1) as usize
        } else {
            f.pc -= (-offset + 1) as usize
        }
        Ok(())
    }

    fn call(&mut self, function: Function, pushed_args: u8) -> RuntimeResult<()> {
        match function {
            Function::User(function) => {
                if pushed_args == function.args_len() {
                    let pc = function.code_start();
                    let stack_top = self.stack.len() - function.args_len() as usize;
                    let upvalues = function.extract_upvalues();
                    self.frames.push(Frame::new(pc, stack_top, upvalues));
                } else {
                    return Err(RuntimeError::WrongNumberOfArgs {
                        expected: function.args_len(),
                        found: pushed_args,
                    });
                }
            },
            Function::Native(native_fn) => {
                let mut args = Vec::new();
                match native_fn.args_len() {
                    ArgsLen::Variadic => {
                        for _ in 0..pushed_args {
                            args.push(self.pop_stack()?);
                        }
                    }
                    ArgsLen::Exact(n) => {
                        if n == pushed_args {
                            for _ in 0..pushed_args {
                                args.push(self.pop_stack()?);
                            }
                        } else {
                            return Err(RuntimeError::WrongNumberOfArgs {
                                expected: n,
                                found: pushed_args,
                            });
                        }
                    }
                }
                self.stack.push((native_fn.function)(args)?)
            }
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
                (Value::Int(a), Value::Int(b)) => Ok({
                    if op.is_arithmetic() {
                        match op {
                            BinaryInstr::Add => Value::Int(a + b),
                            BinaryInstr::Sub => Value::Int(a - b),
                            BinaryInstr::Mul => Value::Int(a * b),
                            BinaryInstr::Div => match b {
                                0 => return Err(RuntimeError::DivideByZero),
                                n if a % n == 0 => Value::Int(a / b),
                                _ => Value::Number(a as f64 / b as f64),
                            },
                            _ => unreachable!(),
                        }
                    } else {
                        match op {
                            BinaryInstr::Gt => Value::Bool(a > b),
                            BinaryInstr::Lt => Value::Bool(a < b),
                            BinaryInstr::Ge => Value::Bool(a >= b),
                            BinaryInstr::Le => Value::Bool(a <= b),
                            _ => unreachable!(),
                        }
                    }
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
        // println!("pc: {}, instr: {:?}", f.pc, instr);
        Ok(instr)
    }

    fn current_frame(&self) -> RuntimeResult<&Frame> {
        match self.frames.last() {
            Some(frame) => Ok(frame),
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

    fn print_call_stack(&self) {
        println!("**********Call stack**********");
        for frame in &self.frames {
            println!("{:?}", frame);
        }
    }

    fn print_stack(&self) {
        println!("**********STACK LEN: {}**********", self.stack.len());
        for value in self.stack.iter() {
            println!("{}", value)
        }
        println!("**********STACK END**********");
    }
}
