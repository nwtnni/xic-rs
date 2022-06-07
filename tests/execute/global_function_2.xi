use io
use conv

x: int, y: int = foo()

main(args: int[][]) {
    print("1 = ")
    println(unparseInt(x))
    print("2 = ")
    println(unparseInt(y))
}

foo(): int, int {
    return 1, 2
}
