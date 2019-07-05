let foo = fn(n)
    if n <= 0 then
        return 1
    else
        return 1 + foo(n - 1)
    end
end
print(foo(5))