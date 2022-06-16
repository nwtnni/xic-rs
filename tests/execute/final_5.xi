use io
use conv

class A {
    field: int
}

final class B extends A {}

main(args: int[][]) {
    a: A = new B
    a.field = 5
    print("5 = ")
    print(unparseInt(a.field))
}
