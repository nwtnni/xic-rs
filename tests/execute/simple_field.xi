use io
use conv

class A {
    x: int
}

main(args: int[][]) {
    a: A = new A
    a.x = 1
    println(unparseInt(a.x))
}
