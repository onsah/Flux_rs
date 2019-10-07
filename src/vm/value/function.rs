use super::{TableRef, Value};
use crate::compiler::FuncProto;
use crate::vm::{RuntimeResult, Vm};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Clone, Debug, Hash, PartialEq)]
pub enum Function {
    User(UserFunction),
    Native(NativeFunction),
}

pub type FuncProtoRef = Rc<FuncProto>;

#[derive(Clone, Debug)]
pub struct UserFunction {
    args_len: u8,
    proto: FuncProtoRef,
    env: Option<TableRef>,
}

type NativeFn = fn(&mut Vm, Vec<Value>) -> RuntimeResult<Value>;

#[derive(Clone)]
pub struct NativeFunction {
    pub function: NativeFn,
    pub args_len: ArgsLen,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub enum ArgsLen {
    Variadic,
    Exact(u8),
}

impl Function {
    pub fn new_user(proto: FuncProtoRef) -> Self {
        Function::User(UserFunction::new(proto))
    }

    pub fn new_user_with_env(proto: FuncProtoRef, env: TableRef) -> Self {
        Function::User(UserFunction::new(proto).with_env(env))
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
    pub fn new(proto: FuncProtoRef) -> Self {
        UserFunction {
            args_len: proto.args_len,
            proto,
            env: None,
        }
    }

    pub fn with_env(mut self, env: TableRef) -> Self {
        self.env = Some(env);
        self
    }

    pub fn args_len(&self) -> u8 {
        self.args_len
    }

    pub fn proto_ref(&self) -> &FuncProtoRef {
        &self.proto
    }

    pub fn take_env(&mut self) -> Option<TableRef> {
        self.env.take()
    }

    pub fn env(&self) -> Option<&TableRef> {
        self.env.as_ref()
    }
}

impl NativeFunction {
    pub fn args_len(&self) -> ArgsLen {
        self.args_len
    }
}

impl PartialEq for UserFunction {
    fn eq(&self, rhs: &Self) -> bool {
        self.proto == rhs.proto
    }
}

impl Hash for UserFunction {
    // Code start should be unique
    fn hash<H: Hasher>(&self, state: &mut H) {
        let adress = self.proto.as_ref() as *const FuncProto;
        adress.hash(state)
    }
}

impl Into<Value> for UserFunction {
    fn into(self) -> Value {
        Value::Function(Function::User(self))
    }
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "native fn({})", self.args_len())
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &NativeFunction) -> bool {
        (self.function as *const NativeFn) == (other.function as *const NativeFn)
    }
}

impl Hash for NativeFunction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.function as *const NativeFn).hash(state);
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
