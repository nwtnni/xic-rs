use io
use conv

x, y, z: int

foo(z: int, y: int, x: int) {
    print("1 = ")
    println(unparseInt(x))
    print("2 = ")
    println(unparseInt(y))
    print("3 = ")
    println(unparseInt(z))
}

main(args: int[][]) {
    x = 1
    y = 2
    z = 3
    foo(z, y, x)
}
