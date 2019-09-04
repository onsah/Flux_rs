## Closures
In flux, all functions are closures.
### Instructions
When a closure scope is entered, its instructions are compiled separately from the main chunk then wrapped with a reference counting pointer. From that a function prototype is created. Whenever a function is instansiated the value holds a reference to its prototype.
### Upvalues
When a closure references a value from its outer scope. It holds a upvalue. An example 
```
let foo = fn()
    let i = 0;
    fn()
        i   // Inner lambda holds upvalue to foo
    end
end;
let bar = foo();    // upvalue that points to i is closed
bar()      // 0
```
When foo is called and returned its value are not on stack anymore, if bar tries to access the `i` variable there is no way to retrieve it from stack. Instead when an outer reference is detected in compile time it opens an upvalue. The upvalue is either open or closed. When it is open, it holds the stack index from its owner closure. If it is closed it holds the value. While owner function on the call stack, upvalue remains open, but when it returns it closes all upvalues that relies on its own stack. This upvalue is wrapped with a shared pointer, therefore any other closure that points to that upvalue don't need to be informed.