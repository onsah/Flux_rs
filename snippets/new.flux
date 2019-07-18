let class = {
    "init" = fn(self)
        self.bar = "this set during init"
    end,
    "foo" = fn(self)
        println(self.bar)
    end
}
let obj = new(class)
obj:foo()