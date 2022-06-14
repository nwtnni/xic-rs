use io

class A {
    foo() {
        this.bar()
    }

    bar() {
        println("Inside class A::bar")
    }
}

class B extends A {
    bar() {
        println("Inside class B::bar")
    }
}

main(args: int[][]) {
    a: A = new B
    a.foo()
}
