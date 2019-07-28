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

#[derive(Clone, Debug)]
pub struct UserFunction {
    args_len: u8,
    proto_index: usize,
    upvalues: Vec<UpValue>,
    this: Option<Rc<RefCell<Table>>>,
}

#[derive(Clone, Debug, Hash, PartialEq)]
pub enum UpValue {
    Open { index: u16 },
    Closed(Value),
    // Upvalue in the stack of itself
    This { index: u16 },
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
    pub fn new_user(proto: &FuncProto, proto_index: usize) -> Self {
        Function::User(UserFunction::new(proto, proto_index))
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

    pub fn new(proto: &FuncProto, proto_index: usize) -> Self {
        UserFunction {
            args_len: proto.args_len,
            proto_index,
            // TODO: Impl Into<UpValue> for UpValueDesc
            upvalues: proto
                .upvalues
                .iter()
                .map(|ud| {
                    if ud.is_this {
                        UpValue::This { index: ud.index }
                    } else {
                        UpValue::Open { index: ud.index }
                    }
                })
                .collect(),
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

    pub fn upvalues(&self) -> &[UpValue] {
        self.upvalues.as_slice()
    }

    pub fn upvalues_mut(&mut self) -> &mut [UpValue] {
        self.upvalues.as_mut_slice()
    }

    pub fn extract_upvalues(self) -> Vec<UpValue> {
        self.upvalues
    }

    pub fn proto_index(&self) -> usize {
        self.proto_index
    }

    pub fn is_method(&self) -> bool {
        self.this.is_some()
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

    pub fn take_this(&mut self) -> Option<Rc<RefCell<Table>>> {
        self.this.take()
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
        self.proto_index == rhs.proto_index
    }
}

impl Hash for UserFunction {
    // Code start should be unique
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.proto_index.hash(state)
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
