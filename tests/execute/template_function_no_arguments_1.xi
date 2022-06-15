use io
use conv

main(args: int[][]) {
    a: int = sum::<>(1, 2)
    print("1 + 2 = ")
    println(unparseInt(a))
}

template sum(a: int, b: int): int {
    return a + b
}

