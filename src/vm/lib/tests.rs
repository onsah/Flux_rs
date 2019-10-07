use crate::vm::Value;

unit_test! {
    arity,
    "
    fn foo(a, b, c) end
    arity(foo)
    ",
    Ok(Value::Int(3))
}
