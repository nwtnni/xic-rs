class A {
    foo(a: int): int {
        return a
    }
}

class B extends A {
    foo(a: int): int {
        return a + 1
    }
}
