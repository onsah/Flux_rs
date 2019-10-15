// example program that implements map and chain functions
import std.list as List;

fn map(iterator, f)
    fn()
        let value = iterator();
        if value != nil then
            f(value)
        else
            nil
        end
    end
end

fn chain(it1, it2)
    fn()
        let val = it1();
        if val != nil then val else it2() end
    end
end

let list1 = {1, 2, 3};
let list2 = {4, 5, 6};

for i in map(
            chain(
                List.iter(list1), 
                List.iter(list2)), 
            fn(x) x * x end)
do
    println(i);
end
// prints square of numbers from 1 to 6