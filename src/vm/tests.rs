use super::{Vm, RuntimeError, Value};
use crate::compiler::Compiler;
use crate::parser::Parser;

#[test]
fn wrong_number_of_args_works() {
    let source = "let dummy = fn(a, b, c) end \
    dummy()";
    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse().unwrap();
    let chunk = Compiler::compile(ast).unwrap();
    let mut vm = Vm::new();
    
    assert_eq!(vm.run(chunk), Err(RuntimeError::WrongNumberOfArgs {
        expected: 3,
        found: 0
    }));
}

#[test]
fn simple_fn_call_works() {
    let source = "let foo = fn(x) return x * x end 
    return foo(5)";
    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse().unwrap();
    let chunk = Compiler::compile(ast).unwrap();
    let mut vm = Vm::new();

    assert_eq!(vm.run(chunk), Ok(Value::Int(25)));
}

#[test]
fn integer_to_float_works() {
    let source = "let i = 5; return 5 / 2";
    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse().unwrap();
    let chunk = Compiler::compile(ast).unwrap();
    let mut vm = Vm::new();

    assert_eq!(vm.run(chunk), Ok(Value::Number(2.5)));
}

#[test]
fn recursion_works() {
    let source = "let fib = fn(n) 
        if n <= 1 then return n
        else return fib(n - 1) + fib(n - 2) end
    end;
    return fib(6)";
    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse().unwrap();
    let chunk = Compiler::compile(ast).unwrap();
    let mut vm = Vm::new();

    assert_eq!(vm.run(chunk), Ok(Value::Int(8)));
}

#[test]
fn closure_works() {
    let source = "let foo = fn(x) 
        return fn(y)
            return fn()
                return x + y 
            end
        end 
    end
    let bar = foo(10)
    let barr = bar(5)
    return barr()";
    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse().unwrap();
    let chunk = Compiler::compile(ast).unwrap();
    let mut vm = Vm::new();

    assert_eq!(vm.run(chunk), Ok(Value::Int(15)));
}