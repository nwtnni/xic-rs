foo() {
    n:int = 1
    m:int = 2
    a:int[n][m]
    b:int[n+m][(3 + n / 2 *>> 3 % (1 * m))]
    b = {{}}
    c:bool[bar()]
}

bar() : int {
    return 1
}