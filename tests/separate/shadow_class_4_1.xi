class Super {
    field: int

    set(field: int) {
        this.field = field
    }

    method(): int {
        return field
    }

    method'(): int {
        return field + 1
    }
}

class Foo extends Super {
    field: int

    method(): int {
        return method''()
    }

    method''(): int {
        return field + 1
    }

    method'''() {}
}

new_super(): Super {
    return new Super
}

new_foo(): Super {
    foo: Foo = new Foo
    foo.field = 5
    return foo
}
