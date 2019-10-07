var iter = fn(arr)
    let i = 0;
    fn()
        let val = arr[i];
        i = i + 1;
        val
    end
end;

var range = fn(n)
    let i = 0;
    fn()
        if i < n then
            let val = i;
            i = i + 1;
            val
        else 
            nil
        end
    end
end;