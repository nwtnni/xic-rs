use io
use conv

class B extends A {
    x: int
    foo() {
        x = 1
        print("1 = ")
        println(unparseInt(x))
    }
}

class A {
    x: int
    bar() {
        x = 2
        print("2 = ")
        println(unparseInt(x))
    }
}

main(args: int[][]) {
    b: B = new B
    b.foo()
    b.bar()
}
