use super::{Table, Value};
use crate::compiler::FuncProto;
use crate::vm::{RuntimeResult, Vm};
use std::cell::RefCell;
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
    // What about holding a rc?
    proto: FuncProtoRef,
    this: Option<Rc<RefCell<Table>>>,
}

#[derive(Clone, Debug, Hash, PartialEq)]
pub enum UpValue {
    Open { index: u16 },
    Closed(Value),
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

    pub fn new(proto: FuncProtoRef) -> Self {
        UserFunction {
            args_len: proto.args_len,
            proto,
            this: None,
        }
    }

    pub fn with_this(mut self, table: Rc<RefCell<Table>>) -> Self {
        self.this = Some(table);
        self
    }

    pub fn args_len(&self) -> u8 {
        if self.is_method() {
            self.args_len - 1
        } else {
            self.args_len
        }
    }

    pub fn upvalues(&self) -> &[(usize, Rc<RefCell<UpValue>>)] {
        self.proto.upvalues.as_slice()
    }

    pub fn extract_upvalues(self) -> Vec<(usize, Rc<RefCell<UpValue>>)> {
        self.proto.upvalues.clone()
    }

    pub fn proto(&self) -> FuncProtoRef {
        self.proto.clone()
    }

    pub fn is_method(&self) -> bool {
        self.this.is_some()
    }

    pub fn close_upvalue(&self, index: usize, value: Value) {
        let upvalue: &mut UpValue = &mut self.upvalues()[index].1.borrow_mut();
        *upvalue = UpValue::Closed(value);
    }

    pub fn take_this(&mut self) -> Option<Rc<RefCell<Table>>> {
        self.this.take()
    }
}

impl UpValue {
    pub fn is_closed(&self) -> bool {
        use UpValue::*;
        match self {
            Open { .. } => false,
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
