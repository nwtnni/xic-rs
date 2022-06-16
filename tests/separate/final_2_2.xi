use io
use conv
use final_2_1

final class B extends A {
    field: int

    get'(): int {
        return field
    }

    set'(field: int) {
        this.field = field
    }
}

main(args: int[][]) {
    a: A = new_a()

    a.set(1)
    print("1 = ")
    println(unparseInt(a.get()))

    b: B = new B

    b.set'(2)
    b.set(3)

    print("2 = ")
    println(unparseInt(b.get'()))

    print("3 = ")
    println(unparseInt(b.get()))

    c: A = b
    c.set(4)

    print("4 = ")
    println(unparseInt(c.get()))
}
