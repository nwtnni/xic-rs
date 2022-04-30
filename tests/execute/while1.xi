use io
use conv

main(args:int[][]) {
    x:int = 0
    while(x<10) {
        println("x = " + unparseInt(x))
        x = x + 1
    }
}