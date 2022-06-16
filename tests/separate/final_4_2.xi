use io
use conv
use final_4_1

main(args: int[][]) {

    a: A = new_a()
    b: B = new_b()
    c: A = b

    a.set(1)
    b.set(2)
    b.set'(3)
    c.set(4)

    print("1 = ")
    println(unparseInt(a.get()))

    print("4 = ")
    println(unparseInt(b.get()))

    print("3 = ")
    println(unparseInt(b.get'()))

    print("4 = ")
    println(unparseInt(c.get()))

    print("A = ")
    println(a.overridden())

    print("B = ")
    println(b.overridden())

    print("B = ")
    println(c.overridden())
}
