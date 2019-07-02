## FLUX_RS
A toy scripting language for learning purposes
Design goals are:
* Minimal syntax
* Null safety (normal type vs nullable type)
* Functional programming features

## Status
Currently has tables as objects, statements and most expressions.

## Problems
* Argument numbers are not checked

## Roadmap
* Implement statements (done)
* Implement functions (as first class citizen) (done)
* Implement closures
* Implement simple OOP (member functions for tables)
* Simple patterns matching for tuple expressions
* Nullable variables and static checking for nullable types
* Optimization for tables used as arrays (Like lua)

## Example programs
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