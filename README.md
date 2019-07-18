## FLUX_RS
A toy scripting language for learning purposes
Design goals are:
* Minimal syntax
* Null safety (normal type vs nullable type)
* Functional programming features (passing functions, returning functions)

## Status
* functions work as any other value
* closures
* tables 
* most statements

## Roadmap
* Implement statements (done)
* Implement functions (as first class citizen) (done)
* Implement closures (done!)
* Implement simple OOP (like lua metatables) (done)
* Simple pattern matching for tuple expressions
* Nullable variables and static checking for nullable types
* Optimization for tables used as arrays (Like lua)

## Example programs
### Argument binding via closures
```
let bind = fn(f, arg) 
    return fn()
        return f(arg)
    end
end
let square = fn(x)
    return x * x
end
let square5 = bind(square, 5)
println(square5())
```
### Cached fibonacci program
```
let cache = { 0, 1 }
let fib = fn(n)
    if n <= 0 then
        return 0
    else if cache[n] == nil then
        cache[n] = fib(n - 1) + fib(n - 2)
    end
    return cache[n]
end
print("enter a number: ")
let i = number(readline());
println(fib(i))
```