# Syntax
This section is a tutorial on syntax of flux language. Since it is in early development the documentation may become invalid.

Some syntax rules:
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
If statement starts with a condition expression seperated from body with `then` keyword. Normally an if statement ends with `end` keyword but if there is else `end` is omitted. Currently following is invalid syntax but can be valid in the future.
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

### Var
`var [name] = [expression];`

Var statements declare a global variable. Local variable can shadow a global variable in their lifetime. One should prefer let statements over var whenever possible. Var can be preferred when you want to export variable to other modules.
```flux
var foo = "bar";
foo // "bar"
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

### Import and Export statements
`export { *( [identifier] {, [identifier] *,} ) }`

`import [identifier] *{.[identifier]} as [identifier];`

Export statements indicate which parts of the block will be exposed to the outside. A module can export only global variables. A module can have either one or zero export statement. Import statement imports the all exports from module to name of that module. Import statements create a global variable. Export statement have not implemented to the language yet. Instead all global variables from the module are imported.

File module.flux
```
var bar = fn() "bar" end;
```

other file 
```
import module as m;

m.bar() // "bar"
```

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

### Grouping

Grouping expression is an expression that is wrapped between parantheses.

```
let n = (5 + 3) * 2;
println(n) // 16
```

### If

If expressions are similar to if statements in other languages and can be used as them, but they also return value so they can used anywhere an expression is valid. If statements without else branch can't be used as expressions since they can't guarantee to return value
```
let foo = 
    if false then
        5
    else
        6
    end;
    println(foo) // 6
```

### Block
`do {statement} *[expression] end`

Block expressions create a new lexical scope. If there is no expression at the end `Unit` is returned

Note: In the future block may return `nil` when there is no expression. 
```
let i = 5;
do
    // Shadows the other i variable
    let i = 10;
    println(i) // 10
end
// Previous i is now visible again
println(i) // 5
```
Since they are expressions this is also valid. This allows hiding temporary local variables using lexical scoping.
```
let i = do
    let j = foo();
    // Complex stuff...
    j
end;
// j can not be reached from this scope
```
It produces cleaner code because block indicates that all code inside it is for initializing the variable.

### Function
A function expression returns a function :) Function definition a function with args followed by block expression. There is no conceptual difference between a function and a closure. Last expression of the block automatically returned from function.
```
let sqrt = fn(x) 
    x * x 
end;
sqrt(5) // 25
```

## Builtin functions and modules
These functions and modules provide some common functionality that most programmers need. Although Flux itself doesn't accept variable number of arguments, native functions can take variable number of arguments.

### `print` and `println`
```
native fn print(...args): ()
    //...
end
```

`print` takes any number of argument and prints them with spaces inserted between them. `println` does the same with `print` but appends new line at the end.
```
print(5); // '5'
print("foo", 3); // 'foo 3'
```

### `readline`
```
native fn read(): string
    //...
end
```

`readline` reads a single line and returns it as a string

### `int`
```
native fn int(str): number
    //...
end
```

`int` takes a string and tries to parse int from it. Panics if string can't be converted to an integer.

### `number`
```
native fn number(str): number
    //...
end
```

Same with `int` but parses a floating point number instead of integer.

### `assert`
```
native fn assert(condition): ()
    //...
end
```

`assert` evaluates the condition and panics if condition returns false. Otherwise it does nothing.

### `new`
```
native fn new(class, ...args): table
    //...
end
```

`new` creates a new table with the provided class and arguments. The semantics of `new` is equivalent to the flux code below if flux has supported variable arguments
```
fn new(class, ...args)
    let t = {};
    t["__class__"] = class;
    t:init(args...);
    t
end
```
### TODO