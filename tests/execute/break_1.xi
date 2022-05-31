use io
use conv

main(args: int[][]) {
    i: int = 0
    while i < 10 {
        println(unparseInt(i))
        if i > 5 {
            break
        }
        i = i + 1
    }
}
