use io
use conv

main(args:int[][]) {
    x:int = 2
    y:int = 3
    print(unparseInt(x) + " + " + unparseInt(y) + " = " + unparseInt(x+y))
}