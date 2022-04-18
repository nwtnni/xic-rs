use io
use conv

main(args:int[][]) {
    x:int = 2
    println("Before: " + unparseInt(x))
    x = 5
    println("After: " + unparseInt(x))
}