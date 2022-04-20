use io
use conv

main (args:int[][]) {
    x:int = foo()
    y:int = bar()
    z:int = x * y * 0
    println(unparseInt(x))
    println(unparseInt(y))
    println(unparseInt(z))
}

foo() : int {
    return 5
}

bar() : int {
    return 6
}
