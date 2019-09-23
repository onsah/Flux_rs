var LinkedList = {
    "init" = fn(self)
        self.head = nil;
    end,
    "push" = fn(self, node)
        node.next = self.head;
        self.head = node;
    end,
    "iter" = fn(self)
        fn()
            let node = self.head;
            if node then
                self.head = node.next;
                node.value
            else
                nil
            end
        end
    end
};

let ll = new(LinkedList);
ll:push({"value" = 1});
ll:push({"value" = 2});
ll:push({"value" = 3});
ll:push({"value" = 4});
ll:push({"value" = 5});

for i in ll:iter() do
    println(i);
end
// 5
// 4
// 3
// 2
// 1