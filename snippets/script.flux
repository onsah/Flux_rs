let range = fn(n)
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

let iter = fn(table)
    let i = 0;
    fn()
        let value = table[i];
        i = i + 1;
        value
    end
end;

for i in iter({"hello", "this", "is", "an", "iterator"}) do
    println(i);
end