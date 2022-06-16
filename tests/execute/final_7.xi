use io
use conv

final class A {
    a: int
    b: int
}

main(args: int[][]) {
    a: A = new A
    a.a = 1
    a.b = 2
    print("1 + 2 = ")
    println(unparseInt(a.a + a.b))
}
