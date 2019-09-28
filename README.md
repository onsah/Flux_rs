## FLUX_RS
A toy scripting language for learning purposes
Design goals are:
* Minimal syntax
* Null safety (normal type vs nullable type)
* Functional programming features (higher order functions)
* Simple OOP support

## What is working?
* closures
* recursion
* tables
* modules (simple)
* OOP
* local scoping
* block expressions
* modules

## What is **not** working?
* nil checking at compile time
* pattern matching for multiple return values

## Roadmap
* Nil checking at compile time
* Pattern matching for multiple return values
* Simple pattern matching for tuple expressions
* Nullable variables and static checking for nullable types
* Optimization for tables used as arrays (Like lua)

## How to run
Download the source code from repository. You need cargo to be installed. Then execute the command in root directory of the project.
`cargo run [file_path]`

## Features
### If expressions
Flux is designed to be expressive where possible, and expressions are preferred over statements. Look this java snippet
```java
int i;
if (someCondition) {
    // Some work
    i = someValue;
} else {
    // Work...
    i = someOtherValue;
}
```
There is no direct way to initialize a value with branching in Java. So we have to create value with unitialized state then assign to its value in both branches. This way is error prone because programmer may forget to assign in one of the branches and programmer has to trace the code to be sure that value is initialized from all possible paths. Now look at the equivalent flux code
```
let i = 
    if someCondition then
        // work....
        someValue
    else 
        // work...
        someOtherValue
    end;
```
There is no double assigning in this case and it is much clearer that we are initializing a value.

Note: Flux also doesn't warn when value is initialized because when a block doesn't have expression it just returns `Unit`. But in the future this problem will be solved by static nullity check.

## OOP
While OOP is not main focus of Flux, it is partialy supported with tables. Its OOP systems is works similarly with Javascript's prototypes. `init` function is called `new` native function is called. Even though `new` is a native function it can be implemented as a regular function.
```
let Class = {
    "init" = fn(x, self)
        self.x = x;
    end,
    "getX" = fn(self) self.x end,
};
let o = new(Class, 5);
o:getX() // 5
```

## Example programs
### Iterators using generators
```
let gen = fn(n)
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
let iter = gen();
println(iter());     // 1
println(iter());     // 2
println(iter());     // 3
```
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
square5()  // 25
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
        cache[n]
    end;
    __fib(n)
end;
print("enter a number: ");
let i = number(readline());
fib(i)
```

more examples are at examples folder