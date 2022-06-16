use io
use conv

class A {
    field: int
}

final class B extends A {
    field: int
}

main(args: int[][]) {
    b: B = new B
    b.field = 5
    print("5 = ")
    println(unparseInt(b.field))

    a: A = b
    a.field = 6
    print("6 = ")
    println(unparseInt(a.field))
}
