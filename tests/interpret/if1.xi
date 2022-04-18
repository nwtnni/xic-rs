use io
use conv

main(args:int[][]) {
    x:int = 0
    if (x == 1) {
        print("If Branch: x = " + unparseInt(x))
    }
    else {
        print("Else Branch: x = " + unparseInt(x))
    }
}