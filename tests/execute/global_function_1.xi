use io
use conv

x: int = foo()

main(args: int[][]) {
    print("5 = ")
    println(unparseInt(x))
}

foo(): int {
    return 5
}


