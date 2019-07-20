use super::value::{ArgsLen, Function, NativeFunction, Table};
use super::{Value, Integer};
use crate::vm::{RuntimeError, Vm};
use std::io::{self, Write};
use std::rc::Rc;

pub const PREDEFINED_CONSTANTS: [(&str, Value); 7] = [
    ("print", PRINT),
    ("println", PRINTLN),
    ("readline", READLINE),
    ("int", INT),
    ("number", NUMBER),
    ("assert", ASSERT),
    ("new", NEW),
];

#[inline]
pub fn constant_names() -> impl Iterator<Item=Value> {
    PREDEFINED_CONSTANTS.iter().map(|&(n, _)| n.into())
}

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
    |_vm, args| {
        let mut args_iter = args.into_iter().rev();
        if let Some(arg) = args_iter.next() {
            print!("{}", arg);
            for arg in args_iter {
                print!(" {}", arg);
            }
        }
        match io::stdout().flush() {
            Ok(_) => Ok(Value::Unit),
            _ => Err(RuntimeError::IOError),
        }
    },
    ArgsLen::Variadic
}

define_native! {
    PRINTLN,
    |_vm, args| {
        for arg in args.into_iter().rev() {
            print!("{} ", arg);
        }
        println!("");
        Ok(Value::Unit)
    },
    ArgsLen::Variadic
}

define_native! {
    READLINE,
    |_, _| {
        let mut string = String::new();
        match io::stdin().read_line(&mut string) {
            Ok(_) => {
                string.pop().unwrap();
                Ok(Value::Str(Rc::new(string)))
            },
            Err(_) => Err(RuntimeError::IOError),
        }
    },
    ArgsLen::Exact(0)
}

define_native! {
    INT,
    |_vm, args| {
        let value = &args[0];
        match value {
            Value::Bool(b) => Ok(Value::Int(match b {
                true => 1,
                false => 0,
            })),
            Value::Nil => Ok(Value::Int(0)),
            Value::Str(string) => {
                match string.parse::<Integer>() {
                    Ok(i) => Ok(Value::Int(i)),
                    Err(_) => Err(RuntimeError::InvalidFormat),
                }
            },
            Value::Int(i) => Ok(Value::Int(*i)),
            Value::Number(i) => Ok(Value::Int(i.round() as Integer)),
            _ => Err(RuntimeError::TypeError),
        }
    },
    ArgsLen::Exact(1)
}

define_native! {
    NUMBER,
    |_vm, args| {
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
    ArgsLen::Exact(1)
}

define_native! {
    ASSERT,
    |_vm, args| {
        let value = &args[0];
        if value.as_bool() {
            Ok(Value::Unit)
        } else {
            Err(RuntimeError::AssertionFailed(value.clone()))
        }
    },
    ArgsLen::Exact(1)
}

define_native! {
    NEW,
    |vm, args| {
        // TODO: too much cloning. Should be able with less
        let table = Table::new().shared();
        let klass = &args[0];
        {
            let mut table = table.borrow_mut();
            table.set(Value::Embedded("__class__"), klass.clone());
        }
        if let Ok(init) = Vm::get_table(&Value::Embedded("init"), &klass)?.to_user_fn() {
            let pushed_args = args.len() as u8;
            for arg in args.into_iter().skip(1) {
                vm.stack.push(arg)
            }
            // Maybe don't pop this somehow?
            vm.stack.push(Rc::clone(&table).into());
            vm.call_user_blocking(init, pushed_args)?;
            vm.pop_stack()?;
        }
        Ok(table.into())
    },
    ArgsLen::Variadic
}
