use std::fmt::{Display, Formatter};
use super::Value;
use crate::compiler::UpValueDesc;
use crate::vm::RuntimeResult;
use std::hash::Hash;

#[derive(Clone, Debug, Hash, PartialEq)]
pub enum Function {
    User(UserFunction),
    Native(NativeFunction)
}

#[derive(Clone, Debug, Hash)]
pub struct UserFunction {
    args_len: u8,
    code_start: usize,
    upvalues: Vec<UpValue>,
}

#[derive(Clone, Debug, Hash)]
pub enum UpValue {
    Open {
        index: u16,
    },
    Closed(Value),
    // Upvalue in the stack of itself
    This {
        index: u16
    },
}

#[derive(Clone, Debug, Hash, PartialEq)]
pub struct NativeFunction {
    pub function: fn(Vec<Value>) -> RuntimeResult<Value>,
    pub args_len: ArgsLen,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub enum ArgsLen {
    Variadic,
    Exact(u8),
}

impl Function {
    pub fn new_user(args_len: u8, code_start: usize, upvalues: &[UpValueDesc]) -> Self {
        Function::User(UserFunction::new(args_len, code_start, upvalues))
    }

    pub fn args_len(&self) -> ArgsLen {
        match self {
            Function::User(func) => ArgsLen::Exact(func.args_len()),
            Function::Native(native) => native.args_len(),
        }
    }

    pub fn is_native(&self) -> bool {
        match self {
            Function::User(_) => false,
            Function::Native(_) => true,
        }
    }
}

impl UserFunction {
    pub const MAX_UPVALUES: u8 = std::u8::MAX;

    pub fn new(args_len: u8, code_start: usize, upvalues: &[UpValueDesc]) -> Self {
        UserFunction {
            args_len,
            code_start,
            // TODO: Impl Into<UpValue> for UpValueDesc
            upvalues: upvalues.iter().map(|ud| if ud.is_this {
                UpValue::This {
                    index: ud.index,
                }
            } else {
                UpValue::Open {
                    index: ud.index,
                }
            }).collect(),
        }
    }

    pub fn args_len(&self) -> u8 {
        self.args_len
    }

    pub fn code_start(&self) -> usize {
        self.code_start
    }

    pub fn upvalues(&self) -> &[UpValue] {
        self.upvalues.as_slice()
    }

    pub fn upvalues_mut(&mut self) -> &mut [UpValue] {
        self.upvalues.as_mut_slice()
    }

    pub fn extract_upvalues(self) -> Vec<UpValue> {
        self.upvalues
    }

    pub fn push_upvalue(&mut self, index: u16) -> Option<u8> {
        if (self.upvalues.len() as u8) < Self::MAX_UPVALUES {
            self.upvalues.push(UpValue::Open { index });
            Some((self.upvalues.len() - 1) as u8)
        } else {
            None
        }
    }

    pub fn close_upvalue(&mut self, index: usize, value: Value) {
        self.upvalues[index] = UpValue::Closed(value);
    }
}

impl UpValue {
    pub fn is_closed(&self) -> bool {
        use UpValue::*;
        match self {
            Open { .. } | This { .. } => false,
            Closed(_) => true,
        }
    }
}

impl NativeFunction {
    pub fn args_len(&self) -> ArgsLen {
        self.args_len
    }
}

impl PartialEq for UserFunction {
    fn eq(&self, rhs: &Self) -> bool {
        self.code_start == rhs.code_start
    }
}

impl Display for ArgsLen {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ArgsLen::Variadic => write!(f, "variadic"),
            ArgsLen::Exact(n) => write!(f, "{} args", n),
        }
    }
}