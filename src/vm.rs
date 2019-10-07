mod error;
mod frame;
pub mod lib;
#[cfg(test)]
mod tests;
mod value;

use crate::compiler::{BinaryInstr, Chunk, CompiledSource, Instruction, UnaryInstr};
pub use error::RuntimeError;
use frame::Frame;
pub use lib::PREDEFINED_CONSTANTS;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
pub use value::{
    ArgsLen, Float, FuncProtoRef, Function, Integer, NativeFunction, Table, UserFunction, Value,
};

pub type RuntimeResult<T> = Result<T, RuntimeError>;

#[derive(Debug, Clone, PartialEq)]
pub struct Vm {
    frames: Vec<Frame>,
    stack: Vec<Value>,
    globals: HashMap<Value, Value>,
    compiled: Option<CompiledSource>,
}

impl Vm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self, source: CompiledSource) -> RuntimeResult<Value> {
        /* self.set_chunk(chunk);
        self.set_constants(constants); */
        self.set_compiled_source(source);
        self.init_call();
        self.main_loop()
    }

    fn main_loop(&mut self) -> RuntimeResult<Value> {
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
                    let value = self.constant_table()[index as usize].clone();
                    self.stack.push(value)
                }
                Instruction::Pop => {
                    self.pop_stack()?;
                }
                Instruction::Return { return_value } => {
                    let value = if return_value {
                        self.pop_stack()?
                    } else {
                        Value::Unit
                    };
                    while self.stack.len() > self.current_frame()?.stack_top() {
                        self.pop_stack()?;
                    }
                    self.stack.push(value);
                    self.frames.pop().expect("Stack frame is empty");
                    // self.print_stack();
                    return Ok(());
                }
                Instruction::Bin(bin) => self.binary(bin)?,
                Instruction::Unary(unary) => self.unary(unary)?,
                Instruction::GetGlobal { index } => {
                    let name = &self.constant_table()[index as usize];
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
                    let name = self.constant_table()[index as usize].clone();
                    let value = self.stack.pop().unwrap();
                    self.globals.insert(name, value);
                }
                Instruction::GetLocal { index, frame } => {
                    let frame_index = self.frame_from_offset(frame);
                    let index = self.frames[frame_index].stack_top() + index as usize;
                    self.stack.push(self.stack[index as usize].clone());
                }
                Instruction::SetLocal { index, frame } => {
                    let frame_index = self.frame_from_offset(frame);
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
                Instruction::GetMethodImm {
                    index,
                    table_stack_index,
                } => self.get_method_imm(index, table_stack_index)?,
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
                Instruction::FuncDef {
                    proto_index,
                    has_env,
                } => {
                    let proto = self.prototypes()[proto_index as usize].clone();
                    let function = Value::Function(if has_env {
                        let env = self
                            .pop_stack()?
                            .into_table()
                            .expect("Expected a table as env");
                        Function::new_user_with_env(proto, env)
                    } else {
                        Function::new_user(proto)
                    });
                    self.stack.push(function)
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
                Instruction::Integer(value) => self.stack.push(value.into()),
                Instruction::Import { name_index } => self.import(name_index as usize)?,
                Instruction::ExitBlock { pop, return_value } => {
                    let return_value = if return_value {
                        let value = self.pop_stack()?;
                        Some(value)
                    } else {
                        None
                    };
                    for _ in 0..pop {
                        self.pop_stack()?;
                    }
                    if let Some(value) = return_value {
                        self.stack.push(value);
                    }
                }
                Instruction::Rec => {
                    let frame = self.frames.last().expect("Expected a call frame");
                    let func = frame
                        .function()
                        .expect("Expected call has a function")
                        .clone();
                    self.stack.push(func.into());
                }
                _ => return Err(RuntimeError::UnsupportedInstruction(instr)),
            }
            let f = self.current_frame_mut()?;
            f.pc += 1;
            self.print_call_stack();
            self.print_stack();
            // self.print_globals();
        }
    }

    fn import(&mut self, name_index: usize) -> RuntimeResult<()> {
        let mod_name = self.constant_table()[name_index].as_str()?.to_string();
        let chunk = self
            .current_chunk_mut()
            .imports()
            .remove(&mod_name)
            .expect("Expected module");
        let mut vm = Vm::new();
        let source = CompiledSource {
            chunk,
            constant_table: Rc::clone(
                &self
                    .compiled
                    .as_ref()
                    .expect("Expected a compiled source")
                    .constant_table,
            ),
        };
        // TODO: wrap error
        // vm.run(chunk, Rc::clone(self.constant_table.as_ref().expect("Expected a constant table")))?;
        vm.run(source)?;
        self.globals
            .insert(mod_name.into(), Table::from_map(vm.globals).into());
        Ok(())
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
        let key = &self.constant_table()[index as usize];
        let value = Self::get_table(key, &table)?;
        self.stack.push(value);
        Ok(())
    }

    fn get_method_imm(&mut self, index: u8, table_stack_index: u8) -> RuntimeResult<()> {
        // let field = self.get_field_imm(index)?;
        let table_stack_index = self.stack.len() - table_stack_index as usize - 1;
        let table = self.stack[table_stack_index].clone();
        let key = &self.constant_table()[index as usize];
        let value = Self::get_table(&key, &table)?;
        self.stack.push(value.into_user_fn()?.into());
        Ok(())
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
        let key = &self.constant_table()[index as usize];
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

    fn call_user(&mut self, function: UserFunction, pushed_args: u8) -> RuntimeResult<()> {
        if pushed_args == function.args_len() {
            let stack_top = self.stack.len() - function.args_len() as usize;

            // Push env if exists
            if let Some(env) = function.env() {
                self.stack.push(Rc::clone(env).into())
            }

            // let upvalues = function.extract_upvalues();
            self.frames.push(Frame::new(0, function, stack_top));
            self.print_call_stack();
            self.print_stack();
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
                    BinaryInstr::Div => {
                        if b == 0.0 {
                            return Err(RuntimeError::DivideByZero);
                        } else {
                            Value::Number(a / b)
                        }
                    }
                    BinaryInstr::Rem => Value::Number(a % b),

                    BinaryInstr::Gt => Value::Bool(a > b),
                    BinaryInstr::Lt => Value::Bool(a < b),
                    BinaryInstr::Ge => Value::Bool(a >= b),
                    BinaryInstr::Le => Value::Bool(a <= b),
                    _ => unreachable!(),
                }),
                (Value::Number(a), Value::Int(b)) => Ok(match op {
                    BinaryInstr::Add => Value::Number(a + (b as f64)),
                    BinaryInstr::Sub => Value::Number(a - (b as f64)),
                    BinaryInstr::Mul => Value::Number(a * (b as f64)),
                    BinaryInstr::Div => {
                        if b == 0 {
                            return Err(RuntimeError::DivideByZero);
                        } else {
                            Value::Number(a / (b as f64))
                        }
                    }
                    BinaryInstr::Rem => Value::Number(a % (b as f64)),

                    BinaryInstr::Gt => Value::Bool(a > (b as f64)),
                    BinaryInstr::Lt => Value::Bool(a < (b as f64)),
                    BinaryInstr::Ge => Value::Bool(a >= (b as f64)),
                    BinaryInstr::Le => Value::Bool(a <= (b as f64)),
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
                            BinaryInstr::Rem => Value::Int(a % b),
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
                (Value::Int(a), Value::Number(b)) => Ok({
                    match op {
                        BinaryInstr::Add => Value::Number((a as f64) + b),
                        BinaryInstr::Sub => Value::Number((a as f64) - b),
                        BinaryInstr::Mul => Value::Number((a as f64) * b),
                        BinaryInstr::Div => {
                            if b == 0.0 {
                                return Err(RuntimeError::DivideByZero);
                            } else {
                                Value::Number((a as f64) / b)
                            }
                        }
                        BinaryInstr::Rem => Value::Number((a as f64) % b),
                        BinaryInstr::Gt => Value::Bool((a as f64) > b),
                        BinaryInstr::Lt => Value::Bool((a as f64) < b),
                        BinaryInstr::Ge => Value::Bool((a as f64) >= b),
                        BinaryInstr::Le => Value::Bool((a as f64) <= b),
                        _ => unreachable!(),
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
        &self
            .compiled
            .as_ref()
            .expect("Expected a compiled source")
            .chunk
    }

    #[inline]
    fn current_chunk_mut(&mut self) -> &mut Chunk {
        &mut self.compiled.as_mut().expect("Expected a chunk").chunk
    }

    fn constant_table(&self) -> &[Value] {
        &self
            .compiled
            .as_ref()
            .expect("Expected a constant table")
            .constant_table
            .constants
    }

    fn prototypes(&self) -> &[FuncProtoRef] {
        &self
            .compiled
            .as_ref()
            .expect("Expected a constant table")
            .constant_table
            .prototypes
    }

    fn instructions(&self) -> RuntimeResult<&[Instruction]> {
        Ok(match self.current_frame()?.proto() {
            Some(proto) => proto.instructions.as_ref(),
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
            debug!("{:#?}", frame);
        }
    }

    #[allow(dead_code)]
    fn print_stack(&self) {
        debug!("**********STACK LEN: {}**********", self.stack.len());
        for value in self.stack.iter() {
            debug!("{}", &value);
        }
        debug!("**********STACK END**********");
    }

    #[allow(dead_code)]
    fn print_globals(&self) {
        debug!("{:#?}", self.globals);
    }

    fn set_compiled_source(&mut self, source: CompiledSource) {
        self.compiled = Some(source);
    }

    #[inline]
    fn frame_from_offset(&self, offset: u8) -> usize {
        if offset != 0 {
            self.frames.len() - offset as usize
        } else {
            0
        }
    }
}

impl Default for Vm {
    fn default() -> Self {
        Vm {
            frames: Vec::new(),
            stack: Vec::new(),
            compiled: None,
            // current_chunk: None,
            // constant_table: None,
            globals: PREDEFINED_CONSTANTS
                .iter()
                .map(|(s, f)| (Value::Embedded(s), f.clone()))
                .collect(),
        }
    }
}
