use io
use conv

main(args: int[][]) {
    a: int = 5
    b: int = 10
    c: int = 0
    d: int = 0
    e: int = 0
    do {
        e = e + 1
        d = a + b * 2
        c = c + d
    } while c < 1000
    println("1000 = " + unparseInt(c))
    println("40 = " + unparseInt(e))
}
