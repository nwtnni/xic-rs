use io
use conv

foo: int = 1

class A {
    foo(foo: int) {
        print("2 = ")
        println(unparseInt(foo))
    }
}

main(args: int[][]) {
    a: A = new A
    print("1 = ")
    println(unparseInt(foo))
    a.foo(2)
}
