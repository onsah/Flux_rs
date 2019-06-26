let i = 0
let a = 0
let b = 1
while i < 30 then
    let c = a
    a = b
    b = b + c
    i = i + 1
end
print (a, b)