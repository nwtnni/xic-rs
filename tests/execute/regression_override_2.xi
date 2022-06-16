use io

class A {
    foo() {
        println("A")
    }
}

class B extends A {}

class C extends B {
    foo() {
        print("C, ")
        super.foo()
    }
}

main(args: int[][]) {
    a: A = new C
    print("C, A = ")
    a.foo()

    print("A = ")
    new A.foo()

    print("A = ")
    new B.foo()
}
