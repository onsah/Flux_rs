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
print(fib(i))