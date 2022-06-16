class A {
    field: int

    set(field: int) {
        this.field = field
    }

    get(): int {
        return field
    }
}

new_a(): A {
    return new A
}
