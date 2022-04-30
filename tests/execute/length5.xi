use io
use conv

main(args:int[][]) {
    x:int[5][][]
    printInt(length(x))
}

printInt(x:int) {
    print(unparseInt(x))
}