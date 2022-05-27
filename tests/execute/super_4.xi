use io
use conv

class D extends C {
    foo() {
        x = 0
        print("0 = ")
        println(unparseInt(x))
        super.foo()
    }
}

class C extends B {
    foo() {
        x = x + 1
        print("1 = ")
        println(unparseInt(x))
        super.foo()
    }
}

class B extends A {
    foo() {
        x = x + 2
        print("3 = ")
        println(unparseInt(x))
        super.foo()
    }
}

class A {
    x: int
    foo() {
        x = x + 3
        print("6 = ")
        println(unparseInt(x))
    }
}

main(args: int[][]) {
    d: B = new D
    d.foo()
}
