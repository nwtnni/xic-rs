use io
use conv

class A {
    x: int
    foo(x: int) {
        x = x + 2
        this.x = this.x + 1
    }
}

main(args: int[][]) {
    a: A = new A
    a.x = 0
    x: int = 10

    a.foo(x)
    println(unparseInt(a.x))
    a.foo(x)
    println(unparseInt(a.x))
    a.foo(x)
    println(unparseInt(a.x))
}
