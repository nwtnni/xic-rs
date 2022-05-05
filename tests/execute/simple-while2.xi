use io
use conv

main(args:int[][]) {
    x:int = 0
    while x<3 {
        println(unparseInt(x))
        x = x+1
    }
}