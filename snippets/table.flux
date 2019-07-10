let vector = {
    "push" = fn(item, self)
        self[self.top] = item
        self.top = self.top + 1
    end,
    "pop" = fn(self)
        self.top = self.top - 1
        return self[self.top]
    end,
    "top" = 0
}
vector:push(5)
let t = { 10 }
t.pop = vector:pop
println(t.pop())