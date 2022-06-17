use io
use conv
use shadow_class_3_1

final class Foo extends Super {
    field': bool
    field: bool

    method(): int {
        if method''() {
            return 1
        } else {
            return -1
        }
    }

    method''(): bool {
        return field' & method'''()
    }

    method'''(): bool {
        return field
    }
}

main(args: int[][]) {

    a: Super = new_super()
    b: Super = new_foo()
    c: Foo = new Foo
    d: Super = c

    a.set(10)
    print("10 = ")
    println(unparseInt(a.method()))
    print("11 = ")
    println(unparseInt(a.method'()))

    b.set(10)
    print("6 = ")
    println(unparseInt(b.method()))
    print("11 = ")
    println(unparseInt(b.method'()))

    c.field = true
    c.field' = false
    d.set(10)
    print("-1 = ")
    println(unparseInt(d.method()))
    print("11 = ")
    println(unparseInt(d.method'()))

    print("false = ")
    if c.method''() {
        println("true")
    } else {
        println("false")
    }

    print("true = ")
    if c.method'''() {
        println("true")
    } else {
        println("false")
    }
}
