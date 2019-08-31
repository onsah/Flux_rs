(fn()
    let foo = 
    do
        let i = 0;
        fn()
            i
        end
    end;
    foo()
end)()