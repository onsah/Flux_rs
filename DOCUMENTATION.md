# Syntax
This section is a tutorial on syntax of flux language. Since it is in early development the documentation may become invalid.

Some rules:
```
*: optional
[something]: syntax structure 
{something}: 1 or more times repeated
```
## Statements
Flux statements are similar to Lua's.
### If
`if [condition] then [body] *{else [block]} end`

If statements executes the first block if condition evaluates true otherwise it evaluates else block.
```
if x < 5 then
    print("x is smaller then 5");
else
    print("x is bigger then 5");
end
```
If statement starts with a condition expression seperated from body by `then` keyword. Normally an if statement ends with `end` keyword but if there is else `end` is omitted. Moreover following is invalid syntax but can be valid in the future.
```
if x < 5 then
    print("x is smaller then 5");
end else    // This line is invalid
    print("x is bigger then 5");
end 
```
### While
`while [condition] then [block] end`

While statements are simple loops where block is executed every time condition evaluates to true

```
while x > 0 then
    x = x - 1
    println(x)
end
```

### Let
`let [name] = [expression];`

Let statements creates a variable in the current lexical scoped block. Semicolon is mandatory.

```
let foo = "bar";
do
    let foo = "hello";
    println(foo); // hello
end
println(foo) // bar
```

### Block
`do {statement} end`

Block statements create a new lexical scope

### Expression statement
`[expression];`

Expression statements are expression followed by a semicolon. Semicolon is important to distinguish an expression whose value will be returned or an expression just to be used for side effect

## Expressions

### Literals

Literals can be a string, number, bool or `nil`. In the future there may be `()` which indicates a `Unit` type. This type will be explained further.

```
let string_literal = "foo";
let number_literal = 5;
let number_literal = 324.21;
let bool_literal = true;
```

### Unary

Unary expressions are an unary operator followed by a literal

```
let not = !false;
let negate = -some_number_variable;
```

### Binary

Binary expressions are two expressions with a binary operator. 
```
let sum = foo + bar;
let mul = 5 * 3 + 2; // 17
let complex_binary = 6 + 2 * 7 / 2; // 13
```

### Assignment

Assignment expressions assign an expression to a value. Return value of assignment is always a `Unit`. This expression will be a statement in the future.

```
let g = 5;
g = "hello";
println(g); // hello
println(g = "world"); // ()
```

### TODO