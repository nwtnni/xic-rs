foo() {
    a:int[]
    b:int[2][3]
    c:int[a[0]]
    d:bool[bar()]["test"[bar()]]
}

bar() : int {
    return 1
}