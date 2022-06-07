use io
use conv

x: int = 5
y: int, z: int = foo(x)

main(args: int[][]) {
    print("6 = ")
    println(unparseInt(y))
    print("7 = ")
    println(unparseInt(z))
}

foo(a: int): int, int {
    return a + 1, a + 2
}
