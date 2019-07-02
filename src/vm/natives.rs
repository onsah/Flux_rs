use std::io::{self, Write};
use std::rc::Rc;
use super::Value;
use super::value::{Function, NativeFunction};
use crate::vm::RuntimeError;

pub const PREDEFINED_CONSTANTS: [(&'static str, Value); 5] = [
    ("print", PRINT),
    ("println", PRINTLN),
    ("readline", READLINE),
    ("int", INT),
    ("number", NUMBER)
];

macro_rules! define_native {
    ($name:ident, $function:expr, $len:expr) => {
        pub const $name: Value = Value::Function(Function::Native(NativeFunction {
            function: $function,
            args_len: $len,
        }));
    };
}

define_native! {
    PRINT,
    |args| {
        print!("{}", args[0]);
        for arg in &args[1..] {
            print!(" {}", arg);
        }
        match io::stdout().flush() {
            Ok(_) => Ok(Value::Unit),
            _ => Err(RuntimeError::IOError),
        }
    },
    1
}

define_native! {
    PRINTLN,
    |args| {
        for arg in args {
            print!("{} ", arg);
        }
        println!("");
        Ok(Value::Unit)
    },
    1
}

define_native! {
    READLINE,
    |_| {
        let mut string = String::new();
        match io::stdin().read_line(&mut string) {
            Ok(_) => {
                string.pop().unwrap();
                Ok(Value::Str(Rc::new(string)))
            },
            Err(_) => Err(RuntimeError::IOError),
        }
    },
    0
}

define_native! {
    INT,
    |args| {
        let value = &args[0];
        match value {
            Value::Bool(b) => Ok(Value::Int(match b {
                true => 1,
                false => 0,
            })),
            Value::Nil => Ok(Value::Int(0)),
            Value::Str(string) => {
                match string.parse::<i32>() {
                    Ok(i) => Ok(Value::Int(i)),
                    Err(_) => Err(RuntimeError::InvalidFormat),
                }
            },
            Value::Int(i) => Ok(Value::Int(*i)),
            Value::Number(i) => Ok(Value::Int(i.round() as i32)),
            _ => Err(RuntimeError::TypeError),
        }
    },
    1
}

define_native! {
    NUMBER,
    |args| {
        let value = &args[0];
        match value {
            Value::Bool(b) => Ok(Value::Int(match b {
                true => 1,
                false => 0,
            })),
            Value::Nil => Ok(Value::Int(0)),
            Value::Str(string) => {
                match string.parse::<f64>() {
                    Ok(i) => Ok(Value::Number(i)),
                    Err(_) => Err(RuntimeError::InvalidFormat),
                }
            },
            Value::Int(i) => Ok(Value::Int(*i)),
            Value::Number(i) => Ok(Value::Number(*i)),
            _ => Err(RuntimeError::TypeError),
        }
    },
    1
}