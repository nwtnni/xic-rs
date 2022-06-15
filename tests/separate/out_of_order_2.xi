use io

class A {
    foo() {
        println("Calling foo")
    }

    bar() {
        println("Calling bar")
    }

    baz() {
        println("Calling baz")
    }
}

new_a(): A {
    return new A
}
