class Foo {
    field: int
    method(): int {
        return field
    }
}

foo(): int {
    foo: Foo = new Foo
    foo.field = 5
    return foo.method()
}
