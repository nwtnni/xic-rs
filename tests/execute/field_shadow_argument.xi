use io
use conv

class A {
    x: int
}

main(args: int[][]) {
    shadow(new A, 1)
}

shadow(a: A, x: int) {
    a.x = 2
    x = 3
    println(unparseInt(a.x))
    println(unparseInt(x))
}
