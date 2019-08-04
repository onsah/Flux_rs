mod error;
mod frame;
pub mod lib;
#[cfg(test)]
mod tests;
mod value;

use crate::compiler::{BinaryInstr, Chunk, Instruction, UnaryInstr};
pub use error::RuntimeError;
use frame::Frame;
pub use lib::PREDEFINED_CONSTANTS;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
pub use value::{
    ArgsLen, Float, Function, Integer, NativeFunction, Table, UpValue, UserFunction, Value,
};

pub type RuntimeResult<T> = Result<T, RuntimeError>;

#[derive(Debug, Clone, PartialEq)]
pub struct Vm {
    frames: Vec<Frame>,
    stack: Vec<Value>,
    globals: HashMap<Value, Value>,
    current_chunk: Option<Chunk>,
}

impl Vm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self, chunk: Chunk) -> RuntimeResult<Value> {
        self.current_chunk = Some(chunk);
        self.init_call();
        loop {
            self.execute()?;
            if self.frames.is_empty() {
                return Ok(self.pop_stack()?);
            }
            let f = self.current_frame_mut()?;
            f.pc += 1;
        }
    }

    #[inline]
    fn init_call(&mut self) {
        let frame = Frame::default();
        self.frames.push(frame)
    }

    fn execute(&mut self) -> RuntimeResult<()> {
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
                    // TODO: fn return
                    let mut value = if return_value {
                        self.pop_stack()?
                    } else {
                        Value::Unit
                    };
                    if let Value::Function(function) = &mut value {
                        if let Function::User(func) = function {
                            let frame = self.current_frame().unwrap();
                            // Closure upvalues that will be dropped
                            for upval in func.upvalues_mut() {
                                if let UpValue::Open { index } = upval {
                                    match &frame.upvalues[*index as usize] {
                                        UpValue::This { index } => {
                                            let index = frame.stack_top + *index as usize;
                                            let value = self.stack[index].clone();
                                            *upval = UpValue::Closed(value);
                                        }
                                        // Note: can be moved somehow since it will be dropped immediately
                                        UpValue::Closed(value) => {
                                            *upval = UpValue::Closed(value.clone());
                                        }
                                        _ => (),
                                    }
                                }
                            }
                        }
                    }
                    while self.stack.len() > self.current_frame()?.stack_top() {
                        self.pop_stack()?;
                    }
                    self.stack.push(value);
                    self.frames.pop().unwrap();
                    // self.print_stack();
                    return Ok(());
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
                    let frame_index = if frame != 0 {
                        self.frames.len() - frame as usize
                    } else {
                        0
                    };
                    let index = self.frames[frame_index].stack_top() + index as usize;
                    self.stack.push(self.stack[index as usize].clone());
                }
                Instruction::SetLocal { index, frame } => {
                    let frame_index = if frame != 0 {
                        self.frames.len() - frame as usize
                    } else {
                        0
                    };
                    let index = self.frames[frame_index].stack_top() + index as usize;
                    if self.stack.len() != index as usize {
                        self.stack[index as usize] = self.pop_stack()?;
                    }
                }
                Instruction::Jump { offset } => self.jump(offset)?,
                Instruction::JumpIf { offset, when_true } => {
                    let value = self.pop_stack()?;
                    if value.as_bool() == when_true {
                        self.jump(offset)?;
                    }
                }
                Instruction::InitTable { len, has_keys } => self.init_table(len, has_keys)?,
                Instruction::GetField => self.get_field()?,
                Instruction::GetFieldImm { index } => self.get_field_imm(index)?,
                Instruction::GetMethodImm { index } => self.get_method_imm(index)?,
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
                    let proto = self.current_chunk().prototypes()[proto_index].clone();
                    self.stack
                        .push(Value::Function(Function::new_user(&proto, proto_index)))
                }
                Instruction::Call { args_len } => {
                    let function = self.pop_stack()?;
                    match function {
                        Value::Function(function) => {
                            self.call(function, args_len)?;
                            continue; // Don't increment pc
                        }
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
            self.print_call_stack();
            self.print_stack()
        }
    }

    // TODO: look recursively for '__class__' attribute when something is returns nil
    fn get_table(key: &Value, table: &Value) -> RuntimeResult<Value> {
        match table {
            Value::Table(rc) => {
                let table = rc.borrow_mut();
                let value = match table.get(&key) {
                    Value::Nil => match table.klass() {
                        Value::Nil => Value::Nil,
                        value => Self::get_table(key, value)?,
                    },
                    value => value.clone(),
                };
                Ok(value)
            }
            _ => Err(RuntimeError::TypeError),
        }
    }

    fn get_field(&mut self) -> RuntimeResult<()> {
        let key = self.pop_stack()?;
        let table = self.pop_stack()?;
        let value = Self::get_table(&key, &table)?;
        self.stack.push(value);
        Ok(())
    }

    fn get_field_imm(&mut self, index: u8) -> RuntimeResult<()> {
        let table = self.pop_stack()?;
        let key = &self.current_chunk().constants()[index as usize];
        let value = Self::get_table(key, &table)?;
        self.stack.push(value);
        Ok(())
    }

    fn get_method_imm(&mut self, index: u8) -> RuntimeResult<()> {
        // let field = self.get_field_imm(index)?;
        let table = self.pop_stack()?;
        let key = &self.current_chunk().constants()[index as usize];
        let value = Self::get_table(&key, &table)?;
        match table {
            Value::Table(rc) => {
                self.stack.push(value.into_user_fn()?.with_this(rc).into());
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
        let table = if has_keys {
            let mut table = Table::new();
            for _ in 0..len {
                let value = self.pop_stack()?;
                let key = self.pop_stack()?;
                table.set(key, value)
            }
            table
        } else {
            let mut values = Vec::new();
            for i in 0..len {
                values.push((Value::Int(i as Integer), self.pop_stack()?))
            }
            Table::from_array(values)
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
            Function::User(function) => self.call_user(function, pushed_args),
            Function::Native(native_fn) => self.call_native(native_fn, pushed_args),
        }
    }

    fn call_user(&mut self, mut function: UserFunction, pushed_args: u8) -> RuntimeResult<()> {
        if pushed_args == function.args_len() {
            let proto_index = function.proto_index();
            let stack_top = self.stack.len() - function.args_len() as usize;
            if let Some(this) = function.take_this() {
                self.stack.push(this.into())
            }
            let upvalues = function.extract_upvalues();
            self.frames
                .push(Frame::new(0, proto_index, stack_top, upvalues));
            Ok(())
        } else {
            Err(RuntimeError::WrongNumberOfArgs {
                expected: function.args_len(),
                found: pushed_args,
            })
        }
    }

    fn call_user_blocking(&mut self, function: UserFunction, pushed_args: u8) -> RuntimeResult<()> {
        self.call_user(function, pushed_args)?;
        // If we don't have these lines we stuck in loop because jump instructions consider incrementing
        /* let f = self.current_frame_mut()?;
        f.pc += 1; */
        self.execute()
    }

    fn call_native(&mut self, native_fn: NativeFunction, pushed_args: u8) -> RuntimeResult<()> {
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
        let value = (native_fn.function)(self, args)?;
        self.stack.push(value);
        let f = self.current_frame_mut()?;
        f.pc += 1;
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
        let instr = self.instructions()?[f.pc];
        debug!("pc: {}, instr: {:?}", f.pc, instr);
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

    fn instructions(&self) -> RuntimeResult<&[Instruction]> {
        Ok(match self.current_frame()?.proto_index {
            Some(index) => self.current_chunk().prototypes()[index]
                .instructions
                .as_ref(),
            None => self.current_chunk().instructions(),
        })
    }

    fn pop_stack(&mut self) -> RuntimeResult<Value> {
        match self.stack.pop() {
            Some(value) => Ok(value),
            None => Err(RuntimeError::EmptyStack),
        }
    }

    #[allow(dead_code)]
    fn top_stack(&self) -> RuntimeResult<&Value> {
        match self.stack.last() {
            Some(value) => Ok(value),
            None => Err(RuntimeError::EmptyStack),
        }
    }

    #[allow(dead_code)]
    fn print_call_stack(&self) {
        debug!("**********Call stack**********");
        for frame in &self.frames {
            debug!("{:?}", frame);
        }
    }

    #[allow(dead_code)]
    fn print_stack(&self) {
        debug!("**********STACK LEN: {}**********", self.stack.len());
        for value in self.stack.iter() {
            debug!("{}", value)
        }
        debug!("**********STACK END**********");
    }
}

impl Default for Vm {
    fn default() -> Self {
        Vm {
            frames: Vec::new(),
            stack: Vec::new(),
            current_chunk: None,
            globals: PREDEFINED_CONSTANTS
                .iter()
                .map(|(s, f)| (Value::Embedded(s), f.clone()))
                .collect(),
        }
    }
}
