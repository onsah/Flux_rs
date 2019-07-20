let class = {
    "init" = fn(self)
        self.foo = -5
    end
}
let obj = new(class)
return obj.foo