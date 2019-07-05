let dummy = 5
let foo = fn(x) 
    return fn(y)
        return fn()
            return x + y
        end
    end
end
let bar = foo(4)
let barr = bar(5)
println(barr())
println(barr())