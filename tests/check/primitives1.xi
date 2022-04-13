foo() {
    x:int = 1
    y:bool = true
    z:int
    z = x
    y':bool
    y' = y
    return
}

bar(a:int, b:bool) {
    x:int = a
    y:bool = b
}

foo2(): int {
    return 1
}

bar2(): bool {
    return true
}

foobar(x:int): bool,bool {
    return x == 1, x ==2
}

foobar2(y:int): int, bool {
    return y, y == y
}

foobar3() {
    v1:bool, v2:bool = foobar(1)
    v3:int, v4:bool = foobar2(1)
}