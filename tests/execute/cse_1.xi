use io
use conv

main(args:int[][]) {
    a: int = 1
    b: int = 2
    c: int = a + b
    d: int = a + b + c
    e: int = a + b + c
    f: int = b + c
    g: int = (a + b + c) + (a + b) + (b + c)
    println("14 = " + unparseInt(g))
}
