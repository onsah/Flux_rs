let foo = (fn()
    let i = 0;
    fn()
        i = i + 1;
        i
    end
end)();

foo();
foo();
foo()