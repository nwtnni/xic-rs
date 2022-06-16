final class A {
    a: int
    b: int

    set_a(a: int) {
        this.a = a
    }

    set_b(b: int) {
        this.b = b
    }

    get_sum(): int {
        return a + b
    }
}

new_a(): A {
    return new A
}
