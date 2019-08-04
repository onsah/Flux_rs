let bind = fn(f, arg) 
    return fn()
        return f(arg)
    end
end;

let square = fn(x)
    return x * x
end;

let square5 = bind(square, 5);
println(square5());  // 25