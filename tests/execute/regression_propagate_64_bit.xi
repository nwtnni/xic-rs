use io
use conv

class A {
    field: int
}

main(args: int[][]) {
    a: A = new A
    a.field = -9223372036854775808
    println(unparseInt(a.field))
}
