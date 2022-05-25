use io
use conv

class A {
    x: int
}

main(args: int[][]) {
    x: int = 1
    a: A = new A
    x = 2
    a.x = 3
    println(unparseInt(x))
    println(unparseInt(a.x))
}
