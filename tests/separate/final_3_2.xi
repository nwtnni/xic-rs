use io
use conv
use final_3_1

main(args: int[][]) {
    a: A = new_a()
    a.set_a(5)
    a.set_b(10)

    print("5 + 10 = ")
    println(unparseInt(a.get_sum()))
}
