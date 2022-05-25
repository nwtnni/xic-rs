use io
use conv

class A {
    x: int
    foo() {
        x = x + 1
    }
}

main(args: int[][]) {
    a: A = new A
    a.x = 0
    a.foo()
    println(unparseInt(a.x))
    a.foo()
    println(unparseInt(a.x))
    a.foo()
    println(unparseInt(a.x))
}
