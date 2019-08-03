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