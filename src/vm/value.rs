use std::borrow::Borrow;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;
pub use table::Table;
pub use function::Function;
use crate::vm::{RuntimeResult, RuntimeError};

mod function;
mod table;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i32),
    Number(f64),
    Str(Rc<String>),
    Table(Rc<RefCell<Table>>),
    Tuple(Vec<Value>),
    Function(Function),
    Unit,
}

impl Value {
    pub fn as_str(&self) -> RuntimeResult<&str> {
        match self {
            Value::Str(rc) => Ok(rc.as_ref()),
            _ => Err(RuntimeError::TypeError)
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Nil => false,
            _ => true,
        }
    }
}

impl From<String> for Value {
    fn from(string: String) -> Self {
        Value::Str(Rc::new(string))
    }
}

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
                // Since literals are unique
                let adress = s.as_ptr();
                adress.hash(state);
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
            },
            Value::Table(t) => {
                let table = t.as_ref().borrow();
                for (k, v) in table.pairs() {
                    writeln!(f, "{}: {}", k, v)?;
                }
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
                write!(f, "fn({} args)", function.args_len())
            }
            Value::Unit => {
                write!(f, "()")
            }
        }
    }
}