use super::{RuntimeError, Value, Vm};
use crate::compiler::Compiler;
use crate::parser::Parser;

unit_test! {
    wrong_number_of_args,
    "
    let dummy = fn(a, b, c) end;
    dummy();
    ",
    Err(RuntimeError::WrongNumberOfArgs {
        expected: 3,
        found: 0
    })
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
    return fib(6);
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
    method,
    "
    let obj = {
        \"setX\" = fn(x, self)
            self.x = x;
        end,
        \"getX\" = fn(self)
            return self.x;
        end,
        \"setXLater\" = fn(x, self) 
            return fn()
                self.x = x;
            end;
        end
    };
    obj:setX(10);
    let setLater = obj:setXLater(5);
    let oldX = obj:getX();
    setLater();
    return (oldX, obj:getX());
    ",
    Ok(Value::Tuple(vec![
        Value::Int(10),
        Value::Int(5)
    ]))
}

unit_test! {
    assert,
    "
    assert(false);
    ",
    Err(RuntimeError::AssertionFailed(Value::Bool(false)))
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
        \"init\" = fn(x, self)
            self.foo = x;
        end
    };
    let obj = new(3, class);
    return obj.foo;
    ",
    Ok(Value::Int(3))
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
