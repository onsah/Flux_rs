let foo = fn()
    let i = 0;
    fn()
        i = i + 1;
        i
    end
end;

foo();

/// Transpiles to

let foo = fn()
    let i = 0; 
    
    bind_env({  // bind_env is native function
        "i" = i,
    }, fn(env)
        env.i = env.i + 1;
        env.i
    end)
end;

foo();  // checks if closure has env then push it as last argument

//

fn(x)
    bind_env({
        "x" = x,
    }, 
    fn(y, env)
        bind_env({
            "x" = env.x,
            "y" = y
        }, fn(env)
            env.x + env.y
        end)
    end
end