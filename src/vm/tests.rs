use super::RuntimeError;
use crate::error::FluxError;
use crate::parser::{ParserError, ParserErrorKind};
use crate::vm::Value;

unit_test! {
    wrong_number_of_args,
    "
    let dummy = fn(a, b, c) end;
    dummy();
    ",
    Err(FluxError::Runtime(Box::new(RuntimeError::WrongNumberOfArgs {
        expected: 3,
        found: 0
    })))
}

unit_test! {
    arithmetic,
    "
    let x = 5 * 2 + 5 - 5;
    return x;
    ",
    Ok(Value::Int(10))
}

unit_test! {
    simple_fn_call,
    "
    let foo = fn(x) return x * x; end;
    return foo(5);
    ",
    Ok(Value::Int(25))
}

unit_test! {
    integer_to_float,
    "
    let i = 5; 
    return 5 / 2;
    ",
    Ok(Value::Number(2.5))
}

unit_test! {
    recursion,
    "
    let fib = fn(n) 
        return if n <= 1 then 
            n
        else 
            fib(n - 1) + fib(n - 2)
        end
    end;
    fib(6)
    ",
    Ok(Value::Int(8))
}

unit_test! {
    closure,
    "
    let foo = fn(x) 
        return fn(y)
            return fn()
                return x + y; 
            end;
        end;
    end;
    let bar = foo(10);
    let barr = bar(5);
    return barr();
    ",
    Ok(Value::Int(15))
}

unit_test! {
    generator,
    "
    let gen = fn()
        let i = 0;
        fn()
            i = i + 1;
            i
        end
    end;
    let iter = gen();
    iter();
    iter();
    iter()
    ",
    Ok(Value::Int(3))
}

unit_test! {
    method,
    "
    let obj = {
        \"setX\" = fn(self, x)
            self.x = x;
        end,
        \"getX\" = fn(self)
            return self.x;
        end,
    };
    obj:setX(17);
    obj:getX()
    ",
    Ok(Value::Int(17))
}

// This stack overflows due to self referencing structs print infinitely
/* unit_test! {
    method_lazy,
    "
    let obj = {
        \"setX\" = fn(self, x)
            self.x = x;
        end,
        \"getX\" = fn(self)
            return self.x;
        end,
        \"setXLater\" = fn(self, x)
            return fn()
                self.x = x;
                self
            end;
        end
    };
    obj:setX(10);
    let setLater = obj:setXLater(5);
    let oldX = obj:getX();
    assert(setLater().x == 5);
    return (oldX, obj:getX());
    ",
    Ok(Value::Tuple(vec![
        Value::Int(10),
        Value::Int(5)
    ]))
} */

unit_test! {
    assert,
    "
    assert(false);
    ",
    Err(FluxError::Runtime(Box::new(RuntimeError::AssertionFailed(Value::Bool(false)))))
}

unit_test! {
    new,
    "
    let class = {
        \"init\" = fn(self)
            self.foo = -5;
        end
    };
    let obj = new(class);
    return obj.foo;
    ",
    Ok(Value::Int(-5))
}

unit_test! {
    new_with_args,
    "
    let class = {
        \"init\" = fn(self, x)
            self.foo = x;
        end
    };
    let obj = new(class, 3);
    return obj.foo;
    ",
    Ok(Value::Int(3))
}

unit_test! {
    new_without_args,
    "
    let obj = new();
    ",
    Err(FluxError::Runtime(Box::new(RuntimeError::ExpectedArgsAtLeast(1))))
}

unit_test! {
    scoping,
    "
    let foo = 1;
    do
        let foo = 2;
        assert(foo == 2);
        do 
            let foo = 3;
            assert(foo == 3);
        end
    end
    assert(foo == 1);
    ",
    Ok(Value::Unit)
}

unit_test! {
    deep_nested_upvalues,
    "
    let foo = fn(x)
        return fn(y)
            return fn(z)
                return fn(t)
                    return x + y + z + t;
                end;
            end;
        end;
    end;
    return foo(1)(2)(3)(4);
    ",
    Ok(Value::Int(10))
}

unit_test! {
    block,
    "
    return do
        let x = 5 * 5;
        x
    end;
    ",
    Ok(Value::Int(25))
}

unit_test! {
    block_return,
    "
    let x = do
        return 5;
    end;
    ",
    Ok(Value::Int(5))
}

unit_test! {
    if_expr,
    "
    let x = if true then   
        10
    else
        5
    end;
    return x;
    ",
    Ok(Value::Int(10))
}

unit_test! {
    if_expr_comp,
    "
    let x = if false then
        5
    else if false then
        10
    else
        15
    end;
    return x;
    ",
    Ok(Value::Int(15))
}

unit_test! {
    func_expr_works,
    "
    let sqrt = fn(n) n * n end;
    return sqrt(5)
    ",
    Ok(Value::Int(25))
}

unit_test! {
    fn_stmt_to_fn_expr,
    "
    let lazySqrt = fn(x)
        fn()
            x * x
        end
    end;
    return lazySqrt(5)()
    ",
    Ok(Value::Int(25))
}

unit_test! {
    block_closure,
    "
    (fn() 
        let foo = do
            let i = 0;
            fn()
                i
            end
        end;
        foo()
    end)()
    ",
    Ok(Value::Int(0))
}

unit_test! {
    set_upvalue,
    "
    let foo = (fn()
        let i = 0;
        let bar = fn()
            i = i + 1;
            i
        end;
        bar();
        bar
    end)();
    foo();
    foo()
    ",
    Ok(Value::Int(3))
}

unit_test! {
    for_loop,
    "
    let range = fn(n)
        let i = 0; 
        fn()
            if i < n then
                i = i + 1;
                i
            else
                nil
            end
        end
    end;
    let i = 1;
    for j in range(10) do
        i = i * 2;
    end
    i
    ",
    Ok(Value::Int(1024))
}

unit_test! {
    global_variable,
    "foo = 5;",
    Err(FluxError::Parse(ParserError {
        kind: ParserErrorKind::Undeclared { name: "foo".to_owned() },
        line: 1,
    }))
}

unit_test! {
    remainder,
    "
    assert(5 % 2 == 1);
    assert(5 % 1.5 == 0.5);
    assert(5.5 % 1 == 0.5);
    ",
    Ok(Value::Unit)
}

#[test]
fn divide_by_zero() {
    use crate::util::eval;

    assert_eq!(eval("5 / 0", ""), Err(RuntimeError::DivideByZero.into()));
    assert_eq!(eval("5 / 0.0", ""), Err(RuntimeError::DivideByZero.into()));
    assert_eq!(eval("5.0 / 0", ""), Err(RuntimeError::DivideByZero.into()));
    assert_eq!(
        eval("5.0 / 0.0", ""),
        Err(RuntimeError::DivideByZero.into())
    );
}

macro_rules! test_file {
    ($test_name: ident, $name:expr, $expected:expr) => {
        #[test]
        fn $test_name() {
            use crate::util::run_file;
            use std::fs::canonicalize;
            use std::path::PathBuf;

            let path = {
                let mut pathbuf = canonicalize(PathBuf::from(file!())).unwrap();
                pathbuf.pop();
                pathbuf.push("tests");
                pathbuf.push($name);
                pathbuf.set_extension("flux");
                pathbuf
            };
            let value = run_file(path.to_str().unwrap());

            assert_eq!(value, $expected);
        }
    };
}

test_file! {
    import,
    "import",
    Ok(Value::Int(25))
}

test_file! {
    import_closure,
    "import_closure",
    Ok(Value::Int(10))
}
