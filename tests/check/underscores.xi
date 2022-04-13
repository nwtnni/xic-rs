foo () {
    _, x:int = foo2()
    y:int, _ = foo2()
    a:int, b:int = foo2()
    _, _ = foo2()
}

foo2():int, int {
    return 2,3
}