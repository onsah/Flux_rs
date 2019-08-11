## FLUX_RS
A toy scripting language for learning purposes
Design goals are:
* Minimal syntax
* Null safety (normal type vs nullable type)
* Functional programming features (higher order functions)
* Simple OOP support

## What is working?
* recursion
* closures
* tables
* OOP
* local scoping
* block expressions

## What is **not** working?
* nil checking at compile time
* pattern matching for multiple return values

## Roadmap
* Simple pattern matching for tuple expressions
* Nullable variables and static checking for nullable types
* Optimization for tables used as arrays (Like lua)

## Example programs
### Argument binding via closures
```
let bind = fn(f, arg) 
    fn()
        f(arg)
    end
end;

let square = fn(x)
    x * x
end;

let square5 = bind(square, 5);
println(square5());  // 25
```
### Cached fibonacci program
```
let fib = fn(n) 
    // We use a closure so cache variable is not leaked to the global scope
    let cache = { 0, 1 };
    let __fib = fn(n)
        if n <= 0 then
            return 0
        else if cache[n] == nil then
            cache[n] = __fib(n - 1) + __fib(n - 2);
        end
        return cache[n]
    end;
    return __fib(n)
end;
print("enter a number: ");
let i = number(readline());
println(fib(i));
```