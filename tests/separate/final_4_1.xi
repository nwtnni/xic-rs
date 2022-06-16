class A {
    field: int

    set(field: int) {
        this.field = field
    }

    overridden(): int[] {
        return "A"
    }

    get(): int {
        return field
    }
}

final class B extends A {
    field: int

    overridden(): int[] {
        return "B"
    }

    set'(field: int) {
        this.field = field
    }

    get'(): int {
        return field
    }
}

new_a(): A {
    return new A
}

new_b(): B {
    return new B
}
