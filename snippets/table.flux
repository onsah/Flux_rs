let foo = fn(x)
    let bar = fn()
        return x
    end
    let y = bar()
    return y
end
println(foo(5))