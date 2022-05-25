use io
use conv

class A {
    x: int
}

main(args: int[][]) {
    a: A = new A
    b: A = a
    c: A = new A
    c = a

    c.x = 1
    println(unparseInt(a.x))
    b.x = 2
    println(unparseInt(c.x))
    c.x = 3
    println(unparseInt(b.x))
}
