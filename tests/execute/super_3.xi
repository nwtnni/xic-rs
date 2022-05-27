use io

class B extends A {
    foo() {
        println("1: Foo")
        super.foo()
    }
}

class A {
    foo() {
        println("2: Bar")
    }
}

main(args: int[][]) {
    b: B = new B
    b.foo()
}
