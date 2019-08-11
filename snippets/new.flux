let setX = fn(self, x) 
    self.x = x;
end;

let setX = fn(x, self) 
    self.x = x;
end;

let setX = fn(x)
    self.x = x;
end;

let setX = self fn(x)
    self.x = x;
end;