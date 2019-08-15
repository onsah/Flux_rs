let Class = {
    "init" = fn(x, self)
        self.x = x;
    end,
    "getX" = fn(self) self.x end,
};
let o = new(5, Class);
o:getX() // 5