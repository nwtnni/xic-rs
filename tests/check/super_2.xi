class C extends B {
    foo(): int {
        return super.bar()
    }
}

class B extends A {
}

class A {
    bar(): int {
        return 0
    }
}
