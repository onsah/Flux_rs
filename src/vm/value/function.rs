use super::Value;
use crate::vm::RuntimeResult;
use std::hash::Hash;

#[derive(Clone, Debug, Hash, PartialEq)]
pub enum Function {
    User(UserFunction),
    Native(NativeFunction)
}

impl Function {
    pub fn new_user(args_len: u8, code_start: usize) -> Self {
        Function::User(UserFunction::new(args_len, code_start))
    }

    pub fn args_len(&self) -> i16 {
        match self {
            Function::User(func) => func.args_len() as i16,
            Function::Native(native) => native.args_len() as i16,
        }
    }

    pub fn is_native(&self) -> bool {
        match self {
            Function::User(_) => false,
            Function::Native(_) => true,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq)]
pub struct NativeFunction {
    pub function: fn(Vec<Value>) -> RuntimeResult<Value>,
    pub args_len: u8,
}

impl NativeFunction {
    pub fn args_len(&self) -> u8 {
        self.args_len
    }
}

#[derive(Clone, Debug, Hash)]
pub struct UserFunction {
    args_len: u8,
    code_start: usize,
}

impl UserFunction {
    pub fn new(args_len: u8, code_start: usize) -> Self {
        UserFunction {
            args_len,
            code_start,
        }
    }

    pub fn args_len(&self) -> u8 {
        self.args_len
    }

    pub fn code_start(&self) -> usize {
        self.code_start
    }
}

impl PartialEq for UserFunction {
    fn eq(&self, rhs: &Self) -> bool {
        self.code_start == rhs.code_start
    }
}