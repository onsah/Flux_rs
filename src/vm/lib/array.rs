pub(super) const ARRAY: &'static str = 
"
    iter = fn(arr)
        let i = 0;
        fn()
            i = i + 1;
            arr[i - 1]
        end
    end;
";