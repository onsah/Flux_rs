use crate::vm::{RuntimeError, RuntimeResult};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

pub use function::{Function, UserFunction, NativeFunction, ArgsLen, UpValue};
pub use table::Table;

mod function;
mod table;

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i32),
    Number(f64),
    Str(Rc<String>),
    Embedded(&'static str),
    Table(Rc<RefCell<Table>>),
    Tuple(Vec<Value>),
    Function(Function),
    Unit,
}

impl Value {
    pub fn new_str(string: impl Into<String>) -> Self {
        Value::Str(Rc::new(string.into()))
    }

    pub fn as_str(&self) -> RuntimeResult<&str> {
        match self {
            Value::Str(rc) => Ok(rc.as_ref()),
            _ => Err(RuntimeError::TypeError),
        }
    }

    pub fn convert_int(&self) -> Option<i32> {
        match self {
            Value::Int(i) => Some(*i),
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    Some(n.round() as i32)
                } else {
                    None
                }
            } 
            _ => None,
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Nil => false,
            _ => true,
        }
    }

    pub fn is_user_fn(&self) -> bool {
        match self {
            Value::Function(function) => {
                match function {
                    Function::User(_) => true,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn to_user_fn(self) -> RuntimeResult<UserFunction> {
        match self {
            Value::Function(function) => {
                match function {
                    Function::User(f) => Ok(f),
                    _ => Err(RuntimeError::TypeError),
                }
            }
            _ => Err(RuntimeError::TypeError),
        }
    }
}

impl From<String> for Value {
    fn from(string: String) -> Self {
        Value::Str(Rc::new(string))
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        use Value::*;
        match (self, other) {
            (Nil, Nil) => true,
            (Bool(a), Bool(b)) => a == b,
            (Int(a), Int(b)) => a == b,
            (Number(a), Number(b)) => a == b,
            // TODO: number and int equality
            (Str(a), Str(b)) => a == b,
            (Embedded(a), Embedded(b)) => a == b,
            (Str(a), Embedded(b)) => a.as_str() == *b,
            (Embedded(a), Str(b)) => *a == b.as_str(),
            (Table(a), Table(b)) => a == b,
            (Tuple(a), Tuple(b)) => a == b,
            (Function(a), Function(b)) => a == b,
            (Unit, Unit) => true,
            _ => false,
        }
    }
}

// hash(5) == hash(5.0)
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Nil => 1.hash(state),
            Value::Bool(b) => {
                2.hash(state);
                b.hash(state);
            }
            Value::Int(i) => {
                3.hash(state);
                i.hash(state);
            }
            Value::Number(d) => {
                4.hash(state);
                (*d as u64).hash(state);
            }
            Value::Str(s) => {
                5.hash(state);
                (*s.as_str()).hash(state);
            }
            Value::Embedded(string) => {
                5.hash(state);
                (*string).hash(state)
            }
            Value::Table(t) => {
                6.hash(state);
                let adress = t.as_ptr();
                adress.hash(state);
            }
            Value::Tuple(values) => {
                7.hash(state);
                for value in values {
                    value.hash(state)
                }
            }
            Value::Function(function) => {
                8.hash(state);
                function.hash(state);
            }
            Value::Unit => {
                9.hash(state);
            }
        }
    }
}

impl Eq for Value {}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Value::Nil => write!(f, "Nil"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::Number(n) => write!(f, "{}", n),
            Value::Str(s) => {
                let s: &String = s.borrow();
                write!(f, "{}", s)
            }
            Value::Table(t) => {
                let table = t.as_ref().borrow();
                writeln!(f, "{{")?;
                for (k, v) in table.pairs() {
                    writeln!(f, "\t{}: {}", k, v)?;
                }
                writeln!(f, "}}")?;
                Ok(())
            }
            Value::Tuple(values) => {
                write!(f, "(")?;
                write!(f, "{}", values[0])?;
                for value in values.iter().skip(1) {
                    write!(f, ", {}", value)?;
                }
                write!(f, ")")?;
                Ok(())
            }
            Value::Function(function) => {
                let is_native = if function.is_native() {
                    "native "
                } else {
                    ""
                };
                write!(f, "{}fn({} args)", is_native, function.args_len())
            },
            Value::Unit => write!(f, "()"),
            Value::Embedded(string) => write!(f, "{}", string),
        }
    }
}


impl From<Rc<RefCell<Table>>> for Value {
    fn from(table: Rc<RefCell<Table>>) -> Self {
        Value::Table(table)
    }
}

#[cfg(test)]
mod tests {
    // TODO
    fn hash_works() {

    }
}