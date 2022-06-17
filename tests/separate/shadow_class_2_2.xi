use io
use conv
use shadow_class_2_1

final class Foo {
    field': bool
    field: bool
    method(): bool {
        return field' & field
    }
}

main(args: int[][]) {
    print("5 = ")
    println(unparseInt(foo()))

    print("false = ")
    foo: Foo = new Foo
    foo.field' = true
    foo.field = false
    if foo.method() {
        println("true")
    } else {
        println("false")
    }
}
