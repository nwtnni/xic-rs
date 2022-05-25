use io
use conv

class A {
    a: A
    x: int
}

main(args: int[][]) {
    a: A = new A
    a.a = a
    a.a.a.a.a.a.a.a.a.a.x = 1
    println(unparseInt(a.x))
    a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.x = 2
    println(unparseInt(a.a.a.a.a.a.a.a.a.a.x))
    a.x = 3
    println(unparseInt(a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.x))
}
