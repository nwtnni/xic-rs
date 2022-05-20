use io
use conv

main(args: int[][]) {
    i: int = 0
    j: int = 0
    do {
        i = i + 1
        j = i * 2
    } while i < 10
    println("20 = " + unparseInt(j))
}
