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

let iter = range(10);
let value = iter();
while value != nil then
    println(value);
    value = iter();
end
