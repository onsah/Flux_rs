## Closures
In flux, all functions are closures. The function that references a variable outside of their scope holds an environment in which captures the values at the time of closure creation. The mechanism is simple. If a variable is not in a local scope until it reaches an enclosing function, there is a table created with name "env" for the closure. These values are stored in this "env" table and bound to the closure at the time of creation. For example
```
let b = fn foo()
    let i = 0;
    fn bar() i end
end;
b() // 0
```
The function bar referene to "i" but this variable is not on the stack at the time bar is called. Therefore bar should take a snapshot of the value at the time of it being returned from foo. Therefore the code desugared as the following.
```
let b = fn foo()
    let i = 0;
    with_env({ "i" = i }, fn bar(env) env.i end)
end;
b() // 0
```
Actually there is no such function called `with_env` but it illustrates how the desugaring actually works. When a function is closure, function is desugared to add an additional parameter and all outer variable accesses are desugared to table accesses of this parameter. The env table is bound to the closure so it is never visible to the user.

### Instructions
When a closure scope is entered, its instructions are compiled separately from the main chunk then wrapped with a reference counting pointer. From that a function prototype is created. Whenever a function is instansiated the value holds a reference to its prototype.
